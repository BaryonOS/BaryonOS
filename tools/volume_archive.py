#!/usr/bin/env python3
"""Independent reader of the S09 archive format v1.

Follows docs/implementation/S09-BACKUP-ARCHIVE.md byte for byte and shares no
code with volume-core, so the native gate can check guest-written archives
against the documented format rather than against the implementation. Any
deviation raises ValueError.
"""
from __future__ import annotations
import struct

from m6c_disk import valid_name
from volume_disk import fnv

MAGIC = b"NOSBAK\x00\x01"
END = b"NOSBAKZ\x00"
HEADER, COLLECTION_RECORD, OBJECT_RECORD, TRAILER = 64, 48, 96, 16
OBJECTS, COLLECTIONS, MAX_LENGTH, NAME = 8, 4, 65536, 32


def _zero(data: bytes) -> bool:
    return not any(data)


def _sealed(record: bytes) -> bool:
    return fnv(record[:-8]) == struct.unpack_from("<Q", record, len(record) - 8)[0]


def _take(data: bytes, at: int, size: int) -> bytes:
    if at + size > len(data):
        raise ValueError("archive truncated")
    return data[at:at + size]


def stored_length(header: bytes) -> int:
    """Archive length implied by a header's counts (header not yet trusted)."""
    if len(header) < HEADER:
        raise ValueError("archive truncated")
    return (HEADER + TRAILER + header[13] * COLLECTION_RECORD + header[12] * OBJECT_RECORD
            + struct.unpack_from("<Q", header, 24)[0])


def parse(data: bytes) -> dict:
    header = _take(data, 0, HEADER)
    count, collection_count = header[12], header[13]
    generation, content_bytes = struct.unpack_from("<QQ", header, 16)
    if (header[:8] != MAGIC or struct.unpack_from("<H", header, 8)[0] != 1
            or not _zero(header[10:12]) or not _zero(header[14:16]) or not _zero(header[32:56])
            or count > OBJECTS or collection_count > COLLECTIONS
            or content_bytes > count * MAX_LENGTH or not _sealed(header)):
        raise ValueError("invalid archive header")
    at = HEADER
    collections, previous = {}, 0
    for _ in range(collection_count):
        record = _take(data, at, COLLECTION_RECORD)
        at += COLLECTION_RECORD
        index, length = record[0], record[1]
        if (not _sealed(record) or not previous < index <= COLLECTIONS or length > NAME
                or not valid_name(record[8:8 + length]) or not _zero(record[2:8])
                or not _zero(record[8 + length:40])):
            raise ValueError("invalid collection record")
        previous = index
        collections[index] = record[8:8 + length].decode("utf-8")
    objects, next_slot, total = [], 0, 0
    for _ in range(count):
        record = _take(data, at, OBJECT_RECORD)
        at += OBJECT_RECORD
        slot, name_length, collection = record[0], record[1], record[2]
        kind = struct.unpack_from("<H", record, 4)[0]
        tags, length = struct.unpack_from("<II", record, 8)
        revision, created, modified, content_sum = struct.unpack_from("<QQQQ", record, 16)
        name = record[48:48 + name_length] if name_length <= NAME else b""
        if (not _sealed(record) or not next_slot <= slot < OBJECTS or name_length > NAME
                or not valid_name(name) or not _zero(record[48 + name_length:80])
                or not (collection == 0 or collection in collections) or length > MAX_LENGTH
                or revision == 0 or record[3] != 0 or not _zero(record[6:8])
                or not _zero(record[80:88])):
            raise ValueError("invalid object record")
        next_slot = slot + 1
        total += length
        if total > content_bytes:
            raise ValueError("content exceeds the header total")
        content = _take(data, at, length)
        at += length
        if fnv(content) != content_sum:
            raise ValueError(f"content checksum of slot {slot} differs")
        objects.append(dict(slot=slot, name=name.decode("utf-8"), collection=collection, kind=kind,
                            tags=tags, length=length, revision=revision, created=created,
                            modified=modified, content_sum=content_sum, content=content))
    trailer = _take(data, at, TRAILER)
    if (total != content_bytes or trailer[:8] != END
            or struct.unpack_from("<Q", trailer, 8)[0] != fnv(data[:at])):
        raise ValueError("invalid archive trailer")
    if len(data) != at + TRAILER:
        raise ValueError("bytes after the archive trailer")
    return dict(generation=generation, content_bytes=content_bytes, collections=collections,
                objects=objects, length=len(data))


