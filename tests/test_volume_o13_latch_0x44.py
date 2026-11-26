"""Host tests for the S09 O13 N38 slice 2 oracle, case (f) (tools/run_volume.py
check_latch_0x44).

The native run is `tools/run_volume.py --case o13-latch-0x44` after
`tools/build.py --kernel-feature test-volume-o13-latch-0x44`; these tests only
show that the oracle refuses a transcript or disk that lacks the required
evidence, one mutation per sub-case marker.
"""
from __future__ import annotations
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_volume as rv  # noqa: E402

SECTOR = rv.SECTOR
SECTORS = 40
EVENTS = [line for _, line in rv.LATCH44_EVENTS]


def transcript(events: list[str] | None = None, values: dict[str, int] | None = None,
               after_ready: list[str] | None = None) -> str:
    """A passing kernel transcript of the o13-latch-0x44 case, with changes."""
    evidence = dict(rv.LATCH44_EVIDENCE, **(values or {}))
    lines = ["VOLUME:MANIFEST:PASS", *(EVENTS if events is None else events), "VOLUME:READY",
             *(after_ready or [])]
    lines += [f"{label}{value:020d}" for label, value in evidence.items()]
    lines.append("VOLUME:CASE:O13_LATCH_0X44:PASS")
    return "\n".join(lines) + "\n"


def disks(sector_bytes: dict[int, bytes] | None = None) -> tuple[bytes, bytes]:
    before = rv.disk1_image(SECTORS)
    after = bytearray(before)
    for index, data in {3: rv.LATCH_WRITTEN, 12: rv.LATCH_LAST, **(sector_bytes or {})}.items():
        after[index * SECTOR:(index + 1) * SECTOR] = data
    return before, bytes(after)


def check(out: str, before_after: tuple[bytes, bytes] | None = None) -> dict:
    before, after = before_after or disks()
    return rv.check_latch_0x44(out, rv.numbers(out), before, after)


def replaced(index: int, line: str) -> list[str]:
    events = list(EVENTS)
    events[index] = line
    return events


class Latch0x44Oracle(unittest.TestCase):
    def test_passing_evidence(self):
        oracle = check(transcript())
        self.assertEqual(oracle["events"], EVENTS)
        self.assertEqual(oracle["latch"]["VOLUME:LATCH:LATCHED_MASK:"], 1)

    def test_each_sub_case_mutation_is_refused(self):
        call = "VOLUME:LATCH_CALL:DEVICE=1:"
        mutations = {
            # f3: 0x44 that always latches, or that latches a partial entry.
            "f3 no entry": (0, call + "FIRST=25:COUNT=10:WRITE=1:RET=0:MASK=0"),
            "f3 part of one entry": (1, call + "FIRST=15:COUNT=10:WRITE=1:RET=0:MASK=2"),
            # f2: a READ-only holder that latches (or is shown holding WRITE).
            "f2 READ-only holder": (2, call + "FIRST=0:COUNT=15:WRITE=0:RET=0:MASK=3"),
            # f3: a refused call that latched something after all.
            "f3 nothing latched": (3, "VOLUME:LATCH_WRITE:SECTOR=3:REFUSED"),
            # f4: -5 for a mixed overlap, or the partly overlapped entry latched.
            "f1/f4 latch call": (5, call + "FIRST=0:COUNT=15:WRITE=1:RET=-5:MASK=0"),
            # f1: a 0x44 that returns 0 but does not latch.
            "f1 WRITE refused": (6, "VOLUME:LATCH_WRITE:SECTOR=3:PUBLISHED"),
            "f4 WRITE admitted": (7, "VOLUME:LATCH_WRITE:SECTOR=12:REFUSED"),
        }
        for label, (index, line) in mutations.items():
            with self.subTest(label), self.assertRaisesRegex(ValueError, label):
                check(transcript(replaced(index, line)))
        with self.subTest("f2 WRITE right"), self.assertRaisesRegex(ValueError, "f2"):
            check(transcript(replaced(2, call + "FIRST=0:COUNT=15:WRITE=1:RET=-5:MASK=0")))
        with self.subTest("f4 partial entry latched"), self.assertRaisesRegex(ValueError, "f1/f4"):
            check(transcript(replaced(5, call + "FIRST=0:COUNT=15:WRITE=1:RET=0:MASK=3")))

    def test_missing_extra_or_late_events_are_refused(self):
        with self.assertRaisesRegex(ValueError, "f4 WRITE admitted"):
            check(transcript(EVENTS[:-1]))
        with self.assertRaisesRegex(ValueError, "after the script"):
            check(transcript(EVENTS + ["VOLUME:LATCH_WRITE:SECTOR=4:PUBLISHED"]))
        with self.assertRaisesRegex(ValueError, "precede VOLUME:READY"):
            check(transcript(EVENTS[:-1], after_ready=EVENTS[-1:]))

    def test_final_evidence_is_exact(self):
        for label, value in (("VOLUME:LATCH:LATCHED_MASK:", 3), ("VOLUME:LATCH:ENTRIES:", 1),
                             ("VOLUME:BROKER:ARCHIVE_DISK_WRITES:", 4),
                             ("VOLUME:BROKER:ARCHIVE_DISK_READS:", 1)):
            with self.subTest(label), self.assertRaisesRegex(ValueError, label):
                check(transcript(values={label: value}))

    def test_disk_is_checked(self):
        with self.assertRaisesRegex(ValueError, "admitted WRITE"):
            check(transcript(), disks({3: rv.LATCH_REFUSED}))
        with self.assertRaisesRegex(ValueError, "admitted WRITE"):
            check(transcript(), disks({12: rv.LATCH_WRITTEN}))
        with self.assertRaisesRegex(ValueError, "outside the admitted"):
            check(transcript(), disks({4: rv.LATCH_REFUSED}))


if __name__ == "__main__":
    unittest.main()
