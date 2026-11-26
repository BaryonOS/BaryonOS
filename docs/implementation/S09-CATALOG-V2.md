# S09 — Destination catalog v2: backup time (O5) and controlled deletion (O3) (design specification)

**Status: design specification, revision 1 (25 November 2026). Nothing in this
document is implemented, native or active.** It specifies the "separate,
versioned extension" that the project owner's package-2 decisions on O3 and O5
(24 November 2026) require. The decisions are recorded in section 8.1 of
[S09-DESTINATION-SERVICE.md](S09-DESTINATION-SERVICE.md), called "the v1
specification" below, and section numbers without a document name refer to
it. Rules that need an owner decision are marked *[proposed; owner question
Vn]* and are listed in section 9. Revision 1 answers the independent AI
review CATALOG-V2-REVIEW-1 (REQUEST_CHANGES); section 11 records each item.
An AI review is not a human audit. S09 stays open.

Markers follow the v1 specification: *[main]* is a fact checked on `main`
`c04ea98d`; *[design]* is specified here and not built; *[unproven]* is an
assumption that no test in this project shows.

## 1. Scope and non-goals

In scope: the on-device catalog format v2 of the destination service, its
validator and mount rule, its compatibility with v1, the transitions that
commit a v2 catalog, the service operations that carry backup time and
deletion, the write-path rules that deletion changes, and the negative tests.

Not in scope:

- The owner principal of O12. O12 is decided in direction (owner binding per
  backup registration; catalog version, trusted identity and access checks to
  be designed with the profile, key and encryption contracts). The identity
  belongs to the B track and the user/session owner. Section 7 reserves space
  and states the hook; it does not define the identity (V1).
- Time-zone conversion and the display of dates. They belong to the UI track.
  This document fixes only what the catalog stores and what the service
  reports.
- A retention policy engine. The decision forbids an invented default period
  or count; section 6 states only the service-side guard (V5, V6).
- A trusted system clock. The only wall clock on `main` is the opt-in
  syscall `0x73` of ABI 1.9, which returns an `UnauthenticatedRtc` value that
  is public process metadata (ADR-050); a production TimeProvider is to be a
  separate isolated service and does not exist (ADR-057). Section 4 stores a
  time with a stated source and never depends on it (V3).
- Encryption. v1 archives are plaintext personal data (S09-BACKUP-ARCHIVE.md,
  "No authenticity or encryption"). Nothing here changes that.

## 2. Invariants (security first)

The v1 invariants stay. v2 adds or restates these:

- **I1 Fail closed across versions.** A service never writes a catalog
  version that it cannot validate, and a service that meets a version it does
  not know writes nothing. A v1 service already refuses a v2 copy: `decode`
  requires version 1 (`services/destination/src/catalog.rs:134` *[main]*), so
  two v2 copies mount as `Corrupt` under v1 and v1 writes nothing.
- **I2 No rollback across a version change.** A v1 service may meet one valid
  v1 copy and one v2 copy (a crash inside the upgrade commit). Whatever it
  mounts must not lose a committed entry or free a committed extent. Section
  3.3 achieves this with a content-neutral upgrade commit.
- **I3 Writes stay inside their region.** The entry path writes only inside
  the region that CREATE reserved, and the erase path writes only inside the
  extent of the one record in state `deleting`. Neither path reaches sectors
  0–15 or an extent that any valid copy on the device lists, other than the
  one being erased. This replaces the v1 rule
  `sector >= max(16, next_free)` (2.7 step 2), which no longer separates free
  from used sectors once gaps exist.
- **I4 No reuse before the removal is durable in both copies.** Sectors of a
  removed entry become reservable only when no valid copy on the device lists
  that entry (5.6). The N8 hazard (a stale copy that still lists an extent
  that a new entry overwrote) then needs damage to both copies, as in v1.
- **I5 Deletion is explicit and visible.** The service never deletes on its
  own. Only the trusted control lane deletes, one entry per request, and a
  deletion leaves a `deleting` record visible until it completes. An entry
  bound on any operation lane is not deleted.
- **I6 Time is information, never authority.** The stored backup time is not
  used for any ordering, deadline, generation, validity, recovery or security
  rule. Entry order is the entry ID and the sealing sequence, as in v1.
- **I7 Monotonic identities survive deletion.** The next entry ID and the
  copy sequence never decrease. A removed entry's ID is never issued again.

## 3. Versioning and compatibility

### 3.1 Version field

A v2 copy has the v1 magic `NOSDST\0\x01` and version u16 = 2 at bytes 8..10.
The magic is unchanged, so every tool that recognises a destination still
recognises it; the version tells them apart. A v1 service treats a v2 copy as
invalid (I1). *[proposed; owner question V2]* The alternative is a new magic.
It gives the same v1 behaviour, because v1 also checks the magic first, and it
loses the shared recognition, so it is not proposed.

