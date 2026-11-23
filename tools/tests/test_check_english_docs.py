# SPDX-License-Identifier: MPL-2.0
"""Regression gate for issue #274: active Markdown stays English."""
from pathlib import Path
import os
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import check_english_docs as checker

DUTCH_TEXT = (
    "Dit is een document dat niet in het Engels is geschreven. De tekst moet "
    "worden vertaald voor de publicatie, want het is van belang dat iedereen "
    "deze uitleg kan lezen. Een lezer zonder kennis van de taal kan dit niet "
    "volgen en ook de tools werken alleen met Engelse tekst. Dat is een "
    "probleem voor het project en voor de bijdragers die op de documentatie "
    "vertrouwen bij het werk aan de kern en aan de diensten. "
) * 2
ENGLISH_TEXT = (
    "This document is written in English. The text must be readable by every "
    "contributor and also by reviewers who are not familiar with the project. "
) * 4


class Classifier(unittest.TestCase):
    def test_dutch_prose_is_flagged(self):
        self.assertTrue(checker.is_dutch(DUTCH_TEXT))

    def test_english_prose_passes(self):
        self.assertFalse(checker.is_dutch(ENGLISH_TEXT))

    def test_short_dutch_fragment_passes(self):
        # Literal parser keys such as `Afhankelijk van:` must not trip the gate.
        self.assertFalse(checker.is_dutch(ENGLISH_TEXT + "Afhankelijk van: #1\nCriterium: X\n"))

    def test_code_fences_are_ignored(self):
        self.assertFalse(checker.is_dutch(ENGLISH_TEXT + "\n```\n" + DUTCH_TEXT + "\n```\n"))

    def test_tilde_fences_are_ignored(self):
        self.assertFalse(checker.is_dutch(ENGLISH_TEXT + "\n~~~text\n" + DUTCH_TEXT + "\n~~~\n"))

    def test_longer_fence_does_not_hide_prose(self):
        # A ``` line inside a ```` block neither closes it nor hides the prose after it.
        text = "````md\n```\n````\n\n" + DUTCH_TEXT + "\n```text\nx\n```\n"
        self.assertTrue(checker.is_dutch(text))

    def test_mismatched_closing_fence_keeps_block_open(self):
        text = ENGLISH_TEXT + "\n~~~\n" + DUTCH_TEXT + "\n```\nmore\n~~~\n"
        self.assertFalse(checker.is_dutch(text))

    def test_evidence_and_third_party_paths_are_excluded(self):
        for path in (
            "docs/verification/x/README.md",
            "drivers/usb/verification/hid-boot-reports.md",
            "docs/history/old.md",
            ".agents/skills/os-dev/references/a.md",
        ):
            self.assertTrue(checker.is_excluded(path), path)
        for path in ("docs/ROADMAP.md", "drivers/usb/README.md", "verification.md"):
            self.assertFalse(checker.is_excluded(path), path)


class Scanner(unittest.TestCase):
    def test_scanner_reports_only_active_dutch_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", str(root)], check=True, env=checker.git_env())
            files = {
                "docs/ACTIVE.md": DUTCH_TEXT,
                "docs/ENGLISH.md": ENGLISH_TEXT,
                "docs/verification/old/README.md": DUTCH_TEXT,
                "drivers/x/verification/note.md": DUTCH_TEXT,
                "docs/untracked.md": DUTCH_TEXT,
            }
            for rel, text in files.items():
                (root / rel).parent.mkdir(parents=True, exist_ok=True)
                (root / rel).write_text(text, encoding="utf-8")
            tracked = [rel for rel in files if rel != "docs/untracked.md"]
            subprocess.run(["git", "-C", str(root), "add", "--", *tracked], check=True, env=checker.git_env())
            self.assertEqual(checker.find_dutch(root), ["docs/ACTIVE.md"])
            # An inherited GIT_DIR naming another repository must not redirect the scan.
            with mock.patch.dict(os.environ, {"GIT_DIR": str(checker.ROOT / ".git")}):
                self.assertEqual(checker.find_dutch(root), ["docs/ACTIVE.md"])


class Repository(unittest.TestCase):
    def test_no_active_markdown_is_dutch(self):
        self.assertEqual(checker.find_dutch(), [])


if __name__ == "__main__":
    unittest.main()
