"""Host tests for the S09 second-disk runner checks (tools/run_volume.py).

Invariant: after a power cut during an export over an older archive, the
archive disk counts as torn (only Corrupt is correct) exactly when sector 0
holds the expected new header and collection records, every changed sector
is a prefix of the expected new archive and lies before the cut limit, and
the reader refuses the result. An untouched older archive, a complete new
one or a damaged sector 0 is not a torn disk. Disk 0 differs from its input
only by the one new collection.
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


# Varied content, so the 48-byte shift between the two archives changes every sector.
OBJECTS = [(slot, f"Doc {slot}".encode(), 1, 0, 0, 1, bytes((slot * 7 + i) % 251 for i in range(6000)))
           for slot in range(3)]
OLDER = encode([(1, b"Werk")], OBJECTS, 7)
# The cut stage's new archive: the portal added collection 3 "Nieuw", so only
# the header (one more collection, newer generation) and the records differ.
NEWER = encode([(1, b"Werk"), (3, b"Nieuw")], OBJECTS, 9)
ADDED = (3, "Nieuw")


def torn(cut: int) -> bytes:
    after = bytearray(disk(OLDER))
    after[:cut * SECTOR] = NEWER[:cut * SECTOR]
    return bytes(after)


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
        for cut in (1, 5, 20):
            result = rv.torn_archive(disk(OLDER), torn(cut), 20, 9, ADDED)
            self.assertEqual(result["classification"], "corrupt")
            self.assertEqual(result["last_changed"], cut - 1)
            self.assertEqual(result["new_length"], len(NEWER))

    def test_corrupted_magic_alone_is_not_torn(self):
        after = bytearray(disk(OLDER))
        after[0] ^= 0xff
        with self.assertRaisesRegex(ValueError, "new archive header"):
            rv.torn_archive(disk(OLDER), bytes(after), 20, 9, ADDED)

    def test_resealed_nonzero_reserved_bytes_are_not_torn(self):
        def reseal(at: int, size: int, offset: int) -> bytes:
            after = bytearray(torn(5))
            after[at + offset] = 1
            body = bytes(after[at:at + size - 8])
            after[at + size - 8:at + size] = va.fnv(body).to_bytes(8, "little")
            self.assertTrue(va._sealed(bytes(after[at:at + size])))
            return bytes(after)
        for offset in (14, 15, 32, 55):
            with self.assertRaisesRegex(ValueError, "new archive header"):
                rv.torn_archive(disk(OLDER), reseal(0, va.HEADER, offset), 20, 9, ADDED)
        second = va.HEADER + va.COLLECTION_RECORD
        for offset in (2, 7, 8 + len("Nieuw"), 39):
            with self.assertRaisesRegex(ValueError, "collection record 3"):
                rv.torn_archive(disk(OLDER), reseal(second, va.COLLECTION_RECORD, offset), 20, 9, ADDED)

    def test_new_header_must_carry_the_domain_generation_and_collection(self):
        with self.assertRaisesRegex(ValueError, "new archive header"):
            rv.torn_archive(disk(OLDER), torn(5), 20, 8, ADDED)
        with self.assertRaisesRegex(ValueError, "collection record 3"):
            rv.torn_archive(disk(OLDER), torn(5), 20, 9, (3, "Ander"))

    def test_changed_sectors_must_be_the_new_prefix(self):
        after = bytearray(torn(5))
        after[3 * SECTOR] ^= 1
        with self.assertRaisesRegex(ValueError, "new archive prefix"):
            rv.torn_archive(disk(OLDER), bytes(after), 20, 9, ADDED)

    def test_untouched_older_archive_is_not_torn(self):
        before = disk(OLDER)
        with self.assertRaisesRegex(ValueError, "torn new header"):
            rv.torn_archive(before, before, 20, 9, ADDED)

    def test_complete_new_archive_is_not_torn(self):
        with self.assertRaises(ValueError):
            rv.torn_archive(disk(OLDER), disk(NEWER), 20, 9, ADDED)

    def test_write_past_the_cut_limit_is_rejected(self):
        after = bytearray(torn(1))
        after[20 * SECTOR] ^= 1
        with self.assertRaisesRegex(ValueError, "torn new header"):
            rv.torn_archive(disk(OLDER), bytes(after), 20, 9, ADDED)

    def test_cut_stage_disk0_differs_only_by_the_new_collection(self):
        def oracle(nieuw: bool, objects: int = 2) -> dict:
            return {"generation": 10 + nieuw, "clocks": [5 + nieuw, 3], "chunks_used": 4,
                    "digests": ["d0", "d1"],
                    "objects": [{"object": o, "present": o < objects, "revision": 1} for o in range(16)],
                    "collections": [{"domain": 0, "index": 1, "present": True, "revision": 1, "name": "Werk"},
                                    {"domain": 0, "index": 3, "present": nieuw, "revision": int(nieuw),
                                     "name": "Nieuw" if nieuw else ""}]}
        rv.cut_volume(oracle(False), oracle(True))
        with self.assertRaisesRegex(ValueError, "beyond one new collection"):
            rv.cut_volume(oracle(False), oracle(True, objects=1))
        changed = oracle(True)
        changed["collections"][0]["name"] = "Anders"
        with self.assertRaisesRegex(ValueError, "changed collection"):
            rv.cut_volume(oracle(False), changed)
        present = oracle(True)
        present.update(generation=10, clocks=[5, 3])
        with self.assertRaisesRegex(ValueError, "new collection is not"):
            rv.cut_volume(present, oracle(True))

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
