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

- **cycle_id:** S-20261124-5 (cloud session; took over from the local S
  writer after `HANDOFF S S-20261124-4`, state CLEAR_READY, on PR #498).
- **Task:** R5 precondition of `docs/implementation/S09-DESTINATION-SERVICE.md`
  (2.3, "Archive verifier"): the production archive verifier. The
  `volume-verifier` feature of `destination-core` adds
  `archive_verifier::VolumeVerifier` over `volume_core::archive::verify`;
  host tests in `services/destination/tests/real_verifier.rs` run N7 and N10
  through the service on real v1 archives. Host only; no native stage.
- **Branch:** `claude/brave-feynman-jbiz6x`.
- **PR and tested head:** recorded in the pull request.
- **Pins:** unchanged (32 volume stages); no workflow change.
- **Blockers:** none for this step. R5 itself still waits for owner
  decisions O4 (a)–(c), a hosted R3 cross-check and reverse differential
  (lead-owned workflow change) and the K5 broker.

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
7. After a merged slice, record its independent review in a new
   `docs/verification/` child directory; merged evidence directories are
   historical and protected.

8. `destination-core` tests that need real v1 archives build them in-process
   with `volume_core` (`Service`, `Client`, `archive::export`) over an
   in-memory `Disk`; the harness switches verifiers with `AnyVerifier`.

## Next executable task

Disjoint S-owned host steps toward R5 that need no owner decision: the R3
reverse differential (the oracle's crafted invalid images mounted by the R2
core, verdicts compared; spec 8.2 R3), and the open R2 items of spec 8.2
(property-3 device-state snapshot, CODE-DELTA revision 5). The hosted R3
cross-check is a workflow change for the integration coordinator.

Also open (S09 stays open): the review-2 hardening
items in section 10 (host check pins the interrupted slot and chunk count,
object 4's appended chunks, collection never-used test) are a small native
follow-up. Pick the next S09 gap from
`docs/progress.json` (`delivery_review`, criterion S09): the user-facing
production copy/restore/export chain (production destination service,
trusted selection/portal and per-operation enforcement), or the open items
in section 10 of `docs/implementation/S09-L2-NATIVE-EXPORT.md`. Owner
decisions D1–D5 and a human audit remain outside S's authority.
