# B01-LOCK L1: native QEMU evidence (26 November 2026)

Issue #564; contract `docs/implementation/B01-KEYRING-LOCK.md` and
ADR-KEYRING-LOCK-20261126 (both on PR #563, not on `main`). TEST profiles on
QEMU only; no real device was used.

- Source commit: `82c5ad03` on `claude/b01-lock-l1-20261126`, base `9fff93f7`
  (`origin/main`). Every run below used that source (`source_sha256`
  `07b1c455c016…` in each `result.json`). The commit that adds this directory
  changes documentation only.
- Each subdirectory holds the unchanged `serial.txt` and `result.json` of one run.

## Commands and raw exit status

| Command | Exit | Result |
|---|---|---|
| `python3 -B tools/build.py` | 0 | default kernel |
| `python3 -B tools/run.py --smoke` | 0 | QEMU raw 33, `M1:PASS` (`smoke-default.log`) |
| `python3 -B tools/build.py --kernel-feature native-smp-m3` | 0 | the disabled `m3_restore` block assembles |
| `python3 -B tools/build.py --kernel-feature native-system-lock` | 0 | LOCK without the fixture |
| `build.py --kernel-feature native-system` + `run_system.py --case normal` | 0 / 0 | passed, raw 33 |
| `build.py --kernel-feature native-system-power` + `run_system.py --case power` | 0 / 0 | passed, raw 0 (S5) |
| `build.py --kernel-feature test-system-s3` + `run_system.py --case power-s3` | 0 / 0 | passed, raw 0; SUSPEND unchanged |
| `build.py --kernel-feature test-system-lock` + `run_system.py --case power-lock` | 0 / 0 | passed, raw 0 (S5) |
| `build.py --kernel-feature test-system-lock-noscrub` + `run_system.py --case power-lock-noscrub` | 0 / 0 | expected failure `LOCK_CANARY_RESIDUE`, raw 35 |
| `build.py --kernel-feature test-system-lock-exit` + `run_system.py --case power-lock-exit` | 0 / 0 | expected failure `UNEXPECTED_EXIT`, raw 35 |

## Acceptance cases

A labelled kernel fixture drives the real 0x92 handler (`system_lock::lock`) on
init's behalf from `poll()`. It starts after the app is done, while init retries
its S5 grant. That grant is held (-8, not logged) until the fixture is done.

| Case | Observed in `power-lock` |
|---|---|
| A9 authority | -1 (reserved argument), -6 (image handle), -4 (forged generation); instance live |
| A2 entropy pending | entropy request on the instance's handle, LOCK -8; the broker drains, no -14 latch |
| A3 reply queued | a KEY1 admin reply still queued (`Engine::queued_from` = 1), LOCK -8 |
| A4 LOCK | LOCK 0: slot 1 dead, `cleanup` owner assert held, 65 recorded frames read back zero, entropy objects 1 → 0, one `SYSTEM:LOCK:LOCKED` |
| A10 residue | 3 canary copies on the service stack at LOCK, 0 one event later; one verified 32 KiB scrub, residue 0 |
| A6 client after LOCK | init's KEY1 admin send and receive both fail; no partial output |
| irreversible | a second LOCK returns -14 |
| A5 own exit | `power-lock-exit`: the Key Service exits on its own; `UNEXPECTED_EXIT`, no LOCKED marker |
| A10 control | `power-lock-noscrub`: without the scrub request the canary survives, `LOCK_CANARY_RESIDUE` |

The exact refusal log is `[-1, -6, -4, -8, -8, -14]`. The admin ingress counts
are pinned at 6 seen and 4 denied; the canary request is the extra refusal.

## Limits

- No ring-3 program issues 0x92. The dispatch block and supervisor flag in
  `m3.rs` are built, but these cases do not exercise them.
- A6 uses init's admin ingress, because the app closes its key client before
  it finishes. No RANDOM or domain request is tried after LOCK; KL5 is partial.
- KL4 holds before the next ring-3 instruction (`m3_restore`), not when the
  handler returns. A path that never restores, such as `finish` into S5, runs
  no further ring-3 code.
- A10 scans the service stack only. By code reading, not a native test, the
  Engine resets a received queue slot (`Queued::EMPTY`) and a retired endpoint
  (`Endpoint::EMPTY`).
- LOCK with `native-smp-m3` is refused at compile time; the §6 quarantine
  rule is not implemented.
- Host Python suites: the 9 CI `unittest discover` patterns pass, except
  `tools/tests`. That suite shows 2 failures and 9 errors in
  `test_agent_pipeline`, `test_agent_isolation` and `test_roadmap_runner`;
  an untouched checkout at `9fff93f7` shows the identical set.
- Hosted CI does not run the power, S3 or LOCK system cases. This evidence is local.
