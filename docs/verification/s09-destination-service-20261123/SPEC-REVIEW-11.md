# SPEC-REVIEW-11: S09 destination service specification, revision 5 (lead pass after review 9)

- **Date:** 23 November 2026
- **Object:** `docs/implementation/S09-DESTINATION-SERVICE.md`, the uncommitted working-tree text (revision 5) in worktree `s09-destination-service`, branch `claude/s09-destination-service`, HEAD `a769bc65`. Its sha256 is `e16492355440675f833c951c26a2c85dd416202ef2753dc763d12c7d9f11d1d2`. Only the changes made after review 9 are reviewed, as `diff -u <review-9 snapshot> <working tree>` (311 diff lines).
- **Base:** the text that review 9 approved, a scratchpad snapshot (`spec-rev5-as-review9.md`) with sha256 `210d480ddb0f6ed08192cbdc123a5cd4b04d6962c8ca65556e1882168878cd39`.
- **Compared against:**
  - draft PR #413 at `f993a1a3` (worktree `s09-destination-core`, read only);
  - *[main]* claims at `035b44e5` and at the local `origin/main` ref `b9aa517c`. The blobs of `kernel/src/m2/m3/volume.rs` (`dca03383`) and `crates/virtio-core/src/lib.rs` (`23a5c2eb`) are identical in both.
- **Inputs read:**
  - SPEC-REVIEW-9 in full;
  - the headers, Q2/Q13 sections and findings tables of SPEC-REVIEW-7 and SPEC-REVIEW-8;
  - section 9 "Review 9" and "Review 10" of the spec.
- **Reviewer:** an independent AI reviewer that did not write the text. This is an AI review, not a human audit. No repository file was created, edited or deleted, no build or test was run, and no commit was made.
- **Process note:** for reading only, the reviewer wrote these working files to the session scratchpad: `r11.diff`, `volume.rs` and `vcore.rs` (from `git show b9aa517c:`), and `o17_old.txt`/`o17_new.txt` (the 8.1 O17 item at `0643aa09` and now). Hosted Graphify was not used. The code was read directly with `git show` and `grep`.

## Verdict

**APPROVE. There are no required findings and nine optional ones (OPT11-1 to OPT11-9).**

- Review 10's required findings are resolved in the text:
  - **RQ10-1:** the review-status statements are corrected.
  - **RQ10-2:** the weaker K5 alternative is removed. ", or at least no device result" no longer occurs anywhere in the file.
- Every OPT9 disposition is applied and truthful.
- No decided rule changes meaning.
- The added *[main]* reading matches the code at `b9aa517c`.
- No new native, hosted or human-audit evidence is claimed.

## Review 10 dispositions (confirmed)

| Finding | Status | Evidence checked |
|---|---|---|
| RQ10-1 | Resolved | The header, 8.2 R1 Status (revision 5) and the Revision 5 record intro state that reviews 7 and 8 covered `a769bc65` and the review-6 corrections, returned REQUEST_CHANGES on other findings, and that review 9 first approved them. This matches the stored files. SPEC-REVIEW-7 lists `git diff 0643aa09 a769bc65` as an object and has "Q2 and Q12" and "Q13" sections. SPEC-REVIEW-8 states that it covers Q2 and Q13. Their required findings (RQ7-1 to RQ7-3, RQ8-1 to RQ8-4) concern other matters. Two optional findings (OPT7-1, OPT8-1) refined the wording of the trailer-form correction, which does not make "on other findings" untrue. The historical `a769bc65` paragraph is kept (OPT11-1). |
| RQ10-2 | Resolved | 2.7 property 3, 4.3 K5, 8.2 R5 and the OPT9-3 row all require -1. The contract is not relaxed. |
| OPT10-1 | Resolved | `Policy::exit` drops the pending read (`lib.rs:788-790`) and `detach` clears `error` (`volume.rs:1017-1026`), at `b9aa517c`. |
| OPT10-2 | Resolved | "(or event)" is removed. The CODE-DELTA Q6 row matches 2.7, the Q6 row and 8.2 R2. |
| OPT10-3 | Resolved | `volume.rs:1068-1079` holds both quarantine paths (failed re-initialisation, reset deadline). `lib.rs:577-579` is the `reset_completed` filter. |
| OPT10-4 | Resolved | The 2.7 "Stale valid content" parenthesis states the portal check as the reporting backstop (see OPT11-4 for two places that were not aligned). |
| OPT10-5 | Resolved | The Q2 row is annotated (see OPT11-3 for Q12 and Q13). |
| OPT10-6 | Resolved | O4 (a) states the delta and that it widens no capability. It adds no owner question. |
| OPT10-7 | Resolved | The final clause of R1 records reviews 9 and 10 and the pending review 11. |
| OPT10-8 | Mostly resolved | One short line remains (OPT11-9). |

