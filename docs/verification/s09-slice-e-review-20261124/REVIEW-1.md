# S09 slice E: independent review 1 (verbatim)

- Subject: squash merge `68596f82` of PR #492 (cases 29 `l2-restore-cut` and 30 `recover-l2-cut`).
- Reviewer: a fresh, read-only AI code-review agent (Rust reviewer role), task S-20261124-4. It received the diff, the spec and the evidence README, not the author's conclusions.
- Scope limit: static reading only; the reviewer executed nothing. An AI review is not a human audit.
- Resolution: section 11 of `docs/implementation/S09-L2-NATIVE-EXPORT.md` (Review 2).

The text below is the reviewer's report, unedited.

---

VERDICT: APPROVE

Read-only review of 68596f82. I ran nothing (no cargo, no QEMU) and re-ran none of the unit tests, so this is static reading only.

REQUIRED
(none)

Answers to the review questions, with evidence:

1. Cut point (robust enough)
- kernel/src/m2/m3/volume.rs:74 sets L2_CUT_TABLE_WRITES = 1 + 6*34 + 4 = 209. Line 1318 stops the guest after the 209th lane-0 table-sector write completes. The counter at lines 1312-1313 excludes device-1 writes (the `return` at about line 1308) and archive-key writes.
- INSTALL_BEGIN and INSTALL_APPEND publish no table, so a table write after commit 6 can only belong to the 7th publish.
- The host verifies the cut from disk bytes, not from the kernel's intent. tools/run_volume.py `check_l2_cut` (about line 1328) requires:
  - the state to equal `expected_l2(archive, 6)`;
  - at least one installed object, and not all planned objects;
  - a torn copy of exactly L2_CUT_TORN = 4 sectors (`torn_sectors`, line 945).
- If the kernel and host constants (6 and 4) drift, or the table-copy size changes, the stage fails closed.
- If the case never cuts, `marker()` returns `VOLUME:FAIL:CUT_NOT_OBSERVED`. Case 26 also pins the `1 + 8*34` count at kernel line 1565.
- Fixture drift: native `check_l2_cut` does not itself assert that the interrupted object is slot 4. A fixture with a different commit order would still pass natively, with the cut in a different (maybe weaker) object. The unit test `CutCheck.test_cut_state` (tests/test_volume_l2_native.py) pins installed [1,2,3] and never_used [4,5] on the shared `rv.L2_SOURCE_STEP` fixture, so such a drift fails the host tests. This is a minor residual (see OPTIONAL 1).

2. Recovery oracle (strong)
- Host: `check_l2_cut` runs `analyse_v3` and `check_l2_state` on the cut image.
  - Every planned object outside the first 6 commits must be exactly never-used (no present, revision, installed, origin or versions).
  - Installed objects must match field by field, including content bytes and provenance.
  - Generation, clocks and chunks_used must match.
- `check_l2` for recover-l2-cut additionally requires:
  - disk 0 `after == before`, with zero lane-0 writes and flushes;
  - the guest masks equal to the report;
  - the service generation equal to the cut state's;
  - the guest digests equal to the disk oracle.
- Kernel (case 30, volume.rs:1620-1626): `l2[..3] == [0b1110, 0b111, 0]`, mismatches `l2[4]==0`, status == -10.
- Guest (app.rs `l2_used`): `restored.count_ones() == listed`.
- Scenarios:
  - Losing object 4 entirely: it cannot be told apart from a correct recovery. Object 4 is meant to be absent, and the oracle expects exactly that.
  - Losing object 3: caught by the host disk oracle, the mask check and the kernel mask.
  - Wrongly keeping a half-committed object 4: caught by the mask (bit 4 would be set), by the generation (7 vs 6) and by the host oracle.
  - A recovered object with wrong provenance: caught by the guest mismatch count and by the host.
  - Writes outside the plan: caught by the `after == before` check and by `check_l2_state` never-used checks on all other slots.
- Unit mutants cover this: "no torn copy", "torn copy too short", "full restore", "one commit more", "object 3 without provenance", "partial object 4", "non-prefix", the service-generation mutant, and a flipped byte in `after`.

3. Chaining and parent binding (correct)
- ALLOWED_PARENTS (run_volume.py:92) and PLAN (test_volume.py) bind l2-restore-cut to l2-export and recover-l2-cut to l2-restore-cut.
- recover-l2-cut copies the parent's volume.img and archive.img (`prepare_disks`, around line 495). `after == before` and `disk1_after == disk1_before` enforce read-only.
- Host tests cover this: `Plan.test_every_parent_is_the_last_writer_of_its_directory` (plan[-6:], len 32) and `Preparation`, which checks that recover-l2-cut's prepared images equal the parent's. Source-fingerprint binding uses the generic gate.

4. Kernel code (no defect found)
- The `cut()` loop has a SAFETY comment and is a deliberate halt.
- Case 29 never reaches PASS: `marker()` returns FAIL for 29, and finish() has `_ => true` only for results it does not reach.
- Case 30 failures call `fail(...)` or take the l2_result path to a FAIL marker.
- `1 << (c.index - 1)` and `1 << o.slot` could overflow only for index 0 or slot >= 64. Fixture values are small and bounded; I did not check the type of `c.index` or `o.slot`.
- The only loops are bounded by the plan counts.

5. Claims beyond evidence (all small)
- README and spec say "object 4's content has been appended but not published". Neither the host oracle nor the kernel checks that object 4's chunk data is on disk. The 7th table copy is necessarily the INSTALL_COMMIT, but the appended-content part is design inference, not asserted.
- README "Deterministic ... measured on both runs below": the two runs are the fast loop and the gate, not independent hosts.
- The gate exit code was "not captured (background run)"; passed=true is inferred from the summary. This is disclosed.
- The 32-stage evidence is bound to ea59c6fc. The post-review tools edit changed the fingerprint, and this is disclosed. Hosted m6e-native-volume still has to cover the head, and the independent review is pending as noted.

6. Other
- None that is a real defect.

OPTIONAL
1. Make the cut point self-pinning on the host. In `check_l2_cut`, assert the interrupted object is the expected one, for example by reading the object slot of the next `expected_l2(archive, 7)` commit. Also assert a minimum chunk count for the last installed object, so a fixture change cannot silently weaken the cut while only unit-test fixtures notice.
2. Optionally have `check_l2_cut` verify that object 4's content chunks are present in the data region of `after` (unreferenced by the table). That would back the "content appended but not published" claim.
3. In `l2_mismatches(partial=true)`, a collection counts as never-used when `stat.revision == 0` alone, which is weaker than the object check `(NotFound, 0)`. The host oracle covers the raw disk, so this is minor.
4. Define L2_CUT_COMMITS and L2_CUT_TORN once. They are duplicated in kernel and host, and the host check is what makes drift fail closed. A comment cross-referencing the two (already present) is sufficient.
5. Capture the native gate's real exit status in the README on the final head, via the hosted gate.

Not checked: the hosted CI result; the (unchanged) full `analyse_v3` implementation; the types of `c.index` and `o.slot` in the shift expressions.
