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
- Gap on #486: the local gh token lacks `read:project`, so project 5 membership and
  fields are not set; owner action needed.

## Current cycle

- cycle_id: b02-key1-relay-20261124
- task: B02-T4 (ledger), native KEY1 key-state commit ingress per ADR-B02-KEY1-RELAY-20261123
- branch: claude/b02-key1-relay-native-20261124
- PR: #486 (ready for review; auto-merge off)
- tested source SHA: recorded in the PR body, not here
- state: RESUME_REQUIRED. Independent security and evidence verifiers returned MERGE; their
  low findings and all CodeRabbit findings are fixed (49b1557d, 07a8879f) and confirmed.
  Main is integrated by merge commits; hosted gates on the current head and the merge remain.

## Evidence / NOT RUN

- Evidence: `docs/verification/b02-key1-relay-20261124/` (native relay cases and B01 gate re-run at `07a8879f`,
  host checks at `d4db823e`; see its README):
  relay, relay-not-committed, relay-timeout pass (raw 33); red relay-leak stops at
  `SYSTEM:FAIL:APP_COMMIT_HANDLE` (raw 35, disk unchanged); wrong-case control 12/12
  rejected; `tools/test_system.py` and `tools/test_keyring.py` exit 0.
- NOT RUN: `agent_gate.py` prefilter (exits 1 at plan resolution: pinned checkout
  needs a local adapter); hosted CI; run_random/run_images/Materiaal/NAS/TCP/cap-bench
  regressions; hardware; human audit.
- Open low items (documented in FR-B02R residuals): native refusal of opcodes 13/14 on the
  domain endpoint not measured; only one red manifest control; pre-supply wait unbounded
  (fails closed); relay cold-start RESTORE path depends on B02-T3.
- progress/PROJECT-STATE untouched; B02 stays open (the ADR grants no credit).

## Blockers (external)

- B02-T3 cold rollback anchor: owner ADR required.
- B05 external human audit: not commissioned.
- B06 native local AI / model loader rows: missing L/P/A components.
- B04 hosted Secure Boot job: protected `.github/**` request
  (docs/implementation/B04-SECURE-BOOT-CI-INTEGRATION-REQUEST.md).

## Parked local work (not pushed, not in this cycle)

- claude/b05-user-abi-20261123 e4e6a9aa — open medium verifier finding (quota off-by-one mutant survives).
- claude/b05-virtio-caps-20261123 5dc89767 — verifier MERGE.
- claude/b03-floor-crash-20261123 f18ba29c — verifier MERGE.

## Lessons

1. `tools.tests.test_roadmap_runner` has 5 pre-existing failures while build/roadmap-pipeline/STOP exists.
2. With `GIT_EDITOR=true` in the environment, git-driven `tools/tests` fail (2 failures, 9 errors); run them with it unset.
3. `tools/run_system.py` requires `NOS_BUILD_DIR` inside the checkout (`private_path`).

## Next executable task

- Finish #486: confirm hosted checks green on the current head (integrate main again if it
  moved), run the SHA-bound acceptance release and let merge-hold revalidate, squash-merge
  with the expected head, verify on main, then post `HANDOFF B b02-key1-relay-20261124` on #486.
- After that cycle: B04 build divergence (loader/image_payload) diagnosis without normalising differences away.
