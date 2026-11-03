#!/usr/bin/env python3
"""Native two-app focus/drag/compositor recovery with disposable media only.

Serial snapshots are observations, not pixel or storage substitutes. This
runner independently checks screenshots, protected chrome and final disk bytes.
"""
from __future__ import annotations
import argparse
from array import array
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
CLIPBOARD_PHASES = ("clipboard-copied", "clipboard-numeric", "clipboard-back",
                    "clipboard-empty", "clipboard-recovered-empty")
CALCULATOR_CLIPBOARD_PHASES = ("clipboard-unsupported", "clipboard-pasted", "clipboard-calculator-copied")
CALCULATOR_BODY = (932, 140, 320, 484)
SWITCHER_MAX_PIXELS = 1_400_000


def switcher_geometry(scale: int) -> tuple:
    if scale == 1:
        return (1280, 720, BODY, CALCULATOR_BODY, (330, 214, 620, 292))
    if scale == 2:
        return (1920, 1080, (56, 228, 1120, 660), (1224, 280, 640, 608), (340, 248, 1240, 584))
    raise ValueError("Only 100% and 200% switcher profiles are supported")


def switcher_matches(raw: bytes, expected: bytes, scale: int) -> bool:
    width, height, _, _, bounds = switcher_geometry(scale)
    return (ppm(raw)[:2] == (width, height) and ppm(expected)[:2] == bounds[2:]
            and crop(raw, bounds) == ppm(expected)[2])


def generate_switcher_fixture(directory: Path, image: bytes, scale: int) -> dict:
    width, height, body, calc, _ = switcher_geometry(scale)
    if ppm(image)[:2] != (width, height):
        raise ValueError("Wrong native backing capture dimensions")
    backing = array("I", [0]) * SWITCHER_MAX_PIXELS
    offset = 0
    for rectangle in (body, calc):
        pixels = crop(image, rectangle)
        for at in range(0, len(pixels), 3):
            backing[offset + at // 3] = int.from_bytes(pixels[at:at + 3], "big")
        offset += rectangle[2] * rectangle[3]
    if backing.itemsize != 4:
        raise ValueError("Host cannot encode 32-bit backing")
    import sys
    if sys.byteorder != "little":
        backing.byteswap()
    publish(directory / "backing.bin", backing.tobytes())
    env = environment()
    env.update(CARGO_TARGET_DIR=str(private_path(TARGET / "window-switcher-fixture")),
               OS_SWITCHER_FIXTURE_DIRECTORY=str(directory), OS_SWITCHER_FIXTURE_SCALE=str(scale))
    command = ["cargo", "+1.90.0", "test", "--offline", "--locked", "--manifest-path",
               str(ROOT / "sdk/examples/window-session/Cargo.toml"), "--test", "switcher_native_fixture"]
    with (directory / "switcher-generator.log").open("xb") as log:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT,
                                timeout=180, check=False)
    if result.returncode:
        raise ValueError("Native-backing host oracle failed: " + str(directory))
    paths = [directory / "backing.bin", directory / "switcher-generator.log",
             directory / "switcher-0.ppm", directory / "switcher-1.ppm"]
    return dict(directory=directory, frames={index: (directory / f"switcher-{index}.ppm").read_bytes()
                                            for index in (0, 1)},
                metadata=dict(kind="host shell reference using captured native app backing; not native execution",
                              source_sha256=source_hash(), generator_commands=[command], generator_exit=0,
                              native_backing_capture_sha256=hashlib.sha256(image).hexdigest(),
                              files_sha256={path.name: sha256(path) for path in paths}))
SNAPSHOT_FIELDS = {
    "MATERIAL": {"PID", "SERVICE_GENERATION", "SURFACE_EPOCH", "INPUT_EPOCH", "READY", "GEN", "FRAMES",
                 "SAVE_ACKS", "REVISION", "SAVED_BYTES", "SAVED_SUM", "SCREEN", "DIRTY", "DRAFT_BYTES", "DRAFT_SUM", "OBJECT", "SCALE", "ERRORS", "PROBES"},
    "CALCULATOR": {"PID", "SERVICE_GENERATION", "SURFACE_EPOCH", "INPUT_EPOCH", "READY", "GEN", "FRAMES",
                   "EVALUATIONS", "FAILURES", "EXPRESSION_BYTES", "EXPRESSION_SUM", "CURRENT", "CARET", "ACTIVE", "SCALE", "ERRORS", "ANSWER_LOW", "ANSWER_HIGH", "HAS_ANSWER", "PROBES"},
    "COMPOSITOR": {"GENERATION", "PID", "COMMITS", "INPUT", "REJECTED", "FOCUSED", "MOVES", "SWITCHES", "MATERIAL", "CALCULATOR", "FATAL", "POINTER_X", "POINTER_Y", "BUTTONS"},
}


