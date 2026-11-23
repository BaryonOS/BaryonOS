"""S2 maturity model helper (scratch evidence, not a gate).

gen   <repo> : replace the <!-- S6:BEGIN -->..<!-- S6:END --> block of
               docs/design/suite/MATURITY-MODEL.md with tables generated from
               ASSIGN below and the S1 names and evidence levels.
check <repo> : validate MATURITY-MODEL.md against S1 (names, evidence levels,
               exactly-once coverage, 14/202 split), the dimension x level
               cells, every S1 citation in a section 4 Basis cell against the
               section 6 level (repair round 1), the acceptance-criteria rows
               with full and partial coverage, the section 8.2 lists, internal
               cross-references and the source URLs.
Exit 1 on any error.
"""
import re
import sys

S1 = "docs/design/suite/CAPABILITY-MATRIX.md"
S2 = "docs/design/suite/MATURITY-MODEL.md"
LEVELS = ["L1", "L2", "L3", "L4", "L5"]
NAMES = {"L1": "First use", "L2": "Everyday", "L3": "Proficient", "L4": "Expert", "L5": "Specialist"}

# Repair round 1 changed GR18 Beginner path L3->L2, PD10 and GR31 Professional
# depth L5->L4 (delivery to an external specification is the section 3 L4
# signal) and XC07 Professional depth L3->L4.
# Judgement (S2 section 6): level from which the S1 row's Beginner path and
# Professional depth are part of the minimum. None = S1 Beginner path is "—".
ASSIGN = {
    "WP01": ("L1", "L1"), "WP02": ("L1", "L3"), "WP03": ("L2", "L4"), "WP04": ("L1", "L4"),
    "WP05": ("L1", "L4"), "WP06": ("L2", "L4"), "WP07": ("L2", "L3"), "WP08": ("L2", "L3"),
    "WP09": ("L2", "L4"), "WP10": ("L2", "L4"), "WP11": ("L3", "L4"), "WP12": ("L3", "L4"),
    "WP13": ("L3", "L4"), "WP14": ("L3", "L4"), "WP15": ("L2", "L3"), "WP16": ("L3", "L4"),
    "WP17": ("L1", "L3"), "WP18": ("L1", "L3"), "WP19": ("L1", "L4"), "WP20": ("L4", "L5"),
    "WP21": ("L1", "L3"), "WP22": ("L1", "L3"), "WP23": ("L1", "L3"), "WP24": ("L2", "L4"),
    "WP25": (None, "L4"), "WP26": ("L4", "L5"), "WP27": ("L1", "L4"), "WP28": ("L1", "L3"),
    "SS01": ("L1", "L3"), "SS02": ("L1", "L4"), "SS03": ("L2", "L4"), "SS04": ("L1", "L4"),
    "SS05": ("L1", "L3"), "SS06": ("L1", "L3"), "SS07": ("L2", "L3"), "SS08": ("L2", "L3"),
    "SS09": (None, "L4"), "SS10": ("L3", "L4"), "SS11": ("L2", "L4"), "SS12": ("L3", "L4"),
    "SS13": ("L3", "L5"), "SS14": ("L4", "L5"), "SS15": ("L1", "L4"), "SS16": ("L1", "L3"),
    "SS17": ("L2", "L4"), "SS18": ("L3", "L4"), "SS19": ("L1", "L4"), "SS20": ("L1", "L3"),
    "PR01": ("L1", "L3"), "PR02": ("L2", "L4"), "PR03": ("L1", "L4"), "PR04": ("L1", "L4"),
    "PR05": ("L1", "L3"), "PR06": ("L2", "L4"), "PR07": ("L2", "L4"), "PR08": ("L1", "L3"),
    "PR09": ("L2", "L4"), "PR10": ("L1", "L4"), "PR11": ("L1", "L4"), "PR12": ("L1", "L3"),
    "PR13": ("L2", "L4"),
    "NK01": ("L1", "L3"), "NK02": ("L1", "L3"), "NK03": ("L1", "L3"), "NK04": ("L2", "L4"),
    "NK05": ("L3", "L4"), "NK06": ("L2", "L4"), "NK07": ("L2", "L4"), "NK08": ("L2", "L4"),
    "NK09": ("L1", "L3"), "NK10": ("L2", "L4"), "NK11": ("L1", "L3"),
    "EM01": ("L1", "L4"), "EM02": ("L1", "L3"), "EM03": ("L1", "L3"), "EM04": ("L1", "L4"),
    "EM05": ("L1", "L3"), "EM06": ("L2", "L4"), "EM07": ("L1", "L3"), "EM08": ("L1", "L3"),
    "EM09": ("L1", "L3"), "EM10": ("L2", "L4"), "EM11": ("L3", "L4"),
    "DB01": ("L3", "L5"), "DB02": ("L2", "L4"), "DB03": ("L3", "L4"), "DB04": ("L1", "L3"),
    "DB05": ("L3", "L5"), "DB06": ("L3", "L5"), "DB07": ("L4", "L5"),
    "PD01": ("L1", "L4"), "PD02": ("L1", "L4"), "PD03": ("L1", "L3"), "PD04": ("L2", "L4"),
    "PD05": ("L2", "L4"), "PD06": ("L1", "L4"), "PD07": ("L2", "L4"), "PD08": ("L3", "L4"),
    "PD09": ("L2", "L4"), "PD10": ("L3", "L4"), "PD11": ("L3", "L4"),
    "GR01": ("L1", "L3"), "GR02": ("L1", "L3"), "GR03": ("L1", "L4"), "GR04": ("L2", "L4"),
    "GR05": ("L3", "L4"), "GR06": ("L3", "L4"), "GR07": ("L2", "L4"), "GR08": ("L1", "L3"),
    "GR09": ("L1", "L4"), "GR10": ("L1", "L4"), "GR11": ("L2", "L4"), "GR12": ("L1", "L3"),
    "GR13": (None, "L4"), "GR14": ("L1", "L4"), "GR15": ("L2", "L4"), "GR16": ("L2", "L4"),
    "GR17": ("L1", "L4"), "GR18": ("L2", "L5"), "GR19": ("L1", "L4"), "GR20": ("L1", "L3"),
    "GR21": ("L1", "L3"), "GR22": ("L1", "L4"), "GR23": ("L3", "L4"), "GR24": ("L2", "L4"),
    "GR25": ("L1", "L4"), "GR26": ("L1", "L4"), "GR27": ("L1", "L3"), "GR28": ("L1", "L3"),
    "GR29": ("L2", "L4"), "GR30": ("L2", "L4"), "GR31": ("L3", "L4"), "GR32": ("L2", "L4"),
    "GR33": ("L3", "L5"), "GR34": ("L1", "L4"), "GR35": ("L1", "L4"), "GR36": ("L3", "L4"),
    "GR37": ("L2", "L5"), "GR38": ("L1", "L4"),
    "VI01": ("L1", "L4"), "VI02": ("L1", "L3"), "VI03": ("L3", "L4"), "VI04": ("L1", "L3"),
    "VI05": ("L1", "L4"), "VI06": ("L1", "L4"), "VI07": ("L1", "L4"), "VI08": ("L2", "L4"),
    "VI09": ("L1", "L5"), "VI10": ("L1", "L4"), "VI11": ("L2", "L4"), "VI12": ("L2", "L5"),
    "VI13": ("L3", "L5"), "VI14": ("L2", "L5"), "VI15": ("L3", "L5"), "VI16": ("L3", "L5"),
    "VI17": ("L2", "L4"), "VI18": ("L3", "L4"), "VI19": ("L2", "L5"), "VI20": ("L1", "L4"),
    "VI21": ("L1", "L4"),
    "AU01": ("L1", "L3"), "AU02": ("L1", "L4"), "AU03": ("L1", "L4"), "AU04": ("L1", "L4"),
    "AU05": ("L2", "L4"), "AU06": ("L3", "L4"), "AU07": ("L3", "L5"), "AU08": ("L2", "L4"),
    "AU09": ("L3", "L5"), "AU10": ("L1", "L3"), "AU11": ("L3", "L5"), "AU12": ("L2", "L4"),
    "AU13": ("L2", "L4"), "AU14": ("L2", "L2"), "AU15": ("L1", "L4"), "AU16": ("L1", "L4"),
    "AU17": ("L1", "L4"), "AU18": ("L3", "L5"), "AU19": ("L3", "L4"),
    "UX01": ("L1", "L3"), "UX02": ("L2", "L4"), "UX03": ("L2", "L4"), "UX04": ("L2", "L4"),
    "UX05": ("L2", "L4"), "UX06": ("L3", "L4"), "UX07": ("L3", "L5"), "UX08": ("L1", "L3"),
    "UX09": ("L3", "L4"), "UX10": ("L2", "L4"), "UX11": ("L1", "L4"), "UX12": ("L2", "L5"),
    "TD01": ("L1", "L4"), "TD02": ("L1", "L3"), "TD03": ("L2", "L4"), "TD04": ("L1", "L4"),
    "TD05": ("L3", "L4"), "TD06": ("L1", "L4"), "TD07": ("L3", "L4"), "TD08": ("L1", "L4"),
    "TD09": ("L2", "L4"), "TD10": ("L3", "L4"), "TD11": ("L1", "L5"), "TD12": ("L2", "L4"),
    "TD13": ("L3", "L4"), "TD14": ("L2", "L4"), "TD15": ("L2", "L5"), "TD16": (None, "L4"),
    "XC01": ("L1", "L4"), "XC02": ("L1", "L3"), "XC03": ("L1", "L4"), "XC04": ("L1", "L4"),
    "XC05": ("L1", "L4"), "XC06": ("L1", "L1"), "XC07": ("L1", "L4"), "XC08": ("L2", "L4"),
    "XC09": ("L1", "L1"),
}

