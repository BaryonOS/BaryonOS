"""Structural check for docs/design/accessibility/ACCESSIBILITY-MATRIX.md (UX-A11Y-R, 15 November 2026).

Usage: python3 -I check.py <repo-root>
Exit 0 when the matrix is structurally valid, 1 on any structural error. Prints a JSON summary.
It checks structure and value sets only; it does not judge evidence quality or user effect. Its one evidence-related
rule is a floor, not a judgement: a row graded strong or moderate must cite at least one peer-reviewed source whose
resolution starts with 'resolved' (repair round 2). It cannot detect barrier-only support or inconsistencies between
the research text and the matrix; those need human or independent review.
Safety rows: a row whose Adaptation starts with 'Safety:', or whose Need or Adaptation mentions flash, seizure or
vestibular safety limit, must be baseline; every limit in SAFETY_TOPICS must be carried by a tagged row. The built-in
self-test demotes and untags every tagged row in turn and expects an error each time (repair round 1).
Named files (repair run 2): every file name or path in a Prototype status or Native status cell must exist, a path
relative to the repository root, a bare file name anywhere under ui/ or docs/. It does not check line numbers,
selectors or what the file says, so a 'works' or 'partial' location can still be wrong in content.
The stripe-texture comfort bound (matrix row 21) is an untagged baseline comfort rule, not a safety limit, since
repair round 1 of run 2; only the tier/axis agreement rule holds it baseline (see Known limits).
Known limits: it does not detect a non-safety baseline row demoted together with a consistent axis edit (tier
'setting' and a non-baseline axis), and it is not evidence verification.
"""
import copy
import json
import re
import sys
from pathlib import Path

MATRIX_HEADER = ("| Need | Barrier | Adaptation | Tier | Axis / configuration | Normative basis | Empirical evidence "
                 "| Sources | Prototype status | Native status | Verification method | User test needed |")
SOURCES_HEADER = "| ID | Reference | Type | Resolution |"
COLS = [c.strip() for c in MATRIX_HEADER.strip("|").split("|")]
TIERS = {"baseline", "setting"}
GRADES = {"strong", "moderate", "limited", "insufficient", "contradicted"}
PROTO = {"works", "partial", "absent"}
NATIVE = {"feasible-now", "needs-extension", "future-research"}
YESNO = {"yes", "no"}
TYPES = {"peer-reviewed", "standard", "w3c-note", "organisation", "platform-doc", "other"}
RESOLUTION = ("resolved", "fetched", "not read in full", "unverified")
OWNER_NEEDS = {
    "colour vision": r"colou?r[ -]vision", "low vision": r"low vision", "blindness": r"\bblind(ness)?\b",
    "light sensitivity": r"light sensitivity", "photosensitive epilepsy": r"photosensitive epilepsy",
    "vestibular": r"vestibular", "dyslexia": r"dyslexia", "dyscalculia": r"dyscalculia", "ADHD": r"\badhd\b",
    "autism": r"autis(m|tic)", "motor": r"\bmotor\b", "hearing": r"hearing|\bdeaf\b",
}
SAFETY = re.compile(r"flash|seizure|vestibular safety limit", re.I)
SAFETY_MARK = re.compile(r"^Safety:")  # explicit tag at the start of Adaptation for every safety row
# Required safety limits (research section 6.1 "Safety"), keyed by stable Adaptation text. Each must be
# carried by a row tagged "Safety:", so untagging, deleting or rewording a safety row is detected.
SAFETY_TOPICS = {
    "flash limit for system UI": r"flashes more than three times",
    "pattern limit": r"stripe pairs",
    "third-party flash guard": r"compositor measures luminance change",
    "absolute-luminance (HDR) rule": r"BT\.1702-3 absolute",
    "native redraw flicker": r"full-screen redraw",
    "vestibular safety limit (large-field motion)": r"vestibular safety limit",
    "auto-moving content stops": r"stops by itself within 5 s",
    "static equivalent of motion": r"designed static equivalent",
    "keyboard-filter safety": r"filter state always visible",
    "consent under alternative input": r"consent prompts and destructive actions",
    "security signals under every setting": r"never batched, delayed, hidden",
}


FILE_TOKEN = re.compile(r"[\w./-]*\w\.(?:css|html|js|rs|md|py|json|toml)\b")


def missing_files(mrows, root):
    """File names in Prototype and Native status cells that do not exist in the repository."""
    names = {p.name for d in ("ui", "docs") if (root / d).is_dir() for p in (root / d).rglob("*") if p.is_file()}
    errors = []
    for n, r in enumerate(mrows, 1):
        if r and len(r) == len(COLS):
            for tok in FILE_TOKEN.findall(r[8] + " " + r[9]):
                if not ((root / tok).is_file() if "/" in tok else tok in names):
                    errors.append(f"matrix row {n}: named file {tok!r} not found in the repository")
    return errors


def split_row(line):
    if not (line.startswith("|") and line.endswith("|")):
        return None
    return [c.strip() for c in line[1:-1].split("|")]


