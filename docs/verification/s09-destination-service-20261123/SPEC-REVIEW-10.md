# SPEC-REVIEW-10: S09 destination service specification, revision 5 (lead pass after review 9)

- **Date:** 23 November 2026
- **Object:** `docs/implementation/S09-DESTINATION-SERVICE.md`, uncommitted revision-5 working tree in worktree `s09-destination-service` (HEAD `a769bc65`), sha256 `2dc9b29f4ca8b328c42f802b65750945e19cc1bd2005ce80ff6c52c296c3656a`.
- **Scope:** only the lead's pass after review 9. This is `diff -u` of the review-9 snapshot against the object.
- **Base:** the review-9 snapshot `spec-rev5-as-review9.md`, sha256 `210d480ddb0f6ed08192cbdc123a5cd4b04d6962c8ca65556e1882168878cd39`, which SPEC-REVIEW-9 approved (no required findings, OPT9-1 to OPT9-9).
- **Evidence compared:**
  - code claims against draft PR #413 at `f993a1a3` (worktree `s09-destination-core`, read only);
  - *[main]* claims against `035b44e5` and the local `origin/main` ref `b9aa517c` (not re-fetched). `kernel/src/m2/m3/volume.rs` and `crates/virtio-core/src/lib.rs` are identical in both (`git diff --stat` is empty);
  - the newly stored SPEC-REVIEW-7.md, SPEC-REVIEW-8.md and SPEC-REVIEW-9.md.
- **Reviewer:** an independent AI reviewer that did not write the text. This is an AI review, not a human audit.
- **Method:** read-only. No file was created, edited or deleted. Nothing was committed, and no build, test or mutation was run. Only `git`, `diff`, `sed`, `grep` and `sha256sum` were used.

## Verdict

**REQUEST_CHANGES. There are two required findings (RQ10-1, RQ10-2) and eight optional ones (OPT10-1 to OPT10-8).**

Both required findings are wording fixes. Neither changes a decided rule or the security floor.
- **RQ10-1** is a false review-status claim: that `a769bc65` was first reviewed by review 9.
- **RQ10-2** is an internally inconsistent K5 target that weakens property 3: "-1, or at least no device result".

## OPT9 dispositions

| Finding | Applied correctly and truthfully? | Evidence checked |
|---|---|---|
| OPT9-1 | Yes | SPEC-REVIEW-7.md, -8.md and -9.md exist, untracked, in `docs/verification/s09-destination-service-20261123/`. The disposition correctly says they are still to be added by the revision-5 commit. Whether they are verbatim was not checked. |
| OPT9-2 | Yes | The O17 question text was compared with `0643aa09`. Exactly three edits exist: the removed conditional sentences ("If latching is chosen ...", "Each part lapses ...", "R7 and the R8 activation wait on O17"), "is recommended" changed to "was recommended", and N37 (d) added to the marking sentence. The meaning is unchanged, and the continuation lines are now indented. |
| OPT9-3 | Partly. The reading of `main` is accurate; the stated K5 target is not (RQ10-2; OPT10-1, OPT10-3). | Read, not executed. `volume.rs:886-912`: `error.take()` consumes only if `peek` succeeds; otherwise the code falls through to `peek(...).map_err(err)`. `err`: `Offline` maps to -14. `lib.rs` `peek`: `Resetting \| Quarantined` returns `Ok` only for a retained mutating terminal result and `Err(Offline)` otherwise. `start_reset` sets `Unknown` on an unfinished mutating request. `reset_completed(true)` keeps only mutating terminal results. `reset_completed(false)` sets `Quarantined` and keeps `pending` (volume.rs:1068-1079). The first poll of a timeout or a device error returns -13, -12 or `err(e)` for a read. A further poll returns -14 while resetting or quarantined, and -1 after an acknowledged reset. A mutating request is consumed on its first poll. |
| OPT9-4 | Mostly | 2.7 and the Q6 row now use the R2 wording. The CODE-DELTA Q6 row keeps "(or event)" (OPT10-2). The RQ7-1 disposition keeps its quoted text, as disclosed. |
| OPT9-5 | Yes, with a wording nuance (OPT10-4) | The parenthesis attributes prevention to the service check. The N26 (d) row (l.1625) attributes the refusal to "the step-3 content check", as the disposition claims. |
| OPT9-6 | Yes | `n26e_poll_beyond_bound_is_unknown` is at `services/destination/tests/broker.rs:154` and `n26f_remount_after_expired_poll_bound` at `:216`, both at `f993a1a3`. |
| OPT9-7 | Yes | Recorded as optional and open in the CODE-DELTA Q4 row, with the opt-outs named. No property is relaxed. |
| OPT9-8 | Yes | The note qualifies the historical paragraph by the 2.7 step 3 precondition and does not edit the paragraph. |
| OPT9-9 | No (RQ10-1) | The stale "await re-review" statements are replaced, but the replacement claims that no review covered `a769bc65` before review 9. |

## Review-status consistency (header, 8.2 R1, section 9)

The three locations agree with one another:
- review 9 returned APPROVE with no required findings and OPT9-1 to OPT9-9;
- the lead's pass followed;
- review 10 is pending ("These edits are to be checked by review 10"; "Review 10. Pending.").

They are not all true. The header (l.83-84), R1 ("were first reviewed by review 9") and the Revision 5 record (l.2383-2384) claim that no review covered `a769bc65` before review 9. Three sources contradict this:
- SPEC-REVIEW-7 reviewed `git diff 0643aa09 a769bc65` and confirmed Q2 and Q13;
- SPEC-REVIEW-8 states "This review covers the owner-decision commit `a769bc65` (Q2) and the three review-6 hand corrections";
- the file's own Q12 and Q13 rows name reviews 7 and 8.

