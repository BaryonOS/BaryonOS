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

- **cycle_id:** `UI-U-C1-20261124`
- **task:** U04 / #447 — History dialog focus and semantic order equal the
  visual reading order (Version, Cancel, Restore).
- **branch:** `claude/ui-u-inventory-20261124` (from `origin/main` `1fc9b8ca`)
- **PR:** recorded in the PR itself and in the `HANDOFF UI-U UI-U-C1-20261124`
  comment after merge.
- **Tested source:** the reviewed and tested head is recorded in the PR
  verification table, not in this file.
- **Evidence scope:** host semantic tree and Tab guard plus the native library
  write and cold-start QEMU gates. No assistive-technology or human trial.

## Inventory on 24 November 2026

- No open PRs. Recent UI work landed through integration PR #480.
- #449 (materiaal probe Tab counts): the keystroke fix is on main
  (`tools/run_materiaal_probe.py`, Tab ×4/×3 with a target comment) and the host
  test `native_probe_reopen_sequence_targets_the_list_object` passes. The issue
  stays open until a native materiaal probe run shows raw exit 33 with the
  reopen through the row.
- #448 (Editor 700–783 px compact overlap/collapse): open; changes frozen
  frames and needs the frame-change list and design approval.
- Ignored defect tests on main besides #447: four in
  `ui/native/tests/recovery_matrix.rs` (UX-SR07 Full/Quota, known I/O and
  deny copy; SR11 640x480@200 % status clipping). A green suite with these
  skips does not close the defects. Deny copy depends on U03-T3 part 2.
- Unmerged, PR-less work: `claude/u04-keyboard-only-20261124` (`6c2771ef`,
  new `sdk/examples/window-session/tests/keyboard_only.rs`, U04-K1). Verify
  and land it in a later cycle or record why not.
- Open follow-ups named by PROJECT-STATE: Search S3 wiring and native run,
  read-only refusal part 2 (live service refusals and counters), Settings
  accounts/recovery, U02 interaction budgets (owner decision), native
  accessibility broker/screen-reader chain (TTS/braille owner decision), U06
  human studies.

## Lessons

1. Native window gates need `NOS_BUILD_DIR` inside the checkout.
2. The History dialog order is coupled to `tools/window_library.py` and
   `ui/native/tests/library_native_fixture.rs`; change all three together.
3. The `delivery_review` snapshot in `docs/progress.json` is pinned to its own
   commit; a task assessment does not refresh it.

## Next executable task

U05: fix the UX-SR07 Full/Quota and known-I/O status copy and the SR11 compact
200 % clipping, un-ignoring the three matching tests in
`ui/native/tests/recovery_matrix.rs` (deny copy stays with U03-T3 part 2).
