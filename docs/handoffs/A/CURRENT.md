# Workstream A handoff (Applications, SDK, Installation and Ecosystem)

Owner session: BaryonOS - Specialist A (`local_163a86f0-2d5c-421d-a9dc-10cda1c5a245`).
Coordination: BaryonOS - PR Bewaker. Merge mandate: owner extended
ADR-MERGE-DELEGATION-20261123 to A on 25 Nov ~11:25Z; recording follows in a CI
governance PR after #519. Work draft-only until then; report kernel write set to
the PR Bewaker before committing under `kernel/` (S and H write `volume.rs`).
Last updated: 2026-11-25 (Europe/Amsterdam).

## Criteria status (from docs/PROGRESS.md at origin/main 5846e1fb)

| Criterion | Status | Ledger tasks |
|---|---|---|
| A01 SDK and API | accepted | — |
| A02 install/update | open | A02-T1 verified (host); A02-T2 blocked (owner: source backend/anchor/actor, APP-SOURCE-RECOVERY.md §6/§8); A02-T3 blocked (kernel dynamic launch ADR, B02 store) |
| A03 permissions | open | A03-T1 verified (host task-grant-core); A03-T2 blocked on trusted dialog (U) — native broker half is A-executable |
| A04 everyday apps | open | A04-T2 blocked (owner engine decision; N02–N04); A04-T3 blocked (H03 audio, S09, P01) |

## Active subtask: A03-T2a native task-grant broker gate

Goal: first native QEMU evidence that helpers cannot bypass task grants.
A ring-3 broker authorises each request by the kernel-supplied IPC sender PID
through `sdk/task-grant-core`; app-supplied identity, object numbers and handles
never authenticate. The trusted permission decision is a labelled kernel
bootstrap fixture standing in for the U-owned trusted dialog.

Acceptance: `python3 -B tools/build.py --kernel-feature native-task-grant` and
`python3 -B tools/run_task_grant.py --case normal` (and `test-task-grant-denied` /
`--case denied`) reach the TASKGRANT markers with raw QEMU exit 33; the
`native-sdk-app` gate still passes; SDK/kernel test, Clippy and fmt pass.

Write set: `sdk/examples/task-grant-broker/**`, `kernel/src/m2/m3/task_grant.rs`
plus the feature/dispatch lines, `tools/build.py` (one branch),
`tools/run_task_grant.py`, `.github/workflows/ci.yml` (one job; coordinate with
the CI session), CI pin tests if required.

Evidence limit: no trusted dialog, no Materiaal volume enforcement, no full A03
acceptance.

## Branch and PR

- Worktree: `~/projects/BaryonOS-worktrees/a-track`
- Branch: `claude/a-track-20261125`
- PR: draft, see the branch on GitHub (self-merge only after CI's governance PR
  records the A mandate and the guard's merge hold allows it).
- Verified on head rebased onto 3b9a94c5: native-task-grant normal and
  test-task-grant-denied QEMU raw 33 accepted; native-sdk-app normal still
  accepted; broker host tests 7/7; progress.py --check exit 0.
- Toolchain is 1.100.0 per rust-toolchain.toml (root AGENTS.md text still says 1.90.0).

## Coordination notes

- PR guard 25 Nov: m3.rs dispatch lines (~93/~1025/~1384/~1874/~1977) do not
  overlap S N38 (fn event ~2320–2340); no need to wait for N38. Rebase on main
  before opening the PR (shared Cargo.toml/build.py blocks with #515, S N38,
  H05-T5c) and report final line ranges in the PR.
- #519 merged as 3b9a94c5: rebase a-track onto it before committing. D3 policy:
  an unchanged score gets a new `docs/progress/assessments/<YYYY-MM-DD>-<slug>.json`
  instead of editing the docs/progress.json header; a register change still
  needs `progress.py --write` plus the views. A self-merge waits for CI's
  governance PR.

- CI D2 review of #531 ci.yml at 3075c1b3: no blocking findings. Conditions:
  hosted qemu-smoke, host-quality and merge-hold pass on the head; any new head
  needs a ci.yml-only delta check by CI before merge.
- A03-T2b (grant retirement on process exit) is implemented in
  `~/projects/BaryonOS-worktrees/a-track-t2b`, branch
  `claude/a03-t2b-retire-20261125`, as a stacked draft PR on #531. The exit notice is a
  kernel-written word in the broker's private page (labelled fixture, as nas.rs).
  Native normal/denied pass; a mutation that skips retirement fails the gate. Retarget it to main after
  #531 merges.

## Next action

Keep #531 and the T2b PR green. Merge #531 after governance PR #527 lands and
CI's conditions hold, then retarget T2b. Next A slice: bind the broker to real
Materiaal volume objects (per-object READ via the volume service; coordinate the
volume.rs boundary with S) and replace the exit fixture with a product
exit-notification path (kernel owner decision).
