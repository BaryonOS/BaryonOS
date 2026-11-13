# K06B-LOCK: liveness map of the contended fixture phases and attack on the progress-window candidate

Task ID: K06B-LOCK. Read-only analysis of the frozen worktree `snap-67e592c` (commit `67e592c`).
Date: 12 November 2026. Nothing was built, booted or measured for this report.

Labels: **[S]** verified in source, **[D]** verified in data, **[I]** inferred, **[U]** unverified.
Path aliases (relative to the worktree root): `smp.rs` = `kernel/src/m2/smp.rs`, `lock.rs` =
`kernel/src/m2/lock.rs`, `lib.rs` = `crates/smp-lock/src/lib.rs`, `sched.rs` =
`crates/smp-lock/src/sched.rs`, `serial.rs` = `kernel/src/serial.rs`, `paging.rs` =
`kernel/src/m2/paging.rs`, `run_smp.py` = `tools/run_smp.py`, `TIMING` =
`docs/verification/k06-smp-sched/TIMING-INVESTIGATION.md`, `ti/` =
`docs/verification/k06-smp-sched/timing-investigation/`, `specs/` = the L0 specifications on branch
`docs/k06-step-specs` (`docs/implementation/...`).

## 0. Coverage bounds

- Read in full: `lock.rs`, `lib.rs` (non-test part), `serial.rs`, `run_smp.py`, `TIMING`,
  `CLOCK-INVESTIGATION.md`, `K06A-SPEC.md`, `K06A-SITES.tsv`, ADR-082, specs 01, 03, 09 and
  06 §1–4, `K06-SMP.md` §4.1–4.2 and §11.
- `smp.rs`: read lines 1–470, 690–1110, 1160–4641, 4722–4810, 4926–4998. Not read line by line:
  470–690 (trampoline assembly; only grepped for `sti`/`cli`), 1110–1160, 4641–4722, 4810–4925 (M3
  glue). `sched.rs` 1–140, `paging.rs` 876–970 and `unmap_shot`.
- Data: `ti/p-*.jsonl.gz` and `ti/m-*.jsonl.gz` were re-analysed in memory (no file written);
  `ti/map/hosted-margins.tsv` and `hosted-markers.tsv` were read by row. Hosted raw serial logs of
  the three incidents are not in the snapshot.
- Hosted Graphify MCP: `recall` and `graphify_callers(within_ms)` were used as context only. The
  index is `main` at `49f80b1` (older line numbers, `settle_scheduler` absent). Every fact below
  was taken from the snapshot.
- QEMU source was not read. Statements about QEMU internals (BQL, PAUSE exits, TSC source) are
  [U] unless a repository file states them.

## 1. Part 1 — liveness map per phase

### 1.1 Facts common to all phases

| Fact | Evidence |
|---|---|
| At most 8 CPUs take part: `MAX_CPUS = 8`, `MAX_APS = 7`; a ninth MADT CPU is skipped. | [S] `paging.rs:16`, `smp.rs:46-47`, `:1309-1312` |
| All 66 cases except early terminal ones run every phase; M3 images run the whole fixture before `SMP:M3:WAITING`. 5 multi-thread cases have 8 or 9 vCPUs (`eight`, `nine-capacity`, `m3-eight`, `m3-migrate-eight`, `m3-load-eight`), 3 single-thread cases have 8. | [S] `smp.rs:1604-1633`, `run_smp.py:22-90` |
| BSP interrupt state: IF=0 from boot. Its GS anchor is set first; IDT gates for 0x20, 0x27, 0xd0, 0xe0, 0xe8, 0xf0, 0xff exist before the first AP starts. | [S] `smp.rs:1236-1239`, `:1293-1300` |
| `bsp_timer_init` runs before `GO`: records `KERNEL_CR3`, programs the 8259 (only IRQ0 unmasked) and PIT channel 0 at 1 kHz, sets `BSP_LOCAL.irq_ready` except in `test-smp-bsp-spin-irq-off`. The BSP LAPIC timer stays masked. | [S] `smp.rs:1348`, `:2579`, `:2583`, `:2617-2636`, `:2640-2642` |
| The first `sti` the BSP ever executes is `idle_halt` inside `clock()` (phase P6). Before that it takes no contended KERNEL/ASPACE lock (window never opens) and never halts. IRQ0 is therefore programmed but not accepted during sync, IPI, shootdown, preemption and spin. | [S] by enumeration of BSP paths `smp.rs:1348-2496`; `:2413`, `:2457`, `:2489` are uncontended acquisitions; first halt `:2735` via `:2753` |
| The IRQ0 stub touches only `CLOCK_EDGE_TSC`, `CLOCK_TICKS` and the PIC EOI port. No GS, no lock, no `irq_depth`, no switch, no `need_resched`. | [S] `smp.rs:426-448` |
| The BSP never loads another CR3: `bsp_switch` has no CR3 write; `sched_switch` and `gs_switch` are reached only for APs. | [S] `smp.rs:2179-2187`, `:2678-2697` |
| APs: IF=0 from the trampoline until their first `idle_halt`; `irq_ready` is set in `ipi_phase`, after stress and serial lines. | [S] `smp.rs:482`, `:563`, `:883-889`, `:923`, `:1070` |
| `within_ms(limit, ready)` = `limit` evaluations of `ready()`, each followed by `wait_us(1_000)`; no evaluation after the last sleep; the result of `wait_us` is ignored. Time base: count of BSP polls of PIT channel 2 with IF=0. | [S] `smp.rs:1396-1403`, `:751-772` |
| Measured stretch of that time base: 50 polls of 1 ms took 51–57 ms (4 vCPUs on 4 host CPUs), 51–93 ms (9 on 4), 54–122 ms (4 on 2), median 52 / 54 / 74 ms. A `within_ms` limit is therefore about 4 %, 8 % and 48 % longer in wall-clock time on those hosts (median). | [D] `ti/p-four-h4`, `p-nine-h4`, `p-four-h2` (host-time gaps between `SMP:DIAG:P=` lines, recomputed), local QEMU 8.2.2 |
| `within_tsc` and `idle_until` use the calibrated TSC; `idle_until` checks its deadline only when `hlt` returns. | [S] `smp.rs:2530-2556` |
| `TSC_PER_MS` is the minimum of ten 10 ms PIT samples, so it can only be too large; TSC windows are never shorter than nominal. | [S] `smp.rs:340-350`, `:2585-2595` |

### 1.2 Work, contended lock and bounding wait

