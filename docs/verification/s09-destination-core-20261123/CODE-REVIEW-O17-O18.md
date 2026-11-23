# Review S09-DEST-O17-O18-REVIEW-1: O17 (a), O18 and the ungated revision-4 items in `services/destination`

- **Date:** 23 November 2026
- **Base:** `8f387ea1` (the uncommitted `git diff HEAD` in `/home/simon/projects/BaryonOS-worktrees/s09-destination-core`)
- **Reviewer:** an independent AI reviewer that did not write the change. This is an AI review, not a human audit.
- **Authority:** the change was judged against `docs/implementation/S09-DESTINATION-SERVICE.md` revision 4 plus the owner decisions of 23 November 2026 (O17 (a), O18). The sections checked were: 2.7 step 3, "Stale valid content", the failure paragraph after step 5, the O16 and O18 paragraphs, the broker contract, section 6 rows N26, N33, N35 and N37, the section 9 CODE-DELTA (revision 4) table and P12.

## Verdict

**APPROVE. There are 0 required findings and 5 optional ones.**

## Scope check

- `git diff HEAD --stat` touches only `services/destination/**`:
  - modified: `src/catalog.rs`, `src/service.rs`, `tests/broker.rs`, `tests/common/mod.rs`, `tests/crash.rs`
  - new, untracked: `tests/sim_contract.rs`
- Nothing else changed: not the spec, not `Cargo.lock`, not the oracle.
- No commit or push was made.
- The worktree was not edited by this review. `sha256sum -c` of every `src/*.rs`, `tests/*.rs` and `tests/common/mod.rs` against a snapshot taken before the review reports all OK. All mutations ran in a scratch copy.

## In-scope items

### A. O17 (a): SEAL content check (2.7 step 3)

**Form chosen.** The change uses the running form, which is the spec's primary wording. `Writing::digest` (`service.rs:127`) starts at `FNV_OFFSET` when CREATE runs. It is updated with the crate's `fnv64`, the same FNV-1a 64 used for the catalog checksum and the archive trailer.

**Only APPENDs answered `Ok` are hashed.** The update is at `service.rs:542`. It is the last statement before the `Ok` reply, and it comes after:
- the `Stale` offset check
- the `Range` and `Full` refusals
- every `write_entry` in the loop

A device error inside the loop returns through `device_failure`, which discards the writing entry. No path updates the digest for an APPEND that was not answered `Ok`. The only fields written are at `service.rs:468` (init), `:532` (`appended`) and `:542` (`digest`).

**The readback digest covers exactly the first `length` bytes.**
- `Extent::get` hashes the slice it returns (`service.rs:911`) and is capped at `length`.
- `check_extent` returns the digest only when `position == length`.
- `Source` is sequential with no seek, so the digest is over the bytes in order.

**Comparison** (`service.rs:583`). It runs after the readback and after the archive verifier. On a mismatch, or on any readback error:
- it calls `discard()`, which sets the export to `NotSealed`;
- it returns `Corrupt`; on an `Err` it returns that error;
- it does not set `safe_path`;
- it writes no catalog sector, because the function returns before step 4.

This matches the O17 recommendation of no latch.

**SEAL `aux` on success** is the readback trailer. With matching digests this is the trailer of the bytes sent, and `sealed_as_sent` asserts `aux == trailer(sent)`.

**Tests:**
- **N33 (iii) became the negative test `n37a_read_timeout_over_stale_valid_archive`.** It covers a READ and a VERIFY reset on lane 3 under `both_reset_models`. With the cache dropped: `Corrupt`, only E1 listed, `catalog_writes == 0`, binding `NotSealed`, no safe path, and a CREATE on a new binding returns `Ok`. With the cache kept: `Ok`, `aux` equals the sent trailer, two entries, and every listed entry verifies.
- **`n37a_own_reset_at_barrier1_over_stale_valid_archive`** covers the device's own reset during the barrier-1 submission (`again: RESET_TIMEOUT` on the first flush after `arm`) under both models. The stale archive here is not sector-aligned, which the N37 row permits.
- **`n37b_unstored_entry_writes_over_stale_valid_archive`** uses the new `Fault::silent` mode. Every write at `sector >= DATA_START` reports success and stores nothing, and the test asserts that the stale extent is still whole and that no reset occurred.
- **`n37_digest_covers_only_appends_answered_ok`** pins the rule that refused `Stale` and `Full` APPENDs are not hashed.

