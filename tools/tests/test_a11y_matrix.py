"""Successor matrix checker regressions; no UI or evidence-quality acceptance."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import check_a11y_matrix as checker


class MatrixContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.reference = checker.load_reference()
        cls.matrix = (checker.ROOT / checker.MATRIX).read_text(encoding="utf-8").splitlines()
        cls.rows, error = cls.reference.table(cls.matrix, cls.reference.MATRIX_HEADER)
        if error:
            raise AssertionError(error)
        cls.sources, error = cls.reference.table(cls.matrix, cls.reference.SOURCES_HEADER)
        if error:
            raise AssertionError(error)
        cls.baselines = [n for n, row in enumerate(cls.rows) if row[3] == "baseline"]

    def test_current_matrix_and_pinned_archive_pass_unchanged(self):
        before = checker.ARCHIVE.read_bytes()
        result = checker.check(checker.ROOT)
        self.assertTrue(result["ok"], result["errors"])
        self.assertEqual(result["tiers"]["baseline"], 51)
        self.assertEqual(hashlib.sha256(before).hexdigest(),
                         "cf1b65d170a385a0e4fde9724d326bbbe4f5d1cdbd7186073800f7b1bd251bab")
        self.assertEqual(checker.ARCHIVE.read_bytes(), before)

    def test_every_recorded_baseline_rejects_consistent_tier_and_axis_demotion(self):
        formerly_undetected = 0
        for index in self.baselines:
            with self.subTest(row=index + 1):
                rows = copy.deepcopy(self.rows)
                rows[index][3:5] = ["setting", "fixture-setting: enabled"]
                formerly_undetected += not self.reference.validate(rows, self.sources)[0]
                self.assertTrue(checker.baseline_errors(rows))
        self.assertGreater(formerly_undetected, 0, "the non-safety regression must be exercised")

    def test_every_recorded_baseline_rejects_removal(self):
        for index in self.baselines:
            with self.subTest(row=index + 1):
                rows = copy.deepcopy(self.rows)
                del rows[index]
                self.assertTrue(checker.baseline_errors(rows))

    def test_baseline_identity_is_not_a_row_number(self):
        self.assertEqual(checker.baseline_errors(list(reversed(self.rows))), [])
        rows = copy.deepcopy(self.rows)
        rows[self.baselines[0]][2] += " changed identity"
        self.assertTrue(checker.baseline_errors(rows))
        self.assertTrue(checker.baseline_errors(self.rows + [self.rows[self.baselines[0]]]))

    def test_reference_bytes_are_verified_before_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            altered = Path(temporary) / "checker.py"
            altered.write_bytes(b"raise AssertionError('unverified code executed')\n")
            with self.assertRaisesRegex(ValueError, "SHA256 mismatch"):
                checker.load_reference(altered)

    def test_baseline_manifest_cannot_silently_remove_a_requirement(self):
        with tempfile.TemporaryDirectory() as temporary:
            altered = Path(temporary) / "baseline-contract.json"
            contract = json.loads(checker.BASELINES.read_bytes())
            contract["baseline_rows"].pop()
            altered.write_text(json.dumps(contract), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "SHA256 mismatch"):
                checker.baseline_errors(self.rows, altered)


class NamedFiles(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.private = Path(temporary.name)
        self.root = self.private / "repository"
        (self.root / "docs/nested").mkdir(parents=True)
        (self.root / "ui").mkdir()
        (self.root / "docs/nested/safe.md").write_text("fixture", encoding="utf-8")
        (self.root / "ui/component.css").write_text("fixture", encoding="utf-8")
        self.outside = self.private / "outside"
        self.outside.mkdir()
        self.external_file = self.outside / "existing.md"
        self.external_file.write_text("not repository evidence", encoding="utf-8")

    def rows(self, token, column=8):
        row = [""] * 12
        row[8] = "absent"
        row[9] = "future-research: fixture component"
        row[column] = f"partial: `{token}` line 1"
        return [row]

    def test_nested_bare_and_internal_alias_references_are_allowed(self):
        (self.root / "docs/nested/alias.md").symlink_to("safe.md")
        for token in ("docs/nested/safe.md", "safe.md", "ui/component.css", "component.css",
                      "docs/nested/alias.md", "alias.md"):
            with self.subTest(token=token):
                self.assertEqual(checker.missing_files(self.rows(token), self.root), [])

    def test_absolute_and_parent_paths_to_existing_files_are_rejected(self):
        reference = checker.load_reference()
        for column in (8, 9):
            for token in (str(self.external_file), "../outside/existing.md"):
                with self.subTest(column=column, token=token):
                    rows = self.rows(token, column)
                    self.assertEqual(reference.missing_files(rows, self.root), [])
                    self.assertTrue(checker.missing_files(rows, self.root))

    def test_unsupported_path_characters_cannot_fall_back_to_an_inside_basename(self):
        (self.root / "ui/safe.md").write_text("inside basename", encoding="utf-8")
        for name in ("outside@safe.md", "outside%safe.md", "outside safe.md", "outside(safe.md"):
            outside = self.outside / name
            outside.write_text("outside evidence", encoding="utf-8")
            for token in (str(outside), "../outside/" + name):
                for quoted in (True, False):
                    with self.subTest(token=token, quoted=quoted):
                        rows = self.rows(token)
                        if not quoted:
                            rows[0][8] = f"partial: {token} line 1"
                        self.assertTrue(checker.missing_files(rows, self.root))

    def test_file_and_directory_symlink_escapes_are_rejected(self):
        (self.root / "docs/alias.md").symlink_to(self.external_file)
        (self.root / "docs/escape").symlink_to(self.outside, target_is_directory=True)
        for token in ("docs/alias.md", "alias.md", "docs/escape/existing.md", "existing.md"):
            with self.subTest(token=token):
                self.assertTrue(checker.missing_files(self.rows(token), self.root))

    def test_top_level_search_directory_cannot_escape_through_a_symlink(self):
        (self.root / "ui").rename(self.root / "saved-ui")
        (self.root / "ui").symlink_to(self.outside, target_is_directory=True)
        self.assertTrue(checker.missing_files(self.rows("existing.md"), self.root))

    def test_a_safe_basename_match_remains_valid_beside_an_escaping_alias(self):
        (self.root / "docs/alias.md").symlink_to(self.external_file)
        (self.root / "ui/alias.md").write_text("repository evidence", encoding="utf-8")
        self.assertEqual(checker.missing_files(self.rows("alias.md"), self.root), [])
        self.assertTrue(checker.missing_files(self.rows("docs/alias.md"), self.root))

    def test_windows_syntax_does_not_degrade_into_a_matching_basename(self):
        for token in (r"C:\elsewhere\safe.md", "C:/elsewhere/safe.md", r"..\outside\safe.md",
                      r"\\server\share\safe.md"):
            with self.subTest(token=token):
                self.assertTrue(checker.missing_files(self.rows(token), self.root))

    def test_missing_names_and_symlink_loops_fail_closed(self):
        (self.root / "docs/loop.md").symlink_to("loop.md")
        for token in ("unknown.md", "docs/loop.md", "loop.md"):
            with self.subTest(token=token):
                self.assertTrue(checker.missing_files(self.rows(token), self.root))

    def test_matrix_itself_cannot_be_an_outside_symlink(self):
        matrix = self.root / checker.MATRIX
        matrix.parent.mkdir(parents=True)
        matrix.symlink_to(self.external_file)
        with self.assertRaisesRegex(ValueError, "matrix is missing or outside"):
            checker.check(self.root)


if __name__ == "__main__":
    unittest.main()
