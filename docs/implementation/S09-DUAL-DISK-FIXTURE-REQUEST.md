# S09 — Request: a test-only second virtual disk for the archive fixture

**Status: approved by the project owner on 19 November 2026 (bounds B1–B6
and the write set below), in the S09 session, verbatim: "Yes, approve
B1–B6". Implementation in progress on branch `claude/s09-independent-recovery`.**
The approval covers a test fixture only. It grants no product capability and
changes no product security contract. It does not decide D1–D5 of
[S09-BACKUP-DESTINATION-PROPOSAL.md](S09-BACKUP-DESTINATION-PROPOSAL.md),
which stay open and unchanged.

## Why a decision is needed

The S09 assignment of 19 November 2026 asks for an export from a real volume
service to a separate second virtual disk, followed by a cold restore while the
source disk is no longer available. The archive range today lies on the volume's own
disposable disk ([S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md), "Native
slice"). A second disk is not a fixture-only change. At main `8f970866`:

- `drivers/virtio-blk/src/lib.rs` drives exactly one PCI function, QEMU
  `00:05.0` (`const BDF: u32 = 5 << 11`). Its `unsafe` port-I/O blocks are
  justified by that fixed address ("exact fixed PCI BDF", "no access to the
  boot ESP's BDF"). Six kernel profiles call `discover()`: `m5`, `m6`, `m6b`,
  `update`, `volume` and the legacy `m6`.
- `kernel/src/m2/m3/volume.rs` keeps one `Device` and one DMA pool, and
  `device_call` refuses every block capability whose `scope.device` is not 0
  (line 656).
- `tools/run_volume.py` attaches one volume image at `addr=0x5`. With
  `-nodefaults`, QEMU auto-assigns the ESP's virtio drive to another slot.

A second disk therefore means the kernel takes ownership of a second
DMA-capable device, and the `unsafe` contract of a shared driver
is extended. The assignment does not allow that without an agreed interface and a
released write set.

## Question

> Do you approve a **test-only** second virtio-blk device in the volume
> fixture profile, under bounds B1–B6 below, with the write set listed under
> "Write set to release"?

## Bounds

- **B1 Driver.** Use a closed set of two fixed PCI functions: `00:05.0`
  (today) and one new fixed function, proposed as `00:06.0`. The
  implementation must show that it is not the ESP's auto-assigned slot. An enum
  selects the function; no caller supplies a bus/device/function number.
  `discover()` keeps its behaviour for all six existing callers.
- **B2 Kernel.** The second device exists only under a new test feature
  (proposed name `test-volume-archive-device`). It has its own DMA pages,
  disjoint from device 0. `scope.device == 1` is routed to it. Without the
  feature, `device != 0` stays refused.
- **B3 Grant.** Only the slot-1 backup portal receives a capability, as
  `BlockScope { device: 1, .. }`. Its scope ends below the end of the disk,
  so a past-the-end probe is refused by the scope and not by the device size.
  Export gets READ|WRITE|FLUSH. Verify and restore get READ only. The volume
  service and slot 2 hold no handle on device 1: the existing
  `ARCHIVE_ESCAPE` accounting is extended to it. These cases do not grant the
  device-0 archive range.
- **B4 Images.** The second disk is a disposable raw image under
  `$NOS_BUILD_DIR/run/`, never a host disk or physical device.
- **B5 No product authority.** There is no dialog, no persistent grant and no
  NAS or provider. This is a fixture. It neither accepts D1 (a) as the product
  design nor rules out D1 (b), D2, D4 or D5.
- **B6 Evidence level.** Local QEMU only. No claim about physical disks.

## Native stages it enables (acceptance)

| Stage | Required result |
|---|---|
| Export to disk 1 | Recovered volume on disk 0, mounted read-only; archive written from sector 0 of disk 1; disk 0 byte-identical before and after; `tools/volume_archive.py` parses disk 1 independently |
| Cold restore | New VM with a blank disk 0 (source disk gone) and the disk-1 image alone, granted READ only; `verify`, then `restore`; content and metadata byte-exact against the oracle; disk 1 unchanged |
| Disk 1 absent | Portal receives a device error; nothing is finalized; the volume is unchanged |
| Disk 1 too small | `Full`; nothing past the scope is written; `verify` refuses the range |
| Scope and escape | Portal past-the-end probe and any volume-service access to device 1 are refused and counted |
| Power cut | QEMU killed during an export over an older archive; after a cold boot, `verify` accepts only the older or the complete new archive, otherwise `Corrupt` |
| Wrong device | Portal granted a range of disk 0 instead: `verify` returns `Corrupt` |

The last three rows already have a host model (next section). Native
evidence still requires a QEMU run.

## Already shown on the host (no decision needed)

Branch `claude/s09-independent-recovery`, commit `c4ac9ec1`: five tests in
`services/volume/tests/archive.rs` model the second disk as a separate
`SimDisk` with a volatile write cache. They are summarised in
[S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md), "Evidence and open points".
One result bears on D1. On a single raw range, an interrupted re-export
destroys the previous archive as soon as one differing new sector lands. Only
the unchanged previous archive, the complete new one, or nothing verifies. Keeping
the previous copy until the new one verifies needs either append-only entries
(D1 (b)) or two alternating ranges under D1 (a).

## Write set to release

- S09 owner: `drivers/virtio-blk/src/lib.rs` and its README,
  `kernel/src/m2/m3/volume.rs`, the kernel feature list, `tools/build.py`
  (portal profile), `sdk/examples/volume-docs/src/` (portal device handle),
  `tools/run_volume.py`, `tools/test_volume.py` (`PLAN`) and
  `tests/test_volume_*.py`.
- Lead-owned, protected: the two workflow values that must equal
  `len(PLAN)` (`.github/workflows/kvm-pilot.yml`,
  `.github/workflows/volume-acceptance.yml`) and their check in
  `tests/test_ci_workflows.py`.

## Limits

This is plaintext archive format v1 with an FNV-64 accidental-damage checksum
only. It suits a user-owned local target only. QEMU's virtio-blk is trusted to
follow the protocol; this is not a malicious-device or IOMMU model. A positive
decision makes the fixture possible. It does not make S09 acceptable, which
still needs D1–D5, a user task and the encrypted variant.
