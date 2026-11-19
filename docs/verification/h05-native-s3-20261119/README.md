# H05 step — native ACPI S3 suspend/resume by the independent kernel

**Date:** 19 November 2026. **Evidence level:** native QEMU/TCG
(`pc-q35-8.2`, QEMU 8.2.2+ds-0ubuntu1.18, Ubuntu `OVMF_CODE_4M.fd`), not
physical hardware. **Profile:** `native-acpi-s3` (IF=0, one CPU, M2
bootstrap; no ring-3 process, device or service across sleep).
**Score:** 41 → 41/100. H05 stays open: its acceptance requires measured,
recoverable sleep/wake, shutdown and thermal/battery behaviour on supported
hardware.

Source binding: commit `c54b9e25b43e` (branch `ccr-38238c58-g8y1l6`, base
main `035b44e`), source fingerprint v3
`ce3e3145f56ba336d767cb9a64976f89b6c566c64badadc5c36104a1be5ced77` for all
three cases. The runs executed on the working tree that was then committed
unchanged as `c54b9e25b43e`, so the reports' `base_commit` names its parent
`6d23553`. The first evidence round on `9c707c8` (fingerprint `e76b8fbf…`)
passed identically before the review hardening. [native-report.json](native-report.json) holds the manifest and
artifact SHA-256, commands, QMP traces and events, and full serial output.
It replaces the absolute checkout prefix with `<CHECKOUT>`; nothing else was
changed.

## What was demonstrated

The independent kernel enters ACPI S3 three times in one boot and resumes
each time through its own wake path:

1. Discovery reads FADT, DSDT and the FADT-designated FACS through the
   firmware-map-checked reader, validates them with `parse_s3`, and proves
   that the whole FACS extent is UEFI ACPI NVS (type 10).
2. Before each sleep the kernel clears `WAK_STS`, fills a 256 KiB canary of
   Kernel-owned frames, arms a fresh nonzero generation, writes the arm
   token into its own wake page below 1 MiB, stores the FACS waking vectors
   (`WakePlan`) and reads the FACS back, saves CR0/CR4/EFER/PAT, flushes
   caches and writes `SLP_TYP`/`SLP_EN` (ACPICA order).
3. QEMU reports `suspended`; the harness alone wakes the guest with QMP
   `system_wakeup`. OVMF runs its S3 resume path and enters the kernel's
   32-bit entry (`ENTRY=PM32`), which loads the kernel GDT, CR3, PAE, LME
   and NXE, switches to long mode and accepts only the armed token.
4. After the wake the kernel restores control state, reloads GDT/TSS/IDT
   (clearing the TSS busy bit), re-initialises COM1, disarms the FACS and
   reads it back, checks `WAK_STS`, the FACS hardware signature, the canary,
   CR3 and the full page-table alias audit, and restores `SCI_EN`, which
   OVMF leaves clear.

## Cases

| Case | Kernel feature | Oracle | Result |
|---|---|---|---|
| `cycles` | `native-acpi-s3` | 3× `query-status`=`suspended` with a 0.5 s serial-silent dwell, 3× harness `system_wakeup`, QMP events `SUSPEND,WAKEUP`×3, one EBS handoff, exact serial tail, raw exit 33 | pass, raw exit 33, 4.0 s, 3 sleep/wake |
| `corrupt-facs` | `test-acpi-s3-corrupt-facs` | `POWER:S3:REJECTED:InvalidSignature`, no QMP `SUSPEND`, raw exit 33 | pass, raw exit 33, 2.2 s, 0 sleep/wake |
| `stale-token` | `test-acpi-s3-stale-token` | one real sleep and wake, then `POWER:S3:WAKE:REJECTED:TOKEN:ARMED=1:PAGE=0x5a5a0001`, no `RESUMED`, raw exit 33 | pass, raw exit 33, 2.9 s, 1 sleep/wake |

Each resumed cycle reports
`ENTRY=PM32:PM1_STS=0x8101:WAK_STS=1:SCI_EN=RESTORED:HWSIG=SAME:FACS=DISARMED:CANARY=PASS:CR3=SAME`.
`PM1_STS=0x8101` is `WAK_STS | PWRBTN_STS | TMR_STS`: QEMU reports a
`system_wakeup` as a power-button wake. The harness requires `WAK_STS` and
does not pin the other bits.

