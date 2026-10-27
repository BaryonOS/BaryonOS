"""Actual synthetic files test the external oracle, not guest device execution."""
import sys
from pathlib import Path
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from common import BUILD
from m5_disk import create_disk, verify_disk


class M5DiskOracle(unittest.TestCase):
    def folder(self, *, run=True):
        parent = BUILD / "run" if run else BUILD / "m5-oracle-outside"
        parent.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="oracle-test-", dir=parent)
        Path(temporary.name).resolve().relative_to(parent.resolve())
        self.addCleanup(temporary.cleanup)
        return Path(temporary.name)

    @staticmethod
    def lines(info):
        return "\n".join(f"M5:SECTOR{s}_DIGEST:{v}" for s, v in info["expected_digests"].items())

    def test_two_images_need_their_own_actual_data_digest(self):
        a, ai = create_disk(self.folder(), 1)
        b, bi = create_disk(self.folder(), 2)
        self.assertNotEqual(ai["sha256_before"], bi["sha256_before"])
        self.assertNotEqual(ai["expected_digests"], bi["expected_digests"])
        self.assertTrue(verify_disk(a, ai, self.lines(ai), data_required=True)["passed"])
        self.assertTrue(verify_disk(b, bi, self.lines(bi), data_required=True)["passed"])
        self.assertFalse(verify_disk(b, bi, self.lines(ai), data_required=True)["passed"])

    def test_pass_marker_missing_or_duplicate_or_extra_digest_never_suffices(self):
        p, info = create_disk(self.folder(), 3)
        lines = self.lines(info)
        for text in ("M5:PASS\nM1:PASS", lines.splitlines()[0], lines + "\n" + lines,
                     lines + "\nM5:SECTOR7_DIGEST:0000000000000000", lines + "\nM5:SECTOR4_DIGEST:bad"):
            with self.subTest(text=text):
                self.assertFalse(verify_disk(p, info, text, data_required=True)["passed"])

    def test_volume_modification_fails_even_with_matching_guest_digest(self):
        p, info = create_disk(self.folder(), 4)
        data = bytearray(p.read_bytes())
        data[0] ^= 1  # Outside the read scopes: whole-image guard must still catch it.
        p.write_bytes(data)
        self.assertFalse(verify_disk(p, info, self.lines(info), data_required=True)["passed"])

    def test_absent_and_quarantine_cases_require_no_data_delivery(self):
        p, info = create_disk(self.folder(), 5)
        self.assertTrue(verify_disk(p, info, "M5:CASE:NO_DEVICE:PASS", data_required=False)["passed"])
        self.assertFalse(verify_disk(p, info, self.lines(info), data_required=False)["passed"])

    def test_only_fresh_files_under_generated_run_tree(self):
        with self.assertRaises(ValueError):
            create_disk(self.folder(run=False), 1)
        folder = self.folder()
        p, info = create_disk(folder, 7)
        with self.assertRaises(ValueError):
            create_disk(folder, 8)
        self.assertEqual(p.stat().st_size, 65536)
        self.assertTrue(verify_disk(p, info, self.lines(info), data_required=True)["passed"])


if __name__ == "__main__":
    unittest.main()
