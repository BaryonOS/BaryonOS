Pre-registered before the first run (R, 2026-11-27 ~15:36Z UTC). Mapping from the dominant bucket over full-repaint frames (>= 1000 blits) to the lever and owner:
- RENDER (tile render + digest in the client) -> Materiaal render path (R write set, U review).
- SEND (7 sends per 1 KiB tile, CHUNK 208 = IPC limit) -> K-11 larger IPC messages / shared-memory tiles (K, after #602).
- REPLY, with compositor CF_INGRESS/CF_TOP/CF_TAIL dominant -> compositor commit/copy cost (U).
- REPLY, with compositor share small (scheduling, round trip per tile) -> SUR1 multi-tile reply protocol note (U approval) or the scheduler (K).
- AWAY/LOOP (yield, other processes, loop remainder) -> Materiaal loop budget, 4 paint steps per yield (R write set, U review).
First readout: does the guest full-frame wall (ticks x 10 ms) account for the 0.49-0.70 s host proxy? If not, the gap lies outside the paint path and the split is the wrong target.
Shares are read only as aggregates over >= ~1000 ticks; one tick = 10 ms. The control arm (clean main) bounds the probe cost (each probe is a syscall and a scheduler switch point).
