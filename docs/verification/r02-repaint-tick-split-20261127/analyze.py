#!/usr/bin/env python3
"""Summarise the R02 tick-split diagnostic (local, KVM, 10 ms ticks)."""
import json, re, sys
from pathlib import Path
O = Path.home() / "projects/BaryonOS-scratch/r/diag-tick/out"
B = ["AWAY", "LOOP", "INPUT", "RENDER", "SEND", "REPLY", "TAIL"]
def last(raw, prefix):
    blocks = re.findall(re.escape(prefix + "SNAPSHOT:BEGIN\n") + r"(.*?)" + re.escape(prefix + "SNAPSHOT:END\n"), raw, re.S)
    for body in reversed(blocks):
        d = {m[1]: int(m[2]) for m in re.finditer(re.escape(prefix) + r"([A-Z_]+):([0-9]+)", body)}
        if any(k.startswith("DIAG") for k in d):
            return d
    return {}
meta = (O / "meta.txt").read_text()
rows = []
for run in ["D1", "M1", "M2", "D2", "D3", "M3"]:
    rep = O / f"report-{run}.json"
    if not rep.exists():
        continue
    r = json.loads(rep.read_text())
    exit_line = re.search(rf"{run} exit (\d+)", meta)
    row = {"run": run, "exit": exit_line[1] if exit_line else "?", "passed": r.get("passed"), "raw": r.get("raw_qemu_exit"),
           "duration": round(r.get("duration_s", 0), 2), "cpu": round(sum(r.get("qemu_host_cpu", {}).get(k, 0) for k in ("user_seconds", "system_seconds")), 2)}
    rd = re.search(rf"{run} rundir (\S+)", meta)
    serial = None
    if rd:
        for p in Path(rd[1]).rglob("*"):
            if p.is_file() and p.suffix in (".log", ".txt") and b"WINDOW:MATERIAL:" in p.read_bytes()[:10_000_000]:
                serial = p; break
    if serial and run.startswith("D"):
        raw = serial.read_text("utf-8", "replace")
        m, c = last(raw, "WINDOW:MATERIAL:"), last(raw, "WINDOW:COMPOSITOR:")
        row["mat"], row["comp"] = m, c
    rows.append(row)
for row in rows:
    print(row["run"], "exit", row["exit"], "raw", row["raw"], "passed", row["passed"], "duration", row["duration"], "qemu_cpu", row["cpu"])
    m = row.get("mat")
    if m:
        n, wall = m["DIAG_F_COUNT"], m["DIAG_F_WALL"]
        print(f"  full frames {n}, wall {wall} ticks = {wall*10/max(n,1):.0f} ms/frame; blits {m['DIAG_F_BLITS']}, rendered {m['DIAG_F_RENDERED']}, retains {m['DIAG_F_RETAINS']}, replies {m['DIAG_F_REPLIES']}, loops {m['DIAG_F_LOOPS']}")
        tot = sum(m[f"DIAG_F_{b}"] for b in B)
        print("  frame buckets:", ", ".join(f"{b} {m[f'DIAG_F_{b}']} ({100*m[f'DIAG_F_{b}']/max(tot,1):.0f}%)" for b in B), f"sum {tot} vs wall {wall}")
        print("  run totals:", ", ".join(f"{b} {m[f'DIAG_T_{b}']}" for b in B))
    c = row.get("comp")
    if c:
        print(f"  compositor full spans {c['DIAG_CF_COUNT']}: away {c['DIAG_CF_AWAY']} top {c['DIAG_CF_TOP']} ingress {c['DIAG_CF_INGRESS']} tail {c['DIAG_CF_TAIL']} ticks; commits {c['DIAG_CF_COMMITS']} turns {c['DIAG_CF_TURNS']}")
        print(f"  compositor run totals: away {c['DIAG_CT_AWAY']} top {c['DIAG_CT_TOP']} ingress {c['DIAG_CT_INGRESS']} tail {c['DIAG_CT_TAIL']} turns {c['DIAG_CT_TURNS']}")
