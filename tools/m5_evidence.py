"""Bind the M5 two-image claim to the two actual guest records."""
from __future__ import annotations
from pathlib import Path
import re
from run import FAIL_MARKERS, MILESTONE_MARKERS, is_success


def _provenance(guest):
    oracle = guest.get("disk_oracle", {})
    artifacts = guest.get("artifact_sha256", {})
    if (guest.get("passed") is not True or guest.get("m5_case") != "read"
            or not is_success(guest.get("raw_qemu_exit"), guest.get("stdout", ""),
                              guest.get("timed_out", True),
                              required_markers=MILESTONE_MARKERS["m5"],
                              forbidden_markers=FAIL_MARKERS)
            or oracle.get("passed") is not True):
        raise ValueError("M5 two-image evidence requires successful read guest records")
    if (not re.fullmatch(r"[0-9a-f]{40}", guest.get("base_commit", ""))
            or guest.get("source_fingerprint_version") != 3
            or not re.fullmatch(r"[0-9a-f]{64}", guest.get("source_sha256", ""))):
        raise ValueError("M5 two-image evidence requires source provenance")
    for name in ("esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI", "test-disk.img"):
        if not re.fullmatch(r"[0-9a-f]{64}", artifacts.get(name, "")):
            raise ValueError(f"M5 two-image evidence requires artifact hash: {name}")
    digests = oracle.get("observed_digests")
    lines = [line for line in guest["stdout"].splitlines() if line.startswith("M5:SECTOR")]
    if (not isinstance(digests, dict) or set(digests) != {"4", "6"}
            or any(not re.fullmatch(r"[0-9a-f]{16}", value) for value in digests.values())
            or sorted(line.lower() for line in lines) != sorted(
                f"M5:SECTOR{sector}_DIGEST:{value}".lower() for sector, value in digests.items())
            or digests != oracle.get("expected_digests")
            or artifacts["test-disk.img"] != oracle.get("sha256_before")
            or oracle.get("sha256_after") != oracle.get("sha256_before")):
        raise ValueError("M5 two-image evidence disagrees with guest image or digests")
    run_id = Path(guest["run_directory"]).name
    if not run_id.startswith(guest["label"] + "-"):
        raise ValueError("M5 two-image evidence requires a stable run ID")
    return {
        "run_id": run_id,
        "label": guest["label"],
        "base_commit": guest["base_commit"],
        "source_fingerprint_version": guest["source_fingerprint_version"],
        "source_sha256": guest["source_sha256"],
        "artifact_sha256": artifacts,
        "image_sha256_before": oracle["sha256_before"],
        "image_sha256_after": oracle["sha256_after"],
        "expected_digests": oracle["expected_digests"],
        "observed_digests": digests,
    }


def two_image_evidence(guests):
    provenance = []
    for label in ("m5-read-a", "m5-read-b"):
        matching = [guest for guest in guests if guest.get("label") == label]
        if len(matching) != 1:
            raise ValueError(f"M5 two-image evidence requires exactly one guest record: {label}")
        provenance.append(_provenance(matching[0]))
    a, b = provenance
    checks = {
        "same_kernel": a["artifact_sha256"]["esp/kernel.elf"] == b["artifact_sha256"]["esp/kernel.elf"],
        "same_loader": a["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"] == b["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"],
        "different_images": a["image_sha256_before"] != b["image_sha256_before"],
        "different_received_digests": a["observed_digests"] != b["observed_digests"],
        "same_source": all(a[key] == b[key] for key in (
            "base_commit", "source_fingerprint_version", "source_sha256")),
    }
    return {"checks": checks, "runs": [run["run_id"] for run in provenance],
            "provenance": provenance, "passed": all(checks.values())}


def validate_two_image_evidence(evidence, guests):
    expected = two_image_evidence(guests)
    if expected["passed"] is not True or evidence != expected:
        raise ValueError("M5 two-image evidence is not bound to verified guest records")
