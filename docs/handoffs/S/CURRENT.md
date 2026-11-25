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

- **cycle_id:** S-20261125-3. Cycle S-20261125-2 landed on `main` as
  `77dd864e` (PR #521: O13 kernel write latch specification, S09 2.7, N38).
- **Owner (25 November 2026):** S is held by the local S session that the PR
  guard started and registered as route S (handoff comment 7158653076 on
  PR #522). Earlier holders: [7279be] (`local_ccd5edd7`) and `local_55c5182e`.
- **This branch:** `claude/s09-o13-virtio-latch-20261125`, draft PR #522:
  O13 step 1, an add-only latch table (4 entries) inside each device
  `Policy` in `crates/virtio-core/src/lib.rs`; the L1 hook where `Unknown` is
  set (`complete`, `start_reset`, `exit`); `latch_operation` (L1b refusal
  rule), `write_latched` (enforcement query) and `latch_scope` (0x44
  whole-entry rule, `Range` when no whole entry; step 2 maps it to -5).
  15 `o13_*` host tests. Independent AI review O13-STEP1-REVIEW-1: APPROVE
  with one should-fix (FLUSH at the `start_reset`/`timeout`/`exit` sites
  untested), applied; the WRITE-only `start_reset` mutant is now killed.
  Executed with toolchain 1.100.0, exit 0: on source `fc38cd11` (the
  current implementation; later commits change only docs and progress),
  virtio-core and the virtio-blk/-net/-rng drivers test and clippy
  `-D warnings` and `fmt --all --check`; on the earlier source `741103a9`
  (before the review fixes), `tools/build.py --kernel-feature
  native-volume`. That build is not evidence for `fc38cd11`; the hosted
  native gates on the PR head are.
- **Coordinator requests:** C1–C5 in `COORDINATOR-REQUEST-20261124.md` are
  still open. The assessment-block conflict is decided by the coordinator
  (D3: per-PR assessment files, CI PR #519; then D1 merge queue); until #519
  lands, the single assessment block stays.
- **Process note:** `main` enforces strict up-to-date required checks; a PR
  that falls behind needs `main` merged, its gates re-run and a `merge-hold`
  re-run before the expected-head squash merge.

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

1. O13 step 1 (PR #522, see above): hosted checks, merge-hold, progress
   assessment, squash merge. Specification: the add-only latch table beside `Policy` in
   `crates/virtio-core` (open-to-latched only; WRITE intersection; a FLUSH
   with an `Unknown` completion or a device-side refused submission latches
   the whole device, full trigger list in S09 2.7; `Unknown` hook in `complete`, `start_reset` and
   `exit`), with host tests and the killing mutations of N38.
2. O13 step 2: `volume.rs` enforcement before the input copy, syscall `0x44`
   routed only under `native-volume`, and the native N38 stage (new stage and
   pins by the CI owner).
3. Native K5 fixtures (new stage; pins by the CI owner, C4/C5).
4. O12 owner-binding design with B and the user/session owner; O1
   remembered-VERIFY store; O3/O5 catalog extension.
5. R5: destination service binary in the test profile on disk 1 with
   `DEST:ID:TEST-ONLY` and a scripted dialog.

Owner decisions D1–D5 and a human audit remain outside S's authority.