# Note appended to a row's Beginner-path cell in section 6 (repair round 1).
NOTES = {"XC09": " (read with ADR-DESIGN-OWNER-20261115 D2; section 6.1, rule 5)"}
PART = ("Beginner path", "Professional depth")
QUAL = re.compile(r"((?:[A-Z]{2}\d\d, )*[A-Z]{2}\d\d) \(([^)]*)\)")

DISCIPLINES = [  # (prefix, S2 subsection title, S1 section)
    ("WP", "Word processing, typography, references, revision tracking, publishing and complex layout", "3.1.1"),
    ("SS", "Spreadsheets", "3.1.2"),
    ("PR", "Presentations", "3.1.3"),
    ("NK", "Notebooks, knowledge management, diagrams, whiteboards and research organisation", "3.1.4"),
    ("EM", "Email, calendars, contacts, scheduling, tasks and collaboration", "3.1.5"),
    ("DB", "Databases, forms, project management and business workflows", "3.1.6"),
    ("PD", "PDF creation, annotation, editing, signing and export", "3.1.7"),
    ("GR", "Graphics, photography and publishing", "3.2"),
    ("VI", "Video, motion graphics and animation", "3.3"),
    ("AU", "Audio production", "3.4"),
    ("UX", "UI/UX and interactive design", "3.5"),
    ("TD", "3D and spatial creation", "3.6"),
    ("XC", "Cross-cutting suite capabilities", "3.7"),
]

