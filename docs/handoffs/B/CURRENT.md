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

## Current cycle

- cycle_id: b02-key1-relay-20261124
- task: B02-T4 (ledger), native KEY1 key-state commit ingress per ADR-B02-KEY1-RELAY-20261123
- branch: claude/b02-key1-relay-native-20261124
- PR: (not yet opened)
- tested source SHA: recorded in the PR body, not here
- state: VERIFIED_LOCAL (code and native evidence committed; not pushed; no PR yet)

## Evidence / NOT RUN

- Evidence: `docs/verification/b02-key1-relay-20261124/` (source `d4db823e`):
  relay, relay-not-committed, relay-timeout pass (raw 33); red relay-leak stops at
  `SYSTEM:FAIL:APP_COMMIT_HANDLE` (raw 35, disk unchanged); wrong-case control 12/12
  rejected; `tools/test_system.py` and `tools/test_keyring.py` exit 0.
- NOT RUN: `agent_gate.py` prefilter (exits 1 at plan resolution: pinned checkout
  needs a local adapter); hosted CI; run_random/run_images/Materiaal/NAS/TCP/cap-bench
  regressions; hardware; human audit.
- Lead follow-up: ledger write set and toolchain note for B02-T4 (docs/handoffs/B/CURRENT.md
  and `+1.100.0`); progress/PROJECT-STATE untouched, B02 stays open.

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

1. `git merge-base --is-ancestor` says nothing under squash merges; check file presence on main.
2. `tools.tests.test_roadmap_runner` has 5 pre-existing failures while build/roadmap-pipeline/STOP exists.
3. With `GIT_EDITOR=true` in the environment, git-driven `tools/tests` fail (2 failures, 9 errors); run them with it unset.
4. `tools/run_system.py` requires `NOS_BUILD_DIR` inside the checkout (`private_path`).

## Next executable task

- Push B02-T4 and open the draft PR after lead review. After merge: B04 build divergence (loader/image_payload) diagnosis without normalising differences away.
