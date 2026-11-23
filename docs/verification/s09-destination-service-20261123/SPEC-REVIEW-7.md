# SPEC-REVIEW-7: S09 destination service specification, revision 5

- **Date:** 23 November 2026
- **Document:** `docs/implementation/S09-DESTINATION-SERVICE.md` in the worktree of branch `claude/s09-destination-service`.
- **What was reviewed:**
  - the uncommitted revision-5 working-tree text against HEAD `a769bc65` (`git diff`);
  - the owner-decision commit `a769bc65` against `0643aa09` (`git diff 0643aa09 a769bc65`), which no earlier review covered.
- **Implementation compared:** draft PR #413 at `f993a1a3` (`services/destination`, `tools/destination_disk.py`, `tests/test_volume_destination_disk.py`). The worktree was clean at `f993a1a3`.
- ***[main]* claims compared:** `035b44e5` and `origin/main` (`47a30a02`). `kernel/src/m2/m3/volume.rs` and `crates/virtio-core/src/lib.rs` are identical in both.
- **Inputs:** the revision-5 ledger (Q1–Q13) and `docs/verification/s09-destination-core-20261123/CODE-REVIEW-O17-O18.md` on PR #413, both read in full.
- **Reviewer:** an independent AI security reviewer that did not write the text. This is an AI review, not a human audit. Only git, cat, sed, grep and awk were used. No build, test or mutation was run.
- **Process note:** contrary to the read-only instruction, one working file was written: the revision-5 diff, saved to the session scratchpad (`rev5.diff`) so that it could be read. No repository file was changed, and no commit was made.

## Verdict

**REQUEST_CHANGES: 3 required findings (RQ7-1 to RQ7-3) and 6 optional findings (OPT7-1 to OPT7-6).**

All three required findings are wording fixes:
- RQ7-1 and RQ7-2 are evidence overclaims about tests that do not exist, or do not assert what the text says.
- RQ7-3 is a new normative sentence that is inconsistent with existing section-6 rows.

No decided rule changed meaning, and the security floor holds.

## Confirmed

### Q2 and Q12: the `a769bc65` relabel

**O17.** The relabel states the decision as accepted in 8.1:
> "(c): the service rule (a) and the portal check (b); a mismatch does not start the safe path, pending the O9 evidence"

This matches the O17 recommendation at `0643aa09` word for word in substance. That recommendation was:
> "**Recommendation: (c)**, with (a) as the normative service rule and (b) as a cheap second check ...; a mismatch replies `Corrupt` and does not start the safe path"

It also includes the sub-question recommendation, "Not latching ... is recommended until O9 ...".

**O18.** The relabel says "the proposal is accepted (byte equality with the encoded copy)". 2.7 states the full form: "2048 bytes equal the copy that the service encoded and checked in step 4". The `f993a1a3` code uses full `Catalog` equality with the decoded, checked copy, which the text and the code review treat as equivalent for valid copies.

**Leftover wording.** `a769bc65` itself said it left conditional wording in place and was unreviewed. In revision 5, `grep` finds no remaining "if declined", "lapses", "under O17/O18", "waits on O17/O18" or "[proposed; owner decision O17/O18]" outside section 9 history. The revision-4 header now reads "(conditional on O17 when revision 4 was written; ...)".

**Revision 5 keeps the decided meaning.** Each check against its source:
- **Header revision-5 paragraph:** "O17 as recommendation (c) ... does not start the safe path, pending the O9 evidence; O18 ... equality with the copy encoded and checked in SEAL step 4".
- **2.7:**
  - "**no latch, pending O9**";
  - "The question reopens only on O9 evidence ... that evidence would require a new owner decision".
  This is a faithful reading of "pending O9" and weakens nothing.
- **2.7 O18 paragraph:** "O18 refines the *sealed* arm of the decided O16 rule and leaves its other arms unchanged".
- **O14–O16:** their decided text is unchanged.
- **No new owner question:** there is no O19, and no O, N, R, K, L or P identifier was renumbered.

### Q1 and Q9: CODE-DELTA (revision 5) and the N37 row against the code at `f993a1a3`

**Coverage of the revision-4 table.** It has 14 rows. The revision-5 table gives one status row for each, and its additional rows cover Q4 and Q5–Q9, Q11. A pointer follows the revision-4 table, which is otherwise unedited.

**Test names verified** in `services/destination/tests`:
- `n26d_legitimate_waits_complete`, which asserts `catalog_writes == 0` in the cache-dropped branch;
- `n33_read_timeout_with_unflushed_append`;
- `n37a_read_timeout_over_stale_valid_archive` and `n37a_own_reset_at_barrier1_over_stale_valid_archive`, the latter with a 1080-byte stale archive that is not sector-aligned;
- `n37b_unstored_entry_writes_over_stale_valid_archive`, which runs under `both_reset_models` with `Fault::silent` and asserts the stale extent whole and `dropped == 0`;
- `n37_digest_covers_only_appends_answered_ok`;
- `n35_remount_contradicting_committed_catalog`, whose comment describes the count-clause, M8 and M7 equivalents without the M names;
- `p12_1`–`p12_5`;
- `n34_record_and_copy_validity`.

