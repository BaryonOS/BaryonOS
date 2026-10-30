#!/usr/bin/env python3
"""Build the real ring-3 memory executable with its explicitly pinned app compiler."""
from __future__ import annotations
import json
import os
from pathlib import Path
import subprocess
import sys

EXAMPLE = Path(__file__).resolve().parent
ROOT = EXAMPLE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from common import environment as toolchain_environment, sha256

TOOLCHAIN = "1.90.0"


def main() -> int:
    env = toolchain_environment()
    # common.environment intentionally keeps the kernel's 1.90 linker first.
    # Use the matching app linker explicitly, without changing the shell/default.
    version = subprocess.run(["rustc", f"+{TOOLCHAIN}", "-vV"], cwd=ROOT, env=env,
                             text=True, capture_output=True, check=True, timeout=15)
    if not version.stdout.startswith(f"rustc {TOOLCHAIN} "):
        raise ValueError("Unexpected memory app compiler")
    host = next(line.removeprefix("host: ") for line in version.stdout.splitlines()
                if line.startswith("host: "))
    sysroot = subprocess.run(["rustc", f"+{TOOLCHAIN}", "--print", "sysroot"], cwd=ROOT,
                             env=env, text=True, capture_output=True, check=True, timeout=15)
    linker = Path(sysroot.stdout.strip()) / "lib/rustlib" / host / "bin/rust-lld"
    if os.name == "nt":
        linker = linker.with_suffix(".exe")
    if not linker.is_file():
        raise ValueError("Matching pinned memory linker is missing")
    flags = ["-C", "code-model=small", "-C", "relocation-model=static",
             "-C", "no-redzone=yes", "-C", f"linker={linker}",
             "-C", f"link-arg=-T{EXAMPLE / 'linker.ld'}", "-C", "link-arg=--no-pie",
             "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
             f"--remap-path-prefix={ROOT}=/nieuw-os"]
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "build/sdk-memory-probe-rust190"))
    target = Path(env["CARGO_TARGET_DIR"])
    env["CARGO_TARGET_DIR"] = str((ROOT / target).resolve())
    command = ["cargo", f"+{TOOLCHAIN}", "build", "--offline", "--locked", "--release",
               "--manifest-path", str(EXAMPLE / "Cargo.toml"), "--features", "native",
               "--bin", "memory-probe", "--target", "x86_64-unknown-none"]
    print(json.dumps({"command": command, "target_dir": env["CARGO_TARGET_DIR"],
                      "app_toolchain": TOOLCHAIN, "linker": str(linker)}), flush=True)
    result = subprocess.run(command, cwd=ROOT, env=env, check=False)
    if result.returncode:
        return result.returncode
    from build_users import inspect_user
    binary = Path(env["CARGO_TARGET_DIR"]).resolve() / "x86_64-unknown-none/release/memory-probe"
    print(json.dumps({"passed": True, "extra_env": {"NOS_MEMORY_ELF": str(binary)},
                      "memory_probe": {"sha256": sha256(binary), "elf": inspect_user(binary)}}), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
