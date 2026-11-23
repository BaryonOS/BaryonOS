# SPEC-REVIEW-9: S09 destination service specification, revision 5

- **Date:** 23 November 2026
- **Object:** `docs/implementation/S09-DESTINATION-SERVICE.md`, uncommitted working-tree text (revision 5) in worktree `s09-destination-service`, branch `claude/s09-destination-service`. Also reviewed: the owner-decision commit `a769bc65`, as `git diff 0643aa09 a769bc65`.
- **Base commits:**
  - spec HEAD `a769bc65`; revision 4 at `0643aa09`
  - implementation evidence: draft PR #413 at `f993a1a3`, read only
  - *[main]* claims checked against `origin/main` (`47a30a02`) and `035b44e5`
- **Inputs read in full:**
  - the revision-5 ledger (Q1–Q13)
  - `CODE-REVIEW-O17-O18.md` (PR #413)
  - both diffs
  - the test sources of `services/destination/tests` at `f993a1a3`
- **Reviewer:** an independent AI reviewer that did not write the text. This is an AI review, not a human audit. It is read-only: no file was edited, no build or test was run, and no commit was made.

## Verdict

**APPROVE. There are no required findings and nine optional ones (OPT9-1 to OPT9-9).**

**Condition:** RQ8-2 (OPT9-1). The revision-5 commit must add SPEC-REVIEW-7.md and SPEC-REVIEW-8.md verbatim, together with this review. Until then, the "Review 7" and "Review 8" paragraphs of section 9 point to files that do not exist.

## Earlier required findings (reviews 7 and 8)

| Finding | Status in this text | Evidence checked |
|---|---|---|
| RQ7-1 | Resolved | 2.7 "Meaning of kernel -8" now states the stray-poll test as an open R2 item, outside the *[PR #413]* label. At `f993a1a3`, `SimDisk::poll` returns -1 with no counter (`tests/common/mod.rs`). `device::run` polls only after a submission returned 0 and stops at the first result other than -8 or at `POLL_BOUND`. |
| RQ7-2 | Resolved | The CODE-DELTA (revision 5) row for property 3 is "partly done". `p12_3_answered_exactly_once` asserts -1 at four further polls and an unchanged buffer only after the further read poll. A `snapshot` helper exists in `sim_contract.rs`, and the per-poll snapshot is listed as open. |
| RQ7-3 | Resolved | The 2.7 exception list was re-swept against every fault injection in `tests/*.rs` (see "Contract exceptions" below). N26 (f) row matches `n26f_remount_after_expired_poll_bound`: SEAL `Unknown`, `Refused(Flush)`, failed-mount state, safe path set, INFO `Io`, LIST `Unknown` with no records, no further mutation, both reset models. |
| RQ8-1 | Resolved | 8.2 R1, the header and the Q12 and Q13 rows name reviews 7 and 8. Only the "await re-review" wording is now stale (OPT9-9). |
| RQ8-2 | **Open: commit action** | The directory holds SPEC-REVIEW-1 to SPEC-REVIEW-6 only (OPT9-1). |
| RQ8-3 | Resolved | `p12_2_refused_submission_changes_nothing` holds a write pending for `POLL_BOUND + SUBMIT_BOUND + 50`. In `device.rs`, `TIMEOUT` is 500, `RESET_TIMEOUT` is 10, `SUBMIT_BOUND` is 20 and `POLL_BOUND` is 520, which gives 590. 2.7 lists that test as the single P12 exception. |
| RQ8-4 | Resolved | The N37 precondition allows a stale archive that is not sector-aligned for (b). `n37b_unstored_entry_writes_over_stale_valid_archive` uses `archive(2, 1, 1000, 5)`, which is 1080 bytes, as `n37a_own_reset_at_barrier1_over_stale_valid_archive` asserts. |

## Ledger coverage (Q1–Q13)

Each of Q1–Q13 has exactly one row in the "Revision 5 record", and each claimed body change is present.

- **Q1:** a pointer after the CODE-DELTA (revision 4) table, which is not edited, and a new CODE-DELTA (revision 5) table.
- **Q2:** see "Owner-decision commit `a769bc65`" below.
- **Q3:** see "Decided rules and history" below.
- **Q4:** 2.7 contract paragraph, 8.2 R2 and CODE-DELTA.
- **Q5 and Q11:** N35 equivalent mutants. The comment of `n35_remount_contradicting_committed_catalog` at `f993a1a3` describes both M7 and M8.
- **Q6:** poll invariant in 2.7, 8.2 R2 and CODE-DELTA.
- **Q7:** property 3 and the 2.5 −1 paragraph.
- **Q8:** 2.7 step 3.
- **Q9:** N37 row.
- **Q10:** header, evidence labels, and R1, R2, R3, R7 and R8.
- **Q12 and Q13:** route items, covered by this review.

### Q4 is not resolved by relaxing anything

- Properties 1–5 are unchanged.
- The R2 acceptance keeps "a host test ... for each property". It adds an opt-in deadline mode and persistent quarantine and offline state as open items, with tests under both reset models.
- The only test that breaks property 4 on purpose for a P12 check is the property-2 "given up on" case. That clause can arise only when property 4 is not met, because `POLL_BOUND` = 520 > `TIMEOUT` from admission.
- No owner question was added, which is correct: the gap is a host-model gap, not native broker evidence.

### CODE-DELTA (revision 5) statuses at `f993a1a3`

Every row was checked. All 76 `#[test]` functions were enumerated, and every test name cited exists.

- **P3:** 13 label-content cases in `n34_record_and_copy_validity` and 13 in the oracle's `LABELS`. There is no U+009F or U+00A0 case.
- **P6:** the guard `wire::FORMAT if self.safe_path => Err(Status::Access)` is present.
- **P8:** `n26d_legitimate_waits_complete` asserts `catalog_writes == 0` when the cache is dropped.
- **P9 (i):** `n33_read_timeout_with_unflushed_append`, with non-zero stale bytes. There is no zero-byte variant.
- **P1 (a):** the running form; `Writing::digest` carries the reason comment.
- **P2 / O18:** `catalog_failure(&checked)` with full `Catalog` equality, as reported in the code review.
- **P12:** `p12_1` to `p12_5`, with the module comment "Not modelled by `SimDisk`".
- **Q6:** no stray-poll counter exists, so the row is correctly open.

### Gate and mutation claims

The spec says the gate and mutation results cover the reviewed diff over `8f387ea1`, and that only comment-only edits followed. This was checked against the scratch snapshot of the reviewed tree (`mut/`, whose `service.rs` sha256 starts with `d1a437c1`):
- `service.rs`, `crash.rs` and `sim_contract.rs` differ from `f993a1a3` only in comments.
- `broker.rs` and `catalog.rs` are identical.

This check is indicative only: the snapshot is the code reviewer's scratch copy, not a git object.

## Owner-decision commit `a769bc65` (Q2)

The relabelling states both decisions exactly as the fixed rules quote them:
- **O17:** "the recommendation is accepted ((c): the service rule (a) and the portal check (b); a mismatch does not start the safe path, pending the O9 evidence)".
- **O18:** "the proposal is accepted (byte equality with the encoded copy)". It refines the *sealed* arm of O16.

The 0643aa09 text of O17 includes "a mismatch replies `Corrupt` and does not start the safe path" and recommends not latching. Revision 5's "no latch, pending O9" is therefore the accepted recommendation, not a new decision.

One fault in `a769bc65` is fixed in revision 5: the 2.6 sentence "Revision 4 proposes ..." under a decided label now reads "The *sealed* resolution requires ...". The section 9 paragraph of `a769bc65` is kept as written and receives a note.

## Decided rules and history (Q3, Q12, Q13)

- **No decided O14–O18 rule changed meaning.**
  - The O16 arms other than *sealed* are unchanged.
  - O18 is stated as equality with the copy encoded and checked in step 4. That means 2048 bytes, which for valid copies equals equality of every field.
  - O17 (a) still replies `Corrupt` -18, before any catalog write.
  - Restricting the trailer form to the real `volume_core::archive::verify` only tightens an allowed alternative. The running form is unchanged, and the real verifier's trailer rule was checked on main: `services/volume/src/archive.rs`, trailer = `END` plus the FNV-64 over every preceding byte. The R2 stand-in hashes `bytes[..n-8]`, as the Q8 row says.
- **No conditional wording on decided O17 or O18 remains outside section 9.** A search for "if O17/O18", "under O17/O18", "lapse", "declined", "waits on" and "proposed; owner" finds only O4, O11, O12 and O13 markers, which are open, and historical header text.
- **Section 9 history is not rewritten.** Every diff hunk after the start of section 9 is an insertion.
- **Q13 final forms:**
  - The N37 row says "no safe path ... (decided: no latch, pending O9, 2.7)", and a latching implementation is now a defect, replacing the OPT6-1 note.
  - The 2.7 step 3 trailer form is valid only with the real verifier linked.
  - The 8.1 O17 latching sentence says "was recommended", followed by the decided "no latch, pending O9" and its reopening condition (O9 evidence that late writes can land, through a new owner decision).
  - These agree with each other, with 2.5 and with N26 (d) and N33.

## Security floor and evidence

- No capability is widened, and the 2.2 check order is untouched.
- Disk bytes stay untrusted, and medium behaviour is stated as outside the broker contract.
- The FNV-64 threat model is stated as accidental faults only, in 2.7, 5.4 and the Revision 5 record.
- The new rules are fail-closed or stricter:
  - a stray −1 poll result gives `Io`, `Unknown` with the safe path, or the failed-mount state, consistent with `poll_error` at `f993a1a3`;
  - property 3 adds "changes nothing";
  - R8 now requires the O17 (b) portal check.
- No native, hosted or human-audit evidence is claimed. The P10 lint and the 30-image cross-check are labelled local only.
- The *[main]* claims were checked:
  - `volume.rs:886-891` (`ok_or(-1)` with no pending request);
  - `Policy::consume` clears `pending`;
  - `Completion::Unknown` is set only for mutating requests, so −16 comes only for a mutating request;
  - the timeout code in `volume.rs` is −16 if mutating and −13 otherwise.

  `:886-891` and `:777` give the same content on `035b44e5` and `origin/main`.

## Contract exceptions (re-sweep)

These fault injections in `tests/*.rs` fall outside the broker contract, and each is listed in 2.7:
- `again: Some(None)`: N26 (a)–(c);
- `pending: 100 * POLL_BOUND`: N26 (e), N26 (f);
- completion −13 on writes or flushes: N13 and N33;
- completion −16 on reads: N8, N33;
- completion −14: N33.

Every other injection is inside the contract:
- −12 completions;
- −16 on writes;
- −13 on reads;
- the reset window `again: Some(Some(RESET_TIMEOUT))`;
- `pending: TIMEOUT - 1`.

N24's late landing and N37 (b)'s `silent` mode are medium behaviour, as stated.

## Internal consistency (2.5, 2.7, section 6, 8.2)

- **`Corrupt` -18 and no latch** on a digest mismatch are stated alike in 2.5, 2.7 step 3, "Stale valid content", N37, 8.1 O17 and R2.
- **N37 (a), (b) and (d)** match the tests:
  - the setups of (a) and (b) end with `refused_without_latch`, which checks INFO and LIST, rebinds and runs CREATE;
  - with the cache kept, `sealed_as_sent` gives `aux` equal to the sent trailer and two entries;
  - (b) asserts the stale extent is whole and `dropped == 0` under both models;
  - (d) uses a fresh format with `Stale` and `Full` APPENDs between APPENDs answered `Ok`.
- **N37 kills** match the code-review mutants: M1, M4, M5, M9, M2b, M3b, M12 and M13.
- **The N35 (e) kill (M6) and the equivalent mutants (M7, M8)** match the code review.
- **One wording mismatch:** the stray-poll obligation is phrased three ways (OPT9-4).

## Findings

### Required

None.

### Optional

| ID | Location | Problem | Fix |
|---|---|---|---|
| OPT9-1 | 9, "Review 7" and "Review 8" ("stored verbatim as ..."); RQ8-2 | The earlier required finding RQ8-2 is still open in the working tree. SPEC-REVIEW-7.md and SPEC-REVIEW-8.md do not exist yet. | Add both, and this review, verbatim in the revision-5 commit. The approval is conditional on this. |
| OPT9-2 | 8.1 O17 note "(Question text as asked in revision 4, without the conditional sentences that revision 5 removed ...)" | The note is not accurate. The question text was also changed in two other places: "is recommended" became "was recommended", and "N37 (a), (b)" became "N37 (a), (b), (d)" in the marking sentence. The meaning is unchanged. The continuation lines are not indented. | Name both edits in the note, or restore "is recommended". Indent the continuation lines. |
| OPT9-3 | 2.7 property 3, "what it returns for a further poll after a request ended by a reset was not checked" | A reading of main, not executed, suggests that after a read ended by a reset, `peek` fails with Offline in `Phase::Resetting`. `pending` then stays set, and a further poll returns −14 until `reset_completed`. The disclaimer is honest and the service does not rely on −1. | Record this as a K5 delta and R5 precondition. |
| OPT9-4 | 2.7 "A host test that counts stray polls ..." against 8.2 R2 "asserted zero in every service-driven host test" | The same obligation is phrased three ways with different strength. | Align 2.7 with 8.2 R2. |
| OPT9-5 | 2.7 "Stale valid content", "(revision 5: stated as the case the decided checks below prevent)" | Only O17 (a) prevents the commit; O17 (b) only reports the export as failed. | Reword so that the service check prevents the commit and the portal check reports the failure. |
| OPT9-6 | 2.7 contract paragraph, "N26 (e) and (f) (host test `n26f_remount_after_expired_poll_bound`)" | The N26 (e) test is not named. | Add `n26e_poll_beyond_bound_is_unknown`. |
| OPT9-7 | 2.7 opt-in deadline; CODE-DELTA (revision 5) Q4 row | The default-off deadline is equivalent to an always-on one only by a manual sweep. | Optionally add a default-mode tripwire for a request pending past `TIMEOUT`, with opt-outs for N26 (e), N26 (f) and p12_2. |
| OPT9-8 | 9, note after the `a769bc65` paragraph | The historical paragraph allows the trailer form without the new precondition. | Point the note to 2.7 step 3 (Q8) as well. |
| OPT9-9 | Header "The fixes await re-review."; 8.2 R1 Status (revision 5) | Both statements are stale after this review. | Record review 9 (APPROVE, no required finding) and the dispositions of OPT9-1 to OPT9-9. |

## Not checked

- No build, test, Clippy, fmt or mutation run. The gate and mutation results are taken from CODE-REVIEW-O17-O18 as reported.
- The originals of SPEC-REVIEW-7 and SPEC-REVIEW-8 are not available. Their findings were judged from the section 9 disposition tables and checked against the text and the code.
- The Python oracle's internals beyond the N34 label list. The 30-image cross-check was not re-run.
- The M3 broker's answer to a further poll after a request ended by a reset was read from the code only, not executed (OPT9-3).
- Native K5 behaviour, R5 to R8, the portal (O17 (b), N37 (c)), hosted CI, the R3 reverse differential, and any hostile-device or FNV-64 collision case. These are out of the threat model.
- Whether the scratch snapshot `mut/` is exactly the reviewed tree, beyond the matching `service.rs` and `common/mod.rs` hashes.

An AI review is not a human audit.

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_b8d1fd72-682` (review 3 of revision 5, base `a769bc65`, implementation compared at `f993a1a3`). It returned APPROVE, so the workflow ran no fix pass. The lead then applied its optional findings in a separate pass, checked by a read-only review (section 9, "Review 9" and later). An AI review, not a human audit.
