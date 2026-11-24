# S09 — Own-domain reads for the archive v2 writer (specification)

**Status:** reviewed and implemented on host (19 November 2026). An
independent AI security review (not a human audit) returned REQUEST_CHANGES;
its five required changes are applied below and recorded in section 8. The
operations, the client methods and the archive v2 writer (`export_v2`) are
implemented in `services/volume` with host tests only; nothing native was run.
It specifies the smallest volume wire extension that lets a
client export a complete archive v2 ([S09-ARCHIVE-V2.md](S09-ARCHIVE-V2.md))
of **its own domain**. It changes a shared security contract (the M6e volume
wire), so it requires the separate independent AI security review accepted
by the project owner before implementation; that review is not a human
audit. Line references are to branch `claude/s09-v2-writer` at `d98b3610`.

## 1. Gap analysis: what the writer needs and what exists

| Archive field (v2 spec section) | Existing source | Gap |
|---|---|---|
| Domain generation (2.1) | LIST reply `revision` = `domain_generation` (`service.rs` `list`, `table.rs:465-474`) | none |
| Source domain clock (2.1, S6–S8) | `table.clocks[domain]`, not on the wire | **missing** |
| Collection index, state, name, revision (2.2) | COLLECTION_STAT (`service.rs:724-736`); deleted: revision, count 0, no name | none |
| Present object: state, collection, kind, tags, revision, created, modified, history count (2.3) | STAT (`service.rs:327-357`) | none |
| Current version: name, length, revision, FNV-64 (2.4) | STAT: name, `aux` = length, data 72..80, data 32..40 | none |
| History version: revision, length, FNV-64 (2.4) | STAT data 80.. (`wire::STAT_HISTORY` = 20 B each) | none |
| History version: **name and name length** (2.4) | not on the wire | **missing** |
| Version content | READ with selector 0..=HISTORY (`service.rs:911-927`, `client.rs` `read_version`) | none |
| Absent slot: object revision (2.3, S9) | STAT error reply `NotFound` + revision | none |
| Absent slot: **created / modified** (2.3, S4–S6) | STAT on an absent slot returns only `NotFound` + revision (`service.rs:329-331`) | **missing** |
| Tombstone versions (TOMBSTONES) | — | not needed: R2 keeps `TOMBSTONES` clear |

`STAT_LENGTH` is 120 of `DATA` = 160 bytes. Two history names need at least
2 × 33 = 66 bytes, so names cannot be appended to STAT; selector-0 STAT stays
byte-identical, because two independent parsers check it strictly
(`client.rs` `validate_reply`, `length == STAT_LENGTH`;
`sdk/document-session/src/volume.rs:280-300`).

Without the absent-slot stamps a writer could only set `ACCOUNTED` for
domains that never deleted an object; it must never fabricate stamps.

## 2. Proposed interface

Two **new read-only operations**, additive within wire version 6 (as
SHUTDOWN was, `wire.rs:10`): `VERSION` stays 6, `SIZE`, `TAG`, header layout
and every existing operation are unchanged. Neither carries an upload token
(bytes 4..8 zero, `upload.rs`).

```text
pub const DOMAIN_STAT: u16 = 17;   // own domain clock and generation
pub const VERSION_STAT: u16 = 18;  // stamps of any own slot, or a history name
```

### 2.1 DOMAIN_STAT (17)

**Request.** `object` = the caller's base object (`domain * PER_DOMAIN`);
`revision`, `length`, `name_length`, `count`, `aux` all 0.

**Reply (Ok).** `revision` = `table.domain_generation(domain)` (the same value
as LIST); `aux` = `table.clocks[domain]`; `length`, `name_length`, `count` 0;
no data. It never returns the global `table.generation`.

The clock is **not** the LIST generation: the generation is the wrapping sum
of all object and collection revisions of the domain, the clock counts the
domain's own publishes (`service.rs:850-851`, `:879`). The comment at
`sdk/document-session/src/volume.rs:313` ("same domain clock") refers to the
LIST generation; it is not changed here.

### 2.2 VERSION_STAT (18)

**Request.** `object` = any slot of the caller's domain; `revision` = the
object revision the caller expects (≠ 0, as returned by STAT or its NotFound
reply); `name_length` = selector `s`, 0..=HISTORY; `length`, `count`, `aux` 0.

