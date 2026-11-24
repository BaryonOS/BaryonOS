# Review S09-DEST-REV5-TESTS-2: spec revision 5 test obligations in services/destination

- Date: 23 November 2026
- Base: `2c96869f` (branch `claude/s09-destination-core`, draft PR #413); the uncommitted diff over that base is reviewed (6 modified test files, no untracked files)
- Reviewer statement: this is an independent AI code review. The reviewer did not write the change. It is not a human audit.
- Verdict: **APPROVE** (no required findings; five optional findings)

## Scope check

| Check | Result |
|---|---|
| Worktree untouched | `sha256sum -c` of the 16 files under `services/destination/` plus `MPL-2.0-FILES.txt`, recorded before any action in `rv2-before.sha` (scratchpad): exit 0, all OK |
| `src` touched | No. `git diff HEAD --stat -- services/destination/src` is empty, and the src checksums are unchanged |
| Cargo.lock, Cargo.toml, toolchain | Unchanged |
| Changed files | `tests/{broker,catalog,common/mod,crash,oracle_images,sim_contract}.rs` |
| Mutations | Run only in a scratch copy (rsync of the worktree without `.git` and `target*`, own `CARGO_TARGET_DIR`). After the runs `diff -r` against the worktree is identical |

## RV1-R1 (review 1): resolved

The deadline is now decided by the request's ready time:
`past_deadline = tick >= admitted + TIMEOUT && ready > admitted + TIMEOUT`.
A sparse poll can no longer skip it.

- Enforced answers -16 or -13 at the first poll at or after the deadline, however late that poll comes.
- Tripwire panics at that poll.
- The comments now say that evaluation is lazy and that the reset starts at the consuming poll (`common/mod.rs` module doc, `Deadline` doc, `sim_contract.rs` module doc).
- New tests: `p12_4_deadline_decided_by_ready_time` (with a boundary case where the result is ready exactly at the deadline) and `deadline_tripwire_flags_overdue_sparse_poll`.
- Mutant D4 reinstates the RV1 gap (decided at the poll tick). Both new tests kill it.
- Probe P1 adds an assertion at drop and at every refused submission that no Tripwire disk holds an overdue request. Only the dedicated sparse-poll tripwire test trips it. No other test, `oracle_images` with `DEST_ORACLE_OUT` included, leaves an overdue request unpolled.

### Ruling on remaining phrasings (overclaim check)

- `p12_4_deadline_answers_and_resets` says "answered by the broker at that tick". Accurate for this test, which polls every tick and asserts `at == admission + TIMEOUT`.
- The `sim_contract.rs` module doc says "fails the test on such a request". Accurate in context: the previous sentence says the deadline is evaluated at the poll, and P1 shows no test where the difference shows.
- A grep of `tests/` for "still pending at", "past its deadline" and "at that tick" finds only the first sentence above.
- Precision suggestions are in RV2-O1. Neither sentence is an overclaim.

## Obligations (CODE-DELTA (revision 5) and broker contract 2.7)

| Row | Status | Evidence |
|---|---|---|
| Q4: opt-in deadline mode answers -16 (WRITE, FLUSH) or -13 (READ) by itself, then resets; reuse only after the ack | Met. Shown by `SimDisk`'s own state; only the device's pending time is injected | `Deadline::Enforced`; `p12_4_deadline_answers_and_resets`, `p12_4_deadline_decided_by_ready_time` (both reset models); deadline runs from admission (sub-case with the device's own reset first) |
| Q4: default off; N26 (e), N26 (f) and the "given up on" case of `p12_2` keep their stubs | Met | Default is `Tripwire`. Those three set `Deadline::Unchecked`, and their bodies are otherwise unchanged (one opt-out line each; `p12_2` restores `Tripwire` afterwards). B1 (opt-out removed) fails n26e. T1 (Enforced as the default) fails only the two should_panic tripwire tests, so default-off equals always-on for all other tests |
| Q4: quarantine (-13) and offline (-14) as device state until a simulated reboot | Met. Entered by `ResetEnd` after a reset caused by the model's deadline or by a -16 completion, not by injection | `down`, `ResetEnd`, `SimDisk::rebooted`; `p12_5_quarantine_and_offline_until_reboot` (3 × TIMEOUT ticks of refusals, snapshot unchanged, ack only before ReinitFails, reboot keeps the durable image only) |
| Q4: `p12_4` and `p12_5` extended under both reset models | Met | `both_reset_models` in all new P12 tests; `dropped > 0` non-vacuity checks |
| Q4: kills (deadline that does not answer; admission after quarantine or offline before reboot) | Met | D7; Q1 and Q6 (table below) |
| Q4 optional (OPT9-7): default-mode tripwire with explicit opt-outs | Done | `Deadline::Tripwire`; `deadline_tripwire_flags_overdue_request`, `deadline_tripwire_flags_overdue_sparse_poll`; T1 and T2 |
| Q6: stray-poll counter, asserted zero in every service-driven test | Met. A service-driven test cannot skip it | Counted in `SimDisk::poll`, asserted in `Drop` (not while panicking). `stray_polls_allowed` is set only in P12 tests, which drive no service, and in the Q6 kill test after it asserts a count > 0. No `mem::forget`, `ManuallyDrop` or leak. The other `Broker` wrappers (`Swapped`, `PollTwice`) forward to `SimDisk::poll`. R1 (src: one extra poll after every delivered result) fails 67 tests, each by the drop assertion (`grep -c` gives 67 "stray poll" panics); 68 with `DEST_ORACLE_OUT` set. The tests left passing never receive a delivered 0 (n26a), drive no service (catalog, checks, P12) or are env-gated (`oracle_images` without the variable) |
| Q7: property-3 snapshot before and after each further poll | Met. Covers device state | `further_poll` compares durable, cache, write and flush totals and `broker_state()` (inflight, current, resetting_until, again_until, again_forever, late, down), plus the caller's buffer. Used in `p12_3` (4 further polls, count asserted) and `p12_4`. S3a–S3e killed |
| Rev. 4 optional: U+009F and U+00A0 | Done, Rust core only (stated in the comment; oracle confirmed without them, RV2-O4) | `n34_record_and_copy_validity`; R3 and R4 killed |
| Rev. 4 optional: N33 (ii) zero bytes | Done | `n33_read_timeout_with_unflushed_append` (zero variant × verify, both models) |
| N35 optional M7 sub-case | Done | `n35_sealed_resolution_uses_checked_copy`; R2 (`&next` for `&checked`) killed |
| Rev. 4 P10 (lint of the non-test build in the hosted gate) | Not part of this change (lead-owned workflow) | none |

## Findings

| ID | Severity | Location | Finding |
|---|---|---|---|
| RV2-O1 | optional | `sim_contract.rs` docs | Two sentences leave out "at a poll". Ruled accurate in context (see above); suggested wording only |
| RV2-O2 | optional | `snapshot()` doc | The exclusion list omits `calls` and the configuration fields; `crashed` is excluded as an instrument. S3f (a poll sets `crashed`) is still killed by later assertions |
| RV2-O3 | optional | `oracle_images.rs` | Predates the change: env-gated and vacuous in the cargo gates. With the variable set it passes unmutated and fails under R1 |
| RV2-O4 | optional | oracle suite | U+009F and U+00A0 are not in `tests/test_volume_destination_disk.py`; the comment states this correctly |
| RV2-O5 | optional | spec 2.7, CODE-DELTA rows Q4, Q6, Q7, 8.2 R2 | Status text still reads open or partial; the lead should update it when this lands |

## Mutation evidence

All mutants ran with `cargo +1.90.0 test --locked -p destination-core --no-fail-fast` in the scratch copy. Each run applied one change, which was restored afterwards. Failure lists include should_panic tests. Baseline in the scratch copy: 84 passed, 0 failed. D3, D6 and P1 are multi-line helper edits (thread-local tick bookkeeping or probe assertions), not literal one-token edits.

| Mutant | Result | Killing tests |
|---|---|---|
| D1 deadline codes swapped (-13 for mutating, -16 for read) | killed | p12_4_deadline_answers_and_resets, p12_4_deadline_decided_by_ready_time, p12_5_quarantine_and_offline_until_reboot |
| D2 deadline answers -16 for every kind | killed | p12_4_deadline_answers_and_resets, p12_4_deadline_decided_by_ready_time |
| D3 deadline measured from the first poll instead of admission | killed | p12_4_deadline_answers_and_resets, p12_4_deadline_decided_by_ready_time, deadline_tripwire_flags_overdue_request, deadline_tripwire_flags_overdue_sparse_poll |
| D4 deadline decided at the poll tick (RV1-R1 gap) | killed | p12_4_deadline_decided_by_ready_time, deadline_tripwire_flags_overdue_sparse_poll |
| D5 ready exactly at the deadline counts as overdue | killed | p12_4_deadline_decided_by_ready_time |
| D6 deadline measured from the first submission attempt | killed | p12_4_deadline_answers_and_resets |
| D7 Enforced deadline does not answer | killed | p12_4_deadline_answers_and_resets, p12_5_quarantine_and_offline_until_reboot |
| D8 deadline answer without reset | killed | p12_4_deadline_answers_and_resets, p12_4_deadline_decided_by_ready_time, p12_5_quarantine_and_offline_until_reboot |
| D9 deadline answer lets the write land (model choice) | killed | p12_4_deadline_answers_and_resets, p12_4_deadline_decided_by_ready_time |
| Q1 quarantine or offline cleared before reboot (`down.take()`) | killed | p12_5_quarantine_and_offline_until_reboot, p12_5_quarantined_or_offline_not_admitted |
| Q2 offline returns -13 | killed | p12_5_quarantine_and_offline_until_reboot |
| Q3 injected -13 or -14 not held | killed | p12_5_quarantined_or_offline_not_admitted |
| Q4 `reset_end` ignored (always ready) | killed | p12_5_quarantine_and_offline_until_reboot |
| Q5 a down device still starts its own reset | killed | p12_5_quarantine_and_offline_until_reboot, p12_5_quarantined_or_offline_not_admitted |
| Q6 down state ends by itself (tick 1000) | killed | p12_5_quarantine_and_offline_until_reboot |
| S1 stray-poll counter never incremented | killed | p12_3_answered_exactly_once, p12_4_deadline_answers_and_resets, q6_extra_poll_after_result_is_counted, stray_poll_tripwire |
| S2 drop assertion disabled | killed | stray_poll_tripwire |
| S3a further poll sets a reset | killed | p12_3, p12_4_deadline_answers_and_resets, p12_5_quarantine_and_offline_until_reboot |
| S3b further poll drops the cache | killed | p12_3, p12_4_deadline_answers_and_resets, p12_5_quarantine_and_offline_until_reboot |
| S3c further poll writes the caller's buffer | killed | p12_3, p12_4_deadline_answers_and_resets |
| S3d further poll quarantines | killed | p12_3, p12_4_deadline_answers_and_resets, p12_5_quarantine_and_offline_until_reboot, q6 |
| S3e further poll counts a flush | killed | p12_3, p12_4_deadline_answers_and_resets, p12_5_quarantine_and_offline_until_reboot |
| S3f further poll sets `crashed` (excluded from the snapshot) | killed | p12_3, p12_4_deadline_answers_and_resets, q6 |
| T1 default `Deadline::Enforced` (always-on equivalence probe) | only the should_panic tripwire tests fail | deadline_tripwire_flags_overdue_request, deadline_tripwire_flags_overdue_sparse_poll. All 82 other tests pass, so the default-off deadline differs from an always-on one only in the explicit opt-outs |
| T2 tripwire does not panic | killed | deadline_tripwire_flags_overdue_request, deadline_tripwire_flags_overdue_sparse_poll |
| B1 N26 (e) opt-out removed (the opt-out is necessary) | fails as expected | n26e_poll_beyond_bound_is_unknown |
| R1 src: `device::run` polls once more after a delivered result (Q6 kill) | killed | 67 tests by the drop assertion; 68 with `DEST_ORACLE_OUT` set (adds oracle_images) |
| R2 src: M7, sealed resolution compares with the catalog as built (`&next`) | killed | n35_sealed_resolution_uses_checked_copy |
| R3 src: C1 range narrowed (U+009F accepted) | killed | n34_record_and_copy_validity |
| R4 src: U+00A0 refused | killed | n34_record_and_copy_validity |
| P1 probe: overdue request unpolled at drop or at a refused submission | trips only the dedicated test | deadline_tripwire_flags_overdue_sparse_poll (run with `DEST_ORACLE_OUT` set) |

## Gates (worktree, `CARGO_TARGET_DIR=<worktree>/target-dc`)

| Command | Exit | Result |
|---|---|---|
| `cargo +1.100.0 test --locked -p destination-core` | 0 | 84 passed, 0 failed |
| `cargo +1.100.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 | |
| `cargo +1.90.0 test --locked -p destination-core` | 0 | 84 passed, 0 failed (broker 16, catalog 14, checks 13, crash 12, engine 5, ops 12, oracle_images 1, sim_contract 11) |
| `cargo +1.90.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 | |
| `cargo +1.90.0 fmt -p destination-core -- --check` | 0 | |
| `cargo +1.90.0 check --locked -p destination-core --target x86_64-unknown-none` | 0 | |
| `cargo +1.100.0 check --locked -p destination-core --target x86_64-unknown-none` | 0 | |
| `DEST_ORACLE_OUT=<scratch> cargo +1.90.0 test --locked -p destination-core --test oracle_images` (scratch copy, unmutated) | 0 | 1 passed, 60 image files emitted |

## Not checked

- The Python oracle (`tools/destination_disk.py`) and its suite were not run. Only grep confirmed they lack U+009F and U+00A0.
- Hosted CI, `agent_gate.py` jobs, QEMU and native gates were not run. No R5 or K5 kernel broker behaviour was checked; this is host-model evidence only.
- Mutants ran on 1.90.0 only. The workspace-wide `cargo fmt --all -- --check` was not run.
- Spec status rows that still read "open" were not edited (RV2-O5).
- The equivalence claims of earlier reviews (M8, A10, A11, B4, B6) were not re-run.
- This is not a human audit.

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_1ab97f17-fed` (review 2 of the spec revision 5 test round on PR #413, base `2c96869f`). It returned APPROVE (0 required, 5 optional). Not applied: OPT RV2-O1 and RV2-O2 (comment wording) are left as they are. RV2-O3 is answered by the lead's gate rerun, which ran the oracle cross-check with `DEST_ORACLE_OUT` set (exit 0, 30 images agree). RV2-O4 (add U+009F/U+00A0 to the Python oracle suite) and RV2-O5 (spec status text) go to the next spec revision and oracle round. An AI review, not a human audit.
