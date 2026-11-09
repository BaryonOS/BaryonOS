# H05 step — native ACPI S5 power-off by the independent kernel

**Branch:** `hw/h05-native-acpi-s5`, based on main `cb3d169c` (after #204).
**Date:** 5 November 2026. **Evidence level:** native QEMU/TCG
(`pc-q35-8.2`, QEMU 10.2.1, OVMF), not physical hardware.
**Score:** 41 → 41/100. H05 remains open because acceptance requires measurements
on supported hardware, sleep/wake, and thermal/battery functions.

## Implementation

Previously, the S5 parser in `drivers/acpi-power` had no caller, so the
independent OS could not power itself off. The opt-in `native-acpi-s5`
kernel profile now does so through project code:

1. `boot-protocol` discovery records the address of the sole `FACP` root
   entry (`Discovery.fadt`). A duplicate FADT zeros that address without
   affecting the MCFG, DMAR or MADT inventory.
2. During the same one-time discovery (BSP, IF=0), the kernel reads the
   FADT (≤ 4 KiB) and DSDT (≤ 64 KiB) through the existing
   `FirmwareReader`, which permits only UEFI ACPI reclaim/NVS memory.
3. `parse_s5` validates signatures, lengths, checksums, FADT-to-DSDT
   binding, PM1 registers, and the AML `_S5_` subset.
4. Port I/O occurs only after a valid plan: PM1a is read and reported;
   `SLP_TYP` with `SLP_EN` cleared is written to PM1a and then PM1b
   (if present), followed by setting `SLP_EN` in the same order.
   Invalid plans produce `UNAVAILABLE`, `UNREADABLE` or
   `REJECTED:<error>`, perform no port write, and take the normal debug exit.

There is no syscall or IPC route; a process cannot request power-off.

## Independent oracle

- QEMU exits with code 0 only on guest power-off in the selected runner
  configuration. The runner replaces the default `-no-reboot` with
  `-action reboot=reset` (`reboot_on_reset` in `tools/run.py`). Local
  verification established that `system_reset` with `-no-reboot` alone
  exits QEMU with code 0, whereas the override keeps the VM running.
  A reset or triple fault therefore leads to a second `BOOT:EBS_OK`
  and is rejected.
- The expected port `0x0604` follows from the fixed q35 ICH9 PMIO base
  `0x600 + 4`, independently of the parser. It also appears in the earlier
  QMP firmware capture at
  [`drivers/acpi-power/verification`](../../../drivers/acpi-power/verification/README.md).
- The serial output tail must match exactly. Additional output after
  `POWER:S5:WRITE`, or a `POWER:S5:WRITE` marker in the rejection
  profile, fails the run. The runner does not independently trace PM1 I/O.

## Native cases (7/7 passed, no timeout)

| Profile | Runs | Exit | Exact tail after `M1:PASS` |
|---|---:|---:|---|
| `native-acpi-s5` | 5 | 0 (power-off) | `POWER:S5:PM1A=0x0604:SLP_TYP=0:SCI_EN=1`, `POWER:S5:WRITE` |
| `test-acpi-s5-corrupt-dsdt` | 2 | 33 (debug exit) | `POWER:S5:REJECTED:InvalidChecksum` |

The negative profile flips one byte in the kernel's DSDT copy (the final
byte). Exit 33 rather than 0 proves that no effective S5 power-off occurred,
not that no PM1 write was attempted. Source inspection of the rejection
branch shows no port-I/O call; independent PM1 write tracing is still open.
Durations, hashes, commands and full serial output are retained in
[native-report.json](native-report.json); both reports are bound to source
fingerprint v3
`f60c0d61af7da029ef9e126afc7aec4913586f96a73eaa6c8b4f5c08099652e0`.

The public JSON report replaces 14 developer-specific absolute home-directory
prefixes in `guest.command` with `<LOCAL_HOME>`. This is a publication
redaction, not a re-executed command. Exit codes, serial output, timing
fields and the recorded source fingerprint were not changed.

## Commands and regression

```bash
python3 -B tools/build.py --kernel-feature native-acpi-s5
NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B tools/run_power.py --repeat 5
NOS_BUILD_DIR=build/corrupt python3 -B tools/build.py --kernel-feature test-acpi-s5-corrupt-dsdt
NOS_BUILD_DIR=build/corrupt NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B tools/run_power.py --repeat 2
```

CI runs these in `.github/workflows/native-power.yml`. Recorded host
results: `boot-protocol` 29 tests (including the new FADT/duplicate test);
`acpi-power` 22 + 2. The root host suite in `ci.yml` reported 413 passed
tests and Clippy `-D warnings` exit 0. `tools/tests` passed 209/209
(`test_power` added three tests); formatting passed. `milestone-m2`,
`milestone-m6c` and `native-smp` profiles built. Kernel Clippy reported
only pre-existing findings in `paging.rs` and `main.rs`, none in the new code.

## Review

An independent AI review (ECC rust-reviewer) reported no blocking issues.
Two low-priority points were addressed: the negative fixture now changes
the final byte, avoiding an indexing panic for a 36-byte DSDT; and the
PM1 order matches ACPICA: first `SLP_TYP` with `SLP_EN` cleared to PM1a
then PM1b, then setting `SLP_EN` in that order. The assumptions for
`0x0604` and `SCI_EN=1` remain limited to fixed q35/OVMF.

## Open limitations

- OVMF already had `SCI_EN=1`. Enabling ACPI mode through
  `SMI_CMD`/`ACPI_ENABLE` is not implemented; a write with
  `SCI_EN=0` is untested on hardware.
- AML parsing covers only the existing `_S5_` subset, not `_PTS` or
  `_TTS` execution. This is sufficient for QEMU, not general ACPI.
- Service- or user-initiated shutdown requires a non-delegable power-off
  capability and prior service flushing (S08/volume); neither is implemented.
- S3 sleep/wake, thermal and battery functions, and physical-hardware
  evidence are absent. H05 and H06 remain open.

**Integration evidence boundary:** Results above apply to the original
source checkpoint. CI and native tests at the final squash-merge revision
must pass separately.
