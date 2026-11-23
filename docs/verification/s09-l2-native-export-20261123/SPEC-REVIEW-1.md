# S09 phase 3 spec review 1 (independent AI review, not a human audit)

**What was reviewed:** revision 0 of `docs/implementation/S09-L2-NATIVE-EXPORT.md`
(sha256 `5ff86f5e6a989e35dbf5fcab20df2ef303292096cb90f1e018c3ffeb26ea96f0`),
on branch `claude/s09-l2-native-export` at `f6a07fe3`.

**Reviewer:** a read-only general-purpose AI subagent, on 23 November 2026. It
read code only; nothing was built or run.

**Record:** the text below is its report, verbatim. Revision 1 of the spec
(section 11) records how each required item is resolved.

---

**Verdict: REQUEST_CHANGES.** I reviewed `docs/implementation/S09-L2-NATIVE-EXPORT.md` rev 0 against branch `claude/s09-l2-native-export` at `f6a07fe3`. I only read code. Nothing was built or run.

## Required changes

**1. `l2-export` with parent `recover` cannot chain as written** (spec §4 table, §6, and the parents in `S09-L2-NATIVE-STAGE-INTEGRATION-REQUEST.md`)
- `recover` and `archive-export` both run in place in the image directory that `normal` created. Only full/ustar, `RESTORED` and `DISK_CASES` create a new directory (`tools/run_volume.py:245-325`).
- After `archive-export`, `last-stage.json` in that directory says `archive-export`. The check at `run_volume.py:267-269` then rejects `(l2-export, archive-export)`.
- `tests/test_volume_harness.py:46-53` only checks static pairs, so this would first fail in slice D.
- The spec's claim "Stage names are unique, so `volumes{}` chaining is unambiguous" (§6) is false. `recover-cut` appears four times in `PLAN`, and in-place stages share one directory.
- The table's text "device-0 archive range reset to canaries" only makes sense for the `archive-export` image.
- Fix: use parent `archive-export`, as `archive-disk-export` does (`run_volume.py:74`). Disk 0 is `outside_archive(parent)`, and the v1 `archive_reference` is not used as a v2 oracle. Update `ALLOWED_PARENTS` and the integration request to match.
- Alternative: keep `recover`, but pin the `PLAN` position before `archive-export` and add a prep branch that does not call `stored_archive()` on canaries.

**2. The preparation branches are not specified, and `l2-used` would get the wrong disk 0** (§6, "Runner bookkeeping")
- In `DISK_CASES`, any case that is not named falls into the source-loss `else` branch (`run_volume.py:291-295`), which gives a fresh empty disk 0.
- For `l2-used` that means `restore_native_l2` succeeds instead of returning `Stale`.
- Fix: specify one branch per case:
  - `l2-export`: the export-style branch, without the v1 reference.
  - `l2-restore`: the source-loss branch.
  - `l2-corrupt`: the source-loss branch, plus a disk-1 flip at a v2 offset computed from the parsed archive. Object 5's content is the 65 536 bytes before the 16-byte trailer. Record the offset, as `archive_flip` is recorded at `run_volume.py:320-324`. This flip is listed nowhere in §6, "Pieces that do not exist yet".
  - `l2-used`: keep both disks, like `archive-disk-cut` (`:286-290`).
- `check_disk` (`:693-769`) also needs explicit l2 branches. Its fall-through `else` calls the v1 `check_restored`.

**3. The case-27 rule "zero lane-0 writes and flushes" contradicts a writable service** (§5.3 `ARCHIVE_IO` table)
- `Service::recover` with `read_only == false` flushes before it serves anything (`services/volume/src/service.rs:252-255`). That flush is published on lane 0, so `d[0][2] >= 1`.
- The existing restore arms 8–10 and 14 do not constrain lane 0 for this reason (`kernel/src/m2/m3/volume.rs:1376, 1385`).
- Fix: require `a[0] > 0`, `a[1..] == [0,0]`, `d[1] == a` and `d[0][1] == 0`. Add an exact flush count equal to the startup flush (plus SHUTDOWN, if one is sent), with `SERVICE:COMMITS == 0` and `TABLE_WRITES == 0`. Keep the service writable so the test still proves "refused before any write".
- No format marker is written, because `ensure_format_initialized` only runs on a mutation (`service.rs:1134`, `1409`). So the host check "byte-identical to `empty_image_bytes()`" holds.
- Case 28 is service-read-only and flushes nothing; its rule is correct.

