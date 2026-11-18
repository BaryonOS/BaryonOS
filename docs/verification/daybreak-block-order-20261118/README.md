# Daybreak block syscall ordering — 18 November 2026

## Task contract

- Goal: reject valid-shape M6b/M6c WRITE and FLUSH calls that arrive outside an
  active accepted mutation instead of panicking the kernel.
- Invariant: capability, scope, pointer and length checks remain first; an
  observer rejection returns `Invalid` before policy preparation or device I/O
  and does not change publication, completion or journal state.
- Write set: `kernel/src/m2/m3/m6b.rs`, `kernel/src/m2/m3/m6c.rs`,
  `userspace/m6b_probe.rs`, `userspace/m6c_probe.rs`, this new evidence child,
  and the lead-owned project checkpoint/progress assessment.
- Acceptance: source-bound negative M6b/M6c `errors` runs on the unrepaired
  kernel; repaired targeted builds and native `errors` runs; M6b/M6c host and
  full native suites; strict preflight, required hosted CI and independent
  exact-head review.
- Evidence limit: host and QEMU results do not establish physical-hardware
  behaviour, formal proof or broader production-storage acceptance.

Base source is main `1c10cdb5b30df2f3d30bea74430383f59c398931`.
Finding `acf_4d7b2e454297c2c851a4b7cadcff2587` remains open until the repaired exact
head and resulting main are rescanned and accepted. The preserved broad WIP in
`sec-m6-observer-local` is not used or modified.

## Result

The repaired source fingerprint is
`5a6a5cca407e288973e302ae46840c7f851a57db70a848917f552c23821d4fbf`.
M6b and M6c now validate shape, capability, scope, sector/ticket and user-copy
arguments before their fixture ordering observer. After startup recovery:

- unexpected READ, WRITE and FLUSH ordering is rejected before policy
  preparation or device publication;
- a second submit cannot define the outcome of any already-live request;
- only a POLL bound to the live capability, ticket and output plan can make its
  own mutating request terminal;
- M6c terminal state is idempotent and rejects later valid, malformed and read
  attempts without changing accepted-call, publication, completion or journal
  state;
- every error retains secondary result zero.

Case 2 performs the following real ring-3 sequence before the ordinary service
continues: active READ (`Invalid`), published WRITE, duplicate WRITE and FLUSH
(`Busy`), malformed WRITE (`Length`), wrong-ticket POLL (`Invalid`), normal
completion of the original ticket and one normal FLUSH. M6c then also performs
mixed repeated terminal retries. Kernel-owned counters require exactly three
M6b and nine M6c observer denials.

## Negative controls

The probe-only pre-active ordering patch had SHA-256
`ba47b436d0f381c6c60fcf65885addc701ae34040682e0fb07f5cb7eb82beb5d`.
Against unrepaired main source fingerprint `ee79396e...`, both `errors` runs
timed out with raw QEMU exit `-9`, no M6 pass marker and unchanged disk images:

- M6b: `build/tokenguard/run-hppfpvog.log`, result
  `build/daybreak-m6b-negative/run/m6b-errors-67009b8f14/result.json`;
- M6c: `build/tokenguard/run-_bm10fjp.log`, result
  `build/daybreak-m6c-negative/run/m6c-errors-c55788edcb/result.json`.

Two further source-bound negative controls exercised the independently found
live-I/O variants. Before Busy classification was corrected, both runs failed
with raw guest exit `35` at source fingerprint `49146705...` (`run-hxrzvm_5.log`
and `run-51djtt_z.log`). After valid-shape Busy handling but before malformed
retry classification was corrected, both runs again failed with raw guest exit
`35` at fingerprint `7bd7751d...` (`run-gut6l2td.log` and
`run-2bdcon3l.log`). These failures are expected negative evidence, not passes.

## Repaired-source verification

| Check | Result | Raw evidence |
| --- | --- | --- |
| M6b targeted build and `errors` QEMU | exit 0; guest 33; disk oracle pass | `run-fzpjwshe.log`, `run-87kor9jd.log`, `build/daybreak-m6b-read-fixed/run/m6b-errors-50eb8bdade/result.json` |
| M6c targeted build and `errors` QEMU | exit 0; guest 33; disk oracle pass | `run-6fdfqj2p.log`, `run-h17b3plz.log`, `build/daybreak-m6c-read-fixed/run/m6c-errors-2cae1117f2/result.json` |
| M6b host harness | 38 tests, exit 0 | `build/tokenguard/run-751n_hf4.log` |
| M6c host harness | 49 tests, exit 0 | `build/tokenguard/run-2qe3egpt.log` |
| object-core | 22 tests, exit 0 | `build/tokenguard/run-e0tg6xog.log` |
| catalog-core | 45 tests, exit 0 | `build/tokenguard/run-hnynuax0.log` |
| Full native M6b | 32 groups, exit 0; final M1 guest 33 | `build/tokenguard/run-euls2rq9.log` |
| Full native M6c | 35 groups, exit 0; final M1 guest 33 | `build/tokenguard/run-iwff7hg3.log` |
| Strict preflight | exit 0; no missing baseline tools | `build/tokenguard/run-bksra411.log` |
| Rust formatting and diff whitespace | exit 0 | `cargo +1.90.0 fmt --manifest-path kernel/Cargo.toml -- --check`; `git diff --check` |
| Independent security re-review | PASS; no remaining ordering counterexample in the bounded paths | exact source fingerprint above |

The final full-suite `errors` and cold-recovery receipts all report raw QEMU
exit `33`, `source_unchanged: true`, matching expected and observed source
fingerprints, and passing independent disk oracles. Their paths and digests are
recorded in `receipt.json`.

Two earlier commands selected `tools/tests` instead of `tests` and exited `5`
with zero discovered tests (`run-vw85fmen.log`, `run-tuiewj9k.log`). They are
retained as selection errors and are not acceptance evidence. The corrected
38-test and 49-test suites above supersede them.

## Limits and remaining gates

No physical hardware, formal proof, hosted CI or fresh Daybreak scan is claimed
by this local evidence. Required current-head `host-quality`, `qemu-smoke`,
CodeQL and merge-hold checks must pass after publication. The finding remains
open until Daybreak accepts the published exact head and the resulting main.