def table(lines, header):
    """Return (rows, error) for the table whose header line equals `header` exactly."""
    idx = [i for i, l in enumerate(lines) if l.rstrip() == header]
    if len(idx) != 1:
        return [], f"header {header[:40]!r} found {len(idx)} times (expected exactly once)"
    i = idx[0] + 1
    if i >= len(lines) or not re.fullmatch(r"\|(\s*:?-{3,}:?\s*\|)+", lines[i].strip()):
        return [], f"separator line missing after header {header[:40]!r}"
    rows = []
    for l in lines[i + 1:]:
        l = l.rstrip()
        if not l.startswith("|"):
            break
        rows.append(split_row(l))
    return rows, None


def status_ok(value, allowed, need_detail):
    m = re.fullmatch(r"([a-z-]+)(?::\s*(.*))?", value, re.S)
    if not m or m.group(1) not in allowed:
        return False
    return bool(m.group(2) and m.group(2).strip()) if m.group(1) in need_detail else True


def validate(mrows, srows):
    errors = []
    src = {}
    kind = {}
    for n, r in enumerate(srows, 1):
        if r is None or len(r) != 4:
            errors.append(f"source row {n}: expected 4 cells")
            continue
        sid, ref, typ, res = r
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", sid):
            errors.append(f"source row {n}: bad id {sid!r}")
        if sid in src:
            errors.append(f"source {sid}: duplicate id")
        if not ref:
            errors.append(f"source {sid}: empty reference")
        if typ not in TYPES:
            errors.append(f"source {sid}: type {typ!r} not in {sorted(TYPES)}")
        if not res.lower().startswith(RESOLUTION):
            errors.append(f"source {sid}: resolution must start with one of {RESOLUTION}")
        src[sid] = res.lower()
        kind[sid] = typ
    covered = {k: 0 for k in OWNER_NEEDS}
    topics = {k: 0 for k in SAFETY_TOPICS}
    used = set()
    for n, r in enumerate(mrows, 1):
        if r is None or len(r) != len(COLS):
            errors.append(f"matrix row {n}: expected {len(COLS)} cells, got {None if r is None else len(r)}")
            continue
        c = dict(zip(COLS, r))
        tag = f"matrix row {n} ({c['Need'][:30]} / {c['Adaptation'][:30]})"
        for f in ("Need", "Barrier", "Adaptation", "Axis / configuration", "Normative basis"):
            if not c[f]:
                errors.append(f"{tag}: empty {f}")
        if c["Tier"] not in TIERS:
            errors.append(f"{tag}: tier {c['Tier']!r}")
        if c["Empirical evidence"] not in GRADES:
            errors.append(f"{tag}: empirical {c['Empirical evidence']!r}")
        if not status_ok(c["Prototype status"], PROTO, {"works", "partial"}):
            errors.append(f"{tag}: prototype status {c['Prototype status'][:40]!r} (works and partial need a location after a colon)")
        if not status_ok(c["Native status"], NATIVE, NATIVE):
            errors.append(f"{tag}: native status {c['Native status'][:40]!r} (needs a colon and the component or contract)")
        if c["User test needed"] not in YESNO:
            errors.append(f"{tag}: user test {c['User test needed']!r}")
        if not c["Verification method"]:
            errors.append(f"{tag}: no verification method")
        ids = [s.strip() for s in c["Sources"].split(";") if s.strip()]
        if not ids:
            errors.append(f"{tag}: no sources")
        for s in ids:
            used.add(s)
            if s not in src:
                errors.append(f"{tag}: source {s} missing from the sources table")
        if (c["Tier"] == "baseline") != c["Axis / configuration"].startswith("none (baseline)"):
            errors.append(f"{tag}: tier and axis disagree (baseline exactly when the axis starts with 'none (baseline)')")
        marked = bool(SAFETY_MARK.match(c["Adaptation"]))
        if (marked or SAFETY.search(c["Need"] + " " + c["Adaptation"])) and c["Tier"] != "baseline":
            errors.append(f"{tag}: safety row ('Safety:' tag, flash, seizure or vestibular safety limit) must be baseline")
        if marked:
            for k, pat in SAFETY_TOPICS.items():
                if re.search(pat, c["Adaptation"]):
                    topics[k] += 1
        if c["Empirical evidence"] in ("strong", "moderate") and ids and all(src.get(s, "unverified").startswith("unverified") for s in ids):
            errors.append(f"{tag}: graded {c['Empirical evidence']} but every source is unverified")
        if c["Empirical evidence"] in ("strong", "moderate") and not any(
                kind.get(s) == "peer-reviewed" and src.get(s, "").startswith("resolved") for s in ids):
            errors.append(f"{tag}: graded {c['Empirical evidence']} without a resolved peer-reviewed source")
        for k, pat in OWNER_NEEDS.items():
            if re.search(pat, c["Need"], re.I):
                covered[k] += 1
    for k, v in covered.items():
        if not v:
            errors.append(f"owner need without a row: {k}")
    for k, v in topics.items():
        if not v:
            errors.append(f"required safety limit without a row tagged 'Safety:': {k}")
    return errors, covered, used


