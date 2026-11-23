# S09 — L2 recorded provenance: table v3 and INSTALL (specification)

**Status: revision 1. The independent AI spec review returned
REQUEST_CHANGES with 10 required items; all are resolved in this revision,
and section 10 records each resolution.** The owner accepted an independent
AI review for this design stage. That review is not a human audit. The
review read revision 0 (sha256 `6b8d4fbd…8978`) at commit `837619ef`. The
resolutions below have not been reviewed again yet. The implementation
review checks them.

**Implementation: on host, with the native volume-docs service built with
the L2 operations at opt-level "s" (section 11, deviation 1).** Four native
INSTALL stages (`install`, `install-recover`, `install-cut`, `recover-cut`)
pass in the local 26-stage gate; their protected pins are pending
(`S09-L2-INSTALL-STAGE-INTEGRATION-REQUEST.md`).

R1 requires review of table v3, crash recovery, compatibility, security and
negative tests before implementation.

References:
- `claude/s09-l2-provenance` at `837619ef`;
- `S09-RESTORE-FIDELITY.md` sections 3.1–3.4 (`claude/s09-destination-service`);
- `S09-ARCHIVE-V2.md` section 7;
- `S09-OWN-DOMAIN-READS.md`.

Constraints:
- Provenance is immutable. It is kept separate from live revisions and clocks, and never feeds `touch`, clocks, CAS or quota.
- L3 is rejected.
- No source identifier is recorded.
- R2, R3 and R4 apply (`MAX_ARCHIVE_V2 = 494 416` is unaffected).
- Values are never fabricated.
- v1 archive, restore and ustar stay unchanged.
- v2 tables keep mounting.

## 1. Revision rule after INSTALL (deviation from fidelity analysis 3.2)

The rule in 3.2 sets the object revision to 1 and the version ordinals to
`n..1`, decoupled from the object revision. It numbers the next version
`1 + max(object revision, newest retained ordinal)`. **That rule breaks the
owner decision R2.**

Example: INSTALL of 3 versions gives object revision 1 and version ordinals
3, 2, 1. DELETE gives object revision 2, with history 3, 2. WRITE then gives
`1 + max(2, 3) = 4`, so the object holds versions 4, 3, 2. These revisions
are consecutive, so `version_stat` (`service.rs:764`), `replayable` and
`export_v2` all treat the pre-delete versions as an unbroken prefix. In v2
that gap exists only because DELETE raises the object revision that the next
version is numbered from.

The 3.2 rule also breaks existing clients. `client::validate_reply` requires
`version <= h.revision` (`client.rs:72-80`), so a STAT of an object installed
with `n ≥ 2` fails with `Protocol`. Archive rule S2 also fails on re-export.
The review confirmed all of this.

**Rule R (selected):** INSTALL of `n` versions sets the object revision to
`n` and the version revisions to `n, n−1, …, 1` (current version first). It
does this in **one** commit, which raises the domain clock and the generation
by exactly one. The following then hold unchanged: every v2 entry invariant
(`Entry::validate`, `table.rs:286-331`), the v2 revision rule
(`current.revision + 1`, `service.rs:562`, `:669`), STAT validation, and the
deletion gap.

Rule R changes what "revision" means. One commit can raise an object
revision by `n`, and version revisions 1 to `n−1` never matched a committed
state of this destination. They are values the destination assigns, not
source values, so 3.4 holds.

**Only the clock bound is relaxed.** In v2, `revision ≤ clock`
(`table.rs:664`) and `revision ≤ generation` (`table.rs:262`) hold because
each commit raises an object revision at most once. INSTALL raises it by `n`
in one commit. Table v3 therefore stores `installed = n` durably, sets
`excess = max(installed, 1) − 1`, and checks:
- `revision ≤ clock + excess`;
- `revision ≤ generation + excess`.

With `installed ≤ 1` these are exactly the v2 bounds.

The bound holds permanently:
- at INSTALL, `n ≤ c + n − 1` for any clock `c ≥ 1`;
- afterwards, each commit raises the clock by one and this object revision
  by at most one (`touch`; TRANSACT indices are distinct, as `wire.rs` validates).

Rejected alternatives:
- the 3.2 rule (breaks R2);
- one commit per version (loses one-commit atomicity);
- raising the clock by `n` (breaks clock sum = generation, or the `g ± 1`
  adjacency, which amounts to L3).

**Owner note:** this deviation is recorded for owner review. It conflicts
with no owner decision. The 3.2 alternative conflicts with R2.

## 2. Table v3 layout (sector budget)

`TABLE_SECTORS = 2 + 2·16 = 34` and `DATA_START` are unchanged; they pin the
data area and the format marker. Table v3 uses only bytes that v2 requires to
be zero, so a v2 copy is a v3 copy whose provenance is all zero.

| Sector | v2 use | v3 additions (little-endian) |
|---|---|---|
| 0 header | 0..56 (`CLOCKS = 40`, 2 clocks) | version field (offset 8) = `3`; 56..120: collection `origin_revision`, one u64 per collection at index `domain·4 + i − 1`; 120..488 zero |
| 1 collections | 8 × 56 B; 448..488 zero | none (40 free bytes cannot hold 8 × u64) |
| `2+2o` head | 0..41, current version 64..280 | 280: `installed` u8 (0..=3); 281..288 zero; 288 `origin_revision`; 296 `origin_created`; 304 `origin_modified`; 312 current version `origin`; 320..488 zero |
| `3+2o` history | 2 × 216 B; 432..488 zero | 432 `history[0].origin`; 440 `history[1].origin`; 448..488 zero |

The 216-byte version record is unchanged. `Version` gains an `origin: u64`
field encoded outside the record. `push_history` and `take_history` move it
with the version (`table.rs:199`).

The magic stays `NOSVOL02`. The format marker (`table.rs:726-735`,
`volume_disk.py:86-97`) still records table version 2. It is decoupled from
the `VERSION` constant and means "claimed by the NOSVOL writer". A first
commit that writes v3 is therefore possible.

### 2.1 v3 invariants

Every decode checks these; any failure is `Corrupt` (fail closed).

