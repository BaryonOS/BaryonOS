# R02 full-window repaint tick split (diagnostic, 27 November 2026)

**Evidence level:** local diagnostic build, KVM, single guest CPU, 10 ms guest
ticks. This is not acceptance evidence and not tier 1 evidence (ADR-R02-L1-BUDGETS-20261127).
`diag-tick.patch` was applied only in a disposable checkout. It touches `kernel/` and
the window compositor only to export diagnostic snapshot words and is never
committed.

## Method

- Source: `main` `bbdbea88`. Arm D is `bbdbea88` with `diag-tick.patch`; arm M
  is clean `bbdbea88` (control arm for probe cost). Six window sessions were run interleaved
  (D1 M1 M2 D2 D3 M3) with `run.sh`:
  `NOS_QEMU_ACCEL=kvm python3 -B tools/run_window_session.py --clipboard --workspace --timeout 1800`
  (scale 1). Before each run the script recorded the host load and the QEMU process count (`meta.txt`, hashes in
  `SHA256SUMS`).
- Buckets are exhaustive: each mark charges the elapsed guest ticks to the
  bucket being left, so the buckets sum exactly to the frame wall. A frame
  counts as a full repaint when it blits at least 1000 tiles.
- The bucket-to-lever mapping was fixed before the first run
  (`preregistration.md`).

## Results (`summary.txt`)

| | D1 | D2 | D3 |
|---|---|---|---|
| Full repaints | 7 | 7 | 7 |
| Guest wall per full repaint | 489 ms | 480 ms | 473 ms |
| REPLY (client blocked in the SUR1 reply wait) | 65 % | 62 % | 63 % |
| SEND (7 sends per 1 KiB tile) | 12 % | 11 % | 8 % |
| RENDER (tile render and digest) | 7 % | 9 % | 6 % |
| INPUT (input and save step per loop) | 8 % | 7 % | 12 % |
| LOOP + TAIL + AWAY | 7 % | 11 % | 10 % |

- The guest wall of a full repaint (473–489 ms) covers the earlier host-side KVM
  proxy of 0.49–0.70 s, so the paint path is the right target.
- One reply per tile: replies = blits + RETAINs (11 104–11 215 per seven full
  repaints), about 0.19 ms reply wait per tile.
- All six runs: `raw_qemu_exit` 33, `passed` true.
- **Probe cost:** the D runs took 42.6–43.8 s (QEMU host CPU 29.5–30.4 s);
  the M runs took 36.3–37.4 s (23.8–24.7 s). The probes add about 16 % wall time
  and 23 % CPU. Each probe is a `monotonic_ticks` syscall and a scheduler
  switch point, so absolute bucket values are inflated. Shares are read only as
  aggregates.

## Interpretation under the pre-registration

- REPLY dominates. R's own levers are small: RENDER is 6–9 % and the loop or
  input remainder is about 15–20 %.
- The compositor attribution is **inconclusive**. Its span counter caught only
  one of the seven full repaints per run. Whether the reply wait is compositor
  work (U) or round trip and switch cost (U protocol note, K-11, K scheduler)
  stays open. Next diagnostic step: attribute compositor turn work by Material
  surface commits.

## Not run

- TCG, hosted runners, scale 2, SMP profiles and physical hardware.
- An un-instrumented per-bucket estimate.
- Guest timestamps finer than 10 ms (K-10).
