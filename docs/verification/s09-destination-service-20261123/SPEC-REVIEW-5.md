# SPEC-REVIEW-5: independent security re-review of S09 destination spec revision 3 (after the review-4 fix pass)

- **Reviewer role:** independent AI security reviewer (read-only). This reviewer did not write revision 3 or its fix pass.
- **Date:** 23 November 2026
- **Subject:** `docs/implementation/S09-DESTINATION-SERVICE.md`, uncommitted working-tree revision 3 (with the fix pass for SPEC-REVIEW-4) in worktree `s09-destination-service`. Diff base: `5d97cd76` (revision 2).
- **Implementation compared:** worktree `s09-destination-core` at core commit `37b7bd0c` (`services/destination` R2 Rust core, `tools/destination_disk.py` R3 oracle, `tests/test_volume_destination_disk.py`). *[main]* claims were checked against `035b44e5` where stated.
- **Input ledger:** `rev3-ledger.md`, items L1–L29. **Previous review:** SPEC-REVIEW-4 (RQ-1–RQ-3, OP-1–OP-10).
- **Status of this review:** this is an AI review, not a human audit. It does not accept any criterion and does not decide any owner item.

Locations are given by section and row, because line numbers have shifted since review 4.

## Verdict

**REQUEST_CHANGES: 2 required findings, 8 optional findings.**

- **Every finding of review 4 is resolved** in the normative text. The table below gives the evidence for each.
- **Ledger coverage is still complete.** Each of L1–L29 has exactly one row in the section 9 revision-3 record (grep count 1 for each ID). The rows changed after review 4 (L2, L5, L10, L22, L29) match the text.
- **The security floor holds.** The fix pass adds only refusals, fail-closed states or test requirements. O1–O13 are unchanged, and no O, N, R or K identifier is renumbered.
- **Why changes are requested.** The fix pass introduced, or left in place, two inconsistencies:
  - The new N36 (a1)/(a2) cases silently assume the *proposed* O15 answer.
  - 5.3 and the 2.5 "Reasons" paragraph still map every kernel -14 after the boot mount to `Io`. The governing 2.5 table and N33 map a write completion of -14 to `Unknown`.

## Status of the review-4 findings

