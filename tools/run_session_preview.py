#!/usr/bin/env python3
"""Verify rejected personal login and the guest UI through QEMU input and pixels.

The oracle is host-rendered Controller/Frame output, not native execution.
Only successful QMP interaction plus matching guest screendumps is native proof.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time
import uuid

from common import BUILD, ROOT, TARGET, environment, sha256, source_hash
from package_windows_qemu import load_json
from run_fpu import private_path, publish_json
from run_graphics import ppm
import run_materiaal_preview as preview

FEATURE = "native-system-session-preview"
ORACLE_KIND = "host-rendered Controller/Frame oracle; not native execution"
PHASES = (
    "accounts", "personal-login", "denied", "accounts-return",
    "guest-login", "desktop", "search", "material",
)
APP_RECT = (16, 48, 768, 536)
MAX_FRAME_BYTES = APP_RECT[2] * APP_RECT[3] * 3 + 128


def generate_fixture(build: Path) -> Path:
    """Run the repository's real Rust renderer on a separate bounded host test."""
    directory = private_path(build / ("session-fixture-" + uuid.uuid4().hex[:10]))
    directory.mkdir(parents=True)
    frozen = source_hash()
    command = [
        "cargo", "+1.90.0", "test", "--offline", "--locked", "--manifest-path",
        str(ROOT / "ui/native/Cargo.toml"), "--test", "desktop_native_fixture",
        "--", "--nocapture",
    ]
    env = environment()
    env["CARGO_TARGET_DIR"] = str(private_path(TARGET / "session-fixture"))
    env["OS_SESSION_FIXTURE_DIRECTORY"] = str(directory)
    with (directory / "generator.log").open("xb") as log:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=log,
                                stderr=subprocess.STDOUT, timeout=180, check=False)
    if result.returncode != 0:
        raise RuntimeError(f"Host session oracle failed: {directory / 'generator.log'}")
    if source_hash() != frozen:
        raise ValueError("Source changed while generating the host session oracle")
    frames = {}
    raw_frames = {}
    for name in PHASES:
        path = private_path(directory / f"{name}.ppm", regular=True)
        if path.stat().st_size > MAX_FRAME_BYTES:
            raise ValueError("Oversized host session frame")
        if ppm(path.read_bytes())[:2] != APP_RECT[2:]:
            raise ValueError("Host session frame has wrong app geometry")
        frames[name] = sha256(path)
        raw_frames[name] = path.read_bytes()
    validate_discriminating_frames(raw_frames)
    publish_json(directory / "fixture.json", {
        "schema_version": 1,
        "kind": ORACLE_KIND,
        "source_fingerprint_version": 3,
        "source_sha256": frozen,
        "kernel_feature": FEATURE,
        "app_rectangle": list(APP_RECT),
        "phases": list(PHASES),
        "frames_sha256": frames,
        "generator_command": command,
        "generator_exit": result.returncode,
    })
    return directory


def validate_discriminating_frames(frames: dict) -> None:
    """Every step must be observable and account return must restore the screen."""
    pixels = {name: ppm(raw)[2] for name, raw in frames.items()}
    if (any(pixels[left] == pixels[right] for left, right in zip(PHASES, PHASES[1:]))
            or pixels["denied"] == pixels["desktop"]
            or pixels["accounts"] != pixels["accounts-return"]):
        raise ValueError("Session oracle does not distinguish transitions or restore account return")


def load_fixture(directory: Path, manifest: dict, frozen: str) -> dict:
    directory = private_path(directory).resolve(strict=True)
    metadata_path = private_path(directory / "fixture.json", regular=True)
    metadata = load_json(metadata_path, "host session oracle", 64 * 1024)
    if (manifest.get("kernel_feature") != FEATURE
            or metadata.get("kernel_feature") != FEATURE
            or metadata.get("kind") != ORACLE_KIND
            or metadata.get("schema_version") != 1
            or type(metadata.get("schema_version")) is not int
            or metadata.get("source_fingerprint_version") != 3
            or type(metadata.get("source_fingerprint_version")) is not int
            or metadata.get("source_sha256") != frozen
            or manifest.get("source_sha256") != frozen
            or metadata.get("app_rectangle") != list(APP_RECT)
            or metadata.get("phases") != list(PHASES)
            or metadata.get("generator_exit") != 0
            or type(metadata.get("generator_exit")) is not int
            or not isinstance(metadata.get("frames_sha256"), dict)
            or set(metadata["frames_sha256"]) != set(PHASES)):
        raise ValueError("Host session oracle binding differs from the native build")
    frames = {}
    for name in PHASES:
        path = private_path(directory / f"{name}.ppm", regular=True)
        if path.stat().st_size > MAX_FRAME_BYTES:
            raise ValueError("Oversized host session oracle")
        raw = path.read_bytes()
        if (ppm(raw)[:2] != APP_RECT[2:]
                or hashlib.sha256(raw).hexdigest() != metadata["frames_sha256"][name]):
            raise ValueError(f"Host session oracle was changed: {name}")
        frames[name] = raw
    validate_discriminating_frames(frames)
    return {"directory": directory, "metadata": metadata,
            "metadata_sha256": sha256(metadata_path), "frames": frames}


