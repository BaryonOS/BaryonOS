# SPEC-REVIEW-6: S09 destination service spec, revision 4

- **Reviewer role:** independent AI security reviewer. I did not write revision 4 or the owner-decision commit.
- **Date:** 23 November 2026.
- **Subject:** `docs/implementation/S09-DESTINATION-SERVICE.md` in worktree `s09-destination-service`, branch `claude/s09-destination-service`.
- **Diff base:** `dd2bb19d` (revision 3). The reviewed diff covers two parts:
  - the owner-decision commit `1735ab06` (O14, O15, O16), which no spec review had covered;
  - the uncommitted revision-4 working-tree changes.
- **Implementation compared:** PR #413 at core commit `e52cb865` (worktree `s09-destination-core`). This covers `services/destination` (the Rust core), `tools/destination_disk.py` with `tests/test_volume_destination_disk.py` (the oracle), and the three code reviews under `docs/verification/s09-destination-core-20261123/`. For the 2.3 citation I also checked `catalog_core::wire::valid_name` on `main` at `035b44e5`.
- **Input ledger:** `rev4-ledger.md`, rows P1–P13.
- **Status:** this is an AI review, not a human audit. It approves no owner item, and it is not evidence that any criterion is accepted.

## Verdict

**APPROVE. 0 required findings, 9 optional findings.**

- Every ledger row P1–P13 has exactly one resolution row in the section 9 "Revision 4 record", and each resolution is reflected in the normative text.
- The decided texts of O14, O15 and O16 keep their meaning.
- Both changes that are stricter than a decision are written as new proposed owner items: O17 for P1 and O18 for P2.
- The security floor is intact.
- The code claims I could check statically are true at `e52cb865`.

## Ledger coverage (P1–P13)

The check was mechanical: `grep '^| P[0-9]'` on the section 9 table finds exactly one row each for P1–P13. The ID sets compared against `dd2bb19d`:
- **N rows:** N1–N36 unchanged, plus N37.
- **O items:** O1–O16 unchanged, plus O17 and O18.
- **R steps:** R1–R8 unchanged.
- **L rows:** L1–L29 unchanged (29 rows).
- **K items:** K1–K5 unchanged.

Nothing was renumbered.

| Ledger | Resolution row | Where it shows in normative text (checked) | Holds? |
|---|---|---|---|
| P1 | proposed owner decision O17 | <ul><li>2.5 reply codes: the digest mismatch replies `Corrupt`, marked O17 (a).</li><li>2.7 step 3: content digest, marked O17 (a).</li><li>2.7 "Stale valid content": the limit is stated unconditionally, with a threat model.</li><li>2.7 portal paragraph: marked O17 (b).</li><li>5.4 and 8.1 O17.</li><li>N33 (iii) and the new N37.</li><li>R2, R7 and R8 rows.</li></ul> | yes |
| P2 | proposed owner decision O18 | <ul><li>2.7: committed-copy equality is clarified as 2048-byte equality, and a new O18 paragraph is added.</li><li>2.6 failed-mount state cross-references it.</li><li>N35 (e), 8.1 O16 status and O18, and R8.</li></ul> | yes |
| P3 | clarification | <ul><li>2.3 names the Cc set U+0000–U+001F and U+007F–U+009F, and says that Cf characters are not refused.</li><li>N34 lists the label cases.</li><li>8.1 O14 status.</li></ul> | yes |
| P4 | clarification | <ul><li>The conditional wording is gone: a grep for "if accepted", "under the alternative" and "proposed O14/15/16" leaves only the open items O11 and O12 and historical rows.</li><li>The R1, R2, R3, R7 and R8 status is brought to `e52cb865`.</li></ul> | yes |
| P5 | clarification | <ul><li>2.7 step 4 has a new paragraph explaining why the sealing-sequence clause implies the record-count clause.</li><li>The equivalent mutant is listed in N29 and N35.</li></ul> | yes |
| P6 | clarification | <ul><li>2.2 "Defensive rule, unreachable since O16".</li><li>2.6 failed-mount state.</li><li>The 2.7 safe-path paragraph refers to the defensive rule.</li><li>The mutant is listed as equivalent in N35.</li></ul> | yes |
| P7 | clarification | <ul><li>2.6 "Label content at mount": the "≥ 1" bound is implied.</li><li>N34 kill claim narrowed, with A10 and A11/X6 listed.</li></ul> | yes |
| P8 | clarification | N26 (d) gives a per-reset-model outcome, plus an invariant that holds under both models. | yes (see OPT6-4) |
| P9 | route or verification-plan item | <ul><li>N33 (i) non-zero stale bytes is required.</li><li>(ii) zero bytes is optional.</li><li>(iii) a stale valid archive goes to N37 (a), or stays a recorded limit.</li></ul> | yes |
| P10 | route or verification-plan item | R2 acceptance adds Clippy `-D warnings` on the non-test library for `x86_64-unknown-none`. This is a lead-owned workflow change. | yes |
| P11 | route or verification-plan item | <ul><li>Section 7 "Cross-check limit".</li><li>The R3 status row.</li><li>The N34 equivalents.</li></ul> | yes |
| P12 | route or verification-plan item | R2 acceptance adds a host test of the simulated disk alone for each of broker-contract properties 1–5. | yes |
| P13 | route or verification-plan item | <ul><li>The header and R1 say that this review must confirm RQ5-1 and RQ5-2.</li><li>That confirmation is below.</li></ul> | yes |

