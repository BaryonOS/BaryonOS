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

- **cycle_id:** S-20261126-3. Landed on `main` on 26 November 2026: N38
  slices 1 and 2 with the 34-stage M6e gate (PR #555, `2289a271`), the
  `seeded-read` image variant for A03-T2c (PR #561, `fb4a40fa`), the
  catalog v2 specification revision 1 (PR #572, `9508ebfa`, tracking #569)
  and the post-wake remount design revision 0 (PR #580, `54c9f462`,
  tracking #564) and `Status::Quarantined` -24 with design
  revision 1 (PR #587, `43101e46`). On 27 November 2026 the post-wake
  remount design revision 2 (#564 follow-ups 1–6, documentation only) landed
  (PR #594, `095c0d24`). None of them accepts S09.
- **Owner:** the S session ("BaryonOS - Specialist S"). Merge order is
  coordinated by the PR guard; S enqueues its own merge-ready PRs.
  Immediately before enqueueing (PR guard, 26 November 2026):
  `gh run list -R BaryonOS/BaryonOS --commit <full head SHA> --limit 200
  --json databaseId,workflowName,status --jq '[.[] | select(.status !=
  "completed" and .workflowName != "Merge hold")]'` must print exactly `[]`;
  any other output, including empty output on an API error, means wait.
- **Open PRs:** none besides this handoff refresh. Copilot review quota is
  exhausted until 3 December 2026; Codex review quota is exhausted; use an
  independent subagent review.
- **Communication:** the GitHub threads #564 (post-wake remount, with the H
  track) and #569 (catalog v2) and the S PR itself carry the canonical state;
  this file is the resume pointer. No local path is needed to resume.

### Local work

None.

### Owner questions

V1–V9 in `docs/implementation/S09-CATALOG-V2.md` section 9 (catalog v2 for
O3 controlled deletion and O5 backup time), tracked in #569 with the
zero-time RTC follow-up. No catalog v2 code before the answers.

The `RemountRefused` status value is fixed by an ADR amendment reserved for
the project owner; S does not prepare that amendment. Revision 2 proposes -26;
the H track uses -25 (KL2 SUSPEND, #597) and -27 (H04-TXDOWN, #601).

### Agreements and debts

- A03-T2c (Specialist A): the read-only volume path is the existing
  whole-volume read-only mount plus a BLOCK_READ-only grant; A grants
  object 1 of `seeded-read`. A per-ingress read-only right is a possible
  later S task with its own design; it is not planned.
- With H: the -24 mapping landed with #587, in `services/volume` and
  its clients only (no kernel, SUSPEND or T5d file). The post-wake remount
  check is specified in `VOLUME-POST-WAKE-REMOUNT.md` revision 2 (one
  read-only grant to the same instance, R1; `RemountRefused` as its own
  service status, R2; both H's proposals on #564, binding only after the
  ADR amendment). `RemountRefused` is not built before that amendment; the
  native PW cases need H's T5d-T, T5e and UNLOCK first and H's agreement
  for `tools/run_system.py`, `tests/test_system_harness.py` and `init.rs`.
- Landing order with H05-T5c (agreed with H, 26 November 2026; both change
  `kernel/Cargo.toml`, `tools/build.py` and `kernel/src/m2/m3/volume.rs`):
  the first branch with a green local gate and the lane lands first; the
  second merges `main` (no rebase or force-push) and reruns its native gates.
  N38 registers latches only in the `test-volume-o13-latch*` profiles, never
  combined with `test-system-s3-blk-reinit`.

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

1. Native K5 fixtures (new stage; S moves its own pins).
2. `RemountRefused` and the post-wake remount code after the ADR amendment;
   the native PW cases after H's T5d-T, T5e and UNLOCK.
3. O12 owner-binding design with B and the user/session owner; O1
   remembered-VERIFY store; catalog v2 implementation after V1–V9 (#569).
4. R5: destination service binary in the test profile on disk 1 with
   `DEST:ID:TEST-ONLY` and a scripted dialog.

Merge procedure (owner decision of 26 November 2026, confirmed in the S
session): S enqueues its own merge-ready PRs in the merge queue without a
per-PR lane grant (mergeStateStatus `CLEAN`, `autoMergeRequest` null, no
unresolved thread, open-run check exactly `[]`, then the GraphQL
`enqueuePullRequest` mutation with `expectedHeadOid`; `gh pr merge` on a
`BLOCKED` PR silently enables auto-merge); the PR guard
coordinates order and dependencies. Non-blocking review comments on a pushed
PR are answered, resolved and recorded on the tracking issue, not pushed.

Owner decisions D1–D5 and a human audit remain outside S's authority.
