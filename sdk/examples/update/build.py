#!/usr/bin/env python3
"""Build the native B03 update service; this never boots, signs or grants capabilities."""
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
    # Explicit target flags prevent inheriting the parent kernel linker script.
    # The portable curve25519 backend avoids SIMD dispatch: the ring-3 profile
    # has CR0.EM=1 and no SSE/AVX state. sha2 is pinned to force-soft by update-core.
    flags = ["-C", "code-model=small", "-C", "relocation-model=static",
             "-C", "no-redzone=yes", "-C", "linker=rust-lld",
             "-C", f"link-arg=-T{EXAMPLE / 'linker.ld'}", "-C", "link-arg=--no-pie",
             "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
             "--cfg", 'curve25519_dalek_backend="serial"']
    environment["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "build/sdk-update"))
    command = ["cargo", "build", "--offline", "--locked", "--release",
               "--manifest-path", str(ROOT / "sdk/Cargo.toml"),
               "-p", "nieuw-os-update-example", "--features", "native",
               "--bins", "--target", "x86_64-unknown-none"]
    print(json.dumps({"command": command, "target_dir": environment["CARGO_TARGET_DIR"]}), flush=True)
    return subprocess.run(command, cwd=ROOT, env=environment, check=False).returncode

if __name__ == "__main__":
    sys.exit(main())
