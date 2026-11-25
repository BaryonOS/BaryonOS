# Specialist H — current handoff

Compact resume pointer for the H track (H01 guard, H02–H06). Merge state is
reported in the PR comment `HANDOFF H <cycle_id>`, not here.

| Field | Value |
|---|---|
| cycle_id | `h02-t5-20261125` |
| Location | local H session (took over from the cloud H session via #503; cloud HANDOFF on #507, comment 7151275971) |
| Task | H02-T5 native per-port refusal (hub, non-boot tablet, high-speed keyboard, storage) — implemented, native evidence at `1bb81100`, hosted gates pending |
| Branch | `claude/h02-t5-usb-refusal-20261125` (base main `ad55e019`) |
| PR | #515 (draft) |
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

10. QEMU routes QMP input to a USB HID device only after the guest's first interrupt-IN poll; wait on MFINDEX before printing a READY marker. Reset Device on a disabled slot completes with code 11 and Slot ID 0.
11. Any `kernel/` or `tools/` change, including the runner, changes the source fingerprint: rebuild before `run_usb_hid.py`. `NOS_QEMU_MACHINE` accepts only recorded machines (`pc-q35-8.2`), not `q35`.

## Next task

H02-T5 evidence: [h02-usb-refusal-20261125](../../verification/h02-usb-refusal-20261125/README.md).
Remaining on #515: hosted `native-xhci` green on the final head, merge main,
`progress.py --write`/`--check`, ready, `merge-hold` rerun, squash-merge with
the expected head, `HANDOFF H` comment; flip H02-T5 to `verified` in the next
PR. Then the next H item: H05 virtio-blk reinit after S3 wake (security review
first); H04 needs agreement with N.