| Phase | Who works | Contended lock (type, max contenders) | Wait that bounds the phase (site, value, time base) | Other bounds inside the phase |
|---|---|---|---|---|
| **Sync stress** | BSP and every AP: 20 000 iterations each (`stress`), then 200 serial lines each. | `LOCKED_TOTAL`: raw `TicketLock::lock()`, no IF change, no relax window, **no stuck bound**; 8 contenders. Then `SERIAL` (`IrqLock` level 70, no window, not lockdep-tracked, stuck bound active); 8 contenders. | `smp.rs:1352-1357`: 200 × `wait_us(10_000)` ≈ 2 s, PIT poll count, **starts after the BSP's own stress and lines**. Fails `SMP:SYNC:TIMEOUT:DONE=:TOTAL=` then `serial::fail`. | BSP's own share `:1350-1351` is unbounded in the kernel (host watchdog 60 s, `run_smp.py:130`). Each `SERIAL` wait: 400 ms TSC (`lock.rs:250-258`). |
| **Preemption start** | One AP at a time: IPI, arm LAPIC timer (period ≈ 320 µs [I] from `smp.rs:2316-2320` and `:4724-4727`), two threads of 2 000 PAUSEs per count. | none | `smp.rs:2339` and `:2345`: 500 ms each, per AP, PIT. `SMP:PREEMPT:TIMEOUT:APIC=` → `quarantine_all`. | — |
| **Preempt under lock** | BSP holds `PREEMPT_LOCK`; on every AP the running thread becomes a waiter and ticks are deferred. | `PREEMPT_LOCK` (`IrqLock` level 10, window open for IF=1 threads); holder BSP, up to 7 waiters. | `smp.rs:2418`, `:2425`: 500 ms each per AP, sequential, **lock held**; `:2439`: 500 ms per AP after release. All fatal (`serial::fail`). | Every AP waiter: 400 ms TSC from its first contended poll, while the BSP's own steps allow up to 7 × 1 000 ms. |
| **Preemption stop** | Each AP: one tick switches home, home loop reports `RESUMED`. | none | `smp.rs:2369`: 200 ms per AP, PIT. `SMP:PREEMPT:STOP_TIMEOUT` → quarantine. | — |
| **Spin window** | BSP holds `SPIN_LOCK`; each AP is IPI'd, stores `spun=1`, waits; one kernel shootdown round; release; FIFO chain. | `SPIN_LOCK` (`IrqLock` level 10, window); holder BSP, up to 7 waiters. | `smp.rs:2462`: 100 ms per AP, PIT, **lock held**; round 100 ms TSC (`:2472`, `:1812`); `:2482`: 100 ms per AP after release (fatal). | AP1's wait spans the whole hold: up to 7 × 100 ms + 100 ms allowed by the BSP's own limits against 400 ms TSC. |
| **Scheduler start** | Per AP: IPI, arm timer, first tick dispatches a thread; then a cross-CPU wake with 0xe8, ≥ 4 CR3 switches, both threads advance. | `RUNQ[slot]` (`IrqLock` level 40, taken in the tick handler, no window): BSP against that AP's handler, 2 contenders. | `smp.rs:3426`, `:3442`, `:3449`: 500 ms each per AP, PIT. `SMP:SCHED:TIMEOUT:APIC=` → quarantine. | 400 ms per `RUNQ` wait. |
| **Scheduler stop / settle** | Every AP: next tick sees STOP, goes home, masks its timer, stores `RESUMED`. | `RUNQ[slot]` (own). | `smp.rs:3476`: 200 ms for all APs together, PIT → `SMP:SCHED:STOP_TIMEOUT`. `:1432` (`settle_scheduler`): 200 ms, result ignored. | — |
| **Wake ping-pong** | Threads A and B (AP1, AP2, or one AP): 10 000 turns each under `KERNEL`, each turn wakes the partner (`RUNQ[partner]`, 0xe8) and blocks through a self 0xe8. | `lock::KERNEL` (level 10, window): 2 contenders. `RUNQ[slot]`: waker against that CPU's own switch (later also the BSP for the kill), ≤ 3. | `smp.rs:3642`: `WAKE_DEADLINE_MS` 20 000 ms, PIT; `:3644`: 100 ms. `SMP:WAKE:TIMEOUT:A=:B=` → quarantine. | 400 ms per `KERNEL`/`RUNQ`/`ICR` wait. |
| **Kill** | Target AP: 0xe8, then at least one further switch (DEAD only after the stub left the stack). | `RUNQ[slot]`. | `smp.rs:3666`: 500 ms; `:3686`: `SHOOTDOWN_DEADLINE_MS` 100 ms, **PIT-polled**. `SMP:KILL:TIMEOUT:APIC=` → quarantine. | — |
| **Scratch** | BSP and every AP: 1 024 zero/fill/verify rounds on an own frame (no lock, local `invlpg` only), then 1 000 `KERNEL` acquisitions with a non-atomic shared increment. | `lock::KERNEL` (level 10, window on BSP and APs): 8 contenders during the BSP's share, 7 during the wait. | `smp.rs:3785`: 500 ms, PIT, **after the BSP's own share** (`:3784`). `SMP:SCRATCH:TIMEOUT` → `quarantine_all`. | Each `KERNEL` wait: 400 ms TSC. BSP share otherwise unbounded. |
| **Lock pair** | BSP and every AP: 1 000 `lock_pair` rounds. | `PAIR[0]` (level 20, key 1, window): 8 / 7 contenders. `PAIR[1]` is requested only while `PAIR[0]` is held, so it is never contended [I] from `lock.rs:184-190`. | `smp.rs:3877`: 2 000 ms, PIT, after the BSP's share. `SMP:LOCKPAIR:TIMEOUT` → quarantine. | 400 ms per acquisition. |
| **ICR** | AP1 and AP2: 10 000 × send 0xe8 to the partner, then spin `sti; nop; pause; cli` until the partner counted it; every 0xd0/0xe8 entry sends a self 0xf0 from the handler. | none across CPUs (per-CPU `ICR` lock, thread holds it with IF=0). | `smp.rs:3995`: `WAKE_DEADLINE_MS` 20 000 ms, PIT → `SMP:ICR:TIMEOUT`. | Per round: 65 536 loop iterations on the sender (`:3955`), a count, not a time → `SMP:ICR:LOST`. |
| **Isolation / unmap race** | Threads 0 and 1 on AP1 and AP2 (one AP runs both): 10 000 write/read rounds each; then the reader loop, one process round, a fault. | `RUNQ` only (BSP against the tick handler). | `smp.rs:4301`: `ISO_DEADLINE_MS` 20 000 ms; `:4315` 500 ms; `:4364` 1 000 ms; all PIT. | Process round 100 ms TSC, 1 target. |