1. The header version is 2 or 3. In a **version-2 copy, every v3 byte must
   be zero**: header 56..120, head 280..320, history 432..448. The decoder
   checks this for each format separately. Bytes outside the v3 fields
   (header 120..488, head 281..288, head 320..488, history 448..488) must be
   zero in both versions.
2. `installed ≤ HISTORY + 1`. If `installed = 0`, every object and version
   origin is zero. If `installed ≥ 1`, then `revision ≥ installed`.
3. `origin_revision ≠ 0` if and only if `origin_created ≠ 0`. In that case
   `origin_created ≤ origin_modified` and `installed ≥ 1`. If
   `origin_revision = 0`, then `origin_created = origin_modified = 0`.
4. A version `origin` may be non-zero only for a retained version; empty
   versions carry 0 (they compare equal to `Version::EMPTY` in full). The
   non-zero origins of one entry are pairwise distinct. When the object
   `origin_revision ≠ 0`, each of them is ≤ that value.
5. Clock bounds:
   - `revision ≤ generation + excess` (checked in `decode_head`);
   - `revision ≤ clock + excess` (checked in `Table::validate`).

   All other v2 checks are unchanged: clock sum = generation, `modified ≤ clock`,
   collection revisions ≤ clock, quota, chunk ownership and `reflected`.
6. A collection `origin_revision ≠ 0` implies that the collection revision
   is ≠ 0 **and** the collection is present.

Provenance feeds no other invariant, CAS, quota or stamp. All bounds are
computed without overflow: `clock + excess` saturates at `u64::MAX`, which
equals the comparison in wide arithmetic (an overflowing bound lies above
every u64 revision), so it neither refuses a legitimate table nor accepts
one it should refuse. A v3 copy, or a v3 entry, with every provenance field
zero is valid; `installed ≥ 1` with zero origins is the normal state after
an incarnation ends (section 2.2).

### 2.2 Provenance lifecycle per operation

Provenance values never change; they are only dropped. The meaning of each
column:
- **object** — `origin_revision`, `origin_created`, `origin_modified`;
- **version** — per-version `origin`;
- **installed** — never cleared. It is the durable bound of invariant 5,
  not a provenance claim.

| Operation | object | version | installed |
|---|---|---|---|
| INSTALL_COMMIT (slot revision 0) | set from the manifest | set from the manifest | `n` |
| upload COMMIT, entry present | unchanged | new current version 0; the replaced current moves to history with its origin | unchanged |
| upload COMMIT, entry absent (re-create, after DELETE or PURGE) | **zeroed** | new current version 0; retained pre-delete history versions keep theirs (hidden by the prefix rule) | unchanged |
| RESTORE, entry absent (undelete, same incarnation) | unchanged | the restored version keeps its origin (`..restored`, `service.rs:676`) | unchanged |
| RESTORE, entry present | unchanged | the restored version keeps its origin; the replaced current moves to history | unchanged |
| DELETE, TRANSACT DELETE | unchanged | the current version moves to history with its origin | unchanged |
| PURGE, TRANSACT PURGE, entry present | unchanged | history versions and their origins dropped | unchanged |
| PURGE, TRANSACT PURGE, entry absent | **zeroed** (the incarnation ends) | history dropped | unchanged |
| RENAME (renames the current version in place, `service.rs:667`) | unchanged | unchanged (provenance identifies source content, not the mutable name) | unchanged |
| META, TRANSACT META or RENAME | unchanged | unchanged | unchanged |
| COLLECTION create or rename | collection origin unchanged | — | — |
| COLLECTION delete | collection origin **zeroed** | — | — |

TRANSACT cannot re-create an entry. A new object never inherits an old
origin.

## 3. Mounting, upgrade and A/B recovery

**Choice:** a v3-capable service writes v2 copies, byte-identical to today's
encoder, until the first INSTALL_COMMIT or INSTALL_COLLECTION commit. That
commit writes the first v3 copy. From then on every copy is v3: the format
is sticky and never downgraded. `Table` and `SlotScan` gain a field
`format ∈ {2, 3}`.

Why: volumes that never use L2 stay readable by current binaries, so
rollback works, and oracles, mutants and native stages see unchanged bytes.

Rejected alternatives:
- a separate upgrade commit: a commit without an object or collection change
  violates `reflected` (`table.rs:687-698`);
- upgrading on any first commit: this locks out older code with no L2 benefit.

Reading and recovery:
- **Dual reader.** `decode_sector` dispatches on the header version.
- **Mixed pair.** For two valid copies, `choose` (`table.rs:840`) also
  requires `format(newer) ≥ format(older)`:
  - (v2 `g`, v3 `g+1`) is valid;
  - (v3 `g`, v2 `g+1`) is `Corrupt`;
  - a torn v3 attempt next to v2 `g` recovers `g` unchanged.

  The post-commit admission check (`service.rs:961`) compares `format` and
  the new `Entry` and `Collection` fields.
- **Old code on a v3 volume.** The uniform v3 copy fails the version check,
  so `compatible_with` is false (`table.rs:825-827`) and the mount is
  `Corrupt`. It fails closed and never silently rolls back to the v2 copy
  `g − 1`. This also covers the format marker followed by a v3 copy at
  generation 1, because `first_attempt` requires a non-uniform copy.
- **Downgrade consequence (owner-accepted trade-off, to be linked to the
  update and rollback policy).** After the first INSTALL, rolling back to a
  binary that cannot read v3 makes the volume unmountable (`Corrupt`).
- **Host oracle.** `tools/volume_disk.py` hard-codes version 2 and the
  reserved-zero rules (`:218-262`). It gains an independently written v3
  decoder. Version-2 checking stays selectable as the **frozen v2 reader**
  (`allow_v3=False`, which is the default for existing callers), so a v2-only
  reader stays testable after the Rust and Python decoders learn v3.
  `run_volume.py`, `capture_volume.py` and `PLAN` in `test_volume.py` are
  unchanged by this slice.

## 4. Wire operations (additive within v6 and v7)

The new codes sit above `VERSION_STAT = 18`. An older service refuses them
with `NoSys` before reading any state (`wire.rs:193`). The `VERSION_STAT +
1 → NoSys` anchors (`tests/service.rs:840`, `:1079`) move to the new maximum
plus 1.

