#!/usr/bin/env python3
"""Acceptance tests: host invariants, three cold boots, corrupt/missing images,
real stale-map-key retry, kernel panic and a real 60-second guest timeout."""
import shutil
import uuid
import json
from datetime import datetime, timezone
from common import BUILD, run_logged, save_json, sha256
from build import build
from run import run_guest, is_success


def expected_artifacts(manifest):
    """Bind to the build result retained in memory, not a mutable disk manifest."""
    return {'expected_kernel_sha256': manifest['kernel']['sha256'],
            'expected_loader_sha256': manifest['loader']['sha256']}


def snapshot_build(manifest, label):
    """Retain this build before later commands can replace the public ESP."""
    esp = BUILD / f"m1-candidate-{label}-{uuid.uuid4().hex}" / "esp"
    shutil.copytree(BUILD / "esp", esp)
    for relative, expected in (("kernel.elf", manifest['kernel']['sha256']),
                               ("EFI/BOOT/BOOTX64.EFI", manifest['loader']['sha256'])):
        actual = sha256(esp / relative)
        if actual != expected:
            raise ValueError(f"M1 candidate artifact SHA256 mismatch: {relative}; expected={expected}; actual={actual}")
    return esp


def main():
    results = []
    suite_report = {"status": "running", "passed": False, "started_utc": datetime.now(timezone.utc).isoformat(), "results": results}
    save_json(BUILD / "test-results.json", suite_report)
    def record(name, passed, evidence=None):
        results.append({"name": name, "passed": bool(passed), "evidence": evidence})
        save_json(BUILD / "test-results.json", suite_report)
        if not passed:
            raise RuntimeError(f"acceptance failed: {name}")
    normal = build()
    normal_esp = snapshot_build(normal, "normal")
    run_logged(["cargo", "+1.90.0", "test", "--locked", "-p", "boot-protocol", "-p", "elf-loader-core"], "host-tests")
    record("host-protocol-elf-tests", True, "build/logs/host-tests.stdout.txt")
    # These are harness-only checks; genuine VM evidence is collected below.
    expected = "\n".join(("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "M1:PASS"))
    record("harness-rejects-wrong-exit-markers-timeout", all((not is_success(0, expected), not is_success(33, "M1:PASS"), not is_success(33, expected, True), not is_success(33, expected + "M1:FAIL:x"), not is_success(33, "\n".join(reversed(expected.splitlines()))), not is_success(33, "\n".join(line + "-bogus" for line in expected.splitlines())), not is_success(33, "\n".join("bogus-" + line for line in expected.splitlines())))))
    for n in range(3):
        report = run_guest(f"cold-{n+1}", esp=normal_esp, **expected_artifacts(normal))
        record(f"cold-boot-{n+1}", report["passed"], report["run_directory"])
    for mutation in ("missing", "corrupt"):
        esp = BUILD / "negative" / f"{mutation}-{uuid.uuid4().hex[:8]}"
        shutil.copytree(normal_esp, esp, ignore=shutil.ignore_patterns("kernel.elf"))
        if mutation == "corrupt":
            data = bytearray((normal_esp / "kernel.elf").read_bytes())
            data[0] = 0
            (esp / "kernel.elf").write_bytes(data)
        report = run_guest(mutation, esp=esp)
        record(f"reject-{mutation}-kernel", report["raw_qemu_exit"] == 35 and "M1:FAIL" in report["stdout"] and "KERNEL:ENTRY" not in report["stdout"] and "M1:PASS" not in report["stdout"] and not report["timed_out"], report["run_directory"])
    try:
        retry = build(loader_feature="test-ebs-retry", label="ebs-retry")
        retry_esp = snapshot_build(retry, "ebs-retry")
        report = run_guest("ebs-retry", esp=retry_esp, **expected_artifacts(retry))
        record("real-ebs-stale-key-retry", report["passed"] and "BOOT:EBS_RETRY" in report["stdout"], report["run_directory"])
        panic = build(kernel_feature="test-panic", label="panic")
        panic_esp = snapshot_build(panic, "panic")
        report = run_guest("panic", esp=panic_esp, **expected_artifacts(panic))
        record("kernel-panic-fails", report["raw_qemu_exit"] == 35 and "KERNEL:ENTRY" in report["stdout"] and "M1:FAIL:KERNEL_PANIC" in report["stdout"].splitlines() and "M1:PASS" not in report["stdout"], report["run_directory"])
        hang = build(kernel_feature="test-hang", label="hang")
        hang_esp = snapshot_build(hang, "hang")
        report = run_guest("hang", esp=hang_esp, **expected_artifacts(hang))
        record("real-hang-times-out", report["timed_out"] and "KERNEL:ENTRY" in report["stdout"] and not report["passed"], report["run_directory"])
    finally:
        build()
    suite_report.update(status="completed", passed=True, finished_utc=datetime.now(timezone.utc).isoformat())
    save_json(BUILD / "test-results.json", suite_report)
    print(f"PASS: {len(results)} acceptance groups; normal artifacts restored")


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        path = BUILD / "test-results.json"
        report = json.loads(path.read_text()) if path.exists() else {"results": []}
        report.update(status="failed", passed=False, error=str(exc), finished_utc=datetime.now(timezone.utc).isoformat())
        save_json(path, report)
        raise