The recorded name `n33_read_timeout_over_stale_valid_archive_recorded` no longer exists.

**Other CODE-DELTA facts verified:**
- The 13 N34 label-content cases are present in both the Rust and the Python suites. No U+009F or U+00A0 case exists, so that row is correctly "open".
- 76 `#[test]` attributes.
- `service.rs` at `f993a1a3` hashes to `33a7a2f1…`, which differs from the reviewed `d1a437c1…`. The text discloses this.
- The comment on `Writing::digest` gives the reason for the running form.
- The oracle and its tests are unchanged between `8f387ea1` and `f993a1a3`.

**N37 row against the tests:**
- **(a), cache dropped, and (b):** the helper `refused_without_latch` asserts:
  - `Corrupt`;
  - `catalog_writes == 0`;
  - only E1 listed;
  - INFO `Ok`;
  - the binding `NotSealed`;
  - no safe path;
  - CREATE `Ok` on a new binding.
- **(a), cache kept:** `sealed_as_sent` asserts that `aux` equals the sent trailer and that two entries are listed.
- **(d):** a fresh FORMAT, refused `Stale` and `Full` APPENDs, then SEAL `Ok` with `aux` equal to the sent trailer.
- **Kills:** the M1, M2b, M3b, M4, M5, M9, M12 and M13 claims match the mutation table of the code review.

### Q4: the broker contract and the R2 acceptance are not relaxed

Properties 1–5 are unchanged, apart from the property-3 addition. 2.7 says:
> "The contract is not relaxed for host models"

R2 keeps "a simulated disk that meets the K5 broker contract". The opt-in deadline mode and the persistent quarantine and offline state are added as open R2 items with tests. The claim is stated as "partial for properties 4 and 5". This matches the `sim_contract.rs` module comment.

### Q7: the *[main]* claim of property 3

At `kernel/src/m2/m3/volume.rs:886-891`, `runtime().error.map(..).or_else(|| runtime().policy.request()).ok_or(-1)?` answers a poll with no request with -1. `Policy::consume` (`crates/virtio-core/src/lib.rs`, `pub fn consume`) sets `self.pending = None`. The behaviour after a reset-ended request is correctly stated as not checked.

### Q6: the poll invariant in the code

`device::run` is the only submit and poll loop in `src`; the `Watch` wrapper only forwards. It polls only after `submit` returned 0, and stops at the first result other than -8 or at `POLL_BOUND`. `poll_error(-1, ..)` gives `Io` for a read and `Unknown` for a mutating request, which matches 2.5.

### Q13: the review-6 corrections in their final form

- **N37 row:** "no safe path, so the later CREATE runs (decided: no latch, pending O9, 2.7)". The OPT6-1 note is explicitly replaced, and a latching implementation is now a defect.
- **2.7 step 3:** the trailer form with the precondition "for a portal archive that is itself a valid v1 archive" and the linked real verifier.
- **8.1 O17:** the latching sentence.

The three agree with one another and with 2.5 (`Corrupt` -18) and 8.2. The only caveat is the over-strict wording in OPT7-1.

### History

- Every revision-5 hunk inside section 9 (old line ≥ 1886) is insert-only: a pointer after the CODE-DELTA (revision 4) table, a note after the `a769bc65` paragraph, the Revision 5 record, CODE-DELTA (revision 5) and "Review 7. Pending."
- Outside section 9, the 8.2 R1–R3 status clauses of earlier revisions are kept.
- In the revision-4 "Revision 4 adds" clause of R2, "under O17 (a)" was restated as "O17 (a) decided". This falls under Q3 and is outside section 9.

### Q12: the security floor

- No hunk touches 2.2, the check order or a capability.
- Disk bytes stay untrusted. The O18 comparison is against service-held bytes.
- The FNV-64 threat model ("accidental faults only", "not a hostile device") is kept in 2.7, 5.4 and the Revision 5 record.
- The mismatch path writes nothing.

### Evidence labels

