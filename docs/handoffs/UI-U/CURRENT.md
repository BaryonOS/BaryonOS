# UI-U handoff — current resume pointer

This file is the compact resume pointer for the UI-U workstream. It is not a
backlog: tasks live in GitHub issues, `docs/progress.json` and the roadmap
ledger. Newer ADRs and live repository evidence take precedence over this file.
The canonical live state is the latest `HANDOFF UI-U` comment on the open UI-U
pull request or, when none is open, on the most recently landed UI-U pull
request (as of 27 November 2026: [#629 resume checkpoint](https://github.com/BaryonOS/BaryonOS/pull/629#issuecomment-7228152411)). This file and GitHub are
sufficient to resume; local paths below are optional additions.

## Resuming

A resuming agent, whatever its client, first verifies the branch, exact HEAD,
`git status --short`, ownership and the freshness of the latest `HANDOFF UI-U`
comment against GitHub, and never redoes landed work. Graphify is optional
context; client-specific start prompts are adapters and hold no state.

## Scope and mandate (recorded once)

- **Owner:** the UI-U specialist session owns the complete UI/UX assignment and
  criteria U02–U06 as one workstream, and guards accepted U01.
- **Preserved scope:** the approved identity, Orbitaal and reference set, the
  product ambition, UX-E/F/G, and the creative/productivity suite S0–S10. Suite
  codes S0–S10 are not the storage criteria S01–S09.
- **Mandate:** commit, push, open, review, repair and squash-merge own PRs under
  ADR-MERGE-DELEGATION-20261123, with strict branch protection, `host-quality`,
  `qemu-smoke`, merge-hold, CodeQL and applicable acceptance holds. No direct
  push to `main`, force-push, admin bypass, auto-merge without a fresh head
  check, or weakened tests. The merge queue is used under the current owner
  mandate; the issue controller remains disabled.
- **Coordination:** storage with S, drivers with H, permissions with N/B.
  U06 requires real users; AI review does not replace them.
- **Progress duties (owner, 27 November 2026):** per own PR, UI-U keeps labels,
  milestone, Project #5 fields and final status, issue and criterion links
  (`Refs`/`Part of #N`), a D3 assessment in
  `docs/progress/assessments/<date>-<slug>.json` (docs and handoff PRs too),
  this file and Graphify notes; the PR body names the affected criteria and the
  evidence behind any progress claim. A merge is not an acceptance. UI-U keeps
  its own criterion rows in `docs/progress.json` `delivery_review` current
  without asking (date and sha in the row text, no global snapshot bump,
  `progress.py --write` then `--check`). UI-U never changes the score: when all
  acceptance conditions of a criterion are met and its independent verification
  is complete, UI-U reports the criterion ID and evidence links to the PR guard,
  which performs the formal acceptance (13:14Z owner clarification).
- **Owner decisions of 27 November 2026 (ADR-U-OWNER-20261127):** compact
  recovery copy and layout, U02 budgets with R under R02-L1, the native
  accessibility broker (GO) and the first U06 study (protocol §12 answered).
- **Working agreements:** one primary open PR per cycle (at most one dependent
  child); own worktree with separate `NOS_BUILD_DIR`/`NOS_TARGET_DIR` inside
  the checkout (`build/` is ignored; the FPU runner rejects a build directory
  outside the checkout); one writer per file.

## Current cycle

| Field | Value |
|---|---|
| cycle_id | `UI-U-RESUME-20261127` |
| Owner | UI-U specialist |
| Task | Resume UI-U after #629; coordinate the bounded #589 observation and local fallback |
| Branch and PR | See the latest `HANDOFF UI-U` comment; do not infer an active source branch from this pointer |
| Evidence scope | #629 records owner decisions; it does not accept U02–U06. Native, physical and human evidence remain distinct |
| Next | Review R's concrete #589 compositor slice and K's reporter against their source-bound single-CPU interface; preserve the live-compositor restart hold, then continue the remaining #630 items |

## Previous cycles

- `UI-U-C12-20261127`: owner decisions and U06 protocol recorded in #629,
  merged as `dc29612c`; the landing and resume HANDOFF are on #629.
- `UI-U-C13-20261127`: compact failure layout, save-progress bar and SR11,
  merged as #639 (`f83a4ef5`).
- Assignment failure status colour: merged as #645 (`8d97a8de`); its landing
  HANDOFF records the host evidence and remaining U05 limits.
- `UI-U-C11-20261127`: #541 no InputLost banner on FocusLost or Resync,
  merged as #612 (`52e4430a`); landing HANDOFF comment on #612.
- `UI-U-C10-20261127`: #537 idle InputLost banner, merged as #596
  (`2d0d2d51`); HANDOFF comment on #596.
- `UI-U-C9-20261126`: #545 window-session reporter sampling and settled
  taskbar wait, merged as #586 (`627437ff`); HANDOFF comment on #586.
- `UI-U-C8-20261125`: U03 read-only Ctrl+S refusal, merged as #523
  (`55eeccc7`) through the merge queue; HANDOFF comment on #523.

- `UI-U-C7-20261125`: #449 native materiaal probe run (no code change);
  `normal` and `keyboard` passed with raw exit 33, `recover` timed out (fixed
  by C8). HANDOFF comment on #449.
- `UI-U-C5-20261125`: U04 Alt+N panel as the single notification centre,
  merged as #520 (`017934d7`); HANDOFF comment on #520.

- `UI-U-C6-20261125`: U04 Space activates a focused search result, merged as
  #516 (`d436a595`); HANDOFF comment on #516.

- `UI-U-C4-20261124`: U04 held Enter/Space activates once, merged as #508
  (`02235c1f`); HANDOFF comment on #508.

- `UI-U-C3-20261124`: U04-K1 keyboard-only host test, merged as #497
  (`8699d65f`); HANDOFF comment on #497.
- `UI-U-C2-20261124`: U05 UX-SR07/SR11 save-failure copy, merged as #487
  (`9f0eb19e`); HANDOFF comment on #487.
- `UI-U-C1-20261124`: U04 / #447 History dialog order, merged as #483
  (`7108f138`); HANDOFF comment on #483.

## Project #5 metadata (standing agreement, 24 November 2026)

The workstream maintains its own issues and PRs on Project #5: membership,
labels, Status, Work Type and milestone (UI-U = M7). Effort (1/2/3/5/8) only on
leaf issues; Target date stays empty without a real deadline. A closing keyword
only for an issue the PR fully resolves. Merged/completed = Done, any other
closure = Closed. No schema or permission changes and no artificial issues.

## Open items (updated 27 November 2026)

- **Compact recovery:** #639 fixed compact status copy and the save-progress
  bar and enabled SR11. Compact workspace button labels at 320x240 remain an
  open #630 item. Human usability evidence is still required.
- Deny copy (`Service(Access)`) stays generic until U03-T3 part 2 proves a real
  deny path; its test stays ignored.
- #449 is closed (C7 native run). The `recover` readonly-phase timeout found
  there is fixed by C8 (#523).
- #559 (Orbitaal `progress.svg`) landed as `c6f954f3`; #568 is closed.
- #448 (Editor 700–783 px compact overlap/collapse): changes frozen frames and
  needs the frame-change list and design approval.
- U04-K1: no keyboard-only defect remains pinned by an ignored test; held-key
  (C4), Space-on-result (C6) and the Notifications route (C5) are fixed on the
  host. Native keyboard evidence is still missing.
- Storage-owner follow-up: the unbounded stat `history_count` decode
  (`services/volume/src/client.rs:483`, `sdk/document-session/src/volume.rs:286`).
- Open follow-ups named by PROJECT-STATE: Search S3 wiring and native run,
  read-only refusal part 2, Settings accounts/recovery. U02 interaction budgets
  are recorded under R02-L1; measurement methods and profile cells remain open.
  The native accessibility broker design (GO; TTS
  and audio with H, braille from the start) and the U06 study kit (consent,
  privacy procedure, Dutch SUS, KVM reference profile; the owner arranges
  participants) are decided by ADR-U-OWNER-20261127 and now executable.
- B01-LOCK (#564): U's UX constraints for the lock, sign-in and unlock surface
  are on #564 (issuecomment-7219441015); B proposes the split after its ADR PR.

## Lessons

1. Native window gates need `NOS_BUILD_DIR` inside the checkout.
2. The History dialog order is coupled to `tools/window_library.py` and
   `ui/native/tests/library_native_fixture.rs`; change all three together.
3. The `delivery_review` snapshot in `docs/progress.json` is pinned to its own
   commit; a task assessment does not refresh it.
4. Status text is laid out by `Frame::status_area`; the painter and both
   semantic Status emitters must use it, and `wrapped_lines` must count lines
   exactly like `Paint::wrapped`. `high_contrast.rs` fingerprints contain no
   `Failed` state, so they do not guard failure layouts.
5. `tools/agent_gate.py` cannot list `native-window-session.yml` (its job env
   needs Actions context); run the workflow's commands directly.
6. The materiaal `recover` case needs its own build
   (`test-system-materiaal-recover`) in the same `NOS_BUILD_DIR`, `--parent`
   pointing at a fresh `normal` run and `--vga-rom`; hosted CI never runs it.
7. A `can_save` gate on a shortcut must still surface a read-only refusal;
   the probe's kernel watchdog (20 000 ticks) is the only symptom otherwise.
8. Host Rust gates use toolchain `1.100.0` (CI), not the `1.90.0` in AGENTS.md.
9. With the merge queue active, `gh pr merge N --squash --match-head-commit
   <sha>` enqueues; on a BLOCKED PR it silently enables auto-merge instead, so
   check `mergeStateStatus` CLEAN and `autoMergeRequest` null first.
10. `tools/test_window_session.py` pins the exact compositor activity offsets;
    change it together with `snapshots` in `kernel/src/m2/m3/window_session.rs`.
11. The window-session compositor sends `Cancel(Resync)` on a shell overlay,
    window move or mode change and on a source `Sync`; status logic that
    treats `Resync` as a failure changes the native status-bar rows.
12. A merge-queue squash does not always close `Closes #N`; verify the issue
    and Project #5 status after landing and close it with the landing SHA.

## Next executable task

The single next safe action is stated in the latest `HANDOFF UI-U` comment.
After #612 lands: #589, then the next independent item from the open list
above.
