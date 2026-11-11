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
