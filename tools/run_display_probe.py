#!/usr/bin/env python3
"""Actual CPL3 display/PS2 boundary probe in one isolated, hash-bound QEMU.

QMP is a hardware counterparty and framebuffer observer. It supplies no guest
input decoder, UI implementation, storage, networking or substitute success.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import time
import uuid

from common import (ROOT, BUILD, TARGET, environment, firmware, qemu_machine,
                    run_logged, sha256, source_hash)
from build_users import inspect_user
from run_fpu import private_path, publish, publish_json, load_json
from run_graphics import Qmp, QmpPipe, ppm

NAMES = ("display-probe", "input-probe", "display-intruder-probe")
ENV_NAMES = ("NOS_DISPLAY_PROBE_ELF", "NOS_INPUT_PROBE_ELF", "NOS_DISPLAY_INTRUDER_ELF")
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS")
END = ("DISPINPUT:DISPLAY:PASS", "DISPINPUT:INPUT:PASS", "DISPINPUT:ISOLATION:PASS", "DISPINPUT:CLEANUP:PASS", "M1:PASS")
FIELDS = ("BLITS", "PIXELS", "DISPLAY_DENIED", "CAP_DENIED", "POINTER_DENIED", "OUTPUT_UNCHANGED",
          "RESETS", "READY", "SAMPLES", "RESET_PENDING", "RESET_DISCARDED", "CLOSED", "OLD_DENIED",
          "EXITS", "SLOT_REUSES", "DISPLAY_EPOCH", "INPUT_EPOCH", "INPUT_TICKET", "TIMER_IRQS",
          "LIVE_PROCESSES", "FRAMES_RECOVERED")
INPUT_BYTES = (0x1e, 0x9e, 0x09, 0x07, 0x03, 0x08, 0, 0)
DEV_PROFILE = "display-input-dev-800x600"
DEV_WIDTH, DEV_HEIGHT = 800, 600
SERIAL_LIMIT = 128 * 1024
HEX = re.compile(r"[0-9a-f]{64}")


def require(ok, message):
    if not ok:
        raise ValueError(message)


def build_users() -> tuple[dict, dict]:
    """Shared builder hook: snapshot three actual inspected standalone ELF's."""
    private_path(BUILD)
    private_path(TARGET)
    output = private_path(BUILD / "users/display-probe")
    output.mkdir(parents=True, exist_ok=True)
    for name in (*[f"{name}.elf" for name in NAMES], "build-report.json"):
        private_path(output / name)
    for suffix in ("stdout.txt", "stderr.txt", "json"):
        private_path(BUILD / f"logs/build-display-probe-users.{suffix}")
    run_logged([sys.executable, "-B", str(ROOT / "sdk/examples/display-input-probe/build.py")],
               "build-display-probe-users", extra_env={"CARGO_TARGET_DIR": str(TARGET / "display-probe-apps")})
    apps = []
    for name in NAMES:
        source = private_path(TARGET / "display-probe-apps/x86_64-unknown-none/release" / name,
                              regular=True, cargo_output=True)
        binary = output / f"{name}.elf"
        publish(binary, source.read_bytes())
        apps.append({"name": name, "sha256": sha256(binary), "elf": inspect_user(binary)})
    artifact = {"passed": True, "native_runtime_performed": False, "apps": apps}
    publish_json(output / "build-report.json", artifact)
    return dict(zip(ENV_NAMES, (str(output / f"{name}.elf") for name in NAMES))), artifact


