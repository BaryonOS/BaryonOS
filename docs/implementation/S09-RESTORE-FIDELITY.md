# S09 — Restore fidelity: analysis and bounded recommendation

**Status:** analysis and recommendation only (19 November 2026). Nothing in
this document is implemented or tested. It changes no code, no existing
specification, no restore semantics and no capability. Format v1 and its
restore rule in [S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md) remain the only
implemented recovery path. No full-fidelity recovery is claimed.

**Binding inputs:** ADR-S09-DESTINATION-20261119 (D1–D6), ADR-S09-USTAR-20261114,
[LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md),
[S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md),
[S09-BACKUP-DESTINATION-PROPOSAL.md](S09-BACKUP-DESTINATION-PROPOSAL.md).

**Owner intent (verbatim):** "preserve source revision history, metadata and
relevant timestamps when the archive format and verified source state can
support them. Never fabricate original revision identifiers or timestamps.
Distinguish native recovery from the intentionally lossy POSIX ustar/pax
interchange export."

Line references are to `main` at `0b0ed91a`.

## 1. Current state (from the code)

**Source volume model (table v2).** An object `Entry` holds an object
revision, a `request_id`, logical `created`/`modified` stamps, tags, kind,
collection, a presence flag, the current `Version` and up to `HISTORY = 2`
earlier versions, newest first (`services/volume/src/table.rs:127-146`,
`services/volume/src/lib.rs:43-44`). A `Version` records its own revision,
length, name, content FNV-64 and chunk list, but **no timestamp**
(`table.rs:31-43`). `created`/`modified` are values of the owning domain's
**logical commit clock**, not wall-clock time (`table.rs:134-136`,
`table.rs:402-406`); every mutation increments the object revision by exactly
one and stamps the destination domain clock (`service.rs:599-619`). A deleted
object keeps its content as history (a tombstone) until PURGE
(`service.rs:682-696`). A collection holds a revision, `request_id`, presence
flag and name (`table.rs:334-341`). No wall-clock time exists anywhere in the
volume.

**Archive v1 records** (`services/volume/src/archive.rs:141-218`; spec table
`S09-BACKUP-ARCHIVE.md:41-47`; independent reader `tools/volume_archive.py:43-97`):

- Header (64 B): magic, version 1, object count (≤ 8), collection count (≤ 4),
  `generation` at offset 16 = LIST's domain generation, i.e. the **sum of the
  domain's object and collection revisions** (`table.rs:465-474`,
  `service.rs:1032-1035`, `S09-BACKUP-ARCHIVE.md:163-165`), total content at
  offset 24, FNV-64 seal.
- Collection record (48 B): index, name length, name. No revision
  (`archive.rs:154-161`).
- Object record (96 B): slot (0), name length (1), collection (2), kind (4),
  tags (8), length (12), object revision (16), created (24), modified (32),
  content FNV-64 (40), name (48..80), seal (88) (`archive.rs:181-193`).
- Content of the **current version only**, then a trailer with FNV-64 over all
  preceding bytes (`archive.rs:195-218`). Only present objects are listed and
  therefore exported (`service.rs:1011`).

**Restore today** (`archive.rs:395-498`): for each object it checks that the
destination slot is free (or a tombstone) and the name unused, otherwise it
reports a conflict and leaves the slot untouched; it maps or creates the
collection by name without renaming; it issues one WRITE (destination revision
1, or tombstone revision + 1) and, if tags, kind or collection are set, one
META (revision + 1) (`archive.rs:457-491`). The destination service assigns
new `created`/`modified` from its own clock. The spec states this explicitly:
"on restore the service assigns new times" (`S09-BACKUP-ARCHIVE.md:62-64`);
the native oracle expects "revision 1, or 2 after metadata"
(`S09-BACKUP-ARCHIVE.md:159-161`; `tools/volume_archive.py:107,128`). PR #396
(unmerged) records the same result for its cold-restore stage: one version,
new times, source revisions, history and timestamps not preserved.

**ustar/pax export** (`services/volume/src/ustar.rs:8-11`): drops kind, tags,
revisions, times and slots; writes mode 0644, uid/gid 0 and mtime 0. It is an
interchange export; nothing restores from it (ADR-S09-USTAR-20261114).