The original SPEC-REVIEW-10 text is not available. Its findings, and its count of two required and eight optional findings, were judged from the section 9 dispositions only.

## OPT9 dispositions

- **OPT9-1:** SPEC-REVIEW-7, -8 and -9 exist in `docs/verification/s09-destination-service-20261123/`. They are untracked (`??`), as the disposition says ("to be added by the revision-5 commit").
- **OPT9-2:** a word diff of the 8.1 O17 item between `0643aa09` and now shows exactly the stated edits:
  - the conditional sentences, which were removed or replaced by the decision text;
  - "is" changed to "was" recommended;
  - "(b)" changed to "(b), (d)" in the marking sentence.

  The continuation lines are indented.
- **OPT9-3:** checked against the code at `b9aa517c` (read, not executed):
  - A read ended by its deadline sets `error` with -13 and starts the reset. A read ended by a device error first completes with `Io`, sets `error` with -12 (or `err(e)`), and then starts the reset.
  - The first poll takes `error` and returns that code. `peek` fails with `Offline` in `Resetting`/`Quarantined` for a non-retained request, so the read is not consumed.
  - A further poll goes through `peek(...).map_err(err)` and returns -14.
  - `reset_completed(true)` drops non-mutating requests. `reset_completed(false)` quarantines and keeps the read.
  - A mutating request is `retained_terminal`, so `peek` succeeds and its first poll consumes it. A further poll then returns -1.

  The text is accurate (bound wording: OPT11-8). It is recorded as a K5 delta (4.3) and an R5 precondition (8.2 R5). The Q7 row is annotated.
- **OPT9-4:** the stray-poll obligation is worded the same in 2.7, the Q6 row, the CODE-DELTA Q6 row and 8.2 R2. The RQ7-1 disposition keeps its quoted text.
- **OPT9-5:** applied in 2.7 and refined by OPT10-4 (see OPT11-4). The N26 (d) row attributes the refusal to the step-3 content check and is unchanged.
- **OPT9-6:** `n26e_poll_beyond_bound_is_unknown` exists (`services/destination/tests/broker.rs:154` at `f993a1a3`), as does `n26f_remount_after_expired_poll_bound` (`:216`).
- **OPT9-7:** added as an optional, open CODE-DELTA item. It claims no code.
- **OPT9-8:** the note points to 2.7 step 3 (Q8). The paragraph is unchanged.
- **OPT9-9:** see RQ10-1 and OPT11-2.

## Decided rules, security floor and history

- **O14–O18 keep their meaning.**
  - **O17 (c):** service check (a) plus portal check (b). A mismatch replies `Corrupt` -18 and does not start the safe path, pending O9.
  - **O18:** *sealed* only on equality with the copy encoded and checked in step 4.
  - The edited 2.7 parenthesis assigns the prevention to (a) and the reporting to (b), which matches the 8.1 O17 options.
- **Security floor:**
  - No capability is widened, and the 2.2 check order is not touched.
  - Disk bytes stay untrusted, and the FNV-64 threat model is unchanged.
  - The K5 delta is a stricter kernel obligation that the contract already required.
- **No history rewritten:**
  - Every hunk inside section 9 lies in revision-5 text: the note after the `a769bc65` paragraph, the Revision 5 record, CODE-DELTA (revision 5), and the new Review 9, 10 and 11 parts.
  - The `a769bc65` paragraph and the Review 7 and Review 8 paragraphs and tables are unchanged.