## P13: review-5 answers RQ5-1 and RQ5-2 confirmed

**Device results in 2.5 and 5.3 agree.** I read 2.5 "Device results" and "Reasons" and 5.3 in full.
- A submission result other than 0 or -8 (that is -13, -14 or another error) replies `Io` -12 for reads and for writes.
- 5.3 states the same for -14 and -13 after the boot mount, and says that a completion result follows the 2.5 table.
- Completions follow the table:
  - completion -12 replies `Io`;
  - completion -13, or a poll that is still -8 past the bound, replies `Timeout` on a read and `Unknown` on a write;
  - any other completion replies `Io` on a read and `Unknown` on a write.
- `NoDevice` has one meaning in both sections: a BIND reply in the mounted state, or an INFO status in the failed-mount state after a mount, remount or FORMAT met kernel -14.
- The FORMAT exception (failed-mount state, not the safe path) appears in both sections.
- The code matches this mapping:
  - `services/destination/src/device.rs` `run`: every non-0, non-(-8) submission gives `Io`; the bounds give `Io`, `Unknown` or `Timeout`.
  - `poll_error` gives a read `Timeout` only for -13; a write gives `Io` only for -12 and `Unknown` otherwise.
  - `Watch` records -14 for the failed-mount INFO.

**N36 matches the decided O15.** The row states only the O15 outcomes:
- (a1) `Invalid` -1, then `Busy` -21;
- (a2) `Invalid` -1, then `Full` -17, because the refused CREATE left the binding's CREATE unused;
- (c) CREATE `Ok`.

The kill is "a refused CREATE that uses up the binding (… returns `Access` -5)". This agrees with 2.2 check 6 (*[owner decision O15]*: only a CREATE answered `Ok` counts) and with 2.3.

**Result:** RQ5-1 and RQ5-2 are confirmed resolved.

## Owner decisions O14, O15 and O16

**`1735ab06` only relabels.** It changes "*[proposed; owner decision Oxx]*" to "*[owner decision Oxx, 23 November 2026]*" and records "Decided … accepted" in 8.1. The rule text is unchanged.

**The O16 definition was already part of the proposal.** The definition of "that copy plus exactly the provisional record" (the step-4 extension check plus the provisional entry ID) was already in revision 3 under *[Proposed; owner decision O16]* (`dd2bb19d` line 732). The owner therefore accepted that definition, and labelling it as the decision is correct.

