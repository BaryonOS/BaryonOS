#!/usr/bin/env python3
"""Read-only bytes of ui/native with and without `inter-atlas` (measurement only).

1. The scalar x86_64-unknown-none rlibs that ui/native/verify.py builds (same
   command and RUSTFLAGS, separate target dir per variant): sum of .rodata* and
   .data.rel.ro* sections over every archive member. Unlinked upper bound.
2. The linked materiaal-app ELF (sdk/examples/materiaal/build.py flags, whose
   linker.ld asserts .rodata <= 512 KiB): readonly PT_LOAD MemSiz.
Run from the repository root: python3 -I docs/verification/native-inter-atlas-20261114/measure_ro.py
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path.cwd()
LIMIT = 512 * 1024
SCALAR = "-C no-redzone=yes -C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2"
EXAMPLE = ROOT / "sdk/examples/materiaal"
ELF_FLAGS = ["-C", "code-model=small", "-C", "relocation-model=static", "-C", "no-redzone=yes",
             "-C", "linker=rust-lld", "-C", f"link-arg=-T{EXAMPLE / 'linker.ld'}", "-C", "link-arg=--no-pie",
             "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
             "-C", "target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2",
             f"--remap-path-prefix={ROOT}=/nieuw-os"]


def run(cmd, env=None):
    result = subprocess.run(cmd, cwd=ROOT, env=dict(os.environ, RUSTUP_AUTO_INSTALL="0", **(env or {})),
                            text=True, capture_output=True, timeout=600)
    if result.returncode:
        sys.exit(f"{cmd[:4]} exit {result.returncode}\n{result.stderr[-4000:]}")
    return result.stdout


def ro_sections(path):
    total = 0
    for line in run(["size", "-A", "-d", str(path)]).splitlines():
        parts = line.split()
        if len(parts) >= 2 and parts[0].startswith((".rodata", ".data.rel.ro")) and parts[1].isdigit():
            total += int(parts[1])
    return total


def rlibs(variant):
    features = ["--features", "inter-atlas"] if variant == "inter" else []
    target = ROOT / ("build/native-materiaal-none-target" + ("-inter" if variant == "inter" else ""))
    out = run(["cargo", "+1.90.0", "build", "--offline", "--locked", "--manifest-path", "ui/native/Cargo.toml",
               "--release", "--lib", "--target", "x86_64-unknown-none", "--message-format=json-render-diagnostics",
               *features], {"CARGO_TARGET_DIR": str(target), "RUSTFLAGS": SCALAR})
    crates = {}
    for line in out.splitlines():
        event = json.loads(line)
        if event.get("reason") == "compiler-artifact":
            (path,) = [p for p in event["filenames"] if p.endswith(".rlib")]
            crates[event["target"]["name"]] = ro_sections(path)
    return crates


def elf(variant):
    features = "native,nieuw-os-materiaal/inter-atlas" if variant == "inter" else "native"
    target = ROOT / ("build/sdk-materiaal" + ("-inter" if variant == "inter" else ""))
    run(["cargo", "+1.90.0", "build", "--offline", "--locked", "--release", "--manifest-path",
         str(EXAMPLE / "Cargo.toml"), "--features", features, "--bin", "materiaal-app",
         "--target", "x86_64-unknown-none"],
        {"CARGO_TARGET_DIR": str(target), "CARGO_ENCODED_RUSTFLAGS": "\x1f".join(ELF_FLAGS)})
    path = target / "x86_64-unknown-none/release/materiaal-app"
    loads = [l.split() for l in run(["readelf", "-lW", str(path)]).splitlines() if l.strip().startswith("LOAD")]
    # Flags print as "R", "RW" or "R E"; only a lone R followed by the alignment is read-only.
    readonly = [int(l[5], 16) for l in loads if l[6] == "R" and l[7].startswith("0x")]
    assert len(readonly) == 1, loads
    symbols = {}
    for line in run(["nm", "-S", "-C", str(path)]).splitlines():
        m = re.match(r"^[0-9a-f]+ ([0-9a-f]+) \w (nieuw_os_graphics::atlas::\w+)$", line)
        if m:
            symbols[m.group(2).rsplit("::", 1)[1]] = int(m.group(1), 16)
    return {"elf": str(path.relative_to(ROOT)), "readonly_pt_load_memsz": readonly, "atlas_symbols": symbols}


def main():
    report = {"limit_bytes": LIMIT}
    for variant in ("default", "inter"):
        crates = rlibs(variant)
        report[variant] = {"rlib_ro_bytes_total": sum(crates.values()), "rlib_ro_bytes": crates, **elf(variant)}
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