## 2. The gap, item by item

| Item | In source volume | Recorded in v1 | Preserved by restore | Reference |
|---|---|---|---|---|
| Content, name, length of current version | yes | yes | yes | `archive.rs:181-200`, `:457-463` |
| Slot | yes | yes | yes (or conflict) | `archive.rs:182`, `:422` |
| Tags, kind | yes | yes | yes (via META) | `archive.rs:185-186`, `:484-490` |
| Collection index and name | yes | yes | yes, unless the index holds another name (`uncollected`) | `archive.rs:436-456` |
| Object revision | yes | yes (offset 16) | no: 1, or 2 after META | `archive.rs:188`, `S09-BACKUP-ARCHIVE.md:159` |
| Current-version revision (`version_revision`) | yes; differs from the object revision after META/RENAME | no | no | `client.rs:189-191`, `service.rs:342` |
| Older versions (≤ 2): content, length, revision | yes | no | no | `table.rs:144-145`, `archive.rs:14` |
| Older versions: names | yes (table) | no | no; STAT reports only revision, length and checksum per history version (`service.rs:343-348`) | `client.rs:181-186` |
| Object `created`/`modified` (logical) | yes | yes (offsets 24, 32) | no: destination clock | `archive.rs:189-190`, `service.rs:614-617` |
| Per-version timestamps | **do not exist** | — | — | `table.rs:31-43` |
| Wall-clock times | **do not exist** | — | — | `table.rs:134`, `ustar.rs:10-11` |
| Tombstones (deleted, unpurged objects with history) | yes | no (LIST skips them) | no | `service.rs:1011` |
| Collection revision | yes (COLLECTION_STAT returns it) | no | no: new local revision | `client.rs:209-216`, `archive.rs:154-161` |
| Deleted collections | yes (revision ≠ 0, not present) | no | no | `table.rs:385-389` |
| Source domain clock | yes (table header) | no; not exposed on the wire | — | `table.rs:402-406`, `service.rs:594-597` |
| Domain generation (sum of revisions) | derivable | yes (header 16) | not used by restore | `table.rs:465-474` |
| `request_id` (object and collection) | yes | no | no — **must stay excluded**: destination-local idempotence token | `table.rs:131-133` |

**v1 suffices for:** content, name, length, slot, tags, kind, collection name
and index, the object revision and the logical `created`/`modified` of the
current state as **recorded source values**. **v1 does not suffice for:**
history, history names, `version_revision`, tombstones, collection revisions,
and a source clock against which recorded stamps can be bounded.

## 3. Required changes

### 3.1 Constraint found in the destination table

A verbatim installation of source revisions and stamps as **live** destination
values conflicts with table v2 invariants that recovery enforces fail-closed:

- every object and collection revision and every `modified` must be ≤ its
  domain clock, and the clocks must sum to the generation (`table.rs:644-665`);
- object revision and `modified` ≤ generation and `created ≤ modified`
  (`table.rs:262-264`, `:290`);
- A/B recovery accepts only adjacent generations, `g` and `g ± 1`
  (`table.rs:822-844`);
- `touch` increments by exactly one and stamps the committing domain's clock
  (`service.rs:603-618`).

A fresh destination domain has clock 0. Installing source revision 57 would
require the clock and the generation to jump, which breaks adjacency and the
clock sum. Verbatim live installation therefore needs a table and recovery
format change, not only a restore mode. Three levels follow; each is an owner
decision point.

| Level | What is preserved | Volume change | Assessment |
|---|---|---|---|
| **L1 — history replay** | content, names and order of all retained versions, tags, kind, collections; source identifiers preserved **in the archive and in the restore report only** | none (client-side restore mode, existing WRITE/META/COLLECTION ops) | smallest; never relabels a destination value as a source value |
| **L2 — recorded provenance** | L1 plus immutable `origin_revision`, `origin_created`, `origin_modified` per object, `origin_revision` per version and per collection, stored in the destination table and returned by STAT | table v3 (fields in currently reserved-zero bytes, e.g. head sector bytes 280..488, `table.rs:256-259`), a changed history-version invariant (3.2), STAT wire extension, new INSTALL operation | preserves source identifiers without touching live clocks; reviewable format and invariant change |
| **L3 — live identity** | live destination revisions and stamps equal source values | per-domain clock base in the table header, changed clock-sum and recovery rules | **not recommended**: alters the crash-recovery invariant and the cross-domain stamp property tested by `stamps_do_not_reveal_other_domain_commits` (`tests/history.rs:391`) |

