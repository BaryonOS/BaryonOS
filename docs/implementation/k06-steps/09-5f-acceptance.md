# L0 specification — K06 step 5f: migration, concurrent stress and K06 acceptance (with K07 under SMP)

> Historical L0 specification, written on 6 November 2026 and ported to `main` on 13 November 2026 from PR #232
> (head `dcd80d1`). English translation; the Dutch original is archived in `docs/history/k06-branch-prs-2026-11-06/k06-steps/`. The step has
> since been implemented; K06 was accepted on 13 November 2026 (`docs/verification/K06-ACCEPTANCE.md`), and later
> decisions (ADR-079/ADR-081 addenda, ADR-082) amend some limits. Statements about branches, worktrees and the state
> "on `<commit>`" describe that date.

**Date:** 6 November 2026 · **Base:** the head of step 08 (5d-2) [not yet built]
**Branch/worktree:** `feat/k06-5f-acceptance` in `/home/simon/projects/nieuw-os-k06-5f`
**Spec:** `K06-SMP.md` §6.2b, §7.3, §9 (in full), §11; `docs/PROGRESS.md` (acceptance rules);
`docs/verification/K07-ACCEPTANCE.md` (boundary "no SMP"); audit findings F10, F12 and F21.

## 1. Goal
Accept K06: *"SMP scheduling, synchronization, interrupts, and address-space switching work under
concurrent load on the supported profile"* (progress.json), on native QEMU q35/OVMF/TCG.
At the same time, the boundary of the K07 evidence is lifted: the K07 quotas remain bounded under SMP.

## 2. Invariants (normative, supplementary)
- **AC-1 Migration only when explicit** (spec §6.2): on quarantine (to the BSP) and in a
  test fixture. The `on_cpu` handover (F1) and the RUNQ release→acquire provide the happens-before to
  the FXRSTOR on the new CPU (§7.3).
- **AC-2 Bounded fairness** (F10): under the big kernel lock, the per-CPU maximum of
  `deferred` and of the wait time on `KERNEL` stays below a limit that is stated in the ADR in advance. Exceeding it
  → `SMP:FAIR:FAIL:CPU=<n>`. Ticket locks under host overcommit are recorded as an R02 risk in the ADR.
- **AC-3 A reliable gate** (F12): every deadline is a named constant with a rationale. The gate
  measures and logs the margin (measured / deadline). A case counts as green only with **50/50** repetitions.
- **AC-4 No 1-vCPU sham evidence:** a case with `CPUS=1` counts as regression, not as SMP evidence.
- **AC-5 Own stacks** (F21): the scheduler threads of the fixtures get their own stacks, or an
  assertion on the coupling with the step-4 stacks.

## 3. The §9 matrix (profiles 1/2/4/8 vCPUs, single- and multi-thread TCG)
**Positive:**
1. **Concurrent load, 30 s:** ≥ 2× as many CPU-bound ring-3 processes as CPUs, plus an
   IPC ping-pong between processes on different CPUs. Per CPU: user ticks > 0, ≥ 4 switches,
   no audit error.
2. **Isolation under SMP** (from 07, now with M3 processes).
3. **Unmap race** (from 07, now with M3 processes).
4. **Migration with canary:** T migrates 1→2→0 with unique GPR and x87/SSE values; after each step they are
   equal and `active_cpus` is correct.
5. **Kill on a remote CPU:** kill a process on CPU 3; the K05 cleanup balance is correct (free frames
   before = after) and `active_cpus == 0`.
6. **BSP interrupts:** Sys3 rises during the load; the idle BSP is demonstrably woken.
7. **K07 under SMP (new relative to spec §9):** the M4 quota test and the qsim pressure from
   `resource-pressure.yml`, but with the processes spread across 4 vCPUs. Requirement: the same
   quota/`EAGAIN`/`ResourceExhausted` outcomes, survivor progress and exact cleanup as on 1 CPU,
   and no kernel failure.

**Negative (correct marker and fail-closed):** `lockdep-order`, `held-across-switch`,
`proc-shootdown-deaf`, `proc-no-shootdown` (control), `double-run` (now `DOUBLE_ENQUEUE`),
`resched-deaf`, canary injection (deliberately corrupt a saved register; the canary must fire),
`spin-irq-off`, `lock-stuck`, `lock-in-handler`, `quarantine-lock-held`, `gs-user`, `gs-iret-fault`.

**Regression (green, unchanged):** `milestone-m2`, K05 6/6, K07 (`resource-pressure.yml`),
`native-rtc` 5/5, all existing `run_smp.py` cases, host tests, Clippy and fmt.

## 4. Repetition harness and soak
- `tools/run_smp.py --repeat 50` (or a small wrapper) per case; the report counts flakes. One flake
  = not green, and the cause is investigated. The deadline is not raised without an ADR.
- A soak profile of 10 min on 4 vCPUs (multi-thread TCG) with the load of §3.1 and §3.7.

## 5. Evidence and progress
- `docs/verification/k06-smp-sched/{README.md,native-report.json,SHA256SUMS}`: source commit, exact
  commands, exit codes, markers per case, repetition counts, margins and hashes. Follow the structure of
  the existing `docs/verification/k06-smp-*/` directories.
- `docs/verification/K06-ACCEPTANCE.md` in the form of `K07-ACCEPTANCE.md`: accepted property,
  checks performed (runs with ID) and **boundaries** (TCG does not model TLB timing or speculation;
  no physical hardware, x2APIC, IOAPIC to APs or power).
- `K07-ACCEPTANCE.md`: a short addendum stating that the quotas have also been proven under SMP (4 vCPUs), with
  a reference to the evidence. K07 remains `accepted`; the point total does not change.
- `docs/progress.json`: K06 → `accepted` (+2, **41 → 43**) only if the native gate on this
  branch head is demonstrably green (PROGRESS.md). Then `python3 tools/progress.py --write` and `--check`.

## 6. Write files
Test fixtures in `kernel/src/m2/smp.rs` and `kernel/src/m2/m3.rs`, `tools/run_smp.py`,
`tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`,
`docs/verification/k06-smp-sched/`, `docs/verification/K06-ACCEPTANCE.md`,
`docs/verification/K07-ACCEPTANCE.md` (addendum), `docs/DECISIONS.md` (fairness limit, deadlines),
`docs/implementation/K06-SMP.md` (status → implemented), `docs/PROJECT-STATE.md`,
`docs/progress.json`, `README.md`, `docs/PROGRESS.md`, `docs/assets/progress.svg`.

## 7. Acceptance commands (the runner does not build by itself)
```
export NOS_BUILD_DIR="$PWD/build/k06-native-smp-m3"
python3 -B tools/build.py --kernel-feature native-smp-m3
python3 -B -u tools/run_smp.py --profile native-smp-m3 --repeat 50
python3 tools/progress.py --write && python3 tools/progress.py --check
```
[to be verified: whether `--repeat` and `native-smp-m3` have these names after step 08; adjust the names to what was built there.]

## 8. Out of scope and residual risk
Work stealing and load balancing (R02), physical hardware (H06), human audit (B05). The big
kernel lock limits parallel syscall throughput: that is acceptable for K06 (correctness), but it
remains an R02 risk. An AI review is not an independent human audit.