def expected_restore(archive: dict, names: dict[int, str], collections: dict[int, str]) -> dict:
    """End state that restore must reach by the spec rule in a domain that already holds the
    present objects `names` (slot -> name, no tombstones) and `collections` (index -> name).

    An occupied slot or an already used name is a conflict and stays untouched. A collection
    index that holds another name is never renamed: its object is restored outside a
    collection. A missing collection is created only for an object that is restored into it.
    Restore writes content (revision 1), then metadata (revision 2) if any is set.
    """
    used = set(names.values())
    state = dict(objects={}, collections=dict(collections), slot_conflicts=set(),
                 name_conflicts=set(), uncollected=set(), reused=set())
    for item in archive["objects"]:
        slot, index = item["slot"], item["collection"]
        if slot in names or item["name"] in used:
            state["slot_conflicts" if slot in names else "name_conflicts"].add(slot)
            continue
        if index:
            name = archive["collections"][index]
            if index not in collections:
                state["collections"][index] = name
            elif collections[index] == name:
                state["reused"].add(slot)
            else:
                state["uncollected"].add(slot)
                index = 0
        state["objects"][slot] = dict(name=item["name"], length=item["length"], content=item["content"],
                                      collection=index, tags=item["tags"], kind=item["kind"],
                                      revision=2 if item["tags"] or item["kind"] or index else 1)
    return state


# --- Format v2 (docs/implementation/S09-ARCHIVE-V2.md). Specification reader only: no writer,
# no restore. Everything above this line is the unchanged v1 reader.

VERSION_V2 = 2
HEADER_V2, COLLECTION_RECORD_V2, OBJECT_RECORD_V2, VERSION_RECORD_V2 = 64, 64, 128, 64
HISTORY, QUOTA, CHUNK = 2, 40, 4096
VERSIONS_V2 = OBJECTS * (HISTORY + 1)
FLAG_TOMBSTONES, FLAG_ACCOUNTED = 0x01, 0x02
# Every version's chunks lie within its object's chunks, and a domain holds at most QUOTA chunks.
CONTENT_V2 = (HISTORY + 1) * QUOTA * CHUNK
MAX_ARCHIVE_V2 = (HEADER_V2 + COLLECTIONS * COLLECTION_RECORD_V2 + OBJECTS * OBJECT_RECORD_V2
                  + VERSIONS_V2 * VERSION_RECORD_V2 + CONTENT_V2 + TRAILER)


def stored_length_v2(header: bytes) -> int:
    """Archive length implied by a v2 header's counts (header not yet trusted)."""
    if len(header) < HEADER_V2:
        raise ValueError("archive truncated")
    return (HEADER_V2 + TRAILER + header[13] * COLLECTION_RECORD_V2 + header[12] * OBJECT_RECORD_V2
            + struct.unpack_from("<H", header, 14)[0] * VERSION_RECORD_V2
            + struct.unpack_from("<Q", header, 24)[0])