Independent oracles (not derived from the kernel's own report): QMP
`query-status`, the QMP `SUSPEND`/`WAKEUP` event order, the wake request
issued only by the harness, serial silence during the dwell, the single
`BOOT:EBS_OK` (no reboot) and `-no-reboot` (a firmware reset during resume
would end QEMU with exit 0, rejected).

## Measured firmware findings (recorded in ADR-H05-S3-WAKE-20261119)

- **QEMU builds an ACPI 1.0b FACS** (version byte 0, length 64, all other
  bytes zero), observed in guest memory. `origin/main` (`035b44e`) rejected
  the real FADT/DSDT/FACS with `Err(UnsupportedFacsFields)`; the parser now
  accepts versions 0 and 1 with exact reserved-field checks. The ignored
  external-fixture test in `drivers/acpi-power` passed against the captured
  bytes (`NOS_ACPI_FACS`).
- **The real-mode waking vector cannot resume under QEMU TCG.** With only
  the ACPI 1.0b vector armed, edk2's `AsmTransferControl` far-returns into a
  16-bit code segment at `0xf743ad5`; QEMU TCG truncates EIP to 16 bits, the
  firmware executes OS memory from `0x3ad5` and stops with `#BP` at
  `0x93e1` (OVMF exception dump, CS `0x28`, CR0 still `PG|PE`). No OS code
  had run. edk2 reads `X_Firmware_Waking_Vector` for every FACS version, so
  the fixed profile also arms a 32-bit entry at offset 24 under the explicit
  `Edk2Version0Extended` policy and restores zero on disarm.
- **OVMF does not restore `SCI_EN`** after the wake (`SCI_EN=0` was observed
  on cycles 2 and 3 before the restore was added). The kernel restores it
  when it was set before sleep, as Linux does.

## Commands

```bash
cargo +1.100.0 test --locked --manifest-path drivers/acpi-power/Cargo.toml
cargo +1.100.0 clippy --locked --manifest-path drivers/acpi-power/Cargo.toml --all-targets -- -D warnings
cargo +1.100.0 fmt --all -- --check
cargo +1.100.0 build --locked --release --lib --target x86_64-unknown-none --manifest-path drivers/acpi-power/Cargo.toml
python3 -B -m unittest discover -s tools/tests -p test_sleep.py -v
NOS_BUILD_DIR=build/sleep-cycles python3 -B tools/build.py --kernel-feature native-acpi-s3
NOS_BUILD_DIR=build/sleep-cycles python3 -B tools/run_sleep.py --case cycles
NOS_BUILD_DIR=build/sleep-corrupt-facs python3 -B tools/build.py --kernel-feature test-acpi-s3-corrupt-facs
NOS_BUILD_DIR=build/sleep-corrupt-facs python3 -B tools/run_sleep.py --case corrupt-facs
NOS_BUILD_DIR=build/sleep-stale-token python3 -B tools/build.py --kernel-feature test-acpi-s3-stale-token
NOS_BUILD_DIR=build/sleep-stale-token python3 -B tools/run_sleep.py --case stale-token
```

Regression on the same tree: `native-acpi-s5` (`run_power.py --repeat 5`),
`test-acpi-s5-corrupt-dsdt` (`--repeat 2`), the standard smoke and the
`milestone-m2` and `milestone-m3` smokes (the shared firmware reader was
refactored). CI runs the S3 cases in `.github/workflows/native-power.yml`.

## Review

An independent AI review (two lenses: security/isolation and low-level
correctness, each finding adversarially verified) reported three low-severity
findings, all refuted as not reachable in this profile. They were applied as
hardening anyway: both entries now run `cld`, the 32-bit stage loads CR4 as
PAE-only instead of OR-ing into the firmware value, the 64-bit stage sets
RFLAGS to 2 on its own wake stack before Rust runs, and an arm failure or a
missing transition now disarms the FACS and reports `:DISARMED`, or fails
closed when the disarm itself fails. Hosted CI also flagged two defects in the
first push, both fixed: the S3 resume duplicated `install`'s GS selector load
and GS-base resets (`tools/tests/test_gs_discipline.py`); both now share one
`arch::load_tables`. The ledger listed a protected workflow in `write_paths`
(moved to `lead_paths`). An AI review is not an independent human audit.

## Not demonstrated / open

- Physical firmware and hardware; any wake source other than QMP
  `system_wakeup` (RTC alarm, power button, devices).
- The real-mode entry: armed, but not exercised natively (the firmware stub
  fails under TCG first). It reuses the natively verified SMP SIPI pattern.
- AML `_PTS`/`_WAK`/`_SST` (no AML interpreter; QEMU q35 profile only).
- Devices, ring-3 processes and services across sleep, interrupt
  controllers/timers after wake, SMP, and the capability-gated user path.
- Independent PM1 I/O tracing; the PM1 writes are inferred from QEMU's
  `suspended` state and events.
- Thermal and battery management (QEMU q35 exposes neither).
