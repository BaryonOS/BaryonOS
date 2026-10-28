"""Host tests for the independent M6e disk oracle (tools/volume_disk.py)."""
from __future__ import annotations
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import volume_disk as vd  # noqa: E402


def image_with(tables: dict[int, list[bytes]], chunks: dict[int, bytes] | None = None) -> bytearray:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "v.img"
        vd.create_image(path)
        image = bytearray(path.read_bytes())
    for slot, sectors in tables.items():
        for index, sector in enumerate(sectors):
            start = (vd.FIRST + slot * vd.TABLE_SECTORS + index) * vd.SECTOR
            image[start:start + vd.SECTOR] = sector
    for chunk, data in (chunks or {}).items():
        start = (vd.FIRST + vd.DATA_START + chunk * vd.CHUNK_SECTORS) * vd.SECTOR
        image[start:start + len(data)] = data
    return image


def document(obj: int, revision: int, length: int, chunks: list[int], request_id: int):
    data = vd.fixture(obj, revision, length)
    sums = [vd.fnv(data[i * vd.CHUNK:(i + 1) * vd.CHUNK]) for i in range(len(chunks))]
    entry = dict(revision=revision, request_id=request_id, length=length, present=True,
                 name=b"Doc", chunks=chunks, sums=sums, content_sum=vd.fnv(data))
    payload = {chunk: data[i * vd.CHUNK:(i + 1) * vd.CHUNK] for i, chunk in enumerate(chunks)}
    return entry, payload


def entries_with(updates: dict[int, dict]):
    entries = [vd.empty_entry() for _ in range(vd.OBJECTS)]
    for obj, entry in updates.items():
        entries[obj] = entry
    return entries


class VolumeOracleTests(unittest.TestCase):
    def test_fresh_image_is_empty_generation_zero(self):
        result = vd.analyse(bytes(image_with({})))
        self.assertEqual((result["generation"], result["chunks_used"]), (0, 0))

    def test_document_content_and_revision_are_verified(self):
        entry, payload = document(1, 1, 5000, [3, 7], 11)
        tables = {1: vd.encode_table(1, 11, entries_with({1: entry}))}
        result = vd.analyse(bytes(image_with(tables, payload)))
        self.assertEqual(result["objects"][1]["content_revision"], 1)
        self.assertEqual(result["chunks_used"], 2)
        broken = image_with(tables, payload)
        broken[(vd.FIRST + vd.DATA_START + 7 * 8) * vd.SECTOR + 3] ^= 1
        with self.assertRaises(ValueError):
            vd.analyse(bytes(broken))

    def test_torn_next_table_falls_back_to_previous_generation(self):
        old, payload = document(0, 1, 100, [0], 5)
        new, payload2 = document(0, 2, 100, [1], 6)
        gen1 = vd.encode_table(1, 5, entries_with({0: old}))
        gen2 = vd.encode_table(2, 6, entries_with({0: new}))
        torn = gen2[:4] + [bytes(vd.SECTOR)] * 5
        result = vd.analyse(bytes(image_with({1: gen1, 0: torn}, {**payload, **payload2})))
        self.assertEqual((result["generation"], result["objects"][0]["revision"]), (1, 1))
        complete = vd.analyse(bytes(image_with({1: gen1, 0: gen2}, {**payload, **payload2})))
        self.assertEqual(complete["generation"], 2)

    def test_sealed_but_noncanonical_newest_copy_is_refused(self):
        entry = dict(vd.empty_entry(), revision=1, request_id=5, present=True, name=b"a/b",
                     content_sum=vd.FNV_OFFSET)
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with({1: vd.encode_table(1, 5, entries_with({0: entry}))})))

    def test_unrelated_generations_are_refused(self):
        a, _ = document(0, 1, 0, [], 5)
        b, _ = document(0, 1, 0, [], 9)
        tables = {1: vd.encode_table(1, 5, entries_with({0: a})),
                  0: vd.encode_table(4, 9, entries_with({0: b}))}
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with(tables)))

    def test_write_outside_scope_is_detected(self):
        image = image_with({})
        image[2 * vd.SECTOR] ^= 0xFF
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image))
        image = image_with({})
        image[(vd.FIRST + vd.VOLUME_SECTORS) * vd.SECTOR] ^= 0xFF
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image))

    def test_names_follow_the_rust_rules(self):
        for good in ("Café", "Notitie ✓", "a" * 32):
            self.assertTrue(vd.valid_name(good.encode()))
        for bad in (b"", b".", b"..", b"a/b", b"a\\b", b"\x00", "x‮y".encode(), b"\xff", b"a" * 33):
            self.assertFalse(vd.valid_name(bad))


if __name__ == "__main__":
    unittest.main()
