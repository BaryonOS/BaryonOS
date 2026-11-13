# PIT clock investigation — design only, 10 November 2026

No source change, guest execution, Git mutation, or native acceptance is associated with this note. The elapsed-counter design below is not implemented. The existing full matrix, repetitions, deadlines, load, idle coverage, and exact 1% criterion remain mandatory.

## Normative boundary

[L0 step 5c-2 §2 BT-1](https://github.com/simonvanieperen/BaryonOS/blob/docs/k06-step-specs/docs/implementation/k06-steps/06-5c-2-bsp-timer-idle.md#2-invarianten-normatief) requires PIT-IRQ0 as the sole monotone-clock source, delivered only to the BSP; it does not itself specify `+= 1`. However, its referenced ADR-053 states in `docs/DECISIONS.md:689–690`: “alleen werkelijk getelde IRQ0's verhogen de monotone klok.” ADR-045 also defines Sys3 in 100 Hz PIT ticks. Current M3 explicitly adds the actual IRQ count, including its idle path. The later measurement decision states that missing IRQs are not synthesized. Crediting multiple elapsed periods per received IRQ therefore requires an explicit normative decision clarifying/revising the counted-IRQ rule; it must not enter as a silent measurement repair. No elapsed-clock or normative change is implemented in this checkpoint.

## Confirmed QEMU mechanism; historical causality unproved

Primary sources are pinned to QEMU v10.2.1:

- [main-loop.c:303–315](https://github.com/qemu/qemu/blob/v10.2.1/util/main-loop.c#L303): polling releases BQL and reacquires it on return; [592–605](https://github.com/qemu/qemu/blob/v10.2.1/util/main-loop.c#L592) subsequently runs timers under BQL.
- [qemu-timer.c:533–567](https://github.com/qemu/qemu/blob/v10.2.1/util/qemu-timer.c#L533): all expired timers are processed against one sampled `current_time`.
- [i8254.c:260–287](https://github.com/qemu/qemu/blob/v10.2.1/hw/timer/i8254.c#L260): PIT callbacks schedule from their previous transition time, allowing multiple overdue edges in one timer-list pass.
- [i8259.c:143–168](https://github.com/qemu/qemu/blob/v10.2.1/hw/intc/i8259.c#L143): rising edges OR a single pending IRR bit; acknowledgement clears it. They are not counted in a queue.
- [cpu-exec.c:800–839](https://github.com/qemu/qemu/blob/v10.2.1/accel/tcg/cpu-exec.c#L800), [seg_helper.c:203–209](https://github.com/qemu/qemu/blob/v10.2.1/target/i386/tcg/system/seg_helper.c#L203), and [x86-cpu.c:69–86](https://github.com/qemu/qemu/blob/v10.2.1/hw/i386/x86-cpu.c#L69): the vCPU needs BQL before PIC acceptance.

Consequently, a delayed timer pass spanning multiple PIT rising edges can merge them before the vCPU can acknowledge one. This mechanism is source-confirmed. No retained trace establishes that it caused the reported clock failures, their frequency, or the host delay responsible. A default 50 microsecond Linux timer slack is not a justified diagnosis: [qemu-timer.c:641–650](https://github.com/qemu/qemu/blob/v10.2.1/util/qemu-timer.c#L641) already sets 1 ns at initialization when detected by [meson.build:2732–2733](https://github.com/qemu/qemu/blob/v10.2.1/meson.build#L2732). The exact CI binary/configuration and successful runtime prctl remain independently unverified.

The current BSP stub uses master-PIC EOI and correctly omits LAPIC EOI for ExtINT. Its `sti; hlt; cli` uses the STI shadow; no software lost-wakeup defect was identified. [hpet.c:538–549](https://github.com/qemu/qemu/blob/v10.2.1/hw/timer/hpet.c#L538) disables PIT output in legacy-replacement mode; [654–678](https://github.com/qemu/qemu/blob/v10.2.1/hw/timer/hpet.c#L654) resets that mode off. Firmware legacy replacement has not been measured in the failed runs; it is not established as their cause.

## Conditional elapsed-PIT design and unresolved boundaries

A continuously gated channel 2, binary mode 2 with reload zero meaning 65536, could provide modulo PIT-cycle deltas. A coherent latch/LSB/MSB read would give `delta = previous.wrapping_sub(current)` as u16. A residual accumulator could convert only measured PIT cycles into whole channel-0 periods; TSC must never supply credited cycles or missing wrap counts. A separate raw IRQ counter would retain actual-delivery evidence. This is a conditional design, not a validated hardware driver.

It requires all of the following:

1. Permanent BSP ownership after initialization. Existing `smp::wait_us()` repeatedly reprograms channel 2 in mode 0, including after the clock fixture. It cannot coexist unchanged. Its post-handover implementation would need bounded delta polling on the reserved counter, retaining missing/stuck-device failure checks and existing wait durations. Latch sequences must be protected against IRQ interleaving, without an IRQ handler waiting for a lock held by interrupted code.
2. Proven unambiguous sampling gaps below 65536 / 1193182 = approximately 54.9254 ms. A 16-bit count alone cannot distinguish one delta from the same delta plus any whole number of wraps. Any ambiguous interval must stop rather than estimate. TSC may reject an interval, but accepting an interval needs a justified conversion bound. Current minimum-of-ten calibration is described as an upper bound on TSC-per-ms; that alone is not the lower frequency bound needed to prove a TSC-based maximum elapsed interval. A safety margin is not a mathematical proof without a corresponding calibration-error assumption.
3. A defined PIT epoch and residual when channel 0 is armed. Initial I/O latency and rearming must not erase elapsed cycles or create an extra period. Merely rebasing after every `wait_us()` loses real elapsed time.
4. Correct M3 transition: `m3::timer_init()` reprograms channel 0 from divisor 1193 to 11932 and uses distinct IRQ0 paths for CPL0 idle/lock waits and CPL3. Updating only `smp::CLOCK_TICKS` would change the acceptance fixture while leaving product Sys3 untouched. A genuine driver design needs an explicit period/epoch handover and integration into both paths without borrowing `State`/Engine from a nested CPL0 interrupt.
5. Separation of elapsed clock periods from actual timer deliveries, preemption events, deferred generations, and idle wake counts. Credited elapsed time must not invent scheduler interrupts. Existing return/FPU/stack safety and timer-handler restrictions remain.

These are substantial driver and contract work. Earlier EOI, PIC auto-EOI, or replacing idle with busy polling does not recover information already lost in a pending bit and is not a demonstrated correction.

## Permitted follow-up diagnosis

Preserve the failed runs. Use separate diagnostic labeling to reproduce the unchanged pinned source/profile without a competing QEMU matrix. Capture timestamped PIC input rises and accepts, PIC IMR/IRR state, EOI writes, and host scheduling of QEMU main/vCPU threads. Existing [PIC trace events](https://github.com/qemu/qemu/blob/v10.2.1/hw/intc/trace-events#L3) include `pic_set_irq`, `pic_interrupt`, `pic_update_irq`, and `pic_ioport_write`; [PIT trace events](https://github.com/qemu/qemu/blob/v10.2.1/hw/timer/trace-events#L52) record programming but do not alone timestamp every timer deadline. Verify available trace backends and associate clock failure samples with trace intervals. Host `sched_switch`/wakeup tracing can distinguish execution gaps from device backlog when permitted. Record actual timer slack, CPU affinity, cgroup throttling, and runner load as context, not as causal proof.

A decisive trace would show multiple IRQ0 rising edges with no intervening PIC accept, the resulting single accept, and corresponding vCPU/main-loop scheduling delay. Trace overhead can alter timing; an instrumented pass does not replace native acceptance. Any runner correction must preserve the specified environment/profile and be followed by a fresh complete 64×50 matrix and executed repository gate.
