# H05-T6 S3 hardening: local evidence (23 November 2026)

**Scope.** Four review items on PR #409 (head 02ef433f), implemented on branch
`claude/h05-hardening-20261123` from base `c208abe4`, code commit `b00f9b21`.
Local QEMU 10.2.1 with `-M pc-q35-8.2`, OVMF, TCG. CI pins QEMU 8.2.2, which
this run did not use. No hosted CI, no physical hardware and no human audit.
This is a builder's local record, not acceptance of any criterion.

## Changes

1. **Guard (`kernel/src/main.rs`).** The review claim was wrong in effect:
   `test-system-s3` already could not combine with net, clock, NAS, random,
   display-input or window-session. The `native-system-power` guard caught
   all six, and so did the `fpu-context` exclusion for display and window
   profiles. A separate `compile_error!` now names them explicitly. A feature
   closure over `kernel/Cargo.toml` found no system profile that escapes.
   `network.rs` now refuses 0x70–0x72 with -14 after a wake. The refusal sits
   after the shape checks and before authority, plans, copies, ring or device
   access, and it updates no `denied` counter. It is defence in depth only:
   the profile combination cannot be built.
2. **Wake order (`kernel/src/m2/sleep.rs`).** `s3_wake` now compares the
   token before it reads `SAVED` or restores any control register. `cycle`
   now writes `SAVED` before it arms a generation.
3. **Exit code.** A rejected wake exits with isa-debug value `0x12` (raw 37)
   in every S3 profile. Before, it used `0x10` (raw 33, the pass code) in
   `native-acpi-s3` and `0x11` (raw 35, the generic failure code) in
   `test-system-s3`. No other kernel or loader path writes `0x12`.
   `run_sleep.py` now requires raw 37 for `stale-token`. `cycles` and
   `corrupt-facs` still require raw 33. `corrupt-facs` is a plan rejection
   with no wake, and it returns to the M2 exit in `m2.rs`.
4. **FACS extent (`kernel/src/m2/paging.rs`).** `write_firmware` refuses any
   store outside the FACS extent. The refusal returns `Permission` before any
   mapping. `resolve()` records the extent once, from the `S3State::Ready`
   plan, which is discovery's ACPI NVS type-10 proof. The pure check is
   `nieuw_os_acpi_power::store_within_extent`, covered by host tests.

## Commands (raw exit codes)

Logs are in the ignored `build/h05t6/logs/`.

| Command | Exit |
|---|---|
| `tools/build.py --kernel-feature native-acpi-s3` + `tools/run_sleep.py --case cycles` | 0 / 0 (QEMU raw 33) |
| `... test-acpi-s3-stale-token` + `--case stale-token` | 0 / 0 (QEMU raw 37) |
| `... test-acpi-s3-corrupt-facs` + `--case corrupt-facs` | 0 / 0 (QEMU raw 33) |
| `python3 -B -u tools/test_power_system.py` (8 stages; power-s3 raw 0) | 0 |
| `run_net_link.py --case normal / nostatus / storm` | 0 / 0 / 0 |
| `run_network_service.py` normal, client-exit, revoke, helper, device-reset, exhaustion | all 0 |
| unittest `test_sleep.py` (18), `tests/test_system_harness.py`, `tools.tests.test_net_link` | 0 |
| `cargo +1.90.0 test --locked` and `clippy --all-targets -D warnings` (drivers/acpi-power) | 0 / 0 |
| `cargo +1.90.0 fmt --all -- --check` | 0 |
| `python3 -B tools/preflight.py --strict` | 0 |
| `cargo +1.100.0 check -p os-kernel --features test-system-s3,<each of 6>` | 101, new guard message present in all 6 |

The baseline at `c208abe4` passed all three S3 cases with raw 33.

## Not exercised

- An unarmed wake with a zeroed `SAVED`. The stale-token fixture has a valid
  `SAVED`, so it passes with or without item 2.
- A store outside the FACS extent at run time. Only the host tests cover it.
- The network refusal at run time. It was type-checked once, with both
  guards removed and dummy ELF variables; that edit was not committed.
- A rejected wake in `test-system-s3`.

## Open follow-ups

The ADR-H05-S3-SYSTEM-20261123 sentence "A rejected wake token exits 0x11 in
this profile" is now stale. It should read 0x12 (raw 37).

## Independent review

A fresh AI reviewer read `c208abe4..a99b56ed` and approved with minors. It built nothing.
This is not a human audit.

The lead fixed one finding in this branch: `write_firmware` now loads the extent length
(Acquire) before the start, so it can never pair a published length with an older start.
That pairing is latent today, since only one BSP runs with IF=0. After the fix,
`native-acpi-s3` cycles gives raw 33, stale-token raw 37 and corrupt-facs raw 33.

The lead amends the ADR (0x12) when merging. Open follow-ups, all of which fail closed:
- Every `resolve()` → `None` plan rejection in `native-acpi-s3` still ends with exit 0x10
  (raw 33). This covers `corrupt-facs`, `FacsExtent`, `NotNvs`, `Pm1bUnsupported` and
  `FacsStore:DISARMED`. Each one is still refused by `run_sleep.py`'s exact-tail check.
- No test fixture covers an unarmed wake with a poisoned `SAVED`, or a store at
  `facs+length`.
- The 32-bit wake stage still loads CR3 from the low wake page before `ARMED` is checked.

The network refusal (item 1) is defence in depth. No gate compiles it, because the guard
forbids `native-system-net` together with `test-system-s3`.
