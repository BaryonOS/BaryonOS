#!/usr/bin/env python3
"""Independent host oracle for the B03 slot store (`NOSSLOT1`, ADR-044).

Pure Python, no cryptography: it parses the fixed table, the header hint and
the structural manifest fields, hashes regions and applies the labelled
tamperings of the acceptance cases. The guest (loader + update service) and
`update-core` never see this code; agreement between the two is the evidence.
"""
from __future__ import annotations
import hashlib
import struct

SECTOR = 512
MANIFEST_MAGIC = b"NOSMAN01"
STORE_MAGIC = b"NOSSLOT1"
FORMAT_VERSION = 1
MANIFEST_LEN = 3500
MANIFEST_SECTORS = 7
SIGNED_LEN = 84
ED25519_SIGNATURE_OFFSET = 84
MLDSA_SIGNATURE_OFFSET = 148
MLDSA_SIGNATURE_LEN = 3309
IMAGE_CAPACITY_SECTORS = 4096
MAX_IMAGE_LEN = IMAGE_CAPACITY_SECTORS * SECTOR
SLOT_A_MANIFEST = 1
SLOT_A_IMAGE = SLOT_A_MANIFEST + MANIFEST_SECTORS
SLOT_B_MANIFEST = SLOT_A_IMAGE + IMAGE_CAPACITY_SECTORS
SLOT_B_IMAGE = SLOT_B_MANIFEST + MANIFEST_SECTORS
INBOX_MANIFEST = SLOT_B_IMAGE + IMAGE_CAPACITY_SECTORS
INBOX_IMAGE = INBOX_MANIFEST + MANIFEST_SECTORS
STORE_SECTORS = INBOX_IMAGE + IMAGE_CAPACITY_SECTORS
STORE_SIZE = STORE_SECTORS * SECTOR
REGIONS = {"A": (SLOT_A_MANIFEST, SLOT_A_IMAGE), "B": (SLOT_B_MANIFEST, SLOT_B_IMAGE),
           "inbox": (INBOX_MANIFEST, INBOX_IMAGE)}
TABLE = (SLOT_A_MANIFEST, SLOT_A_IMAGE, IMAGE_CAPACITY_SECTORS, SLOT_B_MANIFEST, SLOT_B_IMAGE,
         IMAGE_CAPACITY_SECTORS, INBOX_MANIFEST, INBOX_IMAGE, IMAGE_CAPACITY_SECTORS)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def region(store: bytes, sector: int, length: int) -> bytes:
    start = sector * SECTOR
    if start + length > len(store):
        raise ValueError("region outside the store")
    return store[start:start + length]


def parse_header(sector: bytes) -> dict:
    """Header hint: magic, format, active slot, generation and the fixed table."""
    if len(sector) != SECTOR or sector[:8] != STORE_MAGIC:
        raise ValueError("store magic missing")
    version, active = struct.unpack_from("<II", sector, 8)
    (generation,) = struct.unpack_from("<Q", sector, 16)
    table = struct.unpack_from("<9I", sector, 24)
    if version != FORMAT_VERSION or active > 1:
        raise ValueError("header format or active slot invalid")
    if table != TABLE or any(sector[60:]):
        raise ValueError("header table differs from the fixed layout")
    return {"active": active, "generation": generation}


def parse_manifest(data: bytes) -> dict | None:
    """Structural fields only (no signature check); None for an empty region."""
    if len(data) != MANIFEST_LEN:
        raise ValueError("manifest length")
    if not any(data):
        return None
    if data[:8] != MANIFEST_MAGIC:
        raise ValueError("manifest magic missing")
    version, target, image_length = struct.unpack_from("<III", data, 8)
    image_version, min_version = struct.unpack_from("<QQ", data, 20)
    if version != FORMAT_VERSION or target != 1 or not 0 < image_length <= MAX_IMAGE_LEN:
        raise ValueError("manifest format fields invalid")
    if any(data[MLDSA_SIGNATURE_OFFSET + MLDSA_SIGNATURE_LEN:]):
        raise ValueError("manifest tail is not zero")
    return {"image_length": image_length, "version": image_version, "min_version": min_version,
            "image_sha256": data[36:68].hex(), "ed25519_key_id": data[68:76].hex(),
            "mldsa_key_id": data[76:84].hex(), "manifest_sha256": sha256(data)}


def nonzero_sectors(data: bytes) -> int:
    return sum(1 for at in range(0, len(data), SECTOR) if any(data[at:at + SECTOR]))


def analyse(store: bytes) -> dict:
    """Header, per-region manifest and image facts, each independently hashed."""
    if len(store) != STORE_SIZE:
        raise ValueError(f"store must be {STORE_SIZE} bytes, found {len(store)}")
    result = {"header": parse_header(region(store, 0, SECTOR)), "regions": {}}
    for name, (manifest_sector, image_sector) in REGIONS.items():
        manifest_bytes = region(store, manifest_sector, MANIFEST_LEN)
        image_region = region(store, image_sector, MAX_IMAGE_LEN)
        try:
            manifest = parse_manifest(manifest_bytes)
            manifest_error = None
        except ValueError as error:
            manifest, manifest_error = None, str(error)
        facts = {"manifest": manifest, "manifest_error": manifest_error,
                 "manifest_region_sha256": sha256(manifest_bytes),
                 "image_region_sha256": sha256(image_region),
                 "image_nonzero_sectors": nonzero_sectors(image_region),
                 "image_sha256": None, "image_matches_manifest": None}
        if manifest is not None:
            image = image_region[:manifest["image_length"]]
            facts["image_sha256"] = sha256(image)
            facts["image_matches_manifest"] = facts["image_sha256"] == manifest["image_sha256"]
        result["regions"][name] = facts
    return result


def set_active(store: bytearray, active: int) -> None:
    """Host tampering of the unsigned header hint."""
    if active not in (0, 1):
        raise ValueError("active must be 0 or 1")
    struct.pack_into("<I", store, 12, active)


def flip(store: bytearray, offset: int, mask: int = 1) -> None:
    if not 0 <= offset < len(store) or not 1 <= mask <= 255:
        raise ValueError("flip outside the store")
    store[offset] ^= mask


def manifest_offset(name: str, field_offset: int) -> int:
    return REGIONS[name][0] * SECTOR + field_offset


def image_offset(name: str, byte: int) -> int:
    return REGIONS[name][1] * SECTOR + byte


def same_region(before: bytes, after: bytes, sector: int, sectors: int) -> bool:
    return region(before, sector, sectors * SECTOR) == region(after, sector, sectors * SECTOR)


def slot_identical(before: bytes, after: bytes, name: str) -> bool:
    manifest_sector, image_sector = REGIONS[name]
    return (same_region(before, after, manifest_sector, MANIFEST_SECTORS)
            and same_region(before, after, image_sector, IMAGE_CAPACITY_SECTORS))


def image_region_is(store: bytes, name: str, image: bytes) -> bool:
    """The region holds exactly `image` followed by zero padding."""
    data = region(store, REGIONS[name][1], MAX_IMAGE_LEN)
    return data[:len(image)] == image and not any(data[len(image):])


def manifest_region_is(store: bytes, name: str, manifest: bytes) -> bool:
    data = region(store, REGIONS[name][0], MANIFEST_SECTORS * SECTOR)
    return data[:MANIFEST_LEN] == manifest and not any(data[MANIFEST_LEN:])
