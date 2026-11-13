# L0 specification — K06 F3/F4: lockdep with handler context and lock instances

> Historical L0 specification, written on 6 November 2026 and ported to `main` on 13 November 2026 from PR #232
> (head `dcd80d1`). English translation; the Dutch original is archived in `docs/history/k06-branch-prs-2026-11-06/k06-steps/`. The step has
> since been implemented; K06 was accepted on 13 November 2026 (`docs/verification/K06-ACCEPTANCE.md`), and later
> decisions (ADR-079/ADR-081 addenda, ADR-082) amend some limits. Statements about branches, worktrees and the state
> "on `<commit>`" describe that date.

**Date:** 6 November 2026 · **Base:** the head of step 01 (5b-2) [not yet built]
**Branch/worktree:** `fix/k06-f3-f4-lockdep` in `/home/simon/projects/nieuw-os-k06-f3f4`
**Spec:** `K06-SMP.md` §4.2–4.4; ADR-074 (addenda 5b, F2); audit findings F3, F4 and F-T1.

## 1. Problem (why now)
- **F3:** the rule that handlers never acquire `KERNEL`/`ASPACE` exists only in comments. If a handler does
  acquire them on a CPU that holds nothing, lockdep succeeds. The IF window then executes `sti` **inside
  the handler**, with nested interrupts on the same stack as a consequence.
- **F4:** lockdep has one bit per level. An IPC copy between two address spaces (two `ASPACE`)
  or a migration (two `RUNQ`, spec §4.2 "ascending CPU index") currently yields a false
  `SMP:LOCKDEP:ORDER`. This makes the planned IPC route on SMP impossible.

## 2. Invariants (normative)
- **LD-1 Context.** Per CPU, `irq_depth` counts the nested IDT entries; every stub increments it after the
  `swapgs` prologue (ADR-083) and decrements it before the epilogue. `lock_irqsave` at a level
  ≤ `ASPACE` with `irq_depth > 0` → `SMP:LOCKDEP:IN_HANDLER:<niveau>` and stop.
  NMI/#DF/#MC do not use GS (GS-6) and are not counted; they acquire no locks.
- **LD-2 Window only outside handlers.** The `sti; pause; cli` window opens only when
  `irq_depth == 0` (addendum to S1/S3 from 5b).
- **LD-3 Instances.** Two locks of the same level may be held together only via `lock_pair(a, b)`. It
  acquires them in a fixed order: `ASPACE` by ascending pid, `RUNQ` by ascending CPU index. Lockdep
  keeps a counter per level (max 2) and the most recently acquired key. A second acquire at
  the same level outside `lock_pair`, or with a descending key → `SMP:LOCKDEP:ORDER`.
- **LD-4 LIFO remains** (F2): the release checks `level == highest()`, also for pairs (the
  second key first).

## 3. Interface
- `smp_lock::Lockdep`: `acquire(level, key)`, `release(level, key)`; one counter per level.
  `held_mask` remains available for the switch check.
- `lock.rs`: `IrqLock::lock_pair(&self, other: &Self) -> (Guard, Guard)` with the key taken from the lock
  (pid or CPU index, fixed at construction).
- `CpuLocal.irq_depth: u32`, written only by this CPU. The stubs get `inc`/`dec` on
  `gs:[irq_depth]` after the prologue.

## 4. Host tests (F-T1, cheap, before native)
- A property test: random sequences of acquire and release (with pairs) against a simple
  reference model. Lockdep and the model must agree at every step.
- A loom model of `TicketLock` (exclusion and FIFO) [to be verified: loom as a dev-dependency;
  check provenance and licence, and pin the version in accordance with AGENTS.md].
- Cases: a pair in the wrong order, a third lock at the same level, release of the
  first of a pair, `IN_HANDLER`.

## 5. Native fixture and negative tests
- Positive in `native-smp`: two APs take `lock_pair` on two test locks of level ASPACE, with
  opposite argument order, 1000×. No deadlock and no marker may occur. Marker
  `SMP:LOCKPAIR:PASS`.
- `test-smp-lock-in-handler` (2 vCPUs): the 0xe8 handler acquires `KERNEL` →
  `SMP:LOCKDEP:IN_HANDLER:10`, exit 0x10.
- `test-smp-lockpair-order` (2 vCPUs): two ASPACE locks acquired separately in descending order →
  `SMP:LOCKDEP:ORDER:20->20`, exit 0x10.

## 6. Write files
`crates/smp-lock/src/lib.rs` (+ dev-dependency if loom is chosen, with an exception note),
`kernel/src/m2/lock.rs`, `kernel/src/m2/smp.rs` (stubs and fixture), `kernel/src/m2/arch.rs` (stubs
outside `smp.rs`, if they exist), `kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`,
`tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (new ADR
or addendum to ADR-074), `docs/PROJECT-STATE.md`, progress files.

## 7. Acceptance
- `tools/gate.sh` exit 0; `cargo test -p smp-lock` (including the property and loom tests); clippy; fmt.
- Native: `native-smp` (all cases, including `SMP:LOCKPAIR:PASS`), the two negative profiles and
  regression `milestone-m2`.
- K06 remains open (41/100).

## 8. Out of scope
More than two locks of one level, a runtime owner per lock (05), a SERIAL lock (03).