**Service order** (mirrors READ, `service.rs:911-927`):

1. `entry.revision != request.revision` → `NotFound` with revision 0 if
   `entry.revision == 0`, else `Stale`.
2. `s == 0` (any slot, present or absent): reply `revision` = entry revision,
   `count` = present flag (0/1), `length` = 16, data = `created` u64 (0..8),
   `modified` u64 (8..16); `name_length`, `aux` 0.
3. `s ≥ 1` and the entry is **absent** → `NotFound` with the entry revision.
   No version field of a deleted-but-unpurged version is returned (R2).
4. `s > history_count` → `NotFound` with the entry revision.
4a. (Review change 1, option (a).) `s ≥ 1` unless current → history[0] →
   … → history[s − 1] have **consecutive revisions** (each step exactly 1)
   → `NotFound` with the entry revision. This is the L1 prefix rule of
   S09-ARCHIVE-V2 section 3, computed from the entry alone. A DELETE raises
   the entry revision without a version, so a version pushed by a DELETE is
   always followed by a gap of at least 2 to any later current version; the
   rule therefore never names a version from before a DELETE, also after the
   slot was re-created by a COMMIT (which keeps the pre-delete history,
   `table.rs:199-206`). It is conservative: a RENAME or META gap hides the
   older versions too. With this rule step 3 is implied (an absent entry's
   current revision is 0); it is kept as explicit defence in depth.
5. Otherwise, for `v = history[s - 1]`: reply `revision` = entry revision,
   `aux` = `v.revision`, `name_length` = `length` = `v.name_length`, data =
   the name bytes; `count` 0. Length and checksum stay with STAT; `aux` lets
   the client bind the name to the STAT-reported history version.

Errors use `wire::error_reply` unchanged: no data, names or revisions except
the authorised revision on `NotFound`.

### 2.3 Request validation (`wire::validate_request`)

- The operation range becomes `STAT..=VERSION_STAT`; codes ≥ 19 stay `NoSys`.
- Existing checks run first and unchanged: version, bytes 4..8, epoch,
  status, request ID, object range, then `domain_of(object) != domain` →
  `Access` before any state is read, then trailing data bytes zero.
- `DOMAIN_STAT => base && h.revision == 0 && length == 0 && h.name_length == 0
  && h.count == 0 && h.aux == 0` (the SHUTDOWN shape).
- `VERSION_STAT => h.revision != 0 && length == 0 && h.count == 0 && h.aux == 0
  && h.name_length as usize <= HISTORY`.
- Existing text changes only in the `STAT..=SHUTDOWN` range in `wire.rs` and
  the non-mutating `matches!` in `service.rs` (2.4). (Review change 2.)
  `tools/mutate_volume.py` aborts when an anchor occurs other than once, so
  the new arms in `wire.rs`, `client.rs` and `service.rs` must not repeat any
  `MUTANTS` anchor (for example the 16-space `&& h.count <= 1` line, which the
  VERSION_STAT arm avoids by `h.count < 2`). `python3 -B
  tools/mutate_volume.py` is an acceptance command; it gains mutants that drop
  `base` on DOMAIN_STAT, return `table.generation` instead of
  `clocks[domain]`, drop the consecutive-prefix check (4a), and drop
  `revision == request.revision` in both client VERSION_STAT arms. A mutant
  "drop the absent check in step 3" would be equivalent under 4a and is not
  listed.

### 2.4 Service dispatch

Both operations join the non-mutating set at `service.rs:292-295`, so they
are answered on a read-only mount; the `uncertain` check before it still
answers `Unknown` first. Neither calls `publish`, `next_clock` or `touch`, and
neither writes or flushes the disk. Upload sessions are not consulted.

### 2.5 Client (`services/volume/src/client.rs`)

New methods `domain_stat(domain) -> (generation, clock)` and
`version_stat(object, revision, selector) -> VersionStat`. Reply validation (`Protocol` on
failure), in addition to the common checks:

- DOMAIN_STAT: `length`, `name_length`, `count` 0; `clock == 0 ⇒ generation == 0`.
- VERSION_STAT `s = 0`: `revision == request.revision`, `length == 16`,
  `count ≤ 1`, `name_length == 0`, `aux == 0`, `0 < created ≤ modified`.
