# K06B-CLOCK — clock path facts and attack on the candidate measurement design

Task ID: K06B-CLOCK. Read-only. Date: 12 November 2026.

Labels: **[S]** verified in source, **[D]** verified in data, **[I]** inferred, **[U]** unverified.

## 0. Sources, method, limits

| Item | Value |
|---|---|
| Kernel source | frozen worktree `snap-67e592c` (commit `67e592c`); cited as `path:line` relative to its root |
| Emulator source | fetched with `curl` from `raw.githubusercontent.com/qemu/qemu/<tag>/<path>` for tags `v8.2.2` and `v10.2.1` (HTTP 200 for every file cited); cited as `Q8 <file>:<line>` and `Q10 <file>:<line>` |
| PIT/PIC logic across versions | `hw/timer/i8254.c`, `i8254_common.c`, `hw/intc/i8259.c` differ only in tracing and API churn (`diff`: 10, 15 and 9 changed lines, no logic) **[S]** |
| Local data | `docs/verification/k06-smp-sched/timing-investigation/*.jsonl.gz` (QEMU 8.2.2 diagnostic series, not acceptance) |
| Not done | no cargo, build, QEMU or test run; no hosted raw logs re-read; Graphify used only for one `recall` and one `graphify_callers(wait_us)` on `main` (index commit `49f80b1`), both consistent with the worktree |
| Own computation | a 90-line integer re-implementation of the QEMU mode-2 formulas (section 2.6), marked **[I]** |

## 1. Part 1 — kernel clock path

### 1.1 Sites

| Element | Fact | Cite | Label |
|---|---|---|---|
| IRQ0 stub `nos_smp_clock_entry` | `push rax`; read LAPIC ID (MMIO `LAPIC_WINDOW+0x20`); `shr 24`; compare with `BSP_APIC_ID`; on mismatch `call smp_clock_on_ap` (fatal). On the BSP: `push rdx`; `rdtsc`; store `CLOCK_EDGE_TSC`; `lock inc CLOCK_TICKS`; `mov al,0x20; out 0x20,al` (non-specific EOI, master PIC only, no LAPIC EOI); pops; `jmp nos_smp_spurious_entry` (= bare `iretq`). No GS, no lock, no PIT port access | `kernel/src/m2/smp.rs:426-448`, `:403-405`, `:2646-2649` | [S] |
| Gate type | vectors 0x20, 0x27, 0xd0, 0xe0, 0xe8, 0xf0, 0xff are installed with `flags: 0x8e` (64-bit interrupt gate, IF cleared on entry) | `kernel/src/m2/arch.rs:307-320` | [S] |
| Gate targets | 0x20 → clock stub; 0x27 (PIC spurious IRQ7) → bare `iretq`, no count, no EOI | `kernel/src/m2/smp.rs:1298-1300` | [S] |
| `bsp_timer_init` | BSP, IF=0, after AP bring-up, before GO. LAPIC timer: divide 16, LVT masked, initial `u32::MAX` then 0. TSC recalibration: minimum of 10 × `wait_us(10_000)`. LVTs: LINT0 = `0x700` (ExtINT), LINT1 = `0x400` (NMI), error/perf/thermal/CMCI masked; TPR = 0 | `kernel/src/m2/smp.rs:2575-2616` | [S] |
| PIC programming | ICW1 `0x11` both; ICW2 `0x20`/`0x28`; ICW3 4/2; ICW4 1/1 (no auto-EOI); OCW1 master `0xfe`, slave `0xff` (only IRQ0 unmasked) | `kernel/src/m2/smp.rs:2620-2633` | [S] |
| PIT channel 0 | control `0x34` (channel 0, lo/hi, mode 2, binary); reload 1193 (`PIT_DIVISOR`); nominal 1 193 182 / 1193 = 1000.15 Hz, period 999 847 ns | `kernel/src/m2/smp.rs:2502-2504`, `:2634-2636` | [S] |
| `idle_halt` | reads CR3, `KERNEL_CR3`, this CPU's lockdep word (through `rdmsr GS_BASE`), fails `SMP_IDLE_STATE` on mismatch, then one asm block `sti; hlt; cli`. The only `sti; hlt; cli` (BT-3); shared by BSP and APs | `kernel/src/m2/smp.rs:2558-2572`, `:1986-1991`, `:2014-2021` | [S] |
| `clock_at` | loop, IF=0: load `CLOCK_TICKS`; monotonic check; if `now >= target` return `(ticks, CLOCK_EDGE_TSC, rdtsc())`; if `rdtsc() > bound` return `None`; else `idle_halt()` | `kernel/src/m2/smp.rs:2718-2737` | [S] |
| `clock` precision | bound = now + 4 × `CLOCK_SPAN` ms of TSC; sync to the next tick; five segments of 1000 target ticks; sample = `tsc_delta × 1000 × 1000 / (per_ms × tick_delta)` using IRQ0-entry TSCs and the actual tick delta; median of five against `expected` = 999 847 µs; fail if `abs_diff × 100 > expected` | `kernel/src/m2/smp.rs:2748-2812` | [S] |
| `clock` load | APs woken by 0xf0 IPI into the `CLOCK_LOAD` loop; BSP must count 1000 further ticks before 4 × 1 s; every AP must show work | `kernel/src/m2/smp.rs:2813-2848`, `:1026-1046` | [S] |
| `wait_us` | `inb(0x61) & 0x0c`; `outb(0x61, control)`; `outb(0x43, 0xb0)` (channel 2, lo/hi, mode 0); two writes to `0x42`; check OUT2 low; `outb(0x61, control\|1)`; poll `inb(0x61) & 0x20` with PAUSE, at most 2 000 000 iterations; `outb(0x61, control)`. Touches ports 0x61, 0x43, 0x42 only | `kernel/src/m2/smp.rs:747-772` | [S] |
| TSC calibration | `per_ms = min over 10 of (Δtsc of wait_us(10_000)) / 10`; the TSC interval starts before the first port access of `wait_us` and ends after the poll that sees OUT2 high | `kernel/src/m2/smp.rs:340-350`, `:2585-2595` | [S] |
| Position of `clock` in the boot | after `shootdown`, `preemption`, `spin_window`; before `bsp_preempt` | `kernel/src/m2/smp.rs:1604-1610` | [S] |

