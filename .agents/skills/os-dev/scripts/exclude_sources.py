#!/usr/bin/env python3
"""Exclude named wiki source records from the active reference corpus.

Removes each record's section from its corpus volume, recomputes the byte and
line locations of later records in that volume, drops the record from
`references/source-index.json`, `references/pagina-index.md` and the theme
pages, updates the manifest counts and appends a provenance entry to
`references/exclusions.json`. Revision metadata and Git history are untouched.
Usage: exclude_sources.py --reason TEXT --record PATH SOURCE_ID [...]
"""
from __future__ import annotations
import argparse, hashlib, json, os, re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REF = ROOT / "references"


def write_atomic(path: Path, data: bytes) -> None:
    """Replace `path` in one step, so a failed write leaves the old contents."""
    tmp = path.with_name(path.name + ".exclude-tmp")
    try:
        tmp.write_bytes(data)
        os.replace(tmp, path)
    finally:
        tmp.unlink(missing_ok=True)


def keep_original(path: Path, data: bytes | None) -> str:
    """Save an unrestored file's original bytes beside it; return where, or why not."""
    if data is None:
        return "file did not exist"
    copy = path.with_name(path.name + ".exclude-orig")
    try:
        copy.write_bytes(data)
    except OSError as exc:
        return f"not saved: {exc}"
    return str(copy)


