# S09 — Native archive format v2 (specification)

**Status:** specification plus an independent host reader
(`tools/volume_archive.py`: `parse_v2`, `parse_any`, `stored_length_v2`) and
its host tests (`tests/test_volume_archive_v2.py`), a Rust reader
(`walk_v2`/`verify_v2` in `services/volume/src/archive.rs`) checked against
fixtures produced by the Python encoder
(`services/volume/tests/fixtures/archive-v2/`), and an L1 `restore_native`
host model tested against the real service (`services/volume/tests/archive_v2.rs`).
There is **no writer**, no wire extension and nothing native. No v2 archive
produced by `volume-core` has been parsed. This document makes **no claim of
recovery fidelity**; it defines what a v2 archive records and what a reader must refuse.
Format v1 ([S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md)) and its restore rule
remain the only implemented recovery path and are unchanged.

**Scope.** Native recovery level L1, route step 1 (owner decision R1,
19 November 2026): the v2 byte layout and a v2 parser independent of
`volume-core`. The layout follows the indicative proposal in section 3.3 of the
S09 restore-fidelity analysis (`S09-RESTORE-FIDELITY.md`, unmerged at the time
of writing) and is designed so that the later, separately reviewed L2
(immutable provenance) needs no v3 archive. L3 is not pursued.

Line references are to `origin/main` at `c604fd07`.

## 1. Source model being recorded

- An object `Entry` holds an object revision, a `request_id`, logical
  `created`/`modified` stamps, tags, kind, collection, a presence flag, the
  current `Version` and up to `HISTORY = 2` earlier versions, newest first
  (`services/volume/src/table.rs:128-146`, `services/volume/src/lib.rs:43-44`).
- A `Version` records its revision, length, name and content checksum and has
  **no timestamp** (`table.rs:33-43`).
- `created`/`modified` are values of the owning domain's logical commit clock
  (`table.rs:134-136`, `table.rs:402-406`, `services/volume/src/service.rs:594-618`);
  no wall-clock time exists in the volume.
- A deleted object keeps its content as history (tombstone) until PURGE
  (`service.rs:684-696`); PURGE clears the history but keeps the non-zero
  object revision.
- A collection holds a revision, `request_id`, presence flag and name
  (`table.rs:335-341`); a deleted collection keeps a non-zero revision and an
  empty name (`table.rs:386-388`).
- The domain generation reported by LIST is the wrapping sum of all object and
  collection revisions of the domain, present or not
  (`table.rs:465-474`, `service.rs:1035`).

## 2. Byte layout (little-endian)

All integers are little-endian. "Seal" is FNV-64 (the function used by v1,
`tools/volume_disk.py:49-52`) over the record bytes before the seal. FNV-64
detects accidental damage only; it is **not authentication**
(`services/volume/src/archive.rs:11-13`).

Stream order:

1. Header (64 B).
2. Collection records (64 B each), index strictly ascending.
3. Per object, slot strictly ascending: one object record (128 B), then its
   version records (64 B each), each version record **immediately followed by
   its content** (`length` bytes). Version order: the current version first
   (present objects only), then history newest first.
4. Trailer (16 B): `NOSBAKZ\0`, then FNV-64 over all preceding bytes.

The stream must end after the trailer.

### 2.1 Header (64 B)

| Offset | Size | Field | Rule |
|---|---|---|---|
| 0 | 8 | magic `NOSBAK\0\x01` | identical to v1 (see 5) |
| 8 | 2 | version u16 | `2` |
| 10 | 1 | flags u8 | bit 0 `TOMBSTONES`, bit 1 `ACCOUNTED`; bits 2–7 zero |
| 11 | 1 | reserved | zero |
| 12 | 1 | object record count u8 | ≤ 8 (`PER_DOMAIN`) |
| 13 | 1 | collection record count u8 | ≤ 4 (`COLLECTIONS`) |
| 14 | 2 | version record count u16 | ≤ 24 (`8 × (HISTORY + 1)`); equals the version records present |
| 16 | 8 | domain generation u64 | as LIST reports it; verified (S9) only with `ACCOUNTED` |
| 24 | 8 | total content u64 | ≤ version count × 65 536 and ≤ `CONTENT_V2`; equals the sum of version lengths |
| 32 | 8 | source domain clock u64 | the exporting domain's own clock |
| 40 | 16 | reserved | zero |
| 56 | 8 | seal over 0..56 | |

