"""Summarise the R02 (1) A/B: per-phase median and range, acceptance fields, pixel identity."""
import json, statistics, sys, os
from pathlib import Path
d = sys.argv[1] if len(sys.argv) > 1 else '.'
runs = {s: [json.loads(Path(d, f'report-{s}{i}.json').read_text()) for i in range(1, 6)] for s in 'AB'}
out = []
for s, rs in runs.items():
    out.append(f"{s} commit {rs[0]['source_commit'][:8]} kernel.elf {rs[0]['immutable']['esp/kernel.elf'][:16]}")
    out.append(f"{s} raw_qemu_exit {[r['raw_qemu_exit'] for r in rs]} passed {[r['passed'] for r in rs]}")
    du = [round(r['duration_s'], 2) for r in rs]
    cpu = [round(r['qemu_host_cpu']['user_seconds'] + r['qemu_host_cpu']['system_seconds'], 2) for r in rs]
    out.append(f"{s} duration_s {du} median {statistics.median(du)}; qemu_host_cpu_s {cpu} median {statistics.median(cpu)}")
key = 'last_input_to_verified_app_image_s'
phases = [p for p in runs['A'][0]['phases'] if key in runs['A'][0]['phases'][p]]
def stat(s, p):
    v = [r['phases'][p][key] for r in runs[s]]
    return statistics.median(v), min(v), max(v)
rows = sorted(phases, key=lambda p: -stat('A', p)[0])
out.append(f"phase: A median (min-max) | B median (min-max) | B-A median, {key}")
for p in rows:
    a, b = stat('A', p), stat('B', p)
    out.append(f"{p}: {a[0]:.2f} ({a[1]:.2f}-{a[2]:.2f}) | {b[0]:.2f} ({b[1]:.2f}-{b[2]:.2f}) | {b[0]-a[0]:+.2f}")
# Recovery path (retire must wake the waiting client): elapsed-time deltas between phases without the latency key.
for p0, p1 in (('calculated', 'recovered'), ('workspace-arranged', 'recovered'), ('recovered', 'calculator-restored')):
    dt = {s: [r['phases'][p1]['seconds'] - r['phases'][p0]['seconds'] for r in runs[s]] for s in 'AB'}
    out.append(f"delta {p0}->{p1} s: " + ' | '.join(f"{s} {statistics.median(v):.2f} ({min(v):.2f}-{max(v):.2f})" for s, v in dt.items()))
# Pixel identity: every phase frame across all ten runs, plus oracle files and paint-isolation baseline.
mismatch = []
for p in runs['A'][0]['phases']:
    shas = {r['phases'][p].get('frame_sha256') for s in 'AB' for r in runs[s]}
    if len(shas) != 1:
        mismatch.append(p)
oracles = {json.dumps(r['oracle']['files_sha256'], sort_keys=True) for s in 'AB' for r in runs[s]}
paint = {json.dumps(r.get('paint_isolation_baseline'), sort_keys=True) for s in 'AB' for r in runs[s]}
out.append(f"frame_sha256 phases compared {len(runs['A'][0]['phases'])}, differing {mismatch or 'none'}")
out.append(f"oracle files_sha256 distinct sets {len(oracles)}; paint_isolation_baseline distinct {len(paint)}")
# Frames that differ: split each capture into the content above y=650 and the taskbar band below it
# (local screendumps from run_directory; e483-style prefixes are only labels for comparison).
import hashlib
def ppm(path):
    b = Path(path).read_bytes(); parts = b.split(maxsplit=4); return int(parts[1]), parts[4]
for p in mismatch:
    cells = []
    for s in 'AB':
        for i, r in enumerate(runs[s], 1):
            w, px = ppm(os.path.join(r['run_directory'], p + '.ppm')); cut = 650 * w * 3
            cells.append(f"{s}{i}:{hashlib.sha256(px[cut:]).hexdigest()[:4]}/{hashlib.sha256(px[:cut]).hexdigest()[:4]}")
    out.append(f"split {p} taskbar/content: " + ' '.join(cells))
# High contrast (--clipboard --workspace), if present.
ws = {k: json.loads(Path(f).read_text()) for k in ('A1', 'B1', 'B2', 'A2') for f in [os.path.join(d, f'ws-report-{k}.json')] if os.path.exists(f)}
if ws:
    out.append("workspace runs " + ' '.join(f"{k}: raw {r['raw_qemu_exit']} passed {r['passed']} hc_gate {r['app_high_contrast_gate']} dur {r['duration_s']:.2f}" for k, r in ws.items()))
    hc = [p for p in next(iter(ws.values()))['phases'] if 'contrast' in p]
    for p in hc:
        out.append(f"hc {p}: " + ' '.join(f"{k}:{(r['phases'][p].get('frame_sha256') or 'None')[:8]}" for k, r in ws.items()))
    allp = next(iter(ws.values()))['phases']
    diff = [p for p in allp if len({r['phases'][p].get('frame_sha256') for r in ws.values()}) != 1]
    out.append(f"workspace frame_sha256 phases {len(allp)}, differing {diff or 'none'}")
print('\n'.join(out))
