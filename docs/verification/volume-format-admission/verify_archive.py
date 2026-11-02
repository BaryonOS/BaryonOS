"""Root archive/source checks; does not execute a VM or mutate media."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "source"
PROOF = SOURCE / "docs/verification/volume-format-admission"
sys.path.insert(0, str(SOURCE / "tools"))
from common import source_hash


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    manifest_path = PROOF / "ARTIFACTS.json"
    if sys.argv[1:] == ["manifest"]:
        manifest = json.loads(manifest_path.read_text())
        for item in manifest["files"]:
            raw = (PROOF / item["path"]).read_bytes()
            assert len(raw) == item["bytes"] and sha(raw) == item["sha256"], item["path"]
            if "uncompressed_sha256" in item:
                decoded = gzip.decompress(raw)
                assert len(decoded) == item["uncompressed_bytes"]
                assert sha(decoded) == item["uncompressed_sha256"]
        print(f"Manifest and gzip hashes PASS: {len(manifest['files'])} artifacts")
        return
    frozen = json.loads((PROOF / "native/freeze.json").read_text())
    assert source_hash() == frozen["source_sha256"]
    changed = subprocess.check_output(["git", "diff", "--name-only", frozen["head"]], cwd=SOURCE, text=True).splitlines()
    assert all(p == "README.md" or p.startswith("docs/") for p in changed), changed
    untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard"], cwd=SOURCE, text=True).splitlines()
    assert all(p.startswith("docs/verification/volume-format-admission/") for p in untracked), untracked
    imported = json.loads((PROOF / "native-import.json").read_text())
    for item in imported["files"]:
        stored = (PROOF / item["path"]).read_bytes()
        assert sha(stored) == item["stored_sha256"] and len(stored) == item["stored_bytes"]
        original = gzip.decompress(stored) if item["gzip_lossless"] else stored
        assert sha(original) == item["source_sha256"] and len(original) == item["source_bytes"]
        assert original == Path(item["source_path"]).read_bytes(), item["path"]
    for name in ("matrix", "regression"):
        assert json.loads((PROOF / f"independent-native-review/{name}-review.json").read_text())["passed"]
    report = {"schema_version": 1, "passed": True,
              "scope": "Root source and lossless archive integrity; no new VM execution",
              "tested_commit": frozen["head"], "current_source_sha256": source_hash(),
              "runtime_diff_from_tested_commit": [], "native_import_artifacts_checked": len(imported["files"]),
              "all_imported_artifacts_byte_equal_to_original": True,
              "gzip_roundtrip_verified": True, "independent_ai_matrix_review_passed": True,
              "independent_ai_regression_review_passed": True,
              "verifier_sha256": sha(Path(__file__).read_bytes())}
    (PROOF / "root-verification.json").write_bytes((json.dumps(report, indent=2) + "\n").encode())
    (PROOF / "verify_archive.py").write_bytes(Path(__file__).read_bytes())
    print(f"Source unchanged and lossless archive PASS: {len(imported['files'])} imported artifacts")


if __name__ == "__main__":
    main()
