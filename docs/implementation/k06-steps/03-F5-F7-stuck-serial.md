# L0 specification — K06 F5/F7: diagnosis of a stuck lock and a serial oracle without interleaving

> Historical L0 specification, written on 6 November 2026 and ported to `main` on 13 November 2026 from PR #232
> (head `dcd80d1`). English translation; the Dutch original is archived in `docs/history/k06-branch-prs-2026-11-06/k06-steps/`. The step has
> since been implemented; K06 was accepted on 13 November 2026 (`docs/verification/K06-ACCEPTANCE.md`), and later
> decisions (ADR-079/ADR-081 addenda, ADR-082) amend some limits. Statements about branches, worktrees and the state
> "on `<commit>`" describe that date.

**Date:** 6 November 2026 · **Base:** the head of step 01 (5b-2) [not yet built]; independent of 02
**Branch/worktree:** `fix/k06-f5-f7-diagnostics` in `/home/simon/projects/nieuw-os-k06-f5f7`
**Spec:** `K06-SMP.md` §4.1 (bounded spin), §4.2 (level 70 SERIAL), §4.3 (NMI/#DF with `try_lock`);
audit findings F5, F7 and F12 (deadlines part).

## 1. Problem (why now)
- **F5:** the bounded spin with `SMP:LOCK:STUCK` sits behind `cfg!(debug_assertions)`, but every
  native gate builds with `--release`. A deadlock in the gate therefore becomes a hang until the host timeout,
  without diagnosis. Moreover, the limit of 2^24 is not derived from the shootdown deadline.
- **F7:** `kernel/src/serial.rs` has no lock. An AP and the BSP can write markers at the same time.
  The host parser can then miss a terminal marker or read one line made up of two half lines. From
  step 08 onwards, multiple CPUs genuinely write at the same time.

## 2. Invariants (normative)
- **DG-1 Bounded in the gate.** In every `native-smp` and `test-smp-*` image the spin is bounded,
  regardless of `debug_assertions`. The limit is a time, not an iteration count:
  `LOCK_STUCK_MS = 4 × SHOOTDOWN_DEADLINE_MS` (400 ms), measured with the PIT clock. Exceeding it →
  `SMP:LOCK:STUCK:<niveau>:CPU=<n>` and stop.
  *shortcut: under TCG, PIT polling tracks the host clock; replace this with a calibrated TSC as soon as 06 measures the ratio.*
- **DG-2 One line is indivisible.** Every marker is written in its entirety under `SERIAL`
  (level 70, irqsave). Terminal paths (NMI, #DF, panic) use `try_lock`; if that does not succeed within
  a short bounded attempt, they write anyway, with the prefix `!` (so that the oracle recognises a
  possibly interleaved line).
- **DG-3 No change on one CPU.** On one CPU there is never a waiter; the serial output is
  byte-identical.

## 3. Interface
- `lock.rs`: the bounded variant is selected with `cfg(feature = "native-smp")` or a test feature,
  not with `debug_assertions`. Release builds of other profiles remain unbounded (there is no
  waiter there).
- `serial.rs`: `static SERIAL: IrqLock<()>` at level SERIAL; `write_line(&[u8])` takes the lock around the
  whole line; `write_terminal(&[u8])` with `try_lock`.
- `tools/tests/test_smp_serial.py`: a line with `!` may only be a terminal marker.

## 4. Native fixture and negative tests
- Positive: every CPU writes 200 lines `SMP:SER:<cpu>:<seq>`, interleaved with the others. The host checks
  that every line is whole and that the seq increases per CPU. Marker `SMP:SERIAL:PASS`.
- `test-smp-lock-stuck` (2 vCPUs): the BSP holds a test lock and never releases it; the AP
  waits. Expected: `SMP:LOCK:STUCK:10:CPU=1` within 1 s and exit 0x10, no host timeout.
- `test-smp-serial-unlocked` (control profile, 4 vCPUs): without the lock. The host oracle must find an
  interleaved line in at least one of ten runs [to be verified: whether multi-thread TCG shows this
  reliably; if not, only a host test of the parser with an interleaved input].

## 5. Write files
`kernel/src/m2/lock.rs`, `kernel/src/serial.rs`, `kernel/src/m2/smp.rs` (fixture),
`kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`, `tools/tests/test_smp_serial.py`,
`.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (addendum to ADR-074), `docs/PROJECT-STATE.md`,
progress files.

## 6. Acceptance
- `tools/gate.sh` exit 0; fmt; clippy; `test_smp_serial.py` with the new cases.
- Native: `native-smp` (all cases), `test-smp-lock-stuck`, and the regression of all
  single-CPU profiles with a byte comparison of the serial output.
- K06 remains open (41/100).

## 7. Out of scope
A lock-free line buffer per CPU (alternative to DG-2; choose it only if the lock causes measurable
contention), calibration of the LAPIC timer (06), the repetition harness (09).
