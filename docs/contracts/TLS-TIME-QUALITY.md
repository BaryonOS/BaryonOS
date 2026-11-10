# TLS-TIME-QUALITY-1: explicit rejection of unsuitable time

## Objective, owner and scope

TL-CORE, source base `3f3f99efd18ef70d6ca5737a5fd668a520f537e6`
(includes main `0ade801`), branch `simon/tls-time-quality-1`.
Agreed with TL-NET and TL-SEC: only
`sdk/runtime/src/tls_time.rs`, direct `sdk/runtime/tests/tls_time.rs`
and this contract. Release additionally authorised only the export
`pub mod tls_time;` in `sdk/runtime/src/lib.rs`; NET and apps were informed
directly. No other clock, kernel, syscall, TLS-stack, certificate, key or
anchor changes.

This is a direct dependency of FR-N03/FR-S03, not complete TLS, time-sync,
product or rubric acceptance. The RTC interface and ABI 1.9 syscall already
exist and are not reimplemented. The
[RTC contract](../implementation/RTC-WALLTIME.md) remains authoritative.

## Invariant and interface

A readable UTC value does not authorise approval of certificate validity.
The existing SDK knows only `WallClockQuality::UnauthenticatedRtc`.
A stable, plausible, increasing or minimum-stored RTC value does not change
that quality. Boot-monotonic ticks are not a calendar date. KEY_EXPIRED in
update/key policy is a release-version window, not a calendar clock.

The pure allocator-free `no_std` module contains:

```rust,ignore
pub fn certificate_time(
    reading: Option<WallClockReading>,
) -> Result<CertificateTime, TimeUnavailable>;
```

`None` returns `MissingSource`; every current RTC reading returns
`UnauthenticatedRtc`. **No current public input can produce `Ok`.**
The exhaustive match on clock quality requires an explicit code decision
if a new quality is added later.

`CertificateTime` has a private `u64` seconds field and a
`unix_seconds()` getter for future verifier integration. No public
constructor, `From`, `Default`, deserialiser, trust boolean or accessible
numeric conversion. Safe callers therefore cannot promote a timestamp or
`WallClockReading` to this type. The module forbids its own unsafe code.
A type/module constrains safe API users, not a malicious process with
unsafe code or kernel access; actual authority must later be enforced
at the authorised provider/process boundary.

The future TLS consumer must visibly return this error before attempting
date verification. It must not fall back to host time, build time, a fixed
test timestamp, a network date or monotonic ticks. No skipping certificate
dates, accept-all verifier, TOFU or automatic alternative transport connection
as a consequence of this error. This step does not implement a TLS consumer
and therefore does not prove that an actual handshake already uses this boundary.

## Conditions for a future successful source

The following conditions remain **OPEN**, not implicitly resolved:

- Who may issue calendar time, and how is that authority enforced per
  process? A public `trusted: true` or caller-provided enum is insufficient.
- Which maintained standard and actual cryptographic verification bind
  source identity and time content? No custom time-signing protocol.
- How are freshness, uncertainty, maximum age, forward jumps, time between
  receipt and verification, and suspend/restart handled?
- How are rollback, replay and stale samples detected across restarts,
  using which authenticated persistent anchor source and failure transactions?
  An update-version floor or the current RTC floor is not calendar evidence.
- How does the TLS verifier bind the validity boundaries of the complete
  certificate chain to the permitted uncertainty margin? The existing TLS
  port is reused; no second TLS stack.

As long as that source does not exist, unknown, unavailable, stale,
replayed or rolled-back time paths also produce no certificate time:
there is no public success constructor. A future provider must first
represent and negatively test such states explicitly; this module does
not claim to have detected them cryptographically.

## Acceptance and evidence boundary

Focused tests cover an absent source, RTC at negative/epoch/plausible/extreme
`i64` values, stationary/increasing/decreasing sequences and an attempted
RTC fallback. Compile-fail doctests check impossible safe conversion from
RTC, public struct construction and monotonic numbers.
Further gates: existing runtime host tests/doctests, `no_std` target check,
rustfmt, clippy and diff checking under the recorded verification execution.

No new dependency, I/O, clock setting, key persistence or network right.
These pure core tests alone do not establish native TLS, QEMU, Windows,
physical or trusted-time acceptance. The manager maintains the shared
queue/status/rubric; proposal **34% -> 34%**, unless the current rubric has
since acquired a different source-bound baseline score.

## Evidence executed on 30 October 2026

Exact v3 source fingerprint of the modified code:

```text
15eac332d8044bcfb0053ebeb7dfc89d70c90d44d96201ab04e6f2d70cb4844c
```

The commands below were actually executed on this code, all **exit 0**.
The runtime export is the only changed line in an existing source file;
the other three files are new. No dependencies, manifests, lockfiles,
build tools or shared progress files were changed.

```sh
. .tools/nieuw-os/rtc-env.sh
export PATH="$PWD/.tools/nieuw-os:$PATH"
export CARGO_TARGET_DIR="$PWD/target/tls-time-quality-1/host"
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime
python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml
CARGO_TARGET_DIR="$PWD/target/tls-time-quality-1/native" \
  python3 -B tools/dev.py cargo +1.90.0 check --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime --lib --target x86_64-unknown-none
python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime -- --check
git diff --check
```

Rust/Cargo 1.90.0; runtime **27 host tests + 3 compile-fail doctests** passed,
including 4 new semantic time tests and 3 new compile-fail tests.
Complete SDK **81 host tests + the same 3 doctests** passed; that count
includes the runtime and is not added to it. Clippy produced no warnings;
`x86_64-unknown-none` target check passed, requiring no host libc/allocator.
This is a target type check, **not an executed native TLS handshake**.

Complete local logs and SHA-256:

| Log | SHA-256 |
|---|---|
| `/tmp/nieuw-os-tls-time-runtime-tests.log` | `5057c93480b0eb7fa77234f5d2feaf3f73ac44001bc5095c7abd9340ef8c5182` |
| `/tmp/nieuw-os-tls-time-sdk-tests.log` | `66faba2e3df8576e29761d34adc5c3f76b412d75cafdf03ef8a25a79c1a1dfe2` |
| `/tmp/nieuw-os-tls-time-clippy.log` | `841330560dc8011a07ed720344531b084b1b8e7d558510511ab7903f4e5b0f3c` |
| `/tmp/nieuw-os-tls-time-target.log` | `bd2750ef00c4822aa45aa0b0aafda459a894973b40abae988b21d8e41c0b00c3` |
| `/tmp/nieuw-os-tls-time-fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Two development commands initially failed and were not used as acceptance:
Cargo rejected `--test tls_time --doc` with exit 101 because those selectors
cannot be combined; the complete package test above includes both and passes.
Rustfmt rejected the initial position of the new module export (exit 1);
the single export is now in alphabetical order, and the recheck passes.
No code or test fallback.

The new code source has a different v3 from the previously tested Windows
test bundle. That bundle remains bound only to its original
`eabc27b`/`a1998a12...` source; this contract does not relabel that runtime
as tested TLS code. The native TLS consumer, actual trusted source,
freshness, persistent rollback anchor and final-image integration remain open.
