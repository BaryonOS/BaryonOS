# S09 — Integration request: host CI for `destination-core`

**Addressed to:** the Integration Manager. **From:** the S09 workstream, PR
#413 (branch `claude/s09-destination-core`, head `4464d5d4`, stacked on
#403). **Status:** open. The S09 workstream does not change protected
`.github/workflows/` files (project-owner instruction, 19 November 2026).

## Problem

PR #413 adds the workspace member `services/destination` (crate
`destination-core`, route step R2 of `S09-DESTINATION-SERVICE.md`). The
`host-quality` job in `ci.yml` lists root workspace crates explicitly for
tests (with coverage) and Clippy. Without this change, hosted CI runs only
`cargo fmt --all -- --check` on the crate. Its 57 host tests and its Clippy
gate therefore run only locally. Nothing fails today; the gap is missing
coverage, not a red check.

## The two lines

| File | Line on `main` `035b44e5` | Change |
|---|---|---|
| `.github/workflows/ci.yml` | 129 (`cargo +1.100.0 llvm-cov --locked -p boot-protocol … -p update-tool --cobertura …`) | add `-p destination-core` after `-p volume-core` |
| `.github/workflows/ci.yml` | 130 (`cargo +1.100.0 clippy --locked -p boot-protocol … -p update-tool --all-targets -- -D warnings`) | add `-p destination-core` after `-p volume-core` |

## Why these values

- The crate is a root workspace member (root `Cargo.toml` members line in
  #413), so it belongs in the same `-p` lists as `volume-core`. That matches
  `services/AGENTS.md`: workspace members use
  `cargo +1.90.0 test --locked -p <crate>`.
- It builds with the CI toolchain 1.100.0 (`host-quality` on `main`) and with
  1.90.0, and has no external dependency;
  `capability-core` is a path dev-dependency.

## Local evidence at the head that fixes the 1.100.0 Clippy finding

| Command | Exit |
|---|---|
| `cargo +1.100.0 test --locked -p destination-core` | 0 (57 passed) |
| `cargo +1.100.0 clippy --locked -p destination-core --all-targets -- -D warnings` | 0 |
| `cargo +1.100.0 fmt --all -- --check` | 0 |
| `cargo +1.100.0 check --locked -p destination-core --target x86_64-unknown-none` | 0 |

The same four commands also exit 0 on 1.90.0. `cargo llvm-cov` with the added
`-p` was not run locally. The Integration Manager found a 1.100.0-only
`clippy::needless_range_loop` at `4464d5d4` (`service.rs:406`); this head fixes
it.

## Verification after the change

`host-quality` passes on #413 with `destination-core` in the test and Clippy
lines. `python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py'`
and `actionlint -shellcheck= -pyflakes=` pass, as `AGENTS.md` requires for
workflow changes.

## Ordering and rollback

The change only makes sense once #413's crate is on the target branch. If it
lands before #413 merges, `-p destination-core` fails on `main` with "package
not found". Apply it to `main` together with, or after, #413. To roll back,
remove the two `-p destination-core` arguments.
