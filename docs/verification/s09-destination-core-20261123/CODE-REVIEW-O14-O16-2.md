# Re-review S09-DEST-O14-O16-REVIEW-2: O14/O16 change on top of 282e74c0 (after the fix pass)

This is an AI review, not a human audit. I re-reviewed the uncommitted `git diff HEAD` in `/home/simon/projects/BaryonOS-worktrees/s09-destination-core` (HEAD `282e74c04b60d3c399232a2ce16717a3422cba45`) against spec 2.2 (check-7 table), 2.3, 2.6, 2.7, section 6 (N34, N35) and 8.1/9. I did not edit the worktree. All mutants ran in a scratch copy and were restored there, with a sha256 check per mutant. I then diffed the scratch copy against the worktree and it matched.

## Verdict: APPROVE (0 required)

- R-1 is resolved.
- O-1 is resolved, with both lower and upper bounds pinned on both sides of each bidi range in both implementations.
- The fix pass broke nothing. Every gate passes, and every new rule has a test that fails when the rule is removed or weakened.

## Status of the previous findings

### R-1 (required): RESOLVED
- **What changed:** `services/destination/tests/crash.rs` `n35_remount_contradicting_committed_catalog` now has two more cases built from `committed`:
  - `committed_other_id` (`id[0] ^= 1`), labelled "(b) committed records, other destination ID".
  - `committed_later` (`sequence += 1`), labelled "(c) committed records at copy sequence + 1".
- **Expected result:** both must give `Failed { no_device: false }`, binding `Unknown`, and no device write. The doc comment's Kills list now names this.
- **Mutation evidence:** R8 (`entries()` only), R15 (ignore the ID) and R16 (ignore the sequence) are now all KILLED by `n35_remount_contradicting_committed_catalog`.

### O-1 (optional): RESOLVED
- **Invalid labels added:** U+202A and U+2069 are in all three lists:
  - `services/destination/tests/catalog.rs` (13 labels)
  - `services/destination/tests/oracle_images.rs` (13 `n34_label_*` images)
  - `tests/test_volume_destination_disk.py` (`LABELS`)
- **Valid boundary labels added:** U+2029, U+202F, U+2065 and U+206A are now asserted as valid in Rust (`catalog.rs`) and in Python (`test_label_content_accepts_valid_names`).
- **Mutation evidence:**
  - Narrowed bounds are KILLED: R5b and R5c in Rust, P5 and P5b in Python.
  - New widening mutants are also KILLED: X1 (Rust lower bound U+2029), X2 (Rust upper bound U+206A), X3 and X4 (Python widening).
- **Effect:** a bound disagreement between the core and the oracle can no longer pass unnoticed.

### O-2 (optional, owner/spec note): UNCHANGED, still not a code defect
The *sealed* arm (`services/destination/src/service.rs:658-661`) still follows the literal 2.7 definition: `extends_by_one` plus the provisional entry ID. Requiring byte equality with the encoded `checked` copy would be a spec change for the owner. It is not needed for this PR.

## In-scope items, rechecked

