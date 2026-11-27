# R02 full-window repaint tick split (diagnostic, 27 November 2026)

**Evidence level:** local diagnostic build, KVM, single guest CPU, 10 ms guest
ticks. This is not acceptance evidence and not tier 1 evidence (ADR-R02-L1-BUDGETS-20261127).
`diag-tick.patch` is recorded here as a text artefact only. It was applied in a disposable
checkout and is never applied to a tracked source tree. It touches `kernel/` and the
window compositor only to export diagnostic snapshot words.

## Method

- Source: `main` `bbdbea88`. Arm D is `bbdbea88` with `diag-tick.patch`; arm M
  is clean `bbdbea88` (control arm for probe cost). Six window sessions were run interleaved
  (D1 M1 M2 D2 D3 M3) with `run.sh`:
  `NOS_QEMU_ACCEL=kvm python3 -B tools/run_window_session.py --clipboard --workspace --timeout 1800`
  (scale 1). Before each run the script recorded the host load and the QEMU process count (`meta.txt`, hashes in
  `SHA256SUMS`). The script has since been hardened for reruns; the exact
  originally used version is preserved at commit `3be6d3ef`. For a rerun,
  set `R02_OUTPUT_DIR` to a new path; existing evidence is never overwritten.
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

- The guest wall of a full repaint (473–489 ms) is an instrumented figure and
  does not establish an upper or lower bound for uninstrumented wall time. It is consistent with the earlier un-instrumented host
  proxy of 0.49–0.70 s, but does not yet show that the paint path accounts for
  that proxy.
- One reply per tile: replies = blits + RETAINs (11 104–11 215 per seven full
  repaints), about 0.19 ms instrumented reply wait per tile.
- All six runs: `raw_qemu_exit` 33, `passed` true. The recorded process
  count at the start of D3 was two, violating the exclusive-QEMU condition.
  D3 is retained as a functional diagnostic observation, not a valid exclusive
  performance comparison. The other five starts recorded zero QEMU processes.
- **Whole-run comparison, including non-exclusive D3:** the D runs took 42.6–43.8 s (QEMU host CPU 29.5–30.4 s);
  the M runs took 36.3–37.4 s (23.8–24.7 s). The observed differences are about 16 % wall time
  and 23 % CPU; these are not isolated per-frame or per-bucket costs. Each probe is a `monotonic_ticks` syscall and a scheduler
  switch point. The compositor probes run while the client waits for its reply,
  so their cost lands in REPLY. Probes also add cost to other buckets, so the
  uninstrumented REPLY share is unknown. The control arm compares whole runs,
  not per-bucket overhead.

## Host observation of repaint-related phases

These values are `last_input_to_verified_app_image_s` from the six hashed
reports, in seconds. Diagnostic frame counters identify these five phases as
containing a newly completed full repaint in D1. They are host-side proxy
intervals, not guest repaint durations; capture, input and the approximately
0.2 s observation grid contribute. D3 is non-exclusive.

| Phase | D1 | D2 | D3 | M1 | M2 | M3 |
|---|---:|---:|---:|---:|---:|---:|
| editor | 0.455 | 0.452 | 0.431 | 0.401 | 0.400 | 0.401 |
| list | 0.431 | 0.408 | 0.430 | 0.366 | 0.416 | 0.365 |
| reopened | 0.457 | 0.463 | 0.452 | 0.418 | 0.395 | 0.397 |
| document-menu-high-contrast | 0.711 | 0.712 | 0.685 | 0.696 | 0.489 | 0.492 |
| document-menu-contrast-restored | 0.689 | 0.714 | 0.710 | 0.683 | 0.490 | 0.493 |

The comparisons are too coarse and few to bound instrumentation overhead for
individual buckets or establish its sign. The instrumented guest-wall figure
cannot establish that the uninstrumented repaint accounts for the host proxy.

## Interpretation under the pre-registration

- REPLY is the largest instrumented bucket; its uninstrumented share is unknown. In the client, RENDER
  is 6–9 % and the loop or input remainder is about 15–20 %. The round-trip
  lever includes `ui/surface-protocol/` (R write set, U approval for any protocol
  revision), K-11 and the per-switch cost (K).
- The compositor attribution is **inconclusive**. Its span counter caught only
  one of the seven full repaints per run. Whether the reply wait is compositor
  work (U) or round trip and switch cost (U protocol note, K-11, K scheduler)
  stays open. The counter reported exactly 1221 commits in all three runs, which
  points to its own logic rather than sampling. Next steps: compare host phase
  times of the full-repaint phases between the D and M reports, run a client-only
  probe arm, and attribute compositor turn work by Material surface commits.

## Not run

- TCG, hosted runners, scale 2, SMP profiles and physical hardware.
- An un-instrumented per-bucket estimate.
- Guest timestamps finer than 10 ms (K-10).
