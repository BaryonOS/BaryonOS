"""Host tests for the independent S09 archive v2 reader (tools/volume_archive.py, parse_v2/parse_any).

The encoder below follows docs/implementation/S09-ARCHIVE-V2.md directly and shares no code with the
reader or with volume-core. Every record is sealed by the encoder, so a rule violation is presented
in a correctly resealed archive and only the targeted check can refuse it.
"""
from __future__ import annotations
import struct
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / "tests"))
import volume_archive as va  # noqa: E402
from volume_disk import fnv  # noqa: E402

MAGIC, END = b"NOSBAK\x00\x01", b"NOSBAKZ\x00"
TOMBSTONES, ACCOUNTED = 1, 2


def seal(record: bytearray) -> bytes:
    struct.pack_into("<Q", record, len(record) - 8, fnv(bytes(record[:-8])))
    return bytes(record)


def encode(collections, objects, flags=TOMBSTONES | ACCOUNTED, clock=20, generation=None, tweak=None) -> bytes:
    """collections: [dict(index, present, revision, name)];
    objects: [dict(slot, present, collection, kind, tags, revision, created, modified,
                   versions=[(revision, name, content)], current first)].
    tweak maps "header", ("c", i), ("o", i) or ("v", i, j) to a function applied before sealing."""
    tweak = tweak or {}
    if generation is None:
        generation = (sum(o["revision"] for o in objects) + sum(c["revision"] for c in collections)) % (1 << 64)
    versions = [v for o in objects for v in o["versions"]]
    header = bytearray(64)
    header[:8] = MAGIC
    struct.pack_into("<HB", header, 8, 2, flags)
    header[12], header[13] = len(objects), len(collections)
    struct.pack_into("<HQQQ", header, 14, len(versions), generation, sum(len(v[2]) for v in versions), clock)
    tweak.get("header", lambda r: None)(header)
    out = bytearray(seal(header))
    for i, c in enumerate(collections):
        record = bytearray(64)
        name = c["name"].encode()
        record[0], record[1], record[2] = c["index"], int(c["present"]), len(name)
        struct.pack_into("<Q", record, 8, c["revision"])
        record[16:16 + len(name)] = name
        tweak.get(("c", i), lambda r: None)(record)
        out += seal(record)
    for i, o in enumerate(objects):
        record = bytearray(128)
        record[0], record[1], record[2] = o["slot"], int(o["present"]), o["collection"]
        record[3] = len(o["versions"]) - int(o["present"])
        struct.pack_into("<HxxIxxxxQQQ", record, 4, o["kind"], o["tags"], o["revision"], o["created"], o["modified"])
        tweak.get(("o", i), lambda r: None)(record)
        out += seal(record)
        for j, (revision, name, content) in enumerate(o["versions"]):
            version = bytearray(64)
            name = name.encode()
            version[0] = len(name)
            struct.pack_into("<IQQ", version, 4, len(content), revision, fnv(content))
            version[24:24 + len(name)] = name
            tweak.get(("v", i, j), lambda r: None)(version)
            out += seal(version) + content
    return bytes(out + END + struct.pack("<Q", fnv(bytes(out))))


def obj(slot, revision, versions, present=True, collection=0, kind=0, tags=0, created=1, modified=None):
    return dict(slot=slot, present=present, collection=collection, kind=kind, tags=tags, revision=revision,
                created=created, modified=revision if modified is None else modified, versions=versions)


def sample(**changes):
    """A small archive that uses every record kind; `changes` replaces top-level parts."""
    parts = dict(
        collections=[dict(index=1, present=True, revision=3, name="Werk"),
                     dict(index=2, present=False, revision=5, name="")],
        objects=[obj(0, 4, [(1, "Leeg", b"")]),
                 obj(2, 9, [(8, "Notitie ✓", bytes(range(40))), (6, "Notitie", b"oud"), (2, "Eerst", b"a")],
                     collection=1, kind=4, tags=0x11, created=2, modified=12),
                 obj(5, 7, [(7, "Weg", b"verwijderd"), (3, "Weg", b"v1")], present=False, created=4, modified=7),
                 obj(6, 2, [], present=False, created=5, modified=6)])
    parts.update(changes)
    return parts


SAMPLE = encode(**sample())


