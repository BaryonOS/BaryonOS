"""Validate LP evidence references, not privacy compliance or evidence authenticity."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = Path("docs/LOCAL-PRIVACY-CONTRACT.md")
MATRIX = Path("docs/verification/b06-lp-matrix-20261115/matrix.json")


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
    if not path.is_relative_to(root) or not path.is_file() or path.stat().st_size > 1024 * 1024:
        raise ValueError("evidence must be a bounded repository file")
    with path.open("rb") as stream:
        data = stream.read(1024 * 1024 + 1)
    if len(data) > 1024 * 1024 or hashlib.sha256(data).hexdigest() != digest:
        raise ValueError("evidence digest changed")
    return path, data


def check(root: Path = ROOT) -> dict:
    root = root.resolve(strict=True)
    ids = re.findall(r"^\| (LP-[0-9]{2}) \|", (root / CONTRACT).read_text(), re.MULTILINE)
    expected = {f"LP-{i:02}" for i in range(1, 32)}
    if len(ids) != 31 or set(ids) != expected:
        raise ValueError("contract must contain 31 unique LP IDs")
    data = json.loads((root / MATRIX).read_text())
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
                    raise ValueError("native reference is not a passing source-bound QEMU record")
        counts[status] += 1
        risks.append({"id": ident, "status": status, "remaining": row["remaining"]})
    return {"structurally_valid": True, "counts": counts, "residual_coverage": risks,
            "limit": "Reference validation only; no evidence authenticity, complete-scenario or privacy acceptance."}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", required=True)
    parser.parse_args()
    try:
        print(json.dumps(check(), indent=2))
    except (ValueError, OSError, RuntimeError, TypeError) as error:
        print(f"LP matrix: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