### 3.2 Restore contract changes (native mode, L1 then L2)

- **Separate mode.** A new `restore_native` beside the unchanged `restore`.
  The existing v1 restore semantics, report and conflict rule stay as they are.
- **Destination precondition.** Native mode runs only into a domain in which
  every slot and collection has revision 0 (never used). Otherwise it refuses
  before any write, with a distinct status; it does not merge. Merging into a
  used domain remains the v1 restore.
- **Per object (L1).** Replay retained versions oldest first with WRITE (each
  WRITE pushes the previous version into history, `service.rs:551-567`), then
  META, then DELETE for an archived tombstone. The destination assigns its own
  revisions and stamps; the report lists, per slot, the recorded source
  identifiers next to the destination revisions, labelled as such.
- **Per object (L2).** A single INSTALL commit per object writes all versions
  and the provenance fields and advances the domain clock by exactly one, so
  the clock-sum and `g ± 1` adjacency rules of 3.1 still hold. It does **not**
  leave every table invariant unchanged: an object installed in one commit
  has revision 1, while table v2 requires `current.revision ≤ revision` and
  non-zero, strictly decreasing history revisions below it (`table.rs:292`,
  `:308-311`), so a chain of two or three versions cannot carry distinct
  local revisions. L2 therefore also changes the version-revision invariant
  in table v3: local version revisions are decoupled from the object revision
  (for example, per-object version ordinals 1..=3, strictly decreasing from
  current through history, with the object revision bound removed for
  installed objects). This is a further invariant change on the
  crash-recovery path and needs its own recovery tests. The alternative, one
  INSTALL commit per version, keeps the v2 invariant but loses the
  one-commit, no-partial-state property used under "Interruption" below.
  The INSTALL commit writes presence, tags, kind and collection together with
  the versions, so no META step follows it. Provenance fields are immutable
  after installation and are never fed into `touch`.
  **Revision source after INSTALL.** WRITE and RESTORE on `main` give the new
  current version the *object* revision + 1 (`current` is the `Entry`;
  `service.rs:532`, `:555`, `:630`, `:662`). After an INSTALL (object revision
  1, current version ordinal for example 3) that would give ordinal 2, below
  the history ordinals 3 and 2 that `push_history` moves down
  (`table.rs:199-206`), and break the strictly decreasing order. Table v3
  therefore assigns the new current version
  `1 + max(object revision, newest retained version ordinal)`, where the
  newest retained ordinal is the current version's for a present object and
  history slot 1's for a tombstone (`current` is empty, `service.rs:684-692`).
  For a non-installed object the v2 invariant makes the maximum equal to the
  object revision, so the rule reduces to the v2 rule. The value is derived
  from the durable entry alone, so recovery after an interrupted WRITE or
  RESTORE needs no new state: a commit either landed with that ordinal or did
  not. Required tests (not written): INSTALL then WRITE; INSTALL then RESTORE
  of each history slot; INSTALL of a tombstone then WRITE; each with a cut
  before, during and after the table write, followed by a fresh-instance mount
  that validates the v3 order.
