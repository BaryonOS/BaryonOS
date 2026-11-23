# SPEC-REVIEW-4: independent security review of S09 destination spec revision 3

- **Reviewer role:** independent AI security reviewer (read-only). This reviewer did not write revision 3.
- **Date:** 23 November 2026
- **Subject:** `docs/implementation/S09-DESTINATION-SERVICE.md`, uncommitted working-tree revision 3 in worktree `s09-destination-service`. Diff base: `5d97cd76` (revision 2).
- **Implementation compared:** worktree `s09-destination-core` at core commit `37b7bd0c` (`services/destination` R2 Rust core, `tools/destination_disk.py` R3 oracle, `tests/test_volume_destination_disk.py`). *[main]* claims were checked against `035b44e5`.
- **Input ledger:** `rev3-ledger.md`, items L1–L29.
- **Status of this review:** this is an AI review, not a human audit. It does not accept any criterion and does not decide any owner item.

## Verdict

**REQUEST_CHANGES: 3 required findings, 10 optional findings.**

- **Ledger coverage is complete.** Each of L1–L29 has exactly one row in the section 9 revision-3 record (verified by grep count), and each resolution appears in the normative text it names.
- **The security floor holds.** No resolution weakens it.
- **Owner questions are handled correctly.** O14–O16 are marked *proposed*. The existing O1–O13 are unchanged.
- **Why changes are requested:**
  - one negative-test row (N36 (a)) has a setup that cannot occur;
  - one row (N29) has an expected result that cannot hold for its new case;
  - the copy-validity list that revision 3 says it completes still leaves out one rule.

## Required findings

### RQ-1: N36 (a) cannot be set up, so "`Busy` before `Full`" can never be observed

**Location:** 2.3 CREATE order (lines 329–333), N36 (line 1261), section 9 L10.

**Problem.**
- N36 (a) asks for "another lane writing, and no sector free" at the same time. That state cannot be reached:
  - CREATE returns `Full` when 16 entries are sealed or fewer than one sector is free (2.3, measured against the committed `next_free`).
  - So a writing entry exists only if CREATE passed that check against the then-committed catalog.
  - Only that same entry's SEAL commits a new catalog (2.7). While it is `writing`, the committed entry count and `next_free` do not change.
  - Therefore whenever another lane is writing, at least one sector is free and fewer than 16 entries are sealed.
- The consequence: `Busy` and `Full` never apply together. The L10 convention "`Busy` before `Full` … fixed so that tests are deterministic" can never be observed, and N36 (a) cannot be built. This is the defect class L4 removed from N29/N30.
- The R2 code agrees: `service.rs:445-451` reaches the `Full` test only when nothing is writing.

**Fix.**
- In 2.3, state that `Busy` and `Full` are mutually exclusive, with the reason above. Keep or drop the order convention, but do not justify it by test determinism.
- Split N36 (a) into two cases:
  - another lane writing: invalid label → `Invalid` -1, then valid label → `Busy` -21;
  - nothing writing, no sector free (or 16 entries sealed): invalid label → `Invalid` -1, then valid label → `Full` -17.
- Correct the L10 reason accordingly.

### RQ-2: N29's expected result is wrong for the new u64-maximum case

**Location:** N29 (line 1254); 2.7 step 4.

**Problem.**
- Revision 3 adds to N29: "separately, without the hook, a committed catalog with sequence or next entry ID at the u64 maximum".
- The row's single expected result includes "a later export seals". That is true for the hook cases: the next SEAL encodes a correct copy.
- It is false for the u64-maximum case. The committed catalog keeps the maximum, so every later SEAL overflows the same checked increment and replies `Corrupt` -18. A test written to this row must either fail or drop the assertion.

**Fix.** Give the u64-maximum case its own expected result:
- every SEAL replies `Corrupt` -18;
- no catalog sector is written;
- the writing entry is discarded;
- the service is not in the `Corrupt` mount state;
- INFO and LIST are unchanged.

Keep "a later export seals" for the hook cases only.

### RQ-3: the 2.6 validity list is still incomplete (copy sequence ≠ 0)

**Location:** 2.6 header layout (line 459, "sequence u64 ≠ 0") against the "valid only if" list (lines 473–492); revision-3 paragraph (line 37: "completes the 2.6 copy-validity list"); N34.

