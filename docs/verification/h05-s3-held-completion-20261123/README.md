# H05 review R1/R3 — no broker device access after the S3 wake (23 November 2026)

**Decision:** ADR-H05-S3-SYSTEM-20261123, item 3 (devices are lost after a
wake). **Profiles:** `test-system-s3`, `test-system-s3-reinit` and the new
`test-system-s3-held-completion = ["test-system-s3"]` (opt-in test profiles,
no published ABI). **Status:** verified in QEMU, not accepted; H05 stays open.
**Source:** commit `8343d2fa`, source fingerprint v3 `4f071159…` (equal to
`power-system-gate.json` `source_sha256`).

## Defect (cloud review R1 of #409)

After a validated wake only the syscall paths refused (-14). `broker_tick`
(volume `advance`, keyring `advance`) and owner teardown (volume `reset`
through `detach`, keyring `quarantine_pending`) could still read reset
acknowledgements, re-initialise virtio-blk, poll or reset the lost device.
The trigger is a SUSPEND granted while the volume broker holds a request,
including a completed result its owner has not consumed.

## What changed

- **Latches (R1).** Volume `reset` and `advance` return before any device
  access when `system::resumed()` (each `#[cfg(feature = "test-system-s3")]`);
  `device_call` already refused -14. Keyring `call` (0x60), `advance` and
  `quarantine_pending` use `device_lost()` = `resumed() && !reinitialised()`,
  so only the validated re-init of `test-system-s3-reinit` (stage >= serving)
  lifts the keyring latch. A latched teardown reset counts in `suppressed`.
- **Gate (R1).** SUSPEND returns -8 after `storage_closed()` while
  `volume::sleep_quiescent()` is false: any held request (`policy.request()`),
  a non-Ready phase, a stored error or a reset in progress. Counted in
  `held_refusals`; every refusal precedes any I/O.
- **Counter (R3).** `device_access()` precedes every device call site of both
  brokers (submission, descriptor preparation and publication, poll, reset,
  reset-acknowledgement read, re-initialisation). The re-init stages count
  through `reinit_access()`, and broker accesses after `reinitialised()` are
  accounted as re-init I/O. `finish` requires, after CLEANUP, zero post-wake
  accesses outside the accounted path, `suppressed` = 1 in the held fixture
  and 0 otherwise, and accounted accesses only (and necessarily) in re-init.
- **Fixture.** `test-system-s3-held-completion`: the 0x41 consume of init's
  SHUTDOWN flush answers success but leaves the completed request in the
  Policy. Init (word 592 = 2) expects one -8 (step 80) before the grant. A
  labelled gate-defect injection (`held_check_skipped`, compiled into every
  `test-system-s3` build but constant-false outside this profile via `cfg!`)
  then skips the quiescence check on every later SUSPEND until the grant, so
  the sleep happens with the held
  request and only the latch keeps the volume service's exit reset off the
  device. `Policy::exit` leaves a completed request Ready, so the final S5
  gate re-check passes. Exact log `[-8,-6,-4,-1,-8,0,-14]`. The fixture and
  `test-system-s3-reinit` are mutually exclusive (`compile_error!`).
- **Host.** `tools/run_system.py` case `power-s3-held-completion` with exact
  oracle (`SUSPEND_REFUSALS=6`, `HELD_REFUSALS=1`, `DEVICE_IO_AFTER_WAKE=0`,
  `DEVICE_SUPPRESSED=1`, ordered); the other S3 cases require
  `HELD_REFUSALS=0`, `DEVICE_IO_AFTER_WAKE=0`, `DEVICE_SUPPRESSED=0`.
  `tests/test_system_harness.py` adds counterexamples and
  `PostWakeLatchStructure`, which classifies every broker device call site
  (reachable after the wake, re-init, boot/standalone) and requires the latch
  before and the counter at each reachable or re-init call.

## Results

Environment: `NOS_BUILD_DIR=<worktree>/build/r1`,
`CARGO_TARGET_DIR=<worktree>/target/r1`, `NOS_QEMU_MACHINE=pc-q35-8.2`,
QEMU 10.2.1 (Debian), single-thread TCG; run sequentially on `8343d2fa`.