**Revision 4 changes no decided meaning:**
- **O14:**
  - The 2.3 control-character set is the set of the `valid_name` that O14 already referenced: `char::is_control`, General_Category Cc, in `services/catalog/src/wire.rs:134-149` on `main`.
  - The "≥ 1" note only explains that the bound is implied. The bound stays a stated rule.
- **O15:** only the declined alternative is named as declined.
- **O16:**
  - The decided text is unchanged.
  - The 2048-byte equality for the committed-copy arm is a true clarification. `Catalog` derives `PartialEq` over every field, and the validity rules fix every padding byte and the checksum.
  - The stricter *sealed* resolution is proposed separately as O18, with "If O18 is declined, the decided O16 definition above applies unchanged", and N35 (e) lapses in that case. This is the correct treatment of P2.

**P1 changes no decision.** It is framed as a new owner item, O17, with (a), (b) and (c) options and a recommendation.

## P1 (O17): threat model, cryptography and conditional consistency

**The threat model is stated three times:** in 2.7 "Stale valid content", in 5.4 and in 8.1 O17. Each says three things:
- the checks detect accidental device or cache faults only;
- FNV-64 is not collision-resistant;
- a device that misbehaves on purpose stays out of scope, and a defence against it would need a maintained standard cryptographic hash under a separate decision, which this document does not make.

**No cryptography is invented.** FNV-1a 64 is never presented as protection against a hostile device.

**The conditional text is consistent in all four cases:**

| Owner answer | What the spec then requires |
|---|---|
| O17 (a) accepted | <ul><li>The service compares the digest at step 3 and replies `Corrupt` -18 with no safe path.</li><li>N33 (iii) becomes N37 (a); N37 (a) and (b) are R2 rows.</li></ul> |
| O17 (b) accepted | <ul><li>The portal compares the trailer of the SEAL `aux` or the LIST record with its own trailer.</li><li>N37 (c) belongs to R7.</li></ul> |
| (a) declined | <ul><li>N33 (iii) records the limit and is not counted as a pass.</li><li>N37 (a) and (b) lapse.</li></ul> |
| Both declined | The limit stays stated unconditionally in 2.7. |

**Effect on the code.**
- At `e52cb865` the SEAL reply `aux` is the trailer read back from the device (`service.rs`, `reply.header.aux = trailer` from `check_extent`). This matches the "only the SEAL reply's `aux` differs" claim.
- `n33_read_timeout_over_stale_valid_archive_recorded` reproduces the limit under the cache-dropping model with a 1024-byte, sector-aligned stale archive. It records the result and does not assert it as a pass.

## Security floor

- **Check order:** unchanged. No check is moved, and the O17 (a) mismatch is an operation result after check 7.
- **Fail-closed mount:**
  - The rule that a read error is a failed mount is unchanged.
  - O16 remains the decided rule.
  - O18 can only move more remounts from *sealed* to the failed-mount state with the binding *unknown*, which is the safe direction.
- **Safe-path latch:** unchanged.
  - The blank-plus-latch FORMAT refusal is kept as a defensive rule; it was not dropped.
  - Its unreachability is true at `e52cb865`:
    - `State::Blank` is entered only by `mount()`;
    - `mount()` runs at `boot()`, where `safe_path` is false, and in `catalog_failure`, which maps `Blank` to `Failed`;
    - `device_failure`, which sets the latch, is called only from APPEND, the SEAL tail write and barrier 1, all in the mounted state;
    - a FORMAT write failure sets `Failed` without the latch.
- **Capabilities:** no capability, right or lane changes.
- **On-disk bytes:**
  - No new trust in disk bytes. O18 compares the remounted bytes with a copy the service itself encoded and holds.
  - The O17 (a) digest is computed over bytes the service received over IPC.
- **IDs:** no O, N, R, K or L ID is renumbered.

**Removed text.** I listed every word removed since `dd2bb19d` with `git diff --word-diff=porcelain -U0`. No assertion, kill claim or "no device write" clause was lost. The N35 kill "a remount that ends blank with FORMAT permitted (a device write follows)" became "a remount that ends as a blank candidate (INFO reports `NotFound` -19 instead of `Io`)". That is a correct repair: with the latch set, the defensive rule refuses FORMAT, so the old observation could no longer detect a blank remount.

