"""Host tests for the S09 K5 per-disk admission oracle (tools/run_volume.py
check_k5).

The native run is `tools/run_volume.py --case k5` after
`tools/build.py --kernel-feature test-volume-k5`; these tests only show that
the oracle refuses evidence or a disk 1 that lacks what F1 and F2 require.
"""
from __future__ import annotations
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_volume as rv  # noqa: E402

SECTORS = 40


def evidence(**changes: int) -> dict[str, int]:
    values = dict(rv.K5_EVIDENCE, **{label: 1 for label in rv.K5_AT_LEAST})
    values.update({f"VOLUME:ADMISSION:{name}:": value for name, value in changes.items()})
    return values


class K5OracleTests(unittest.TestCase):
    disk = rv.disk1_image(SECTORS)

    def test_passing_evidence(self):
        result = rv.check_k5(evidence(), self.disk, self.disk)
        self.assertEqual(result["k5"]["VOLUME:ADMISSION:REFUSED_IN_RESET:"], 1)

    def test_each_mutation_is_refused(self):
        for name, value in [
            ("ARCHIVE_ADMITTED_DURING_VOLUME_IO", 0),     # F1: disk 1 refused while disk 0 busy
            ("ARCHIVE_COMPLETED_DURING_VOLUME_IO", 0),    # F1: the hold ended too early
            ("VOLUME_COMPLETED_DURING_ARCHIVE_HOLD", 0),  # F2: no disk-0 progress in the hold
            ("REFUSED_IN_RESET", 0),                      # F2: no refusal in the reset window
            ("ADMITTED_IN_RESET", 1),                     # F2: admitted during the reset
            ("RESET_MARGIN", 0),                          # F2: refusal not before the deadline
            ("RESET_MARGIN", rv.RESET_TIMEOUT + 1),
            ("TIMEOUTS", 2),
            ("QUARANTINED", 1),
        ]:
            with self.subTest(name=name, value=value), self.assertRaises(ValueError):
                rv.check_k5(evidence(**{name: value}), self.disk, self.disk)

    def test_missing_label_is_refused(self):
        values = evidence()
        del values["VOLUME:ADMISSION:VOLUME_COMPLETED_DURING_ARCHIVE_HOLD:"]
        with self.assertRaises(ValueError):
            rv.check_k5(values, self.disk, self.disk)

    def test_disk1_must_be_unchanged(self):
        after = bytearray(self.disk)
        after[rv.SECTOR] ^= 1
        with self.assertRaises(ValueError):
            rv.check_k5(evidence(), self.disk, bytes(after))

    def test_labels_are_parsed(self):
        # numbers() accepts only [A-Z_:] labels; a digit would silently drop one.
        lines = [f"{label}{1:020d}" for label in (*rv.K5_EVIDENCE, *rv.K5_AT_LEAST)]
        self.assertEqual(len(rv.numbers("\n".join(lines))), len(lines))

    def test_k5_is_a_fresh_read_only_stage(self):
        self.assertIn("k5", rv.FRESH)
        self.assertIn("k5", rv.DISK1_READ_ONLY)
        self.assertEqual(rv.FEATURE["k5"], "test-volume-k5")


if __name__ == "__main__":
    unittest.main()
