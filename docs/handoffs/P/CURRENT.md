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
  head). At ~18:10Z main was de242a3f (#542 merged) and #533 was BEHIND, so
  merge-hold is not rerun on f7fa786f: the catch-up head re-runs it with the
  #542 poll fix. Reviews are handled:
  the CodeRabbit race is fixed, and the AI review and Graphify findings are
  answered. Codex and Copilot have no quota, so they are not retriggered.
- P0 CI-first mode (owner, ~17:25Z; corrected ~18:05Z): CI keeps priority
  until all CI batches are resolved, so the main freeze continues after #544
  lands. #533 gets the lane only when a CI PR is genuinely blocked (non-infra
  failure, 15 min without progress, or 60 min) or the CI exit criterion is
  met; order #515 → #539 → #533. No push that triggers hosted CI. Local
  handoff commits stay local and go out in the single catch-up push when the
  PR guard grants #533 the lane.

## Local stacked cycle: P02-T3 slice 1 (not pushed)

- branch `claude/p02-t3-nasp-head-20261125`, worktree
  `~/projects/BaryonOS-worktrees/p02-t3-head`, stacked on this branch because
  it touches the same probe files. Local only under the CI-first freeze; it
  becomes its own PR after #533 lands (merge main in, never rebase).
- row reconciliation (25 Nov ~18:30Z, origin/main de242a3f): P01-T2 waits on
  the owner's context-mapping disposition; P01-T3 waits on P01-T2, B02-T1/T3
  and N03-T3 (all blocked); P03-T2 waits on the owner's authority model
  (P03-T1). P02-T3 is the only P row without an owner or cross-track gate.
- proposed spec for the lead (ledger P02-T3 is skeletal):
  - goal: slice 1 adds NASP opcode 15 Head, delegating to the existing
    `Service::head`; slice 2 adds a native guest stage that edits offline,
    survives a NAS cold restart and reconciles via Head; slice 3 (guest
    checkpoint persistence) needs a guest storage grant and is an OWNER
    question before any code.
  - invariant: Head is read-only, session-scoped and caller-bound by the
    kernel-supplied sender; staging is invisible; no new capability and no
    core, snapshot, volume or checkpoint format change.
  - write set (slice 1): `services/nas/native-probe/src/lib.rs`,
    `services/nas/native-probe/tests/ipc.rs`,
    `services/nas/native-probe/CONTRACT.md`.
  - acceptance: probe `cargo test`, `clippy -D warnings` and `fmt --check`
    (1.90.0); native `clippy --features native --bins` (1.100.0); probe and
    NAS Python unittests; the unchanged four-stage native gate.
  - local result (b5d1221f): probe host tests 8+2+3 pass, clippy/fmt pass,
    native clippy pass, Python unittests OK, native gate 4/4 raw 33
    (normal, recover, interrupt, repair; source 72b890a4…).
  - evidence limit: host dispatch tests plus a native regression run; slice 1
    makes no native Head call and no offline reconcile in QEMU.

## Next

1. Wait for the PR guard's lane signal for #533. Then merge main into the
   branch, re-run the native gate (`python3 -B -u tools/test_nas.py`; its
   NOS_BUILD_DIR must lie inside the checkout, e.g. `build/<name>`, or
   `private_path` refuses it), and make one push. Rerun only infra failures,
   then squash with `--match-head-commit <new head>`.
2. Locally: P02-T3 slice 2 (native offline edit, NAS cold restart, reconcile
   via Head) on the stacked branch.