| Code | Name | Request (fields not listed are zero) |
|---|---|---|
| 19 | INSTALL_BEGIN | own slot; `revision = 0`; `count = n` (1..=3); `name_length = 0`; `aux = meta(tags, kind, collection)` in canonical form (`wire.rs:96`); `length = 24 + 45n`; data = manifest |
| 20 | INSTALL_APPEND | same slot and `request_id`; `revision = 0`; `count` = version index (≤ `HISTORY`); `aux` = offset within that version; `name_length = 0`; `length` 1..=160 |
| 21 | INSTALL_COMMIT | same slot and `request_id`; `revision = 0`; `count = n`; `aux` = Σ lengths; `name_length = 0`; `length = 0` |
| 5 | ABORT | also cancels an install session (`revision = 0`) |
| 22 | INSTALL_COLLECTION | base object; `aux` = collection index 1..=4; `revision = 0`; `count = 0`; `name_length = k` (1..=`NAME`); `length = k + 8`; data = name (`k` bytes), then `origin_revision` u64 |
| 23 | PROVENANCE_STAT | read-only, v6 and v7. `count = 0`: object, with `revision` = the expected live revision (≠ 0). `count = 1`: base object, with `aux` = collection index and `revision` = the expected collection revision (≠ 0). `length = name_length = 0` |

Operations 19–22 are `mutating` (read-only mode answers `Access`). 23 joins
the non-mutating set.

**Manifest** (at most 24 + 3·45 = 159 bytes, within `DATA = 160`):
- offset 0: `origin_revision`; 8: `origin_created`; 16: `origin_modified`;
- then one 45-byte record per version `i` at offset `24 + 45i`, where index 0
  is the current version, followed by history newest first (the archive
  order). Each record holds, at record offsets: 0 `origin` u64; 8 `length`
  u32 (≤ `MAX_LENGTH`); 12 `name_length` u8; 13..45 name, with zero padding
  after `name_length` bytes.

`validate_request` enforces the shape (`Invalid` on any failure):
- the header fields as in the table above;
- each name is non-empty and passes `valid_name`, with zero padding;
- all origins ≠ 0, and `origin_created ≤ origin_modified`;
- `origin_i = origin_0 − i` (consecutive: the service refuses history behind
  a revision gap, as R2 defence in depth);
- `origin_0 ≤ origin_revision` and `origin_0 ≥ n`;
- Σ lengths ≤ `3 · MAX_LENGTH`, computed without overflow.

Tombstones cannot be expressed, because an installed object is always
present (R2).

**INSTALL_COLLECTION validation:**
- the generic `named()` helper is not used (it requires `name_length ==
  length`);
- a dedicated arm checks `length = name_length + 8`, `valid_name(data[..k])`
  and `origin_revision = get64(data, k) ≠ 0`. A zero origin is `Invalid`,
  because it would otherwise look like an ordinary COLLECTION create.

### 4.1 Session matrix

There is at most one staged session per domain. `sessions[DOMAINS]` becomes
`[Option<Staged>; DOMAINS]` with `enum Staged { Upload(Session),
Install(InstallSession) }`.

**`begin()` checks the kind of the staged session before any `take()`.**
Only an untokened *upload* may be replaced, and only by a v6 BEGIN (the
legacy rule).

| Staged session | v6 BEGIN | v6 INSTALL_BEGIN | v7 BEGIN or INSTALL_BEGIN | v6 APPEND, COMMIT, ABORT | v6 INSTALL_APPEND, INSTALL_COMMIT | v7 continuation ops |
|---|---|---|---|---|---|---|
| none | stage upload | stage install | stage, token issued | `Invalid` | `Invalid` | `Access` |
| upload, untokened | replace (legacy) | `Busy` | `Busy` | continue if matching | `Invalid` (kind) | `Access` |
| upload, tokened | `Busy` | `Busy` | `Busy` | `Busy` | `Busy` | continue only with matching principal and token |
| install, untokened | `Busy` | `Busy` | `Busy` | APPEND, COMMIT: `Invalid` (kind); ABORT: continue if matching | continue if matching | `Access` |
| install, tokened | `Busy` | `Busy` | `Busy` | `Busy` | `Busy` | continue only with matching principal and token |

An abandoned untokened install blocks new sessions in its domain until a
matching ABORT, SHUTDOWN or service restart. Staged sessions were never
ACKed, so nothing durable is lost.

**Code paths that must change:**
1. `service.rs:178-187`: the v6 legacy `Busy` guard covers BEGIN, APPEND,
   COMMIT, ABORT, **INSTALL_BEGIN, INSTALL_APPEND and INSTALL_COMMIT** while
   the staged session is tokened.
2. `service.rs:224-252`: `handle_for` refuses BEGIN and INSTALL_BEGIN with
   `Busy` while any session is staged. On a successful INSTALL_BEGIN it
   issues the token and binds the principal. It checks principal and token
   for APPEND, COMMIT, ABORT, INSTALL_APPEND and INSTALL_COMMIT.
3. `upload.rs:43`: INSTALL_APPEND and INSTALL_COMMIT join the operations
   that require a token.
4. `upload.rs:59`: a token is required on a successful INSTALL_BEGIN reply
   as well as on BEGIN.

Install tokens come from the same per-domain `next_upload_token` counter, so
a token is never reused within a service instance. `matching()` also checks
the session kind. SHUTDOWN drops install sessions, and `detach` drops
tokened install sessions of the principal, exactly as it does for uploads.

### 4.2 Install session state

`InstallSession` contains:
- `principal`, `upload_token`, `object`, `request_id`;
- `n`, `tags`, `kind`, `collection`, and the three object origins;
- per version (`[_; 3]`): `origin`, `length`, `received`, `name`,
  `name_length`, `content_sum`, `chunk_start` and `chunk_count`;
- one shared `chunks: [u16; 48]` and `sums: [u64; 48]` (3 × `MAX_CHUNKS`);
- **one** 512-byte `sector` buffer.

**Partial-sector rule:** the APPEND that completes version `i` writes that
version's last partial sector, zero-padded, before it replies. One buffer is
therefore enough. This is safe because the chunks are unreferenced and
reserved, and an `Io` there is a certain abort: the session is dropped and
the reply is `Io`.

