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

## Fix in progress (25 Nov 2026)

- Branch `claude/r02-dirty-repaint-20261125` (worktree
  `~/projects/BaryonOS-worktrees/r-dirty`), head `5cd55d1c`, not pushed.
- Root cause: with `enable_retained`, the sparse digest skip was disabled,
  so the conservative whole-viewport damage re-sent every tile. The fix
  RETAINs digest-proven unchanged runs from the existing scan loop; a
  superseded partial frame disables this until the next complete frame
  (the receiver refuses RETAIN for tiles changed since base).
- Native `.text` 0x27e73 under the 0x28000 linker assertion (base 0x27ee3);
  a separate probe renderer overflowed it, so do not reintroduce one.
- Host: `tests/retained_digest.rs` passes; against `origin/main` `lib.rs`
  the edit test fails with 1608 tiles sent (mutation control). Host suite,
  clippy and fmt green.
- Running: `~/projects/BaryonOS-scratch/r/ab2.sh` (materiaal-probe normal
  and recover, then five interleaved A/B window runs; output in `r/ab2/`).

## Next action

Check probe exit 33 and A/B results, write evidence to a new
`docs/verification/` directory, update `VM-HARDWARE-AND-PERFORMANCE.md`,
then push and open the draft PR (merge `main` once after #523 lands).
