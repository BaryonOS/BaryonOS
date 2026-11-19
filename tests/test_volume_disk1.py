"""Host tests for the S09 second-disk runner checks (tools/run_volume.py).

Invariant: after a power cut during an export over an older archive, the
archive disk counts as torn (only Corrupt is correct) exactly when the new
header landed, nothing at or past the cut limit changed, and the reader
refuses the result. An untouched older archive or a complete new one is not
a torn disk.
"""
from __future__ import annotations
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_volume as rv  # noqa: E402
import volume_archive as va  # noqa: E402
from test_volume_archive import encode  # noqa: E402

SECTOR = rv.SECTOR


def disk(archive: bytes, sectors: int = rv.DISK1_SECTORS) -> bytes:
    image = bytearray(rv.disk1_image(sectors))
    image[:len(archive)] = archive
    return bytes(image)


def archive(generation: int, objects: int, size: int) -> bytes:
    return encode([(1, b"Werk")], [(slot, f"Doc {slot}".encode(), 1, 0, 0, 1, bytes([slot]) * size)
                                   for slot in range(objects)], generation)


OLDER = archive(7, 3, 6000)
NEWER = archive(9, 4, 6000)


class ArchiveDiskTests(unittest.TestCase):
    def test_fresh_disk_holds_canaries_and_no_archive(self):
        image = rv.disk1_image(rv.SMALL_DISK_SECTORS)
        self.assertEqual(len(image), rv.SMALL_DISK_SECTORS * SECTOR)
        self.assertEqual(image[SECTOR:2 * SECTOR], rv.volume_disk.canary(1))
        rv.refused_disk(image, "fresh disk")

    def test_valid_archive_on_the_disk_is_not_refused(self):
        self.assertEqual(rv.disk_archive(disk(OLDER)), OLDER)
        va.parse(rv.disk_archive(disk(OLDER)))
        with self.assertRaisesRegex(ValueError, "accepted"):
            rv.refused_disk(disk(OLDER), "valid archive")

    def test_new_header_over_older_body_is_torn(self):
        before = disk(OLDER)
        after = bytearray(before)
        cut = 5
        after[:cut * SECTOR] = NEWER[:cut * SECTOR]
        result = rv.torn_archive(before, bytes(after), 20)
        self.assertEqual(result["classification"], "corrupt")
        self.assertEqual(result["last_changed"] < 20, True)

    def test_untouched_older_archive_is_not_torn(self):
        before = disk(OLDER)
        with self.assertRaisesRegex(ValueError, "torn new header"):
            rv.torn_archive(before, before, 20)

    def test_complete_new_archive_is_not_torn(self):
        before = disk(OLDER)
        after = disk(NEWER)
        with self.assertRaises(ValueError):
            rv.torn_archive(before, after, 20)

    def test_write_past_the_cut_limit_is_rejected(self):
        before = disk(OLDER)
        after = bytearray(before)
        after[:SECTOR] = NEWER[:SECTOR]
        after[20 * SECTOR] ^= 1
        with self.assertRaisesRegex(ValueError, "torn new header"):
            rv.torn_archive(before, bytes(after), 20)

    def test_grant_on_the_wrong_device_is_rejected(self):
        evidence = {"VOLUME:ARCHIVE:READS:": 1, "VOLUME:ARCHIVE:WRITES:": 0, "VOLUME:ARCHIVE:FLUSHES:": 0,
                    "VOLUME:BROKER:ARCHIVE_DISK_READS:": 1, "VOLUME:BROKER:ARCHIVE_DISK_WRITES:": 0,
                    "VOLUME:BROKER:ARCHIVE_DISK_FLUSHES:": 0, "VOLUME:ARCHIVE:DEVICE:": 0,
                    "VOLUME:ARCHIVE:DISK_OFFLINE:": 0}
        with self.assertRaisesRegex(ValueError, "wrong device"):
            rv.check_disk("archive-disk-restore", b"", b"", b"", b"", evidence, None)

    def test_scope_refusal_and_escape_probes_are_required(self):
        evidence = {"VOLUME:ARCHIVE:READS:": 1, "VOLUME:ARCHIVE:WRITES:": 0, "VOLUME:ARCHIVE:FLUSHES:": 0,
                    "VOLUME:BROKER:ARCHIVE_DISK_READS:": 1, "VOLUME:BROKER:ARCHIVE_DISK_WRITES:": 0,
                    "VOLUME:BROKER:ARCHIVE_DISK_FLUSHES:": 0, "VOLUME:ARCHIVE:DEVICE:": 1,
                    "VOLUME:ARCHIVE:DISK_OFFLINE:": 0, "VOLUME:ARCHIVE:DISK_FOUND:": 1,
                    "VOLUME:ARCHIVE:SCOPE_REFUSALS:": 0, "VOLUME:ARCHIVE:ESCAPE_DENIALS:": 2}
        with self.assertRaisesRegex(ValueError, "scope refusals"):
            rv.check_disk("archive-disk-restore", b"", b"", b"", b"", evidence, None)
        evidence["VOLUME:ARCHIVE:SCOPE_REFUSALS:"] = 1
        evidence["VOLUME:ARCHIVE:ESCAPE_DENIALS:"] = 1
        with self.assertRaisesRegex(ValueError, "escape probes"):
            rv.check_disk("archive-disk-restore", b"", b"", b"", b"", evidence, None)


if __name__ == "__main__":
    unittest.main()
