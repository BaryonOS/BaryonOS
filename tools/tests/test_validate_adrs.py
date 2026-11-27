"""ADR file validation preserves the unique historical decision namespace."""

import tempfile
import unittest
from pathlib import Path

from tools.validate_adrs import validate


class ValidateAdrs(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "docs/decisions").mkdir(parents=True)
        (self.root / "docs/DECISIONS.md").write_text("## ADR-OLD-20261127 — Old\n")
        self.path = self.root / "docs/decisions/ADR-NEW-20261127.md"
        self.valid = ("# ADR-NEW-20261127 — New\n\n> Ported note\n\n"
                      "**Status:** decided\n\n**Decision:**\nText\n\n**Rollback:** undo\n")

    def test_valid_with_note_before_status(self):
        self.path.write_text(self.valid)
        self.assertEqual(validate(self.root), [])

    def test_mismatch_and_missing_fields_fail(self):
        self.path.write_text(self.valid.replace("ADR-NEW", "ADR-OTHER", 1))
        self.assertTrue(validate(self.root))
        for field in ("Status", "Decision", "Rollback"):
            with self.subTest(field=field):
                self.path.write_text(self.valid.replace(f"**{field}:**", "omitted"))
                self.assertTrue(validate(self.root))

    def test_unrecognised_markdown_filename_fails(self):
        (self.root / "docs/decisions/notes.md").write_text(self.valid)
        self.assertTrue(validate(self.root))

    def test_nested_adr_fails_instead_of_bypassing_validation(self):
        nested = self.root / "docs/decisions/nested"
        nested.mkdir()
        (nested / self.path.name).write_text(self.valid)
        self.assertTrue(validate(self.root))

    def test_wrong_extension_and_invalid_date_fail(self):
        (self.root / "docs/decisions/ADR-NEW-20261127.MD").write_text(self.valid)
        self.assertTrue(validate(self.root))
        (self.root / "docs/decisions/ADR-NEW-20261127.MD").unlink()
        bad = self.root / "docs/decisions/ADR-NEW-20261340.md"
        bad.write_text(self.valid.replace("20261127", "20261340"))
        self.assertTrue(any("date" in issue for issue in validate(self.root)))

    def test_fields_inside_fenced_code_do_not_count(self):
        self.path.write_text("# ADR-NEW-20261127 — New\n```markdown\n"
                             "**Status:** decided\n**Decision:** text\n**Rollback:** undo\n```\n")
        self.assertEqual(sum("missing" in issue for issue in validate(self.root)), 3)

    def test_fields_inside_html_comments_do_not_count(self):
        self.path.write_text("# ADR-NEW-20261127 — New\n<!--\n"
                             "**Status:** decided\n**Decision:** text\n**Rollback:** undo\n-->\n")
        self.assertEqual(sum("missing" in issue for issue in validate(self.root)), 3)

    def test_unclosed_html_comment_does_not_supply_fields(self):
        self.path.write_text("# ADR-NEW-20261127 — New\n<!--\n"
                             "**Status:** decided\n**Decision:** text\n**Rollback:** undo\n")
        self.assertEqual(sum("missing" in issue for issue in validate(self.root)), 3)

    def test_comment_marker_inside_fence_does_not_hide_real_fields(self):
        self.path.write_text("# ADR-NEW-20261127 — New\n```\n<!--\n```\n"
                             "**Status:** decided\n**Decision:** text\n**Rollback:** undo\n")
        self.assertEqual(validate(self.root), [])

    def test_symlink_cannot_hide_an_adr(self):
        (self.root / "docs/decisions/linked").symlink_to(self.root / "docs", target_is_directory=True)
        self.assertTrue(any("symlink" in issue for issue in validate(self.root)))

    def test_historical_id_collision_fails(self):
        old = self.root / "docs/decisions/ADR-OLD-20261127.md"
        old.write_text(self.valid.replace("ADR-NEW", "ADR-OLD"))
        self.assertTrue(any("duplicate" in issue for issue in validate(self.root)))

    def test_missing_directory_is_permitted_before_migration(self):
        (self.root / "docs/decisions").rmdir()
        self.assertEqual(validate(self.root), [])


if __name__ == "__main__":
    unittest.main()
