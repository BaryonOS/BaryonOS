"""Independent evidence checks for the native Materiaal library walkthrough.

Only disposable, source-bound window-probe runs may supply a cold-reopen image.
This module decodes stopped disk bytes; app counters do not certify persistence.
"""
from __future__ import annotations

from pathlib import Path

from common import sha256
from run_fpu import load_json, private_path
import volume_disk

TITLE = "Reisnotities"
COLLECTION = "Reis"
ORIGINAL = b"nieuw os werkt"
EDITED = b"route langs de kust"


def disk_oracle(image: bytes) -> dict:
    # `opaque` requests exact content bytes from the existing strict decoder;
    # this does not exempt checksums, table structure, bounds or disk canaries.
    decoded = volume_disk.analyse(image, opaque=frozenset({0}))
    present = [item for item in decoded["objects"] if item["present"]]
    if len(present) != 1 or present[0]["object"] != 0:
        raise ValueError("Library must contain exactly its one actual document")
    document = present[0]
    versions = document["versions"]
    if (document["collection"] != 1 or document["tags"] != 0 or document["kind"] != 0
            or len(versions) != 2 or document["revision"] < 4
            or versions[0]["name"] != TITLE
            or bytes.fromhex(versions[0]["content_hex"]) != ORIGINAL
            or bytes.fromhex(versions[1]["content_hex"]) != EDITED
            or any(item["name"] != TITLE for item in versions)
            or not versions[0]["revision"] > versions[1]["revision"]):
        raise ValueError("Wrong native rename, organization or restore-as-new-revision")
    collections = [item for item in decoded["collections"] if item["present"]]
    if (len(collections) != 1 or collections[0]["domain"] != 0
            or collections[0]["index"] != 1 or collections[0]["name"] != COLLECTION):
        raise ValueError("Wrong persisted collection")
    if any(item["revision"] for item in decoded["objects"][1:]):
        raise ValueError("Library touched another document or domain")
    return decoded


def cold_parent(directory: Path, *, source: str, manifest: dict) -> dict:
    """Revalidate a stopped successful parent; never use a user disk as input."""
    directory = private_path(directory)
    report_path = private_path(directory / "result.json", regular=True)
    report = load_json(report_path)
    required = {
        "passed": True, "library_gate": True, "library_requested": True,
        "library_cold": False, "native_gate": False, "raw_qemu_exit": 33,
        "kernel_feature": "native-window-session-probe", "source_fingerprint_version": 3,
        "source_sha256": source, "manifest": manifest,
        "run_directory": str(directory),
    }
    if any(type(report.get(key)) is not type(value) or report.get(key) != value
           for key, value in required.items()):
        raise ValueError("Cold library parent is not a current accepted writable run")
    if report.get("harness_terminated_process") or report.get("error"):
        raise ValueError("Cold library parent did not finish normally")
    disk = private_path(directory / "volume.img", regular=True)
    transcript = private_path(directory / "serial.txt", regular=True)
    if (disk.stat().st_size != volume_disk.IMAGE_SECTORS * volume_disk.SECTOR
            or transcript.stat().st_size > 4 * 1024 * 1024
            or report.get("volume_sha256") != sha256(disk)
            or report.get("serial_sha256") != sha256(transcript)):
        raise ValueError("Cold library parent disk or transcript changed")
    raw = transcript.read_bytes()
    if (not raw.endswith(b"WINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n")
            or b"WINDOW:FAIL:" in raw or b"PANIC" in raw or b"\0" in raw):
        raise ValueError("Cold library parent lacks real verified cleanup")
    image = disk.read_bytes()
    decoded = disk_oracle(image)
    if report.get("disk_oracle") != decoded:
        raise ValueError("Cold library parent oracle does not match actual bytes")
    return {"image": image, "volume_sha256": sha256(disk),
            "report_sha256": sha256(report_path), "directory": str(directory)}
