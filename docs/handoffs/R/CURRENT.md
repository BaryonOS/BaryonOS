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

## Update 25 Nov 2026, 13:30Z

- Fix head `a155749c` (local): superseded-frame fallback narrowed to
  `stale_below` (tiles a superseded partial frame may have blitted); new test
  `input_lost_banner_sends_exactly_the_changed_tiles` (exact changed-tile
  count, banner on screen); both new tests fail on base `lib.rs`.
- Acceptance items (1) and (2) above are superseded: on 25 Nov 2026 U
  accepted retain-by-exact-digest in their place (cache updated only after
  the blit acknowledgement; `stale_below` until the next complete frame;
  `surface_failed` and per-epoch geometry; the banner test). Focus or
  selection-ring, theme, layout, size and banner changes take no separate
  full-repaint trigger: every tile in the scan is rendered from the current
  view, and any tile whose bytes differ from the cached on-screen digest is
  blitted, so such a change is never retained. A scale or theme change that
  alters most tiles therefore re-sends most tiles; a resize means a new
  surface epoch with an empty cache. Covered by `tests/retained_digest.rs`
  (layout change with a superseded partial frame; input-lost banner with an
  exact changed-tile count). U reviews the code in the fix PR.
- Recover: fails on `main` until #523 (C7, #449). After #523 lands, merge
  `main` once and run U's sequence: build `native-system-materiaal-probe`,
  `run_materiaal_probe.py --case normal --qmp-transport pipe --timeout 240`,
  then build `test-system-materiaal-recover` and `--case recover --parent
  <normal run dir>`. Normal-only is not acceptable for merge; draft with
  recover Not run is fine.
- `document-menu-cancelled` (A ≈0.03 s, B ≈0.5 s at `5cd55d1c`): the phase
  injects Shift+F10 then Esc. A never finishes the menu frame before Esc
  supersedes it; B presents the menu and then fell back to a full repaint.
  Must appear as a finding in the PR. Rerun on `a155749c` in
  `~/projects/BaryonOS-scratch/r/ab3/`.
- Side finding (pre-existing on `main`, U's code path): input loss on an idle
  view triggers no repaint, so the InputLost banner appears only on the next
  view change.

## Update 25 Nov 2026, 14:10Z

- Fix head `c3721894` (local): the superseded-frame exclusion now covers only
  the hull of tiles that frame actually blitted. New test
  `superseded_partial_frame_tiles_are_resent_even_when_unchanged` fails with
  the exclusion removed (receiver refuses the RETAIN: `Err(Grid)`); the older
  supersede test did not cover this.
- `document-menu-cancelled` explained by guest counters: on the fix the
  menu-open generation is superseded by Esc after its menu tiles were already
  blitted (GEN 24→26, one completed frame); the base never reached those tiles,
  so its ≈0.05 s only shows that the expected image equals the pre-input one.
- `ab3` at `a155749c`: B2 timed out in `calculator-restored` (Calculator
  EVALUATIONS stayed 2, compositor FOCUSED 0; guest alive). Base: 20/20
  passes over three batches; fix: 11/12. Treat as an open regression signal
  until the final 10+10 batch (`r/ab4`, `--timeout 240`) is read.
- #526 is green at `b59c1294`; update it from `main` once, after #527, then
  merge. Do not push #526 before then (each push restarts its CI).

## Update 25 Nov 2026, 15:00Z

- Draft PR #540 (`claude/r02-dirty-repaint-20261125`), pushed head
  `44ec9722`: metadata verified by the PR guard (M11, enhancement, Refs #530,
  Project In Progress/UI/Feature). U review posted (issuecomment-7163393499):
  no UI blocker; merge condition remains recover after #523.
- Local commits `ddd7de58`, `6bdd50c1`, `2609e752` (not pushed) address U's
  non-blocking points and two CodeRabbit doc findings (final-tile claim
  scoped to the digest scan; one transaction per `paint_step`, up to four per
  `step_interactive`):
  reset `blitted`/`stale` in `rebind_presentation`; FNV-1a collision limit
  named as an evidence limit in the A/B README. Host suite (40 chain + 4
  retained_digest), clippy and fmt green.

- 16:40Z: hosted native-scale1 failed at `workspace-selected` on `44ec9722`:
  a harness paint race (base never presented Material's focus-lost banner).
  With U's approval, the gate fix `0797f42d` (wait for latest Material GEN,
  exact match against new oracle `initial-focus-lost`; interim until #541)
  was pushed with the review commits. Local `--workspace` scale 1/2, fix and
  base: raw 33. U review of `0797f42d` requested.

## Next action

1. When U reports #523 merged: in `r-dirty` merge `origin/main` once, re-measure
   `.text` (< 0x28000), rerun the host suite, then U's normal + recover
   sequence (see 13:30Z update); record results in the PR; push; update
   the PR verification table.
2. After #527 merges: update #526 from `main` once and push this handoff with
   it; merge #526 and #540 only under the recorded R delegation.
3. Keep the `calculator-restored` timeout signal open.
