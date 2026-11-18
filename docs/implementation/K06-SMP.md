<a id="l0-specificatie-k06--meerdere-processorkernen-smp"></a>

# L0 specification K06 — Multiple processor cores (SMP)

**Status:** proposal (design), not implemented, not tested.
**Date:** 6 November 2026 (Europe/Amsterdam).
**Baseline:** branch `docs/niet-wachten-op-ci`, HEAD `0d968f3d` (main with #204 = K06 step 4).
**Sources read:** `docs/progress.json` (K06), `docs/PROJECT-STATE.md` (step 4 handoff),
`docs/DECISIONS.md` ADR-024, -045, -053, -060, -070 through -073,
`docs/verification/k06-smp-preempt/README.md`, `kernel/AGENTS.md`, beginning of `kernel/src/m2/smp.rs`.
**Not read, therefore marked [to be verified]:** the M3 scheduler, the IRQ0 path
(8259 PIC or IOAPIC), the FXSAVE implementation, the IPC queues, `tools/run_smp.py`.

K06 acceptance (progress.json), source quotation (Dutch): *"SMP-scheduling, synchronisatie, interrupts en
adresruimtewisseling werken onder concurrente belasting op het ondersteunde profiel."*
**English translation:** *"SMP scheduling, synchronization, interrupts and
address-space switching work under concurrent load on the supported profile."*
Evidence level: native QEMU q35/OVMF/TCG; physical hardware is outside K06.

---

<a id="0-uitgangspunt-al-gerealiseerd-stap-14"></a>

## 0. Baseline: already implemented (steps 1–4)

| Step | ADR | Existing implementation (`native-smp` only) |
|---|---|---|
| 1 | 070 | MADT as untrusted input; INIT-SIPI-SIPI per AP (unicast); CAS STARTING→CLAIMED/ABANDONED; AP stacks with guards in the kernel image; `MAX_CPUS = 8`; xAPIC MMIO only via IA32_APIC_BASE; the AP copies CR0/CR3/CR4/EFER/PAT exactly. |
| 2 | 071 | A separate GDT/TSS per AP (RSP0, IST1 #DF, IST2 NMI), GS anchor; IPI vector 0xf0, spurious 0xff; unicast only; an unresponsive AP → INIT for all APs. |
| 3 | 072 | TLB shootdown (vector 0xe0) with generation acknowledgement before freeing a frame; a missing acknowledgement → frame quarantine and INIT for all APs. One bootstrap window. |
| 4 | 073 | LAPIC timer (vector 0xd0) per AP; two preempted kernel threads per AP; GPR context; freeze after STOP. |

**Missing functionality** (handoff + ADR-073 "Open"): BSP interrupts/preemption, an SMP-safe
scheduler with per-CPU run queues, a lock order for kernel subsystems, user processes
on APs, a CR3 switch per thread, shootdown for process address spaces and revision of ADR-024.

---

## 1. Scope

**Within K06:** steps 5a–5f below, in the opt-in `native-smp` profile, upgraded from
a bringup profile to a profile that runs M3 processes.

**Outside K06 (explicitly open):** x2APIC, IOAPIC routing of device IRQs to APs,
PCID/INVPCID, TSC-deadline, NUMA, CPU hotplug, physical hardware, IOMMU/DMA containment, power management.
Other profiles (`milestone-m2`, M3–M6, GUI) remain single-CPU and unchanged.

**Kernel rule (`kernel/AGENTS.md`), source quotation (Dutch):** "Geen SMP uitbreiden vóór single-core-invarianten
aantoonbaar zijn." **English translation:** "Do not extend SMP before single-core invariants
have been demonstrated." These have been accepted (K02–K05, K07). Each step below must
leave the single-CPU regression passing unchanged.

---

<a id="2-beveiligingsinvarianten-eerst-normatief"></a>

## 2. Security invariants (first, normative)

Each invariant has a falsification condition; that condition becomes a negative test (§9).

- **INV-1 Exclusive frame ownership.** A physical frame receives a new owner only
  after no CPU can still use a translation of it (ADR-072, extended
  to all address spaces).
  *Falsified if* a CPU reads or writes the frame through an old TLB entry after `free`.
- **INV-2 No shared mutable kernel state without a lock.** The engine, frame allocator,
  paging, IPC queues, process table and quotas are modified only under the lock specified in §4.
  *Falsified if* lockdep (§4.4) detects a modification without the required lock, or if
  the stress run (§9) produces an audit mismatch.
- **INV-3 One CPU per thread.** A thread runs on at most one CPU at a time. Its
  kernel stack, FXSAVE image and GPR frame belong exclusively to that CPU while it runs.
  *Falsified if* two CPUs publish `RUNNING` for the same TID simultaneously.
- **INV-4 Address-space isolation remains per process.** A CPU has loaded only the CR3
  of the process it is executing, or the kernel CR3 when idle. A process's
  `active_cpus` bitmap is a superset of the CPUs that can cache its translations.
  *Falsified if* a CPU caches translations of process P while its bit in `P.active_cpus` is 0.
- **INV-5 Fail closed on an unresponsive CPU.** A CPU that does not acknowledge a shootdown or reschedule IPI
  within the deadline is never ignored: frames remain quarantined and
  the CPU is reset with INIT (ADR-071/072). A failed INIT stops the boot.
- **INV-6 Capability and usercopy boundaries remain unchanged.** SMP adds no permission, syscall
  or ABI. Usercopy revalidates the pointer under the destination address-space lock,
  so a concurrent unmap by a thread of the same process does not
  open a TOCTOU window.
- **INV-7 No lock across a context switch.** A CPU holds no lock during a switch,
  except for the run-queue handoff in §6.3. No lock can be held by a thread
  while it is parked.

---

<a id="3-apic--en-smp-initialisatie-precieze-volgorde"></a>

## 3. APIC and SMP initialization (exact order)

Steps 1–4 remain unchanged. The only additions concern what the BSP and an AP do after
existing bringup.

### 3.1 BSP

1. Existing M2 boot: IDT, paging, frames, BootInfo, MADT inventory (ADR-070).
2. Existing AP bringup, IPI self-test, shootdown self-test and preemption self-test
   (steps 1–4), with IF=0. The self-tests remain mandatory gates: if any fails,
   all APs are quarantined (INIT) and boot continues with one CPU, as it does now.
3. **New:** a per-CPU block for the BSP (CPU index 0) using the same GS anchor mechanism
   as the APs (ADR-071), so `this_cpu()` follows the same path everywhere.
4. **New:** a LAPIC timer on the BSP with the same vector 0xd0 and the same divisor as the APs.
   PIT IRQ0 remains the monotonic clock source **on the BSP only** (ADR-045 Sys3,
   ADR-053). The LAPIC timer supplies only preemption ticks and is not used as a clock.
   [to be verified: the current IRQ0 route (8259 or IOAPIC) and whether it remains masked to the APs]
5. Only then perform M3 initialization (process table, manifest, initial processes) **under the kernel lock** (§4).
6. The BSP sets IF=1 only in user mode and in the idle loop (`sti; hlt; cli`, the existing
   ADR-053 pattern). Kernel code continues to run with IF=0, except where §4.3 permits otherwise.

### 3.2 AP

1. Steps 1–4 remain unchanged up to and including the preemption self-test. The self-test threads stop.
2. **New:** the AP waits in the idle loop for `SMP_SCHED_GO` (one atomic, released by the BSP
   after step 3.1.5). Only then does it select work from its own run queue (§6).
3. On an AP, IF=0 remains in force in kernel code, and IF=1 in user mode and in the idle loop.

<a id="33-vectoren-vast"></a>

### 3.3 Vectors (fixed)

| Vector | Purpose | Recipient |
|---|---|---|
| 0x20 (or the existing IRQ0 vector) [to be verified] | PIT, monotonic clock | BSP only |
| 0xd0 | LAPIC timer, preemption | all CPUs |
| 0xe0 | TLB shootdown | target CPUs from `active_cpus` or all online CPUs |
| 0xe8 | **new:** reschedule/kill IPI | one target CPU |
| 0xf0 | test IPI (existing) | self-test only |
| 0xff | spurious | all |

Unicast ICR only, no broadcast shorthands (ADR-071). Priority classes (vector >> 4):
shootdown 0xe is above timer 0xd, so a CPU with a pending timer still
accepts a shootdown. Each ICR write occurs under `ICR_LOCK` (§4.2), with a delivery-status
check before the next write.

---

<a id="4-spinlocks-en-lockvolgorde"></a>

## 4. Spinlocks and lock order

<a id="41-lockprimitief"></a>

### 4.1 Lock primitive

- **Ticket lock** (`next`/`owner` as `AtomicU32`, FIFO, no starvation under TCG).
  `lock`: `fetch_add(next, Relaxed)`, then read `owner` with `Acquire` and use `pause` in the
  spin loop. `unlock`: `owner.fetch_add(1, Release)`.
- **IRQ-safe:** `lock_irqsave` saves RFLAGS.IF, executes `cli` and then acquires the lock;
  `unlock_irqrestore` restores IF only after release. Any lock also acquired from a handler
  (0xd0/0xe0/0xe8) may be acquired **only** through `irqsave`.
- **Bounded spinning in debug builds:** after `2^24` iterations, emit a terminal
  `SMP:LOCK:STUCK:<naam>:CPU=<n>` and stop the VM. Release builds spin without a bound, but
  lockdep (§4.4) must already have proven the order in the gate.
- A guard type (RAII) without `Send`, so a guard cannot be carried across a context
  switch (INV-7). This is a compile-time boundary; the runtime check is specified in §4.4.
- No `unsafe` outside access to `UnsafeCell` in the guard; each SAFETY note specifies
  aliasing (exclusive under the lock), alignment (repr) and lifetime (guard).

<a id="42-locks-en-vaste-volgorde-buiten--binnen"></a>

### 4.2 Locks and fixed order (outer → inner)

A CPU may acquire only a lock at a **higher** level than the highest level it already holds.

| Level | Lock | Protects | From a handler? |
|---|---|---|---|
| 10 | `KERNEL` (big kernel lock) | capability engine, process table, IPC queues, quotas, supervisor, M3 services, volume/net/keyring brokers | no |
| 20 | `ASPACE[pid]` | page tables and mapping arena of one process (ADR-024 2048 mappings) | no |
| 30 | `FRAMES` | frame allocator and ownership metadata | no |
| 40 | `RUNQ[cpu]` | run queue of one CPU; two at once only in ascending CPU-index order | yes (0xd0, 0xe8) |
| 50 | `SHOOTDOWN` | one ongoing shootdown round (address range, target set, generation) | no (the handler only reads the published round) |
| 60 | `ICR_LOCK` | xAPIC ICR writes on the local CPU [per CPU, because the ICR is local; needed only against reentry from a handler] | yes |
| 70 | `SERIAL` | serial port | yes (terminal paths only) |

**Rationale for a big kernel lock (level 10).** Existing subsystems were built and
proven under the contract "één CPU, IF=0" (source quotation, Dutch; **English translation:**
"one CPU with interrupts disabled") (ADR-024, -053, -060). Fine-grained locks per
subsystem would reopen all that evidence. The big kernel lock preserves that contract
literally: inside `KERNEL`, the kernel behaves as a single CPU. Parallelism in
this step comes from user mode on multiple CPUs, rather than from parallel syscalls.
*shortcut: syscalls serialize on one lock; replace it with separate locks per subsystem when measurements show contention (review criterion in ADR-074).*

**Consequences for existing ADRs:**
- ADR-060 (preallocated 461.816-byte buffer with a non-reentrant lease;
  the period is a thousands separator):
  remains valid because it is used only under `KERNEL`. The explicit
  single-CPU/IF0 check becomes "houdt `KERNEL`" (**English translation:** "holds the kernel lock").
- ADR-053 (IPC-wait): parking and waking a waiter occur under `KERNEL`;
  waking a thread on another CPU sends a 0xe8 IPI (§6.4).
- ADR-045 (FXSAVE): see §7.3.

<a id="43-interruptstatus-per-context"></a>

### 4.3 Interrupt state per context

| Context | IF | Locks permitted |
|---|---|---|
| user mode | 1 | none |
| syscall/exception path in the kernel | 0 | all, following §4.2 |
| idle loop | 1 between `sti; hlt`, otherwise 0 | none during `hlt` |
| handler 0xd0/0xe8 | 0 | only `RUNQ[eigen cpu]` (irqsave), `ICR_LOCK` |
| handler 0xe0 | 0 | no lock: reads the published round with `Acquire`, executes `invlpg`/CR3 reload, acknowledges the generation with `Release` |
| NMI/#DF (IST) | — | no lock; terminal path only via `SERIAL` with `try_lock` |

**Shootdown deadlock freedom:** a CPU waiting for acknowledgements holds `SHOOTDOWN` and
possibly `ASPACE`/`FRAMES`, but has IF=0. Target CPUs must therefore be able to
accept the 0xe0 IPI while spinning in the kernel on `KERNEL`. Therefore:
**spinning on `KERNEL` and `ASPACE` occurs with IF=1** (the spin loop executes `sti; nop; pause; cli`),
and the 0xe0 handler acquires no lock. This allows a CPU waiting for a lock to
handle a shootdown at any time. The CPU holding the lock never waits for a CPU that
itself needs a handler lock.

<a id="44-lockdep-debug-verplicht-in-de-gate"></a>

### 4.4 Lockdep (debug, mandatory in the gate)

A `held_mask: u64` and `max_level` per CPU in the per-CPU block. `lock` checks
`level > max_level`; otherwise, emit `SMP:LOCKDEP:ORDER:<a>-><b>` and stop. On a context switch,
`held_mask` must be empty, with the §6.3 exception for `RUNQ`, which the switch code itself hands over.
Otherwise, emit `SMP:LOCKDEP:HELD_ACROSS_SWITCH`. A negative test profile deliberately acquires locks
in reverse order and must emit that marker (§9).

---

<a id="5-herziening-adr-024-voorgesteld-als-adr-074"></a>

## 5. Revision of ADR-024 (proposed as ADR-074)

ADR-024 itself requires revision "bij SMP" (source quotation, Dutch; **English translation:** "when introducing SMP"). Proposed new text:

1. **Mapping arena:** unchanged at 2048 mappings per process, static, outside the kernel stack.
   Access only under `ASPACE[pid]` (+ `KERNEL`).
2. **Per-CPU scratch slot instead of one global slot.** CPU *i* uses only VA
   `SCRATCH_BASE + i·PAGE` (supervisor, NX, outside every user range, in the shared
   kernel half). Invalidation before use and after clearing remains local (`invlpg`).
   **Precondition:** no CPU ever dereferences another CPU's slot. Any
   speculatively cached entry for another CPU's slot is then never used architecturally.
   This precondition is enforced by making the address available only through `this_cpu().scratch_va()`.
   *Residual risk:* speculative side channels involving such a translation are outside this model
   (recorded, not resolved).
3. **Executable-alias rejection and RO/NX helper:** unchanged.
4. **No identity map of free RAM/MMIO:** unchanged.
5. **ASID:** remains an internal counter; PCID remains off. Each CR3 load therefore flushes
   that CPU's non-global TLB entries.
6. **Kernel mapping change:** a change in the shared kernel half (except the local
   scratch slot) requires a shootdown to **all online CPUs** (ADR-072 protocol).
7. **Review on:** PCID, fine-grained subsystem locks, DMA/IOMMU or a new mapping API.

---

## 6. Scheduler (SMP)

<a id="61-datastructuren"></a>

### 6.1 Data structures

- `PerCpu { index, apic_id, runq: RUNQ lock<FixedQueue<Tid, N>>, current: Option<Tid>,
  idle_tid, scratch_va, tss, lockdep, need_resched: AtomicBool, timer_ticks: AtomicU64 }`
  via the GS anchor.
- `Thread { tid, pid, state: AtomicU8 {READY, RUNNING(cpu), BLOCKED, DEAD}, home_cpu,
  kstack, fxsave: [u8; 512] align 16, gpr_frame }`.
  [to be verified: how the current M3 process entry combines thread and process; this becomes
  one thread per process as long as M3 has no threads]
- `Process.active_cpus: AtomicU64` (bit per CPU index; `MAX_CPUS = 8` fits comfortably).
- A run queue has a fixed capacity (no allocator), equal to the existing
  process capacity; overflow is impossible because a thread is in only one queue.

<a id="62-plaatsing-en-migratie"></a>

### 6.2 Placement and migration

- At spawn, the kernel selects `home_cpu` = the online CPU with the shortest run queue (lowest
  index on a tie), under `KERNEL`.
- **No automatic load balancing in K06.** Migration occurs only explicitly:
  (a) when a CPU is quarantined (all threads to the BSP), and (b) in a test fixture
  that forces migration to prove §7.
  *shortcut: no work stealing; add it only when R02 performance budgets require it.*

<a id="63-contextwissel-exacte-volgorde-op-cpu-c-if0"></a>

### 6.3 Context switch (exact order, on CPU *c*, IF=0)

1. Trigger: a 0xd0 tick sets `need_resched`, a syscall blocks, or a 0xe8 IPI
   arrives. The switch itself occurs only on return to user mode or in the idle loop,
   never in the middle of a `KERNEL` section.
2. `held_mask` must be empty (lockdep).
3. FXSAVE of the current thread `T` to `T.fxsave` (ADR-045: after the last Rust code of the
   old context, before CR3 selection).
4. `lock_irqsave(RUNQ[c])`; `T.state`: RUNNING(c) → READY (in the queue) or BLOCKED/DEAD
   (not in the queue). Select the next `U` (round-robin); `U.state` READY → RUNNING(c)
   with `compare_exchange` (INV-3; if it fails, emit `SMP:SCHED:DOUBLE_RUN` and stop).
5. **Address-space switch:** if `U.pid != T.pid`:
   a. `U.process.active_cpus.fetch_or(1<<c, SeqCst)` **before** the CR3 load;
   b. `mov cr3, U.process.cr3`;
   c. `T.process.active_cpus.fetch_and(!(1<<c), SeqCst)` **after** the CR3 load.
   Order a→b→c preserves INV-4: the bit is already set before the CPU can
   cache the new translations, and is cleared only when the old ones are no longer reachable
   (a CR3 load without PCID flushes them).
6. CPU *c*'s TSS.RSP0 ← top of `U.kstack`.
7. `unlock_irqrestore(RUNQ[c])` (IF remains 0 until IRETQ).
8. FXRSTOR from `U.fxsave`, immediately before GPR pops/IRETQ (ADR-045).
9. Idle: no thread READY → CR3 ← kernel CR3 (step 5 with the kernel as a "process"), then
   `sti; hlt; cli` and check again.

<a id="64-wekken-en-kill-over-cpus-heen"></a>

### 6.4 Waking and kill across CPUs

- Waking a thread on CPU *d* from CPU *c* (for example an IPC reply, ADR-053):
  under `KERNEL` → `lock_irqsave(RUNQ[d])` → BLOCKED→READY, enqueue → unlock → if *d* ≠ *c*,
  a 0xe8 IPI to *d*. The IPI is a hint: *d* always checks its queue itself.
  A readiness wake (no timeout/error) from *c* ≠ *d* goes to the front of `RUNQ[d]`, once per
  BLOCKED→READY and without an extra quantum (ADR-081 addendum R3-5, 11 November 2026).
- Kill of a thread that is RUNNING on *d*: mark `DEAD_PENDING`, send 0xe8 and wait,
  bounded (deadline equal to the shootdown deadline), until *d* publishes `state = DEAD`
  at its next switch. Only then do the existing `cleanup` (K05) and the
  address-space shootdown (§7.2) follow. No acknowledgement within the deadline → INV-5.

<a id="65-preëmptie"></a>

### 6.5 Preemption

LAPIC timer 0xd0 on every CPU, periodic, the same divisor as ADR-073. The handler does only:
`timer_ticks += 1`; on every *k*-th tick `need_resched = true`; EOI. Calibration against the PIT is
**not** needed for K06 (the clock comes from the PIT), but the number of ticks per PIT tick is
logged in the gate. [uncalibrated, see ADR-073 "Open"]

---

<a id="7-adresruimtewissel-en-tlb-shootdown-per-proces"></a>

## 7. Address-space switching and TLB shootdown per process

<a id="71-algemeen-protocol-veralgemening-van-adr-072"></a>

### 7.1 General protocol (generalization of ADR-072)

For each change that removes or weakens an existing user translation of process P
(unmap, permission reduction, process exit):

1. Acquire `KERNEL` → `ASPACE[P]` → (if freeing) `FRAMES` → `SHOOTDOWN`.
2. Clear or change the PTE(s); local `invlpg` per page (≤ 32 pages), otherwise a
   CR3 reload if P is locally active.
3. `fence(SeqCst)`; snapshot `targets = P.active_cpus.load(SeqCst) & !(1<<self)`.
   Because of the order in §6.3 (step 5a before the CR3 load), any CPU that loads P afterwards
   already sees the new PTE; any CPU that loaded P earlier is in `targets`.
4. Publish the round `{pid, range, generatie+1}` with `Release`; send unicast 0xe0 to each
   CPU in `targets`.
5. Handler on the target CPU: if its current CR3 is P's → `invlpg` per page or a
   CR3 reload; otherwise nothing (a CR3 load without PCID has already flushed). Then always
   acknowledge with the generation (`Release`).
6. Wait (IF=0, `pause`) for all acknowledgements until the deadline (the same as ADR-072).
   If any is missing → frames quarantined (owner and mapping metadata retained), INIT
   for all APs, all threads to the BSP (§6.2a), and boot continues with one CPU (INV-5).
7. Only after all acknowledgements: frames to `FRAMES.free`, followed by the existing paging audit.

**Kernel half:** the same steps with `targets = alle online CPU's` (ADR-024 revision, item 6).

<a id="72-procesexit"></a>

### 7.2 Process exit

Order: all threads DEAD (§6.4) → `P.active_cpus` must be 0, except for the exiting CPU
itself, which first switches to the kernel CR3 → shootdown with `targets` = the snapshot
(usually empty, but the protocol is always followed) → `cleanup` (K05) → free frames.
`check_invariants` gains: *no free frame is still referenced by a PTE in a live address space*
and *a dead process's `active_cpus` is 0*.

<a id="73-fpu-context-bij-migratie-adr-045"></a>

### 7.3 FPU context during migration (ADR-045)

`T.fxsave` is valid as soon as step 6.3.3 completes on the old CPU. The `RUNQ` release (step 6.3.7)
and the `RUNQ` acquire on the new CPU establish a happens-before relationship with FXRSTOR there. A thread
is therefore never restored before its image has been written completely. A migration fixture
proves this with an FPU canary (§9).

<a id="74-usercopy-tijdens-gelijktijdige-unmap-inv-6"></a>

### 7.4 Usercopy during concurrent unmap (INV-6)

Usercopy validates and copies under `ASPACE[P]`. A concurrent unmap of the same P
must therefore wait. The PTE may disappear only after the copy, and the shootdown ensures that the
copying CPU no longer uses an old translation. With one thread per process, this is currently
unreachable, but the contract applies prospectively.

---

<a id="8-implementatiestappen-elk-een-eigen-kleine-pr-gestapeld"></a>

## 8. Implementation steps (each a separate small PR, stacked)

| Step | Result | Write files (expected) | Gate |
|---|---|---|---|
| 5a | Ticket lock + irqsave + lockdep + per-CPU block for the BSP | `kernel/src/m2/smp.rs` or a new `kernel/src/m2/lock.rs` [to be verified: module convention] | host tests of the lock (loom/stress on the host), native lockdep negative |
| 5b | `KERNEL` lock around all M3 syscalls; per-CPU scratch slot; ADR-074 | M3 entry point, paging scratch code, `docs/DECISIONS.md` | full `milestone-m2` and M3 regression on 1 CPU, unchanged |
| 5c | BSP LAPIC timer + idle with `sti; hlt`; PIT remains the clock | `smp.rs`, idle loop | `native-rtc` 5/5, monotonic-clock check, BSP preemption canary |
| 5d | Per-CPU run queues; user processes on APs; CR3 switch §6.3; 0xe8 | M3 scheduler, `smp.rs` | K05 6/6 on 4 vCPUs; `SMP:SCHED:CPUS=n` with user work on every CPU |
| 5e | Process-wide shootdown §7.1/7.2 | paging/unmap, cleanup | deaf-AP negative, control without shootdown (must show a stale read) |
| 5f | Migration fixture + concurrent stress + K06 acceptance | test fixtures, `tools/run_smp.py`, `docs/verification/k06-smp-sched/` | §9 complete; progress K06 → accepted |

After each step: update PROJECT-STATE, commit, push, create a draft PR with auto-merge
(AGENTS.md, 5 November 2026), then proceed immediately to the next step.

---

<a id="9-acceptatie--en-negatieve-tests-native-qemu-q35ovmftcg"></a>

## 9. Acceptance and negative tests (native QEMU q35/OVMF/TCG)

Profiles: 1, 2, 4 and 8 vCPUs; both single-thread and multi-thread TCG (`-accel tcg,thread=multi`).

**Positive (must pass):**
1. **Concurrent load:** ≥ 2× as many CPU-bound ring-3 processes as CPUs, plus
   IPC ping-pong between processes on **different** CPUs, 30 s. Per-CPU requirement: user ticks > 0,
   context switches ≥ 4, no audit error, and all K07 quotas remain bounded per app.
2. **Address-space isolation under SMP:** two processes on different CPUs write
   different patterns to the same user VA and check them repeatedly (the K03 test, concurrent).
3. **Unmap race:** process P runs on CPU 1 and reads page X in a loop; from CPU 0, X is
   unmapped and the frame freed and reused. Requirement: P receives a #PF and neither the old nor
   the new pattern.
4. **Migration with canary:** the fixture migrates T between CPU 1→2→0 with unique GPR and
   x87/SSE values (ADR-045); after each migration, all values match and `active_cpus` is correct.
5. **Kill on a remote CPU:** kill of a process running on CPU 3; K05 cleanup balance
   (free frames before = after), `active_cpus == 0`.
6. **BSP interrupts:** the monotonic clock advances during the load (Sys3 increases) and the idle BSP
   is demonstrably woken.

**Negative (must emit the correct marker and exhibit fail-closed behavior):**
7. `test-smp-lockdep-order`: reversed lock order → `SMP:LOCKDEP:ORDER`.
8. `test-smp-held-across-switch` → `SMP:LOCKDEP:HELD_ACROSS_SWITCH`.
9. `test-smp-proc-shootdown-deaf`: one AP ignores 0xe0 → frame quarantine, INIT for
   all APs, threads to the BSP, boot continues with one CPU.
10. `test-smp-proc-no-shootdown` (control profile): without shootdown, P in test 3 reads the
    reused frame. This proves that test 3 measures a real hazard (the same method as ADR-072).
11. `test-smp-double-run`: force the same TID onto two run queues → `SMP:SCHED:DOUBLE_RUN`.
12. `test-smp-resched-deaf`: AP ignores 0xe8 during a kill → INV-5 path.
13. Canary injection (open item from ADR-073): deliberately corrupt a saved register → the
    canary must trigger.

**Regression (unchanged passing):** `milestone-m2` (TCG/1CPU), K05 6/6, `native-rtc` 5/5,
all step-1–4 cases (14/14), host tests `boot-protocol`, `tools/tests`, Clippy, fmt.

**Evidence:** `docs/verification/k06-smp-sched/{README.md,native-report.json}` with source commit,
exact commands, exit codes, markers per case and hashes. Then `progress.json` K06 →
`accepted` (+2 points, 41 → 43) and `tools/progress.py --write` and `--check`.

**Commands (from the handoff; `run_smp.py` does not build):**
```
export NOS_BUILD_DIR="$PWD/build/k06-native-smp"
python3 -B tools/build.py --kernel-feature native-smp
python3 -B -u tools/run_smp.py --profile native-smp
```

---

<a id="10-primaire-bronnen"></a>

## 10. Primary sources

- Intel SDM Vol. 3A (current edition, https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html):
  MP initialization (INIT-SIPI-SIPI); APIC (ICR, delivery status, LVT timer, TPR/PPR,
  priority classes per vector >> 4, EOI); "Invalidation of TLBs and Paging-Structure Caches"
  and "Propagation of Paging-Structure Changes to Multiple Processors" (§7.1, including a CR3 load
  without PCID); memory ordering (locked instructions, LFENCE/MFENCE) for §4.1/§6.3;
  FXSAVE/FXRSTOR. Search by section title; numbering differs between editions.
- ACPI 6.5 §5.2.12 (MADT), https://uefi.org/specs/ACPI/6.5/ (unchanged from ADR-070).
- Ticket lock: Mellor-Crummey & Scott, "Algorithms for Scalable Synchronization on
  Shared-Memory Multiprocessors", ACM TOCS 9(1), 1991. [not reread for this specification yet]

---

<a id="11-open-aannames-en-risicos"></a>

## 11. Open assumptions and risks

- **[to be verified]** The IRQ0 route and whether APs never receive it; the structure of the
  M3 process entry; the location of the FXSAVE code; the module convention for a new lock module.
- The big kernel lock limits parallel syscall throughput. This is acceptable for K06
  (the criterion requires correctness under concurrent load, rather than scalability), but
  is an R02 risk.
- QEMU TCG does not model real TLB timing or speculation. Test 10 proves that stale
  translations are measured, rather than that physical CPUs behave identically.
- Device IRQs remain polled/on the BSP; IOAPIC routing to APs is outside K06.
- AI review is not a human audit (B05 remains open).

## Branch additions, 6–11 November 2026 (ported 13 November 2026)
> English translation of lines that PRs #246, #229 and #227 added to this specification on the K06 branches and
> that never reached `main`. The Dutch originals are archived as patches in `docs/history/k06-branch-prs-2026-11-06/patches/`;
> the *Amends* lines quote the replaced Dutch text above in translation. The additions describe the
> branch state of those dates; ADR-075 to ADR-081, ADR-083 and the step specifications in
> `docs/implementation/k06-steps/` hold the corresponding decisions.

### §2 Security invariants: new INV-8

*Amends §2: inserted directly after INV-7 (PR #246; identical in PR #229).*

- **INV-8 GS discipline (ADR-083, audit finding F8).** Every kernel entry from CPL3 uses
  the GS anchor only after `swapgs` and `lfence`; ring 3 can never bring GS_BASE into kernel context.
  - GS-1: between prologue and epilogue, GS_BASE contains the anchor of this CPU and KERNEL_GS_BASE
    the user value; in CPL3 the reverse.
  - GS-2: `swapgs` only if `CS & 3 == 3` in the frame of *that* entry, or of the frame that
    `iretq` restores.
  - GS-3: at most one entry `swapgs`.
  - GS-4: no `gs:`, `call` or Rust before the prologue.
  - GS-5: `lfence` at the join point.
  - GS-6: NMI/#DF/#MC never `swapgs` or GS; terminal, the CPU follows from the IST address.
  - GS-7: a frame RIP on a `nos_iret_user_*` label (a faulting `iretq`) also swaps.
  - GS-8: IF=0 from the exit `swapgs` until `iretq`.
  - GS-9: no `mov gs` or GS MSR write after init.
  - GS-10: CR4.FSGSBASE=0 and EFER.SCE=0.

  *Falsified if* an IPI, tick or exception from CPL3 with a GS_BASE chosen by ring 3
  counts or writes anywhere other than in its own `CpuLocal` (§9 tests 15–17).

  *PR #229 only (already English in the source, reproduced verbatim; it follows the refutation
  clause inside INV-8):*

  **Current evidence boundary:** N1–N3 previously exercised user-entry
  behavior, the GS-7 `iretq` fixup, and NMI via IST. The current patch
  strengthens N1/N3: the AP's user descriptor has a nonzero GS base and the
  BSP waits for an observed timer frame after `mov gs`, checking that base
  before sending IPIs and shootdowns. The strengthened behavior requires new
  native CI evidence and is not yet accepted. GS-4, GS-5, GS-8 and GS-9 have
  static lint coverage only. No native negative GS-10 precondition profile
  or CPL3 #DF/#MC test has been executed.

### §3.1 BSP, item 4

*Amends §3.1 item 4: replaces its last line, "[to be verified: the current IRQ0 route (8259 or IOAPIC) and whether it remains masked to the APs]".*

   **Verified and built (step 5c-2, ADR-080):** `native-smp` excludes M3, so no PIC
   programming or Sys3 existed there. `bsp_timer_init` remaps the 8259 to 0x20/0x28 (only IRQ0
   unmasked), sets PIT channel 0 to ~1 kHz and delivers via LINT0 (ExtINT) to the BSP only; APs mask
   LINT0. The IOAPIC is not used. In addition: LAPIC/PIT calibration (`SMP:TIMER:RATIO`, band in
   ADR-080), `irq_ready` on the BSP and the idle context (`idle_halt`).

### §3.3 Vectors (fixed)

*Amends the §3.3 vector table: replaces the row "0x20 (or the existing IRQ0 vector) [to be verified] | PIT, monotonic clock | BSP only" and adds a 0x27 row after it.*

| Vector | Purpose | Recipient |
|---|---|---|
| 0x20 | PIT IRQ0 via the 8259 (ExtINT on LINT0), monotonic clock (ADR-080) | BSP only |
| 0x27 | spurious IRQ7 from the 8259 | BSP only |

### §5 heading

*Amends the §5 heading: replaces "5. Revision of ADR-024 (proposed as ADR-074)".*

**5. Revision of ADR-024 (recorded as ADR-075, step 5b-2)**

### §6.1 Data structures

*Amends §6.1: extends the end of the `PerCpu` item ("via the GS anchor.") and replaces the `Thread` item, "`Thread { tid, pid, state: AtomicU8 {READY, RUNNING(cpu), BLOCKED, DEAD}, home_cpu, kstack, fxsave: [u8; 512] align 16, gpr_frame }`."*

- (`PerCpu` item, continued after "via the GS anchor.") The idle thread is the CPU's home loop
  (`sti; hlt; cli`, step 6.3.9; in the fixture `CpuLocal.saved[0]`); it is never in a queue
  (BW-4, ADR-078).
- `Thread { tid, pid, state: AtomicU32 {READY, RUNNING(cpu), BLOCKING, READY_PENDING, BLOCKED,
  DEAD_PENDING, DEAD}, queued: AtomicBool, home_cpu, kstack, fxsave: [u8; 512] align 16,
  gpr_frame }`. Transitions only via `smp_lock::sched` (one atomic step per function,
  ADR-078); `queued` is set by `RunQueue::push` and cleared by `pop` (BW-2).

### §6.3 Context switch, item 4

*Amends §6.3 item 4: replaces "`lock_irqsave(RUNQ[c])`; `T.state`: RUNNING(c) → READY (in the queue) or BLOCKED/DEAD (not in the queue). Select the next `U` (round-robin); `U.state` READY → RUNNING(c) with `compare_exchange` (INV-3; if it fails, `SMP:SCHED:DOUBLE_RUN` and a stop follow)."*

4. `lock_irqsave(RUNQ[c])`; `T.state`: RUNNING(c) → READY (in the queue), BLOCKING → BLOCKED
   (not in the queue), READY_PENDING → READY (in the queue) or DEAD_PENDING (back in the queue,
   to be cleaned up at a later switch; §6.4). Select the next `U` (round-robin);
   `U.state` READY → RUNNING(c) with `compare_exchange` (INV-3; if it fails,
   `SMP:SCHED:DOUBLE_RUN` and a stop follow). A popped DEAD_PENDING thread becomes DEAD as soon
   as no CPU is still on its stack, otherwise it goes back into the queue. A second push of a
   thread that is already in a queue gives `SMP:SCHED:DOUBLE_ENQUEUE` and a stop (BW-2).

### §6.4 Waking and kill across CPUs

*Amends §6.4: inserts a new first item (BW-1); in the wake item, replaces "under `KERNEL` → `lock_irqsave(RUNQ[d])` → BLOCKED→READY, enqueue → unlock → if *d* ≠ *c*, a 0xe8 IPI to *d*. The IPI is a hint: *d* always checks its queue itself." (the readiness-wake sentence that `main` carries after it is not part of the branch diff); and replaces the kill item, "Kill of a thread that is RUNNING on *d*: mark `DEAD_PENDING`, send 0xe8 and wait, bounded (deadline equal to the shootdown deadline), until *d* publishes `state = DEAD` at its next switch. Only then do the existing `cleanup` (K05) and the address-space shootdown (§7.2) follow. No ack within the deadline → INV-5."*

- **Blocking in two phases (BW-1, ADR-078).** The thread itself: `block_prepare`
  (RUNNING(c) → BLOCKING) before or while it checks its wait condition under the lock
  of the waker; then release the lock (INV-7) and request a switch (tick or its own 0xe8). The switch
  performs `block_commit`: BLOCKING → BLOCKED, or READY_PENDING → READY if a wake has occurred in the meantime.
  A condition that already turns out to be true after `block_prepare` is handled by the thread with `wake` on itself.
- Waking a thread on CPU *d* from CPU *c* (for example an IPC reply, ADR-053):
  under `KERNEL` → `lock_irqsave(RUNQ[d])` → `wake`: BLOCKED→READY and enqueue, or
  BLOCKING→READY_PENDING (no enqueue; the switch on *d* puts it back) → unlock → if it
  has been placed in a queue and *d* ≠ *c*, a 0xe8 IPI to *d*. The IPI is a hint: *d*
  always checks its queue itself. A `wake` on a thread that is still RUNNING does nothing:
  that thread checks its condition after `block_prepare`.
- Kill (BW-3): `kill_request` sets every live state to `DEAD_PENDING`; if it was BLOCKED,
  the killer places it in the queue of *d*. Send 0xe8 and wait, bounded
  (`SHOOTDOWN_DEADLINE_MS`, equal to the shootdown deadline), for `DEAD`. *d* puts a dying
  current thread back into its queue at the switch; the next switch that pops it
  publishes `DEAD` with `kill_commit`, and only if the stub has already cleared `on_cpu` (F1).
  Only then do the existing `cleanup` (K05) and the address-space shootdown (§7.2) follow. No
  `DEAD` within the deadline → INV-5 (until step 05: `SMP:KILL:TIMEOUT:APIC=<n>` and quarantine).

### §6.5 Preemption

*Amends §6.5: replaces the whole paragraph, "LAPIC timer 0xd0 on every CPU, periodic, the same divisor as ADR-073. The handler does only: `timer_ticks += 1`; on every *k*-th tick `need_resched = true`; EOI. Calibration against the PIT is **not** needed for K06 (the clock comes from the PIT), but the number of ticks per PIT tick is logged in the gate. [uncalibrated, see ADR-073 "Open"]".*

LAPIC timer 0xd0 on every CPU, BSP included (step 5c-2), periodic, the same divisor as
ADR-073. Built (ADR-080, BT-4): 0xd0 counts `tick_count` and is a switch point; 0xe8 counts
`resched_count` separately (no longer a tick, F11) and sets `need_resched`. A 0xe8 deferred by INV-5c
remains as a flag until the next switch point: the release of the last lock back to IF=1
(self-0xe8), idle, or a later entry with an empty lockdep. Calibration against the PIT is **not** needed
for K06 (the clock comes from the PIT); the number of LAPIC ticks per PIT tick is measured at boot,
logged (`SMP:TIMER:RATIO`) and checked by the gate against a band fixed in advance.

### §7.2a (new subsection)

*Amends §7: inserts a new subsection 7.2a between §7.2 (Process exit) and §7.3.*

**7.2a Realised in step 5e (ADR-072 addendum, 6 November 2026)**

`smp.rs`: `shootdown_process(space, start, pages)` and `shootdown_kernel(start, pages)`, with one
internal `round` under `SHOOTDOWN`. `paging.rs`: `unmap_shot` = clear the PTE + local `invlpg` →
round → free the metadata (PS-3). The two test spaces are now real spaces (`build_spaces`, own
PML4/PDPT/PD/PT, kernel PD shared via PDPT entry 0, a different frame at `0x4000_0000`). The
PML4 copies from 5d-1 no longer exist. Native fixture (after ICR): `SMP:ISOLATION:PASS` and
`SMP:UNMAP_RACE:PASS` (`SKIPPED` with one CPU), with the controls `test-smp-proc-shootdown-deaf` and
`test-smp-proc-no-shootdown`. The reader's #PF is controlled: `nos_smp_race_read` receives `RACE_FAULT`
through an exception fixup and resumes through the reviewed `iretq` of the spurious stub
(CPL0 only). **Not built:** the M3 path (no M3 under `native-smp`), the
`check_invariants` rules in the engine and a rights-reduction API. See ADR-072 (addendum).

### §8 Implementation steps (step table)

*Amends the §8 step table: replaces rows 5b ("`KERNEL` lock around all M3 syscalls; per-CPU scratch slot; ADR-074"), 5c ("BSP LAPIC timer + idle with `sti; hlt`; PIT remains the clock"), 5d ("Per-CPU run queues; user processes on APs; CR3 switch §6.3; 0xe8") and 5e ("Process-wide shootdown §7.1/7.2") with the following rows between 5a and 5f.*

| Step | Result | Write files (expected) | Gate |
|---|---|---|---|
| 5b | Controlled spinning with IF=1 on `KERNEL`/`ASPACE` (§4.3, ADR-074 addendum); APs only | `crates/smp-lock`, `kernel/src/m2/lock.rs`, `smp.rs` fixture | `SMP:SPIN:PASS`; control `test-smp-spin-irq-off` → `SMP:SPIN:SHOOTDOWN_TIMEOUT` + quarantine |
| 5b-2 | `KERNEL` lock around all M3 syscalls; per-CPU scratch slot; ADR-075 (realised) | `lock.rs`, `m3.rs`, `paging.rs`, `smp.rs` fixture, `docs/DECISIONS.md` | `SMP:SCRATCH:PASS:CPUS=n`; control `test-smp-shared-scratch` → `SMP:SCRATCH:CROSS:CPU=n`; full `milestone-m2` and M3 regression on 1 CPU, unchanged |
| 5c | Preemption deferred while lockdep is not empty (held or watched lock); `HELD_ACROSS_SWITCH` on violation (ADR-074 addendum) | `smp.rs` | lock fixture in `SMP:PREEMPT:PASS`; `test-smp-held-across-switch` → marker |
| 5c-2 | BSP LAPIC timer + idle with `sti; hlt`; PIT remains the clock | `smp.rs`, idle loop | `native-rtc` 5/5, monotonic-clock check, BSP preemption canary |
| 5d(-1) | Run queues per AP, thread-state CAS (INV-3), 0xe8 wake-up, CR3 switch §6.3 with `active_cpus` (INV-4), with kernel threads in two address spaces (ADR-074 addendum) | `crates/smp-lock` (`sched`), `smp.rs` | `SMP:SCHED:PASS:CPUS=n`; `test-smp-double-run` → `SMP:SCHED:DOUBLE_RUN` |
| F8 | `swapgs` discipline in all IDT stubs (INV-8, ADR-083); ring-3 GS fixture | `arch.rs`, `smp.rs`, `tools/tests/test_gs_discipline.py` | red on the old stubs; `SMP:GS:PASS`, `SMP:GS:IRET_FIXUP:PASS`, NMI from CPL3; static lint |
| 5d-2 | Ring-3 processes on APs with the same scheduler core (after 5b-2 and 5c-2; INV-2) | M3 scheduler, `smp.rs` | K05 6/6 on 4 vCPUs; `SMP:SCHED:CPUS=n` with user work on every CPU |
| 5d-2a | **Built (ADR-081):** profile `native-smp-m3`, base M3 manifest on every online CPU (home at spawn, `current` per CPU, entry under `KERNEL`, kernel round before freeing, stop-the-world) | `m3.rs`, `smp.rs`, `arch.rs` | `SMP:M3:CPU=<i>:USER_TICKS>0:SWITCHES>=4`, `SMP:SCHED:CPUS=n` on 2/4/8 vCPUs; `test-smp-m3-fault` → `SMP:M3:K03:PASS` |
| 5d-2b | **Built (ADR-081, addendum):** FPU context per CPU (R3-3), FPU canary in all SMP processes, 1000 switches per CPU | `smp.rs`, `arch.rs`, `m3.rs`, `fpu.rs`, `userspace/probe.rs` | `SMP:M3:FPU:PASS:PROCESSES=16`; `test-smp-m3-fpu-skip` → `SMP:M3:FPU:CORRUPT:CONTROL` |
| 5d-2c | Open: IPC across CPUs (M4 + `ipc-wait` on SMP, 0xe8, `lock_pair`), usercopy under `ASPACE`, K05 on SMP, GS N1-N3 with M3 | `m3.rs`, `wait.rs`, `m4.rs`, `smp.rs` | spec 08 §4 |
| 5e | Process-wide shootdown §7.1/7.2 (realised for the fixture spaces, §7.2a) | paging/unmap, cleanup | deaf-AP negative, control without shootdown (must show a stale read) |

*PR #227 (head `1bbd85d`) carried an earlier wording of the 5b-2 row; PR #246 carries the row above:*

| Step | Result | Write files (expected) | Gate |
|---|---|---|---|
| 5b-2 | `KERNEL` lock around all M3 syscalls; per-CPU scratch slot; ADR-075 | M3 entry point, paging scratch code, `docs/DECISIONS.md` | full `milestone-m2` and M3 regression on 1 CPU, unchanged |

### §9 Acceptance and negative tests

*Amends §9 (negative tests): replaces test 8, "`test-smp-held-across-switch` → `SMP:LOCKDEP:HELD_ACROSS_SWITCH`.", and appends tests 14–18 after test 13.*

Replacement for test 8:

8. `test-smp-held-across-switch` (step 5c: preemption without deferral in the presence of a watched lock) →
   `SMP:LOCKDEP:HELD_ACROSS_SWITCH`.

Appended after test 13:

14. `test-smp-spin-irq-off` (control profile, step 5b): waiting on `KERNEL` without an IF window →
    the waiting AP does not acknowledge the shootdown → `SMP:SPIN:SHOOTDOWN_TIMEOUT` and INIT for all APs.
15. `test-smp-gs-user` (F8, INV-8): ring 3 on every AP loads GS itself (0x1b or null). 0xf0, 0xe0,
    0xe8 and ticks from CPL3 count in the CPU's own anchor → `SMP:GS:PASS`. On the old stubs
    this gave `SMP:GS:HIJACK` (red evidence).
16. `test-smp-gs-iret-fault`: an `iretq` to CPL3 fails (SS 0x13) after the exit `swapgs` → the
    GS-7 fixup restores the anchor → `SMP:GS:IRET_FIXUP:PASS`.
17. `test-smp-gs-nmi-user`: an NMI to an AP in CPL3 with user GS → AP recognised via the
    IST address, without GS → `SMP:FAULT:AP_NMI_IST:PASS`.

18. `test-smp-shared-scratch` (control profile, step 5b-2): all CPUs in scratch slot 0 →
    `SMP:SCRATCH:CROSS:CPU=n` (occupied slot PTE or foreign pattern). This proves that
    `SMP:SCRATCH:PASS` measures a real hazard.
