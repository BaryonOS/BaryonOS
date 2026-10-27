#!/usr/bin/env python3
"""Reproduce Q01 checks locally; never install tools or use runtime endpoints."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
OUTPUT = ROOT / "verification"


def main() -> int:
    env = os.environ.copy()
    local = Path(env.get("LOCALAPPDATA", "")) / "nieuw-os"
    if not shutil.which("cargo") and (local / "cargo/bin/cargo.exe").is_file():
        env["CARGO_HOME"] = str(local / "cargo")
        env["RUSTUP_HOME"] = str(local / "rustup")
        env["PATH"] = str(local / "cargo/bin") + os.pathsep + env.get("PATH", "")
    cargo = shutil.which("cargo", path=env.get("PATH"))
    if not cargo:
        print("Rust/Cargo missing; install the documented project toolchain first.", file=sys.stderr)
        return 2
    env["RUSTUP_AUTO_INSTALL"] = "0"
    env["CARGO_TARGET_DIR"] = str(ROOT / "target")
    OUTPUT.mkdir(exist_ok=True)
    report = {"scope": "Q01 host simulation; no OS port or physical QPU", "commands": [], "fixtures": [], "files": {}}

    def command(name: str, argv: list[str], timeout: int) -> bool:
        start = time.monotonic()
        try:
            result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, timeout=timeout, check=False)
            code = result.returncode
            out, err = result.stdout, result.stderr
        except subprocess.TimeoutExpired as error:
            code, out, err = None, error.stdout or b"", error.stderr or b""
        (OUTPUT / f"{name}.stdout.txt").write_bytes(out)
        (OUTPUT / f"{name}.stderr.txt").write_bytes(err)
        report["commands"].append({"name": name, "argv": argv, "exit": code, "timeout_seconds": timeout,
                                   "elapsed_seconds": round(time.monotonic() - start, 6)})
        print(f"{name}: exit {code}")
        return code == 0

    checks = [
        ("cargo-version", [cargo, "+1.90.0", "--version"], 30),
        ("rust-version", [str(Path(cargo).with_name("rustc.exe" if os.name == "nt" else "rustc")), "+1.90.0", "-vV"], 30),
        ("format", [cargo, "+1.90.0", "fmt", "--all", "--", "--check"], 30),
        ("build-tests", [cargo, "+1.90.0", "test", "--workspace", "--locked", "--offline", "--no-run"], 180),
        ("tests", [cargo, "+1.90.0", "test", "--workspace", "--locked", "--offline", "--", "--nocapture"], 30),
        ("clippy", [cargo, "+1.90.0", "clippy", "--workspace", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"], 180),
        ("no-std", [cargo, "+1.90.0", "check", "-p", "qsim-core", "--target", "x86_64-unknown-none", "--locked", "--offline"], 30),
        ("dependency-tree", [cargo, "+1.90.0", "tree", "--locked", "--offline"], 30),
    ]
    good = True
    for name, argv, timeout in checks:
        if not command(name, argv, timeout):
            good = False
            break
    binary = ROOT / "target/debug" / ("qsim-cli.exe" if os.name == "nt" else "qsim-cli")
    expectations = {"identity": (0, {"000": 8}), "x1": (0, {"10": 8}), "bell8": (0, {"00": 5, "11": 3}),
                    "bell4096": (0, {"00": 2082, "11": 2014}), "invalid-target": (2, None)}
    if good:
        for name, (expected_exit, counts) in expectations.items():
            start = time.monotonic()
            try:
                result = subprocess.run([str(binary)], input=(ROOT / f"fixtures/{name}.json").read_bytes(),
                                        capture_output=True, cwd=ROOT, env=env, timeout=30, check=False)
                code, out, err = result.returncode, result.stdout, result.stderr
            except subprocess.TimeoutExpired as error:
                code, out, err = None, error.stdout or b"", error.stderr or b""
            (OUTPUT / f"fixture-{name}.json").write_bytes(out)
            try:
                data = json.loads(out)
            except (ValueError, UnicodeDecodeError):
                data = {}
            passed = code == expected_exit and not err
            if counts is not None:
                passed = passed and data.get("counts") == counts and data.get("status") == "completed"
            else:
                passed = passed and data.get("error", {}).get("code") == "INVALID_REQUEST"
            report["fixtures"].append({"name": name, "exit": code, "passed": passed,
                                       "elapsed_seconds": round(time.monotonic() - start, 6)})
            good = good and passed
    for path in [ROOT / "Cargo.lock", Path(__file__), binary, *ROOT.glob("qsim-*/src/*.rs"), *ROOT.glob("qsim-*/tests/*.rs")]:
        if path.is_file():
            report["files"][str(path.relative_to(ROOT))] = hashlib.sha256(path.read_bytes()).hexdigest()
    report["passed"] = good
    (OUTPUT / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"report: {OUTPUT / 'report.json'}")
    return 0 if good else 1


if __name__ == "__main__":
    raise SystemExit(main())
