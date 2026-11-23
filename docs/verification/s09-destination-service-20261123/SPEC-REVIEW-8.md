# SPEC-REVIEW-8: S09 destination service specification, revision 5

- **Date:** 23 November 2026
- **Document:** `docs/implementation/S09-DESTINATION-SERVICE.md`, uncommitted revision-5 working tree. Worktree `s09-destination-service`, branch `claude/s09-destination-service`.
- **Base commits:**
  - The revision-5 diff is against HEAD `a769bc65`.
  - The owner-decision commit was reviewed as `git diff 0643aa09 a769bc65`.
  - Implementation claims were checked against `f993a1a3` (draft PR #413, worktree `s09-destination-core`; that commit is on `origin/claude/s09-destination-core`).
  - *[main]* claims were checked against `origin/main` (`47a30a02`) and `035b44e5`. Both give identical results for `kernel/src/m2/m3/volume.rs` and `crates/virtio-core/src/lib.rs`.
- **Inputs:**
  - the revision-5 ledger (Q1–Q13);
  - `docs/verification/s09-destination-core-20261123/CODE-REVIEW-O17-O18.md` at `f993a1a3`;
  - the section 9 "Review 7" dispositions.
- **Reviewer:** an independent AI reviewer that wrote none of this text. It worked read-only: it edited no file and ran no build or test. It used `git`, `sed`, `grep`, `awk`, `sha256sum` and a scratchpad `diff`.
- **Status of this review:** an independent AI review, not a human audit.
- **Limit:** SPEC-REVIEW-7 is not available (RQ8-2). The review-7 required findings were therefore confirmed against their recorded dispositions and against the code, not against the original finding text.

## Verdict

**REQUEST_CHANGES. There are 4 required findings and 6 optional ones.**

This review covers the owner-decision commit `a769bc65` (Q2) and the three review-6 hand corrections in their final form (Q13), which the specification says are still owed.

- **Decided rules.** No decided O14–O18 rule changed meaning.
- **Security floor.** It holds.
- **Evidence labels.** They are honest, except for one dangling evidence pointer (RQ8-2).
- **Required findings.** They concern:
  - status wording that the review-7 fix pass left stale (RQ8-1);
  - a missing review record (RQ8-2);
  - a list of contract exceptions that RQ7-3 meant to make complete and that still misses one test (RQ8-3);
  - an N37 (b) precondition that does not match its named test (RQ8-4).

## Earlier required findings (review 7)

| Finding | Status | Evidence |
|---|---|---|
| RQ7-1 | Resolved | At `f993a1a3`, `SimDisk::poll` (`tests/common/mod.rs`) returns -1 when no request is in flight. It records no event and no counter. 2.7 ("Meaning of kernel -8") now states the stray-poll test as an open R2 item, outside the *[PR #413]* label. This matches 8.2 R2 and the Q6 CODE-DELTA row. `device::run` is the only submit/poll loop; `Watch` in `device.rs` only delegates. |
| RQ7-2 | Resolved | `p12_3_answered_exactly_once` asserts -1 after delivered write and read results and after requests ended by a reset. It checks an unchanged caller buffer only after the further read poll and takes no `snapshot`. The CODE-DELTA row is marked "partly done", and the snapshot is open. 8.2 R2 agrees. |
| RQ7-3 | Partly resolved | N13, N26 (a)–(c), N26 (e) and (f), and N33 are listed and match the injections in `crash.rs` and `broker.rs`. N26 (f) is added to the row with outcomes that match `n26f_remount_after_expired_poll_bound`. The list still misses `p12_2_refused_submission_changes_nothing`, which keeps a request pending for 590 ticks against `TIMEOUT` = 500 (RQ8-3). |

## Ledger coverage (Q1–Q13)

- Each of Q1–Q13 has exactly one row in the "Revision 5 record". Each claimed body change is present:
  - Q1: the CODE-DELTA (revision 5) table and the pointer after the revision-4 table.
  - Q2: the header and 8.1 consequences.
  - Q3: the conditional wording; see "Decided rules" below.
  - Q4: the 2.7 host-model paragraph, the 8.2 R2 item and the CODE-DELTA row.
  - Q5: the N35 equivalent mutant M7, with its test-hooks qualifier.
  - Q6: the 2.7 poll invariant.
  - Q7: property 3 and the 2.5 -1 paragraph.
  - Q8: 2.7 step 3.
  - Q9: the N37 rewrite, including the new case (d).
  - Q10: the 8.2 status clauses and the evidence-label paragraph.
  - Q11: N35 M8 "observed".
  - Q12 and Q13: route items.
- **CODE-DELTA (revision 5).** All 14 revision-4 rows have a status. Each status was checked at `f993a1a3`:
  - the test names exist (`n26d_legitimate_waits_complete`, `n33_read_timeout_with_unflushed_append`, `n37a_read_timeout_over_stale_valid_archive`, `n37a_own_reset_at_barrier1_over_stale_valid_archive`, `n37b_unstored_entry_writes_over_stale_valid_archive`, `n37_digest_covers_only_appends_answered_ok`, `n35_remount_contradicting_committed_catalog`, `n34_record_and_copy_validity`, `p12_1`–`p12_5`);
  - `n26d` asserts `catalog_writes == 0` in the cache-dropped branch;
  - no U+009F or U+00A0 case and no N33 (ii) zero-byte variant exist, so "open" is correct;
  - the N35 (e) body has the four cases: length, trailer, label and next entry ID.
- **Test count.** There are 76 `#[test]` attributes (15 + 14 + 13 + 11 + 5 + 12 + 1 + 5).
- **Hashes.** `service.rs` at `f993a1a3` hashes to `33a7a2f1…`, not the reviewed `d1a437c1…`, so "no review or gate run covers those edits" is accurate. `common/mod.rs` equals the reviewed hash.
- **Mutant counts.** "14 of 16 mutants killed" matches the mutant table of CODE-REVIEW-O17-O18.
- **Q4.** It is not resolved by relaxation:
  - properties 1–5 stay normative for K5;
  - R2 gains an opt-in deadline mode and persistent quarantine and offline state, with tests;
  - the P12 claim is stated as partial.

  RQ8-3 is a completeness defect in the exception list, not a relaxation.

## Owner-decision commit `a769bc65` (Q2)

The relabel states the decisions as decided:
- **O17 decided note:** "the recommendation is accepted ((c): the service rule (a) and the portal check (b); a mismatch does not start the safe path, pending the O9 evidence)". This matches the O17 recommendation at `0643aa09` and the fixed rule.
- **O18 decided note:** "the proposal is accepted (byte equality with the encoded copy)". The 2.7 O18 paragraph keeps "the copy that the service encoded and checked in step 4", and 2.6 states that full-field equality is equivalent for valid copies.
- **Section 9 paragraph:** it records the consequences correctly. Revision 5 adds a note and does not rewrite it.
- **Remaining defect:** the one residual defect of `a769bc65` (2.6: "Revision 4 proposes ..." beside a decided label) is fixed in revision 5.

## Review-6 corrections in final form (Q13)

- **N37 row and its safe-path statement.** The row is correct. It says "no safe path, so the later CREATE runs (decided: no latch, pending O9, 2.7)", and "a latching implementation is a defect". This matches 2.7 "Stale valid content", 8.1 O17 and `refused_without_latch` in the tests, which asserts `!safe_path()` and then CREATE `Ok`.
- **2.7 step 3 trailer form and its precondition.** The content is correct. The real `archive::verify` on main requires the 16-byte trailer to be END plus the FNV-64 of every preceding byte (`services/volume/src/archive.rs`). The stand-in in `tests/common/mod.rs` hashes every byte before the last 8 and checks a length floor and two magic values, as the Q8 row says. The first sentence still says "An equivalent form" without the precondition (OPT8-1).
- **8.1 O17 latching sentence.** It is correct and consistent with 2.7. It reads "no latch, pending O9", and only O9 evidence of late-landing writes reopens the question, through a new owner decision.

## Decided rules, security floor, consistency

- **No meaning change to O14–O18.** "Declined" names only alternatives that the decisions replaced. Outside section 9, a search finds no remaining "if O17/O18", "lapses", "under O17" or "waits on O17/O18" wording, apart from the historical "Status (revision 4)" clauses and the revision-4 header paragraph.
- **Section 9 history is not rewritten.** The `git diff -U0` hunks in section 9 are insertions only.
- **Security floor:**
  - no capability is widened;
  - there are no diff hunks in 2.2 or section 4;
  - the check order is unchanged;
  - disk bytes stay untrusted;
  - the -1 poll result fails closed (`poll_error` maps it to `Io` for a read and to `Unknown` for a write);
  - the FNV-64 threat model stays limited to accidental faults (2.7, 5.4).
- **Consistency of 2.5, 2.7, section 6 and 8.2.** These agree on `Corrupt` -18, no latch, the -1 handling, the running form and the R2/R7 split. The exceptions are RQ8-1 (status) and RQ8-4 (N37 (b) setup).
- ***[main]* claims.** `volume.rs:886-891` (`ok_or(-1)` in the completion poll, `n == 0x41`) and `Policy::consume` (which sets `pending = None`) are verified. The reset case is correctly marked as not checked.
- **No overclaim.** No native, hosted or human-audit evidence is claimed.

## Required findings

| ID | Location | Problem | Fix |
|---|---|---|---|
| RQ8-1 | 8.2 R1 "Status (revision 5)": "revision 5 ... awaits its review (review 7), which must also cover them"; 9 Revision 5 record: "are not yet reviewed"; Q12 row: "Review 7 must cover `a769bc65`"; Q13 row: "review 7 must cover the N37 row" | These contradict the header and section 9 "Review 7": review 7 returned REQUEST_CHANGES, and its fixes await re-review. The review-7 fix pass did not touch R1. | State review 7 REQUEST_CHANGES, the fixes applied, and the re-review (review 8) pending, including the `a769bc65` and review-6 coverage. Mark the record as reviewed by review 7. In Q12 and Q13, write "the review of revision 5 (reviews 7 and 8)". |
| RQ8-2 | 9 "Review 7": "stored verbatim as `docs/verification/s09-destination-service-20261123/SPEC-REVIEW-7.md`" | The file does not exist, tracked or untracked. The pointer leads to nothing. | Commit the verbatim file with revision 5, as SPEC-REVIEW-6 was committed with `0643aa09`. |
| RQ8-3 | 2.7 host-model paragraph: "only where the case of a section 6 row requires it ... These cases are: ..."; RQ7-3 disposition: "checked against the fault injections of the host tests" | `p12_2_refused_submission_changes_nothing` keeps a write pending 590 ticks with `TIMEOUT` = 500. That is beyond the property-4 deadline, in a P12 test that is not a section 6 row. Property 2's "given up on" clause is reachable only when property 4 is not met (`POLL_BOUND` = `TIMEOUT` + 2 × `RESET_TIMEOUT`). | List the P12 property-2 test as a deliberate exception. State why the clause presupposes a missed property 4. Correct the disposition note. Do not relax properties 2 or 4. |
| RQ8-4 | 6 N37 Precondition: "exactly E2's sector-aligned length ... (for the barrier-1 variant of (a) ... need not be sector-aligned)" | The named (b) test uses a 1080-byte stale archive, which is not sector-aligned. | Extend the exception to (b), where no write of the entry, the SEAL tail included, is stored. Alternatively, annotate the (b) test name. |

## Optional findings

| ID | Location | Problem | Fix |
|---|---|---|---|
| OPT8-1 | 2.7 step 3: "An equivalent form compares ..." | Equivalence is stated before its precondition. | Begin with "With the real `volume_core::archive::verify` linked (below), an equivalent form ...". |
| OPT8-2 | 2.7 exception list; 2.5 table | It is not stated whether a read completion -16, or a read -13 before the deadline, is inside the contract. N8 and N33 inject both, and only the section 9 history reasons about them. | State it in 2.7 or 2.5. |
| OPT8-3 | 8.1 O17: "(Question as asked in revision 4: ...)" | The closing conditional sentences were removed in revision 5. | Write "as asked in revision 4, without the conditional sentences removed in revision 5 (Q3)". |
| OPT8-4 | 8.2 R1: "(O14–O18 decided; O4 (a)–(c) and O11–O13 open)" | Other 8.1 items (O1, O9, O10 and others) are also open. | Add "the other items of 8.1 remain open as stated there". |
| OPT8-5 | 2.5, -1 paragraph | The parenthesis on the failed-mount state omits a mount or remount catalog read. | Add "or catalog read", or refer to the 2.6 mount rule. |
| OPT8-6 | 9 CODE-DELTA (revision 5), row "N35 equivalent mutants M7 and M8" | The test comment describes the mutants but does not name them M7 and M8. | Write "described (not by name)". |

## Not checked

- The original text of SPEC-REVIEW-7. It is not available (RQ8-2), so RQ7-1–RQ7-3 were checked against their dispositions and the code only.
- Execution of any build, test, Clippy, fmt, oracle or mutation run. The gate and mutant results are those reported by CODE-REVIEW-O17-O18 for the diff over `8f387ea1`. The later comment-only edits in `f993a1a3` were read, not run.
- What the main M3 broker returns for a further poll after a request ended by a reset. The spec also marks this as not checked.
- Native K5 broker behaviour, the O9 question, the R5 preconditions, any QEMU stage, hosted CI and the R3 hosted cross-check and reverse differential.
- The O17 (b) portal check and N37 (c). These are R7 work; there is no portal code.
- Sections of the specification outside the revision-5 diff, apart from targeted searches for O17/O18 wording and the 2.5/2.7/6/8.2 consistency points named above.
- Whether FNV-64 fits the threat model beyond the accidental-fault scope stated in 2.7 and 5.4. A hostile device is out of scope.
- Human audit. This review is an AI review and does not replace one.

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_b8d1fd72-682` (review 2 of revision 5, base `a769bc65`, implementation compared at `f993a1a3`). The fixer applied the findings in the text; RQ8-2 (store reviews 7 and 8) is done by adding these files in the revision-5 commit; review 9 re-reviewed the result. An AI review, not a human audit.
