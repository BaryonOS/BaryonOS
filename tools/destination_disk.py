#!/usr/bin/env python3
"""Independent host oracle for the S09 destination layout (spec 2.6, R3).

A separate Python reading of docs/implementation/S09-DESTINATION-SERVICE.md
sections 2.6 and 2.7: catalog copy validation, the mount rule (mounted,
blank candidate, corrupt, failed mount) and the sealed entry extents. It
shares no code with services/destination and treats every image byte as
untrusted data: reads are bounded by the layout, counts are checked before
use and nothing in an image is executed or imported.

The functions under "Test-only encoder" build crafted copies and images for
host tests and disposable QEMU fixtures (N4, N14, N30); they are not a
product format writer.
"""
from __future__ import annotations

import json
from pathlib import Path
import struct

from volume_disk import fnv

SECTOR = 512
COPY = 2048
COPY_SECTORS = 4
AREA_SECTORS = 16
AREA = AREA_SECTORS * SECTOR
DATA_START = 16
MAGIC = b"NOSDST\x00\x01"
VERSION = 1
MAX_ENTRIES = 16
RECORD = 80
RECORDS_AT = 64
RECORDS_END = RECORDS_AT + MAX_ENTRIES * RECORD
SUM_AT = 2040
ID = 16
LABEL = 32
MIN_ARCHIVE = 80
MAX_ARCHIVE = 525_328
SLOTS = ("A", "B")


class Invalid(ValueError):
    """A catalog copy that breaks the named rule of spec 2.6."""

    def __init__(self, rule: str):
        super().__init__(rule)
        self.rule = rule


def _zero(data: bytes) -> bool:
    return not any(data)


