"""Validate S1 matrix tables and count shared dependencies.

Usage: python3 -B check.py <repo-root> <markdown-file>...
Exits 1 on any structural or evidence error.
"""
import json
import os
import re
import sys
from collections import Counter

SUPPORT = {"full", "with limitations", "lossy", "unsupported", "proprietary research"}
LEVELS = {"proposed", "prototyped (host)", "passed (native)"}
PRIORITY = {"P0", "P1", "P2", "P3"}
DEPS = [
    "Materiaal history", "Text & font engine", "Colour management", "GPU compositing",
    "2D vector engine", "Pixel pipeline", "Media framework", "Low-latency audio",
    "Calculation engine", "Plugin sandbox", "Automation runtime", "Interchange converters",
    "Collaboration & sync", "Accessibility service", "Input & command system",
    "PDF & print engine", "Search & index", "Signing & identity", "Internet protocols",
    "Asset library & linking", "Background job queue", "3D scene engine", "Node-graph engine",
]
HEADER = ["Capability", "Professional expectation", "Beginner path", "Professional depth",
          "Shared dependency", "BaryonOS support today", "Evidence level", "Evidence", "Priority"]


def rows(path):
    in_table = False
    for n, line in enumerate(open(path, encoding="utf-8"), 1):
        if not line.startswith("|"):
            in_table = False
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if cells == HEADER:
            in_table = True
            continue
        if not in_table or set(line.strip()) <= set("|-"):
            continue
        yield path, n, cells


def main():
    root, files = sys.argv[1], sys.argv[2:]
    errors, out, deps, ids = [], [], Counter(), set()
    for f in files:
        for path, n, c in rows(f):
            where = f"{os.path.basename(path)}:{n}"
            if len(c) != 9:
                errors.append(f"{where}: {len(c)} cells")
                continue
            cap, _, _, _, dep, sup, lvl, ev, pri = c
            rid = cap.split()[0]
            if rid in ids:
                errors.append(f"{where}: duplicate id {rid}")
            ids.add(rid)
            if sup not in SUPPORT:
                errors.append(f"{where}: support {sup!r}")
            if lvl not in LEVELS:
                errors.append(f"{where}: level {lvl!r}")
            if pri not in PRIORITY:
                errors.append(f"{where}: priority {pri!r}")
            names = [d.strip() for d in dep.split(";")]
            for d in names:
                if d not in DEPS:
                    errors.append(f"{where}: dependency {d!r}")
                deps[d] += 1
            paths = re.findall(r"`([^`]+)`", ev)
            for p in paths:
                if not os.path.exists(os.path.join(root, p)):
                    errors.append(f"{where}: missing evidence {p}")
            if lvl != "proposed" and not paths:
                errors.append(f"{where}: {lvl} without evidence path")
            if lvl == "proposed" and sup not in {"unsupported", "proprietary research"} \
                    and "native QEMU run" not in ev:
                errors.append(f"{where}: proposed row claims {sup!r} without a named native run")
            if sup == "full":
                errors.append(f"{where}: 'full' needs explicit review")
            out.append({"id": rid, "capability": cap, "support": sup, "level": lvl,
                        "evidence": ev, "priority": pri, "deps": names})
    summary = {
        "rows": len(out),
        "by_level": Counter(r["level"] for r in out),
        "by_support": Counter(r["support"] for r in out),
        "by_priority": Counter(r["priority"] for r in out),
        "deps_ranked": deps.most_common(),
        "unused_deps": [d for d in DEPS if d not in deps],
        "non_proposed": [r for r in out if r["level"] != "proposed"],
        "limited_proposed": [r for r in out if r["level"] == "proposed"
                             and r["support"] not in {"unsupported", "proprietary research"}],
    }
    print(json.dumps(summary, indent=1, ensure_ascii=False))
    for e in errors:
        print("ERROR", e, file=sys.stderr)
    sys.exit(1 if errors else 0)


if __name__ == "__main__":
    main()
