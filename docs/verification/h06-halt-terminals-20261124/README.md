# H06-T5 halt-based HW-INV terminal paths (QEMU) — 24 November 2026

Scope: QEMU/TCG evidence that every terminal path of the `native-hw-inventory`
kernel and `hw-inventory` loader halts with CLI/HLT instead of writing QEMU's
isa-debug-exit port `0xF4` (undefined on physical PCs). Not physical hardware,
not a signed hardware record and not H06 acceptance.

| Item | Value |
|---|---|
| Source commit | `0a9915bf` (branch `claude/h06-t5-halt-terminals-20261124`, base main `bf0d336a`); the runs were made on its uncommitted tree, whose source fingerprint equals the commit's |
| Source fingerprint (v3) | `d1143a08b87506ee…` (full value in the result files) |
| Emulator | local QEMU 10.2.1, machine `pc-q35-8.2`, TCG, one CPU, 256 MiB, OVMF without enrolled keys, `isa-debug-exit` at `0xf4` attached |
| Change | `serial::exit` in `kernel/src/serial.rs` (feature `native-hw-inventory`) and `boot/loader/src/serial.rs` (feature `hw-inventory`) only halts; protocol `HW-INV` v3 adds failure marker `M2:FAIL*` |

| Fixture | Build | Markers (in order) | Result |
|---|---|---|---|
| `panic` | `--kernel-feature test-hwinv-panic` | `BOOTINFO:OK`, `M2:PANIC_LINE:…`, `M1:FAIL:KERNEL_PANIC` | passed |
| `m2-fault` | `--kernel-feature test-hwinv-m2-fault` | `M2:FAULT:KERNEL_UD:PASS`, `M2:FAIL:KERNEL_EXCEPTION` | passed |
| `hwinv-fail` | `--kernel-feature test-hwinv-fail` | `M2:PASS`, `HWINV:BEGIN`, `HWINV:FAIL:FIXTURE` | passed |
| `missing-kernel` | production `native-hw-inventory`, `kernel.elf` removed from the run copy | loader `M1:FAIL:KERNEL_OPEN` | passed |
| `plain`, `iommu` | production `native-hw-inventory` | full HWINV block, QMP `query-pci` equal | passed (unchanged) |

Every fixture check passed: markers in order, no `HWINV:PASS`, the log rejected
by `tools/hw_inventory.py` as reporting a protocol failure marker, QEMU still
running one second after the marker, and raw exit 0 only from the QMP `quit`.

Negative control: with the `0xF4` write restored in `kernel/src/serial.rs`, the
`panic` fixture failed (`halted_without_exit_port`, `clean_exit_after_quit`; raw
QEMU exit 35 = `(0x11 << 1) | 1`). The source was restored afterwards.

Regression on the same tree: default profile and `milestone-m2` smoke both raw
exit 33; 84 `test_hw_*.py` host tests; `hw_evidence.py --check`, `hw_matrix.py
--check`; CI workflow tests (39) and the roadmap ledger validity test.

Commands: `tools/build.py --kernel-feature <feature>` then
`tools/run_hw_inventory.py --case <case> --run-root <short dir>`, each exit 0;
`tools/run.py --smoke [--milestone m2]` exit 0 (raw 33).

Not run: a serial-timeout fixture (QEMU's UART is always ready; the path calls
the same `exit`); `actionlint` (not installed locally; hosted CI); any physical
PC, removable-media image or signed record. The loader's `handoff.S` tail still
writes `0xF4` but is reached only if the kernel entry returns, which its `-> !`
type excludes.