| Command | Exit | Result |
|---|---|---|
| `python3 -B -u tools/test_power_system.py` | 0 | 12/12 stages: power (raw 0) + recover (33), power-flush-error (33) + recover (33), power-flush-timeout (33) + recover (33), power-s3 (raw 0, `LOST_ENTROPY=14`, `LOST_BLOCK=14`) + recover (33), power-s3-reinit (raw 0, `BARS=RESTORED`) + recover (33), **power-s3-held-completion (raw 0, 5.3 s) + recover (33)** |
| `python3 -B -m unittest tests.test_system_harness` | 0 | 46 tests (10 new) |
| `env -u GIT_EDITOR python3 -B -m unittest discover -s tools/tests` | 0 | 668 tests, 15 skipped |
| `python3 -B tools/build.py --kernel-feature native-acpi-s3` | 0 | — |
| `python3 -B tools/run_sleep.py --case cycles` | 0 | raw 33, `POWER:S3:PASS` |
| `python3 -B -m unittest discover -s tests -p 'test_system_*.py'` / `'test_ci_workflows.py'` | 0 | 80 / 28 tests |
| `cargo +1.90.0 fmt --all --check` (and `--manifest-path sdk/examples/system/Cargo.toml`) | 0 | — |
| `python3 -B tools/preflight.py --strict` | 0 | — |

power-s3-held-completion serial (abridged, in order):

```
POWER:S3:ENTER:CYCLE=1:SCI_EN=1
POWER:S3:RESUMED:CYCLE=1:ENTRY=PM32:PM1_STS=0x8101:WAK_STS=1:SCI_EN=RESTORED:HWSIG=SAME:FACS=DISARMED:CANARY=PASS:CR3=SAME
SYSTEM:SUSPEND_REFUSALS:00000000000000000006
SYSTEM:SUSPEND_HELD_REFUSALS:00000000000000000001
SYSTEM:SUSPEND_LOST_ENTROPY:00000000000000000014
SYSTEM:SUSPEND_LOST_BLOCK:00000000000000000014
SYSTEM:CLEANUP:PASS
SYSTEM:SUSPEND_DEVICE_IO_AFTER_WAKE:00000000000000000000
SYSTEM:SUSPEND_DEVICE_SUPPRESSED:00000000000000000001
SYSTEM:CASE:POWER_S3_HELD_COMPLETION:PASS
POWER:S5:WRITE
```

## Mutation check (uncommitted, reverted, tree clean afterwards)

Removing the volume `reset()` latch (5 lines) and running
`tools/run_system.py --case power-s3-held-completion` (separate
`NOS_BUILD_DIR=build/r1-mut`) failed natively: `passed=False`, raw 35,
`SYSTEM:SUSPEND_DEVICE_IO_AFTER_WAKE:…1`, `SYSTEM:SUSPEND_DEVICE_SUPPRESSED:…0`,
`SYSTEM:FAIL:SUSPEND_DEVICE_EVIDENCE`. On the host, deleting the counter in
volume `reset` or the latch in keyring `advance` fails `PostWakeLatchStructure`.

## Limits

- The fixture provokes only the volume `reset()` suppression. The keyring
  `quarantine_pending`/`advance` latches and the volume `advance` latch are
  checked by source shape only (the structural test); the counter
  (`DEVICE_IO_AFTER_WAKE=0` in every S3 run) shows only that no actual
  post-wake access happened in these runs, not that those latches work.
- The grant after the held -8 is a labelled gate-defect injection, active
  only in `test-system-s3-held-completion`. At `8343d2fa` its source comment
  ("Never compiled into another profile") was inaccurate; the review
  follow-up below makes it true (`#[cfg]`-gated).
- **Ungated shared changes.** Init reads word 592 == 2 in every system kernel
  (only this profile sets 2). Keyring `device_lost()`, the counter calls and
  the restructured `quarantine_pending` and 0x60 guard compile in every
  keyring profile (false or no-op without S3); the volume counter calls are
  no-ops outside S3. The `power-s3` and `power-s3-reinit` oracles now require
  three new lines (`HELD_REFUSALS`, `DEVICE_IO_AFTER_WAKE`,
  `DEVICE_SUPPRESSED`).
