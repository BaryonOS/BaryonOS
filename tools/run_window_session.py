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
import window_library

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
SNAPSHOT_FIELDS = {
    "MATERIAL": {"PID", "SERVICE_GENERATION", "SURFACE_EPOCH", "INPUT_EPOCH", "READY", "GEN", "FRAMES",
                 "SAVE_ACKS", "REVISION", "SAVED_BYTES", "SAVED_SUM", "SCREEN", "DIRTY", "DRAFT_BYTES", "DRAFT_SUM", "OBJECT", "SCALE", "ERRORS", "PROBES"},
    "CALCULATOR": {"PID", "SERVICE_GENERATION", "SURFACE_EPOCH", "INPUT_EPOCH", "READY", "GEN", "FRAMES",
                   "EVALUATIONS", "FAILURES", "EXPRESSION_BYTES", "EXPRESSION_SUM", "CURRENT", "CARET", "ACTIVE", "SCALE", "ERRORS", "ANSWER_LOW", "ANSWER_HIGH", "HAS_ANSWER", "PROBES"},
    "COMPOSITOR": {"GENERATION", "PID", "COMMITS", "INPUT", "REJECTED", "FOCUSED", "MOVES", "SWITCHES", "MATERIAL", "CALCULATOR", "FATAL", "POINTER_X", "POINTER_Y", "BUTTONS", "SHELL", "SHELL_READY"},
}


def generate_fixture(*, clipboard: bool = False, library: bool = False) -> dict:
    if clipboard and library:
        raise ValueError("Library and clipboard have separate complete oracles")
    directory = private_path(BUILD / ("window-fixture-" + uuid.uuid4().hex))
    directory.mkdir(parents=True)
    frozen = source_hash()
    env = environment()
    env["CARGO_TARGET_DIR"] = str(private_path(TARGET / "window-fixture"))
    env["OS_WINDOW_FIXTURE_DIRECTORY"] = str(directory)
    command = ["cargo", "+1.90.0", "test", "--offline", "--locked", "--manifest-path",
               str(ROOT / "ui/native/Cargo.toml"), "--test",
               "library_native_fixture" if library else "window_native_fixture", "--", "--nocapture"]
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
        for phase in (window_library.PHASES if library else FIXTURE_PHASES) + (CLIPBOARD_PHASES if clipboard else ()):
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
                    generator_commands=commands, clipboard_oracles=clipboard, library_oracles=library,
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