def app_pixels(raw: bytes) -> bytes:
    width, height, pixels = ppm(raw)
    if (width, height) != (preview.WIDTH, preview.HEIGHT):
        raise ValueError("Native session screenshot has wrong framebuffer geometry")
    x, y, app_width, app_height = APP_RECT
    return b"".join(
        pixels[((y + row) * width + x) * 3:
               ((y + row) * width + x + app_width) * 3]
        for row in range(app_height)
    )


def matching_app_frame(raw: bytes, expected: bytes) -> bool:
    if ppm(expected)[:2] != APP_RECT[2:]:
        raise ValueError("Host session oracle has wrong app geometry")
    return app_pixels(raw) == ppm(expected)[2]


def input_groups() -> tuple:
    def key(name):
        return {"execute": "send-key",
                "arguments": {"keys": [{"type": "qcode", "data": name}], "hold-time": 30}}
    return (
        ("accounts", []),
        ("personal-login", [key(name) for name in ("tab", "ret", "t", "e", "s", "t")]),
        ("denied", [key("ret")]),
        ("accounts-return", [key("esc")]),
        ("guest-login", [key("ret")]),
        ("desktop", [key("ret")]),
        ("search", [key(name) for name in ("tab", "tab", "ret", "m", "a", "t")]),
        ("material", [key("ret")]),
    )


def run_walkthrough(control, process, directory: Path, fixture: dict, deadline: float) -> dict:
    if tuple(fixture.get("frames", {})) != PHASES:
        raise ValueError("Incomplete host session oracle")
    for raw in fixture["frames"].values():
        if ppm(raw)[:2] != APP_RECT[2:]:
            raise ValueError("Wrong host session oracle geometry")
    validate_discriminating_frames(fixture["frames"])
    stages = []
    for name, requests in input_groups():
        stage_start = time.monotonic()
        for request in requests:
            if not stages or process.poll() is not None or time.monotonic() >= deadline:
                raise TimeoutError("Session input requires the previous verified native frame")
            preview.verify_ready_transcript(preview.read_serial(directory / "serial.txt"))
            control.command(request["execute"], request["arguments"])
            time.sleep(min(0.08, max(0, deadline - time.monotonic())))
        path = directory / f"session-{name}.ppm"
        attempts = 0
        while True:
            if process.poll() is not None or time.monotonic() >= deadline:
                raise TimeoutError(f"Native session frame did not match host oracle: {name}")
            preview.verify_ready_transcript(preview.read_serial(directory / "serial.txt"))
            control.command("screendump", {"filename": str(path)})
            screenshot = private_path(path, regular=True)
            if screenshot.stat().st_size > preview.WIDTH * preview.HEIGHT * 3 + 128:
                raise ValueError("Oversized native session screenshot")
            raw = screenshot.read_bytes()
            attempts += 1
            if matching_app_frame(raw, fixture["frames"][name]):
                stages.append({
                    "phase": name, "duration_s": time.monotonic() - stage_start,
                    "screenshot_sha256": hashlib.sha256(raw).hexdigest(),
                    "app_pixels_sha256": hashlib.sha256(app_pixels(raw)).hexdigest(),
                    "oracle_sha256": hashlib.sha256(fixture["frames"][name]).hexdigest(),
                    "qmp_index": len(control.trace), "attempts": attempts,
                })
                break
            time.sleep(min(0.08, max(0, deadline - time.monotonic())))
    return {
        "stages": stages,
        "compared_rectangle": list(APP_RECT),
        "uncompared_regions": "Only framebuffer pixels outside the app-owned rectangle",
        "pixel_masks_inside_app": [],
        "input_route": (
            "accounts -> personal login with synthetic test input -> rejected/cleared "
            "-> accounts -> guest login -> desktop -> search mat -> Materiaal"
        ),
        "authentication": (
            "Personal password rejected; explicit disposable VM guest only; "
            "no authenticated user or key unlock"
        ),
        "host_oracle_is_native_evidence": False,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-dir", type=Path, default=BUILD)
    parser.add_argument("--bios-rom", type=Path, required=True)
    parser.add_argument("--vga-rom", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=240)
    parser.add_argument("--machine")
    parser.add_argument("--fixture-dir", type=Path)
    args = parser.parse_args()
    directory = args.fixture_dir or generate_fixture(args.build_dir)
    preview.run_preview(
        build_directory=args.build_dir, bios_rom=args.bios_rom, vga_rom=args.vga_rom,
        timeout=args.timeout, machine=args.machine, session_fixture_directory=directory,
    )


if __name__ == "__main__":
    main()
