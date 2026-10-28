#!/usr/bin/env python3
"""Read-only inventory for the OS starter. Does not install tools or access devices."""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
from common import environment, firmware

ROOT = Path(__file__).resolve().parents[1]
PROBE_ENV = environment()

def probe(name: str) -> dict:
    path = shutil.which(name, path=PROBE_ENV["PATH"])
    result = {"available": bool(path), "path": path, "version": None}
    if path:
        try:
            completed = subprocess.run(
                [path, "--version"], capture_output=True, text=True,
                timeout=10, check=False, cwd=ROOT, env=PROBE_ENV,
            )
            lines = (completed.stdout or completed.stderr).splitlines()
            result["version"] = lines[0] if lines else None
            result["probe_exit"] = completed.returncode
        except (OSError, subprocess.TimeoutExpired) as exc:
            result["probe_error"] = type(exc).__name__
    return result

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--strict", action="store_true", help="Exit 2 if boot tools/firmware are missing.")
    parser.add_argument("--firmware", type=Path, help="Explicit OVMF code file, read-only.")
    args = parser.parse_args()
    tools = {name: probe(name) for name in (
        "git", "rustc", "cargo", "rustup", "qemu-system-x86_64",
        "ld.lld", "clang", "cc", "python3",
    )}
    known_firmware = [
        Path("/usr/share/OVMF/OVMF_CODE_4M.fd"),
        Path("/usr/share/OVMF/OVMF_CODE.fd"),
        Path("/usr/share/edk2/ovmf/OVMF_CODE.fd"),
        Path("/usr/share/qemu/OVMF_CODE.fd"),
    ]
    if args.firmware:
        known_firmware.insert(0, args.firmware.expanduser())
    else:
        try:
            known_firmware.insert(0, firmware()[0])
        except (RuntimeError, OSError):
            pass
    installed_targets = []
    if tools["rustup"]["available"]:
        try:
            done = subprocess.run(
                [tools["rustup"]["path"], "target", "list", "--installed"],
                capture_output=True, text=True, timeout=10, check=False, cwd=ROOT, env=PROBE_ENV,
            )
            if done.returncode == 0:
                installed_targets = done.stdout.splitlines()
        except (OSError, subprocess.TimeoutExpired):
            pass
    firmware_candidates = [str(f.resolve()) for f in known_firmware if f.is_file()]
    required = ("git", "rustc", "cargo", "rustup", "qemu-system-x86_64")
    missing = [name for name in required
               if not tools[name]["available"]
               or tools[name].get("probe_exit", 0) != 0
               or "probe_error" in tools[name]]
    required_targets = ("x86_64-unknown-uefi", "x86_64-unknown-none")
    missing_targets = [target for target in required_targets if target not in installed_targets]
    if not firmware_candidates:
        missing.append("OVMF code firmware (supply --firmware if stored elsewhere)")
    report = {
        "schema_version": 1,
        "scope": "inventory only; no build, boot, acceleration or ABI validated",
        "workspace": str(ROOT),
        "host": {
            "system": platform.system(),
            "architecture": platform.machine(),
            "python": platform.python_version(),
        },
        "tools": tools,
        "installed_rust_targets": installed_targets,
        "missing_rust_targets": missing_targets,
        "firmware_candidates": firmware_candidates,
        "missing": missing,
        "baseline_tools_present": not missing and not missing_targets,
        "next": "Compare versions against docs/implementation/BOOT-AND-TOOLCHAIN.md; then build and run the real smoke test.",
    }
    print(json.dumps(report, indent=2))
    return 2 if args.strict and (missing or missing_targets) else 0

if __name__ == "__main__":
    raise SystemExit(main())
