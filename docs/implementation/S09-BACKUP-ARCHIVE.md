# S09 — Recovery copy and export archive (format v1)

**Status:** host-implemented in `services/volume/src/archive.rs`, tested in
`services/volume/tests/archive.rs`. No native portal yet, no user task,
no physical disk. S09 remains open.

## Goal and invariant

A user can make an **independent copy** of one document domain that
is verifiable and restorable without the source volume, source service or source disk.

- **Export** is a consistent snapshot: no archive is finalized if the domain
  changes during the export (`Stale`), if the target becomes full or gives an I/O error
  (error from the sink), or if read content deviates from the service's own
  content checksum (`Corrupt`). After an error the output is invalid, and
  `verify` refuses it.
- **Verify** has no side effects and accepts only a complete, canonical archive.
- **Restore** never overwrites anything. An occupied target slot or an already used name
  is reported as a conflict and stays untouched. A target collection with
  a different name is never renamed; the object then ends up outside a collection
  (`uncollected`). Each object is committed only after its record and content
  have been checked. Because the trailer only comes after the last object, a
  portal must **first run `verify` on the same bytes**. Without verify,
  on truncation only the earlier, fully checked objects may have been
  restored; the `Report` names those exactly.
- A write with outcome `Unknown` counts as restored only if `stat` shows exactly
  the expected revision, name, length and content checksum; otherwise restore stops with
  `Unknown`. If setting tags, kind or collection fails after a successful write,
  the object is in `restored` and in `bare`, and restore stops with that error.
- Restore assumes there is one writer on the target domain during the restore. A concurrent
  slot change aborts with `Stale`; a concurrently created identical name
  is not detected (the volume does not enforce unique names).
- A deleted target slot (tombstone) is filled. The kept history of that
  slot remains and can later be retrieved via local history; no
  data is lost, but local undo of that slot then shows older, different content.

## Bytes (little-endian)

| Part | Size | Content |
|---|---|---|
| Header | 64 | `NOSBAK\0\x01`, version u16=1, 2×0, number of objects u8 (≤8), number of collections u8 (≤4), 2×0, source generation u64, total content u64, 24×0, FNV-64 over bytes 0..56 |
| Collection record | 48 | index u8 (1..4, strictly ascending), name length u8, 6×0, name[32] (padding 0), FNV-64 over 0..40 |
| Object record | 96 | slot u8 (0..7, strictly ascending), name length u8, collection u8 (0 or present), 0, kind u16, 2×0, tags u32, length u32 (≤64 KiB), source revision u64 (≠0), created u64, modified u64, content FNV-64, name[32] (padding 0), 8×0, FNV-64 over 0..88 |
| Content | length | raw document bytes directly after its own record |
| Trailer | 16 | `NOSBAKZ\0`, FNV-64 over all preceding bytes |

After the trailer the stream must end. Names follow `valid_name` (UTF-8,
1–32 bytes, no `.`/`..`). Duplicate names are allowed, just as in the volume
itself. A source that reports more
bytes than were requested yields `Protocol`; any other deviation yields `Corrupt`.

## What is deliberately not in v1

- **No authenticity or encryption.** FNV-64 only catches accidental
  corruption, truncation and torn transfers, not deliberate tampering. The archive
  is plaintext personal data. It may only go to a target chosen by the user
  that is their own (LOCAL-PRIVACY-CONTRACT: own offline recovery copy). For a NAS or a
  provider client-side encryption is needed (B02, P01, ENCRYPTED-OBJECT-STORAGE);
  that is still open here.
- **Current versions only.** Local history is not a backup and is not
  included. `created`/`modified` are kept and are exportable,
  but on restore the service assigns new times.
- **One domain per archive**, within the current volume limits (8 objects,
  64 KiB per object, 4 collections).

## Evidence and open points

Host evidence (gnu host target, Rust 1.90.0, access via the real volume client and service):
round trip after loss of the source and cold restart of the target; every prefix, every
bit flip (bits 0x01 and 0x80 on every byte) and an extra byte are refused;
a too-high source count gives `Protocol`, also at the EOF check after the trailer;
a full target at 132 failing boundaries (every 37th byte and the last byte) and success
at exactly the archive size; concurrent change of a listed object and
of a new object; content that contradicts the service checksum; conflicts without
overwriting; quota during restore; invalid domain. Of 13 mutation trials,
11 make a test fail (name conflict, EOF, both `Protocol` checks, generation,
content checksum on export and on `Unknown`, collection mapping, `bare`, revision 0,
name padding); see the evidence file. Resealed, non-canonical records
(20 cases) are refused; `Unknown` (landed/not landed, and landed with a
different content checksum), failed META, a tombstone slot and duplicate source names are
tested. Not covered by a test: omitting the revision comparison on
`Unknown` (a revision that is one too far is caught) and the length check
list/stat on export (only relevant if another writer or the service itself
deviates).

Open for S09/FR-O06: native backup portal with a target volume and capability
chosen by the user, native QEMU trial with a second virtual disk,
user export to common formats (e.g. plain text/image per
document), encrypted variant for NAS/provider, and a user task.

## Native slice — local QEMU gate passes (13 November 2026, branch `feat/s09-native-archive`)

