# Workstream A handoff (Applications, SDK, Installation and Ecosystem)

Owner session: BaryonOS - Specialist A (session ref `f2d7d9`, resumed 2026-11-25 ~18:15Z;
the earlier `local_163a86f0` session is retired).
Coordination: BaryonOS - PR Bewaker. Merge mandate: recorded on main in
ADR-SPECIALIST-TEAM-20261125 clause 2 (#527, 9cc1b86f), which extends
ADR-MERGE-DELEGATION-20261123 to A; squash with --match-head-commit after the
required checks and merge-hold pass. Report the kernel write set to
the PR Bewaker before committing under `kernel/` (S and H write `volume.rs`).
Last updated: 2026-11-25 (Europe/Amsterdam).

## Criteria status (from docs/PROGRESS.md at origin/main 5846e1fb)

| Criterion | Status | Ledger tasks |
|---|---|---|
| A01 SDK and API | accepted | — |
| A02 install/update | open | A02-T1 verified (host); A02-T2 blocked (owner: source backend/anchor/actor, APP-SOURCE-RECOVERY.md §6/§8); A02-T3 blocked (kernel dynamic launch ADR, B02 store) |
| A03 permissions | open | A03-T1 verified (host task-grant-core); A03-T2 blocked on trusted dialog (U) — native broker half is A-executable |
| A04 everyday apps | open | A04-T2 blocked (owner engine decision; N02–N04); A04-T3 blocked (H03 audio, S09, P01) |

## Done

- A03-T2a native task-grant broker gate: MERGED as #531 (aa6d1c70, 25 Nov).
  A ring-3 broker authorises by the kernel-supplied IPC sender PID; the trusted
  decision is a labelled kernel bootstrap fixture standing in for U's dialog.

## Active subtask: A03-T2b grant retirement on process exit (#535)

Goal: grants die with their process. The broker learns client exit from a
kernel-written exit notice (labelled fixture in the broker's private page, as
in nas.rs) and calls `retire_process`; a forged retire request is denied.

Acceptance: `python3 -B tools/build.py --kernel-feature native-task-grant` +
`python3 -B tools/run_task_grant.py --case normal` and
`test-task-grant-denied` + `--case denied` reach raw QEMU exit 33 accepted
(normal includes FORGED_RETIRE_DENIED and EXIT_RETIRES); a mutation that skips
retirement fails the gate (FAIL:EXIT_RETIREMENT); broker host tests 10/10
(`cargo test --locked -p nieuw-os-task-grant-broker-example` in `sdk/`);
Clippy `-D warnings` and fmt pass; `native-sdk-app` normal still accepted.

Write set: `sdk/examples/task-grant-broker/**`, `kernel/src/m2/m3/task_grant.rs`,
`tools/run_task_grant.py`, the A handoff and A assessments.

Evidence limit: no trusted dialog, no Materiaal volume enforcement, exit
notice is a fixture; no full A03 acceptance.

## Branch and PR

- Worktree: `~/projects/BaryonOS-worktrees/a-track-t2b`
- Branch: `claude/a03-t2b-retire-20261125`; PR #535 (draft, base main);
  tracking issue #536. Remote head 228adee8; local HEAD is ahead with the main
  catch-up merge 8b50895b and handoff commits, deliberately NOT pushed.
- Merge mandate: ADR-SPECIALIST-TEAM-20261125 clause 2 (#527, 9cc1b86f); squash
  with --match-head-commit after the required checks and merge-hold pass.
- Toolchain is 1.100.0 per rust-toolchain.toml (root AGENTS.md text still says 1.90.0).

## Coordination notes

- PR guard 25 Nov: m3.rs dispatch lines (~93/~1025/~1384/~1874/~1977) do not
  overlap S N38 (fn event ~2320–2340); no need to wait for N38. Rebase on main
  before opening the PR (shared Cargo.toml/build.py blocks with #515, S N38,
  H05-T5c) and report final line ranges in the PR.
- #519 merged as 3b9a94c5: rebase a-track onto it before committing. D3 policy:
  an unchanged score gets a new `docs/progress/assessments/<YYYY-MM-DD>-<slug>.json`
  instead of editing the docs/progress.json header; a register change still
  needs `progress.py --write` plus the views.

- CI D2 review of #531 ci.yml at 3075c1b3: no blocking findings; CI confirmed
  it still holds for later heads with no `.github` delta. Conditions:
  hosted qemu-smoke, host-quality and merge-hold pass on the head; any new head
  needs a ci.yml-only delta check by CI before merge.
- A03-T2b (grant retirement on process exit) is implemented in
  `~/projects/BaryonOS-worktrees/a-track-t2b`, branch
  `claude/a03-t2b-retire-20261125`, as a stacked draft PR on #531. The exit notice is a
  kernel-written word in the broker's private page (labelled fixture, as nas.rs).
  Native normal/denied pass; a mutation that skips retirement fails the gate. Retarget it to main after
  #531 merges.

## Merge-lane status (25 Nov ~17:05Z)

- Merge-lane protocol (owner P0, 16:00Z) and temporary CI priority (owner,
  ~17:00Z) apply: CI optimization PRs (#542) get the lane first; no catch-up
  pushes outside the lane; infrastructure failures get `gh run rerun --failed`
  only.
- #531 MERGED 25 Nov ~17:25Z as aa6d1c70 (squash at 1b0efa25, CI D2 GO,
  all gates green on first attempt).
- #535 (draft) is based on main. Its add/add conflicts after the #531 squash are
  resolved LOCALLY (merge commit 8b50895b plus this handoff; main's A files equal
  #531's, so T2b's versions win). Validated locally: broker tests 10/10, clippy,
  fmt, progress --check, native task-grant normal/denied and native-sdk-app
  normal accepted. NOT pushed: the PR guard holds it under CI-first until #535 is
  named lane holder (queue after #515, #539, #533, #543). Owner correction
  ~18:05Z: CI keeps priority until all CI batches are resolved, so the main
  freeze continues after #544; the lane comes only via the safety valve (a
  genuinely blocked CI PR) or the CI end criterion. Then merge current
  main once more, re-validate, push once, ask CI for the ci.yml delta, merge.

- 25 Nov: main is c04ea98d (#544 landed). `git merge-tree` dry run of
  c04ea98d into local d1475f34: clean, no conflicts. T2c-1 is developed in the
  separate worktree `~/projects/BaryonOS-worktrees/a-track-t2c1` (branch
  `claude/a03-t2c1-volume-broker-20261125`, stacked on d1475f34), so #535 stays
  'merge main, re-validate, push once'. S was asked to agree to the broker
  being the sole volume domain-0 client (T2c-2 prerequisite).

- 25 Nov 18:38Z: **T2c-1 done locally** (d0b034c2 + db76a416) on
  `claude/a03-t2c1-volume-broker-20261125` (stacked on d1475f34; NOT pushed,
  no PR yet). Host: broker tests 22/22 (request shape pinned: STAT/READ only,
  known object, READ selector 0), clippy, fmt, task-grant-core 12/12; native
  regression native-task-grant normal + denied raw exit 33 (source 5a0431f3).
  Assessment `2026-11-25-a03-volume-read-broker-host.json`, score 43 unchanged.
  Open: native bin still on a labelled FixtureVolume; a failed READ revokes all
  holders' grants on that object (fail closed; Stale + failed/not-newer re-STAT
  now tested). Submit as its own PR after #535 merges (retarget to main).
- T2c-2 inputs: S read-only interface + seeded-read image 6206b8c8 (T2C-SPEC.md).
  T2c-3 waits for the owner's A/B/C choice (A recommended; PIDs are monotonic,
  `crates/kernel-core/src/user.rs:147`, so no PID-reuse risk for A).
- Native runs: NOS_BUILD_DIR must lie inside the checkout (default `build/`).

## Next action

1. Wait for the PR guard's explicit lane grant for #535 (CI-first freeze).
2. Merge current main into the branch once (`git fetch origin main`, merge
   FETCH_HEAD; no rebase, no force push).
3. Re-validate per the acceptance above.
4. Push once.
5. Ask CI to confirm the ci.yml delta (D2 carry-over) for that exact head.
6. When all required checks and merge-hold are green: mark ready and squash with
   --match-head-commit at that head; report the merge to the PR guard.

Then A03-T2c, specified in `docs/handoffs/A/T2C-SPEC.md` (read-only analysis,
25 Nov): T2c-1 host-only broker over `volume_core::client::Transport` (no kernel,
no owner decision; may be developed locally on a branch stacked on #535);
T2c-2 native `native-system` variant without `volume.rs` edits (shared
`m3.rs`/`Cargo.toml`/`build.py` ranges reported to the PR guard first);
T2c-3 product exit notification needs an OWNER decision (A reply-endpoint
closure recommended, B Process object, C kernel exit-event channel).
