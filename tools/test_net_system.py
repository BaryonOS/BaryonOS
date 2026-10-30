#!/usr/bin/env python3
"""Frozen native network pair; the local peer only supplies protocol replies."""
from __future__ import annotations

import hashlib
import subprocess
import sys

from build import build
from common import ROOT, BUILD, environment, save_json, source_hash
from run_net_system import FEATURE, run_net_system


def inputs() -> dict[str, str]:
    paths = sorted((ROOT / "tests").glob("test_net_*.py"))
    return {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}


def main() -> int:
    frozen, test_inputs = source_hash(), inputs()
    summary = dict(schema_version=1, source_fingerprint_version=3, source_sha256=frozen,
                   test_inputs=test_inputs, status="running", passed=False, host=[], stages=[])
    path = BUILD / "net-system-gate.json"
    save_json(path, summary)
    commands = [
        [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", "test_net_*.py"],
        ["cargo", "+1.90.0", "test", "--locked", "-p", "capability-core"],
        ["cargo", "+1.90.0", "test", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime"],
    ]
    try:
        for index, command in enumerate(commands):
            result = subprocess.run(command, cwd=ROOT, env=environment(), capture_output=True, text=True)
            logs = BUILD / "logs" / f"net-host-{index}"
            logs.parent.mkdir(parents=True, exist_ok=True)
            logs.with_suffix(".stdout.txt").write_text(result.stdout)
            logs.with_suffix(".stderr.txt").write_text(result.stderr)
            summary["host"].append(dict(command=command, exit=result.returncode,
                                        stdout=str(logs.with_suffix(".stdout.txt")),
                                        stderr=str(logs.with_suffix(".stderr.txt"))))
            save_json(path, summary)
            if result.returncode:
                raise RuntimeError(f"host check {index} failed; see {logs}")
        volume = None
        normal = None
        for case, feature in FEATURE.items():
            manifest = build(None, feature, feature)
            if case == "normal":
                normal = manifest["kernel"]["sha256"]
            result, volume = run_net_system(case, volume)
            summary["stages"].append({key: result[key] for key in
                                      ("case", "passed", "network_gate", "run_directory", "raw_qemu_exit",
                                       "before_sha256", "after_sha256", "duration_s")})
            save_json(path, summary)
            if result["passed"] is not True or result["network_gate"] is not True:
                raise RuntimeError(result.get("network_error", result.get("error", "network run failed")))
        final = build(None, FEATURE["normal"], FEATURE["normal"])
        if normal != final["kernel"]["sha256"]:
            raise ValueError("final network artifact differs from the actually tested normal kernel")
        if frozen != source_hash() or test_inputs != inputs():
            raise ValueError("source or tests changed during network gate")
        summary.update(status="completed", passed=True, final_kernel_sha256=normal,
                       final_manifest=str(BUILD / "manifest.json"))
    except Exception as error:
        summary.update(status="failed", error=f"{type(error).__name__}: {error}")
    save_json(path, summary)
    print(f"Native network gate passed={summary['passed']} stages={len(summary['stages'])} {path}")
    if "error" in summary:
        print(summary["error"])
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