## Internal consistency

- **2.3 and 2.6:**
  - The label rule is the same in both: length 1..=32 at check 2, content by CREATE and, under O14, at mount.
  - The control-character set matches the N34 cases: C0, U+007F, U+0085, the surrogate, both ends of both bidirectional ranges, and the valid code points just outside them.
- **2.5 and 5.3:** they agree (P13 above).
- **N rows and their rules:**
  - The N29 and N35 count-clause equivalence follows from the validity rules: sealing sequences are strictly ascending and at most the copy's sequence.
  - The N34 equivalents A10 and A11/X6 match the code:
    - Rust `decode` checks `label_length == 0` before `valid_name`, which refuses an empty slice.
    - In the oracle, `decode_record` refuses lengths outside 1–32 before `valid_label`.
  - The N35 (c) committed-copy cases exist (`crash.rs:725-782`).
- **Route table:**
  - R3: "30 images, locally only". `CODE-REVIEW-O14-O16-2.md` records 30 agreeing images.
  - No workflow references the cross-check (`grep .github/workflows`).
  - No reverse differential exists (no match in the tests).
  - The verifier is a stand-in (`TestVerifier`, `tests/common/mod.rs:439`).
  - R2: the non-test library is only `check`ed for `x86_64-unknown-none` (`CODE-REVIEW-REV3.md` F7, gate 4).
- **Section 7 cross-check scope:** the claim "mount state, header and records" is true.
  - `RustCrossCheck.test_rust_images` compares the state and, for a mounted image, the destination ID, sequence, next entry ID, `next_free`, formatted count and every record field including the label.
  - The code reviews' "mount state only" refers to the invalid N34 images, for which only the state exists.

## Code claims at `e52cb865` checked statically

All of the following are true:
- **`valid_name`:**
  - The core `valid_name` (`lib.rs:65-80`) uses `char::is_control` and is called at mount (`catalog.rs:193`) after the length and padding clauses.
  - The oracle `valid_label` uses `unicodedata.category == "Cc"`.
- **N34 label cases:** 13 content cases each in `tests/catalog.rs` and `tests/test_volume_destination_disk.py`, including U+007F, U+0085 and the encoded surrogate.
- **Remount resolution** (`service.rs` `catalog_failure`):
  - exact `Catalog` equality with the committed copy resolves *not sealed*;
  - `extends_by_one` plus the provisional ID resolves *sealed*;
  - any other mounted catalog, or a blank result, gives `Failed { no_device: false }`.
- **`n26d_legitimate_waits_complete`:**
  - It asserts `Ok` and the entry listed when the cache is kept, and `Corrupt` with nothing listed and `dropped > 0` when it is dropped. The latch stays clear.
  - It does not assert the catalog-write count, as CODE-DELTA says.
- **`n33_read_timeout_with_unflushed_append`:** it uses non-zero stale bytes, a copy of the archive with one byte flipped.
- **Defensive guard:** kept with an "Unreachable since O16" comment (`service.rs:357-360`).
- **Count clause:** a test comment notes the equivalence (`crash.rs:709`).

## Findings

### Required

None.

### Optional

**OPT6-1 — 2.7 "Stale valid content", N37 (a): a digest mismatch does not latch the safe path, which the O9 reasoning leaves open.**
- **Where:** 2.7 "Stale valid content"; N37 kill "a mismatch that starts the safe path"; 8.1 O17.
- **Problem:**
  - The choice mirrors the revision-3 rule that a failed step-3 readback starts no safe path, so it does not weaken anything relative to revision 3.
  - The stated reason is "no write or flush is outstanding after barrier 1". In N37 (b), though, the mismatch itself shows that the device did not keep writes it reported as completed and flushed.
  - In N37 (a) the lost writes were accepted before a device reset, which is exactly the uncertainty O9 leaves unproven.
  - If such writes could land late, the next CREATE in the same boot reuses that extent. This is the hazard the safe path exists to exclude.
  - The N37 kill turns "no latch" into a test requirement.
