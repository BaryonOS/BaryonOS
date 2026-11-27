# Specialist K — current handoff

Compact resume pointer for the K workstream (kernel engineering, maintenance
and performance). It is not a backlog: criteria live in `docs/progress.json`
(K01–K07), tasks come from other specialists' kernel dependencies and GitHub.

## Scope and mandate

- **Scope:** `kernel/` development, maintenance and optimisation, and kernel
  dependencies of other specialists (project owner, 26 Nov 2026 ~21:25Z).
  The original K roadmap is complete; it is not reopened generically.
- **Continuous mandate (project owner, 26 Nov 2026 22:24Z):** K owns kernel
  maturity: scheduler and switches, IPC, memory, SMP and locks, IRQs and
  timers, idle and energy, boot, robustness, telemetry, technical debt and
  other specialists' kernel blockers. Priority order: user or performance
  gain > reliability and security > blockers for others > cost and risk >
  CI time. Measure targeted before and after; no broad audit and no
  speculative change. Escalate only fundamental architecture changes,
  material security risks or changes to binding contracts. With nothing
  demonstrably valuable left, K is on call.
- **Rules:** root `AGENTS.md`, `kernel/AGENTS.md`, Security Floor, D1–D3
  review policy, GR-12 for Graphify review findings, autonomous merge queue
  (own merge-ready PRs, PR guard coordinates), handoff and CLEAR_READY at
  70/75/80/85 % of a 400K window.
- **Gate preserved:** K03 (isolated preempted processes) covers the
  single-CPU scheduler; its regression gates are the `milestone-m3` smoke,
  the `test-m3-*` fault profiles and `m3-single-regression`.

## Current cycle

- Session "BaryonOS - Specialist K", registered with "BaryonOS - PR Bewaker".
- Branch `claude/k-sched-continue-20261126`, worktree
  `~/projects/BaryonOS-worktrees/k-sched`, base `main` `bbf4df38` (contains
  #573 `40c10d3a`, #588 `295cb307`, #538 `ac495b2f` and #582 `a96963e9`).
- Scratch: `~/projects/BaryonOS-scratch/k/` (`RESUME.md`, design note
  `C3-DESIGN.md` revision 2, `C3-IMPL-SPEC.md`, batch logs).

## Task: R02-C3 single-CPU scheduler continuation (approved)

- The project owner approved design revision 2 on 26 Nov 2026 22:22Z with
  the recommended options D1–D6 (continuation; keep the `sched::Quantum`
  parity check; §5 kill criterion; qsim-job fixture fix first; new case
  `test-m3-syscall-spin`; exact mode-0 boundary of 4 unchanged).
- **Landing gate:** the code branch is cut from `main` only after #573 (B,
  keyring LOCK), #538 (L, native AI job) and #588 (L, D4: qsim-job yields on
  EAGAIN, head `52035629`, local `native-qsim-job` pass) have landed. The
  PR guard reports the landing SHAs. The ADR goes on top of
  `docs/DECISIONS.md` only after #582. Coordinate `m3.rs` pushes with H
  (T5d-T) via the PR guard.
- Write set: design note §6.

## Candidates (prioritised, 26 Nov 2026 22:29Z)

| ID | Area | Finding and evidence | Status | Next step |
|----|------|----------------------|--------|-----------|
| K-1 | Scheduler | R02 baseline: about 40–50 kernel entries per changed tile, each a switch; 0.35–0.39 ms per tile (`docs/verification/r02-window-baseline-20261126`). | Approved, waiting on landings | Implement C3 per design §3, test per §5. |
| K-2 | Memory, switches | Every M3 kernel entry loads the kernel root (`event()`, `kernel/src/m2/m3.rs:2105` at `d5022f9b`) and `select()` loads the user root (`m3.rs:1576`). PCID and PGE are off (`m3/heap.rs:186`, `paging.rs:1041`, `smp.rs:3795`), so each system call flushes the TLB twice, also for a caller that C3 continues. Validated in source; cost not measured. | Open, after K-1 | After C3: measure CR3 loads and phase times; then a design note. Any change (PCID with explicit invalidation, or no root switch on entry) alters the address-space isolation invariant: owner escalation expected. |
| K-1a | Scheduler (C3 delta) | #573's LOCK fixture advances per kernel entry (\`system_lock.rs:370-391\` via \`progress_observers\`). Under C3, stage-3 \`cleanup(KEY)\` can run in KEY's own next entry after KEY was marked Ready, so the continuation must re-check \`slot == current\`, Ready and not Dying at select time. \`LOCK_CANARY_ABSENT\` (\`:392\`) and #538's \`ENTRY_SWITCH_COUNTS\` (\`ai_job.rs:276-281\`) also depend on entry and restore counts. Subagent survey at PR heads; not re-verified by K. | Part of K-1 | Re-verify at the landing SHAs; add \`native-system-lock\`, \`test-system-lock*\` and \`native-ai-job\` to the §5 local list. |
| K-3 | Fixture limits | C3 §7.5 unchecked item: fixture guards `syscall_calls < 2_000_000` (`fpu.rs:230`, display and memory fixtures) under a higher call rate. | Part of K-1 | Check during C3 verification. |
| K-4 | SMP CI time | `m3-ipc-four` hosted timeout: IPC phase about one round per tick behind CPU-bound canaries; no kernel defect shown; the timeout is an owner decision (Graphify note, 11 Nov 2026). | Parked | None unless a new failure appears. |
| K-5 | SMP locks | A CPU that dies while waiting on a ticket lock is not detected (K06 F19, open point 4). SMP only; no consumer report. | Parked | Only on a reported failure. |
| K-6 | SMP quarantine | `held_by` coverage for ASPACE/FRAMES/SHOOTDOWN: moot, only `KERNEL` is a global `IrqLock` on `main` (`lock.rs:111`). | Closed | None. |
| K-7 | SMP timing | `wait_us` (`kernel/src/m2/smp.rs:761-782`) returns `false` after 2,000,000 polls of port 0x61, but `within_ms` (`:1583`) and `within_progress` (`:1638`) ignore that result, so a capped poll silently shortens every PIT-based deadline. Low risk under TCG and KVM; unmeasured (survey of 27 Nov 2026 at `20d9089f`). | Open, after K-1 | Make a capped `wait_us` fatal or retried; SMP native gates. |

Not K: #545 (window-session reporter, UI-U, fix PR #586) and R's #578
follow-ups 1 and 3 (ingress batch time versus the 200-tick budget, userspace).

## Next action (27 Nov 2026 ~10:35Z)

1. C3 is implemented on the branch: probe pin fix, `test-m3-syscall-spin`
   fixture and gate, and the scheduler change. The red control fails at the
   new assert; the green case passes both mode-0 rounds.
2. Remaining local evidence: the design §5 native list (ipc-wait, fpu,
   memory, qsim, qsim-job, sdk-app, system-lock and its `test-system-lock*`
   cases, window session scale 1 and 2, NAS probe, Materiaal), then the
   5+5 A/B against `main` at the landing SHA. Announce QEMU runs and the A/B
   to R first.
3. ADR-K-SCHED-CONTINUE-20261126 on top of `docs/DECISIONS.md` (#582 has
   landed), CI D2 delta review of the `native-smp.yml` and `tools/test_smp.py`
   hunks, then ready and enqueue.
4. Then K-2 measurement.
