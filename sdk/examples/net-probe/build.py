#!/usr/bin/env python3
"""Build the actual ring-3 network probe, never execute a host network client."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import subprocess
import sys

EXAMPLE = Path(__file__).resolve().parent
ROOT = EXAMPLE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from common import environment as toolchain_environment


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--link", action="store_true",
                        help="add the H04-T3 link-loss/reconnect phase (kernel test-system-net-link)")
    options = parser.parse_args()
    environment = toolchain_environment()
    flags = ["-C", "code-model=small", "-C", "relocation-model=static",
             "-C", "no-redzone=yes", "-C", "linker=rust-lld",
             "-C", f"link-arg=-T{EXAMPLE / 'linker.ld'}", "-C", "link-arg=--no-pie",
             "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000"]
    environment["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "build/sdk-net-probe"))
    command = ["cargo", "build", "--offline", "--locked", "--release",
               "--manifest-path", str(EXAMPLE / "Cargo.toml"), "--features", "native,link" if options.link else "native",
               "--bin", "net-probe", "--target", "x86_64-unknown-none"]
    print(json.dumps({"command": command, "target_dir": environment["CARGO_TARGET_DIR"]}), flush=True)
    return subprocess.run(command, cwd=ROOT, env=environment, check=False).returncode


if __name__ == "__main__":
    sys.exit(main())