A **zero-length version** (`chunks_for(0) = 0`) is complete at
INSTALL_BEGIN: `received = length = 0` and `content_sum = FNV_OFFSET`. No
APPEND targets it, because the target is always the lowest-index incomplete
version.

Size: the tagged enum grows `Service` by about 1.1 KiB per domain.
`size_of::<Staged>()` (≤ 1600 B) and `size_of::<Service>()` (≤ 24 KiB) are
pinned by a host test (`staged_session_size_is_pinned`). The native
volume-docs service includes `Staged::Install` (deviation 1 in section 11).
The native service keeps one `Service` on its 64 KiB private stack: recovery
runs in its own non-inlined frame, `_start` borrows the service inside the
recovery `Result` instead of moving it out, and the serving loop has its own
frame. There is no separate static-size check.

### 4.3 INSTALL_BEGIN, INSTALL_APPEND and INSTALL_COMMIT

**INSTALL_BEGIN**, checked in this order:
- session matrix (4.1);
- entry revision must be 0, otherwise `Stale` (a purged slot has revision ≠
  0 and is refused);
- `collection` must be 0 or present, otherwise `Range`, before anything is
  reserved;
- `domain_chunks + Σ chunks_for(length_i) ≤ QUOTA`, otherwise `Quota`.

Then it takes fresh chunks that are unreferenced and unreserved (lowest
index first, never deduplicated, `Full` if they run out); each version
starts at its own chunk. `reserved()` covers both session kinds, and staged
chunks of another domain are never handed out. Finally it calls
`ensure_format_initialized`.

Reply: `revision = 0`, `aux` = Σ lengths, `count` = Σ chunks, plus the
issued token on v7.

**INSTALL_APPEND:**
- `count` must equal the target version (the lowest-index incomplete one),
  otherwise `Range`;
- `aux` must equal that version's `received`, otherwise `Range`;
- no write past the version's length (`Range`);
- an `Io` on a content write is a certain abort.

Reply: `revision = 0`, `count` = the request's `count`, `aux` = the new
`received` of that version.

**INSTALL_COMMIT:**
- re-checks that the entry revision is 0 (`Stale`) and that `collection` is
  0 or present (`Range`);
- `count = n` and `aux` = Σ lengths, otherwise `Invalid`;
- `Range` unless every version is complete;
- then FLUSH and **one** `publish` (`service.rs:914`).

The published entry contains:
- revision `n` and the `request_id`;
- `created = modified =` the new own clock;
- present, with the tags, kind and collection;
- `installed = n`;
- versions with revisions `n..1`, carrying the manifest's names, lengths,
  chunk sums and origins;
- the object origins from the manifest;
- `patch.format = 3`.

INSTALL_COMMIT does not call `touch`. Reply: `revision = n`, `aux` = Σ
lengths. A failure before the table write is `Io`: durable state is
unchanged and the chunks are released. A failure during or after the write
is `Unknown`.

**INSTALL_COLLECTION:**
- the collection revision must be 0, otherwise `Stale`;
- one publish: revision 1, present, the name, the `request_id`, the origin,
  `format = 3`, and the clock raised by 1.

Reply: `revision = 1`, `aux` = the index.

### 4.4 PROVENANCE_STAT reply

**Object:**
- `revision` = the live revision; `count` = `installed`;
- if `installed ≠ 0`, the data holds the three object origins (24 B; zero
  once the incarnation has ended);
- **for a present entry only**, the data continues with the origins of the
  versions in `version_stat`'s consecutive-revision prefix (current version
  first; 0 where a version has none), and `name_length` = the number of
  these version origins (≤ `HISTORY + 1`);
- an absent entry returns the object origins only (the R2 mirror of
  selector 0).

**Collection:** `count` = 1 if `origin_revision ≠ 0`; `aux` = the origin;
`length = 0`.

Errors are as for VERSION_STAT (`NotFound`, `Stale`).

Disclosure note (R2): `count` = `installed` keeps reporting the version count
of the original INSTALL after that incarnation has ended (re-create or
PURGE), while the object origins are zero. This is own-domain metadata, not
content, and reveals no deleted content or version origin.

STAT, VERSION_STAT and COLLECTION_STAT replies are **unchanged**. Existing
clients validate their lengths strictly (`client.rs:58-80`), so extending
them would break those clients. Provenance is therefore read through
PROVENANCE_STAT, not through the STAT and COLLECTION_STAT extensions that
3.3 proposed.

### 4.5 Client reply validation

New `client::validate_reply` arms:
- **INSTALL_BEGIN:** `revision = 0`; `aux` = request Σ lengths; `count` = Σ
  `chunks_for(length_i)`; bare.
- **INSTALL_APPEND:** `revision = 0`; `count` = request `count`; `aux` =
  request `aux` + request `length`; bare.
- **INSTALL_COMMIT:** `revision` = request `count`; `aux` = request `aux`;
  `count = 0`; bare.
- **INSTALL_COLLECTION:** `revision = 1`; `aux` = request `aux`;
  `count = 0`; bare.
- **PROVENANCE_STAT, object:** `revision` = request `revision`;
  `count ≤ 3`.
  - `count = 0`: `length = name_length = 0`.
  - `count ≥ 1`: `length = 24 + 8 · name_length` and `name_length ≤ 3`.
  - The three object origins obey invariant 3.
- **PROVENANCE_STAT, collection:** `revision` = request `revision`;
  `length = name_length = 0`; `count ≤ 1`; `(count = 1) ⇔ (aux ≠ 0)`.

## 5. Restore mode L2 (`restore_native_l2`, client, `archive.rs`)

This is a separate function beside `restore` (v1) and `restore_native` (L1).
Before the first write it checks, in this order:
1. `verify_v2`: a v1 or corrupt archive gives `Corrupt`;
2. `TOMBSTONES` gives `Invalid` (R2);
3. `never_used`, otherwise `Stale` (R3);
4. capacity: Σ over replayable versions of `chunks_for(length)` must be ≤
   `QUOTA` (else `Quota`) and ≤ `GUARANTEED_FREE` (else `Full`).