ROW_HEADER = ["Capability (S1 row)", "Evidence level (S1, unchanged)",
              "Beginner path required from", "Professional depth required from"]
SUM_HEADER = ["Level", "Beginner path of", "Professional depth of"]
DIM_HEADER = ["Cell", "Level", "The user can", "The system must provide", "Basis"]
AC_HEADER = ["ID", "Level", "Scenario", "Measure and proposed threshold", "Method",
             "User test needed", "Cells and rules tested", "Status"]
OPENED = {  # URLs opened in this task, 18 November 2026
    "https://csse.canterbury.ac.nz/andrew.cockburn/papers/blur.pdf",
    "https://research.autodesk.com/app/uploads/2023/03/a-survey-of-software.pdf_recqWPTEYd6nsP0r8.pdf",
    "https://www.cs.ubc.ca/labs/edapt/papers/findlater2004.pdf",
    "https://www.dgp.toronto.edu/public_user/RMB/papers/p28.pdf",
    "https://www.cs.umd.edu/~ben/ACM-CUU2003.pdf",
    "https://www.nngroup.com/articles/progressive-disclosure/",
    "https://www.nngroup.com/articles/response-times-3-important-limits/",
    "https://www.w3.org/TR/WCAG22/",
    "https://www.w3.org/WAI/WCAG22/Understanding/error-prevention-legal-financial-data.html",
}


