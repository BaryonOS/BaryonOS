"""Bounded preservation checks; no historical artifact code or VM is executed."""
import ast
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
INITIAL = "024cede12dbe16697ef6a5221d95297be0f6c56d"
AUTHORIZED = "13782f92f74725920a125a2b7a5e0ec8bc8231be"
BASELINE = "5343b0a681ebc4cb9b6461bbe8d4447722eec91c"
CENTRAL = (
    "README.md",
    "docs/progress.json",
    "docs/PROGRESS.md",
    "docs/PROJECT-STATE.md",
    "docs/assets/progress.svg",
)


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


class AuthorizedDelta(unittest.TestCase):
    def test_only_documentation_and_evidence_changed_and_all_bytes_survive(self):
        names = git("diff", "--name-only", INITIAL, AUTHORIZED).decode().splitlines()
        self.assertEqual(len(names), 18)
        for name in names:
            with self.subTest(path=name):
                self.assertTrue(name in CENTRAL or name.startswith("docs/verification/"))
                expected = BASELINE if name in CENTRAL else AUTHORIZED
                self.assertEqual((ROOT / name).read_bytes(), git("show", f"{expected}:{name}"))

    def test_all_archived_evidence_hashes_still_match(self):
        folder = ROOT / "docs/verification/materiaal-library-20261102"
        manifest = json.loads((folder / "artifact-hashes.json").read_text())
        self.assertEqual(manifest["source_code_commit"], "64044318c622eaacab25272611acf856a5fca5d5")
        for name, expected in manifest["files"].items():
            with self.subTest(path=name):
                path = (folder / name).resolve(strict=True)
                self.assertTrue(path.is_relative_to(folder))
                self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), expected)

    def test_historical_claims_remain_bound_to_original_source(self):
        folder = ROOT / "docs/verification/materiaal-library-20261102"
        report = json.loads((folder / "ci-readback.json").read_text())
        self.assertEqual(report["github_head"], "64044318c622eaacab25272611acf856a5fca5d5")
        self.assertEqual(report["ci_merge_source"]["source_sha256"], "069666e5406a893f64aa5d8e00298863f33f8be9cef57b6e0522b8bdf9648670")
        self.assertEqual(set(report["runs"]), {"regression", "write", "cold"})
        self.assertEqual([report["runs"][name]["phases"] for name in ("regression", "write", "cold")], [22, 10, 4])
        self.assertEqual(report["runs"]["write"]["volume_sha256"], report["runs"]["cold"]["volume_sha256"])
        self.assertEqual(len(report["embedded_users"]), 8)
        self.assertEqual(report["normal_package_payloads"], 19)
        ast.parse((folder / "verify-ci-artifact.py").read_text())

    def test_new_historical_central_overlay_is_reversible(self):
        report = json.loads((ROOT / "docs/verification/stack130-historical-central.json").read_text())
        self.assertTrue(report["historical_only"])
        for name, entry in report["overlays"][AUTHORIZED].items():
            with self.subTest(path=name):
                self.assertEqual(hashlib.sha256(git("show", f"{AUTHORIZED}:{name}")).hexdigest(), entry["historical_sha256"])
                subprocess.run(["git", "apply", "--check", "--whitespace=nowarn", "-"], cwd=ROOT, input=entry["unified_diff"], text=True, check=True, capture_output=True)

    def test_product_source_fingerprint_did_not_change(self):
        sys.path.insert(0, str(ROOT / "tools"))
        from common import source_hash

        self.assertEqual(source_hash(), "7ad8b7df9afc12fe0ba19690d4a103792564e813c69ac8714ff128f8efe71940")


if __name__ == "__main__":
    unittest.main()
