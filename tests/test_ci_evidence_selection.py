"""Bound overlapping evidence globs without repeated directory traversal."""

import io
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import ci_evidence


class EvidenceSelectionTests(unittest.TestCase):
    def test_overlapping_directories_do_not_exhaust_scan_limit(self):
        """Selection:add must not rescan a directory returned by overlapping patterns."""
        with tempfile.TemporaryDirectory(prefix="ci-selection-") as temporary:
            root = Path(temporary)
            (root / "reports/sub").mkdir(parents=True)
            (root / "reports/a.json").write_text("{}\n", encoding="utf-8")
            (root / "reports/sub/b.json").write_text("{}\n", encoding="utf-8")
            with patch.object(ci_evidence, "MAX_SCAN_ENTRIES", 3):
                receipt = ci_evidence.collect(
                    name="selection-regression",
                    paths="reports\nreports/sub",
                    output=root / "evidence",
                    status="success",
                    workspace=root,
                    environ={},
                    stdout=io.StringIO(),
                )
            self.assertEqual(
                {record["path"] for record in receipt["retained"]},
                {"reports/a.json", "reports/sub/b.json"},
            )
            self.assertEqual(receipt["omitted_count"], 0)


if __name__ == "__main__":
    unittest.main()
