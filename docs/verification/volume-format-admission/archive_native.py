"""Archive only completed native receipts/logs; never images or executables."""
from __future__ import annotations

import argparse
import gzip
import hashlib
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parent
DEST = ROOT / "source/docs/verification/volume-format-admission"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def archive() -> None:
    native = ROOT / "native"
    matrix = json.loads((native / "matrix.json").read_text())
    regression = json.loads((native / "regression/invocation.json").read_text())
    gate = json.loads((native / "regression/build/volume-gate.json").read_text())
    source = matrix.get("source")
    labels = {"own-normal", "own-cold-recover"} | {
        f"{candidate}-{mode}" for candidate in ("btrfs", "zero", "zero-prefix-payload-tail", "openzfs-leaf")
        for mode in ("readonly", "writable")}
    runs = matrix.get("runs", [])
    plan = [("normal", None), ("recover", "normal"), ("cut-data", None),
            ("recover-cut", "cut-data"), ("cut-table", None), ("recover-cut", "cut-table"),
            ("cut-txn", None), ("recover-cut", "cut-txn"), ("corrupt", None)]
    if not (isinstance(source, dict) and source.get("head") and source.get("source_sha256")
            and regression.get("source") == source and gate.get("source_sha256") == source["source_sha256"]
            and matrix.get("passed") is True and matrix.get("source_unchanged") is True
            and len(runs) == 10 and {run.get("label") for run in runs} == labels
            and all(run.get("passed") is True and run.get("timed_out") is False
                    and run.get("source_unchanged") is True and run.get("source") == source for run in runs)
            and regression.get("passed") is True
            and regression.get("returncode") == 0 and regression.get("source_unchanged") is True
            and regression.get("gate_passed") is True and regression.get("stage_count") == 9
            and gate.get("passed") is True and gate.get("status") == "completed"
            and [(stage.get("case"), stage.get("parent")) for stage in gate.get("stages", [])] == plan
            and all(stage.get("passed") is True for stage in gate["stages"])):
        raise RuntimeError("native acceptance incomplete; preserve current evidence")
    imported = []
    for source_root, destination in ((native, DEST / "native"),
                                     (ROOT / "independent-native-review", DEST / "independent-native-review")):
        if not source_root.is_dir():
            raise RuntimeError(f"missing independent/native proof directory: {source_root}")
        for path in sorted(source_root.rglob("*")):
            if not path.is_file():
                continue
            relative = path.relative_to(source_root)
            if any(part in {"target", "__pycache__", "process-local-appdata", "esp"} for part in relative.parts):
                continue
            if path.suffix.lower() not in {".json", ".txt", ".log", ".md", ".py"}:
                continue
            data = path.read_bytes()
            target = destination / relative
            compressed = path.name == "stderr-trace.txt"
            if compressed:
                target = target.with_suffix(target.suffix + ".gz")
                stored = gzip.compress(data, compresslevel=9, mtime=0)
                if gzip.decompress(stored) != data:
                    raise RuntimeError("lossless trace archive check failed")
            else:
                stored = data
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(stored)
            if target.read_bytes() != stored:
                raise RuntimeError(f"archive reread mismatch: {target}")
            imported.append({"path": target.relative_to(DEST).as_posix(),
                             "source_path": str(path), "source_bytes": len(data),
                             "source_sha256": digest(data), "stored_bytes": len(stored),
                             "stored_sha256": digest(stored), "gzip_lossless": compressed})
    shutil.copyfile(__file__, DEST / "archive_native.py")
    (DEST / "native-import.json").write_text(json.dumps({"schema_version": 1,
        "tested_source_commit": matrix["source"]["head"], "matrix_runs": 10,
        "regression_stages": 9, "files": imported}, indent=2) + "\n", encoding="utf-8")
    print(f"Archived {len(imported)} native/review text artifacts; raw traces losslessly gzip-compressed.")


def manifest() -> None:
    files = []
    for path in sorted(DEST.rglob("*")):
        if not path.is_file() or path.name == "ARTIFACTS.json":
            continue
        data = path.read_bytes()
        entry = {"path": path.relative_to(DEST).as_posix(), "bytes": len(data), "sha256": digest(data)}
        if path.name.endswith("stderr-trace.txt.gz"):
            original = gzip.decompress(data)
            entry.update(uncompressed_bytes=len(original), uncompressed_sha256=digest(original))
        files.append(entry)
    (DEST / "ARTIFACTS.json").write_text(json.dumps({"schema_version": 1, "files": files}, indent=2) + "\n", encoding="utf-8")
    print(f"Hashed {len(files)} archived artifacts, {sum(f['bytes'] for f in files)} stored bytes.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("archive", "manifest"))
    args = parser.parse_args()
    {"archive": archive, "manifest": manifest}[args.action]()
