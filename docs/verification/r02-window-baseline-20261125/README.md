# R02 local KVM baseline: native window session (25 November 2026)

Diagnostic baseline for R02 (long-term stability and performance budgets).
It is **local KVM evidence**, not hosted CI acceptance, physical-hardware
evidence or a product-latency claim. No criterion or progress point changes.

## Configuration

- Source: `origin/main` `5846e1fb3da539de8670fce00c12241a3b2f8eb8`, branch
  `claude/r-track-20261125`; profile `native-window-session-probe`.
- Commands: [`baseline.sh`](baseline.sh) (one build, then five sequential
  `tools/run_window_session.py --clipboard --timeout 1800` runs with SeaBIOS
  ROMs, `NOS_QEMU_ACCEL=kvm`).
- QEMU 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3), `-cpu qemu64 -smp 1 -m 256M`
  as fixed by the runner; host with 32 logical CPUs, shared with other
  sessions (load average per run in [`meta.txt`](meta.txt), 2.3–4.3).
- Raw reports and serial logs stay local (≈158 KiB per report); their SHA-256
  values are in [`report-sha256.txt`](report-sha256.txt). The full derived
  table is [`summary.txt`](summary.txt).

## Results (five runs)

- 5/5 passed, `raw_qemu_exit` 33; duration 26.85–27.85 s (median 26.93 s).
- QEMU host CPU 24.3–24.9 s per run, 0.88–0.91 of wall time. This is
  host-process accounting for the whole QEMU process, not guest vCPU
  utilisation; guest utilisation and idle behaviour were not measured.
- `last_input_to_verified_app_image_s` clusters at about 0.05, 0.13, 0.22–0.29,
  0.43–0.46, 0.66, 0.84–0.92 and 1.0–1.15 s. The harness retries a
  mismatching screendump every 0.2 s (`run_window_session.py` `wait`), so
  these values are quantised upper bounds; the tight spread reflects the grid,
  not low guest variance. The measured interval also includes QMP, screendump
  and oracle time on the host.
- `COMPOSITOR:COMMITS` deltas between consecutive serial snapshots show
  14–15 full-surface repaints per run, median 1780–1786 committed SUR1 tiles
  each, besides smaller partial updates. The 14 phases near 0.85–0.92 s
  match this count.

## Interpretation and limits

- The dominant interactive cost in this profile is a full app-body repaint
  after most view changes: about 1785 tiles of 32×8 pixels, each sent as
  BEGIN, five CHUNKs and COMMIT with one acknowledgement. The phases that
  coincide with these repaints were observed at 0.84–0.92 s from last input
  to the first matching screendump. That interval ends on an application-
  state match, not on the repaint's last tile commit, so it does not bound
  the per-tile cost; no per-tile figure is derived. The ranking of
  changed-region repaint in the client paint path (one tile per
  `paint_step`, restart at tile zero on a new view generation) above further
  compositor batching (8 messages per turn since `380111b6`) rests on the
  tile counts: a one-character change re-sends ≈1785 tiles, whereas batching
  only changes how existing messages are drained.
- The rendering section of `docs/implementation/VM-HARDWARE-AND-PERFORMANCE.md`
  predates that batching and the retained-tile operation and is stale on
  those points.
- Not measured: guest-side timestamps, scanout latency, first-feedback
  latency, SMP profiles, TCG, WHPX, physical hardware and energy. These
  values cannot be compared with candidate 50 ms or 100 ms first-feedback
  targets; no budget is proposed as binding here (R02-L1 remains an owner
  decision).
