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
full app-body repaints (~1785 SUR1 tiles each) per run; the matching phases were observed at 0.84–0.92 s
(input to first matching screendump) on `-smp 1`. QEMU host CPU time is
0.88–0.91 of wall time (host-process accounting; guest utilisation and idle
behaviour were not measured). The runner's 0.2 s
screendump retry quantises latency; exact timing needs per-attempt
timestamps (local diagnostic patch only, not committed).

## Writer agreement with U (25 Nov 2026)

R is the single writer for changed-region repaint in the Materiaal client
paint path, in its own PR. Write set: `sdk/examples/materiaal/src/*` and new
R-owned test files under `sdk/examples/materiaal/tests/`. Until #523 merges,
do not touch `sdk/examples/materiaal/tests/chain.rs`,
`ui/native/src/controller.rs` or `docs/handoffs/UI-U/*`; after #523 lands,
merge `main` once before pushing. U acceptance: (1) full repaint remains the
fallback on layout/size/theme change, focus/selection-ring change and the
read-only/error banner (U03 refusal stays visible); (2) negative test with a
mutation control: one changed region, all other tiles byte-identical, the
changed tile correct; (3) materiaal host suite, clippy, fmt green; native
materiaal-probe normal and recover with raw exit 33 (readonly phase still
ERRORS 1, SAVE_ACKS 0); (4) A/B on the same host in a new verification
directory. Request U review (UI/a11y); U does not merge. R updates
`docs/implementation/VM-HARDWARE-AND-PERFORMANCE.md`.

## Fix in progress (25 Nov 2026, checkpoint 12:30Z)

- Branch `claude/r02-dirty-repaint-20261125` (worktree
  `~/projects/BaryonOS-worktrees/r-dirty`), head `cceeb461`, not pushed.
  Base `3b9a94c5` (identical Materiaal `lib.rs` to `origin/main`).
- Change: in retained mode the existing sparse scan RETAINs runs of tiles
  whose rendered bytes match the digest cache (render every tile, RETAIN on
  match; no separate full-repaint trigger). Gates: complete frame
  (`damage_valid`), retained base present, never the last tile; a
  superseded partial frame disables it until the next complete frame. Blits
  are acknowledged within the same step, so no in-flight tile escapes that
  check. Native `.text` stays under the 0x28000 assertion; do not add a
  separate probe renderer (it overflowed by 99 bytes).
- Host: suite, clippy, fmt green; `tests/retained_digest.rs` fails on the
  base `lib.rs` (1608 tiles for a one-character edit) and passes on the fix.
- Local KVM A/B (`~/projects/BaryonOS-scratch/r/ab2/`, head `5cd55d1c`,
  load average 3.3–9.3, host-side latencies on the 0.2 s retry grid):
  10/10 window sessions pass raw 33; duration A 26.55–28.16 s (median 26.83),
  B 21.83–22.99 s (median 22.74). Many phases drop from ~0.85 s to ~0.25 s
  (summary in `r/ab2/summary.txt`). No QEMU CPU figure in these reports.
- Materiaal probe normal: raw 33 on the fix.

## Open items (block the U acceptance claim)

1. `document-menu-cancelled`: A ≈0.03 s (5/5), B 0.44–0.66 s in 4/5 runs.
   Materiaal `GEN` is unchanged around this phase on both sides; the cause
   (legitimate intermediate frame vs. wrong pixels presented) is not yet
   established. Next: per-phase compositor commit/retain counters from the
   serial logs, then the first mismatching screendump of one B run.
2. Materiaal recover probe fails identically on base `3b9a94c5` and the fix
   (raw 35, `MATERIAAL:FAIL:TIMEOUT` after the snapshot, KEYS 0), using the
   contract invocation with a normal parent in the same build directory.
   Not yet compared with the passing invocations in
   `docs/verification/native-test-version/gui-recover-evidence.json`. Ask U
   whether normal-only is acceptable if it stays failing.

## Next action

Resolve open item 1, compare item 2 with the recorded passing invocation,
then write evidence to a new `docs/verification/` directory, update
`VM-HARDWARE-AND-PERFORMANCE.md`, push and open the draft PR (merge `main`
once after #523; re-measure `.text`, rerun host suite and normal probe).
