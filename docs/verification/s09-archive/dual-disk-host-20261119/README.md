# S09 independent backup device: host model and native regression (19 November 2026)

- Base: main `8f9708663a8c7b94db3d35613047bcbad3a308c3`.
- Tested commit: `c4ac9ec161b06b4c8196b4b910e6de77e75f6996`. It changes tests only:
  `services/volume/tests/archive.rs`.
- Environment: Linux host, Rust 1.100.0, QEMU 10.2.1, local KVM-less run under
  the repository defaults. Raw logs remain local under the worktree's ignored `build/`
  (`build/gate-c4ac9ec1.log`, `build/volume-gate.json`, `build/run/`).

| Command | Source | Result |
|---|---|---|
| `python3 -B tools/test_volume.py` | `8f970866` (baseline) | exit 0; passed=true; 15/15 stages; fingerprint `60c6b1991863a7753a326b5fd07568cd7afa44f40fb461e384839c1eb0d0fb6a`; summary sha256 `a069514e…` |
| `cargo +1.100.0 test --locked -p volume-core --test archive` | `c4ac9ec1` | exit 0; 43 passed, 5 of them new |
| `cargo +1.100.0 clippy --locked -p volume-core --all-targets -- -D warnings` | `c4ac9ec1` | exit 0 |
| `cargo +1.100.0 fmt --all -- --check` | `c4ac9ec1` | exit 0 |
| Mutation M1: `DiskSink::finish` without the flush | `c4ac9ec1` plus the mutation, reverted | 4 tests fail (3 new, 1 existing) |
| Mutation M2: `DiskSink::finish` ignoring a flush error | `c4ac9ec1` plus the mutation, reverted | 2 tests fail, both new; no earlier test catches it |
| `python3 -B tools/test_volume.py` (host suites and 15 QEMU stages) | `c4ac9ec1` | exit 0; passed=true; 15/15 stages; fingerprint `c75b21fa815b35748fb4c00d2e2131a63ec635b08595c312e0c965f122a1b558`; summary sha256 `d7178ab8…` |
| `python3 -B tools/progress.py --check` | `c4ac9ec1` | exit 0; 41/100 consistent |

Limits: the five new tests are host tests. The QEMU stages are the existing
single-disk S09 stages, re-run as a regression on current main. No native
stage uses a second disk yet; see
[S09-DUAL-DISK-FIXTURE-REQUEST.md](../../../implementation/S09-DUAL-DISK-FIXTURE-REQUEST.md).
This is not physical-hardware or human-audit evidence.
