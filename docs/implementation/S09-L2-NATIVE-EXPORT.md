# S09 — Native L2 export and cold restore through the archive disk (phase 3)

**Status: revision 3. Slice A (5.1) is on `main` (#477), slice B (5.2) on
`main` (#482), slices C and D (kernel cases 25–28, the 30-stage native gate)
on `main` (#490). Slice E (section 4.1: a power cut inside the L2 restore,
kernel cases 29 and 30, the 32-stage native gate and both stage-count pins)
is implemented on `claude/s09-l2-restore-cut`
([README](../verification/s09-l2-restore-cut-20261124/README.md)).**

- **Branch:** designed on `claude/s09-l2-native-export`; slice B lands from
  `claude/s09-l2-slice-b-main` on top of `main`.
- **Review:** revision 0 was reviewed by an independent AI reviewer, verdict
  REQUEST_CHANGES
  ([SPEC-REVIEW-1.md](../verification/s09-l2-native-export-20261123/SPEC-REVIEW-1.md)).
  Section 11 records how each required item is resolved.
- **Evidence levels:** those of `docs/FULL-TEST-RELEASE.md`. A host test, an AI
  review or a design is not native acceptance and not a human audit.

## 1. Goal

The S09 assignment of 19 November 2026 asks for:

- an independent native recovery copy on a second virtual disk;
- an export from the volume service;
- verification with a host oracle;
- a cold restore without the source disk;
- a byte-exact comparison, including revisions and timestamps;
- negative tests.

What exists today:

- **v1 native stages** (`archive-disk-*`, 7 stages) cover the second disk, the
  export, the cold restore and the negative cases. They use archive format v1
  and the v1 restore, which reassigns revisions and stamps, as
  `S09-DUAL-DISK-FIXTURE-REQUEST.md` records.
- **L2** (`S09-L2-PROVENANCE.md`, #411) keeps the source revisions and logical
  timestamps as immutable provenance. Natively, only the INSTALL operations
  are exercised, by a fixed fixture (`install`, `install-recover`,
  `install-cut`). `export_v2` and `restore_native_l2` have no native caller.

Phase 3 connects the two. It adds native stages in which:

1. the volume service is exported with archive format v2 (`export_v2`) to the
   test-only device-1 disk;
2. that disk is restored cold with `restore_native_l2` into a fresh volume,
   with the source disk absent;
3. a host oracle compares the source, the archive and the restored volume
   byte for byte. This covers content, names, lengths and metadata, and the
   source revisions and logical timestamps, which survive as provenance.

## 2. What "byte-exact including revisions and timestamps" means here

Under rule R (`S09-L2-PROVENANCE.md` section 1), an L2 restore does **not**
reproduce the source's live revision numbers or stamps. Live values are
reassigned by the destination, and the source values are kept as provenance.
For every present object over its replayable prefix, the oracle requires the
following.

| Value | Source of truth | Destination after L2 |
|---|---|---|
| Content of every version | source volume (frozen v2 reader) | byte-equal (live data) |
| Version names and lengths; tags, kind, collection | source volume | equal (live data) |
| Object revision | source STAT | equal to `origin_revision` (provenance); live revision is `n` |
| Object `created`, `modified` (logical clock) | source table | equal to `origin_created`, `origin_modified`; live values equal the destination clock |
| Per-version revisions | source table | equal to `origins[i]`; live values are `n-i` |
| Collection index, name | source table | equal (live data) |
| Collection revision | source table | equal to the collection origin; live revision is 1 |

The archive carries no per-version timestamps and no wall time (format v2,
`S09-ARCHIVE-V2.md`), so none can be compared. Logical stamps exist only per
object.

The following are not restored by design, and the oracle checks that they are
absent rather than equal:

- versions after the first revision gap (`omitted`);
- tombstoned and absent slots;
- deleted collections;
- the source clock, generation and request ids.

**Discrimination invariant.** A provenance check proves nothing when the
source value happens to equal the value the destination assigns: a path that
wrote live values into the origin fields would then pass. The host oracle
therefore computes every compared pair (origin versus live) from the
exported archive and `expected_l2`. It **fails the stage as
non-discriminating** if any pair is equal. This makes the fixture's
suitability a checked property, not an assumption.

## 3. Source fixture

### 3.1 Base

The parent is the `archive-export` image (review item 1): the `recover`
fixture after the v1 export, with the device-0 archive range reset to
canaries, exactly as `archive-disk-export` prepares it. Domain 0 contains:

- object 0 as a revision-4 tombstone;
- objects 1–5 present, each with an empty history;
- collections (0,1) `Werk` and (0,2) `Privé`, both at revision 1;
- clock 57.

Object 3 is at object revision 42. Its version revision is 41 (RESTORE sets
41, and PURGE keeps the current version), and its fixture content revision
is 38. Revision 0 of this spec wrongly stated the version revision as 42.

On this base alone, five values collide with the values the destination
would assign (review item 4):

- both collection revisions (1);
- object 5's revision (1);
- the version revisions of objects 1, 2 and 5 (1);
- object 3's `created` (5).

### 3.2 Writable source step (adopted from optional item 1)

`l2-export` mounts disk 0 **writable**. Before it exports, the portal (a
domain-0 client) applies a fixed mutation set:

- **S1.** Two consecutive zero-length WRITEs to object 2 (`Leeg`). Object 2
  is at object revision 2 with version revision 1 (`normal_a` writes it at
  revision 1, and a META makes revision 2). The two WRITEs give it versions
  4, 3 and 1, current first. The replayable prefix is therefore 4, 3
  (`n = 2`, a native multi-version chain with `origins[1]`), and version 1
  lies behind a revision gap: `export_v2` omits it (`omitted = 1`). This
  covers R2 natively in the export (word 544). It was found while writing
  slice B; slice C confirms it on the host. S1 costs no chunk, because
  `chunks_for(0) == 0`, so the replay stays at 24 ≤ `GUARANTEED_FREE` (review
  item 5).
- **S2.** A META on object 5 (a tags change). Its revision becomes 2, and its
  `modified` moves to the new clock.
- **S3.** A rename of collections 1 and 2 through COLLECTION with the expected
  revision. Their revisions become 2.
- **S4.** Creating collection 3 and renaming it once, to revision 2. Three
  collections are then installed first, which moves the object installs to
  clocks 4–8 and separates object 3's `created` (5) from its live value (6).

- **S5** (added in slice C). On objects 1 and then 5: a zero-length WRITE,
  then RESTORE of the older version (selector 1). Each object then has a
  two-version chain, and no chunk is used.

Slice C computed all compared pairs on the measured `archive-export` image
(the discrimination invariant in section 2). With S1–S4 alone, two pairs
collided: the version revisions of objects 1 and 5 (origin 1, live 1). S5
removes them, so all 26 compared pairs differ. The oracle recomputes the
invariant on the real archive in every L2 stage, and a host test keeps S1–S4
alone as a negative case. A same-content WRITE to object 5 was not possible
(62 of 64 chunks are used), and a bare zero-length WRITE would have dropped
object 5's content from the replayable prefix.

The source step is part of `l2-export`, not a separate stage, so `len(PLAN)`
stays at 30. The disk-0 oracle for `l2-export` checks that the disk changed
by exactly the 11 commits of S1–S5 (section 6).

### 3.3 Expected destination state

This state is derived, not measured. `expected_l2` computes it from the
parsed archive, and the host tests pin it.

- Collections 1–3 are installed first, at clocks 1–3. Objects 1–5 follow in
  slot order, at clocks 4–8.
- Objects 1, 2 and 5 are installed with `n = 2`: live revision 2 and
  version revisions 2, 1 (origins 5, 4; 4, 3; 4, 3). Objects 3 and 4 have
  `n = 1`.
- `L2Report`: `restored = 0b0011_1110`, `collections = 0b111`,
  `committed = 8`, `skipped = 0`. The archive holds only the prefix, so the
  restore skips nothing; the gap shows as `ExportV2.omitted = 1` in
  `l2-export`.
- The table generation and the domain generation and clock are derived by
  `expected_l2` from `service.rs`. This spec does not claim that they
  coincide (optional item 4).

## 4. Stages

All six stages run in the backup portal (slot 1, domain 0) and use device 1
(`00:06.0`, serial `NOS-S09-ARCHIVE`).

- **Case numbers** start at 25, because the U-track's U05 (#436) takes 23 and
  24 (agreed with the U-track on 23 November 2026).
- **Flow values** equal the case numbers. `flow()` passes unknown cases
  through unchanged, so no `flow()` arm is added.
- **Preparation** has one explicit branch per case (review item 2), so no
  case falls into the source-loss default by accident.

| `PLAN` entry | Parent | Case | Preparation: disk 0 | Preparation: disk 1 | Service | Device-1 grant | What the stage proves |
|---|---|---|---|---|---|---|---|
| `l2-export` | `archive-export` | 25 | `outside_archive(parent)`; no v1 reference | fresh canary disk (2048 sectors) | writable | R\|W\|F | After S1–S5, `export_v2` writes a v2 archive to device 1. It reads back through `verify_v2` equal to the export summary. (Revision 1 also required a refused write here; with a writable service that cannot hold, so slice B dropped it.) |
| `l2-restore` | `l2-export` | 26 | source loss: fresh empty image | parent's `archive.img`, `readonly=on` | writable | READ | Cold `restore_native_l2` into the never-used domain 0 returns the expected `L2Report`. The guest then reads every slot's `provenance()` and every collection's `collection_provenance()` and counts mismatches against the verified plan; zero are expected. |
| `l2-corrupt` | `l2-export` | 27 | source loss: fresh empty image | parent's archive with one content byte of the largest object flipped (offset computed from `parse_v2` and recorded); `readonly=on` | writable | READ | `restore_native_l2` returns `Corrupt` before any service mutation. Disk 0 stays byte-identical to the fresh empty image. |
| `l2-used` | `l2-restore` | 28 | keep the parent's restored `volume.img`, `readonly=on` (device level, added to `read_only()`; optional item 2) | keep the parent's `archive.img`, `readonly=on` | read-only | READ | After a cold boot, provenance read through `PROVENANCE_STAT` still matches the plan. A second `restore_native_l2` into the now-used domain returns `Stale` with `committed = 0`. Both disks are byte-identical. |
| `l2-restore-cut` | `l2-export` | 29 | source loss: fresh empty image (the `l2-restore` branch) | parent's `archive.img`, `readonly=on` | writable | READ | The same restore as 26, stopped by a real host kill inside object 4's INSTALL_COMMIT table copy (section 4.1). Disk 0 holds exactly the first six commits and a torn seventh copy; disk 1 is byte-identical. |
| `recover-l2-cut` | `l2-restore-cut` | 30 | keep the parent's cut `volume.img`, `readonly=on` (device level, added to `read_only()`) | keep the parent's `archive.img`, `readonly=on` | read-only | READ | After a cold boot, every planned object and collection is fully installed with provenance equal to the plan or exactly never-used; the guest reports the observed masks. A second `restore_native_l2` returns `Stale` with `committed = 0`. Both disks are byte-identical. |

Negative coverage:

- **Natively, in these stages:**
  - a corrupt archive (`l2-corrupt`);
  - a restore over an existing domain (`l2-used`, the conflict case);
  - a write by the portal outside the granted rights: a device-1 write
    under the READ-only grant (26, 27, 28) and a volume WRITE to the
    read-only service (28);
  - a revision gap: `export_v2` omits the version behind it (25). The guest
    requires word 544 (`omitted`) to be 1, and the host compares
    `VOLUME:ARCHIVE:SKIPPED` with its own omitted-version count;
  - the scope and escape probes, which apply to every device-1 grant
    (`ARCHIVE_ESCAPE`, the past-the-end probe).
- **On the host only** (already in `services/volume/tests`):
  - a v1 archive given to `restore_native_l2`;
  - a `TOMBSTONES` archive;
  - quota and `Full`;
  - an interrupted restore (no resume), except the power cut inside an
    INSTALL_COMMIT copy, which is native since slice E (section 4.1);
  - a lost INSTALL reply;
  - a revision gap inside an archive given to the restore (`skipped`, R2).

### 4.1 Power cut inside the L2 restore (slice E)

Slice E closes the first open item of section 10 with cases 29 and 30. The
semantics come from the host tests in `services/volume/tests/l2_restore.rs`:
an interrupted restore keeps its committed collections and objects, is never
resumed, never claims an uncertain object, and a second restore into the
same domain returns `Stale`. INSTALL_BEGIN and INSTALL_APPEND publish no
table (`service.rs`); a staged install lives only in service memory, so after
a cold boot the object being installed is exactly never-used.

**Cut point.** On the fresh image, the restore writes the one-sector format
marker and then one full 34-sector table copy per commit: collections 1–3
(commits 1–3), then objects 1–5 in slot order (commits 4–8). Case 26 pins
this count (`1 + 8·34`). Case 29 stops the guest when the table-sector write
count reaches `1 + 6·34 + 4` (`L2_CUT_TABLE_WRITES`), the fourth sector of
commit 7, which is object 4's INSTALL_COMMIT. The count is taken from boot of
a fresh image, which is equivalent to a base latched at the format marker.
By design, object 4's content (5000 bytes, two chunks) has then been appended
but not published. Neither the host oracle nor the kernel asserts that these
chunks are present on disk; the native check covers the six published commits,
the torn seventh table copy and the never-used remainder. The point is chosen so that the installed prefix includes
object 3 (five chunks; origin revision 42 against live revision 1), and so
that one object with written but unpublished content and one untouched
object remain.

**Deterministic outcome** (pinned by kernel and host):

- table generation 6, domain-0 clock 6, format 3;
- collections 1–3 and objects 1–3 installed exactly as `expected_l2` with the
  first six commits (provenance, live values, content);
- objects 4 and 5, every other slot and domain 1 exactly never-used;
- the seventh copy torn after exactly 4 sectors over the generation-5 copy;
- case 30: guest masks `restored = 0b0000_1110`, `collections = 0b111`,
  0 provenance mismatches, a second restore `Stale` with `committed = 0`.

Recovery itself is not new behaviour: the table choice is the existing
two-copy rule that `install-cut` → `recover-cut` already exercises.

## 5. Guest changes

### 5.1 A v2-aware archive reader (`services/volume/src/archive.rs`)

`DiskSource` computes the stored length from the v1 header only. It ignores
`version_count` (header bytes 14–15) and the v2 record sizes. A v2 archive
read through it is truncated, and `walk_v2` then fails with `Corrupt`.

Change (implemented in slice A):

- `DiskSource` reads the format version at header offset 8.
- For version 1 the computation is unchanged.
- For version 2 the stored length is
  `HEADER_V2 + 64·collections + 128·objects + 64·versions + TRAILER + content`.
  It is bounded by the range and by `MAX_ARCHIVE_V2`, as the range-adapter
  rule of `S09-ARCHIVE-V2.md` section 4.4 requires. Through `walk_v2` that
  bound is redundant (its header checks refuse such counts first); a raw
  `Source::get` caller sees it, and a host test pins it there.
- Any other version is `Corrupt`.

Effect on v1 callers (review item 6):

- For valid v1 archives, the bytes returned are identical.
- A header with an unknown version now fails in `get()` instead of later in
  `walk`. The status is still `Corrupt`, and the number of sector reads is the
  same.
- The existing v1 host tests pin this, together with a new test that reads a
  v1 and a v2 archive through the same source.

`restore_native_l2` calls `open()` twice and needs a fresh reader each time.
`DiskSource` borrows `&mut D`, so a closure cannot return one. The change
makes `DiskSource` own its `D` and adds `impl<D: Disk> Disk for &mut D`. That
keeps existing callers (`DiskSource::new(&mut disk, …)`) compiling unchanged,
except where a `&mut` parameter is passed by value twice: slice A found one
such call in the v1 portal (`verified_restore`), which now reborrows it.
The portal builds each reader from a new `NativeDisk { device }`;
`BlockDevice` is `Copy`.

### 5.2 Portal flows (`sdk/examples/volume-docs/src/app.rs`)

- **Binary.** The feature `l2-portal` (slice B) builds a separate portal binary. It
  links:
  - `export_v2`, `verify_v2` and `restore_native_l2`;
  - the install and provenance client methods;
  - the mutating client calls of S1–S5.

  It does not link the v1 export, restore or ustar flows.
- **v1 portal.** The v1 portal (`archive-portal`) keeps its flows, but it now
  links the changed `DiskSource`, so its size changes too and is measured
  together with the new one.
- **Exclusivity.** The two features are mutually exclusive (`compile_error!`).
- **Flows.** `l2_portal` runs the shared probes (701, 702) and dispatches
  case 25 to `l2_export`, cases 26 and 27 to `l2_restore` (27 expects the
  refusal) and case 28 to `l2_used`. Slot 2 runs the existing `phase(1)` /
  `wait_gate(2)` / `report(760)` arm.
- **Scratch.** The `MAX_LENGTH` scratch stays on the 64 KiB private heap.
- **Stack.** `restore_native_l2` itself holds about three `ArchiveV2` copies,
  each about 2.2–2.3 KiB (an estimate from the field layout). Each case
  function is `#[inline(never)]`, so its `ArchiveV2` copies (including the
  inlined `restore_native_l2`) leave the stack when it returns; none stays in
  the frame of `run`. The stack fault recorded in `S09-L2-PROVENANCE.md`
  section 11 shows the risk; slice B measured the bound.
- **Evidence words** (new; inside the 1024-byte startup area; none collides
  with 40–512 or with U05):

  | Word | Value |
  |---|---|
  | 520 | `L2Report.restored`; case 30: the observed mask of fully installed objects |
  | 528 | `L2Report.collections`; case 30: the observed mask of installed collections |
  | 536 | `L2Report.committed`: confirmed INSTALL_COLLECTION and INSTALL_COMMIT commits only (since #477 an ambiguous COMMIT is not counted) |
  | 544 | `L2Report.skipped`, or `ExportV2.omitted` for case 25 |
  | 552 | provenance mismatches, 0 expected |
  | 560 | archive length in bytes |
  | 568 | result status of the restore attempt (26–28) as `Status as i32`, sign-extended to 64 bits; 0 is both `Ok` and "not written", so case 26 is judged by words 520–552, and the kernel must require a non-zero word 568 equal to `Corrupt` (27) or `Stale` (28) |

### 5.3 Kernel test profile (`kernel/src/m2/m3/volume.rs`, `kernel/Cargo.toml`)

This is test-profile code only. It changes no kernel ABI, no capability
semantics and no device ownership beyond the approved B1–B6 fixture
(`S09-DUAL-DISK-FIXTURE-REQUEST.md`). Cases 25–28 are added to:

- `case()`;
- `read_only()` (28) and `service_read_only()` (28);
- `archive_device()` (device 1);
- `device1_writable()` (25 only);
- `marker()` (`L2_EXPORT`, `L2_RESTORE`, `L2_CORRUPT`, `L2_USED`);
- the `archive_grant` gate, its rights and its scope. Rights are R|W|F for 25
  and READ otherwise; the scope is `ARCHIVE_SECTORS`;
- the `ARCHIVE_IO` arms of `finish()`;
- the device-1 evidence block.

The `ARCHIVE_IO` arms require the following (review item 3). The notation is:

- `a`: per-operation counts on the archive object (reads, writes, flushes);
- `d[lane]`: per-operation counts per device lane (0 = volume disk, 1 =
  archive disk).

| Case | Archive object and lane 1 | Lane 0 |
|---|---|---|
| 25 | `a` all non-zero; `d[1] == a` | writes allowed; service commits equal exactly the 11 S1–S5 commits; table writes 11 and broker table-sector writes 11 · 34 (host) |
| 26 | reads only; `d[1] == a` | writes allowed (the service installs). Service commits 8 and table writes 8; broker table-sector writes pinned as in `check_install` (optional item 3) |
| 27 | `a[0] > 0`, `a[1..] == [0, 0]`, `d[1] == a` | `d[0][1] == 0`; `d[0][2]` equals the startup flush of a writable `Service::recover` (plus SHUTDOWN, if one is sent); `SERVICE:COMMITS == 0`; `TABLE_WRITES == 0` |
| 28 | reads only; `d[1] == a` | zero writes and flushes (device and service read-only) |

The ranges currently written as `7..=19` become an explicit set, so cases
20–24 keep zero archive I/O.

Kernel features: `test-volume-l2-export`, `-l2-restore`, `-l2-corrupt` and
`-l2-used`. Each depends on `test-volume-archive-device`.

### 5.4 Build (`tools/build.py`)

`VOLUME_L2_CASES` builds the app with `--l2-portal` into
`target/volume-docs-l2portal`. `sdk/examples/volume-docs/build.py` gains the
flag.

A host test asserts that `VOLUME_L2_CASES`, `VOLUME_ARCHIVE_CASES` and
`INSTALL_CASES` are pairwise disjoint. Once U05 (#436) lands, the test also
covers U05's cases (optional item 6).

## 6. Host oracle (`tools/run_volume.py`, `tools/volume_disk.py`)

**Runner bookkeeping.**

- The new rows go into `FEATURE`, `CASE_MARKER`, `ALLOWED_PARENTS`,
  `DISK_CASES`, `DISK1_READ_ONLY` (26, 27 and 28), `READ_ONLY` (28) and
  `build.VOLUME_FEATURES`.
- The preparation branches are the explicit per-case branches of section 4.
  They are tested on the host, so a case missing from a branch fails before
  any QEMU run.
- `recover-cut` appears four times in `PLAN`, and in-place stages share one
  image directory. The new stages therefore rely on the `last-stage.json`
  parent check (`run_volume.py:250-269`), not on unique names. A host test
  walks `PLAN` and verifies that every new stage's parent is the stage that
  last wrote that directory.

**Oracle pieces** (implemented in slices C and D and passed in the local native gate; the hosted `m6e-native-volume` run on the final head is still pending, see section 9):

1. **v2 extraction.** `stored_archive_in` and `disk_archive` dispatch on the
   header version, through `volume_archive.stored_length_v2` and `parse_v2`.
   A v2 archive is never read with the v1 length.
2. **`expected_source(parent)`.** It applies S1–S5 to the frozen v2 analysis
   of the parent's disk 0. The `l2-export` disk 0 after the run must equal it
   (frozen v2 reader, `opaque` for object 2's zero-length versions). The
   device-0 archive range must still hold its canaries.
3. **`matches_volume_v2(archive, after)`.** The parsed archive must equal the
   analysis of disk 0 after the run:
   - header clock = domain-0 clock, and generation = domain generation;
   - `ACCOUNTED` set and `TOMBSTONES` clear;
   - slot 0 is a content-free record at revision 4;
   - per present object: revision, created, modified, tags, kind and
     collection;
   - per version: revision, name, length, FNV and the content bytes;
   - per collection: index, name, presence and revision.
4. **`analyse_v3(..., opaque=)`.** It gains the `opaque` content access that
   the frozen `analyse` has. For a slot in `opaque`, it returns the version
   bytes instead of searching the fixture. The search is the defect that
   would reject restored object 3.
5. **`expected_l2(archive)`.** It derives the destination state from the
   parsed archive alone:
   - collections are installed first, then objects in slot order;
   - live revision `n` and version revisions `n..1`;
   - `created == modified ==` the commit clock;
   - clock and generations as `service.rs` assigns them;
   - `installed = n`;
   - `origin = (archive revision, created, modified)`;
   - `origins = archive version revisions`;
   - collection origin = archive collection revision.

   It also returns the list of compared pairs used by the discrimination
   invariant (section 2).
6. **`flip_v2(archive)`.** It returns the offset of one content byte of the
   largest object, from `parse_v2`. The offset is recorded in the stage
   result, as `archive_flip` is today.
7. **Per-stage checks** (explicit `check_disk` branches; the v1
   `check_restored` fall-through is not reached by an l2 case):
   - **`l2-export`:**
     - disk 0 equals `expected_source(parent)`;
     - the archive on disk 1 equals `matches_volume_v2` against disk 0;
     - zero padding, and no write past the archive;
     - the device-1 write count equals the archive sectors;
     - guest word 560 equals the host length;
     - the discrimination invariant holds.
   - **`l2-restore`:**
     - disk 0 read with `choose(allow_v3=True)`; the state equals
       `expected_l2(parse_v2(disk 1))`;
     - content byte-equal to the archive versions, hence to the source,
       because `l2-export` proved archive = source on the parent;
     - disk 1 byte-identical;
     - guest words 520–552 equal the expected report, with 0 mismatches;
     - the service and broker counters match (section 5.3);
     - the guest and host digests agree. The digest does not cover stamps or
       provenance, so these are compared field by field above.
   - **`l2-corrupt`:** disk 0 byte-identical to `create_image('empty')`; disk
     1 byte-identical to the flipped copy; guest status `Corrupt` and
     `committed = 0`.
   - **`l2-used`:** disk 0 and disk 1 byte-identical; guest mismatches 0;
     guest status `Stale` and `committed = 0`.
8. **Explicit source-loss check** (26, 27; review item 6). Disk 0 before the
   run equals `volume_disk.empty_image_bytes()` with a fresh identity. The
   QEMU command line has exactly the `volume` and `archive` virtio-blk drives
   (besides the pflash and ESP drives), and neither file is the parent's
   `volume.img`.
9. **Evidence keys** carry no digits (the parser accepts `[A-Z_:]` only). The
   kernel prints:
   - `VOLUME:ARCHIVE:RESTORED`;
   - `VOLUME:ARCHIVE:COLLECTIONS`;
   - `VOLUME:ARCHIVE:COMMITTED`;
   - `VOLUME:ARCHIVE:SKIPPED`;
   - `VOLUME:ARCHIVE:PROVENANCE_MISMATCHES`;
   - `VOLUME:ARCHIVE:LENGTH`;
   - `VOLUME:ARCHIVE:RESULT`.

**Host tests** (`tests/test_volume_runner.py` and a new
`tests/test_volume_l2_native.py`) feed synthetic images and archives through
each check. They include mutations that must fail:

- a flipped origin;
- a live value written into each colliding field of section 3.1;
- a missing collection origin;
- a v2 archive read with the v1 length;
- a disk 0 that is not empty for a source-loss stage;
- a mutation set that leaves one pair equal, which must trip the
  discrimination invariant;
- an l2 case that reaches the v1 fall-through.

## 7. Gate impact and protected pins

`len(PLAN)` goes from 26 to 30, and in slice E from 30 to 32. The two protected pins change in the same
pull request as the stage commit. The project owner instructed on
24 November 2026 that no integration coordinator exists and that S makes
this change itself; the integration request
(`S09-L2-NATIVE-STAGE-INTEGRATION-REQUEST.md`) is therefore superseded.

The added runtime is estimated at under 6 minutes (optional item 7). That is
four boots plus up to four kernel builds and one extra portal build, against
the 40-minute `m6e` job timeout. This is an estimate, not a measurement. The
last hosted 22-stage run took 6 min 21 s.

## 8. Write set

| File | Change |
|---|---|
| `services/volume/src/archive.rs` | `DiskSource` v2 length and ownership (5.1) |
| `services/volume/tests/*` | v1/v2 `DiskSource` tests |
| `tools/mutate_volume.py` | mutants for the `DiskSource` v2 length |
| `sdk/examples/volume-docs/src/app.rs`, `Cargo.toml`, `build.py` | `l2-portal` feature and flows (5.2) |
| `kernel/src/m2/m3/volume.rs`, `kernel/Cargo.toml` | test profile cases 25–28 (5.3) |
| `tools/build.py`, `tools/run_volume.py`, `tools/volume_disk.py`, `tools/test_volume.py` | 5.4 and section 6 |
| `tests/test_volume_*.py` | host oracle tests |
| `docs/implementation/S09-L2-NATIVE-EXPORT.md`, the integration request, `docs/verification/s09-l2-native-export-*/` | records |

`MPL-2.0-FILES.txt`, `docs/progress.json` and `docs/PROJECT-STATE.md` are left
to the lead.

## 9. Slices and acceptance

| Slice | Content | Acceptance |
|---|---|---|
| A | 5.1 reader and its host tests | `cargo test`/`clippy`/`fmt` for `volume-core`, both feature sets; `mutate_volume.py` caught |
| B | `l2-portal` build. Measure first: `.text` (64 KiB), `.data`+`.bss` (one page) and `.rodata` (32 KiB) for both portals, and the stack depth of the deepest flow. | Both portals link within every bound. If not, stop and revise 5.2. **Done:** measured in [SLICE-B.md](../verification/s09-l2-native-export-20261124/SLICE-B.md); every bound holds. |
| C | kernel cases 25–28, runner and oracle, host tests; S1–S4 finalised by the discrimination computation | `unittest test_volume_*.py` passes; `test_volume_harness` is consistent; the discrimination invariant holds on the computed fixture |
| D | native gate and pins | `tools/test_volume.py` gives `passed=True` with 30 stages; `test_ci_workflows` passes with both pins at 30. **Done locally:** see [README](../verification/s09-l2-slices-cd-20261124/README.md); hosted `m6e-native-volume` is the final evidence. |

Each slice gets an independent AI review before the next begins. Hosted CI
(`m6e-native-volume`) is the final evidence after the pins land.

## 10. Open items

- Closed in slice E: a native power cut during an L2 restore (section 4.1,
  cases 29 and 30). One cut point is covered natively: inside an
  INSTALL_COMMIT table copy. Cuts inside INSTALL_COLLECTION, between
  INSTALL_APPEND writes, or after the last commit remain covered only by the
  host tests and the generic table-recovery stages. Open hardening from
  review 2: make the native host check pin the interrupted slot and the
  installed prefix's chunk count itself, verify object 4's appended chunks
  in the data region, and use the object-style `(NotFound, 0)` test for
  never-used collections in `l2_mismatches(partial=true)`.
- A revision gap inside an archive given to the restore (`skipped`, R2)
  remains host-only: `export_v2` never writes such an archive, so a native
  stage would need a crafted archive disk.
- Slice B measured the size and a static stack bound
  ([SLICE-B.md](../verification/s09-l2-native-export-20261124/SLICE-B.md)). The
  stack bound covers direct calls only. The kernel reads word 568 as an `i64`.
- Closed in slice C: the refused device-1 write probe (step 702) is
  counted as `VOLUME:ARCHIVE:RIGHTS_REFUSALS` (0 for 25, exactly 1 for
  26–28); the kernel checks `omitted == 1` (word 544) for case 25; the host
  oracle requires every slot and collection outside the installs to stay
  exactly never-used after `l2-restore` and `l2-used`.
- The broker serves one block request at a time across both disks. In the
  first native run, an archive read by the portal during the read-only
  service's recovery failed that recovery with `Protocol` (-15) instead of a
  retryable `Busy`. `l2_used` now reaches a READY service first, as every
  other portal flow does. Whether the broker should answer `Busy` there is
  an open kernel question outside S09.
- The merge order against U05 #436: whichever lands second resolves the
  appended lines in `case()`, `read_only()`, `marker()`, `kernel/Cargo.toml`
  and `tools/build.py`.
- Human audit: not done.

## 11. Review record

**Review 1:** an independent AI spec review (read-only) of revision 0, at
`f6a07fe3`. Verdict: REQUEST_CHANGES. The verbatim text is in
[SPEC-REVIEW-1.md](../verification/s09-l2-native-export-20261123/SPEC-REVIEW-1.md).

| # | Required change | Resolution in revision 1 |
|---|---|---|
| 1 | `l2-export` cannot chain from `recover` | Parent `archive-export`; disk 0 is `outside_archive(parent)`; no v1 reference (sections 3.1 and 4). Unique-name claim withdrawn; a host test checks parents against `last-stage.json` (section 6) |
| 2 | Preparation branches unspecified; `l2-used` would get an empty disk 0 | Explicit branch per case, including the recorded v2 flip offset and keep-both-disks for `l2-used`; explicit `check_disk` branches (sections 4 and 6) |
| 3 | Case-27 lane-0 rule contradicts a writable service | Rule replaced by the exact startup-flush count, plus zero commits and zero table writes (section 5.3) |
| 4 | Source values collide with live values; object 3 version revision is 41 | Collisions listed (3.1); writable source step S1–S4 (3.2); discrimination invariant enforced by the oracle (section 2); host mutants per colliding field (section 6) |
| 5 | "Richer fixture not possible" is wrong | Zero-length versions cost no chunk; S1 gives a native 2-version chain plus a revision gap (3.2, corrected in slice B) |
| 6 | Text corrections | v1 portal size changes (5.2); scope of the v1 claim stated (5.1); source-loss drive check reworded (6.8); `mutate_volume.py` in the write set (section 8) |

Optional items 1–7 were taken over:

1. the writable source step (3.2);
2. a device-level read-only disk for `l2-used`;
3. pinned counters (5.3);
4. no claim that the generations coincide (3.3);
5. the measurement gate covers all linker limits (section 9);
6. U05 cases in the disjointness test (5.4);
7. build time in the runtime estimate (section 7).

**Slice E (revision 3):** cases 29 and 30 (section 4.1) were added on
`claude/s09-l2-restore-cut` by the builder of task S-20261124-3-B1. Before
implementation, a reviewer model checked the plan in an advisory capacity
(it moved the cut point from object 3 to object 4, so that the installed
prefix holds a multi-chunk object). Native evidence is recorded in the
[slice E README](../verification/s09-l2-restore-cut-20261124/README.md).

**Review 2 (slice E):** an independent, read-only AI code review of the merged
slice at `68596f82` (task S-20261124-4). It was static reading only; nothing
was executed. Verdict: APPROVE, no required changes. The verbatim text is in
[REVIEW-1.md](../verification/s09-slice-e-review-20261124/REVIEW-1.md).
Resolution of its findings:

| Finding | Resolution |
|---|---|
| "Appended but not published" is not asserted (section 5 of the review) | Section 4.1 now marks it as design inference |
| Optional 1: host check does not itself pin the interrupted slot or chunk count | Open (section 10); the host unit test `CutCheck.test_cut_state` pins only the installed prefix (objects 1–3) and the never-used set (objects 4–5) on the shared fixture, not which object the seventh commit targets |
| Optional 2: check object 4's chunks in the data region | Open (section 10) |
| Optional 3: partial collection check uses `revision == 0` only | Open (section 10); the host disk oracle covers the raw state |
| Optional 4: `L2_CUT_COMMITS`/`L2_CUT_TORN` duplicated in kernel and host | No change; drift fails closed on the host, and both sides cross-reference each other |
| Optional 5: gate exit status not captured in the slice E README | Closed by the hosted `m6e-native-volume` pass on the merged head `5d873a13` (HANDOFF of PR #492) |

An AI review is not a human audit.
