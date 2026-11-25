# H02-T5 feasibility: native refusal of out-of-profile USB devices — 25 November 2026

Scope: a QEMU feasibility probe before any kernel change. No source file was
modified; `probe.py` wraps `run_usb_hid.controlled_guest` at launch time and
replaces the port-1 `usb-kbd` with one candidate device. The mouse on port 2 and
the rest of the `native-usb-hid-probe` profile are unchanged.

| Field | Value |
|---|---|
| Source | main `ad55e019` plus ledger/doc commit `b36d26ff` (no kernel or tool change) |
| Profile | `python3 -B tools/build.py --kernel-feature native-usb-hid-probe` (exit 0) |
| QEMU | local 10.2.1; machine default q35 and `NOS_QEMU_MACHINE=pc-q35-8.2` |
| Controller | `qemu-xhci,p2=2,p3=0` (two USB2 root ports, no USB3 ports) |
| Command | `python3 -B docs/verification/h02-usb-refusal-feasibility-20261125/probe.py "$PWD" <device>` |
| Control | unmodified `reports` case: runner exit 0 |

## Observations

| Device on port 1 | First guest refusal | q35 | pc-q35-8.2 |
|---|---|---|---|
| `usb-hub` | `USB-DIAG:REJECT:0e` (`Session::new`, configuration not a boot interface) | yes | yes |
| `usb-tablet,usb_version=1` | `USB-DIAG:REJECT:0e` (same code as the hub) | yes | yes |
| `usb-kbd,usb_version=2` | `USB-DIAG:REJECT:08` (PORTSC speed is not low/full) | yes | yes |
| `usb-storage` (1 MiB disposable image) | `USB-DIAG:REJECT:08` (attaches at high speed) | yes | yes |

## Conclusions

1. Every candidate is refused before `SET_CONFIGURATION`, but each refusal ends the
   whole controller run: `usb::execute` returns `Err`, `xhci.rs` halts the
   controller and then `result.expect("xHCI contained DMA gate")` panics
   (`M2:PANIC_LINE:0000000565`, observed for storage; the runner kills QEMU on the
   first `USB-DIAG:REJECT` line, so the panic line is not always captured). This
   is a deliberate fail-closed terminal path with bus mastering cleared, not a
   device-reachable crash, but the in-profile mouse on port 2 is never served.
2. Hub and tablet are indistinguishable: `map_err(|_| bad(14))` collapses every
   `boot_interface::Error` variant. H02-T5 needs reason-specific markers.
3. Code 8 fires after the port reset but before Enable Slot; code 14 fires after
   Address Device, with a slot and DCBAA entry live. Per-port refusal must clean
   up at the matching depth.
4. A class-level storage refusal is not reachable in this fixture: QEMU
   `usb-storage` attaches at high speed on a USB2 port, so it is refused on speed
   first. It is recorded as a speed refusal only.

## Not run

Hosted QEMU 8.2.2 binary (only the machine type was pinned locally); physical
devices; any kernel change. This record is not H02 acceptance.