### 3.2 A v2 service on v1 copies

A v2 service validates each copy with the validator of its own version: a v1
copy with the v1 rules of 2.6, unchanged, and a v2 copy with section 5.2. For
comparison, a valid v1 copy is *normalised* to v2: the same header and records,
version 2, every metadata entry zero (time unknown, never verified, state
`sealed`, no owner). A v2 service mounts a pair of two valid v1 copies under
the v1 mount rule, unchanged, and reports every entry as "date unknown"
(4.3). It writes no v2 copy at mount (2.6: a mount writes nothing).

### 3.3 Upgrade commit

Before its first content commit (SEAL, MARK, REMOVE or RECORD, section 5) on a
mounted v1 catalog, a v2 service commits an **upgrade**: the normalised copy
with sequence + 1, and nothing else changed. It writes the target slot (2.7
step 4 rule), flushes (barrier U2), mirrors into the other slot and flushes
(barrier U3). Only after barrier U3 does it run the content commit, which then
starts from two identical v2 copies.

Between U2 and U3 the device holds one v2 copy (sequence n + 1) and one v1
copy (sequence n) with the same records, `next_free` and next entry ID.

- A v2 service mounts that pair under the v2 mount rule (5.3): the higher copy
  is the lower one normalised at sequence + 1, the UPGRADE transition.
- A v1 service mounts only the v1 copy, which has the same content (I2). Its
  next commit targets the slot it sees as invalid, the v2 one, and writes v1
  there again. No entry is lost and no extent is freed.

A combined upgrade-and-content commit is not permitted. With it, the window
would hold a v2 copy one record ahead (or one record behind, after a
deletion) beside a v1 copy, and a v1 service would mount the v1 copy and
reserve sectors that the v2 copy lists: the N8 hazard across versions. N-V3
kills the combined commit.

After barrier U3 both copies are v2, and a v1 service mounts `Corrupt` and
writes nothing (I1). That downgrade costs availability, not data. A rollback
to an older OS build that only knows v1 therefore cannot use a destination
that a v2 service has written to. *[proposed; owner question V2]*

### 3.4 Wire compatibility

The wire version stays 1 (`wire::VERSION`, `services/destination/src/wire.rs:8`
*[main]*). The v1 operations 1–11 keep their field rules and replies, so a v1
portal works against a v2 service. v2 adds operations 12–14 (section 5). A v1
service answers them `NoSys` -2 at check 2, because `validate` accepts only
`INFO..=FORMAT` (`wire.rs:112-114` *[main]*). A v2 service reports its
catalog feature level in the INFO reply's `offset` field, which v1 leaves 0
(`service.rs:410-423` *[main]*): `offset` = 2 in every mount state (mounted,
safe path, blank candidate, failed-mount, `Corrupt`), because it describes the
service, not the device. A portal uses v2 fields only when INFO reports 2.

## 4. Backup time (O5)

### 4.1 What is stored

Each record gets, in the metadata table of 5.1, a **backup time** and a
**time source**:

- time: i64, seconds since 1970-01-01T00:00:00Z, leap seconds not counted
  (POSIX time);
- source: u8. 0 = unknown, 1 = system clock, not synchronised, 2 = system
  clock synchronised by a trusted time service. Other values are invalid.

The rule is "source 0 if and only if time 0". A time before 1970 is not
representable, and the validator refuses a negative time. The meaning is
fixed: **the time at which the trusted portal asked the service to seal the
entry, as the portal read it from the system clock.** It is not the time of
the source data, and not the time of the durable commit.

### 4.2 Where it comes from

The service reads no clock. Under ADR-050 the only wall clock is the opt-in
syscall `0x73`, whose `UnauthenticatedRtc` value carries no more trust in the
service than in the portal, and a future TimeProvider (ADR-057) is a
separate service that the portal, which already talks to the user, is the
natural client of. Keeping the service clock-free keeps its ABI surface and
its rule set unchanged for information it never uses (I6). The trusted portal
reads the clock and passes time and source in SEAL (5.4). The service checks
only the encoding rules of 4.1. *[proposed; owner question V3]* The stored
value is therefore the portal's claim: a portal on a host without a trusted
clock passes source 0, and a crafted image can hold any valid time (5.4 of the
v1 specification applies: FNV-64 detects only accidental damage).

Source value 1 is a reading of the `UnauthenticatedRtc` clock of ADR-050.
Source value 2 needs the TimeProvider of ADR-057, which does not exist yet;
until it does, a conforming portal passes only 0 or 1. Which provider counts
as "synchronised" is the TimeProvider contract's job, outside S (V3).

### 4.3 What is reported