def validate_build(build: Path, frozen: str) -> dict:
    build = private_path(Path(build)).resolve(strict=True)
    require(isinstance(frozen, str) and HEX.fullmatch(frozen), "Malformed source fingerprint")
    manifest = load_json(private_path(build / "manifest.json", regular=True))
    require(isinstance(manifest, dict) and manifest.get("kernel_feature") == "native-display-probe"
            and manifest.get("loader_feature") == "framebuffer"
            and type(manifest.get("source_fingerprint_version")) is int
            and manifest["source_fingerprint_version"] == 3 and manifest.get("source_sha256") == frozen
            and manifest.get("sdk_runtime") is False and manifest.get("document_session") is False,
            "Build the paired native-display-probe/framebuffer profile on the current source first")
    artifact = load_json(private_path(build / "users/display-probe/build-report.json", regular=True))
    require(isinstance(artifact, dict) and set(artifact) == {"passed", "native_runtime_performed", "apps"}
            and artifact.get("passed") is True and artifact.get("native_runtime_performed") is False
            and isinstance(artifact.get("apps"), list) and len(artifact["apps"]) == 3
            and manifest.get("display_probe") == artifact, "Unbound three-ELF build report")
    kernel_bytes = private_path(build / "esp/kernel.elf", regular=True).read_bytes()
    for app, name in zip(artifact["apps"], NAMES):
        require(isinstance(app, dict) and set(app) == {"name", "sha256", "elf"} and app.get("name") == name,
                "Wrong or reordered display/input program")
        binary = private_path(build / f"users/display-probe/{name}.elf", regular=True)
        require(isinstance(app.get("sha256"), str) and HEX.fullmatch(app["sha256"])
                and sha256(binary) == app["sha256"] and inspect_user(binary) == app["elf"]
                and binary.read_bytes() in kernel_bytes, "ELF hash/inspection or embedded bytes mismatch")
    expected = {}
    for kind, name in (("kernel", "esp/kernel.elf"), ("loader", "esp/EFI/BOOT/BOOTX64.EFI")):
        value = manifest.get(kind)
        require(isinstance(value, dict) and isinstance(value.get("sha256"), str) and HEX.fullmatch(value["sha256"]),
                "Malformed kernel/loader identity")
        require(sha256(private_path(build / name, regular=True)) == value["sha256"], "Kernel/loader hash mismatch")
        expected[name] = value["sha256"]
    files = set()
    for path in (build / "esp").rglob("*"):
        private_path(path)
        if path.is_file():
            files.add(path.relative_to(build).as_posix())
    require(files == set(expected), "Unexpected ESP files")
    return {"manifest": manifest, "artifact": artifact, "expected": expected}


def numeric(line: str, name: str) -> int:
    match = re.fullmatch(re.escape("DISPINPUT:" + name + ":") + r"([0-9]{20})", line)
    require(match is not None and int(match[1]) <= (1 << 64) - 1, "Malformed native numeric field " + name)
    return int(match[1])