### 1.2 Every user of ports 0x40–0x43 and 0x20/0x21/0xa0/0xa1 in `kernel/src`

Active tree is `kernel/src/m2` unless feature `native-smp-legacy` selects `m2_legacy` (`kernel/src/main.rs:9-13`) **[S]**. Result of a grep over every `in`/`out` instruction and helper call in `kernel/src` outside `m2_legacy`:

| Site | Ports | Operation | When | Label |
|---|---|---|---|---|
| `kernel/src/m2/smp.rs:442` | 0x20 | OCW2 EOI | clock stub | [S] |
| `kernel/src/m2/smp.rs:2620-2633` | 0x20, 0x21, 0xa0, 0xa1 | full 8259 init + masks | `bsp_timer_init` | [S] |
| `kernel/src/m2/smp.rs:2634-2636` | 0x43, 0x40 | channel 0 mode 2, 1193 | `bsp_timer_init` | [S] |
| `kernel/src/m2/smp.rs:755-770` | 0x61, 0x43 (`0xb0`), 0x42 | channel 2 only | every `wait_us` | [S] |
| `kernel/src/m2/m3.rs:2615-2628` (`timer_init`, called `:887`) | 0x20, 0x21, 0xa0, 0xa1, 0x43 (`0x36`), 0x40 | 8259 re-init (ICW1 again), masks `0xfe`/`0xff`; channel 0 **mode 3**, reload **11932** (100 Hz) | M3 start, after `smp::start` returned (`kernel/src/m2.rs:169` then `:212`), therefore after `clock()` | [S] |
| `kernel/src/m2/m3.rs:2134`, `:2149`; `kernel/src/m2/arch.rs:695` | 0x20 | OCW2 EOI | M3 phase | [S] |

- No code reads port 0x21, 0x20 or 0x40–0x43; the PIC mask and the PIT counter are never read back today **[S]**.
- Between `bsp_timer_init` and the end of `clock()` nothing writes the PIC mask or PIT channel 0; the only PIT traffic is `wait_us` on channel 2, and `clock_at`/`clock` precision contain no `wait_us` or `within_ms` call (`kernel/src/m2/smp.rs:2718-2812`) **[S]**.
- The kernel never touches the IOAPIC or the HPET (grep for `ioapic`, `hpet`, `0xfec0`, `0xfed0` in `kernel/src/m2`: no hit) **[S]**. Firmware state of both at hand-over: **[U]**.

### 1.3 BSP between two consecutive `hlt` instructions (precision phase)

| # | Step | IF | Cite | Label |
|---|---|---|---|---|
| 1 | `hlt` executing; CPU halted and receptive (the STI shadow covered only `hlt`) | 1 | `smp.rs:2571` | [S] |
| 2 | IRQ0 edge → PIC INT → BSP LINT0 ExtINT → vector 0x20, interrupt gate; pushed RIP = the `cli` after `hlt`, pushed RFLAGS.IF = 1. From INTA to the EOI of step 4 the PIC holds ISR bit 0 | 0 | `smp.rs:2605`, `:2623`; `arch.rs:316` | [S] |
| 3 | stub: `push rax`; LAPIC ID MMIO read; compare; `push rdx`; `rdtsc` → `CLOCK_EDGE_TSC`; `lock inc CLOCK_TICKS` | 0 | `smp.rs:428-440` | [S] |
| 4 | `out 0x20, 0x20` (EOI); pops; `jmp` → `iretq` | 0 | `smp.rs:441-445`, `:405` | [S] |
| 5 | instruction boundary between `iretq` and `cli`: IF=1, no shadow. A pending IRQ0 (edge latched during steps 2–4) is taken here; steps 2–4 repeat and overwrite `CLOCK_EDGE_TSC` | 1 | `smp.rs:2571` | [S] code; [I] that TCG takes it at this boundary |
| 6 | `cli`; `idle_halt` returns | 0 | `smp.rs:2571` | [S] |
| 7 | `clock_at`: `CLOCK_TICKS.load`; monotonic check; `now >= target` (if true: load `CLOCK_EDGE_TSC`, `rdtsc`, return to `clock`, sample arithmetic, re-enter `clock_at`); else `rdtsc() > bound` | 0 | `smp.rs:2720-2734`, `:2760-2779` | [S] |
| 8 | `idle_halt`: `mov cr3`; load `KERNEL_CR3`; `rdmsr 0xc0000101` + anchor search; lockdep load; compare | 0 | `smp.rs:2561-2567`, `:1986-1991`, `:2014-2021` | [S] |
| 9 | `sti` (shadow), then `hlt` again | 0 → 1 | `smp.rs:2571` | [S] |

Consequences **[I]**: IRQ0 is non-receptive from INTA (step 2) to the next `hlt` except for the single boundary of step 5; one edge arriving in that interval is retained by the PIC (section 2.2). The bound of step 7 is evaluated only after a wake: with APs halted, the BSP LAPIC timer masked with initial count 0 (`smp.rs:2583`, `:2594`) and only IRQ0 unmasked, a complete stop of IRQ0 leaves the BSP in `hlt` forever; `SMP_CLOCK_STOPPED` cannot fire in that case and the failure surfaces as a host watchdog timeout.

### 1.4 APs during the precision phase