- **Validation before install** (both levels; source-state bound from 3.3):
  per object `version revisions strictly decreasing` from the current version
  through history; `current version revision ≤ object revision` for objects
  without L2 provenance (for installed objects the v3 ordinal rule under L2
  applies instead: ordinals ≥ 1, strictly decreasing, no bound by the object
  revision);
  `0 < created ≤ modified ≤ source clock`; `object revision ≤ source clock`;
  collection revision ≤ source clock. **Conditional:** the recomputed sum of
  object and collection revisions equals the header domain generation. The
  domain generation sums every slot and collection revision, present or not,
  including purged slots (revision ≠ 0, no history) and deleted collections
  (`table.rs:465-474`). The check is therefore applicable only when the
  archive records the revision of every such slot and collection; with
  tombstones excluded (the R2 default) that needs content-free revision
  records for deleted, purged and tombstoned slots and for deleted
  collections. Whether v2 carries such records is part of R2; without them
  the check is not applicable and is not performed. Any violation: `Corrupt`,
  nothing installed. Values are copied only from validated records, never derived,
  interpolated or defaulted.
- **Interruption.** A rerun after interruption treats a slot whose state
  equals the archived object completely (presence; all versions by length,
  name and checksum; tags, kind and collection; under L2 the provenance) as
  already restored, installs empty slots and refuses any other state with a
  conflict, with one exception under L1: when every version matches exactly
  and only the metadata differs, the rerun completes the deterministic tail
  that the interruption cut off. That is META when the destination metadata is
  still the post-WRITE default (tags 0, kind 0, collection 0), and for an
  archived tombstone the final DELETE when the slot is still present with the
  archived versions and default metadata (DELETE resets metadata,
  `service.rs:684-692`, so a present slot with non-default metadata is a
  conflict). This is safe because the versions prove that the slot holds
  exactly the replayed content, and META or DELETE then moves it to the
  archived state without touching content; any other metadata difference
  was not produced by this restore and is a conflict. Under L2 the single
  INSTALL commit writes metadata with the versions, so a version match with a
  metadata difference is always a conflict. History names are compared through
  the extension proposed under 3.3 for every slot, present or tombstone, since
  STAT on `main` returns no history names; for a tombstone slot all version
  fields come from that extension. The destination precondition is
  checked per slot on resume, not per domain. A partly replayed L1 object (a
  prefix of its version chain) is a conflict that no in-domain operation can
  clear: DELETE leaves a tombstone and PURGE clears history but keeps the
  slot's non-zero revision (`service.rs:684-696`), so the slot never returns
  to revision 0. The recovery path after an interrupted L1 replay that left
  such a slot is therefore a new native restore into a fresh (never-used)
  domain or a fresh volume; the partly restored domain stays as it is and is
  reported. L2's one-commit INSTALL avoids that state.
- **Unknown outcome.** After an `Unknown` table write the volume service
  instance is uncertain and answers every later request, STAT included, with
  `Unknown` (`service.rs:6-9`, `:288-291`; set by the table writes in `publish`,
  `:874`, `:894`, `:903`, and by the SHUTDOWN flush, `:518-519`); the current v1 rule
  (`archive.rs:464-478`) therefore returns `Unknown` when it runs on that
  instance. The native restore stops at an `Unknown`, and its outcome is
  decided only after a fresh volume service instance has recovered from disk
  (a service restart or reboot; this design specifies no restart mechanism).
  The outcome is accepted only if STAT on that fresh instance shows exactly
  the expected state; otherwise the rerun proceeds by the interruption rule
  above.

### 3.3 Archive format changes (v2)

Indicative layout; sizes are proposals, not decided:

- Header (64 B): magic, version 2, counts, **source domain clock** (new),
  domain generation, total content, flags (bit 0: tombstones included), seal.
  Recording the clock needs a wire change: the own domain clock is not exposed
  today. Exposing a domain's own clock to that domain does not contradict the
  cross-domain property in `service.rs:591-593`; this needs review.
- Collection record (64 B): index, presence, name, **collection revision**.
- Object record (128 B): slot, presence, collection, kind, tags, object
  revision, created, modified, `history_count`, seal.
