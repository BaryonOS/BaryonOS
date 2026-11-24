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

- **cycle_id:** S-20261124-2
- **Task:** S09 phase 3 slices C and D: kernel cases 25–28, runner and
  oracle, the 30-stage native gate and both stage-count pins
  (`docs/implementation/S09-L2-NATIVE-EXPORT.md` section 9).
- **Branch:** `claude/s09-l2-slice-cd`.
- **PR and tested head:** recorded in the pull request.
- **Evidence:** `docs/verification/s09-l2-slices-cd-20261124/README.md`
  (local native gate 30/30 at `61349b1f`; host tests).
- **Pins:** the project owner instructed on 24 November 2026 that there is
  no integration coordinator; S moves the pins its own change needs, in the
  same PR.
- **Blockers:** none.

## Lessons

1. `tools/test_volume.py` requires `NOS_BUILD_DIR` inside the checkout
   (`run_volume.py` takes a path relative to the repository root); use an
   ignored `build/<name>` directory.
2. Never match `pkill`/`pgrep -f` on a pattern that also appears in the
   invoking shell command; it kills the caller.
3. `App::meta` reports `step` and `step + 1`, and `App::write` reports
   `step` to `step + 3`; leave a gap after every such step id.
4. The broker serves one block request at a time across both disks. A portal
   must reach a READY service (any client call) before its first archive
   read, or the service's recovery read can fail with `Protocol`.
5. A native gate failure in one stage can be reproduced alone: copy the
   parent's `volume-image-*` directory and run `tools/run_volume.py --case
   <stage> --volume <copy>`. The parent check binds the source fingerprint,
   so a diagnostic kernel build needs the copied `last-stage.json` updated.

## Next executable task

After the slice-C/D PR merges: S09 stays open. Pick the next S09 gap from
`docs/progress.json` (`delivery_review`, criterion S09): the user-facing
production copy/restore/export chain (production destination service,
trusted selection/portal and per-operation enforcement), or the open items
in section 10 of `docs/implementation/S09-L2-NATIVE-EXPORT.md`. Owner
decisions D1–D5 and a human audit remain outside S's authority.
