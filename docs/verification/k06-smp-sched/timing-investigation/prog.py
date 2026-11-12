#!/usr/bin/env python3
"""Progress analysis: SMP:DIAG:P=<total> lines printed every 50 guest polls."""
import json, sys, statistics as st
for path in sys.argv[1:]:
    with open(path) as source:
        rows = [json.loads(l) for l in source]
    if not rows: continue
    print(f"\n== {path.split('/')[-1]} vCPUs={rows[0]['cpus']} host={rows[0]['hostcpus']} accel={rows[0]['accel']} boots={len(rows)}")
    for r in rows:
        p = [(m[1], int(m[0].split('=')[1])) for m in r["marks"] if m[0].startswith("SMP:DIAG:P=")]
        t_ratio = next((m[1] for m in r["marks"] if m[0].startswith("SMP:TIMER:RATIO")), None)
        t_end = next((m[1] for m in r["marks"] if m[0].startswith("SMP:SERIAL:PASS")), None)
        # longest host-time span over which the counter did not change
        stall, start = 0.0, 0
        final = max((v for _, v in p), default=0)
        tail = (p[-1][0] - min(t for t, v in p if v == final)) if p else 0
        p_all, p = p, [x for x in p if x[1] < final]
        for i in range(1, len(p)):
            if p[i][1] != p[start][1]:
                start = i
            else:
                stall = max(stall, p[i][0] - p[start][0])
        gaps = [b[0] - a[0] for a, b in zip(p, p[1:])]
        rates = [(b[1] - a[1]) / (b[0] - a[0]) for a, b in zip(p, p[1:]) if b[0] > a[0]]
        fail = [m[0] for m in r["marks"] if "FAIL" in m[0]]
        run = sum(t[1] for t in r["vcpu"]) / 1e9; wait = sum(t[2] for t in r["vcpu"]) / 1e9
        dur = (t_end - t_ratio) if t_end and t_ratio else float('nan')
        print(f" boot {r['i']:2d} exit {r['exit']:3d} sync {dur:7.2f}s lines {len(p):4d} final {final:6d} tail-after-count {tail:4.2f}s "
              f"longest no-progress {stall:5.2f}s max line gap {max(gaps, default=0):5.2f}s "
              f"rate/s min {min(rates, default=0):7.0f} med {st.median(rates) if rates else 0:7.0f} max {max(rates, default=0):7.0f} "
              f"| vCPU run {run:5.1f}s wait {wait:5.1f}s {fail}")