### A. O14: label content at mount
- **One function, both paths:** `catalog::decode` calls `crate::valid_name` (`catalog.rs:193`), after the length and padding clauses, so the slice is always in bounds. CREATE uses the same function.
- **Match with 2.3:** `valid_name` (`lib.rs:65-80`) is unchanged and matches 2.3.
- **Oracle independence:** `valid_label` in `tools/destination_disk.py` is independent of Rust. It uses strict UTF-8, which also rejects surrogates, plus `unicodedata` Cc, `/` and `\`, and its own `BIDI` set.
- **Oracle reasons checked by hand** for all 23 `n34_*` images from this run:
  - Each `n34_label_*` content case gives "label content".
  - label length 0 gives "label length"; label length 33 gives "label padding".
  - length 79 and 525 329 give "entry length"; zero records gives "sequence".
  - `reserved_sectors_nonzero` gives None, which is the valid case, as intended.

### B. O16: remount after the safe-path trigger
- **Logic:** unchanged since review 1 and correct.
  - Exact `Catalog` equality resolves *not sealed*.
  - `extends_by_one` plus the provisional ID resolves *sealed*.
  - Any other mounted catalog, or a blank result, gives `Failed { no_device: false }` with the binding `Unknown`.
  - `Corrupt` and `Failed` are left as they are.
- **Comparison strength:** "that copy plus exactly the provisional record" cannot be met by a different catalog. The ten N35 cases plus mutants R8–R16 and X5 show this. X5 (the sealed arm using `extends` instead of `extends_by_one`) is KILLED.
- **Old guard:** the Blank + safe-path FORMAT `Access` guard is kept as a defence, with a comment saying it is unreachable since O16.

### C. Follow-ups
- **F2:** `assert_eq!(h.disk.dropped, 0)` is in N26 (a), (b), (c), (e) and (f).
- **F3:** the label-length-0 copy has its label bytes zeroed (`label_length_0`), and the oracle reason is "label length".
- **F6:** all four remaining images are emitted. `crafted()` asserts that the base copy is valid and the crafted one is invalid.

### Out of scope and security floor
- **Write set:** exactly 8 modified files, all inside the permitted set. `Cargo.lock` and the spec are untouched.
- **SEAL semantics:** unchanged apart from the shared `extends_by_one`. F1, F4, F5 and F7 are untouched.
- **Code rules:** no `unsafe`; `no_std` target check passes; no new trust in disk bytes; checked add on the sequence in `extends_by_one`.

## Findings

### N-1 (optional, nit): case label for the committed-ID case
- **Where:** `services/destination/tests/crash.rs`, `cases` in `n35_remount_contradicting_committed_catalog`.
- **Problem:** "(b) committed records, other destination ID" does not match the literal (b) of N35, which requires the catalog to list the provisional entry ID. That case is closer to (c)-style or a plain "other ID" case.
- **Also:** the doc comment wraps unevenly ("…another ID); (c) a valid catalog / with the same ID…").
- **Fix:** relabel it and rewrap the comment. Cosmetic only; behaviour and assertions are correct.

## Mutation evidence (scratch copy `/tmp/claude-1000/-home-simon-projects-BaryonOS/171c7b49-ee33-4342-8270-24a1e42ae3e2/scratchpad/rev2-yFZ8`; runner `mutate.py`; each mutant restored with a sha256 check; final `diff -r` against the worktree is identical)
- **KILLED, Rust:**
  - Label content: R1, R2, R3, R4, R5, R5b, R5c, R6, R7, X1, X2, all by `n34_record_and_copy_validity`.
  - O16: R8, R9, R10, R11, R12, R15, R16, X5, all by `n35_remount_contradicting_committed_catalog`.
  - R13, by `n29_prewrite_sealing_sequence_accepted_by_validity_rules`.
- **KILLED, Python oracle:** P1, P2, P3, P3b, P4, P5, P5b, P6, P6b, P7, X3, X4.
- **SURVIVED (equivalent mutants):**
  - R14 (count clause dropped): equivalent, as documented in the test comment.
  - X6 (length guard removed inside `valid_label`): equivalent, because `decode_record` rejects lengths outside 1–32 before it calls `valid_label`.

## Gate table (worktree, `CARGO_TARGET_DIR=/home/simon/projects/BaryonOS-worktrees/s09-destination-core/target-dc`)

| Command | Exit | Result |
|---|---|---|
| `cargo +1.100.0 test --locked -p destination-core` | 0 | all suites ok (12/14/13/11/5/12/1) |
| `cargo +1.100.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 | clean (also rerun after a forced rebuild in the scratch copy: exit 0, "Checking destination-core") |
| `cargo +1.100.0 fmt --all -- --check` | 0 | clean |
| `cargo +1.100.0 check --locked -p destination-core --target x86_64-unknown-none` | 0 | ok |
| `cargo +1.90.0 test --locked -p destination-core` | 0 | all ok |
| `cargo +1.90.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 | clean |
| `cargo +1.90.0 fmt --all -- --check` | 0 | clean |
| `python3 -B -m unittest discover -s tests -p 'test_volume_destination_disk.py'` | 0 | 43 tests OK, 1 skipped (cross-check without env) |
| `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 0 | 109 tests OK, 1 skipped |
| `python3 -m compileall -q tools/destination_disk.py` | 0 | ok |
| `DEST_ORACLE_OUT=…/scratchpad/oracle2-Yusf cargo +1.100.0 test --locked -p destination-core --test oracle_images` | 0 | 30 images emitted |
| `DEST_ORACLE_OUT=…/scratchpad/oracle2-Yusf python3 -B -m unittest tests.test_volume_destination_disk.RustCrossCheck -v` | 0 | `test_rust_images` OK (30 images agree) |

## Not checked
- I did not compare `catalog_core::wire::valid_name` on main with the crate's `valid_name`.
- I did not run the reverse differential (oracle-crafted images mounted by the core), hosted CI, or native/QEMU gates; all are out of scope.
- The cross-check compares mount state only. I checked the refusal reasons by hand for the 23 `n34_*` images (listed above).
- The worktree clippy run came from the cached build. To avoid touching the worktree, I confirmed the clippy result with a forced rebuild in the scratch copy instead.

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_dbc68a07-430` (re-review 2 after the fix pass, on top of `282e74c0`; APPROVE; the lead applied the cosmetic nit N-1 (case label and doc-comment wrap in `crash.rs`) by hand afterwards). An AI review, not a human audit.
