#!/usr/bin/env python3
"""Build and inspect the native qsim job caller/service ELF; no VM is started."""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "tools"))
from build_users import inspect_user  # noqa: E402

def load_native_builder():
    path = HERE.parent / "build.py"
    spec = importlib.util.spec_from_file_location("qsim_native_builder", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module

NATIVE = load_native_builder()

def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def sources() -> dict[str, str]:
    roots = [
        HERE,
        HERE.parent / "src",
        HERE.parent.parent / "qsim-core",
        HERE.parent.parent / "job-protocol",
        ROOT / "sdk/runtime",
    ]
    result = {}
    for root in roots:
        for path in sorted(root.rglob("*")):
            if path.is_file() and not {"target", "build", "__pycache__"}.intersection(path.parts):
                result[path.relative_to(ROOT).as_posix()] = sha(path)
    return result

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    args = parser.parse_args()
    output = args.output_dir.resolve()
    target = args.target_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    report = {
        "schema_version": 1,
        "kind": "L02 native caller-to-qsim-job service artifact",
        "passed": False,
        "native_runtime_performed": False,
        "source_sha256": sources(),
        "commands": [],
    }
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    env.pop("RUSTFLAGS", None)
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join([
        "-C", "code-model=small", "-C", "relocation-model=static",
        "-C", "no-redzone=yes", "-C", "linker=rust-lld",
        "-C", f"link-arg=-T{HERE / 'linker.ld'}", "-C", "link-arg=--no-pie",
        "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
    ])

    def run(command: list[str], name: str) -> str:
        completed = subprocess.run(command, cwd=ROOT, env=env, text=True,
                                   capture_output=True, timeout=240)
        (output / f"{name}.stdout.txt").write_text(completed.stdout)
        (output / f"{name}.stderr.txt").write_text(completed.stderr)
        report["commands"].append({"argv": command, "returncode": completed.returncode})
        if completed.returncode:
            raise RuntimeError(f"{name} failed: {completed.stderr}")
        return completed.stdout

    try:
        report["rustc"] = run(["rustc", "+1.90.0", "--version"], "rustc").strip()
        run([
            "cargo", "+1.90.0", "build", "--manifest-path", str(HERE / "Cargo.toml"),
            "--locked", "--offline", "--release", "--target", "x86_64-unknown-none",
            "--bin", "qsim-job-native",
        ], "build")
        binary = output / "qsim-job-native.elf"
        shutil.copyfile(target / "x86_64-unknown-none/release/qsim-job-native", binary)
        report["binary_sha256"] = sha(binary)
        report["elf"] = inspect_user(binary)
        disassembly = run(
            ["objdump", "-d", "--no-show-raw-insn", "-M", "intel", str(binary)],
            "disassembly",
        )
        report["scalar_gate"] = NATIVE.inspect_instructions(disassembly)
        report["disassembly_sha256"] = sha(output / "disassembly.stdout.txt")
        symbols = run(["nm", str(binary)], "symbols")
        wanted = ["__adddf3", "__muldf3", "__divdf3", "__floatdidf"]
        report["software_float_symbols"] = [
            name for name in wanted if any(line.rstrip().endswith(" " + name) for line in symbols.splitlines())
        ]
        if report["software_float_symbols"] != wanted:
            raise RuntimeError("missing scalar software-float helpers")
        report["passed"] = True
        print(f"Native qsim job artifact passed: {binary}")
        return 0
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
        print(report["error"])
        return 1
    finally:
        (output / "build-report.json").write_text(json.dumps(report, indent=2) + "\n")

if __name__ == "__main__":
    raise SystemExit(main())
