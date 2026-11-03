"""Root archive/source checks; does not execute a VM or mutate media."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "source"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def need(condition, message):
    if not condition:
        raise RuntimeError(message)


def verify_manifest(proof):
    manifest = json.loads((proof / "ARTIFACTS.json").read_text())
    entries = [item["path"] for item in manifest["files"]]
    actual = {p.relative_to(proof).as_posix() for p in proof.rglob("*")
              if p.is_file() and p != proof / "ARTIFACTS.json"}
    required = {"native-import.json", "native/freeze.json", "native/matrix.json", "native/regression/build/volume-gate.json",
                "independent-native-review/matrix-review.json", "independent-native-review/regression-review.json"}
    need(len(entries) == len(set(entries)) and set(entries) == actual and required <= actual,
         "manifest inventory missing, duplicated or incomplete")
    for item in manifest["files"]:
        raw = (proof / item["path"]).read_bytes()
        need(len(raw) == item["bytes"] and sha(raw) == item["sha256"], item["path"])
        if item["path"].endswith(".gz") or "uncompressed_sha256" in item or "uncompressed_bytes" in item:
            need("uncompressed_sha256" in item and "uncompressed_bytes" in item,
                 item["path"] + ": missing uncompressed metadata")
            decoded = gzip.decompress(raw)
            need(len(decoded) == item["uncompressed_bytes"], item["path"] + ": uncompressed length")
            need(sha(decoded) == item["uncompressed_sha256"], item["path"] + ": uncompressed hash")
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", nargs="?", choices=("manifest",), help="check only the archive manifest")
    parser.add_argument("--source", type=Path, default=SOURCE, help="source checkout in the external dossier")
    parser.add_argument("--proof", type=Path, help="archive proof directory")
    args = parser.parse_args()
    source = args.source.resolve()
    proof = (args.proof or (ROOT if args.mode == "manifest" else
                           source / "docs/verification/volume-format-admission")).resolve()
    manifest = verify_manifest(proof)
    manifest_count = len(manifest["files"])
    if args.mode == "manifest":
        print(f"Manifest and gzip hashes PASS: {manifest_count} artifacts")
        return
    sys.path.insert(0, str(source / "tools"))
    from common import source_hash

    frozen = json.loads((proof / "native/freeze.json").read_text())
    current_source_hash = source_hash()
    need(current_source_hash == frozen["source_sha256"], "source hash differs from freeze")
    changed = subprocess.check_output(["git", "diff", "--name-only", frozen["head"]], cwd=source, text=True).splitlines()
    need(all(p == "README.md" or p.startswith("docs/") for p in changed), f"runtime changes: {changed}")
    untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard"], cwd=source, text=True).splitlines()
    need(all(p.startswith("docs/verification/volume-format-admission/") for p in untracked), f"untracked files: {untracked}")
    imported = json.loads((proof / "native-import.json").read_text())
    for item in imported["files"]:
        stored = (proof / item["path"]).read_bytes()
        need(sha(stored) == item["stored_sha256"] and len(stored) == item["stored_bytes"], item["path"] + ": stored bytes")
        original = gzip.decompress(stored) if item["gzip_lossless"] else stored
        need(sha(original) == item["source_sha256"] and len(original) == item["source_bytes"], item["path"] + ": source bytes")
        need(original == Path(item["source_path"]).read_bytes(), item["path"] + ": differs from original")
    for name, field, artifact in (("matrix", "matrix_sha256", "native/matrix.json"),
                                  ("regression", "gate_sha256", "native/regression/build/volume-gate.json")):
        receipt = json.loads((proof / f"independent-native-review/{name}-review.json").read_text())
        need(receipt.get("passed") is True, f"{name}: review incomplete")
        need(receipt.get(field) == sha((proof / artifact).read_bytes()), f"{name}: missing or mismatched {field}")
    report = {"schema_version": 1, "passed": True,
              "scope": "Root source and lossless archive integrity; no new VM execution",
              "tested_commit": frozen["head"], "current_source_sha256": current_source_hash,
              "runtime_diff_from_tested_commit": [], "native_import_artifacts_checked": len(imported["files"]),
              "manifest_artifacts_checked": manifest_count,
              "all_imported_artifacts_byte_equal_to_original": True,
              "gzip_roundtrip_verified": True, "independent_ai_matrix_review_passed": True,
              "independent_ai_regression_review_passed": True,
              "verifier_sha256": sha(Path(__file__).read_bytes())}
    outputs = {"root-verification.json": (json.dumps(report, indent=2) + "\n").encode(),
               "verify_archive.py": Path(__file__).read_bytes()}
    # Only these generated outputs change in this flow. Existing entries were
    # validated above; absent outputs are added on the first verification.
    for name, data in outputs.items():
        entries = [entry for entry in manifest["files"] if entry["path"] == name]
        need(len(entries) <= 1 and all("uncompressed_sha256" not in entry for entry in entries), f"manifest entry invalid: {name}")
        if entries:
            entries[0].update(bytes=len(data), sha256=sha(data))
        else:
            manifest["files"].append({"path": name, "bytes": len(data), "sha256": sha(data)})
    for name, data in outputs.items():
        (proof / name).write_bytes(data)
    (proof / "ARTIFACTS.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    verify_manifest(proof)
    print(f"Source unchanged and lossless archive PASS: {len(imported['files'])} imported artifacts")


if __name__ == "__main__":
    main()
