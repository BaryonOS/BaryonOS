"""Read archived M5 records and validate the amended comparison; no VM runs."""
from __future__ import annotations
import copy
import hashlib
import json
from pathlib import Path
import random
import sys

ROOT = Path(__file__).resolve().parents[4]
CANDIDATE = ROOT
sys.path.insert(0, str(CANDIDATE / "tools"))
from m5_evidence import two_image_evidence, validate_two_image_evidence


def fnv64(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"{value:016x}"


def main():
    archive = CANDIDATE / "docs/verification/regression-m6b/m5"
    guests_path = archive / "guest-runs.json"
    evidence_path = archive / "m5-two-image-check.json"
    guests = json.loads(guests_path.read_text())
    evidence = json.loads(evidence_path.read_text())
    validate_two_image_evidence(evidence, guests)
    normal = [next(g for g in guests if g["label"] == label)
              for label in ("m5-read-a", "m5-read-b")]
    reconstructed = []
    for guest in normal:
        oracle = guest["disk_oracle"]
        data = bytearray(random.Random(oracle["seed"]).randbytes(64 * 1024))
        data[:1024] = bytes(1024)
        header = b"NOS-M5-TEST-VOLUME\0"
        data[:len(header)] = header
        digest = hashlib.sha256(data).hexdigest()
        sectors = {str(n): fnv64(data[n * 512:(n + 1) * 512]) for n in (4, 6)}
        assert digest == oracle["sha256_before"] == oracle["sha256_after"]
        assert digest == guest["artifact_sha256"]["test-disk.img"]
        assert sectors == oracle["observed_digests"] == oracle["expected_digests"]
        reconstructed.append({"run_id": Path(guest["run_directory"]).name,
                              "source_commit": guest["base_commit"],
                              "image_sha256": digest, "sector_digests": sectors})

    negative = []
    for field in ("esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI"):
        modified = copy.deepcopy(normal)
        modified[1]["artifact_sha256"][field] = "0" * 64
        assert not two_image_evidence(modified)["passed"]
        negative.append("different_" + field)
    modified = copy.deepcopy(normal)
    for field in ("sha256_before", "sha256_after"):
        modified[1]["disk_oracle"][field] = modified[0]["disk_oracle"][field]
    modified[1]["artifact_sha256"]["test-disk.img"] = modified[0]["artifact_sha256"]["test-disk.img"]
    assert not two_image_evidence(modified)["passed"]
    negative.append("same_image")
    modified = copy.deepcopy(normal)
    for field in ("expected_digests", "observed_digests"):
        modified[1]["disk_oracle"][field] = copy.deepcopy(modified[0]["disk_oracle"][field])
    lines = [line for line in modified[1]["stdout"].splitlines()
             if not line.startswith("M5:SECTOR")]
    modified[1]["stdout"] = "\n".join(lines + [
        f"M5:SECTOR{sector}_DIGEST:{value}"
        for sector, value in modified[0]["disk_oracle"]["observed_digests"].items()]) + "\n"
    assert not two_image_evidence(modified)["passed"]
    negative.append("same_received_digests")
    modified = copy.deepcopy(normal)
    modified[1]["base_commit"] = "0" * 40
    assert not two_image_evidence(modified)["passed"]
    negative.append("different_source_commit")
    for label, records, value in (
            ("missing_run", normal[:1], evidence),
            ("duplicate_run", normal + normal[:1], evidence),
            ("unbound_true_flags", normal, {"passed": True, "checks": {"same_kernel": True}})):
        try:
            validate_two_image_evidence(value, records)
        except ValueError:
            negative.append(label)
        else:
            raise AssertionError(label)
    result = {
        "scope": "Hercontrole van bestaande gecommitteerde gastrecords en reconstructie uit opgeslagen seeds; geen nieuwe VM-runs of opnieuw gelezen oorspronkelijke binaries.",
        "bot_commit": "56c43da0806f5f4de57cfb49dfd0418954e2ed9c",
        "guest_archive_sha256": hashlib.sha256(guests_path.read_bytes()).hexdigest(),
        "comparison_sha256": hashlib.sha256(evidence_path.read_bytes()).hexdigest(),
        "saved_comparison_matches_guest_records": True,
        "reconstructed_images": reconstructed,
        "negative_checks_rejected": negative,
        "passed": True,
    }
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()

