"""For a serial log: Calculator EVALUATIONS history, then the compositor snapshots around the post-recovery CALCULATOR=0 -> 21 transition with COMMITS and the changed activity fields."""
import sys, re
lines = open(sys.argv[1], errors="replace").read().splitlines()
blocks, cur = [], None
for i, l in enumerate(lines):
    m = re.match(r"WINDOW:(\w+):SNAPSHOT:BEGIN", l)
    if m: cur = (m.group(1), i, {}); continue
    if cur:
        if l == "WINDOW:%s:SNAPSHOT:END" % cur[0]: blocks.append(cur); cur = None; continue
        m = re.match(r"WINDOW:%s:(\w+):(\S+)" % cur[0], l)
        if m: cur[2][m.group(1)] = int(m.group(2))
calc = [(i, f["SERVICE_GENERATION"], f["GEN"], f["EVALUATIONS"]) for r, i, f in blocks if r == "CALCULATOR"]
print("calc (line, svc, gen, evals):", calc)
comp = [(i, f) for r, i, f in blocks if r == "COMPOSITOR" and f.get("GENERATION") == 2]
act = ["INPUT","REJECTED","FOCUSED","MOVES","SWITCHES","POINTER_X","POINTER_Y","BUTTONS","SHELL","SHELL_READY"]
z = [n for n, (i, f) in enumerate(comp) if f["CALCULATOR"] == 0]
for n in range(max(z[0]-1, 0) if z else 0, min(len(comp), (z[-1] + 3) if z else 0)):
    i, f = comp[n]; p = comp[n-1][1] if n else f
    print(i, "COMMITS=%d (+%d) CALC=%d MAT=%d changed=%s" % (f["COMMITS"], f["COMMITS"]-p["COMMITS"], f["CALCULATOR"], f["MATERIAL"], [k for k in act if f.get(k) != p.get(k)]))
