"""Check the accessibility matrix without modifying its archived evidence.

The pinned historical parser and structural rules are reused. Named-file locations
must stay in the supplied repository, and the recorded baseline row identities
cannot disappear or become settings. This is not evidence-quality or UI acceptance.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import sys
import types


ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / "docs/verification/a11y-research-20261115/check.py"
ARCHIVE_SHA256 = "cf1b65d170a385a0e4fde9724d326bbbe4f5d1cdbd7186073800f7b1bd251bab"
BASELINES = ROOT / "docs/verification/a11y-checker-20261118/baseline-contract.json"
BASELINES_SHA256 = "c14d155d0cb4573bbd816cff13b60e557eab73e1af192d98fb482b45b2b42a5f"
MATRIX = Path("docs/design/accessibility/ACCESSIBILITY-MATRIX.md")
FILE_SUFFIX = re.compile(r"\.(?:css|html|js|rs|md|py|json|toml)\b")
QUOTED = re.compile(r"(`[^`]*`)")
# Reject unsafe prefixes in the complete cell too: whitespace or punctuation in
# an unquoted path must not hide its absolute/traversing prefix from a basename.
UNSAFE_LOCATION = re.compile(r"(?<![\w./\\%-])(?:[/\\]|[A-Za-z]:)|(?<![\w.])\.\.[/\\]")
MATRIX_COLUMNS = 12


def pinned_bytes(path: Path, expected: str) -> bytes:
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError(f"SHA256 mismatch for pinned contract: {path.name}")
    return data


def load_reference(path: Path = ARCHIVE) -> types.ModuleType:
    """Execute only the bytes whose digest was checked, without a second read."""
    data = pinned_bytes(path, ARCHIVE_SHA256)
    reference = types.ModuleType("baryonos_a11y_reference")
    exec(compile(data, str(path), "exec"), reference.__dict__)
    return reference


def baseline_errors(rows, path: Path = BASELINES) -> list[str]:
    contract = json.loads(pinned_bytes(path, BASELINES_SHA256))
    by_identity = {}
    for row in rows:
        if row is not None and len(row) == MATRIX_COLUMNS:
            by_identity.setdefault((row[0], row[2]), []).append(row)
    errors = []
    for required in contract["baseline_rows"]:
        identity = (required["need"], required["adaptation"])
        found = by_identity.get(identity, [])
        label = f"required baseline ({identity[0]} / {identity[1][:60]})"
        if len(found) != 1:
            errors.append(f"{label}: expected exactly one row, found {len(found)}")
        elif found[0][3] != "baseline" or not found[0][4].startswith("none (baseline)"):
            errors.append(f"{label}: tier and axis must retain the baseline contract")
    return errors


def contained_file(root: Path, path: Path) -> bool:
    try:
        resolved = path.resolve(strict=True)
        return resolved.is_relative_to(root) and resolved.is_file()
    except (OSError, RuntimeError):
        return False


def file_references(value: str) -> list[str]:
    references = []
    for part in QUOTED.split(value):
        if part.startswith("`") and part.endswith("`"):
            # A quoted location stays whole, including unsupported punctuation
            # or spaces; it can never degrade into a matching suffix/basename.
            candidates = [part[1:-1].strip()]
        else:
            candidates = [token.lstrip("([{\"'").rstrip(".,;:!?)]}\"'") for token in part.split()]
        references.extend(token for token in candidates if FILE_SUFFIX.search(token))
    return references


def missing_files(rows, root: Path) -> list[str]:
    root = root.resolve(strict=True)
    names = set()
    for name in ("ui", "docs"):
        directory = root / name
        if directory.resolve().is_relative_to(root) and directory.is_dir():
            # Path.rglob does not descend into directory symlinks. File symlinks
            # qualify only when their resolved regular file remains in root.
            names.update(path.name for path in directory.rglob("*") if contained_file(root, path))
    errors = []
    for number, row in enumerate(rows, 1):
        if row is None or len(row) != MATRIX_COLUMNS:
            continue
        for cell in row[8:10]:
            references = file_references(cell)
            if references and UNSAFE_LOCATION.search(cell):
                errors.append(f"matrix row {number}: named-file location has an unsafe path prefix")
                continue
            for token in references:
                parts = PurePosixPath(token)
                safe = not (parts.is_absolute() or ".." in parts.parts or "\\" in token or ":" in token)
                found = contained_file(root, root / token) if safe and "/" in token else safe and token in names
                if not found:
                    errors.append(f"matrix row {number}: named file {token!r} is absent or outside the repository")
    return errors


def check(root: Path) -> dict:
    reference = load_reference()
    root = root.resolve(strict=True)
    path = root / MATRIX
    if not contained_file(root, path):
        raise ValueError(f"matrix is missing or outside the repository: {MATRIX}")
    data = path.read_bytes()
    lines = data.decode("utf-8").splitlines()
    rows, matrix_error = reference.table(lines, reference.MATRIX_HEADER)
    sources, source_error = reference.table(lines, reference.SOURCES_HEADER)
    errors = [error for error in (matrix_error, source_error) if error]
    out = {}
    if not errors:
        structural, _, _ = reference.validate(rows, sources)
        errors += structural + missing_files(rows, root) + baseline_errors(rows)
        out = reference.summary(rows, sources)
    out.update(ok=not errors, errors=errors, matrix_sha256=hashlib.sha256(data).hexdigest(),
               reference_checker_sha256=ARCHIVE_SHA256, baseline_contract_sha256=BASELINES_SHA256)
    return out


def main(argv: list[str] | None = None) -> int:
    arguments = sys.argv[1:] if argv is None else argv
    try:
        if len(arguments) != 1:
            raise ValueError("usage: python3 -B tools/check_a11y_matrix.py <repo-root>")
        result = check(Path(arguments[0]))
    except (OSError, RuntimeError, ValueError) as error:
        result = {"ok": False, "errors": [str(error)]}
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
