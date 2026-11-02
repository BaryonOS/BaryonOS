"""Host tests for the M6e/S06 runner's cut evidence rule (tools/run_volume.py).

Invariant: a native cut case only counts as "killed inside the table write"
when the other table copy holds a real torn attempt at the interrupted
generation; a content cut must leave that copy untouched.
"""
from __future__ import annotations
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_volume as rv  # noqa: E402
import volume_disk as vd  # noqa: E402

N = vd.TABLE_SECTORS


def oracle(generation: int, other: list[int]) -> dict:
    slots = [None, None]
    slots[generation % 2] = [generation] * N
    slots[(generation + 1) % 2] = other
    return {"generation": generation, "slots": slots}


class TornTableWriteTests(unittest.TestCase):
    def test_formatted_generation_zero_remains_valid_cut_evidence(self):
        image = bytearray(vd.empty_image_bytes())
        entries = [vd.empty_entry() for _ in range(vd.OBJECTS)]
        entries[0] = dict(vd.empty_entry(), revision=1, request_id=5, created=1, modified=1, present=True,
                          current=dict(vd.empty_version(), revision=1, name=b"Doc", content_sum=vd.FNV_OFFSET))
        start = (vd.FIRST + vd.TABLE_SECTORS) * vd.SECTOR
        image[start:start + vd.TABLE_SECTORS * vd.SECTOR] = b"".join(vd.encode_table(1, 5, entries))
        # Fully validated format sectors and legacy zero sectors both report
        # numeric generation 0; admission has already checked their difference.
        self.assertEqual(rv.torn_sectors(vd.analyse(image), False), 0)
        entries[0] = dict(entries[0], revision=2, request_id=6, modified=2,
                          current=dict(entries[0]["current"], revision=2))
        start = vd.FIRST * vd.SECTOR
        image[start:start + 4 * vd.SECTOR] = b"".join(vd.encode_table(2, 6, entries)[:4])
        self.assertEqual(rv.torn_sectors(vd.analyse(image), True), 4)

    def test_torn_prefix_after_the_previous_copy_is_accepted(self):
        self.assertEqual(rv.torn_sectors(oracle(3, [4] * 5 + [2] * (N - 5)), True), 5)
        # The first commit's companion has format generation 0; a legacy
        # never-written zero companion has the same numeric sector summary.
        self.assertEqual(rv.torn_sectors(oracle(1, [2] * 4 + [0] * (N - 4)), True), 4)

    def test_untouched_or_unexplained_copies_are_refused_for_a_table_cut(self):
        for other in ([2] * N,                          # cut before the table write
                      [4] * N,                          # not torn at all
                      [2] + [4] * 4 + [2] * (N - 5),    # not a prefix
                      [4] * 5 + [-1] + [2] * (N - 6),   # garbage after the prefix
                      [4] * 5 + [0] * (N - 5)):         # previous copy missing
            with self.assertRaises(ValueError, msg=str(other[:8])):
                rv.torn_sectors(oracle(3, other), True)

    def test_content_cut_leaves_the_other_copy_untouched(self):
        self.assertEqual(rv.torn_sectors(oracle(1, [0] * N), False), 0)
        with self.assertRaises(ValueError):
            rv.torn_sectors(oracle(1, [2] + [0] * (N - 1)), False)


if __name__ == "__main__":
    unittest.main()