Sources [S]: `smp.rs:774-796`, `:363-373`, `:883-889`, `:1349-1377`, `:2215-2247`, `:2325-2443`,
`:2454-2496`, `:3295-3358`, `:3398-3499`, `:3555-3592`, `:3607-3705`, `:3736-3825`, `:3838-3904`,
`:3945-4031`, `:4156-4180`, `:4260-4403`; `serial.rs:108-163`; `lock.rs:224-258`;
`sched.rs:101-121`; `paging.rs:942-968`.

### 1.3 What the BSP executes while it waits

| Wait | IF | BSP lockdep | CR3 | IRQ0 | `irq_ready` (BSP) |
|---|---|---|---|---|---|
| Sync `:1352-1357` | 0 (never yet 1) | empty (the raw lock is not tracked) | kernel | programmed, never yet accepted | true (false in `test-smp-bsp-spin-irq-off`) |
| Preemption `:2339`, `:2345`, `:2369`; resume `:2439` | 0 | empty | kernel | programmed, not accepted | as above |
| Under lock `:2418`, `:2425` | 0 | **KERNEL bit set** (`PREEMPT_LOCK`) | kernel | as above | as above |
| Spin `:2462` | 0 | **KERNEL bit set** (`SPIN_LOCK`) | kernel | as above | as above |
| Spin grant `:2482` | 0 | empty | kernel | as above | as above |
| Every wait from scheduler start on (`:3426` … `:4622`) | 0 inside `wait_us`; IRQ0 is taken only in `idle_halt` and in BSP KERNEL/ASPACE lock windows | empty | kernel | live since `clock()` | true (the control image has already ended at `bsp_spin`) |
| Scratch `:3785`, lock pair `:3877` | 0 | empty (own share finished, `KERNEL`/`PAIR` released) | kernel | live | true |

All waits busy-poll `inb(0x61)` with PAUSE inside `wait_us` [S `smp.rs:766-769`]. The BSP holds
no ticket in any of them, so it is not part of a FIFO queue while it waits.

### 1.4 Monotone progress evidence that exists today

| Evidence | Writer(s) | Atomic / single-writer | Usable as a polled progress value |
|---|---|---|---|
| `ATOMIC_TOTAL` `smp.rs:775`, `:782` | every CPU in `stress`, **before** taking the lock | atomic RMW, shared | Yes for the counter part. Counts iteration starts. Flat during the whole serial tail. |
| `LOCKED_TOTAL` value `:776`, `:789-792` | lock holder | non-atomic under the raw lock | No: reading needs the lock, the reader would join the FIFO queue. |
| `INSIDE`, `VIOLATIONS` `:777-778`, `:784-793` | every CPU | atomic | Verdict only. |
| `ApContext.state` DONE / IPI_READY `:888`, `:926` | that AP (BSP during bring-up) | single writer per AP | Completion flag, one step per AP. |
| Serial line sequence `:366` | — | local variable, **no kernel counter** | Not available; the host sees the lines. |
| `SCRATCH_COUNT` `:3716`, `:3757` | every CPU under `KERNEL` | plain load + store, shared; monotone only if exclusion holds | Lock-free load is possible; not single-writer. |
| Scratch fill rounds `:3740-3753` | — | **no counter** | Not available. |
| `CpuLocal.scratched` `:1002`, `paired` `:1007` | that AP | single writer, 0 → 1 | Completion flag only. The BSP has none (its share is synchronous). |
| `PAIR[0]`, `PAIR[1]` values `:3849-3850` | lock holder | under the pair | No: needs both locks. No per-CPU round counter. |
| `PROGRESS[tid]` (ping-pong `:3579`, isolation `:4176`, scheduler `:3379`, victim `:3567`) | the thread `tid` | single writer per tid | Yes (ping-pong and isolation run to 10 000). |
| `icr_sent` `:3954`, `resched_count` `:3195`, `icr_done` `:3966` | that AP (thread / its own stub) | single writer | Yes. |
| `tick_count`, `switches`, `deferred`, `counts[]`, `cr3_switches`, `sched_state`, `preempt`, `spun` | that CPU | single writer | Yes for the start/stop waits; already the `ready` conditions. |
| `RACE_READS` `:4195` | reader thread | single writer | Yes. |

Result [S]: the candidate's "iterations completed" and "serial lines written" per CPU do not exist
for sync, and neither fill rounds nor `KERNEL` rounds per CPU exist for scratch. With today's
variables a polled value is flat by design during the serial tail (0.38–0.51 s at 9 on 4, [D]
`TIMING:95-96`) and during the fill part.

## 2. Part 2 — lock algorithm, stuck bound, acknowledgement deadlines

### 2.1 `crates/smp-lock`

| Item | Fact | Evidence |
|---|---|---|
| Algorithm | `ticket = next.fetch_add(1)`; spin while `owner != ticket`; unlock = `owner.fetch_add(1)`. Strict FIFO by ticket. u32 wrap is handled. | [S] `lib.rs:66-73`, `:143-147`, `:380-387` |
| `lock_with(relax)` | `relax()` runs only on a failed poll, before `spin_loop()`. An uncontended acquisition never calls it. | [S] `lib.rs:66-73`, `:373-376` |
| Handoff | The next owner is fixed at release. No waiter can leave the queue: a ticket that gives up makes the lock permanently unusable. | [S] `lib.rs:31-35`, `:77-87`, `:342-344` |
| `try_lock` | CAS `next: owner → owner+1`; succeeds only on a free lock; terminal paths. | [S] `lib.rs:90-101`, `lock.rs:167-169` |

### 2.2 Where `LOCK_STUCK_MS` can fire

