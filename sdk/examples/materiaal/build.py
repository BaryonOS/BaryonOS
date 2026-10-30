#!/usr/bin/env python3
"""Build only the isolated Materiaal executable; never boots or grants caps."""
from __future__ import annotations
import json
from pathlib import Path
import subprocess
import sys

EXAMPLE = Path(__file__).resolve().parent
ROOT = EXAMPLE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from common import environment as toolchain_environment

def main() -> int:
    environment = toolchain_environment()
    flags = ["-C", "code-model=small", "-C", "relocation-model=static",
             "-C", "no-redzone=yes", "-C", "linker=rust-lld",
             "-C", f"link-arg=-T{EXAMPLE / 'linker.ld'}", "-C", "link-arg=--no-pie",
             "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
             "-C", "target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2",
             f"--remap-path-prefix={ROOT}=/nieuw-os"]
    environment["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "build/sdk-materiaal"))
    command = ["cargo", "+1.90.0", "build", "--offline", "--locked", "--release",
               "--manifest-path", str(EXAMPLE / "Cargo.toml"), "--features", "native",
               "--bin", "materiaal-app", "--target", "x86_64-unknown-none"]
    print(json.dumps({"command": command, "target_dir": environment["CARGO_TARGET_DIR"]}), flush=True)
    return subprocess.run(command, cwd=ROOT, env=environment, check=False).returncode

if __name__ == "__main__":
    sys.exit(main())
