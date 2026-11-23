"""Host tests for the U05-N1 settings oracle and damage helpers (no QEMU).

Records are built here byte by byte with hashlib, independent of
settings-core and of the runner's encoder, then decoded by the runner's
oracle. Damage helpers run on synthetic volume images from tools/volume_disk.py.
"""
from __future__ import annotations
import hashlib
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_settings as rs  # noqa: E402
import volume_disk as vd  # noqa: E402


def settings(theme: int, motion: int, scale: int) -> bytes:
    return bytes([theme, motion]) + scale.to_bytes(2, "little") + bytes(4)


def record(generation: int, current: bytes, previous: bytes | None, *, schema: int = 1,
           patch: dict[int, int] | None = None) -> bytes:
    body = bytearray(64)
    body[0:8] = b"NOSSET01"
    body[8:12] = schema.to_bytes(4, "little")
    body[12] = 1 if previous is not None else 0
    body[16:24] = generation.to_bytes(8, "little")
    body[24:32] = current
    body[32:40] = previous or bytes(8)
    for offset, value in (patch or {}).items():
        body[offset] = value
    return bytes(body) + hashlib.sha256(bytes(body)).digest()


A1, A2 = settings(1, 0, 100), settings(2, 1, 150)
B1, B2 = settings(0, 0, 125), settings(1, 1, 200)


def value(raw: bytes) -> int:
    return int.from_bytes(raw, "little")


def synthetic(commits):
    """Volume image after `commits` [(object, content)], one revision per
    commit, each into a fresh chunk; the last two generations hold tables."""
    image = bytearray(vd.empty_image_bytes())
    entries = [vd.empty_entry() for _ in range(vd.OBJECTS)]
    clocks = [0] * vd.DOMAINS
    tables = []
    for generation, (obj, content) in enumerate(commits, start=1):
        domain = obj // vd.PER_DOMAIN
        clocks[domain] += 1
        chunk = generation - 1
        start = (vd.FIRST + vd.DATA_START + chunk * vd.CHUNK_SECTORS) * vd.SECTOR
        image[start:start + len(content)] = content
        old = entries[obj]
        version = {"revision": old["revision"] + 1, "length": len(content), "name": rs.SLOT_NAMES[obj % 8].encode(),
                   "content_sum": vd.fnv(content), "chunks": [chunk], "sums": [vd.fnv(content)]}
        history = ([old["current"]] + old["history"])[:vd.HISTORY] if old["present"] else []
        entries[obj] = dict(old, revision=old["revision"] + 1, request_id=100 + generation,
                            created=old["created"] or clocks[domain], modified=clocks[domain],
                            present=True, current=version, history=history)
        tables.append((generation, vd.encode_table(generation, 100 + generation,
                                                   [dict(e) for e in entries], clocks=list(clocks))))
    for generation, sectors in tables[-2:]:
        for index, sector in enumerate(sectors):
            offset = (vd.FIRST + (generation % 2) * vd.TABLE_SECTORS + index) * vd.SECTOR
            image[offset:offset + vd.SECTOR] = sector
    return bytes(image)


def step3_image() -> bytes:
    """Domain 0 and 1 after the writable steps 1 and 3 (init, set, revert)."""
    return synthetic([(0, record(1, A1, None)), (8, record(1, B1, None)),
                      (1, record(2, A2, A1)), (9, record(2, B2, B1)),
                      (0, record(3, A1, A2)), (8, record(3, B1, B2))])


