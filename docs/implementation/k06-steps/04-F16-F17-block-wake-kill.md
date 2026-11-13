# L0 specification — K06 F16/F17: blocking, waking and killing across CPUs

> Historical L0 specification, written on 6 November 2026 and ported to `main` on 13 November 2026 from PR #232
> (head `dcd80d1`). English translation; the Dutch original is archived in `docs/history/k06-branch-prs-2026-11-06/k06-steps/`. The step has
> since been implemented; K06 was accepted on 13 November 2026 (`docs/verification/K06-ACCEPTANCE.md`), and later
> decisions (ADR-079/ADR-081 addenda, ADR-082) amend some limits. Statements about branches, worktrees and the state
> "on `<commit>`" describe that date.

**Date:** 6 November 2026 · **Base:** the head of step 02 (F3/F4) [not yet built]
**Branch/worktree:** `fix/k06-f16-f17-sched-protocol` in `/home/simon/projects/nieuw-os-k06-f16`
**Spec:** `K06-SMP.md` §2 (INV-3, INV-5, INV-7), §6.1, §6.3, §6.4; ADR-053 (IPC wait), ADR-074;
audit findings F16, F17 and F1 (the `on_cpu` protocol from #230 remains the basis).

## 1. Problem (why now)
- **F16:** `crates/smp-lock/src/sched.rs` has no transition RUNNING→BLOCKED, no DEAD or
  DEAD_PENDING, no kill and no idle thread. The IPC route from spec §6.4 has a
  **lost-wakeup race**: a thread decides to block while a waker on another CPU is already
  waking it. `wake` then fails with `Err(RUNNING)` and the reply is lost.
- **F17:** the run queue does not detect a double enqueue. The `double-run` test therefore relies on
  timing and not on the invariant.

## 2. Invariants (normative)
- **BW-1 No lost wake.** Blocking takes place in two phases: `RUNNING(c) → BLOCKING` (the
  wait condition is not yet true) and then, at the switch, `BLOCKING → BLOCKED`. A `wake` that
  sees `BLOCKING` sets `BLOCKING → READY_PENDING`. The switch sees this and puts the thread back to
  READY instead of BLOCKED. All transitions use `compare_exchange`.
  *Falsified if* the loom model finds an execution in which a wake was seen and the thread nevertheless
  ends up BLOCKED.
- **BW-2 Once in the queue.** Per thread a `queued` bit: `push` performs CAS 0→1, `pop` 1→0. A
  second push → `SMP:SCHED:DOUBLE_ENQUEUE` and stop. The double-run test therefore fails deterministically
  at the push.
- **BW-3 Kill is bounded.** Kill of a thread on CPU *d*: set `DEAD_PENDING`, send 0xe8 and wait
  for `DEAD` within `SHOOTDOWN_DEADLINE_MS`. *d* publishes `DEAD` at its next switch, after
  leaving the stack (`on_cpu` cleared, F1). No `DEAD` within the deadline → INV-5 path
  (step 05 determines the policy). Only after `DEAD` may the K05 cleanup and the shootdown (07) run.
- **BW-4 Idle per CPU.** Every CPU has a fixed idle thread that is never in a queue. If the
  queue is empty, idle runs on the kernel CR3 (spec §6.3 step 9).
- **BW-5 No lock across the switch** (INV-7, INV-5c): unchanged. Only RUNQ is handed over within the
  switch.

## 3. Interface
- `smp_lock::sched`: states `READY, RUNNING(cpu), BLOCKING, READY_PENDING, BLOCKED,
  DEAD_PENDING, DEAD`; functions `block_prepare`, `block_commit`, `wake`, `kill_request`,
  `kill_commit`; `RunQueue::push` checks `queued`.
- `smp.rs`: `sched_switch` handles `BLOCKING`, `READY_PENDING` and `DEAD_PENDING`. One idle thread
  per CPU in `CpuLocal`.
- Kernel feature `test-smp-resched-deaf` (the target AP ignores 0xe8 during a kill).

## 4. Host tests (before native)
- A loom model of block/wake with two threads and a waker on another "CPU": no lost wakeup.
- A loom model of kill during RUNNING: the cleanup never starts before `DEAD`, and `DEAD` comes only after
  `on_cpu` has been cleared.
- A double push always fails, regardless of timing.

## 5. Native fixture and negative tests (`native-smp`, kernel threads)
- Ping-pong: thread A on AP1 and thread B on AP2 block and wake each other 10 000× via a shared
  "mailbox" under `KERNEL`. Requirement: both counters 10 000, no timeout. Marker `SMP:WAKE:PASS`.
- Kill: the BSP kills a running thread on AP3; `DEAD` within the deadline, the stack is free and
  `active_cpus` of its space is 0. Marker `SMP:KILL:PASS`.
- `test-smp-double-run` (existing): now expects `SMP:SCHED:DOUBLE_ENQUEUE` at the push, in every run.
- `test-smp-resched-deaf`: kill to a deaf AP → the INV-5 path according to 05 (until 05 is done:
  `SMP:KILL:TIMEOUT:APIC=<n>` followed by quarantine).

## 6. Write files
`crates/smp-lock/src/sched.rs`, `crates/smp-lock/src/lib.rs`, `kernel/src/m2/smp.rs`,
`kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`, `tools/tests/test_smp_serial.py`,
`.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (new ADR: scheduler protocol),
`docs/implementation/K06-SMP.md` (update §6.1 and §6.4), `docs/PROJECT-STATE.md`, progress files.

## 7. Acceptance
- `tools/gate.sh` exit 0; `cargo test -p smp-lock` (including loom); clippy; fmt; `test_smp_serial.py`.
- Native: `native-smp` (all cases), `test-smp-double-run`, `test-smp-resched-deaf`, regression
  `milestone-m2`.
- K06 remains open (41/100).

## 8. Out of scope
The coupling with the real IPC wait queues of M3 (08), migration and load balancing (09), ring 3.
