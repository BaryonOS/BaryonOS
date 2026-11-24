# S09-DEST-REV3-CODE-REVIEW-1: independent review of the revision-3 code delta

**Scope reviewed:** the uncommitted change in `/home/simon/projects/BaryonOS-worktrees/s09-destination-core` on top of HEAD `36ee6dc5`. It touches 14 tracked files (+1115/−277) and adds no untracked files. I judged it against spec revision 3: sections 2.3, 2.5 "Device results", the 2.6 validity list, 2.7 step 4 and the K5 contract, the section 6 preamble with rows N13/N24/N26/N29/N33/N34/N36, and the section 9 L1–L29 and RQ/OP tables. I also cross-read `rev3-author.md` (b) and `rev3-fix.md` D1–D9.

**Verdict: APPROVE, with 0 required findings.** Every in-scope item is implemented as the spec states it. No out-of-scope change was found. Every new rule I mutated made at least one test fail. All gates exit 0.

This is an AI review, not a human audit.

## In-scope items

1. **L1/L2, Device results (2.5): done.**
   - In `device.rs:56`, `poll_error(code, mutating)` maps completions by request kind:
     - Read: −13 gives `Timeout`, every other code gives `Io`.
     - Write or flush: −12 gives `Io`; −13, −16, −14 and undefined codes give `Unknown`.
   - `device.rs:118`: every submission result other than 0 or −8 gives `Io`.
   - Poll −8 past the bound gives `Unknown` (mutating) or `Timeout` (read). Submission −8 past its bound gives `Io`.
   - −14 is never `NoDevice` from a device result. `device::Watch` records any −14, at submission or completion, for the mount, remount and FORMAT write/flush paths, so INFO still reports `NoDevice` in the failed-mount state.
   - FORMAT now returns the write-or-flush status itself.
   - `kernel_offline_on_write` now expects `Io`.
   - N33 tests cover reads (READ, VERIFY, FORMAT scan), writes and flushes (APPEND write, SEAL tail write, barrier 1), and −14 reaching the failed-mount state at mount, remount and FORMAT.
2. **L7 and OP-3: done.**
   - `catalog.rs:180-182` enforces sealing sequence ≥ 2, ≤ the copy's sequence, and strictly ascending.
   - The pre-write clause `copy.records[committed.count].sealed_sequence == sequence` is only in the SEAL step-4 filter (`service.rs:617`), not in the mount rule.
   - N34 is in `catalog.rs::n34_record_and_copy_validity`. It includes the zero-record copy with sequence 0, beside a valid copy, both copies crafted, and the case with sectors 8–15 non-zero.
   - N29 has the crafted 2/5/9 → 7 fixture that the validity rules accept, plus the u64-maximum case (repeated SEAL on a new binding, every SEAL `Corrupt`, no catalog write, INFO and LIST unchanged).
3. **Oracle: done.**
   - `tools/destination_disk.py:62,66,72,105,110` adds entry ID ≥ 1, label length ≥ 1 and the three sealing-sequence rules.
   - The encoder default is now `index + 2`.
   - SpecGapPins was split: entry ID 0 and label length 0 are now refused; label content (O14) stays pinned as unchecked.
   - The new oracle code reads as written from the spec, with its own rule names and its own structure. It is not a transliteration of the Rust code.
4. **L8 and N36: done.**
   - `service.rs:514` applies the order `Stale` → `Range` → `Full`.
   - N36 (b) runs on scopes of 1042 and 1043 sectors. It checks that nothing is appended, that `Stale` still comes first, and that the entry stays writing (it completes to 525 328 bytes, or ABORT succeeds).
   - (a1), (a2) and (c) pass, under the proposed O15 rule, as the task notes.
5. **L21: done.**
   - The `tamper` field and its use sit behind `#[cfg(feature = "test-hooks")]` (`service.rs:149,172,608`). Only the crate's self dev-dependency (`Cargo.toml:17`) enables the feature.
   - The workspace uses resolver 2. I verified that the `x86_64-unknown-none` check rustc line has no `--cfg feature=` at all, while the test build has `feature="test-hooks"`. So the product/no_std build has no tamper field.
   - No workspace crate or workflow enables the feature, and no workflow uses `--all-features`.
   - **Cargo.lock:** one added line (`"destination-core"` in its own dependency list). This is unavoidable for a self dev-dependency. The alternative, `required-features` on the test targets, would make the gate command `cargo test -p destination-core` silently skip the hook tests. It passes `--locked`.
