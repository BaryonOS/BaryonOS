# S06 — collections, metadata, revision history and transactions (volume v2)

Date: 29 October 2026. Baseline: M6e (`NOSVOL01`, IPC v5, ADR-040). Decision:
ADR-041. Evidence status: [S06-ACCEPTANCE](../verification/S06-ACCEPTANCE.md).

## Objective and boundaries

M6e stores large documents consistently across crashes but knows only a name
and current content. Materiaal (PRODUCT-SPEC §2, §6) requires more from the
storage layer: find documents in collections, retain type and labels, view
an older version and restore it as a new revision, recover a deleted document
and move multiple documents in one consistent step. Volume v2 provides these
storage functions natively, in the same ring-3 service and using the same
copy-on-write commit protocol.

| Materiaal task (PRODUCT-SPEC) | Storage function v2 |
|---|---|
| Find by title/type/collection | LIST with collection filter; STAT returns name, `kind` and `tags` |
| Collections “Vandaag”, “Huis”, “Archief” (Today, Home, Archive) | COLLECTION: create, rename, remove (only when empty); COLLECTION_STAT: read name, presence and revision |
| Open object history | STAT with history; READ with version selector |
| Restore version as a new revision (P2-05) | RESTORE: old content becomes a new revision; remaining history is preserved |
| Undo deletion | DELETE retains the last version as history; RESTORE retrieves it |
| Move/organise multiple documents together | TRANSACT: all-or-nothing over at most 4 objects |
| Free space | PURGE: permanently remove history |

Boundary: this is the storage layer. The Materiaal interface is not yet
native (G/U criteria); title search occurs in the app over LIST results.

Trial budget (project choice, not a product limit): 2 historical versions
per object, 4 collections per domain, at most 4 objects per transaction.
Other M6e limits apply: 16 objects, 64 KiB per version, 64 chunks, quota 40.

Outside this step: search index, arbitrary key/value metadata, nested
collections, collections shared between domains, unlimited history, B02
(authentication/encryption), S09 (backup/export) and migration of v1 volumes.

## Disk format v2 (`NOSVOL02`)

Scoped sectors `[4, 584)`. Each table copy has 34 sectors
(copy A 0–33, copy B 34–67); content chunks begin at sector 68.

- Sector 0: header (magic, version 2, geometry, generation, request ID,
  `MAX_CHUNKS`, quota, history depth, collections per domain) and at offset 40
  a clock per domain (u64): that domain's commit count.
- Sector 1: 8 collection records of 56 bytes (revision, request ID, name
  length, presence, name ≤ 32 bytes). Index = domain × 4 + collection − 1.
- Two sectors per object: header (revision, request ID, created/modified as
  own-domain clock values, `tags` u32, `kind` u16, presence, collection,
  history depth and current version) and history (2 versions of 216 bytes).
- A version: version revision, length, chunk list with per-chunk FNV,
  content checksum and name. Unused fields are canonically zero / `0xFFFF`.
- All sectors carry the same trailer as v1: generation, attempt digest over
  the payload of all 34 sectors and a sector checksum.

## Admission of empty and foreign volumes

An empty table is insufficient to treat a volume as new: a foreign file
system may leave the NOSVOL02 table region unused while containing data
elsewhere. If both table copies are empty and no valid format marker exists,
recovery therefore also reads every sector of the experimental volume boundary
`[0, 580)`. Only a completely zero-filled volume is then accepted as generation
0. Non-zero data without recognisable NOSVOL02 state returns `Corrupt`; the
service neither formats nor writes anything.

This also includes both table copies: arbitrary non-zero bytes in slot 1
alone are not evidence of an interrupted write to the project's table. Without
a valid marker or recovered table, the complete table region must also be
zero. With a valid flushed marker, an interrupted first table remains recoverable.

Before the first upload, `BEGIN` first accepts valid empty or already marked
volume input and writes sector 0 as a `NOSFMT02` format marker, followed by
`FLUSH`, before the first content sector. Other initial mutations place the
same marker before their table commit. After an interrupted first upload,
recovery can therefore recognise the marker and return the logically empty
generation; uncommitted content remains orphaned data and is not presented
as a file. Once generation 2 is written to table slot 0, that table replaces
the marker. An invalid or damaged marker is not evidence of emptiness.

A directly constructed service does not trust a supplied generation either:
before its first mutation, it compares recovered disk state with the supplied
table. Only `Service::recover` may mark the admission check as already complete.

The marker contains format version and geometry and has a checksum against
accidental damage. This is neither authentication nor protection against a
deliberately forged marker. The native foreign-media fixture contains only
a Btrfs-like magic value at the known 64 KiB offset; it is not a valid Btrfs
volume and does not demonstrate complete detection of other file systems.