def validate_sequence(stdout: str) -> dict:
    require(isinstance(stdout, str) and len(stdout.encode("utf-8")) <= SERIAL_LIMIT
            and not any(token in stdout for token in ("FAIL", "PANIC", "\x00")), "Failed or oversized serial transcript")
    lines = stdout.splitlines()
    require(lines.count(BOOT[0]) == 1, "Missing/repeated boot handoff")
    start = lines.index(BOOT[0])
    require(not any(re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|DISPINPUT:)", line)
                    for line in lines[:start]), "Native marker before boot handoff")
    lines = lines[start:]
    require(tuple(lines[:8]) == BOOT + ("DISPINPUT:MANIFEST:PASS",), "Wrong native boot/manifest sequence")
    at = 8
    transcripts = []
    for generation in (1, 2):
        markers = lines[at:at + 2]
        require(len(markers) == 2 and set(markers) == {
            f"DISPINPUT:DISPLAY_READY:{generation:020d}", f"DISPINPUT:INPUT_READY:{generation:020d}"},
            "Missing/duplicate/wrong lifetime readiness")
        at += 2
        samples = []
        for index, byte in enumerate(INPUT_BYTES):
            require(at + 1 < len(lines), "Truncated raw native samples")
            sequence = numeric(lines[at], "SEQ")
            packed = numeric(lines[at + 1], "RAW")
            status = packed >> 8
            require(sequence == index + 1 and packed <= 0xffff and packed & 0xff == byte
                    and status & 0xc1 == 1 and bool(status & 0x20) == (index >= 2),
                    "Wrong actual input sequence/byte/status/channel")
            samples.append({"sequence": sequence, "status": status, "byte": byte})
            at += 2
        transcripts.append(samples)
        if generation == 1:
            require(at < len(lines) and lines[at] == "DISPINPUT:REUSE:PASS", "Missing actual slot reuse")
            at += 1
    require(len(lines) == at + len(FIELDS) + len(END), "Unknown/missing/duplicated native output")
    values = {name: numeric(line, name) for name, line in zip(FIELDS, lines[at:at + len(FIELDS)])}
    require(tuple(lines[at + len(FIELDS):]) == END, "Wrong final acceptance markers")
    fixed = {"BLITS": 2, "PIXELS": 512, "DISPLAY_DENIED": 22, "CAP_DENIED": 44,
             "RESETS": 2, "READY": 2, "SAMPLES": 16, "CLOSED": 4, "OLD_DENIED": 2,
             "EXITS": 6, "SLOT_REUSES": 3, "LIVE_PROCESSES": 0, "FRAMES_RECOVERED": 1}
    require(all(values[name] == value for name, value in fixed.items()), "Incomplete native lifecycle/refusal counters")
    require(18 <= values["POINTER_DENIED"] < 2_000_000
            and values["OUTPUT_UNCHANGED"] == values["POINTER_DENIED"] + 27
            and 2 <= values["RESET_PENDING"] <= 16384
            and values["RESET_DISCARDED"] <= 512
            and values["DISPLAY_EPOCH"] == 2 and values["INPUT_EPOCH"] == 2
            and values["INPUT_TICKET"] == 2 and 2 <= values["TIMER_IRQS"] < 10000,
            "Inconsistent bounds, generations or output-preservation counters")
    return {"counters": values, "raw_samples": transcripts}


def validate_frames(first: bytes, second: bytes) -> dict:
    frames = []
    for generation, raw in ((1, first), (2, second)):
        width, height, pixels = ppm(raw)
        require((width, height) == (DEV_WIDTH, DEV_HEIGHT), "Wrong development-profile screenshot geometry")
        require(not any(pixels[at:at + 3] == b"\xee\xee\xee" for at in range(0, len(pixels), 3)),
                "Rejected payload reached visible framebuffer")
        for tile_generation in range(1, generation + 1):
            left, top = (64, 64) if tile_generation == 1 else (112, 80)
            for y in range(8):
                for x in range(32):
                    at = ((top + y) * width + left + x) * 3
                    expected = bytes((0x20 + tile_generation * 0x20 + x * 3, 0x30 + y * 0x13, 0x70 + x + y))
                    require(pixels[at:at + 3] == expected, "Actual tile pixel differs from native contract")
        frames.append(pixels)
    old, new = frames
    # The second valid tile is the only permitted framebuffer change. This
    # checks addressing/stride and rejected blits outside either known tile.
    for y in range(DEV_HEIGHT):
        start = y * DEV_WIDTH * 3
        if 80 <= y < 88:
            left, right = start + 112 * 3, start + 144 * 3
            require(old[start:left] == new[start:left] and old[right:start + DEV_WIDTH * 3] == new[right:start + DEV_WIDTH * 3],
                    "Framebuffer changed outside second authorized rectangle")
        else:
            require(old[start:start + DEV_WIDTH * 3] == new[start:start + DEV_WIDTH * 3], "Framebuffer changed outside authorized rows")
    return {"development_profile": DEV_PROFILE, "width": DEV_WIDTH, "height": DEV_HEIGHT, "pixel_comparisons": 768, "unique_native_pixels": 512,
            "first_tile_preserved": True, "rejected_color_absent": True, "outside_second_tile_unchanged": True}