| Fact | Cite | Label |
|---|---|---|
| Each AP runs the `ipi_phase` loop and reaches `idle_halt()` (`sti; hlt; cli`) when no phase flag is set; `CLOCK_LOAD` is false during precision | `smp.rs:944-1071`, `:1028`, `:2817` | [S] |
| AP LVTs written masked: timer 0x320, LINT0 0x350, LINT1 0x360, error 0x370, perf/thermal/CMCI; TPR 0; SVR enabled with vector 0xff | `smp.rs:899-921` | [S] |
| AP LAPIC timer is unmasked periodic in the preemption phase and masked again by the handler at `PREEMPT_STOP`; `preemption()` waits for `RESUMED`, which requires `current == 0` after a tick | `smp.rs:2316-2320`, `:2190-2192`, `:973-978`, `:2366-2375` | [S] |
| Wake sources of a halted AP: fixed IPI 0xf0, 0xe0 (shootdown), 0xe8 (reschedule), 0xd0 (masked), NMI/INIT IPIs. The BSP sends none between the end of `spin_window` and `smp.rs:2820` | `smp.rs:2480-2495`, `:2748-2820` | [S] |
| A PIT IRQ0 taken by an AP is fatal (`SMP_CLOCK_AP_IRQ0`) | `smp.rs:429-432`, `:446-448`, `:2646-2649` | [S] |
| After any wake an AP runs one loop iteration (all flags false) and halts again | `smp.rs:944-1070` | [S] |
| The precision phase does not wait for, or assert, that every AP has reached `hlt`: `spin_window` ends on `spun == 2`, which the AP stores before finishing its loop iteration | `smp.rs:1019-1023`, `:2481-2495`, `:2748-2753` | [S] |
| In QEMU a masked LVT timer schedules no host timer, so a stopped AP timer causes no vCPU wake | Q8 `hw/intc/apic_common.c:132-145`; Q10 `hw/intc/apic_common.c:129-141` | [S] |

## 2. Part 2 — emulator semantics (QEMU v8.2.2 and v10.2.1)

### 2.1 i8254

| # | Claim | Q8 | Q10 | Label |
|---|---|---|---|---|
| P1 | Counter read: `d = (now_virtual − count_load_time) × 1193182 / 1e9` (floor); mode 2 returns `count − (d % count)`, range 1..count; `now` is `QEMU_CLOCK_VIRTUAL` at the moment of the call | `hw/timer/i8254.c:52-75` | `:53-76` | [S] |
| P2 | `count_load_time` is set only when a reload value is written (`pit_load_count`) or on a gate rising edge in modes 1/2/3/5; no IRQ delivery or callback changes it. The counter is pure arithmetic on the virtual clock | `i8254.c:78-115` | `:79-116` | [S] |
| P3 | Counter-latch command: control word with access bits 00 latches the channel in bits 7:6 (`0x00` channel 0, `0x40` channel 1, `0x80` channel 2): `latched_count = pit_get_count()`, `count_latched = rw_mode`. **"if already latched, do not latch again"**: a second latch before the first is fully read is ignored | `i8254.c:117-124`, `:157-161` | `:118-125`, `:160-164` | [S] |
| P4 | Read of a latched lo/hi channel returns LSB, then MSB, then clears the latch; an unlatched lo/hi read calls `pit_get_count()` separately for each byte (incoherent pair) | `i8254.c:194-252` | `:197-256` | [S] |
| P5 | Read-back command (bits 7:6 = 11): per selected channel, latch count unless bit 5 set, latch status unless bit 4 set; status = `OUT<<7 \| rw_mode<<4 \| mode<<1 \| bcd`; null-count bit not implemented; a latched status is returned before a latched count | `i8254.c:136-156`, `:209-211` | `:139-159`, `:212-214` | [S] |
| P6 | A mode/access control word changes only the addressed channel's `rw_mode`, `read_state`, `write_state`, `mode`, `bcd`; data ports index the channel by address | `i8254.c:158-190` | `:161-193` | [S] |
| P7 | Only channel 0 has an IRQ timer; `pit_irq_timer_update` returns immediately for channels without one | `i8254.c:254-261`, `:340-343` | `:260-267`, `:346-349` | [S] |
| P8 | Port 0x61: write → `pit_set_gate(pit, 2, bit0)`; read → gate and OUT of channel 2. The gate handler touches only the passed channel; for mode 0 it does nothing ("XXX: just disable/enable counting"), so channel 2 counts from the reload write, not from the gate | `hw/audio/pcspk.c:135-162`; `i8254.c:78-106`; `i8254_common.c:35-42` | `pcspk.c:135-167`; `i8254.c:79-107`; `i8254_common.c:35-41` | [S] |
| P9 | **Channel 2 programming and port 0x61 gating do not disturb channel 0** (follows from P6–P8) | as above | as above | [S] |
| P10 | Mode 2 output: `out = 1` iff `d % count == 0 && d != 0`; next transition at `base + count` when `d == base != 0`, else `base + count + 1`; converted to ns by floor; "fix potential rounding problems": never ≤ current time | `hw/timer/i8254_common.c:45-74`, `:77-131` | `:44-73`, `:76-130` | [S] |
| P11 | The IRQ timer callback evaluates level and next expiry at the **scheduled** time (`s->next_transition_time`), not at the time it actually runs | `i8254.c:277-282` | `:283-288` | [S] |
| P12 | A timer-list pass samples `current_time` once and runs every timer that is expired against it, including timers re-armed by their own callback | `util/qemu-timer.c:546-580` | `:533-567` | [S] |
| P13 | The main loop drops the BQL only around `poll` and runs timers with the BQL held | `util/main-loop.c:305-311`, `:603` | same lines | [S] |
| P14 | HPET legacy-replacement mode disables the channel-0 IRQ timer; HPET reset re-enables it | `hw/timer/hpet.c:623-629`, `:700`; `i8254.c:299-311` | `hpet.c:540-548`, `:675`; `i8254.c:305-317` | [S] |