def validate_build(frozen: str, feature: str = FEATURE) -> dict:
    manifest = load_json(private_path(BUILD / "manifest.json", regular=True))
    if (feature not in (FEATURE, "native-session-route") or manifest.get("kernel_feature") != feature or manifest.get("loader_feature") != "desktop-mode"
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
        recovery_handshake: bool = False, clipboard: bool = False, session_route: bool = False,
        library: bool = False, library_parent: Path | None = None) -> dict:
    if not 30 <= timeout <= 1800:
        raise ValueError("Timeout must be bounded 30..1800 seconds")
    if smoke and recovery_handshake or clipboard and (smoke or recovery_handshake):
        raise ValueError("Clipboard requires the full native gate; choose one diagnostic mode")
    if session_route and (smoke or recovery_handshake or clipboard):
        raise ValueError("The public session route is separate from the fault/clipboard gate")
    if library and (smoke or recovery_handshake or clipboard or session_route) or library_parent is not None and not library:
        raise ValueError("Library requires its separate complete walkthrough")
    feature = "native-session-route" if session_route else FEATURE
    diagnostic = smoke or recovery_handshake or session_route or library
    private_path(BUILD)
    frozen = source_hash()
    manifest = validate_build(frozen, feature) if session_route else validate_build(frozen)
    fixture = generate_fixture(library=True) if library else (None if diagnostic else generate_fixture(clipboard=clipboard))
    parent = window_library.cold_parent(library_parent, source=frozen, manifest=manifest) if library_parent is not None else None
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
    if parent is None:
        volume_disk.create_image(directory / "volume.img", "empty")
    else:
        publish(directory / "volume.img", parent["image"])
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
    if parent is not None and (parent["qemu_sha256"] != qemu_hash or parent["command"] != command):
        raise ValueError("Cold library emulator or configuration differs from parent")
    start = time.monotonic()
    deadline = start + timeout
    report = dict(schema_version=1, passed=False, native_gate=False, smoke_only=diagnostic and not library,
                  recovery_handshake_only=recovery_handshake, clipboard_requested=clipboard, clipboard_gate=False, source_sha256=frozen,
                  source_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                  source_fingerprint_version=3, kernel_feature=feature, session_route_requested=session_route, session_route_gate=False, manifest=manifest, command=command,
                  library_requested=library, library_gate=False, library_cold=parent is not None,
                  library_parent=None if parent is None else {k: v for k, v in parent.items() if k != "image"},
                  run_directory=str(directory), qemu_sha256=qemu_hash, immutable=immutable, phases={})
    publish_json(directory / "result.json", report)
    report["boot_milestone_observations"] = {}
    def observed_serial():
        raw = serial(directory / "serial.txt")
        # First-observed host bounds, never invented guest phase durations.
        for label, marker in (
            ("firmware_exit", b"BOOT:EBS_OK\n"),
            ("kernel_entry", b"KERNEL:ENTRY\n"),
            ("kernel_memory_ready", b"KS-03:PASS\n"),
            ("process_manifest_ready", b"WINDOW:MANIFEST:PASS\n"),
            ("required_services_ready", b"WINDOW:INITMASK:00000000000000000031\n"),
            ("material_complete_frame", b"WINDOW:MATERIAL:READY:00000000000000000001\n"),
            ("calculator_complete_frame", b"WINDOW:CALCULATOR:READY:00000000000000000001\n"),
        ):
            if marker in raw and label not in report["boot_milestone_observations"]:
                report["boot_milestone_observations"][label] = {
                    "first_observed_seconds": time.monotonic() - start,
                    "scope": "host monotonic polling bound; not exact guest duration"}
        return raw
    process = control = None
    cpu_before = None
    try:
        with (directory / "stderr.txt").open("xb") as stderr:
            cpu_before = os.times() if os.name == "posix" else None
            process = subprocess.Popen(command, cwd=directory, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, bufsize=0)
            control = QmpPipe(process, deadline)

            implementation = Path(os.readlink(f"/proc/{process.pid}/exe"))
            implementation_hash = sha256(implementation)
            if parent is not None and parent["qemu_implementation_sha256"] != implementation_hash:
                raise ValueError("Cold library QEMU implementation differs from parent")
            report["qemu_implementation"] = {"path": str(implementation), "sha256": implementation_hash}
            captured = {}
            group_generation = [1]

            def comp(raw):
                values = snapshots(raw, "COMPOSITOR")
                return values[-1] if values else {}

            def observe(label, predicate):
                while True:
                    raw = observed_serial()
                    if predicate(raw):
                        return raw
                    if process.poll() is not None or time.monotonic() >= deadline:
                        raise TimeoutError("Native input acknowledgement missing: " + label)
                    time.sleep(0.02)

            action_started = [None]
            def wait(name, predicate, oracle=None, image_check=None):
                while True:
                    raw = observed_serial()
                    state = {role: snapshots(raw, role) for role in ("MATERIAL", "CALCULATOR")}
                    if predicate(raw, state):
                        path = private_path(directory / "candidate.ppm", regular=(directory / "candidate.ppm").exists())
                        control.command("screendump", {"filename": str(path)})
                        image = private_path(path, regular=True).read_bytes()
                        if ppm(image)[:2] != (1280, 720):
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
                raw = observed_serial()
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

            initial = None if recovery_handshake or session_route or library else wait("boot", lambda raw, s: all(s.values()) and 31 in numbers(raw, "WINDOW:INITMASK"))
            if library:
                observe("library app frames", lambda raw:
                    all(snapshots(raw, role) for role in ("MATERIAL", "CALCULATOR"))
                    and comp(raw).get("GENERATION") == 1)
                move(1260, 50)
                window_library.walkthrough(key=key, wait=wait, cold=parent is not None)
                key("f10")
            elif session_route:
                # These are real compositor/input states plus unmasked captured
                # frames. They do not certify a credential/keyring boundary or
                # the existing crash/storage/clipboard gate.
                def shell_page(name, page):
                    return wait(name, lambda raw, state: comp(raw).get("SHELL") == page
                                and comp(raw).get("SHELL_READY") == 1)

                def shell_key(*codes):
                    before = comp(observed_serial())["INPUT"]
                    control.command("send-key", {"keys": [{"type": "qcode", "data": k} for k in codes], "hold-time": 1})
                    observe("shell input ACK", lambda raw: comp(raw).get("INPUT", 0) > before)

                shell_page("lock", 0)
                observe("bounded app input counters", lambda raw: all(numbers(raw, f"WINDOW:{r}:KEYS") for r in ("MATERIAL", "CALCULATOR")))
                before = {r: numbers(observed_serial(), f"WINDOW:{r}:KEYS")[-1]
                          for r in ("MATERIAL", "CALCULATOR")}
                shell_key("ret"); shell_page("users", 1)
                shell_key("tab"); shell_key("ret"); shell_page("personal-auth", 2)
                shell_key("ret"); shell_key("tab"); shell_key("ret")
                shell_key("tab"); shell_key("ret")
                shell_page("hardware-unavailable", 2)
                raw = observed_serial()
                after = {r: numbers(raw, f"WINDOW:{r}:KEYS")[-1] for r in before}
                if before != after:
                    raise ValueError("Trusted shell leaked authentication input into an app")
                shell_key("esc"); shell_page("lock-again", 0)
                shell_key("ret"); shell_page("users-again", 1)
                shell_key("ret"); shell_page("guest-auth", 2)
                unlock_started = time.monotonic()
                shell_key("ret"); shell_page("desktop", 4)
                report["guest_to_desktop_observed_s"] = time.monotonic() - unlock_started
                shell_key("f3"); shell_page("search", 5)
                for code in ("r", "e", "k"): shell_key(code)
                shell_key("ret"); shell_page("calculator", 4)
                if comp(observed_serial()).get("FOCUSED") != 1:
                    raise ValueError("Search did not activate the correct native app")
                shell_key("compose"); shell_page("session-menu", 8)
                shell_key("tab"); shell_key("tab"); shell_key("ret"); shell_page("settings", 6)
                shell_key("esc"); shell_page("desktop-again", 4)
                shell_key("meta_l", "l"); shell_page("covered", 0)
                shell_key("ret"); shell_page("users-return", 1)
                shell_key("ret"); shell_page("guest-return", 2)
                shell_key("ret"); shell_page("desktop-return", 4)
                shell_key("compose"); shell_page("shutdown-menu", 8)
                for _ in range(3): shell_key("tab")
                control.command("send-key", {"keys": [{"type": "qcode", "data": "ret"}], "hold-time": 1})
                report["blocked_app_keys_before"] = before
                report["blocked_app_keys_after"] = after
            elif recovery_handshake:
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
            else:
                move(1260, 50)
                initial = wait("initial", lambda raw, s: all(s.values()), "initial")
                initial_comp = comp(observed_serial())
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
                recovered_comp = comp(observed_serial())
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
                key("ctrl", "f"); key("ret")
                wait("reopened", lambda raw, s: s["MATERIAL"] and s["MATERIAL"][-1]["SCREEN"] == 2 and s["MATERIAL"][-1]["DIRTY"] == 0 and s["MATERIAL"][-1]["DRAFT_SUM"] == TEXT_SUM, "reopened")
                key("f10")
            process.wait(timeout=max(0.1, deadline - time.monotonic()))
            raw = observed_serial()
            report["raw_qemu_exit"] = process.returncode
            if library:
                if process.returncode != 33 or not raw.endswith(b"WINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n"):
                    raise ValueError("Library did not shut down with verified cleanup")
                disk_path = private_path(directory / "volume.img", regular=True)
                report["disk_oracle"] = window_library.disk_oracle(disk_path.read_bytes())
                report["volume_sha256"] = sha256(disk_path)
                if parent is not None:
                    current_parent = window_library.cold_parent(library_parent, source=frozen, manifest=manifest)
                    if current_parent != parent or disk_path.read_bytes() != parent["image"]:
                        raise ValueError("Cold reopen changed the image or its parent evidence")
                report["library_gate"] = True
            elif session_route:
                if process.returncode != 33 or not raw.endswith(b"WINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n"):
                    raise ValueError("Public session route did not shut down with verified cleanup")
                report["session_route_gate"] = True
            elif diagnostic:
                if process.returncode != 0:
                    raise ValueError("Smoke QEMU quit failed")
            else:
                if process.returncode != 33 or not raw.endswith(b"WINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n"):
                    raise ValueError("Missing actual cleanup/normal exit")
                if numbers(raw, "WINDOW:FAULT_SLOT") != [4] or numbers(raw, "WINDOW:FAULT_VECTOR") != [6] or numbers(raw, "WINDOW:RESTARTS") != [1]:
                    raise ValueError("Wrong actual fault/restart")
                report["disk_oracle"] = disk_oracle(private_path(directory / "volume.img", regular=True).read_bytes())
            current_manifest = validate_build(frozen, feature) if session_route else validate_build(frozen)
            if source_hash() != frozen or current_manifest != manifest or sha256(qemu) != qemu_hash:
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
            report.update(passed=True, native_gate=not diagnostic, clipboard_gate=clipboard and not diagnostic, duration_s=time.monotonic() - start, serial_sha256=sha256(directory / "serial.txt"), qmp_trace=control.trace)
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
            if cpu_before is not None:
                cpu_after = os.times()
                report["qemu_host_cpu"] = {
                    "user_seconds": cpu_after.children_user - cpu_before.children_user,
                    "system_seconds": cpu_after.children_system - cpu_before.children_system,
                    "scope": "POSIX reaped child CPU during this QEMU invocation; excludes host fixture generation, not guest CPU cycles or scanout latency"}
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
    parser.add_argument("--session-route", action="store_true", help="Check the separate public VM compositor session route; no authentication/security-gate claim")
    parser.add_argument("--library", action="store_true", help="Native naming, collections and history; separate from the full compositor/clipboard gate")
    parser.add_argument("--library-cold-parent", type=Path, help="Copy only a validated successful library run's disposable image for cold reopening")
    args = parser.parse_args()
    raise SystemExit(0 if run(timeout=args.timeout, bios_rom=args.bios_rom, vga_rom=args.vga_rom,
                            smoke=args.smoke_only, recovery_handshake=args.recovery_handshake_only, clipboard=args.clipboard, session_route=args.session_route,
                            library=args.library, library_parent=args.library_cold_parent)["passed"] else 1)