- VERSION_STAT `s ≥ 1`: `revision == request.revision`, `count == 0`,
  `name_length == length`, `named()` (1..=32, `valid_name`),
  `0 < aux < revision`.

The writer additionally requires `aux` to equal the history revision STAT
reported for that slot and selector, and `count == 0` for every slot whose
STAT returned `NotFound`.

## 3. Snapshot consistency (writer rule)

An export spans many requests. Every publish of the domain raises
`clocks[domain]` by exactly one and no other path changes it
(`service.rs:850-879`; recovery reads it from the table header,
`table.rs:616-618`, and `table.rs:648-656` requires the clocks to sum to the
generation). Other domains never change this domain's entries, collections
or clock (publish takes the caller's kernel-bound `domain`). Hence: the
writer calls DOMAIN_STAT first and last; **equal clocks prove that no
own-domain commit happened in between**, so every STAT, VERSION_STAT,
COLLECTION_STAT, LIST and READ reply describes one state. On a difference the
writer discards the partial archive and retries a bounded number of times,
then refuses (`Busy`). The recorded header clock is the first value. Per-slot
freshness is additionally enforced by `revision` on READ and VERSION_STAT.

(Review change 3.) The proof holds only **within one service instance and
one epoch**: a restart with the same epoch that recovers an older table copy
could take the clock from c + 1 back to c and a new commit to c + 1 again over
different content. The writer pins the epoch for the whole export (the
client epoch is fixed at construction) and treats **any non-Ok reply** to
the final DOMAIN_STAT (`Unknown`, `Invalid`, `NoSys`, a transport error)
exactly like a clock difference: no trailer, output discarded.

**Implementation deviation.** `export_v2` writes to a `Sink` that it cannot
rewind, so it makes one attempt, like the v1 `export`: on a difference it
returns `Stale` without the trailer, and retrying is the caller's policy.
The final DOMAIN_STAT is the last request before the trailer.

## 4. Compatibility

- **Old service.** A service without this change refuses codes 17 and 18 in
  `validate_request` with `NoSys` (`wire.rs:186-188`) before reading state.
  The writer maps `NoSys` to "v2 export unsupported" and writes nothing; it
  never falls back to synthesised values. v1 export, v1 restore and the
  ustar/pax export do not use the new operations and are unchanged.
- **Old clients.** Unaffected: no existing request or reply changes.
- **SDK.** `sdk/document-session`, `sdk/examples/{system,materiaal,volume-docs}`
  use only existing operations and need no change; their suites must still
  pass. `volume-docs` accepts `wire::TAG | upload::TAG` bodies of
  `wire::SIZE` and dispatches them to `handle_for` (token-bound ingress) or
  `handle`, always with the kernel-named domain (`service.rs:130-160` there);
  only SHUTDOWN gets an extra domain check. No change is needed there.
- **Kernel.** Searched: `grep -rniE "4d36_?454c"` (the M6e tag) over
  `kernel/`, `boot/`, `userspace/` (no hit); `volume_core` over `kernel/`,
  `boot/` (no hit); `COLLECTION_STAT`/`collection_stat` over `kernel/`,
  `userspace/`, `tools/`, `tests/` (only `tools/mutate_volume.py`);
  `kernel/src/m2/m3/system.rs` has no tag or operation filter for this wire.
  `kernel/src/m2/m3/volume.rs` handles the block-protocol leg. No kernel
  change is expected. This is static evidence only; no native QEMU run.
- **Tables.** No table, recovery or on-disk change; the read exposes values
  the table already holds.
- **Existing test.** `tests/service.rs:838` (`SHUTDOWN + 1` → `NoSys`) becomes
  `VERSION_STAT + 1` → `NoSys`; DOMAIN_STAT gets its own field cases.

## 5. Security invariant and disclosure

**Invariant.** A request reads only state of the caller's kernel-bound domain:
the domain comes from the ingress, never from the request; the object
selector is checked against it before any state is consulted; DOMAIN_STAT
indexes `clocks[domain]` and `domain_generation(domain)` only. Neither
operation mutates state, allocates, or affects another domain.

**Cross-domain stamp property** (`service.rs:591-593`, test
`stamps_do_not_reveal_other_domain_commits`, `tests/history.rs:391`). The
own clock counts only the caller's own publishes, so it is unchanged by any
number of commits in other domains, as STAT stamps already are. DOMAIN_STAT
must never return `table.generation` (the global sum of all clocks), which
would reveal other domains' commit counts.

**Newly disclosed to the caller, about itself only:**

| New value | Already reachable today |
|---|---|
| Own clock (exact count of own commits) | only at the last touch of a present object (its `modified` stamp). Collection-only commits (`collection`, which does not call `touch`) appear in no stamp, so DOMAIN_STAT newly reveals the exact count of own collection commits since the last object touch (review change 4) |
| Names of history versions of **present** objects within the consecutive-revision prefix (2.2, 4a) | revision, length and checksum (STAT), content (READ selector), and the name after a RESTORE (a mutation, `service.rs:650-662`) |
| — | Names of pre-delete versions of a re-created object are **not** disclosed: rule 4a refuses them (review change 4) |
| `created`/`modified` of **absent** slots with revision ≠ 0 (deleted or purged objects) | the object revision only (STAT `NotFound`) |

**Not disclosed:** names, lengths, revisions or checksums of versions of an
absent slot (deleted-but-unpurged, R2), an absent slot's history count, and
the name of any history version behind a revision gap (4a), in particular
versions from before a DELETE of a re-created object.
This is consistent with R2 but is defence in depth, not a confidentiality
boundary: READ with a history selector already returns a tombstone's
content to its own domain, and RESTORE followed by STAT yields its names.
The absent-slot stamps are exactly what R2 (c) permits a content-free record
to carry; the writer uses them only for content-free records under
`ACCOUNTED`, with `TOMBSTONES` clear.

## 6. Negative tests to add (`services/volume/tests/`)

1. `validate_request`, per operation, one case per field: non-zero
   `revision`/`length`/`name_length`/`count`/`aux` for DOMAIN_STAT,
   non-base object → `Invalid`; VERSION_STAT with revision 0, selector
   HISTORY + 1, non-zero `length`/`count`/`aux` → `Invalid`; trailing data →
   `Invalid`; non-zero bytes 4..8 → `Invalid`, on both `handle` and the
   token-bound `handle_for` path (`upload::validate_request`).
2. Foreign object or foreign base object → `Access`, with a disk that panics
   on any access and unchanged counters except `rejected`.
3. `VERSION_STAT + 1` → `NoSys`; old-service behaviour: a client whose
   transport answers `NoSys` makes the writer return "unsupported" and
   produce no bytes.
4. Cross-domain: domain A reads DOMAIN_STAT, domain B makes five commits and a
   collection change, A's DOMAIN_STAT and all A's VERSION_STAT replies are
   byte-identical; A's clock equals A's commit count; `table.clocks` checked.
5. Absent slot with history (deleted, not purged): selector 1 and 2 →
   `NotFound` with the entry revision and an all-zero data area; selector 0 →
   stamps only, `count` 0.
6. Purged slot: selector 0 returns stamps; selector 1 → `NotFound`.
7. Never-used slot (revision 0) with request revision 1 → `NotFound`, revision 0.
8. Stale revision → `Stale` with no data; selector > history count → `NotFound`.
9. Present object with two history versions: names and `aux` equal the table
   versions; `aux` equals the STAT history revision.
10. Read-only mount: both operations succeed, zero disk writes and flushes.
    Uncertain instance: both answer `Unknown`.
11. Client reply validation: forged replies (each rule in 2.5 broken once,
    including `created == 0`, `created > modified`, `aux ≥ revision`, invalid
    name, `clock == 0` with `generation != 0`) → `Protocol`.
12. Snapshot: a second client of the same domain commits between two writer
    reads; the clocks differ and the writer retries, then refuses.

Added by the review (change 5), all implemented:

13. RENAME of the current version, then COMMIT: refused by 4a (gap); a name
    at push time is checked through two COMMITs with different names.
14. VERSION_STAT with the pre-mutation revision after DELETE, PURGE, RESTORE
    or META → `Stale`, no data.
15. Object = OBJECTS → `Range`; wrong epoch → `Invalid`; opcode 19 → `NoSys`.
16. A foreign slot → `Access` with zero disk accesses.
17. Successful reads leave `commits`, `table_writes`, `flushes`, `restores`
    and an active token-bound upload session (token, principal) unchanged.
18. `handle_for` with principal 0 → `Access`; success → reply token 0.
19. Fresh domain: DOMAIN_STAT = (0, 0); after `Service::recover` the clock
    equals the pre-restart clock.
20. VERSION_STAT s = 0 on a present slot: stamps equal STAT, `count == 1`.
21. Name reply: bytes after the name zero, `count == 0`, no length/checksum.
22. DELETE → COMMIT → VERSION_STAT s = 1 (and 2) → `NotFound` (change 1).

Not implemented (optional): the random-sequence property test over two
domains; using DOMAIN_STAT clock == 0 in `never_used` (R3).

Existing suites that must stay green: `cargo +1.100.0 test --locked -p
volume-core`, the `sdk/document-session` and `sdk/examples/materiaal` tests,
`tests/test_volume_archive_v2.py`, and `python3 -B tools/mutate_volume.py`.

## 7. Out of scope

- A native (on-target) v2 export path, native `walk` dispatch,
  the v2 range adapter and any native QEMU stage.
- Tombstone version names or fields (`TOMBSTONES` stays clear by R2); an
  absent slot's history count.
- L2: provenance fields, INSTALL, table v3, any COLLECTION_STAT extension.
- A volume-wide free-chunk read (the `Full` limit for 25–40 chunk replays).
- Per-version timestamps, wall-clock time, a source volume/domain identifier
  (owner decision: none, also not behind a flag), and any read of another
  domain's clock or state.
- (Closed by review change 1.) Versions from before a DELETE are neither
  named on the wire (4a) nor exported: under the binding R2 and the owner's
  "Nee, weigeren" the writer exports only the consecutive-revision prefix,
  and the archive's history count reflects only that prefix.

## 8. Review record and implementation

**Review** (independent AI security review, not a human audit, static,
REQUEST_CHANGES at `d98b3610`). Required changes and where they are applied:
(1) R2 exposure of pre-delete names through VERSION_STAT — option (a), rule
4a in 2.2; (2) mutation anchors — 2.3, new mutants in
`tools/mutate_volume.py`; (3) snapshot epoch dependency and non-Ok final
DOMAIN_STAT — section 3; (4) disclosure table — section 5; (5) negative tests
— section 6, items 13–22.

**Implementation** (host only): `wire.rs` (`DOMAIN_STAT`, `VERSION_STAT`,
request shapes), `service.rs` (`domain_stat`, `version_stat`, both in the
non-mutating set), `client.rs` (reply rules, `domain_stat`, `version_stat`,
`VersionStat`), `archive.rs` (`export_v2`, `ExportV2`). Tests:
`services/volume/tests/service.rs` (wire, service and client cases; these
are the targets of the mutation run), `services/volume/tests/archive_v2_writer.rs`
(writer, round trip, old service, concurrent commit) and
`tests/test_volume_archive_v2_written.py` (Python oracle on the committed
writer output in `services/volume/tests/fixtures/archive-v2-written/`).

**Writer policy** (`export_v2`). `TOMBSTONES` is never set (R2).
`ACCOUNTED` is always set, and the only content-free records are those S9
needs: one per absent slot with a non-zero revision (deleted, tombstoned or
purged; stamps from VERSION_STAT s = 0, no name, no content) and every
deleted collection. A present object carries only its consecutive-revision
prefix; the number of left-out versions is returned as `omitted`. Every value
is read from the service (DOMAIN_STAT, COLLECTION_STAT, STAT, VERSION_STAT,
READ checked against the STAT content sum); none is synthesised, and no
source identifier exists. DOMAIN_STAT is the first request (an old service's
`NoSys` leaves the sink empty) and the last before the trailer. The writer
also refuses (`Stale`) a revision or `modified` above the first clock (S6–S8),
which under a fixed clock can only mean a concurrent change. v1 `export`,
v1 restore and the ustar/pax export are unchanged.

**Kernel and capability semantics.** Unchanged; no kernel file was edited.
The kernel search in section 4 is static evidence only.
