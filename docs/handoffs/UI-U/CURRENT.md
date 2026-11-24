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

- **cycle_id:** `UI-U-C2-20261124`
- **task:** U05 — UX-SR07 Full/Quota/known-I/O save-failure copy and UX-SR11
  room for long compact status labels.
- **branch:** `claude/ui-u-recovery-copy-20261124` (from `origin/main` `7108f138`)
- **PR:** recorded in the PR itself and in the `HANDOFF UI-U UI-U-C2-20261124`
  comment after merge.
- **Tested source:** the reviewed and tested head is recorded in the PR
  verification table, not in this file.
- **Evidence scope:** host copy, layout, pixel and semantic tests; native
  library write and cold-start QEMU gates (scale 1); hosted window gates. No
  native run reaches a Full/Quota/I/O failure, and no human trial.

## Previous cycle

`UI-U-C1-20261124`: U04 / #447 History dialog order, merged as #483
(`7108f138`); its HANDOFF comment is on #483.

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
- Unmerged, PR-less work: `claude/u04-keyboard-only-20261124` (`6c2771ef`,
  new `sdk/examples/window-session/tests/keyboard_only.rs`, U04-K1).
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

U04-K1: verify `claude/u04-keyboard-only-20261124` (`6c2771ef`) against
current `main`, then land it as its own PR or record why not.
