#!/usr/bin/env python3
"""Preserve a completed gate only after rechecking its raw guest evidence."""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import shutil
import tempfile
from common import ROOT, BUILD, save_json, sha256
from run import is_success, MARKERS, FAIL_MARKERS, MILESTONE_MARKERS
from gate_contract import M1_MARKERS, guest_expectation


def requires_guest_evidence(phase, group):
    if phase in ("m2", "m3", "m4"):
        return guest_expectation(phase, group) is not None
    if phase == "m1":
        return group in {
            "cold-boot-1", "cold-boot-2", "cold-boot-3",
            "reject-missing-kernel", "reject-corrupt-kernel",
            "real-ebs-stale-key-retry", "kernel-panic-fails", "real-hang-times-out",
        }
    if phase == "m5":
        from build import M5_CASES
        return (group in ("m1-regression-after-m5", "m5-read-a", "m5-read-b")
                or group.startswith("test-m5-")
                and group.removeprefix("test-m5-") in M5_CASES)
    return False


def _replace_archive(staged: Path, destination: Path) -> None:
    backup_root = Path(tempfile.mkdtemp(
        prefix=f".{destination.name}-backup-", dir=destination.parent))
    backup = backup_root / "previous"
    had_destination = destination.exists() or destination.is_symlink()
    try:
        if had_destination:
            os.replace(destination, backup)
        try:
            os.replace(staged, destination)
        except BaseException:
            if had_destination:
                try:
                    os.replace(backup, destination)
                except BaseException as restore_error:
                    raise RuntimeError(
                        f"Could not restore the previous archive; it remains at {backup}"
                    ) from restore_error
            raise
        if had_destination:
            shutil.rmtree(backup, ignore_errors=True)
    finally:
        if not backup.exists():
            backup_root.rmdir()


def guest_matches(phase, group, result):
    if not isinstance(group, str) or not isinstance(result, dict) or not all(
            field in result for field in ("raw_qemu_exit", "stdout", "timed_out")):
        return False
    if not isinstance(result["stdout"], str) or not isinstance(result["timed_out"], bool):
        return False
    if phase == "m5":
        if group == "m1-regression-after-m5":
            required = M1_MARKERS
        elif group in ("m5-read-a", "m5-read-b"):
            if result.get("m5_case") != "read":
                return False
            required = MILESTONE_MARKERS["m5"]
        elif group.startswith("test-m5-"):
            case = group.removeprefix("test-m5-")
            from build import M5_CASES
            if case not in M5_CASES or result.get("m5_case") != case:
                return False
            marker = case.upper().replace("-", "_")
            required = MARKERS[:3] + (
                f"M5:CASE:{marker}:PASS", "M5:SURVIVOR_PROGRESS:PASS",
                "M5:RESOURCES:PASS", "M5:PASS", "M1:PASS")
        else:
            return False
        if not is_success(result["raw_qemu_exit"], result["stdout"], result["timed_out"],
                          required_markers=required, forbidden_markers=FAIL_MARKERS):
            return False
        oracle = result.get("disk_oracle")
        return (group == "m1-regression-after-m5"
                or isinstance(oracle, dict) and oracle.get("passed") is True)
    if phase != "m1":
        expectation = guest_expectation(phase, group)
        if expectation is None:
            return False
        required, expected_exit, forbidden = expectation
        return is_success(result["raw_qemu_exit"], result["stdout"], result["timed_out"],
                          required_markers=required, expected_exit=expected_exit,
                          forbidden_markers=forbidden)
    # M1 also tests that the guest/harness rejects real failures. Preserve the
    # original passed=false in those raw guest records; only the outer test passes.
    lines = result["stdout"].splitlines()
    if group in ("reject-missing-kernel", "reject-corrupt-kernel"):
        return (result["raw_qemu_exit"] == 35 and not result["timed_out"]
                and any(line.startswith("M1:FAIL:") for line in lines)
                and "KERNEL:ENTRY" not in lines and "M1:PASS" not in lines)
    if group == "kernel-panic-fails":
        return (result["raw_qemu_exit"] == 35 and not result["timed_out"]
                and "KERNEL:ENTRY" in lines and "M1:FAIL:KERNEL_PANIC" in lines
                and "M1:PASS" not in lines)
    if group == "real-hang-times-out":
        return (result["timed_out"] and result["raw_qemu_exit"] is None
                and result["passed"] is False and "KERNEL:ENTRY" in lines
                and "M1:PASS" not in lines and not any("M1:FAIL" in line for line in lines))
    if group == "real-ebs-stale-key-retry":
        return is_success(result["raw_qemu_exit"], result["stdout"], result["timed_out"],
                          required_markers=(MARKERS[0], "BOOT:EBS_RETRY") + MARKERS[1:])
    if group in ("cold-boot-1", "cold-boot-2", "cold-boot-3"):
        return is_success(result["raw_qemu_exit"], result["stdout"], result["timed_out"])
    return False


