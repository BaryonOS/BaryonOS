# K06 — remaining steps after F8 (L0 specifications)

> Historical L0 specification, written on 6 November 2026 and ported to `main` on 13 November 2026 from PR #232
> (head `dcd80d1`). English translation; the Dutch original is archived in `docs/history/k06-branch-prs-2026-11-06/k06-steps/`. The step has
> since been implemented; K06 was accepted on 13 November 2026 (`docs/verification/K06-ACCEPTANCE.md`), and later
> decisions (ADR-079/ADR-081 addenda, ADR-082) amend some limits. Statements about branches, worktrees and the state
> "on `<commit>`" describe that date.

**Date:** 6 November 2026 (Europe/Amsterdam) · **Status:** design; none of this has been built or tested.
**Base:** `7ffdacfa` (`fix/k06-f8-swapgs`, PR #231), the top of the stacked K06 chain
#227 (5b) → #228 (5c) → #229 (5d-1) → #230 (F1/F2) → #231 (F8). `main` contains only 5a (#225).
**Read:** `docs/implementation/K06-SMP.md`, `docs/progress.json`, `docs/PROGRESS.md`,
`docs/verification/K07-ACCEPTANCE.md`, `docs/PROJECT-STATE.md` (handoff 5b), ADR-074 and ADR-083,
the local task specifications 5b/5c/5d/F8 and the audit report `/tmp/k06_audit_report.md` (local, not in Git).

## Why there is no "K07 after K06"

K01–K07 are **acceptance criteria** in the points register (`docs/progress.json`), not consecutive
milestones. **K07 (Resource management under pressure) has already been achieved** on the single-CPU QEMU profile
([evidence](../../verification/K07-ACCEPTANCE.md)). The boundary of that evidence states literally: *"no
SMP evidence"*. K06 is the last open criterion of track K (13/15). What comes "after K06" is therefore:

1. the remaining K06 steps below, and
2. within 5f, the renewed proof of the K07 quotas under SMP (spec §9.1 already requires this).

After acceptance of K06 (+2 points), track K is complete (15/15). The next work then falls in other
tracks (S09, H02–H06 and so on). That lies outside this directory.

## Order (audit report §4, supplemented)

| # | File | Step | Covers | Blocks |
|---|---|---|---|---|
| 1 | [01-5b-2-kernel-lock-scratch.md](01-5b-2-kernel-lock-scratch.md) | 5b-2 | spec §4.2/§5, F20, ADR-075 | everything with M3 on SMP |
| 2 | [02-F3-F4-lockdep-context.md](02-F3-F4-lockdep-context.md) | F3/F4 | handler context, lock per instance | 04, 07, 08 |
| 3 | [03-F5-F7-stuck-serial.md](03-F5-F7-stuck-serial.md) | F5/F7 | bounded spin in release, SERIAL lock | reliable oracle for 09 |
| 4 | [04-F16-F17-block-wake-kill.md](04-F16-F17-block-wake-kill.md) | F16/F17 | blocking, waking and killing without lost wakeup | 08 |
| 5 | [05-F19-F15-icr-quarantine.md](05-F19-F15-icr-quarantine.md) | F19/F15 | ICR lock, quarantine policy with held locks | 08 |
| 6 | [06-5c-2-bsp-timer-idle.md](06-5c-2-bsp-timer-idle.md) | 5c-2 | BSP timer, idle, BSP window (F6, F9, F11) | 07, 08 |
| 7 | [07-5e-process-shootdown.md](07-5e-process-shootdown.md) | 5e | process-wide shootdown on `active_cpus` (F18) | 08 |
| 8 | [08-5d-2-ring3-on-aps.md](08-5d-2-ring3-on-aps.md) | 5d-2 | M3 processes on APs | 09 |
| 9 | [09-5f-acceptance.md](09-5f-acceptance.md) | 5f | migration, stress, §9 matrix, K07 under SMP | K06 → accepted |

Steps 2, 3 and 5 barely touch each other's files (`crates/smp-lock` and `lock.rs` versus `smp.rs`
and `serial.rs`). They can run in parallel in separate worktrees, but stacked on 01. No two
writers may work on `kernel/src/m2/smp.rs` at the same time.

## Fixed rules for every step (AGENTS.md)

- One small PR per step, as a draft, with auto-merge. Do not wait for CI; a red check is fixed
  before new work on the same branch.
- Order: first the invariant, then the interface, then the implementation, then the negative test.
- The single-CPU regression (`milestone-m2`, M3, K05, K07, `native-rtc`) remains green, unchanged.
- ADR numbers: 075 is reserved for 5b-2, 083 is F8. After that, take the **next free
  number at the start**: a parallel session is working in the same repository.
- After every step: `PROJECT-STATE.md` (handoff), `tools/progress.py --write` and `--check`. The score
  remains 41/100 until step 9 is natively green.
- "Not run" is not a green result. QEMU is not certain to be present locally (WSL). Check
  this with `command -v qemu-system-x86_64`; if it is missing, `native-smp.yml` on the PR applies.
