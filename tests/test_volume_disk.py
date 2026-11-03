"""Host tests for the independent volume disk oracle (tools/volume_disk.py, format v2)."""
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


def version(obj: int, revision: int, length: int, chunks: list[int], name: bytes = b"Doc"):
    data = vd.fixture(obj, revision, length)
    sums = [vd.fnv(data[i * vd.CHUNK:(i + 1) * vd.CHUNK]) for i in range(len(chunks))]
    payload = {chunk: data[i * vd.CHUNK:(i + 1) * vd.CHUNK] for i, chunk in enumerate(chunks)}
    return dict(revision=revision, length=length, name=name, content_sum=vd.fnv(data),
                chunks=chunks, sums=sums), payload


def document(obj: int, revision: int, length: int, chunks: list[int], request_id: int, **fields):
    current, payload = version(obj, revision, length, chunks)
    entry = dict(vd.empty_entry(), revision=revision, request_id=request_id, created=1,
                 modified=revision, present=True, current=current)
    entry.update(fields)
    return entry, payload


def entries_with(updates: dict[int, dict]):
    entries = [vd.empty_entry() for _ in range(vd.OBJECTS)]
    for obj, entry in updates.items():
        entries[obj] = entry
    return entries


def alone(generation: int, table: list[bytes]) -> dict[int, list[bytes]]:
    """A table whose other copy holds an interrupted (unsealed) rewrite."""
    return {generation % 2: table, (generation + 1) % 2: [b"\xa5" * vd.SECTOR] * vd.TABLE_SECTORS}


def collections_with(updates: dict[int, bytes], request_id: int, revision: int = 1):
    collections = [{"revision": 0, "request_id": 0, "present": False, "name": b""}
                   for _ in range(vd.DOMAINS * vd.COLLECTIONS)]
    for index, name in updates.items():
        collections[index] = {"revision": revision, "request_id": request_id, "present": True, "name": name}
    return collections