### B. O18: sealed only on equality with the encoded copy

- `catalog_failure` now takes `encoded`, and the call site passes `&checked` (`service.rs:638`). `checked` is the `catalog::decode` of the exact `bytes` written in steps 4–5, after the test-only tamper hook, so it is the copy checked in step 4.
- `remounted == *encoded` (`service.rs:673`) is full `Catalog` equality. `catalog::decode` is canonical:
  - the magic, version and checksum bytes are fixed;
  - bytes 11..16, the record padding at 41..48, the label tail, the unused records and `RECORDS_END..SUM_AT` must all be zero.

  For valid copies, field equality is therefore byte equality, as the spec says.
- **No other catalog can satisfy the equality.** Any catalog that differs in length, trailer, label or next entry ID falls through to `State::Mounted | State::Blank`, which gives `Failed { no_device: false }` with the binding left `Unknown`. That includes one that passes `extends_by_one` and carries the provisional ID.
- The committed-copy arm still resolves `NotSealed`.
- **`extends_by_one` is kept** only for the step-4 pre-write check, which the spec still requires, and its doc comment was updated.
- **N35 (e)** (`crash.rs`) adds the four cases: length (+1 within the same sector count), trailer, label and next entry ID. Each case asserts that it passes `extends_by_one` with the provisional ID and that it is a valid copy. It then asserts the (a)–(c) outcomes: failed-mount state, binding `Unknown`, INFO `Io`, export-lane LIST `Unknown` with zero records, VERIFY and FORMAT `Io`, and no write after the failed one.

### C. Ungated revision-4 items

**N26 (d).** `assert_eq!(h.disk.counts.catalog_writes, 0)` was added in the cache-dropped branch (`broker.rs:141`). `catalog_writes` counts submissions to sectors below 8, so the assertion is meaningful. Mutant M9 makes it fail with "no catalog write, left: 4, right: 0".

**P12** (`tests/sim_contract.rs`). There is one test per property 1–5, each under `both_reset_models`. Property 3 covers:
- a second poll after a delivered write result and after a delivered read result (−1, buffer unchanged);
- requests ended by a reset with −16 and with −13, each answered once and followed by −1;
- an assertion that the reset model was actually exercised.

The harness's poll with no request now returns −1. This matches the kernel mapping of `WrongTicket` to −1 (`kernel/src/m2/m3/volume.rs:777`). Properties 4 (the deadline itself) and 5 (persistence until reboot) are not modelled by `SimDisk` and are stated as such in the module comment rather than faked; see OPT-2.

## Findings

