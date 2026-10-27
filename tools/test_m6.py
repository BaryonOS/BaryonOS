#!/usr/bin/env python3
"""M6a gate: real WRITE/FLUSH, six service cuts and six actual VM crash pairs."""
from __future__ import annotations
from datetime import datetime, timezone
import sys
from common import ROOT, BUILD, run_logged, save_json, source_hash
from build import build, M6A_CASES
from run import run_guest
from run_m6 import run_m6, UNKNOWN


def test_m6():
    path = BUILD / "m6-test-results.json"
    frozen = source_hash()
    suite = {"phase": "M6a", "status": "running", "passed": False,
             "source_fingerprint_version": 3, "source_sha256": frozen,
             "started_utc": datetime.now(timezone.utc).isoformat(), "results": []}
    save_json(path, suite)

    def record(name, passed, evidence, requirements, **details):
        suite["results"].append({"name": name, "passed": bool(passed), "evidence": str(evidence),
                                 "requirements": requirements, **details})
        save_json(path, suite)
        if not passed:
            raise RuntimeError(f"M6a gate failed: {name}")

    def guest(case, label=None, volume=None):
        report, volume = run_m6(case, label=label, volume=volume)
        record(label or case, report["passed"] and report["source_sha256"] == frozen,
               report["run_directory"], ["M6A-" + case.upper(), "M6A-EXTERNAL-ORACLE", "M6A-RESOURCES"])
        return report, volume

    def recovery(previous, volume, label):
        build(kernel_feature="test-m6a-recover", label="m6a-recover")
        report, _ = run_m6("recover", volume=volume, label=label)
        checks = {"same_volume_identity": report["disk_oracle"]["identity"] == previous["disk_oracle"]["identity"],
                  "same_volume_path": report["volume_path"] == previous["volume_path"],
                  "new_invocation": report["run_directory"] != previous["run_directory"],
                  "linked_previous": report["previous_stage"] == previous["run_directory"],
                  "exact_previous_after": report["artifact_sha256"]["test-disk.before.img"] == previous["artifact_sha256"]["test-disk.after.img"],
                  "recovery_has_no_volume_writes": report["artifact_sha256"]["test-disk.after.img"] == previous["artifact_sha256"]["test-disk.after.img"],
                  "frozen_source": report["source_sha256"] == previous["source_sha256"] == frozen}
        record(label, report["passed"] and all(checks.values()), report["run_directory"],
               ["M6A-COLD-RECOVERY", "M6A-SAME-IMAGE", "M6A-NO-WRITE-REPLAY"], recovery_checks=checks)
        return report

    try:
        run_logged([sys.executable, "tools/preflight.py", "--strict"], "m6a-preflight")
        record("strict-development-inventory", True, "build/logs/m6a-preflight.stdout.txt", ["M6A-TOOLS"])
        run_logged(["cargo", "+1.90.0", "test", "--locked", "--offline", "-p", "capability-core",
                    "-p", "virtio-core", "-p", "kernel-core", "-p", "block-protocol"], "m6a-host-tests")
        record("core-protocol-journal-host-tests", True, "build/logs/m6a-host-tests.stdout.txt", ["M6A-AUTHORITY", "M6A-POLICY", "M6A-JOURNAL"])
        run_logged([sys.executable, "-m", "unittest", "discover", "-s", "tests", "-p", "test_m6_*.py"], "m6a-harness-tests")
        record("external-volume-and-crash-harness-tests", True, "build/logs/m6a-harness-tests.stdout.txt", ["M6A-EVIDENCE", "M6A-IMAGE-BOUNDARY"])
        build(kernel_feature="milestone-m6a", label="m6a")
        run_logged([sys.executable, "-B", "-X", "utf8", "userspace/tests/test_m6_probe.py",
                    "--elf", "build/users/m6_probe.elf"], "m6a-user-artifact-tests")
        record("m6-user-artifact-tests", True, "build/logs/m6a-user-artifact-tests.stdout.txt", ["M6A-USER-ELF"])
        normals = []
        for suffix in ("a", "b"):
            if suffix == "b": build(kernel_feature="milestone-m6a", label="m6a")
            report, volume = guest("normal", label=f"m6a-normal-{suffix}")
            normals.append(report)
            recovery(report, volume, f"m6a-normal-{suffix}-cold-recovery")
        a, b = normals
        comparison = {"same_kernel": a["artifact_sha256"]["esp/kernel.elf"] == b["artifact_sha256"]["esp/kernel.elf"],
                      "same_loader": a["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"] == b["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"],
                      "different_actual_initial_images": a["disk_oracle"]["sha256_initial"] != b["disk_oracle"]["sha256_initial"],
                      "different_received_values": a["disk_oracle"]["observed"]["value_digest"] != b["disk_oracle"]["observed"]["value_digest"]}
        save_json(BUILD / "logs/m6a-two-image-check.json", {"checks": comparison,
                  "run_directories": [a["run_directory"], b["run_directory"]], "passed": all(comparison.values())})
        record("same-normal-binary-two-real-images", all(comparison.values()), "build/logs/m6a-two-image-check.json", ["M6A-EXTERNAL-ORACLE"])
        for case in M6A_CASES[:17]:
            build(kernel_feature=f"test-m6a-{case}", label=f"test-m6a-{case}")
            report, volume = guest(case)
            if case in UNKNOWN or case == "full":
                recovery(report, volume, f"{case}-cold-recovery")
        for point in range(1, 7):
            case = f"service-cut-{point}"
            build(kernel_feature=f"test-m6a-{case}", label=f"test-m6a-{case}")
            guest(case)
        for point in range(1, 7):
            case = f"vm-cut-{point}"
            build(kernel_feature=f"test-m6a-{case}", label=f"test-m6a-{case}")
            report, volume = guest(case)
            recovery(report, volume, f"{case}-cold-recovery")
        build()
        baseline = run_guest("m1-after-m6a")
        record("m1-regression-after-m6a", baseline["passed"] and baseline["source_sha256"] == frozen,
               baseline["run_directory"], ["M1"])
        if source_hash() != frozen:
            raise RuntimeError("Source changed during M6a gate; cannot bind combined proof")
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
    print(f"M6a gate passed: {len(suite['results'])} groups; full regression and independent audit remain separate")


if __name__ == "__main__":
    test_m6()