## Invariants (additional to M6e)

1. **History is strictly older.** Version revisions decrease: current >
   history[0] > history[1]. A deleted object has no current version,
   collection, `tags` or `kind`, but may retain history.
2. **Shared chunks only when unchanged.** Versions of one object may share a
   chunk only at the same position with the same checksum (RESTORE references
   again, does not rewrite). Objects never share chunks.
3. **Quota includes history.** A chunk counts once per object; retained history
   holds space until PURGE. `Quota`/`Full` before any I/O.
4. **Collections are domain-bound.** An object can belong only to an existing
   collection in its own domain; deleting a non-empty collection returns
   `Busy`. Domain derives exclusively from the kernel-bound endpoint.
5. **Transactions are all-or-nothing.** TRANSACT checks every precondition
   (existence, expected revision, collection, distinct objects) before one
   table write. One transaction = one generation. A crash during the table
   write leaves the previous generation entirely intact.
6. **Every commit is traceable.** The header's request ID appears on at least
   one changed object or collection; COLLECTION_STAT and STAT return that
   request ID so an `Unknown` outcome can be resolved by reading. The response
   is definitive only from a writable instance, after its flush under
   invariant 9. A read-only instance flushes nothing: it may see an attempt
   that exists only in disk cache and disappears after power loss. Its
   response is therefore provisional.
7. **Domains do not reveal each other's activity.** Every commit increments
   exactly one domain clock; the clocks sum to the generation. A domain's
   revisions and timestamps never exceed its own clock. Read responses
   (STAT, LIST, COLLECTION_STAT, READ) and revisions, timestamps and counters
   in mutation responses contain only own-domain state. Two cross-domain
   channels deliberately remain (residual risk):
   - `Full` depends on shared volume space. Combined quotas (2 × 40 chunks)
     exceed the volume (64), so whether BEGIN returns `Full` also depends
     on what the other domain retains.
   - If one domain's table write fails, the instance closes and the other
     domain also receives `Unknown` until a new instance starts.

   Complete separation requires non-overcommitted per-domain space and
   per-domain fault isolation; that is outside S06.
8. **RESTORE loses no history.** The restored version leaves history and the
   replaced current version takes its place.
9. **Recovery is durable before new writes.** After recovery, a writable
   instance flushes before the first request. Otherwise, a reused chunk may
   damage the fallback copy if the selected copy existed only in disk cache.
   Read-only recovery writes and flushes nothing.

M6e residual risks remain (one unreadable sector in the newest copy causes
fallback; FNV is not authentication).

## IPC v6 (TAG `0x4d36454c4b000006`)

New operations: RESTORE (selector 1–2), PURGE, META (`tags`/`kind`/collection
with revision CAS), COLLECTION (create/rename/remove with CAS), TRANSACT
(1–4 META/DELETE/PURGE sub-operations of 24 bytes on an object index within the
own domain) and COLLECTION_STAT (at 15, read-only: revision, presence, name
and request ID of one own-domain collection). READ gains a version selector;
LIST gains a filter (0 = all, 1 = without collection, c + 1 = collection c);
STAT supplies metadata and, per historical version, revision, length and
content checksum. All unused bytes must be zero; the client strictly validates
every response.

## Acceptance

1. Host tests `volume-core`: v2 format, history, collections, transactions,
   negative cases and exhaustive crash simulation (now also over transactions,
   collections, RESTORE and PURGE), plus the mixed-write-attempt test.
2. Oracle tests (`tests/test_volume_disk.py`): v2 decoding, history and chunk
   rules, collection membership, digest and recovery rules.
3. Native gate `tools/test_volume.py`, nine actual QEMU invocations on one
   frozen source: normal → cold read-only recovery; cut-data, cut-table and
   cut-txn (kill during the table write of a two-object transaction), each
   followed by cold recovery; corrupt image rejected. For cut-table and
   cut-txn, the runner requires an actually torn attempt in the other copy
   (a non-empty, incomplete prefix of g+1 sectors over the previous copy);
   for cut-data, that copy must remain untouched.
4. Normal demonstrates natively: create collections and duplicate creation
   (`Stale`), META with invalid collection (`Range`), a transaction with one
   stale precondition changes nothing (`Stale`), a valid transaction moves
   two documents in one commit, LIST filters, `Busy` for a non-empty collection,
   read history and restore as a new revision, undo deletion, `Quota` caused
   by retained history and PURGE to release space, and read collections with
   COLLECTION_STAT. Cold recovery verifies the complete final state including
   history, collection names and domain clocks (57/27), and rejects every
   mutation, including RESTORE and TRANSACT (`Access`).

An emulator kill is not physical power loss. No production, encryption or
physical-disk claim is made.
