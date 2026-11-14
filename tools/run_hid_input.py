#!/usr/bin/env python3
"""Verify synthetic HID reports through native input-policy/client IPC in QEMU.

This observes the guest's explicit assertions, including cancellation and cleanup.
It provides no USB enumeration, physical-device or rendering evidence.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import stat

from common import BUILD, save_json, source_hash
from build_users import inspect_user
from run import is_success, run_guest

FEATURE = "native-hid-input-probe"
LOADER_FEATURE = "desktop-mode"
BOOT = (
    "BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS",
    "KS-02:PASS", "KS-03:PASS",
)
COUNTERS = (
    ("SOURCE_PUBLICATIONS", 11),
    ("SOURCE_REJECTIONS", 6),
    ("CLIENT_EVENTS", 10),
    ("CLIENT_CANCELS", 4),
    ("STALE_ACK", 1),
    ("ROLE_DENIAL", 1),
    ("POLICY_REJECTIONS", 2),
    ("POLICY_RECEIVE_RIGHTS", 15),
)
STAGES = (
    "HID:SYNTHETIC_REPORTS:ACTUAL_IPC",
    "HID:BOOT:PASS",
    "HID:KEYBOARD:PASS",
    "HID:MOUSE:PASS",
    "HID:FOCUS:PASS",
    "HID:RESET:PASS",
    "HID:DETACH:PASS",
    "HID:REATTACH:PASS",
    "HID:REJECT:PASS",
) + tuple(f"HID:{name}:{value:020d}" for name, value in COUNTERS) + (
    "HID:CLEANUP:PASS",
    "HID:CASE:PASS",
)
END = "M1:PASS"
FORBIDDEN = ("FAIL", "PANIC")
KERNEL = "esp/kernel.elf"
LOADER = "esp/EFI/BOOT/BOOTX64.EFI"
APP_NAMES = {
    "window-init", "window-compositor", "materiaal-app", "calculator-app",
    "keyring-service", "volume-service", "input-source", "input-policy",
    "input-hid-source", "window-hid-probe",
}
INPUT_APPS = ("input-hid-source", "window-hid-probe", "input-policy")
# Harness bounds, not performance or input-latency acceptance thresholds.
MAX_TIMEOUT = 120
SERIAL_LIMIT = 256 * 1024
SCOPE = "Synthetic HID through native kernel IPC; no USB, physical-device or rendering acceptance"


def evaluate(stdout: str, raw_exit: int | None, timed_out: bool) -> dict:
    if not isinstance(stdout, str) or len(stdout.encode("utf-8")) > SERIAL_LIMIT:
        raise ValueError("HID serial output is not a bounded text transcript")
    # Do not ask the shared helper to infer its M1-only profile. The native
    # service path may print M2/M3 diagnostics; END is checked exactly below.
    if (type(raw_exit) is not int or timed_out is not False
            or not is_success(raw_exit, stdout, timed_out, required_markers=BOOT + STAGES,
                              forbidden_markers=FORBIDDEN)):
        raise ValueError("HID boot, stage, timeout or raw-exit assertion failed")
    lines = stdout.splitlines()
    observed = tuple(line for line in lines
                     if line in BOOT or line.lstrip().startswith(("HID:", "M1:")))
    if observed != BOOT + STAGES + (END,) or lines[-1] != END:
        raise ValueError("Missing, duplicate, unknown, reordered or nonterminal HID evidence")
    return {
        "markers": list(STAGES + (END,)),
        "counts": dict(COUNTERS),
        "synthetic_reports": True,
        "native_ipc_assertions": True,
        "cancellation_stages": ["focus", "reset", "detach"],
        "usb_enumeration": False,
        "physical_hardware": False,
        "rendering": False,
    }


def _bounded_file(path: Path, maximum: int) -> bytes:
    info = path.lstat()
    if (not stat.S_ISREG(info.st_mode) or info.st_nlink != 1
            or not 0 < info.st_size <= maximum):
        raise ValueError(f"Expected a bounded regular file: {path}")
    data = path.read_bytes()
    if len(data) != info.st_size:
        raise ValueError(f"Input changed while being read: {path}")
    return data


def _unique_object(pairs: list) -> dict:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate manifest key: {key}")
        result[key] = value
    return result


def _digest(value: object) -> bool:
    return (isinstance(value, str) and len(value) == 64
            and all(character in "0123456789abcdef" for character in value))


def load_manifest(path: Path, frozen: str) -> tuple[dict, str]:
    raw = _bounded_file(path, 1 << 20)
    manifest = json.loads(raw, object_pairs_hook=_unique_object)
    if (not isinstance(manifest, dict)
            or manifest.get("kernel_feature") != FEATURE
            or manifest.get("loader_feature") != LOADER_FEATURE
            or type(manifest.get("source_fingerprint_version")) is not int
            or manifest["source_fingerprint_version"] != 3
            or not _digest(frozen) or manifest.get("source_sha256") != frozen):
        raise ValueError("Build the unchanged native-hid-input-probe/desktop-mode profile first")
    for name in ("kernel", "loader"):
        item = manifest.get(name)
        if not isinstance(item, dict) or not _digest(item.get("sha256")):
            raise ValueError(f"Missing canonical {name} artifact hash")
    return manifest, hashlib.sha256(raw).hexdigest()


def check_input_apps(manifest: dict) -> dict:
    """Bind the two synthetic actors and the unchanged production input policy.

    The window build also produces two unused legacy actors. They stay in the
    exact app inventory, but need not be embedded in this HID-specific kernel.
    Paths come from this build contract, never from manifest-supplied locations.
    """
    metadata = manifest.get("materiaal")
    if (not isinstance(metadata, dict) or metadata.get("passed") is not True
            or metadata.get("hid_synthetic_probe") is not True
            or metadata.get("probe_controller") is not False
            or metadata.get("native_runtime_performed") is not False):
        raise ValueError("Missing exact HID synthetic app build profile")
    apps = metadata.get("apps")
    if (not isinstance(apps, list) or len(apps) != len(APP_NAMES)
            or any(not isinstance(app, dict) or not isinstance(app.get("name"), str) for app in apps)
            or {app["name"] for app in apps} != APP_NAMES):
        raise ValueError("Wrong, duplicate or missing HID build app identity")
    kernel = _bounded_file(BUILD / KERNEL, 64 << 20)
    if hashlib.sha256(kernel).hexdigest() != manifest["kernel"]["sha256"]:
        raise ValueError("Source kernel differs from the inspected manifest")
    by_name = {app["name"]: app for app in apps}
    hashes = {}
    for name in INPUT_APPS:
        path = BUILD / "users/materiaal" / (name + ".elf")
        binary = _bounded_file(path, 16 << 20)
        digest = hashlib.sha256(binary).hexdigest()
        if (by_name[name] != {"name": name, "sha256": digest, "elf": inspect_user(path)}
                or binary not in kernel):
            raise ValueError(f"HID app hash, ELF metadata or kernel embedding differs: {name}")
        hashes[name] = digest
    return hashes


def run_probe(*, vga_rom: Path, timeout: int = 60) -> dict:
    destination = BUILD / "hid-input-report.json"
    destination.unlink(missing_ok=True)
    report = {"schema_version": 1, "passed": False, "scope": SCOPE}
    save_json(destination, report)
    try:
        if type(timeout) is not int or not 1 <= timeout <= MAX_TIMEOUT:
            raise ValueError(f"HID timeout must be an integer in [1,{MAX_TIMEOUT}]")
        frozen = source_hash()
        manifest_path = BUILD / "manifest.json"
        manifest, manifest_hash = load_manifest(manifest_path, frozen)
        app_hashes = check_input_apps(manifest)
        rom = Path(vga_rom)
        rom_hash = hashlib.sha256(_bounded_file(rom, 1 << 20)).hexdigest()
        report.update(source_sha256=frozen, source_fingerprint_version=3,
                      kernel_feature=FEATURE, loader_feature=LOADER_FEATURE,
                      manifest_sha256=manifest_hash, vga_rom_sha256=rom_hash,
                      input_app_sha256=app_hashes)
        save_json(destination, report)
        guest = run_guest(
            "hid-input", timeout=timeout, required_markers=BOOT + STAGES,
            forbidden_markers=FORBIDDEN, hid_input_probe=True, vga_rom=rom,
            expected_kernel_sha256=manifest["kernel"]["sha256"],
            expected_loader_sha256=manifest["loader"]["sha256"],
        )
        report["guest"] = guest
        save_json(destination, report)
        report["observed"] = evaluate(guest["stdout"], guest["raw_qemu_exit"], guest["timed_out"])
        artifacts = guest.get("artifact_sha256", {})
        if (guest.get("passed") is not True
                or guest.get("hid_input_probe") is not True
                or guest.get("artifact_binding_passed") is not True
                or type(guest.get("source_fingerprint_version")) is not int
                or guest.get("source_fingerprint_version") != 3
                or guest.get("source_sha256") != frozen
                or artifacts.get(KERNEL) != manifest["kernel"]["sha256"]
                or artifacts.get(LOADER) != manifest["loader"]["sha256"]
                or artifacts.get("VGA.rom") != rom_hash
                or not _digest(guest.get("volume_initial_sha256"))
                or not _digest(guest.get("volume_final_sha256"))
                or source_hash() != frozen
                or hashlib.sha256(_bounded_file(manifest_path, 1 << 20)).hexdigest() != manifest_hash
                or hashlib.sha256(_bounded_file(rom, 1 << 20)).hexdigest() != rom_hash
                or check_input_apps(manifest) != app_hashes):
            raise ValueError("HID source, manifest, ROM, profile or executed artifact binding changed")
    except Exception as error:
        report["error"] = str(error)
        save_json(destination, report)
        raise
    report["passed"] = True
    save_json(destination, report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vga-rom", type=Path, required=True,
                        help="Explicit bounded VGA ROM; copied into the disposable run directory")
    parser.add_argument("--timeout", type=int, default=60,
                        help=f"Harness deadline in seconds, at most {MAX_TIMEOUT} (default: 60)")
    arguments = parser.parse_args()
    run_probe(vga_rom=arguments.vga_rom, timeout=arguments.timeout)
    print(f"HID native IPC probe passed; {BUILD / 'hid-input-report.json'}")