def cells(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]


def s1_rows(repo):
    out = {}
    for line in open(f"{repo}/{S1}", encoding="utf-8"):
        if re.match(r"^\| [A-Z]{2}\d\d ", line):
            c = cells(line)
            rid, name = c[0].split(" ", 1)
            out[rid] = (name, c[6])
    return out


def tables(text, header):
    """Yield (line_no, cells) for rows of every table whose header equals `header`."""
    on = False
    for n, line in enumerate(text.splitlines(), 1):
        if not line.startswith("|"):
            on = False
            continue
        c = cells(line)
        if c == header:
            on = True
            continue
        if on and not set(line.strip()) <= set("|-: "):
            yield n, c


def idx(level):
    return LEVELS.index(level)


def basis_check(text, err):
    """Every S1 row cited in a section 4 Basis cell carries a Beginner path or
    Professional depth tag; the tagged part is required by section 6 at or below
    the cell's level, and a part required earlier is marked 'from Lk'."""
    from_cells = set()
    for n, c in tables(text, DIM_HEADER):
        if len(c) != 5:
            continue
        cell, lvl, basis, tagged = c[0], c[1][:2], c[4], set()
        for m in QUAL.finditer(basis):
            q = m.group(2)
            parts = ((0, 1) if q.startswith("Beginner path and Professional depth") else
                     (0,) if q.startswith("Beginner path") else
                     (1,) if q.startswith("Professional depth") else ())
            if not parts:
                continue
            f = re.search(r"\bfrom (L[1-5])\b", q)
            for r in (x for x in m.group(1).split(", ") if x in ASSIGN):
                tagged.add(r)
                for p in parts:
                    need = ASSIGN[r][p]
                    if need is None:
                        err.append(f"{n}: {cell} cites {r} {PART[p]}, which S1 does not have")
                    elif idx(need) > idx(lvl):
                        err.append(f"{n}: {cell} requires {r} {PART[p]} at {lvl}; section 6 requires it from {need}")
                    elif idx(need) < idx(lvl):
                        if not f or f.group(1) != need:
                            err.append(f"{n}: {cell} cites {r} {PART[p]}, required from {need}: mark 'from {need}'")
                        else:
                            from_cells.add(cell)
                    elif f:
                        err.append(f"{n}: {cell} marks {r} {PART[p]} 'from' at its own level")
        for r in sorted(set(re.findall(r"\b[A-Z]{2}\d\d\b", basis)) & set(ASSIGN) - tagged):
            err.append(f"{n}: {cell} cites S1 {r} without a Beginner path or Professional depth tag")
    return sorted(from_cells)


def xref_check(text, ac_ids, allcells, err):
    """AC, cell, owner-decision and section 8.2 item references resolve."""
    for ref in sorted(set(re.findall(r"\bAC-[1-5X]\.\d+\b", text)) - ac_ids):
        err.append(f"dangling criterion reference {ref}")
    for ref in sorted(set(re.findall(r"\bD\d\d\.L\d\b", text)) - allcells):
        err.append(f"dangling cell reference {ref}")
    os_ = set(re.findall(r"\*\*(O\d+) ", text))
    for ref in sorted(set(re.findall(r"\bO\d+\b", text)) - os_):
        err.append(f"dangling owner-decision reference {ref}")
    s82 = text.split("### 8.2", 1)[-1].split("### 8.3", 1)[0]
    items = {int(x) for x in re.findall(r"^(\d+)\. ", s82, flags=re.M)}
    for k in sorted({int(x) for x in re.findall(r"section 8\.2,? items? (\d+)", text)} - items):
        err.append(f"dangling section 8.2 item {k}")
    for m in re.finditer(r"\bsection 8(?![.\d])", text):
        err.append(f"unspecific reference '{text[m.start():m.start() + 14]}': name 8.1 or 8.2")


def lv(x):
    return "—" if x is None else f"{x} {NAMES[x]}"


