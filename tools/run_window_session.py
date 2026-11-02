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

from common import BUILD, ROOT, TARGET, environment, firmware, qemu_machine, sha256, source_hash
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
BODY = (28, 114, 880, 510)
FIXTURE_PHASES = ("initial", "editor", "dirty", "saved", "list", "reopened",
                  "dirty-focus-lost", "dirty-rebound", "dirty-recovered")
SNAPSHOT_FIELDS = {
    "MATERIAL": {"PID", "SERVICE_GENERATION", "SURFACE_EPOCH", "INPUT_EPOCH", "READY", "GEN", "FRAMES",
                 "SAVE_ACKS", "REVISION", "SAVED_BYTES", "SAVED_SUM", "SCREEN", "DIRTY", "DRAFT_BYTES", "DRAFT_SUM", "OBJECT", "SCALE", "ERRORS"},
    "CALCULATOR": {"PID", "SERVICE_GENERATION", "SURFACE_EPOCH", "INPUT_EPOCH", "READY", "GEN", "FRAMES",
                   "EVALUATIONS", "FAILURES", "EXPRESSION_BYTES", "CURRENT", "CARET", "ACTIVE", "SCALE", "ERRORS", "ANSWER_LOW", "ANSWER_HIGH", "HAS_ANSWER"},
}


def generate_fixture() -> dict:
    directory = private_path(BUILD / ("window-fixture-" + uuid.uuid4().hex))
    directory.mkdir(parents=True)
    frozen = source_hash()
    env = environment()
    env["CARGO_TARGET_DIR"] = str(private_path(TARGET / "window-fixture"))
    env["OS_WINDOW_FIXTURE_DIRECTORY"] = str(directory)
    command = ["cargo", "+1.90.0", "test", "--offline", "--locked", "--manifest-path",
               str(ROOT / "ui/native/Cargo.toml"), "--test", "window_native_fixture", "--", "--nocapture"]
    with (directory / "generator.log").open("xb") as log:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=180, check=False)
    if result.returncode or source_hash() != frozen:
        raise ValueError("Host fixture failed or source changed: " + str(directory))
    hashes, frames = {}, {}
    for size in ("880x510", "1120x660"):
        for phase in FIXTURE_PHASES:
            path = private_path(directory / f"{size}-{phase}.ppm", regular=True)
            if path.stat().st_size > 3 * 1120 * 660 + 128:
                raise ValueError("Oversized oracle")
            raw = path.read_bytes()
            if ppm(raw)[:2] != tuple(map(int, size.split("x"))):
                raise ValueError("Wrong host oracle dimensions")
            hashes[path.name] = sha256(path)
            if size == "880x510":
                frames[phase] = raw
    metadata = dict(kind="host-rendered real Controller/Frame oracle; not native execution",
                    source_sha256=frozen, source_fingerprint_version=3, generator_command=command,
                    generator_exit=result.returncode, files_sha256=hashes)
    publish_json(directory / "fixture.json", metadata)
    return dict(directory=directory, metadata=metadata, frames=frames)


def crop(raw: bytes, rectangle: tuple) -> bytes:
    width, height, pixels = ppm(raw)
    x, y, w, h = rectangle
    if min(x, y) < 0 or min(w, h) <= 0 or x + w > width or y + h > height:
        raise ValueError("Crop exceeds actual framebuffer")
    return b"".join(pixels[((y + row) * width + x) * 3:((y + row) * width + x + w) * 3] for row in range(h))


def material_matches(raw: bytes, expected: bytes) -> bool:
    return ppm(expected)[:2] == BODY[2:] and crop(raw, BODY) == ppm(expected)[2]


def changed_outside(before: bytes, after: bytes, rectangle: tuple) -> bool:
    width, height, old = ppm(before)
    w2, h2, new = ppm(after)
    if (width, height) != (w2, h2):
        raise ValueError("Framebuffer geometry changed")
    crop(before, rectangle)  # Validate bounds before indexing.
    x, y, w, h = rectangle
    for row in range(height):
        start, end = row * width * 3, (row + 1) * width * 3
        if y <= row < y + h:
            left, right = start + x * 3, start + (x + w) * 3
            if old[start:left] != new[start:left] or old[right:end] != new[right:end]:
                return True
        elif old[start:end] != new[start:end]:
            return True
    return False


def serial(path: Path) -> bytes:
    if not path.exists():
        return b""
    path = private_path(path, regular=True)
    if path.stat().st_size > SERIAL_LIMIT:
        raise ValueError("Oversized native serial")
    raw = path.read_bytes()
    raw.decode("utf-8", "strict")
    if b"WINDOW:FAIL:" in raw or b"PANIC" in raw or b"\0" in raw:
        raise ValueError("Native guest failed")
    return raw