The META operation (5.4) returns time and source per entry. A v1 entry, or a
v2 entry with source 0, is reported with source 0, and the dialog shows "date
unknown" (decision O5). The service never derives a time from the sealing
sequence, the entry ID or the per-domain logical clock of v1 objects
(S09-BACKUP-ARCHIVE.md). Conversion to the local time zone is the UI track's.

## 5. Catalog v2 and controlled deletion (O3)

### 5.1 Copy layout (2048 bytes)

| Bytes | Content |
|---|---|
| 0..64 | header, as v1 (2.6), version = 2; `next_free` has the v2 meaning below |
| 64..1344 | 16 entry records of 80 bytes, the v1 record format unchanged |
| 1344..1728 | metadata table: 16 entries of 24 bytes; entry *i* belongs to record *i* |
| 1728..1984 | owner table: 16 entries of 16 bytes, reserved for O12 (section 7); all zero in this revision |
| 1984..2040 | zero |
| 2040..2048 | FNV-64 over 0..2040, as v1 |

The 80-byte record stays the v1 record, because it is also the LIST wire
record and two of them fill the 160-byte data limit (`wire::LIST_RECORD`,
`LIST_MAX`, `wire.rs:10-11` *[main]*). The new fields live in parallel tables,
so LIST is unchanged.

Metadata entry (24 bytes): backup time i64 (0..8), verified sequence u64
(8..16, section 6), time source u8 (16), state u8 (17: 0 = `sealed`,
1 = `deleting`), 6 × 0 (18..24). Unused metadata and owner entries (index ≥
count) are zero.

### 5.2 v2 validity rules

A v2 copy is valid only if all of the following hold:

- every v1 rule of 2.6 except the two extent rules below, including O14 label
  content, with version 2 in place of 1;
- **non-overlapping extents** in place of "contiguous extents": every record's
  extent `[first, first + ⌈length/512⌉)` lies inside `[16, formatted)`, and
  no two extents intersect;
- **`next_free`** is the end of the highest extent, or 16 with zero records.
  It is a high-water mark, not "first free sector". The v1 rule "records
  strictly ascending by first sector" is dropped, because a new entry can
  reuse a gap below an older extent (5.5). Ascending entry ID and ascending
  sealing sequence stay;
- every metadata entry below count: time and source per 4.1; state 0 or 1;
  verified sequence 0, or ≥ the record's sealing sequence and ≤ the copy's
  sequence; padding zero;
- every metadata and owner entry at index ≥ count is zero;
- at most one record in state `deleting`;
- the owner table is all zero (until V1 is decided);
- bytes 1984..2040 are zero.

The checksum, magic, count ≤ 16, sequence ≥ 1, next entry ID rules and label
rules are as v1.

### 5.3 Transitions and the v2 mount rule

Every v2 commit writes exactly one transition at sequence + 1. Starting from a
committed copy *C*, the encoded copy *N* is:

| Transition | Operation | *N* equals *C* except |
|---|---|---|
| UPGRADE | before the first content commit on a v1 catalog (3.3) | version 2; normalised metadata (*C* is v1) |
| SYNC | before an erase whose other slot does not yet list the entry as `deleting` (5.6) | nothing but the sequence (*C* is v2) |
| SEAL | SEAL (2.7 step 4) | one record appended at index count (its entry ID is the largest); its sealing sequence = *N*'s sequence; its metadata = time and source from the request, verified 0, state `sealed`; next entry ID + 1; `next_free` = max(`next_free`, new extent end) |
| MARK | DELETE, phase 1 (5.6) | one record's state `sealed` → `deleting` |
| REMOVE | DELETE, phase 3 (5.6) | the `deleting` record and its metadata and owner entries removed, later entries moved down one index; `next_free` recomputed per 5.2 (it may decrease) |
| RECORD | RECORD-VERIFY (section 6) | one `sealed` record's verified sequence = *N*'s sequence |

All other fields, including next entry ID except under SEAL, are unchanged.
Sequence + 1 and next entry ID + 1 use checked arithmetic, as v1.

**Pre-write check (replaces 2.7 step 4 `extends_by_one` for v2).** Before
any catalog write, the service runs the v2 validator on *N* and checks that
*N* is exactly the intended transition of *C* (the table above, field by
field). On failure it replies `Corrupt` -18 for that request, writes nothing
and does not enter the `Corrupt` mount state, as in v1.

**v2 mount rule (replaces the record-prefix "extends" rule of 2.6 for v2
copies).** Exactly one valid copy: use it. Two valid copies, byte-identical:
use either. Two valid v1 copies: the v1 mount rule of 2.6, unchanged (3.2),
including a higher copy more than one commit ahead; the one-transition rule
below does not apply to them. Two valid copies that differ, at least one of
them v2: the one with the higher sequence, *H*, must have sequence = the
lower's (*L*) + 1, and *H* must equal one transition of the table applied to
*L*. If *L* is v1 and *H* is v2, UPGRADE is the only accepted transition. If
both are v2, UPGRADE is not accepted (it would turn a `deleting` record back
into `sealed` and drop verified state); SYNC, SEAL, MARK, REMOVE and RECORD
are. *H* v1 with *L* v2 gives `Corrupt`. Then use *H*. Otherwise: `Corrupt`.
Equal sequences with different bytes: `Corrupt`.

