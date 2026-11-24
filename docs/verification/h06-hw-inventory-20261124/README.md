# H06-T3 native HW-INV inventory (QEMU) — 24 November 2026

Scope: QEMU/TCG evidence for the read-only `native-hw-inventory` kernel and
`hw-inventory` loader (ADR-HWINV-20261124). Not physical hardware, not a signed
hardware record and not H06 acceptance.

| Item | Value |
|---|---|
| Source commit | `966cfb4404d32584e2e5a191aaebf985c51ed5d6` (branch `claude/h06-hw-inventory-20261124`, base main `1fc9b8ca`) |
| Source fingerprint (v3) | `f93da4dc144542eb…` (full value in the result files) |
| Emulator | local QEMU 10.2.1, machine `pc-q35-8.2`, TCG, one CPU, 256 MiB, OVMF without enrolled keys |
| Commands | `python3 -B tools/build.py --kernel-feature native-hw-inventory` (exit 0); `python3 -B tools/run_hw_inventory.py --case all --run-root <short dir>` (exit 0) |

| Case | Configuration | Result |
|---|---|---|
| `plain` | `qemu64,vendor=AuthenticAMD,-hypervisor`, no VT-d | passed; 5 PCI functions; `SECUREBOOT=ABSENT`; `DMAR=ABSENT:DRHD=0` |
| `iommu` | `qemu64,vendor=GenuineIntel`, `intel-iommu`, `pcie-root-port` 00:06.0, `qemu-xhci` on bus 1 | passed; 7 PCI functions including bridge 00:06.0 and xHCI 01:00.0; `DMAR=PRESENT:DRHD=1` |

Every check in each `*-result.json` passed: marker order, one `BOOT:EBS_OK`, no
failure marker, the kernel ACPI marker, QEMU still running one second after
`HWINV:PASS` (no exit-port write; the raw exit 0 comes from the QMP `quit`), CPUID
and DMAR equal to the configuration, `SECUREBOOT` not `1`, inventory bound to its
own log, PCI set equal to QMP `query-pci`, and rejection by `tools/hw_evidence.py`
as a virtual capture. `*-inventory.json` are the converted schema-2 inventories.

Regression checks on the same branch: default profile and `milestone-m2` smoke
both raw exit 33; `pci-core` 11 and `boot-protocol` 36 unit tests; 82
`test_hw_*.py` host tests; roadmap ledger and CI workflow tests.

Not run: `SECUREBOOT=0`/`1` natively (host-tested decode only); a native
missing-extension fixture; `actionlint` (not installed locally; hosted CI);
any physical PC, removable-media image or signed record. Panic and M1/M2
failure paths still exit through port `0xF4`.
