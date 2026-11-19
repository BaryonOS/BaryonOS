"""Validate proposed review inventory paths, never security acceptance."""
from __future__ import annotations
import json
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
SCOPE = Path("docs/security-review/scope.json")
BOUNDARIES = {"syscalls-usercopy", "ipc-lengths", "capability-engine", "virtio-dma",
              "network-parsers", "elf-loader", "keyring-recovery"}


def check(root: Path = ROOT) -> dict:
    root = root.resolve(strict=True)
    scope = json.loads((root / SCOPE).read_text())
    if not isinstance(scope, dict) or set(scope) != {"status", "boundaries"} or scope["status"] != "PROPOSED":
        raise ValueError("review scope must remain PROPOSED")
    rows = scope["boundaries"]
    if not isinstance(rows, list) or len(rows) != len(BOUNDARIES):
        raise ValueError("required review boundaries missing")
    seen, references = set(), 0
    for row in rows:
        if not isinstance(row, dict) or set(row) != {"id", "invariants", "files", "tests", "evidence"}:
            raise ValueError("invalid boundary")
        ident = row["id"]
        if not isinstance(ident, str) or ident not in BOUNDARIES or ident in seen:
            raise ValueError("duplicate/unknown boundary")
        seen.add(ident)
        invariants = row["invariants"]
        if not isinstance(invariants, list) or not invariants or any(not isinstance(x, str) or not x.strip() for x in invariants):
            raise ValueError("boundary invariants missing")
        for field in ("files", "tests", "evidence"):
            paths = row[field]
            if not isinstance(paths, list) or not paths or len(paths) > 32:
                raise ValueError("bounded path list required")
            for name in paths:
                if not isinstance(name, str):
                    raise ValueError("path must be a string")
                path = PurePosixPath(name)
                if path.is_absolute() or path.as_posix() != name or ".." in path.parts or "\\" in name or ":" in name:
                    raise ValueError("noncanonical scope path")
                actual = (root / name).resolve(strict=True)
                if not actual.is_relative_to(root) or not actual.is_file():
                    raise ValueError("scope path escapes repository or is not a file")
                references += 1
    return dict(proposed=True, boundaries=len(seen), references=references,
                limit="Path inventory only; no test execution, evidence acceptance or external review.")


if __name__ == "__main__":
    try:
        print(json.dumps(check(), indent=2))
    except (OSError, ValueError, TypeError, RuntimeError) as error:
        raise SystemExit(f"review scope: {error}") from error
