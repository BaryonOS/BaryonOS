#!/usr/bin/env python3
"""Recheck the specific DEV1 build-isolation repair; never starts a VM."""
from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import tomllib

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
NAMES = ("__kernel_start", "__text_end", "__rodata_end", "__kernel_end")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def check_flags(actual: list[str], expected: list[str]) -> None:
    require(actual == expected, "Kernel rustflags are not exactly one source config sequence")


def check_bounds(table: str) -> dict[str, int]:
    rows = [line.split() for line in table.splitlines()
            if line.split() and line.split()[-1] in NAMES]
    require(len(rows) == len(NAMES), "Missing or duplicate kernel boundary symbols")
    bounds = {row[-1]: int(row[0], 16) for row in rows}
    require(set(bounds) == set(NAMES), "Wrong kernel boundary symbol set")
    start, text, rodata, end = (bounds[name] for name in NAMES)
    require(start == 0x02000000 < text <= rodata < end <= 0x03000000,
            "Kernel boundary symbols do not enclose ordered loaded sections")
    require(all(value % 4096 == 0 for value in bounds.values()),
            "Kernel boundary symbols are not page aligned")
    return bounds


def main() -> None:
    folder = HERE / "isolated-build-1"
    freeze = json.loads((folder / "build-freeze.json").read_text())
    source = Path(freeze["source_worktree"])
    require(source.is_absolute() and source.is_dir(), "Frozen source checkout is unavailable")
    sys.path.insert(0, str(source / "tools"))
    import common
    import package_windows_qemu as package
    import run_materiaal_probe as probe

    require(common.ROOT == source, "Imported tooling is not the frozen source")
    require(common.source_hash() == freeze["source_sha256"], "Frozen source fingerprint changed")
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=source, text=True).strip()
    require(commit == freeze["source_commit"], "Frozen source commit changed")
    require(not subprocess.check_output(["git", "status", "--porcelain"],
                                        cwd=source, text=True).strip(),
            "Frozen source checkout has uncommitted changes")
    config = tomllib.loads((source / ".cargo/config.toml").read_text())
    expected = config["target"]["x86_64-unknown-none"]["rustflags"]
    apps = {}
    checked = {}
    target = source / "target/dev1/x86_64-unknown-none/release/.fingerprint"
    for role, details in freeze["builds"].items():
        build = Path(details["directory"])
        require(build == source / f"build/dev1-{role}", "Unexpected frozen build directory")
        for key, name in (("manifest", "manifest.json"), ("kernel", "esp/kernel.elf"),
                          ("loader", "esp/EFI/BOOT/BOOTX64.EFI")):
            require(common.sha256(build / name) == details[key + "_sha256"],
                    f"Frozen {role} {key} bytes changed")
        manifest = json.loads((build / "manifest.json").read_text())
        require((build / "manifest.json").read_bytes()
                == (folder / f"{role}-manifest.json").read_bytes(),
                f"Archived {role} manifest differs")
        if role == "reference":
            probe.validate_build(build, freeze["source_sha256"], "normal")
        elif role == "ordinary":
            package.validate_build(build)
        else:
            raise ValueError("Unexpected frozen build role")
        apps[role] = manifest["materiaal"]["apps"]
        fingerprint = json.loads((folder / f"{role}-kernel-fingerprint.json").read_text())
        check_flags(fingerprint["rustflags"], expected)
        require(any(json.loads(path.read_text()) == fingerprint
                    for path in target.glob("os-kernel-*/bin-os-kernel.json")),
                f"Archived {role} compiler fingerprint no longer exists in target cache")
        table = subprocess.check_output(
            ["objdump", "-t", "-C", str(build / "esp/kernel.elf")], text=True)
        bounds = check_bounds(table)
        require({name: hex(value) for name, value in bounds.items()}
                == details["boundary_symbols"], f"Frozen {role} boundaries changed")
        checked[role] = {"kernel_sha256": details["kernel_sha256"],
                         "boundary_symbols": details["boundary_symbols"]}
    require(len(apps["ordinary"]) == 7 and len(apps["reference"]) == 8
            and apps["reference"][:7] == apps["ordinary"],
            "Ordinary and reference app identities differ")
    for name, details in freeze["files"].items():
        require(common.sha256(Path(details["path"])) == details["sha256"],
                f"Frozen external input changed: {name}")

    red = HERE / "reference-ac4f038e86"
    for role in ("probe", "ordinary"):
        fingerprint = json.loads((red / f"{role}-kernel-fingerprint.json").read_text())
        require(fingerprint["rustflags"] == expected + expected,
                "Original red fingerprint is not the recorded duplicate")
        try:
            check_flags(fingerprint["rustflags"], expected)
        except ValueError:
            pass
        else:
            raise ValueError("Original duplicate rustflags were accepted")
    try:
        check_bounds((red / "bootstrap-boundary-symbols.txt").read_text())
    except ValueError:
        pass
    else:
        raise ValueError("Original collapsed kernel boundaries were accepted")
    print(json.dumps({"build_isolation_check_passed": True, "builds": checked,
                      "negative_original_duplicate_flags_rejected": 2,
                      "negative_original_collapsed_bounds_rejected": True,
                      "vm_started_by_this_check": False,
                      "native_acceptance_granted": False}, indent=2))


if __name__ == "__main__":
    main()
