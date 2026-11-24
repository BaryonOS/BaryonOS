# Specialist H — current handoff

Compact resume pointer for the H track (H01 guard, H02–H06). Merge state is
reported in the PR comment `HANDOFF H <cycle_id>`, not here.

| Field | Value |
|---|---|
| cycle_id | `h06-t7-20261124` |
| Task | H06-T7 serial-timeout HW-INV terminal fixture and HLT oracle (QEMU); ledger H06-T6 → `verified` |
| Branch | `claude/h06-t7-serial-timeout-20261124` (base main `4ab40610`) |
| PR | see the `HANDOFF H h06-t7-20261124` comment |
| Tested source | `6d923b55` (source fingerprint `16374a07…`), clean tree; later commits add docs, evidence, ledger and progress only |
| Evidence | [h06-serial-timeout-20261124](../../verification/h06-serial-timeout-20261124/README.md): serial-timeout fixture passes (`HLT=0` while polling, then `HLT=1`); every terminal fixture now requires `HLT=1`; three mutants fail (exit port raw 35, injection skipped, spin) |
| NOT RUN | physical boot; local `actionlint`; hosted run on the `pc-q35-8.2` pin until CI on the PR |
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
7. HMP `info cpus` no longer reports `(halted)` in current QEMU; `info registers` still prints `HLT=1`.
8. A bash gate helper that sets a global `name` clobbers the caller's variable: use `local` in step/mutant functions, or mutant runs silently target the wrong build directory.

## Next task

The next eligible H task from `docs/roadmap-pipeline/ledger.json`, starting with
ledger H06-T7 → `verified` once its PR merges. Physical H06 work stays blocked
on the owner decisions above. Candidate without owner input: record the image
hash reproduction once B04 (#493) is merged.