def summary(mrows, srows):
    ok_rows = [dict(zip(COLS, r)) for r in mrows if r and len(r) == len(COLS)]
    axes = set()
    for c in ok_rows:
        if c["Tier"] == "setting":
            for a in re.split(r"\s+x\s+", re.split(r"[:(]", c["Axis / configuration"])[0]):
                axes.add(a.strip())
    count = lambda f: {v: sum(1 for c in ok_rows if c[f] == v) for v in sorted({c[f] for c in ok_rows})}
    needs = []
    for c in ok_rows:
        if c["Need"] not in needs:
            needs.append(c["Need"])
    return dict(rows=len(mrows), sources=len(srows), needs=needs, axes=sorted(axes), tiers=count("Tier"),
                empirical=count("Empirical evidence"), user_test_needed=count("User test needed"))


def self_test(mrows, srows, root):
    """Each mutation of a valid matrix must produce at least one error."""
    m = copy.deepcopy(mrows)
    m[0][8] = "partial: no-such-file-a11y.css line 1"
    if not missing_files(m, root):
        return "missing named file was not detected"
    i = next(n for n, r in enumerate(mrows) if r and SAFETY.search(r[0] + r[2]))
    muts = [
        lambda m, s: m[0].__setitem__(3, "optional"),
        lambda m, s: m[0].__setitem__(6, "proven"),
        lambda m, s: m[0].__setitem__(7, m[0][7] + "; NO-SUCH-ID"),
        lambda m, s: m[0].__setitem__(10, ""),
        lambda m, s: m[i].__setitem__(3, "setting"),
        lambda m, s: m[0].__setitem__(8, "works"),
        lambda m, s: s[0].__setitem__(3, "looked up"),
        # strong or moderate backed only by a standard (WCAG22 is type standard in the sources table)
        lambda m, s: (m[0].__setitem__(6, "moderate"), m[0].__setitem__(7, "WCAG22")),
    ]
    for k, f in enumerate(muts):
        m, s = copy.deepcopy(mrows), copy.deepcopy(srows)
        f(m, s)
        if not validate(m, s)[0]:
            return f"mutation {k} was not detected"
    # every tagged safety row, flash-related or not (for example keyboard-filter safety), must fail when
    # demoted to a setting, and must fail when its 'Safety:' tag is removed
    marked = [n for n, r in enumerate(mrows) if r and len(r) == len(COLS) and SAFETY_MARK.match(r[2])]
    if not marked:
        return "no row tagged 'Safety:'"
    for n in marked:
        for field, value in ((3, "setting"), (2, SAFETY_MARK.sub("", mrows[n][2]).lstrip())):
            m = copy.deepcopy(mrows)
            m[n][field] = value
            if not validate(m, srows)[0]:
                return f"safety row {n + 1} ({'demoted' if field == 3 else 'untagged'}) was not detected"
    # every baseline row (mandatory access limits included) must fail when demoted to a setting
    for n, r in enumerate(mrows):
        if r and len(r) == len(COLS) and r[3] == "baseline":
            m = copy.deepcopy(mrows)
            m[n][3] = "setting"
            if not validate(m, srows)[0]:
                return f"baseline row {n + 1} demoted to setting was not detected"
    m = copy.deepcopy(mrows)
    for r in m:
        if r and re.search(OWNER_NEEDS["dyscalculia"], r[0], re.I):
            r[0] = "Removed need"
    if not validate(m, srows)[0]:
        return "removed owner need was not detected"
    return "passed"


def main():
    if len(sys.argv) != 2:
        print(json.dumps({"ok": False, "errors": ["usage: python3 -I check.py <repo-root>"]}))
        return 1
    path = Path(sys.argv[1]) / "docs/design/accessibility/ACCESSIBILITY-MATRIX.md"
    if not path.is_file():
        print(json.dumps({"ok": False, "errors": [f"missing {path}"]}))
        return 1
    lines = path.read_text(encoding="utf-8").splitlines()
    mrows, e1 = table(lines, MATRIX_HEADER)
    srows, e2 = table(lines, SOURCES_HEADER)
    errors = [e for e in (e1, e2) if e]
    if not errors:
        errs, covered, used = validate(mrows, srows)
        errors += errs + missing_files(mrows, Path(sys.argv[1]))
    out = summary(mrows, srows) if not (e1 or e2) else {}
    if not errors:
        out["self_test"] = self_test(mrows, srows, Path(sys.argv[1]))
        if out["self_test"] != "passed":
            errors.append("self-test failed: " + out["self_test"])
        out["owner_needs_rows"] = covered
        out["safety_rows"] = sum(1 for r in mrows if SAFETY_MARK.match(r[2]))
        text_only = {r[0] for r in srows if "research-text-only" in r[3]}
        out["research_text_only_sources"] = sorted(text_only)
        out["unused_sources"] = sorted({r[0] for r in srows} - used - text_only)
    out.update(ok=not errors, errors=errors)
    print(json.dumps(out, indent=1, ensure_ascii=False))
    return 0 if not errors else 1


if __name__ == "__main__":
    sys.exit(main())
