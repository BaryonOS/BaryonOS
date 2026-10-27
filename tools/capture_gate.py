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
from run import is_success, MARKERS
from gate_contract import guest_expectation


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
        expectation = guest_expectation(phase, group)
        if expectation is None or not all(
                field in result for field in ("raw_qemu_exit", "stdout", "timed_out")):
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