It then runs INSTALL_COLLECTION for each archived present collection
(deleted collections are never installed), followed by one INSTALL session
per present object over its `replayable` prefix only. Versions after the
first gap are counted in `skipped`.

Content streams from a fresh `open()` in archive order. Before
INSTALL_COMMIT, the client recomputes each version's FNV-64 against the
verified plan, and ABORTs on a mismatch (`Corrupt`). An old service answers
the first INSTALL operation with `NoSys`, and nothing has been committed.

**Values are copied, never derived:**
- object `origin_*` = the archive object revision, `created` and `modified`;
- version `origin` = the archive version revision;
- collection origin = the archive collection revision.

The source clock and generation are validation inputs only. The report keeps
source and destination values in separate fields. The service cannot verify
the source: provenance is **asserted by the own domain**, without authority
or authenticity (FNV-64 is not authentication; 3.4).

**R3 enforcement boundary.** The client alone enforces R3 at domain level,
through `never_used`. The service enforces only "slot revision 0" (or
"collection revision 0"), so any app in the domain can INSTALL into a free
slot of a used domain. This mirrors L1 `restore_native`. It is not a
security boundary between apps of one domain, which are one trust domain.

**No resume (review item 1).** After the first INSTALL_COLLECTION or
INSTALL, the domain is used, and `never_used` refuses a rerun with `Stale`.
An interrupted L2 restore is therefore repeated into a fresh, never-used
domain, consistent with L1 today. A resume entry point needs the ownership
control that 3.2 leaves open (an exclusive grant held across the
interruption, or a durable restore-ownership record) and an explicit owner
ruling. It is out of scope. `Unknown` stops the restore. The report names
the committed slots and collections, so the user can discard the partial
domain.

**Re-export (review item 5, bounded rule).** v2 has no provenance field. An
installed domain may be exported as v2 whenever its live state satisfies the
v2 archive rules. The archive then records live (destination) revisions and
stamps; provenance is not carried until a provenance-carrying archive is
decided, and the report says so. Under rule R, S7 (`revision ≤ clock`) can
fail for a while after an INSTALL. That window closes once the domain has
made `n − 1` further commits.

`export_v2` distinguishes the two causes of `o.revision > clock`:
- inside the snapshot (between the first and the final DOMAIN_STAT, which
  detects concurrent change), it issues PROVENANCE_STAT for that object;
- if `installed ≥ 1` and `o.revision ≤ clock + installed − 1`, it returns
  **`Range`** ("not exportable as v2 at this clock"). This is distinct from
  `Stale` and is not retried automatically. The user can export v1, or make
  further commits;
- otherwise it returns `Stale` (concurrent change), as today. An older
  service answers PROVENANCE_STAT with `NoSys`, which is read as
  `installed = 0` and so gives `Stale`.

This refusal is a format limit, not a security boundary. Already-deployed
`export_v2` clients return `Stale` in that window. v1 export is unchanged and
is tested on an installed domain.

## 6. Security invariants

1. **Own domain only.** The domain comes from the kernel-bound ingress, and
   selectors are checked against it before any state is read (`wire.rs:199-204`).
   On v7, INSTALL is also bound to the principal and the token.
2. **Never-used slot or collection only** (revision 0). Nothing is
   overwritten or merged, and tombstones are never installed.
3. **Provenance never influences** `touch`, clocks, stamps, CAS, quota or
   chunk allocation. Live values are assigned by the destination only.
4. **Cross-domain stamp property.** INSTALL advances only the caller's clock.
   Other domains' stamps, DOMAIN_STAT and PROVENANCE_STAT are unchanged
   (`tests/history.rs:391`).
5. **Disclosure.** PROVENANCE_STAT returns only values the own domain
   installed, under VERSION_STAT's presence and prefix rule. No source
   identifier exists anywhere. INSTALL grants nothing beyond WRITE and META
   in the own domain, except caller-asserted provenance about itself.
6. **Untrusted lengths.** Every manifest length, name length and offset is
   bounded before use. Σ is computed with checked arithmetic, and APPEND
   offsets are compared against session state, never trusted.

## 7. Crash and interruption semantics

Every commit follows the same order:
1. content is written into unreferenced, reserved chunks (the last partial
   sector at the completing APPEND);
2. FLUSH;
3. one table copy is written into slot `(g+1) % 2`;
4. FLUSH.

The outcome depends on where the cut falls:
- **before the table write:** the volume stays at `g` (slot revision 0,
  format unchanged, chunks unreferenced);
- **during the table write (torn):** it recovers `g`, or `g+1` if every
  sector landed;
- **after the table write:** `g+1` holds the complete object.

A partial INSTALL is impossible. The first INSTALL is also the v2 → v3
upgrade commit, with the same semantics.

## 8. Required tests

Each new check gets a killing mutant in `tools/mutate_volume.py`, run with a
separate `CARGO_TARGET_DIR`.

1. **`validate_request`:** one case per manifest rule and per field of
   operations 19–23. This includes a foreign slot (`Access`), `revision ≠ 0`,
   `name_length ≠ 0`, non-zero name padding, `count > HISTORY` on
   INSTALL_APPEND, Σ overflow, a zero INSTALL_COLLECTION origin, and
   `length ≠ k + 8`.
2. **Sessions:** every cell of the session matrix (4.1), both tokened and
   untokened. Also:
   - read-only mode refuses 19–22 and allows 23;
   - the first commit on a marked-empty volume is an INSTALL with `n = 3` at
     generation 1, so `revision = generation + 2` lies exactly on the bound.
   - mutants: excess computed as `n` instead of `n − 1`, and excess applied
     to an entry with `installed = 0`.
3. **INSTALL_BEGIN refusals:**
   - present, tombstoned and purged slots give `Stale`;
   - an absent collection gives `Range` with no reservation;
   - `Quota` versus `Full` with Σ > 16 chunks for one object (3 × 13 = 39
     passes, 3 × 14 = 42 gives `Quota`);
   - domain B's staged chunks are never handed to domain A's INSTALL.

   Continuation refusals:
   - APPEND out of order, overrun, wrong `count`;
   - COMMIT before completion, `count` or `aux` mismatch, a COMMIT of an
     install session, an INSTALL_COMMIT of an upload session;
   - an absent collection at INSTALL_COMMIT gives `Range` with no table
     write;
   - a zero-length version.
