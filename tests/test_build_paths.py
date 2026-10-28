"""QEMU runners must reject unsupported output roots before any mutation."""
from contextlib import ExitStack
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run
import run_m6
import run_m6b
import run_m6c


class PrivateBuildRoots(unittest.TestCase):
    def reject_without_side_effects(self, root, build):
        for module, function in ((run, run.run_guest), (run_m6, run_m6.run_m6),
                                 (run_m6b, run_m6b.run_m6b), (run_m6c, run_m6c.run_m6c)):
            with self.subTest(runner=module.__name__), ExitStack() as stack:
                stack.enter_context(patch.object(module, "ROOT", root))
                stack.enter_context(patch.object(module, "BUILD", build))
                env = stack.enter_context(patch.object(module, "environment"))
                firmware = stack.enter_context(patch.object(module, "firmware"))
                with self.assertRaisesRegex(ValueError, "inside the checkout"):
                    function()
                env.assert_not_called()
                firmware.assert_not_called()
                self.assertFalse((build / "run").exists())

    def test_external_build_fails_before_creation_or_tool_invocation(self):
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repo"
            root.mkdir()
            self.reject_without_side_effects(root, parent / "outside")
            self.assertFalse((parent / "outside").exists())

    def test_similarly_named_sibling_is_not_inside_the_checkout(self):
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repo"
            root.mkdir()
            self.reject_without_side_effects(root, parent / "repo-other")

    def test_symlink_cannot_disguise_an_external_build_root(self):
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root, external = parent / "repo", parent / "outside"
            root.mkdir()
            external.mkdir()
            link = root / "build-link"
            try:
                link.symlink_to(external, target_is_directory=True)
            except OSError as error:
                self.skipTest(f"Directory symlinks unavailable: {error}")
            self.reject_without_side_effects(root, link)


if __name__ == "__main__":
    unittest.main()
