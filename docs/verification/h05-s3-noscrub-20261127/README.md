# H05-T5d follow-up: S3 noscrub red control (2026-11-27)

**Scope.** Build plumbing and one labelled red control for the KL2 SUSPEND
gate of H05-T5d-T (#597, landed as `8b00a9c0`). **Base:** `main` `8ee0d501`.
**Status:** local native evidence only; hosted CI does not run the LOCK red
controls (the same holds for `power-lock-noscrub` and `power-lock-exit`).

## Defect

B's #590 (`9fe65ba9`) added the kernel profile `test-system-s3-lock-noscrub`
to `LOCK_SYSTEM_FEATURES`. `tools/build.py` computed `S3_SYSTEM_FEATURES`
before `POWER_SYSTEM_FEATURES += LOCK_SYSTEM_FEATURES`, so that profile was
built with the plain native init instead of the S3 TEST init (`--test-system-s3`,
which carries the LOCK caller). The red control therefore never reached init's
LOCK. Fix: compute `S3_SYSTEM_FEATURES` after the LOCK features are added.

## Invariant under test

KL4 / F1 (ADR-KEYRING-LOCK-20261126): after LOCK, `LOCKED` latches only once
`m3_restore` has scrubbed the whole service stack and read it back as zero.
The `test-system-s3-lock-noscrub` kernel omits only the scrub request, so the
next kernel entry after init's LOCK must halt with `SYSTEM:FAIL:LOCK_STACK_RESIDUE`
before any SUSPEND; the KL2 gate is never reached.

## Results

| Run | Command | Raw exit | Outcome |
|---|---|---|---|
| `power-s3-noscrub/` | `NOS_BUILD_DIR=build/s3-noscrub python3 -B tools/build.py --kernel-feature test-system-s3-lock-noscrub` then `python3 -B tools/run_system.py --case power-s3-noscrub` | build 0, QEMU 35 | expected failure: one `POWER:S3:PLAN` bootstrap line, `SYSTEM:MANIFEST:PASS`, exactly one `SYSTEM:FAIL:LOCK_STACK_RESIDUE`, zero `SYSTEM:LOCK:LOCKED`, no `SYSTEM:SUSPEND`, no PM1 write, no `M1:PASS`; harness `passed=True` |
| `mutant-build-order/` | same, with `tools/build.py` from `main` `8ee0d501` (the defect) | build 0, QEMU 35 | killed: init stops at its step 81 (`SYSTEM:ERROR:STEP:…81`, the plain init without the LOCK caller) with `SYSTEM:FAIL:APP_OR_INIT_RESULT`, rejected as "stopped for another reason" |
| `power-s3-noscrub` reruns | same as the first row, after a forced kernel rebuild (see Limits) | build 0, QEMU 35 | 2/2 `passed=True` |
| S5 red controls | `--case power-lock-noscrub` and `--case power-lock-exit` with their kernel features | build 0, QEMU 35 | both `passed=True` under the tightened evaluator |
| local power gate | `NOS_BUILD_DIR=build/power-system-gate python3 -B -u tools/test_power_system.py` | 0 | `passed=True`, 26 stages (13 `POWER_FEATURE` cases, each followed by a cold recovery), source fingerprint equal to the red run |
| host | `python3 -B -m unittest discover -s tests -p 'test_system_*.py'` and the other `ci.yml#host-quality` groups, `tools/tests`, `compileall` | 0 | 114 system tests; the plumbing test and the evaluator test fail on unfixed `main` (checked before the fix) |

Machine: QEMU 10.2.1, `pc-q35-8.2`, OVMF, `-accel tcg,thread=single` (from
`result.json`). Source fingerprint of the red run:
`74a76386f42955acabe18b7ad08a6e0900adfa43a57d8f7fc01536117f4cd253`.

## Evaluator change

`evaluate_lock_red` admits, for `power-s3-noscrub` only, exactly one line that
fully matches `run_sleep.PLAN` (the S3 plan resolved at bootstrap) and no other
`POWER:S3:`, `KEYRING:S3:` or `VOLUME:S3:` line. All LOCK red controls now also
reject any `SYSTEM:SUSPEND` or `SYSTEM:S3:` line. The S5 red controls are
otherwise unchanged.

## Not run

- Hosted CI for the red control (no workflow runs `LOCK_RED` cases; a hosted
  job would be a CI-owned workflow change).
- Hardware, physical S3, and an independent human audit.

## Limits

- TEST profiles only; the unwrap secret is a public TEST constant and no
  key-state durability is claimed (O5 open).
- The red control shows that a missing scrub request halts before SUSPEND; it
  does not measure residue values on the stack.
- Local build pitfall (not a product defect): the kernel embeds the user ELFs
  with `include_bytes!`, and Cargo decides freshness by mtime. Rebuilding the
  same kernel feature with a different init variant in one `NOS_TARGET_DIR`
  (here: after the mutant) left the kernel fresh with the mutant's init,
  because the uplifted `system-init` hardlink keeps an older mtime; two runs
  then failed at init step 81. Touching the user ELFs forced the rebuild.
  Hosted CI builds from a clean target directory and is not affected.
