# Specialist H — current handoff

Compact resume pointer for the H track (H01 guard, H02–H06). Merge state is
reported in the PR comment `HANDOFF H <cycle_id>`, not here.

| Field | Value |
|---|---|
| cycle_id | `h02-t5-20261125` |
| Location | local H session (took over from the cloud H session via #503; cloud HANDOFF on #507, comment 7151275971) |
| Task | H02-T5 native refusal fixtures for out-of-profile USB devices (hub, tablet without boot interface, storage, high-speed); starts with a QEMU feasibility probe |
| Branch | `claude/h02-t5-usb-refusal-20261125` (base main `ad55e019`) |
| PR | see the `HANDOFF H h02-t5-20261125` comment |
| Previous | #507 (H06-T8) merged as `78dc7b18`; ledger H06-T8 is `verified` in this branch |
| Score | 43/100 unchanged; H02 and H06 stay open |

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
9. `evaluate_fixture` markers that end in `:` are value markers (prefix plus non-empty value); every other fixture marker must be the whole line.

## Next task

H02-T5: probe in QEMU (local 10.2.1 and `pc-q35-8.2`) which `USB-DIAG:REJECT:`
code each out-of-profile device reaches in `native-usb-hid-probe`, then add one
exclusive refusal profile with per-reason markers, cleanup at the matching depth
and one mutant per check. After that, H05 virtio-blk re-initialisation after an
S3 wake (needs a security review). H04 in-flight TX at a link-down edge touches
`kernel/src/m2/m3/network.rs` and needs agreement with workstream N first.
Physical H02–H06 work stays blocked on the owner decisions above; image-hash
reproduction once B04 (#493) is merged.
