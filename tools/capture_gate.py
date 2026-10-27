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
from build import M3_CASES, M4_CASES
from run import FAIL_MARKERS, MILESTONE_MARKERS, is_success, MARKERS


M2_AUDIT = ("KS-01:PASS", "KS-02:PASS", "KS-03:PASS")


def _guest_expectation(phase, group):
    boot = MARKERS[:3]
    if phase == "m2":
        if group == "own-cr3-frame-ownership-wx":
            return 33, MILESTONE_MARKERS["m2"], FAIL_MARKERS
        if group in ("test-m2-ro", "test-m2-nx", "test-m2-absent"):
            marker = {
                "test-m2-ro": "M2:FAULT:RO:PASS",
                "test-m2-nx": "M2:FAULT:NX:PASS",
                "test-m2-absent": "M2:FAULT:ABSENT:PASS",
            }[group]
            return 33, boot + M2_AUDIT + (marker,), FAIL_MARKERS
        if group == "kernel-fault-stops-controlled":
            return (35, boot + M2_AUDIT +
                    ("M2:FAULT:KERNEL_UD:PASS", "M2:FAIL:KERNEL_EXCEPTION"),
                    ("M1:PASS", "M2:PASS", "M1:FAIL"))
        if group == "unusable-main-stack-double-fault-ist":
            return (35, boot + M2_AUDIT + ("M2:FAULT:DF_IST:PASS",),
                    ("M1:PASS", "M2:PASS", "M1:FAIL", "M2:FAIL"))
        if group == "m1-regression":
            return 33, MILESTONE_MARKERS["m1"], FAIL_MARKERS
    elif phase == "m3":
        if group == "private-address-spaces-preemption-fairness-cleanup":
            return (33, boot + ("KS-01:PASS", "KS-02:PASS", "KS-03:PASS",
                                "KS-04:PASS", "KS-07:PASS", "KS-08:PASS",
                                "M3:SYSCALL_DF_ENTRY:PASS", "M3:PASS", "M1:PASS"),
                    FAIL_MARKERS)
        if group.startswith("test-m3-"):
            case = group.removeprefix("test-m3-")
            if case in M3_CASES:
                nonfault_events = {
                    "exit": "M3:EXIT:PASS",
                    "b-exit": "M3:EXIT_NONZERO_B:PASS",
                    "invalid-rsp": "M3:INVALID_RSP:PASS",
                }
                marker = nonfault_events.get(
                    case, f"M3:FAULT:{case.upper().replace('-', '_')}:PASS")
                requirement = ("KS-05" if case in ("peer-read", "peer-write", "kernel-write")
                               else "KS-06")
                observed = (marker, "M3:SURVIVOR_PROGRESS:PASS", "M3:CLEANUP:PASS")
                if case not in nonfault_events:
                    observed += (f"{requirement}:PASS",)
                observed += ("KS-08:PASS", "M3:SYSCALL_DF_ENTRY:PASS", "M3:PASS", "M1:PASS")
                return 33, boot + observed, FAIL_MARKERS
        if group == "all-processes-exit-idle-timer-and-resources":
            return 33, boot + ("M3:IDLE:PASS", "M3:PASS", "M1:PASS"), FAIL_MARKERS
        if group == "large-bss-image-exceeds-old-64-mapping-limit":
            return 33, MILESTONE_MARKERS["m3"], FAIL_MARKERS
        if group == "m1-regression-after-m3":
            return 33, MILESTONE_MARKERS["m1"], FAIL_MARKERS
    elif phase == "m4":
        if group == "real-ring3-message-exchange":
            return 33, MILESTONE_MARKERS["m4"], FAIL_MARKERS
        if group.startswith("test-m4-"):
            case = group.removeprefix("test-m4-")
            requirements = dict(M4_CASES)
            if case in requirements:
                return (33, boot + (f"M4:CASE:{case.upper()}:PASS",
                                    "M4:SURVIVOR_PROGRESS:PASS", "M4:CLEANUP:PASS",
                                    f"{requirements[case]}:PASS", "M4:PASS", "M1:PASS"),
                        FAIL_MARKERS)
        if group == "m1-regression-after-m4":
            return 33, MILESTONE_MARKERS["m1"], FAIL_MARKERS
    return None


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
    if phase != "m1":
        expectation = _guest_expectation(phase, group)
        if expectation is None:
            return False
        expected_exit, required_markers, forbidden_markers = expectation
        return is_success(result["raw_qemu_exit"], result["stdout"], result["timed_out"],
                          required_markers=required_markers, expected_exit=expected_exit,
                          forbidden_markers=forbidden_markers)
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


def capture(phase: str):
    suite_path = BUILD / ("test-results.json" if phase == "m1" else f"{phase}-test-results.json")
    suite = json.loads(suite_path.read_text(encoding="utf-8"))
    if suite.get("status") != "completed" or suite.get("passed") is not True:
        raise ValueError("Only a completed, passed gate can be captured")
    destination = ROOT / "docs/verification/m2-m4" / phase
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=f".{phase}-capture-", dir=destination.parent) as temporary:
        temporary = Path(temporary)
        staged = temporary / "archive"
        staged.mkdir()

        runs = []
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
                runs.append(result)
            elif evidence.is_file() and evidence.name.endswith(".stdout.txt"):
                host_logs = (BUILD / "logs").resolve()
                resolved_evidence = evidence.resolve()
                resolved_evidence.relative_to(host_logs)
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
                for source in paths:
                    target = staged / source.name
                    if target.exists():
                        raise ValueError(f"Duplicate host log name: {source.name}")
                    target.write_text(
                        source.read_text(encoding="utf-8").rstrip() + "\n", encoding="utf-8")
            else:
                raise ValueError(f"Missing evidence: {evidence}")
        save_json(staged / "suite.json", suite)
        save_json(staged / "guest-runs.json", runs)

        _replace_archive(staged, destination)
    print(f"Captured {phase}: {len(suite['results'])} groups, {len(runs)} verified guest runs")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=["m1", "m2", "m3", "m4"])
    capture(parser.parse_args().phase)
