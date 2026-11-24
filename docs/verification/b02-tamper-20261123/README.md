# B02-T1 native tamper and key-loss cold-reopen evidence (23 November 2026)

Task: B02-T1, "native tamper and key-loss cold-reopen cases with exact refusal markers".
This directory holds the native runs for that task at one frozen source. Volume images
and binaries are not copied. `SHA256SUMS` covers every other file here.

## Source and environment

| Item | Value |
|---|---|
| Source commit (recorded by every `result.json`) | `18643f779f03ce9614ee9a56df5bcf5ce082b28f` (branch `claude/b02-tamper-native-20261123`, base `297d7bfc`) |
| Source fingerprint (`source_sha256`, v3) | `031b10c20abd…` (full value in each `result.json`) |
| Rust toolchain | `1.100.0` (from `rust-toolchain.toml`; `tools/build.py` invokes `cargo +1.100.0 build --locked`) |
| QEMU | 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3), `-machine pc-q35-8.2`, `-accel tcg,thread=single`, OVMF |
| Isolation | `NOS_BUILD_DIR`, `CARGO_TARGET_DIR`, `NOS_TARGET_DIR` inside the worktree; one QEMU guest at a time |

## Commands

The driver ran these in order (`steps.tsv` holds each runner exit and run directory):

```
unset NOS_SYSTEM_COLD_CASE
python3 -B tools/build.py --kernel-feature native-system
python3 -B tools/run_system.py --case normal                       # parent image P
for c in keystate record-tag swapped-blocks withheld-secret; do
  NOS_SYSTEM_COLD_CASE=$c python3 -B tools/build.py --kernel-feature test-system-recover
  NOS_SYSTEM_COLD_CASE=$c python3 -B tools/run_system.py --case recover --cold-case $c --volume P
done
# negative control: the withheld-secret kernel is still built
NOS_SYSTEM_COLD_CASE=keystate python3 -B tools/run_system.py --case recover --cold-case keystate --volume P
unset NOS_SYSTEM_COLD_CASE
python3 -B tools/build.py --kernel-feature test-system-recover
python3 -B tools/run_system.py --case recover --volume P            # plain recover, last on P
python3 -B -u tools/test_system.py                                  # own normal + recover + final artifact check
```

Each build produced a distinct kernel (`runs.json` → `kernels_built`). That shows the
compile-time `NOS_SYSTEM_COLD_CASE` reached each kernel.

## Results

| Run | Runner exit | Raw QEMU exit | Cold markers | Plaintext lines | Disk writes / flushes | Copy before = after | Parent before = after |
|---|---|---|---|---|---|---|---|
| 0-normal | 0 | 33 | none | 0 | 113 / 8 (normal writes) | n/a (writable) | n/a |
| 1-cold-keystate | 0 | 33 | exactly `SYSTEM:COLD:REFUSED:keystate` | 0 | 0 / 0 | yes | yes |
| 2-cold-record-tag | 0 | 33 | exactly `SYSTEM:COLD:REFUSED:record-tag` | 0 | 0 / 0 | yes | yes |
| 3-cold-swapped-blocks | 0 | 33 | exactly `SYSTEM:COLD:REFUSED:swapped-blocks` | 0 | 0 / 0 | yes | yes |
| 4-cold-withheld-secret | 0 | 33 | exactly `SYSTEM:COLD:REFUSED:withheld-secret` | 0 | 0 / 0 | yes | yes |
| 5-control-wrong-case | **1** (expected) | 33 | `SYSTEM:COLD:REFUSED:withheld-secret` (≠ requested `keystate`) | 0 | n/a | yes | yes |
| 6-recover (plain, on P) | 0 | 33 | none | 0 | 0 / 0 | yes | n/a |
| 7-gate-normal, 8-gate-recover (`test_system.py`) | 0 (gate) | 33 / 33 | none | 0 | 0 / 0 on recover | yes on recover | n/a |