| Item | Fact | Evidence |
|---|---|---|
| Value and clock | `4 × SHOOTDOWN_DEADLINE_MS` = 400 ms of calibrated TSC. | [S] `smp.rs:334`, `:353-359` |
| Scope | Every contended `IrqLock::take` in every `native-smp` build: levels 10 (`KERNEL`, `STUCK_TEST`, `PREEMPT_LOCK`, `SPIN_LOCK`, `IDLE_LOCK`, `OUTER`), 20 (`PAIR`, `ASPACE`), 30 (`INNER`, self-test only), 40 (`RUNQ`), 50 (`SHOOTDOWN`), 60 (per-CPU `ICR`), 70 (`SERIAL`), and the M3 `ASPACE`/`RUNQ` arrays. Not the raw `LOCKED_TOTAL`. | [S] `lock.rs:250-258`; instances `lock.rs:110`, `:335-337`, `serial.rs:108`, `smp.rs:270`, `:303`, `:376`, `:776`, `:2404`, `:2446`, `:2981`, `:3159-3167`, `:3829-3832`, `:4046-4049`, `kernel/src/m2/m3.rs:353-368` |
| Start of the interval | Lazily at the first failed poll, after the first relax window; one interval per `take`, never restarted. | [S] `lock.rs:251-258` |
| Order of checks | The poll loop is `load owner → relax → deadline check → PAUSE`. The deadline check is not preceded by a second grant check. A waiter that is descheduled between the load and the check and resumes after the deadline reports STUCK even if the lock was granted to it meanwhile. | [S] `lib.rs:68-71` with `lock.rs:252-258` |
| Window | `sti; nop; pause; cli` only for level ≤ 20, outside a handler, with `irq_ready`, not in `test-smp-spin-irq-off`. | [S] `lock.rs:224-238` |
| Effect | `SMP:LOCK:STUCK:<level>:CPU=<n>` through `write_terminal`, then `M1:FAIL:SMP_LOCK_STUCK` (exit 0x10 only in the control). | [S] `smp.rs:2086-2100` |
| Hosted record | In run 43187501026 (3 301 boots) the marker occurs only in the `lock-stuck` case (50 of 50). | [D] `ti/map/hosted-markers.tsv:21` (only STUCK row) |
| Local record | 134 completed local boots, including 18 with scratch up to 2.65 s on 2 host CPUs: no STUCK marker. | [D] `ti/m-*.jsonl.gz`, `ti/p-*.jsonl.gz` (marks searched), `TIMING:67-69` |

### 2.3 Maximum queue per phase and where a legitimate wait can exceed 400 ms

A waiter's wait is the remaining hold of the owner plus, for each earlier ticket, that CPU's hold
and the time until its vCPU thread runs again. "Design hold" is what the fixture's own limits
allow the holder.

| Phase | Lock | Max waiters | Design hold of the owner | Can a legitimate wait exceed 400 ms? |
|---|---|---|---|---|
| Sync | `SERIAL` (no window) | 7 | one line, about 14 bytes, 2 port accesses per byte | Only if holder or earlier waiters lose ≥ 400 ms in total (≥ 57 ms per predecessor with a full queue). [I] |
| Sync | `LOCKED_TOTAL` (raw) | 7 | 2 volatile accesses and one PAUSE | No stuck bound exists; a wait is unbounded. [S] |
| Preempt under lock | `PREEMPT_LOCK` | 7 (one thread per AP, `smp.rs:2172-2176`) | **Sum over all APs of two BSP waits, each allowed 500 ms**; all waiters start together | **Yes by design**: the BSP may legally hold up to 7 s; any hold above 400 ms after the first waiter arrived is fatal on that waiter. Local phase total (all of P4) 0.14–0.21 s at 8 on 4. [S] `:2413-2436`, [D] `ti/m-eight-h4` |
| Spin window | `SPIN_LOCK` | 7 | **Up to 7 × 100 ms + 100 ms round** | **Yes by design** for AP1. Local phase total ≤ 19 ms. [S] `:2457-2480`, [D] |
| BSP spin | `lock::KERNEL` | 1 (BSP) | AP1: ≤ 100 ms + ≤ 100 ms TSC | If AP1's thread loses > 200 ms while holding. [I] |
| Idle part 2 | `IDLE_LOCK` | 1 (BSP thread) | AP1: ≤ 100 ms + ≤ 100 ms TSC (`:3013-3020`) | As above. [I] |
| Scheduler, isolation | `RUNQ[slot]` (handler, no window) | 1 | queue operations | Only on a single ≥ 400 ms loss of the holder. [I] |
| Ping-pong | `lock::KERNEL` | 1 | one turn: `RUNQ` + ICR send | As above; 20 000 acquisitions. [I] |
| Ping-pong, kill | `RUNQ[slot]` | 2 | queue operations | As above. `test-smp-quarantine-sched-busy` holds `RUNQ[0]` for about the 100 ms kill wait with no waiter. [S] `:3311-3319`, `:3678-3690` |
| Scratch | `lock::KERNEL` | 7 (8 contenders while the BSP runs its own share) | load + store | As sync `SERIAL`: ≥ 57 ms average per predecessor. 8 000 acquisitions per boot at 8 CPUs. [I] |
| Lock pair | `PAIR[0]` | 7 | two increments, uncontended inner lock | As scratch; 8 000 acquisitions. [I] |
| M3 | `lock::KERNEL` | 7 | one M3 event | Hosted maximum 23 ms (`m3-load-eight`). [D] `K06A-SPEC.md:209` (not re-derived) |

### 2.4 Shootdown and kill acknowledgements (100 ms)

| Phase | Site | Targets | Clock | What each target must do |
|---|---|---|---|---|
| P3 shootdown | `smp.rs:1691` (two rounds) | all online APs, ≤ 7 | TSC, one deadline per generation (`:1812-1813`) | one 0xe0 interrupt from `hlt` |
| P5 spin window | `:2472` | all APs, ≤ 7 | TSC | one 0xe0 inside the lock's `sti` window (contended-lock context) |
| P8 BSP spin | `:2923` (AP1 initiates) | BSP only | TSC | one 0xe0 inside the BSP's `KERNEL` wait window; before it, 100 ms TSC for the BSP to queue |
| P15 unmap race | `:4343` → `paging.rs:608` | `ACTIVE[0]` minus the BSP: AP1 (checked `:4386`) | TSC | one 0xe0 in thread context or after its tick handler |
| P15 deaf control | `:4322` | all APs | TSC | one 0xe0 (normal ack), then the process round times out by design |
| P15 exit of both spaces | `:4226`, `:4232` | empty set after `stop_scheduler` | TSC | none (protocol only) |
| P16 GS (three images) | `:4605` | all APs | TSC | one 0xe0 from ring 3 |
| P11 kill | `:3686` | 1 AP | **PIT poll count** (`within_ms`) | one 0xe8, then a second scheduler pass (tick) under its own `RUNQ` (`sched.rs:114-121`, `smp.rs:3267-3271`) |
| M3 kill | `kernel/src/m2/m3.rs:2969-2982`, `kernel/src/m2/m3/smp_lifecycle.rs:11`, `:62` | 1 CPU | TSC (`m3_time_ms`, `smp.rs:4926-4928`) | 0xe8 or tick, then contended `KERNEL` |

### 2.5 The 100 ms and 200 ms `within_ms` sites