6. **L19, two reset models: done.**
   - `SimDisk` has `reset_drops_cache` (set per thread through `both_reset_models`) and a `dropped` counter for non-vacuity. The cache is dropped at a timeout reset and at the device's own −8 reset window.
   - N13, N24 (a/b/c/d), N26 (a–f) and the N33 read-timeout case run under both models. Where the reset matters, the tests assert that the model was actually exercised.
   - The N33 case seeds the free sectors with different stale bytes (the archive with byte 100 flipped), so a stale same-length archive cannot pass the readback by accident.

**Out of scope, confirmed unchanged:**
- O14 label content at mount (not in `decode`, oracle pin kept).
- O15 alternative.
- O16: `catalog_failure` and the blank+latch FORMAT path are untouched, and there is no N35.
- The real `archive::verify`, the reverse differential, and the hosted cross-check.
- Nothing outside the write set was touched.

**Security floor:**
- The order of checks is unchanged.
- Mount is still fail-closed: a failed read or FLUSH gives the failed-mount state.
- No new trust is placed in disk bytes. The new rules only tighten validation, and the only arithmetic on them is comparisons.
- `#![no_std]` and the crate's prohibition on unsafe code are intact.
- The tamper hook is absent from the non-test build.

## Findings (all optional; none required)

| id | req/opt | file:line | problem | fix |
|---|---|---|---|---|
| F1 | optional, **escalate to the owner and spec** | services/destination/tests/broker.rs:531 (`n33_read_timeout_over_stale_valid_archive_recorded`) | Under the cache-dropping reset model, SEAL returns `Ok` for an entry whose bytes are a stale, *valid* archive left by a discarded write, not the archive the portal sent. Only the SEAL reply `aux` differs. The row's invariant ("never a listed entry that fails VERIFY") holds, so this is not a defect of this diff. But a sealed entry can hold another backup's content, and restore-time `archive::verify` would accept it. The test correctly records this rather than passing it as acceptance. | Spec decision, outside this write set. Options: the service keeps a running FNV over the APPENDed bytes and compares it with the step-3 readback, or the spec requires the portal to compare SEAL `aux` with its own trailer FNV before it reports the export as sealed. |
| F2 | optional | services/destination/tests/broker.rs:27,45,72,142,202 | N26 (a), (b), (c), (e) and (f) run under `both_reset_models`, but the stub never reaches a reset that drops cached data (only (d) does: my mutant H1 failed n26d alone among the N26 tests). So the second run repeats the first. n24c says this in a comment; the N26 cases do not. | Add the same one-line note, or assert `h.disk.dropped == 0` to make it explicit. |
| F3 | optional | tests/test_volume_destination_disk.py:214 | The oracle's "label length 0" crafted copy leaves the label bytes non-zero, so it also breaks the label-padding rule. It is therefore not "valid except for one rule" (N34). With the oracle's label-length rule removed, the mount-state test still passes; only the rule-name assertion catches it. The Rust fixture (`oracle_images.rs`) zeroes the label bytes correctly. | Zero `rec(1, 48):rec(1, 80)` in that case. |
| F4 | optional | services/destination/tests/broker.rs:505 | The spec's N33 separate case says the APPEND sectors "were zero before". The test uses non-zero stale bytes, which is stronger and what the task asked for, but the literal zero variant is not run. | Optionally add a zero-sector variant. |
| F5 | optional (spec note) | services/destination/tests/broker.rs:107 (n26d) | Under the dropping model, the device's own reset during the barrier-1 submission loses the cached tail write, and SEAL replies `Corrupt` with nothing listed. The row's stated (d) invariant (both requests complete, no `Io`, no safe path) holds, and this is fail-closed. Row N26 (d) states no per-model outcome. | The lead may record the per-model outcome in N26 (d), as OP-6 did for N33. |
| F6 | optional | services/destination/tests/oracle_images.rs (N34 emit block) | The Rust → oracle cross-check emits 5 of the 9 N34 crafted cases. It omits lengths 79 and 525 329, label length 33, and the copy with sequence 0. Both implementations still test all 9 cases separately. | Optionally emit the other four; the reverse differential (L29) remains out of scope. |
| F7 | optional | services/destination/Cargo.toml:17 | Clippy and test always compile the library with `test-hooks`. The build without the feature is only `check`ed (gate 4), with no warnings in `g4.log`, and is never linted. | Optionally lint the library without the feature for the no_std target. |

## Mutation evidence (scratch copy; the worktree was never edited)

