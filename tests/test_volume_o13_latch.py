"""Host tests for the S09 O13 N38 slice 1 oracle (tools/run_volume.py check_latch).

The native run is `tools/run_volume.py --case o13-latch` after
`tools/build.py --kernel-feature test-volume-o13-latch`; these tests only show
that the oracle refuses a transcript or disk that lacks the required evidence.
"""
from __future__ import annotations
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_volume as rv  # noqa: E402

SECTOR = rv.SECTOR
SECTORS = 16


def transcript(values: dict[str, int] | None = None, *, injected: bool = True) -> str:
    """A passing kernel transcript of the o13-latch case, with `values` replaced."""
    evidence = dict(rv.LATCH_EVIDENCE, **{"VOLUME:LATCH:VOLUME_WRITES_AFTER:": 36})
    evidence.update(values or {})
    marker = ["VOLUME:MANIFEST:PASS"]
    if injected:
        marker.append(rv.LATCH_INJECTED)
    marker += [f"{label}{evidence.pop(label):020d}" for label in
               ("VOLUME:LATCH:LATCHED_AT_COMPLETION:", "VOLUME:LATCH:POLLS_BEFORE_MARKER:")]
    marker.append("VOLUME:READY")
    marker += [f"{label}{value:020d}" for label, value in evidence.items()]
    marker.append("VOLUME:CASE:O13_LATCH:PASS")
    return "\n".join(marker) + "\n"


def disks(sector_bytes: dict[int, bytes] | None = None) -> tuple[bytes, bytes]:
    before = rv.disk1_image(SECTORS)
    after = bytearray(before)
    for index, data in {1: rv.LATCH_WRITTEN, 2: rv.LATCH_WRITTEN, **(sector_bytes or {})}.items():
        after[index * SECTOR:(index + 1) * SECTOR] = data
    return before, bytes(after)


def check(out: str, before_after: tuple[bytes, bytes] | None = None) -> dict:
    before, after = before_after or disks()
    return rv.check_latch(out, rv.numbers(out), before, after)


class LatchOracle(unittest.TestCase):
    def test_passing_evidence(self):
        oracle = check(transcript())
        self.assertEqual(oracle["latch"]["VOLUME:LATCH:LATCHED_AT_COMPLETION:"], 1)
        self.assertTrue(oracle["injected_sector_stored"])
        # The injected WRITE may also not have reached the device.
        self.assertFalse(check(transcript(), disks({2: rv.disk1_image(3)[2 * SECTOR:]}))["injected_sector_stored"])

    def test_no_injection_is_refused(self):
        with self.assertRaisesRegex(ValueError, "not injected exactly once"):
            check(transcript(injected=False))
        with self.assertRaisesRegex(ValueError, "INJECTED"):
            check(transcript({"VOLUME:LATCH:INJECTED:": 0}))

    def test_open_latch_or_a_poll_first_is_refused(self):
        # An open entry at the completion is the "latch set at the poll" mutation.
        for label, value in (("VOLUME:LATCH:LATCHED_AT_COMPLETION:", 0), ("VOLUME:LATCH:POLLS_BEFORE_MARKER:", 1),
                             ("VOLUME:LATCH:LATCHED:", 0)):
            with self.subTest(label=label), self.assertRaisesRegex(ValueError, label):
                check(transcript({label: value}))

    def test_marker_must_follow_the_injection(self):
        out = transcript().replace(f"{rv.LATCH_INJECTED}\n", "")
        out = out.replace("VOLUME:READY\n", f"VOLUME:READY\n{rv.LATCH_INJECTED}\n")
        with self.assertRaisesRegex(ValueError, "does not follow"):
            check(out)

    def test_published_refused_write_or_missing_read_flush_is_refused(self):
        for label, value in (("VOLUME:LATCH:REFUSALS:", 1), ("VOLUME:LATCH:WRITES_AT_REFUSAL:", 1),
                             ("VOLUME:BROKER:ARCHIVE_DISK_WRITES:", 3), ("VOLUME:LATCH:ARCHIVE_READS_AFTER:", 0),
                             ("VOLUME:LATCH:ARCHIVE_FLUSHES_AFTER:", 0), ("VOLUME:LATCH:ENTRIES:", 2)):
            with self.subTest(label=label), self.assertRaisesRegex(ValueError, label):
                check(transcript({label: value}))

    def test_no_device0_write_after_the_latch_is_refused(self):
        with self.assertRaisesRegex(ValueError, "device-0 WRITE"):
            check(transcript({"VOLUME:LATCH:VOLUME_WRITES_AFTER:": 0}))

    def test_disk1_bytes(self):
        with self.assertRaisesRegex(ValueError, "answered Ok"):
            check(transcript(), disks({1: rv.disk1_image(2)[SECTOR:]}))
        for sector in (3, 4, 9):
            with self.subTest(sector=sector), self.assertRaisesRegex(ValueError, "outside the admitted"):
                check(transcript(), disks({sector: rv.LATCH_REFUSED}))


if __name__ == "__main__":
    unittest.main()
