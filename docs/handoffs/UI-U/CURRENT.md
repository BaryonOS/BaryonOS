# UI-U handoff — current resume pointer

This file is the compact resume pointer for the UI-U workstream. It is not a
backlog: tasks live in GitHub issues, `docs/progress.json` and the roadmap
ledger. Newer ADRs and live repository evidence take precedence over this file.

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
  check, or weakened tests. The roadmap runner and merge queue stay disabled.
- **Coordination:** storage with S, drivers with H, permissions with N/B.
  U06 requires real users; AI review does not replace them.
- **Working agreements:** one primary open PR per cycle (at most one dependent
  child); own worktree with separate `NOS_BUILD_DIR`/`NOS_TARGET_DIR` inside
  the checkout (`build/` is ignored; the FPU runner rejects a build directory
  outside the checkout); one writer per file.

## Current cycle

- **cycle_id:** `UI-U-C8-20261125`
- **owner session:** local session "BaryonOS - Specialist U (UI/UX)"; a
  second UI-U session started by the PR watcher withdrew as writer.
- **task:** U03 read-only refusal — Ctrl+S on a read-only document shows the
  SDK's `ReadOnly` refusal again. Since #210 the `can_save` gate in
  `ui/native/src/controller.rs` made it a silent no-op, so the native
  materiaal `recover` probe timed out in its `readonly` phase (C7).
- **branch:** `claude/ui-u-readonly-ctrl-s-20261125` (based on `main`),
  worktree `~/projects/BaryonOS-worktrees/ui-u-cycle8`.
- **PR:** recorded in the PR itself and in its HANDOFF comment after merge.
- **Evidence scope:** host tests (`chain.rs`
  `readonly_reopened_ctrl_s_refuses_visibly`, with a mutation control) and
  local native `normal` and `recover` probe runs; results are in the PR.
  Hosted gates never run the `recover` case.

## Previous cycles

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

## Open items on 24 November 2026

- **OWNER:** at logical height 240 (320x240 @ 100 %, 640x480 @ 200 %) the
  library screen's single list row leaves no room for a second status line, so
  a two-line failure label still runs below the viewport there. Options: hide
  the row while a failure shows, add vertical screen scrolling (the save
  recovery spec rules out hiding and overlap), or approve shorter compact copy.
  The SR11 test in `ui/native/tests/recovery_matrix.rs` stays ignored for this.
  The same clipping occurs where a long wrapped list name fills the free space
  (for example 320x350 with 32-byte names); the status then keeps its one-line
  rect rather than overlap the name.
- The compact save-progress bar is drawn at `Layout::status.y + 24`, which is
  the bottom edge of a 240-high compact viewport; verify and fix it with the
  owner decision above.
- Deny copy (`Service(Access)`) stays generic until U03-T3 part 2 proves a real
  deny path; its test stays ignored.
- #449 is closed (C7 native run). The `recover` readonly-phase timeout found
  there is fixed by C8.
- #448 (Editor 700–783 px compact overlap/collapse): changes frozen frames and
  needs the frame-change list and design approval.
- U04-K1: no keyboard-only defect remains pinned by an ignored test; held-key
  (C4), Space-on-result (C6) and the Notifications route (C5) are fixed on the
  host. Native keyboard evidence is still missing.
- Storage-owner follow-up: the unbounded stat `history_count` decode
  (`services/volume/src/client.rs:483`, `sdk/document-session/src/volume.rs:286`).
- Open follow-ups named by PROJECT-STATE: Search S3 wiring and native run,
  read-only refusal part 2, Settings accounts/recovery, U02 interaction budgets
  (owner decision), native accessibility broker/screen-reader chain (TTS/braille
  owner decision), U06 human studies.

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

## Next executable task

After C8 merges: the next independent U item from the open list above (for
example the unbounded `history_count` decode with the storage owner S).
