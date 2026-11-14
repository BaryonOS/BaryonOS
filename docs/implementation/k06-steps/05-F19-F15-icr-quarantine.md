# L0 specification — K06 F19/F15: ICR lock and quarantine policy for held locks

> Historical L0 specification, written on 6 November 2026 and ported to `main` on 13 November 2026 from PR #232
> (head `dcd80d1`). English translation; the Dutch original is archived in `docs/history/k06-branch-prs-2026-11-06/k06-steps/`. The step has
> since been implemented; K06 was accepted on 13 November 2026 (`docs/verification/K06-ACCEPTANCE.md`), and later
> decisions (ADR-079/ADR-081 addenda, ADR-082) amend some limits. Statements about branches, worktrees and the state
> "on `<commit>`" describe that date.

**Date:** 6 November 2026 · **Base:** the head of step 04 (F16/F17) [not yet built]
**Branch/worktree:** `fix/k06-f19-f15-icr-quarantine` in `/home/simon/projects/nieuw-os-k06-f19`
**Spec:** `K06-SMP.md` §2 (INV-5), §3.3, §4.2 (level 60 ICR), §6.2a, §7.1 step 6; ADR-071, -072;
audit findings F19 and F15.

## 0. Established facts (on `7ffdacfa`)
- `kernel/src/m2/smp.rs:555` `send_ipi(apic_id, low)` writes ICR high (0x310) and ICR low (0x300)
  without a lock. It assumes that only the BSP sends.
- Quarantine (`quarantine_all`) sends INIT to all APs and then continues on a single CPU.

## 1. Problem (why now)
- **F19:** from 04 and 08 onwards, APs send 0xe8 and 0xe0, both from thread context and from a
  handler. An interrupt between writing ICR high and ICR low yields a wrong
  destination. The consequence is a lost wake or ack, and with it a false quarantine or a
  kill timeout.
- **F15:** quarantine is currently safe only because APs hold nothing real. As soon as they do work,
  a deaf AP can hold `KERNEL`, `ASPACE` or `RUNQ`. After INIT that lock stays held forever, and
  the BSP blocks at the next acquire. The context of the thread that was running on that AP is gone, so
  "all threads to the BSP" (spec §6.2a) is not possible for that thread.

## 2. Invariants (normative)
- **IQ-1 ICR atomic per CPU.** The high/low pair plus the wait on the delivery status take place within
  `ICR_LOCK[this_cpu]` (level 60, irqsave). *Falsified if* lockdep sees a `send_ipi` without that lock.
- **IQ-2 Every lock knows its owner.** In `native-smp` and test builds, `IrqLock` writes the CPU index of the
  owner on acquire and clears it on release (one `AtomicU8`; no change
  to the ticket logic).
- **IQ-3 Policy for a dead CPU.** Only the BSP carries out quarantine. Before INIT the BSP checks
  whether the deaf CPU holds a lock (the owner fields of `KERNEL`, all `ASPACE`, `FRAMES`,
  `RUNQ`, `SHOOTDOWN`):
  - if it holds **nothing** → INIT, the READY threads from its queue go to the BSP, and the thread that
    was RUNNING is treated as crashed (the process is terminated via the K05 fault route, no migration);
  - if it **does** hold something → `SMP:QUARANTINE:LOCK_HELD:<niveau>:CPU=<n>` and a
    **system stop** (fail-closed). "Breaking" a lock is not permitted: the state beneath it is unknown.
- **IQ-4 An AP never initiates quarantine.** An AP that exceeds a deadline reports this to the
  BSP (a flag plus 0xe8) and waits. [to be verified: whether an AP in 08 can start a shootdown
  itself; if so, the BSP must take over the round.]

## 3. Interface
- `smp.rs`: `ICR_LOCK` as a field in `CpuLocal`; `send_ipi` takes that lock.
- `lock.rs`: `IrqLock::owner() -> Option<u8>` (debug and SMP builds).
- `smp.rs`: `quarantine_cpu(slot)` with the IQ-3 policy; `quarantine_all` becomes a loop over it.
- Kernel features `test-smp-quarantine-lock-held` and `test-smp-icr-race`.

## 4. Native fixture and negative tests
- Positive: AP1 and AP2 each send the other 10 000 0xe8 IPIs, while their own timer (0xd0) also
  sends IPIs from the handler. Requirement: received = sent per pair, no timeout. Marker
  `SMP:ICR:PASS`.
- `test-smp-icr-race` (control profile): without `ICR_LOCK`, with a deliberate 0xd0 between high
  and low. The oracle must see a misaddressed or lost IPI
  (`SMP:ICR:MISROUTED` or `SMP:ICR:LOST`).
- `test-smp-quarantine-lock-held` (4 vCPUs): AP2 acquires `KERNEL` and becomes deaf. Expected:
  `SMP:QUARANTINE:LOCK_HELD:10:CPU=2` and a stop with exit 0x10, no hang.
- Existing deaf-AP profiles (`ap-ipi-deaf`, `ap-shootdown-deaf`, `ap-timer-dead`) continue to follow the old
  path, because that AP holds nothing.

## 5. Write files
`kernel/src/m2/smp.rs`, `kernel/src/m2/lock.rs`, `kernel/Cargo.toml`, `tools/build.py`,
`tools/run_smp.py`, `tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`,
`docs/DECISIONS.md` (new ADR: quarantine policy; addendum to ADR-071/072), `docs/PROJECT-STATE.md`,
progress files.

## 6. Acceptance
- `tools/gate.sh` exit 0; fmt; clippy; `test_smp_serial.py`.
- Native: `native-smp` (all cases), the two new negative profiles and all existing
  deaf-AP profiles unchanged.
- K06 remains open (41/100).

## 7. Out of scope
Recovery of a lock after a dead owner, x2APIC (ICR in a single MSR write), broadcast IPIs.
