# Specialist K — resume pointer

Compact resume pointer for the kernel engineering, maintenance and performance
workstream (K01–K07 in `docs/progress.json`). This file is not a backlog:
tasks come from kernel dependencies of other workstreams, issues and pull
requests. Newer ADRs and live evidence take precedence over it.

The canonical live state is the latest `HANDOFF K` comment on issue #614
(as of 27 November 2026:
[issuecomment-7217960025](https://github.com/BaryonOS/BaryonOS/issues/614#issuecomment-7217960025))
and the open K pull requests. This file plus GitHub is sufficient to resume;
local paths at the end are optional additions for a local session.

## Resuming

A resuming agent, whatever its client, first verifies branch, exact HEAD,
`git status --short`, ownership and the freshness of the latest `HANDOFF K`
comment against GitHub, and never redoes completed work. Graphify is optional
context; client-specific notes are adapters and hold no state.

## Scope, mandate and working agreements (recorded once)

- **Scope:** `kernel/` development, maintenance and optimisation, and kernel
  dependencies of other specialists (project owner, 26 November 2026). The
  original K roadmap is complete and is not reopened generically.
- **Continuous mandate (project owner, 26 November 2026):** K owns kernel
  maturity: scheduler and switches, IPC, memory, SMP and locks, IRQs and
  timers, idle and energy, boot, robustness, telemetry, technical debt and
  other specialists' kernel blockers. Priority: user or performance gain >
  reliability and security > blockers for others > cost and risk > CI time.
  Measure before and after; no broad audit and no speculative change.
  Escalate only fundamental architecture changes, material security risks or
  changes to binding contracts. With nothing demonstrably valuable left, K is
  on call.
- **Continuous optimisation (project owner, 27 November 2026, relayed by the
  PR guard):** K keeps maximising safe kernel performance, scalability and
  robustness without new permission requests inside the mandate; the
  Security Floor and D1–D3 stay unchanged.
- **Rules:** root `AGENTS.md`, `kernel/AGENTS.md`, the Security Floor, D1–D3
  review policy (CI files need a D2 delta review by CI), GR-12 for Graphify
  review findings.
- **Merge route:** K enqueues its own merge-ready pull requests in its own
  area under ADR-MERGE-QUEUE-AUTONOMY-20261126, which the project owner
  extended to all appointed specialists on 27 November 2026 (relayed by the PR
  guard; Specialist G records it). The PR guard coordinates and verifies
  landings. Never `--admin`, never auto-merge; CI pull requests go first.
  Before enqueueing: all applicable gates green, `merge-hold` green,
  `autoMergeRequest` null, no unresolved review threads, no queued or
  in-progress run on the head; enqueue with
  `gh pr merge <N> --squash --match-head-commit <sha>` and check
  `autoMergeRequest` again afterwards. A `merge-hold` failure with only
  `Pending:` lines means wait for its re-check; a base-tip failure means merge
  `main` once (no rebase) and push; `Failed:` means repair.
- **Gate preserved:** K03 (isolated preempted processes) covers the
  single-CPU scheduler; its regression gates are the `milestone-m3` smoke,
  the `test-m3-*` fault cases (`tools/gate_contract.py` `M3_CASES`),
  `test-m3-syscall-spin` (its own gate group
  `syscall-spin-continues-within-quantum`, not part of `M3_CASES`; it also
  gates the -8 rule through a polling second process) and
  `m3-single-regression` (`tools/test_smp.py`).

## Invariants and security boundaries

- Single-CPU scheduling follows ADR-K-SCHED-CONTINUE-20261126 and
  `docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md` §3: a continued
  caller never outlives its quantum (I1); yield, exit, termination, faults,
  a -8 (Again) result, non-`Ready` returns and the timer tick always select
  (I2); every entry runs
  the full selection housekeeping (I3); no ABI change (I4); the candidate is
  single-use (I5).
- CR4.PCIDE and PGE stay off and every M3 entry switches address space
  (contract §2). Changing that (candidate K-2) alters the address-space
  isolation invariant and needs an owner decision.
- No change to capability enforcement, key isolation or `crates/smp-lock`
  semantics without its own design and negative tests.

## Last cycle

| Field | Value |
|---|---|
| cycle_id | `K-20261127-sched-continue` |
| Task | R02-C3 (issue #530): single-CPU continuation after ordinary system calls; design revision 2 approved by the project owner on 26 November 2026 22:22 UTC (D1–D6); D3 judged met by the project owner on 27 November 2026 at about 12:03 UTC. Hosted step-level network regression on `6a74ad67`; project-owner decision received 13:26:52 UTC (verbatim on #602): no queue until a structural fix is verified. Amendment: a -8 (Again) result selects like a yield |
| Prerequisite | #604 (the K05 client waits for the HELLO receipt before exit), landed as `20c7e18a` |
| Pull request | #602, branch `claude/k-sched-continue-20261126`; this file lands with it |
| Write set | `kernel/src/m2/m3.rs`, `kernel/Cargo.toml`, `userspace/probe.rs`, `userspace/tests/test_probe.py`, `tools/gate_contract.py`, `tools/test_milestones.py`, `tools/test_smp.py`, `tools/tests/test_smp_host.py`, `tests/test_capture_gate.py`, `tools/build.py`, `.github/workflows/native-smp.yml` (step name only, D2), `docs/DECISIONS.md` (ADR), contract §3 sentence, `docs/verification/k-sched-continue-20261127/`, `docs/progress/assessments/2026-11-27-k-sched-continue.json`, this file |
| Evidence | `docs/verification/k-sched-continue-20261127/` (local QEMU TCG native list, local KVM A/B, and the A/B/C of the -8 amendment for the net system, net-link and window-session gates; not hosted acceptance). Hosted CI, the D2 review, the pre-registered criteria and the hosted-duration comparisons are recorded on #602 |
| Coordination | L: #598 landed before #602, so K re-measured `JOB_IRQS` on #602 (28–41 per run locally on `478b4ba3` and 35–50 on the fix head `d71dc04f`, inside the documented range; no code change). R: QEMU batches announced; its re-baseline waits on the hosted fix head. N: accepted the -8 rule on three conditions (exactly -8; scheduling only; hosted `native-network-service` run link on the fix head). H: informed for the net-link gate. CI: D2 delta review GO on 27 November 2026 (on `c1af50f8`; the CI-file hunks are unchanged since) |
| Score | 43/100 unchanged; K03 stays accepted under its regression gates |

## Candidates (prioritised)

| ID | Area | Finding and evidence | Status | Next step |
|----|------|----------------------|--------|-----------|
| K-1 | Scheduler | R02 baseline: about 40–50 kernel entries per changed tile, each a switch (`docs/verification/r02-window-baseline-20261126`). | Delivered by #602 (ADR-K-SCHED-CONTINUE-20261126) | None; a regression in the K03 gates reopens it. |
| K-2 | Memory, switches | Every M3 kernel entry loads the kernel root and `select()` loads the user root; PCID and PGE are off, so each system call flushes the TLB twice, also for a continued caller. Validated in source; cost not measured. | Open | Measure CR3 loads and phase times; then a design note with owner escalation. |
| K-3 | Fixture limits | Fixture guards `syscall_calls < 2_000_000` (FPU, display, memory) under a higher call rate. | Checked in the K-1 native list | None unless a guard trips. |
| K-4 | SMP CI time | `m3-ipc-four` hosted timeout: IPC phase about one round per tick behind CPU-bound canaries; no kernel defect shown; the timeout is an owner decision. | Parked | Only on a new failure. |
| K-5 | SMP locks | A CPU that dies while waiting on a ticket lock is not detected (K06 F19, open point 4). | Parked | Only on a reported failure. |
| K-8 | SMP scheduler | The SMP continuation rule (pre-C3) still continues a -8 (Again) result, so an SMP poller keeps its quantum while other CPUs run the producer. No regression shown; unmeasured. | Open | Measure an SMP net guest; if it matters, apply the same -8 rule with the SMP gates. |
| K-9 | Polling guests | The -8 rule removes the cost of a failed poll, but `sdk/examples/system` (`init.rs` `wait()`, the status loop) still polls. | Open (owners N and H) | Move those loops to `ipc_wait`; K advises. |
| K-10 | Timing | Window and input paths measure time in 10 ms ticks only. Proposal agreed with R: a `monotonic_us` system call from a calibrated invariant TSC (calibrated as `TSC_PER_MS` in `kernel/src/m2/smp.rs`), falling back to ticks, behind a manifest right for the compositor, the input path and R02 measurement builds; no user-mode `rdtsc`. | Proposed | ADR with R and B; ABI minor bump coordinated by K. |
| K-11 | IPC | Tile updates cross IPC in small messages; larger messages or shared-memory tiles may cut entries further. Unmeasured. | Proposed | Measure entries per tile after #602 lands; then a design note. |
| K-7 | SMP timing | `wait_us` (`kernel/src/m2/smp.rs`) returns `false` after 2,000,000 polls of port 0x61, but `within_ms` and `within_progress` ignore it, so a capped poll silently shortens a PIT deadline. Unmeasured. | Open | Make a capped `wait_us` fatal or retried; SMP native gates. |

## Acceptance commands and evidence limits

- Scheduler: `python3 -B tools/test_milestones.py --through m4`,
  `python3 -B tools/test_smp.py` (at least `m3-single-regression`),
  `python3 -B tools/test_supervisor.py` (and `--smp`), plus the native list in
  the verification directory. Only actual kernel markers plus raw exit 33
  substantiate a pass; lock cases pass on runner exit 0 with their documented
  raw codes.
- Several runners refuse a reused build directory (for example
  `tools/test_resource_pressure.py`); give every gate a fresh
  `NOS_BUILD_DIR`. `tools/test_smp.py` writes an untracked JSON file into the
  protected `docs/verification/sm-smp/` directory; move it out and never
  commit it.
- Performance claims need the 5+5 A/B on one host with the spread reported,
  labelled "local KVM, not hosted".
- Local results are not hosted acceptance; hosted CI on the final head is
  required before enqueueing.

## Cloud agent limits (GitHub Copilot cloud agent)

Hosted CI also runs QEMU under TCG, so single native gates are reproducible in
the cloud agent; the limits are wall time, KVM and parallel load.

- **Suitable:** host-side Rust and Python work (`crates/`, `tools/tests/`,
  unit tests, clippy, fmt), kernel builds, and one or a few native QEMU gates
  per task. Measured on a local TCG host on 27 November 2026: a kernel build
  takes 2–11 s with a warm cache, most native gates take 2–60 s, window
  session scale 1 about 1 min 51 s and scale 2 about 4–5 min.
- **Tight:** the full scheduler verification list (45 steps, including
  `tools/test_smp.py` with 16 profiles, `tools/test_supervisor.py` and
  `tools/test_milestones.py --through m4`) took 16 min 16 s sequentially
  under TCG with warm build caches. One pass fits; the cold-cache duration
  was not measured, and a fix-and-rerun cycle may not fit.
- **Not suitable within 59 minutes on one branch:** the SMP repetition
  suites (`SMP 50/50`, the 74×50 aggregate: thousands of QEMU runs that hosted
  CI spreads over matrix jobs) and any performance A/B, which needs KVM and
  one quiet host and is labelled "local KVM, not hosted". TCG timings are not
  performance evidence.
- **Not available:** KVM, physical hardware, Graphify (OAuth), local scratch
  notes, and pausing other specialists' native runs on a shared host.

## Local additions (optional)

- Local session notes: `~/projects/BaryonOS-scratch/k/RESUME.md` (newest
  section wins), design note `C3-DESIGN.md`, batch logs under `s5/` and `s6/`.
- Worktree used locally: `~/projects/BaryonOS-worktrees/k-sched`.