def gen(repo):
    s1 = s1_rows(repo)
    assert set(s1) == set(ASSIGN), sorted(set(s1) ^ set(ASSIGN))
    out = []
    for i, (p, title, sec) in enumerate(DISCIPLINES, 1):
        ids = [r for r in s1 if r.startswith(p)]
        out += [f"### 6.{i + 1} {title} (S1 §{sec})", "",
                "| " + " | ".join(ROW_HEADER) + " |", "|---|---|---|---|"]
        for r in ids:
            b, d = ASSIGN[r]
            assert b is None or LEVELS.index(b) <= LEVELS.index(d), r
            out.append(f"| {r} {s1[r][0]} | {s1[r][1]} | {lv(b)}{NOTES.get(r, '')} | {lv(d)} |")
        out += ["", "| " + " | ".join(SUM_HEADER) + " |", "|---|---|---|"]
        for L in LEVELS:
            bs = ", ".join(r for r in ids if ASSIGN[r][0] == L) or "—"
            ds = ", ".join(r for r in ids if ASSIGN[r][1] == L) or "—"
            out.append(f"| {L} {NAMES[L]} | {bs} | {ds} |")
        out.append("")
    text = open(f"{repo}/{S2}", encoding="utf-8").read()
    new = re.sub(r"<!-- S6:BEGIN -->.*<!-- S6:END -->",
                 "<!-- S6:BEGIN -->\n" + "\n".join(out) + "<!-- S6:END -->", text, flags=re.S)
    assert new != text or "<!-- S6:BEGIN -->" in text
    open(f"{repo}/{S2}", "w", encoding="utf-8").write(new)
    print(f"generated {len(s1)} rows in {len(DISCIPLINES)} disciplines")


