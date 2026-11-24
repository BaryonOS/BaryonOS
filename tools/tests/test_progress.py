# SPDX-License-Identifier: MPL-2.0
"""Progress reporting invariants: no partial points, false evidence or stale views."""
import contextlib
import copy
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import progress


class ProgressReviewTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "docs").mkdir()
        (self.root / "evidence.txt").write_text("Synthetic host-test fixture only.\n")
        (self.root / "README.md").write_text("\n".join(progress.README_MARKERS) + "\n")
        (self.root / "docs/PROGRESS.md").write_text("\n".join(progress.RUBRIC_MARKERS) + "\n")
        self.path = self.root / "docs/progress.json"
        root_patch = patch.object(progress, "ROOT", self.root)
        data_patch = patch.object(progress, "DATA", self.path)
        root_patch.start()
        data_patch.start()
        self.addCleanup(root_patch.stop)
        self.addCleanup(data_patch.stop)
        self.data = {
            "schema_version": 1, "rubric_version": 1,
            "assessment": {"date": "2026-11-24", "source_commit": "a" * 40,
                           "branch": "test", "task": "Test report", "summary": "Synthetic fixture",
                           "previous_score": 0},
            "tracks": [{"id": "B", "title": "Test track", "points": 100,
                        "gates": [{"id": "B01", "title": "Test criterion", "points": 100,
                                   "acceptance": "All clauses must pass.", "status": "open", "evidence": []}]}],
            "delivery_review": {
                "method_version": 1, "date": "2026-11-23", "source_commit": "b" * 40,
                "scope": "One originally open synthetic criterion.",
                "criteria": [{"id": "B01", "remaining": "Native operation remains unproved.",
                              "requirements": [{"description": "Bounded host core", "implementation": "partial",
                                                "verification": ["host"], "observation": "Host fixture only.",
                                                "remaining": "Native integration.", "evidence": ["evidence.txt"]}]}],
            },
        }

    def load(self):
        self.path.write_text(json.dumps(self.data))
        return progress.load_progress()

    def item(self):
        return self.data["delivery_review"]["criteria"][0]["requirements"][0]

    def run_main(self, mode):
        output = io.StringIO()
        with patch.object(sys, "argv", ["progress.py", mode]), \
                contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
            result = progress.main()
        return result, output.getvalue()

    def test_implemented_and_native_inventory_never_awards_points(self):
        self.item().update(implementation="implemented", verification=["host", "native"])
        data = self.load()
        self.assertEqual(progress.total_score(data), 0)
        implemented, evidence = progress.delivery_counts(data)
        self.assertEqual(implemented["implemented"], 1)
        self.assertEqual(evidence["native"], 1)
        self.assertEqual(evidence["host"], 1)
        self.assertEqual(evidence["hardware"], 0)
        self.assertIn("**0/100 weighted points**", progress.delivery_summary(data))

    def test_acceptance_requires_its_own_evidence(self):
        self.data["tracks"][0]["gates"][0]["status"] = "accepted"
        with self.assertRaisesRegex(ValueError, "no points without an evidence reference"):
            self.load()

    def test_missing_unknown_and_absent_evidence_stay_visible(self):
        for state in ("missing", "unknown"):
            with self.subTest(state=state):
                self.item().update(implementation=state, verification=[], evidence=[])
                data = self.load()
                implementation, verification = progress.delivery_counts(data)
                self.assertEqual(implementation[state], 1)
                self.assertEqual(sum(verification.values()), 0)
                self.assertIn("No execution evidence", progress.delivery_block(data))

    def test_claims_require_references_and_evidence_levels_are_explicit(self):
        original = copy.deepcopy(self.item())
        invalid = (
            {"verification": ["native"], "evidence": []},
            {"verification": [], "evidence": [], "implementation": "implemented"},
            {"verification": ["host", "host"]},
            {"verification": ["passed"]},
            {"implementation": "almost_done"},
        )
        for changes in invalid:
            with self.subTest(changes=changes):
                self.item().clear()
                self.item().update(original, **changes)
                with self.assertRaises(ValueError):
                    self.load()

    def test_review_must_cover_each_open_criterion_without_duplicates(self):
        criteria = self.data["delivery_review"]["criteria"]
        duplicate = copy.deepcopy(criteria[0])
        criteria.append(duplicate)
        with self.assertRaisesRegex(ValueError, "duplicate reviewed criterion"):
            self.load()
        duplicate["id"] = "B02"
        with self.assertRaisesRegex(ValueError, "Unknown or duplicate"):
            self.load()
        criteria.pop()
        self.data["tracks"][0]["gates"].append({"id": "B02", "title": "Other", "points": 50,
                                               "status": "open", "acceptance": "Needs evidence.", "evidence": []})
        self.data["tracks"][0]["gates"][0]["points"] = 50
        with self.assertRaisesRegex(ValueError, "open criterion is missing"):
            self.load()

    def test_duplicate_requirements_and_table_breaks_rejected(self):
        criterion = self.data["delivery_review"]["criteria"][0]
        criterion["requirements"].append(copy.deepcopy(self.item()))
        with self.assertRaisesRegex(ValueError, "duplicate requirement"):
            self.load()
        criterion["requirements"].pop()
        for value in ("unsafe|cell", "unsafe\nrow", "<img>"):
            self.item()["observation"] = value
            with self.assertRaisesRegex(ValueError, "invalid requirement observation"):
                self.load()

    def test_paths_reject_absolute_traversal_noncanonical_and_symlink_escape(self):
        for value in (str(self.root / "evidence.txt"), "docs/../evidence.txt", "./evidence.txt",
                      "docs//file.txt", "../outside.txt", "missing.txt"):
            with self.subTest(value=value):
                self.assertFalse(progress.evidence_path(value))
        with tempfile.TemporaryDirectory() as outside:
            target = Path(outside) / "outside.txt"
            target.write_text("Outside repository")
            (self.root / "escape.txt").symlink_to(target)
            self.assertFalse(progress.evidence_path("escape.txt"))
        self.assertTrue(progress.evidence_path("evidence.txt"))

    def test_remote_references_cannot_escape_the_explicit_repository(self):
        prefix = "https://github.com/BaryonOS/BaryonOS/"
        for suffix in ("actions/runs/123/job/456", "pull/480#issuecomment-123",
                       "blob/" + "a" * 40 + "/src/lib.rs"):
            self.assertTrue(progress.review_reference(prefix + suffix))
        for value in ("https://example.com/proof", prefix + "blob/main/src/lib.rs",
                      prefix + "blob/" + "a" * 40 + "/../../other/repo/file",
                      prefix + "blob/" + "a" * 40 + "//file", prefix + "pull/1)![x](url"):
            with self.subTest(value=value):
                self.assertFalse(progress.review_reference(value))

    def test_snapshot_keeps_its_own_date_and_immutable_source_links(self):
        data = self.load()
        rendered = progress.delivery_block(data)
        self.assertIn("2026-11-23", rendered)
        self.assertNotIn("2026-11-24", rendered)
        self.assertIn("/blob/" + "b" * 40 + "/evidence.txt", rendered)
        self.assertNotIn("(../evidence.txt)", rendered)
        self.assertNotIn("/blob/" + "a" * 40, rendered)

    def test_review_prose_cannot_inject_markdown_links_or_images(self):
        self.item()["observation"] = "![proof](https://example.com) [other](file) **native**"
        rendered = progress.delivery_block(self.load())
        self.assertNotIn("![proof](", rendered)
        self.assertNotIn("[other](", rendered)
        self.assertIn(r"\*\*native\*\*", rendered)

    def test_write_check_roundtrip_and_stale_view_detection(self):
        self.load()
        self.assertEqual(self.run_main("--check")[0], 1)
        self.assertEqual(self.run_main("--write")[0], 0)
        self.assertEqual(self.run_main("--check")[0], 0)
        self.item()["observation"] = "A newer scoped observation."
        self.load()
        status, message = self.run_main("--check")
        self.assertEqual(status, 1)
        self.assertIn("docs/PROGRESS.md", message)
        self.assertEqual(self.run_main("--write")[0], 0)
        self.assertEqual(self.run_main("--check")[0], 0)

    def test_zero_svg_means_no_acceptance_not_no_implementation(self):
        rendered = progress.svg(self.load())
        self.assertIn("No criteria accepted", rendered)
        self.assertNotIn("Not started", rendered)


if __name__ == "__main__":
    unittest.main()