def qmp_requests(directory: Path) -> list[dict]:
    actions = [("qmp_capabilities", None)]
    down = [{"type": "rel", "data": {"axis": "x", "value": 7}},
            {"type": "rel", "data": {"axis": "y", "value": -3}},
            {"type": "btn", "data": {"down": True, "button": "left"}}]
    up = [{"type": "btn", "data": {"down": False, "button": "left"}}]
    for generation in (1, 2):
        actions.extend((("screendump", {"filename": str(directory / f"generation-{generation}.ppm")}),
                        ("send-key", {"keys": [{"type": "qcode", "data": "a"}], "hold-time": 30}),
                        ("input-send-event", {"events": down}), ("input-send-event", {"events": up})))
    return [{"execute": name, "id": index, **({"arguments": arguments} if arguments is not None else {})}
            for index, (name, arguments) in enumerate(actions, 1)]


def validate_qmp_trace(trace: object, directory: Path):
    expected = qmp_requests(directory)
    require(isinstance(trace, list) and len(trace) == len(expected), "Missing/extra QMP command")
    for actual, request in zip(trace, expected):
        require(isinstance(actual, dict) and set(actual) == {"request", "response"}
                and json.dumps(actual["request"], sort_keys=True, allow_nan=False)
                == json.dumps(request, sort_keys=True, allow_nan=False)
                and json.dumps(actual["response"], sort_keys=True, allow_nan=False)
                == json.dumps({"return": {}, "id": request["id"]}, sort_keys=True, allow_nan=False),
                "Uncorrelated/failed/altered QMP hardware request")