class VolumeOracleTests(unittest.TestCase):
    def test_fresh_image_is_empty_generation_zero(self):
        image = bytes(image_with({}))
        result = vd.analyse(image)
        self.assertEqual((result["generation"], result["chunks_used"]), (0, 0))
        start = vd.FIRST * vd.SECTOR
        sectors, states = vd.slot_states(image[start:], 0)
        self.assertTrue(all(kind == "zero" for kind, _, _ in states))
        with self.assertRaises(ValueError):
            vd.decode_table(0, sectors)
        self.assertEqual(vd.slot_states(image[start:], 1)[1], [("zero", 0, 0)] * vd.TABLE_SECTORS)

    def test_full_zero_volume_is_admitted_as_empty(self):
        zero = [bytes(vd.SECTOR)] * vd.TABLE_SECTORS
        result = vd.analyse(bytes(image_with({0: zero, 1: zero})))
        self.assertEqual((result["generation"], result["chunks_used"]), (0, 0))

    def test_zero_table_sectors_do_not_admit_unrelated_tail_data(self):
        zero = [bytes(vd.SECTOR)] * vd.TABLE_SECTORS
        image = image_with({0: zero, 1: zero}, {3: b"foreign filesystem or user content"})
        before = bytes(image)
        with self.assertRaisesRegex(ValueError, "nonzero data in an unmarked volume"):
            vd.analyse(image)
        self.assertEqual(image, before)

    def test_partial_first_commit_without_format_is_refused(self):
        entry, _ = document(0, 1, 0, [], 5)
        gen1 = vd.encode_table(1, 5, entries_with({0: entry}))
        zero = [bytes(vd.SECTOR)] * vd.TABLE_SECTORS
        for prefix in (1, 4, vd.TABLE_SECTORS - 1):
            with self.subTest(prefix=prefix), self.assertRaisesRegex(ValueError, "unmarked volume"):
                vd.analyse(bytes(image_with({0: zero, 1: gen1[:prefix] + zero[prefix:]})))

    def test_generation_zero_table_is_not_used_as_recovery_identity(self):
        formatted = vd.encode_table(0, 0, entries_with({}))
        with self.assertRaises(ValueError):
            vd.decode_table(0, formatted)
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with({0: formatted})))

    def test_first_commit_cuts_recover_the_format_marker_state(self):
        entry, payload = document(0, 1, 100, [0], 5)
        gen1 = vd.encode_table(1, 5, entries_with({0: entry}))
        zero = [bytes(vd.SECTOR)] * vd.TABLE_SECTORS
        for prefix in range(vd.TABLE_SECTORS):
            with self.subTest(prefix=prefix):
                image = image_with({1: gen1[:prefix] + zero[prefix:]}, payload)
                marker_start = vd.FIRST * vd.SECTOR
                image[marker_start:marker_start + vd.SECTOR] = vd.format_marker()
                result = vd.analyse(bytes(image))
                self.assertEqual((result["generation"], result["chunks_used"]), (0, 0))
        torn = bytearray(gen1[3])
        torn[31] ^= 1
        image = image_with({1: gen1[:3] + [bytes(torn)] + zero[4:]}, payload)
        marker_start = vd.FIRST * vd.SECTOR
        image[marker_start:marker_start + vd.SECTOR] = vd.format_marker()
        result = vd.analyse(bytes(image))
        self.assertEqual(result["generation"], 0)
        image = image_with({1: gen1}, payload)
        marker_start = vd.FIRST * vd.SECTOR
        image[marker_start:marker_start + vd.SECTOR] = vd.format_marker()
        self.assertEqual(vd.analyse(bytes(image))["generation"], 1)

    def test_format_with_any_bad_sector_checksum_is_refused(self):
        formatted = vd.encode_table(0, 0, entries_with({}))
        for index in range(vd.TABLE_SECTORS):
            sectors = list(formatted)
            bad = bytearray(sectors[index])
            bad[504] ^= 1
            sectors[index] = bytes(bad)
            with self.subTest(sector=index), self.assertRaises(ValueError):
                vd.analyse(bytes(image_with({0: sectors})))

    def test_generation_zero_table_is_not_a_recovery_source(self):
        variants = (
            vd.encode_table(0, 1, entries_with({})),
            vd.encode_table(0, 0, entries_with({}), clocks=[1, 0]),
            vd.encode_table(0, 0, entries_with({0: dict(vd.empty_entry(), tags=1)})),
            vd.encode_table(0, 0, entries_with({}), collections_with({0: b"foreign"}, 1)),
        )
        for index, sectors in enumerate(variants):
            with self.subTest(variant=index), self.assertRaises(ValueError):
                vd.analyse(bytes(image_with({0: sectors})))
        canonical = vd.encode_table(0, 0, entries_with({}))
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with({0: canonical})))

    def test_committed_generation_one_with_legacy_zero_companion_is_accepted(self):
        entry, payload = document(0, 1, 100, [0], 5)
        zero = [bytes(vd.SECTOR)] * vd.TABLE_SECTORS
        result = vd.analyse(bytes(image_with({0: zero, 1: vd.encode_table(1, 5, entries_with({0: entry}))}, payload)))
        self.assertEqual((result["generation"], result["chunks_used"]), (1, 1))

    def test_companion_generation_boundaries_do_not_wrap(self):
        self.assertTrue(vd.compatible([("zero", 0, 0), ("valid", 1, 1)], 0, False))
        self.assertFalse(vd.compatible([("valid", vd.MASK, 1)], 0, False))
        self.assertFalse(vd.compatible([("valid", 0, 1)], vd.MASK, False))
        self.assertTrue(vd.compatible([("valid", vd.MASK - 1, 1)], vd.MASK, False))
        self.assertFalse(vd.compatible([("zero", 0, 0)], 2, False))

    def test_nonzero_unmarked_volume_is_not_an_empty_volume(self):
        image = image_with({})
        start = (vd.FIRST + vd.VOLUME_SECTORS - 1) * vd.SECTOR
        image[start] = 0xB5
        with self.assertRaisesRegex(ValueError, "unmarked volume"):
            vd.analyse(bytes(image))

        marker_start = vd.FIRST * vd.SECTOR
        image[marker_start:marker_start + vd.SECTOR] = vd.format_marker()
        result = vd.analyse(bytes(image))
        self.assertEqual((result["generation"], result["chunks_used"]), (0, 0))

    def test_unmarked_nonzero_table_sectors_are_not_empty(self):
        baseline = image_with({})
        for sector in range(vd.DATA_START):
            with self.subTest(sector=sector):
                image = bytearray(baseline)
                image[(vd.FIRST + sector) * vd.SECTOR + 17] = 0xB5
                with self.assertRaises(ValueError):
                    vd.analyse(bytes(image))

    def test_format_marker_preserves_a_torn_first_table(self):
        image = image_with({})
        image[(vd.FIRST + vd.DATA_START - 1) * vd.SECTOR + 17] = 0xB5
        start = vd.FIRST * vd.SECTOR
        image[start:start + vd.SECTOR] = vd.format_marker()
        result = vd.analyse(bytes(image))
        self.assertEqual((result["generation"], result["chunks_used"]), (0, 0))

    def test_torn_first_format_marker_is_never_silently_admitted(self):
        marker = vd.format_marker()
        for prefix in range(vd.SECTOR + 1):
            with self.subTest(prefix=prefix):
                volume = bytearray(vd.VOLUME_SECTORS * vd.SECTOR)
                volume[:prefix] = marker[:prefix]
                if not any(volume) or volume[:vd.SECTOR] == marker:
                    self.assertEqual(vd.choose(bytes(volume))["generation"], 0)
                else:
                    with self.assertRaises(ValueError):
                        vd.choose(bytes(volume))

    def test_document_content_and_revision_are_verified(self):
        entry, payload = document(1, 1, 5000, [3, 7], 11)
        tables = {1: vd.encode_table(1, 11, entries_with({1: entry}))}
        result = vd.analyse(bytes(image_with(tables, payload)))
        self.assertEqual(result["objects"][1]["versions"][0]["content_revision"], 1)
        self.assertEqual(result["chunks_used"], 2)
        broken = image_with(tables, payload)
        broken[(vd.FIRST + vd.DATA_START + 7 * 8) * vd.SECTOR + 3] ^= 1
        with self.assertRaises(ValueError):
            vd.analyse(bytes(broken))

    def test_history_versions_are_verified_and_counted_once(self):
        old, old_payload = version(2, 1, 5000, [4, 5])
        # The current version re-references the unchanged second chunk at the
        # same position with the same checksum, plus one new chunk.
        new = dict(old, revision=2, chunks=[6, 5])
        new_payload = {6: old_payload[4]}
        entry = dict(vd.empty_entry(), revision=2, request_id=8, created=1, modified=2, present=True,
                     current=new, history=[old])
        tables = alone(2, vd.encode_table(2, 8, entries_with({2: entry})))
        result = vd.analyse(bytes(image_with(tables, {**old_payload, **new_payload})))
        self.assertEqual([v["revision"] for v in result["objects"][2]["versions"]], [2, 1])
        self.assertEqual(result["chunks_used"], 3)
        broken = image_with(tables, {**old_payload, **new_payload})
        broken[(vd.FIRST + vd.DATA_START + 4 * 8) * vd.SECTOR] ^= 1
        with self.assertRaises(ValueError):
            vd.analyse(bytes(broken))

    def test_history_must_be_strictly_older_and_share_chunks_consistently(self):
        current, payload = version(2, 2, 100, [1])
        older, older_payload = version(2, 1, 100, [0])
        payload.update(older_payload)
        good = dict(vd.empty_entry(), revision=3, request_id=9, created=1, modified=3, present=True,
                    current=current, history=[older])
        vd.analyse(bytes(image_with(alone(3, vd.encode_table(3, 9, entries_with({2: good}))), payload)))
        newer = dict(good, history=[dict(older, revision=3)])
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with(alone(3, vd.encode_table(3, 9, entries_with({2: newer}))), payload)))
        aliased = dict(good, history=[dict(older, chunks=[1], sums=[older["sums"][0] ^ 1])])
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with(alone(3, vd.encode_table(3, 9, entries_with({2: aliased}))), payload)))

    def test_deleted_object_keeps_history_but_no_metadata(self):
        kept, payload = version(3, 1, 10, [2])
        tomb = dict(vd.empty_entry(), revision=2, request_id=4, created=1, modified=2, present=False,
                    history=[kept])
        result = vd.analyse(bytes(image_with(alone(2, vd.encode_table(2, 4, entries_with({3: tomb}))), payload)))
        self.assertEqual((result["objects"][3]["present"], result["chunks_used"]), (False, 1))
        tagged = dict(tomb, tags=1)
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with(alone(2, vd.encode_table(2, 4, entries_with({3: tagged}))), payload)))

    def test_collections_membership_and_digest(self):
        entry, payload = document(1, 1, 10, [0], 5, collection=1, tags=3, kind=7)
        collections = collections_with({0: b"Werk"}, 5)
        table = vd.encode_table(1, 5, entries_with({1: entry}), collections)
        result = vd.analyse(bytes(image_with({1: table}, payload)))
        self.assertEqual(result["collections"][0], {"index": 1, "domain": 0, "present": True,
                                                    "revision": 1, "name": "Werk"})
        loose = vd.encode_table(1, 5, entries_with({1: dict(entry, collection=0)}), collections)
        other = vd.analyse(bytes(image_with({1: loose}, payload)))
        self.assertNotEqual(result["digests"][0], other["digests"][0])
        self.assertEqual(result["digests"][1], other["digests"][1])
        missing = vd.encode_table(1, 5, entries_with({1: dict(entry, collection=2)}), collections)
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with({1: missing}, payload)))
        # Domain B's collection 1 does not make domain A's collection 1 exist.
        foreign = collections_with({vd.COLLECTIONS: b"Fotos"}, 5)
        table = vd.encode_table(2, 5, entries_with({1: entry}), foreign, clocks=[1, 1])
        with self.assertRaisesRegex(ValueError, "missing collection"):
            vd.decode_table(0, table)
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with(alone(2, table), payload)))

    def test_domain_clocks_add_up_and_bound_every_stamp(self):
        entry, payload = document(1, 1, 10, [0], 5)
        good = vd.analyse(bytes(image_with({1: vd.encode_table(1, 5, entries_with({1: entry}))}, payload)))
        self.assertEqual(good["clocks"], [1, 0])
        for clocks, reason in (([0, 0], "add up"), ([1, 1], "add up"), ([0, 1], "domain clock")):
            table = vd.encode_table(1, 5, entries_with({1: entry}), clocks=clocks)
            with self.assertRaisesRegex(ValueError, reason):
                vd.decode_table(1, table)
            with self.assertRaises(ValueError):
                vd.analyse(bytes(image_with({1: table}, payload)))
        # A collection revision is bounded by its own domain's clock too.
        collections = collections_with({vd.COLLECTIONS: b"Fotos"}, 6)
        table = vd.encode_table(1, 6, entries_with({}), collections, clocks=[1, 0])
        with self.assertRaisesRegex(ValueError, "domain clock"):
            vd.decode_table(1, table)
        # Each object bound on its own: generation 2 admits the value, only
        # the owning domain's clock (1) does not.
        for field in ("modified", "revision"):
            table = vd.encode_table(2, 5, entries_with({1: dict(entry, **{field: 2})}), clocks=[1, 1])
            with self.assertRaisesRegex(ValueError, "domain clock"):
                vd.decode_table(0, table)

    def test_commit_may_be_reflected_by_a_collection_only(self):
        collections = collections_with({0: b"Werk"}, 6)
        result = vd.analyse(bytes(image_with({1: vd.encode_table(1, 6, entries_with({}), collections)})))
        self.assertEqual(result["generation"], 1)
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with({1: vd.encode_table(1, 7, entries_with({}), collections)})))

    def test_torn_next_table_falls_back_to_previous_generation(self):
        old, payload = document(0, 1, 100, [0], 5)
        new, payload2 = document(0, 2, 100, [1], 6)
        gen1 = vd.encode_table(1, 5, entries_with({0: old}))
        gen2 = vd.encode_table(2, 6, entries_with({0: new}))
        torn = gen2[:4] + [bytes(vd.SECTOR)] * (vd.TABLE_SECTORS - 4)
        result = vd.analyse(bytes(image_with({1: gen1, 0: torn}, {**payload, **payload2})))
        self.assertEqual((result["generation"], result["objects"][0]["revision"]), (1, 1))
        # The oracle reports the per-sector generations of both copies.
        self.assertEqual(result["slots"][0], [2] * 4 + [0] * (vd.TABLE_SECTORS - 4))
        self.assertEqual(result["slots"][1], [1] * vd.TABLE_SECTORS)
        complete = vd.analyse(bytes(image_with({1: gen1, 0: gen2}, {**payload, **payload2})))
        self.assertEqual(complete["generation"], 2)

    def test_sealed_but_noncanonical_newest_copy_is_refused(self):
        bad = dict(vd.empty_version(), revision=1, name=b"a/b", content_sum=vd.FNV_OFFSET)
        entry = dict(vd.empty_entry(), revision=1, request_id=5, created=1, modified=1, present=True,
                     current=bad)
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with({1: vd.encode_table(1, 5, entries_with({0: entry}))})))

    def test_unrelated_generations_are_refused(self):
        a, _ = document(0, 1, 0, [], 5)
        b, _ = document(0, 1, 0, [], 9)
        tables = {1: vd.encode_table(1, 5, entries_with({0: a})),
                  0: vd.encode_table(4, 9, entries_with({0: b}))}
        with self.assertRaises(ValueError):
            vd.analyse(bytes(image_with(tables)))

    def test_sectors_of_two_attempts_never_form_one_copy(self):
        first, _ = document(0, 1, 0, [], 5)
        gen1 = vd.encode_table(1, 5, entries_with({0: first}))
        a = dict(first, revision=2, modified=2, request_id=6, current=dict(first["current"], name=b"Eerste"))
        b = dict(first, revision=2, modified=2, request_id=7, current=dict(first["current"], name=b"Tweede"))
        attempt_a = vd.encode_table(2, 6, entries_with({0: a}))
        attempt_b = vd.encode_table(2, 7, entries_with({0: b}))
        mixed = attempt_b[:1] + attempt_a[1:]
        self.assertTrue(all(vd.classify(0, i, sector)[:2] == ("valid", 2) for i, sector in enumerate(mixed)))
        result = vd.analyse(bytes(image_with({1: gen1, 0: mixed})))
        self.assertEqual(result["generation"], 1)
        complete = vd.analyse(bytes(image_with({1: gen1, 0: attempt_a})))
        self.assertEqual(complete["generation"], 2)

    def test_corrupt_fixture_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "c.img"
            vd.create_image(path, "corrupt")
            with self.assertRaises(ValueError):
                vd.analyse(path.read_bytes())

    def test_foreign_volume_fixture_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "foreign.img"
            vd.create_image(path, "foreign")
            with self.assertRaisesRegex(ValueError, "unmarked volume"):
                vd.analyse(path.read_bytes())

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
        for bad in (b"", b".", b"..", b"a/b", b"a\\b", b"\x00", "x\u202ey".encode(), b"\xff", b"a" * 33):
            self.assertFalse(vd.valid_name(bad))


if __name__ == "__main__":
    unittest.main()