- Version record (64 B, `1 + history_count` per object, current first,
  omitted current for a tombstone): version revision, length, content FNV-64,
  name, seal; content follows each version record. On `main` the per-version
  fields are reachable for present objects only: STAT returns history
  revision, length and checksum but no history names (`service.rs:333-348`),
  and for a non-present slot (tombstone) it returns only `NotFound` with the
  object revision (`service.rs:327-331`, error reply `:322`); LIST omits
  tombstones (`service.rs:1011`); READ with a history selector reaches a
  tombstone's history once given that revision (`service.rs:911-926`) but
  returns content bytes only, without version revision, checksum or name.
  **Proposed extension (step-2 wire work, not reviewed, does not exist):** a
  STAT variant with a selector that, for the caller's own domain only,
  returns `present`, `history_count` and, per history version, revision,
  length, content checksum and name, for present and non-present slots
  alike. The exporter uses it for the history names of present objects and for every
  tombstone version, and the resume comparison under 3.2 uses it for the
  history names of every slot and for all version fields of a tombstone slot. It discloses only the
  caller's own retained history; whether that is acceptable after a DELETE
  belongs to the R2 decision.
- Trailer unchanged in shape. Content is stored per version without
  deduplication, so the independent Python reader stays simple.

**Bounds (derived from `lib.rs` constants, untested).** Each version's chunks
are a subset of its object's unique chunks, and a domain holds at most
`QUOTA = 40` unique chunks of 4 KiB. Total content is therefore at most
`(HISTORY + 1) × QUOTA × CHUNK = 3 × 40 × 4096 = 491 520` bytes; with the
indicative records the largest v2 archive is about
`64 + 4·64 + 8·128 + 24·64 + 491 520 + 16 = 494 416` bytes (966 sectors). The
reader must enforce this constant, not `count × MAX_LENGTH`. It fits the
1040-sector fixture range (`S09-BACKUP-ARCHIVE.md:107-111`); the size of the
second disk range in PR #396 (unmerged) has not been checked here.

**Compatibility.** `walk` dispatches on the header version. v1 stays readable,
verifiable, restorable by the v1 rule and convertible to ustar. A v1 archive
offered to native mode is refused with a distinct status; it is never upgraded
by synthesising history, a clock or provenance. A v2 archive offered to the
v1-only code on `main` fails the version check (`archive.rs:283`) and is
refused as `Corrupt`. The ustar converter may accept v2 and export current
versions only; it stays lossy.

**Restore capacity.** L1 replay allocates fresh chunks for every WRITE
(`service.rs:410-420`) and checks the quota conservatively (`:388-392`). A
source whose versions share chunks after RESTORE can exceed the destination
quota on replay and stop with `Quota`. L2's INSTALL can share chunks that are
byte-identical at the same position, which `Entry::validate` already permits
(`table.rs:316-329`).

### 3.4 What must never happen

- A destination-assigned revision or stamp presented as a source value, or a
  source identifier, stamp or per-version time that the archive does not
  record being synthesised, defaulted or interpolated.
- Wall-clock times shown as "original" times: the source has none.
- `request_id` copied from the source.
- The ustar/pax export described or used as native recovery, or a restore path
  from ustar.
- A claim of authenticity. FNV-64 detects accidental damage only
  (`archive.rs:11-13`). The validation in 3.2 proves internal consistency of
  an untampered archive; a deliberately resealed archive with consistent
  invented values would be accepted. "Never fabricated" therefore holds
  relative to archive integrity. Authenticity belongs to the encryption and
  key-management contracts referenced by D5 and is out of scope here; no new
  cryptographic construction is proposed.

## 4. Recommendation

Implement **L1**, then decide on **L2**; do not pursue L3. Reviewable steps,
each with its own tests and without changing v1 behaviour:

1. **Spec and oracle first:** v2 byte layout in `S09-BACKUP-ARCHIVE.md` and a
   v2 parser in `tools/volume_archive.py`, independent of `volume-core`.
2. **Wire read extensions** (volume service, review required): own domain
   clock and history version names. Read-only; no new authority.
3. **Writer:** `export` with an explicit v2 selector; v1 export unchanged.
4. **Reader:** version dispatch in `walk`, v2 validation and bounds.
5. **Restore mode L1:** `restore_native` with the precondition, replay order,
   resume rule and a report that keeps source and destination identifiers in
   separate fields.
6. **Host tests** (section 5), then **native stage** on the PR #396 harness.
7. **L2 (separate decision):** table v3 provenance fields, INSTALL operation,
   STAT extension, recovery and crash tests.

**Owner decisions (R1 decided; R2–R4 open):**