def generate_fixture(*, clipboard: bool = False) -> dict:
    directory = private_path(BUILD / ("window-fixture-" + uuid.uuid4().hex))
    directory.mkdir(parents=True)
    frozen = source_hash()
    env = environment()
    env["CARGO_TARGET_DIR"] = str(private_path(TARGET / "window-fixture"))
    env["OS_WINDOW_FIXTURE_DIRECTORY"] = str(directory)
    command = ["cargo", "+1.90.0", "test", "--offline", "--locked", "--manifest-path",
               str(ROOT / "ui/native/Cargo.toml"), "--test", "window_native_fixture", "--", "--nocapture"]
    commands = [command]
    if clipboard:
        commands.append(["cargo", "+1.90.0", "test", "--offline", "--locked", "--manifest-path",
                         str(ROOT / "sdk/examples/calculator/Cargo.toml"), "--test",
                         "clipboard_native_fixture", "--", "--nocapture"])
    with (directory / "generator.log").open("xb") as log:
        for generated in commands:
            result = subprocess.run(generated, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=180, check=False)
            if result.returncode:
                break
    if result.returncode or source_hash() != frozen:
        raise ValueError("Host fixture failed or source changed: " + str(directory))
    hashes, frames = {}, {}
    for size in ("880x510", "1120x660"):
        for phase in FIXTURE_PHASES + (CLIPBOARD_PHASES if clipboard else ()):
            path = private_path(directory / f"{size}-{phase}.ppm", regular=True)
            if path.stat().st_size > 3 * 1120 * 660 + 128:
                raise ValueError("Oversized oracle")
            raw = path.read_bytes()
            if ppm(raw)[:2] != tuple(map(int, size.split("x"))):
                raise ValueError("Wrong host oracle dimensions")
            hashes[path.name] = sha256(path)
            if size == "880x510":
                frames[phase] = raw
    if clipboard:
        for size in ("320x484", "640x608"):
            for phase in CALCULATOR_CLIPBOARD_PHASES:
                path = private_path(directory / f"calculator-{size}-{phase}.ppm", regular=True)
                if path.stat().st_size > 3 * 640 * 608 + 128:
                    raise ValueError("Oversized calculator oracle")
                raw = path.read_bytes()
                if ppm(raw)[:2] != tuple(map(int, size.split("x"))):
                    raise ValueError("Wrong calculator oracle dimensions")
                hashes[path.name] = sha256(path)
                if size == "320x484":
                    frames[phase] = raw
    metadata = dict(kind="host-rendered real Controller/Frame oracle; not native execution",
                    source_sha256=frozen, source_fingerprint_version=3, generator_command=command,
                    generator_commands=commands, clipboard_oracles=clipboard,
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


def calculator_matches(raw: bytes, expected: bytes) -> bool:
    return ppm(expected)[:2] == CALCULATOR_BODY[2:] and crop(raw, CALCULATOR_BODY) == ppm(expected)[2]


def oracle_matches(raw: bytes, expected: bytes, name: str) -> bool:
    if name in CALCULATOR_CLIPBOARD_PHASES:
        return calculator_matches(raw, expected)
    return material_matches(raw, expected)


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
        valid = (current.get("GENERATION", 0) > 0 and current.get("PID", 0) > 0 and current.get("FATAL") == 0) if role == "COMPOSITOR" else (current.get("READY") == 1 and current.get("GEN", 0) > 0)
        if set(current) != SNAPSHOT_FIELDS[role] or not valid:
            raise ValueError("Unready native app snapshot")
        result.append(current)
    return result


def numbers(raw: bytes, name: str) -> list[int]:
    return [int(m[1]) for m in re.finditer(rb"(?:^|\n)" + re.escape(name.encode()) + rb":([0-9]{20})\n", raw)]


def validate_build(frozen: str, scale: int = 1) -> dict:
    switcher_geometry(scale)
    manifest = load_json(private_path(BUILD / "manifest.json", regular=True))
    feature = FEATURE if scale == 1 else "native-window-session-hidpi-probe"
    loader = "desktop-mode" if scale == 1 else "desktop-hidpi"
    if (manifest.get("kernel_feature") != feature or manifest.get("loader_feature") != loader
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


def run(*, timeout: int, bios_rom: Path, vga_rom: Path, smoke: bool = False,
        recovery_handshake: bool = False, clipboard: bool = False,
        switcher: bool = False, scale: int = 1) -> dict:
    if not 30 <= timeout <= 1800:
        raise ValueError("Timeout must be bounded 30..1800 seconds")
    if smoke and recovery_handshake or clipboard and (smoke or recovery_handshake):
        raise ValueError("Clipboard requires the full native gate; choose one diagnostic mode")
    if switcher and (smoke or recovery_handshake or clipboard) or scale != 1 and not switcher:
        raise ValueError("Switcher is a separate native part gate; scale 2 requires --switcher")
    width, height, body, calc_body, switcher_bounds = switcher_geometry(scale)
    diagnostic = smoke or recovery_handshake
    private_path(BUILD)
    frozen = source_hash()
    manifest = validate_build(frozen, scale)
    fixture = None if diagnostic or switcher else (generate_fixture(clipboard=True) if clipboard else generate_fixture())
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
    report = dict(schema_version=1, passed=False, native_gate=False, smoke_only=diagnostic,
                  recovery_handshake_only=recovery_handshake, clipboard_requested=clipboard, clipboard_gate=False, source_sha256=frozen,
                  source_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                  source_fingerprint_version=3, kernel_feature=manifest["kernel_feature"], manifest=manifest, command=command,
                  switcher_requested=switcher, switcher_gate=False, scale=scale, viewport=[width, height],
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
            group_generation = [1]

            def comp(raw):
                values = snapshots(raw, "COMPOSITOR")
                return values[-1] if values else {}

            def observe(label, predicate, *, timeout_s=None):
                observation_deadline = deadline if timeout_s is None else min(deadline, time.monotonic() + timeout_s)
                while True:
                    raw = serial(directory / "serial.txt")
                    if predicate(raw):
                        return raw
                    if process.poll() is not None or time.monotonic() >= observation_deadline:
                        raise TimeoutError("Native input acknowledgement missing: " + label)
                    time.sleep(0.02)

            action_started = [None]
            def wait(name, predicate, oracle=None, image_check=None):
                while True:
                    raw = serial(directory / "serial.txt")
                    state = {role: snapshots(raw, role) for role in ("MATERIAL", "CALCULATOR")}
                    if predicate(raw, state):
                        path = private_path(directory / "candidate.ppm", regular=(directory / "candidate.ppm").exists())
                        control.command("screendump", {"filename": str(path)})
                        image = private_path(path, regular=True).read_bytes()
                        if ppm(image)[:2] != (width, height):
                            raise ValueError("Wrong actual GOP size")
                        if (oracle is not None and not oracle_matches(image, fixture["frames"][oracle], oracle)) or (image_check is not None and not image_check(image)):
                            if time.monotonic() >= deadline:
                                raise TimeoutError("Native pixels differ from complete host oracle: " + name)
                            time.sleep(0.2)
                            continue
                        final_path = private_path(directory / (name + ".ppm"))
                        publish(final_path, image)
                        captured[name] = image
                        report["phases"][name] = {"seconds": time.monotonic() - start, "state": {r: s[-1] for r, s in state.items() if s}, "frame_sha256": sha256(final_path), "oracle": oracle}
                        if oracle is not None and action_started[0] is not None:
                            report["phases"][name]["last_input_to_verified_app_image_s"] = time.monotonic() - action_started[0]
                            report["phases"][name]["latency_scope"] = "Conservative host/QMP/observation/capture interval from last key chord to exact full app-body oracle; not whole-screen/guest/scanout latency or U02 budget acceptance."
                        print("WINDOW phase " + name + " " + str(round(time.monotonic() - start, 2)), flush=True)
                        return state
                    if process.poll() is not None or time.monotonic() >= deadline:
                        raise TimeoutError("Native phase did not complete: " + name)
                    time.sleep(0.05)

            def key(*keys):
                raw = serial(directory / "serial.txt")
                current = comp(raw)
                if current.get("GENERATION") != group_generation[0]:
                    raise ValueError("Keyboard injection lacks a current compositor observation")
                role = "CALCULATOR" if current["FOCUSED"] == 1 else "MATERIAL"
                old_keys = numbers(raw, f"WINDOW:{role}:KEYS")[-1]
                old_acks = numbers(raw, f"WINDOW:{role}:INPUT_ACKS")[-1]
                action_started[0] = time.monotonic()
                control.command("send-key", {"keys": [{"type": "qcode", "data": k} for k in keys], "hold-time": 1})
                if keys == ("f6",):
                    observe("F6 focus", lambda raw: comp(raw).get("GENERATION") == group_generation[0] and comp(raw).get("FOCUSED") == 1-current["FOCUSED"])
                elif keys not in (("f10",), ("f12",)):
                    # The compositor consumes Delete itself; the four genuine
                    # modifier edges still reach the focused application.
                    count = 4 if keys == ("ctrl", "shift", "delete") else 2*len(keys)
                    observe("/".join(keys), lambda raw: numbers(raw, f"WINDOW:{role}:KEYS")[-1] >= old_keys+count and numbers(raw, f"WINDOW:{role}:INPUT_ACKS")[-1] >= old_acks+count)

            pointer = [0, 0]
            def move(x, y):
                while pointer != [x, y]:
                    changes = [max(-96, min(96, target - current)) for target, current in zip((x, y), pointer)]
                    events = [{"type": "rel", "data": {"axis": axis, "value": delta}} for axis, delta in zip(("x", "y"), changes) if delta]
                    control.command("input-send-event", {"events": events})
                    pointer[:] = [current + delta for current, delta in zip(pointer, changes)]
                    observe("pointer", lambda raw: comp(raw).get("GENERATION") == group_generation[0] and (comp(raw).get("POINTER_X"), comp(raw).get("POINTER_Y")) == tuple(pointer))

            def button(down):
                control.command("input-send-event", {"events": [{"type": "btn", "data": {"down": down, "button": "left"}}]})
                observe("mouse button", lambda raw: comp(raw).get("GENERATION") == group_generation[0] and comp(raw).get("BUTTONS") == int(down))

            def select_title(x, y, index):
                move(x, y); button(True); button(False)
                observe("mouse focus", lambda raw: comp(raw).get("GENERATION") == group_generation[0] and comp(raw).get("FOCUSED") == index)

            initial = None if recovery_handshake else wait("boot", lambda raw, s: all(s.values()) and 31 in numbers(raw, "WINDOW:INITMASK"))
            if recovery_handshake:
                # Exercise the actual busy-RPC cancellation/retire/rebind path
                # before waiting for full app paints. This never certifies
                # pixels, preserved work, persistence or the complete U01 gate.
                before = comp(observe("initial group", lambda raw:
                    31 in numbers(raw, "WINDOW:INITMASK") and
                    comp(raw).get("GENERATION") == 1 and comp(raw).get("COMMITS", 0) > 0))
                control.command("send-key", {"keys": [{"type": "qcode", "data": "f12"}], "hold-time": 1})
                raw = observe("actual fresh-group rebind", lambda raw:
                    1 in numbers(raw, "WINDOW:RECOVERIES") and
                    numbers(raw, "WINDOW:PRESENTATION_RETIRED") == [1] and
                    numbers(raw, "WINDOW:RESTART_GENERATION") == [2] and
                    comp(raw).get("GENERATION") == 2 and
                    comp(raw).get("PID") != before["PID"])
                if numbers(raw, "WINDOW:FAULT_SLOT") != [4] or numbers(raw, "WINDOW:FAULT_VECTOR") != [6]:
                    raise ValueError("Wrong actual handshake fault")
                report["handshake"] = {"before": before, "after": comp(raw),
                                       "seconds": time.monotonic() - start,
                                       "scope": "actual group rebind only; no full-frame, work, disk or cleanup acceptance"}
                control.command("quit")
            elif smoke:
                control.command("quit")
            elif switcher:
                def complete(raw, state):
                    return (all(state.values()) and all(
                        comp(raw).get(role) == state[role][-1]["GEN"]
                        for role in ("MATERIAL", "CALCULATOR")))

                def progress(raw):
                    return {role: {field: numbers(raw, f"WINDOW:{role}:{field}")[-1]
                                   for field in ("KEYS", "INPUT_ACKS")}
                            for role in ("MATERIAL", "CALCULATOR")}

                def edge(physical, down):
                    before = comp(serial(directory / "serial.txt"))["INPUT"]
                    control.command("input-send-event", {"events": [
                        {"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": physical}}}]})
                    observe("switcher edge " + physical, lambda raw: comp(raw).get("INPUT", 0) >= before + 1,
                            timeout_s=90)

                def shell(physical):
                    edge(physical, True); edge(physical, False)

                def focus_cycle():
                    raw = serial(directory / "serial.txt")
                    role = "CALCULATOR" if comp(raw)["FOCUSED"] else "MATERIAL"
                    before = progress(raw)[role]
                    key("f6")
                    observe("previous focus cancellation acknowledged", lambda raw:
                            progress(raw)[role]["KEYS"] == before["KEYS"]
                            and progress(raw)[role]["INPUT_ACKS"] == before["INPUT_ACKS"] + 1,
                            timeout_s=90)

                def open_switcher():
                    raw = serial(directory / "serial.txt")
                    role = "CALCULATOR" if comp(raw)["FOCUSED"] else "MATERIAL"
                    before = progress(raw)[role]
                    edge("alt", True)
                    observe("Alt delivered before shell ownership", lambda raw:
                            all(progress(raw)[role][field] == count + 1 for field, count in before.items()),
                            timeout_s=90)
                    before = progress(serial(directory / "serial.txt"))
                    shell("tab")
                    observe("switcher opening cancellations", lambda raw:
                            all(progress(raw)[role]["KEYS"] == old["KEYS"]
                                and progress(raw)[role]["INPUT_ACKS"] == old["INPUT_ACKS"] + 1
                                for role, old in before.items()), timeout_s=90)
                    return progress(serial(directory / "serial.txt"))

                def same_models(state, baseline):
                    fields = {"MATERIAL": ("PID", "DRAFT_BYTES", "DRAFT_SUM", "DIRTY", "SAVE_ACKS"),
                              "CALCULATOR": ("PID", "EXPRESSION_BYTES", "EXPRESSION_SUM",
                                             "EVALUATIONS", "FAILURES", "CARET", "CURRENT",
                                             "HAS_ANSWER", "ANSWER_LOW", "ANSWER_HIGH")}
                    return all(state[role][-1][field] == baseline[role][-1][field]
                               for role, names in fields.items() for field in names)

                move(width - 20, 50 * scale)
                wait("switcher-initial", complete)
                key("ret")
                wait("switcher-editor", lambda raw, s: complete(raw, s) and s["MATERIAL"][-1]["SCREEN"] == 2)
                for char in TEXT.decode():
                    key("spc" if char == " " else char)
                focus_cycle()
                for chord in (("2",), ("shift", "equal"), ("3",), ("ret",)):
                    key(*chord)
                focus_cycle()
                baseline = wait("switcher-work", lambda raw, s: complete(raw, s)
                                and s["MATERIAL"][-1]["DRAFT_SUM"] == TEXT_SUM
                                and s["MATERIAL"][-1]["DIRTY"] == 1
                                and s["CALCULATOR"][-1]["EVALUATIONS"] == 1
                                and s["CALCULATOR"][-1]["EXPRESSION_SUM"] == volume_disk.fnv(b"2+3")
                                and s["CALCULATOR"][-1]["ANSWER_LOW"] == 5_000_000_000)
                open_switcher()
                shell("esc"); edge("alt", False)
                wait("switcher-ready", lambda raw, s: complete(raw, s) and same_models(s, baseline))
                fixture = generate_switcher_fixture(directory, captured["switcher-ready"], scale)

                def switcher_wait(name, selected, focused, ledger):
                    return wait(name, lambda raw, s: complete(raw, s)
                                and same_models(s, baseline) and comp(raw).get("FOCUSED") == focused
                                and progress(raw) == ledger,
                                image_check=lambda image: switcher_matches(image, fixture["frames"][selected], scale))

                ledger = open_switcher()
                switcher_wait("switcher-preview", 1, 0, ledger)
                for physical in ("x", "f6", "delete"):
                    shell(physical)
                switcher_wait("switcher-key-isolated", 1, 0, ledger)
                shell("tab")
                switcher_wait("switcher-cycle", 0, 0, ledger)
                shell("left")
                switcher_wait("switcher-reverse", 1, 0, ledger)
                shell("esc"); edge("alt", False)
                wait("switcher-cancelled", lambda raw, s: same_models(s, baseline) and progress(raw) == ledger,
                     image_check=lambda image: image == captured["switcher-ready"])
                ledger = open_switcher()
                switcher_wait("switcher-commit-preview", 1, 0, ledger)
                edge("alt", False)
                ledger["MATERIAL"]["INPUT_ACKS"] += 1
                wait("switcher-committed", lambda raw, s: complete(raw, s) and same_models(s, baseline)
                     and comp(raw).get("FOCUSED") == 1 and progress(raw) == ledger)
                ledger = open_switcher()
                switcher_wait("switcher-mouse-preview", 0, 1, ledger)
                x, y, _, _ = switcher_bounds
                move(x + 32 * scale, y + 64 * scale); button(True)
                move(body[0] + 8 * scale, body[1] + 8 * scale); button(False)
                move(width - 20, 50 * scale); edge("alt", False)
                ledger["CALCULATOR"]["INPUT_ACKS"] += 1
                wait("switcher-mouse-committed", lambda raw, s: complete(raw, s) and same_models(s, baseline)
                     and comp(raw).get("FOCUSED") == 0 and progress(raw) == ledger,
                     image_check=lambda image: image == captured["switcher-ready"])
                ledger = open_switcher()
                switcher_wait("switcher-outside-preview", 1, 0, ledger)
                move(calc_body[0] + calc_body[2] - 4, calc_body[1] + calc_body[3] - 4)
                button(True); button(False); move(width - 20, 50 * scale); edge("alt", False)
                wait("switcher-outside-cancelled", lambda raw, s: complete(raw, s)
                     and same_models(s, baseline) and comp(raw).get("FOCUSED") == 0 and progress(raw) == ledger,
                     image_check=lambda image: image == captured["switcher-ready"])
                key("z")
                ledger["MATERIAL"]["KEYS"] += 2
                ledger["MATERIAL"]["INPUT_ACKS"] += 2
                wait("switcher-real-input-restored", lambda raw, s: complete(raw, s) and progress(raw) == ledger
                     and s["MATERIAL"][-1]["DRAFT_SUM"] == volume_disk.fnv(TEXT + b"z")
                     and s["MATERIAL"][-1]["DRAFT_BYTES"] == len(TEXT) + 1
                     and s["CALCULATOR"][-1]["EXPRESSION_SUM"] == volume_disk.fnv(b"2+3"))
                report["switcher_scope"] = "Live app previews, deferred focus, key/mouse isolation, cancel/commit and resumed real input; not the complete window-session/recovery/storage gate."
                key("f10")
            else:
                move(1260, 50)
                initial = wait("initial", lambda raw, s: all(s.values()), "initial")
                initial_comp = comp(serial(directory / "serial.txt"))
                if any(initial[r][-1]["PROBES"] != 3 for r in initial):
                    raise ValueError("Missing initial real CPL3 isolation refusals")
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
                if clipboard:
                    key("ctrl", "a"); key("ctrl", "c")
                    wait("clipboard-copied", lambda raw, s: s["MATERIAL"][-1]["DRAFT_SUM"] == TEXT_SUM, "clipboard-copied")
                    key("f6"); key("ctrl", "v")
                    wait("clipboard-rejected", lambda raw, s: s["CALCULATOR"][-1]["EXPRESSION_BYTES"] == 0 and s["CALCULATOR"][-1]["EVALUATIONS"] == 0, "clipboard-unsupported")
                    key("f6"); key("3"); key("ctrl", "a"); key("ctrl", "c")
                    wait("clipboard-numeric", lambda raw, s: s["MATERIAL"][-1]["DRAFT_SUM"] == volume_disk.fnv(b"3"), "clipboard-numeric")
                    key("f6"); key("ctrl", "v")
                    wait("clipboard-pasted", lambda raw, s: s["CALCULATOR"][-1]["EXPRESSION_BYTES"] == 1 and s["CALCULATOR"][-1]["EXPRESSION_SUM"] == volume_disk.fnv(b"3") and s["CALCULATOR"][-1]["EVALUATIONS"] == 0, "clipboard-pasted")
                    key("shift", "equal"); key("4"); key("ctrl", "a"); key("ctrl", "c")
                    wait("clipboard-calculator-copied", lambda raw, s: s["CALCULATOR"][-1]["EXPRESSION_SUM"] == volume_disk.fnv(b"3+4"), "clipboard-calculator-copied")
                    key("ctrl", "a"); key("backspace")
                    key("f6"); key("ctrl", "v")
                    wait("clipboard-back", lambda raw, s: s["MATERIAL"][-1]["DRAFT_BYTES"] == 3 and s["MATERIAL"][-1]["DRAFT_SUM"] == volume_disk.fnv(b"3+4"), "clipboard-back")
                    key("ctrl", "shift", "delete"); key("ctrl", "v")
                    wait("clipboard-empty", lambda raw, s: s["MATERIAL"][-1]["DRAFT_SUM"] == volume_disk.fnv(b"3+4"), "clipboard-empty")
                    # Populate again before the genuine fault, then restore the
                    # original workload; recovery must clear even this text.
                    key("ctrl", "a"); key("ctrl", "c"); key("ctrl", "a")
                    for char in TEXT.decode():
                        key("spc" if char == " " else char)
                    wait("clipboard-reset", lambda raw, s: s["MATERIAL"][-1]["DRAFT_SUM"] == TEXT_SUM, "dirty")
                key("f6")
                for keys in (("2",), ("shift", "equal"), ("3",), ("ret",)):
                    key(*keys)
                calculated = wait("calculated", lambda raw, s: s["CALCULATOR"] and s["CALCULATOR"][-1].get("HAS_ANSWER") == 1 and s["CALCULATOR"][-1].get("ANSWER_LOW") == 5_000_000_000 and s["CALCULATOR"][-1].get("ANSWER_HIGH") == 0)
                calc = calculated["CALCULATOR"][-1]
                if any(calc[field] != value for field,value in {"EXPRESSION_BYTES":3,"EXPRESSION_SUM":volume_disk.fnv(b"2+3"),"EVALUATIONS":1,"CURRENT":1,"CARET":3,"FAILURES":0}.items()):
                    raise ValueError("Actual calculator model differs from typed 2+3")
                move(1060, 118); button(True); move(900, 100); button(False)
                wait("moved", lambda raw, s: comp(raw).get("MOVES", 0) > 0,
                     image_check=lambda raw: crop(captured["calculated"], (932, 140, 320, 484)) == crop(raw, (772, 122, 320, 484)))
                key("f6")
                key("f12")
                recovered = wait("recovered", lambda raw, s: 1 in numbers(raw, "WINDOW:RECOVERIES") and all(s.values()) and all(s[r][-1]["SERVICE_GENERATION"] == 2 and s[r][-1]["PROBES"] == 10 for r in s) and comp(raw).get("GENERATION") == 2 and comp(raw).get("MATERIAL") == s["MATERIAL"][-1]["GEN"] and comp(raw).get("CALCULATOR") == s["CALCULATOR"][-1]["GEN"])
                recovered_comp = comp(serial(directory / "serial.txt"))
                if recovered_comp["PID"] == initial_comp["PID"]:
                    raise ValueError("Recovery reused the compositor process")
                for field in ("PID", "DRAFT_BYTES", "DRAFT_SUM", "DIRTY", "SAVE_ACKS", "OBJECT"):
                    if recovered["MATERIAL"][-1][field] != dirty["MATERIAL"][-1][field]:
                        raise ValueError("Recovery changed actual unsaved work: " + field)
                for field in ("PID", "EXPRESSION_BYTES", "EXPRESSION_SUM", "ANSWER_LOW", "ANSWER_HIGH", "HAS_ANSWER", "EVALUATIONS", "CARET", "CURRENT", "FAILURES"):
                    if recovered["CALCULATOR"][-1][field] != calculated["CALCULATOR"][-1][field]:
                        raise ValueError("Recovery changed actual calculation: " + field)
                for role in recovered:
                    if any(recovered[role][-1][f] <= initial[role][-1][f] for f in ("SURFACE_EPOCH", "INPUT_EPOCH")):
                        raise ValueError("Recovery reused presentation epoch")
                    previous = dirty if role == "MATERIAL" else calculated
                    if recovered[role][-1]["FRAMES"] <= previous[role][-1]["FRAMES"]:
                        raise ValueError("Recovery did not complete a new app frame")
                group_generation[0] = 2
                pointer[:] = [0, 0]
                move(1260, 50)
                wait("rebound-editor", lambda raw, s: s["MATERIAL"][-1]["SERVICE_GENERATION"] == 2, "dirty-rebound")
                select_title(1060, 118, 1)
                move(1260, 50)
                key("ret")
                wait("calculator-restored", lambda raw, s: s["CALCULATOR"][-1]["EVALUATIONS"] == calculated["CALCULATOR"][-1]["EVALUATIONS"]+1 and s["CALCULATOR"][-1]["ANSWER_LOW"] == 5_000_000_000 and comp(raw).get("CALCULATOR") == s["CALCULATOR"][-1]["GEN"],
                     image_check=lambda raw: crop(captured["calculated"], (932, 140, 320, 484)) == crop(raw, (932, 140, 320, 484)))
                select_title(300, 90, 0)
                move(1260, 50)
                key("right")
                wait("restored-editor", lambda raw, s: s["MATERIAL"][-1]["SERVICE_GENERATION"] == 2, "dirty-recovered")
                if clipboard:
                    key("ctrl", "v")
                    wait("clipboard-recovered-empty", lambda raw, s: s["MATERIAL"][-1]["DRAFT_SUM"] == TEXT_SUM, "clipboard-recovered-empty")
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
            if diagnostic:
                if process.returncode != 0:
                    raise ValueError("Smoke QEMU quit failed")
            else:
                if process.returncode != 33 or not raw.endswith(b"WINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n"):
                    raise ValueError("Missing actual cleanup/normal exit")
                if switcher:
                    if numbers(raw, "WINDOW:FAULT_SLOT") or numbers(raw, "WINDOW:RESTARTS") != [0]:
                        raise ValueError("Unexpected fault/restart in switcher part gate")
                elif numbers(raw, "WINDOW:FAULT_SLOT") != [4] or numbers(raw, "WINDOW:FAULT_VECTOR") != [6] or numbers(raw, "WINDOW:RESTARTS") != [1]:
                    raise ValueError("Wrong actual fault/restart")
                if not switcher:
                    report["disk_oracle"] = disk_oracle(private_path(directory / "volume.img", regular=True).read_bytes())
            if source_hash() != frozen or validate_build(frozen, scale) != manifest or sha256(qemu) != qemu_hash:
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
            report.update(passed=True, native_gate=not diagnostic and not switcher, switcher_gate=switcher,
                          clipboard_gate=clipboard and not diagnostic, duration_s=time.monotonic() - start,
                          serial_sha256=sha256(directory / "serial.txt"), qmp_trace=control.trace)
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    finally:
        if process is not None and process.poll() is None:
            # A guest FAIL marker can reach the serial file just before QEMU
            # completes its debug exit. Preserve that exit when available.
            try:
                process.wait(timeout=0.25)
            except subprocess.TimeoutExpired:
                report["harness_terminated_process"] = True
                process.kill(); process.wait(timeout=5)
        if process is not None:
            report["raw_qemu_exit"] = process.returncode
        report["duration_s"] = time.monotonic() - start
        if control is not None:
            report["qmp_trace"] = control.trace
            control.close()
        serial_path = directory / "serial.txt"
        if serial_path.exists() and serial_path.stat().st_size <= SERIAL_LIMIT:
            report["serial_sha256"] = sha256(private_path(serial_path, regular=True))
        if fixture is not None:
            report["oracle"] = fixture["metadata"]
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
    parser.add_argument("--recovery-handshake-only", action="store_true")
    parser.add_argument("--clipboard", action="store_true", help="Include genuine bounded clipboard exchange, clear and post-fault empty checks")
    parser.add_argument("--switcher", action="store_true", help="Separate window-switcher interaction/pixel part gate, not full session acceptance")
    parser.add_argument("--scale", type=int, choices=(1, 2), default=1, help="Switcher profile: 1=100%%, 2=200%%; default session stays 100%%")
    args = parser.parse_args()
    raise SystemExit(0 if run(timeout=args.timeout, bios_rom=args.bios_rom, vga_rom=args.vga_rom,
                            smoke=args.smoke_only, recovery_handshake=args.recovery_handshake_only,
                            clipboard=args.clipboard, switcher=args.switcher, scale=args.scale)["passed"] else 1)