| Site | Limit | Needs | Class |
|---|---|---|---|
| `:1556` IPI round | 100 | one 0xf0 on one AP | one interrupt |
| `:1638` park | 100 | one 0xf0 and one home-loop pass | one interrupt |
| `:1902` probe | 100 | one 0xf0 and one home-loop pass | one interrupt |
| `:2462` spin waiter | 100 | one 0xf0 and one home-loop pass up to `spun=1` | one interrupt (BSP holds a lock) |
| `:2482` spin grant | 100 | FIFO handoff through up to 7 APs | **contended lock work** |
| `:2944` AP1 holds `KERNEL` | 100 | one 0xf0, uncontended acquisition | one interrupt |
| `:3106` AP1 holds `IDLE_LOCK` | 100 | one 0xf0, uncontended acquisition | one interrupt |
| `:3644` both blocked | 100 | per thread: contended `KERNEL`, self 0xe8, switch under `RUNQ` | **contended lock work** (2 contenders) |
| `:3680` sched-busy (control) | 100 | one tick on AP1, own `RUNQ` | one interrupt |
| `:3686` kill | 100 | one 0xe8 and one further tick, own `RUNQ` | two interrupts, uncontended lock |
| `:4599` GS IPI | 100 | one 0xf0 from ring 3 | one interrupt |
| `:1432` settle | 200 | one tick per AP, own `RUNQ`; deaf APs never report (full expiry in 3 controls, result ignored) | one interrupt per AP, shared window |
| `:2369` preempt stop | 200 | one tick, home-loop pass | one interrupt |
| `:3476` scheduler stop | 200 | one tick per AP, own `RUNQ` | one interrupt per AP, shared window |
| `:4622` GS stop | 200 | one tick per AP from ring 3 | one interrupt per AP, shared window |

## 3. Part 3 — can the BSP halt instead of polling?

Preconditions [S]: `idle_halt` stops with `SMP_IDLE_STATE` unless this CPU's lockdep mirror is 0
and CR3 equals `KERNEL_CR3` (`smp.rs:2560-2572`). `idle_until(ms, ready)` halts between checks and
evaluates its TSC deadline only after a wake-up (`:2545-2556`). Spec BT-3 allows `sti; hlt; cli`
only in the idle thread, on the kernel CR3, with an empty lockdep set (`specs/k06-steps/06-…:29`).
`idle_halt` does not read `irq_ready`; `clock()` already halts in the
`test-smp-bsp-spin-irq-off` image before `bsp_spin` (`smp.rs:1607-1609`).

Common to every row [S]: IRQ0 is programmed (`:1348` precedes all sites); the BSP is on the kernel
CR3; the only interrupt the BSP can take there besides IRQ0 is one that a CPU sends to it, and no
CPU sends one during a `within_ms` wait (0xe0 to the BSP only from `bsp_spin_work` after the BSP
queued on `KERNEL`, `:2921-2923`; 0xe8 to the BSP only in `idle`, `:2993`, `:3015`, `:3111`). "Safe"
is an inference from that enumeration [I]. `irq_ready` of the BSP is true at every site, except in
the `test-smp-bsp-spin-irq-off` image, where it is false throughout (`:2640-2642`); that image
reaches the sync wait and sites 2, 3 and 8–17 only. No row depends on it.

| # | Site | Limit (ms) | BSP lockdep | Control that expects expiry or depends on IF=0 here | Safe to halt | Reason |
|---|---|---|---|---|---|---|
| S | `:1352-1357` sync | 200 × 10 | empty | none | yes | First BSP `sti` moves from P6 to P1; IRQ0 is then consumed from here on. `clock()` uses relative tick counts (`:2751-2753`). APs have IF=0 and take nothing. |
| 1 | `:1432` settle | 200 | empty (callers hold nothing when `SCHED` runs) | full expiry in `resched-deaf`, `quarantine-sched-busy`, `proc-shootdown-deaf`; result ignored | yes | — |
| 2 | `:1545` IPI ready | 1 000 | empty | none | yes | — |
| 3 | `:1556` IPI round | 100 | empty | expiry expected: `ap-ipi-deaf`, `quarantine-lock-held`, `quarantine-lock-pending` | yes | The expiry needs a wake-up to be noticed (IRQ0). |
| 4 | `:1567` #DF sleep | 1 000 | empty | inverse: the AP must end the VM first | yes | Pure sleep. |
| 5 | `:1593` lock-stuck sleep | 1 000 | **`STUCK_TEST` held** | inverse | **no** | Lockdep not empty: `SMP_IDLE_STATE`, BT-3. |
| 6 | `:1600` NMI sleep | 1 000 | empty | inverse | yes | Pure sleep. |
| 7 | `:1638` park | 100 | empty | none | yes | — |
| 8 | `:1902` probe | 100 | empty | none | yes | — |
| 9 | `:2339` preempt progress | 500 | empty | expiry expected: `ap-timer-dead` | yes | — |
| 10 | `:2345` both advance | 500 | empty | none | yes | — |
| 11 | `:2369` preempt stop | 200 | empty | none | yes | — |
| 12 | `:2418` first deferred tick | 500 | **`PREEMPT_LOCK` held** | none | **no** | Lockdep not empty; BT-3. A longer hold also moves the AP waiters toward their 400 ms. |
| 13 | `:2425` two more ticks | 500 | **`PREEMPT_LOCK` held** | none | **no** | As 12. |
| 14 | `:2439` resume | 500 | empty (dropped `:2436`) | none | yes | — |
| 15 | `:2462` spin waiter | 100 | **`SPIN_LOCK` held** | `spin-irq-off-control` concerns the AP windows, not the BSP | **no** | Lockdep not empty; BT-3. |
| 16 | `:2482` spin grant | 100 | empty (dropped `:2480`) | none | yes | — |
| 17 | `:2944` AP1 holds `KERNEL` | 100 | empty | `bsp-spin-irq-off-control` needs the BSP deaf **during its `KERNEL` wait**, which follows this site | yes | AP1 sends its round only after the BSP's lockdep shows KERNEL, set after `cli` in `take`. AP1's 100 ms TSC wait (`:2923`) then measures the BSP's wake-from-halt latency instead of its poll latency. |
| 18 | `:3106` AP1 holds `IDLE_LOCK` | 100 | empty; `BSP_PHASE = BSP_DEFER` | none | yes | Only a 0xe8 switches the BSP; the self 0xe8 is sent after this wait (`:3111`), AP1's after the BSP thread queued (`:3013-3017`). The check `switches + 2` (`:3127`) is unaffected by IRQ0. AP1's 100 ms TSC wait includes the BSP's reaction time. |
| 19 | `:3426` thread progress | 500 | empty | none | yes | — |
| 20 | `:3442` resched, CR3 switches | 500 | empty | none | yes | — |
| 21 | `:3449` both advance | 500 | empty | none | yes | — |
| 22 | `:3476` scheduler stop | 200 | empty | none | yes | — |
| 23 | `:3642` ping-pong | 20 000 | empty | none | yes | — |
| 24 | `:3644` both blocked | 100 | empty | none | yes | — |
| 25 | `:3666` victim runs | 500 | empty | none | yes | — |
| 26 | `:3680` sched-busy | 100 | empty | precondition of `quarantine-sched-busy` | yes | — |
| 27 | `:3686` kill | 100 | empty | expiry expected: `resched-deaf`, `quarantine-sched-busy` | yes | Changes the kill deadline from PIT poll count to TSC (as the M3 kill already is). |
| 28 | `:3785` scratch | 500 | empty | none (`shared-scratch` ends by its own marker) | yes | — |
| 29 | `:3877` lock pair | 2 000 | empty | none | yes | — |
| 30 | `:3995` ICR | 20 000 | empty | `icr-race-control` ends through the AP-side loop bound | yes | AP handlers send IPIs to themselves only (`:2154-2155`). |
| 31 | `:4301` isolation | 20 000 | empty | none | yes | — |
| 32 | `:4315` reader runs | 500 | empty | none | yes | — |
| 33 | `:4329` deaf precondition | 500 | empty | precondition of `proc-shootdown-deaf` | yes | — |
| 34 | `:4364` fault or stale read | 1 000 | empty (guards dropped `:4340-4344`) | positive detector of `proc-no-shootdown-control` | yes | — |
| 35 | `:4576` enter ring 3 | 500 | empty | none | yes | — |
| 36 | `:4582` IRET sleep | 1 000 | empty | inverse | yes | Pure sleep. |
| 37 | `:4590` NMI sleep | 1 000 | empty | inverse | yes | Pure sleep. |
| 38 | `:4599` GS IPI | 100 | empty | none | yes | — |
| 39 | `:4612` resched and ticks | 500 | empty | none | yes | — |
| 40 | `:4622` GS stop | 200 | empty | none | yes | — |