**Status:** implemented and locally gate-tested; S09 stays **open** (see
"Still open"). Evidence: [verification/s09-archive/NATIVE-20261113.md](../verification/s09-archive/NATIVE-20261113.md).

**Range adapters (host-tested).** `archive::DiskSink` and `archive::DiskSource`
store an archive from sector 0 of a dedicated raw block range. No extra
framing: the length follows from the header counts (bounded by the range,
otherwise `Corrupt`) and `verify` remains the only acceptance gate. The native
block adapter is shared by service and portal in `volume-docs/src/disk.rs`.

**Kernel fixture profile** (`kernel/src/m2/m3/volume.rs`, features
`test-volume-archive-export|restore|corrupt|conflict`, cases 7–10):
- Slot 1 is the backup portal. It alone receives a block capability on device
  0, sectors 1000..2040 of the disposable test image, disjoint from the volume
  scope 4..584 (compile-time assertion). The range ends below the 2048-sector
  image, so the portal's past-the-end probe is refused by the capability
  scope rather than by the device size; the largest v1 archive (525 328 bytes,
  1027 sectors) fits. Export grants READ|WRITE|FLUSH; restore, corrupt and
  conflict grant READ only (`ARCHIVE_READONLY_RIGHTS`). The handle is startup word 504.
  No handle enumeration exists, so the manifest counts instead: it fails with
  `ARCHIVE_ESCAPE` unless the grant adds exactly one block object, node and
  handle, that handle is held by slot 1, the service and slot 2 hold exactly
  as many handles as before, and the service's own block handle resolves to a
  different object key. Service and slot 2 keep their earlier authority.
- During export the device stays writable but the volume service receives
  READ only (`service_read_only`), so every write and flush goes to the archive.
- Archive I/O is counted per capability object at publication and kept out of
  the volume table/data write classification. `finish` fails with
  `ARCHIVE_IO` unless export wrote, flushed and read back only through the
  archive object, and restore, corrupt and conflict only read it. Evidence lines
  `VOLUME:ARCHIVE:READS|WRITES|FLUSHES`; markers
  `VOLUME:CASE:ARCHIVE_EXPORT|ARCHIVE_RESTORE|ARCHIVE_CORRUPT|ARCHIVE_CONFLICT:PASS`.
- Calls on the portal's own archive object (pending collects, the scope
  probe) are not counted as app device denials; every app's own-handle and
  forged-handle probes still are, and each app must show at least one.

**Portal app.** `volume-app` built with feature `archive-portal` (separate
target directory `volume-docs-portal`; the normal build would exceed the
64 KiB startup text boundary with both paths). The archive cases use the
normal `volume-service` build unchanged. The 64 KiB archive scratch is the
private RW/NX heap the kernel premaps at `0x6000_0000`. Restore, corrupt and
conflict share one flow: `verify` the whole archive, then `restore`. Corrupt must stop
at `verify` with `Corrupt`, and the volume must stay empty.

**Serialization.** The broker serves one block request at a time and answers
others with Busy. The portal therefore reaches a READY service before its
first archive read, and slot 2 waits for kernel gate 2 (portal phase 2) before
using the service. The first gate run without this ordering failed the
corrupt case with a Busy status instead of `Corrupt` (step 740).

**Runner stages** (`tools/run_volume.py`, plan of `tools/test_volume.py`, now
13 QEMU runs): `archive-export` on the image left by `recover` (volume mounted
read-only, every byte outside the archive range unchanged, zero padding, no
write past the archive, writes = ⌈length/512⌉); `archive-restore` and
`archive-corrupt` on a fresh image that holds only the copied archive range
(source loss); corrupt flips one byte at `length // 2`. The independent reader
`tools/volume_archive.py` parses every guest-written archive and must refuse
the damaged one. Restored objects are compared byte for byte with the archive
through `volume_disk.analyse(..., opaque=...)` (revision 1, or 2 after
metadata), and the guest digests must equal the oracle digests. Export also
compares each object's created and modified times with the volume; restore
expects only the collections a restored object refers to, because restore
creates no unreferenced collection. The archive
header carries LIST's domain generation (sum of the domain's object and
collection revisions), not the global volume generation.

`archive-conflict` restores the same archive into a fresh image whose domain
already holds two written objects and two collections: one object occupies an
archived slot, the other carries the name of another archived object;
collection 1 has the archived name, collection 2 another name. The expected end
state comes from `volume_archive.expected_restore`, which follows the restore
rule above and shares no code with `volume-core`. The stage requires at least
one slot conflict, name conflict, uncollected object and reused collection.
Both conflicting archived objects stay unrestored, the pre-existing objects
stay at revision 1, collection 2 is not renamed, and the archived object that
referred to it is restored outside a collection. The conflict and plain restore
oracles each refuse the other stage's after-image. Two rules cannot be reached
with this two-collection archive and are host tests in
`services/volume/tests/archive.rs` instead: no collection is created for a
conflicting object, and an uncollected object without other metadata stays at
revision 1.

**Still open for S09:** a user-chosen target behind its own capability (not a
fixture range on the same disk); export to documented common formats; the
encrypted NAS/provider variant; a user-facing task; a native full-target case;
a second simultaneously attached disk (archive and volume
share one virtual device here). Plaintext archives are used only on
harness-assigned local test images.
