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

- **cycle_id:** S-20261124-3
- **Task:** close the first open item of section 10 of
  `docs/implementation/S09-L2-NATIVE-EXPORT.md`: a native power cut inside
  an L2 restore (`l2-restore-cut`, kernel case 29) and its cold read-only
  recovery (`recover-l2-cut`, case 30). The cut fires inside the seventh
  INSTALL_COMMIT (table-sector write 209 on the fresh image).
- **Branch:** `claude/s09-l2-restore-cut`.
- **PR and tested head:** recorded in the pull request.
- **Evidence:** `docs/verification/s09-l2-restore-cut-20261124/README.md`
  (local native gate 32/32 at `ea59c6fc`; host tests).
- **Pins:** both stage-count pins move 30 → 32 in this PR.
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

6. The L2 restore order on a fresh image is: the format marker sector, then
   34 table sectors per commit (collections 1–3, then objects 1–5 in slot
   order). A cut is placed by counting lane-0 table-sector writes.

## Next executable task

After the restore-cut PR merges: S09 stays open. Pick the next S09 gap from
`docs/progress.json` (`delivery_review`, criterion S09): the user-facing
production copy/restore/export chain (production destination service,
trusted selection/portal and per-operation enforcement), or the open items
in section 10 of `docs/implementation/S09-L2-NATIVE-EXPORT.md`. Owner
decisions D1–D5 and a human audit remain outside S's authority.
