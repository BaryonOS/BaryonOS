# Specialist S — current handoff

Compact resume pointer for the S workstream. It is not a backlog: tasks live
in `docs/progress.json`, the S09 specifications and GitHub.

## Scope and mandate (recorded once)

- **Scope:** finish S09 (backup, recovery and data export) and guard the
  accepted S01–S08. S owns the storage contract and recovery behaviour; the
  UI track owns the shared screens; the B track owns key and metadata
  properties.
- **Mandate:** the project owner's specialist assignment of 24 November 2026
  under `ADR-MERGE-DELEGATION-20261123`: S commits, reviews, repairs and
  squash-merges its own PRs with the expected-head procedure. One primary
  open PR at a time. No force-push, admin bypass, direct `main` push or
  weakened test.
- **Working rules:** a new cycle starts on a fresh branch from `origin/main`;
  native evidence is bound to the tested head; a merged slice does not accept
  the criterion.

## Current cycle

- **cycle_id:** S-20261124-1
- **Task:** S09 phase 3 slice B, the `l2-portal` build
  (`docs/implementation/S09-L2-NATIVE-EXPORT.md` section 9).
- **Branch:** `claude/s09-l2-slice-b-main` (replaces the unmerged
  `claude/s09-l2-slice-b`, which was based on the pre-#477 stack).
- **PR and tested head:** recorded in the pull request.
- **Evidence:** `docs/verification/s09-l2-native-export-20261124/SLICE-B.md`
  (host build and static analysis). NOT RUN: any guest execution of the
  `l2-portal` code (no kernel case selects it yet).
- **Blockers:** none for slice B.

## Lessons

1. `tools/test_volume.py` requires `NOS_BUILD_DIR` inside the checkout
   (`run_volume.py` takes a path relative to the repository root); use an
   ignored `build/<name>` directory.
2. Never match `pkill`/`pgrep -f` on a pattern that also appears in the
   invoking shell command; it kills the caller.
3. `App::meta` reports `step` and `step + 1`; leave a gap after every `meta`
   step id.

## Next executable task

S09 phase 3 slices C and D together in one PR: kernel cases 25–28, runner
and host oracle, `PLAN` 26 → 30 and both protected stage-count pins
(`.github/workflows/volume-acceptance.yml`, `kvm-pilot.yml`) as described in
`docs/implementation/S09-L2-NATIVE-STAGE-INTEGRATION-REQUEST.md`. A `PLAN`
of 30 fails required host-quality until the pins move, so they land
atomically. The kernel `ARCHIVE_IO` arms must accept the portal's refused
device-1 write probe (step 702).
