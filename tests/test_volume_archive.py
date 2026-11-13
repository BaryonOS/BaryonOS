"""Host tests for the independent S09 archive reader (tools/volume_archive.py, format v1).

The encoder below follows docs/implementation/S09-BACKUP-ARCHIVE.md directly;
it shares no code with volume-core, so the reader is tested against the spec.
"""
from __future__ import annotations
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import volume_archive as va  # noqa: E402
from volume_disk import fnv  # noqa: E402


def seal(record: bytearray) -> bytes:
    struct.pack_into("<Q", record, len(record) - 8, fnv(bytes(record[:-8])))
    return bytes(record)


def encode(collections, objects, generation=7) -> bytes:
    """collections: [(index, name)]; objects: [(slot, name, collection, kind, tags, revision, content)]."""
    header = bytearray(va.HEADER)
    header[:8] = va.MAGIC
    struct.pack_into("<H", header, 8, 1)
    header[12], header[13] = len(objects), len(collections)
    struct.pack_into("<QQ", header, 16, generation, sum(len(o[6]) for o in objects))
    out = bytearray(seal(header))
    for index, name in collections:
        record = bytearray(va.COLLECTION_RECORD)
        record[0], record[1] = index, len(name)
        record[8:8 + len(name)] = name
        out += seal(record)
    for slot, name, collection, kind, tags, revision, content in objects:
        record = bytearray(va.OBJECT_RECORD)
        record[0], record[1], record[2] = slot, len(name), collection
        struct.pack_into("<H", record, 4, kind)
        struct.pack_into("<IIQQQQ", record, 8, tags, len(content), revision, 1, 2, fnv(content))
        record[48:48 + len(name)] = name
        out += seal(record) + content
    return bytes(out + va.END + struct.pack("<Q", fnv(bytes(out))))


SAMPLE = encode([(1, "Werk".encode()), (3, "Privé".encode())],
                [(0, "Leeg".encode(), 0, 0, 0, 1, b""),
                 (2, "Notitie ✓".encode(), 3, 4, 0x11, 2, bytes(range(200))),
                 (7, b"Laatste", 1, 0, 0, 9, b"x" * 513)])
# (offset, size) of the header, the two collection records and the three object records.
HEADER_AT, COLLECTION_AT, OBJECT_AT = 0, va.HEADER, va.HEADER + 2 * va.COLLECTION_RECORD
SPANS = [(HEADER_AT, va.HEADER), (COLLECTION_AT, va.COLLECTION_RECORD),
         (COLLECTION_AT + va.COLLECTION_RECORD, va.COLLECTION_RECORD), (OBJECT_AT, va.OBJECT_RECORD),
         (OBJECT_AT + va.OBJECT_RECORD, va.OBJECT_RECORD), (OBJECT_AT + 2 * va.OBJECT_RECORD + 200, va.OBJECT_RECORD)]


def reseal(data: bytes, at: int, size: int, mutate, seal_record: bool = True) -> bytes:
    """Mutate one record, then reseal it (optionally) and the trailer, so only the targeted check can refuse.

    Without this every tamper is caught by the trailer checksum alone, which would hide a missing field check.
    """
    out = bytearray(data)
    record = bytearray(out[at:at + size])
    mutate(record)
    out[at:at + size] = seal(record) if seal_record else record
    struct.pack_into("<Q", out, len(out) - 8, fnv(bytes(out[:-va.TRAILER])))
    return bytes(out)


