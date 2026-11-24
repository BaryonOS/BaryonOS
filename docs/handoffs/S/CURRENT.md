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

- **cycle_id:** S-20261124-6 (cloud session). PR #505 (production archive
  verifier) merged as `9dfff70c`; the branch was restarted from it.
- **Task:** (1) record the owner decisions of 24 November 2026 on O4 (a)–(c),
  O11, O12 and O13 in `docs/implementation/S09-DESTINATION-SERVICE.md`;
  (2) the R3 reverse differential (spec 8.2 R3), host only.
- **Branch / PR:** `claude/brave-feynman-jbiz6x`, PR #509 (draft); tested
  head is recorded in the PR. S-owned files only; self-merge allowed once
  `host-quality`, `qemu-smoke` and `merge-hold` pass on the head.
- **Results:** reverse differential 182 images agree; forward cross-check
  30 images agree; both mutations killed; fmt, clippy, all destination tests
  and the oracle suite pass locally.
- **Pins:** unchanged; no workflow change.
- **Handed to the integration coordinator (not S-owned):** the ADR
  `ADR-S09-R5-DECISIONS-20261124` text in the PR #509 body for
  `docs/DECISIONS.md`; a hosted CI step that runs both cross-check
  directions (forward: `DEST_ORACLE_OUT`, oracle_images then RustCrossCheck;
  reverse: `DEST_REVERSE_OUT` ReverseDifferential then `DEST_REVERSE_IN`
  reverse_differential).

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

9. The reverse differential is two-step: the Python test writes the corpus
   (`DEST_REVERSE_OUT=build/<dir>`), then the Rust test reads it
   (`DEST_REVERSE_IN`, absolute path). Without the variable the Rust test
   passes vacuously; the Python corpus checks always run.

## Next executable task

1. Land PR #509 (verify head, required checks, squash-merge with expected
   head) and confirm the result on `main`.
2. K5 per-device broker state (owner-approved under O4 (a)): read
   `kernel/src/m2/m3/volume.rs` broker (`Policy`, `resetting`, `error`,
   `active`, data buffer, around `:877-1014`), the 2.7 broker contract and
   section 4.3 K5 of the S09 destination spec; design the per-device split
   including property 3 (a further poll after a read ended by a reset
   returns -1), with native timeout fixtures. Kernel change: one small step,
   with an os-security-reviewer pass.
3. Open R2 item: property-3 device-state snapshot in `SimDisk`
   (`p12_3_answered_exactly_once`, review 7 RQ7-2) and the CODE-DELTA
   revision 5 rows.
4. O12 owner-binding design (catalog version, trusted identity, access
   checks, recovery route) against PROFILES-AND-USERS and the key and
   encryption contracts, before R7; O13 kernel write latch spec and native
   test before R8.

Also open (S09 stays open): the review-2 hardening items in section 10 of
`docs/implementation/S09-L2-NATIVE-EXPORT.md`. Owner decisions D1–D5 and a
human audit remain outside S's authority.

