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
CORE_GENERATION = 11
# Eleven snapshot publications; all but the first also purge old volume history.
VOLUME_GENERATION = 21


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
    namespaces = [snapshot[64 + i * NS_BYTES:96 + i * NS_BYTES] for i in range(2)]
    if any(ns == bytes(32) for ns in namespaces) or namespaces[0] == namespaces[1]:
        raise ValueError("missing or duplicate NAS namespace")
    # Reconstruct both exact final tombstones, including all cleared bytes.
    # This is deliberately not a second general-purpose NAS schema decoder.
    expected = bytearray(SNAPSHOT_BYTES)
    expected[:8] = b"NOSNAS01"
    struct.pack_into("<H", expected, 8, 1)
    struct.pack_into("<Q", expected, 16, CORE_GENERATION)
    expected[24:56] = admin
    for index, quota in enumerate((256, 128)):
        at = 64 + index * NS_BYTES
        expected[at:at + 32] = namespaces[index]
        expected[at + 32] = 4
        struct.pack_into("<Q", expected, at + 40, 2)
        struct.pack_into("<I", expected, at + 48, quota)
        expected[at + 56:at + 88] = guest
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
        "quota": 256,
        "namespace_count": 2,
        "stored_ciphertext_bytes": 0,
        "authority_history_versions": 0,
        "scope_canaries_valid": True,
        "cold_disk_unchanged": True if normal_sha256 is not None else None,
        "guest_execution": "not-assessed",
    }


# P04-T1 (LP-25) interrupted-purge fixture: initialize, invite, accept, commit,
# revoke, evict = six core publications. The NAS dies after evict's volume WRITE
# and before its history PURGE: volume revision 10 (1 + 4 write/purge pairs + 1
# write) keeps one history version. The repair boot's explicit PURGE gives 11.
INTERRUPT_CORE_GENERATION = 6
INTERRUPT_VOLUME_GENERATION = {"interrupt": 10, "repair": 11}
INTERRUPT_QUOTA = 128
INTERRUPT_CONTENT = bytes((i * 17 + 9) & 0xFF for i in range(96))
NS_HEADER = 128


def interrupt_snapshot(namespace: bytes, admin_pid: int, guest_pid: int, *,
                       generation: int, revoked_copy: bool) -> bytes:
    """Exact single-namespace snapshot: the Evicted tombstone, or the Revoked
    predecessor that still holds the guest's synthetic ciphertext copy."""
    admin, guest = principal(admin_pid), principal(guest_pid)
    if admin == guest or len(namespace) != 32 or namespace == bytes(32):
        raise ValueError("invalid interrupted-purge identity")
    expected = bytearray(SNAPSHOT_BYTES)
    expected[:8] = b"NOSNAS01"
    struct.pack_into("<H", expected, 8, 1)
    struct.pack_into("<Q", expected, 16, generation)
    expected[24:56] = admin
    expected[64:96] = namespace
    expected[96] = 3 if revoked_copy else 4
    struct.pack_into("<Q", expected, 104, 2)
    struct.pack_into("<I", expected, 112, INTERRUPT_QUOTA)
    expected[120:152] = guest
    if revoked_copy:
        at = 64 + NS_HEADER
        struct.pack_into("<QH", expected, at, 1, len(INTERRUPT_CONTENT))
        expected[at + 16:at + 16 + len(INTERRUPT_CONTENT)] = INTERRUPT_CONTENT
    return bytes(expected)


def check_interrupted_image(image: bytes, admin_pid: int, guest_pid: int, stage: str) -> dict:
    """`interrupt`: Evicted current plus exactly one retained Revoked copy.
    `repair`: the same Evicted current and no history. Logical volume state
    only; freed sectors are not inspected and no physical erasure is claimed."""
    if stage not in INTERRUPT_VOLUME_GENERATION:
        raise ValueError("unknown interrupted-purge stage")
    generation = INTERRUPT_VOLUME_GENERATION[stage]
    history = 1 if stage == "interrupt" else 0
    versions = authority_versions(image, generation, history, "interrupted-purge")
    current = versions[0]
    namespace = current[64:96]
    if current != interrupt_snapshot(namespace, admin_pid, guest_pid,
                                     generation=INTERRUPT_CORE_GENERATION, revoked_copy=False):
        raise ValueError("interrupted-purge current tombstone mismatch")
    if history and versions[1] != interrupt_snapshot(
            namespace, admin_pid, guest_pid,
            generation=INTERRUPT_CORE_GENERATION - 1, revoked_copy=True):
        raise ValueError("interrupted-purge retained version is not the exact revoked copy")
    return {
        "schema_version": 1,
        "oracle": f"NAS-IPC-PROBE-1-interrupted-purge-{stage}",
        "image_sha256": hashlib.sha256(image).hexdigest(),
        "volume_generation": generation,
        "core_generation": INTERRUPT_CORE_GENERATION,
        "status": "Evicted",
        "epoch": 2,
        "quota": INTERRUPT_QUOTA,
        "namespace_count": 1,
        "authority_history_versions": history,
        "retained_ciphertext_bytes": len(INTERRUPT_CONTENT) * history,
        "scope_canaries_valid": True,
        "physical_erasure": "not-assessed",
        "guest_execution": "not-assessed",
    }