class RecordOracle(unittest.TestCase):
    def test_canonical_record_decodes(self):
        self.assertEqual(rs.decode_record(record(2, A2, A1)), ("good", 2, value(A2), value(A1)))
        self.assertEqual(rs.decode_record(record(1, B1, None)), ("good", 1, value(B1), None))
        self.assertEqual(rs.decode_record(bytes(96)), ("blank",))
        # The runner's encoder agrees byte for byte with this independent one.
        self.assertEqual(rs.encode_record(2, value(A2), value(A1)), record(2, A2, A1))

    def test_resealed_invalid_fields_stay_bad(self):
        for patch in ({13: 1}, {40: 1}, {63: 1}, {12: 2}, {24: 3}, {25: 2}, {28: 1}):
            with self.subTest(patch=patch):
                self.assertEqual(rs.decode_record(record(2, A2, A1, patch=patch)), ("bad",))
        self.assertEqual(rs.decode_record(record(0, A1, None)), ("bad",))
        self.assertEqual(rs.decode_record(record(1, settings(1, 0, 74), None)), ("bad",))
        self.assertEqual(rs.decode_record(record(1, settings(1, 0, 301), None)), ("bad",))
        self.assertEqual(rs.decode_record(record(1, A1, None, patch={32: 1})), ("bad",))
        flipped = bytearray(record(1, A1, None))
        flipped[20] ^= 1
        self.assertEqual(rs.decode_record(bytes(flipped)), ("bad",))
        self.assertEqual(rs.decode_record(b"NOSSET02" + record(1, A1, None)[8:]), ("bad",))
        with self.assertRaises(ValueError):
            rs.decode_record(bytes(95))

    def test_unknown_schema_with_valid_digest(self):
        self.assertEqual(rs.decode_record(record(1, A1, None, schema=2)), ("unsupported",))

    def test_recovery_selection(self):
        good1, good2 = record(1, A1, None), record(2, A2, A1)
        bad = rs.damaged(good2)
        self.assertEqual(rs.expected_recovery([good1, good2]), (2, value(A2), 0))
        self.assertEqual(rs.expected_recovery([good1, bad]), (1, value(A1), 0))
        self.assertEqual(rs.expected_recovery([bad, rs.damaged(good1)]), (0, 0, rs.CORRUPT))
        self.assertEqual(rs.expected_recovery([good2, record(2, A1, A2)]), (0, 0, rs.CORRUPT))
        self.assertEqual(rs.expected_recovery([good2, record(1, A1, None, schema=2)]), (0, 0, rs.UNSUPPORTED))
        self.assertEqual(rs.expected_recovery([bytes(96), bytes(96)]), (0, 0, 3))

    def test_fixture_values_match_the_app_encoding(self):
        self.assertEqual((rs.S1[0], rs.S2[0], rs.S1[1], rs.S2[1]),
                         (value(A1), value(A2), value(B1), value(B2)))
        self.assertEqual(rs.S2[0], 2 | 1 << 8 | 150 << 16)


class DamageHelpers(unittest.TestCase):
    def test_synthetic_image_is_accepted(self):
        oracle = vd.analyse(step3_image(), rs.OPAQUE)
        self.assertEqual(oracle["generation"], 6)
        self.assertEqual(oracle["clocks"], [3, 3])
        for domain in range(2):
            slots, _ = rs.domain_slots(step3_image(), domain)
            self.assertEqual(rs.expected_recovery(slots), rs.STEPS[3][2][domain])

    def test_rewrite_repairs_both_table_copies(self):
        image = step3_image()
        slots, _ = rs.domain_slots(image, 0)
        changed = rs.rewrite_object(image, 0, rs.damaged(slots[0]))
        self.assertEqual(rs.domain_slots(changed, 0)[0], [rs.damaged(slots[0]), slots[1]])
        self.assertEqual(rs.domain_slots(changed, 1)[0], rs.domain_slots(image, 1)[0])
        volume = changed[vd.FIRST * vd.SECTOR:(vd.FIRST + vd.VOLUME_SECTORS) * vd.SECTOR]
        chunk = vd.choose(volume)["entries"][0]["current"]["chunks"][0]
        for slot in range(2):
            table = vd.decode_table(slot, vd.slot_states(volume, slot)[0])
            for entry in table["entries"]:
                for version in vd.retained(entry):
                    if version["chunks"] == [chunk]:
                        self.assertEqual(version["sums"], [vd.fnv(rs.damaged(slots[0]))])
        with self.assertRaises(ValueError):
            rs.rewrite_object(image, 0, bytes(95))

    def test_derived_inputs_give_the_step_expectations(self):
        image = step3_image()
        for step in (4, 5, 6):
            _, source, want = rs.STEPS[step]
            with self.subTest(step=step):
                derived = rs.derive(image, source)
                for domain in range(2):
                    self.assertEqual(rs.expected_recovery(rs.domain_slots(derived, domain)[0]), want[domain])
                self.assertEqual(rs.domain_slots(derived, 1)[0], rs.domain_slots(image, 1)[0])

    def test_unknown_source_refused(self):
        with self.assertRaises(ValueError):
            rs.derive(step3_image(), "other")


if __name__ == "__main__":
    unittest.main()