4. **Normal use after INSTALL:**
   - STAT through the unchanged client (`validate_reply` accepts);
   - revisions `n..1`, stamps equal to the destination clock, clock + 1;
   - READ of every version (read cache included);
   - then WRITE, RESTORE of each history slot (origins travel), RENAME,
     META.
5. **Lifecycle:** one test per row of the table in 2.2. Include INSTALL →
   DELETE → COMMIT (re-create) → RESTORE of an installed-era history
   version: the version origin travels while the object origins are zero.
   Also cover INSTALL → DELETE → RESTORE (undelete) together with
   PROVENANCE_STAT's gap behaviour, and TRANSACT DELETE and PURGE.
   `installed` is kept throughout.
6. **Cross-domain:** INSTALLs in one domain leave the other domain's stamps,
   DOMAIN_STAT and PROVENANCE_STAT unchanged. Foreign selectors give
   `Access`.
7. **Crash (`tests/crash.rs`):**
   - cut before the table write, during it (each sector) and after it, for
     INSTALL, INSTALL_COLLECTION, INSTALL then WRITE, and INSTALL then
     RESTORE; a fresh-instance mount validates v3 and finds the old or the
     new state;
   - `Io` on an INSTALL_APPEND content write, on the completing APPEND's
     partial-sector write, and on the INSTALL_COMMIT FLUSH: certain abort,
     chunks released, no table write;
   - SHUTDOWN and `detach` with a staged install, tokened and untokened.
8. **v2 compatibility:**
   - v2 fixtures mount;
   - every non-install commit of a v3-capable service writes copies
     byte-identical to the current encoder;
   - the first INSTALL writes v3, and later commits stay v3.
9. **Mixed A/B with the frozen v2 reader** (the `volume_disk.py` v2 mode):
   - the v3-capable decoder: (v2 `g`, v3 `g+1`) → v3; (v3 `g`, v2 `g+1`) →
     `Corrupt`; torn v3 next to v2 → v2;
   - the frozen v2 reader returns `Corrupt`, never `g − 1`, on (v2 `g−1`,
     v3 `g`), on the marker followed by v3 at generation 1, and on (v3 `g`,
     torn v3 `g+1`).
10. **Corrupt provenance refused on mount:** one case per invariant in 2.1.
    This includes non-zero v3 bytes in a version-2 header, head and history;
    non-zero bytes in 320..488; `installed = 4`; and `installed = 0`
    together with `revision > clock`.
11. **`restore_native_l2`:** a v1 archive, `TOMBSTONES`, a used domain,
    capacity, a `NoSys` service, a streamed checksum mismatch, skipped
    versions after a gap, report labelling, and a rerun after interruption
    (`Stale`, with completion into a fresh domain). `Unknown` stops the
    restore.
12. **Export:**
    - `export_v2` on an installed domain inside the excess window gives
      `Range`; after `n − 1` further commits it exports live values;
    - v1 export of an installed domain;
    - the v1 restore and ustar suites are unchanged and passing;
    - the existing `revision-clock-bound`, `modified-clock-bound` and
      `choose-unrelated-generations` mutant anchors change and are
      re-anchored.

## 9. Open items

- **Owner review:** rule R versus the 3.2 rule (section 1); the downgrade
  consequence (section 3).
- **Out of scope:** the ownership control and resume (3.2); a
  provenance-carrying archive.
- **Native stage:** the `volume_disk.py` v3 decoder is in scope (section
  3). The four native INSTALL stages are implemented; the protected pins
  (22 → 26) wait on `S09-L2-INSTALL-STAGE-INTEGRATION-REQUEST.md`.

## 10. Review record

**Review 1:** an independent AI spec review (read-only) of revision 0 at
`837619ef`. Verdict: REQUEST_CHANGES. The full text is in
[SPEC-REVIEW-1.md](../verification/s09-l2-provenance-20261123/SPEC-REVIEW-1.md). It verified the following:
- the layout uses only bytes v2 requires to be zero;
- an old reader fails closed on v3;
- the R2 failure of the 3.2 rule and the STAT incompatibility;
- the soundness of the rule R bound;
- the origin move under RESTORE;
- that a purged slot keeps revision ≠ 0;
- the order of checks before state is read in `wire.rs`;
- the `GUARANTEED_FREE` reasoning.

How each required item is resolved:

| # | Required change | Resolution in revision 1 |
|---|---|---|
| 1 | Resume contradicts R3 `never_used` | Resume dropped; an interrupted restore is repeated into a fresh domain (section 5) |
| 2 | Contradictory v6 session rules | One session matrix; `begin()` checks the kind before `take()` (4.1) |
| 3 | Name every token and principal code path | Four paths named; shared token counter (4.1) |
| 4 | Install session state and partial sectors | `InstallSession` layout, write at the completing APPEND, zero-length versions, size pinned (4.2) |
| 5 | Permanent v2-export loss is a same-domain DoS | Replaced by a bounded rule: `Range` only inside the excess window, through PROVENANCE_STAT within the snapshot; not a security boundary (section 5) |
| 6 | "Re-create" undefined per operation | Per-operation table and one test per row (2.2, test 5) |
| 7 | INSTALL_COLLECTION zero origin, invariant 6, data layout | Origin ≠ 0 (`Invalid`); invariant 6 requires present; dedicated arm instead of `named()` (2.1, section 4) |
| 8 | Remaining wire shape checks and reply arms | Header and manifest checks, collection check at BEGIN, client arms (sections 4, 4.3, 4.5) |
| 9 | Downgrade tests not possible as written | Frozen v2 reader in `volume_disk.py` (section 3, test 9) |
| 10 | Missing negative and crash tests | Added to tests 1–3, 5, 7, 10 and 12 |

The optional items were taken over too: rule R is recommended and its change
in meaning stated (section 1); `revision ≥ installed` is checked
(invariant 2); R3 is enforced only by the client (section 5); the downgrade
consequence (section 3); `SlotScan.format` and the admission check
(section 3).

## 11. Implementation status and deviations