I ran the mutants in a copy of the worktree, `/tmp/claude-1000/-home-simon-projects-BaryonOS/171c7b49-ee33-4342-8270-24a1e42ae3e2/scratchpad/rev3-review/mut/wt`, using the script `/tmp/claude-1000/-home-simon-projects-BaryonOS/171c7b49-ee33-4342-8270-24a1e42ae3e2/scratchpad/rev3-review/mutate.py`. The script restores each file and checks it byte-identically by SHA-256. Every mutant made at least one test fail:
- **Catalog validity (C1–C5):** dropping sealing ≥ 2, sealing ≤ copy sequence, sealing ascending, label length ≥ 1, or copy sequence ≥ 1 fails `n34_record_and_copy_validity`, plus `validator_rejects_*` where they apply.
- **SEAL step 4 (S1):** dropping the sealing-sequence clause fails `n29_prewrite_sealing_sequence_accepted_by_validity_rules`.
- **APPEND order (S2–S4):** `Range`→`Full`, the scope check moved before `Range`, or `Range` checked before `Stale` each fail `n36_append_past_largest_archive`.
- **FORMAT (S5, F1):** losing `no_device` on a FORMAT failure, or always replying `Io`, fails `n33_no_device_reaches_failed_mount_state`; F1 also fails `n25_format_write_failure`.
- **Device mapping (D1–D7):** a write completing −13 → `Timeout`; completion −14 → `NoDevice`; submission −14 → `NoDevice` (also fails `kernel_offline_on_write`); write −16/−14 → `Io`; `Watch` ignoring the poll or the submit; a read failure → `Timeout`. Each fails N33 tests.
- **Reset model (H1):** a `SimDisk` that never drops the cache fails the N13, N24 a/b/d, n26d and both N33 reset tests.
- **SEAL readback (S7):** a SEAL that ignores a failed readback fails `n33_read_timeout_with_unflushed_append`, n10 and n26d.
- **CREATE and read failures (K1–K3):** the CREATE label check after `Busy`, VERIFY answering `Corrupt` on a device error, and a read failure starting the safe path each fail N36 or N33 tests.
- **Oracle (P1–P6):** dropping entry ≥ 1, label ≥ 1, sealing ≥ 2, sealing ≤ copy or sealing ascending, or reverting the encoder default to 1, each fail `RecordAndCopyValidity` tests.

Mutants I did not run:
- a refused CREATE that uses up the binding;
- a sectors 8–15 validity rule;
- removing the `test-hooks` cfg (covered by the rustc cfg inspection instead).

## Gates (run by me; CARGO_TARGET_DIR=target-dc)

| gate | exit |
|---|---|
| `cargo +1.100.0 test --locked -p destination-core` | 0 (68 tests: 12/14/13/11/5/12/1) |
| `cargo +1.100.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 |
| `cargo +1.100.0 fmt --all -- --check` | 0 |
| `cargo +1.100.0 check --locked -p destination-core --target x86_64-unknown-none` | 0 (rustc line has no feature cfg) |
| `cargo +1.90.0 test --locked -p destination-core` | 0 |
| `cargo +1.90.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 |
| `python3 -B -m unittest discover -s tests -p 'test_volume_destination_disk.py'` | 0 (43 tests, 1 skipped) |
| `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 0 (109 tests, 1 skipped) |
| `python3 -m compileall -q tools/destination_disk.py` | 0 |
| `DEST_ORACLE_OUT=<fresh scratch dir> cargo +1.100.0 test --locked -p destination-core --test oracle_images` | 0 |
| `DEST_ORACLE_OUT=<same> python3 -B -m unittest tests.test_volume_destination_disk.RustCrossCheck -v` | 0 (14 images, 6 of them N34, all agree) |

The fresh oracle directory was `/tmp/claude-1000/-home-simon-projects-BaryonOS/171c7b49-ee33-4342-8270-24a1e42ae3e2/scratchpad/rev3-review/oracle.Reem`. Logs are under `/tmp/claude-1000/-home-simon-projects-BaryonOS/171c7b49-ee33-4342-8270-24a1e42ae3e2/scratchpad/rev3-review/` (g1–g11.log, codes, mutants.log).

## Not checked / limits

- **Worktree integrity:** the content is unchanged; the diff stat is the same before and after (14 files, +1115/−277, no untracked files). I did run `touch` on `services/destination/src/lib.rs` three times to force a rebuild for the cfg inspection. That changed only the file's modification time.
- **Hosted Graphify:** `recall` was not used; I read the code directly.
- **Old code paths:** whitespace-insensitive diffs of the test files show no assertion removed or loosened, except two that the spec's revision-3 rules require: the −14 → `NoDevice` expectation, and n26d's single `Ok` expectation, which is now per model. n24d now injects the fault into the second entry write so that the dropping model has something to lose; the row's assertions are kept.
- **Not run (out of scope):** the real `volume_core::archive::verify`, native/QEMU stages, hosted CI, and the reverse differential.
- **Unverified:** that `SimDisk` meets K5 property 3 ("answered exactly once") in every path; I checked it by reading only, not with a test.

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_26df05da-a0f` (review 1 of the revision-3 code delta, on top of `36ee6dc5`). An AI review, not a human audit.
