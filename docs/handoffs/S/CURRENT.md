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

- **cycle_id:** S-20261126-1. Cycle S-20261125-3 landed on `main`: O13 step 1
  (PR #522, `192ef556`) and O13 step 2, the volume-broker write latch wiring
  (PR #539, squash `29a159cc` on 26 November 2026 08:12:34Z, tree verified
  equal to the tested head `45cb3136`). Neither merge accepts S09.
- **Owner:** the local S session ("BaryonOS - Specialist S"). Lane order and
  push permission come from the PR guard; the merge lane protocol applies.
- **Open PRs:** none for S. Copilot review quota is exhausted until
  3 December 2026; Codex review quota is exhausted; use subagent review.

### Local work, not pushed (all based on `main` `29a159cc` or `c04ea98d`)

| Worktree (`~/projects/BaryonOS-worktrees/`) | Branch | HEAD | Content |
|---|---|---|---|
| `s-n38` | `claude/s09-n38-latch-main-20261126` | see HANDOFF comment | N38 slices 1+2 (six commits) rebased onto `29a159cc`, plus this file |
| `s-seed` | `claude/s09-volume-seed-read-20261125` | `6206b8c8` | `seeded-read` image variant for A03-T2c (base `c04ea98d`) |
| `s-catv2` | `claude/s09-catalog-v2-spec-20261125` | `e86aa336` | `docs/implementation/S09-CATALOG-V2.md` revision 1 (base `c04ea98d`) |

Verified on the rebased `s-n38` source (26 November 2026, toolchain 1.100.0,
exit 0): `o13-latch` and `o13-latch-0x44` PASS raw 33 (`build/n38main/`),
`test_volume_*.py` 214 OK (2 skipped), `test_system_harness` OK,
`progress.py --check` consistent. The full M6e gate has not been rerun on
this head (last 32/32 PASS on `9664801e`); it is required before the push.

The older pushed branch `claude/s09-n38-latch-stacked-20261125` (`9664801e`,
on `e093c837`) is superseded by the rebased branch; it is not force-pushed.

### Owner questions

V1–V9 in `docs/implementation/S09-CATALOG-V2.md` section 9 (catalog v2 for
O3 controlled deletion and O5 backup time). No catalog v2 code before the
answers.

### Agreements and debts

- A03-T2c (Specialist A): the read-only volume path is the existing
  whole-volume read-only mount plus a BLOCK_READ-only grant; A grants
  object 1 of `seeded-read`. A per-ingress read-only right is a possible
  later S task with its own design; it is not planned.
- Owed to H: the quarantine code -24 mapping (fail closed, separate from
  -13/-14) once H's code is on `main`; the post-wake remount
  generation-check design item. H05-T5c may start now that #539 is on `main`.

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

11. `tools/run_volume.py --case <c>` needs `tools/build.py --kernel-feature
   test-volume-<c>` with the same `NOS_BUILD_DIR` first; it checks the
   manifest's feature and source hash.

9. The reverse differential is two-step: the Python test writes the corpus
   (`DEST_REVERSE_OUT=build/<dir>`), then the Rust test reads it
   (`DEST_REVERSE_IN`, absolute path). Without the variable the Rust test
   passes vacuously; the Python corpus checks always run.

## Next executable task

1. N38 PR from `s-n38` when the PR guard grants the lane: push the rebased
   branch under its new name (no force-push), open the PR with the native
   N38 stage, the gate integration and the stage-count pins moved in the
   same PR (the workstream moves its own pins; workflow files need the CI
   D2 review). Before the push, rerun the full M6e gate on the head.
2. `s-seed` PR (seeded-read image), then `s-catv2` PR (catalog v2
   specification), one primary open PR at a time; rebase each onto `main`
   first.
3. Native K5 fixtures (new stage; S moves its own pins).
4. O12 owner-binding design with B and the user/session owner; O1
   remembered-VERIFY store; catalog v2 implementation after V1–V9.
5. R5: destination service binary in the test profile on disk 1 with
   `DEST:ID:TEST-ONLY` and a scripted dialog.

Owner decisions D1–D5 and a human audit remain outside S's authority.