def commit(staged: dict[Path, bytes]) -> None:
    """Write every staged file or none: restore the originals if any write fails.

    Each file is replaced atomically, so the file whose write failed keeps its
    original contents. Rollback attempts every written file even when one
    restoration fails, and reports the files it could not restore together with
    a saved copy of their original contents.
    """
    originals = {p: (p.read_bytes() if p.exists() else None) for p in staged}
    written = []
    try:
        for path, data in staged.items():
            write_atomic(path, data)
            written.append(path)
    except OSError as err:
        unrestored = []
        for path in written:
            try:
                if originals[path] is None:
                    path.unlink(missing_ok=True)
                else:
                    write_atomic(path, originals[path])
            except OSError:
                unrestored.append(f"{path} (original: {keep_original(path, originals[path])})")
        if unrestored:
            raise OSError("write failed and rollback could not restore: " + "; ".join(unrestored)) from err
        raise


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--reason", required=True)
    ap.add_argument("--record", required=True, help="repository-relative evidence path")
    ap.add_argument("ids", nargs="+")
    a = ap.parse_args()
    ids = set(a.ids)
    rows = json.loads((REF / "source-index.json").read_text(encoding="utf-8"))
    by_id = {r["source_id"]: r for r in rows}
    missing = sorted(i for i in ids if i not in by_id)
    if missing:
        raise SystemExit(f"unknown or already excluded source IDs: {missing}")
    if any(by_id[i]["kind"] != "wiki" for i in ids):
        raise SystemExit("only wiki records can be excluded")

    excluded = []
    staged: dict[Path, bytes] = {}
    for path in sorted({by_id[i]["local_path"] for i in ids}):
        file = ROOT / path
        data = file.read_bytes()
        cut = []  # (start, end, lines) of removed sections, ascending
        for i in sorted((i for i in ids if by_id[i]["local_path"] == path),
                        key=lambda i: by_id[i]["start_byte"]):
            r = by_id[i]
            anchor = f'<a id="{r["anchor"]}"></a>\n'.encode()
            start = data.index(anchor)
            nxt = data.find(b'<a id="', start + len(anchor))
            end = len(data) if nxt < 0 else nxt
            section = data[start:end]
            body = data[r["start_byte"]:r["start_byte"] + r["length_bytes"]]
            if not (start <= r["start_byte"] and r["start_byte"] + r["length_bytes"] <= end):
                raise SystemExit(f"{i}: source text outside its section")
            if hashlib.sha256(body).hexdigest() != r["text_sha256"]:
                raise SystemExit(f"{i}: source text does not match its recorded hash")
            cut.append((start, end, section.count(b"\n")))
            excluded.append(r)
        out, pos = bytearray(), 0
        for s, e, _ in cut:
            out += data[pos:s]
            pos = e
        out += data[pos:]
        staged[file] = bytes(out)
        for r in rows:
            if r["local_path"] != path or r["source_id"] in ids:
                continue
            gone = [(s, e, n) for s, e, n in cut if e <= r["start_byte"]]
            r["start_byte"] -= sum(e - s for s, e, _ in gone)
            r["start_line"] -= sum(n for *_, n in gone)
            r["end_line"] -= sum(n for *_, n in gone)

    rows = [r for r in rows if r["source_id"] not in ids]
    staged[REF / "source-index.json"] = (
        json.dumps(rows, separators=(",", ":"), ensure_ascii=False) + "\n").encode()

    new = {r["source_id"]: r for r in rows}
    page = REF / "pagina-index.md"
    lines = []
    for line in page.read_text(encoding="utf-8").split("\n"):
        m = re.match(r"- \[(W\d+) — .*?\(corpus/[^)]+\) · source lines (\d+)–(\d+)", line)
        if m and m.group(1) in ids:
            continue
        if m and m.group(1) in new:
            r = new[m.group(1)]
            line = line.replace(f"source lines {m.group(2)}–{m.group(3)}",
                                f"source lines {r['start_line']}–{r['end_line']}", 1)
        lines.append(line)
    staged[page] = "\n".join(lines).encode()

    alt = "|".join(sorted(ids))
    item = re.compile(r"^- .*\[(" + alt + r")\](?:\s*·.*)?\s*$")
    for theme in sorted((REF / "themas").glob("*.md")):
        text = theme.read_text(encoding="utf-8").split("\n")
        kept = [l for l in text if not item.match(l)]
        if kept != text:
            staged[theme] = "\n".join(kept).encode()
    # Inline "[Wnnnnnn: Title]; " entry points in the knowledge guide.
    inline = re.compile(r"\[(?:" + alt + r"): [^\]]*\](?:; |(?=\.))")
    guide = REF / "kennisgids.md"
    text = guide.read_text(encoding="utf-8")
    new_text = inline.sub("", text).replace("; .", ".")
    if new_text != text:
        staged[guide] = new_text.encode()
    media = ROOT / "assets/wiki-media-references.json"
    refs = json.loads(media.read_text(encoding="utf-8"))
    kept_refs = [m for m in refs if m["source_id"] not in ids]
    if kept_refs != refs:
        staged[media] = (json.dumps(kept_refs, indent=2, ensure_ascii=False) + "\n").encode()

    mf = ROOT / "assets/source-manifest.json"
    m = json.loads(mf.read_text(encoding="utf-8"))
    for r in excluded:
        m["source_records"] -= 1
        m["wiki_pages"] -= 1
        m["source_text_bytes"] -= r["length_bytes"]
        if r["namespace"] == 0:
            m["wiki_main_namespace_pages"] -= 1
            if not r["redirect"]:
                m["wiki_main_articles_excluding_redirects"] -= 1
    staged[mf] = (json.dumps(m, indent=2, ensure_ascii=False) + "\n").encode()

    ex = REF / "exclusions.json"
    log = json.loads(ex.read_text(encoding="utf-8")) if ex.exists() else []
    for r in sorted(excluded, key=lambda r: r["source_id"]):
        log.append({k: r[k] for k in ("source_id", "page_id", "title", "revision_id",
                                       "timestamp", "text_sha256", "length_bytes",
                                       "source_url", "revision_url")}
                   | {"reason": a.reason, "record": a.record})
    staged[ex] = (json.dumps(log, indent=2, ensure_ascii=False) + "\n").encode()
    commit(staged)
    print(f"excluded {len(excluded)}: {', '.join(sorted(ids))}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