def snapshots(raw: bytes, role: str) -> list[dict]:
    if role not in SNAPSHOT_FIELDS:
        raise ValueError("Unknown native role")
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
        if set(current) != SNAPSHOT_FIELDS[role] or current.get("READY") != 1 or current.get("GEN", 0) == 0:
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
    fixture = None if smoke else generate_fixture()
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

            implementation = Path(os.readlink(f"/proc/{process.pid}/exe"))
            implementation_hash = sha256(implementation)
            report["qemu_implementation"] = {"path": str(implementation), "sha256": implementation_hash}
            captured = {}

            def wait(name, predicate, oracle=None):
                while True:
                    raw = serial(directory / "serial.txt")
                    state = {role: snapshots(raw, role) for role in ("MATERIAL", "CALCULATOR")}
                    if predicate(raw, state):
                        path = private_path(directory / "candidate.ppm", regular=(directory / "candidate.ppm").exists())
                        control.command("screendump", {"filename": str(path)})
                        image = private_path(path, regular=True).read_bytes()
                        if ppm(image)[:2] != (1280, 720):
                            raise ValueError("Wrong actual GOP size")
                        if oracle is not None and not material_matches(image, fixture["frames"][oracle]):
                            if time.monotonic() >= deadline:
                                raise TimeoutError("Native pixels differ from complete host oracle: " + name)
                            time.sleep(0.2)
                            continue
                        final_path = private_path(directory / (name + ".ppm"))
                        publish(final_path, image)
                        captured[name] = image
                        report["phases"][name] = {"seconds": time.monotonic() - start, "state": {r: s[-1] for r, s in state.items() if s}, "frame_sha256": sha256(final_path), "oracle": oracle}
                        print("WINDOW phase " + name + " " + str(round(time.monotonic() - start, 2)), flush=True)
                        return state
                    if process.poll() is not None or time.monotonic() >= deadline:
                        raise TimeoutError("Native phase did not complete: " + name)
                    time.sleep(0.05)

            def key(*keys):
                control.command("send-key", {"keys": [{"type": "qcode", "data": k} for k in keys], "hold-time": 1})
                time.sleep(0.04)

            pointer = [0, 0]
            def move(x, y):
                while pointer != [x, y]:
                    changes = [max(-96, min(96, target - current)) for target, current in zip((x, y), pointer)]
                    events = [{"type": "rel", "data": {"axis": axis, "value": delta}} for axis, delta in zip(("x", "y"), changes) if delta]
                    control.command("input-send-event", {"events": events})
                    pointer[:] = [current + delta for current, delta in zip(pointer, changes)]
                    time.sleep(0.04)

            def button(down):
                control.command("input-send-event", {"events": [{"type": "btn", "data": {"down": down, "button": "left"}}]})
                time.sleep(0.1)

            initial = wait("boot", lambda raw, s: all(s.values()) and 31 in numbers(raw, "WINDOW:INITMASK"))
            if smoke:
                control.command("quit")
            else:
                move(1260, 50)
                initial = wait("initial", lambda raw, s: all(s.values()), "initial")
                key("ret")
                wait("editor", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["SCREEN"] == 2, "editor")
                for char in TEXT.decode():
                    key("spc" if char == " " else char)
                dirty = wait("dirty", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["DIRTY"] == 1 and s["MATERIAL"][-1]["DRAFT_BYTES"] == len(TEXT) and s["MATERIAL"][-1]["DRAFT_SUM"] == TEXT_SUM, "dirty")
                # Exact app bounds, without pixel masks: typing may only change
                # Material's assigned body, never the other app or system chrome.
                for phase in ("editor", "dirty"):
                    if changed_outside(captured["initial"], captured[phase], BODY):
                        raise ValueError("App paint escaped its assigned body")
                key("f6")
                for keys in (("2",), ("shift", "equal"), ("3",), ("ret",)):
                    key(*keys)
                calculated = wait("calculated", lambda raw, s: s["CALCULATOR"] and s["CALCULATOR"][-1].get("HAS_ANSWER") == 1 and s["CALCULATOR"][-1].get("ANSWER_LOW") == 5_000_000_000 and s["CALCULATOR"][-1].get("ANSWER_HIGH") == 0)
                move(1060, 118); button(True); move(900, 100); button(False)
                wait("moved", lambda raw, s: any(v > 0 for v in numbers(raw, "WINDOW:COMPOSITOR:MOVES")))
                if crop(captured["calculated"], (932, 140, 320, 484)) != crop(captured["moved"], (772, 122, 320, 484)):
                    raise ValueError("Moved opaque window did not preserve its actual app pixels")
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
                pointer[:] = [0, 0]
                move(1260, 50)
                wait("rebound-editor", lambda raw, s: s["MATERIAL"][-1]["SERVICE_GENERATION"] == 2, "dirty-rebound")
                key("right")
                wait("restored-editor", lambda raw, s: s["MATERIAL"][-1]["SERVICE_GENERATION"] == 2, "dirty-recovered")
                key("ctrl", "s")
                wait("saved", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["SAVE_ACKS"] == 1 and s["MATERIAL"][-1]["DIRTY"] == 0, "saved")
                key("esc")
                wait("list", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["SCREEN"] == 1, "list")
                key("tab"); key("tab"); key("ret")
                wait("reopened", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["SCREEN"] == 2 and s["MATERIAL"][-1]["DIRTY"] == 0 and s["MATERIAL"][-1]["DRAFT_SUM"] == TEXT_SUM, "reopened")
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
            if sha256(implementation) != implementation_hash:
                raise ValueError("QEMU implementation changed")
            for name, image in captured.items():
                if private_path(directory / (name + ".ppm"), regular=True).read_bytes() != image:
                    raise ValueError("Stored frame changed: " + name)
            if fixture is not None:
                for name, digest in fixture["metadata"]["files_sha256"].items():
                    if sha256(private_path(fixture["directory"] / name, regular=True)) != digest:
                        raise ValueError("Host oracle changed")
                report["oracle"] = fixture["metadata"]
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
