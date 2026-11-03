"""Check inventory handling of interpreter caches in a synthetic archive."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SCRIPT = (Path(__file__).resolve().parents[1] / "docs/verification/volume-format-admission/verify_archive.py")
spec = importlib.util.spec_from_file_location("volume_archive_verifier", SCRIPT)
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


class ArchiveCacheInventory(unittest.TestCase):
    def test_nested_caches_are_ignored_but_other_files_are_counted(self):
        required = ["native-import.json", "native/freeze.json", "native/matrix.json",
                    "native/regression/build/volume-gate.json",
                    "independent-native-review/matrix-review.json",
                    "independent-native-review/regression-review.json"]
        with tempfile.TemporaryDirectory() as tmp:
            proof = Path(tmp)
            entries = []
            for name in required:
                path = proof / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"fixture")
                entries.append({"path": name, "bytes": 7, "sha256": verifier.sha(b"fixture")})
            (proof / "ARTIFACTS.json").write_text(json.dumps({"files": entries}))
            for name in ("__pycache__/root.pyc", "native/runs/__pycache__/deep.pyc"):
                path = proof / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"cache")
            verifier.verify_manifest(proof)
            other = proof / "native/not__pycache__/ordinary.txt"
            other.parent.mkdir(parents=True)
            other.write_bytes(b"unlisted")
            with self.assertRaisesRegex(RuntimeError, "manifest inventory"):
                verifier.verify_manifest(proof)


if __name__ == "__main__":
    unittest.main()
