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
    def test_torn_prefix_after_the_previous_copy_is_accepted(self):
        self.assertEqual(rv.torn_sectors(oracle(3, [4] * 5 + [2] * (N - 5)), True), 5)
        # The first commit's copy (g == 1): the rest was never written.
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
