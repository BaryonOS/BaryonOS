# Review S09-DEST-O14-O16-REVIEW-1: O14/O16 change on top of 282e74c0

This is an AI review, not a human audit. I reviewed the uncommitted `git diff HEAD` in `/home/simon/projects/BaryonOS-worktrees/s09-destination-core` against spec 2.2 (check-7 table), 2.3, 2.6, 2.7, section 6 (N34, N35) and 8.1/9. I did not edit the worktree. All mutants ran in a scratch copy, which I restored and diffed against the worktree each time.

## Verdict: REQUEST_CHANGES (1 required)

The implementation matches the decided rules, and every gate passes. One new O16 rule is not fully pinned by a test: a remount that reads the committed copy with changed header fields. The project rule says every new rule needs a test that fails without it, so this one is required. The fix is cheap and test-only.

## In-scope items

### A. O14: label content at mount
- **Rust:** `catalog::decode` now calls the CREATE-side `crate::valid_name` (`services/destination/src/catalog.rs:193`). One function serves both paths. The call sits after the length and padding clauses, so the slice is always within bounds.
- **Match with 2.3:** `valid_name` (`lib.rs:65-80`) checks valid UTF-8 of 1–32 bytes, not `.` or `..`, no `char::is_control` (that is Cc), no `/` or `\`, and no U+202A–U+202E or U+2066–U+2069. This matches 2.3. I did not compare it against `catalog_core::wire::valid_name` on main.
- **Oracle:** `valid_label` in `tools/destination_disk.py` is an independent Python implementation (`unicodedata` Cc, strict UTF-8, its own `BIDI` set). It does not reference the Rust code.
- **Test changes:**
  - The `SpecGapPins` pin is removed.
  - `RecordAndCopyValidity` gains 11 content cases, each with reason "label content", plus a positive test for valid names.
  - `oracle_images.rs` emits 11 `n34_label_*` images.
  - I checked the oracle's refusal reason for every `n34_*` image: each one names the intended rule.

### B. O16: remount after the safe-path trigger
- `catalog_failure` (`service.rs:642-677`) now takes the committed `Catalog` and decides as follows:
  - **Equal to the committed copy:** resolves *not sealed*.
  - **Passes `extends_by_one` and the new record carries the provisional ID:** resolves *sealed*.
  - **Any other mounted catalog, or a blank result:** enters `Failed { no_device: false }` with the export still *unknown*.
  - **`Corrupt` or `Failed`:** left as they are.
- **Comparison strength:** the comparison uses full `Catalog` equality. `decode` refuses any byte outside the fields, and the checksum is fixed by them, so equal catalogs mean byte-identical copies.
- **Shared check:** `extends_by_one` (`catalog.rs:215-226`) moves the old pre-write filter into one function without changing its meaning. Checked add on the sequence; `count + 1` is bounded at 16; the `count` comparison runs before the record index.
- **Probe:** I ran crafted remount reads in scratch:
  - Committed copy with another destination ID: `Failed`, binding `Unknown`.
  - Committed copy at sequence + 1: `Failed`, binding `Unknown`.
- **Old guard:** `mount(` is called only at boot (`service.rs:177`) and in `catalog_failure` (`:654`), and the latch is set only in the Mounted state (`:495`, `:652`). The Blank + safe-path FORMAT `Access` guard is therefore unreachable. It stays as a defence with a comment saying so.
- **Old test:** `o7_safe_path_refuses_format_after_blank_remount` is replaced by `n35_remount_contradicting_committed_catalog`.
- **N35 coverage:** (a) all zero; (b) another ID listing the provisional ID; (c) E1 changed, new record under another ID, two new records, copy sequence + 2; (d) foreign bytes, which give `Corrupt`. The test also binds a verify lane and checks INFO, export-lane LIST (`Unknown`, zero records), VERIFY and FORMAT (`Io`), or `Corrupt` everywhere for (d). It checks that no `Write` is submitted after the failed one, that no mutation follows, and that the binding is never resolved as sealed. This matches spec row N35.

### C. Review follow-ups
- **F2:** N26 (a)(b)(c)(e)(f) now assert `dropped == 0`. The counter is live: (d) asserts `dropped > 0`.
- **F3:** the Python label-length-0 copy now zeroes the label bytes, and the oracle reason is "label length". The Rust image already zeroed them.
- **F6:** `oracle_images.rs` now emits the four remaining images (`zero_records_sequence_0`, `length_79`, `length_525329`, `label_length_33`). Each breaks one rule; `crafted()` asserts the base copy is valid and the crafted one invalid.

### Out of scope and security floor
- **Write set:** exactly 8 files, all inside the permitted set. Cargo.lock and the spec are untouched.
- **Out-of-scope items:** SEAL semantics unchanged apart from the shared function, so F1 is untouched. F4, F5 and F7 are untouched.
- **Code rules:** `#![forbid(unsafe_code)]` is kept, the no_std target check passes, and no new trust in disk bytes is introduced.
- **Failed state after O16:** `info()` emits catalog fields only in the Mounted state, so a contradicting catalog left in `selection` never reaches a reply.

## Findings

### R-1 (required): the committed-copy arm of O16 has no negative test
- **Where:** `services/destination/tests/crash.rs`, around line 756 (`cases` in `n35_remount_contradicting_committed_catalog`); the rule is at `services/destination/src/service.rs:657`.
- **Problem:** every N35 (b)/(c) case is derived from `sealed`, so the `remounted == *committed` arm is never given a wrong catalog. In scratch, mutants that weaken that equality all pass the full suite:
  - R8: compare `entries()` only.
  - R15: ignore the destination ID.
  - R16: ignore the sequence.

  The real code is correct, but a regression would let a remount that reads the committed records under another destination ID, or at another sequence, resolve as *not sealed* and stay Mounted. Under O16 that remount must be the failed-mount state.
- **Fix:** add two (c)-style cases built from `committed`:
  - `committed` with `id[0] ^= 1`.
  - `committed` with `sequence += 1`, which is valid because sealing sequences stay at or below the copy's sequence.

  Both must give `Failed`, binding `Unknown` and no write. My probe confirms the current code does that, so the cases pass now and kill R8, R15 and R16.

### O-1 (optional): bidi range bounds are not tested in either implementation
- **Where:** `services/destination/tests/catalog.rs` (labels list near line 579), `services/destination/tests/oracle_images.rs` (labels list near line 235), `tests/test_volume_destination_disk.py` (`LABELS`).
- **Problem:** only U+202E and U+2066 are tested. These mutants survive:
  - Rust: lower bound U+202B (R5b) and upper bound U+2068 (R5c).
  - Python: `range(0x202B, …)` (P5) and `range(0x2066, 0x2069)` (P5b).

  A bound disagreement between the core and the oracle would also pass the cross-check.
- **Fix:** add U+202A and U+2069 as invalid labels to all three lists. U+2029 and U+206A could be added as valid ones.

### O-2 (optional, owner/spec note, no code defect): the *sealed* arm does not pin the new record's content
- **Where:** `services/destination/src/service.rs:658-661`.
- **Observation:** the probe shows that a remount catalog which passes `extends_by_one` and carries the provisional ID resolves *sealed* even when the new record's trailer, label or `next_entry` differ from what the service encoded. This follows the literal 2.7 definition (the extension check plus the provisional entry ID), so it conforms.
- **Possible tightening for the owner:** require byte equality with the encoded copy `checked`, which the service still holds at this point. That would be a spec change, not something for this PR.

### Mutation evidence (scratch copy; each mutant restored, with sha256 checked)
- **Killed, label content:** R1 (label content clause removed from decode), R2 (control), R3 (`/`), R4 (`\`), R5 (bidi), R6 (`.`), R7 (`..`), all by `n34_record_and_copy_validity`.
- **Killed, O16:**
  - R9 (provisional-ID clause dropped), R10 (a remount that ends blank stays blank) and R11 (any other catalog resolves *not sealed*), by N35.
  - R12 (sequence clause dropped), by N35.
  - R13 (sealing-sequence clause dropped), by `n29_prewrite_sealing_sequence_accepted_by_validity_rules`.
- **Killed, oracle:** P1 (no label content check), P2 (no Cc), P3 (no `/`), P3b (no `\`), P4 (no bidi), P6 (no `.`), P6b (no `..`), P7 (lenient UTF-8).
- **Survived:**
  - R8, R15, R16: see R-1.
  - R5b, R5c, P5, P5b: see O-1.
  - R14 (count clause dropped): a confirmed equivalent mutant, as the test comment states. Both callers pass `decode` output, where sealing sequences are strictly ascending and at most the copy's sequence, so a second new record cannot be valid.

## Gate table (worktree, `CARGO_TARGET_DIR=…/target-dc`)

| Command | Exit | Result |
|---|---|---|
| `cargo +1.100.0 test --locked -p destination-core` | 0 | all suites ok (12/14/13/11/5/12/1) |
| `cargo +1.100.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 | clean (cached build) |
| `cargo +1.100.0 fmt --all -- --check` | 0 | clean |
| `cargo +1.100.0 check --locked -p destination-core --target x86_64-unknown-none` | 0 | ok |
| `cargo +1.90.0 test --locked -p destination-core` | 0 | all ok |
| `cargo +1.90.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 | clean |
| `cargo +1.90.0 fmt --all -- --check` | 0 | clean |
| `python3 -B -m unittest discover -s tests -p 'test_volume_destination_disk.py'` | 0 | 43 tests, 1 skipped (cross-check without env) |
| `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 0 | 109 tests, 1 skipped |
| `python3 -m compileall -q tools/destination_disk.py` | 0 | ok |
| `DEST_ORACLE_OUT=…/scratchpad/oracle-usQZ cargo +1.100.0 test --locked -p destination-core --test oracle_images` | 0 | 28 images emitted |
| `DEST_ORACLE_OUT=…/scratchpad/oracle-usQZ python3 -B -m unittest tests.test_volume_destination_disk.RustCrossCheck -v` | 0 | `test_rust_images` ok (28 images agree) |

## Not checked
- `catalog_core::wire::valid_name` on main was not compared with the crate's `valid_name`.
- The reverse differential (oracle-crafted images mounted by the core), hosted CI and native/QEMU gates were not run; all are out of scope.
- The cross-check compares mount state only, not refusal reasons. I compared reasons manually for the `n34_*` images.
- Clippy results came from the cached build. I did not touch files to force a rebuild.
- Side effect: one ad-hoc Python run refreshed the gitignored `tools/__pycache__/volume_disk.cpython-314.pyc` in the worktree. No tracked file changed (`git status --short` shows the same 8 modified files).

Scratch paths:
- Mutation copy and runner: `/tmp/claude-1000/-home-simon-projects-BaryonOS/171c7b49-ee33-4342-8270-24a1e42ae3e2/scratchpad/rev-t6RO` (`mutate.py`).
- Oracle images: `/tmp/claude-1000/-home-simon-projects-BaryonOS/171c7b49-ee33-4342-8270-24a1e42ae3e2/scratchpad/oracle-usQZ`.

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_dbc68a07-430` (review 1 of the O14/O16 code change, on top of `282e74c0`; REQUEST_CHANGES, R-1 fixed and O-1 applied in the fix pass, O-2 left to the owner). An AI review, not a human audit.
