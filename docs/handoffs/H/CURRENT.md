# Specialist H — current handoff

Compact resume pointer for the H track (H01 guard, H02–H06). Merge state is
reported in the PR comment `HANDOFF H <cycle_id>`, not here.

| Field | Value |
|---|---|
| cycle_id | `h06-t3-20261124` |
| Task | H06-T3 native read-only HW-INV inventory profile |
| Branch | `claude/h06-hw-inventory-20261124` (base main `1fc9b8ca`) |
| PR | [BaryonOS/BaryonOS#484](https://github.com/BaryonOS/BaryonOS/pull/484) (draft) |
| Tested source | `966cfb4404d32584e2e5a191aaebf985c51ed5d6` (native QEMU cases); later commits add docs, evidence and progress only |
| Evidence | [h06-hw-inventory-20261124](../../verification/h06-hw-inventory-20261124/README.md): `plain` and `iommu` QEMU cases pass against QMP `query-pci`; default and M2 smoke raw 33; host tests green |
| NOT RUN | native `SECUREBOOT=0/1`; native missing-extension fixture; local `actionlint`; any physical PC, image or signed record |
| Score | 43/100 unchanged; H06 stays open |

## Blockers

- Owner decisions OD-HW-TARGETS, -BOOT, -CAPTURE, -SIGNERS, -INV-ROUTE (see
  `docs/drivers/hardware-matrix.json`) before any physical HW-INV run.
- Panic and M1/M2 failure paths still exit through port `0xF4`.
- H03-T3b audio broker waits for owner decisions B1/B2/B4.

## Lessons

1. `tools/run.py` requires `NOS_BUILD_DIR` inside the checkout; `run_hw_inventory.py` needs a short `--run-root` for the QMP socket path.
2. A kernel profile that runs after `M2:PASS` must keep the non-feature `serial::exit(0x10)` as the cfg'd tail expression, or the default M2 smoke changes.
3. `tools/build.py` accepts comma-joined kernel features; single-feature profiles must reject them explicitly.

## Next task

H06-T5: halt-based (no `0xF4`) panic and M1/M2 failure terminals for the
`native-hw-inventory` profile, with a native fixture per failure marker. This is
the last software blocker of protocol `HW-INV` that needs no owner decision.