The domain generation is a recorded source value. With `ACCOUNTED` clear the
reader returns it unverified, and a restore report must label it as an
unchecked recorded value.

`stored_length_v2(header) = 64 + 16 + 64·collections + 128·objects +
64·versions + total content`; the header is not trusted at that point.

### 2.2 Collection record (64 B)

| Offset | Size | Field | Rule |
|---|---|---|---|
| 0 | 1 | index u8 | 1..4, strictly ascending |
| 1 | 1 | state u8 | 1 present, 0 deleted |
| 2 | 1 | name length u8 | ≤ 32; present: `valid_name`; deleted: 0 |
| 3 | 5 | reserved | zero |
| 8 | 8 | collection revision u64 | ≠ 0 |
| 16 | 32 | name | padding after the name zero |
| 48 | 8 | reserved | zero |
| 56 | 8 | seal over 0..56 | |

### 2.3 Object record (128 B)

| Offset | Size | Field | Rule |
|---|---|---|---|
| 0 | 1 | slot u8 | 0..7, strictly ascending |
| 1 | 1 | state u8 | 1 present, 0 absent |
| 2 | 1 | collection u8 | 0, or the index of a **present** archived collection; absent: 0 |
| 3 | 1 | history count u8 | ≤ 2 |
| 4 | 2 | kind u16 | absent: 0 |
| 6 | 2 | reserved | zero |
| 8 | 4 | tags u32 | absent: 0 |
| 12 | 4 | reserved | zero |
| 16 | 8 | object revision u64 | ≠ 0 |
| 24 | 8 | created u64 | logical stamp |
| 32 | 8 | modified u64 | logical stamp |
| 40 | 80 | reserved | zero |
| 120 | 8 | seal over 0..120 | |

The object name is the current version's name, as in the table
(`table.rs:163-165`); it is not repeated in the object record. Number of
version records that follow: `state + history count`. An absent object with
history count 0 is a **content-free record**; an absent object with history
count ≥ 1 is a **tombstone**.

### 2.4 Version record (64 B)

| Offset | Size | Field | Rule |
|---|---|---|---|
| 0 | 1 | name length u8 | 1..32, `valid_name` (`tools/m6c_disk.py:42-52`) |
| 1 | 3 | reserved | zero |
| 4 | 4 | length u32 | ≤ 65 536 (`MAX_LENGTH`) |
| 8 | 8 | version revision u64 | ≠ 0 |
| 16 | 8 | content FNV-64 | equals FNV-64 of the following content |
| 24 | 32 | name | padding after the name zero |
| 56 | 8 | seal over 0..56 | |

Content is stored per version without deduplication, so the reader stays
simple; versions that share chunks in the source are stored twice.

## 3. Flags and the owner decisions R2–R4

| `TOMBSTONES` | `ACCOUNTED` | Permitted beyond present objects and present collections | Generation check |
|---|---|---|---|
| 0 | 0 | nothing | not performed |
| 1 | 0 | tombstones | not performed |
| 0 | 1 | content-free records, deleted collections | performed |
| 1 | 1 | tombstones, content-free records, deleted collections | performed |

- A tombstone with `TOMBSTONES` clear, a content-free record with `ACCOUNTED`
  clear, and a deleted collection with `ACCOUNTED` clear are refused.
- With `ACCOUNTED` set the writer must list **every** slot and every collection
  of the domain whose revision is non-zero; the reader cannot see an omission
  directly. The generation check (S9) does not detect every omission: it is a
  sum modulo 2^64 (as `table.rs:465-474`), so
  omitted revisions whose sum is 0 modulo 2^64 (for example two revisions of
  2^63) leave it unchanged. S9 is therefore a consistency check for
  accidental omission by an honest writer, not proof that every non-zero
  revision is present; like every rule here it gives no protection against a
  deliberately resealed archive (section 6).
- A content-free record carries `created`/`modified` and is checked under the
  same stamp rules as any object (the table requires both stamps non-zero for
  every entry with a non-zero revision, `table.rs:286-290`).
- A purged slot and a tombstone recorded without its content are
  indistinguishable in the archive: both are content-free records.

