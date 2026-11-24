# Specialist H — current handoff

Compact resume pointer for the H track (H01 guard, H02–H06). Merge state is
reported in the PR comment `HANDOFF H <cycle_id>`, not here.

| Field | Value |
|---|---|
| cycle_id | `h06-t5-20261124` |
| Task | H06-T5 halt-based HW-INV terminal paths (QEMU fixtures) |
| Branch | `claude/h06-t5-halt-terminals-20261124` (base main `bf0d336a`) |
| PR | see the `HANDOFF H h06-t5-20261124` comment |
| Tested source | `0a9915bf` (source fingerprint `d1143a08…`); later commits add docs, evidence and progress only |
| Evidence | [h06-halt-terminals-20261124](../../verification/h06-halt-terminals-20261124/README.md): `panic`, `m2-fault`, `hwinv-fail` and `missing-kernel` fixtures halt with QEMU still running; negative control raw 35; `plain`/`iommu` and default/M2 smoke (raw 33) unchanged |
| NOT RUN | serial-timeout fixture; native `SECUREBOOT=0/1`; local `actionlint`; any physical PC, image or signed record |
| Score | 43/100 unchanged; H06 stays open |

## Blockers

- Owner decisions OD-HW-TARGETS, -BOOT, -CAPTURE, -SIGNERS, -INV-ROUTE (see
  `docs/drivers/hardware-matrix.json`) before any physical HW-INV run.
- No removable-media image: `tools/build.py` exports only a QEMU ESP directory.
- H03-T3b audio broker waits for owner decisions B1/B2/B4.

## Lessons

1. `tools/run.py` requires `NOS_BUILD_DIR` inside the checkout; `run_hw_inventory.py` needs a short `--run-root` for the QMP socket path.
2. A kernel profile that runs after `M2:PASS` must keep the non-feature `serial::exit(0x10)` as the cfg'd tail expression, or the default M2 smoke changes.
3. `tools/build.py` accepts comma-joined kernel features, so HW-INV fixtures are separate Cargo features listed in `HWINV_FEATURES`, each paired with the `hw-inventory` loader.
4. A raw QEMU exit of 35 from an HW-INV fixture means a `0xF4` write (`0x11`); a halted guest ends only through QMP `quit` (raw 0).
5. Locally, five `test_roadmap_runner.Session` tests also fail on unmodified main (environment); the ledger validity test is the relevant one.

## Next task

H06-T6: a host-built removable-media HW-INV image (raw GPT/FAT disk image with a
recorded SHA-256, built from a clean checkout, QEMU-booted as a disk instead of
the virtual FAT directory) so protocol `HW-INV` can bind an image hash. Booting it
on a physical PC still needs the owner decisions above.