This is RQ10-1. The Review 9 paragraph's sentence "It covered `a769bc65` (Q2), the review-6 corrections in their final form (Q13) and the security floor (Q12)" is true on its own.

## Decided rules and history

- **No O14–O18 rule changed meaning.**
  - O17 (c) is intact: service check (a), portal check (b), `Corrupt` -18, no safe path, no latch pending O9 (2.7, 8.1 O17).
  - O18 is untouched.
  - The O17 note edits only describe edits to the question text.
- **Section 9 history before revision 5 is not rewritten.**
  - Every section 9 hunk lies in revision-5 text: the note after the `a769bc65` paragraph, the Revision 5 record intro, the Q3, Q6 and Q7 rows, and the CODE-DELTA (revision 5) Q4 and Q6 rows.
  - The rest is appended: "Review 9" and "Review 10. Pending."
  - The `a769bc65` paragraph and the Review 7 and Review 8 paragraphs and tables have no hunks.
  - A word-level diff confirms that the R1 Status (revision 3) and Status (revision 4) clauses are unchanged.

## Security floor and evidence

- **No capability is widened and the check order of 2.2 is untouched.** No hunk touches 2.2.
- **Section 4 has a hunk this time.** The new 4.3 K5 sentence only adds a broker obligation. Review 8 had recorded no section-4 hunks. Its target wording is the subject of RQ10-2.
- **Disk bytes stay untrusted, and the FNV-64 threat model is unchanged.**
- **No native, hosted or human-audit evidence is claimed.** The new *[main]* reading is labelled "read, not executed" and holds for `035b44e5` and `b9aa517c`. The one new *[PR #413]* claim (the N26 (e) test name) holds at `f993a1a3`.

## Findings

### Required

| ID | Location | Problem | Fix |
|---|---|---|---|
| RQ10-1 | Header l.83-84; 8.2 R1 Status (revision 5); 9 Revision 5 record l.2383-2384; OPT9-9 row | Says no review covered `a769bc65` (and the review-6 corrections) before review 9. SPEC-REVIEW-7, SPEC-REVIEW-8 and the Q12 and Q13 rows show that reviews 7 and 8 covered them, with REQUEST_CHANGES on other findings. | State that reviews 7 and 8 covered them and review 9 is the first to approve. Correct the OPT9-9 disposition. Leave the historical paragraphs as written. |
| RQ10-2 | 2.7 property 3 (l.1062-1064); 4.3 K5; 8.2 R5; OPT9-3 row | "returns -1, or at least no device result" weakens property 3, which requires -1. The term is undefined. It contradicts the Review 9 paragraph's "no resolution ... relaxes a property of the broker contract". | Delete ", or at least no device result" in all four places. A weaker target would be a contract change with its own disposition. |

### Optional

| ID | Location | Problem | Fix |
|---|---|---|---|
| OPT10-1 | 2.7 property 3; 4.3 K5 ("until reboot") | `Policy::exit` (lib.rs:788-790) and `detach` (volume.rs:1017-1026) clear the read when its owner exits. | "until reboot or until the owning process exits". |
| OPT10-2 | CODE-DELTA (revision 5) Q6 row | "(or event)" survives the alignment claimed in OPT9-4. | Drop it, or disclose it in the disposition. |
| OPT10-3 | 2.7 property 3 citations | `volume.rs:1065-1079` starts in the success branch. `lib.rs:576-579` starts at a comment. | Cite `volume.rs:1068-1079` and `lib.rs:577-579`. |
| OPT10-4 | 2.7 "Stale valid content" parenthesis | Could be read as saying the portal trailer comparison reports the decided case. In that case SEAL fails first; (b) is the backstop for a service defect. | "... and that the decided portal check would report as a failed export if the service check did not". |
| OPT10-5 | 9 Q2 row | Still an open obligation ("must confirm them ... was not reviewed"), with no annotation. | Annotate "(confirmed by reviews 7 and 8; approved by review 9)". |
| OPT10-6 | 8.1 O4 (a); 4.3 K5 | The approval text names per-device state only. K5 now carries a stated delta. | Point O4 (a) to the K5 delta. |
| OPT10-7 | 8.2 R1 final clause | The clause is hard to parse. "were applied" is loose: OPT9-1 needed no text change, and OPT9-7 is an open optional item. | Reword as "dispositioned in a separate fix pass". |
| OPT10-8 | Header l.84; 2.7 l.861; 9 l.2384 | Edited lines are not rewrapped. | Rewrap them. |

## Not checked

- Whether SPEC-REVIEW-7.md, SPEC-REVIEW-8.md and SPEC-REVIEW-9.md are verbatim copies of the review outputs. Only their presence and the content quoted above were checked.
- Any build, test, Clippy, fmt or mutation run. The M3 broker behaviour after a reset was read from code only, not executed. Native K5 behaviour does not exist yet.
- The `origin/main` ref was not re-fetched. `b9aa517c` is the local tracking ref.
- Hosted Graphify was not used. The code was read directly at the stated commits.
- Text outside the review-9 to working-tree diff, beyond the targeted greps for the touched terms: "stray", "device result", K5, O4 (a), `a769bc65`, the review-status phrases, "decided checks", and N26 (d).

An AI review is not a human audit.

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_89347062-243` (review of the lead's fix pass over the optional findings of SPEC-REVIEW-9, revision 5, base: the review-9 text, sha256 `210d480d…`). It returned REQUEST_CHANGES; the workflow fix pass applied all ten findings and review 11 re-reviewed the result. An AI review, not a human audit.
