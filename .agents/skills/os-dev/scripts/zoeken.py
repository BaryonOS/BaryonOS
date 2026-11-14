#!/usr/bin/env python3
"""Read-only local search over the generated OS development source index."""
from __future__ import annotations
import argparse, hashlib, json, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INDEX = ROOT / "references" / "source-index.json"

def rows():
    return json.loads(INDEX.read_text(encoding="utf-8"))

def body(row):
    p = (ROOT / row["local_path"]).resolve()
    data = p.read_bytes()
    start, length = row["start_byte"], row["length_bytes"]
    chunk = data[start:start+length]
    if hashlib.sha256(chunk).hexdigest() != row["text_sha256"]:
        raise ValueError("source hash mismatch: " + row["source_id"])
    return chunk.decode("utf-8")

def search(query, source="all", all_namespaces=False, limit=10, title=None):
    q = (query or "").casefold().split()
    out = []
    for r in rows():
        if source != "all" and r["kind"] != source:
            continue
        if r["kind"] == "wiki" and not all_namespaces and (r.get("namespace", 0) != 0 or r.get("redirect")):
            continue
        title_text = r["title"].casefold()
        if title and title.casefold() not in title_text:
            continue
        text = body(r)
        hay = (r["title"] + "\n" + text).casefold()
        if q and not all(x in hay for x in q):
            continue
        score = sum(8 for x in q if x in title_text) + sum(hay.count(x) for x in q)
        rr = dict(r); rr["_body"] = text; rr["_score"] = score
        out.append(rr)
    out.sort(key=lambda r: (-r["_score"], r["title"].casefold()))
    return out[:limit]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("query", nargs="?")
    ap.add_argument("--id")
    ap.add_argument("--title")
    ap.add_argument("--source", choices=["all","wiki","pdf"], default="all")
    ap.add_argument("--all-namespaces", action="store_true")
    ap.add_argument("--limit", type=int, default=10)
    ap.add_argument("--start-line", type=int, default=1)
    ap.add_argument("--max-lines", type=int, default=40)
    ap.add_argument("--full", action="store_true")
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    rs = rows()
    if a.id:
        rs = [r for r in rs if r["source_id"] == a.id]
        if not rs:
            raise SystemExit("unknown source ID")
        for r in rs: r["_body"] = body(r); r["_score"] = 0
    else:
        rs = search(a.query or "", a.source, a.all_namespaces, a.limit, a.title)
    if a.json:
        clean = [{k:v for k,v in r.items() if not k.startswith("_")} for r in rs]
        print(json.dumps(clean, ensure_ascii=False, indent=2)); return
    for r in rs:
        print(f'[{r["source_id"]}] {r["title"]} ({r["kind"]})')
        if r["kind"] == "wiki":
            print("revision:", r.get("revision_timestamp",""))
        lines = r["_body"].splitlines()
        start = max(0, a.start_line - 1)
        end = len(lines) if a.full else min(len(lines), start + a.max_lines)
        for i, line in enumerate(lines[start:end], start+1):
            print(f"{i:>5} {line}")
        print()

if __name__ == "__main__":
    try: main()
    except (OSError, ValueError, json.JSONDecodeError) as e:
        print("ERROR:", e, file=sys.stderr); raise SystemExit(2)
