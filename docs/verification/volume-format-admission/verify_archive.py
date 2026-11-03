"""Root archive/source checks; does not execute a VM or mutate media."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "source"
PROOF = SOURCE / "docs/verification/volume-format-admission"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def need(condition, message):
    if not condition:
        raise ValueError(message)


def verify_manifest(proof):
    manifest = json.loads((proof / "ARTIFACTS.json").read_text())
    items = manifest["files"]
    need(isinstance(items, list) and items, "empty or invalid manifest")
    paths = [item["path"] for item in items]
    need(all(isinstance(path, str) and path and not Path(path).is_absolute()
             and ".." not in Path(path).parts for path in paths), "invalid manifest path")
    need(len(set(paths)) == len(paths), "duplicate manifest path")
    labels = {"own-normal", "own-cold-recover"} | {
        f"{candidate}-{mode}" for candidate in ("btrfs", "zero", "zero-prefix-payload-tail", "openzfs-leaf")
        for mode in ("readonly", "writable")}
    required = {"native/freeze.json", "native/matrix.json", "native/regression/invocation.json",
                "native/regression/build/volume-gate.json", "native-import.json",
                "independent-native-review/matrix-review.json", "independent-native-review/regression-review.json"}
    required |= {f"native/runs/{label}/{name}" for label in labels
                 for name in ("result.json", "serial.txt", "stderr-trace.txt.gz", "build-manifest.json")}
    need(required <= set(paths), "manifest missing required evidence")
    actual = {path.relative_to(proof).as_posix() for path in proof.rglob("*")
              if path.is_file() and path != proof / "ARTIFACTS.json"}
    need(set(paths) == actual, "manifest does not cover archive files")
    for item in items:
        need((proof / item["path"]).resolve().is_relative_to(proof), "manifest path outside archive")
        raw = (proof / item["path"]).read_bytes()
        need(len(raw) == item["bytes"] and sha(raw) == item["sha256"], item["path"])
        if "uncompressed_sha256" in item:
            decoded = gzip.decompress(raw)
            need(len(decoded) == item["uncompressed_bytes"], item["path"] + ": uncompressed size")
            need(sha(decoded) == item["uncompressed_sha256"], item["path"] + ": uncompressed hash")
    return len(manifest["files"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", nargs="?", choices=("manifest",), default=None)
    parser.add_argument("--source", type=Path, default=SOURCE)
    parser.add_argument("--proof", type=Path)
    args = parser.parse_args()
    source = args.source.resolve()
    proof = (args.proof or (ROOT if args.action == "manifest" else
                             source / "docs/verification/volume-format-admission")).resolve()
    count = verify_manifest(proof)
    if args.action == "manifest":
        print(f"Manifest and gzip hashes PASS: {count} artifacts")
        return
    frozen = json.loads((proof / "native/freeze.json").read_text())
    head = frozen["head"]
    need(isinstance(head, str) and re.fullmatch(r"[0-9a-f]{40}", head), "invalid source commit")
    # Execute only the exact common.py from the historical tested commit, and
    # compile the checked bytes directly without adding source/tools to sys.path.
    common_path = source / "tools/common.py"
    common_bytes = common_path.read_bytes()
    need(sha(common_bytes) == "ac5cf2f1f576e68571212c85c70d4081b41761c913ecc46562edddccf74a6959",
         "historical source hash implementation differs")
    common = {"__file__": str(common_path), "__name__": "historical_common"}
    exec(compile(common_bytes, str(common_path), "exec"), common)
    current_hash = common["source_hash"]()
    need(current_hash == frozen["source_sha256"], "source fingerprint differs")
    changed = subprocess.check_output(["git", "diff", "--name-only", head], cwd=source, text=True).splitlines()
    need(all(p == "README.md" or p.startswith("docs/") for p in changed), f"source changes: {changed}")
    untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard"], cwd=source, text=True).splitlines()
    need(all(p.startswith("docs/verification/volume-format-admission/") for p in untracked), f"untracked files: {untracked}")
    imported = json.loads((proof / "native-import.json").read_text())
    for item in imported["files"]:
        stored = (proof / item["path"]).read_bytes()
        need(sha(stored) == item["stored_sha256"] and len(stored) == item["stored_bytes"], item["path"] + ": stored bytes")
        original = gzip.decompress(stored) if item["gzip_lossless"] else stored
        need(sha(original) == item["source_sha256"] and len(original) == item["source_bytes"], item["path"] + ": original bytes")
        need(original == Path(item["source_path"]).read_bytes(), item["path"] + ": source differs")
    for name, field, artifact in (("matrix", "matrix_sha256", "native/matrix.json"),
                                  ("regression", "gate_sha256", "native/regression/build/volume-gate.json")):
        receipt = json.loads((proof / f"independent-native-review/{name}-review.json").read_text())
        need(receipt.get("passed") is True, f"{name} review missing or failed")
        need(receipt.get(field) == sha((proof / artifact).read_bytes()), f"{name} review artifact hash differs")
    verifier_bytes = Path(__file__).read_bytes()
    verifier_path = proof / "verify_archive.py"
    need(not verifier_path.exists() or verifier_path.read_bytes() == verifier_bytes,
         "proof verifier differs from running verifier")
    report = {"schema_version": 1, "passed": True,
              "scope": "Root source and lossless archive integrity; no new VM execution",
              "tested_commit": frozen["head"], "current_source_sha256": current_hash,
              "runtime_diff_from_tested_commit": [], "native_import_artifacts_checked": len(imported["files"]),
              "all_imported_artifacts_byte_equal_to_original": True,
              "gzip_roundtrip_verified": True, "independent_ai_matrix_review_passed": True,
              "independent_ai_regression_review_passed": True,
              "verifier_sha256": sha(verifier_bytes)}
    (proof / "root-verification.json").write_bytes((json.dumps(report, indent=2) + "\n").encode())
    verifier_path.write_bytes(verifier_bytes)
    # The report and verifier are generated outputs, so keep their manifest
    # entries synchronized after validating the pre-existing archive.
    manifest_path = proof / "ARTIFACTS.json"
    manifest = json.loads(manifest_path.read_text())
    for name in ("root-verification.json", "verify_archive.py"):
        data = (proof / name).read_bytes()
        entry = next((item for item in manifest["files"] if item["path"] == name), None)
        if entry is None:
            entry = {"path": name}
            manifest["files"].append(entry)
        entry.update(bytes=len(data), sha256=sha(data))
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"Source unchanged and lossless archive PASS: {len(imported['files'])} imported artifacts")


if __name__ == "__main__":
    main()
