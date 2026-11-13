# L0 specification — K06 step 5c-2: BSP timer, idle with `sti; hlt` and the spin window on the BSP

> Historical L0 specification, written on 6 November 2026 and ported to `main` on 13 November 2026 from PR #232
> (head `dcd80d1`). English translation; the Dutch original is archived in `docs/history/k06-branch-prs-2026-11-06/k06-steps/`. The step has
> since been implemented; K06 was accepted on 13 November 2026 (`docs/verification/K06-ACCEPTANCE.md`), and later
> decisions (ADR-079/ADR-081 addenda, ADR-082) amend some limits. Statements about branches, worktrees and the state
> "on `<commit>`" describe that date.

**Date:** 6 November 2026 · **Base:** the head of step 05 (F19/F15) [not yet built]
**Branch/worktree:** `feat/k06-5c2-bsp-timer` in `/home/simon/projects/nieuw-os-k06-5c2`
**Spec:** `K06-SMP.md` §3.1 steps 3–6, §3.3, §4.3, §6.5, §9.6; ADR-045 (Sys3), -053, -073, -074;
audit findings F6, F9, F11 and F13.

## 0. Established facts (on `7ffdacfa`)
- The PIT IRQ0 runs via the 8259 PIC (`m3.rs:1904-1908` programs the PIC). [to be verified: whether the
  PIC is initialised in `native-smp`, whether IRQ0 is masked there, and whether the PIC can deliver to an AP.
  In virtual-wire mode it delivers only to the BSP LINT0.]
- AP timer: vector 0xd0, `TIMER_INITIAL = 20000`, divisor 16, uncalibrated (F9).
- 0xe8 jumps to the timer path and therefore also counts as a tick (F11).

## 1. Problem (why now)
The BSP has no preemption, no idle with interrupts and no spin window. As a result:
- the BSP cannot share ring-3 work fairly (step 08);
- the BSP does not receive 0xe0 if it spins on `KERNEL` while an AP starts a round, or on
  a change in the kernel half. The consequence is a **false quarantine of all APs** (F6);
- a deferred 0xe8 is lost if the timer is masked (F11).

## 2. Invariants (normative)
- **BT-1 One clock.** The PIT IRQ0 remains the only source of the monotonic clock and delivers only to the BSP.
  The LAPIC timer delivers only preemption ticks. *Falsified if* Sys3 runs faster or slower than the PIT,
  or if an AP receives an IRQ0.
- **BT-2 BSP equal to an AP.** The BSP receives vector 0xd0 with the same divisor as the APs, plus the
  INV-5c deferral and the spin window (ADR-074 S1/S3). S3 is permitted only when **every** unmasked
  source on the BSP has an installed IDT gate (IRQ0, 0xd0, 0xe0, 0xe8, 0xff).
- **BT-3 Idle.** `sti; hlt; cli` only in the idle thread on the kernel CR3, with an empty lockdep.
- **BT-4 Reschedule flag instead of tick.** 0xe8 gets its own counter and sets `need_resched`. A
  0xe8 that is deferred by INV-5c remains recorded as `need_resched`. The release of the last
  lock (lockdep becomes empty) checks the flag and then switches at the next safe point
  (return to user mode or idle). Ticks and rescheds are counted separately (F11).
- **BT-5 Calibrated band.** At boot the BSP counts LAPIC ticks per PIT tick over 100 ms and logs
  `SMP:TIMER:RATIO:<r>`. The gate requires a band [min, max] that is stated in the ADR in advance and is not
  adjusted afterwards.
- **BT-6 Contract for kernel threads** (F13): record in the ADR that fixture kernel threads run with IF=1
  as "pseudo-user". Real kernel code remains IF=0 (spec §4.3).

## 3. Interface
- `smp.rs`: `bsp_timer_init()` after the M2 init and before `SMP_SCHED_GO` is released;
  `CpuLocal.need_resched: AtomicBool`, with `resched_count` and `tick_count` kept separately.
- `lock.rs`: the window predicate becomes `level <= ASPACE && irq_depth == 0 && this_cpu().irq_ready`
  (instead of `on_ap()`); `irq_ready` is set per CPU as soon as BT-2 holds.
- Idle thread per CPU (from 04), now also on the BSP.

## 4. Native fixture and negative tests
- **Clock:** during the existing SMP load, Sys3 remains monotonic and rises within 1% of the
  PIT expectation. Marker `SMP:CLOCK:PASS`.
- **BSP preemption:** two kernel threads on the BSP with register canaries, as in step 4 on the APs.
  Marker `SMP:BSP_PREEMPT:PASS`.
- **BSP as shootdown target:** an AP holds `KERNEL` and starts a round with only the BSP as target;
  the BSP spins on `KERNEL`. Requirement: ack within 100 ms, no quarantine. Marker `SMP:BSP_SPIN:PASS`.
- **Idle wake:** the BSP is idle; an AP wakes it with 0xe8; the wake latency is logged. Marker
  `SMP:IDLE:PASS`.
- **Deferred resched:** 0xe8 during a lock wait with the timer masked → the thread switches after
  the release. Marker folded into `SMP:IDLE:PASS`.
- `test-smp-bsp-spin-irq-off` (control profile): the BSP window is off → the round to the BSP must
  produce a timeout (`SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=0`).
- Regression: `native-rtc` 5/5 (CMOS ownership does not change) and the monotonic clock check.

## 5. Write files
`kernel/src/m2/smp.rs`, `kernel/src/m2/lock.rs`, `kernel/src/m2/arch.rs` (IDT gates on the BSP, if they
are located there), `kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`, `tools/tests/test_smp_serial.py`,
`.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (new ADR, or an addendum to ADR-073:
BSP timer, calibration band, need_resched), `docs/implementation/K06-SMP.md` (§3.1 and §6.5),
`docs/PROJECT-STATE.md`, progress files.

## 6. Acceptance
- `tools/gate.sh` exit 0; fmt; clippy; `test_smp_serial.py`.
- Native: `native-smp` (all cases, 1/2/4/8 vCPUs), `test-smp-bsp-spin-irq-off`, `native-rtc` 5/5,
  `milestone-m2`, and the K05 regression on 1 CPU.
- K06 remains open (41/100).

## 7. Out of scope
TSC deadline, tickless idle, IOAPIC routing of device IRQs to APs, power management (H05).
