# Review S09-DEST-REV5-TESTS-1: spec revision 5 test obligations in services/destination

- **Date:** 23 November 2026
- **Base:** `2c96869f` (branch `claude/s09-destination-core`, draft PR #413); the review covers the uncommitted diff over this base (6 modified test files, no untracked files)
- **Reviewer statement:** independent AI code review. It is not a human audit, and the reviewer did not write the change.
- **Verdict:** REQUEST_CHANGES (one required finding: comments overclaim the deadline and tripwire behaviour; the fix is one line and keeps all 82 tests green)

## Scope check

| Check | Result |
|---|---|
| `src/` touched | No. `git diff --quiet HEAD -- services/destination/src Cargo.lock services/destination/Cargo.toml rust-toolchain.toml` exit 0 |
| Files changed | `tests/{broker,catalog,common/mod,crash,oracle_images,sim_contract}.rs`; no untracked files |
| Worktree unmodified by review | `sha256sum -c rv1-before.sha`: 16/16 OK, exit 0 (all files under `services/destination/` plus `MPL-2.0-FILES.txt`). Every mutation ran in a scratch copy (`mut.Edpr/wt`) with its own `CARGO_TARGET_DIR`, and each file was restored and byte-compared after each mutant |
| Test count | 82, up from 76: `p12_4_deadline_answers_and_resets`, `p12_5_quarantine_and_offline_until_reboot`, `deadline_tripwire_flags_overdue_request`, `stray_poll_tripwire`, `q6_extra_poll_after_result_is_counted`, `n35_sealed_resolution_uses_checked_copy` |

### CODE-DELTA (revision 5) obligations

| Obligation (spec) | Status | Evidence |
|---|---|---|
| Q4 property 4: opt-in deadline mode, -16 mutating / -13 read by the model, then reset, under both reset models | Met, with the overclaim in RV1-R1 | `Deadline::Enforced`; `p12_4_deadline_answers_and_resets` (both models; from admission; ack before reuse). Kills D1, D1b, D2a, D2b, D3, R1, T3 |
| Q4 default off; N26 (e), N26 (f) and the p12_2 "given up on" case keep their stubs | Met | Those three set `Deadline::Unchecked`, and the Unchecked path is identical to the old poll code. O1–O3 (removing each opt-out) each fail exactly that test. D5c shows that only `p12_5_quarantined_or_offline_not_admitted` depends on the new hold, so no existing row relies on it |
| Q4 property 5: quarantine (-13) and offline (-14) held as device state until a simulated reboot; kills: admits before reboot, ends by itself | Met | `down` field, `rebooted()`, `ResetEnd`; `p12_5_quarantine_and_offline_until_reboot` (both models, reset caused by the model's deadline or by a -16 result, 3 × TIMEOUT rounds) and the extended `p12_5_quarantined_or_offline_not_admitted` (held with the hook removed). Kills D4a–D4d, D5a–D5e |
| Properties 4/5 shown by SimDisk's own state, not by injection | Met | The deadline answer and the hold come from model state. Injection is limited to device latency (`pending`) and the reset outcome (`reset_end`) |
| Q4 optional OPT9-7 tripwire | Done (default `Tripwire`), with the gap in RV1-R1 and the unpinned boundary in RV1-O1 | T2, T4 killed; T1 survives |
| Q6: stray-poll counter in `SimDisk::poll`, asserted zero in every service-driven host test; kill: a service loop that polls again after a result | Met | `Drop` asserts zero unless `stray_polls_allowed`, which only P12 tests and the Q6 kill test set. All test `Broker` wrappers (`Swapped`, `Watch`, `PollTwice`) delegate to `SimDisk::poll`. The real `src` mutant D7a (an extra poll in `device::run` after a 0) fails 67 tests. The 6 non-`sim_contract` tests that still pass complete no device request: 4 codec tests in `catalog.rs`, `n26a_mount_flush_again_forever` (never admitted, so never polled) and `oracle_images` (returns early without `DEST_ORACLE_OUT`) |
| Q7: device-state snapshot before and after each further poll | Met | `further_poll` with `snapshot` (durable, cache, totals, `broker_state()`), 4 times in `p12_3` and once in `p12_4_deadline`. The broker state matters: with it ablated, F1 is no longer killed by `p12_3` (S1F1) and F7 survives (S1F7) |
| Optional N35 sub-case pinning the checked copy (Q5) | Done | `n35_sealed_resolution_uses_checked_copy` kills M7 (D8) |
| Optional N33 (ii) zero bytes | Done | Zero-stale variant added to `n33_read_timeout_with_unflushed_append` (not separately mutated) |
| Optional U+009F / U+00A0 | Done in the Rust core only (stated in the comment) | C9F, CA0 killed by `n34_record_and_copy_validity` |

## Findings

| ID | Class | Location | Problem | Fix |
|---|---|---|---|---|
| RV1-R1 | REQUIRED (overclaim) | `common/mod.rs` `SimDisk::poll` `past_deadline`; module comment; `Deadline::Tripwire`/`Enforced` docs; `sim_contract.rs` module comment and `deadline_tripwire_flags_overdue_request` doc | The comments claim that a request still pending at admission + TIMEOUT is answered -16/-13 with a reset (Enforced) or fails the test (Tripwire; "a request whose device result is ready only after admission + `TIMEOUT` fails the test"). The model checks only polls before `ready`. Probe: Enforced, FLUSH with `pending(TIMEOUT + 5)`, ticks advanced by refused submissions, first poll at tick 512 (admission 1) returned `code 0` with no reset. The same sequence under Tripwire returned `code 0` with no panic. No current test reaches the gap (P2: 82/82 with it closed) | `past_deadline = self.tick >= admitted + TIMEOUT && ready > admitted + TIMEOUT`, plus a sparse-poll P12 case. Alternatively, qualify the five comment sites |
| RV1-O1 | optional | `deadline_tripwire_flags_overdue_request` | T1 (`< ready` changed to `<= ready`) survives, because `should_panic` accepts the panic from the first FLUSH, so "ready exactly at the deadline is the device's" is unpinned | Put the second admission tick in `expected`, or add a separate non-panicking boundary test |
| RV1-O2 | optional | `SimDisk::submit` ordering | On a down device, an `again` hook still sets `again_until` and, in the dropping model, drops the cache. Probe: `code -13, cache before 1 after 0, dropped 0 -> 1`. No stated claim is falsified, because the p12_5 claims hold with the hook removed | Apply `again` and the cache drop after the `down` check |
| RV1-O3 | optional | `sim_contract.rs` `snapshot` doc | The exclusion list omits `counts`, `crashed` and `dropped`. F6 (a poll increments `counts`) survives | Name these as instruments |
| RV1-O4 | optional | spec section 9 CODE-DELTA (rev. 5), 8.2 R2 status | The rows still read open, and the oracle side of U+009F/U+00A0 does not exist yet | The lead updates the status after the commit |

Classification rule: a finding is REQUIRED only when a stated comment or claim is falsified, or a CODE-DELTA obligation as worded is unmet. RV1-O2 changes state only under a fault combination that no comment covers, so it is optional.

## Mutation evidence

Each mutant is a single logical change made in a scratch copy and restored with a byte compare afterwards. Each ran `cargo +1.100.0 test --locked -p destination-core --no-fail-fast`. Every mutant compiled and rebuilt (a `Compiling destination-core` line appears in every survivor log).

| ID | Mutant | Result | Killed by |
|---|---|---|---|
| D1 | Deadline answers the other kind's code | killed | `p12_4_deadline_answers_and_resets`, `p12_5_quarantine_and_offline_until_reboot` |
| D1b | Deadline answers -12 (no reset) | killed | same two |
| D2a | Deadline measured from the first poll instead of admission | killed | `p12_4_deadline_answers_and_resets`, `deadline_tripwire_flags_overdue_request` |
| D2b | Deadline measured from the first submission attempt | killed | `p12_4_deadline_answers_and_resets` |
| D3 | Deadline never answers (Enforced acts as Unchecked) | killed | `p12_4_deadline…`, `p12_5_quarantine…` |
| R1 | Reset after -16/-13 does not wait (reuse before ack) | killed | `p12_2`, `p12_4_admission_time_and_reset_fence`, `p12_4_deadline…`, `p12_5_quarantine…` |
| D4a | Quarantine/offline cleared after one refused submission | killed | `p12_5_quarantine…`, `p12_5_quarantined_or_offline_not_admitted` |
| D4b | Quarantine/offline ends by itself (tick ≥ 3000) | killed | `p12_5_quarantine…` |
| D4c | Quarantine/offline cleared by a stray poll | killed | `p12_5_quarantine…` |
| D4d | Reboot keeps quarantine/offline | killed | both p12_5 tests |
| D5a | Offline (failed re-init) returns -13 | killed | `p12_5_quarantine…` |
| D5b | Injected -13/-14 always held as -13 | killed | `p12_5_quarantined_or_offline_not_admitted` |
| D5c | Injected -13/-14 not held | killed | `p12_5_quarantined_or_offline_not_admitted` |
| D5d | Reset timeout does not quarantine | killed | `p12_5_quarantine…` |
| D5e | Reset timeout acknowledged, device ready | killed | `p12_5_quarantine…` |
| D6a | Stray-poll counter never incremented | killed | `p12_3`, `p12_4_deadline…`, `q6_extra_poll_after_result_is_counted`, `stray_poll_tripwire` |
| D6b | `Drop` assertion removed | killed | `stray_poll_tripwire` |
| D7a | `src`: `device::run` polls once more after a delivered 0 | killed | 67 tests (every service-driven test that completes a request) |
| D7b | `src`: `device::run` polls once more after an error result | killed | 23 tests (N8, N11–N13, N17, N24, N25, N28, N31, N33, N35, N37a, O1) |
| D8 | `src` M7: sealed resolution compares with the pre-decode catalog | killed | `n35_sealed_resolution_uses_checked_copy` |
| F1 | Further poll clears the reset | killed | `p12_3`, `p12_4_deadline…` |
| F2 | Further poll drops the cache | killed | `p12_3`, `p12_4_deadline…`, `p12_5_quarantine…` |
| F3 | Further poll writes the caller's buffer | killed | `p12_3`, `p12_4_deadline…` |
| F4 | Further poll sets `crashed` | killed | `p12_3`, `p12_4_deadline…`, `q6…` (by later results; not in the snapshot) |
| F5 | Further poll increments the flush total | killed | `p12_3`, `p12_4_deadline…`, `p12_5_quarantine…` |
| F6 | Further poll increments the fault-hook `counts` | **survived** | a test instrument outside the snapshot (RV1-O3) |
| F7 | Further poll arms a late write | killed | `p12_3`, `p12_4_deadline…`, `p12_5_quarantine…` |
| T1 | Tripwire/deadline boundary `tick < ready` changed to `<=` | **survived** | RV1-O1 |
| T2 | Default `Deadline` is Unchecked | killed | `deadline_tripwire_flags_overdue_request` |
| T3 | Deadline one tick late (`>=` changed to `>`) | killed | `deadline_tripwire…`, `p12_4_deadline…` |
| T4 | Default `Deadline` is Enforced (always-on) | killed only by `deadline_tripwire…` | shows an always-on deadline changes no other test |
| E1 | Deadline answer lets the write land (model choice) | killed | `p12_4_deadline…` |
| O1 / O2 / O3 | Remove the Unchecked opt-out of N26 (e) / N26 (f) / p12_2 | killed | `n26e…` / `n26f…` / `p12_2…` respectively (the opt-outs are not vacuous) |
| C9F | `src`: U+009F accepted in labels | killed | `n34_record_and_copy_validity` |
| CA0 | `src`: U+00A0 refused in labels | killed | `n34_record_and_copy_validity` |
| S1F1 | Snapshot without broker state, plus F1 | killed only by `p12_4_deadline…` (no longer by `p12_3`) | the broker state in the snapshot is what lets `p12_3` kill F1 |
| S1F7 | Snapshot without broker state, plus F7 | survived | the broker state in the snapshot is needed |

Probes (not mutants):
- **P2:** `past_deadline` decided by `ready > admitted + TIMEOUT`. 82/82 pass, so no current test reaches the sparse-poll gap.
- **P2e:** P2 with default Enforced fails only `deadline_tripwire_flags_overdue_request`.
- **Sparse-poll probe output:** `PROBE sparse enforced: code 0 at tick 512 (admission 1)` and `PROBE sparse tripwire: code 0, no panic`.
- **Down-plus-again probe output:** `drops false: code -13, cache before 1 after 1`; `drops true: code -13, cache before 1 after 0, dropped 0 -> 1`.
- **`oracle_images` with `DEST_ORACLE_OUT` set (scratch dir):** 1 passed, and the images were emitted, so the `.durable.clone()` change still works.

## Gates (worktree, `CARGO_TARGET_DIR=…/s09-destination-core/target-dc`)

| Command | Exit |
|---|---|
| `cargo +1.100.0 test --locked -p destination-core` | 0 (82 passed, 0 failed) |
| `cargo +1.90.0 test --locked -p destination-core` | 0 (82 passed, 0 failed) |
| `cargo +1.100.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 |
| `cargo +1.90.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 |
| `cargo +1.100.0 fmt -p destination-core -- --check` | 0 |
| `cargo +1.100.0 fmt --all -- --check` | 0 |
| `cargo +1.100.0 check --locked -p destination-core --lib --target x86_64-unknown-none` | 0 |
| `cargo +1.90.0 check --locked -p destination-core --lib --target x86_64-unknown-none` | 0 |
| `cargo +1.90.0 clippy --locked -p destination-core --lib --target x86_64-unknown-none -- -D warnings` (P10, local only) | 0 |
| `sha256sum -c rv1-before.sha` (after all runs) | 0 (16/16 OK) |

## Not checked

- Hosted CI (`ci.yml#host-quality`, `qemu-smoke`) and the P10 hosted gate. Everything above ran locally.
- Native or QEMU behaviour and the real K5 broker. This review concerns the host model only.
- The oracle (`tools/destination_disk.py`) side of U+009F/U+00A0, which does not exist yet.
- Mutants under Rust 1.90.0. Mutants ran on 1.100.0 only.
- The N33 (ii) zero-byte variant was not separately mutated.
- The spec status rows (CODE-DELTA (revision 5), 8.2 R2), which still read open. This diff makes no doc changes.
- Hosted Graphify `recall` was not used, because the review read the worktree files directly.
- Human audit: this review is not one.

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_1ab97f17-fed` (review 1 of the spec revision 5 test round on PR #413, base `2c96869f`). It returned REQUEST_CHANGES (1 required, 4 optional); the workflow fix pass resolved all five and review 2 re-reviewed the result. An AI review, not a human audit.
