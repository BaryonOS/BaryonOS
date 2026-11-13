# L0 specification — K06 step 5b-2: KERNEL lock around the M3 entry and scratch slot per CPU (ADR-075)

> Historical L0 specification, written on 6 November 2026 and ported to `main` on 13 November 2026 from PR #232
> (head `dcd80d1`). English translation; the Dutch original is archived in `docs/history/k06-branch-prs-2026-11-06/k06-steps/`. The step has
> since been implemented; K06 was accepted on 13 November 2026 (`docs/verification/K06-ACCEPTANCE.md`), and later
> decisions (ADR-079/ADR-081 addenda, ADR-082) amend some limits. Statements about branches, worktrees and the state
> "on `<commit>`" describe that date.

**Date:** 6 November 2026 · **Base:** `7ffdacfa` (`fix/k06-f8-swapgs`, PR #231, top of the K06 stack)
**Branch/worktree:** `feat/k06-5b2-kernel-lock` in `/home/simon/projects/nieuw-os-k06-5b2`
**Spec:** `docs/implementation/K06-SMP.md` §2 (INV-2, INV-6), §4.2, §5 (revision of ADR-024), §8;
ADR-024, -053, -060, -074; audit finding F20.

## 0. Established facts (on `7ffdacfa`)
- `kernel/src/m2/paging.rs:14` has a single global `static mut SCRATCH_ENTRY`. The comment on line 809
  calls it a *"single-CPU scratch-entry address"*. It is used in the paths on lines 774–860
  (zeroing and copying frames via a temporary mapping).
- `kernel/src/m2/m3.rs:1564` `m3_event` is the only Rust entry point for M3 events. It accesses
  `static mut STATE` (`m3.rs:458`) under the SAFETY assumption *"No scheduler can run concurrently"*.
- `kernel/src/main.rs:29` excludes M3 from `native-smp` with `compile_error!`. That remains so in this step.
- [to be verified] Whether any paths outside `m3_event` still mutate `STATE`, the frame allocator or paging
  (IRQ0 stub `m3.rs:1159-1203` only counts raw counters; check the M4/M6 services).

## 1. Problem (why now)
Ring 3 on APs (step 08) means that two CPUs can handle a syscall or fault at the same time.
Without a single lock, they mutate `STATE`, the frame allocator and the mapping arena simultaneously (INV-2). Two
CPUs that zero or copy a frame via the same `SCRATCH_ENTRY` at the same time read or write
each other's frame: that is a data leak or corruption between processes. This step makes the kernel
SMP-safe **without** changing the single-CPU behaviour.

## 2. Invariants (normative)
- **K-1 One kernel section.** Every mutation of `STATE`, frame allocator, paging, IPC wait queues,
  quotas and supervisor takes place with `KERNEL` (level 10) held. `m3_event` acquires `KERNEL` as its
  first action after the entry stub and releases it only just before returning to the stub.
  *Falsified if* a debug check `kernel_held()` fails in a mutating function.
- **K-2 Scratch per CPU.** CPU *i* uses only VA `SCRATCH_BASE + i·4096` and its own PTE address.
  The address is reachable only via `this_cpu().scratch_*()`. There is no longer a global scratch entry.
  Invalidation remains local (`invlpg`). *Falsified if* a CPU touches another scratch VA
  (assert in the helper), or if the fixture in §4 reads a foreign pattern.
- **K-3 No change on one CPU.** On one CPU the lock is always acquired immediately (no waiter). The
  order of markers, quotas and memory usage of all existing profiles remains byte-identical.
- **K-4 Existing single-CPU SAFETY notes** (ADR-060 buffer, ADR-053 wait, `STATE`) are
  rewritten from "single-CPU/IF0" to "holds `KERNEL`". No SAFETY text still cites "single-CPU"
  as its justification.
- **K-5 Window on the BSP only in 5c-2.** In this step the BSP still spins with IF=0 (ADR-074 addendum
  5b). On one CPU there is never any spinning.

## 3. Interface (minimal)
- `lock.rs`: a single static `KERNEL: IrqLock<()>` at level KERNEL, and `fn kernel_held() -> bool`
  (lockdep bit of this CPU). A guard type without `Send` (already exists from 5a).
- `m3_event`: `let _k = KERNEL.lock_irqsave();` as the first line. [to be verified: whether the
  context switch takes place in `m3_event` itself; if so, the guard must be dropped before the switch (INV-7),
  see spec §6.3 step 2.]
- `paging.rs`: `SCRATCH_ENTRY` → `[u64; MAX_CPUS]`, filled in during setup for every CPU index.
  `scratch_slot()` returns (VA, PTE ptr) of `this_cpu().index`. The BSP is index 0 (GS anchor from 5a).
- ADR-075: the revision of ADR-024 according to spec §5 (points 1–7), plus the K-1 rule.

## 4. Native fixture (`native-smp`, after `SMP:SCHED:PASS`, before `SMP:PARKED`)
Every AP and the BSP simultaneously zero and fill a frame of **their own** via their own scratch slot with a
pattern `0xA5 ^ cpu_index`. They do this 1024 times, and after every fill they check the whole frame.
Then every AP acquires `KERNEL` 1000× and increments a shared counter (not atomic, only
under the lock). Requirement within 500 ms: all patterns match, counter = 1000 × (online+1), lockdep empty.
Marker `SMP:SCRATCH:PASS:CPUS=<n>`.

## 5. Negative test
`test-smp-shared-scratch` (4 vCPUs): all CPUs use scratch slot 0 (the old state). The
fixture must see a foreign pattern → `SMP:SCRATCH:CROSS:CPU=<n>` and exit 0x10. This proves that the
positive test measures a real hazard. The oracle rejects a PASS in this profile and the marker in a
normal profile.

## 6. Write files
`kernel/src/m2/lock.rs`, `kernel/src/m2/paging.rs`, `kernel/src/m2/m3.rs` (lock and SAFETY texts),
`kernel/src/m2/smp.rs` (fixture), `kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`,
`tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (ADR-075),
`docs/implementation/K06-SMP.md` (§8), `docs/PROJECT-STATE.md`, progress files.

## 7. Acceptance
- `tools/gate.sh` exit 0; `cargo fmt --check`; clippy `-D warnings` on `smp-lock`; `test_smp_serial.py`.
- Kernel builds `native-smp`, `test-smp-shared-scratch`, `milestone-m2` and all M3/M4/M6 profiles.
- **Single-CPU regression, mandatory and native:** `milestone-m2`, K05 (`tools/run_supervisor.py`, 6/6), K07
  (`resource-pressure.yml`: M4 quotas and qsim), `native-rtc` 5/5, `run_ipc_wait.py`, `run_fpu.py`,
  `run_memory.py`. Output and markers unchanged relative to `7ffdacfa`.
- Native `native-smp` (all cases) and `test-smp-shared-scratch` via `run_smp.py` or `native-smp.yml`.
- Progress: K06 remains open (41/100).

## 8. Out of scope
M3 in `native-smp` (remains excluded until 08), the window on the BSP (06), fine-grained locks per
subsystem (ADR-075 names the revision criterion), PCID.
