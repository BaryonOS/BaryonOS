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

- **cycle_id:** S-20261125-1 (cloud session). Cycle S-20261124-6 landed on
  `main` as `ad55e019` (PR #509, squash): owner decisions O1–O13 in
  `docs/implementation/S09-DESTINATION-SERVICE.md`, the R3 reverse
  differential (host), the R2 status record and K5 step 1 (per-device broker
  state in `kernel/src/m2/m3/volume.rs`, S3 latch in `advance_lane`).
  Evidence for K5: native volume gate 32/32 and H05 power gate 12/12 locally
  on `9b30c556`, and every hosted gate on the merged head.
- **This branch:** `claude/brave-feynman-jbiz6x`, restarted from `ad55e019`.
  Task: the two valid Graphify review findings on PR #509 (a real
  formatted-count-below-scope case; every reverse-corpus image needs a
  verdict) and this handoff.
- **Coordinator requests:** C1–C5 in `COORDINATOR-REQUEST-20261124.md` are
  still open (ADR, hosted R3 checks, P10, destination gate, K5 routing).
- **Process note:** `main` enforces strict up-to-date required checks; with
  merges about hourly and gates of about 45 minutes, a PR can stay behind.
  A merge queue would remove that race (repository setting, coordinator).

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

10. `tests/test_system_harness.py` (PostWakeLatchStructure) pins the source
   shape of the volume broker: every device call must follow the S3 latch
   in the same function and be counted by `device_access()`, and a
   rustfmt-split `.request()` line reads as a device call. Run it locally
   after any change to `kernel/src/m2/m3/volume.rs`.

9. The reverse differential is two-step: the Python test writes the corpus
   (`DEST_REVERSE_OUT=build/<dir>`), then the Rust test reads it
   (`DEST_REVERSE_IN`, absolute path). Without the variable the Rust test
   passes vacuously; the Python corpus checks always run.

## Next executable task

1. O13 kernel write latch in the per-device broker: specify the trigger and
   the service interface, then implement with a native test; keep READ and
   the remount FLUSH, keep N32; O17 mismatch alone sets no latch.
2. Native K5 fixtures (new stage; pins by the CI owner, C4/C5).
3. O12 owner-binding design with B and the user/session owner; O1
   remembered-VERIFY store; O3/O5 catalog extension.
4. R5: destination service binary in the test profile on disk 1 with
   `DEST:ID:TEST-ONLY` and a scripted dialog.

Owner decisions D1–D5 and a human audit remain outside S's authority.
