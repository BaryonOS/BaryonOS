# H06-T7 serial-timeout HW-INV terminal fixture (QEMU) — 24 November 2026

Scope: QEMU/TCG evidence that the serial-timeout terminal path of the
`native-hw-inventory` kernel halts with CLI/HLT, without writing QEMU's
isa-debug-exit port `0xF4`. It closes the gap that
[h06-halt-terminals-20261124](../h06-halt-terminals-20261124/README.md) left
open. This is not physical hardware, not a signed hardware record and not H06
acceptance.

| Item | Value |
|---|---|
| Source commit | `6d923b55c604cb44f4debf7885f5dca7c7fe903c` (branch `claude/h06-t7-serial-timeout-20261124`, base main `4ab40610`), clean tree; later commits change only `docs/` |
| Source fingerprint (v3) | `16374a07ab47c930…` (full value in the result files) |
| Emulator | local QEMU 10.2.1, machine `pc-q35-10.0` (local default), TCG, one CPU, 256 MiB, OVMF without enrolled keys, `isa-debug-exit` attached at `0xf4`; hosted CI uses the `pc-q35-8.2` pin |
| Injection | Feature `test-hwinv-serial-timeout` (implies `native-hw-inventory`). After printing `HWINV:FIXTURE:SERIAL_TIMEOUT_ARMED`, the kernel sets a static flag, and from then on `transmit_ready` in `kernel/src/serial.rs` reports the transmit holding register as busy. The real read of port `0x3fd` still takes place. The unmodified bounded wait in `write_raw` (1,000,000 polls) therefore runs out and calls `exit(0x11)`. |

## Fixtures

All fixtures ran from one gate on the source above. Each run directory holds
`serial.txt`, `qmp-trace.json` and `result.json`; the result files are
copied here.

| Fixture | Result | `cpu_halted` |
|---|---|---|
| `serial-timeout` (new) | passed, raw 0 after QMP `quit`, 2.3 s | `HLT=1` |
| `panic`, `m2-fault`, `hwinv-fail` | passed | `HLT=1` |
| `missing-kernel` (loader `M1:FAIL:KERNEL_OPEN`) | passed | `HLT=1` |
| `plain`, `iommu`, `image` | passed, unchanged | not applicable |

The `serial-timeout` fixture passed all of its checks:

- `M2:PASS`, `HWINV:BEGIN` and the armed marker appear in order.
- Nothing follows the armed marker.
- There is no `HWINV:PASS`.
- `tools/hw_inventory.py` rejects the truncated block ("log needs exactly one
  HWINV:BEGIN and one HWINV:PASS").
- QEMU was still running one second later.
- HMP `info registers` reports `HLT=1`.

The QMP trace of that run holds three `info registers` replies. The first, taken
while the bounded wait was still polling, reads `HLT=0`; the next two read
`HLT=1`.

From H06-T7 on, every terminal fixture also requires `HLT=1` (`cpu_halted`). The
loader `missing-kernel` path passes this check as well.

## Mutants

Each mutant was built and run on the same tree, and the source was restored
afterwards. Each one fails the fixture for the intended reason.

| Mutant | Failed checks | Raw QEMU exit |
|---|---|---|
| Exit-port write restored in `serial::exit` | `halted_without_exit_port`, `cpu_halted`, `clean_exit_after_quit` | 35 |
| Injection not applied (`if false && STUCK…`) | `no_hwinv_pass`, `protocol_failure_recognised`, `silent_after_injection` | 0 (it reached `HWINV:PASS`) |
| Timeout spins instead of halting | `halted_without_exit_port`, `cpu_halted`, `clean_exit_after_quit` | runner deadline (`HLT=1` never seen), process killed |

## Regression on the same tree

- The `plain`, `iommu`, `image` and `missing-kernel` cases pass.
- The default profile and `milestone-m2` smoke both ended with raw exit 33.
- The `test_hw_*.py` host tests passed; the gate log does not record the count.
- `hw_evidence.py --check` and `hw_matrix.py --check` passed.
- `tests/test_ci_workflows.py` passed.
- `cargo fmt --check` passed for the kernel.
- The roadmap ledger tests passed (150).

## Limits

- The busy UART is injected in software; no real stuck UART was observed.
- After arming, the log and HLT state are the same for any terminal path that
  tries to write and halts. The fixture therefore shows "halts silently once the
  UART stops accepting bytes". The `HLT=0`-then-`HLT=1` sequence shows only that
  the halt followed a polling phase. It does not show that the cause was
  `exit(0x11)` specifically (independent review, minor 1).
- Not run: hosted `native-hw-inventory` on the `pc-q35-8.2` pin (runs on the
  PR), any physical PC, and local `actionlint`.
