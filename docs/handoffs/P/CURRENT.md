# Specialist P — resume pointer

Compact resume pointer for the personal-storage specialist (P01–P04).
This file is not a backlog: tasks come from `docs/progress.json`, the roadmap
ledger, issues and pull requests. Newer ADRs and live evidence take precedence.

## Scope and working agreements

- Scope: P01 native end-to-end encrypted NAS storage, P02 automatic
  synchronisation and conflicts, P03 multiple devices and key lifecycle, P04
  offering, revoking and removing storage. Crypto choices are coordinated with
  B (keys, trust) and N (transport); P designs no cryptography.
- Session: "BaryonOS - Specialist P", coordinated by "BaryonOS - PR Bewaker".
- Merge: `ADR-SPECIALIST-TEAM-20261125` extends `ADR-MERGE-DELEGATION-20261123`
  to P. P squash-merges its own PR with `--match-head-commit` once the
  delegation's conditions hold, following the PR guard's merge-lane protocol.
- Keep host prototype, host simulation and native QEMU evidence separate.

## Current cycle

- task: P04-T1 native LP-25 interrupted purge (criterion P04)
- branch: `claude/p04-t1-interrupted-purge-20261125`; PR number, tested
  source SHA and hosted runs are in the pull request description
- delivered: `tools/test_nas.py` stages `interrupt` (NAS exits between evict's
  volume WRITE and its history PURGE; one Revoked 96-byte copy retained) and
  `repair` (first recover refuses with `ReopenRequired`, then
  `VolumeStorage::finish_cleanup` purges history, then recovery). The repair
  stage boots only on the exact passing interrupt image. Contract:
  `services/nas/native-probe/CONTRACT.md` "Interrupted purge and repair".
- evidence limit: a controlled NAS process exit with the volume service alive.
  Not power loss, torn writes, disk errors, the revoke-PURGE window, remote
  sessions or physical erasure.
- ledger: `docs/roadmap-pipeline/ledger.json` P04-T1 is lead/Codex-owned; the
  proposed write set, checks and acceptance are in the pull request for the
  lead to record.

- state (25 Nov ~17:30Z): PR #533 is ready for review at f7fa786f (main
  a571d87c), and the local native gate passes there (4/4, raw exit 33). The
  GitHub incident reruns were accepted at 16:34Z (runs 45644871049 coverage,
  45644871306 CodeQL). merge-hold failed only on a wait timeout (unchanged
  head), so rerun it once after its pending runs finish. Reviews are handled:
  the CodeRabbit race is fixed, and the AI review and Graphify findings are
  answered. Codex and Copilot have no quota, so they are not retriggered.
- P0 CI-first mode (owner, ~17:25Z): main is frozen while CI batch #544 is
  verified. No push that triggers hosted CI. Lane order after #544:
  #515 → #539 → #533. This handoff commit (local, ahead of origin) goes out in
  the single catch-up push when #533 gets the lane.

## Next

1. Wait for the PR guard's lane signal for #533. Then merge main into the
   branch, re-run the native gate (`python3 -B -u tools/test_nas.py` with
   separate NOS_BUILD_DIR/NOS_TARGET_DIR), and make one push. Rerun only infra
   failures, then squash with `--match-head-commit <new head>`.
2. Reconcile P01–P03 open rows (P03-T1 waits on the owner's authority-model
   decision) and pick the next native subtask.
