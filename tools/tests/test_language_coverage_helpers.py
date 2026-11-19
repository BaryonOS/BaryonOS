"""Regression controls for source-bound language and Wiki helpers."""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import unittest.mock


ROOT = Path(__file__).resolve().parents[2]
LANGUAGE = Path("docs/verification/issue274-language-coverage-20261118")
WIKI = Path("docs/verification/issue274-wiki-publication-20261118")
GIT_ENV = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
GIT_ENV.update(GIT_NO_LAZY_FETCH="1", GIT_TERMINAL_PROMPT="0")


class LanguageHelpers(unittest.TestCase):
    def fixture(self, outer):
        root = outer / "repository"
        (root / LANGUAGE).mkdir(parents=True)
        shutil.copyfile(ROOT / LANGUAGE / "check.py", root / LANGUAGE / "check.py")
        raw = b"# English fixture\n"
        for name in ("README.md", "docs/ARCHITECTURE.md"):
            target = root / name
            target.parent.mkdir(exist_ok=True)
            target.write_bytes(raw)
        digest = hashlib.sha256(raw).hexdigest()
        record = {
            "documents": [{"path": name, "sha256": digest, "candidate_lines": []} for name in ("README.md", "docs/ARCHITECTURE.md")],
            "community_forms": [], "compatibility_anchors": [],
            "translated_original": {"path": "README.md", "sha256": digest, "english_equivalent": "README.md"},
            "counts": {"normal_navigation_reachable": 1}, "historical_dutch_residuals": [],
        }
        (root / LANGUAGE / "coverage.json").write_text(json.dumps(record))
        subprocess.run(["git", "-C", str(root), "init", "-q"], check=True, capture_output=True, env=GIT_ENV)
        subprocess.run(["git", "-C", str(root), "add", "--all"], check=True, capture_output=True, env=GIT_ENV)
        decoy = outer / "decoy"
        decoy.mkdir()
        (decoy / "UNTOUCHED.md").write_text("# Foreign repository\n")
        subprocess.run(["git", "-C", str(decoy), "init", "-q"], check=True, capture_output=True, env=GIT_ENV)
        subprocess.run(["git", "-C", str(decoy), "add", "--all"], check=True, capture_output=True, env=GIT_ENV)
        poison = {"GIT_DIR": str(decoy / ".git"), "GIT_WORK_TREE": str(decoy), "GIT_INDEX_FILE": str(decoy / ".git/index"), "GIT_CONFIG_COUNT": "1", "GIT_CONFIG_KEY_0": "core.worktree", "GIT_CONFIG_VALUE_0": str(decoy)}
        return root, record, decoy, poison

    def test_checker_uses_its_repository_with_foreign_git_environment(self):
        with tempfile.TemporaryDirectory() as location:
            root, _, decoy, poison = self.fixture(Path(location))
            before = (decoy / ".git/index").read_bytes()
            result = subprocess.run([sys.executable, "-B", str(root / LANGUAGE / "check.py")], text=True, capture_output=True, env={**GIT_ENV, **poison})
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual((decoy / ".git/index").read_bytes(), before)

    def test_self_test_does_not_stage_the_foreign_repository(self):
        with tempfile.TemporaryDirectory() as location:
            root, record, decoy, poison = self.fixture(Path(location))
            before = (decoy / ".git/index").read_bytes()
            spec = importlib.util.spec_from_file_location("language_fixture", root / LANGUAGE / "check.py")
            module = importlib.util.module_from_spec(spec)
            with unittest.mock.patch.dict(os.environ, poison), contextlib.redirect_stdout(io.StringIO()):
                spec.loader.exec_module(module)
                with unittest.mock.patch.object(module, "path_controls", return_value={}):
                    module.self_test(root, record)
            self.assertEqual((decoy / ".git/index").read_bytes(), before)
            self.assertEqual(subprocess.check_output(["git", "-C", str(decoy), "diff", "--cached", "--name-only"], env=GIT_ENV).decode().splitlines(), ["UNTOUCHED.md"])

    def test_wiki_validation_remains_active_under_optimization(self):
        with tempfile.TemporaryDirectory() as location:
            outer = Path(location)
            root = outer / "repository"
            (root / WIKI).mkdir(parents=True)
            shutil.copyfile(ROOT / WIKI / "verify.py", root / WIKI / "verify.py")
            source = root / "docs/wiki/Home.md"
            source.parent.mkdir(parents=True)
            raw = b"# English fixture\n"
            source.write_bytes(raw)
            wiki = outer / "wiki"
            wiki.mkdir()
            (wiki / "Home.md").write_bytes(raw)
            (wiki / "Guide.md").write_bytes(raw)
            digest = hashlib.sha256(raw).hexdigest()
            record = {"source_pages": [{"path": "docs/wiki/Home.md", "sha256": digest}], "new_concept_pages": [], "existing_wiki_pages": [{"path": "Guide.md", "preserved_byte_identical": True, "sha256_before": digest}]}
            manifest = root / WIKI / "publication.json"
            manifest.write_text(json.dumps(record))
            command = [sys.executable, "-B", "-O", str(root / WIKI / "verify.py"), "--wiki", str(wiki)]
            result = subprocess.run(command, capture_output=True, text=True, env=GIT_ENV)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            for name, target in (("Wiki content", wiki / "Home.md"), ("source hash", source), ("preserved page", wiki / "Guide.md")):
                with self.subTest(check=name):
                    target.write_bytes(raw + b"Changed.\n")
                    result = subprocess.run(command, capture_output=True, text=True, env=GIT_ENV)
                    self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                    target.write_bytes(raw)
            outside = outer / "outside.md"
            outside.write_bytes(raw)
            (wiki / "outside.md").write_bytes(raw)
            (wiki / "Home.md").unlink()
            (wiki / "Home.md").symlink_to(outside)
            result = subprocess.run(command, capture_output=True, text=True, env=GIT_ENV)
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
            (wiki / "Home.md").unlink()
            (wiki / "Home.md").write_bytes(raw)
            record["source_pages"][0]["path"] = "../outside.md"
            manifest.write_text(json.dumps(record))
            result = subprocess.run(command, capture_output=True, text=True, env=GIT_ENV)
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