**Status: implemented on host** on `claude/s09-l2-provenance` (uncommitted
at the time of writing). Not yet reviewed against this specification; the
implementation review is open. A host test, a passing mutant run and an AI
review are not native acceptance and not a human audit.

Implemented:
- table v3 decode, encode, sticky format, mixed-pair choice and admission
  (`table.rs`, `tests/table_v3.rs`);
- operations 19–23 (`wire.rs`, `upload.rs`, `service.rs`) with the session
  matrix of 4.1 and the shared v7 token counter;
- client methods `install`, `install_begin`, `install_stream`,
  `install_commit`, `install_collection`, `provenance` and
  `collection_provenance` with the reply arms of 4.5 (`client.rs`);
- `export_v2` with the bounded `Range` rule and `restore_native_l2`
  (`archive.rs`);
- the independent v3 decoder and the frozen v2 reader in
  `tools/volume_disk.py` (`allow_v3=False` by default);
- killing mutants in `tools/mutate_volume.py`.

Test map (section 8):

| Test | Location |
|---|---|
| 1 | `tests/install.rs::install_requests_are_validated_field_by_field`, `provenance_reply_validation` |
| 2 | `session_matrix_every_cell`, `install_tokens_share_the_upload_counter_and_are_required_in_v7`, `read_only_refuses_install_operations_and_allows_provenance_stat`, `first_commit_installs_three_versions_exactly_on_the_rule_r_bound`, `staged_session_size_is_pinned`; mutants `excess-n`, `excess-without-install` |
| 3 | `install_begin_refuses_used_slots_absent_collections_and_capacity`, `service.rs::install_commit_tests` (INSTALL_COMMIT `Stale` re-check), `staged_chunks_of_another_domain_are_never_handed_out`, `continuation_refusals_and_zero_length_versions`, `absent_collection_at_commit_is_range_without_a_table_write` |
| 4 | `normal_use_after_install_and_provenance_travel_with_versions` |
| 5 | `lifecycle_*`, `collection_install_provenance_and_delete` |
| 6 | `installs_leave_the_other_domain_unobservable_and_foreign_selectors_are_refused`, `tests/l2_restore.rs::restore_leaves_the_other_domain_alone` |
| 7 | `tests/crash.rs::every_install_crash_point_recovers_to_before_or_after` (INSTALL, INSTALL_COLLECTION, INSTALL then WRITE, INSTALL then RESTORE; every mutating operation cut, torn and reordered images, fresh-instance mount); `io_during_install_is_a_certain_abort_and_releases_the_chunks`; `shutdown_and_detach_drop_staged_installs` |
| 8 | `tests/table_v3.rs` (v2 golden bytes, first v3 copy, later copies stay v3); the existing v2 suites |
| 9 | `tests/table_v3.rs` (Rust mixed pairs); `tests/test_volume_disk_v3.py` (Python v3 decoder and frozen v2 reader) |
| 10 | `tests/table_v3.rs::corrupt_provenance_fields_are_refused_on_mount`, `nonzero_reserved_and_v2_provenance_bytes_are_refused`; `tests/test_volume_disk_v3.py::CorruptProvenance` |
| 11 | `tests/l2_restore.rs`; the streamed mismatch flips a byte inside a history version's content (`a_content_mismatch_on_the_second_read_aborts_and_stops`, one ABORT observed), and `a_different_archive_on_the_second_read_installs_nothing` serves another valid archive on the second read |
| 2.1 boundary | `tests/table_v3.rs::rule_r_bound_is_computed_without_overflow_near_u64_max`, `revision_u64_max_on_a_crafted_image_is_full_without_a_write`, `unknown_table_versions_are_refused_on_a_plain_copy` (also the zero-provenance v3 mount control) |
| 12 | `export_v2_refuses_with_range_inside_the_excess_window_then_exports_live_values`, `v1_export_of_an_installed_domain`; v1 restore and ustar suites unchanged; re-anchored mutants `revision-clock-bound`, `modified-clock-bound`, `version-stat-prefix`, `oracle-revision-clock-bound` (`choose-unrelated-generations` kept its anchor) |

Deviations and additions (for the implementation review):
1. **Native service builds with L2 at opt-level "s" (owner decision,
   23 November 2026).** With the L2 operations linked in at opt-level 3, the
   native volume-docs service exceeds its 64 KiB `.text` bound (link error
   "app text exceeds startup boundary"). The project owner chose a
   size-optimised build over leaving L2 out natively.
   `sdk/examples/volume-docs/build.py` therefore defaults
   `CARGO_PROFILE_RELEASE_OPT_LEVEL` to `s` for every volume-docs build
   (service and app), and volume-docs uses the volume-core default features,
   including `l2-install`. Measured volume-service `.text`: 49 701 bytes with
   L2 at `s`; 64 021 bytes without L2 at opt-level 3; L2 at opt-level 3 does
   not link. The `l2-install` feature remains: the NAS native crates
   (`services/nas/native-probe`, `services/nas/native-volume`) build
   volume-core with `default-features = false`, so their `nas-volume`
   binary answers 19–23 with `NoSys` and writes v2 tables only.
   Stack: with L2 linked in, the native `archive-restore` stage first
   faulted in the service (`VOLUME:FAIL:UNEXPECTED_FAULT`, slot 0), while
   the same stage passed without L2 at opt-level 3 and at `s`. The `_start`
   frame held two copies of `Service` (about 42 KiB of the 64 KiB stack),
   and `Staged::Install` adds about 1.1 KiB per domain. The service binary
   (`sdk/examples/volume-docs/src/service.rs`, shared with the NAS
   `nas-volume` binary) now keeps a single copy, which reduces the `_start`
   frame to about 20.5 KiB.
2. **INSTALL_BEGIN reply.** `client::validate_reply` sees only the request
   header, not the manifest, so it checks the reply structurally
   (`revision 0`, no data, `aux ≤ 3·MAX_LENGTH`, `count ≤ 48`). The exact
   Σ length and Σ chunks are checked by `client::install_begun(manifest,
   reply)`, which `Client::install_begin` calls.
3. **Report.** `restore_native_l2` returns `L2Report { restored,
   collections, committed, skipped, revisions }`; `revisions` holds the live
   revision per slot, separate from the archive values in the verified plan.