- **Fix:**
  - In O17, state that not latching follows the revision-3 readback rule, and name the O9 dependency.
  - Optionally let the owner choose between latching or not latching on a digest mismatch, and on any device reset while unflushed entry writes exist.
  - Until then, phrase the N37 kill as "no safe path is required" rather than as a defect.

**OPT6-2 — 2.3 / R7: allowed format characters near trusted text.**
- **Where:** 2.3; R7.
- **Problem:** the spec now says that Cf characters outside the two bidirectional ranges are allowed. Examples are U+200E, U+200F, U+061C and the zero-width characters U+200B–U+200D. Directional marks can reorder adjacent neutral characters when a label is shown inline. "The dialog must not rely on a label" does not by itself cover the dialog's own surrounding text.
- **Fix:** add an R7 check that every label is rendered in an isolated bidirectional context, for example first-strong isolate, and visibly delimited from trusted text.

**OPT6-3 — N35 and the section 6 preamble: B6 attribution overstated.**
- **Where:** N35 ("B6, unreachable by that review's reading of the call sites") and the section 6 preamble ("B6 rests on that review's reading of the call sites").
- **Problem:** the stored reviews (`CODE-REVIEW-O14-O16-1.md` and `-2.md`, "Old guard") only report that the guard is kept with a comment saying it is unreachable. They contain no call-site analysis. B4, B6, A10 and A11 appear only in the O14/O16 round ledger, which is not stored on PR #413. The substance is correct: unreachability holds at `e52cb865` (Security floor, above).
- **Fix:** attribute the unreachability to this text's reading of the call sites of `mount` and `device_failure`, or store the round ledger.

**OPT6-4 — N26 (d) and 2.7: the per-model outcome assumes the extent holds no valid archive.**
- **Where:** N26 (d); 2.7 "Stale valid content"; N37.
- **Problem:**
  - The cache-dropped outcome "SEAL `Corrupt` -18 at the step-3 readback" holds only when the free sectors at the extent do not hold a valid archive of the same length. The test uses a freshly formatted, zero image.
  - Here the device's own reset happens during the barrier-1 submission, so the tail write is lost as well. A stale valid archive of the same length, aligned or not, would then seal. This is a second path into the P1 limit.
  - N37 covers only the reset before SEAL (sector-aligned) and the "stores nothing" disk.
  - The text also says the reset loses "the completed, unflushed last-sector write". It loses every unflushed write of the entry.
- **Fix:**
  - State the free-space precondition in N26 (d).
  - Say that the dropped writes include all of the entry's unflushed writes.
  - Add the barrier-1 reset as an N37 case under O17 (a), or cite it in "Stale valid content".

**OPT6-5 — 2.7 failure paragraph under O18: the kept state is not named.**
- **Where:** 2.7 failure paragraph, "discards its in-memory writing-entry state … keeping the committed copy only to judge the remount".
- **Problem:** under O18 the service must also keep the encoded copy across the remount. The O18 paragraph and 8.1 say so, but this sentence does not.
- **Fix:** add "(and, under O18, the copy encoded in step 4)".

**OPT6-6 — 2.2 check 7 table: the blank-candidate row omits the defensive rule.**
- **Where:** 2.2 check 7 table.
- **Problem:** the blank-candidate row still says "FORMAT and UNBIND run". The defensive rule overrides it in a separate paragraph, and 2.7 cross-references that paragraph.
- **Fix:** add a short "(FORMAT `Access` -5 if the safe-path latch is set; unreachable, see below)" to the cell, so that the table remains the single state table, as review 2 F4 intended.