def check(repo):
    s1 = s1_rows(repo)
    text = open(f"{repo}/{S2}", encoding="utf-8").read()
    err, seen = [], {}
    split = {"prototyped (host)": 0, "proposed": 0, "passed (native)": 0}
    for _, lvl in s1.values():
        split[lvl] += 1
    if split != {"prototyped (host)": 14, "proposed": 202, "passed (native)": 0}:
        err.append(f"S1 split changed: {split}")
    for n, c in tables(text, ROW_HEADER):
        rid, name = c[0].split(" ", 1)
        if rid in seen:
            err.append(f"{n}: {rid} twice")
        seen[rid] = (c[2], c[3])
        if rid not in s1:
            err.append(f"{n}: {rid} not in S1")
            continue
        if name != s1[rid][0]:
            err.append(f"{n}: {rid} name {name!r} != S1 {s1[rid][0]!r}")
        if c[1] != s1[rid][1]:
            err.append(f"{n}: {rid} level {c[1]!r} != S1 {s1[rid][1]!r}")
        b, d = (None if c[2] == "—" else c[2][:2]), c[3][:2]
        if (b, d) != ASSIGN.get(rid):
            err.append(f"{n}: {rid} levels {b},{d} != data {ASSIGN.get(rid)}")
    missing = set(s1) - set(seen)
    if missing:
        err.append(f"S1 rows missing from section 6: {sorted(missing)}")
    sums = list(tables(text, SUM_HEADER))
    if len(sums) != 5 * len(DISCIPLINES):
        err.append(f"{len(sums)} summary rows, expected {5 * len(DISCIPLINES)}")
    dims = {}
    for n, c in tables(text, DIM_HEADER):
        if len(c) != 5 or any(not x or x == "—" for x in c[:4]):
            err.append(f"{n}: dimension cell incomplete {c[:2]}")
            continue
        m = re.fullmatch(r"(D\d\d)\.(L\d)", c[0])
        if not m or m.group(2) != c[1][:2]:
            err.append(f"{n}: cell id {c[0]!r} vs level {c[1]!r}")
            continue
        dims.setdefault(m.group(1), set()).add(m.group(2))
    if len(dims) < 15 or any(v != set(LEVELS) for v in dims.values()):
        err.append(f"dimensions incomplete: {sorted((k, sorted(v)) for k, v in dims.items())}")
    from_cells = basis_check(text, err)
    acs = list(tables(text, AC_HEADER))
    allcells = {f"{d}.{L}" for d in dims for L in dims[d]}
    covered, part, ids, above = set(), set(), set(), set()
    for n, c in acs:
        if len(c) != 8 or any(not x for x in c) or c[7] != "proposed" or c[5] not in ("yes", "no"):
            err.append(f"{n}: acceptance row malformed {c[:1]}")
            continue
        if c[0] in ids:
            err.append(f"{n}: duplicate criterion {c[0]}")
        ids.add(c[0])
        toks = [x.strip() for x in c[6].split(",")]
        for x in toks:
            m = re.fullmatch(r"(D\d\d\.L\d)( \(part\))?", x)
            if m and m.group(1) in allcells:
                (part if m.group(2) else covered).add(m.group(1))
                own = c[1][:2] if c[1][:2] in LEVELS else "L1"  # cross-level criteria count as L1
                if idx(m.group(1)[-2:]) > idx(own):
                    above.add(f"{c[0]} {m.group(1)}")
            elif not (re.fullmatch(r"RULE-([1-9]|1[0-2])", x) or re.fullmatch(r"L[1-5] signal", x)):
                err.append(f"{n}: unknown cell, rule or signal {x!r}")
    mixed = sorted(covered & part)
    if mixed:  # section 7 rule: a cell is "(part)" in every criterion naming it, or in none
        err.append(f"cells marked both full and '(part)' in section 7: {mixed}")
    unc = sorted(allcells - covered - part)
    partial = sorted(part - covered)

    def listed(label):
        m = re.search(label + r": (.*?)\.(?:\s|$)", text, flags=re.S)
        return sorted(x.strip() for x in m.group(1).replace("\n", " ").split(",")) if m else None
    if listed("Cells without a criterion") != unc:
        err.append(f"section 8.2 uncovered list {listed('Cells without a criterion')} != computed {unc}")
    if listed("Cells tested only in part") != partial:
        err.append(f"section 8.2 partial list {listed('Cells tested only in part')} != computed {partial}")
    if listed(r"as criterion and cell") != sorted(above):  # repair round 2
        err.append(f"section 8.2 above-level list {listed('as criterion and cell')} != computed {sorted(above)}")
    if listed("Cells whose Basis marks an earlier requirement") != from_cells:
        err.append(f"section 8.2 'from' list {listed('Cells whose Basis marks an earlier requirement')} != computed {from_cells}")
    print(f"cells fully covered {len(covered)}/{len(allcells)}; partial only {len(partial)}; uncovered {len(unc)}")
    print(f"  uncovered: {', '.join(unc)}")
    print(f"  partial only: {', '.join(partial)}")
    print(f"  basis 'from' cells: {', '.join(from_cells)}")
    xref_check(text, ids, allcells, err)
    # repair round 2: progression rules are RULE-n; PRn outside S1 contexts is an old rule id
    body = re.sub(r"<!-- S6:BEGIN -->.*<!-- S6:END -->", "", text, flags=re.S)
    for line in body.splitlines():
        scope = " ".join(cells(line)[:4]) if line.startswith("| D") else line
        for m in re.finditer(r"\bPR([1-9]|1[0-2])\b", scope):
            err.append(f"rule identifier {m.group(0)} outside S1 context: use RULE-{m.group(1)}")
    per = {L: sum(1 for _, c in acs if c[1].startswith(L)) for L in LEVELS}
    if any(v == 0 for v in per.values()):
        err.append(f"a level has no acceptance criteria: {per}")
    src = text.split("## 9. Sources", 1)[-1]
    urls = set(re.findall(r"https?://[^\s|)>`]+", src))
    if urls - OPENED:
        err.append(f"section 9 cites URLs not opened: {sorted(urls - OPENED)}")
    print(f"S1 rows {len(s1)} split {split}; section 6 rows {len(seen)}; summary rows {len(sums)}")
    print(f"dimensions {len(dims)} x levels {sorted(set().union(*dims.values())) if dims else []}")
    print(f"acceptance criteria {len(acs)} per level {per}; section 9 URLs {len(urls)}")
    for e in err:
        print("ERROR", e)
    print("FAIL" if err else "PASS")
    return 1 if err else 0


if __name__ == "__main__":
    mode, repo = sys.argv[1], sys.argv[2]
    sys.exit(gen(repo) if mode == "gen" else check(repo))