def parse_v2(data: bytes) -> dict:
    header = _take(data, 0, HEADER_V2)
    flags, count, collection_count = header[10], header[12], header[13]
    version_count = struct.unpack_from("<H", header, 14)[0]
    generation, content_bytes, clock = struct.unpack_from("<QQQ", header, 16)
    if (header[:8] != MAGIC or struct.unpack_from("<H", header, 8)[0] != VERSION_V2
            or flags & ~(FLAG_TOMBSTONES | FLAG_ACCOUNTED) or header[11] != 0
            or not _zero(header[40:56]) or count > OBJECTS or collection_count > COLLECTIONS
            or version_count > VERSIONS_V2 or content_bytes > version_count * MAX_LENGTH
            or not _sealed(header)):
        raise ValueError("invalid archive header")
    if content_bytes > CONTENT_V2:
        raise ValueError("content exceeds the v2 bound")
    tombstones, accounted = bool(flags & FLAG_TOMBSTONES), bool(flags & FLAG_ACCOUNTED)
    at, previous, collections = HEADER_V2, 0, {}
    for _ in range(collection_count):
        record = _take(data, at, COLLECTION_RECORD_V2)
        at += COLLECTION_RECORD_V2
        index, present, length = record[0], record[1], record[2]
        revision = struct.unpack_from("<Q", record, 8)[0]
        name = record[16:16 + length] if length <= NAME else b""
        if (not _sealed(record) or not previous < index <= COLLECTIONS or present > 1
                or length > NAME or not _zero(record[3:8]) or not _zero(record[16 + length:56])
                or revision == 0 or (not valid_name(name) if present else length != 0)):
            raise ValueError("invalid collection record")
        if not present and not accounted:
            raise ValueError("deleted collection without the accounted flag")
        if revision > clock:
            raise ValueError("collection revision exceeds the source clock")
        previous = index
        collections[index] = dict(present=bool(present), revision=revision, name=name.decode("utf-8"))
    objects, next_slot, total, versions_seen, chunks = [], 0, 0, 0, 0
    for _ in range(count):
        record = _take(data, at, OBJECT_RECORD_V2)
        at += OBJECT_RECORD_V2
        slot, present, collection, history_count = record[0], record[1], record[2], record[3]
        kind = struct.unpack_from("<H", record, 4)[0]
        tags = struct.unpack_from("<I", record, 8)[0]
        revision, created, modified = struct.unpack_from("<QQQ", record, 16)
        if (not _sealed(record) or not next_slot <= slot < OBJECTS or present > 1
                or history_count > HISTORY or not _zero(record[6:8]) or not _zero(record[12:16])
                or not _zero(record[40:120]) or revision == 0
                or (collection != 0 and not collections.get(collection, {}).get("present"))
                or (not present and (collection or kind or tags))):
            raise ValueError("invalid object record")
        if not present and history_count and not tombstones:
            raise ValueError("tombstone without the tombstone flag")
        if not present and not history_count and not accounted:
            raise ValueError("content-free record without the accounted flag")
        if created == 0:
            raise ValueError("created stamp is zero")
        if created > modified:
            raise ValueError("created stamp after modified stamp")
        if modified > clock:
            raise ValueError("modified stamp exceeds the source clock")
        if revision > clock:
            raise ValueError("object revision exceeds the source clock")
        next_slot = slot + 1
        versions, newer = [], None
        for _ in range(present + history_count):
            version = _take(data, at, VERSION_RECORD_V2)
            at += VERSION_RECORD_V2
            name_length = version[0]
            length = struct.unpack_from("<I", version, 4)[0]
            version_revision, content_sum = struct.unpack_from("<QQ", version, 8)
            name = version[24:24 + name_length] if name_length <= NAME else b""
            if (not _sealed(version) or name_length > NAME or not valid_name(name)
                    or not _zero(version[1:4]) or not _zero(version[24 + name_length:56])
                    or length > MAX_LENGTH):
                raise ValueError("invalid version record")
            if version_revision == 0:
                raise ValueError("version revision is zero")
            if newer is None and version_revision > revision:
                raise ValueError("newest version revision exceeds the object revision")
            if newer is not None and version_revision >= newer:
                raise ValueError("version revisions not strictly decreasing")
            newer = version_revision
            versions_seen += 1
            total += length
            if versions_seen > version_count or total > content_bytes:
                raise ValueError("records exceed the header counts")
            content = _take(data, at, length)
            at += length
            if fnv(content) != content_sum:
                raise ValueError(f"content checksum of a slot {slot} version differs")
            versions.append(dict(revision=version_revision, name=name.decode("utf-8"),
                                 length=length, content_sum=content_sum, content=content))
        objects.append(dict(slot=slot, present=bool(present), collection=collection, kind=kind,
                            tags=tags, revision=revision, created=created, modified=modified,
                            versions=versions))
        # An object's unique chunks are at least its largest version's chunk count.
        chunks += max((-(-v["length"] // CHUNK) for v in versions), default=0)
        if chunks > QUOTA:
            raise ValueError("chunk usage exceeds the domain quota")
    trailer = _take(data, at, TRAILER)
    if (total != content_bytes or versions_seen != version_count or trailer[:8] != END
            or struct.unpack_from("<Q", trailer, 8)[0] != fnv(data[:at])):
        raise ValueError("invalid archive trailer")
    if len(data) != at + TRAILER:
        raise ValueError("bytes after the archive trailer")
    if accounted and (sum(o["revision"] for o in objects)
                      + sum(c["revision"] for c in collections.values())) % (1 << 64) != generation:
        raise ValueError("revision sum differs from the domain generation")
    return dict(version=VERSION_V2, tombstones=tombstones, accounted=accounted,
                generation=generation, clock=clock, content_bytes=content_bytes,
                collections=collections, objects=objects, length=len(data))


def parse_any(data: bytes) -> dict:
    """Dispatch on the version field; anything that is not version 2 takes the unchanged v1 path."""
    if data[8:10] == struct.pack("<H", VERSION_V2):
        return parse_v2(data)
    return parse(data)