Totals: 36 of 40 sites and the sync wait satisfy the preconditions; 4 do not (5, 12, 13, 15).
No site is "unknown" on the source reading; three properties are not verified by execution and
apply to every "yes":

1. **Liveness now depends on IRQ0 delivery.** `idle_until` never re-checks its deadline without
   an interrupt. A dead IRQ0 path would hang the first halting wait (host watchdog, no marker)
   instead of timing out on PIT channel 2. `clock_at` has the same property today
   (`smp.rs:2718-2737`), but only from P6 on. [S]
2. **Deadline semantics change** from "N BSP polls" to "N ms of TSC". On an oversubscribed host
   that is 8–48 % less wall-clock time ([D] section 1.1), against one fewer busy vCPU thread.
   The net effect on healthy boots is not measured. [U]
3. **Pending interrupts at the first `sti`** (firmware leftovers on the BSP LAPIC or PIC) would
   surface in P1 instead of P6. Same exposure in kind as today. [U]

A recalled workspace note (unverified) states that a one-shot LAPIC timer with initial count 1
froze the whole VM under QEMU/TCG; a LAPIC one-shot as a second wake source for a bounded halt
would need its own test.

## 4. Part 4 — attack on the candidate

Candidate as given: exact final checks unchanged; (1) a monotone progress value from single-writer
per-CPU counters must advance within every TSC window W, else a distinct fatal marker; (2) hard cap
of 10 × the present deadline; (3) measured/limit logged against the present deadline, enforced by
the host oracle only where guest vCPUs ≤ host CPUs; the BSP halts between polls.

### 4.1 Structural facts the candidate must absorb

| # | Fact | Consequence |
|---|---|---|
| F1 | During the sync phase the BSP is itself a contender on the raw lock (`smp.rs:1350`, `:783`; `lib.rs:60-73`), with IF=0 and no bound. With FIFO and equal work all CPUs finish within a few rotations of each other, so the BSP is inside its own share for almost the whole phase [I]. | A deadlock, a broken FIFO order that starves the BSP, or an AP that dies holding a ticket hangs the BSP **before any progress rule runs**. Today and under the candidate the verdict is the 60 s host watchdog without a kernel marker. The sync deadline (and a progress window placed at the same spot) covers only the tail. ADR-082's `sync-deadlock` case cannot produce a kernel marker while the BSP queues behind the stuck AP. |
| F2 | The required per-CPU counters do not exist (section 1.4). `ATOMIC_TOTAL` is flat during the serial tail; `SCRATCH_COUNT` is flat during the fill part and is not single-writer. | New single-writer counters are needed for stress iterations, serial lines, fill rounds, `KERNEL` rounds and pair rounds, or W must exceed the longest flat stretch. |
| F3 | Every `IrqLock` wait keeps its 400 ms TSC bound (section 2.2). | In scratch and lock pair the candidate does not relax the limit that a slow host hits first; see 4.5. |
| F4 | A scratch timeout calls `quarantine_all`, whose `stop_if_held` reports `KERNEL` as held for any AP that holds it or has a ticket in transit (`smp.rs:1442-1480`, `lock.rs:249-263`). | A window or cap expiry on a slow but working boot ends with `SMP:QUARANTINE:LOCK_HELD:10:CPU=<n>` and `M1:FAIL`, as today, unless every remaining AP is in the fill part. [I] |
| F5 | `run_smp.py` has no host-CPU input; the workflow records `cpu_count` and affinity only in a diagnostic artifact (`.github/workflows/native-smp.yml:714-715`). The oracle is an exact sequence (`run_smp.py:576-581`), forbids the substrings `FAIL` and `PANIC` (`:17`), and in `serial-unlocked-control` drops every line between `SMP:ONLINE` and `SMP:SERIAL:PASS` except the ratio (`:158`). | The margin line, the two new markers and the host-CPU predicate need oracle and host-test changes for all 66 expected sequences; the verdict of one transcript becomes host-dependent. |
| F6 | Single-thread TCG cases with 8 vCPUs exist (`m3-eight-single`, `m3-migrate-eight-single`, `m3-load-eight-single`). The same work ran 20 times faster under single-thread TCG ([D] `TIMING:85-92`). | "vCPUs > host CPUs" would exempt three cases that are not oversubscribed. The predicate needs "multi-thread TCG and vCPUs > host CPUs". |
| F7 | Hosted runners have "4 vCPUs = 2 cores with SMT" ([D] `TIMING:11`); QEMU also runs a main-loop thread. | "vCPUs ≤ host CPUs" does not establish that a 4-vCPU guest is free of host contention. No hosted sync or scratch failure on a 4-vCPU case is recorded in the three matrices ([D] `TIMING:14-19`). |
| F8 | Hosted whole-boot time of `m3-load-eight` is 46.0–47.3 s of the 60 s watchdog ([D] `ti/map/hosted-margins.tsv:34`). | Caps of 20 s (sync) and 5 s (scratch) exceed the 12.7 s of slack. On that case the host watchdog fires before the guest cap, without a marker. |
| F9 | "Scratch within 500 ms" is a normative spec requirement (`specs/k06-steps/01-…:53-57`); the sync and lock-pair values have no normative source ([S] `K06A-SPEC.md:168-186`). | Non-fatal 500 ms on any profile needs an accepted ADR and a spec revision, not only a code change. |

