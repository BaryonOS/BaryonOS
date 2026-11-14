# Issue #41 — explicit TEST-policy compiler guards

Date: 14 November 2026. Tested source:
`08cf002b9a4a23ffab583a3e6d2663ec33af2dcf`, based on
`2009fcf79a6f2f6d542a341e43e6d568cd8932c9`.
Source-v3 fingerprint:
`ad89c731af95576955d0e9e890da3b983acac28270f1d5f5606a0ec435122611`.

## Requirement and implementation

ADR-B03K2 requires explicit TEST features; an absent production signing policy
must not fall back to public TEST anchors. Existing Python build-profile tests
exercise wrapper rejection. The update gate now also executes Cargo against the
Rust guards in the default loader, SMP/legacy loader and native update service.

Each positive TEST configuration must compile before its negative check runs.
A negative passes only with Cargo exit 101 and the exact intended error from a
JSON `compiler-message` for the expected target. Unrelated compiler errors,
warnings, stderr text and a missing toolchain cannot satisfy the guard. The
source fingerprint must remain unchanged throughout the checks.

The normal `tools/test_update.py` gate executes these checks before its existing
host and 18-stage native matrix, retaining their results in `update-gate.json`.
`--policy-guards-only` executes just the compiler checks and writes
`update-policy-guards.json`; it does not overwrite an existing native gate report
or claim a native pass.

## Executed checks

`python3 -B -m unittest discover -s tests -p 'test_update_*.py'`:
**exit 0, 15 tests**. The added oracle test uses synthetic Cargo records;
it rejects failed controls, unrelated errors, warning/stderr/plain-text
imitations, a different target, a nonmatching diagnostic, unexpected exit
codes and a changed source fingerprint. These mock results are not compilation
evidence.

`python3 -B tools/test_update.py --policy-guards-only`:
**exit 0, 9.888 seconds**, with actual Rust 1.90.0 Cargo invocations:

| Target | Explicit TEST features | Positive exit | Missing-policy features | Negative exit |
|---|---|---:|---|---:|
| `os-loader`, UEFI | `signed-boot-test` | 0 | `signed-boot` | 101 |
| `os-loader`, UEFI legacy | `signed-boot-test,smp-handoff` | 0 | `signed-boot,smp-handoff` | 101 |
| `update-service`, freestanding x86-64 | `native,test-signing-anchors` | 0 | `native` | 101 |

All six use `cargo +1.90.0 check --locked --no-default-features
--message-format=json`. Loader checks select `-p os-loader --target
x86_64-unknown-uefi`; service checks select `--manifest-path sdk/Cargo.toml
-p nieuw-os-update-example --bin update-service --target x86_64-unknown-none`.
The exact commands and compiler output are retained in the JSON report.
The run used an isolated `CARGO_TARGET_DIR` and `CARGO_BUILD_JOBS=2`.
`git diff --check` also returned exit 0.

## Retained evidence

Local logs and the 240,115-byte compiler report are under
`build/issue41-guards/` in the task worktree; binaries and caches are not committed.

| Evidence | SHA-256 |
|---|---|
| `update-policy-guards.json` | `7896543ef1e93a9d86cc1a54672755cfd938e6f8ba59488722383c6855f78b48` |
| `actual-guards.log` | `e8178cd853a4da7443c7610ae0f57eaa41b51d5396b48849c6082f2e315ae8d3` |
| `host.log` | `4c05ac9797a0cf41ea5b35121c857e3e82c290238a6f117b0764c908f5009608` |
| `tools/test_update.py` | `6156bf19a45485281df8a8e9718bb8e948542a4da4cd974c830ca906c4552f8b` |
| `tests/test_update_harness.py` | `352ce503aa2cda713e211f909818b0018da72e6e23279a57f175b9d7df5b60d3` |

## Evidence boundary

These are host-oracle and cross-compilation results. The full 18-stage native
update gate, QEMU smoke and physical-device checks were not run for this scoped
addition. Integrated-source verification remains separate. No production
policy, key, signing authority, dependency, manifest format or capability was
introduced. D1–D7 in the production-policy proposal remain unapproved; #41 stays
open and no acceptance points are awarded by these checks.
