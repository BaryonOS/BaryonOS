"""Document-session selection must fail before any user build side effect."""
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import build_users


class DocumentSessionUserSelection(unittest.TestCase):
    def rejected_before_build(self, kind, sdk_runtime, **extra):
        with tempfile.TemporaryDirectory(prefix="nieuw-os-g11-selection-") as directory:
            output = Path(directory) / "not-created"
            with patch.object(build_users, "BUILD", output), patch.object(build_users, "run_logged") as compile_command:
                with self.assertRaises(ValueError):
                    build_users.build_users(kind, sdk_runtime=sdk_runtime, document_session=True, **extra)
                compile_command.assert_not_called()
                self.assertFalse(output.exists())

    def test_document_session_cannot_bypass_required_sdk_bridge(self):
        self.rejected_before_build("m6c", False)

    def test_document_session_cannot_attach_to_any_other_user_kind(self):
        for kind in ("probe", "ipc", "large", "m5", "m6", "m6b", "unknown"):
            for sdk_runtime in (False, True):
                with self.subTest(kind=kind, sdk_runtime=sdk_runtime):
                    self.rejected_before_build(kind, sdk_runtime)

    def test_document_session_does_not_enable_short_poll_fixture(self):
        self.rejected_before_build("m6c", True, short_poll=True)


if __name__ == "__main__":
    unittest.main()