### 4.2 Defect classes: which check catches each, and when

"Tail" = the part after the BSP's own share. W and the cap are the candidate's values.

| Defect | Today | Under the candidate | Lost? |
|---|---|---|---|
| **Deadlock**, sync raw lock, while the BSP is still queued | Host watchdog, 60 s, no marker (F1) | Same | No change; not improved |
| Deadlock, sync raw lock, in the tail | `SMP:SYNC:TIMEOUT` ≈ 2 s after the BSP finished | No-progress marker after W | No |
| Deadlock on `SERIAL`, `KERNEL`, `PAIR` with at least one running waiter | `SMP:LOCK:STUCK:<level>` at 400 ms TSC on the waiter | Same, before any W ≥ 400 ms | No |
| Deadlock with no waiter left (last CPU, or holder dead and all others done) | Phase deadline (2 s / 500 ms / 2 s), then for scratch and pair `LOCK_HELD` | No-progress marker after W | No |
| **Livelock** without acquisitions | As deadlock | As deadlock | No |
| Livelock with a counter that still advances | Phase deadline | Only the cap, 10 × later; impossible if every counter counts bounded completed work (20 000, 200, 1 024, 1 000 per CPU) | No, given bounded counters |
| **Starvation of one CPU** (raw lock; FIFO would have to be broken) | AP starved: `SMP:SYNC:TIMEOUT` 2 s after the BSP finished. BSP starved: host watchdog | AP starved: with a summed value, W after the other CPUs finished, at most the cap; with per-CPU values, W. BSP starved: host watchdog | No; later with a summed value |
| Starvation on an `IrqLock` | `LOCK:STUCK` at 400 ms | Same | No |
| **Lost update** | `SMP_SYNC_LOST_UPDATE` (`smp.rs:1372-1374`), `SMP_SCRATCH_LOST_UPDATE` (`:3797`), `SMP_LOCKPAIR_LOST_UPDATE` / `_TORN` (`:3847`, `:3888`) at phase end | Same checks, at phase end, at most the cap later | No |
| **Exclusion violation** | Sync: `SMP_SYNC_EXCLUSION` (`:1375-1377`), reported after the lost-update check. Scratch: only through the lost update; no direct counter (`:3754-3758`) | Same | No |
| **Handoff 100 × slower** | Not reliably caught by the kernel: the BSP's share stretches unmeasured, the tail stays a small fraction (F1). Caught by the host watchdog where the whole phase exceeds its slack. `LOCK:STUCK` needs 57 ms per handoff with 7 waiters. | Same blind spot. On exempt profiles the tail limit is 10 × weaker (cap instead of deadline). The logged figure shows only the tail unless the whole phase (GO to last DONE) is timed. | Weakened on exempt profiles only |
| **Waiter never served while others cycle** | Raw lock: as starvation. `IrqLock`: `LOCK:STUCK` at 400 ms | Same | No |
| **AP died silently**, outside the queue | Phase deadline after the BSP finished (`DONE=` count shows it) | No-progress marker W after the last other progress | No |
| AP died holding a ticket or the raw lock | Host watchdog (sync, BSP queued); `LOCK:STUCK` at 400 ms (`IrqLock` with waiters) | Same | No |
| AP never started its share (IPI lost, still halted) | Phase deadline | W after the others finished | No |
| **Tail slower than 1 × but below 10 × the present deadline on an exempt profile**, whatever the cause | Fatal (this is what rejected two matrices) | Passes with a logged figure | **Yes, by design**: a regression that appears only with more than 4 contenders is bounded by the cap alone |
| Complete stall longer than W inside a tail that still ends within the present deadline | Passes | Fatal if W is below the present deadline | New failure mode, healthy or not |

No defect class in the list is lost outright. The differences are: detection is weakened by a factor
of 10 on the exempt profiles; the sync blind spot of F1 is unchanged; with a summed progress value
a single starved or dead AP is reported later and without attribution unless the marker prints the
per-CPU counters.

### 4.3 Ways a real defect could pass

1. Any slowdown of the lock or of one CPU that keeps the tail between 1 × and 10 × the present
   deadline on `eight`, `nine-capacity`, `m3-eight`, `m3-migrate-eight`, `m3-load-eight` (and on the
   three single-thread 8-vCPU cases if the predicate ignores the TCG mode, F6).
2. A defect confined to the BSP's own sync share (F1): unchanged, host watchdog only.
3. A progress value fed by something that is not completed bounded work (lock attempts, relax
   polls, a count taken before the lock like `ATOMIC_TOTAL` without its bound): livelock would show
   as progress until the cap.
4. A misdetected host CPU count: affinity does not reflect a cgroup CPU quota [U], and 4 logical
   CPUs are 2 cores (F7). The gate could exempt a profile it should enforce, or the reverse.
5. `TSC_PER_MS` too large (all ten calibration samples delayed): W and the cap lengthen in
   proportion. `clock()` would flag more than 1 % (`smp.rs:2789`), but only after the sync phase. [I]
6. The measured figure covers only the tail: a uniform slowdown of the phase is invisible in it.

### 4.4 Ways a healthy boot could fail

1. **`LOCK_STUCK` at 400 ms is unchanged** (scratch `KERNEL`, lock pair `PAIR[0]`, sync `SERIAL`,
   and by design `PREEMPT_LOCK` / `SPIN_LOCK`, section 2.3). The caps of 5 s and 20 s are not the
   binding limits on a slow host.
2. **Counters flat by design** (F2) if W is shorter than the serial tail or the fill part.
3. **Host watchdog before the cap** on `m3-load-eight` (F8).
4. **Time base**: measured and W on TSC are wall-clock; today's limit is a poll count that is
   4–48 % longer under load (section 1.1). On enforced profiles the gate is slightly stricter than
   today. Local headroom at 4 on 4 is large: sync tail ≤ 0.17 s of 2 s, scratch ≤ 0.05 s of 0.5 s
   ([D] `TIMING:61`).
5. **Halting**: dependence on IRQ0 for every deadline; AP-side 100 ms TSC waits at `smp.rs:2923`
   and `:3013` now include the BSP's wake-up from `hlt` (section 3).