### 2.2 i8259

| # | Claim | Q8 | Q10 | Label |
|---|---|---|---|---|
| I1 | Edge-triggered input: on level 1, IRR is set only if `last_irr` was 0; on level 0 only `last_irr` is cleared. **A rising edge while IRR is already set changes nothing: edges merge into one pending bit, no count** | `hw/intc/i8259.c:145-155` | `:142-152` | [S] |
| I2 | IRR is set regardless of the mask; the mask is applied when the INT output is computed | `i8259.c:78-104`, `:147-151` | `:75-101`, `:144-148` | [S] |
| I3 | INTA clears IRR (edge inputs) and sets ISR (no auto-EOI) | `i8259.c:160-174` | `:157-171` | [S] |
| I4 | **Edge while the same IRQ is in service**: IRR is set again (one edge retained); INT is not asserted because the pending priority is not higher than the in-service priority; non-specific EOI clears the highest ISR bit and recomputes INT, which then asserts | `i8259.c:78-104`, `:265-276` | `:75-101`, `:262-273` | [S] |
| I5 | No pending request at INTA → spurious IRQ7 (`irq_base + 7`), no ISR bit | `i8259.c:199-203` | `:196-200` | [S] |
| I6 | ICW1 resets `last_irr`, IRR (edge inputs), IMR and ISR: a PIC re-initialisation discards a pending edge | `i8259.c:241-247`; `hw/intc/i8259_common.c:37-55` | `i8259.c:238-244`; `i8259_common.c:36-54` | [S] |
| I7 | IRR/ISR/IMR are readable (OCW3 select, port 0x21) | `i8259.c:324-352` | `:321-349` | [S] |
| I8 | PIC INT is offered to every CPU whose LAPIC accepts it: LINT0 not masked (or APIC disabled); ExtINT raises `CPU_INTERRUPT_HARD`; "ExtINT interrupts ignore the priority" (TPR does not gate it) | `hw/i386/x86.c:548-569`, `:576-594`; `hw/intc/apic.c:140-192`, `:571-620` | `hw/i386/x86-cpu.c:41-87`; `hw/intc/apic.c:152-203`, `:706-753` | [S] |

### 2.3 Time bases under TCG without icount

| # | Claim | Q8 | Q10 | Label |
|---|---|---|---|---|
| T1 | Guest `RDTSC` = `cpu_get_tsc() + tsc_offset`; `cpu_get_tsc` = `cpus_get_elapsed_ticks()` = `cpu_get_ticks()` unless the accelerator overrides it | `target/i386/tcg/misc_helper.c:64-76`; `hw/i386/x86.c:542-545`; `system/cpus.c:236-242` | `misc_helper.c:64-76`; `hw/i386/x86-cpu.c:35-38`; `system/cpus.c:249-255` | [S] |
| T2 | `cpu_get_ticks()` = offset + `cpu_get_host_ticks()`, forced monotonic; on an x86-64 host `cpu_get_host_ticks()` is the host `rdtsc` | `system/cpu-timers.c:43-72`; `include/qemu/timer.h:884-895` | `cpu-timers.c:43-72`; `timer.h:878-889` | [S] |
| T3 | `QEMU_CLOCK_VIRTUAL` = `cpus_get_virtual_clock()` = `cpu_get_clock()` = offset + `get_clock()` = host `clock_gettime(CLOCK_MONOTONIC)` | `util/qemu-timer.c:633-640`; `system/cpus.c:211-229`; `cpu-timers.c:74-101`; `timer.h:832-843` | `qemu-timer.c:620-627`; `cpus.c:213-231`; `cpu-timers.c:74-101`; `timer.h:837-848` | [S] |
| T4 | The TCG accelerator overrides both clocks with the instruction counter only when icount is enabled, and only in round-robin mode; MTTCG asserts icount off | `accel/tcg/tcg-accel-ops.c:197-214`; `tcg-accel-ops-mttcg.c:71` | `tcg-accel-ops.c:201-219`; `tcg-accel-ops-mttcg.c:70` | [S] |
| T5 | The harness passes `-accel tcg,thread=<single\|multi>` and no `-icount` | `tools/run.py:104` | — | [S] |
| T6 | **Result: guest TSC and PIT are not the same variable.** TSC = host cycle counter; PIT (channels 0 and 2), PIT counter latch and LAPIC current count = host `CLOCK_MONOTONIC`. Both advance with host real time, also while a vCPU thread is not running | T1–T3; `hw/intc/apic_common.c:165-180` | T1–T3 | [S] for each source; [I] for the comparison |
| T7 | Whether host TSC and `CLOCK_MONOTONIC` keep a constant ratio on the hosted runner (itself a virtual machine) | — | — | [U] |

### 2.4 Waking a halted vCPU (multi-thread TCG)