| ID | Severity | File:line | Problem | Fix |
|---|---|---|---|---|
| OPT-1 | optional | `services/destination/src/service.rs:638`; `services/destination/tests/crash.rs:716` | Passing `&next`, the pre-decode catalog, instead of `&checked` to `catalog_failure` survives every test (M7). In product builds it is equivalent, because the `test-hooks` tamper is absent and `decode(next.encode()) == next` for every copy that passes the validator. It differs only in test builds, under a tamper that passes both `decode` and `extends_by_one` followed by a failed step-4 write. The N35 comment lists only "adding the extension check back" as equivalent. | Add M7 to the N35 equivalent-mutant list as "equivalent in every build without `test-hooks`". Optionally, add an N35 sub-case that pins `checked`: a tamper that passes step 4 (for example another valid label byte with the checksum recomputed), then a step-4 write failure, with the tampered copy remounted, expecting `Sealed`. |
| OPT-2 | optional | `services/destination/tests/sim_contract.rs:5` | The heading says properties 4 and 5 are "Not observable on `SimDisk`". More precisely, `SimDisk` does not model them: it has no admission-based deadline that answers −16 or −13 by itself, and quarantine and offline are injected per submission, not held as device state. The spec requires every host model to reproduce properties 1–5. N26 (e) and (f) need a stub that breaks property 4 on purpose, so a default deadline would conflict with them. | Reword the heading to "not modelled by `SimDisk`". The lead should report P12 and the claim "simulated disk meets the K5 broker contract" as partial for properties 4 and 5, not as complete. A later step can add an opt-in deadline mode and a persistent quarantine and offline state. |
| OPT-3 | optional | `services/destination/tests/common/mod.rs:383` | `SimDisk::poll` with no request used to panic (`expect("poll without request")`) and now returns −1. Property 3 needs this, but it removes a tripwire: a service bug that polls with nothing pending now surfaces only as an `Io` or `Unknown` status through `poll_error`, not as a panic. | Push an `Event::StrayPoll` (or a counter) in that branch, and assert that it is zero in the service-driven harness. The P12 tests may still observe −1. |
| OPT-4 | optional | `services/destination/src/service.rs:127` | The task asked the implementer to pick a form and say why. The code shows the running form but gives no reason. | State the reason in the PR body or in one comment line. The running form follows the spec's primary wording and covers every byte regardless of the archive format or the verifier stand-in. The trailer form relies on `archive::verify`'s trailer rule, which the test stand-in only approximates. |
| OPT-5 | optional, outside the write set (lead) | `docs/implementation/S09-DESTINATION-SERVICE.md:2124` | The CODE-DELTA row still names `n33_read_timeout_over_stale_valid_archive_recorded`. That test is now `n37a_read_timeout_over_stale_valid_archive`. Historical mentions in `docs/verification/**` stay as they are. | Lead spec update in a later revision. |

No required findings:
- The content check cannot be bypassed.
- The mismatch path writes no catalog sector and does not latch.
- O18 equality cannot be met by a different valid catalog.
- The security floor holds: `#![forbid(unsafe_code)]` is still in force, the crate is `no_std` and allocator-free (the `x86_64-unknown-none` check passes), and no new trust is placed in disk bytes. The FNV `wrapping_mul` is intended; no new unchecked arithmetic was added on disk-derived values.

## Mutation evidence

All mutants ran in a scratch copy under `/tmp/claude-1000/-home-simon-projects-BaryonOS/171c7b49-ee33-4342-8270-24a1e42ae3e2/scratchpad/mut`. The script was `/tmp/claude-1000/-home-simon-projects-BaryonOS/171c7b49-ee33-4342-8270-24a1e42ae3e2/scratchpad/mutate.py`. Each mutant is a single-occurrence replacement, run with `cargo +1.100.0 test --locked -p destination-core --no-fail-fast`, then restored. The restoration was checked by sha256: `service.rs` `d1a437c1422a15d7…` and `common/mod.rs` `99613cc2340ede40…` were identical before and after every run.

| Mutant | Rule | Result | Killing tests |
|---|---|---|---|
| M1 content check removed (`Ok((_, trailer, _)) => trailer`) | O17 (a) | killed | n37a ×2, n37b |
| M2b digest update moved before the `Stale` offset check | APPENDs answered `Ok` only | killed | n37_digest_covers_only_appends_answered_ok (also n17_replay_and_rebind) |
| M3b digest update moved before the `Full` scope check | APPENDs answered `Ok` only | killed | n37_digest_covers_only_appends_answered_ok |
| M13 digest never updated | O17 (a) | killed | n26b/d/f, n33 ×3, n37 ×4 |
| M12 digest skipped for full-size APPENDs | O17 (a) | killed | n26b/d/f, n33 ×3, n37 ×4 |
| M4 mismatch sets `safe_path` | no latch | killed | n37a ×2, n37b, n26d, n33_read_timeout_with_unflushed_append, n10_non_archive_bytes |
| M5 mismatch replies `Io` instead of `Corrupt` | −18 | killed | n37a ×2, n37b |
| M9 mismatch path writes a catalog copy | no catalog write | killed | n26d (panics at `broker.rs:141`, "no catalog write", 4 ≠ 0), n37a ×2, n37b, n33_read_timeout_with_unflushed_append |
| M6 sealed resolution by `extends_by_one` plus provisional ID (the pre-O18 code) | O18 | killed | n35_remount_contradicting_committed_catalog |
| M16 sealed resolution never taken | O18, positive arm | killed | n13_unknown_at_seal, n24a_late_write_before_fence |
| M8 `remounted == *encoded && extends_by_one(..)` | O18 | **survived** | Equivalent: byte equality implies the extension check. Listed in the spec's N35 row and at `crash.rs:716`. |
| M7 `&next` instead of `&checked` at `service.rs:638` | O18 encoded copy | **survived** | Equivalent in product builds (OPT-1); not listed. |
| M10 `Fault::silent` ignored by `SimDisk` | N37 (b) harness | killed | n37b |
| M11 `SimDisk::poll` panics again with no request | P12 property 3 | killed | p12_3, p12_5 |
| M14 `SimDisk` admits a submission while one is in flight | P12 properties 1, 2 | killed | p12_1, p12_2, n26f |
| M15 `SimDisk` has no reset fence | P12 properties 2, 4 | killed | p12_2, p12_4 |

