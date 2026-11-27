# Specialist H — current handoff

Compact resume pointer for the H track (H01 guard, H02–H06). Merge state is
reported in the PR comment `HANDOFF H <cycle_id>`, not here.

| Field | Value |
|---|---|
| cycle_id | `h05-t5c-verified-20261126` |
| Location | local H session; the latest `HANDOFF H` comment is on #515 |
| Task | H05-T5c virtio-blk re-initialisation after an S3 wake: slice 1 (kernel re-init, quarantine `-24`, O13 latch carryover) merged; slices 2–4 open |
| Branch | `claude/h05-t5c-verified-20261126` (ledger follow-up); slice 1 came from `claude/h05-t5c-blk-reinit-20261125` |
| PR | #556 merged as `945847d8` (26 Nov 2026); ledger H05-T5c is `verified` (hosted gates green on `0e9e1b91`) |
| Previous | #515 (H02-T5) merged as `744f1b75`; ledger H02-T5 is `verified`; native evidence bound to `f8662109`. T5c evidence: [h05-s3-blk-reinit-20261125](../../verification/h05-s3-blk-reinit-20261125/README.md) |
| Score | 43/100 unchanged; H02, H05 and H06 stay open |

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
10. QEMU routes QMP input to a USB HID device only after the guest's first interrupt-IN poll; wait on MFINDEX before printing a READY marker. Reset Device on a disabled slot completes with code 11 and Slot ID 0.
11. Any `kernel/` or `tools/` change, including the runner, changes the source fingerprint: rebuild before `run_usb_hid.py`. `NOS_QEMU_MACHINE` accepts only recorded machines (`pc-q35-8.2`), not `q35`.
12. `run_system.py` puts the QMP socket at `build/<gate>/run/system-<case>-<32 hex>/qmp.sock`; it must stay below 108 bytes. Keep case names short; never shorten the 32-hex suffix, which `capture_system.py` and the RANDOM/images validators pin.

## Next task

H02-T5 is done; its evidence is in
[h02-usb-refusal-20261125](../../verification/h02-usb-refusal-20261125/README.md).

H05-T5c slice 1 is merged (#556, `945847d8`) and `verified` in the ledger.

Slices 2–4 of T5c wait for the following:

- Slice 2 needs B's LOCK/UNLOCK ADR (key erasure).
- Slice 3 is S's remount.
- Slice 4 is the ring-3 re-grant.

Other open items:

- H04 (in-flight TX at a link-down edge) needs agreement with N.
- Physical H02–H06 work stays blocked on the owner decisions above.
