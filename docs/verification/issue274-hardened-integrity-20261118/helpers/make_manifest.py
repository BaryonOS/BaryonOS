#!/usr/bin/env python3
"""Create the deterministic SHA-256 manifest for the hardened draft bundle."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    draft = Path(__file__).resolve().parents[1]
    root = next(
        (candidate for candidate in (draft, *draft.parents) if (candidate / ".git").exists()),
        None,
    )
    if root is None:
        raise ValueError("Bundle is not inside a Git repository")
    relative = lambda value: value.relative_to(root).as_posix()
    coverage_relative = relative(draft / "output" / "coverage.json")
    collector_relative = relative(draft / "output" / "collector.json")
    review_relative = relative(draft / "inputs" / "review.json")
    helper_paths = [
        relative(draft / "helpers" / "collect.cjs"),
        relative(draft / "helpers" / "check.py"),
        relative(draft / "helpers" / "verify.py"),
    ]
    supplementary = [
        relative(draft / "helpers" / "capture.py"),
        relative(draft / "helpers" / "make_manifest.py"),
        relative(draft / "helpers" / "self_test.py"),
        relative(draft / "inputs" / "publication.json"),
        relative(draft / "inputs" / "published.json"),
    ]
    coverage = json.loads((root / coverage_relative).read_text())
    members = helper_paths + supplementary + [coverage_relative, collector_relative, review_relative]
    design = relative(draft / "DESIGN.md")
    if (root / design).is_file():
        members.append(design)
    manifest = {
        "schema_version": 1,
        "source_commit": coverage["source_commit"],
        "source_tree": coverage["source_tree"],
        "wiki_commit": coverage["wiki_commit"],
        "wiki_tree": coverage["wiki_tree"],
        "inventory": coverage_relative,
        "collector_output": collector_relative,
        "review": review_relative,
        "collector_helper": helper_paths[0],
        "publication_record": relative(draft / "inputs" / "publication.json"),
        "published_receipt": relative(draft / "inputs" / "published.json"),
        "helpers": helper_paths,
        "files": {relative: digest(root / relative) for relative in sorted(set(members))},
    }
    (draft / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    hash_members = sorted(
        path for path in draft.rglob("*")
        if path.is_file()
        and path.name != "hashes.json"
        and "__pycache__" not in path.parts
    )
    hashes = {
        "algorithm": "sha256",
        "files": {
            path.relative_to(draft).as_posix(): {
                "bytes": path.stat().st_size,
                "sha256": digest(path),
            }
            for path in hash_members
        },
    }
    (draft / "hashes.json").write_text(json.dumps(hashes, indent=2, sort_keys=True) + "\n")
    print(json.dumps({
        "files": len(manifest["files"]),
        "hash_files": len(hashes["files"]),
        "output": str(draft / "manifest.json"),
    }, sort_keys=True))


if __name__ == "__main__":
    main()
