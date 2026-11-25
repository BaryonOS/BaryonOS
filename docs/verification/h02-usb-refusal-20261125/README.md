# H02-T5 native per-port USB refusal

The opt-in `native-usb-hid-refusal` profile passes in QEMU for four
out-of-profile devices on root port 1: a hub, a non-boot tablet, a high-speed
keyboard and USB storage. In each case the driver refuses the port-1 device,
keeps serving the boot mouse on port 2, and recovers port 1 with a
new-generation boot keyboard after an unplug. The `reports`, `readonly`,
`reattach` and `reattach-mouse` cases and the default smoke boot still pass.
H02 remains **open**. No score change is proposed.

The behaviour and markers are specified in
[NATIVE-USB-HID.md](../../implementation/NATIVE-USB-HID.md#per-port-refusal).
The feasibility probe that preceded it is in
[h02-usb-refusal-feasibility-20261125](../h02-usb-refusal-feasibility-20261125/README.md).

## Source identity

- Source commit: `1bb81100034ae40a0f39760978e519cb93a179ed` on
  `claude/h02-t5-usb-refusal-20261125` (PR #515).
- Executed source fingerprint (all eight USB cases):
  `67f95ddf1f2152e40a09b7968bbb9abc00d1d86d9f4eadc9f50f507fc391d011`.
  `docs/` is outside this fingerprint. `kernel/` and `tools/` had no
  uncommitted changes during the runs.
- Toolchain: rustc 1.100.0 (0ef1bc386 2026-11-09).
- QEMU: 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3), `NOS_QEMU_MACHINE=pc-q35-8.2`.
  The four refusal cases also passed with the default machine. Hosted CI
  (QEMU 8.2.2) runs them through `native-xhci.yml`; that result is recorded
  on the PR, not here.
- Isolated environment: `NOS_BUILD_DIR=build/h02t5/final/<profile>`,
  separate `CARGO_TARGET_DIR`; one QEMU guest at a time.
- Compact per-case reports: [native-report.json](native-report.json).

## Native results

| Case | Profile | Runner exit | QEMU raw exit |
|---|---|---|---|
| `refuse-hub` | `native-usb-hid-refusal` | 0 | 33 |
| `refuse-tablet` | `native-usb-hid-refusal` | 0 | 33 |
| `refuse-hs-kbd` | `native-usb-hid-refusal` | 0 | 33 |
| `refuse-storage` | `native-usb-hid-refusal` | 0 | 33 |
| `reports` | `native-usb-hid-probe` | 0 | 33 |
| `readonly` | `test-usb-hid-readonly-fault` | 0 | 33 |
| `reattach` | `native-usb-hid-reattach` | 0 | 33 |
| `reattach-mouse` | `native-usb-hid-mouse-reattach` | 0 | 33 |
| default smoke | default | 0 | 33 |

Refusal reasons observed: hub `HUB` and tablet `NOT_BOOT`, both after
`SLOT_DISABLED`; high-speed keyboard and storage `SPEED`, before Enable Slot.

## Mutation controls

Each mutant was built in a disposable `git archive` copy outside the
repository and run against its target case. Every mutant failed.

| Mutant | Case | Result |
|---|---|---|
| speed check accepts high speed | `refuse-hs-kbd` | fail (oracle) |
| hub check removed | `refuse-hub` | fail (oracle: NOT_BOOT instead of HUB) |
| refusal routed to terminal `bad(14)` | `refuse-tablet` | fail (raw 35) |
| Disable Slot skipped | `refuse-hub` | fail (Reset Device not refused) |
| DCBAA clear skipped | `refuse-hub` | fail (raw 35, code 62) |
| MFINDEX settle removed | `refuse-tablet` | fail (input lost, timeout) |

The DCBAA mutant first survived: the controller does not read DCBAA for a
disabled slot and the recovered keyboard rewrites the entry. The read-back
check (code 62) was added after that result.

## Host checks

| Command | Exit |
|---|---|
| `python3 -B -m unittest discover -s tools/tests -p 'test_usb_hid*.py'` | 0 (23 tests) |
| `python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py'` | 0 (57 tests) |
| `python3 -B -m unittest tools.tests.test_roadmap_runner -k ledger` | 0 |
| `cargo +1.100.0 fmt --all -- --check` | 0 |
| `python3 -m compileall -q` on the changed tools | 0 |

## Review

An AI security review (not an independent human audit) found no DMA or
authority defect. Its findings and their handling:

- Malformed device or configuration descriptors (codes 10–13) halted the
  controller and stopped port-2 service. Fixed: in this profile they are
  refused per port as `BINDING`. No QEMU fixture reaches this path.
- The recovered keyboard reused the refused device's output context without
  zeroing it. Fixed.
- Storage and the high-speed keyboard give the same transcript. Documented;
  the runner argument test now checks all four device definitions. The class
  of the attached device is not queried from QEMU.
- The refused generation-1 ticket is not replayed after recovery; the port is
  not disabled through PORTSC.PED; the mouse is not exercised after recovery.
  Documented as limits.

## Not run

`actionlint` (not installed locally; the hosted workflow lint applies);
hosted QEMU 8.2.2 (pending on the PR); physical USB devices, hubs and
hotplug; human acceptance.
