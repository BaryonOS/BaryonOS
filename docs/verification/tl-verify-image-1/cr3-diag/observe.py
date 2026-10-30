#!/usr/bin/env python3
"""One diagnostic boot, read-only GDB observations before the CR3 write."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import socket
import struct
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
RUN = ROOT / "build/ks-cr3-diag-run"
NET = Path("/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/"
           "simonvanieperen-musical-happiness/build/tls-port-worktree")
ORIGINAL = NET / "build/tcp-main94-regression/run/system-normal-a06babbb1f89497ab60b59c514d0ac3c"
QEMU = Path("/tmp/nieuw-os-qemu-102/root/usr/bin/qemu-system-x86_64")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Remote:
    def __init__(self, path):
        self.socket = socket.socket(socket.AF_UNIX)
        self.socket.settimeout(45)
        self.socket.connect(os.path.relpath(path))

    def packet(self, command):
        data = command.encode()
        self.socket.sendall(b"$" + data + b"#" + f"{sum(data) & 255:02x}".encode())
        while self.socket.recv(1) != b"$":
            pass
        data = bytearray()
        while True:
            char = self.socket.recv(1)
            if not char:
                raise EOFError("Debugger disconnected")
            if char == b"#":
                break
            data.extend(char)
        checksum = self.socket.recv(2)
        if int(checksum, 16) != sum(data) & 255:
            raise ValueError("Debugger checksum mismatch")
        self.socket.sendall(b"+")
        return data.decode()

    def read(self, address, length):
        result = self.packet(f"m{address:x},{length:x}")
        if result.startswith("E"):
            raise ValueError(f"Debugger read failure: {result}")
        return bytes.fromhex(result)

    def walk(self, root, address):
        table = root & 0x000FFFFFFFFFF000
        result = []
        for shift in (39, 30, 21, 12):
            slot = table + ((address >> shift) & 511) * 8
            entry = struct.unpack("<Q", self.read(slot, 8))[0]
            result.append({"shift": shift, "slot": slot, "entry": entry})
            if entry & 1 == 0 or entry & 128:
                break
            table = entry & 0x000FFFFFFFFFF000
        return result


def main():
    if RUN.exists():
        raise ValueError("Diagnostic directory already exists; preserve the first run")
    RUN.mkdir(parents=True)
    original = json.loads(subprocess.check_output(
        ["git", "show", "4a0ce99:services/net-tls/verification/diagnostic-result.json"],
        cwd=ROOT, text=True))
    for relative, expected in original["esp_sha256"].items():
        if sha(ORIGINAL / relative) != expected:
            raise ValueError(f"Original artifact drift: {relative}")
    shutil.copytree(ORIGINAL / "esp", RUN / "esp")
    shutil.copyfile(ORIGINAL / "OVMF_CODE.fd", RUN / "OVMF_CODE.fd")
    template = ROOT / "build/tl-verify-image-1/reference/run/materiaal-normal-a69441f209/OVMF_VARS.template.fd"
    if sha(template) != original["firmware_sha256"]["vars_template"]:
        raise ValueError("Firmware template mismatch")
    shutil.copyfile(template, RUN / "OVMF_VARS.fd")
    shutil.copyfile(ORIGINAL / "volume.after.img", RUN / "volume.img")
    if sha(RUN / "volume.img") != original["before_sha256"]:
        raise ValueError("Diagnostic initial disk mismatch")
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen()
    listener.settimeout(45)
    port = listener.getsockname()[1]
    peer = []

    def accept():
        connection, _ = listener.accept()
        peer.append(connection)

    thread = threading.Thread(target=accept, daemon=True)
    thread.start()
    command = [str(QEMU), "-L", str(QEMU.parent.parent / "share/qemu")]
    args = original["command"][1:]
    index = 0
    while index < len(args):
        key, value = args[index:index + 2] if index + 1 < len(args) else (args[index], None)
        if key in ("-no-reboot", "-nodefaults"):
            command.append(key)
            index += 1
            continue
        if key == "-serial":
            value = "file:serial.txt"
        elif key == "-drive":
            prefix, filename = value.rsplit("file=", 1)
            if filename.startswith("fat:"):
                filename = "fat:esp"
            else:
                filename = Path(filename).name
            value = prefix + "file=" + filename
        elif key == "-netdev":
            value = f"socket,id=net0,connect=127.0.0.1:{port}"
        command.extend((key, value))
        index += 2
    command += ["-S", "-gdb", "unix:gdb.sock,server=on,wait=off",
                "-d", "int,cpu_reset,guest_errors", "-D", str(RUN / "exceptions.log")]
    started = time.monotonic()
    observation = {"scope": "one stopped-before-CR3 diagnostic; not TCP acceptance",
                   "original_source": original["source_commit"],
                   "original_source_sha256": original["source_sha256"],
                   "command": command, "cwd": str(RUN), "qemu_sha256": sha(QEMU),
                   "kernel_sha256": sha(RUN / "esp/kernel.elf"),
                   "loader_sha256": sha(RUN / "esp/EFI/BOOT/BOOTX64.EFI")}
    process = None
    remote = None
    environment = os.environ.copy()
    environment["LD_LIBRARY_PATH"] = str(QEMU.parent.parent / "lib/x86_64-linux-gnu")
    observation["LD_LIBRARY_PATH"] = environment["LD_LIBRARY_PATH"]
    try:
        with (RUN / "stderr.txt").open("wb") as error:
            process = subprocess.Popen(command, cwd=RUN, stdout=error, stderr=error, env=environment)
            for _ in range(500):
                if (RUN / "gdb.sock").exists():
                    break
                if process.poll() is not None:
                    raise ValueError("QEMU exited before debugger connection")
                time.sleep(0.01)
            remote = Remote(RUN / "gdb.sock")
            observation["initial_stop"] = remote.packet("?")
            if remote.packet("Z1,2001309,1") != "OK":
                raise ValueError("Read-only hardware breakpoint unavailable")
            observation["breakpoint_stop"] = remote.packet("c")
            registers = bytes.fromhex(remote.packet("g"))
            values = struct.unpack_from("<17Q", registers)
            observation["registers"] = dict(zip(
                ["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp",
                 "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15", "rip"], values))
            if values[16] != 0x2001309:
                raise ValueError("Wrong breakpoint address")
            if remote.packet("Qqemu.PhyMemMode:1") != "OK":
                raise ValueError("Physical read mode unavailable")
            root = values[5]
            observation["root_page_hex"] = b"".join(
                remote.read(root + offset, 512) for offset in range(0, 4096, 512)
            ).hex()
            observation["walks"] = {
                hex(va): remote.walk(root, va)
                for va in [0x200130c, values[7], 0x22f6038, 0x1000]
            }
            if remote.packet("z1,2001309,1") != "OK":
                raise ValueError("Cannot remove hardware breakpoint")
            if remote.packet("D") != "OK":
                raise ValueError("Debugger detach failed")
            remote.socket.close()
            remote = None
            observation["raw_qemu_exit"] = process.wait(timeout=30)
    finally:
        if remote is not None:
            remote.socket.close()
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=10)
        for connection in peer:
            connection.close()
        listener.close()
        observation["duration_s"] = time.monotonic() - started
        (RUN / "observation.json").write_text(json.dumps(observation, indent=2) + "\n")
    observation["original_artifacts_unchanged"] = all(
        sha(ORIGINAL / rel) == expected for rel, expected in original["esp_sha256"].items())
    observation["serial"] = (RUN / "serial.txt").read_text()
    (EVIDENCE / "observation.json").write_text(json.dumps(observation, indent=2) + "\n")
    print(json.dumps({k: v for k, v in observation.items() if k != "root_page_hex"}, indent=2))


if __name__ == "__main__":
    main()