- **Evidence:**
  - The *[main]* readings are marked "read, not executed".
  - *[PR #413]* claims were checked at `f993a1a3`.
  - No native, hosted or human-audit evidence is claimed.
- **Consistency:** a whole-file grep for "await", "not reviewed", "must cover", "at least no device result", "(or event)", "K5 delta", "stray-poll", "is recommended", "review 10/11", `a769bc65`, `origin/main` and `b9aa517c` found:
  - live statements consistent with the header and R1;
  - historical statements only inside the history that the text says it keeps (the `a769bc65` paragraph, the Review 7 and 8 paragraphs, the RQ8-1 row).

  The evidence-labels paragraph (`035b44e5`) does not conflict with the `b9aa517c` reading, because the files are identical.

## Findings

### Required

None.

### Optional

| ID | Location | Problem | Fix |
|---|---|---|---|
| OPT11-1 | 9, `a769bc65` paragraph and its revision-5 note | "These relabelled markers and this paragraph were not reviewed" stands without a pointer to the later coverage. The note was edited in this pass. | Add to the note that reviews 7 and 8 covered them and review 9 first approved them. |
| OPT11-2 | 9, Review 9 table, OPT9-9 row | It still says in the present tense that the header and R1 state "review 10 (pending)". This is now superseded. The RQ8-1 row is stale in the same way and is deliberately kept. | Add "(superseded after review 10)". |
| OPT11-3 | 9, Q12 and Q13 rows | They read "must cover" with no outcome, unlike the annotated Q2 row. | Annotate as in Q2. |
| OPT11-4 | 9, Q3 row annotation; OPT9-5 row | They say the portal check "reports the export as failed" with no condition. 2.7 says it would report only if the service check did not. | Align with the 2.7 (OPT10-4) wording. |
| OPT11-5 | Header; 8.2 R1 | Review 9's APPROVE is stated without its condition (RQ8-2/OPT9-1). The review files are still untracked, and SPEC-REVIEW-10 is not stored. | Name the condition, and add SPEC-REVIEW-7 to -10 in the revision-5 commit. |
| OPT11-6 | 4.3 K5; 8.1 O4 (a) | "One delta" can read as exhaustive. Only property 3 was compared against `main` in this pass. | Write "one delta identified so far". |
| OPT11-7 | 9, Q9 row | "Section changed" omits 8.1 O17, which the O17 note attributes to Q9. | Add it. |
| OPT11-8 | 2.7 property 3 | "A further poll returns -14" has no bound in its own clause. After an acknowledged reset, the code returns -1. | Write "a further poll while the reset runs returns -14". |
| OPT11-9 | Header; Revision 5 record intro | Editorial: one line was not rewrapped, and "the first review to approve" has no object. | Rewrap; add "it". |

## Not checked

- No build, test, Clippy, fmt or mutation run.
- The M3 broker behaviour of property 3 was read from code, not executed natively.
- No `git fetch` was run: `origin/main` is the local ref `b9aa517c`, which may lag the remote.
- SPEC-REVIEW-10 is not available. Its findings were judged from the section 9 dispositions only.
- Whether the stored SPEC-REVIEW-7, -8 and -9 are verbatim cannot be confirmed: no originals are available, and SPEC-REVIEW-9 carries a provenance footer added by the lead.
- The properties of the 2.7 broker contract other than property 3 were not re-compared against the M3 broker on `main` (OPT11-6).
- Native K5 behaviour, R5 to R8, the portal (O17 (b), N37 (c)) and hosted CI.

An AI review is not a human audit.

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_89347062-243` (review of the lead's fix pass over the optional findings of SPEC-REVIEW-9, revision 5, base: the review-9 text, sha256 `210d480d…`). It returned APPROVE, so the workflow ran no fix pass. Its nine optional findings are deferred to the next revision; the lead added only the status record of this review to the spec (section 9, "Review 11"), which no review has checked. An AI review, not a human audit.
