#!/usr/bin/env python3
"""M5 gate: typed policies, real ring3 service, virtio device and host disk oracle."""
from __future__ import annotations
from datetime import datetime, timezone
import sys
from common import BUILD, run_logged, save_json
from build import build, M5_CASES
from run import run_guest, MILESTONE_MARKERS


def test_m5():
    path = BUILD / "m5-test-results.json"
    suite = {"phase": "M5", "status": "running", "passed": False,
             "started_utc": datetime.now(timezone.utc).isoformat(), "results": []}
    save_json(path, suite)

    def record(name, passed, evidence, requirements):
        suite["results"].append({"name": name, "passed": bool(passed),
                                 "requirements": requirements, "evidence": str(evidence)})
        save_json(path, suite)
        if not passed:
            raise RuntimeError(f"M5 gate failed: {name}")

    try:
        run_logged(["cargo", "+1.90.0", "test", "--locked", "--offline", "-p", "virtio-core",
                    "-p", "capability-core", "-p", "block-protocol", "-p", "kernel-core"], "m5-host-tests")
        record("typed-capability-virtio-policy-protocol-host-tests", True,
               "build/logs/m5-host-tests.stdout.txt", ["M5-TYPED", "M5-QUEUE", "M5-PROTOCOL"])
        run_logged([sys.executable, "-m", "unittest", "discover", "-s", "tests", "-p", "test_*harness.py"],
                   "m5-parser-tests")
        record("guest-evidence-parser-tests", True, "build/logs/m5-parser-tests.stdout.txt", ["M5-EVIDENCE"])
        run_logged([sys.executable, "-m", "unittest", "discover", "-s", "tests", "-p", "test_m5_disk.py"],
                   "m5-disk-oracle-tests")
        record("external-disk-oracle-negative-tests", True, "build/logs/m5-disk-oracle-tests.stdout.txt", ["M5-ORACLE"])
        build(kernel_feature="milestone-m5", label="m5")
        run_logged([sys.executable, "-B", "-X", "utf8", "userspace/tests/test_m5_probe.py",
                    "--elf", "build/users/m5_probe.elf"], "m5-user-artifact-tests")
        record("m5-user-artifact-tests", True, "build/logs/m5-user-artifact-tests.stdout.txt", ["M5-USER-ELF"])
        normal = []
        for name in ("m5-read-a", "m5-read-b"):
            guest = run_guest(name, required_markers=MILESTONE_MARKERS["m5"], m5_case="read")
            record(name, guest["passed"], guest["run_directory"], ["M5-READ", "M5-ORACLE", "M5-RESOURCES"])
            normal.append(guest)
        a, b = normal
        checks = {
            "same_kernel": a["artifact_sha256"]["esp/kernel.elf"] == b["artifact_sha256"]["esp/kernel.elf"],
            "same_loader": a["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"] == b["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"],
            "different_images": a["disk_oracle"]["sha256_before"] != b["disk_oracle"]["sha256_before"],
            "different_received_digests": a["disk_oracle"]["observed_digests"] != b["disk_oracle"]["observed_digests"],
        }
        evidence = BUILD / "logs/m5-two-image-check.json"
        save_json(evidence, {"checks": checks, "runs": [g["run_directory"] for g in normal],
                             "passed": all(checks.values())})
        record("same-binary-two-actual-disks", all(checks.values()), "build/logs/m5-two-image-check.json", ["M5-ORACLE"])
        for case in M5_CASES:
            feature = f"test-m5-{case}"
            build(kernel_feature=feature, label=feature)
            marker = case.upper().replace("-", "_")
            guest = run_guest(feature, m5_case=case, required_markers=(
                "BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", f"M5:CASE:{marker}:PASS",
                "M5:SURVIVOR_PROGRESS:PASS", "M5:RESOURCES:PASS", "M5:PASS", "M1:PASS"))
            record(feature, guest["passed"], guest["run_directory"], [f"M5-{marker}", "M5-RESOURCES"])
        build()
        guest = run_guest("m1-after-m5")
        record("m1-regression-after-m5", guest["passed"], guest["run_directory"], ["M1"])
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
    print(f"M5 gate passed: {len(suite['results'])} groups; combined regression/review still required")


if __name__ == "__main__":
    test_m5()