| Finding | Status | Evidence (section) |
|---|---|---|
| RQ-1 (N36 (a) impossible setup; `Busy`/`Full` order) | Resolved | 2.3 now states that `Busy` and `Full` are mutually exclusive, with the commit-order reason. The determinism argument is removed. N36 is split into (a1) another lane writing and (a2) nothing writing with no free sector, or with 16 sealed. The L10 row is corrected. The reason holds: `create()` tests `Full` against the committed catalog only (`service.rs:449-451`), and only that entry's SEAL commits. **New issue in the split rows: see RQ5-1** |
| RQ-2 (N29 u64-maximum expected result) | Resolved | N29 now gives the u64-maximum case its own expected result: every SEAL `Corrupt` -18, no catalog write, entry discarded, no `Corrupt` mount state, INFO and LIST unchanged, no later export seals. 2.7 step 4 states that such a catalog never commits again. This matches `service.rs:571-577`: checked add, `discard()`, `Corrupt`, no safe-path latch |
| RQ-3 (copy sequence ≠ 0 missing from list) | Resolved | 2.6 list has "copy sequence ≥ 1 †". N34 gains a zero-record sequence-0 case. The L5 row is extended. Both implementations refuse sequence 0 (`catalog.rs:153`; oracle `decode_copy` `Invalid("sequence")`). The fix pass claims that no other rule lives only in a field description. This was re-checked against the header and record field descriptions and holds: the destination ID and the trailer FNV have no validity rule in either place |
| OP-1 (submission row incomplete) | Resolved | 2.5 table row 2: "any other submission result than 0 or -8 … or another error" |
| OP-2 (meaning of `NoDevice`) | Resolved | 2.5 "Reasons": "a mount, remount or FORMAT met kernel -14 … (INFO, 5.3)", or BIND |
| OP-3 (sealing sequence caught only sometimes) | Resolved | 2.7 step 4 extension check now requires "the new record's sealing sequence equal to the new sequence". 2.6 is corrected and the clause is kept out of the mount rule. N29 uses a fixture that the validity rules accept (sealing sequences 2 and 5 under sequence 9, new record at 7). See OPT-1 for a small wording point |
| OP-4 (O14 oracle claim) | Resolved | The O14 parenthetical matches `decode_record` and `decode_copy` at `37b7bd0c`: length > 32 is refused through the "label padding" rule; length 0 and entry ID 0 are accepted. The encoder default `record.get("sealed_sequence", 1)` is at `destination_disk.py:219`, as cited |
| OP-5 (other remount results) | Resolved | 2.7 makes a remount that ends `Corrupt` leave the binding *unknown*. This matches R2: `catalog_failure` resolves only when `State::Mounted` (`service.rs:630-646`). The O16 proposal now also covers another valid catalog. The definition is in 2.7, the state is in the 2.2 check-7 table and the 2.6 failed-mount state, and N35 (b)–(d) test it. O16 stays *proposed* (8.1) |
| OP-6 (reset models) | Resolved | The section 6 preamble and N33 add the read-timeout reset case with unflushed APPEND writes, under both models. The invariant "never a listed entry that fails VERIFY" holds: with the cache dropped, the step-3 readback refuses |
| OP-7 (hosted cross-check deadline) | Resolved | 8.2 R5 preconditions: "the R3 reverse differential exists and the R3 cross-check runs in a hosted gate". R3 says "both are required before R5" |
| OP-8 (K1 dependency) | Resolved | 2.5 "Reasons": the mapping "is to be decided again" if K1 is approved |
| OP-9 (R2 broker test) | Resolved | The L2 row names `kernel_offline_on_write` and the -14 arms of `submit_error`/`poll_error`. Verified: `tests/broker.rs:144-148` asserts `(-14, Status::NoDevice)`, and `device.rs:55` and `:63` map -14 to `NoDevice` |
| OP-10 (editorial) | Resolved | All six items are applied: the safe-path exception (2.5); readback `Io` in the diagram; "from the admission of the request" (2.7); the trailer FNV worded as "the FNV-64 value stored in bytes 8..16"; *[PR #413]* applied in O14–O16 and in R2/R3 status; the long O16 label in the check-7 table |

## Required findings

### RQ5-1: N36 (a1) and (a2) silently assume the proposed O15 answer

**Location:** section 6, N36 (a1)/(a2). Related: 2.2 check 6 (O15) and 8.1 O15.

**Problem.**
- (a1) and (a2) send two CREATEs on the same export binding:
  - a CREATE with a content-invalid label, expected `Invalid` -1;
  - then a CREATE with a valid label, expected `Busy` -21 or `Full` -17.
- A content-invalid label passes checks 1–6. Check 2 checks only the label length (2.2 check 2, 2.3), and `valid_name` is applied by the operation after check 7.
- Under the O15 alternative ("every CREATE that passes checks 1–6 uses the binding's CREATE"), the first CREATE therefore uses up the binding. The second CREATE then returns `Access` -5 at check 6, not `Busy` or `Full`.
- The rows state their expected result without condition. Only case (c) of the same row names the alternative.
- This is the defect class of review-4 RQ-2 (an expected result that cannot hold in a stated configuration). It also assumes an undecided owner item without marking it. A test written to (a1)/(a2) passes on R2 only because R2 follows the proposal (`permitted()`: CREATE only while `Export::Fresh`, `service.rs:789`).

**Fix.** Choose one:
- send the valid-label CREATE on a fresh export binding (UNBIND, then BIND with a higher generation), so that (a1)/(a2) test the refusal order under either answer; or
- mark (a1)/(a2) "under the proposed O15 rule" and give the alternative's result for the second CREATE (`Access` -5 at check 6), as (c) does.

### RQ5-2: 5.3 and 2.5 "Reasons" contradict the 2.5 table and N33 for a write completion of -14

**Location:** 5.3 ("A kernel -14 or offline -13 on any destination request after the boot mount … the request gets `Io` -12"); 2.5 "Reasons" ("A kernel -14 after the boot mount … is answered `Io`"); against the 2.5 "Device results" table (last row: a completion of -14 on a write or flush → `Unknown` -16) and N33 ("`Unknown` -16 for completion -13, -16 and -14").

**Problem.**
- The 2.5 table is declared governing ("follows this table, whatever the operation"). It distinguishes the reply by phase:
  - a -14 at submission → `Io`;
  - a -14 at completion → `Io` for a read, `Unknown` for a write or flush.
- 5.3 and the "Reasons" paragraph state an unconditional `Io` for any -14 after the boot mount. An implementation that follows 5.3 answers a write completion of -14 with `Io`, and N33 then fails.
- **Severity: low.** Both replies start the safe path, and neither claims success. However, the two normative statements disagree, and the review task names exactly this pair (2.5 vs 5.3).
- The K5 broker contract (2.7, property 5) produces -14 only at submission (offline). On `main`, the broker stores completion errors of -16, -13 or -12 (`volume.rs:1087-1094`, `:1176-1182` at `035b44e5`). So the intended reading is "on submission".

**Fix.**
- In 5.3 and in the 2.5 "Reasons" paragraph, qualify the statement as "a submission refused with kernel -14 (offline) or -13 (quarantined) gets `Io` -12".
- Add that a completion result follows the "Device results" table.
- Alternatively, drop completion -14 from the table and from N33 as outside the broker contract, and say so. Do not do both.

## Optional findings

- **OPT-1: the "only when" claim in 2.6 is slightly too strong** (2.6, the paragraph after the validity list).
  - The text says "The validity rules alone catch a wrong sealing sequence on a new record only when the committed last record's sealing sequence equals the committed sequence."
  - A committed copy with zero records and sequence 1 (a fresh FORMAT) also forces the value: the new record needs sealing sequence ≥ 2 and ≤ 2.
  - Suggested wording: "only when the committed copy's highest sealing sequence (1 if it has no record) equals its sequence".
  - The normative step-4 clause is unaffected.
- **OPT-2: the O16 match tests only the entry ID** (2.7, the definition of "that copy plus exactly the provisional record").
  - The match is "passes the step-4 extension check … and whose new record carries the provisional entry ID". It does not compare the new record's length, trailer FNV or label with what SEAL attempted to write.
  - The service still holds the encoded copy that it tried to write. A byte comparison with that copy (or with the committed copy) is both simpler and stricter.
  - This matters only for a misbehaving device, which 5.4 places out of scope. Hence optional.
- **OPT-3: the device-result wording is stale after L1.**
  - 2.7 (the SEAL steps 4–5 failure paragraph and the safe-path trigger) and N13 list a write or flush failure as "`Io`, `Timeout` or `Unknown`". The 2.6 mount rule lists a read failure as "`Io`, `Timeout` or `Unknown`".
  - Under the 2.5 mapping, a write or flush yields only `Io` or `Unknown`, and a read yields only `Io` or `Timeout`.
  - This is harmless, because the operative condition everywhere is "any device result other than success". The lists could still be aligned.
- **OPT-4: the L1 row names no code change**, although `poll_error` (`device.rs:61-68`) differs from the new table in several places:
  - completion -13 on a write: R2 gives `Timeout`, the table `Unknown`;
  - completion -16 on a read: R2 gives `Unknown`, the table `Io`;
  - the default arm on a write: R2 gives `Io`, the table `Unknown`.

  The -14 arms are already named in L2. The generic statement ("where the PR #413 code differs … the code must change") covers these, and N33 would catch them. Naming them, as OP-9 did for L2, would make the R2 change list complete. No R2 test currently asserts `Timeout` for a write (grep).
- **OP-5 follow-up: the L1 and L7 rows were not updated** for review-4 OP-1 (every submission result) and OP-3 (the step-4 sealing-sequence clause). Only the review-4 disposition table records them. Consider one clause in each row.
- **OPT-6: early refusal at the u64 maximum** (2.7 step 4, N29).
  - With sequence or next entry ID at the u64 maximum, CREATE is still answered `Ok`. A whole export is written and flushed before SEAL refuses it.
  - CREATE could refuse such a catalog up front (for example `Full` or `Corrupt`), with no device write.
  - This is a design choice that fails closed either way; if adopted, N29 would change.
- **OPT-7: the section 6 preamble says only revisions 1 and 2** ("Rows added or changed in revisions 1 and 2 name, after 'Kills:'"). The revision-3 rows N33–N36 follow the same convention. Say "revisions 1–3".
- **OPT-8: editorial.**
  - N36 (b) needs a scope of at least 1 043 sectors, so that 525 280 bytes can be appended. Otherwise `Full` is met during setup. This is implicit only.
  - Section 9 says that SPEC-REVIEW-4 is not yet stored. That is true: `docs/verification/s09-destination-service-20261123/` holds only SPEC-REVIEW-1–3. Store SPEC-REVIEW-4 and this review, and link them from the header and section 9.

## Ledger coverage check (L1–L29)

| ID | Record rows | Normative text present | Review note |
|---|---|---|---|
| L1 | 1 | 2.5 table, 5.3 | Present. See RQ5-2 (5.3 vs table on completion -14), OPT-3 and OPT-4 |
| L2 | 1 | 2.5, 5.3 | Corrected after review 4: names `kernel_offline_on_write` and the -14 arms (verified). See RQ5-2 |
| L3 | 1 | N13 | Three operation lanes; consistent with 2.8 |
| L4 | 1 | 2.6 mount rule, N29, N30 | Unchanged by the fix pass; still correct |
| L5 | 1 | 2.6 list, N34 | Corrected after review 4: copy sequence ≥ 1 added (RQ-3) |
| L6 | 1 | 2.3, 2.6, O14 | Proposed; fails closed. The `valid_name` equivalence with catalog-core is carried from review 4, not re-verified |
| L7 | 1 | 2.6, 2.7 step 4 | Present. The step-4 clause comes from OP-3; see the OP-5 follow-up above |
| L8 | 1 | 2.3, N36 (b) | `Range` after `Stale`; justified |
| L9 | 1 | 2.2 check 6, O15 | Proposed. R2 follows it (`service.rs:441-466`, `:789`, re-verified). See RQ5-1 |
| L10 | 1 | 2.3, N36 | Corrected after review 4: mutual exclusion stated, with its reason. See RQ5-1 |
| L11 | 1 | 2.6 Format | Correct |
| L12 | 1 | 2.3, 2.5 | Correct |
| L13 | 1 | 2.3 LIST | Correct |
| L14 | 1 | 2.7 | Matches `device.rs` (`submitted = broker.now()` after admission) |
| L15 | 1 | section 7 | Correct |
| L16 | 1 | 2.3 INFO | Correct |
| L17 | 1 | 2.3, 2.7 step 3 | The `archive.rs:366-377` claim is carried from review 4, not re-verified |
| L18 | 1 | 2.7 broker contract | Consistent with `volume.rs` at `035b44e5` (completion codes -16/-13/-12; offline -13/-14 at submission) |
| L19 | 1 | section 6 preamble | Extended with the N33 case (OP-6) |
| L20 | 1 | 2.3, R2, R5 | Correct |
| L21 | 1 | N29, R5 | The hook is still `#[doc(hidden)] pub tamper` (`service.rs:146-149`), re-verified. This is a required code change before R5 |
| L22 | 1 | 2.2, 2.6, 2.7, N35, O16 | Corrected after review 4 (OP-5); proposed. See OPT-2 |
| L23 | 1 | 2.6 | Correct; not a weakening |
| L24 | 1 | 2.6 list | Correct (`catalog.rs:178-179`; oracle `decode_record`) |
| L25 | 1 | 2.6 | Correct |
| L26 | 1 | section 7 | Correct |
| L27 | 1 | 2.6, section 7 | Correct |
| L28 | 1 | section 7 | Correct |
| L29 | 1 | R3, R5, N34 | Corrected after review 4: the hosted cross-check is an R5 precondition (OP-7) |

## Security-floor checks

- **Check order unchanged.** Check 2 still checks only label length. The hunks in 2.2 add the O15 sentences to check 6 and the O16 entries to the check-7 table; they move no check.
- **Mount still fails closed.**
  - A read error is a failed mount.
  - O16 (proposed) adds failed-mount cases, which is stricter.
  - A remount that ends `Corrupt` now keeps the binding *unknown*, which is stricter than "not sealed".
- **Safe-path latch.** No path re-admits CREATE, APPEND, SEAL or FORMAT after the trigger. In the alternative to O16, the latch is kept: R2 refuses FORMAT with `Access` in the blank state when `safe_path` is set (`service.rs:352`).
- **Capabilities.** The saved diff has no hunk in 3.x, 4.x, 5.1 or 5.2 (hunk headers jump from old line 578 to 966). No right or grant changes.
- **Disk bytes.** No new trust is placed in disk bytes. The fix pass adds the copy-sequence rule, the step-4 sealing-sequence clause and the O16 contradiction checks, which are all refusals.
- **Identifiers.**
  - The O1–O13 bullets are unchanged in the diff; O14–O16 are new and marked *proposed*.
  - N13, N29 and N30 are edited in place; N33–N36 are new.
  - R1–R8 and K1–K5 are not renumbered.

## Are normative choices independent of the code?

- **Owner decisions are marked proposed.** O14–O16 are *proposed*, R7 and R8 are gated on them, and the R2 divergence (the O16 alternative) is stated.
- **Exception:** N36 (a1)/(a2) assume the O15 proposal unmarked (RQ5-1).
- **Choices argued from design or fail-closed reasoning:**
  - the RQ-1 reason (commit order);
  - the OP-3 clause (kept out of the mount rule so that it does not refuse valid copies after O3);
  - the OP-5 extension (no resolution from a read that contradicts the catalog mounted earlier in the boot).

  None of them simply copies the code. R2 differs from the L1/L2 mapping, and that difference is recorded as a required code change.

## Claims about the code at `37b7bd0c`

| Claim (section) | Result | Evidence |
|---|---|---|
| R2 follows the O15 proposal (8.1 O15) | true | `create()` `service.rs:441-466`; `permitted()` CREATE only when `Export::Fresh`, `:789` |
| R2 implements the O16 alternative, including the extended cases (8.1 O16) | true | `catalog_failure` `:630-646`: resolves from whatever catalog mounts; blank state with `safe_path` → FORMAT `Access` `:352` |
| A remount that ends `Corrupt` leaves the binding *unknown*, matching R2 (9, OP-5) | true | `:634-645` resolve only on `State::Mounted` |
| A copy at the u64 maximum passes the validator, and the increments are checked (2.7 step 4) | true | `catalog.rs:151-159` has no upper bound on sequence or next entry; `service.rs:571-577` |
| Both implementations refuse sequence 0 (9, RQ-3) | true | `catalog.rs:153`; oracle `decode_copy` |
| Oracle: label length > 32 refused by the padding rule; 0 and entry ID 0 accepted (O14) | true | `destination_disk.py` `decode_record`/`decode_copy` |
| Encoder default sealing sequence 1 at `destination_disk.py:219` (9, OP-4) | true | line 219 |
| `kernel_offline_on_write` asserts `NoDevice` for -14 (L2) | true | `tests/broker.rs:144-148` |
| Tamper hook is `#[doc(hidden)] pub` (N29, L21) | true | `service.rs:146-149` |
| Cross-check skipped in hosted CI; 7 images (8.2 R3) | true (structure only) | `skipUnless(DEST_ORACLE_OUT)` at `tests/test_volume_destination_disk.py:403`; no `.github/` reference; 7 `emit` calls plus its definition. The agreement result was not re-run |
| Reverse differential absent (8.2 R3) | true | none found |
| SPEC-REVIEW-4 not yet stored (9) | true | `docs/verification/s09-destination-service-20261123/` has SPEC-REVIEW-1–3 only |

## Commands run (read-only)

- `git log`, `git status --short`, `git diff 5d97cd76 -- docs/implementation/S09-DESTINATION-SERVICE.md` (saved to the session scratchpad for reading), and `git show 035b44e5:kernel/src/m2/m3/volume.rs` (lines 900–925, 1060–1100, 1170–1190); all exit 0.
- `sed -n`, `grep -n` and `grep -c` over the spec, the ledger, `services/destination/{src,tests}`, `tools/destination_disk.py`, `tests/test_volume_destination_disk.py` and `.github/`; all exit 0, except a `.github` grep with no match (exit 1).
- `ls docs/verification/s09-destination-service-20261123/`.
- Per-ID grep count of the L1–L29 rows in section 9: 1 each.

## Not checked

- **Fix-pass delta not isolated.** The fix pass is uncommitted on top of the uncommitted revision 3, so its delta cannot be separated from the earlier revision-3 text. This review compared the full revision against `5d97cd76` and against the review-4 findings.
- **Tests not run.** `cargo test`, `clippy` and the Python suites were not run, because they write build output. No oracle probe was re-run this time.
- **Claims carried from review 4, not re-verified:**
  - `archive.rs:366-377` (trailing bytes);
  - the `valid_name` equivalence with `catalog_core::wire::valid_name`;
  - virtio-core `prepare` deadline;
  - the `volume-core` dependency set;
  - "7 images, agreement";
  - the partial status of N7, N10, N11 and N12, and N24 (b) "recorded".
- **Sections not re-read.** Sections 3, 4, 5.1, 5.2 and O1–O12 were not re-read in full; they have no hunk in the diff.
- **No graph query.** Hosted Graphify was not queried: it reflects `main`, not these worktrees.
- **No native or QEMU evidence.** None exists for revision 3.
- **No human audit.**

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_c619eee0-6d3` (review 5). An AI review, not a human audit.