For pairs with a v2 copy this is stricter than the v1 prefix rule, which
accepts a higher copy any number of records ahead (`catalog.rs:206-213`
*[main]*). The legitimate states of the commit protocol never put the two
copies more than one commit apart: a completed commit leaves them identical, a
crash between barriers 2 and 3 leaves the target slot one commit ahead, and
the next commit targets the lower slot. The v1 prefix rule cannot be kept for
v2, because REMOVE makes the higher copy one record *shorter*. The "identical
v2 copies" starting point of 3.3 holds within one instance; after a reboot
between U2 and U3 the new instance mounts *H* and its next commit targets the
v1 slot directly, which the one-transition rule accepts.

**Remount after a failed catalog write (O16, O18).** The v1 rule generalises
unchanged. After a failed write or flush at barrier 2 or 3 of any v2 commit,
the remount resolves *done* only if the remounted catalog equals the encoded
copy *N*, and *not done* only if it equals the committed copy *C*. Anything
else is the failed-mount state. A failed catalog write is a safe-path trigger
for every v2 transition, as for SEAL (2.7, O13).

**Replies after a failed commit.** A failed write or flush in any catalog
commit (UPGRADE, SYNC, SEAL, MARK, REMOVE, RECORD) replies `Unknown` -16 to the
request that caused it, starts the safe path and runs the remount above. A
failed erase write or barrier-E flush replies per "Device results" (2.5:
`Io` -12, or `Unknown` -16 without a definite result) and starts the safe path;
the record stays `deleting` on the device. After the remount the portal learns
the outcome only through INFO and META, never from the reply. For SEAL, the
v1 rules for the export binding (2.7, N13) apply unchanged; a remount that
resolves *done* for an UPGRADE that preceded the SEAL's own commit resolves the
binding *not sealed*.

**Where UPGRADE and SYNC run.** A request that needs an UPGRADE (or SYNC) runs
it only after every refusal check of that request has passed and, for SEAL,
after step 3 (barrier 1, readback, `archive::verify` and the content check),
immediately before step 4. A request that is refused therefore never crosses
the v1 downgrade boundary.

### 5.4 Operations

| Op | Name | Request | Reply | Lane |
|---|---|---|---|---|
| 5 | SEAL (v2 form) | as v1, plus `aux` = backup time (i64 as u64) and `count` = time source 0..=2; both 0 for "unknown" | as v1 | export lane |
| 12 | DELETE | `entry` = a sealed or `deleting` entry ID; `count` = origin 1 or 2 (section 6); all other fields 0 | `Ok` after the REMOVE commit's barrier 3 | control |
| 13 | META | `entry` 0, `count` = cursor 0..16, as LIST | up to 5 META records of 32 bytes, `count` = next cursor: entry ID u64, backup time i64, verified sequence u64, source u8, state u8, 6 × 0, under the visibility rule of LIST | every lane, as LIST |
| 14 | RECORD-VERIFY | `entry` = sealed entry ID | `Ok` after the RECORD commit's barrier 3 | control (section 6) |

The META record carries the entry ID so that a portal can match it with LIST.
The verified sequence is reported only on the control lane; on an operation
lane that field is 0. *[proposed: META carries no
owner field until V1 is decided.]* The v1 SEAL field rule (`aux` = 0,
`count` = 0, `wire.rs:136,155` *[main]*) becomes, at feature level 2,
`aux` and `count` as above with "source 0 if and only if time 0". A v1 SEAL
(both 0) stores "unknown".

**DELETE refusals, in order:** check 2 field rules (`Invalid` -1); the mount
states of 2.2 check 7 (failed-mount, `Corrupt`, blank candidate) as for every
other request; no record with that ID (`NotFound` -19); a different record is
`deleting` (`Busy` -21); any operation lane is bound to the entry (`Busy`
-21); *[pending V1]* the owner check. A DELETE of the entry that is already
`deleting` resumes at phase 2. BIND of a `deleting` entry returns `NotFound`
-19, as for an unsealed ID. An export lane in the state *sealed* for *E*
counts as bound to *E*. *[proposed; owner question V8]*

**Safe path and mount states.** In the mounted safe path (2.2 check 7; 2.7),
DELETE and RECORD-VERIFY return `Access` -5 after checks 1–6 and issue no
device request, like CREATE, APPEND, SEAL and FORMAT; META runs like LIST. An
UPGRADE or SYNC only runs inside an admitted content request, so none runs in
the safe path. In the failed-mount, `Corrupt` and blank-candidate states the
v2 operations follow the table of 2.2 check 7 as every other request.