**Problem.**
- L5 and L24 exist because a rule stated only in a field description, and missing from the list, let an implementation that follows the list literally diverge. The R3 oracle did exactly that for entry ID 0 and label length 0.
- The rule "sequence ≠ 0" is in the same position: it appears only in the header field description.
- Both implementations enforce it (`catalog.rs:153`; oracle `Invalid("sequence")`, confirmed by an in-memory probe), but a new implementation written from the list would accept a zero-record copy with sequence 0.
- This is not exploitable: such a copy has no records, and its next SEAL would fail the pre-write check because of the sealing-sequence rule. However, the revision's claim of completeness is false, and N34 does not cover the rule.

**Fix.**
- Add "copy sequence ≥ 1" to the list, marked †.
- Add a sequence-0 copy case to N34.
- Add one line to the L5/L24 record.

## Optional findings

- **OP-1: the "Device results" table does not cover every broker result** (2.5, lines 411–417).
  - The table has no row for a submission result other than -8, -13 or -14 (for example a `prepare` refusal). The "Reasons" prose covers it ("a submission refused by the broker was not admitted: `Io`"), and R2 maps it to `Io` (`device.rs:86`, `submit_error`).
  - Fix: change the second row to "any other submission result (-13, -14, or another error)".
- **OP-2: the stated "one meaning" of `NoDevice` is narrower than the rules** (2.5 lines 428–431; L2).
  - The text says `NoDevice` means "no device was usable at mount (INFO, 5.3)". The same section, and the failed-mount row of the 2.2 check-7 table, make INFO report `NoDevice` after a FORMAT write or flush meets -14, and after a remount meets -14. By the L2 reasoning, both are mid-boot device faults.
  - The rules themselves are consistent with each other; only the reason sentence is inaccurate.
  - Fix: restate the meaning as "kernel -14 met by a mount, remount or FORMAT", or report `Io` in those two cases.
