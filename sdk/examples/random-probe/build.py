#!/usr/bin/env python3
"""Build the native RANDOM caller; this helper never generates random output."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import subprocess
import sys

EXAMPLE = Path(__file__).resolve().parent
ROOT = EXAMPLE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from common import environment


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("build", "clippy"), nargs="?", default="build")
    action = parser.parse_args().action
    env = environment()
    flags = ["-C", "code-model=small", "-C", "relocation-model=static",
             "-C", "no-redzone=yes", "-C", "linker=rust-lld",
             "-C", f"link-arg=-T{EXAMPLE / 'linker.ld'}", "-C", "link-arg=--no-pie",
             "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000"]
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "build/sdk-random-probe"))
    command = ["cargo", "+1.90.0", action, "--offline", "--locked", "--release",
               "--manifest-path", str(EXAMPLE / "Cargo.toml"), "--features", "native",
               "--bin", "random-probe", "--target", "x86_64-unknown-none"]
    if action == "clippy":
        command += ["--", "-D", "warnings"]
    print(json.dumps({"command": command, "target_dir": env["CARGO_TARGET_DIR"],
                      "scope": "native target ELF only, not a running RANDOM acceptance"}), flush=True)
    return subprocess.run(command, cwd=ROOT, env=env, check=False).returncode


if __name__ == "__main__":
    sys.exit(main())