def v1_sample() -> bytes:
    """A small v1 archive built locally from S09-BACKUP-ARCHIVE.md (one collection, two objects)."""
    header = bytearray(64)
    header[:8] = MAGIC
    struct.pack_into("<H", header, 8, 1)
    contents = [b"", bytes(range(200))]
    header[12], header[13] = len(contents), 1
    struct.pack_into("<QQ", header, 16, 7, sum(map(len, contents)))
    out = bytearray(seal(header))
    collection = bytearray(48)
    collection[0], collection[1] = 1, 4
    collection[8:12] = b"Werk"
    out += seal(collection)
    for slot, (name, content) in enumerate(zip((b"Leeg", "Notitie ✓".encode()), contents)):
        record = bytearray(96)
        record[0], record[1], record[2] = slot * 2, len(name), slot
        struct.pack_into("<HxxIIQQQQ", record, 4, slot * 4, slot * 0x11, len(content), slot + 1, 1, slot + 1,
                         fnv(content))
        record[48:48 + len(name)] = name
        out += seal(record) + content
    return bytes(out + END + struct.pack("<Q", fnv(bytes(out))))


V1_SAMPLE = v1_sample()


def present_only():
    """Variant without deleted collections, tombstones or content-free records (valid with flags 0)."""
    parts = sample()
    parts["collections"] = parts["collections"][:1]
    parts["objects"] = parts["objects"][:2]
    return parts


def first_object(**fields):
    """sample() with fields of the slot-2 object (index 1) replaced."""
    parts = sample()
    parts["objects"][1] = dict(parts["objects"][1], **fields)
    return parts


class RoundTrip(unittest.TestCase):
    def test_values_round_trip(self) -> None:
        archive = va.parse_v2(SAMPLE)
        self.assertEqual((archive["version"], archive["tombstones"], archive["accounted"]), (2, True, True))
        self.assertEqual((archive["clock"], archive["generation"], archive["length"]), (20, 30, len(SAMPLE)))
        self.assertEqual(archive["collections"], {1: dict(present=True, revision=3, name="Werk"),
                                                  2: dict(present=False, revision=5, name="")})
        for got, want in zip(archive["objects"], sample()["objects"], strict=True):
            for field in ("slot", "present", "collection", "kind", "tags", "revision", "created", "modified"):
                self.assertEqual(got[field], want[field])
            self.assertEqual([(v["revision"], v["name"], v["content"]) for v in got["versions"]],
                             [(r, n, c) for r, n, c in want["versions"]])
        self.assertEqual(va.parse_any(SAMPLE), archive)
        self.assertEqual(va.stored_length_v2(SAMPLE[:64]), len(SAMPLE))

    def test_empty_archive(self) -> None:
        data = encode([], [], flags=0, clock=0)
        self.assertEqual(va.parse_v2(data)["objects"], [])

    def test_every_flag_combination_with_its_records(self) -> None:
        self.assertEqual(len(va.parse_v2(encode(**present_only(), flags=0))["objects"]), 2)
        tomb = present_only()
        tomb["objects"].append(sample()["objects"][2])
        self.assertTrue(va.parse_v2(encode(**tomb, flags=TOMBSTONES))["tombstones"])
        free = sample()
        del free["objects"][2]
        self.assertTrue(va.parse_v2(encode(**free, flags=ACCOUNTED))["accounted"])

    def test_bound_constant(self) -> None:
        self.assertEqual(va.CONTENT_V2, 491_520)
        self.assertEqual(va.MAX_ARCHIVE_V2, 494_416)
        header = bytearray(64)
        header[12], header[13] = 8, 4
        struct.pack_into("<H", header, 14, 24)
        struct.pack_into("<Q", header, 24, va.CONTENT_V2)
        self.assertEqual(va.stored_length_v2(bytes(header)), va.MAX_ARCHIVE_V2)

    def test_content_bound_is_reached_by_a_source_feasible_archive(self) -> None:
        """Eight objects with three versions of five chunks each: 40 chunks, exactly CONTENT_V2."""
        def full(extra=0):
            collections = [dict(index=i, present=True, revision=1, name=f"c{i}") for i in range(1, 5)]
            objects = [obj(s, 3, [(3 - j, f"n{s}{j}", b"\x5a" * (5 * va.CHUNK + (extra if s == j == 0 else 0)))
                                  for j in range(3)], collection=s % 4 + 1) for s in range(8)]
            return encode(collections, objects, flags=0)
        data = full()
        self.assertEqual(len(data), va.MAX_ARCHIVE_V2)
        self.assertEqual(va.parse_v2(data)["content_bytes"], va.CONTENT_V2)
        with self.assertRaisesRegex(ValueError, "v2 bound"):
            va.parse_v2(full(1))

    def test_chunk_budget(self) -> None:
        """Sum over objects of the largest version's chunk count <= QUOTA; here a history version is largest."""
        def three(history_length):
            return encode([], [obj(0, 1, [(1, "a", b"\x01" * va.MAX_LENGTH)]),
                               obj(1, 1, [(1, "b", b"\x02" * va.MAX_LENGTH)]),
                               obj(2, 3, [(3, "c", b"x"), (2, "c", b"\x03" * history_length)])], flags=0)
        self.assertEqual(len(va.parse_v2(three(8 * va.CHUNK))["objects"]), 3)
        with self.assertRaisesRegex(ValueError, "chunk usage exceeds the domain quota"):
            va.parse_v2(three(8 * va.CHUNK + 1))


