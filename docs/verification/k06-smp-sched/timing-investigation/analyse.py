#!/usr/bin/env python3
"""Summarise timing.py output: per-phase distribution and vCPU scheduling."""
import json, statistics as st, sys

PH = {"sync_total": "SMP:SERIAL:PASS", "sync_wait": "DIAG:SYNC_WAIT_AFTER_BSP",
      "scratch": "SMP:SCRATCH:PASS", "wake": "SMP:WAKE:PASS", "lockpair": "SMP:LOCKPAIR:PASS",
      "icr": "SMP:ICR:PASS", "isolation": "SMP:ISOLATION:PASS", "preempt": "SMP:PREEMPT:PASS"}


def q(v, p):
    v = sorted(v)
    return v[min(len(v) - 1, int(p * len(v)))]


for path in sys.argv[1:]:
    with open(path) as source:
        rows = [json.loads(l) for l in source]
    if not rows:
        continue
    r0 = rows[0]
    fails = [(r["i"], r["exit"], [m[0] for m in r["marks"] if "FAIL" in m[0] or "TIMEOUT" in m[0]]) for r in rows if r["exit"] != 33]
    print(f"\n== {path.split('/')[-1]}: vCPUs={r0['cpus']} host={r0['hostcpus']} accel={r0['accel']} boots={len(rows)} non-pass={len(fails)} {fails[:4]}")
    print("phase         n    min    med    p90    max   (s)")
    for name, key in PH.items():
        v = [m[2] for r in rows for m in r["marks"] if m[0].startswith(key)]
        if v:
            print(f"{name:11s} {len(v):3d} {min(v):6.3f} {st.median(v):6.3f} {q(v,.9):6.3f} {max(v):6.3f}")
    clk = [int(m[0].split("=")[1].split(":")[0]) for r in rows for m in r["marks"] if m[0].startswith("SMP:DIAG:CLOCK_US")]
    if clk:
        dev = [abs(c - 999847) / 9998.47 for c in clk]
        print(f"clock median dev %: med {st.median(dev):.3f} max {max(dev):.3f} over1%={sum(d > 1 for d in dev)}")
    wall = [r["wall"] for r in rows]
    novc = sum(not r["vcpu"] for r in rows)
    rows = [r for r in rows if r["vcpu"]] or rows
    run = [sum(t[1] for t in r["vcpu"]) / 1e9 for r in rows]
    wait = [sum(t[2] for t in r["vcpu"]) / 1e9 for r in rows]
    inv = [sum(t[4] for t in r["vcpu"]) for r in rows]
    steal = [100 * r["steal_ticks"] / max(1, r["total_ticks"]) for r in rows]
    print(f"wall med {st.median(wall):.1f}s max {max(wall):.1f}s | vCPU run med {st.median(run):.1f}s, runqueue wait med {st.median(wait):.1f}s "
          f"(wait/run {st.median(wait)/max(1e-9,st.median(run)):.2f}) | involuntary switches med {int(st.median(inv))} | host steal med {st.median(steal):.2f}% max {max(steal):.2f}% | boots without vCPU sample {novc}")
