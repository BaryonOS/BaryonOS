# L0 specification — K06 step 5d-2: M3 ring-3 processes on APs

> Historical L0 specification, written on 6 November 2026 and ported to `main` on 13 November 2026 from PR #232
> (head `dcd80d1`). English translation; the Dutch original is archived in `docs/history/k06-branch-prs-2026-11-06/k06-steps/`. The step has
> since been implemented; K06 was accepted on 13 November 2026 (`docs/verification/K06-ACCEPTANCE.md`), and later
> decisions (ADR-079/ADR-081 addenda, ADR-082) amend some limits. Statements about branches, worktrees and the state
> "on `<commit>`" describe that date.

**Date:** 6 November 2026 · **Base:** the head of step 07 (5e) [not yet built]
**Branch/worktree:** `feat/k06-5d2-ring3-smp` in `/home/simon/projects/nieuw-os-k06-5d2`
**Spec:** `K06-SMP.md` §2 (INV-1 to INV-7), §3.1 step 5, §3.2, §6.1–6.4, §7.3, §7.4, §8 (5d), §9.1;
ADR-045 (FXSAVE), -053, -060, -074, -075, -083 and the ADRs of steps 02–07.

## 0. Preconditions (all previously merged or stacked, and natively green)
F8 (swapgs), 01 (KERNEL lock and scratch per CPU), 02 (lockdep with context and pairs), 03 (stuck and
serial), 04 (block/wake/kill), 05 (ICR and quarantine), 06 (BSP timer and idle), 07 (process-wide
shootdown). If any one of these is missing, this step **does not start**: record that as a blocker in PROJECT-STATE.

## 1. Problem (why now)
`kernel/src/main.rs:29` excludes M3 from `native-smp`. After the self-tests all APs are parked.
No AP performs production work. K06 requires that processes genuinely run on multiple CPUs.

## 2. Invariants (normative)
- **R3-1 Only the known path.** M3 runs in a new profile `native-smp-m3`. `native-smp` remains
  the bring-up profile; the `compile_error!` on `main.rs:29` is lifted only for `native-smp-m3`.
  All other profiles remain single-CPU.
- **R3-2 One thread per process.** As long as M3 has no threads, the M3 process is exactly one
  scheduler thread with `home_cpu` = the online CPU with the shortest queue at spawn (lowest index on a
  tie). No automatic migration (spec §6.2).
- **R3-3 FPU in the switch** (ADR-045, spec §6.3 steps 3 and 8): FXSAVE after the last Rust code of the
  old context, FXRSTOR directly before the GPR pops/IRETQ. The existing code is in
  `kernel/src/m2/arch.rs:658/674` and `kernel/src/m2/m3/fpu.rs`; it is reused, not rewritten.
- **R3-4 Entry under the lock.** Every M3 entry (syscall, fault, IRQ0 path with process impact) acquires
  `KERNEL` (01). An AP uses `swapgs` in accordance with ADR-083, and the TSS.RSP0 of that CPU points to the
  kernel stack of the running thread (spec §6.3 step 6).
- **R3-5 IPC across CPUs.** The ADR-053 wait uses the block/wake protocol from 04. A reply to
  a thread on another CPU sends 0xe8. A copy between two address spaces acquires
  `lock_pair(ASPACE[a], ASPACE[b])` (02).
- **R3-6 Usercopy under `ASPACE`** (INV-6, spec §7.4): validation and copy take place under the lock of
  the target space.
- **R3-7 Quotas unchanged.** All K07 quotas (handles, delegation depth, IPC queue, heap, CPU deadline)
  apply per process, not per CPU. *Falsified if* a process on two CPUs together receives more than its
  quota.

## 3. Interface
- Profile `native-smp-m3` in `kernel/Cargo.toml` and `tools/build.py`; runner `tools/run_smp.py` gets
  M3 cases, or a new `tools/run_smp_m3.py` [choose whichever follows the existing runner convention].
- `m3.rs`: `State.current` becomes per CPU (`this_cpu().current`); the M3 scheduler places threads in
  `RUNQ[home_cpu]`. [to be verified: all places that read `state.current`; `graphify_callers` on
  `m3_event` and the scheduler functions.]
- AP startup: after `SMP_SCHED_GO` an AP picks work from its own queue instead of parking.

## 4. Native fixture and tests
- **K05 on SMP:** the supervisor test (K05, 6/6) in `native-smp-m3` on 4 vCPUs, with services spread
  across the CPUs. Requirement: the same outcomes and cleanup balance as on 1 CPU.
- **User work on every CPU:** `SMP:SCHED:CPUS=<n>` with, per CPU, user ticks > 0 and ≥ 4 switches.
- **IPC ping-pong** between two processes on different CPUs, 10 000 rounds, without a lost
  reply. Marker `SMP:IPC:PASS`.
- **FPU canary:** two processes on the same CPU with different x87/SSE values; after 1000 switches
  the values are intact.
- **Ring-3 GS** (F8 N1–N3): again in `native-smp-m3`, now with real M3 processes.
- **Negative:** an app fault on AP2 stops only that process; the other processes on all CPUs
  keep running (K03 under SMP).

## 5. Write files
`kernel/src/main.rs` (only the cfg for the new profile), `kernel/src/m2/m3.rs`,
`kernel/src/m2/smp.rs`, `kernel/src/m2/arch.rs` (only if TSS/RSP0 is located there),
`kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py` (or a new runner),
`tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (new ADR:
M3 on SMP), `docs/implementation/K06-SMP.md` (update §8), `docs/PROJECT-STATE.md`, progress files.

## 6. Acceptance
- `tools/gate.sh` exit 0; fmt; clippy; the serial oracle.
- Native `native-smp-m3` on 2/4/8 vCPUs (single- and multi-thread TCG) and `native-smp` (all existing
  cases).
- **Single-CPU regression unchanged:** `milestone-m2`, K05, K07 (`resource-pressure.yml`), `native-rtc`,
  `run_ipc_wait.py`, `run_fpu.py`.
- K06 remains open (41/100): the §9 matrix and the stress tests follow in 09.

## 7. Out of scope
Migration and load balancing (09), multiple threads per process, device IRQs to APs, the GUI and
M6 profiles on SMP.