- The structural test is a source-shape check (regex and brace matching), not
  a proof; the counter counts call sites, not individual MMIO cycles.
- `test-system-s3-reinit` is checked as "no unaccounted access"; the number of
  accounted accesses depends on ticks and is not pinned.
- QEMU `pc-q35-8.2`/OVMF/TCG only; no physical hardware.
- **Not run:** the other system profiles that embed init (only the five
  power-gate profiles, `native-keyring` and `native-acpi-s3` were rebuilt),
  CI's QEMU 8.2.2, kernel clippy and hosted CI.

## Review follow-up (commits `a4f3c040`, `c51ad0b7`)

### What changed

- **A, evidence order.** `check_power` now also requires, for every S3 case,
  `SUSPEND_LOST_BLOCK` (the last wake line, after the re-init tail in
  `power-s3-reinit`) < `SYSTEM:CLEANUP:PASS` < the device evidence < the case
  marker, the kernel's real order. The host transcript helper emits that
  order (wake evidence and counters, `BROKERS`, teardown, `CLEANUP`, device
  evidence); counterexamples move the device lines above `CLEANUP` or after
  the case marker, and the wake line below `CLEANUP`.
- **B, latch shape.** `PostWakeLatchStructure` derives its device-call list
  from the `pub fn`/`pub unsafe fn` entry points of `drivers/virtio-rng` and
  `drivers/virtio-blk`, minus a named pure set (`validate`,
  `negotiate_rng_features`, `validate_used`, `classify_bars`, `pending`), so
  `kick`, `submit`, `discover` and `discover_function` are now covered. It
  requires the guarded statement itself: keyring `advance` latch `return;`,
  the 0x60 latch `return Err(-14);` with every device call of `call` inside
  the 0x60 arm, `quarantine_pending` binding the device only through
  `.filter(|_| touch)` with `let touch = !device_lost();`, volume `reset`
  `device_access_suppressed(); return;`, volume `advance` `return;`, volume
  `device_call` `return Err(-14);`, `reinit_fail` binding the device only
  through `.filter(|_| validated)` with `validated` from
  `reinit_state().bars.is_some()`, and `p.bars = Some(` in stage 0 of
  `reinit_poll` before its first `begin_reset`. Matching ignores comments.
- **C.** `held_check_skipped` exists only under
  `test-system-s3-held-completion`; the other S3 profiles compile a
  constant-false variant. Its doc comment now states that it skips the check
  on every later SUSPEND until the grant.
- **D, counter liveness.** `io_at_wake` is kept per broker. `finish` prints
  `SUSPEND_DEVICE_IO_AT_WAKE_KEYRING` and `SUSPEND_DEVICE_IO_AT_WAKE_VOLUME`
  (before its `require`) and requires both > 0; the host oracle pins both as
  live (> 0, not exact: the counts depend on ticks).
- **E.** The volume `reset()` comment now says a published, uncompleted
  request fails the S5 gate through `publications != completions` (-16).
  This README: "skips once" corrected; the Limits bullet on the unprovoked
  latches corrected.

### Rerun

Environment: `NOS_BUILD_DIR=<worktree>/build/r1m`,
`CARGO_TARGET_DIR=<worktree>/target/r1m`, `NOS_QEMU_MACHINE=pc-q35-8.2`,
QEMU 10.2.1 (Debian), single-thread TCG; run sequentially on `c51ad0b7`.

