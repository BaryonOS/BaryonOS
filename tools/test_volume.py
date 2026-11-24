#!/usr/bin/env python3
"""Complete M6e/S06 gate: host tests, then thirty-two real QEMU invocations in order.

normal -> cold read-only recover (same image) -> S09 archive export into a
disjoint range of that image (volume mounted read-only) -> restore of only that
range on a fresh image, the same range with one damaged byte refused, and a
restore into a domain that already holds conflicting names and collections,
and an export into a target too small for the archive that must fail cleanly,
and the D6(a) ustar user export of that archive, read back with Python tarfile;
S09 test-only second disk at 00:06.0: export to it (disk 0 unchanged), an
absent and a too-small archive disk, a cold restore from that disk alone, the
wrong disk granted, and a power cut during an export over the older archive
followed by a cold verify that must report Corrupt;
cut-data (real kill) -> cold
recover; cut-table (real kill) -> cold recover; cut-txn (real kill inside a
multi-object transaction) -> cold recover; corrupt image refused;
S09 L2: install (INSTALL_COLLECTION, then INSTALL of one object with three
fixture versions, into never-used slots) -> cold read-only install-recover;
install-cut (real kill inside the INSTALL_COMMIT table write) -> cold
recover-cut, which must find the object never-used or fully installed;
S09 phase 3 on the device-1 disk: l2-export (writable source step, then a
v2 export) -> cold L2 restore without the source disk, a corrupt archive
refused, and the restored domain after a cold boot refusing a second restore;
l2-restore-cut (real kill inside an INSTALL_COMMIT of that restore) -> cold
recover-l2-cut, which must find every planned object fully installed or
never-used, exactly the commits before the cut, and refuse a second restore.
Every stage is bound to one frozen source fingerprint. The host test inputs
under tests/ fall outside that fingerprint, so they are frozen separately and
re-checked before the verdict. Only the final summary with passed=true counts
as a green gate.
"""
from __future__ import annotations
import hashlib
import subprocess
import sys
from pathlib import Path
from common import BUILD, ROOT, save_json, source_hash
from build import build
from run_volume import FEATURE, run_volume

TEST_INPUTS = "test_volume_*.py"
HOST = [
    ["cargo", "+1.100.0", "test", "--locked", "-p", "volume-core"],
    ["cargo", "+1.100.0", "test", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime"],
    [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", TEST_INPUTS],
]


def test_inputs() -> dict[str, str]:
    """SHA-256 of every file unittest discovery can load for the gate: the
    matching test modules at any depth plus every package __init__.py, since
    discovery also descends into packages under tests/."""
    tests = ROOT / "tests"
    paths = set(tests.rglob(TEST_INPUTS)) | set(tests.rglob("__init__.py"))
    return {path.relative_to(ROOT).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(paths)}


PLAN = [("normal", None), ("recover", "normal"), ("archive-export", "recover"),
        ("archive-restore", "archive-export"), ("archive-corrupt", "archive-export"),
        ("archive-conflict", "archive-export"), ("archive-full", "archive-export"),
        ("archive-ustar", "archive-export"),
        ("archive-disk-export", "archive-export"), ("archive-disk-absent", "archive-export"),
        ("archive-disk-small", "archive-export"), ("archive-disk-restore", "archive-disk-export"),
        ("archive-disk-wrong", "archive-disk-export"), ("archive-disk-cut", "archive-disk-export"),
        ("archive-disk-verify", "archive-disk-cut"),
        ("cut-data", None), ("recover-cut", "cut-data"), ("cut-table", None),
        ("recover-cut", "cut-table"), ("cut-txn", None), ("recover-cut", "cut-txn"), ("corrupt", None),
        ("install", None), ("install-recover", "install"), ("install-cut", None),
        ("recover-cut", "install-cut"),
        ("l2-export", "archive-export"), ("l2-restore", "l2-export"), ("l2-corrupt", "l2-export"),
        ("l2-used", "l2-restore"), ("l2-restore-cut", "l2-export"), ("recover-l2-cut", "l2-restore-cut")]


def main() -> int:
    frozen = source_hash()
    tests = test_inputs()
    summary = {"schema_version": 1, "source_fingerprint_version": 3, "source_sha256": frozen,
               "test_inputs": tests, "status": "running", "passed": False, "host": [], "stages": []}
    path = BUILD / "volume-gate.json"
    save_json(path, summary)
    for command in HOST:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        # Keep the complete output: cargo prints the per-binary test totals
        # long before its final lines, and they are part of the evidence.
        totals = [line for line in (result.stdout + result.stderr).splitlines()
                  if line.startswith(("test result:", "Ran ", "OK", "FAILED"))]
        summary["host"].append({"command": command, "exit": result.returncode, "totals": totals,
                                "stdout": result.stdout, "stderr": result.stderr})
        if result.returncode != 0:
            summary["status"] = "failed"
            save_json(path, summary)
            print(f"host check failed: {' '.join(command)}")
            return 1
    volumes = {}
    built = None
    for case, parent in PLAN:
        if built != FEATURE[case]:
            build(None, FEATURE[case], FEATURE[case])
            built = FEATURE[case]
        report, volume = run_volume(case, volumes.get(parent))
        volumes[case] = volume
        summary["stages"].append({"case": case, "parent": parent, "passed": report["passed"],
                                  "run_directory": report["run_directory"],
                                  "volume_identity": report["volume_identity"],
                                  "before_sha256": report["before_sha256"],
                                  "after_sha256": report["after_sha256"],
                                  "archive_after_sha256": report.get("archive_after_sha256"),
                                  "raw_qemu_exit": report["raw_qemu_exit"],
                                  "duration_s": report["duration_s"],
                                  "error": report.get("error")})
        save_json(path, summary)
        if not report["passed"]:
            summary["status"] = "failed"
            save_json(path, summary)
            return 1
    # Leave the ordinary M1 image in place like the other gates. The verdict
    # is written only after that build, so a failing build never leaves a
    # passing summary behind.
    try:
        build()
    except Exception:
        summary["status"] = "failed"
        save_json(path, summary)
        raise
    summary.update(status="completed", passed=source_hash() == frozen and test_inputs() == tests)
    save_json(path, summary)
    print(f"M6e gate passed={summary['passed']} stages={len(summary['stages'])}")
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
