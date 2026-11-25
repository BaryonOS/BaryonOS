# Specialist R — current handoff

Compact resume pointer for the R workstream (Reliability, Performance and
Release). It is not a backlog: criteria live in `docs/progress.json` (R01–R03),
`docs/FULL-TEST-RELEASE.md` (FR-R02) and GitHub.

## Scope and mandate

- **Scope:** R01 installation and accessible recovery, R02 long-term stability
  and performance budgets, R03 reproducible release and maintenance; system-wide
  performance methodology. R does not own compositor (U), drivers (H),
  storage (S), network (N), security (B) or SDK/apps (A) source: R delivers
  measured bottleneck analyses and agrees one writer with the owner.
- **Merge:** draft PRs only until the R merge delegation is durably recorded on
  `main` (expected after CI PR #519). No force-push, admin bypass, direct `main`
  push or weakened test.
- **Budgets:** candidate values only. R02-L1 requires a project-owner decision
  before any numeric budget becomes binding; budgets are never relaxed after a
  failed measurement.

## Current cycle

- **cycle_id:** R-20261125-1. Session "BaryonOS - Specialist R" [162009],
  session id `local_42f1c599-7345-4b4f-9270-7850f491fca1`, registered with
  "BaryonOS - PR Bewaker" (confirmed). Merge hold applies until #519 lands and
  the CI governance PR records the A/P/L/R delegation after owner review.
  Before committing in `sdk/examples/compositor` agree the writer with U
  (`local_f190cfa9`, open PR #523); report any `kernel/` write set to the PR
  guard in advance (S and H currently write `volume.rs`).
- **Branch/worktree:** `claude/r-track-20261125` at
  `~/projects/BaryonOS-worktrees/r-track`, base `origin/main` `5846e1fb`.
- **Subtask:** reproducible local KVM baseline of the native window session
  (build `native-window-session-probe`, `tools/run_window_session.py
  --clipboard`, five runs), to locate the dominant interactive-latency cost.
- **Driver:** `~/projects/BaryonOS-scratch/r/baseline.sh`; raw output in
  `~/projects/BaryonOS-scratch/r/baseline2/` (meta.txt with load average per
  run, build.log, run<N>.log, report<N>.json). Build dirs must stay inside the
  checkout (`build/r`, `build/r-target`): `run_fpu.private_path` rejects a
  `NOS_BUILD_DIR` outside it.

## Verified findings (source inspection, 25 Nov 2026, `5846e1fb`)

- `docs/implementation/VM-HARDWARE-AND-PERFORMANCE.md` is stale on rendering:
  the bounded SUR1 batch it calls "only a proposal" exists since `380111b6`
  (2 Nov): `sdk/examples/compositor/src/main.rs` drains up to 8 messages per
  turn; `Operation::Retain` (retained tiles) exists in `ui/surface-protocol`;
  the kernel has SMP (`kernel/src/m2/smp.rs`, `tools/run_smp.py`).
- Still open on the rendering path (to be ranked by measurement): one tile
  per client `paint_step`, one acknowledgement per tile, unconditional
  `yield_now` at the end of every compositor turn.
- `run_window_session.py` phase times are host/QMP/screendump intervals with
  0.2 s polling, not guest or scanout latency; pair them with the guest
  counters the compositor exports.
- SMP deadlines: ADR-082 is a proposal; do not change any SMP deadline
  without an explicit owner decision.

## Baseline result

Five KVM runs passed; evidence in
`docs/verification/r02-window-baseline-20261125/`. Dominant cost: 14–15
full app-body repaints (~1785 SUR1 tiles each) per run, ≤0.87 s each on
`-smp 1`; guest vCPU busy ~90% of wall time (no idle). The runner's 0.2 s
screendump retry quantises latency; exact timing needs per-attempt
timestamps (local diagnostic patch only, not committed).

## Next action

Agree one writer with U for changed-region repaint in the client paint path
(Materiaal `paint_step`), then A/B the same baseline (interleaved builds,
same host) before and after the fix.
