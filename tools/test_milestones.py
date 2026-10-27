#!/usr/bin/env python3
"""Execute milestone gates against actual built guest code. Never infer a pass
from a feature name or compilation; require exact kernel observations and exit."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
import sys
from common import BUILD, ROOT, run_logged, save_json
from build import build
from gate_contract import M3_CASES, M3_NONFAULT_CASES, M4_CASES
from run import run_gate_case

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
        normal = run_gate_case("m2", "own-cr3-frame-ownership-wx", label="m2")
        record("own-cr3-frame-ownership-wx", normal["passed"], normal["run_directory"],
               ["KS-01", "KS-02", "KS-03", "OWN_GDT_TSS_IDT_CR3"])
        for feature in ("test-m2-ro", "test-m2-nx", "test-m2-absent"):
            build(kernel_feature=feature, label=feature)
            guest = run_gate_case("m2", feature)
            record(feature, guest["passed"], guest["run_directory"],
                   ["KS-03", "REAL_PAGE_FAULT"])
        build(kernel_feature="test-m2-kernel-fault", label="m2-kernel-fault")
        guest = run_gate_case("m2", "kernel-fault-stops-controlled", label="m2-kernel-fault")
        record("kernel-fault-stops-controlled", guest["passed"], guest["run_directory"],
               ["KERNEL_FAULT_FAIL_CLOSED"])
        build(kernel_feature="test-m2-double-fault", label="m2-double-fault")
        guest = run_gate_case("m2", "unusable-main-stack-double-fault-ist",
                              label="m2-double-fault")
        record("unusable-main-stack-double-fault-ist", guest["passed"], guest["run_directory"],
               ["DOUBLE_FAULT_IST1", "KERNEL_STACK_GUARDS"])
        # Return the default ESP to its ordinary profile even after a failed case.
        build()
        baseline = run_gate_case("m2", "m1-regression", label="m1-after-m2")
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


def test_m3():
    path = BUILD / "m3-test-results.json"
    suite = {"phase": "M3", "status": "running", "passed": False,
             "started_utc": datetime.now(timezone.utc).isoformat(), "results": []}
    save_json(path, suite)

    def record(name, guest, requirements):
        suite["results"].append({"name": name, "passed": bool(guest["passed"]),
                                 "requirements": requirements, "evidence": guest["run_directory"]})
        save_json(path, suite)
        if not guest["passed"]:
            raise RuntimeError(f"M3 gate failed: {name}")

    try:
        run_logged(["cargo", "+1.90.0", "test", "--locked", "-p", "kernel-core"], "m3-host-tests")
        suite["results"].append({"name": "user-elf-frame-abi-host-tests", "passed": True,
                                 "requirements": ["KS-04", "USER_ELF", "RETURN_FLAGS"],
                                 "evidence": "build/logs/m3-host-tests.stdout.txt"})
        save_json(path, suite)
        build(kernel_feature="milestone-m3", label="m3")
        run_logged([sys.executable, "-B", "-X", "utf8", "userspace/tests/test_probe.py",
                    "--elf", "build/users/probe.elf"], "m3-user-artifact-tests")
        suite["results"].append({"name": "user-artifact-negative-tests", "passed": True,
                                 "requirements": ["USER_ELF", "USER_SYSCALL_FIXTURE"],
                                 "evidence": "build/logs/m3-user-artifact-tests.stdout.txt"})
        save_json(path, suite)
        guest = run_gate_case("m3", "private-address-spaces-preemption-fairness-cleanup",
                              label="m3")
        record("private-address-spaces-preemption-fairness-cleanup", guest,
               ["KS-04", "KS-07", "KS-08"])
        for case in M3_CASES:
            feature = f"test-m3-{case}"
            build(kernel_feature=feature, label=feature)
            requirement = "KS-05" if case in ("peer-read", "peer-write", "kernel-write") else "KS-06"
            guest = run_gate_case("m3", feature)
            record(feature, guest, ([requirement] if case not in M3_NONFAULT_CASES else []) + ["KS-08"])
        build(kernel_feature="test-m3-idle", label="test-m3-idle")
        guest = run_gate_case("m3", "all-processes-exit-idle-timer-and-resources",
                              label="test-m3-idle")
        record("all-processes-exit-idle-timer-and-resources", guest,
               ["KS-08", "IDLE_CPL0_IRQ0", "RESOURCE_RECLAIM"])
        build(kernel_feature="test-m3-large-image", label="test-m3-large-image")
        run_logged([sys.executable, "-B", "-X", "utf8", "userspace/tests/test_large_probe.py",
                    "--elf", "build/users/probe_large.elf"], "m3-large-artifact-tests")
        suite["results"].append({"name": "large-bss-artifact-host-tests", "passed": True,
                                 "requirements": ["MAPPING_CAPACITY", "USER_ELF"],
                                 "evidence": "build/logs/m3-large-artifact-tests.stdout.txt"})
        save_json(path, suite)
        guest = run_gate_case("m3", "large-bss-image-exceeds-old-64-mapping-limit",
                              label="test-m3-large-image")
        record("large-bss-image-exceeds-old-64-mapping-limit", guest,
               ["KS-04", "KS-07", "KS-08", "MAPPING_CAPACITY"])
        build()
        record("m1-regression-after-m3",
               run_gate_case("m3", "m1-regression-after-m3", label="m1-after-m3"), ["M1"])
        suite.update(status="completed", passed=True)
    except Exception as exc:
        suite.update(status="failed", passed=False, error=str(exc))
        save_json(path, suite)
        try:
            build()
        except Exception as restore:
            suite["restore_error"] = str(restore)
        raise
    finally:
        suite["finished_utc"] = datetime.now(timezone.utc).isoformat()
        save_json(path, suite)
    print(f"M3 gate passed: {len(suite['results'])} groups; M4 may now be considered")


def test_m4():
    path = BUILD / "m4-test-results.json"
    suite = {"phase": "M4", "status": "running", "passed": False,
             "started_utc": datetime.now(timezone.utc).isoformat(), "results": []}
    save_json(path, suite)

    def record(name, passed, evidence, requirements):
        suite["results"].append({"name": name, "passed": bool(passed),
                                 "requirements": requirements, "evidence": evidence})
        save_json(path, suite)
        if not passed:
            raise RuntimeError(f"M4 gate failed: {name}")

    try:
        run_logged(["cargo", "+1.90.0", "test", "--locked", "-p", "capability-core",
                    "-p", "kernel-core"], "m4-host-tests")
        record("capability-transaction-wire-generation-host-tests", True,
               "build/logs/m4-host-tests.stdout.txt", [r for _, r in M4_CASES])
        build(kernel_feature="milestone-m4", label="m4")
        run_logged([sys.executable, "-B", "-X", "utf8", "userspace/tests/test_ipc_probe.py",
                    "--elf", "build/users/ipc_probe.elf"], "m4-user-artifact-tests")
        record("ipc-user-artifact-negative-tests", True,
               "build/logs/m4-user-artifact-tests.stdout.txt", ["USER_IPC_ELF"])
        guest = run_gate_case("m4", "real-ring3-message-exchange", label="m4-exchange")
        record("real-ring3-message-exchange", guest["passed"], guest["run_directory"], ["IPC_POSITIVE"])
        for case, requirement in M4_CASES:
            feature = f"test-m4-{case}"
            build(kernel_feature=feature, label=feature)
            guest = run_gate_case("m4", feature)
            record(feature, guest["passed"], guest["run_directory"], [requirement])
        build()
        guest = run_gate_case("m4", "m1-regression-after-m4", label="m1-after-m4")
        record("m1-regression-after-m4", guest["passed"], guest["run_directory"], ["M1"])
        suite.update(status="completed", passed=True)
    except Exception as exc:
        suite.update(status="failed", passed=False, error=str(exc))
        save_json(path, suite)
        try:
            build()
        except Exception as restore:
            suite["restore_error"] = str(restore)
        raise
    finally:
        suite["finished_utc"] = datetime.now(timezone.utc).isoformat()
        save_json(path, suite)
    print(f"M4 gate passed: {len(suite['results'])} groups; final review/audit still required")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--through", choices=["m2", "m3", "m4"], default="m2")
    args = parser.parse_args()
    test_m2()
    if args.through in ("m3", "m4"):
        test_m3()
    if args.through == "m4":
        test_m4()
