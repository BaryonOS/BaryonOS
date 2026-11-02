#!/usr/bin/env python3
"""Native two-app focus/drag/compositor recovery with disposable media only.

Serial snapshots are observations, not pixel or storage substitutes. This
runner independently checks screenshots, protected chrome and final disk bytes.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import time
import uuid

from common import BUILD, ROOT, environment, firmware, qemu_machine, sha256, source_hash
from build_users import inspect_user
from run_fpu import private_path, publish, publish_json, load_json
from run_graphics import QmpPipe, ppm
from run_system import check_esp, check_image
from run_materiaal_probe import disk_oracle, TEXT, TEXT_SUM
import volume_disk

FEATURE = "native-window-session-probe"
NAMES = {"window-init", "window-compositor", "materiaal-app", "calculator-app",
         "keyring-service", "volume-service", "input-source", "input-policy"}
SERIAL_LIMIT = 4 * 1024 * 1024


def serial(path: Path) -> bytes:
    if not path.exists():
        return b""
    path = private_path(path, regular=True)
    if path.stat().st_size > SERIAL_LIMIT:
        raise ValueError("Oversized native serial")
    raw = path.read_bytes()
    raw.decode("utf-8", "strict")
    if b"WINDOW:FAIL:" in raw or b"PANIC" in raw:
        raise ValueError("Native guest failed")
    return raw


def snapshots(raw: bytes, role: str) -> list[dict]:
    prefix = f"WINDOW:{role}:"
    result = []
    for match in re.finditer(re.escape(prefix + "SNAPSHOT:BEGIN\n") + r"(.*?)" +
                             re.escape(prefix + "SNAPSHOT:END\n"), raw.decode("utf-8", "strict"), re.S):
        current = {}
        for line in match[1].splitlines():
            field = re.fullmatch(re.escape(prefix) + r"([A-Z_]+):([0-9]{20})", line)
            if field is None or field[1] in current or int(field[2]) > (1 << 64) - 1:
                raise ValueError("Malformed/duplicate native app snapshot")
            current[field[1]] = int(field[2])
        if current.get("READY") != 1 or current.get("GEN", 0) == 0:
            raise ValueError("Unready native app snapshot")
        result.append(current)
    return result


def numbers(raw: bytes, name: str) -> list[int]:
    return [int(m[1]) for m in re.finditer(rb"(?:^|\n)" + re.escape(name.encode()) + rb":([0-9]{20})\n", raw)]


def validate_build(frozen: str) -> dict:
    manifest = load_json(private_path(BUILD / "manifest.json", regular=True))
    if (manifest.get("kernel_feature") != FEATURE or manifest.get("loader_feature") != "desktop-mode"
            or manifest.get("source_sha256") != frozen or manifest.get("source_fingerprint_version") != 3):
        raise ValueError("Build the unchanged native window probe first")
    check_esp(BUILD / "esp", manifest)
    apps = manifest.get("materiaal", {}).get("apps", [])
    if len(apps) != len(NAMES) or {a["name"] for a in apps} != NAMES:
        raise ValueError("Wrong native app set")
    kernel = private_path(BUILD / "esp/kernel.elf", regular=True).read_bytes()
    for app in apps:
        path = private_path(BUILD / "users/materiaal" / (app["name"] + ".elf"), regular=True)
        if app != {"name": app["name"], "sha256": sha256(path), "elf": inspect_user(path)} or path.read_bytes() not in kernel:
            raise ValueError("Native app identity/embedding differs")
    return manifest


def run(*, timeout: int, bios_rom: Path, vga_rom: Path, smoke: bool = False) -> dict:
    if not 30 <= timeout <= 1800:
        raise ValueError("Timeout must be bounded 30..1800 seconds")
    private_path(BUILD)
    frozen = source_hash()
    manifest = validate_build(frozen)
    env = environment()
    qemu = Path(shutil.which("qemu-system-x86_64", path=env["PATH"]) or "missing")
    qemu_hash = sha256(qemu)
    code, variables = firmware()
    directory = private_path(BUILD / "run" / ("window-" + uuid.uuid4().hex))
    directory.mkdir(parents=True)
    shutil.copytree(BUILD / "esp", directory / "esp")
    check_esp(directory / "esp", manifest)
    for original, name, limit in ((code, "OVMF_CODE.fd", 16 << 20), (variables, "OVMF_VARS.template.fd", 16 << 20),
                                  (variables, "OVMF_VARS.fd", 16 << 20), (bios_rom, "bios-256k.bin", 1 << 20), (vga_rom, "VGA.rom", 1 << 20)):
        info = original.lstat()
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or not 0 < info.st_size <= limit:
            raise ValueError("Invalid external firmware/ROM")
        publish(directory / name, original.read_bytes())
    volume_disk.create_image(directory / "volume.img", "empty")
    check_image(directory / "volume.img")
    immutable = {p.relative_to(directory).as_posix(): sha256(p) for p in directory.rglob("*")
                 if p.is_file() and p.name not in {"OVMF_VARS.fd", "volume.img"}}
    originals = {str(p): sha256(p) for p in (code, variables, bios_rom, vga_rom)}
    command = [str(qemu), "-L", ".", "-machine", qemu_machine(), "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M",
               "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "file:serial.txt", "-nic", "none", "-no-reboot",
               "-device", "VGA,vgamem_mb=16,romfile=VGA.rom", "-qmp", "stdio",
               "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
               "-drive", "if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
               "-drive", "if=pflash,unit=1,format=raw,file=OVMF_VARS.fd",
               "-drive", "if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
               "-object", "rng-random,id=rng0,filename=/dev/urandom", "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000",
               "-drive", "if=none,id=volume,format=raw,cache=writeback,file=volume.img",
               "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST"]
    start = time.monotonic()
    deadline = start + timeout
    report = dict(schema_version=1, passed=False, native_gate=False, smoke_only=smoke, source_sha256=frozen,
                  source_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                  source_fingerprint_version=3, kernel_feature=FEATURE, manifest=manifest, command=command,
                  run_directory=str(directory), qemu_sha256=qemu_hash, immutable=immutable, phases={})
    publish_json(directory / "result.json", report)
    process = control = None
    try:
        with (directory / "stderr.txt").open("xb") as stderr:
            process = subprocess.Popen(command, cwd=directory, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, bufsize=0)
            control = QmpPipe(process, deadline)

            def wait(name, predicate):
                while True:
                    raw = serial(directory / "serial.txt")
                    state = {role: snapshots(raw, role) for role in ("MATERIAL", "CALCULATOR")}
                    if predicate(raw, state):
                        path = private_path(directory / (name + ".ppm"))
                        control.command("screendump", {"filename": str(path)})
                        image = path.read_bytes()
                        if ppm(image)[:2] != (1280, 720):
                            raise ValueError("Wrong actual GOP size")
                        report["phases"][name] = {"seconds": time.monotonic() - start, "state": {r: s[-1] for r, s in state.items() if s}, "frame_sha256": sha256(path)}
                        print("WINDOW phase " + name + " " + str(round(time.monotonic() - start, 2)), flush=True)
                        return state
                    if process.poll() is not None or time.monotonic() >= deadline:
                        raise TimeoutError("Native phase did not complete: " + name)
                    time.sleep(0.05)

            def key(*keys):
                control.command("send-key", {"keys": [{"type": "qcode", "data": k} for k in keys], "hold-time": 1})
                time.sleep(0.04)

            initial = wait("initial", lambda raw, s: all(s.values()) and 31 in numbers(raw, "WINDOW:INITMASK"))
            if smoke:
                control.command("quit")
            else:
                key("ret")
                wait("editor", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["SCREEN"] == 2)
                for char in TEXT.decode():
                    key("spc" if char == " " else char)
                dirty = wait("dirty", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["DIRTY"] == 1 and s["MATERIAL"][-1]["DRAFT_BYTES"] == len(TEXT) and s["MATERIAL"][-1]["DRAFT_SUM"] == TEXT_SUM)
                key("f6")
                for keys in (("2",), ("shift", "equal"), ("3",), ("ret",)):
                    key(*keys)
                calculated = wait("calculated", lambda raw, s: s["CALCULATOR"] and s["CALCULATOR"][-1].get("HAS_ANSWER") == 1 and s["CALCULATOR"][-1].get("ANSWER_LOW") == 5_000_000_000 and s["CALCULATOR"][-1].get("ANSWER_HIGH") == 0)
                key("f6")
                key("f12")
                recovered = wait("recovered", lambda raw, s: 1 in numbers(raw, "WINDOW:RECOVERIES") and all(s.values()) and all(s[r][-1]["SERVICE_GENERATION"] == 2 for r in s))
                for field in ("PID", "DRAFT_BYTES", "DRAFT_SUM", "DIRTY", "SAVE_ACKS", "OBJECT"):
                    if recovered["MATERIAL"][-1][field] != dirty["MATERIAL"][-1][field]:
                        raise ValueError("Recovery changed actual unsaved work: " + field)
                for field in ("PID", "EXPRESSION_BYTES", "ANSWER_LOW", "ANSWER_HIGH", "HAS_ANSWER", "EVALUATIONS"):
                    if recovered["CALCULATOR"][-1][field] != calculated["CALCULATOR"][-1][field]:
                        raise ValueError("Recovery changed actual calculation: " + field)
                for role in recovered:
                    if any(recovered[role][-1][f] == initial[role][-1][f] for f in ("SURFACE_EPOCH", "INPUT_EPOCH")):
                        raise ValueError("Recovery reused presentation epoch")
                key("ctrl", "s")
                wait("saved", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["SAVE_ACKS"] == 1 and s["MATERIAL"][-1]["DIRTY"] == 0)
                key("esc")
                wait("list", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["SCREEN"] == 1)
                key("tab"); key("tab"); key("ret")
                wait("reopened", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["SCREEN"] == 2 and s["MATERIAL"][-1]["DIRTY"] == 0 and s["MATERIAL"][-1]["DRAFT_SUM"] == TEXT_SUM)
                key("f10")
            process.wait(timeout=max(0.1, deadline - time.monotonic()))
            raw = serial(directory / "serial.txt")
            report["raw_qemu_exit"] = process.returncode
            if smoke:
                if process.returncode != 0:
                    raise ValueError("Smoke QEMU quit failed")
            else:
                if process.returncode != 33 or not raw.endswith(b"WINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n"):
                    raise ValueError("Missing actual cleanup/normal exit")
                if numbers(raw, "WINDOW:FAULT_SLOT") != [4] or numbers(raw, "WINDOW:FAULT_VECTOR") != [6] or numbers(raw, "WINDOW:RESTARTS") != [1]:
                    raise ValueError("Wrong actual fault/restart")
                report["disk_oracle"] = disk_oracle(private_path(directory / "volume.img", regular=True).read_bytes())
            if source_hash() != frozen or validate_build(frozen) != manifest or sha256(qemu) != qemu_hash:
                raise ValueError("Source/build/QEMU changed during run")
            if {name: sha256(private_path(directory / name, regular=True)) for name in immutable} != immutable or {name: sha256(Path(name)) for name in originals} != originals:
                raise ValueError("Immutable inputs changed during run")
            report.update(passed=True, native_gate=not smoke, duration_s=time.monotonic() - start, serial_sha256=sha256(directory / "serial.txt"), qmp_trace=control.trace)
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    finally:
        if process is not None and process.poll() is None:
            process.kill(); process.wait(timeout=5)
        if control is not None:
            control.close()
        publish_json(directory / "result.json", report)
        publish_json(BUILD / "window-session-report.json", report)
    print(json.dumps({k: report.get(k) for k in ("passed", "native_gate", "error", "run_directory", "duration_s")}), flush=True)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=int, default=600)
    parser.add_argument("--bios-rom", type=Path, required=True)
    parser.add_argument("--vga-rom", type=Path, required=True)
    parser.add_argument("--smoke-only", action="store_true")
    args = parser.parse_args()
    raise SystemExit(0 if run(timeout=args.timeout, bios_rom=args.bios_rom, vga_rom=args.vga_rom, smoke=args.smoke_only)["passed"] else 1)
