#!/usr/bin/env python3
"""Synthetic M6e volume images and an independent host disk oracle.

This is a separate Python implementation of the volume-core v1 on-disk rules
(table copies, sector seals, cross-copy choice, entries, chunk checksums and
the app fixture content). It never trusts guest output: the runner compares
guest-reported digests with the digests computed here from the raw image.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import struct
import uuid

SECTOR = 512
IMAGE_SECTORS = 2048
FIRST = 4
TABLE_SECTORS = 9
DATA_START = 18
CHUNK_SECTORS = 8
CHUNK = 4096
CHUNKS = 64
VOLUME_SECTORS = DATA_START + CHUNKS * CHUNK_SECTORS
OBJECTS = 16
PER_DOMAIN = 8
MAX_CHUNKS = 16
MAX_LENGTH = MAX_CHUNKS * CHUNK
QUOTA = 40
NAME = 32
ENTRY = 240
TRAILER = 488
NO_CHUNK = 0xFFFF
MAGIC = b"NOSVOL01"
FNV_OFFSET = 0xCBF29CE484222325
FNV_PRIME = 0x100000001B3
MASK = (1 << 64) - 1


def fnv(data: bytes, value: int = FNV_OFFSET) -> int:
    for byte in data:
        value = ((value ^ byte) * FNV_PRIME) & MASK
    return value


def canary(index: int) -> bytes:
    seed = hashlib.sha256(b"NOS-M6E-CANARY:" + index.to_bytes(4, "little")).digest()
    return (seed * (SECTOR // len(seed)))[:SECTOR]


def fixture_byte(obj: int, revision: int, offset: int) -> int:
    return (obj * 31 + revision * 7 + offset * 13 + offset // 251) & 0xFF


def fixture(obj: int, revision: int, length: int) -> bytes:
    return bytes(fixture_byte(obj, revision, i) for i in range(length))


def valid_name(name: bytes) -> bool:
    if not 0 < len(name) <= NAME:
        return False
    try:
        text = name.decode("utf-8")
    except UnicodeDecodeError:
        return False
    if text in (".", ".."):
        return False
    for ch in text:
        code = ord(ch)
        if code < 0x20 or 0x7F <= code < 0xA0 or ch in "/\\":
            return False
        if 0x202A <= code <= 0x202E or 0x2066 <= code <= 0x2069:
            return False
    return True


def sector_sum(slot: int, index: int, sector: bytes) -> int:
    return fnv(sector[:504], fnv(bytes([slot, index])))


def classify(slot: int, index: int, sector: bytes):
    """(kind, generation, attempt digest) of one table sector."""
    if sector == bytes(SECTOR):
        return ("zero", 0, 0)
    generation, attempt = struct.unpack_from("<QQ", sector, TRAILER)
    if generation == 0 or struct.unpack_from("<Q", sector, 504)[0] != sector_sum(slot, index, sector):
        return ("invalid", 0, 0)
    return ("valid", generation, attempt)


def attempt_digest(sectors) -> int:
    """Digest over the payloads of every sector of one copy (one attempt)."""
    digest = FNV_OFFSET
    for sector in sectors:
        digest = fnv(sector[:TRAILER], digest)
    return digest


def decode_entry(raw: bytes, generation: int):
    revision, request_id, length = struct.unpack_from("<QQI", raw, 0)
    present, name_length, chunk_count = raw[20], raw[21], struct.unpack_from("<H", raw, 22)[0]
    content_sum = struct.unpack_from("<Q", raw, 24)[0]
    name = raw[32:64]
    chunks = list(struct.unpack_from("<16H", raw, 64))
    sums = list(struct.unpack_from("<16Q", raw, 96))
    if present not in (0, 1) or any(raw[224:ENTRY]) or revision > generation:
        raise ValueError("entry not canonical")
    if (revision == 0) != (request_id == 0) or name_length > NAME or any(name[name_length:]):
        raise ValueError("entry identity not canonical")
    if chunk_count > MAX_CHUNKS or any(c != NO_CHUNK for c in chunks[chunk_count:]) \
            or any(sums[chunk_count:]) or any(c >= CHUNKS for c in chunks[:chunk_count]):
        raise ValueError("entry chunks not canonical")
    if present:
        if revision == 0 or not valid_name(name[:name_length]) or length > MAX_LENGTH \
                or chunk_count != -(-length // CHUNK):
            raise ValueError("present entry shape")
    elif length or name_length or chunk_count or content_sum:
        raise ValueError("tombstone shape")
    return dict(revision=revision, request_id=request_id, length=length, present=bool(present),
                name=name[:name_length], chunks=chunks[:chunk_count], sums=sums[:chunk_count],
                content_sum=content_sum)


def decode_table(slot: int, sectors: list[bytes]):
    states = [classify(slot, i, s) for i, s in enumerate(sectors)]
    seals = {(g, a) for kind, g, a in states if kind == "valid"}
    if any(kind != "valid" for kind, _, _ in states) or len(seals) != 1:
        raise ValueError("table sectors not uniform")
    generation, attempt = seals.pop()
    if attempt_digest(sectors) != attempt:
        raise ValueError("table sectors from different write attempts")
    header = sectors[0]
    fields = struct.unpack_from("<8sHHHHQQHH", header, 0)
    if (generation % 2 != slot or fields[0] != MAGIC or fields[1:5] != (1, OBJECTS, CHUNKS, CHUNK_SECTORS)
            or fields[5] != generation or fields[6] == 0 or fields[7:9] != (MAX_CHUNKS, QUOTA)
            or any(header[36:TRAILER])):
        raise ValueError("table header")
    entries = []
    for sector in sectors[1:]:
        if any(sector[2 * ENTRY:TRAILER]):
            raise ValueError("entry sector padding")
        for half in range(2):
            entries.append(decode_entry(sector[half * ENTRY:(half + 1) * ENTRY], generation))
    used = [c for e in entries for c in e["chunks"]]
    if len(used) != len(set(used)):
        raise ValueError("chunk aliasing")
    for domain in range(2):
        if sum(len(e["chunks"]) for e in entries[domain * 8:(domain + 1) * 8]) > QUOTA:
            raise ValueError("quota exceeded on disk")
    if not any(e["revision"] and e["request_id"] == fields[6] for e in entries):
        raise ValueError("commit request not reflected")
    return dict(generation=generation, request_id=fields[6], entries=entries)


def compatible(states, g: int, uniform_invalid: bool) -> bool:
    if uniform_invalid:
        return False
    for kind, found, _ in states:
        if kind == "zero" and g != 1:
            return False
        if kind == "valid" and not (found + 1 == g or found == g + 1):
            return False
    return True


def choose(volume: bytes):
    scans = []
    for slot in range(2):
        sectors = [volume[(slot * TABLE_SECTORS + i) * SECTOR:(slot * TABLE_SECTORS + i + 1) * SECTOR]
                   for i in range(TABLE_SECTORS)]
        states = [classify(slot, i, s) for i, s in enumerate(sectors)]
        try:
            table = decode_table(slot, sectors)
        except ValueError:
            table = None
        uniform = states[0][0] == "valid" and all(s == states[0] for s in states)
        scans.append(dict(states=states, table=table, uniform_invalid=uniform and table is None))
    a, b = scans
    if a["table"] and b["table"]:
        newer, older = sorted((a["table"], b["table"]), key=lambda t: -t["generation"])
        if newer["generation"] != older["generation"] + 1:
            raise ValueError("two tables with unrelated generations")
        return newer
    for mine, other in ((a, b), (b, a)):
        if mine["table"] and not other["table"]:
            if not compatible(other["states"], mine["table"]["generation"], other["uniform_invalid"]):
                raise ValueError("other table copy unexplained")
            return mine["table"]
    first_attempt = not b["uniform_invalid"] and all(
        kind == "zero" or kind == "invalid" or (kind == "valid" and g == 1) for kind, g, _ in b["states"])
    if all(kind == "zero" for kind, _, _ in a["states"]) and first_attempt:
        return dict(generation=0, request_id=0, entries=[dict(
            revision=0, request_id=0, length=0, present=False, name=b"", chunks=[], sums=[],
            content_sum=0) for _ in range(OBJECTS)])
    raise ValueError("no recoverable table")


def content(volume: bytes, entry) -> bytes:
    data = bytearray()
    for index, chunk in enumerate(entry["chunks"]):
        used = min(CHUNK, entry["length"] - index * CHUNK)
        start = (DATA_START + chunk * CHUNK_SECTORS) * SECTOR
        raw = volume[start:start + -(-used // SECTOR) * SECTOR]
        if fnv(raw[:used]) != entry["sums"][index] or any(raw[used:]):
            raise ValueError("chunk checksum or padding mismatch")
        data += raw[:used]
    if fnv(bytes(data)) != entry["content_sum"]:
        raise ValueError("content checksum mismatch")
    return bytes(data)


def domain_digest(entries, domain: int) -> int:
    digest = FNV_OFFSET
    for index in range(PER_DOMAIN):
        e = entries[domain * PER_DOMAIN + index]
        if e["present"]:
            digest = fnv(bytes([index, 1]), digest)
            digest = fnv(struct.pack("<QQQ", e["revision"], e["length"], e["content_sum"]), digest)
            digest = fnv(e["name"], digest)
        else:
            digest = fnv(bytes([index, 0]), digest)
            digest = fnv(struct.pack("<Q", e["revision"]), digest)
    return digest


def analyse(image: bytes):
    if len(image) != IMAGE_SECTORS * SECTOR:
        raise ValueError("image size changed")
    outside = [i for i in range(IMAGE_SECTORS)
               if not FIRST <= i < FIRST + VOLUME_SECTORS and image[i * SECTOR:(i + 1) * SECTOR] != canary(i)]
    if outside:
        raise ValueError(f"sectors outside the granted scope changed: {outside[:8]}")
    volume = image[FIRST * SECTOR:(FIRST + VOLUME_SECTORS) * SECTOR]
    table = choose(volume)
    objects = []
    for obj, entry in enumerate(table["entries"]):
        item = dict(object=obj, revision=entry["revision"], present=entry["present"],
                    length=entry["length"], name=entry["name"].decode("utf-8"),
                    chunks=entry["chunks"], request_id=entry["request_id"])
        if entry["present"]:
            data = content(volume, entry)
            item["content_revision"] = next(
                (r for r in range(entry["revision"], 0, -1) if data == fixture(obj, r, entry["length"])), None)
            if item["content_revision"] is None:
                raise ValueError(f"object {obj} content is not a written fixture revision")
        objects.append(item)
    used = sum(len(e["chunks"]) for e in table["entries"])
    return dict(generation=table["generation"], objects=objects, chunks_used=used,
                digests=[domain_digest(table["entries"], d) for d in range(2)])


def encode_table(generation: int, request_id: int, entries) -> list[bytes]:
    """Test/fixture encoder (independent of the Rust encoder)."""
    slot = generation % 2
    sectors = []
    header = bytearray(SECTOR)
    struct.pack_into("<8sHHHHQQHH", header, 0, MAGIC, 1, OBJECTS, CHUNKS, CHUNK_SECTORS,
                     generation, request_id, MAX_CHUNKS, QUOTA)
    sectors.append(header)
    for index in range(8):
        sector = bytearray(SECTOR)
        for half in range(2):
            e = entries[index * 2 + half]
            raw = bytearray(ENTRY)
            name = e["name"]
            struct.pack_into("<QQIBBHQ", raw, 0, e["revision"], e["request_id"], e["length"],
                             int(e["present"]), len(name), len(e["chunks"]), e["content_sum"])
            raw[32:32 + len(name)] = name
            chunks = list(e["chunks"]) + [NO_CHUNK] * (MAX_CHUNKS - len(e["chunks"]))
            sums = list(e["sums"]) + [0] * (MAX_CHUNKS - len(e["sums"]))
            struct.pack_into("<16H", raw, 64, *chunks)
            struct.pack_into("<16Q", raw, 96, *sums)
            sector[half * ENTRY:(half + 1) * ENTRY] = raw
        sectors.append(sector)
    attempt = attempt_digest(sectors)
    for index, sector in enumerate(sectors):
        struct.pack_into("<QQ", sector, TRAILER, generation, attempt)
        struct.pack_into("<Q", sector, 504, sector_sum(slot, index, bytes(sector)))
    return [bytes(s) for s in sectors]


def empty_entry():
    return dict(revision=0, request_id=0, length=0, present=False, name=b"", chunks=[], sums=[],
                content_sum=0)


def create_image(path: Path, variant: str = "empty") -> dict:
    """Create a fresh 1 MiB image: canaries outside the scope, zero volume.

    `corrupt` writes a fully sealed generation-1 table whose entry is not
    canonical (an invalid name), which recovery must refuse before READY.
    """
    if path.exists():
        raise FileExistsError(path)
    image = bytearray()
    for index in range(IMAGE_SECTORS):
        image += bytes(SECTOR) if FIRST <= index < FIRST + VOLUME_SECTORS else canary(index)
    if variant == "corrupt":
        entries = [empty_entry() for _ in range(OBJECTS)]
        entries[0] = dict(revision=1, request_id=5, length=0, present=True, name=b"bad/name",
                          chunks=[], sums=[], content_sum=FNV_OFFSET)
        for index, sector in enumerate(encode_table(1, 5, entries)):
            start = (FIRST + TABLE_SECTORS + index) * SECTOR
            image[start:start + SECTOR] = sector
    elif variant != "empty":
        raise ValueError("unknown volume variant")
    path.write_bytes(bytes(image))
    info = dict(identity=uuid.uuid4().hex, variant=variant, sha256=hashlib.sha256(image).hexdigest())
    path.with_suffix(".json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
    return info


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", type=Path)
    args = parser.parse_args()
    print(json.dumps(analyse(args.image.read_bytes()), indent=2))