4. **No-feature test targets.** `tests/install.rs` and `tests/l2_restore.rs`
   require the `l2-install` feature, and the L2 crash scenario is compiled
   only with it.
5. **Python v3 decoder.** It records provenance in extra dictionary keys
   (`installed`, `origin`, `origins`; collection `origin`; table `format`)
   only for v3 copies, so existing v2 callers see unchanged structures.
   There is no cross-implementation fixture yet: the Python v3 decoder is
   tested against its own encoder, the Rust decoder against the Rust
   encoder, and both against the byte offsets of section 2.
6. **Mutants without a mutant of their own.** `v.origin != 0` in the
   manifest rule is implied by the consecutive rule together with
   `first ≥ n`, so its mutant is equivalent and was removed. The
   per-version FNV-64 comparison in `restore_native_l2` repeats the check
   `walk_v2` makes of every version against its record before the callback;
   the plan comparison binds the record. Removing the checksum alone is
   therefore equivalent, and mutant `l2-restore-second-read` removes both.
   The INSTALL_COMMIT `Stale` re-check is unreachable through the wire (no
   operation can use the slot while an install is staged) and is killed by
   a unit test in `service.rs`. Further equivalent mutants found by the
   implementation review (no mutant of their own):
   - the v2 history reserved-origin check in `table.rs` decode: in a v2 copy
     `installed = 0`, so `validate_provenance` already refuses any nonzero
     origin, and empty versions are compared with `Version::EMPTY` in full;
   - `first ≥ n` in the manifest rule: the dual of the `v.origin ≠ 0` case
     above. Consecutive origins with `origin ≠ 0` imply `first ≥ n` and the
     reverse, so the two checks are redundant with each other and only one
     can carry a killing mutant;
   - the `Manifest::valid` precheck in `restore_native_l2`: the `walk_v2`
     record rules (created ≥ 1, created ≤ modified, valid names,
     length ≤ MAX_LENGTH, a strictly decreasing chain, first version ≤ the
     object revision) already imply it;
   - `<` → `<=` on the `export_v2` excess window: the boundary
     `revision = clock + installed` breaks the table bound, so an honest
     service cannot produce it;
   - the `decode_head` bound `revision ≤ generation + excess` loosened (for
     example `excess = n`, review mutant M7b): `Table::validate` always runs
     after decode and checks `revision ≤ clock + excess`, and the clocks sum
     to the generation without overflow, so each clock ≤ generation and the
     clock bound is at least as strict. The decode bound is kept as defence
     in depth; its overflow semantics are pinned by `head-bound-fails-closed`
     and `head-bound-wraps` (both killed by the u64::MAX boundary test). The
     v2 history reserved-origin check above is review mutant M4.
7. **Mutant run history.** The first run (66 mutants) reported 6
   survivors and exit 1: `install-begin-stale` (the test only used the
   client, whose commit re-check also answers `Stale`; it now checks the raw
   INSTALL_BEGIN), `install-commit-stale` (unit test added),
   `manifest-origin-nonzero` (equivalent, removed), `l2-restore-checksum`
   (equivalent, replaced as above), `oracle-installed-max` (the case also
   broke `revision ≥ installed`; fixed) and `oracle-excess-n` (the
   generation bound is implied by the domain clock bound; both now share one
   `excess()` helper). A second run failed its baseline because
   `tests/l2_restore.rs` resolved fixtures through `CARGO_MANIFEST_DIR`,
   which a reused target directory baked to an earlier scratch copy; the
   fixtures are now read relative to the package directory.
   The implementation review then found four non-equivalent survivors and
   they gained tests and mutants: `provenance-object-stale`
   (`object_provenance_refuses_a_wrong_revision_and_a_never_used_slot`),
   `provenance-reply-versions` and `provenance-reply-origin-shape` (forged
   replies in `provenance_reply_validation`), and `install-append-received`
   (a misreported received offset in the same test).
8. **Slice A review fold-in (F1–F6).** F1: the 2.2 lifecycle sites
   (COLLECTION delete zeroes the collection origin; COMMIT re-creating an
   absent entry and PURGE or TRANSACT PURGE of an absent entry call
   `end_incarnation`) are covered by `lifecycle_*` and
   `collection_install_provenance_and_delete` (which remounts the v3 copy
   after the delete), with mutants `collection-delete-keeps-origin`,
   `recreate-keeps-origin` and `purge-keeps-origin`. F2: every new
   `table.rs` check has a mutant (`table-unknown-version-as-v2/-v3`,
   `table-v2/-v3-header-reserved`, `table-v2/-v3-head-reserved`,
   `table-head-installed-padding`, `table-v3-history-reserved`,
   `choose-format-sticky`, `provenance-*` for invariants 2–4,
   `collection-origin-present` for invariant 6, `collection-origin-carry`
   for the sector-1 carry). F3: format 0, 1, 4 and `u16::MAX` on a copy
   without provenance are refused. F4: WRITE commit, RESTORE and
   COLLECTION compute the next revision with `checked_add` and answer
   `Full` (what `touch` and `next_generation` answer for exhausted
   counters) before anything is published; a crafted v3 image with
   revision `u64::MAX` (rule R) panicked the dev profile before. The
   COLLECTION site is a pre-existing v2 case (collection revision ≤ clock =
   generation = `u64::MAX`) fixed with it. Mutants
   `commit-/restore-/collection-revision-overflow`. F5: the bounds keep
   `saturating_add` (2.1 amended); mutants `rule-r-bound-fails-closed`,
   `rule-r-bound-wraps`, `head-bound-fails-closed`, `head-bound-wraps`.
   F6: the service upgrade through `Patch.format = 3` is covered by the
   INSTALL and INSTALL_COLLECTION tests (`install-commit-v2`,
   `install-collection-v2`, `format-not-adopted`); a v3 copy with zero
   provenance mounts (table control and the collection delete test). The
   crash suite gained `install_collection_upgrade_crash_points_recover_to_before_or_after`,
   where INSTALL_COLLECTION is the v2 to v3 upgrade commit (outside the
   mutant targets). The native volume-service `.text` grew from 63973 to
   64021 bytes (bound 65536).