class ArchiveReaderTests(unittest.TestCase):
    def test_round_trip_matches_the_spec_encoder(self):
        archive = va.parse(SAMPLE)
        self.assertEqual(va.stored_length(SAMPLE[:va.HEADER]), len(SAMPLE))
        self.assertEqual((archive["generation"], archive["content_bytes"], archive["length"]),
                         (7, 713, len(SAMPLE)))
        self.assertEqual(archive["collections"], {1: "Werk", 3: "Privé"})
        self.assertEqual([(o["slot"], o["name"], o["collection"], o["kind"], o["tags"], o["revision"],
                           o["created"], o["modified"], o["content"]) for o in archive["objects"]],
                         [(0, "Leeg", 0, 0, 0, 1, 1, 2, b""),
                          (2, "Notitie ✓", 3, 4, 0x11, 2, 1, 2, bytes(range(200))),
                          (7, "Laatste", 1, 0, 0, 9, 1, 2, b"x" * 513)])

    def test_resealing_helper_keeps_a_valid_archive_valid(self):
        for at, size in SPANS:
            self.assertEqual(va.parse(reseal(SAMPLE, at, size, lambda record: None)), va.parse(SAMPLE))

    def test_record_seals_and_content_sums_are_checked_under_a_valid_trailer(self):
        for at, size in SPANS:
            for offset in range(size):
                with self.assertRaises(ValueError, msg=f"record {at} byte {offset}"):
                    va.parse(reseal(SAMPLE, at, size, lambda r: r.__setitem__(offset, r[offset] ^ 1), False))
        content = OBJECT_AT + 2 * va.OBJECT_RECORD  # first byte of slot 2's content
        with self.assertRaises(ValueError):
            va.parse(reseal(SAMPLE, content, 1, lambda r: r.__setitem__(0, r[0] ^ 1), False))

    def test_resealed_reserved_and_padding_bytes_are_refused(self):
        reserved = {HEADER_AT: [*range(10, 12), *range(14, 16), *range(32, 56)],
                    COLLECTION_AT: [*range(2, 8), *range(8 + len("Werk"), 40)],
                    OBJECT_AT: [3, 6, 7, *range(48 + len("Leeg"), 80), *range(80, 88)]}
        sizes = dict(SPANS)
        for at, offsets in reserved.items():
            for offset in offsets:
                with self.assertRaises(ValueError, msg=f"record {at} byte {offset}"):
                    va.parse(reseal(SAMPLE, at, sizes[at], lambda r: r.__setitem__(offset, 1)))

    def test_resealed_header_and_record_limits_are_refused(self):
        u16 = lambda at, value: lambda r: struct.pack_into("<H", r, at, value)  # noqa: E731
        u32 = lambda at, value: lambda r: struct.pack_into("<I", r, at, value)  # noqa: E731
        u64 = lambda at, value: lambda r: struct.pack_into("<Q", r, at, value)  # noqa: E731
        byte = lambda at, value: lambda r: r.__setitem__(at, value)  # noqa: E731
        cases = {
            "magic": (HEADER_AT, byte(7, 2)),
            "version": (HEADER_AT, u16(8, 2)),
            "object count": (HEADER_AT, byte(12, va.OBJECTS + 1)),
            "collection count": (HEADER_AT, byte(13, va.COLLECTIONS + 1)),
            "content above count limit": (HEADER_AT, u64(24, 3 * va.MAX_LENGTH + 1)),
            "content total too high": (HEADER_AT, u64(24, 714)),
            "content total too low": (HEADER_AT, u64(24, 712)),
            "collection name length": (COLLECTION_AT, byte(1, va.NAME + 1)),
            "object name length": (OBJECT_AT, byte(1, va.NAME + 1)),
            "object length": (OBJECT_AT, u32(12, va.MAX_LENGTH + 1)),
            "object kind reserved": (OBJECT_AT, u16(6, 1)),
        }
        sizes = dict(SPANS)
        for label, (at, mutate) in cases.items():
            with self.assertRaises(ValueError, msg=label):
                va.parse(reseal(SAMPLE, at, sizes[at], mutate))

    def test_every_single_bit_flip_is_refused(self):
        for at in range(len(SAMPLE)):
            for bit in range(8):
                damaged = bytearray(SAMPLE)
                damaged[at] ^= 1 << bit
                with self.assertRaises(ValueError, msg=f"byte {at} bit {bit}"):
                    va.parse(bytes(damaged))

    def test_truncation_and_trailing_bytes_are_refused(self):
        for length in range(len(SAMPLE)):
            with self.assertRaises(ValueError, msg=f"length {length}"):
                va.parse(SAMPLE[:length])
        with self.assertRaises(ValueError):
            va.parse(SAMPLE + b"\0")

    def test_sealed_but_invalid_records_are_refused(self):
        cases = {
            "unsafe name": ([], [(0, b"a/b", 0, 0, 0, 1, b"")]),
            "bidi name": ([], [(0, "a‮b".encode(), 0, 0, 0, 1, b"")]),
            "zero revision": ([], [(0, b"a", 0, 0, 0, 0, b"")]),
            "unknown collection": ([(1, b"W")], [(0, b"a", 2, 0, 0, 1, b"")]),
            "slot order": ([], [(3, b"a", 0, 0, 0, 1, b""), (3, b"b", 0, 0, 0, 1, b"")]),
            "slot range": ([], [(8, b"a", 0, 0, 0, 1, b"")]),
            "collection order": ([(2, b"W"), (1, b"V")], []),
            "collection zero": ([(0, b"W")], []),
            "collection range": ([(5, b"W")], []),
        }
        for label, (collections, objects) in cases.items():
            with self.assertRaises(ValueError, msg=label):
                va.parse(encode(collections, objects))


class ExpectedRestoreTests(unittest.TestCase):
    """The spec rule the native conflict stage is checked against (tools/run_volume.py)."""

    def test_empty_domain_restores_everything_and_creates_referenced_collections(self):
        state = va.expected_restore(va.parse(SAMPLE), {}, {})
        self.assertEqual({slot: (o["revision"], o["collection"]) for slot, o in state["objects"].items()},
                         {0: (1, 0), 2: (2, 3), 7: (2, 1)})
        self.assertEqual(state["collections"], {1: "Werk", 3: "Privé"})
        self.assertFalse(state["slot_conflicts"] | state["name_conflicts"] | state["uncollected"] | state["reused"])

    def test_conflicts_are_untouched_and_a_renamed_collection_is_not_reused(self):
        state = va.expected_restore(va.parse(SAMPLE), {0: "Bezet", 5: "Laatste"}, {3: "Anders"})
        self.assertEqual((state["slot_conflicts"], state["name_conflicts"], state["uncollected"]), ({0}, {7}, {2}))
        # Slot 2 keeps its kind and tags (revision 2) but lands outside a collection.
        self.assertEqual({slot: (o["revision"], o["collection"], o["kind"]) for slot, o in state["objects"].items()},
                         {2: (2, 0, 4)})
        # Collection 1 is only referenced by the conflicting slot 7, so it is not created.
        self.assertEqual(state["collections"], {3: "Anders"})

    def test_an_equal_collection_is_reused(self):
        state = va.expected_restore(va.parse(SAMPLE), {}, {1: "Werk"})
        self.assertEqual((state["reused"], state["objects"][7]["collection"]), ({7}, 1))
        self.assertEqual(state["collections"], {1: "Werk", 3: "Privé"})


if __name__ == "__main__":
    unittest.main()