- *[PR #413]* for revision-5 claims is defined as `f993a1a3`.
- The gate and mutant results are attributed to CODE-REVIEW-O17-O18, run on the reviewed diff over `8f387ea1`. The later comment-only edits are disclosed as unreviewed.
- "Nothing was re-run for this revision" is stated.
- No native, hosted or human-audit evidence is claimed.

The two exceptions are RQ7-1 and RQ7-2.

## Findings: Required

| ID | Location | Problem | Fix |
|---|---|---|---|
| RQ7-1 | 2.7, "Meaning of kernel -8": "... or at the poll bound below *[PR #413]*; a host test counts stray polls (8.2 R2)." | This is a present-tense claim of a test that does not exist at `f993a1a3`. `SimDisk::poll` returns -1 without counting, and a grep for `stray` is empty. CODE-DELTA (revision 5), row Q6, says "open", and 8.2 R2 lists the counter under "Revision 5 adds". | Replace with "a host test that counts stray polls and asserts zero is an open R2 item (8.2 R2; CODE-DELTA (revision 5), Q6)", outside the *[PR #413]* label. |
| RQ7-2 | Section 9, CODE-DELTA (revision 5), row "2.7 property 3, further poll (Q7)": "asserts -1 and no change after a delivered write and read result and after requests ended by a reset" | `p12_3_answered_exactly_once` asserts -1 at four further polls, but "no change" only for the caller's buffer after the further read poll. No device-state snapshot is taken around any further poll, whereas property 3 now requires "no device state, no buffer content" change. | Mark the row "partly done" and state exactly what is asserted. Add a before-and-after device-state snapshot (the `snapshot` helper of `sim_contract.rs`) as open or optional, or add it to the test first. |
| RQ7-3 | 2.7, after the broker contract: "A test stub may break a property on purpose only where its row says so: N26 (e) and the host test `n26f_remount_after_expired_poll_bound` ..." | Read as a normative rule, which is how this review reads it given "only" in contract text, it conflicts with section 6. The N26 row has no case (f) and does not say that (e) breaks property 4. N26 (a)–(c) (-8 forever), N33 (completion -14 and a write's -13, both called outside the contract in 2.5) and N24 (late completion) also leave the contract on purpose, and their rows do not say so. If the sentence is only an example list, it is incomplete and the finding is optional. | List every deliberate breach with its row (N24; N26 (a)–(c); N26 (e) and n26f; N33), or reword to "where the row's case requires a result outside the contract" and add a clause to those rows. Map n26f to an existing N26 case, or add (f) to the row. |

## Findings: Optional

| ID | Location | Problem | Fix |
|---|---|---|---|
| OPT7-1 | 2.7 step 3: "The trailer form is equivalent only when the linked verifier is the real `volume_core::archive::verify` ..."; Q8 row | This is over-strict. The stand-in also binds the trailer to FNV-64 over every preceding byte, so for archives valid under it the trailer form would be equally strong. What fails is only that a v1 portal archive is not valid under the stand-in. The conclusion, which prefers the running form, is sound. The stand-in's different rule was verified: it hashes the end marker, whereas the real verifier at `035b44e5` excludes it. | Write "the trailer form is relied on only with the real verifier linked (R5)". |
| OPT7-2 | 2.5: "`Unknown` -16 for a write or flush, which starts the safe path" | A mount or remount FLUSH, or a FORMAT write or flush, leads to the failed-mount state instead. | Add that parenthetical. |
| OPT7-3 | 8.2 R2, Status (revision 5): "N33 (iii) and N37 (a), (b), (d) are negative tests" | N37 (d) expects SEAL `Ok` and does not run under both reset models. It is a regression test. 8.1 O17 ("N37 (a), (b) ... under both reset models") is the accurate statement. | In R2: "N33 (iii) and N37 (a), (b) are negative tests under both reset models; N37 (d) pins that only APPENDs answered `Ok` are hashed". Do not add (d) to 8.1. |
| OPT7-4 | 6 N37 Kills: "(the later CREATE returns `Access` -5; M4)" | The test first trips on `assert!(!h.svc.safe_path())`, before the CREATE. | Optionally name both observations. |
| OPT7-5 | 8.1 O17 question body: "Today a device fault ... SEAL then commits ..."; option (a): "the equivalent trailer comparison" | The verbatim question keeps current-state wording and an unqualified "equivalent". | Add "(as asked in revision 4)" or a pointer to the precondition in 2.7 step 3. |
| OPT7-6 | 8.2 R8: "the activation no longer waits on O17 or O18" | R8's acceptance covers QEMU-marked rows only, not N37 (c) (host with portal, R7). No R8 precondition names the decided O17 (b) check. This is not a regression from revision 4. | Name R7's O17 (b) portal check and N37 (c) as R8 preconditions. |

## Not checked

- **Builds, gates and mutants were not re-run.** Test, Clippy, fmt, the `x86_64-unknown-none` check, the oracle cross-check and the mutation table are taken from CODE-REVIEW-O17-O18. The comment-only edits in `f993a1a3` after that review were read, not executed.
- **No hosted CI** results and no R3 reverse differential.
- **The native K5 per-device broker** (R5), and `main`'s answer to a further poll after a request ended by a reset. The text states this as not checked.
- **Portal code, O17 (b) and N37 (c)** (R7): none exists.
- **The real `volume_core::archive::verify`** linked into the service. N7, N10, N11 and N12 stay partial.
- **A hostile device or an FNV-64 collision.** These are outside the stated threat model.
- **Unchanged *[main]* claims and line references** elsewhere in the document, which revision 5 did not touch.
- **Hosted Graphify MCP was not consulted.** The task restricted tools to git, cat, sed and grep, so code was read directly from the worktrees.
- **This review is not a human audit.**

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_b8d1fd72-682` (review 1 of revision 5, base `a769bc65`, implementation compared at `f993a1a3`). The fixer applied all 3 required and 6 optional findings; review 8 re-reviewed the result. An AI review, not a human audit.
