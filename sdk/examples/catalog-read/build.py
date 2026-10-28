#!/usr/bin/env python3
"""Build the native example; this never boots or grants capabilities."""
from __future__ import annotations
import json
import os
from pathlib import Path
import subprocess
import sys

EXAMPLE = Path(__file__).resolve().parent
ROOT = EXAMPLE.parents[2]

def main() -> int:
    environment = os.environ.copy()
    # Explicit target flags prevent inheriting the parent kernel linker script.
    flags = ["-C", "code-model=small", "-C", "relocation-model=static",
             "-C", "no-redzone=yes", "-C", "linker=rust-lld",
             "-C", f"link-arg=-T{EXAMPLE / 'linker.ld'}", "-C", "link-arg=--no-pie",
             "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000"]
    environment["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "build/sdk-catalog-read"))
    command = ["cargo", "build", "--offline", "--locked", "--release",
               "--manifest-path", str(ROOT / "sdk/Cargo.toml"),
               "-p", "nieuw-os-catalog-read-example", "--features", "native",
               "--target", "x86_64-unknown-none"]
    print(json.dumps({"command": command, "target_dir": environment["CARGO_TARGET_DIR"]}), flush=True)
    return subprocess.run(command, cwd=ROOT, env=environment, check=False).returncode

if __name__ == "__main__":
    sys.exit(main())
