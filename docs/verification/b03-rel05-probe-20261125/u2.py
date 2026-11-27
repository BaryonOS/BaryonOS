#!/usr/bin/env python3
"""B03-REL05-P0 U2 probe (feasibility, not acceptance evidence).

Boots the repo-selected OVMF (tools/common.firmware()) with a disposable swtpm
behind QEMU tpm-crb, no disk, no network. If OVMF drops to its built-in UEFI
Shell, sends `dh -p <TCG2 GUID>` over serial to list handles carrying
EFI_TCG2_PROTOCOL (GUID 607F766C-7455-42BE-930B-E4D76DB2720F, edk2
MdePkg/Include/Protocol/Tcg2Protocol.h). Also captures the OVMF debugcon log
and the swtpm command log (which PCRs OVMF extends).

Usage: python3 -B u2.py <workdir> <repo-root>
"""
import os
import shutil
import socket
import struct
import subprocess
import sys
import time

TCG2_GUID = "607F766C-7455-42BE-930B-E4D76DB2720F"


def main(work, repo):
    sys.path.insert(0, os.path.join(repo, "tools"))
    import common
    code, varst = common.firmware()
    work = os.path.abspath(work)
    os.chdir(work)
    for f in ("u2ctl.sock", "ser.sock", "mon.sock", "ovmf-debug.log", "swtpm-u2.log", "serial-u2.txt"):
        if os.path.exists(f):
            os.remove(f)
    shutil.rmtree("state-u2", ignore_errors=True)
    os.makedirs("state-u2")
    shutil.copyfile(varst, "OVMF_VARS.fd")
    sw = subprocess.Popen(["swtpm", "socket", "--tpm2", "--tpmstate", "dir=state-u2",
                           "--ctrl", "type=unixio,path=u2ctl.sock", "--log", "file=swtpm-u2.log,level=20"])
    while not os.path.exists("u2ctl.sock"):
        time.sleep(0.05)
    qemu = ["qemu-system-x86_64", "-machine", common.qemu_machine() if hasattr(common, "qemu_machine") else "q35",
            "-accel", "tcg", "-cpu", "qemu64", "-m", "256M", "-nodefaults", "-display", "none",
            "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={code}",
            "-drive", "if=pflash,unit=1,format=raw,file=OVMF_VARS.fd",
            "-chardev", "socket,id=chrtpm,path=u2ctl.sock",
            "-tpmdev", "emulator,id=tpm0,chardev=chrtpm", "-device", "tpm-crb,tpmdev=tpm0",
            "-serial", "unix:ser.sock,server=on,wait=on",
            "-debugcon", "file:ovmf-debug.log", "-global", "isa-debugcon.iobase=0x402",
            "-monitor", "unix:mon.sock,server=on,wait=off"]
    print("QEMU:", " ".join(qemu))
    q = subprocess.Popen(qemu)
    serial_log = b""
    try:
        while not os.path.exists("ser.sock"):
            time.sleep(0.05)
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.connect("ser.sock")
        s.settimeout(0.5)

        def pump(seconds, until=None):
            nonlocal serial_log
            end = time.time() + seconds
            while time.time() < end:
                try:
                    chunk = s.recv(65536)
                    if not chunk:
                        break
                    serial_log += chunk
                except socket.timeout:
                    pass
                if until and until in serial_log:
                    return True
            return False

        pump(120, b"Press any key")
        s.sendall(b"\r")  # BdsDxe waits for a key before drawing the Boot Manager menu
        pump(60, b"Internal")
        shell = b"Shell>" in serial_log
        if not shell and b"Internal" in serial_log:
            # Boot Manager menu: second entry is the built-in shell
            pump(3)
            for key in (b"\x1b[B", b"\r"):
                n = len(serial_log)
                s.sendall(key)
                pump(3)
                print("key", key, "-> new serial bytes", len(serial_log) - n)
            shell = pump(60, b"Shell>")
        print("shell prompt reached:", shell)
        if shell:
            # skip startup.nsh countdown already passed; issue the query
            for cmd in (f"dh -p {TCG2_GUID}\r", "ver\r"):
                for ch in cmd:
                    s.sendall(ch.encode())
                    time.sleep(0.02)
                pump(8)
        else:
            # Interrupt the countdown (ESC) if it is still pending, give one more chance
            s.sendall(b"\x1b")
            pump(30, b"Shell>")
    finally:
        try:
            m = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            m.connect("mon.sock")
            m.sendall(b"quit\n")
            time.sleep(1)
            m.close()
        except OSError as exc:
            print("monitor quit failed:", exc)
        q.wait(timeout=20)
        try:
            c = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            c.connect("u2ctl.sock")
            c.sendall(struct.pack(">I", 3))
            c.recv(4)
        except OSError as exc:
            print("swtpm ctrl shutdown:", exc)
        sw.wait(timeout=10)
        with open("serial-u2.txt", "wb") as f:
            f.write(serial_log)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
