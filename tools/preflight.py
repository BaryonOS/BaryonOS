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
from common import environment, firmware

ROOT = Path(__file__).resolve().parents[1]
PROBE_ENV = environment()
RUST_TOOLCHAIN = "1.90.0"
REQUIRED_TARGETS = ("x86_64-unknown-uefi", "x86_64-unknown-none")
REQUIRED_RLIBS = ("core", "alloc", "compiler_builtins")
def host_test_tools(os_name: str) -> tuple[str, ...]:
    return ("link.exe",) if os_name == "nt" else ("cc",)


HOST_TEST_TOOLS = host_test_tools(os.name)


def probe(name: str) -> dict:
    path = shutil.which(name, path=PROBE_ENV["PATH"])
    result = {"available": bool(path), "path": path, "version": None}
    if not path:
        return result
    pinned = name in ("rustc", "cargo")
    candidates = (["+" + RUST_TOOLCHAIN], []) if pinned else ([],)
    version_args = ["/?"] if name == "link.exe" else ["--version"]
    result["probe_attempts"] = []
    if pinned:
        result["selected_toolchain"] = RUST_TOOLCHAIN
    for arguments in candidates:
        attempt = query([path, *arguments, *version_args])
        result["probe_attempts"].append(attempt)
        if "error" in attempt:
            result["probe_error"] = attempt["error"]
            return result
        lines = (attempt["stdout"] or attempt["stderr"]).splitlines()
        result.update(version=lines[0] if lines else None, probe_exit=attempt["exit_code"])
        if attempt["exit_code"] != 0:
            # A rejected selector can mean a direct standalone binary. A
            # crash/timeout is a failed tool probe, not a reason to hide it.
            if arguments and attempt["exit_code"] > 0:
                continue
            return result
        if pinned and not re.match(r"^" + re.escape(name + " " + RUST_TOOLCHAIN) + r"(?:\s|$)", result["version"] or ""):
            result["probe_error"] = "ToolchainVersionMismatch"
            return result
        if name == "rustc":
            # A standalone rustc can print --version while ignoring an input
            # named +1.90.0, then reject that input for --print sysroot. Verify
            # the actual compiler-query invocation before selecting its form.
            capability = query([path, *arguments, "--print", "sysroot"])
            attempt["sysroot_query"] = capability
            try:
                queried_directory(capability)
            except (OSError, ValueError, RuntimeError) as error:
                attempt["sysroot_error"] = str(error)
                if arguments and "error" not in capability and capability.get("exit_code", -1) >= 0:
                    continue
                result["probe_error"] = "InvalidCompilerSysroot"
                return result
        if pinned:
            result.update(toolchain_args=arguments,
                          invocation_mode="pinned-selector" if arguments else "verified-direct")
        return result
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


def rlib_inventory(directory: Path, target_root: Path, library: str) -> dict:
    """Inspect real archive files, not rustup's installed-target metadata.

    Header/extent inspection is inventory only: no member/ABI/link validation
    and no claim that a build has executed.
    """
    result = {"library": library, "available": False, "files": []}
    for path in sorted(directory.glob("lib" + library + "-*.rlib")):
        entry = {"path": str(path), "usable_archive": False}
        try:
            resolved = path.resolve(strict=True)
            if not resolved.is_relative_to(target_root) or not resolved.is_file():
                raise ValueError("rlib is not a regular file within the selected target")
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
    arguments = rustc.get("toolchain_args", ["+" + RUST_TOOLCHAIN])
    if arguments not in ([], ["+" + RUST_TOOLCHAIN]):
        result["error"] = "unsupported compiler invocation form"
        return result
    compiler = [rustc["path"], *arguments]
    result["compiler_command"] = compiler
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
            target_root = sysroot / "lib/rustlib" / target
            # Preserve the target identity before resolving a lib symlink:
            # resolving both paths first would accept uefi/lib -> none/lib.
            if target_root.resolve(strict=True) != target_root:
                raise ValueError("selected target directory redirects to another location")
            expected = (target_root / "lib").resolve(strict=True)
            if directory != expected or not directory.is_relative_to(target_root):
                raise ValueError("target library directory is outside its selected sysroot/target")
            libraries = {library: rlib_inventory(directory, target_root, library)
                         for library in REQUIRED_RLIBS}
            item.update(directory=str(directory), target_root=str(target_root), libraries=libraries,
                        available=all(value["available"] for value in libraries.values()))
        except (OSError, ValueError, RuntimeError) as error:
            item["error"] = str(error)
        result["targets"][target] = item
    return result

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--strict", action="store_true", help="Exit 2 if boot tools/firmware are missing.")
    parser.add_argument("--strict-host-tests", action="store_true",
                        help="Exit 2 if host test linker tools are missing.")
    parser.add_argument("--firmware", type=Path, help="Explicit OVMF code file, read-only.")
    args = parser.parse_args()
    tools = {name: probe(name) for name in (
        "git", "rustc", "cargo", "rustup", "qemu-system-x86_64",
        "ld.lld", "clang", "cc", "link.exe", "python3",
    )}
    known_firmware = [
        Path("/usr/share/OVMF/OVMF_CODE_4M.fd"),
        Path("/usr/share/OVMF/OVMF_CODE.fd"),
        Path("/usr/share/edk2/ovmf/OVMF_CODE.fd"),
        Path("/usr/share/qemu/OVMF_CODE.fd"),
    ]
    firmware_lookup_error = None
    if args.firmware:
        known_firmware.insert(0, args.firmware.expanduser())
    else:
        try:
            known_firmware.insert(0, firmware()[0])
        except (RuntimeError, OSError) as error:
            firmware_lookup_error = f"{type(error).__name__}: {error}"
    rust = rust_inventory(tools["rustc"], tools["rustup"])
    installed_targets = [target for target in REQUIRED_TARGETS
                         if rust["targets"].get(target, {}).get("available", False)]
    firmware_candidates = [str(f.resolve()) for f in known_firmware if f.is_file()]
    required = ("git", "rustc", "cargo", "qemu-system-x86_64")
    missing = [name for name in required if not usable(tools[name])]
    if not rust["compiler_valid"] and "rustc" not in missing:
        missing.append("Rust 1.90.0 compiler/sysroot")
    missing_targets = [target for target in REQUIRED_TARGETS if target not in installed_targets]
    missing_host_test_tools = [name for name in HOST_TEST_TOOLS if not usable(tools[name])]
    if not firmware_candidates:
        missing.append("OVMF code firmware (supply --firmware if stored elsewhere)")
    report = {
        "schema_version": 2,
        "scope": "inventory only; no build, boot, acceleration or ABI validated",
        "build_launcher_scope": "Shared builds still require a pinned launcher or compatible +1.90.0 wrapper; direct-tool inventory does not validate that build chain.",
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
        "firmware_lookup_error": firmware_lookup_error,
        "missing": missing,
        "host_test_tools": list(HOST_TEST_TOOLS),
        "missing_host_test_tools": missing_host_test_tools,
        "baseline_tools_present": not missing and not missing_targets,
        "next": "Compare versions against docs/implementation/BOOT-AND-TOOLCHAIN.md; then build and run the real smoke test.",
    }
    print(json.dumps(report, indent=2))
    boot_requirements_missing = bool(missing or missing_targets)
    host_test_requirements_missing = bool(missing_host_test_tools)
    return 2 if ((args.strict and boot_requirements_missing)
                 or (args.strict_host_tests and host_test_requirements_missing)) else 0

if __name__ == "__main__":
    raise SystemExit(main())
