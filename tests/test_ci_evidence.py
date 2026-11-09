"""Regression tests for bounded, symlink-safe CI evidence collection."""
import hashlib
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import ci_evidence


class EvidenceCollectorTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def collect(self, patterns):
        return ci_evidence.collect(
            name="collector-regression",
            paths=patterns,
            output=Path("collected"),
            status="success",
            workspace=self.root,
            environ={},
            stdout=io.StringIO(),
        )

    def test_retains_exact_bytes_and_hash(self):
        report = b'{"ok":true}\n'
        (self.root / "report.json").write_bytes(report)
        receipt = self.collect("report.json")
        self.assertEqual(receipt["evidence_state"], "compact-reports-retained")
        self.assertEqual(receipt["retained"], [{
            "path": "report.json",
            "size_bytes": len(report),
            "sha256": hashlib.sha256(report).hexdigest(),
        }])
        self.assertEqual((self.root / "collected/report.json").read_bytes(), report)
        persisted = json.loads((self.root / "collected/_meta/receipt.json").read_text())
        self.assertEqual(persisted["retained"], receipt["retained"])

    def test_overlapping_directories_count_unique_traversal(self):
        directory = self.root / "reports"
        (directory / "sub").mkdir(parents=True)
        (directory / "a.json").write_text("{}\n")
        (directory / "sub/b.json").write_text("{}\n")
        with patch.object(ci_evidence, "MAX_SCAN_ENTRIES", 3):
            receipt = self.collect("reports\nreports/sub")
        self.assertEqual(
            {entry["path"] for entry in receipt["retained"]},
            {"reports/a.json", "reports/sub/b.json"},
        )

    @unittest.skipUnless(os.name == "posix", "requires POSIX O_NOFOLLOW")
    def test_symlinks_are_not_collected_or_followed(self):
        (self.root / "target.txt").write_text("outside collection")
        (self.root / "link.txt").symlink_to(self.root / "target.txt")
        receipt = self.collect("link.txt")
        self.assertEqual(receipt["retained"], [])
        self.assertEqual(receipt["omitted_reason_counts"].get("symlink"), 1)
        self.assertFalse((self.root / "collected/link.txt").exists())
        with self.assertRaises(OSError):
            ci_evidence._read_report(self.root, "link.txt")

    def test_sensitive_content_is_rejected(self):
        (self.root / "runner.log").write_text("Authorization: Bearer synthetic-test-value\n")
        receipt = self.collect("runner.log")
        self.assertEqual(receipt["retained"], [])
        self.assertEqual(receipt["omitted_reason_counts"].get("credential-like-content"), 1)
        self.assertFalse((self.root / "collected/runner.log").exists())

    def test_per_file_byte_limit_is_enforced(self):
        (self.root / "big.log").write_bytes(b"x" * (ci_evidence.MAX_FILE_BYTES + 1))
        receipt = self.collect("big.log")
        self.assertEqual(receipt["retained"], [])
        self.assertEqual(receipt["omitted_reason_counts"].get("per-file-byte-limit"), 1)

    def test_file_count_limit_reserves_receipt_slot(self):
        (self.root / "a.json").write_text("{}\n")
        (self.root / "b.json").write_text("{}\n")
        with patch.object(ci_evidence, "MAX_FILES", 2):
            receipt = self.collect("a.json\nb.json")
        self.assertEqual([row["path"] for row in receipt["retained"]], ["a.json"])
        self.assertEqual(receipt["omitted_reason_counts"].get("file-count-limit"), 1)

    def test_total_byte_limit_is_enforced(self):
        (self.root / "a.json").write_bytes(b'{"x":1}\n')
        (self.root / "b.json").write_bytes(b'{"x":2}\n')
        with patch.object(ci_evidence, "MAX_TOTAL_BYTES", ci_evidence.MAX_FILE_BYTES + 8):
            receipt = self.collect("a.json\nb.json")
        self.assertEqual([row["path"] for row in receipt["retained"]], ["a.json"])
        self.assertEqual(receipt["omitted_reason_counts"].get("total-byte-limit"), 1)

    def test_receipt_truncates_omissions_without_losing_retained_hashes(self):
        digest = "a" * 64
        receipt = {
            "warnings": [],
            "retained": [{"path": "report.json", "sha256": digest}],
            "omitted": [{"path": f"discarded-{n:03d}.log", "reason": "limit"}
                        for n in range(100)],
        }
        with patch.object(ci_evidence, "MAX_FILE_BYTES", 650):
            encoded = ci_evidence._receipt_bytes(receipt)
        self.assertLessEqual(len(encoded), 650)
        parsed = json.loads(encoded)
        self.assertGreater(parsed["omitted_list_truncated_count"], 0)
        self.assertEqual(parsed["retained"][0]["sha256"], digest)


if __name__ == "__main__":
    unittest.main()