| Command | Exit | Result |
|---|---|---|
| `python3 -B -u tools/test_power_system.py` | 0 | 12/12 stages: power (raw 0) + recover (33), power-flush-error (33) + recover (33), power-flush-timeout (33) + recover (33), power-s3 (0) + recover (33), power-s3-reinit (0, `BARS=RESTORED`) + recover (33), power-s3-held-completion (0) + recover (33) |
| `tools/build.py --kernel-feature native-acpi-s3`; `run_sleep.py --case cycles` | 0; 0 | raw 33, `POWER:S3:PASS` |
| `test-acpi-s3-stale-token`; `--case stale-token` | 0; 0 | raw 37, `WAKE:REJECTED:TOKEN:ARMED=1` |
| `test-acpi-s3-corrupt-facs`; `--case corrupt-facs` | 0; 0 | raw 39, `REJECTED:InvalidSignature` |
| `test-acpi-s3-unarmed-wake`; `--case unarmed-wake` | 0; 0 | raw 37, `WAKE:REJECTED:TOKEN:ARMED=0` |
| `test-acpi-s3-facs-end`; `--case facs-end` | 0; 0 | raw 39, `REJECTED:FacsStore:DISARMED` |
| `python3 -B -m unittest tests.test_system_harness` | 0 | 50 tests |
| `env -u GIT_EDITOR python3 -B -m unittest discover -s tools/tests` | 0 | 670 tests, 15 skipped |
| `env -u GIT_EDITOR python3 -B -m unittest discover -s tests -p 'test_ci_*.py'` | 0 | 66 tests |
| `python3 -B -m unittest discover -s tests -p 'test_system_*.py'` | 0 | 84 tests |
| `cargo +1.90.0 fmt --all -- --check` (and `--manifest-path sdk/examples/system/Cargo.toml`) | 0 | — |
| `python3 -B tools/preflight.py --strict` | 0 | — |

Device evidence (serial, after `SYSTEM:CLEANUP:PASS`, before the case marker):

| Case | IO_AT_WAKE_KEYRING | IO_AT_WAKE_VOLUME | IO_AFTER_WAKE | SUPPRESSED |
|---|---|---|---|---|
| power-s3 | 11 | 11392 | 0 | 0 |
| power-s3-reinit | 11 | 10563 | 0 | 0 |
| power-s3-held-completion | 11 | 10659 | 0 | 1 |

### Mutation results (uncommitted, restored; tree clean afterwards)

Each mutant against `c51ad0b7`, running
`python3 -B -m unittest tests.test_system_harness.PostWakeLatchStructure`
(exit 1 for every mutant):

| Mutant | Failing test |
|---|---|
| keyring `advance`: latch without `return;` | `test_each_latch_returns_before_any_device_statement` |
| `quarantine_pending`: `r.device.as_mut()` without `.filter(\|_\| touch)` | `test_each_latch_returns_before_any_device_statement` |
| volume `reset`: `return;` after `device_access_suppressed()` deleted | `test_each_latch_returns_before_any_device_statement` |
| volume `advance`: latch without `return;` | `test_each_latch_returns_before_any_device_statement` |
| volume `advance`: uncounted `d.kick()` before the latch | `test_reachable_calls_follow_the_latch_and_are_counted` |
| volume `advance`: uncounted `virtio_blk_driver::discover()` before the latch | `test_reachable_calls_follow_the_latch_and_are_counted` |
| `reinit_fail`: `.filter(\|_\| validated)` dropped | `test_each_latch_returns_before_any_device_statement` |

Counter liveness (D): removing the `fetch_add` from volume `device_access()`
fails the host oracle (`test_device_counters_must_be_live_at_the_wake`: a 0
or a missing `IO_AT_WAKE` line is refused in all three S3 cases) and natively:
`test-system-s3-held-completion` built and run with the mutant
(`NOS_BUILD_DIR=build/r1m-mut`) gave `passed=False`, raw 35,
`SYSTEM:SUSPEND_DEVICE_IO_AT_WAKE_VOLUME:…0`, `SYSTEM:FAIL:SUSPEND_DEVICE_EVIDENCE`.

### Source binding

`power-system-gate.json` and all five `sleep-*-report.json` record
`source_sha256` `75e5e75e51c8f3b906abb38a9ae0fde94f7b1292a81a6844c0699d87ff0e63d0`,
equal to `tools/common.py` `source_hash()` of the final commit (this README
and `tests/` lie outside the fingerprint).

### Remaining limits

- The keyring `quarantine_pending`/`advance` and volume `advance` latches
  remain unprovoked natively; they are checked by source shape only.
- The structural test is still a source-shape check, not a proof; the pure
  set is named by hand and must be reviewed when a driver gains an entry
  point.
- **Not run:** kernel clippy, CI's QEMU 8.2.2, hosted CI, the other system
  profiles that embed init.
