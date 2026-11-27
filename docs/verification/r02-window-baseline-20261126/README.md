# R02 local KVM re-baseline after #540: native window session (26 November 2026)

Diagnostic baseline for R02 (long-term stability and performance budgets),
repeated on `main` after the Materiaal changed-region repaint (#540). It is
**local KVM evidence**, not hosted CI acceptance, physical-hardware evidence or
a product-latency claim. No criterion or progress point changes.

## Configuration

- Source: `origin/main` `ffea3883870d0aa0f26170d8676015ae31067f53`, branch
  `claude/r02-compositor-turn-20261126`; profile `native-window-session-probe`.
- Commands: [`baseline.sh`](baseline.sh) (one build, then five sequential
  `tools/run_window_session.py --clipboard --timeout 1800` runs with SeaBIOS
  ROMs, `NOS_QEMU_ACCEL=kvm`), the same driver as
  [`../r02-window-baseline-20261125/`](../r02-window-baseline-20261125/).
- QEMU 10.2.1, `-cpu qemu64 -smp 1 -m 256M` as fixed by the runner; shared
  host, load average per run in [`meta.txt`](meta.txt) (3.2–3.5).
- Raw reports and serial logs stay local; their SHA-256 values are in
  [`report-sha256.txt`](report-sha256.txt). The derived table is
  [`summary.txt`](summary.txt).

## Results (five runs)

- 5/5 passed, `raw_qemu_exit` 33; duration 19.29–19.78 s (median 19.50 s),
  against 26.85–27.85 s (median 26.93 s) on `5846e1fb` before #540.
- QEMU host CPU 13.8–14.2 s per run, 0.71–0.72 of wall time (host-process
  accounting, not guest vCPU utilisation).
- `last_input_to_verified_app_image_s` (host-side, 0.2 s screendump retry
  grid): the three slowest phases are whole-screen changes, `reopened` 0.70 s,
  `editor` 0.68 s and `list` 0.65 s (medians). All other phases are at
  0.30 s or below. Before #540 many phases took 0.84–0.92 s.

## Per-tile present path (source inspection at `ffea3883`)

Verified in source:

- This profile's compositor is `window-compositor`
  (`sdk/examples/window-session/src/main.rs`, embedded by
  `kernel/src/m2/m3/window_session.rs`), not `sdk/examples/compositor`.
- The SUR1 client keeps one tile in flight: a full tile is BEGIN, five CHUNKs
  and COMMIT, then it waits for the reply (`ui/surface-protocol/src/client.rs`,
  `blit`). The wait in `receive_reply` loops receive, `EAGAIN`, `yield_now`
  up to the poll budget; it does not block.
- Without `native-smp-m3`, every kernel entry ends in `select()`, which starts
  at `current + 1` with `continuing = false` (`kernel/src/m2/m3.rs`), so each
  system call (send, receive, clock read, blit) is a switch point, not only
  `yield_now`.
- A blocking primitive exists: `ipc_wait` (`sdk/runtime/src/native.rs`,
  `kernel/src/m2/m3/wait.rs`); the window compositor already uses it when
  quiescent (`sdk/examples/window-session/src/idle.rs`).

Estimates (not measured): with Materiaal and the compositor alternating on
every system call, a full tile costs about 40–50 kernel entries and context
switches. 0.63–0.70 s for about 1785 changed tiles is about 0.35–0.39 ms per
tile, which is consistent with that model but does not prove it.

## Candidate next steps (proposals, not implemented)

1. Client blocks for the SUR1 reply with `ipc_wait` instead of spinning
   (`ui/surface-protocol` client and runtime transport; ABI-gated; the poll
   budget, deadlines and delivery reporting must stay).
2. Compositor reads the clock once per turn instead of per message
   (`sdk/examples/window-session/src/main.rs`; no ABI change; SUR1 deadlines
   still need a fresh enough time).
3. Kernel: on the single-CPU path, let the caller continue after ordinary
   system calls and switch on yield, wait, exit or the timer quantum
   (`kernel/src/m2/m3.rs`). Largest expected effect; touches scheduling
   fairness and existing round-robin assertions, so it needs the kernel
   owner's agreement and an explicit decision.

Several tiles in flight is lower value: it conflicts with the compositor's
drop-while-reply-pending rule, single-tile staging and the shared 16-message
ingress queue, and would need a protocol revision.

## Not run

- Guest-side or scanout timestamps, per-entry switch counts, TCG, WHPX, SMP
  profiles, physical hardware. No candidate budget is evaluated (R02-L1 is
  still open).
