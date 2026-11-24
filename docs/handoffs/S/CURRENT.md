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

- **cycle_id:** S-20261124-6 (cloud session).
- **Owner decisions:** packages 1 and 2 of 24 November 2026 (O1–O13; O14–O18
  confirmed) are recorded in `docs/implementation/S09-DESTINATION-SERVICE.md`
  (header note, 8.1). ADR text and CI/K5 needs: `COORDINATOR-REQUEST-20261124.md`
  (C1–C5) beside this file.
- **Branch / PR:** `claude/brave-feynman-jbiz6x`, PR #509 (draft until the
  required checks pass on its head). Contents: decisions, R3 reverse
  differential (host), R2 status, K5 step 1 (`kernel/src/m2/m3/volume.rs`).
- **Evidence on `1a3924de` (local):** native volume gate 32/32 and H05
  power gate 12/12 (`build/k5d-*`); earlier on `04b06de2`: native volume gate 32/32
  (`build/k5c-volume.log`), H05 power gate 12/12 with S3, S3 re-init and
  held completion (`build/k5c-power/power-system-gate.json`); destination
  tests, oracle suite, reverse differential 182 and forward 30 images.
- **Review:** K5-REVIEW-1 (independent AI security review) REQUEST_CHANGES,
  R1 (S3 quiescence with an absent lane) fixed in `04b06de2`; O1–O3 applied;
  O4 (tickets restart per lane) noted, no consumer found.
- **Hosted:** `native-hw-inventory` failed once on `409614d4` with "Missing
  QMP greeting" in the plain boot (doc-only head; not attributed; commented).

## Lessons

1. `tools/test_volume.py` requires `NOS_BUILD_DIR` inside the checkout
   (`run_volume.py` takes a path relative to the repository root); use an
   ignored `build/<name>` directory.
2. Never match `pkill`/`pgrep -f` on a pattern that also appears in the
   invoking shell command; it kills the caller.
3. `App::meta` reports `step` and `step + 1`, and `App::write` reports
   `step` to `step + 3`; leave a gap after every such step id.
4. Since K5 (PR #509) the broker serves at most one block request at a time
   per disk; requests on different disks progress independently. Before K5
   one request was served across both disks, and a portal had to reach a
   READY service before its first archive read or the service's recovery
   read could fail with `Protocol`; keep that order in existing flows until
   a native fixture shows the per-disk admission.
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

9. The reverse differential is two-step: the Python test writes the corpus
   (`DEST_REVERSE_OUT=build/<dir>`), then the Rust test reads it
   (`DEST_REVERSE_IN`, absolute path). Without the variable the Rust test
   passes vacuously; the Python corpus checks always run.

## Next executable task

1. PR #509: confirm `host-quality`, `qemu-smoke`, `merge-hold` (and
   `native-hw-inventory`) on the head, then squash-merge with the expected
   head and confirm on `main`.
2. Native K5 fixtures (new test stage; pins by the CI owner, C4/C5):
   disk-1 admitted while disk 0 is in flight; no disk-1 request between a
   disk-1 timeout and the acknowledged reset.
3. O13 kernel write latch in the per-device broker (trigger and service
   interface first, then a native test; keeps READ and remount FLUSH).
4. O12 owner-binding design (catalog version, trusted identity, access
   checks, recovery without the original disk) with B and the user/session
   owner; O1 remembered-VERIFY store; O3/O5 catalog extension.
5. R5: destination service binary in the test profile on disk 1 with the
   deterministic `DEST:ID:TEST-ONLY` ID and a scripted dialog.

Owner decisions D1–D5 and a human audit remain outside S's authority.
