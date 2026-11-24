# Specialist H — current handoff

Compact resume pointer for the H track (H01 guard, H02–H06). Merge state is
reported in the PR comment `HANDOFF H <cycle_id>`, not here.

| Field | Value |
|---|---|
| cycle_id | `h06-t6-20261124` |
| Task | H06-T6 removable-media HW-INV image (raw GPT/FAT32, QEMU-booted) |
| Branch | `claude/h06-t6-hwinv-image-20261124` (base main `e5ab3693`) |
| PR | see the `HANDOFF H h06-t6-20261124` comment |
| Tested source | `f99b4c58` (after merge with main `68596f82`; source fingerprint `2b72b62e…`), clean checkout; later commits add docs, evidence, ledger and progress only |
| Evidence | [h06-hwinv-image-20261124](../../verification/h06-hwinv-image-20261124/README.md): image SHA-256 `a4ca34eb…` (64 MiB); `image`, `plain` and `iommu` cases pass; sgdisk, fsck.fat and mtools agree; an image without `kernel.elf` stops at `M1:FAIL:KERNEL_OPEN` |
| NOT RUN | physical or USB boot; image-hash reproduction (an independent clean clone gave `d716f697…`: same kernel, different loader, until B04 lands); native `SECUREBOOT=0/1`; local `actionlint` |
| Score | 43/100 unchanged; H06 stays open |

## Blockers

- Owner decisions OD-HW-TARGETS, -BOOT, -CAPTURE, -SIGNERS, -INV-ROUTE (see
  `docs/drivers/hardware-matrix.json`) before any physical HW-INV run; the
  GPT/FAT32 image layout is an assumption until OD-HW-BOOT.
- B04 loader reproducibility (PR #493) before an image hash can be reproduced from
  an independent clean checkout.
- H03-T3b audio broker waits for owner decisions B1/B2/B4.

## Lessons

1. `tools/run.py` requires `NOS_BUILD_DIR` inside the checkout; `run_hw_inventory.py` needs a short `--run-root` for the QMP socket path.
2. A kernel profile that runs after `M2:PASS` must keep the non-feature `serial::exit(0x10)` as the cfg'd tail expression, or the default M2 smoke changes.
3. `tools/build.py` accepts comma-joined kernel features, so HW-INV fixtures are separate Cargo features listed in `HWINV_FEATURES`, each paired with the `hw-inventory` loader.
4. A raw QEMU exit of 35 from an HW-INV fixture means a `0xF4` write (`0x11`); a halted guest ends only through QMP `quit` (raw 0).
5. Locally, five `test_roadmap_runner.Session` tests also fail on unmodified main (environment); the ledger validity test is the relevant one.
6. `fsck.fat` drops a boot-sector volume label that has no matching root-directory label entry; the image uses `NO NAME`. `fsck.fat -v` prints "32 bit entries", not "FAT32".

## Next task

Ledger H06-T6 → `verified` in the next H PR. Then the next eligible H task from
`docs/roadmap-pipeline/ledger.json`; physical H06 work stays blocked on the owner
decisions above. Candidates without owner input: a serial-timeout fixture, or
recording the image hash reproduction once B04 (#493) is merged.