**Changes to the v1 lane rules.** DELETE and RECORD-VERIFY extend the
"Selection (dialog)" row of 4.1 and the control-lane operation list of 2.2
check 6 (today INFO, LIST, ABORT, BIND, UNBIND, FORMAT); META is admitted
wherever LIST is, including every operation-lane binding (`permitted`,
`services/destination/src/service.rs:825-835` *[main]*). The portal still
holds no control lane (v1 4.1, 2.7). N31 (the lane-rights matrix) gains these
rows. Who issues a policy DELETE (origin 2) is *[owner question V7]*:
proposed, the trusted dialog component itself runs an explicitly configured
retention rule and issues it on the control lane, so no new component gains a
control right.

**Ownership and revocation.** *[proposed; owner question V8]* A DELETE of a
bound entry is refused with `Busy`; the dialog unbinds first, which revokes
the lane grant (2.3 UNBIND), and the user sees which operation is cancelled.
A remembered VERIFY permission (3.3, O1) that names a removed entry ID becomes
void: BIND with that ID returns `NotFound` -19, and because a removed ID is
never issued again (I7) it can never address another entry; the dialog removes
the stale permission. Owner rules wait for V1.

**FORMAT.** *[proposed; owner question V2]* A v2 service formats a v1 copy,
exactly as 2.6, so that an empty destination stays usable by a v1 build; the
first content commit then upgrades it (3.3).

### 5.5 Allocation with gaps (CREATE)

A **free region** is a maximal run of sectors in `[16, formatted)` that no
record of the committed catalog covers *and* that no record of the other
slot's copy covers if that copy is valid (I4; 5.6). The service therefore
keeps the other slot's decoded copy from mount until the next commit's
barrier 2 (the v1 `Selection` keeps only validity flags and sequences,
`catalog.rs:229-236` *[main]*); "valid" means valid under that copy's own
version, so a valid v1 copy also blocks its extents. CREATE reserves:

1. the lowest free region of at least `MAX_ARCHIVE_SECTORS` = ⌈525 328 / 512⌉
   = 1027 sectors, the largest v1 archive (2.8); otherwise
2. the free region that ends at the scope end, if one exists; otherwise
3. nothing, and replies `Full` -17.

The reservation is `[first, region_end)`. APPEND accepts only writes inside it
and replies `Full` -17 at `region_end`, as v1 does at the scope end (2.7 step
2). With no gaps, rule 2 is v1's allocation exactly. A gap smaller than 1027
sectors that is not at the scope end is reused only once an adjacent removal
merges it into a larger region. This is user-visible: deleting small,
non-adjacent entries frees no space that CREATE can use, although INFO
reports it as free (5.6). The threshold is *[proposed; owner question V9]*;
the fragmentation cost is stated, not measured.

Freed regions may hold stale bytes: zero after an erase, or a valid old
archive if V4 chooses "unlink only". The O17 (a) content check of SEAL step 3
already prevents a stale valid archive from being committed as new content
(2.7, "Stale valid content at the readback"). Deletion adds a new route to
such stale bytes, but no new rule is needed.

### 5.6 DELETE sequence and durability

DELETE of entry *E* on a mounted v2 catalog (after an upgrade commit if it was
v1):

1. **MARK.** Commit MARK for *E* (target slot, barrier 2, mirror, barrier 3).
   From the first write, META shows *E* as `deleting` on the control lane
   (the LIST record has no state field), and BIND refuses it. LIST and the
   INFO `count` keep counting *E* until REMOVE. A crash leaves *E* `sealed` (not reached) or
   `deleting` (reached), never half.
