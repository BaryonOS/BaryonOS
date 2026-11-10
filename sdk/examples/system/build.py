#!/usr/bin/env python3
"""Build the native FR-I01 init, integration app and worker; this never boots or grants capabilities."""
from __future__ import annotations
import json
import argparse
from pathlib import Path
import subprocess
import sys

EXAMPLE = Path(__file__).resolve().parent
ROOT = EXAMPLE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from common import environment as toolchain_environment

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-clock", action="store_true")
    args = parser.parse_args()
    environment = toolchain_environment()
    # Explicit target flags prevent inheriting the parent kernel linker script.
    # The three --cfg flags force the portable integer backends of the pinned
    # RustCrypto crates: the ring-3 profile has CR0.EM=1 and no SSE/AVX state.
    flags = ["-C", "code-model=small", "-C", "relocation-model=static",
             "-C", "no-redzone=yes", "-C", "linker=rust-lld",
             "-C", f"link-arg=-T{EXAMPLE / 'linker.ld'}", "-C", "link-arg=--no-pie",
             "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
             "--cfg", "chacha20_force_soft", "--cfg", 'chacha20_backend="soft"', "--cfg", 'poly1305_backend="soft"',
             "--cfg", 'sha2_backend="soft"']
    environment["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "build/sdk-system"))
    features = "native,native-clock" if args.native_clock else "native"
    command = ["cargo", "build", "--offline", "--locked", "--release",
               "--manifest-path", str(ROOT / "sdk/Cargo.toml"),
               "-p", "nieuw-os-system-example", "--features", features,
               "--bins", "--target", "x86_64-unknown-none"]
    print(json.dumps({"command": command, "target_dir": environment["CARGO_TARGET_DIR"]}), flush=True)
    return subprocess.run(command, cwd=ROOT, env=environment, check=False).returncode

if __name__ == "__main__":
    sys.exit(main())
