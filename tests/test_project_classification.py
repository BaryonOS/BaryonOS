"""Unit tests for conservative GitHub Projects classification (no network)."""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from project_classification import classify, proposed_updates  # noqa: E402


class ClassificationTests(unittest.TestCase):
    def test_trusted_prefixes(self):
        self.assertEqual(
            classify("feat(veld): integrate workspace", []),
            {"Work Type": "Feature", "Subsystem": "UI"},
        )
        self.assertEqual(
            classify("docs: update README", []),
            {"Work Type": "Documentation", "Subsystem": "Documentation"},
        )
        self.assertEqual(
            classify("fix(memory): reject invalid mapping", []),
            {"Work Type": "Bug", "Subsystem": "Memory"},
        )

    def test_only_explicit_priority_and_no_effort_guess(self):
        self.assertEqual(
            classify("fix(kernel): critical issue", ["priority:p0"]),
            {"Work Type": "Bug", "Subsystem": "Kernel", "Priority": "P0 Critical"},
        )
        self.assertNotIn("Priority", classify("fix(kernel): critical issue", []))
        self.assertNotIn("Effort", classify("feat(kernel): complex rewrite", []))

    def test_conflicting_signals_are_not_guessed(self):
        self.assertNotIn(
            "Subsystem",
            classify("feat(kernel): work", ["subsystem:storage"]),
        )
        self.assertNotIn(
            "Work Type",
            classify("feat(ui): work", ["type:bug"]),
        )

    def test_do_not_overwrite_existing_values(self):
        item = {
            "content": {"title": "feat(veld): create shell", "number": 42,
                        "labels": {"nodes": []}},
            "fieldValues": {"nodes": [
                {"name": "Bug", "field": {"name": "Work Type"}}
            ]},
        }
        fields = {
            "Work Type": {"id": "f1", "options": [{"id": "o1", "name": "Feature"}]},
            "Subsystem": {"id": "f2", "options": [{"id": "o2", "name": "UI"}]},
        }
        self.assertEqual(
            proposed_updates(item, fields),
            [("Subsystem", "UI", "f2", "o2")],
        )

    def test_missing_field_or_option_remains_unmodified(self):
        item = {
            "content": {"title": "feat(kernel): work", "number": 7,
                        "labels": {"nodes": []}},
            "fieldValues": {"nodes": []},
        }
        self.assertEqual(proposed_updates(item, {}), [])


if __name__ == "__main__":
    unittest.main()