def qemu_arguments(qemu: str, machine: str, transport: str) -> list[str]:
    require(machine == qemu_machine(machine) and transport in ("unix", "pipe"), "Invalid machine/QMP profile")
    return [qemu, "-machine", machine, "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M",
            "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "file:serial.txt" if transport == "pipe" else "stdio",
            "-nic", "none", "-no-reboot", "-device", "VGA,vgamem_mb=16,romfile=VGA.rom",
            "-qmp", "stdio" if transport == "pipe" else "unix:qmp.sock,server=on,wait=off",
            "-drive", "if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
            "-drive", "if=pflash,unit=1,format=raw,file=OVMF_VARS.fd",
            "-drive", "if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]


def external_file(path: Path, maximum: int) -> Path:
    path = Path(path).resolve(strict=True)
    metadata = path.stat()
    require(stat.S_ISREG(metadata.st_mode) and 0 < metadata.st_size <= maximum, "Invalid bounded external tool/firmware file")
    return path


def serial_bytes(directory: Path) -> bytes:
    with private_path(directory / "serial.txt", regular=True).open("rb") as stream:
        raw = stream.read(SERIAL_LIMIT + 1)
    require(len(raw) <= SERIAL_LIMIT, "Oversized serial evidence")
    return raw


def geometry_contract() -> dict:
    return {"path": "boot/loader/src/graphics.rs", "function": "prepare",
            "sha256": sha256(ROOT / "boot/loader/src/graphics.rs"), "width": DEV_WIDTH, "height": DEV_HEIGHT,
            "selection": "existing bounded QueryMode selection; GOP_800_600 on absence; no geometry fallback"}


def frame_bytes(directory: Path, generation: int) -> bytes:
    path = private_path(directory / f"generation-{generation}.ppm", regular=True)
    maximum = 1920 * 1080 * 3 + 128
    require(path.stat().st_size <= maximum, "Oversized framebuffer artifact")
    with path.open("rb") as stream:
        raw = stream.read(maximum + 1)
    ppm(raw)
    return raw


def validate_execution(report: dict, bound: dict, directory: Path, qemu: str, machine: str, transport: str) -> dict:
    require(type(report.get("raw_qemu_exit")) is int and report["raw_qemu_exit"] == 33
            and report.get("timed_out") is False and report.get("command") == qemu_arguments(qemu, machine, transport)
            and report.get("cwd") == str(directory) and report.get("profile") == f"{machine}/qemu64/TCG/1CPU/256MiB/VGA/PS2"
            and report.get("development_profile") == DEV_PROFILE
            and report.get("loader_geometry_contract") == geometry_contract(),
            "VM exit/timeout/machine/CPU/RAM/device profile mismatch")
    frozen = report.get("frozen_artifacts")
    expected_names = set(bound["expected"]) | {"OVMF_CODE.fd", "OVMF_VARS.template.fd", "VGA.rom"}
    expected_names |= {f"inputs/{name}.elf" for name in NAMES} | {"inputs/build-report.json", "inputs/build-manifest.json"}
    require(isinstance(frozen, dict) and set(frozen) == expected_names, "Wrong frozen VM input set")
    for name, value in frozen.items():
        require(isinstance(value, str) and HEX.fullmatch(value)
                and sha256(private_path(directory / name, regular=True)) == value, "Frozen VM input changed")
    require(all(frozen[name] == value for name, value in bound["expected"].items()), "Wrong copied kernel/loader")
    require(load_json(private_path(directory / "inputs/build-manifest.json", regular=True)) == bound["manifest"]
            and load_json(private_path(directory / "inputs/build-report.json", regular=True)) == bound["artifact"],
            "Unbound build snapshots")
    require(all(frozen[f"inputs/{name}.elf"] == app["sha256"] for name, app in zip(NAMES, bound["artifact"]["apps"])),
            "Wrong userspace ELF snapshot")
    raw = serial_bytes(directory)
    require(raw.decode("utf-8", "replace") == report.get("stdout"), "Reported output differs from actual serial bytes")
    observed = validate_sequence(report["stdout"])
    trace = load_json(private_path(directory / "qmp-trace.json", regular=True))
    validate_qmp_trace(trace, directory)
    observed["framebuffer"] = validate_frames(*(frame_bytes(directory, i) for i in (1, 2)))
    return observed


def run_display_probe(*, build_directory=BUILD, run_root=None, machine=None,
                      qmp_transport="unix", vga_rom=None, vga_rom_sha256=None, timeout=60) -> dict:
    require(sys.platform == "linux", "Native display QMP counterpart currently requires Linux/procfs")
    require(type(timeout) in (int, float) and 1 <= timeout <= 120, "Invalid bounded VM timeout")
    machine = qemu_machine(machine)
    require(qmp_transport in ("unix", "pipe"), "Unsupported QMP transport")
    require(vga_rom is not None and (vga_rom_sha256 is None or isinstance(vga_rom_sha256, str) and HEX.fullmatch(vga_rom_sha256)),
            "Select an explicit existing VGA ROM; optional expected hash must be canonical SHA256")
    build = private_path(Path(build_directory)).resolve(strict=True)
    run_root = private_path(build / "run" if run_root is None else Path(run_root))
    require(run_root.resolve().is_relative_to(build), "Run outputs must stay inside the selected build directory")
    run_root.mkdir(parents=True, exist_ok=True)
    report_path = private_path(build / "display-probe-report.json")
    # Retire a prior pass before reading any new manifest/source. Invalid or
    # incomplete preparation must leave an explicit failed current attempt.
    publish_json(report_path, {"schema_version": 1, "passed": False, "stage": "preflight",
                               "build_directory": str(build)})
    frozen = source_hash()
    bound = validate_build(build, frozen)
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    require(qemu is not None, "Missing QEMU")
    qemu_entry = external_file(Path(qemu), 256 * 1024 * 1024)
    qemu_hash = sha256(qemu_entry)
    qemu_version = subprocess.check_output([qemu, "--version"], env=env, text=True, timeout=10).strip()
    require(qemu_version.startswith("QEMU emulator version "), "Unrecognized QEMU version")
    code, variables = firmware()
    firmware_sources = [(external_file(code, 16 * 1024 * 1024), "OVMF_CODE.fd"),
                        (external_file(variables, 16 * 1024 * 1024), "OVMF_VARS.template.fd"),
                        (external_file(vga_rom, 1024 * 1024), "VGA.rom")]
    directory = private_path(run_root / ("display-input-" + uuid.uuid4().hex[:10]))
    require(qmp_transport != "unix" or len(os.fsencode(directory / "qmp.sock")) < 104, "QMP socket path too long; use --qmp-transport pipe")
    directory.mkdir()
    (directory / "inputs").mkdir()
    inputs = {}
    for source, name in firmware_sources:
        publish(directory / name, source.read_bytes())
        inputs[name] = sha256(directory / name)
    if vga_rom_sha256 is not None:
        require(inputs["VGA.rom"] == vga_rom_sha256, "VGA ROM differs from explicit expected hash")
    publish(directory / "OVMF_VARS.fd", (directory / "OVMF_VARS.template.fd").read_bytes())
    for name, expected in bound["expected"].items():
        path = directory / name
        path.parent.mkdir(parents=True, exist_ok=True)
        publish(path, private_path(build / name, regular=True).read_bytes())
        require(sha256(path) == expected, "Kernel/loader changed during snapshot")
        path.chmod(0o444)
        inputs[name] = expected
    for name, app in zip(NAMES, bound["artifact"]["apps"]):
        path = directory / f"inputs/{name}.elf"
        publish(path, private_path(build / f"users/display-probe/{name}.elf", regular=True).read_bytes())
        require(sha256(path) == app["sha256"], "User ELF changed during snapshot")
        inputs[f"inputs/{name}.elf"] = app["sha256"]
    for name, value in (("build-report.json", bound["artifact"]), ("build-manifest.json", bound["manifest"])):
        publish_json(directory / "inputs" / name, value)
        inputs["inputs/" + name] = sha256(directory / "inputs" / name)
    arguments = qemu_arguments(qemu, machine, qmp_transport)
    checks = {"before_launch_source_unchanged": source_hash() == frozen}
    report = {"schema_version": 1, "scope": "actual own-kernel CPL3 display/PS2 boundary, two lifetimes; no desktop/inputbroker/physical-hardware acceptance",
              "passed": False, "checks": checks, "source_fingerprint_version": 3, "source_sha256": frozen,
              "base_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True, timeout=10).strip(),
              "command": arguments, "cwd": str(directory), "run_directory": str(directory), "build_directory": str(build),
              "machine": machine, "qmp_transport": qmp_transport, "profile": f"{machine}/qemu64/TCG/1CPU/256MiB/VGA/PS2",
              "development_profile": DEV_PROFILE,
              "loader_geometry_contract": geometry_contract(),
              "qemu_entry_path": qemu, "qemu_entry_sha256": qemu_hash, "qemu_version": qemu_version,
              "firmware_sources": {name: str(source) for source, name in firmware_sources},
              "frozen_artifacts": inputs, "raw_qemu_exit": None, "timed_out": False, "timeout_s": timeout}
    publish_json(report_path, report)
    publish_json(directory / "result.json", report)
    start = time.monotonic()
    deadline = start + timeout
    process = control = None
    trace = []
    try:
        require(checks["before_launch_source_unchanged"], "Source changed while snapshotting")
        with (directory / "serial.txt").open("xb") as stdout, (directory / "stderr.txt").open("xb") as stderr:
            process = subprocess.Popen(arguments, cwd=directory, env=env, stdout=subprocess.PIPE if qmp_transport == "pipe" else stdout,
                                       stdin=subprocess.PIPE if qmp_transport == "pipe" else subprocess.DEVNULL, stderr=stderr, bufsize=0)
            try:
                control = QmpPipe(process, deadline) if qmp_transport == "pipe" else Qmp(directory / "qmp.sock", deadline, process)
                actual_tool = external_file(Path(os.readlink(f"/proc/{process.pid}/exe")), 256 * 1024 * 1024)
                report["qemu_implementation_path"] = str(actual_tool)
                report["qemu_implementation_sha256"] = sha256(actual_tool)
                def wait_ready(generation):
                    expected = {f"DISPINPUT:DISPLAY_READY:{generation:020d}", f"DISPINPUT:INPUT_READY:{generation:020d}"}
                    while True:
                        text = serial_bytes(directory).decode("utf-8", "replace")
                        require(not any(word in text for word in ("FAIL", "PANIC")), "Guest failed before input stage")
                        lines = text.splitlines()
                        require(all(lines.count(marker) <= 1 for marker in expected), "Duplicate readiness marker")
                        if expected.issubset(lines):
                            return
                        require(time.monotonic() < deadline and process.poll() is None, "Guest did not reach actual display/input readiness")
                        time.sleep(0.01)
                requests = qmp_requests(directory)
                for generation in (1, 2):
                    wait_ready(generation)
                    for request in requests[1 + (generation - 1) * 4:1 + generation * 4]:
                        if request["execute"] == "screendump":
                            private_path(directory / f"generation-{generation}.ppm")
                        control.command(request["execute"], request.get("arguments"))
                        if request["execute"] == "screendump":
                            raw = frame_bytes(directory, generation)
                            require(ppm(raw)[:2] == (DEV_WIDTH, DEV_HEIGHT), "Wrong actual development-profile framebuffer geometry")
                        else:
                            remaining = deadline - time.monotonic()
                            require(remaining > 0, "Shared QMP deadline exceeded")
                            time.sleep(min(0.08, remaining))
                report["raw_qemu_exit"] = process.wait(timeout=max(0.01, deadline - time.monotonic()))
                if time.monotonic() >= deadline:
                    raise TimeoutError("Native run exceeded its shared deadline")
            finally:
                if control is not None:
                    trace = control.trace
                    control.close()
                if process.poll() is None:
                    process.kill()
                    process.wait(timeout=10)
                if report["raw_qemu_exit"] is None:
                    report["raw_qemu_exit"] = process.returncode
        publish_json(directory / "qmp-trace.json", trace)
        report["stdout"] = serial_bytes(directory).decode("utf-8", "replace")
        report["observed"] = validate_execution(report, bound, directory, qemu, machine, qmp_transport)
        checks["native_exit_transcript_qmp_pixels_and_snapshots"] = True
        checks["source_unchanged"] = source_hash() == frozen
        checks["qemu_entry_unchanged"] = sha256(qemu_entry) == qemu_hash
        checks["qemu_implementation_unchanged"] = sha256(actual_tool) == report["qemu_implementation_sha256"]
        validate_build(build, frozen)
        checks["build_artifacts_unchanged"] = True
        report["passed"] = all(checks.values())
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
        report["timed_out"] = isinstance(error, (TimeoutError, subprocess.TimeoutExpired)) or time.monotonic() >= deadline
        if (directory / "serial.txt").is_file():
            try:
                report["stdout"] = serial_bytes(directory).decode("utf-8", "replace")
            except (OSError, ValueError):
                pass
        publish_json(directory / "qmp-trace.json", trace)
    finally:
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=10)
        socket = directory / "qmp.sock"
        if socket.exists() and stat.S_ISSOCK(socket.lstat().st_mode):
            socket.unlink()
        report["duration_s"] = time.monotonic() - start
        report["artifact_sha256"] = {path.relative_to(directory).as_posix(): sha256(private_path(path, regular=True))
                                     for path in directory.rglob("*") if path.is_file() and path.name != "result.json"}
        publish_json(directory / "result.json", report)
        publish_json(report_path, report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-dir", type=Path, default=BUILD)
    parser.add_argument("--run-root", type=Path)
    parser.add_argument("--machine", choices=("pc-q35-10.0", "pc-q35-8.2", "pc-q35-7.2"))
    parser.add_argument("--qmp-transport", choices=("unix", "pipe"), default="unix")
    parser.add_argument("--vga-rom", type=Path, required=True)
    parser.add_argument("--vga-rom-sha256")
    args = parser.parse_args()
    result = run_display_probe(build_directory=args.build_dir, run_root=args.run_root, machine=args.machine,
                               qmp_transport=args.qmp_transport, vga_rom=args.vga_rom, vga_rom_sha256=args.vga_rom_sha256)
    print(result.get("stdout", ""), end="")
    print(f'Display/input raw={result["raw_qemu_exit"]}; passed={result["passed"]}; {result["run_directory"]}')
    if "error" in result:
        print(result["error"])
    raise SystemExit(0 if result["passed"] else 1)