def authority_versions(image: bytes, generation: int, history: int, label: str) -> list[bytes]:
    """The authority object's current version and `history` older ones, after
    checking the exact volume publication count and untouched other objects."""
    if len(image) != volume_disk.IMAGE_SECTORS * volume_disk.SECTOR:
        raise ValueError("synthetic image size")
    volume = image[
        volume_disk.FIRST * volume_disk.SECTOR:
        (volume_disk.FIRST + volume_disk.VOLUME_SECTORS) * volume_disk.SECTOR
    ]
    table = volume_disk.choose(volume)
    if table["generation"] != generation or table["clocks"] != [generation, 0]:
        raise ValueError(f"unexpected {label} volume publication count or domain")
    decoded = volume_disk.analyse(image, opaque=frozenset({0}))
    authority = decoded["objects"][0]
    if (
        not authority["present"]
        or authority["revision"] != generation
        or len(authority["versions"]) != 1 + history
        or authority["collection"] != 0
        or authority["tags"] != 0
        or authority["kind"] != 0
        or any(obj["revision"] != 0 for obj in decoded["objects"][1:])
        or any(c["revision"] != 0 for c in decoded["collections"])
    ):
        raise ValueError(f"{label} authority history or unrelated volume state differs")
    if any(v["name"] != "nas-authority-v1" or v["length"] != SNAPSHOT_BYTES
           for v in authority["versions"]):
        raise ValueError("wrong authority record")
    return [bytes.fromhex(v["content_hex"]) for v in authority["versions"]]


# P02-T3 offline/resume: initialize, invite, accept, commit = four core
# publications (offline); the resume boot's reconciled commit adds the fifth.
# Open, Stage, Head, Read and plain recovery publish nothing.
RESUME_CORE_GENERATION = {"offline": 4, "resume": 5}
RESUME_REVISION = {"offline": (1, INTERRUPT_CONTENT),
                   "resume": (2, bytes((i * 29 + 3) & 0xFF for i in range(32)))}


def check_resume_image(image: bytes, admin_pid: int, guest_pid: int, stage: str) -> dict:
    """One Active namespace (epoch 1, quota 128) whose object 0 holds exactly
    revision 1 after `offline` or the reconciled revision 2 after `resume`."""
    if stage not in RESUME_CORE_GENERATION:
        raise ValueError("unknown offline/resume stage")
    core = RESUME_CORE_GENERATION[stage]
    volume = 1 + 2 * (core - 1)
    (current,) = authority_versions(image, volume, 0, "offline/resume")
    admin, guest = principal(admin_pid), principal(guest_pid)
    namespace = current[64:96]
    if admin == guest or namespace == bytes(32):
        raise ValueError("invalid offline/resume identity")
    revision, content = RESUME_REVISION[stage]
    expected = bytearray(SNAPSHOT_BYTES)
    expected[:8] = b"NOSNAS01"
    struct.pack_into("<H", expected, 8, 1)
    struct.pack_into("<Q", expected, 16, core)
    expected[24:56] = admin
    expected[64:96] = namespace
    expected[96] = 2
    struct.pack_into("<Q", expected, 104, 1)
    struct.pack_into("<I", expected, 112, INTERRUPT_QUOTA)
    expected[120:152] = guest
    at = 64 + NS_HEADER
    struct.pack_into("<QH", expected, at, revision, len(content))
    expected[at + 16:at + 16 + len(content)] = content
    if current != bytes(expected):
        raise ValueError("offline/resume snapshot, object revision or cleared bytes mismatch")
    return {
        "schema_version": 1,
        "oracle": f"NAS-IPC-PROBE-1-{stage}",
        "image_sha256": hashlib.sha256(image).hexdigest(),
        "volume_generation": volume,
        "core_generation": core,
        "status": "Active",
        "epoch": 1,
        "quota": INTERRUPT_QUOTA,
        "namespace_count": 1,
        "object_revision": revision,
        "stored_ciphertext_bytes": len(content),
        "authority_history_versions": 0,
        "scope_canaries_valid": True,
        "client_persistence": "not-assessed",
        "guest_execution": "not-assessed",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", type=Path)
    parser.add_argument("--admin-pid", type=int, required=True)
    parser.add_argument("--guest-pid", type=int, required=True)
    parser.add_argument("--normal-sha256")
    parser.add_argument("--stage", choices=sorted(INTERRUPT_VOLUME_GENERATION))
    args = parser.parse_args()
    try:
        with args.image.open("rb") as stream:
            info = os.fstat(stream.fileno())
            expected = volume_disk.IMAGE_SECTORS * volume_disk.SECTOR
            if not stat.S_ISREG(info.st_mode) or info.st_size != expected:
                raise ValueError("only a regular 1-MiB synthetic image is accepted")
            image = stream.read(expected + 1)
        if args.stage:
            if args.normal_sha256:
                raise ValueError("--normal-sha256 applies only to the normal/cold oracle")
            result = check_interrupted_image(image, args.admin_pid, args.guest_pid, args.stage)
        else:
            result = check_image(image, args.admin_pid, args.guest_pid, args.normal_sha256)
    except (OSError, ValueError) as error:
        parser.exit(2, f"NAS disk oracle failed: {error}\n")
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
