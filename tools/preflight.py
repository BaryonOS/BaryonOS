#!/usr/bin/env python3
"""Read-only inventory for the OS starter. Does not install tools or access devices."""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
from common import environment, firmware

ROOT = Path(__file__).resolve().parents[1]
PROBE_ENV = environment()
RUST_TOOLCHAIN = "1.90.0"
REQUIRED_TARGETS = ("x86_64-unknown-uefi", "x86_64-unknown-none")
REQUIRED_RLIBS = ("core", "alloc", "compiler_builtins")

def probe(name: str) -> dict:
    path = shutil.which(name, path=PROBE_ENV["PATH"])
    result = {"available": bool(path), "path": path, "version": None}
    if path:
        selector = ["+" + RUST_TOOLCHAIN] if name in ("rustc", "cargo") else []
        if selector:
            result["selected_toolchain"] = RUST_TOOLCHAIN
        try:
            completed = subprocess.run(
                [path, *selector, "--version"], capture_output=True, text=True,
                timeout=10, check=False, cwd=ROOT, env=PROBE_ENV,
            )
            lines = (completed.stdout or completed.stderr).splitlines()
            result["version"] = lines[0] if lines else None
            result["probe_exit"] = completed.returncode
        except (OSError, subprocess.TimeoutExpired) as exc:
            result["probe_error"] = type(exc).__name__
    return result


def usable(tool: dict) -> bool:
    return (tool.get("available", False) and tool.get("probe_exit", 0) == 0
            and "probe_error" not in tool)


def query(command: list[str]) -> dict:
    result = {"command": command}
    try:
        done = subprocess.run(command, capture_output=True, text=True, timeout=10,
                              check=False, cwd=ROOT, env=PROBE_ENV)
        result.update(exit_code=done.returncode, stdout=done.stdout, stderr=done.stderr)
    except (OSError, subprocess.TimeoutExpired) as error:
        result["error"] = type(error).__name__
    return result


def queried_directory(result: dict) -> Path:
    if result.get("exit_code") != 0:
        raise ValueError("compiler path query failed")
    lines = result["stdout"].splitlines()
    if len(lines) != 1 or not lines[0] or not Path(lines[0]).is_absolute():
        raise ValueError("compiler did not return one absolute directory")
    path = Path(lines[0]).resolve(strict=True)
    if not path.is_dir():
        raise ValueError("compiler path is not a directory")
    return path


def rlib_inventory(directory: Path, sysroot: Path, library: str) -> dict:
    """Inspect real archive files, not rustup's installed-target metadata.

    Header/extent inspection is inventory only: no member/ABI/link validation
    and no claim that a build has executed.
    """
    result = {"library": library, "available": False, "files": []}
    for path in sorted(directory.glob("lib" + library + "-*.rlib")):
        entry = {"path": str(path), "usable_archive": False}
        try:
            resolved = path.resolve(strict=True)
            if not resolved.is_relative_to(sysroot) or not resolved.is_file():
                raise ValueError("rlib is not a regular file within the selected sysroot")
            size = resolved.stat().st_size
            with resolved.open("rb") as stream:
                header = stream.read(68)
            # Traditional ar header followed by one complete member header.
            # Thin/external-member archives are intentionally not accepted.
            if (len(header) != 68 or header[:8] != b"!<arch>\n"
                    or header[66:68] != b"`\n"):
                raise ValueError("rlib lacks a complete regular archive/member header")
            member_size = header[56:66].strip()
            if not member_size.isdigit() or 68 + int(member_size) > size:
                raise ValueError("rlib first member extends beyond the file")
            entry.update(resolved_path=str(resolved), size_bytes=size, usable_archive=True)
            result["available"] = True
        except (OSError, ValueError, RuntimeError) as error:
            entry["error"] = str(error)
        result["files"].append(entry)
    return result


def rust_inventory(rustc: dict, rustup: dict) -> dict:
    result = {"toolchain": RUST_TOOLCHAIN, "compiler_valid": False,
              "sysroot": None, "targets": {}, "rustup_reported_targets": [],
              "scope": "compiler-selected directories and archive presence; no build or ABI validation"}
    if usable(rustup):
        manager = query([rustup["path"], "target", "list", "--installed",
                         "--toolchain", RUST_TOOLCHAIN])
        result["rustup_query"] = manager
        if manager.get("exit_code") == 0:
            result["rustup_reported_targets"] = manager["stdout"].splitlines()
    # The manager is optional. Its target list never substitutes for checking
    # the libraries selected by the actual pinned compiler.
    if not usable(rustc) or not rustc.get("path"):
        result["error"] = "pinned rustc probe unavailable or failed"
        return result
    if not re.match(r"^rustc 1\.90\.0(?:\s|$)", rustc.get("version") or ""):
        result["error"] = "selected compiler is not the pinned Rust 1.90.0"
        return result
    compiler = [rustc["path"], "+" + RUST_TOOLCHAIN]
    root_query = query([*compiler, "--print", "sysroot"])
    result["sysroot_query"] = root_query
    try:
        sysroot = queried_directory(root_query)
    except (OSError, ValueError, RuntimeError) as error:
        result["error"] = "invalid selected sysroot: " + str(error)
        return result
    result.update(compiler_valid=True, sysroot=str(sysroot))
    for target in REQUIRED_TARGETS:
        target_query = query([*compiler, "--print", "target-libdir", "--target", target])
        item = {"query": target_query, "available": False}
        try:
            directory = queried_directory(target_query)
            expected = (sysroot / "lib/rustlib" / target / "lib").resolve(strict=True)
            if directory != expected or not directory.is_relative_to(sysroot):
                raise ValueError("target library directory is outside its selected sysroot/target")
            libraries = {library: rlib_inventory(directory, sysroot, library)
                         for library in REQUIRED_RLIBS}
            item.update(directory=str(directory), libraries=libraries,
                        available=all(value["available"] for value in libraries.values()))
        except (OSError, ValueError, RuntimeError) as error:
            item["error"] = str(error)
        result["targets"][target] = item
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
    rust = rust_inventory(tools["rustc"], tools["rustup"])
    installed_targets = [target for target in REQUIRED_TARGETS
                         if rust["targets"].get(target, {}).get("available", False)]
    firmware_candidates = [str(f.resolve()) for f in known_firmware if f.is_file()]
    required = ("git", "rustc", "cargo", "qemu-system-x86_64")
    missing = [name for name in required if not usable(tools[name])]
    if not rust["compiler_valid"] and "rustc" not in missing:
        missing.append("Rust 1.90.0 compiler/sysroot")
    missing_targets = [target for target in REQUIRED_TARGETS if target not in installed_targets]
    if not firmware_candidates:
        missing.append("OVMF code firmware (supply --firmware if stored elsewhere)")
    report = {
        "schema_version": 2,
        "scope": "inventory only; no build, boot, acceleration or ABI validated",
        "workspace": str(ROOT),
        "host": {
            "system": platform.system(),
            "architecture": platform.machine(),
            "python": platform.python_version(),
        },
        "tools": tools,
        "rust_toolchain": rust,
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
