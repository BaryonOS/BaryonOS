"""Validate LP evidence references, not privacy compliance or evidence authenticity."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = Path("docs/LOCAL-PRIVACY-CONTRACT.md")
# Default is the newest revision; earlier revisions stay readable through --matrix.
MATRIX = Path("docs/verification/b06-lp-matrix-20261123/matrix.json")
MAX_BYTES = 1024 * 1024


def bounded_read(path: Path) -> bytes:
    with path.open("rb") as stream:
        data = stream.read(MAX_BYTES + 1)
    if len(data) > MAX_BYTES:
        raise ValueError(f"{path.name} exceeds {MAX_BYTES} bytes")
    return data


def evidence_file(root: Path, reference: dict) -> tuple[Path, bytes]:
    if not isinstance(reference, dict) or set(reference) != {"path", "sha256"}:
        raise ValueError("evidence requires path and sha256")
    name, digest = reference["path"], reference["sha256"]
    if not isinstance(name, str) or not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
        raise ValueError("invalid evidence reference")
    relative = PurePosixPath(name)
    if relative.is_absolute() or relative.as_posix() != name or ".." in relative.parts or "\\" in name or ":" in name:
        raise ValueError("noncanonical evidence path")
    path = (root / name).resolve(strict=True)
    if not path.is_relative_to(root) or not path.is_file() or path.stat().st_size > MAX_BYTES:
        raise ValueError("evidence must be a bounded repository file")
    data = bounded_read(path)
    if hashlib.sha256(data).hexdigest() != digest:
        raise ValueError("evidence digest changed")
    return path, data


def check(root: Path = ROOT, matrix: Path = MATRIX) -> dict:
    root = root.resolve(strict=True)
    matrix_path = (root / matrix).resolve(strict=True)
    # Only archived matrix revisions: docs/verification/<revision>/matrix.json, bounded like evidence.
    if (not matrix_path.is_relative_to(root) or not matrix_path.is_file()
            or matrix_path.relative_to(root).parts[:2] != ("docs", "verification")
            or len(matrix_path.relative_to(root).parts) != 4 or matrix_path.name != "matrix.json"
            or matrix_path.stat().st_size > MAX_BYTES):
        raise ValueError("matrix must be a bounded docs/verification/<revision>/matrix.json file")
    ids = re.findall(r"^\| (LP-[0-9]{2}) \|", (root / CONTRACT).read_text(), re.MULTILINE)
    expected = {f"LP-{i:02}" for i in range(1, 32)}
    if len(ids) != 31 or set(ids) != expected:
        raise ValueError("contract must contain 31 unique LP IDs")
    data = json.loads(bounded_read(matrix_path))
    if not isinstance(data, dict) or set(data) != {"schema", "rows"} or type(data["schema"]) is not int or data["schema"] != 1:
        raise ValueError("unknown LP matrix schema")
    rows = data["rows"]
    if not isinstance(rows, list) or len(rows) != 31:
        raise ValueError("matrix must contain exactly 31 rows")
    seen, counts, risks = set(), dict(native=0, host=0, none=0), []
    for row in rows:
        if not isinstance(row, dict) or set(row) != {"id", "status", "evidence", "remaining"}:
            raise ValueError("invalid LP row")
        ident, status = row["id"], row["status"]
        if not isinstance(ident, str) or ident not in expected or ident in seen:
            raise ValueError("missing, duplicate or unknown LP ID")
        seen.add(ident)
        if not isinstance(status, str) or status not in counts or not isinstance(row["remaining"], str) or not row["remaining"].strip():
            raise ValueError("invalid status or missing residual coverage")
        reference = row["evidence"]
        if status == "none":
            if reference is not None:
                raise ValueError("unmapped row cannot claim evidence")
        else:
            path, evidence = evidence_file(root, reference)
            if status == "native":
                relative = path.relative_to(root)
                if relative.parts[:2] != ("docs", "verification"):
                    raise ValueError("native evidence must be an archived verification record")
                report = json.loads(evidence)
                if (not isinstance(report, dict) or report.get("passed") is not True
                        or report.get("timed_out") is not False
                        or type(report.get("raw_qemu_exit")) is not int or report["raw_qemu_exit"] != 33
                        or not isinstance(report.get("source_sha256"), str)
                        or not re.fullmatch(r"[0-9a-f]{64}", report["source_sha256"])):
                    raise ValueError("native reference is not a passing QEMU record with a source_sha256 tag")
        counts[status] += 1
        risks.append({"id": ident, "status": status, "remaining": row["remaining"]})
    return {"matrix": matrix_path.relative_to(root).as_posix(), "structurally_valid": True,
            "counts": counts, "residual_coverage": risks,
            "limit": ("Reference validation only; no evidence authenticity, complete-scenario or privacy acceptance. "
                      "A native row only means that its evidence is an archived QEMU record with passed, "
                      "timed_out, raw_qemu_exit 33 and source_sha256 fields as required; this tool does not "
                      "inspect what that run exercised. Whether it exercised the row's core denial is the "
                      "human-reviewed mapping in the matrix revision's README and the row's residual text, "
                      "and a native row does not mean that the complete LP scenario is covered.")}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", required=True)
    parser.add_argument("--matrix", type=Path, default=MATRIX, help="repository-relative matrix revision")
    args = parser.parse_args()
    try:
        print(json.dumps(check(matrix=args.matrix), indent=2))
    except (ValueError, OSError, RuntimeError, TypeError) as error:
        print(f"LP matrix {args.matrix.as_posix()}: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