**4. "Every source revision and stamp differs from the destination" is false, so the coincidence-proof claim fails** (§3)
- I derived the domain-0 clocks from `normal_a` (`sdk/examples/volume-docs/src/app.rs:508-671`; 57 commits, which matches `DOMAIN_CLOCKS`). Object 3's first write is commit 5, and `touch` sets `created` only once (`service.rs:778`). The destination installs collections at clocks 1 and 2 and objects 1–5 at clocks 3–7, so object 3 is installed at clock 5. Its `origin_created` 5 equals its live `created` 5.
- Object 5's source revision is 1, equal to its live revision 1.
- Both collections, Werk and Privé, have source revision 1 (`app.rs:581-592`). INSTALL_COLLECTION sets live revision 1 (`service.rs:1551`). Origin and live are equal.
- The version revisions of objects 1, 2 and 5 are 1. RENAME keeps the version revision (`service.rs:809-814`). They equal the live version revision 1.
- Factual error: object 3's version revision is **41**, not 42. RESTORE sets it to 41 (`service.rs:825-828`; `app.rs:552-556`), and PURGE does not change the current version (`service.rs:857-864`). Object revision is 42, version revision 41, fixture content revision 38.
- Consequence: a path that writes live values into the origin fields passes for the collection origins, object 5's `origin_revision`, the version origins of objects 1, 2 and 5, and object 3's `origin_created`.
- Fix: correct §3 and list the colliding fields. Either change the fixture (optional item 1) or state which fields are distinguished natively. The install stage (case 20) covers distinct origins natively. Add host mutants that target exactly these fields.

**5. "A richer fixture is not possible within the capacity" is incorrect** (§3, §10)
- The total of 24 chunks is right: 1 + 0 + 5 + 2 + 16 = `GUARANTEED_FREE` = 64 − 40.
- However, `chunks_for(0) == 0` (`services/volume/src/lib.rs:107`), and the `needed` sum counts only replayable versions (`archive.rs:1464-1468`). Zero-length versions therefore cost nothing.
- Two or three consecutive zero-length writes to object 2 ("Leeg", length 0) would give a replayable 2–3-version chain within 24 chunks.
- Fix: reword the claim to "not possible without a writable source step". Optionally adopt optional item 1.

**6. Smaller corrections to the text**
- "The v1 portal (`archive-portal`) is unchanged" (§5.2): it links the changed `DiskSource`, so its `.text` grows. The slice B measurement covers this, but the text must not say "unchanged".
- "v1 callers see identical bytes" (§5.1): this holds for valid v1 archives only. A non-v1 header now fails in `get()` instead of in `walk`. The status is still `Corrupt` and the read count is the same, so state the scope of the claim.
- The source-loss check "QEMU command line has exactly the two drives" (§6.6) is wrong as written. There are five `-drive` arguments: two pflash, the ESP and the two virtio-blk disks (`run_volume.py:337-354`). Say instead: exactly the `volume` and `archive` virtio-blk drives, and neither file is the parent's `volume.img`.
- `tools/mutate_volume.py` is missing from the §8 write-set table. It is only mentioned under the table.

## Optional improvements
1. **Add a writable source step** before export, or a writable `l2-export` with an explicit "disk 0 changed exactly by these commits" oracle:
   - make zero-length consecutive writes to object 2;
   - rename one collection so its revision exceeds 1.
   - This fixes items 4 and 5 together: native `origins[1..]` through the chain, a native `ExportV2.omitted`/R2 case, and origins distinct from live values.