class Damage(unittest.TestCase):
    def test_every_single_bit_flip_is_refused(self) -> None:
        for at in range(len(SAMPLE)):
            for bit in range(8):
                data = bytearray(SAMPLE)
                data[at] ^= 1 << bit
                for reader in (va.parse_v2, va.parse_any):
                    with self.subTest(at=at, bit=bit, reader=reader.__name__), self.assertRaises(ValueError):
                        reader(bytes(data))

    def test_every_truncation_and_trailing_byte_is_refused(self) -> None:
        for reader in (va.parse_v2, va.parse_any):
            for end in range(len(SAMPLE)):
                with self.subTest(end=end, reader=reader.__name__), self.assertRaises(ValueError):
                    reader(SAMPLE[:end])
            for extra in (b"\x00", b"\xff"):
                with self.assertRaisesRegex(ValueError, "after the archive trailer"):
                    reader(SAMPLE + extra)


class Compatibility(unittest.TestCase):
    def test_v1_archive_through_parse_any_equals_parse(self) -> None:
        self.assertEqual(va.parse_any(V1_SAMPLE), va.parse(V1_SAMPLE))
        for damaged in (V1_SAMPLE[:-1], V1_SAMPLE[:40], V1_SAMPLE + b"\x00",
                        V1_SAMPLE[:20] + bytes([V1_SAMPLE[20] ^ 1]) + V1_SAMPLE[21:]):
            with self.assertRaises(ValueError) as v1:
                va.parse(damaged)
            with self.assertRaises(ValueError) as any_:
                va.parse_any(damaged)
            self.assertEqual(str(v1.exception), str(any_.exception))

    def test_v1_reader_refuses_v2(self) -> None:
        with self.assertRaisesRegex(ValueError, "invalid archive header"):
            va.parse(SAMPLE)

    def test_v2_reader_refuses_v1(self) -> None:
        with self.assertRaisesRegex(ValueError, "invalid archive header"):
            va.parse_v2(V1_SAMPLE)


def put(offset, fmt, value):
    return lambda record: struct.pack_into(fmt, record, offset, value)


def setbyte(offset, value):
    def apply(record):
        record[offset] = value
    return apply


