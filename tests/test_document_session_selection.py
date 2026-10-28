"""Explicit document-fixture selection must fail before any build side effect."""
import os
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import build
import build_users


class CompilationBoundary(Exception):
    pass


class DocumentSessionSelection(unittest.TestCase):
    def rejected(self, values, feature, message):
        with patch.dict(os.environ, values, clear=True), \
                patch.object(build_users, "build_users") as users, \
                patch.object(build, "run_logged") as commands:
            with self.assertRaisesRegex(ValueError, message):
                build.build(kernel_feature=feature)
            users.assert_not_called()
            commands.assert_not_called()

    def test_noncanonical_selection_rejected_before_compilation(self):
        for value in ("", "true", "2", "01", " 1", "1 "):
            with self.subTest(value=value):
                self.rejected({"NOS_DOCUMENT_SESSION": value, "NOS_SDK_RUNTIME": "1"},
                              "milestone-m6c", "NOS_DOCUMENT_SESSION must be exactly")

    def test_document_mode_requires_real_sdk_selection(self):
        for values in ({"NOS_DOCUMENT_SESSION": "1"},
                       {"NOS_DOCUMENT_SESSION": "1", "NOS_SDK_RUNTIME": "0"}):
            for feature in ("milestone-m6c", "test-m6c-recover"):
                with self.subTest(values=values, feature=feature):
                    self.rejected(values, feature, "Document session requires")

    def test_other_native_profiles_cannot_claim_document_session(self):
        for feature in (None, "native-qsim", "graphics-g0", "milestone-m6b",
                        "test-m6c-errors", "test-m6c-full", "test-m6c-write-timeout"):
            with self.subTest(feature=feature):
                self.rejected({"NOS_DOCUMENT_SESSION": "1", "NOS_SDK_RUNTIME": "1"},
                              feature, "Document session requires")

    def test_only_explicit_normal_and_recover_select_document_user_elf(self):
        for document in ("0", "1"):
            for feature in ("milestone-m6c", "test-m6c-recover"):
                with self.subTest(document=document, feature=feature), \
                        patch.dict(os.environ, {"NOS_DOCUMENT_SESSION": document,
                                                "NOS_SDK_RUNTIME": "1"}, clear=True), \
                        patch.object(build_users, "build_users", return_value=Path("document-fixture.elf")) as users, \
                        patch.object(build, "run_logged", side_effect=CompilationBoundary):
                    with self.assertRaises(CompilationBoundary):
                        build.build(kernel_feature=feature)
                    users.assert_any_call("m6c", sdk_runtime=True, document_session=document == "1")


if __name__ == "__main__":
    unittest.main()
