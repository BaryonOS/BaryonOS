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

- **cycle_id:** `UI-U-C3-20261124`
- **task:** U04-K1 — keyboard-only reachability and activation host test
  (`sdk/examples/window-session/tests/keyboard_only.rs`), landing the PR-less
  commit `6c2771ef` on current `main`.
- **branch:** `claude/ui-u-keyboard-only-20261124` (from `origin/main` `9f0eb19e`)
- **PR:** recorded in the PR itself and in the `HANDOFF UI-U UI-U-C3-20261124`
  comment after merge.
- **Tested source:** the reviewed and tested head is recorded in the PR
  verification table, not in this file.
- **Evidence scope:** host test only (5 passing, 3 ignored with named
  defects). No native keyboard run, no screen reader, no human trial.

## Previous cycles

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
- #449 (materiaal probe Tab counts): the keystroke fix is on main; the issue
  stays open until a native materiaal probe run shows raw exit 33 with the
  reopen through the row.
- #448 (Editor 700–783 px compact overlap/collapse): changes frozen frames and
  needs the frame-change list and design approval.
- U04-K1 keyboard defects, each pinned by an ignored test in
  `keyboard_only.rs` (shell.rs line numbers at `9f0eb19e`): held Enter/Space
  repeats activation (`KeyAction::Repeat` accepted, line 432); Space on a
  focused search result types instead of activating (line 485); the
  Notifications page opens only by pointer (lines 528-537). Fixes change shell
  behaviour and frozen frames, so they need design-track agreement first.
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

## Next executable task

After C3 merges: fix the held-key repeat defect in `Shell::key` (smallest,
no frame change) and un-ignore its test, after checking with the design track
that no frozen frame depends on repeat activation.