"Plaintext lines" counts serial lines that contain `PLAINTEXT`, other than the
`SYSTEM:COLD_PLAINTEXT_REPLIES:00000000000000000000` line. That line is not a measured counter.
The kernel prints it as a constant zero (`kernel/src/m2/m3/system.rs`), and only after its
`require` has confirmed zero Key Service seal and unseal successes (KEY words 224 and 232) and
exactly one authentication failure (word 240). The no-plaintext evidence therefore comes from
that kernel check plus the absence of any `PLAINTEXT` or `SYSTEM:RECORD_` line. The harness
check that the value is zero cannot fail on its own.

`tools/test_system.py` completed with `passed: true` (`system-gate.json`). Its host stage
(`test_system_*.py` and `cargo +1.100.0 test --locked -p nieuw-os-sdk-runtime`) exited 0. The
final rebuilt normal kernel equals the tested one (`42d44a0c…`).

## What the harness now enforces (`tools/run_system.py`)

Known limits, kept open (independent review, low severity):
- The refusal label comes from the compile-time `NOS_SYSTEM_COLD_CASE`. `keystate` and
  `withheld-secret` both refuse in the INIT slot, and their serial logs differ only in that label.
- No run boots a mutating-case kernel against the unmutated parent image, so it is not shown
  natively that the refusal depends on the host mutation rather than on the compiled case.
- The harness does not assert that the mutated copy differs from the parent. This fails closed:
  an unmutated copy would restore successfully and break the expected Auth result. The recorded
  hashes do differ.

- `evaluate_cold` requires exactly one `SYSTEM:COLD:` line, and it must equal the requested
  case. It also requires raw exit 33, no timeout, `DISK_WRITES` and `DISK_FLUSHES` zero (and the constant
  `COLD_PLAINTEXT_REPLIES` line to read zero), and the boot and manifest markers.
- It fails closed if a cold log contains any `SYSTEM:CASE:`, `M1:PASS`, `SYSTEM:BROKERS:`,
  `SYSTEM:CLEANUP:`, `SYSTEM:RECORD_`, `SYSTEM:POWER:` or `SYSTEM:KEYSTATE_` marker, or any
  `FAIL`, `PANIC` or plaintext marker.
- A cold run hashes the parent normal image before the copy is mutated and again after the guest
  exits, and fails if the two hashes differ. It also fails if the runner's
  `NOS_SYSTEM_COLD_CASE` differs from `--cold-case`. That env check covers the runner only; the
  binding between kernel and case is the exact refusal marker, which run 5 exercises.
- The mutated copy is attached read-only, and its before and after hashes must match.

## Mutations (host side, `cold_image`)

- `keystate`: flips one AEAD byte (offset 56) in key-state object 8 and repairs the
  non-cryptographic volume checksums.
- `record-tag`: flips the last byte (the AEAD tag) of the sealed record (object 1).
- `swapped-blocks`: exchanges the two 64-byte ciphertext halves inside the record's single
  chunk.
- `withheld-secret`: leaves the image unchanged. The guest substitutes a zero secret.

## Open items (not proven here)

- **Old-generation replay: UNPROTECTED, open boundary.** Every report records
  `cold_antirollback.status = "UNPROTECTED"`. The reason is that cold restore uses
  `minimum_generation 0`. The older A/B slot (generation 2) has no key-state object. A stale
  image at the same generation with the same `VOLUME_ID` is accepted. Replay is not a
  `COLD_CASES` case and is not reported as a pass. No native stale-image observation was run.
- **True chunk swap is blocked.** A swap across multiple chunks or across objects is not
  possible within this task's write set: the sealed record occupies one chunk. The
  `swapped-blocks` case covers only reordering inside a single chunk.
- The rollback anchor needs an owner ADR and is out of scope. Cold anti-rollback is not claimed.
- No physical hardware (QEMU TCG only). No external or human audit.
