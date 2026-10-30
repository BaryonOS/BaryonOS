#!/usr/bin/env python3
"""Build only the isolated calculator executable; never boots or grants caps."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import subprocess
import sys

EXAMPLE = Path(__file__).resolve().parent
ROOT = EXAMPLE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from common import environment as toolchain_environment, sha256
from build_users import inspect_user

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("build", "clippy"), nargs="?", default="build")
    action = parser.parse_args().action
    environment = toolchain_environment()
    flags = ["-C", "code-model=small", "-C", "relocation-model=static",
             "-C", "no-redzone=yes", "-C", "linker=rust-lld",
             "-C", f"link-arg=-T{EXAMPLE / 'linker.ld'}", "-C", "link-arg=--no-pie",
             "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
             "-C", "target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2"]
    environment["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "build/sdk-calculator"))
    command = ["cargo", "+1.90.0", action, "--offline", "--locked", "--release",
               "--manifest-path", str(EXAMPLE / "Cargo.toml"), "--features", "native",
               "--bin", "calculator-app", "--target", "x86_64-unknown-none"]
    if action == "clippy":
        command += ["--", "-D", "warnings"]
    print(json.dumps({"command": command, "target_dir": environment["CARGO_TARGET_DIR"]}), flush=True)
    result = subprocess.run(command, cwd=ROOT, env=environment, check=False)
    if result.returncode or action == "clippy":
        return result.returncode
    target = Path(environment["CARGO_TARGET_DIR"])
    binary = (target if target.is_absolute() else ROOT / target) / "x86_64-unknown-none/release/calculator-app"
    print(json.dumps({"native_runtime_performed": False,
                      "extra_env": {"NOS_CALCULATOR_ELF": str(binary.resolve())},
                      "calculator_app": {"sha256": sha256(binary), "elf": inspect_user(binary)}}), flush=True)
    return 0

if __name__ == "__main__":
    sys.exit(main())
