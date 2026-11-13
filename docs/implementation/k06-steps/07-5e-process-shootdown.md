# L0 specification — K06 step 5e: process-wide TLB shootdown on `active_cpus`

> Historical L0 specification, written on 6 November 2026 and ported to `main` on 13 November 2026 from PR #232
> (head `dcd80d1`). English translation; the Dutch original is archived in `docs/history/k06-branch-prs-2026-11-06/k06-steps/`. The step has
> since been implemented; K06 was accepted on 13 November 2026 (`docs/verification/K06-ACCEPTANCE.md`), and later
> decisions (ADR-079/ADR-081 addenda, ADR-082) amend some limits. Statements about branches, worktrees and the state
> "on `<commit>`" describe that date.

**Date:** 6 November 2026 · **Base:** the head of step 06 (5c-2) [not yet built]
**Branch/worktree:** `feat/k06-5e-proc-shootdown` in `/home/simon/projects/nieuw-os-k06-5e`
**Spec:** `K06-SMP.md` §2 (INV-1, INV-4, INV-5), §5 point 6, §7.1, §7.2, §9.3, §9.9–9.10; ADR-072, -075;
audit finding F18.

## 0. Established facts (on `7ffdacfa`, from the audit; check them at the start)
- `ACTIVE[space]` is maintained in `sched_switch` (5d-1), but `shootdown_round` targets **all**
  APs.
- The 0xe0 handler flushes 2 fixed VAs without a pid or CR3 check. A round contains at most 2 VAs.
- The two test spaces are byte-identical PML4 copies of the kernel root. They prove the
  switching mechanism, **not isolation**. A PML4 copy falls out of step with later changes in the
  kernel half at PML4 level.

## 1. Problem (why now)
As soon as M3 processes run on APs (08), every unmap, rights reduction or process exit must reach all CPUs
that may cache translations of that process, and only those CPUs. If that does not happen, a CPU
reads or writes, via a stale TLB entry, a frame that already has a new owner
(INV-1). Data then leaks between processes.

## 2. Invariants (normative)
- **PS-1 Target set = `active_cpus`.** For process P: `targets = P.active_cpus.load(SeqCst) & !self`, after
  modifying the PTE and a `fence(SeqCst)` (spec §7.1 step 3). The order in §6.3 steps 5a–c
  guarantees that no CPU outside `targets` holds a stale translation.
- **PS-2 Published round.** `{pid, start, pages, gen}` with Release, under `SHOOTDOWN`. Range
  bounded: ≤ 32 pages per `invlpg`, otherwise a CR3 reload. The handler compares its current
  CR3 with that of P; if they are equal it flushes, otherwise it does not. It **always** acks with the generation.
- **PS-3 Release after all acks.** Frames go to `FRAMES.free` only after all acks. A missing
  ack → the frames remain in quarantine and the policy from 05 (IQ-3) follows.
- **PS-4 Kernel half.** A change in the shared kernel half (except the CPU's own scratch slot from
  01) goes to **all online CPUs**, including the BSP (possible since 06). The kernel-half PML4 entries
  are fully pre-populated at boot, so that a new PML4 entry, which copies would miss, is never needed later.
- **PS-5 Process exit** (spec §7.2): all threads `DEAD` (04) → the exiting CPU switches to the
  kernel CR3 → `active_cpus` must be 0 → shootdown (usually an empty target set, but the protocol
  always runs) → K05 cleanup → frames free. `check_invariants` gets two rules: *no free frame
  is still referenced by a PTE in a live space* and *`active_cpus` of a dead process is 0*.

## 3. Interface
- `smp.rs`: `shootdown_process(pid, range) -> Result<(), Quarantined>` and
  `shootdown_kernel(range)`; the old `shootdown_round` becomes an internal helper.
- `paging.rs`: unmap and rights reduction call `shootdown_process` under `ASPACE[pid]` (+`FRAMES`).
- Test spaces: two **real** user spaces with a different frame at the same VA (no
  PML4 copy), only in the fixture.
- Kernel features `test-smp-proc-shootdown-deaf` and `test-smp-proc-no-shootdown`.

## 4. Native fixture and negative tests (spec §9.2, §9.3, §9.9, §9.10)
- **Isolation:** space A on AP1 and space B on AP2 write and read the same VA with different
  patterns, 10 000×. Requirement: never the other's pattern. Marker `SMP:ISOLATION:PASS`.
- **Unmap race:** a thread in space A on AP1 reads page X in a loop. The BSP unmaps X, frees the
  frame and reuses it with a new pattern. Requirement: the reader gets a controlled #PF
  (fixture handler) and never sees the new pattern. Marker `SMP:UNMAP_RACE:PASS`.
- **Targeted set:** a round for A demonstrably reaches only the CPUs in `ACTIVE[A]` (counter per
  CPU in the handler). Marker folded into `SMP:UNMAP_RACE:PASS`.
- `test-smp-proc-no-shootdown` (control profile): without shootdown the reader reads the reused
  frame → `SMP:SHOOTDOWN:STALE_READ` (proves that the unmap race measures a real hazard, as in ADR-072).
- `test-smp-proc-shootdown-deaf`: AP1 ignores 0xe0 for space A → frames in quarantine and the
  IQ-3 policy. AP1 holds no lock, so INIT follows and the boot continues.

## 5. Write files
`kernel/src/m2/smp.rs`, `kernel/src/m2/paging.rs`, `kernel/src/m2/m3.rs` (only the exit and
cleanup route, if that already runs here), `kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`,
`tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (addendum to
ADR-072 and ADR-075 point 6), `docs/implementation/K06-SMP.md` (§7), `docs/PROJECT-STATE.md`,
progress files.

## 6. Acceptance
- `tools/gate.sh` exit 0; fmt; clippy; `test_smp_serial.py`; a host model for the
  `ACTIVE` superset rule with a fake CR3 (audit §5).
- Native: `native-smp` (all cases, 2/4/8 vCPUs, single- and multi-thread TCG), both negative
  profiles, the existing `ap-shootdown-deaf`/`shootdown-skipped-control`, regression `milestone-m2`.
- K06 remains open (41/100).

## 7. Out of scope
PCID/INVPCID, a lazy TLB (a CPU that "borrows" a space), large pages, IOMMU/DMA.