The first-batch M2 and M3 inserted a second digest update and so hashed the data twice. They are not valid evidence and were replaced by M2b and M3b.

## Gates

All gates ran from the worktree with `CARGO_TARGET_DIR=…/target-dc`.

| Command | Exit | Evidence |
|---|---|---|
| `cargo +1.100.0 test --locked -p destination-core` | 0 | 10 result lines, all ok (76 tests) |
| `cargo +1.100.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 | |
| `cargo +1.100.0 fmt --all -- --check` | 0 | |
| `cargo +1.100.0 check --locked -p destination-core --target x86_64-unknown-none` | 0 | |
| `cargo +1.90.0 test --locked -p destination-core` | 0 | 10 result lines, all ok |
| `cargo +1.90.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 | |
| `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 0 | 109 tests, OK; the one skip is `RustCrossCheck.test_rust_images` ("DEST_ORACLE_OUT not set"), which is run separately below |
| `python3 -m compileall -q tools/destination_disk.py` | 0 | |
| `DEST_ORACLE_OUT=<fresh mktemp -d> cargo +1.100.0 test --locked -p destination-core --test oracle_images` | 0 | 30 `.expect` files plus images written to `/tmp/claude-1000/-home-simon-projects-BaryonOS/171c7b49-ee33-4342-8270-24a1e42ae3e2/scratchpad/oracle.cJc83R` |
| `DEST_ORACLE_OUT=<same> python3 -B -m unittest tests.test_volume_destination_disk.RustCrossCheck -v` | 0 | `test_rust_images ... ok`; the test iterates every `*.expect` in the directory (30) |
| Extra, P10 indicative: `cargo +1.90.0 clippy --locked -p destination-core --lib --target x86_64-unknown-none -- -D warnings` | 0 | Local only; the hosted gate is still a lead-owned workflow change |

## Not checked

- O17 (b) portal trailer comparison and N37 (c): R7, no portal on this PR.
- The real `volume_core::archive::verify`. The verifier is still a test stand-in, so N7, N10, N11 and N12 stay partial.
- A hostile device or an FNV-64 collision. This is outside the threat model in 2.7: the content check detects accidental faults only.
- P12 property 4's deadline and property 5's persistence. `SimDisk` does not model them (OPT-2).
- Native behaviour of the K5 broker (R5), hosted CI, and the R3 reverse differential.
- Whether a digest mismatch should latch the safe path. That follows the O17 recommendation of no latch and was not re-decided here.

---

*Provenance:* recorded verbatim from the structured output of the local workflow run `wf_8e6662b8-58e` (review 1 of the O17 (a) / O18 code change, on top of `8f387ea1`). It returned APPROVE, so the workflow ran no fix pass and no re-review. The lead applied OPT-1 (N35 comment lists the equivalent mutant M7), OPT-2 (`sim_contract.rs` heading: properties 4 and 5 not modelled, claim partial) and OPT-4 (reason for the running form) as comments only; OPT-3 (stray-poll tripwire) is deferred and OPT-5 (stale test name in the spec CODE-DELTA) goes to the next spec revision. An AI review, not a human audit.
