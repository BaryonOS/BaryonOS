"""Host tests for the independent table v3 decoder in tools/volume_disk.py and
its frozen v2 reader (docs/implementation/S09-L2-PROVENANCE.md sections 2.1
and 3, required tests 9 and 10 on the Python side)."""
from __future__ import annotations
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import volume_disk as vd  # noqa: E402
from test_volume_disk import alone, document, entries_with, image_with  # noqa: E402


def volume(image: bytes) -> bytes:
    return bytes(image[vd.FIRST * vd.SECTOR:(vd.FIRST + vd.VOLUME_SECTORS) * vd.SECTOR])


def empty(revision: int, name: bytes = b"V"):
    return dict(revision=revision, length=0, name=name, content_sum=vd.FNV_OFFSET, chunks=[], sums=[])


def installed(request_id: int, clock: int):
    """Object installed with n = 3 at `clock`: revisions 3..1, provenance set."""
    return dict(vd.empty_entry(), revision=3, request_id=request_id, created=clock, modified=clock,
                present=True, current=empty(3), history=[empty(2), empty(1)], installed=3,
                origin=(40, 3, 11), origins=[30, 29, 28])


def written(generation: int):
    entry, _ = document(0, 1, 0, [], 5)
    return dict(entry, modified=1, created=1, current=empty(1, b"Doc"))


def v2_g1():
    return vd.encode_table(1, 5, entries_with({0: written(1)}))


def v3_g2(**changes):
    entries = entries_with({0: written(1), 1: dict(installed(6, 2), **changes)})
    return vd.encode_table(2, 6, entries, fmt=3)


def reseal(sectors, generation: int):
    sectors = [bytearray(s) for s in sectors]
    attempt = vd.attempt_digest(sectors)
    for index, sector in enumerate(sectors):
        struct.pack_into("<QQ", sector, vd.TRAILER, generation, attempt)
        struct.pack_into("<Q", sector, 504, vd.sector_sum(generation % 2, index, bytes(sector)))
    return [bytes(s) for s in sectors]


def patched(sectors, generation: int, edits):
    sectors = [bytearray(s) for s in sectors]
    for sector, offset, value in edits:
        sectors[sector][offset:offset + len(value)] = value
    return reseal(sectors, generation)


def torn(sectors, index: int = 7):
    out = list(sectors)
    out[index] = bytes(vd.SECTOR)
    return out


class TableV3Decoder(unittest.TestCase):
    def test_v2_then_v3_recovers_the_v3_copy_with_its_provenance(self):
        chosen = vd.choose(volume(image_with({1: v2_g1(), 0: v3_g2()})), allow_v3=True)
        self.assertEqual((chosen["generation"], chosen["format"]), (2, 3))
        entry = chosen["entries"][1]
        self.assertEqual((entry["installed"], entry["origin"], entry["origins"]), (3, (40, 3, 11), [30, 29, 28]))
        self.assertLessEqual(entry["revision"], chosen["clocks"][0] + 2)

    def test_v3_then_v2_is_a_downgrade_and_refused(self):
        v3_g1 = vd.encode_table(1, 5, entries_with({0: written(1)}), fmt=3)
        bumped = dict(written(1), revision=2, request_id=7, modified=2, current=empty(2, b"Doc"))
        v2_g2 = vd.encode_table(2, 7, entries_with({0: bumped}))
        with self.assertRaisesRegex(ValueError, "downgraded"):
            vd.choose(volume(image_with({1: v3_g1, 0: v2_g2})), allow_v3=True)

    def test_torn_v3_attempt_next_to_v2_recovers_the_v2_copy(self):
        for index in (0, 1, 3, vd.TABLE_SECTORS - 1):
            chosen = vd.choose(volume(image_with({1: v2_g1(), 0: torn(v3_g2(), index)})), allow_v3=True)
            self.assertEqual((chosen["generation"], chosen["format"]), (1, 2), index)

    def test_v3_copy_without_provenance_matches_v2_except_the_version_field(self):
        v2 = vd.encode_table(1, 5, entries_with({0: written(1)}))
        v3 = vd.encode_table(1, 5, entries_with({0: written(1)}), fmt=3)
        self.assertEqual(v2[0][10:vd.TRAILER], v3[0][10:vd.TRAILER])
        self.assertEqual(v2[1:], [s for s in reseal_as(v3, 2)[1:]])


def reseal_as(sectors, fmt: int):
    header = bytearray(sectors[0])
    struct.pack_into("<H", header, 8, fmt)
    return reseal([bytes(header)] + list(sectors[1:]), 1)


class FrozenV2Reader(unittest.TestCase):
    def assert_refused(self, tables):
        image = volume(image_with(tables))
        with self.assertRaises(ValueError):
            vd.choose(image)

    def test_v2_then_v3_is_refused_never_rolled_back(self):
        self.assert_refused({1: v2_g1(), 0: v3_g2()})

    def test_format_marker_then_v3_at_generation_one_is_refused(self):
        entries = entries_with({1: installed(5, 1)})
        v3_g1 = vd.encode_table(1, 5, entries, fmt=3)
        marker = [vd.format_marker()] + [bytes(vd.SECTOR)] * (vd.TABLE_SECTORS - 1)
        image = volume(image_with({0: marker, 1: v3_g1}))
        self.assertEqual(vd.choose(image, allow_v3=True)["entries"][1]["installed"], 3)
        with self.assertRaises(ValueError):
            vd.choose(image)

    def test_v3_then_torn_v3_is_refused(self):
        v3_g1 = vd.encode_table(1, 5, entries_with({0: written(1)}), fmt=3)
        self.assert_refused({1: v3_g1, 0: torn(v3_g2())})
        self.assertEqual(vd.choose(volume(image_with({1: v3_g1, 0: torn(v3_g2())})), allow_v3=True)["generation"], 1)

    def test_v2_volumes_decode_unchanged(self):
        chosen = vd.choose(volume(image_with(alone(1, v2_g1()))))
        self.assertEqual((chosen["format"], chosen["generation"]), (2, 1))
        self.assertNotIn("installed", chosen["entries"][0])


