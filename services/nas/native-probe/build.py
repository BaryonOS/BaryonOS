#!/usr/bin/env python3
"""Build and inspect the local NAS acceptance ELFs; never boot or grant authority."""
from __future__ import annotations

import argparse
from pathlib import Path
import shutil
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from build_users import inspect_user
from common import run_logged, save_json, sha256


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target-dir", type=Path, default=ROOT / "target/nas-native")
    parser.add_argument("--output-dir", type=Path, default=ROOT / "build/nas-probe/users")
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    binaries = {}
    for binary, linker in (
        ("nas-probe", ROOT / "sdk/examples/system/linker.ld"),
        ("nas-volume", ROOT / "sdk/examples/volume-docs/linker.ld"),
    ):
        flags = [
            "-C", "code-model=small", "-C", "relocation-model=static",
            "-C", "no-redzone=yes", "-C", "linker=rust-lld",
            "-C", f"link-arg=-T{linker}", "-C", "link-arg=--no-pie",
            "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
        ]
        target = args.target_dir.resolve() / binary
        run_logged(
            ["cargo", "build", "--offline", "--locked", "--release",
             "--manifest-path", str(HERE / "Cargo.toml"), "--features", "native",
             "--bin", binary, "--target", "x86_64-unknown-none"],
            f"build-{binary}",
            extra_env={
                "CARGO_TARGET_DIR": str(target),
                "CARGO_ENCODED_RUSTFLAGS": "\x1f".join(flags),
            },
        )
        built = target / "x86_64-unknown-none/release" / binary
        plan = inspect_user(built)
        destination = args.output_dir.resolve() / f"{binary}.elf"
        shutil.copyfile(built, destination)
        binaries[binary] = {
            "path": str(destination),
            "bytes": destination.stat().st_size,
            "sha256": sha256(destination),
            "elf": plan,
            "mapped_elf_bytes": sum(s["page_end"] - s["page_start"] for s in plan["segments"]),
            "zero_fill_bytes": sum(s["memsz"] - s["filesz"] for s in plan["segments"]),
        }
    save_json(args.output_dir / "build-report.json", {
        "schema_version": 1,
        "gate": "crosscompile-and-elf-inspection-only",
        "native_execution": False,
        "binaries": binaries,
        "private_stack_reserved": 65536,
        "initial_private_heap_reserved": 65536,
        "workspace_native_peak": "pending native execution; host workspace size is recorded separately",
        "runtime_touched_stack_high_water": "startup word 448, pending native execution",
    })
    return 0


if __name__ == "__main__":
    sys.exit(main())