6. **Oracle**: any new line not added to all expected sequences, or a marker containing `FAIL`,
   rejects every boot (F5).
7. **W too small for the host**: see 4.5.

### 4.5 Minimum defensible W; per-CPU against global; relation to `LOCK_STUCK`

Data available:

| Source | Content | Limit of the evidence |
|---|---|---|
| [D] `ti/p-*.jsonl.gz`, 24 boots (20 multi-thread), recomputed | 2 565 full windows of 50 BSP polls (50–122 ms of host time each), 167.7 s of contended sync in total. No window without progress. Smallest increment per window: 93 (4 on 2), 276 (9 on 4), 2 729 (4 on 4), 14 228 (9 on 4, single-thread). Lowest rate 936 per second. | Shared counter only, no per-CPU trace. Raw-lock part only: no serial tail, no scratch. Local QEMU 8.2.2. A matrix holds 2 200 multi-thread boots with ≥ 4 vCPUs (44 cases × 50, [S] `run_smp.py:22-90`); the sample is under 1 % of one matrix. |
| [D] `ti/m-*.jsonl.gz`, 110 boots | Scratch (whole phase) up to 0.56 s at 8 on 4 and 2.65 s at 4 on 2; lock pair up to 0.72 s and 2.71 s. No STUCK marker, so no single `IrqLock` wait reached 400 ms. | Phase totals; no stall lengths. |
| [D] `ti/map/hosted-markers.tsv` | No unexpected `LOCK:STUCK` in 3 301 hosted boots. | One matrix. Zero events in 3 301 boots bounds the rate at about 1 in 1 100 boots (95 %), which still allows about 3 per matrix. With the three matrices of `TIMING:141-144` (about 9 800 guests, incident list only, [D] by document) the bound is about 1 per matrix. |
| [D] hosted incident `nine-capacity` r029 | One sample: `TOTAL=157489` when the wait expired, `DONE=0`. The boot ended after 5.6 s against a median of 13.8 s (`ti/map/hosted-margins.tsv:54`). | The counter value at the start of the wait is unknown. The record cannot tell a complete stall of the seven APs for 2 s from slow continuous progress (`TIMING:97-100` says the same). If it was a complete stall, a progress window W ≤ 2 s would have rejected the same boot. **Cannot verify.** |
| Host scheduler | Not measured on the hosted runner. | [U] |

Findings:

- **Below 400 ms no W is defensible in an `IrqLock` phase.** It would be tighter than the existing
  per-wait bound and rests on 20 local boots. [I]
- **Between 400 ms and the present deadline** a window adds detection only where no running waiter
  exists (last finisher, AP in the fill part, AP that never started). [I]
- **W equal to the present deadline (500 ms scratch, 2 s sync, 2 s lock pair)** is the smallest value
  that is neither tighter than an existing accepted limit nor slower in detection than today. Its
  false-failure rate on hosted runners is not established by any data in the snapshot: the only
  hosted sync sample is compatible with a 2 s stall. [I]
- A shorter sync W (for example 500 ms) is 4 times the longest local observation window (122 ms,
  none of which was empty), but is unverified on QEMU 10.2.1. Logging the longest no-progress gap per boot (ADR-082 part 1) would supply the
  missing distribution. [I]

Per-CPU window against a global (summed) one:

| Aspect | Global value | Per-CPU values |
|---|---|---|
| Fails a healthy boot when | every unfinished CPU makes no progress for W | **any one** unfinished CPU makes no progress for W |
| In a FIFO lock part | A descheduled owner or next ticket stalls all CPUs, so both rules see the same event | Same event, plus a CPU descheduled while outside the queue |
| In a lock-free part (scratch fill) | Insensitive to one slow vCPU thread | Fails on one vCPU thread without host CPU for W |
| Exposure | one condition per window | up to 7 conditions per window |
| Extra defects caught | — | None as a class: a starved or dead CPU also stalls the global value once the others finish, at most the cap later (section 4.2). Earlier detection and attribution only. |
| Data | shared counter measured | **no per-CPU data exists** (`TIMING:148-152`); cannot verify |

By FIFO order, a global advance of at least twice the number of contenders in a window implies an
advance of every CPU that was queued throughout [I from `lib.rs:66-73`]. The smallest measured
window increment (93 with 3 contenders, 276 with 7) satisfies this, so the local data imply
per-CPU progress in every window for the lock part only.

Does `LOCK_STUCK` already bound the scratch phase more tightly than a progress window?

| Question | Answer |
|---|---|
| Per wait | **Yes.** A `KERNEL` waiter that is not served within 400 ms of TSC is fatal (`lock.rs:250-258`, `smp.rs:334`). Any stall of the `KERNEL` queue with a running waiter ends there before a window W ≥ 400 ms can expire. |
| Whole phase | **No.** 1 000 acquisitions per CPU, each legal up to 400 ms, give no useful total. The cap is still needed. |
| What a window adds in scratch | Only the no-waiter cases: an AP in the fill part or never started, the last finisher, a dead holder after all others finished. |
| What the candidate does not change | A healthy but slow boot in which one `KERNEL` (or `PAIR[0]`, `SERIAL`) wait reaches 400 ms still fails with `SMP:LOCK:STUCK`. With 7 predecessors that is 57 ms per handoff on average. The stuck check also fires for a waiter that was itself descheduled past the deadline between its last poll and the check (section 2.2). |
| Sync | The raw lock has no such bound; there the window (and the host watchdog during the BSP's share) is the only liveness limit. |

## 5. Unverified items and things that cannot be verified here

| Item | Status |
|---|---|
| QEMU behaviour: TSC and PIT follow host time under TCG; PAUSE leaves the execution loop; port I/O takes the global lock; the PIC keeps one pending IRQ0 | [U] here; stated in `TIMING:125`, `smp.rs:1029-1032`, `CLOCK-INVESTIGATION.md` |
| LAPIC timer period of 320 µs | [I] from `smp.rs:2316-2320` and the comment at `:4724-4727` |
| Host scheduler latencies and any cgroup CPU quota on the hosted runner | [U], no data |
| Whether progress continued during the 2 s of the hosted sync incident | cannot verify: one counter sample |
| Per-CPU progress distribution in any phase; stall lengths in scratch and lock pair | cannot verify: not recorded |
| Hosted margins of sync, scratch, lock pair, preempt-under-lock hold and spin hold | cannot verify: not logged (AC-3 gap) |
| Net effect of halting the BSP on phase durations and on the 36 converted limits | [U], not measured |
| Interrupts pending on the BSP at its first `sti` when that moves to the sync wait | [U] |
| `smp.rs` ranges listed in section 0 as not read line by line | not covered |
