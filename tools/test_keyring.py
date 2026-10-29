#!/usr/bin/env python3
"""Complete B01 gate: host tests, then three real QEMU invocations (normal,
crash, denied, parent-exit, quota) on one frozen source fingerprint. The host
test inputs under tests/ are frozen separately and re-checked before the
verdict. Only the final summary with passed=true counts as a green gate."""
from __future__ import annotations
import hashlib
import subprocess
import sys
from common import BUILD, ROOT, save_json, source_hash
from build import build
from run_keyring import FEATURE, run_keyring

TEST_INPUTS = "test_keyring_*.py"
HOST = [
    ["cargo", "+1.90.0", "test", "--locked", "-p", "capability-core", "-p", "virtio-rng-driver", "-p", "keyring-core"],
    ["cargo", "+1.90.0", "test", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime"],
    [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", TEST_INPUTS],
]
PLAN = ("normal", "no-device", "stuck")


def test_inputs() -> dict[str, str]:
    tests = ROOT / "tests"
    paths = set(tests.rglob(TEST_INPUTS)) | set(tests.rglob("__init__.py"))
    return {path.relative_to(ROOT).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(paths)}


def main() -> int:
    frozen = source_hash()
    tests = test_inputs()
    summary = {"schema_version": 1, "source_fingerprint_version": 3, "source_sha256": frozen,
               "test_inputs": tests, "status": "running", "passed": False, "host": [], "stages": []}
    path = BUILD / "keyring-gate.json"
    save_json(path, summary)
    for command in HOST:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        totals = [line for line in (result.stdout + result.stderr).splitlines()
                  if line.startswith(("test result:", "Ran ", "OK", "FAILED"))]
        summary["host"].append({"command": command, "exit": result.returncode, "totals": totals,
                                "stdout": result.stdout, "stderr": result.stderr})
        if result.returncode != 0:
            summary["status"] = "failed"
            save_json(path, summary)
            print(f"host check failed: {' '.join(command)}")
            return 1
    for case in PLAN:
        build(None, FEATURE[case], FEATURE[case])
        report = run_keyring(case)
        summary["stages"].append({"case": case, "passed": report["passed"],
                                  "run_directory": report["run_directory"],
                                  "raw_qemu_exit": report["raw_qemu_exit"],
                                  "duration_s": report["duration_s"],
                                  "evidence": report.get("evidence"),
                                  "error": report.get("error")})
        save_json(path, summary)
        if not report["passed"]:
            summary["status"] = "failed"
            save_json(path, summary)
            return 1
    try:
        build()
    except Exception:
        summary["status"] = "failed"
        save_json(path, summary)
        raise
    summary.update(status="completed", passed=source_hash() == frozen and test_inputs() == tests)
    save_json(path, summary)
    print(f"B01 gate passed={summary['passed']} stages={len(summary['stages'])}")
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