| Step | Q8 | Q10 | Label |
|---|---|---|---|
| `hlt`: clear the STI inhibit flag, `halted = 1`, leave the execution loop | `target/i386/tcg/sysemu/misc_helper.c:483-499` | `target/i386/tcg/system/misc_helper.c:509-517`; `target/i386/tcg/helper-tcg.h:125-130` | [S] |
| vCPU thread: takes the BQL after every return from execution, then waits on `halt_cond` while `cpu_thread_is_idle()` (halted and no work) | `accel/tcg/tcg-accel-ops-mttcg.c:91-119`; `system/cpus.c:85-100`, `:422-438` | `tcg-accel-ops-mttcg.c:87-114`; `cpus.c:87-102`, `:462-479` | [S] |
| "work" for x86 = `CPU_INTERRUPT_HARD` with IF=1 and no inhibit (also POLL, NMI, SMI, SIPI) | `target/i386/cpu.c:7677-7724` | `cpu.c:9805-9848` | [S] |
| PIT callback (main loop, BQL held) → PIC → LINT0 ExtINT → `cpu_interrupt(HARD)` → `tcg_handle_interrupt` → `qemu_cpu_kick`: broadcast `halt_cond`, `cpu_exit` | `i8254.c:254-275`; `i8259.c:107-157`; `apic.c:140-192`; `tcg-accel-ops.c:89-104`; `cpus.c:458-466` | `i8254.c:260-281`; `i8259.c:104-154`; `apic.c:152-203`; `tcg-accel-ops.c:96-109`; `cpus.c:499-507` | [S] |
| vCPU thread wakes when the host schedules it and it re-acquires the BQL; `cpu_handle_halt` clears `halted`; `cpu_handle_interrupt` takes the BQL again and calls `x86_cpu_exec_interrupt` → `cpu_get_pic_interrupt` → `pic_read_irq` (INTA) | `accel/tcg/cpu-exec.c:676-698`, `:794-896`; `target/i386/tcg/sysemu/seg_helper.c:130-175` | `cpu-exec.c:653-669`, `:777-840`; `target/i386/tcg/system/seg_helper.c:165-206` | [S] |
| MMIO and port I/O issued by TCG code take the BQL (the stub's LAPIC ID read and every PIT/PIC port access) | `accel/tcg/cputlb.c:2033`, `:2057`; `system/physmem.c:2638-2651` | `cputlb.c:1986`, `:2007`; `physmem.c:3196-3209` | [S] |
| Single-thread TCG: one host thread runs all vCPUs; when all are idle it waits on the shared `halt_cond`; the same kick wakes it | `accel/tcg/tcg-accel-ops-rr.c:108-122`, `:313-335` | `tcg-accel-ops-rr.c:108-122` | [S] |

### 2.5 PAUSE

| Mode | v8.2.2 | v10.2.1 | Label |
|---|---|---|---|
| Translation | `helper_pause(env, len)`; block ends | `gen_PAUSE` → `helper_pause(env)`; block ends | Q8 `target/i386/tcg/translate.c:5524-5529`; Q10 `target/i386/tcg/emit.c.inc:2819-2825` [S] |
| Helper | `do_pause`: `exception_index = EXCP_INTERRUPT; cpu_loop_exit` ("Just let another CPU run"). **Does not clear the STI inhibit flag** | `do_end_instruction()` (**clears the STI inhibit flag** and RF), then the same exit | Q8 `target/i386/tcg/misc_helper.c:91-106`; Q10 `misc_helper.c:91-102`, `helper-tcg.h:125-130` [S] |
| Multi-thread TCG | execution returns; the thread locks the BQL, runs the idle/work check (not idle, no sleep), unlocks, re-enters. No host yield, no sleep: one BQL round trip per PAUSE | same structure (`qemu_process_cpu_events`) | Q8 `tcg-accel-ops-mttcg.c:91-119`; Q10 `:87-114` [S] |
| Single-thread TCG | execution returns; `cpu = CPU_NEXT(cpu)`: the next vCPU runs; after the last one the loop restarts at the first | same | Q8 `tcg-accel-ops-rr.c:245-303`; Q10 `:262-308` [S] |
| Round-robin kick period (`TCG_KICK_PERIOD`) | value not located in the fetched files | same | [U] |

### 2.6 Derived consequences

| # | Statement | Basis | Label |
|---|---|---|---|
| D1 | With reload 1193 the model raises IRQ0 at `load + floor((k·1193+1)·1e9/1193182)` ns and lowers it one PIT tick before the next rise: high about 999.0 µs, low about 0.84 µs; rise-to-rise 999 847 or 999 848 ns | P10, P11; integer model, 20 000 periods | [I] |
| D2 | The lattice of rising edges is fixed by the reload write; a late callback does not shift later edges (no drift), it compresses them | P11 | [I] |
| D3 | **Catch-up:** if the main loop is late by more than one period, one timer pass presents fall/rise/fall/rise back to back with the BQL held; no INTA can occur in between; all but one rising edge merge (I1). This happens whatever the guest is doing | P11–P13, I1, section 2.4 | [I] |
| D4 | **vCPU wake latency:** with a punctual main loop, an edge is also lost when the BSP vCPU thread needs more than one period between the IRR set and its INTA (host scheduling or BQL wait) | I1, section 2.4 | [I] |
| D5 | Edges are never presented before their lattice time (timers fire only when expired); therefore counted IRQ0 ≤ lattice edges elapsed, at any instant | P12 | [I] |
| D6 | A channel-0 latch returns the phase on that lattice: `L = 1193 − latched` PIT ticks since the modelled reload. The counter wraps (reads 1193) one PIT tick (838 ns) before the IRQ rise is scheduled: latch at rise −900 ns reads 1, at rise −1 ns reads 1193 | P1, P10; integer model | [I] |
| D7 | The latch value depends on neither IRQ delivery nor on PIC state; it stays coherent if a PIT callback runs between the three port accesses (the callback does not touch `latched_count`/`count_latched`) | P3, P4, `i8254.c:254-275` (Q8) | [S] |
| D8 | **Model-intrinsic skipped edge:** when `(k·1193+1)·1e9` is divisible by 1193182 the floor rounding that D1 relies on is exact and the rise callback evaluates `out = 0`: the edge is not raised. 1193182 = 2·41·14551; first solution k = 59009, i.e. exactly **59.000 s after the channel-0 reload write**, then every 596.5 s (and one "stays high" case at 596.5 s). Rise-to-rise there is 1 999 695 ns. Independent of host load | P10; exact arithmetic; integer model over 700 000 periods found gaps > 1.5 P at rises 59009, 596590, 655598 only | [I], not observed in a run |
| D9 | In the local series the precision phase ends 5.5–14.8 s after `SMP:TIMER:RATIO` on four host CPUs and 30.7–49.9 s on two; D8 is outside every observed window but inside reach of a slow host | `timing-investigation/m-*.jsonl.gz`, `p-*.jsonl.gz` (143 boots) | [D] local |
| D10 | `TSC_PER_MS` is biased high: the TSC interval of `wait_us` includes four port exits before the reload write and the poll granularity after expiry (P8: counting starts at the reload write). Shortest passing segment in the local series: −0.19 % to +0.59 % of nominal | `smp.rs:751-772`, `:340-350`; local series | [I] bias; [D] range |
| D11 | Local passing boots: median segment +0.4 % to +0.6 % typical (max +0.95 %), longest segment +1.1 % to +1.5 % typical (max +4.95 %). Edge loss is routine on that host, also in boots that pass | local series (DIAG line `SMP:DIAG:CLOCK_US=median:min:max`, `diag-long.patch`) | [D] local, QEMU 8.2.2 |
| D12 | A spinning (not halted) BSP would shorten the INTA latency of D4 (no condition-variable wake), not D3. "Halted with IF=1" is the most receptive architectural state, not the most receptive emulator state | section 2.4 | [I] |

## 3. Part 3 — attack on the candidate

Notation: P = nominal period in TSC units; `c_i` latched count; `e_i` edge time; `k_i` rounded periods; N counted IRQ0; J lattice index; window `w_i = p_i − entry_i`.

### 3.1 What each check measures on QEMU

| Check | Measures | Does not measure | Label |
|---|---|---|---|
| (1) `Sum(gap)/Sum(k)` within 1 % | `e_i` are lattice points (D6), so the ratio equals the channel-0 period in TSC units by construction: reload value × TSC calibration error. Both come from the same virtual clock (T6) | IRQ delivery in any form. Lost, masked, misrouted or uncounted IRQ0 leave (1) unchanged | [I] |
| Per-gap residual | `ε·gap` (calibration error, D10) + latch sandwich + 0.84 µs quantisation; a non-integer divisor error | — | [I] |
| (2) `lost = Sum(k) − N` | net deficit of counted IRQ0 against the lattice, ±1 for an edge due but not yet delivered at either end (D5, D6) | double counts hidden by an equal number of losses | [I] |
| (3) attribution | length of the kernel's measured IF=0 path per tick | everything in section 3.5 | [I] |
| Today (`smp.rs:2774-2789`) | mean interval of ~1000 **counted** IRQ0 per segment; median of five | cause of a deviation | [S] |

Finding: the unchanged "1 %" of (1) no longer applies to the counted-IRQ0 clock. On QEMU check (1) is a divisor-and-calibration test; every bit of IRQ-delivery sensitivity moves to (2), (3) and the unspecified "separate limit".

### 3.2 Kernel defects: verdict today against verdict under the candidate

| Defect | Today | Candidate as written | Label |
|---|---|---|---|
| Divisor ×2 (reload 2386) or any integer multiple m | fails: every segment +100 % | **passes (1)**: `c_i` up to 2386, `L_i` shifts by a constant, gaps = m·P, `k_i = m`, residual 0. `lost = (1 − 1/m)·Sum(k)`, every window short → all "platform-attributed". Verdict rests only on the separate limit (must be far below 50 %) | [I] from P1 |
| Divisor ×1.5 (1790) | fails (+50 %) | residual ±0.5 P: fails only if the residual bound is below P/2 | [I] |
| Divisor 2 % off | fails | fails (1) | [I] |
| Divisor ≤ 1 % off | passes | passes | [I] |
| IRQ0 delivered to an AP | fatal in stub | same, stub unchanged. An AP with LINT0 unmasked that never wins the INTA race only takes spurious 0x27 and is invisible in both | [S] `smp.rs:429-448`; [I] I5, I8 |
| Double counting (two increments per IRQ0) | fails (−50 %) | fatal by `N > Sum(k)+1` | [I] |
| One synthesised tick per 1000 | passes (−0.1 %; also unlimited in two of five segments) | fatal only if net loss in the boot is smaller than the number of extra ticks; local typical loss is 4–6 per 1000 (D11), so it is masked | [I], [D] |
| Stub never sends EOI | BSP never wakes again: host watchdog, not `SMP_CLOCK_STOPPED` (section 1.3) | identical | [I] |
| IRQ0 masked (IMR) by BSP code and unmasked before `hlt` | one edge retained (I2); loss only if the masked span covers ≥ 2 presentations | span lies inside the window; classified by window length | [I] |
| IRQ0 masked across `hlt` by the BSP | hang (no other wake source) | hang | [I] |
| PIC/PIT disturbed by a non-halted AP (IMR write, ICW1 re-init, channel-0 write) while the BSP is halted | fails if the median exceeds 1 % | BSP windows stay short → **platform-attributed**; end-point assertions miss a transient | [I] from I6 |
| LINT0 masked transiently | delays delivery (INT stays asserted; re-offered at the next PIT transition) | same | [I] from `i8259.c:107-118` (Q8) |
| TPR raised | no effect on ExtINT in QEMU (I8) | the TPR assertion is not load-bearing on QEMU | [S] |
| IRQ0 taken on a path that sends EOI but does not count | tick count stalls: `SMP_CLOCK_STOPPED` after 20 s | wake without a fresh stub TSC; safe only if such a gap is forced to "unattributed" (the text does not say so) | [I] |
| Kernel path with IF=0 for T per tick, T < P/2 (up to ~500 µs) | fails once it pushes the median above 1 % | window "provably short"; every resulting loss is platform-attributed | [I] |
| Kernel path with IF=0 ≥ P/2 | as above | unattributed, 1 % limit | [I] |
| Work added in the stub before its `rdtsc`, or between the `p_i` read and `sti` | counted through its effect on the median | outside the measured window | [I] |

Answer: yes. At least four classes of real defect pass under the candidate while they fail today: integer-multiple divisor; loss caused by an actor other than the BSP's measured path; IF=0 paths up to P/2; anything inside the two unmeasured intervals. All of them depend on the separate limit being looser than today's effective tolerance.

### 3.3 Ways a healthy kernel can fail under the candidate

| # | Mechanism | Label |
|---|---|---|
| H1 | `k_i` rounding: the latch gives the phase only; the integer part comes from TSC ÷ P_nominal. Error = ε·gap/P. With the local ε range of 0.2–0.6 % (D10) a gap above roughly 80–250 ms rounds ambiguously. Long gaps are exactly the stall events of interest. Effect: `lost` off by one or more, or a false `N > Sum(k)+1` | [I], [D] |
| H2 | A fixed per-gap residual bound fails on long gaps for the same reason (residual grows with the gap) | [I] |
| H3 | Host preemption of the BSP vCPU thread inside its IF=0 path makes that window long; the edges lost during it are "unattributed" and consume the 1 % budget although the kernel executed the same instructions. The guest cannot tell the two apart: TCG has no instruction counter without icount (`rdpmc` unimplemented, Q8 `target/i386/tcg/misc_helper.c:78-89`) | [I], [S] |
| H4 | Latch retries: each retry is a port exit under the BQL and lengthens the window, converting a platform delay into H3 | [I] |
| H5 | The separate limit: local passing boots already show +0.4–0.6 % median and up to +4.95 % in one segment (D11); hosted distributions do not exist (`TIMING-INVESTIGATION.md:148-149`). A limit chosen below these figures fails healthy boots that pass today, because today two of five segments are unbounded | [D] |
| H6 | D8: one deterministic lost edge if the phase overlaps 59.000 s after the reload write | [I] |

`N > Sum(k)+1` is otherwise sound for a healthy kernel: D5 gives N ≤ J at every instant, and at most one due edge can be undelivered at the first sample **[I]**.

### 3.4 Latch safety

| Question | Answer | Label |
|---|---|---|
| Against `wait_us` channel-2 traffic | Safe. Separate channel state (P6–P9); `0xb0` and port 0x61 address channel 2 only; no `wait_us` runs inside the precision phase (section 1.2) | [S] |
| Against an IRQ0 during the three port accesses | Safe with IF=0: the stub touches no PIT port (`smp.rs:426-448`); inside QEMU the callback leaves the latch untouched (D7) | [S] |
| Hazard 1 | A latch command is ignored while an earlier latch of channel 0 is unread (P3). Every latch must be followed by exactly two reads, also when the sample is rejected | [S] |
| Hazard 2 | A read-back command with status selected makes the next read return the status byte (P5) | [S] |
| Hazard 3 | Only the `out 0x43` access carries the timestamp; the two reads return the stored value. The TSC sandwich belongs around that single access | [S] P1, P3 |
| Hazard 4 | `c_i` must be asserted in 1..1193. Without the assertion a larger reload value is invisible to the edge arithmetic (section 3.2, first row) | [I] |
| Hazard 5 | Valid only for the `bsp_timer_init` programming; `m3::timer_init` later switches channel 0 to mode 3 / 11932, where the counter formula is `count − (2d % count)` | [S] `m3.rs:2626-2628`; Q8 `i8254.c:66-69` |
| Real 8254 behaviour | not examined | [U] |

### 3.5 Audit of the attribution argument

| Claim in the candidate | Finding | Label |
|---|---|---|
| "If that window is shorter than half a period, at most one edge can occur inside it" | False on QEMU. A catch-up pass presents several rising edges within microseconds (D3). True only for lattice (ideal) times | [I] from [S] P11–P13 |
| "… so every lost edge of that gap occurred while the BSP was halted with IF=1" | Not provable. With window w and emulator lateness ℓ ≥ P − w, two presentations fall inside the window. The provable statement is: *with w < P/2, a loss in this gap required a presentation or INTA delay above P − w > P/2* | [I] |
| "… which the kernel cannot prevent" | Holds for D3. For D4 it holds architecturally, not for the emulator (D12) | [I] |
| Window = [IRQ0-entry TSC, p_i] | Starts late: INTA → stub `rdtsc` contains a LAPIC MMIO read that takes the BQL (section 2.4). Ends early: `p_i` → `sti` cannot be timestamped. A nested IRQ0 at the `iretq`/`cli` boundary overwrites the entry TSC (section 1.3, step 5) | [S] code, [I] effect |
| Per-gap independence | An edge retained in one window is delivered at the start of the next; the P/2 threshold tolerates a kernel that is non-receptive almost half of every period, which lowers the emulator lateness needed for a loss from about P to about P/2 | [I] |
| "PIC mask, LINT0 and TPR asserted unchanged at both ends of the window" | New code (no read-back exists today, section 1.2). End-point assertions do not see transients; ISR/IRR are also readable (I7) and are not included; AP LINT0 is not included | [S], [I] |
| "all APs halted" | Not asserted by the current code (section 1.4) | [S] |
| "unattributed … stay under the unchanged 1 % limit together with the period error" | The combination of a count ratio and a rate error is not defined | — |
| "bounded by a separate limit" | Not specified. Since windows are microseconds in normal operation, nearly all loss is platform-attributed; the separate limit becomes the only effective acceptance bound for loss | [I], [D] D11 |

### 3.6 Change against the normative text

Normative text: owner decision of 11 November (`docs/DECISIONS.md:2635-2648`, precision clause `:2639`), measurement repair of 10 November (`:2567`), ADR-053 (`:843`), spec 06 BT-1 (`specs/docs/implementation/k06-steps/06-5c-2-bsp-timer-idle.md:23-25`, outside the worktree, in the scratchpad `specs` tree). ADR-082 (`docs/DECISIONS.md:2680-2741`) is a proposal and not in force (`:2682`).

| Normative element | Candidate | Kind |
|---|---|---|
| "five segments" (`:2567`, `:2639`) | one aggregate over per-wake samples | contract |
| "the median segment" (`:2639`) | no median; totals | contract |
| "must last the PIT's own period within 1 %" applied to segments of counted IRQ0 | 1 % applied to the lattice period (3.1) plus unattributed loss | contract (same number, different quantity) |
| "sustained drift remains a failure" (`:2567`) | sustained platform-attributed loss below the separate limit passes | contract |
| BT-1 "Weerlegd als Sys3 … trager loopt dan de PIT" | Sys3 may run slower than the PIT by the platform-attributed amount | contract |
| ADR-053 "only actually counted IRQ0s advance the monotonic clock"; "Missing IRQs are not synthesized" | respected for the clock itself; the verdict subtracts attributed loss, which is compensation at the level of the acceptance rule. `docs/DECISIONS.md:2595`: "elapsed-counter compensation requires an explicit normative decision" | clock: unchanged; verdict: contract |
| observation deadline 4 × `CLOCK_SPAN`, monotonicity, `SMP_CLOCK_STOPPED`, `SMP_CLOCK_AP_IRQ0` | unchanged if kept | none |

| Measurement-method changes (keep the contract; recorded decision suffices, cf. `K06A-SPEC.md` §5) | Contract changes (owner decision and spec 06 §4 revision) |
|---|---|
| per-wake sampling with channel-0 latch and TSC sandwich | replacing five segments and their median by an aggregate |
| exact lattice loss count and per-tick delivery latency as logged diagnostics | exempting platform-attributed loss from the 1 % limit |
| `N > Sum(k)+1` fatal (stricter than today) | any separate loss limit and its value |
| assertion `1 ≤ c_i ≤ 1193`; PIC mask / LINT0 assertions; "every wake has a fresh counted IRQ0" | redefining what the 1 % applies to |
| window length logged per tick | the P/2 attribution threshold |

Fact relevant to sequencing **[I]**: the candidate's samples contain everything today's statistic needs (counted ticks and entry TSCs), so both verdicts can be computed and logged in the same boot; today's verdict can stay the gate while the new quantities are collected on hosted runners.

### 3.7 Implementability notes

| Topic | Fact | Label |
|---|---|---|
| `p_i` | `idle_halt` is the single `sti; hlt; cli` site for BSP and APs (BT-3); a TSC store placed there runs on every idle CPU, or needs a BSP-only variant inside the same asm block. All checks must precede the `rdtsc` to keep `rdtsc`→`sti` minimal | [S] `smp.rs:2558-2572` |
| Stub | unchanged by the candidate; it must stay GS-free and lock-free | [S] `smp.rs:422-425` |
| Cost | three port exits per tick (≈ 5000 ticks), each a BQL round trip; duration of one exit under hosted TCG | [U] |
| Serial output | must stay outside the phase; a serial write inside lengthens the IF=0 window | [I] |
| Oracle | exact-sequence oracle rejects unknown lines; new diagnostic lines need oracle and host-test updates; no `FAIL`/`PANIC` substring | `K06A-SPEC.md` §5 (not re-verified) [U] |
| Negative cases | "IRQ0 masked for more than 10 % of each segment" (ADR-082 table) cannot be built by masking across `hlt` (hang); built inside the BSP window it tests the window rule, built from an AP it is platform-attributed (3.2) | [I] |
| ADR-082 "clock-synthesised: one tick counted twice per segment → lower bound fails" | one extra tick per 1000 is −0.1 %, below a −1 % bound and maskable under the candidate (3.2) | [I] |

## 4. Corrections required before the candidate is defensible

1. Assert `1 ≤ c_i ≤ PIT_DIVISOR`; otherwise an integer-multiple divisor passes.
2. Do not call `Sum(k)` exact: bound the gap for which rounding is proven (ε·gap < P/2 − noise), or refine P from the accumulated lattice before rounding while still comparing the refined P with nominal for check (1); treat an ambiguous gap as a distinct outcome.
3. Replace the net rule by a prefix rule: counted ticks may exceed lattice progress only by resolving a deficit opened at the immediately preceding sample. This catches a double count that later losses would hide.
4. Restate the attribution theorem in lattice terms (3.5, row 2) and drop "occurred while halted".
5. Define a gap without a fresh stub TSC (spurious 0x27, uncounted path) as unattributed.
6. Choose the window threshold from measured normal windows, not P/2; log the maximum window.
7. Specify the separate limit, its statistic (per segment or total) and its data basis; local data show +5 % in single passing segments.
8. State the two unmeasured intervals and the AP-actor case as residual risk, or add AP-side evidence (halted flag, AP LINT0 read).
9. Keep two reads after every latch; sandwich only the `out 0x43`.
10. Record explicitly that check (1) tests the divisor, not the clock, and that the 1 % has changed its object.

## 5. Unverified

- Firmware (OVMF) state of HPET legacy replacement, IOAPIC redirection entries and PIT channel 0 at ExitBootServices.
- Ratio stability of host TSC against host `CLOCK_MONOTONIC` on GitHub-hosted runners.
- Duration of a TCG port exit and of the stub's LAPIC MMIO read under hosted load; typical and worst window length.
- `TCG_KICK_PERIOD` value.
- That QEMU takes a pending interrupt at the `iretq`/`cli` boundary (read from the interrupt-acceptance condition, not traced).
- D8 has not been observed in a run; it is exact arithmetic on the fetched formulas.
- Real 8254/8259 hardware behaviour; KVM.
- Hosted loss distribution (only pass/fail and failing medians exist).
- `K06A-SPEC.md` oracle constraints were taken as written; `K06A-SITES.tsv` was read only for the clock and `wait_us` rows.