2. **Make case 28 device-level read-only**: add it to `read_only()` and to the runner's `READ_ONLY`, as was done for `install-recover` (21). QEMU `readonly=on` is stronger evidence than a READ-only grant to the service.
3. **For `l2-restore`, also pin the service and broker counters**, as `check_install` does (`run_volume.py:996-1007`): commits 7, table writes 7, and table-sector writes 1 + 7·`TABLE_SECTORS`.
4. **State that `expected_l2`'s value 7 is both the table and the domain generation**, and that the two coincide only because n = 1.
5. **Extend the stage-1 measurement gate (§9, slice B)** to the linker's `.data`+`.bss` one-page limit and its 32 KiB `.rodata` limit (`sdk/examples/volume-docs/linker.ld`), not only `.text`. Note that `restore_native_l2` itself holds about three `ArchiveV2` copies, roughly 2.2 KiB each.
6. **Make the disjointness host test cover U05's `VOLUME_SETTINGS_CASES`** as well, once #436 lands.
7. **Include gate build time in the runtime estimate:** four extra kernel and portal builds, not only four boots. The `m6e` job timeout is 40 minutes.

## Claims I verified
- **`DiskSource`** ignores `version_count` and computes the v1 length only (`archive.rs:607-613`). The v2 length formula matches the header layout and `export_v2` (`:1316-1325`), and matches host `stored_length_v2` (`tools/volume_archive.py:146-152`).
- **Archive size:** `MAX_ARCHIVE_V2` is 494 416 bytes, which fits 1040 sectors. The actual archive is about 92 441 bytes, about 181 sectors.
- **`DiskSource` ownership change plus a blanket `impl Disk for &mut D`:** this compiles for every existing caller. No `DiskSource<'_, …>` annotation exists, and there is no conflicting `&mut` impl. `BlockDevice` is `Copy`.
- **Read-only `export_v2` sends only non-mutating operations:** DOMAIN_STAT, COLLECTION_STAT, STAT, VERSION_STAT and READ; PROVENANCE_STAT is not reached because every revision is at most 57. `never_used` and the `l2-used` `Stale` path send no mutation. A read-only `recover` neither flushes nor writes (`service.rs:248-256`; table recovery writes nothing). PROVENANCE_STAT is allowed on a read-only service (`:417`).
- **`l2-corrupt`:** the flipped object-5 content byte fails the first `verify_v2` pass (`archive.rs:899-900`) before any service request.
- **Expected destination state:** clocks 1–7, domain-0 clock 7, table and domain generation 7, `L2Report` restored `0b0011_1110`, collections `0b11`, committed 7, skipped 0 (`service.rs:1470-1566`, `table.rs:572`, `archive.rs:1487-1545`).
- **Device-1 grant** rights and scope stay within approval bounds B1–B6. The `ARCHIVE_ESCAPE` accounting is unchanged (`volume.rs:702-780`). Capability semantics are not widened.
- **`flow()`** passes unknown cases through unchanged.
- **Evidence words:** 520–560 lie inside the 1024-byte startup area; U05 uses none of them.
- **U05** (`01b63da5`) adds cases 23 and 24, including 24 in `read_only()`. It does not change `test_volume.py` or the pinned workflows, so 26 → 30 is the right pin target. With the pins unchanged, `test_ci_workflows` fails with `['26'] != ['30']`.
- **`analyse_v3`** searches fixtures from revision down to 1 (`run_volume.py:915`), which rejects restored object 3. The `opaque` change is needed.
- **Host helpers:** `parse_v2`, `stored_length_v2` and `empty_image_bytes` already exist.

Not verified: `.text`, stack, `.data` and `.rodata` sizes (nothing was built); the exact clock values from a real run (derived by hand from `normal_a`); and whether the kernel already uses words 520–560 anywhere beyond `AppEvidence`.