2. **Erase.** *[proposed; owner question V4]* Only when every valid copy on
   the device lists *E* as `deleting`: if the other slot's valid copy still
   lists *E* as `sealed` (a reboot between MARK's barriers 2 and 3), first
   commit SYNC, so that no single surviving copy can show *E* `sealed` over
   zeroed bytes. Then write zero sectors over *E*'s
   whole extent through the erase path, which accepts only sectors inside the
   extent of the single `deleting` record of the committed catalog, then flush
   (barrier E). A failed write or flush is a safe-path trigger (2.7). The
   record stays `deleting`.
3. **REMOVE.** Commit REMOVE (barriers 2 and 3). Reply `Ok`.

A crash or reboot after step 1 leaves *E* `deleting`. The service never
resumes on its own (a mount writes nothing). The portal sees the state through
META and repeats DELETE, which starts at step 2. Step 2 is idempotent. An
erase is at most 1027 sector writes and one flush inside one request; the
service serves no other lane meanwhile, as for any request (2.2). While
*E* is `deleting`, CREATE, SEAL, VERIFY and READ of other entries proceed; a
SEAL commit carries the `deleting` record unchanged.

**Reuse after REMOVE (I4).** After step 3 both copies omit *E* (barrier 3).
After a crash between REMOVE's barriers 2 and 3, the other slot's copy still
lists *E* as `deleting`, and 5.5 keeps *E*'s sectors out of every free region
until the next commit has overwritten that slot. Combined with the
one-transition mount rule, no single damaged or stale copy can list an extent
that a later entry overwrote. Damage to both copies is outside this guarantee,
as in v1 (2.6, N8).

**INFO free sectors** become `formatted − 16 − Σ ⌈length/512⌉` over every
record, `deleting` included. That is the total free space, not the largest
reservable region. *[proposed]* CREATE's `Full` still decides by 5.5. For a
v1 portal this changes the meaning of `aux` from "the free tail" to "total
free"; such a portal may expect space that CREATE then refuses with `Full`,
which it already handles.

**Cursors.** REMOVE moves later records down one index, so a LIST or META
cursor taken before a DELETE can skip one entry; a portal that needs a
complete list restarts at cursor 0 after a DELETE.

## 6. Retention guard and "proven restorable"

The decision says a retention policy must never silently delete a user's last
proven restorable backup. The catalog stores no verify result, and VERIFY
writes nothing (2.3). *[proposed; owner questions V5 and V6]*

- **Proof.** A successful VERIFY of entry *E* in the current service instance
  allows one RECORD-VERIFY (op 14) for *E* on the control lane. RECORD-VERIFY
  without such a VERIFY in the same instance returns `Access` -5. It commits
  RECORD (5.3), which stores the new sequence as *E*'s verified sequence.
  "Proven restorable" then means "a VERIFY that this destination service ran
  in full over the stored bytes passed `archive::verify` and was recorded".
  It is a media and format check, not a completed restore (V6).
- **Guard.** DELETE carries an origin in `count`: 1 = the user confirmed this
  entry in the trusted dialog, 2 = a retention policy; other values are
  `Invalid` -1 at check 2. A policy DELETE of the only `sealed` record with a
  verified sequence ≠ 0 (in the owner's scope, once V1 is decided) returns
  `Access` -5 and changes nothing; a `deleting` record does not count as a
  remaining proven backup. A user DELETE is not
  refused by this guard; the dialog warns. The service holds no policy and
  never issues a deletion itself.

The origin value is the trusted portal's claim, like the time (4.2). The guard
protects against policy code in the portal that is wrong, not against a
compromised portal.

## 7. Owner binding hook (O12)

The owner table (5.1) reserves 16 bytes per entry for the O12 owner
principal. In this revision it must be zero and no operation reads it. One
catalog version change should carry time, deletion and owner together, so
that destinations do not pass through a v2 without owners and a v3 with them.
*[proposed; owner question V1]*: implement the v2 codec only when the owner
field's meaning is fixed; until then, host-level work on allocation (5.5) and
the transitions (5.3) may use a zero owner field.

## 8. Negative test matrix

All are host tests on `services/destination` with the simulated disk, plus the
image oracle (`tools/destination_disk.py`) and the R3 reverse differential,
both extended to v2. Native stages are a later route step and need the CI
owner's stage-count pins (O6).

| ID | Property | Setup | Expected | Kills |
|---|---|---|---|---|
| N-V1 | I1 | two v2 copies, v1 service | `Corrupt`, no write | a v1 decoder that ignores the version |
| N-V2 | v2 on v1 | two identical v1 copies, v2 service | mounted; META source 0 for every entry; no write at mount | an upgrade written at mount |
| N-V3 | I2, 3.3 | crash after upgrade barrier U2 (v2 *n*+1, v1 *n*); mount with v1 and with v2 | v1: mounts the v1 copy, entries and `next_free` equal; v2: mounts *H*; the combined upgrade-and-SEAL mutant leaves a v1 mount that reserves a listed extent | a combined upgrade and content commit |
| N-V4 | 5.2 | v2 copies with: overlapping extents; an extent ending past `formatted`; `next_free` ≠ highest end; two `deleting`; time ≠ 0 with source 0; source 3; negative time; verified sequence below the sealing sequence or above the copy's; non-zero owner or padding; a non-zero metadata or owner entry at index ≥ count | each invalid | each dropped clause |
| N-V5 | 5.2 | valid copy with a gap and a newer entry below an older extent | valid | a v2 validator that keeps "ascending first sector" or contiguity |
| N-V6 | 5.3 mount | valid pairs: *H* two commits ahead; *H* = REMOVE of *L* but with a second record dropped; *H* = MARK of two records; *H* = SEAL with next entry ID + 2; both v2 with *H* = *L* with metadata zeroed (an UPGRADE between v2 copies); *H* v1 at *L* v2's sequence + 1 | `Corrupt` each; the legitimate one-transition pair of each kind (UPGRADE only from v1, SYNC between v2) mounts *H* | the v1 prefix rule kept for v2 (it accepts the first case); a rule that accepts any shorter *H*; UPGRADE accepted between v2 copies |
| N-V7 | pre-write check | test hook alters *N* for each transition (as N29) | `Corrupt` -18 for that request, no catalog write, no `Corrupt` mount state | a commit without the transition check |
| N-V8 | 5.6 crash cuts | cut DELETE at: MARK copy 1 written; MARK barrier 2; MARK barrier 3; mid-erase; barrier E; REMOVE copy 1; REMOVE barrier 2; REMOVE barrier 3; and a reboot between MARK barriers 2 and 3 followed by a repeated DELETE, then loss of the `deleting` copy | after each, the next mount shows *E* `sealed` or `deleting` or gone, never another state; a repeated DELETE completes; no other extent changed (oracle); after the reboot case the repeated DELETE commits SYNC before any erase write, so no surviving copy shows *E* `sealed` over zeroed sectors | an erase before MARK; REMOVE before barrier E; an erase without the every-copy precondition |
| N-V9 | I4 | crash between REMOVE barriers 2 and 3; mount; CREATE; APPEND across *E*'s old extent | CREATE does not reserve *E*'s sectors until a further commit has overwritten the stale slot | 5.5 without the other-slot clause |
| N-V10 | I3 | entry-path write at `region_end`; erase-path write one sector outside the extent; any write to sectors 0–15 | refused before the device (write counter unchanged) | the v1 `>= next_free` rule alone |
| N-V11 | 5.4 DELETE | DELETE while bound on verify/restore lane; while another entry `deleting`; unknown ID; on an operation lane; in failed-mount, `Corrupt`, blank | `Busy`, `Busy`, `NotFound`, `Access` (check 6), mount-state replies; nothing written | a DELETE that skips the binding check |
| N-V12 | 5.4 BIND | BIND verify/restore to a `deleting` entry | `NotFound` -19 | BIND that checks only presence |
| N-V13 | 5.5 | free gap of 1026 sectors below the tail; gap of 1027 | the 1026 gap is skipped; the 1027 gap is reserved; APPEND past `region_end` gives `Full` and appends nothing | a first-fit rule without the size bound |
| N-V14 | 4.1, 5.4 | SEAL with time ≠ 0 and source 0; source 3; at feature level 2 and as v1 (aux 0, count 0) | `Invalid` for the first two; the v1 form stores "unknown" | a SEAL that stores an unchecked source |
| N-V15 | 6 | RECORD-VERIFY without VERIFY in this instance; after a failed VERIFY; after reboot | `Access` -5, no write | a RECORD that trusts the request |
| N-V16 | 6 guard | policy DELETE of the only verified `sealed` entry; the same while a second verified entry is `deleting`; user DELETE of it; policy DELETE of an unverified entry | `Access`, `Access`, `Ok`, `Ok` | a guard that ignores origin or verified state; a guard that counts a `deleting` record |
| N-V17 | I6 | two entries whose times are reversed relative to their IDs | LIST, META, allocation and every rule ignore time; order by ID | any use of time for order |
| N-V18 | O16/O18; 5.3 replies | failed write at barrier 2 of REMOVE, MARK, RECORD, UPGRADE before a SEAL, and SYNC; remount reads *C*, *N*, or a third valid catalog; a failed erase write | reply `Unknown` -16 (erase: `Io` -12 or `Unknown` -16) and safe path; not done, done, failed-mount; after a failed UPGRADE before a SEAL the binding resolves *not sealed*; a SEAL refused at step 3 writes no UPGRADE | a resolution that compares only records; an UPGRADE before the refusal checks |
| N-V19 | 5.4 safe path | after a safe-path trigger: DELETE, RECORD-VERIFY; META | `Access` -5 with no device request; META runs | a safe path that admits DELETE or RECORD-VERIFY |
| N-V20 | 5.4 wire and lanes | SEAL with a negative time, and with time 0 and source 1; DELETE with origin 0 and 3; META with cursor ≥ 16; META on an operation lane (verified field); INFO `offset` in each mount state; DELETE and RECORD-VERIFY on an operation lane | `Invalid`; `Invalid`; `Invalid`; verified 0; `offset` 2 in every state; `Access` -5 (check 6) | each dropped field rule; a lane rule without the new control operations |
| N-V21 | 5.4 FORMAT | FORMAT by a v2 service, then mount with a v1 service and a v2 service | both mount the empty v1 catalog | a FORMAT that writes v2 (if V2 confirms the proposal) |

## 9. Owner questions

- **V1** (O12 coupling): does one catalog version change carry time, deletion
  and the owner field together, so that the v2 codec waits for the owner
  field's meaning (proposed), or does v2 ship time and deletion first, with a
  later v3 for owners? Option (b) costs a second upgrade commit and a second
  downgrade boundary.
- **V2** (downgrade): is it acceptable that after the first v2 content commit
  a v1-only build sees the destination as `Corrupt` and writes nothing
  (proposed), or must v2 keep a v1-readable form, which section 5's deletion
  and gaps exclude? With the same magic (proposed) or a new magic?
- **V3** (time source): the portal supplies backup time and source in SEAL,
  and the service gets no clock (proposed); or the service gets a time
  capability. Which component defines "system clock" and "synchronised" for
  source values 1 and 2?
- **V4** (erase): does DELETE overwrite the extent with zeros before removing
  the record (proposed: v1 archives are unencrypted personal data, and
  "deleted" should mean the bytes are no longer readable through the device),
  or only unlink the record? Zeroing a virtual or flash device does not show
  physical erasure *[unproven]*, and no claim beyond "reads back as zero
  through the device" is made.
- **V5** (proof): is "proven restorable" a recorded, passing in-service VERIFY
  (proposed, section 6), or a completed test restore, which the destination
  service cannot observe?
- **V6** (guard): does the service enforce the last-verified-entry guard for
  policy deletions (proposed), or is it left to the portal? Which entries
  count: the owner's own (needs V1) or all on the destination?
- **V7** (policy DELETE): who may issue a DELETE with origin 2: the trusted
  dialog component running an explicitly configured retention rule on the
  control lane (proposed; no new component gets a control right), or a
  separate policy component with a right of its own (a new trust boundary that
  needs its own design)?
- **V8** (bound entries): is a DELETE of an entry bound on any lane, including
  an export lane that sealed it, refused with `Busy` until the dialog unbinds
  (proposed), or does DELETE revoke those bindings itself? Is the proposed
  voiding of a remembered VERIFY permission for a removed ID (5.4) accepted?
- **V9** (reuse threshold): does CREATE reuse only gaps of at least one
  largest archive (1027 sectors; proposed, simple and always safe for any
  archive), or any gap with `Full` at its end, at the cost of failed exports
  that a larger region would have taken?

## 10. Route and evidence

1. Owner answers V1–V9.
2. Independent AI re-review of revision 1, then a revision if needed.
3. Host implementation in `services/destination`: v2 codec and validator,
   transitions and pre-write check, v2 mount rule, allocation, DELETE, META,
   RECORD-VERIFY, and host tests N-V1 to N-V18. Oracle and R3 differential
   extended to v2.
4. Native stages after the CI owner's pins (O6). R8 preconditions are
   unchanged.

A specification is not an implementation. No criterion changes on this text.

## 11. Review record

CATALOG-V2-REVIEW-1 (independent AI review of revision 0, 25 November 2026,
REQUEST_CHANGES; citations and byte arithmetic confirmed). Resolution in
revision 1:

| Item | Severity | Resolution | Where |
|---|---|---|---|
| 1 | required | DELETE and RECORD-VERIFY refused with `Access` in the safe path; no UPGRADE or SYNC there; N-V19 | 5.4 |
| 2 | required | the changes to 4.1, check 6, `permitted` and N31 stated; who issues a policy DELETE is V7 | 5.4; 9 |
| 3 | required | bound-entry rule, sealed export lane counted as bound, remembered permission for a removed ID stated as proposals under V8 | 5.4; 9 |
| 4 | should-fix | erase only when every valid copy lists `deleting`; SYNC transition; N-V8 cut | 5.3; 5.6; 8 |
| 5 | should-fix | UPGRADE only from v1 to v2; other mixed pairs `Corrupt`; N-V6 cases | 5.3; 8 |
| 6 | should-fix | replies after every failed commit or erase; UPGRADE after all refusal checks and SEAL step 3; N-V18 extended | 5.3; 8 |
| 7 | should-fix | guard counts only `sealed` verified records; N-V16 case | 6; 8 |
| 8 | should-fix | the service keeps the other slot's decoded copy; validity per own version; threshold user-visible, V9 | 5.5; 9 |
| 9 | should-fix | state only in META; LIST and INFO count include `deleting`; INFO `aux` meaning for v1 portals | 5.6 |
| 10 | should-fix | FORMAT writes v1 (proposed under V2); N-V21 | 5.4; 8 |
| 11 | should-fix | ADR-050 and ADR-057 cited; source 1 is `UnauthenticatedRtc`; rationale narrowed | 1; 4.2 |
| 12 | should-fix | index ≥ count zero rule moved into 5.2; N-V4 case | 5.2; 8 |
| 13 | should-fix | wire cases in N-V20; INFO `offset` in every mount state | 3.4; 8 |
| 14 | optional | cursor skip, reboot between U2 and U3, erase bound stated | 5.3; 5.6 |
