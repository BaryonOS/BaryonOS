#!/usr/bin/env python3
"""Execute milestone gates against actual built guest code. Never infer a pass
from a feature name or compilation; require exact kernel observations and exit."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
from common import BUILD, ROOT, run_logged, save_json
from build import build
from run import run_guest

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")


def test_m2():
    path = BUILD / "m2-test-results.json"
    suite = {"phase": "M2", "status": "running", "passed": False,
             "started_utc": datetime.now(timezone.utc).isoformat(), "results": []}
    save_json(path, suite)

    def record(name, passed, evidence, requirements):
        suite["results"].append({"name": name, "passed": bool(passed),
                                 "requirements": requirements, "evidence": evidence})
        save_json(path, suite)
        if not passed:
            raise RuntimeError(f"M2 gate failed: {name}")

    try:
        build(kernel_feature="milestone-m2", label="m2")
        run_logged(["cargo", "+1.90.0", "test", "--locked", "-p", "kernel-core",
                    "-p", "boot-protocol", "-p", "elf-loader-core"], "m2-host-tests")
        record("frame-map-protocol-host-tests", True, "build/logs/m2-host-tests.stdout.txt",
               ["KS-01", "KS-02", "KS-03"])
        normal = run_guest("m2", required_markers=BOOT + (
            "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS", "M1:PASS"))
        record("own-cr3-frame-ownership-wx", normal["passed"], normal["run_directory"],
               ["KS-01", "KS-02", "KS-03", "OWN_GDT_TSS_IDT_CR3"])
        for feature, marker in (("test-m2-ro", "M2:FAULT:RO:PASS"),
                                ("test-m2-nx", "M2:FAULT:NX:PASS"),
                                ("test-m2-absent", "M2:FAULT:ABSENT:PASS")):
            build(kernel_feature=feature, label=feature)
            guest = run_guest(feature, required_markers=BOOT + (marker,))
            record(feature, guest["passed"], guest["run_directory"],
                   ["KS-03", "REAL_PAGE_FAULT"])
        build(kernel_feature="test-m2-kernel-fault", label="m2-kernel-fault")
        guest = run_guest("m2-kernel-fault", expected_exit=35,
                          required_markers=BOOT + ("M2:FAIL:KERNEL_EXCEPTION",),
                          forbidden_markers=("M1:PASS", "M2:PASS", "M1:FAIL"))
        record("kernel-fault-stops-controlled", guest["passed"], guest["run_directory"],
               ["KERNEL_FAULT_FAIL_CLOSED"])
        # Return the default ESP to its ordinary profile even after a failed case.
        build()
        baseline = run_guest("m1-after-m2")
        record("m1-regression", baseline["passed"], baseline["run_directory"], ["M1"])
        suite.update(status="completed", passed=True)
    except Exception as exc:
        suite.update(status="failed", passed=False, error=str(exc))
        # A failure remains visible even if restoring the default image also fails.
        save_json(path, suite)
        try:
            build()
        except Exception as restore:
            suite["restore_error"] = str(restore)
        raise
    finally:
        suite["finished_utc"] = datetime.now(timezone.utc).isoformat()
        save_json(path, suite)
    print(f"M2 gate passed: {len(suite['results'])} groups; M3 may now be considered")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--through", choices=["m2"], default="m2")
    parser.parse_args()
    test_m2()
