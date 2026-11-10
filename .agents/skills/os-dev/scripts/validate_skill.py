#!/usr/bin/env python3
from __future__ import annotations
import hashlib, json, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def fail(msg):
    raise ValueError(msg)

def main():
    skill=ROOT/"SKILL.md"
    if not skill.is_file(): fail("SKILL.md is missing")
    text=skill.read_text(encoding="utf-8")
    if not text.startswith("---\n") or "name: os-dev" not in text.split("---",2)[1]:
        fail("invalid skill frontmatter")

    rows=json.loads((ROOT/"references/source-index.json").read_text(encoding="utf-8"))
    manifest=json.loads((ROOT/"assets/source-manifest.json").read_text(encoding="utf-8"))
    wiki=sum(r["kind"]=="wiki" for r in rows)
    pdf=sum(r["kind"]=="pdf" for r in rows)
    if wiki < 3000: fail(f"unexpectedly few wiki pages: {wiki}")
    if pdf != 78: fail(f"incorrect book page count: {pdf}")
    if manifest["source_records"] != len(rows) or manifest["wiki_pages"] != wiki or manifest["pdf_pages"] != pdf:
        fail("manifest counts do not match the source index")

    seen=set(); cache={}
    for r in rows:
        if r["source_id"] in seen: fail("duplicate source ID "+r["source_id"])
        seen.add(r["source_id"])
        p=(ROOT/r["local_path"]).resolve()
        if not p.is_relative_to(ROOT.resolve()) or not p.is_file():
            fail("invalid source path "+r["source_id"])
        data=cache.setdefault(str(p),p.read_bytes())
        start=r["start_byte"]; size=r["length_bytes"]; block=data[start:start+size]
        if len(block)!=size or hashlib.sha256(block).hexdigest()!=r["text_sha256"]:
            fail("source hash "+r["source_id"])
        if f'<a id="{r["anchor"]}"></a>'.encode() not in data:
            fail("anchor "+r["source_id"])

    if not (ROOT/"assets/book.pdf").is_file(): fail("original PDF is missing")
    print(json.dumps({
      "status":"PASS","source_records":len(rows),"wiki_pages":wiki,"pdf_pages":pdf,
      "newest_wiki_revision":manifest["newest_wiki_revision"],
      "source_mode":manifest.get("source_mode"),
      "skill_files":sum(p.is_file() for p in ROOT.rglob("*"))
    },indent=2))

if __name__=="__main__":
    try: main()
    except (OSError,ValueError,KeyError,json.JSONDecodeError) as e:
        print("FAIL:",e,file=sys.stderr)
        raise SystemExit(1)
