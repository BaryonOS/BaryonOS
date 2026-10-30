#!/usr/bin/env python3
"""FR-I01 gate on one frozen source: normal, exact-image cold readonly recovery.

Keeps the joint normal build as the final artifact, never substitutes M1 for
the tested image. Legacy profiles are separate required regression commands.
"""
from __future__ import annotations

import hashlib
from pathlib import Path
import subprocess
import sys

from build import build
from common import ROOT, BUILD, environment, save_json, source_hash
from run_system import FEATURE, run_system


def inputs() -> dict[str, str]:
    # Capture tests reuse both independent network wire fixture families.
    files = sorted({*(ROOT / "tests").glob("test_system_*.py"),
                    *(ROOT / "tests").glob("test_net_*.py"),
                    *(ROOT / "tests").glob("test_tcp_*.py"),
                    ROOT / "sdk/examples/tcp-probe/test_peer.py"})
    return {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}


def main() -> int:
    frozen, test_inputs = source_hash(), inputs()
    summary = dict(schema_version=1, source_fingerprint_version=3, source_sha256=frozen,
                   test_inputs=test_inputs, status="running", passed=False, host=[], stages=[])
    path = BUILD / "system-gate.json"
    save_json(path, summary)
    try:
        commands = [
            [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", "test_system_*.py"],
            ["cargo", "+1.90.0", "test", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime"],
        ]
        for number, command in enumerate(commands):
            result = subprocess.run(command, cwd=ROOT, env=environment(), capture_output=True, text=True)
            logs = BUILD / "logs" / f"system-host-{number}"
            logs.parent.mkdir(parents=True, exist_ok=True)
            logs.with_suffix(".stdout.txt").write_text(result.stdout)
            logs.with_suffix(".stderr.txt").write_text(result.stderr)
            summary["host"].append(dict(command=command, exit=result.returncode,
                                        stdout=str(logs.with_suffix(".stdout.txt")),
                                        stderr=str(logs.with_suffix(".stderr.txt"))))
            save_json(path, summary)
            if result.returncode:
                raise RuntimeError(f"host check {number} failed; see {logs}")
        volume = None
        normal_hash = None
        for case in FEATURE:
            manifest = build(None, FEATURE[case], FEATURE[case])
            if case == "normal":
                normal_hash = manifest["kernel"]["sha256"]
            result, volume = run_system(case, volume)
            summary["stages"].append({key: result[key] for key in
                                      ("case", "passed", "run_directory", "raw_qemu_exit",
                                       "before_sha256", "after_sha256", "duration_s")})
            save_json(path, summary)
            if not result["passed"]:
                raise RuntimeError(result.get("error", f"{case} failed"))
        final = build(None, FEATURE["normal"], FEATURE["normal"])
        if final["kernel"]["sha256"] != normal_hash:
            raise ValueError("final joint-system artifact differs from the actually tested normal kernel")
        if source_hash() != frozen or inputs() != test_inputs:
            raise ValueError("source or test inputs changed during gate")
        summary.update(status="completed", passed=True, final_kernel_sha256=normal_hash,
                       final_manifest=str(BUILD / "manifest.json"))
    except Exception as error:
        summary.update(status="failed", error=f"{type(error).__name__}: {error}")
    save_json(path, summary)
    print(f"FR-I01 gate passed={summary['passed']} stages={len(summary['stages'])} {path}")
    if "error" in summary:
        print(summary["error"])
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