- **OP-3: the step-4 check catches a wrong sealing sequence only in some cases** (2.6 line 498; 2.7 step 4; N29).
  - The text says the step-4 check "also catches an encoder that writes a wrong sealing sequence". The validator bounds (≥ 2, ≤ the copy's sequence, strictly ascending) catch every wrong value only when the committed last record's sealing sequence equals the committed sequence.
  - That holds for histories produced by the v1 commit order. It does not hold for a crafted but valid catalog (for example sequences 2 and 5 under copy sequence 9), and it will not hold after any O3 deletion. The extension check of 2.7 step 4 does not require "new record's sealing sequence = new copy sequence".
  - Fix: add that one clause to the step-4 extension check (it fails closed), so that the N29 sealing-sequence case holds for any fixture.
- **OP-4: inaccurate claim about the oracle in O14** (lines 1542–1544).
  - The text says "the oracle does not yet check the label length either". At `37b7bd0c`, `decode_record` refuses label length 33 through its padding rule and accepts label length 0. It also accepts entry ID 0 and sealing sequence 1.
  - Recorded as a consequence, not a spec defect: the test encoder's default `sealed_sequence` of 1 (`destination_disk.py:219`) makes every crafted copy with records invalid under revision 3. The R3 code change must update the default.
- **OP-5: other remount results need the same treatment as O16** (2.7 lines 689–694; O16).
  - The O16 reasoning ("a read that contradicts the catalog mounted earlier in the same boot") applies equally to a remount that:
    - ends `Corrupt`. The text resolves "*not sealed*", while R2 leaves the binding `Unknown` (`service.rs:634-645`).
    - reads a valid catalog with a different destination ID.
    - reads a catalog that is neither the committed copy nor that copy plus exactly the provisional record.
  - In the last two cases, the binding can resolve as *sealed* from another catalog, and bound verify and restore lanes keep entry IDs that now name another catalog's entries.
  - This was already true in revision 2, and 5.4 places malicious devices out of scope. Even so, applying O16 here is a small change that fails closed: leave the binding *unknown* and enter the failed-mount state. Consider folding it into O16.
- **OP-6: the list of rows run under both reset models looks incomplete** (section 6 preamble; L19).
  - Rows under both reset models are named as N13, N24 and N26. A device reset is also caused by a read timeout (completion -13, N33) while another lane has unflushed APPEND writes. If the reset drops the volatile cache, those writes vanish before barrier 1, and SEAL step 3 must answer `Corrupt`.
  - Fix: add that case under both models.
- **OP-7: the hosted cross-check has no route deadline** (8.2).
  - R3 names a hosted cross-check (a workflow change owned by the lead), but no later step depends on it. R5 requires only the reverse differential.
  - Fix: consider making the hosted cross-check an R5 or R8 precondition.
- **OP-8: the `-14 → Io` argument depends on K1 never landing** (2.5 Reasons).
  - That mapping rests on "v1 has no runtime attachment (K1)". K1 is an O4 (a) candidate.
  - Fix: add one line saying that the mapping is revisited if K1 is approved.
- **OP-9: a further R2 test contradicts the L2 resolution and is not named.**
  - The L2 record says it read the N14 test. `tests/broker.rs:144-148` also asserts `NoDevice` for a write submission that meets -14, which the revision-3 mapping contradicts.
  - Fix: list that test among the required code changes.
- **OP-10: editorial.**
  - The "Every failure of a destination write or flush … starts the safe path" sentence (line 431) should except mount and remount FLUSH and FORMAT writes, which go to the stricter failed-mount state.
  - The 2.5 diagram omits a readback `Io`.
  - Line 746 says "from the submission", which conflicts with the admission-based bound that follows it.
  - The phrase "FNV value of the archive's 16-byte trailer" (line 468) can be read as an FNV computed over the trailer. It should say "the FNV-64 value stored in trailer bytes 8..16".
  - The *[PR #413]* evidence label is defined but never applied.
  - "*[proposed; O16]*" in the check-7 table should match the long form.

## Ledger coverage check (L1–L29)

| ID | Record row | Normative text present | Review note |
|---|---|---|---|
| L1 | 1 | 2.5 table, 5.3 | Write without a definite result → `Unknown`. Stricter than revision 2. Differs from R2 (`device.rs:65`), which is recorded as a code change |
| L2 | 1 | 2.5, 5.3 | Correct. See OP-2 and OP-9 |
| L3 | 1 | N13 | Three operation lanes, consistent with 2.8 |
| L4 | 1 | 2.6, N29, N30 | The implication holds: a common record prefix plus contiguity give `next_free` not lower. The new N30 cases are reachable |
| L5 | 1 | 2.6 list | Present. See RQ-3 for the sibling rule |
| L6 | 1 | 2.3, 2.6, O14 | Proposed; fails closed. `valid_name` text matches `catalog_core::wire::valid_name` at `035b44e5` |
| L7 | 1 | 2.6, 2.7 step 4 | Follows from the commit order. See OP-3 |
| L8 | 1 | 2.3, N36 (b) | `Range` after `Stale`. Order justified; checking `Stale` first also prevents overflow |
| L9 | 1 | 2.2 check 6, O15 | Proposed. At most one `Ok` CREATE per binding is preserved |
| L10 | 1 | 2.3 | See RQ-1 |
| L11 | 1 | 2.6 Format | No write was issued, so a blank candidate stays correct |
| L12 | 1 | 2.3, 2.5 | VERIFY never `Corrupt` on a read failure; reads never start the safe path |
| L13 | 1 | 2.3 LIST | Matches `service.rs:421-439` |
| L14 | 1 | 2.7 | Justified by `volume.rs:970` and `prepare` setting `deadline = now + timeout` (virtio-core, `035b44e5`) |
| L15 | 1 | section 7 | Matches `service.rs:236-240` |
| L16 | 1 | 2.3 INFO | Matches `service.rs:401` |
| L17 | 1 | 2.3, 2.7 step 3 | `archive.rs:366-377` refuses trailing bytes (verified at `035b44e5`) |
| L18 | 1 | 2.7 broker contract | Properties 4–5 match `volume.rs:1075-1099` (-16/-13 on timeout; -13 quarantined, -14 offline) |
| L19 | 1 | section 6 preamble | See OP-6 |
| L20 | 1 | 2.3, R2, R5 | `volume-core` is `no_std` and depends only on `catalog-core` (verified) |
| L21 | 1 | N29, R5 | The hook is still `#[doc(hidden)] pub` at `service.rs:148-149`; recorded as a required code change |
| L22 | 1 | 2.2, 2.6, 2.7, N35, O16 | Proposed. Both options write nothing. See OP-5 |
| L23 | 1 | 2.6 | Not a weakening: revision 2 had no such rule; the FORMAT scan and the blank test are unchanged |
| L24 | 1 | 2.6 list | Both implementations apply 80..=525 328 (`catalog.rs:178-179`, oracle line 66). 525 328 = 1040 + 8 × 65 536 (verified) |
| L25 | 1 | 2.6 | FNV-1a with standard basis and prime in catalog-core, R2 and `volume_disk.py` |
| L26 | 1 | section 7 | Matches oracle `analyse` |
| L27 | 1 | 2.6, section 7 | Oracle `--first`/`--sectors` |
| L28 | 1 | section 7 | Oracle `mount` treats a short or unreadable area as failed |
| L29 | 1 | R3, R5, N34 | See OP-7 |

The checked arithmetic (outside the ledger) is correct and present in R2 (`service.rs:571-577`). N29 needs the change in RQ-2.

## Security-floor checks

- **Check order unchanged.** Check 2 still checks only label length. Label content is an operation refusal after check 7, as in R2.
- **Mount still fails closed.** A read error is a failed mount; L28 extends this to the oracle.
- **Safe-path latch.** The proposed O16 is stricter than revision 2; the alternative also writes nothing.
- **Capabilities.** No capability is widened.
- **Disk bytes.** No new trust is placed in disk bytes: the new rules only add refusals.
- **Identifiers.** The 8.1 diff shows only the additions O14–O16. N1–N32 are kept (N13, N29 and N30 are edited in place). N33–N36 are new. R1–R8 and K1–K5 are not renumbered.

## Are normative choices independent of the code?

- **Independent of the code:** L1, L2, L8 and L22 deliberately depart from R2. The other choices are argued from design or from failing closed.
- **Owner decisions:** they are marked *proposed* (O14, O15, O16), and they gate R7 and R8.

## Claims about the code at `37b7bd0c`

Every claim below was checked against the code.

| Claim | Result | Evidence |
|---|---|---|
| R2 follows the O15 proposal | true | `service.rs:441-466, 789` |
| R2 implements the O16 alternative | true | `service.rs:349-353, 634` |
| Neither implementation checks label content at mount | true | partly inaccurate parenthetical, see OP-4 |
| N14 test covers only absent-at-mount | true | `ops.rs:117-127`; see OP-9 |
| Hosted CI skips the cross-check | true | `skipUnless(DEST_ORACLE_OUT)`; no workflow references it |
| 7 cross-check images | true | 7 `emit` calls; the outcome was not re-run |
| Reverse differential absent | true | none found |

## Commands run (read-only)

- `git log`, `git status`, `git diff 5d97cd76 -- docs/implementation/S09-DESTINATION-SERVICE.md`, `git show 5d97cd76:…` and `git show 035b44e5:…` (`archive.rs`, `volume.rs`, virtio-core `lib.rs`, `catalog wire.rs`, `volume Cargo.toml`/`lib.rs`); exit 0.
- `grep` and `sed` over the spec, `services/destination/{src,tests}`, `tools/destination_disk.py`, `tools/volume_disk.py` and `tests/test_volume_destination_disk.py`; exit 0 (one grep exit 1, no match).
- An in-memory oracle probe, which wrote nothing: `python3 -B -I -c` with `encode_copy`/`check_copy` on crafted copies:

| Crafted copy | Oracle result |
|---|---|
| entry ID 0 | accepted |
| label length 0 | accepted |
| label length 33 | refused ("label padding") |
| sealing sequence 1 | accepted |
| copy sequence 0 | refused ("sequence") |

## Not checked

- **Tests not run.** `cargo test`, `clippy` and the Python suites were not run, because they write build output.
- **Unverified status claims.** The result "7 images, agreement", the partial status of N7, N10, N11 and N12, and N24 (b) "recorded" were not verified.
- **Inputs not read.** `rev3-inputs-r2.md` and `rev3-inputs-r3.md` were not read; only the deduplicated ledger was.
- **No graph query.** Hosted Graphify was not queried: it reflects `main`, not these worktrees.
- **No native or QEMU evidence.** None exists for revision 3.
- **No human audit.**

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_c619eee0-6d3` (review 4). An AI review, not a human audit.
