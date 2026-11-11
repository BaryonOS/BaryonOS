"""Repository contracts for the relocated Dutch README snapshot and provenance."""

import hashlib
from pathlib import Path
import re
import unittest
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[2]
ARCHIVE = ROOT / "docs/history/README-2026-11-09.nl.md"
CURRENT_README = ROOT / "README.md"
TRANSLATION_RECORD = (
    ROOT / "docs/verification/english-translation-20261109/README.md"
)
SOURCE_COMMIT = "8331b69b9b06294ad8a8c5c571580271655ba8d1"
SOURCE_BLOB = "e2d1a486c4d396c2c164f2935d89224bb9d83c07"
PROJECT_STATE_URL = (
    "https://github.com/simonvanieperen/BaryonOS/blob/"
    f"{SOURCE_COMMIT}/docs/PROJECT-STATE.md"
)
PROGRESS_IMAGE_URL = (
    "https://raw.githubusercontent.com/simonvanieperen/BaryonOS/"
    f"{SOURCE_COMMIT}/docs/assets/progress.svg"
)
MARKDOWN_LINK = re.compile(
    r"!?\[[^\]]*\]\((?P<target>[^\s)]+)(?:\s+[^)]*)?\)"
)


def markdown_targets(text):
    """Return inline Markdown link and image targets in document order."""
    return [match.group("target") for match in MARKDOWN_LINK.finditer(text)]


def source_blob_id(relocated_text):
    """Restore sanctioned target rewrites and calculate the original Git blob ID."""
    restored = relocated_text.replace(PROJECT_STATE_URL, "docs/PROJECT-STATE.md")
    restored = restored.replace(PROGRESS_IMAGE_URL, "docs/assets/progress.svg")

    def remove_relocation_prefix(match):
        """Undo one repository-link prefix while retaining the full link token."""
        target = match.group("target")
        if target.startswith("../../"):
            target = target.removeprefix("../../")
        return match.group(0).replace(match.group("target"), target, 1)

    restored = MARKDOWN_LINK.sub(remove_relocation_prefix, restored)
    payload = restored.encode("utf-8")
    header = f"blob {len(payload)}\0".encode("ascii")
    return hashlib.sha1(header + payload, usedforsecurity=False).hexdigest()


class HistoricalReadmeArchiveContractTests(unittest.TestCase):
    """Keep the dated donor immutable while allowing audited link relocation."""

    @classmethod
    def setUpClass(cls):
        """Load both checked-in documents as strict UTF-8."""
        cls.archive = ARCHIVE.read_text(encoding="utf-8", errors="strict")
        cls.current_readme = CURRENT_README.read_text(
            encoding="utf-8", errors="strict"
        )
        cls.translation_record = TRANSLATION_RECORD.read_text(
            encoding="utf-8", errors="strict"
        )
        cls.targets = markdown_targets(cls.archive)

    def test_archive_content_matches_pr270_donor_except_link_targets(self):
        """Reject edits beyond the explicitly sanctioned relocation rewrites."""
        self.assertEqual(source_blob_id(self.archive), SOURCE_BLOB)

    def test_all_relocated_relative_targets_exist(self):
        """Resolve all 52 repository links from the archive's nested location."""
        relative_targets = [
            target for target in self.targets if not urlsplit(target).scheme
        ]
        self.assertEqual(len(relative_targets), 52)
        self.assertTrue(
            all(target.startswith("../../") for target in relative_targets)
        )

        for target in relative_targets:
            path = target.split("#", 1)[0].split("?", 1)[0]
            resolved = (ARCHIVE.parent / path).resolve()
            with self.subTest(target=target):
                self.assertTrue(resolved.is_relative_to(ROOT))
                self.assertTrue(resolved.exists(), resolved)

    def test_status_sensitive_links_are_immutable(self):
        """Bind historical status claims and the progress image to their source."""
        self.assertEqual(self.targets.count(PROJECT_STATE_URL), 7)
        self.assertEqual(self.targets.count(PROGRESS_IMAGE_URL), 1)

        historical_targets = [
            target
            for target in self.targets
            if target in {PROJECT_STATE_URL, PROGRESS_IMAGE_URL}
        ]
        self.assertEqual(len(historical_targets), 8)
        self.assertTrue(all(SOURCE_COMMIT in target for target in historical_targets))

    def test_current_readme_labels_the_snapshot_as_historical(self):
        """Keep the Dutch snapshot discoverable without presenting it as current."""
        self.assertIn(
            "[9 November 2026 snapshot]"
            "(docs/history/README-2026-11-09.nl.md)",
            self.current_readme,
        )
        self.assertIn(
            "it is not the current implementation specification",
            self.current_readme,
        )

    def test_translation_record_scopes_claims_and_links(self):
        """Keep historical scope explicit and require local evidence to resolve."""
        self.assertNotIn(
            "[github-before.json](github-before.json)",
            self.translation_record,
        )
        self.assertIn(
            "that file is absent from the PR #269 donor tree",
            self.translation_record,
        )
        self.assertIn(
            "within the recorded PR #269 translation scope",
            self.translation_record,
        )
        self.assertIn(
            "does not claim that every active document in the current "
            "repository is English",
            self.translation_record,
        )

        local_targets = [
            target
            for target in markdown_targets(self.translation_record)
            if not urlsplit(target).scheme and not target.startswith("#")
        ]
        for target in local_targets:
            path = target.split("#", 1)[0].split("?", 1)[0]
            resolved = (TRANSLATION_RECORD.parent / path).resolve()
            with self.subTest(target=target):
                self.assertTrue(resolved.is_relative_to(ROOT))
                self.assertTrue(resolved.exists(), resolved)


if __name__ == "__main__":
    unittest.main()