def _sectors_of(length: int) -> int:
    return -(-length // SECTOR)


def decode_record(raw: bytes) -> dict:
    entry, first, length, trailer, sealed = struct.unpack_from("<5Q", raw, 0)
    label_length = raw[40]
    if entry < 1:
        raise Invalid("entry id 0")  # 2.3: entry ID 0 is invalid (revision 3 list)
    if not _zero(raw[41:48]):
        raise Invalid("record padding")
    if label_length == 0:
        raise Invalid("label length")  # 2.3: labels are 1-32 bytes
    if label_length > LABEL or not _zero(raw[48 + label_length:RECORD]):
        raise Invalid("label padding")
    if not MIN_ARCHIVE <= length <= MAX_ARCHIVE:
        raise Invalid("entry length")
    if sealed < 2:
        # FORMAT commits sequence 1 with no record, so no record is sealed below 2.
        raise Invalid("sealing sequence")
    return {"id": entry, "first": first, "length": length, "sectors": _sectors_of(length),
            "end": first + _sectors_of(length), "trailer": trailer, "sealed_sequence": sealed,
            "label": raw[48:48 + label_length].hex()}


def decode_copy(raw: bytes, sectors: int) -> dict:
    """Decode one 2048-byte copy for a scope of `sectors`; raise Invalid(rule)."""
    if len(raw) != COPY:
        raise Invalid("copy size")
    if raw[:8] != MAGIC:
        raise Invalid("magic")
    if struct.unpack_from("<H", raw, 8)[0] != VERSION:
        raise Invalid("version")
    if fnv(raw[:SUM_AT]) != struct.unpack_from("<Q", raw, SUM_AT)[0]:
        raise Invalid("checksum")
    count = raw[10]
    if count > MAX_ENTRIES:
        raise Invalid("count")
    if not _zero(raw[11:16]):
        raise Invalid("header padding")
    sequence, next_entry, next_free, formatted = struct.unpack_from("<4Q", raw, 32)
    if sequence == 0:
        raise Invalid("sequence")
    if not _zero(raw[RECORDS_AT + count * RECORD:SUM_AT]):
        raise Invalid("unused records")
    records = [decode_record(raw[RECORDS_AT + i * RECORD:RECORDS_AT + (i + 1) * RECORD])
               for i in range(count)]
    end = DATA_START
    previous = None
    for record in records:
        if record["sealed_sequence"] > sequence:
            raise Invalid("sealing sequence above copy")
        if previous is not None and not (record["id"] > previous["id"]
                                         and record["first"] > previous["first"]):
            raise Invalid("ascending")
        if previous is not None and record["sealed_sequence"] <= previous["sealed_sequence"]:
            raise Invalid("sealing sequence ascending")
        if record["first"] != end:
            raise Invalid("contiguous")
        end = record["end"]
        previous = record
    if next_free != end:
        raise Invalid("next_free")
    if not DATA_START <= next_free <= formatted:
        raise Invalid("next_free bounds")
    if formatted != sectors:
        raise Invalid("formatted count")
    if next_entry < 1:
        raise Invalid("next entry")
    if any(record["id"] >= next_entry for record in records):
        raise Invalid("entry id")
    return {"destination_id": raw[16:32].hex(), "sequence": sequence, "next_entry_id": next_entry,
            "next_free": next_free, "formatted_sectors": formatted, "entries": records}


def check_copy(raw: bytes, sectors: int) -> tuple[dict | None, str | None]:
    try:
        return decode_copy(raw, sectors), None
    except Invalid as error:
        return None, error.rule


def extends(high: dict, low: dict) -> bool:
    """Spec 2.6: the higher-sequence copy extends the lower one."""
    count = len(low["entries"])
    return (high["destination_id"] == low["destination_id"]
            and high["formatted_sectors"] == low["formatted_sectors"]
            and high["entries"][:count] == low["entries"]
            and high["next_free"] >= low["next_free"]
            and high["next_entry_id"] >= low["next_entry_id"])


def mount(area: bytes, sectors: int, unreadable: frozenset | set = frozenset()) -> dict:
    """Apply the mount rule to sectors 0-15 (`area`) of a scope of `sectors`.

    A sector in `unreadable`, or one missing because `area` is short, is a read
    error: the failed-mount state, never an invalid copy and never zero.
    """
    result = {"state": "failed", "slot": None, "catalog": None, "copies": [], "reason": None}
    missing = sorted(s for s in range(AREA_SECTORS) if s in unreadable or (s + 1) * SECTOR > len(area))
    if missing:
        result["reason"] = f"read error in sectors {missing}"
        return result
    raws = [bytes(area[slot * COPY:(slot + 1) * COPY]) for slot in range(2)]
    copies = [check_copy(raw, sectors) for raw in raws]
    result["copies"] = [{"slot": SLOTS[i], "valid": c is not None, "reason": why,
                         "sequence": c["sequence"] if c else None} for i, (c, why) in enumerate(copies)]
    valid = [i for i, (c, _) in enumerate(copies) if c is not None]
    if len(valid) == 1:
        chosen = valid[0]
    elif len(valid) == 2:
        a, b = copies[0][0], copies[1][0]
        if raws[0] == raws[1]:
            chosen = 0
        elif a["sequence"] == b["sequence"]:
            return dict(result, state="corrupt", reason="equal sequences, different bytes")
        else:
            chosen = 0 if a["sequence"] > b["sequence"] else 1
            if not extends(copies[chosen][0], copies[1 - chosen][0]):
                return dict(result, state="corrupt", reason="higher copy does not extend the lower")
    elif _zero(area[:AREA]):
        return dict(result, state="blank", reason="no valid copy, sectors 0-15 zero")
    else:
        return dict(result, state="corrupt", reason="no valid copy")
    return dict(result, state="mounted", slot=SLOTS[chosen], catalog=copies[chosen][0])


def read_extent(handle, record: dict, first: int = 0) -> bytes:
    """Read the sectors of one validated record (length <= MAX_ARCHIVE)."""
    handle.seek((first + record["first"]) * SECTOR)
    return handle.read(record["sectors"] * SECTOR)


def analyse(path: Path, sectors: int | None = None, first: int = 0) -> dict:
    """Mount the scope [first, first + sectors) of `path` and check each sealed extent."""
    if sectors is None:
        sectors = path.stat().st_size // SECTOR - first
    with path.open("rb") as handle:
        handle.seek(first * SECTOR)
        result = mount(handle.read(AREA), sectors)
        result.update(scope_first=first, scope_sectors=sectors)
        if result["state"] != "mounted":
            return result
        for record in result["catalog"]["entries"]:
            data = read_extent(handle, record, first)
            readable = record["end"] <= sectors and len(data) == record["sectors"] * SECTOR
            record["readable"] = readable
            record["trailer_ok"] = readable and struct.unpack_from(
                "<Q", data, record["length"] - 8)[0] == record["trailer"]
            record["padding_zero"] = readable and _zero(data[record["length"]:])
    return result


def extent_bytes(path: Path, record: dict, first: int = 0) -> bytes:
    """Entry bytes of a record returned by `analyse` (length bytes)."""
    with path.open("rb") as handle:
        return read_extent(handle, record, first)[:record["length"]]


# Test-only encoder: crafted copies and images for host tests and disposable
# QEMU fixtures. Not a product writer; the oracle's parser above is the judge.

def seal_copy(raw: bytearray) -> bytearray:
    """Recompute the FNV-64 of a (possibly crafted) copy in place."""
    struct.pack_into("<Q", raw, SUM_AT, fnv(bytes(raw[:SUM_AT])))
    return raw


def encode_record(record: dict, index: int = 0) -> bytes:
    """Encode one record; without "sealed_sequence" the record at `index` is
    sealed at index + 2, as consecutive SEALs after FORMAT (sequence 1) seal
    them, so a copy of sequence >= count + 1 is valid under spec 2.6."""
    label = bytes.fromhex(record.get("label", ""))
    if len(label) > LABEL:
        # A longer label would grow the record and shift every later field.
        raise ValueError("label over 32 bytes: patch byte 40 of a sealed copy instead")
    raw = bytearray(RECORD)
    struct.pack_into("<5QB", raw, 0, record["id"], record["first"], record["length"],
                     record.get("trailer", 0), record.get("sealed_sequence", index + 2), len(label))
    raw[48:48 + len(label)] = label
    return bytes(raw)


def encode_copy(destination_id: bytes, sequence: int, next_entry_id: int, next_free: int,
                formatted: int, records: list[dict] = ()) -> bytearray:
    raw = bytearray(COPY)
    raw[:8] = MAGIC
    struct.pack_into("<HB", raw, 8, VERSION, len(records))
    raw[16:32] = destination_id
    struct.pack_into("<4Q", raw, 32, sequence, next_entry_id, next_free, formatted)
    for index, record in enumerate(records):
        raw[RECORDS_AT + index * RECORD:RECORDS_AT + (index + 1) * RECORD] = encode_record(record, index)
    if len(raw) != COPY:
        raise ValueError(f"encoded copy is {len(raw)} bytes, not {COPY}: too many records")
    return seal_copy(raw)


def build_image(sectors: int, copy_a: bytes, copy_b: bytes, extents: dict[int, bytes] = None) -> bytearray:
    image = bytearray(sectors * SECTOR)
    image[:COPY] = copy_a
    image[COPY:2 * COPY] = copy_b
    for first, data in (extents or {}).items():
        image[first * SECTOR:first * SECTOR + len(data)] = data
    return image


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", type=Path)
    parser.add_argument("--first", type=int, default=0, help="first image sector of the granted scope")
    parser.add_argument("--sectors", type=int, help="granted scope sector count (default: to the image end)")
    args = parser.parse_args()
    if args.first < 0:
        parser.error("--first must not be negative")
    print(json.dumps(analyse(args.image, args.sectors, args.first), indent=2))
