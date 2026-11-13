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
