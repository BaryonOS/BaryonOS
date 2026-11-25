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
- Merge: P is not named in `ADR-MERGE-DELEGATION-20261123` (six workstreams).
  P opens and repairs draft pull requests only and never merges until a
  delegation naming P is recorded on `main`.
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

## Next

1. Repair hosted CI on the P04-T1 pull request if it fails.
2. Reconcile P01–P03 open rows (P03-T1 waits on the owner's authority-model
   decision) and pick the next native subtask.