**OPT6-7 — Section 9 historical rows edited or now superseded without a note.**
- **Where:** section 9, review-4 and review-5 records.
- **Problem:**
  - `1735ab06` edited the review-4 OP-5 row to "folded into the O16 rule …", but it still ends "O16 stays undecided", which now reads as contradictory.
  - The review-5 RQ5-1 row still says that N36 states "the alternative's result". Revision 4 (P4) removed that.
  - The review-5 OPT-2 row ("Not taken …") is now answered by O18.
- **Fix:** restore OP-5 to its historical wording or add "(decided 23 November 2026)". Mark RQ5-1 and OPT-2 "superseded in revision 4, P4 and P2", as earlier rows did ("superseded in revision 3, L1").

**OPT6-8 — 2.7 step 4 / N29 / N35: the count-clause equivalence depends on the 16-entry guards.**
- **Where:** 2.7 step 4; N29; N35.
- **Problem:** in the Rust core the record-count clause also short-circuits the index `next.records[committed.count]`. Without it, a committed copy with 16 records would index out of bounds. The equivalence therefore holds only because CREATE refuses with `Full` at 16 sealed entries and SEAL refuses at `count >= MAX_ENTRIES` before step 4. Neither caller can reach the check with 16 records.
- **Fix:** add one clause saying that the equivalence assumes those guards.

**OPT6-9 — O17 (a): a cheaper equivalent of the digest.**
- **Where:** 2.7 step 3; 8.1 O17.
- **Problem:** `archive::verify` already requires the stored trailer to be the FNV-64 of every preceding byte (`services/volume/src/archive.rs:9`, `:368`). For a readback that passes verify, comparing its trailer with the last 8 bytes the portal appended has the same accidental-fault strength as a running FNV-64 over all appended bytes. This is not a defect.
- **Fix:** optionally allow either form, and say that "the bytes appended so far" means only APPENDs answered `Ok`.

## Commands run (all read-only)

- `git log`, `git status --short`, `git diff --stat` and `git diff dd2bb19d -- docs/implementation/S09-DESTINATION-SERVICE.md` in the spec worktree.
- `git diff --word-diff=porcelain -U0 dd2bb19d -- …` to list every removal.
- `git show 1735ab06` and `git show dd2bb19d:<spec>`.
- `git show 035b44e5:services/catalog/src/wire.rs`.
- `grep` and `awk` ID-set comparison (N, O, R, L, P, K) between `dd2bb19d` and the worktree.
- Reads of the full spec sections 2.1–2.8, 5.3, 5.4, 6, 7, 8.1, 8.2 and 9.
- Reads and greps in the core worktree at `e52cb865`:
  - `services/destination/src/{catalog.rs,service.rs,device.rs,lib.rs}`;
  - `tests/{broker.rs,catalog.rs,crash.rs,ops.rs,oracle_images.rs,common/mod.rs}`;
  - `tools/destination_disk.py` and `tests/test_volume_destination_disk.py`;
  - `services/volume/src/archive.rs`;
  - `.github/workflows/*.yml`;
  - the three code reviews.

## Not checked

- **No cargo, Python or cross-check run.** Cargo writes build output, and the task allowed read-only commands only. The "30 images agree" count comes from `CODE-REVIEW-O14-O16-2.md`, not from my own run.
- **The O14/O16 round ledger** that names B4, B6, A10 and A11 is not stored on PR #413. I verified those mutants by reading the code, not by running them.
- **The owner's acceptance of O14, O15 and O16** is taken from the spec's own record in section 9; I did not see the owner's message.
- **Hosted CI, native or QEMU stages, the real `volume_core::archive::verify`, and the reverse differential** were not run. The reverse differential does not exist.
- **Hosted Graphify `recall`** was not used; I read the code directly.
- **The proposed rules** O17 and O18 have no implementation to compare.
- **This is an AI review, not a human audit.**

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_61baa554-463` (review of revision 4 plus the owner-decision commit `1735ab06`, diff base `dd2bb19d`). It returned APPROVE, so the workflow ran no fix pass and no re-review; the lead applied the nine optional findings by hand (spec section 9, "Review 6"). An AI review, not a human audit.
