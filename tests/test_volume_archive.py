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


class ArchiveReaderTests(unittest.TestCase):
    def test_round_trip_matches_the_spec_encoder(self):
        archive = va.parse(SAMPLE)
        self.assertEqual(va.stored_length(SAMPLE[:va.HEADER]), len(SAMPLE))
        self.assertEqual((archive["generation"], archive["content_bytes"], archive["length"]),
                         (7, 713, len(SAMPLE)))
        self.assertEqual(archive["collections"], {1: "Werk", 3: "Privé"})
        self.assertEqual([(o["slot"], o["name"], o["collection"], o["kind"], o["tags"], o["revision"],
                           o["content"]) for o in archive["objects"]],
                         [(0, "Leeg", 0, 0, 0, 1, b""), (2, "Notitie ✓", 3, 4, 0x11, 2, bytes(range(200))),
                          (7, "Laatste", 1, 0, 0, 9, b"x" * 513)])

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


if __name__ == "__main__":
    unittest.main()