class CorruptProvenance(unittest.TestCase):
    """One case per invariant of section 2.1; each refused by the v3 decoder."""

    def assert_corrupt(self, sectors, generation: int = 2):
        image = volume(image_with(alone(generation, sectors)))
        with self.assertRaises(ValueError):
            vd.choose(image, allow_v3=True)

    def test_valid_reference_decodes(self):
        self.assertEqual(vd.choose(volume(image_with(alone(2, v3_g2()))), allow_v3=True)["format"], 3)

    def test_v3_bytes_in_a_version_2_copy(self):
        head, history = 2 + 2 * 1, 3 + 2 * 1
        for sector, offset in ((0, vd.HEADER_ORIGINS), (0, vd.HEADER_ORIGINS_END - 1), (head, vd.INSTALLED),
                               (head, vd.OBJECT_ORIGINS), (head, vd.OBJECT_ORIGINS_END - 1),
                               (history, vd.HISTORY_ORIGINS), (history, vd.HISTORY_ORIGINS_END - 1)):
            base = vd.encode_table(2, 6, entries_with({0: written(1), 1: dict(installed(6, 2), installed=0,
                                                                          origin=(0, 0, 0), origins=[0, 0, 0],
                                                                          revision=2, current=empty(2),
                                                                          history=[empty(1)])}), fmt=3)
            clean = patched(base, 2, [(0, 8, struct.pack("<H", 2))])
            self.assertEqual(vd.choose(volume(image_with(alone(2, clean))))["format"], 2)
            sectors = patched(base, 2, [(0, 8, struct.pack("<H", 2)), (sector, offset, b"\x01")])
            self.assert_corrupt(sectors)
            with self.assertRaises(ValueError):
                vd.choose(volume(image_with(alone(2, sectors))))

    def test_reserved_bytes_in_both_formats(self):
        head, history = 2 + 2 * 1, 3 + 2 * 1
        for sector, offset in ((0, vd.HEADER_ORIGINS_END), (0, vd.TRAILER - 1), (head, vd.INSTALLED + 1),
                               (head, vd.OBJECT_ORIGINS - 1), (head, vd.OBJECT_ORIGINS_END),
                               (head, vd.TRAILER - 1), (history, vd.HISTORY_ORIGINS_END),
                               (history, vd.TRAILER - 1)):
            self.assert_corrupt(patched(v3_g2(), 2, [(sector, offset, b"\x01")]))

    def test_installed_above_history_plus_one(self):
        # Revision 4 at clock 2 keeps every other rule satisfied.
        self.assert_corrupt(v3_g2(installed=4, revision=4, current=empty(4)))

    def test_origins_without_an_install(self):
        self.assert_corrupt(v3_g2(installed=0, revision=2, current=empty(2), history=[empty(1)]))
        self.assert_corrupt(v3_g2(installed=0, origin=(0, 0, 0), revision=2, current=empty(2),
                                  history=[empty(1)], origins=[0, 5, 0]))

    def test_installed_zero_with_revision_above_clock(self):
        self.assert_corrupt(v3_g2(installed=0, origin=(0, 0, 0), origins=[0, 0, 0]))

    def test_revision_above_the_excess_bound(self):
        # clock 2, installed 3: revision 4 is the bound, 5 is refused.
        self.assert_corrupt(v3_g2(revision=5, current=empty(5), history=[empty(2), empty(1)]))

    def test_revision_below_installed(self):
        self.assert_corrupt(v3_g2(revision=2, current=empty(2), history=[empty(1)]))

    def test_object_origin_stamps(self):
        for origin in ((40, 0, 11), (0, 3, 11), (40, 12, 11), (0, 0, 11)):
            self.assert_corrupt(v3_g2(origin=origin))

    def test_version_origins(self):
        self.assert_corrupt(v3_g2(origins=[30, 30, 28]))  # repeated
        self.assert_corrupt(v3_g2(origins=[41, 29, 28]))  # above origin_revision 40
        self.assert_corrupt(v3_g2(history=[empty(2)], origins=[30, 29, 28]))  # empty slot with origin

    def test_collection_origin_on_an_absent_collection(self):
        entries = entries_with({0: written(1), 1: installed(6, 2)})
        collections = [{"revision": 0, "request_id": 0, "present": False, "name": b"", "origin": 0}
                       for _ in range(vd.DOMAINS * vd.COLLECTIONS)]
        collections[2]["origin"] = 9
        self.assert_corrupt(vd.encode_table(2, 6, entries, collections, fmt=3))
        collections[2].update(revision=1, request_id=4, present=False, name=b"")
        self.assert_corrupt(vd.encode_table(2, 6, entries, collections, fmt=3))
        collections[2].update(present=True, name=b"Oud")
        chosen = vd.choose(volume(image_with(alone(2, vd.encode_table(2, 6, entries, collections, fmt=3)))),
                           allow_v3=True)
        self.assertEqual(chosen["collections"][2]["origin"], 9)

    def test_version_field_four_is_refused(self):
        self.assert_corrupt(patched(v3_g2(), 2, [(0, 8, struct.pack("<H", 4))]))


if __name__ == "__main__":
    unittest.main()