- **R1 — decided by the project owner, 19 November 2026:** L1 first,
  followed by a separately reviewed L2 implementation; L3 is not pursued.
  L1 is completed and independently verified before L2 begins. L2 is the
  intended long-term native recovery target: original source revisions and
  logical timestamps are kept as immutable provenance, separate from the
  destination-assigned live revisions and clocks. Before implementation or
  activation, L2 requires explicit review of table v3, the crash-recovery
  invariants, compatibility, security boundaries and negative tests.
  Historical values are never fabricated, v1 restore behaviour is unchanged,
  and the POSIX ustar/pax export is not native recovery.
- **R2:** include tombstones (deleted, unpurged content) in native archives.
  Recommendation: off by default, because a backup would otherwise retain
  content the user deleted after PURGE on the source.
- **R3:** native mode only into a never-used domain (recommended) or also
  into conflict-free slots of a used domain.
- **R4:** v2 range sizing for the destination service (D1) based on the
  derived 494 416-byte bound.

**Risks:** archive size grows up to about threefold for objects with history
(bounded as above); L1 replay can hit the destination quota; L2 is a table
format change on the crash-recovery path; the wire extensions widen what a
domain can read about itself; adding native stages changes the 22-stage count
pinned in protected workflows, which is coordinated by the PR/Issue Executor
for PR #396 and is not changed by this work.

## 5. Verification plan (not executed)

**Host property tests** (`services/volume/tests/`, real client and service as
in `tests/archive.rs` and `tests/history.rs`): random sequences of WRITE,
RENAME, META, RESTORE, DELETE and COLLECTION on a domain; export v2; the
Python reader must parse the bytes, and its values must equal the source
STAT, COLLECTION_STAT and READ-with-selector results byte for byte (revisions,
`version_revision`, created, modified, per-version name, length and content,
collection revisions). Then source loss, cold recovery of a fresh destination,
`restore_native`, and comparison: L1 versions, names, order and metadata equal
the archive and the report's source fields equal the archive's; L2 provenance
fields equal the archive's.

**Negative tests:**

- resealed archive with a history revision ≥ the current version revision, or
  non-decreasing history: `Corrupt`, nothing installed;
- `modified` or object revision greater than the recorded source clock;
  `created > modified`; `created = 0`;
- header domain generation not equal to the recomputed revision sum
  (conditional: only for archives that record every non-zero slot and
  collection revision, see 3.2);
- truncated history (`history_count` 2, one version record): `Corrupt` at
  every prefix, as in `every_truncation_flip_and_trailing_byte_is_rejected`;
- v1 archive into native mode: refused, distinct status, destination
  untouched; the same bytes still restore by the v1 rule;
- v2 archive into the v1 reader: `Corrupt`;
- used destination domain: refused before any write;
- quota exhaustion during L1 replay: stops, report names the restored slots;
- interruption at every commit boundary (crash harness pattern of
  `tests/crash.rs`), then rerun: converges to the complete state or reports a
  conflict, never a mixed object;
- partly replayed L1 object, then a rerun into the same domain: conflict for
  that slot, nothing overwritten; a rerun into a fresh domain completes;
  PURGE of the partial slot does not make it eligible (revision stays ≠ 0).

**Native QEMU (disposable images only),** reusing the PR #396 (unmerged)
dual-disk harness, `test-volume-archive-*` kernel features and
`tools/volume_archive.py`: a source fixture that creates history (rewrite,
rename, META, RESTORE), export v2 to disk 1, cold restore on a blank disk 0
with READ-only access to disk 1, and a host oracle that compares versions,
names, order, metadata and (L2) provenance byte for byte against the source
image; plus one stage that offers a v1 archive to native mode. Existing v1
stages stay unchanged as regression evidence. The new stage count needs the
coordinated workflow pin update described in section 4.

## 6. Open points and limits

- No item in this document is implemented or tested.
- The collection-sector space for a collection `origin_revision` under L2 has
  not been checked.
- Whether the READ reply can carry a version name without a format change has
  not been checked; step 2 assumes a wire extension.
- The PR #396 disk-1 range size is not verified against the derived bound.
