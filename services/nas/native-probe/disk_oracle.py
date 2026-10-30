#!/usr/bin/env python3
"""Read-only oracle for the fixed NAS-IPC-PROBE-1 disk outcome, not guest execution."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import struct
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import volume_disk

SNAPSHOT_BYTES = 17216
NS_BYTES = 4288
CORE_GENERATION = 6
# Six snapshot publications; all but the first also purge old volume history.
VOLUME_GENERATION = 11


def principal(pid: int) -> bytes:
    if type(pid) is not int or not 0 < pid < 1 << 64:
        raise ValueError("invalid local probe PID")
    return b"NOSPROC1" + pid.to_bytes(8, "little") + bytes(16)


def check_snapshot(snapshot: bytes, admin_pid: int, guest_pid: int) -> None:
    admin, guest = principal(admin_pid), principal(guest_pid)
    if admin == guest:
        raise ValueError("admin and guest must be different local principals")
    if len(snapshot) != SNAPSHOT_BYTES:
        raise ValueError("NAS snapshot length")
    namespace = snapshot[64:96]
    if namespace == bytes(32):
        raise ValueError("missing NAS namespace")
    # This reconstructs only this fixed probe's exact final tombstone. It is
    # deliberately not a second general-purpose NAS schema decoder.
    expected = bytearray(SNAPSHOT_BYTES)
    expected[:8] = b"NOSNAS01"
    struct.pack_into("<H", expected, 8, 1)
    struct.pack_into("<Q", expected, 16, CORE_GENERATION)
    expected[24:56] = admin
    expected[64:96] = namespace
    expected[96] = 4
    struct.pack_into("<Q", expected, 104, 2)
    struct.pack_into("<I", expected, 112, 128)
    expected[120:152] = guest
    if snapshot != expected:
        raise ValueError("NAS final tombstone, identity or cleared bytes mismatch")


def check_image(
    image: bytes, admin_pid: int, guest_pid: int, normal_sha256: str | None = None
) -> dict:
    if len(image) != volume_disk.IMAGE_SECTORS * volume_disk.SECTOR:
        raise ValueError("synthetic image size")
    digest = hashlib.sha256(image).hexdigest()
    if normal_sha256 is not None:
        if (
            len(normal_sha256) != 64
            or any(c not in "0123456789abcdef" for c in normal_sha256)
        ):
            raise ValueError("normal image SHA-256 must be 64 lowercase hex characters")
        if digest != normal_sha256:
            raise ValueError("cold recovery changed disk bytes")
    volume = image[
        volume_disk.FIRST * volume_disk.SECTOR:
        (volume_disk.FIRST + volume_disk.VOLUME_SECTORS) * volume_disk.SECTOR
    ]
    table = volume_disk.choose(volume)
    if table["generation"] != VOLUME_GENERATION or table["clocks"] != [VOLUME_GENERATION, 0]:
        raise ValueError("unexpected fixed-probe volume publication count or domain")
    # Bound revisions before analyse's fixture search; arbitrary u64 revisions
    # must not turn a malicious image into an effectively unbounded host loop.
    decoded = volume_disk.analyse(image, opaque=frozenset({0}))
    authority = decoded["objects"][0]
    if (
        not authority["present"]
        or authority["revision"] != VOLUME_GENERATION
        or len(authority["versions"]) != 1
        or authority["collection"] != 0
        or authority["tags"] != 0
        or authority["kind"] != 0
        or any(obj["revision"] != 0 for obj in decoded["objects"][1:])
        or any(c["revision"] != 0 for c in decoded["collections"])
    ):
        raise ValueError("authority object history or unrelated volume state changed")
    current = authority["versions"][0]
    if current["name"] != "nas-authority-v1" or current["length"] != SNAPSHOT_BYTES:
        raise ValueError("wrong authority record")
    check_snapshot(bytes.fromhex(current["content_hex"]), admin_pid, guest_pid)
    return {
        "schema_version": 1,
        "oracle": "NAS-IPC-PROBE-1-fixed-end-state",
        "image_sha256": digest,
        "volume_generation": VOLUME_GENERATION,
        "core_generation": CORE_GENERATION,
        "status": "Evicted",
        "epoch": 2,
        "quota": 128,
        "stored_ciphertext_bytes": 0,
        "authority_history_versions": 0,
        "scope_canaries_valid": True,
        "cold_disk_unchanged": True if normal_sha256 is not None else None,
        "guest_execution": "not-assessed",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", type=Path)
    parser.add_argument("--admin-pid", type=int, required=True)
    parser.add_argument("--guest-pid", type=int, required=True)
    parser.add_argument("--normal-sha256")
    args = parser.parse_args()
    try:
        with args.image.open("rb") as stream:
            info = os.fstat(stream.fileno())
            expected = volume_disk.IMAGE_SECTORS * volume_disk.SECTOR
            if not stat.S_ISREG(info.st_mode) or info.st_size != expected:
                raise ValueError("only a regular 1-MiB synthetic image is accepted")
            image = stream.read(expected + 1)
        result = check_image(image, args.admin_pid, args.guest_pid, args.normal_sha256)
    except (OSError, ValueError) as error:
        parser.exit(2, f"NAS disk oracle failed: {error}\n")
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
