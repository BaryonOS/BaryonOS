#!/usr/bin/env python3
"""Synthetic M6e volume images and an independent host disk oracle (format v2).

This is a separate Python implementation of the volume-core v2 on-disk rules
(table copies, sector seals with attempt digest, cross-copy choice, objects
with metadata and history, collections, chunk checksums and the app fixture
content). It never trusts guest output: the runner compares guest-reported
digests with the digests computed here from the raw image.
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
OBJECTS = 16
PER_DOMAIN = 8
DOMAINS = 2
COLLECTIONS = 4
HISTORY = 2
TABLE_SECTORS = 2 + 2 * OBJECTS
DATA_START = 2 * TABLE_SECTORS
CHUNK_SECTORS = 8
CHUNK = 4096
CHUNKS = 64
VOLUME_SECTORS = DATA_START + CHUNKS * CHUNK_SECTORS
MAX_CHUNKS = 16
MAX_LENGTH = MAX_CHUNKS * CHUNK
QUOTA = 40
NAME = 32
HEAD = 64
VERSION_BYTES = 216
COLLECTION_BYTES = 56
CLOCKS = 40
TRAILER = 488
NO_CHUNK = 0xFFFF
MAGIC = b"NOSVOL02"
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


def empty_version():
    return {"revision": 0, "length": 0, "name": b"", "content_sum": 0, "chunks": [], "sums": []}


def decode_version(raw: bytes):
    revision, length, count, name_length = struct.unpack_from("<QIHB", raw, 0)
    content_sum = struct.unpack_from("<Q", raw, 16)[0]
    name = raw[24:56]
    chunks = list(struct.unpack_from("<16H", raw, 56))
    sums = list(struct.unpack_from("<16Q", raw, 88))
    if raw[15] or count > MAX_CHUNKS or name_length > NAME or any(name[name_length:]):
        raise ValueError("version header")
    if any(c != NO_CHUNK for c in chunks[count:]) or any(sums[count:]) or any(c >= CHUNKS for c in chunks[:count]):
        raise ValueError("version chunks")
    version = {"revision": revision, "length": length, "name": name[:name_length],
               "content_sum": content_sum, "chunks": chunks[:count], "sums": sums[:count]}
    if revision == 0:
        if version != empty_version():
            raise ValueError("empty version not canonical")
    elif not valid_name(version["name"]) or length > MAX_LENGTH or count != -(-length // CHUNK):
        raise ValueError("version shape")
    return version


def decode_object(head: bytes, history: bytes, generation: int):
    revision, request_id, created, modified, tags, kind, present, collection, depth = \
        struct.unpack_from("<QQQQIHBBB", head, 0)
    if present not in (0, 1) or any(head[41:HEAD]) or any(head[HEAD + VERSION_BYTES:TRAILER]):
        raise ValueError("object head padding")
    if depth > HISTORY or collection > COLLECTIONS or revision > generation or modified > generation \
            or created > modified:
        raise ValueError("object head range")
    if any(history[HISTORY * VERSION_BYTES:TRAILER]):
        raise ValueError("history padding")
    current = decode_version(head[HEAD:HEAD + VERSION_BYTES])
    versions = [decode_version(history[i * VERSION_BYTES:(i + 1) * VERSION_BYTES]) for i in range(HISTORY)]
    entry = {"revision": revision, "request_id": request_id, "created": created, "modified": modified,
             "tags": tags, "kind": kind, "present": bool(present), "collection": collection,
             "current": current, "history": versions[:depth]}
    if revision == 0:
        if request_id or created or modified or tags or kind or present or collection or depth \
                or current != empty_version() or any(v != empty_version() for v in versions):
            raise ValueError("untouched object not empty")
        return entry
    if not request_id or not created or not modified:
        raise ValueError("object identity")
    if present:
        if not current["revision"] or current["revision"] > revision:
            raise ValueError("current version")
        newer = current["revision"]
    else:
        if current != empty_version() or collection or tags or kind:
            raise ValueError("deleted object keeps metadata")
        newer = revision + 1
    for index, version in enumerate(versions):
        if index < depth:
            if not version["revision"] or version["revision"] >= newer:
                raise ValueError("history order")
            newer = version["revision"]
        elif version != empty_version():
            raise ValueError("unused history slot")
    positions = {}
    for version in retained(entry):
        for position, chunk in enumerate(version["chunks"]):
            key = (position, version["sums"][position])
            if positions.setdefault(chunk, key) != key:
                raise ValueError("versions share a chunk inconsistently")
    return entry


def retained(entry):
    return ([entry["current"]] if entry["present"] else []) + entry["history"]


def unique_chunks(entry) -> set:
    return {chunk for version in retained(entry) for chunk in version["chunks"]}


def decode_collection(raw: bytes, generation: int):
    revision, request_id, name_length, present = struct.unpack_from("<QQBB", raw, 0)
    name = raw[24:56]
    if any(raw[18:24]) or revision > generation or (revision == 0) != (request_id == 0) \
            or name_length > NAME or any(name[name_length:]) or present not in (0, 1):
        raise ValueError("collection")
    if present and (not revision or not valid_name(name[:name_length])):
        raise ValueError("collection name")
    if not present and name_length:
        raise ValueError("deleted collection keeps a name")
    return {"revision": revision, "request_id": request_id, "present": bool(present),
            "name": name[:name_length]}


def decode_table(slot: int, sectors: list[bytes]):
    states = [classify(slot, i, s) for i, s in enumerate(sectors)]
    seals = {(g, a) for kind, g, a in states if kind == "valid"}
    if any(kind != "valid" for kind, _, _ in states) or len(seals) != 1:
        raise ValueError("table sectors not uniform")
    generation, attempt = seals.pop()
    if attempt_digest(sectors) != attempt:
        raise ValueError("table sectors from different write attempts")
    header = sectors[0]
    fields = struct.unpack_from("<8sHHHHQQHHBB", header, 0)
    clocks = list(struct.unpack_from(f"<{DOMAINS}Q", header, CLOCKS))
    if (generation % 2 != slot or fields[0] != MAGIC or fields[1:5] != (2, OBJECTS, CHUNKS, CHUNK_SECTORS)
            or fields[5] != generation or fields[6] == 0 or fields[7:11] != (MAX_CHUNKS, QUOTA, HISTORY, COLLECTIONS)
            or any(header[38:CLOCKS]) or any(header[CLOCKS + 8 * DOMAINS:TRAILER])):
        raise ValueError("table header")
    # One commit advances exactly one domain clock; object and collection
    # revisions and stamps never exceed their own domain's clock.
    if sum(clocks) != generation:
        raise ValueError("domain clocks do not add up to the generation")
    if any(sectors[1][DOMAINS * COLLECTIONS * COLLECTION_BYTES:TRAILER]):
        raise ValueError("collection sector padding")
    collections = [decode_collection(sectors[1][i * COLLECTION_BYTES:(i + 1) * COLLECTION_BYTES], generation)
                   for i in range(DOMAINS * COLLECTIONS)]
    entries = [decode_object(sectors[2 + 2 * o], sectors[3 + 2 * o], generation) for o in range(OBJECTS)]
    if any(e["revision"] > clocks[o // PER_DOMAIN] or e["modified"] > clocks[o // PER_DOMAIN]
           for o, e in enumerate(entries)) or \
            any(c["revision"] > clocks[i // COLLECTIONS] for i, c in enumerate(collections)):
        raise ValueError("revision or stamp beyond its domain clock")
    owner = {}
    for obj, entry in enumerate(entries):
        if entry["collection"] and not collections[(obj // PER_DOMAIN) * COLLECTIONS + entry["collection"] - 1]["present"]:
            raise ValueError("object in a missing collection")
        for chunk in unique_chunks(entry):
            if owner.setdefault(chunk, obj) != obj:
                raise ValueError("chunk aliasing between objects")
    for domain in range(DOMAINS):
        if sum(len(unique_chunks(e)) for e in entries[domain * PER_DOMAIN:(domain + 1) * PER_DOMAIN]) > QUOTA:
            raise ValueError("quota exceeded on disk")
    reflected = any(e["revision"] and e["request_id"] == fields[6] for e in entries) or \
        any(c["revision"] and c["request_id"] == fields[6] for c in collections)
    if not reflected:
        raise ValueError("commit request not reflected")
    return {"generation": generation, "request_id": fields[6], "clocks": clocks, "entries": entries,
            "collections": collections}


def compatible(states, g: int, uniform_invalid: bool) -> bool:
    if uniform_invalid:
        return False
    for kind, found, _ in states:
        if kind == "zero" and g != 1:
            return False
        if kind == "valid" and not (found + 1 == g or found == g + 1):
            return False
    return True


def empty_table():
    entries = [empty_entry() for _ in range(OBJECTS)]
    collections = [{"revision": 0, "request_id": 0, "present": False, "name": b""}
                   for _ in range(DOMAINS * COLLECTIONS)]
    return {"generation": 0, "request_id": 0, "clocks": [0] * DOMAINS, "entries": entries,
            "collections": collections}


def slot_states(volume: bytes, slot: int):
    sectors = [volume[(slot * TABLE_SECTORS + i) * SECTOR:(slot * TABLE_SECTORS + i + 1) * SECTOR]
               for i in range(TABLE_SECTORS)]
    return sectors, [classify(slot, i, s) for i, s in enumerate(sectors)]


def choose(volume: bytes):
    scans = []
    for slot in range(2):
        sectors, states = slot_states(volume, slot)
        try:
            table = decode_table(slot, sectors)
        except ValueError:
            table = None
        uniform = states[0][0] == "valid" and all(s == states[0] for s in states)
        scans.append({"states": states, "table": table, "uniform_invalid": uniform and table is None})
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
        kind in ("zero", "invalid") or (kind == "valid" and g == 1) for kind, g, _ in b["states"])
    if all(kind == "zero" for kind, _, _ in a["states"]) and first_attempt:
        return empty_table()
    raise ValueError("no recoverable table")


def content(volume: bytes, version) -> bytes:
    data = bytearray()
    for index, chunk in enumerate(version["chunks"]):
        used = min(CHUNK, version["length"] - index * CHUNK)
        start = (DATA_START + chunk * CHUNK_SECTORS) * SECTOR
        raw = volume[start:start + -(-used // SECTOR) * SECTOR]
        if fnv(raw[:used]) != version["sums"][index] or any(raw[used:]):
            raise ValueError("chunk checksum or padding mismatch")
        data += raw[:used]
    if fnv(bytes(data)) != version["content_sum"]:
        raise ValueError("content checksum mismatch")
    return bytes(data)


def domain_digest(table, domain: int) -> int:
    """Same digest the guest app reports (objects, then collection masks)."""
    digest = FNV_OFFSET
    entries = table["entries"][domain * PER_DOMAIN:(domain + 1) * PER_DOMAIN]
    for index, e in enumerate(entries):
        if e["present"]:
            current = e["current"]
            digest = fnv(bytes([index, 1]), digest)
            digest = fnv(struct.pack("<QQQ", e["revision"], current["length"], current["content_sum"]), digest)
            digest = fnv(current["name"], digest)
            digest = fnv(bytes([e["collection"], len(e["history"])]), digest)
            digest = fnv(struct.pack("<IH", e["tags"], e["kind"]), digest)
        else:
            digest = fnv(bytes([index, 0]), digest)
            digest = fnv(struct.pack("<Q", e["revision"]), digest)
    for collection in range(1, COLLECTIONS + 1):
        mask = sum(1 << i for i, e in enumerate(entries) if e["present"] and e["collection"] == collection)
        digest = fnv(bytes([0xC0 | collection, mask]), digest)
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
        versions = []
        for version in retained(entry):
            data = content(volume, version)
            # Content is written at the object revision that created this
            # version, never later; an empty document matches its own revision.
            written = next((r for r in range(version["revision"], 0, -1)
                            if data == fixture(obj, r, version["length"])), None)
            if written is None:
                raise ValueError(f"object {obj} version {version['revision']} is not a written fixture")
            versions.append({"revision": version["revision"], "length": version["length"],
                             "name": version["name"].decode("utf-8"), "content_revision": written,
                             "chunks": version["chunks"]})
        objects.append({"object": obj, "revision": entry["revision"], "present": entry["present"],
                        "collection": entry["collection"], "tags": entry["tags"], "kind": entry["kind"],
                        "request_id": entry["request_id"], "versions": versions})
    used = sum(len(unique_chunks(e)) for e in table["entries"])
    collections = [{"index": i % COLLECTIONS + 1, "domain": i // COLLECTIONS, "present": c["present"],
                    "revision": c["revision"], "name": c["name"].decode("utf-8")}
                   for i, c in enumerate(table["collections"])]
    # Generation per table sector of both copies (0 = zero, -1 = invalid),
    # so a runner can tell a torn table write from an untouched copy.
    slots = [[g if kind == "valid" else (0 if kind == "zero" else -1)
              for kind, g, _ in slot_states(volume, slot)[1]] for slot in range(2)]
    return {"generation": table["generation"], "clocks": table["clocks"], "objects": objects,
            "collections": collections, "chunks_used": used, "slots": slots,
            "digests": [domain_digest(table, d) for d in range(DOMAINS)]}


def encode_version(version) -> bytes:
    raw = bytearray(VERSION_BYTES)
    struct.pack_into("<QIHB", raw, 0, version["revision"], version["length"], len(version["chunks"]),
                     len(version["name"]))
    struct.pack_into("<Q", raw, 16, version["content_sum"])
    raw[24:24 + len(version["name"])] = version["name"]
    chunks = list(version["chunks"]) + [NO_CHUNK] * (MAX_CHUNKS - len(version["chunks"]))
    sums = list(version["sums"]) + [0] * (MAX_CHUNKS - len(version["sums"]))
    struct.pack_into("<16H", raw, 56, *chunks)
    struct.pack_into("<16Q", raw, 88, *sums)
    return bytes(raw)


def empty_entry():
    return {"revision": 0, "request_id": 0, "created": 0, "modified": 0, "tags": 0, "kind": 0,
            "present": False, "collection": 0, "current": empty_version(), "history": []}


def encode_table(generation: int, request_id: int, entries, collections=None, clocks=None) -> list[bytes]:
    """Test/fixture encoder (independent of the Rust encoder). Without explicit
    `clocks`, every commit is attributed to domain 0."""
    clocks = clocks or [generation] + [0] * (DOMAINS - 1)
    collections = collections or [{"revision": 0, "request_id": 0, "present": False, "name": b""}
                                  for _ in range(DOMAINS * COLLECTIONS)]
    slot = generation % 2
    header = bytearray(SECTOR)
    struct.pack_into("<8sHHHHQQHHBB", header, 0, MAGIC, 2, OBJECTS, CHUNKS, CHUNK_SECTORS,
                     generation, request_id, MAX_CHUNKS, QUOTA, HISTORY, COLLECTIONS)
    struct.pack_into(f"<{DOMAINS}Q", header, CLOCKS, *clocks)
    sectors = [header]
    sector = bytearray(SECTOR)
    for index, c in enumerate(collections):
        struct.pack_into("<QQBB", sector, index * COLLECTION_BYTES, c["revision"], c["request_id"],
                         len(c["name"]), int(c["present"]))
        sector[index * COLLECTION_BYTES + 24:index * COLLECTION_BYTES + 24 + len(c["name"])] = c["name"]
    sectors.append(sector)
    for e in entries:
        head = bytearray(SECTOR)
        struct.pack_into("<QQQQIHBBB", head, 0, e["revision"], e["request_id"], e["created"], e["modified"],
                         e["tags"], e["kind"], int(e["present"]), e["collection"], len(e["history"]))
        head[HEAD:HEAD + VERSION_BYTES] = encode_version(e["current"])
        history = bytearray(SECTOR)
        for index in range(HISTORY):
            version = e["history"][index] if index < len(e["history"]) else empty_version()
            history[index * VERSION_BYTES:(index + 1) * VERSION_BYTES] = encode_version(version)
        sectors += [head, history]
    attempt = attempt_digest(sectors)
    for index, sector in enumerate(sectors):
        struct.pack_into("<QQ", sector, TRAILER, generation, attempt)
        struct.pack_into("<Q", sector, 504, sector_sum(slot, index, bytes(sector)))
    return [bytes(s) for s in sectors]


def create_image(path: Path, variant: str = "empty") -> dict:
    """Create a fresh 1 MiB image: canaries outside the scope, zero volume.

    `corrupt` writes a fully sealed generation-1 table whose current version
    has a non-canonical name, which recovery must refuse before READY.
    """
    if path.exists():
        raise FileExistsError(path)
    image = bytearray()
    for index in range(IMAGE_SECTORS):
        image += bytes(SECTOR) if FIRST <= index < FIRST + VOLUME_SECTORS else canary(index)
    if variant == "corrupt":
        entries = [empty_entry() for _ in range(OBJECTS)]
        current = dict(empty_version(), revision=1, name=b"bad/name", content_sum=FNV_OFFSET)
        entries[0] = dict(empty_entry(), revision=1, request_id=5, created=1, modified=1, present=True,
                          current=current)
        for index, sector in enumerate(encode_table(1, 5, entries)):
            start = (FIRST + TABLE_SECTORS + index) * SECTOR
            image[start:start + SECTOR] = sector
    elif variant != "empty":
        raise ValueError("unknown volume variant")
    path.write_bytes(bytes(image))
    info = {"identity": uuid.uuid4().hex, "variant": variant, "sha256": hashlib.sha256(image).hexdigest()}
    path.with_suffix(".json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
    return info


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", type=Path)
    args = parser.parse_args()
    print(json.dumps(analyse(args.image.read_bytes()), indent=2))