**R2: decided by the project owner on 19 November 2026.** Tombstone content
is excluded by default: a writer leaves `TOMBSTONES` clear unless a separately
reviewed decision says otherwise. Content-free records (`ACCOUNTED`) are
permitted only where they are demonstrably needed for the consistency check
(S9); they carry no content and restore never recreates deleted content from
them. Consequently: (a) `TOMBSTONES` is off by default; (b) content-free
records for deleted, purged and tombstoned-but-excluded slots and for deleted
collections exist only so that the generation check applies; (c) a
content-free record carries the source `created`/`modified` stamps as
historical data; (d) a purged slot and an excluded tombstone are not
distinguished. A restore treats every content-free record as absent.
For L1 the native restore (`restore_native`) refuses any archive with
`TOMBSTONES` set, before its first write, rather than replaying tombstones;
replaying them would recreate deleted content in the destination.

**Versions from before a delete: decided by the project owner on
19 November 2026 — refused** (answer to "may `restore_native` replay
versions from before a delete?": "Nee, weigeren"). A DELETE moves the current
version into history and raises the entry revision; a later COMMIT on the
absent entry does not push history (`push_history` is a no-op while absent,
`table.rs:199`), so pre-delete versions become history of the rewritten
object, and nothing in the archive or on the wire marks the delete. A
committed version's revision equals the entry revision after that commit,
and DELETE, RENAME, META and TRANSACT each raise the entry revision without
a new version; RESTORE creates a current version at the next revision. A
step of exactly 1 therefore excludes an intervening DELETE, but does not
prove that no operation occurred (the newer version may come from a
RESTORE). L1 therefore replays, per present object, only the
longest prefix of its versions (current first, history newest first) whose
consecutive revisions differ by exactly 1; every version after the first gap
is not replayed and is counted in `NativeReport.skipped`, and the capacity
checks count replayed versions only. L1 cannot distinguish a DELETE gap from
a META or RENAME gap, so a gap caused only by metadata also drops the older
versions (conservative). The reader still verifies every version; only the
replay is limited.

**Source identity: decided by the project owner on 19 November 2026 — no
source identifier is recorded.** The archive records no volume or domain
identifier; `services/volume/src` defines none, so there is no source value to
record and inventing one would be fabrication. A stable identifier would also
link all archives of one domain to each other, and L1/L2 as analysed need
none. Recorded revisions and stamps are therefore meaningful relative to the
source domain clock recorded in the same header, not to a named source. Header
bytes 40..56 stay reserved and zero; a v2 reader refuses any non-zero value
there and any unknown flag bit. This holds for archive v2 as a whole: no
source identifier is recorded behind a feature flag either. The decision, its
consequences for the format and for the L2 `origin_*` provenance fields, and
the condition for revisiting it (a new owner decision) are recorded in
`ADR-S09-RESTORE-FIDELITY-20261119`, together with R2–R4. That ADR is pending
in draft [PR #403](https://github.com/BaryonOS/BaryonOS/pull/403) and is not
yet in `docs/DECISIONS.md` on `main` or on this branch.

**R3: decided by the project owner on 19 November 2026.** Native restore
targets only a demonstrably never-used domain; existing domains are neither
overwritten nor merged. This is restore policy, not archive state; the format
has no field for it and needs none. The L1 host model proves "never used"
with existing operations only: every slot's STAT reports `NotFound` with
revision 0 and every collection's COLLECTION_STAT reports revision 0 (every
publish path raises at least one such revision and none decreases).

**R4: decided by the project owner on 19 November 2026.** A destination range
for one archive is sized for the maximum archive of 494 416 bytes
(`MAX_ARCHIVE_V2`, section 4.4). Destination metadata (catalog and entries of
the destination service) has its own capacity and its own verification and is
not counted in that bound.

## 4. Validation rules

A reader accepts only a complete, canonical archive. Every rule failure is a
refusal (`ValueError` in the host reader; `Corrupt` in a future native reader).
Values are only copied from validated records; nothing is derived, defaulted or
interpolated.

### 4.1 Structural rules

- Header: magic, version 2, no unknown flag bit, reserved bytes zero, counts
  within limits, total content ≤ version count × 65 536, seal.
- Collection records: seal, index range and strict ascent, state ∈ {0, 1},
  name length ≤ 32, present ⇒ `valid_name`, deleted ⇒ name length 0, padding
  and reserved bytes zero, revision ≠ 0.
- Object records: seal, slot range and strict ascent, state ∈ {0, 1},
  history count ≤ 2, reserved bytes zero, revision ≠ 0, collection 0 or a
  present archived collection, absent ⇒ collection, kind and tags zero
  (`table.rs:293-298`).
- Flag gating as in section 3.
- Version records: seal, `valid_name`, reserved and padding zero, length
  ≤ 65 536, content checksum.
- Totals: version records ≤ and finally = header version count; content
  ≤ and finally = header total; trailer magic and checksum; no trailing bytes.

### 4.2 Semantic rules (fidelity analysis 3.2)

Each is a separate check with its own refusal message and its own test.

| # | Rule | Table source |
|---|---|---|
| S1 | every version revision ≠ 0 | `table.rs:310` |
| S2 | newest version revision ≤ object revision (present: the current version; tombstone: the newest history version, the table's `revision + 1` exclusive bound) | `table.rs:292`, `table.rs:300-306` |
| S3 | version revisions strictly decreasing from current through history | `table.rs:308-311` |
| S4 | `created ≠ 0` | `table.rs:290` |
| S5 | `created ≤ modified` | `table.rs:264` |
| S6 | `modified ≤ source clock` | `table.rs:664` |
| S7 | `object revision ≤ source clock` | `table.rs:664` |
| S8 | `collection revision ≤ source clock` | `table.rs:657` |
| S9 | with `ACCOUNTED`: (Σ object revisions + Σ collection revisions) mod 2^64 = header generation; detects accidental omission, not every omission (section 3) | `table.rs:465-474` (`wrapping_add`) |
| S10 | chunk budget: Σ over objects of (largest version's ⌈length / 4096⌉) ≤ `QUOTA = 40` | `lib.rs:107`, `table.rs:117`, `table.rs:178-191`, `table.rs:316-329`, `table.rs:684` |

S10 is derived from the table invariants, not from the fidelity analysis.
A version's chunk count is `⌈length / 4096⌉` (`table.rs:117`), a chunk occurs
at most once within a version (`table.rs:316-329`), an object's unique chunk
usage is therefore at least its largest version's chunk count
(`table.rs:178-191`), and a domain holds at most `QUOTA` unique chunks
(`table.rs:684`). The rule is necessary for a source-produced archive, not
sufficient; it refuses archives no valid source domain can produce.

The source clock bound uses the exporting domain's own clock; each domain
clock bounds its own revisions and stamps (`table.rs:644-666`).

### 4.3 Explicitly not checked

- Destination capacity. S10 shows only that a source could have held the
  content. L1 replay allocates fresh chunks for every WRITE
  (`service.rs:410-420`), so a source whose versions share chunks can still
  stop with `Quota` on replay and leave a partly restored slot that no
  in-domain operation clears (fidelity analysis 3.2, "Interruption").
  Capacity is therefore checked by `restore_native`, not by the reader:
  before its first write it sums ⌈length / 4096⌉ over every version to be
  replayed and refuses more than `QUOTA` (`Quota`; exact, because a
  never-used domain holds no chunks and at most `HISTORY + 1` versions per
  object all stay referenced) and more than
  `GUARANTEED_FREE = CHUNKS − (DOMAINS − 1) × QUOTA = 24` (`Full`; the
  volume-wide free count is not observable, but BEGIN keeps every other
  domain's committed plus staged chunks within `QUOTA`). All WRITEs come
  first, then COLLECTION creates, then META, so a refusal or a failed first
  BEGIN commits nothing in the domain, and its report counts every commit
  of this restore that it confirmed (`committed`) so a caller can tell
  "refused, unchanged by this restore" from "failed, used". The count is a
  lower bound after `Unknown` (a lost reply that read-back could not
  confirm) or `Protocol` (a malformed reply): that request may have
  committed even at `committed == 0`, so the domain must then be treated as
  possibly used, and `never_used` decides on a retry. `committed == 0` does not mean the
  domain is never used: `Stale` reports an already used domain with zero.
  The zero count covers committed state only. If a BEGIN reaches the
  service but its reply is lost, or an upload is interrupted before its
  COMMIT, a staged upload may remain and reserve chunks until it is
  replaced; `never_used` observes committed revisions, not staged uploads.
  The first mutation is a BEGIN only when a version is replayed; a
  token-bound upload staged in the domain then makes it `Busy` before any
  commit, while a legacy (token 0) staged upload is replaced by that BEGIN,
  as for any legacy BEGIN. An archive with no replayable version but a present collection
  starts with COLLECTION, which a staged upload does not refuse: that
  restore succeeds and makes the domain used, and the staged upload can
  still COMMIT afterwards because COLLECTION leaves object revisions
  unchanged. No read operation exposes staged uploads, so `restore_native`
  cannot detect this; it falls under the existing precondition that the
  caller is the only writer of the destination domain
  (`a_collection_only_restore_is_not_refused_by_a_staged_upload`).
- `modified` versus the object revision: the table has no such relation.
- Uniqueness of names: the volume does not enforce it (v1 likewise).

### 4.4 Bound (constants)

Each version's chunks are a subset of its object's chunks, and a domain holds
at most `QUOTA = 40` chunks of `CHUNK = 4096` bytes (`lib.rs:30,40`,
`table.rs:684`). Hence

- `CONTENT_V2 = (HISTORY + 1) × QUOTA × CHUNK = 491 520` bytes; the reader
  refuses a larger header total (`content exceeds the v2 bound`);
- `MAX_ARCHIVE_V2 = 64 + 4·64 + 8·128 + 24·64 + 491 520 + 16 = 494 416` bytes
  (966 sectors of 512 bytes), the largest `stored_length_v2`. It follows from
  `CONTENT_V2` and the count limits, so the reader does not check it
  separately (a test reaches it exactly with eight objects of three
  20 480-byte versions, 40 chunks under S10); a range adapter compares `stored_length_v2` against it before
  reading (R4).

The v1 header bound `count × MAX_LENGTH` is kept for v2 as
`version count × MAX_LENGTH`, in addition to `CONTENT_V2`. Both header bounds
are early refusals before any record is read; S10 is the per-object rule and
implies `CONTENT_V2`.

## 5. Compatibility

- **Same magic family.** v1 and v2 share the 8-byte magic `NOSBAK\0\x01`; the
  trailing `\x01` is part of the constant and is not a version number. The u16
  at offset 8 is the only discriminator.
- **v1 readers refuse v2.** The v1 reader requires version 1
  (`services/volume/src/archive.rs:283`, `tools/volume_archive.py:47`), so a
  v2 archive is refused as corrupt.
- **v2-capable reader dispatches.** `parse_any` passes any input whose bytes
  8..10 are not `02 00` unchanged to the v1 `parse`, so v1 behaviour, including
  error messages, is identical. A future native `walk` dispatches the same way.
- **No upgrade by synthesis.** A v1 archive is never converted to v2: it
  records no history, clock, version revisions or collection revisions, and
  these are never synthesised. A v1 archive offered to a native restore mode
  is refused; it still restores by the v1 rule.
- **ustar/pax export is not native recovery.** A converter may export the
  current versions of a v2 archive and stays lossy
  (`services/volume/src/ustar.rs:8-11`); nothing restores from it.

## 6. Deliberately not recorded

- **Wall-clock time:** the source has none.
- **Per-version timestamps:** a `Version` has none (`table.rs:33-43`); none is
  invented. Only the object's `created`/`modified` exist.
- **`request_id`** (objects and collections): a destination-local idempotence
  token (`table.rs:131-133`); copying it would be wrong.
- **Chunk numbers and chunk checksums:** destination allocation is local.
- **The other domain's clock:** only the exporting domain's own clock.
- **Source identity** (volume or domain): none exists in the source, and the
  owner decided on 19 November 2026 that none is recorded (section 3).
- **Authenticity:** FNV-64 detects accidental damage only; a deliberately
  resealed archive with consistent invented values is accepted. "Never
  fabricated" holds relative to archive integrity; authenticity belongs to the
  encryption and key-management contracts (D5) and is out of scope.

## 7. Field use by L1 replay and L2 provenance

| Field | L1 (history replay, client-side) | L2 (immutable provenance, table v3) |
|---|---|---|
| slot, state | target slot; archives with `TOMBSTONES` refused (R2) | target slot; tombstone ⇒ install then DELETE only if a reviewed decision permits `TOMBSTONES` |
| version records (oldest first: content, name, length) | one WRITE per version, in order, for the consecutive-revision prefix only; versions after the first revision gap may predate a DELETE and are skipped (owner decision, section 3) | written by INSTALL |
| version revision | reported as source value only | `origin_revision` per version |
| object revision | reported as source value only | `origin_revision` per object |
| created / modified | reported as source values only | `origin_created` / `origin_modified` |
| tags, kind, collection | META after every WRITE of the archive and the COLLECTION creates | same |
| collection index, name, state | COLLECTION create at the same index (present only) | same |
| collection revision | reported as source value only | `origin_revision` per collection |
| source clock | validation bound (S6–S8) | same; not installed as a live clock |
| domain generation, `ACCOUNTED` | S9 consistency check; without `ACCOUNTED` reported as an unchecked recorded value | same |
| content-free records, deleted collections | not replayed; S9 only | not installed; S9 only |
| `TOMBSTONES` | archive refused before the first write (R2) | permits tombstone install only after a separately reviewed decision |

In both levels destination-assigned live revisions and clocks stay separate
from the recorded source values; L2 stores the latter as immutable fields and
never feeds them into `touch` (`service.rs:603-618`).

## 8. Not done here (later steps)

- Wire extensions (step 2): the own domain clock and history version names are
  not exposed today, so **no writer can fill a v2 archive yet**; the fields
  exist in the format only. Two caveats from the fidelity analysis (3.3)
  apply: exposing a domain's own clock to that domain does not appear to
  contradict the cross-domain property in `service.rs:591-593` (a domain
  cannot observe how many commits another domain makes), but this needs
  review; and the proposed STAT history extension discloses the caller's own
  retained history after a DELETE, which R2 limits to content-free records and needs separate review.
- Replays of 25 to 40 chunks are refused with `Full` even when they would
  fit (section 4.3); lifting that needs a reviewed free-count read.
- Writer (`export` with a v2 selector), native reader dispatch in `walk`, a v2
  range adapter for `DiskSource`, native QEMU stage, and L2 (table v3, INSTALL, STAT extension, recovery and crash tests).
- The destination range size of PR #396 has not been checked against
  966 sectors.

## 9. Evidence (host only)

`tests/test_volume_archive_v2.py` encodes archives from this document (no code
shared with the reader or `volume-core`; the encoder and the reader both use
`fnv` from `tools/volume_disk.py`, a common-mode dependency shared with the v1
tests) and checks: round trip; every single bit flip (8 bits × every byte)
and every truncation prefix and trailing byte refused through `parse_v2` and
`parse_any`; one case per structural rule; every byte of every reserved and
name-padding range (header, present and deleted collection, object, version)
refused one at a time in a resealed archive; a damaged seal of each record
kind with a recomputed trailer checksum; one test per semantic rule S1–S10
with a distinct message, with accepted boundary cases for S5–S8 and S10
(equality accepted) and a three-version case for the S3 chain; positive cases
for the conditional rules (generation check only with `ACCOUNTED`, tombstones
only with `TOMBSTONES`); a source-feasible archive of exactly `CONTENT_V2`
content and `MAX_ARCHIVE_V2` length accepted and `CONTENT_V2 + 1` refused; a
locally built v1 archive through `parse_any` equal to `parse`, including error
messages; v2 refused by the v1 `parse`.

**Mutation pass (manual, not stored in the repository).** A scratch script
copied `tools/` and `tests/` to a temporary directory, applied one textual
mutant at a time to `tools/volume_archive.py` (each replacement asserted to
match exactly once) and ran
`python3 -B -m unittest discover -s tests -p test_volume_archive_v2.py`.
All 26 mutants were killed (non-zero exit): S3 chain
(`newer = newer or version_revision`); S3 `>=` → `>`; S2, S5, S6, S8 and
`CONTENT_V2` `>` → `>=`; S7 removed; header byte 11 check removed; start and
end of header 40..56, collection 3..8 and padding, object 6..8, 12..16 and
40..120, version 1..4 and padding each shifted by one byte; S10 floor instead
of ceiling, `>` → `>=`, sum instead of max, current version only, and check
removed. Removal mutants of the earlier semantic, flag-gating and record-seal
checks were killed in the writer's earlier pass and in the independent verifier's reproduction, and are covered by the same
named tests. The script and its log are not part of the evidence set; the
pass is a test-quality check, not acceptance evidence.
