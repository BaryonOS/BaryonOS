# B workstream — current resume pointer

This file is the compact resume pointer for specialist B. It is not a backlog;
tasks live in `docs/roadmap-pipeline/ledger.json` and criteria in `docs/progress.json`.

## Scope and mandate (recorded once)

- Scope: finish B02 and B04–B06; guard B01/B03 (both accepted on main as of 1fc9b8ca).
- Mandate: specialist owner under ADR-MERGE-DELEGATION-20261123. May commit, push,
  open and repair PRs, resolve review threads with evidence and squash-merge
  (expected-head procedure). No direct main push, force-push, admin bypass,
  auto-merge without proven checks, or weakened tests. Old roadmap runner and
  merge queue stay disabled.
- At most one primary open PR (plus one managed child only when demonstrably needed).
- TEST keys/profiles grant no release signing, production anchors or new trust boundaries.
- Extra AI review is not external human audit (B05).

## Project metadata agreement (project owner, via the PR guardian, 24 Nov)

- Maintain project 5 membership, labels, Status, Subsystem, Work Type and milestone
  (B02: M6; B04–B06: M8). Priority/Start/Target are org issue fields; Start date only
  when proven, Target only for a real deadline. Effort (1/2/3/5/8) only on leaf issues.
- Record real issue–PR, parent/sub and blocked-by links; closing keywords only for the
  scoped issue a PR fully resolves. Merged = Done, other closure = Closed. A merge is not
  criterion acceptance. No schema/permission changes, no artificial issues.
- Gap: neither the local gh token (no `read:project`) nor the cloud session (repository-scoped) can reach project 5, so membership and
  fields are not set; owner action needed.

## Current cycle

- Session: cloud (CLOUD-HANDOFF-B, 25 Nov); the local B session is archived and no longer writes.
- cycle_id: b03-f1-floor-crash-20261125 (previous b05-t6-virtio-caps-20261125: PR #512; see its HANDOFF comment).
- task: B03-F1 native characterization of REL-05 (anti-rollback floor raised before kernel health); not a fix.
- branch: claude/b03-floor-crash-20261123
- PR: see the HANDOFF comment for the number and expected head
- state: RESUME_REQUIRED until the PR is squash-merged and verified on main.
- result: `tools/characterize_update_floor.py` + 9 host tests; evidence `docs/verification/b03-floor-crash-20261123/`.
  Re-run on main-merged source 25 Nov (QEMU 8.2.2, pc-q35-8.2): hazard observed, all four boots as expected.

## Evidence / NOT RUN

- Evidence: the README above (original run at a91d5ade + re-run section). Host tests 9/9 OK.
- NOT RUN: physical hardware; human audit. B03 stays accepted on main; REL-05 is an open design hazard that needs an
  owner policy decision (trial boot / health confirmation) before any loader change.

## Merge procedure lesson (25 Nov)

- With `pull_request_wait_minutes = 0`, merge-hold fails at once while gates run. After the applicable gates finish
  (ignore `upload-coverage-*`, `copilot*`, Graphify), re-run the merge-hold job, then squash-merge with the expected
  head before `main` moves. #493 needed six update rounds; signalled on #503.

## Blockers (external)

- B02-T3 cold rollback anchor: owner ADR required.
- B05 external human audit: not commissioned.
- B06 native local AI / model loader rows: missing L/P/A components.
- B04 hosted Secure Boot job: protected `.github/**` request
  (docs/implementation/B04-SECURE-BOOT-CI-INTEGRATION-REQUEST.md).

## Queued B branches (pushed, merged with main on 24–25 Nov, no PR yet)

- claude/b05-virtio-caps-20261123 7762e63f — B05-T6, verifier MERGE; virtio-core tests/clippy/fmt/scope exit 0.
- REL-05 fix (floor raised before kernel health) needs an owner policy decision (trial boot / health
  confirmation); B drafts a proposal, no silent change to anti-rollback semantics.

## Lessons

1. `tools.tests.test_roadmap_runner` has 5 pre-existing failures while build/roadmap-pipeline/STOP exists.
2. With `GIT_EDITOR=true` in the environment, git-driven `tools/tests` fail (2 failures, 9 errors); run them with it unset.
3. `tools/run_system.py` requires `NOS_BUILD_DIR` inside the checkout (`private_path`).

## Next executable task

- Finish the B03-F1 PR (gates, merge-hold re-run, expected-head squash merge, verify on main, post
  `HANDOFF B b03-f1-floor-crash-20261125`). REL-05 policy proposal: `docs/implementation/B03-REL05-FLOOR-POLICY-PROPOSAL.md`
  (owner decision A/B/C pending; no loader change before it).
