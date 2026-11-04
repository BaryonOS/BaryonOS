#!/usr/bin/env python3
"""Read-only inventory of local Graphify extraction checkpoints.

This tool intentionally does not merge partial graph data or replace any
Graphify output. Existing chunks stay the source of truth. Python 3.10+ stdlib.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

TEXT_CHUNK = re.compile(r"^\.graphify_chunk_(\d+)\.json$")
IMAGE_CHUNK = re.compile(
    r"^\.?graphify[_-].*(?:image|img|vision|visual)[_-].*(?:chunk|batch)[_-]?(\d+)\.json$"
    r"|^\.?graphify[_-].*(?:chunk|batch)[_-]?(\d+)[_-](?:image|img|vision|visual).*\.json$",
    re.IGNORECASE,
)
FINAL_ARTIFACTS = ("graph.json", "graph.html", "GRAPH_REPORT.md")


def _sha256(path: Path) -> str:
    sha = hashlib.sha256()
    with path.open("rb") as stream:
        for part in iter(lambda: stream.read(1024 * 1024), b""):
            sha.update(part)
    return sha.hexdigest()


def _inspect_json(path: Path) -> dict[str, Any]:
    result: dict[str, Any] = {"name": path.name, "bytes": path.stat().st_size,
                              "sha256": _sha256(path)}
    try:
        with path.open("r", encoding="utf-8") as stream:
            payload = json.load(stream)
        if not isinstance(payload, dict):
            raise ValueError("JSON-topniveau moet een object zijn")
    except (OSError, UnicodeError, json.JSONDecodeError, ValueError) as exc:
        result["valid"] = False
        result["error"] = f"{type(exc).__name__}: {exc}"
        return result
    result["valid"] = True
    result["keys"] = sorted(payload.keys())
    result["item_counts"] = {
        key: len(payload[key]) for key in ("nodes", "edges", "links", "hyperedges")
        if isinstance(payload.get(key), list)
    }
    return result


def inspect(output_dir: Path, expected_text: int = 20, expected_images: int = 35,
            ast: Path | None = None) -> dict[str, Any]:
    """Inspect a checkpoint without modifying source data or assuming an edge schema."""
    if expected_text < 0 or expected_images < 0:
        raise ValueError("expected_text en expected_images moeten niet-negatief zijn")
    if not output_dir.is_dir():
        raise FileNotFoundError(f"Graphify-outputmap niet gevonden: {output_dir}")

    text_groups: dict[int, list[Path]] = {}
    image_groups: dict[int, list[Path]] = {}
    unrecognized: list[str] = []
    for path in sorted(output_dir.iterdir()):
        if not path.is_file():
            continue
        m = TEXT_CHUNK.fullmatch(path.name)
        if m:
            text_groups.setdefault(int(m.group(1)), []).append(path)
        else:
            m = IMAGE_CHUNK.fullmatch(path.name)
            if m:
                image_groups.setdefault(int(m.group(1) or m.group(2)), []).append(path)
            elif "graphify" in path.name.lower() and "chunk" in path.name.lower() and path.suffix == ".json":
                unrecognized.append(path.name)

    records = [(_inspect_json(p), "text", index) for index, ps in sorted(text_groups.items()) for p in ps]
    records.extend((_inspect_json(p), "image", index) for index, ps in sorted(image_groups.items()) for p in ps)
    for rec, kind, index in records:
        rec.update({"kind": kind, "index": index})

    if ast is not None and not ast.is_file():
        raise FileNotFoundError(f"AST-bestand niet gevonden: {ast}")
    ast_record = _inspect_json(ast) if ast else None
    any_invalid = any(not rec["valid"] for rec, _, _ in records)
    if ast_record and not ast_record["valid"]:
        any_invalid = True
    duplicate_text = {str(index): [p.name for p in ps] for index, ps in text_groups.items() if len(ps) > 1}
    duplicate_image = {str(index): [p.name for p in ps] for index, ps in image_groups.items() if len(ps) > 1}
    missing_text = [i for i in range(1, expected_text + 1) if i not in text_groups]
    extra_text = [i for i in text_groups if i < 1 or i > expected_text]
    missing_images = [i for i in range(1, expected_images + 1) if i not in image_groups]
    final_status = {name: (output_dir / name).is_file() for name in FINAL_ARTIFACTS}
    return {
        "scope": "local Graphify extraction checkpoint, not a completed or verified graph",
        "directory": str(output_dir),
        "expected_text_chunks": expected_text,
        "expected_image_chunks": expected_images,
        "text_chunks_found": len(text_groups),
        "image_chunks_recognized": len(image_groups),
        "text_indices": sorted(text_groups),
        "image_indices": sorted(image_groups),
        "missing_text_indices": missing_text,
        "missing_image_indices_if_named_conventionally": missing_images,
        "extra_text_indices": sorted(extra_text),
        "duplicate_text_indices": duplicate_text,
        "duplicate_image_indices": duplicate_image,
        "unrecognized_chunk_files": unrecognized,
        "chunk_records": [rec for rec, _, _ in records],
        "ast": ast_record if ast else "not supplied (AST output path unknown)",
        "final_artifacts_present": final_status,
        "json_valid": not any_invalid,
        "complete": False,  # Deliberate: not established by a filename and JSON syntax audit.
        "why_not_complete": "Actual AST, semantic schema, community labels, links and HTML output must still be independently validated.",
    }


def _write_new(path: Path, data: str) -> None:
    """Never replace an existing report, even when previous runs left one."""
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        stream.write(data)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=Path("graphify-out"))
    parser.add_argument("--expected-text", type=int, default=20)
    parser.add_argument("--expected-images", type=int, default=35)
    parser.add_argument("--ast", type=Path, help="Exact path to existing AST JSON artifact, if known")
    parser.add_argument("--write-manifest", type=Path, help="Write a NEW JSON manifest (never overwrite)")
    args = parser.parse_args(argv)
    try:
        result = inspect(args.output_dir, args.expected_text, args.expected_images, args.ast)
        if args.write_manifest:
            _write_new(args.write_manifest, json.dumps(result, indent=2, ensure_ascii=False) + "\n")
    except (FileNotFoundError, FileExistsError, PermissionError, ValueError, OSError) as exc:
        print(f"FOUT: {exc}", file=sys.stderr)
        return 2

    print(f"Graphify-checkpoint: {result['directory']}")
    print(f"Tekstchunks: {result['text_chunks_found']}/{result['expected_text_chunks']}; "
          f"ontbrekende indices: {result['missing_text_indices']}")
    print(f"Afbeeldingschunks: {result['image_chunks_recognized']}/"
          f"{result['expected_image_chunks']} herkend; naamconventie niet bevestigd")
    print(f"Chunk-JSON geldig: {'ja' if result['json_valid'] else 'NEE'}")
    print("Eindproducten aanwezig: " + ", ".join(f"{k}={'ja' if v else 'nee'}"
                                                      for k, v in result['final_artifacts_present'].items()))
    print("Status: inventarisatie uitgevoerd; volledige graaf NIET gevalideerd")
    return 0 if result["json_valid"] and not result["duplicate_text_indices"] and not result["duplicate_image_indices"] else 1


if __name__ == "__main__":
    sys.exit(main())