def capture(phase: str, destination: str | None = None):
    if phase == "m6a":
        if destination is not None:
            raise ValueError("M6a has a dedicated evidence directory")
        from capture_m6 import capture as capture_m6
        return capture_m6()
    suite_path = BUILD / ("test-results.json" if phase == "m1" else f"{phase}-test-results.json")
    suite = json.loads(suite_path.read_text(encoding="utf-8"))
    if suite.get("status") != "completed" or suite.get("passed") is not True:
        raise ValueError("Only a completed, passed gate can be captured")
    if phase == "m5":
        comparisons = [group for group in suite["results"]
                       if group.get("name") == "same-binary-two-actual-disks"]
        if len(comparisons) != 1:
            raise ValueError("M5 requires exactly one two-image comparison group")
        evidence = Path(comparisons[0]["evidence"])
        if not evidence.is_absolute():
            evidence = ROOT / evidence
        if evidence.resolve() != (BUILD / "logs/m5-two-image-check.json").resolve():
            raise ValueError("M5 comparison requires m5-two-image-check.json evidence")
    if destination is None:
        destination = (ROOT / "docs/verification/m5" if phase == "m5"
                       else ROOT / "docs/verification/m2-m4" / phase)
    else:
        destination = (ROOT / destination).resolve()
        destination.relative_to((ROOT / "docs/verification").resolve())
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=f".{phase}-capture-", dir=destination.parent) as temporary:
        temporary = Path(temporary)
        staged = temporary / "archive"
        staged.mkdir()

        runs = []
        two_image_checks = []
        for group in suite["results"]:
            if group.get("passed") is not True:
                raise ValueError(f"Failed group: {group['name']}")
            if group["evidence"] is None and phase == "m1" and group["name"] == "harness-rejects-wrong-exit-markers-timeout":
                # This synthetic parser assertion has no guest artifact. It remains
                # labeled as such in the unmodified suite, never as an extra VM run.
                continue
            evidence = Path(group["evidence"])
            if not evidence.is_absolute():
                evidence = ROOT / evidence
            if requires_guest_evidence(phase, group["name"]) and not evidence.is_dir():
                raise ValueError(f"Guest case requires run-directory evidence: {group['name']}")
            if evidence.is_dir():
                evidence.resolve().relative_to((BUILD / "run").resolve())
                result = json.loads((evidence / "result.json").read_text(encoding="utf-8"))
                if not guest_matches(phase, group["name"], result):
                    raise ValueError(f"Guest evidence failed: {evidence}")
                if (evidence / "serial.txt").read_bytes().decode("utf-8") != result["stdout"]:
                    raise ValueError(f"Serial evidence changed: {evidence}")
                if (evidence / "stderr.txt").read_bytes().decode("utf-8") != result["stderr"]:
                    raise ValueError(f"Stderr evidence changed: {evidence}")
                for name, expected in result["artifact_sha256"].items():
                    artifact = evidence / name
                    artifact.resolve().relative_to(evidence.resolve())
                    # Older runs recorded the immutable package template's hash,
                    # but retained only the firmware-mutated working VARS file.
                    # Verify that original source explicitly; never hash working VARS.
                    if name == "OVMF_VARS.template.fd" and not artifact.exists():
                        artifact = Path(result["firmware_source"][1])
                    if sha256(artifact) != expected:
                        raise ValueError(f"Artifact changed: {artifact}")
                if phase == "m5" and group["name"] != "m1-regression-after-m5":
                    from m5_disk import verify_disk
                    oracle = verify_disk(
                        evidence / "test-disk.img", result["disk_oracle"], result["stdout"],
                        data_required=result["m5_case"] not in ("no-device", "reset-timeout"))
                    if not oracle["passed"]:
                        raise ValueError(f"Disk oracle failed: {evidence}")
                runs.append(result)
            elif evidence.is_file() and (
                    evidence.name.endswith(".stdout.txt")
                    or (phase == "m5" and evidence.name == "m5-two-image-check.json")):
                host_logs = (BUILD / "logs").resolve()
                resolved_evidence = evidence.resolve()
                resolved_evidence.relative_to(host_logs)
                if resolved_evidence.name.endswith(".stdout.txt"):
                    # Preserve textual host output; never turn a guest result into a host log.
                    stem = resolved_evidence.name.removesuffix(".stdout.txt")
                    metadata = resolved_evidence.with_name(stem + ".json")
                    paths = [resolved_evidence, metadata,
                             resolved_evidence.with_name(stem + ".stderr.txt")]
                    for source in paths:
                        source.resolve().relative_to(host_logs)
                        if not source.is_file():
                            raise ValueError(f"Missing host log: {source}")
                    details = json.loads(metadata.read_text(encoding="utf-8"))
                    if details.get("returncode") != 0:
                        raise ValueError(f"Failed host command: {metadata}")
                else:
                    details = json.loads(resolved_evidence.read_text(encoding="utf-8"))
                    checks = details.get("checks")
                    if (details.get("passed") is not True or not isinstance(checks, dict)
                            or not checks or any(value is not True for value in checks.values())):
                        raise ValueError(f"Failed host evidence: {resolved_evidence}")
                    two_image_checks.append(details)
                    paths = [resolved_evidence]
                for source in paths:
                    target = staged / source.name
                    if target.exists():
                        raise ValueError(f"Duplicate host log name: {source.name}")
                    target.write_text(
                        source.read_text(encoding="utf-8").rstrip() + "\n", encoding="utf-8")
            else:
                raise ValueError(f"Missing evidence: {evidence}")
        if phase == "m5" and len(two_image_checks) != 1:
            raise ValueError("M5 requires exactly one bound two-image evidence file")
        if two_image_checks:
            from m5_evidence import validate_two_image_evidence
            for details in two_image_checks:
                validate_two_image_evidence(details, runs)
        save_json(staged / "suite.json", suite)
        save_json(staged / "guest-runs.json", runs)

        _replace_archive(staged, destination)
    print(f"Captured {phase}: {len(suite['results'])} groups, {len(runs)} verified guest runs")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=["m1", "m2", "m3", "m4", "m5", "m6a"])
    parser.add_argument("--destination", help="Evidence directory within docs/verification/")
    args = parser.parse_args()
    capture(args.phase, args.destination)
