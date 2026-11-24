# H06-T6 removable-media HW-INV image (QEMU) — 24 November 2026

Scope: QEMU/TCG evidence that the paired `native-hw-inventory` kernel and
`hw-inventory` loader boot from a host-built raw disk image instead of QEMU's
virtual FAT directory, and that the image is deterministic and bound to a
recorded SHA-256. Not a physical boot, not a USB medium, not a signed hardware
record and not H06 acceptance.

| Item | Value |
|---|---|
| Source commit | `2a96d3dde3a1da2507a6659edfe6bf890e344a1f` (branch `claude/h06-t6-hwinv-image-20261124`, base main `e5ab3693`), clean checkout |
| Source fingerprint (v3) | `32158ae88e6e2071…` (full value in the result files) |
| Emulator | local QEMU 10.2.1, machine `pc-q35-10.0`, TCG, one CPU, 256 MiB, OVMF without enrolled keys, `isa-debug-exit` at `0xf4` attached |
| Image | `tools/hw_inventory_image.py`: 64 MiB raw GPT disk, one FAT32 EFI system partition at LBA 2048–131038, one sector per cluster, 8.3 upper-case names, timestamps 1980-01-01, GUIDs and volume serial derived from the payload |
| Image SHA-256 | `a4ca34eb088294df74b3839bdd32dde990d9d98aaddd4859982697dcae15c3ed` (67 108 864 bytes), [image-record.json](image-record.json) |
| Payload | `KERNEL.ELF` `80d6f1c8…`, `EFI/BOOT/BOOTX64.EFI` `45fde841…` (equal to the build manifest) |

| Case | Boot medium | Result |
|---|---|---|
| `image` | the raw image as a read-only virtio disk | passed: full HWINV block in order, QMP `query-pci` equal, halted without the exit port, image SHA-256 unchanged after the run, raw exit 0 from QMP `quit` |
| `plain`, `iommu` | virtual FAT directory | passed (unchanged) |

Determinism: two `tools/hw_inventory_image.py` runs on the same build gave the
same SHA-256. The host tests also check that the same payload gives identical
bytes and that a changed payload changes the hash.

Independent readers (host tests and a manual run): `sgdisk -v` finds no problems
and reports the ESP type GUID; `fsck.fat -n -v` on the extracted partition
accepts the volume (32-bit FAT entries, 126 959 data clusters) and changes
nothing; `mcopy` reads back every file byte-identically.

Negative control: an image packed from the same ESP without `kernel.elf` stopped
at the loader's `M1:FAIL:KERNEL_OPEN` and stayed halted, so the case really
loads from the image's FAT32 volume.

Fail-closed inputs (host tests): names outside 8.3, symbolic links, case-folded
name collisions, a payload larger than the volume and any ESP entry other than
the inspected `kernel.elf` and `EFI/BOOT/BOOTX64.EFI` (exact-set check shared
with `tools/package_windows_qemu.py`) are refused.

Independent verifier (fresh clone of `730a96a7`, clean, own build and target
directories): `image` case passed (raw 0, all checks), two image runs gave the
same hash, sgdisk/fsck.fat/mtools agreed and the FAT32/GPT layout was reviewed
against the specifications without format defects. Its kernel SHA-256 equalled
this build's, but its loader differed (`c8e1baf6…`), so its image hash was
`d716f697…`: the expected effect of the not-yet-reproducible loader. It found
two defects, both fixed in `872322d5` (re-captured at `2a96d3dd` after review fixes: one ESP snapshot for the image and every recorded hash): the `test_hw_evidence` "ready requires"
mutation no longer reached its check, and the image tool did not enforce the
exact ESP file set.

Commands: `tools/build.py --kernel-feature native-hw-inventory` (exit 0);
`tools/hw_inventory_image.py` twice (exit 0, same hash);
`tools/run_hw_inventory.py --case all --run-root <short dir>` (exit 0);
`python3 -B -m unittest discover -s tools/tests -p 'test_hw_*.py'` (92 tests,
OK); `tools/hw_evidence.py --check`, `tools/hw_matrix.py --check` (exit 0).

Not run: a physical boot or USB mass-storage boot; OVMF boot from a USB
controller; a rebuild that reproduces the image hash from an independent
checkout (the loader is not bit-reproducible until the B04 linker fix lands, so
the hash binds this recorded build only); `SECUREBOOT=0`/`1` native runs;
default/M2 smoke (no kernel or loader source changed); `actionlint` (hosted CI).
GPT/FAT32 is an assumption until owner decision `OD-HW-BOOT`.