class Rules(unittest.TestCase):
    """One case per rule; each archive is valid except for that rule and fully resealed."""

    def refused(self, parts, message, **kw) -> None:
        with self.assertRaisesRegex(ValueError, message):
            va.parse_v2(encode(**parts, **kw))

    def test_structural_rules(self) -> None:
        cases = {
            "magic": ({"header": setbyte(0, ord("X"))}, "invalid archive header"),
            "version": ({"header": put(8, "<H", 3)}, "invalid archive header"),
            "unknown flag bit": ({"header": setbyte(10, 0x07)}, "invalid archive header"),
            "object count > 8": ({"header": setbyte(12, 9)}, "invalid archive header"),
            "collection count > 4": ({"header": setbyte(13, 5)}, "invalid archive header"),
            "version count > 24": ({"header": put(14, "<H", 25)}, "invalid archive header"),
            "content > versions x MAX_LENGTH": ({"header": put(14, "<H", 0)}, "invalid archive header"),
            "header version count above records": ({"header": put(14, "<H", 7)}, "invalid archive trailer"),
            "header version count below records": ({"header": put(14, "<H", 5)}, "exceed the header counts"),
            "header content total above records": ({"header": put(24, "<Q", 70)}, "invalid archive trailer"),
            "header content total below records": ({"header": put(24, "<Q", 10)}, "exceed the header counts"),
            "collection index 0": ({("c", 0): setbyte(0, 0)}, "invalid collection record"),
            "collection index not ascending": ({("c", 1): setbyte(0, 1)}, "invalid collection record"),
            "collection index > 4": ({("c", 1): setbyte(0, 5)}, "invalid collection record"),
            "collection state > 1": ({("c", 0): setbyte(1, 2)}, "invalid collection record"),
            "collection name length > 32": ({("c", 0): setbyte(2, 33)}, "invalid collection record"),
            "collection invalid name": ({("c", 0): setbyte(16, 0xff)}, "invalid collection record"),
            "collection revision 0": ({("c", 0): put(8, "<Q", 0)}, "invalid collection record"),
            "deleted collection with name": ({("c", 1): setbyte(2, 1)}, "invalid collection record"),
            "object slot not ascending": ({("o", 1): setbyte(0, 0)}, "invalid object record"),
            "object slot > 7": ({("o", 3): setbyte(0, 8)}, "invalid object record"),
            "object state > 1": ({("o", 0): setbyte(1, 2)}, "invalid object record"),
            "object revision 0": ({("o", 0): put(16, "<Q", 0)}, "invalid object record"),
            "collection not archived": ({("o", 1): setbyte(2, 3)}, "invalid object record"),
            "collection deleted": ({("o", 1): setbyte(2, 2)}, "invalid object record"),
            "absent object with collection": ({("o", 2): setbyte(2, 1)}, "invalid object record"),
            "absent object with kind": ({("o", 2): put(4, "<H", 1)}, "invalid object record"),
            "absent object with tags": ({("o", 2): put(8, "<I", 1)}, "invalid object record"),
            "version name length 0": ({("v", 0, 0): setbyte(0, 0)}, "invalid version record"),
            "version name length > 32": ({("v", 0, 0): setbyte(0, 33)}, "invalid version record"),
            "version invalid name": ({("v", 0, 0): setbyte(24, 0xff)}, "invalid version record"),
            "version length > 64 KiB": ({("v", 0, 0): put(4, "<I", 65537)}, "invalid version record"),
            "version content checksum": ({("v", 1, 1): put(16, "<Q", 1)}, "content checksum"),
        }
        for name, case in cases.items():
            tweak, message = case
            with self.subTest(rule=name):
                self.refused(sample(), message, tweak=tweak)
        # history_count 3 (> HISTORY): a fourth version on a present object.
        self.refused(first_object(versions=sample()["objects"][1]["versions"] + [(1, "x", b"")]),
                     "invalid object record")
        data = bytearray(SAMPLE)
        data[-16] ^= 1
        struct.pack_into("<Q", data, len(data) - 8, fnv(bytes(data[:-16])))
        with self.assertRaisesRegex(ValueError, "invalid archive trailer"):
            va.parse_v2(bytes(data))

    def test_every_reserved_and_padding_byte(self) -> None:
        """Every byte of every reserved and name-padding range, one at a time, in a resealed archive."""
        ranges = [("header", [11, *range(40, 56)], "invalid archive header"),
                  (("c", 0), [*range(3, 8), *range(16 + 4, 56)], "invalid collection record"),
                  (("c", 1), [*range(3, 8), *range(16, 56)], "invalid collection record"),
                  (("o", 0), [6, 7, *range(12, 16), *range(40, 120)], "invalid object record"),
                  (("v", 0, 0), [*range(1, 4), *range(24 + 4, 56)], "invalid version record")]
        for key, offsets, message in ranges:
            for offset in offsets:
                with self.subTest(record=key, offset=offset):
                    self.refused(sample(), message, tweak={key: setbyte(offset, 1)})

    def test_record_seals(self) -> None:
        """A damaged seal with a recomputed trailer checksum is refused by the record's own seal check."""
        for at, size, message in ((0, 64, "invalid archive header"), (64, 64, "invalid collection record"),
                                  (192, 128, "invalid object record"), (320, 64, "invalid version record")):
            data = bytearray(SAMPLE)
            data[at + size - 1] ^= 1
            struct.pack_into("<Q", data, len(data) - 8, fnv(bytes(data[:-16])))
            with self.subTest(at=at), self.assertRaisesRegex(ValueError, message):
                va.parse_v2(bytes(data))

    def test_deleted_collection_requires_accounted(self) -> None:
        parts = sample()
        del parts["objects"][3]
        self.refused(parts, "deleted collection without the accounted flag", flags=TOMBSTONES)

    def test_tombstone_requires_tombstone_flag(self) -> None:
        self.refused(sample(), "tombstone without the tombstone flag", flags=ACCOUNTED)

    def test_content_free_record_requires_accounted(self) -> None:
        parts = sample()
        parts["collections"] = parts["collections"][:1]
        self.refused(parts, "content-free record without the accounted flag", flags=TOMBSTONES)

    # Semantic rules (fidelity analysis 3.2).

    def test_version_revision_zero(self) -> None:
        self.refused(first_object(versions=[(8, "a", b"1"), (6, "b", b"2"), (0, "c", b"3")]),
                     "version revision is zero")

    def test_version_revisions_strictly_decreasing(self) -> None:
        self.refused(first_object(versions=[(8, "a", b"1"), (8, "b", b"2")]), "not strictly decreasing")
        self.refused(first_object(versions=[(6, "a", b"1"), (8, "b", b"2")]), "not strictly decreasing")
        # Only the history[0] -> history[1] step is out of order.
        self.refused(first_object(versions=[(8, "a", b"1"), (5, "b", b"2"), (6, "c", b"3")]),
                     "not strictly decreasing")

    def test_current_version_revision_within_object_revision(self) -> None:
        self.refused(first_object(versions=[(10, "a", b"1")]), "newest version revision exceeds")
        self.assertEqual(va.parse_v2(encode(**first_object(versions=[(9, "a", b"1")])))["objects"][1]["revision"], 9)

    def test_tombstone_newest_history_within_object_revision(self) -> None:
        parts = sample()
        parts["objects"][2] = dict(parts["objects"][2], versions=[(8, "Weg", b"x")])
        self.refused(parts, "newest version revision exceeds")

    def test_created_nonzero(self) -> None:
        self.refused(first_object(created=0), "created stamp is zero")

    def test_created_not_after_modified(self) -> None:
        self.refused(first_object(created=13), "created stamp after modified")
        self.assertEqual(va.parse_v2(encode(**first_object(created=12)))["objects"][1]["created"], 12)

    def test_modified_within_clock(self) -> None:
        self.refused(first_object(modified=21), "modified stamp exceeds the source clock")
        self.assertEqual(va.parse_v2(encode(**first_object(modified=20)))["objects"][1]["modified"], 20)

    def test_object_revision_within_clock(self) -> None:
        self.refused(first_object(revision=21), "object revision exceeds the source clock")

    def test_collection_revision_within_clock(self) -> None:
        parts = sample()
        parts["collections"][0] = dict(parts["collections"][0], revision=21)
        self.refused(parts, "collection revision exceeds the source clock")
        parts["collections"][0] = dict(parts["collections"][0], revision=20)
        self.assertEqual(va.parse_v2(encode(**parts))["collections"][1]["revision"], 20)

    def test_generation_sum_is_conditional_on_accounted(self) -> None:
        self.refused(sample(), "differs from the domain generation", generation=31)
        parts = present_only()
        self.assertEqual(va.parse_v2(encode(**parts, flags=0, generation=31))["generation"], 31)
        self.assertEqual(va.parse_v2(encode(**sample(), flags=TOMBSTONES | ACCOUNTED))["generation"], 30)

    def test_generation_sum_wraps_like_the_table(self) -> None:
        parts = present_only()
        parts["objects"] = parts["objects"][:1]
        parts["objects"][0] = obj(0, 2**64 - 1, [(1, "a", b"")], modified=2**64 - 1)
        data = encode(**parts, flags=ACCOUNTED, clock=2**64 - 1)
        self.assertEqual(va.parse_v2(data)["generation"], (2**64 - 1 + 3) % 2**64)


if __name__ == "__main__":
    unittest.main()
