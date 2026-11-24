# S09 phase 3 slice A: v2-aware `DiskSource` (verification record)

**What was checked:** commit `262cb0f8` on branch `claude/s09-l2-slice-a`
(stacked on #459, `ee22e63a`). The design is section 5.1 of
[S09-L2-NATIVE-EXPORT.md](../../implementation/S09-L2-NATIVE-EXPORT.md).

**Evidence level:** host tests, a host mutation check and the existing native
volume gate on a local QEMU/KVM host. This is not hosted CI, not a new native
stage and not a human audit.

## Change

- `DiskSource` computes the stored length per format version:
  - v1: unchanged;
  - v2: adds the version records, and is bounded by `MAX_ARCHIVE_V2` (the
    range-adapter rule of `S09-ARCHIVE-V2.md` section 4.4) and by the range;
  - any other version: `Corrupt` after reading the header sector only.
- `DiskSource` owns its disk. A new `impl<D: Disk> Disk for &mut D` keeps
  borrowing callers working. One caller in the v1 portal (`verified_restore`)
  had to reborrow its disk.

## Commands and results

All commands were run in the slice A worktree on 23 November 2026.

| Command | Exit | Result |
|---|---|---|
| `cargo +1.100.0 fmt --all -- --check` | 0 | formatted |
| `cargo +1.100.0 clippy --locked --offline -p volume-core --all-targets -- -D warnings` | 0 | no warnings |
| `cargo +1.100.0 test --locked --offline -p volume-core` | 0 | 172 passed, 0 failed |
| `python3 -B tools/mutate_volume.py` | 0 | `passed=True`: 100 of 100 mutants caught, both baselines exit 0, source `be398087…` |
| `python3 -B sdk/examples/volume-docs/build.py` and `--archive-portal` | 0, 0 | sizes below |
| `NOS_QEMU_MACHINE=pc-q35-8.2 NOS_QEMU_ACCEL=kvm python3 -B tools/test_volume.py` | 0 | `passed=True`, 26 stages, source `be398087…` |

The three new mutants and the tests that caught them:

| Mutant | Caught by |
|---|---|
| `disk-source-v2-version-records` (v2 length without version records) | `disk_source_reads_every_v2_fixture_exactly_as_stored`, `restore_native_l2_reads_an_archive_range_through_owned_sources` |
| `disk-source-v2-as-unknown` (v2 treated as an unknown version) | all three new `archive_v2` tests |
| `disk-source-v2-bound` (no `MAX_ARCHIVE_V2` bound) | `disk_source_refuses_a_short_range_and_an_unknown_version_after_one_read` |

Portal sizes, measured with `size -A` before (at `ee22e63a`) and after.
Bounds come from `sdk/examples/volume-docs/linker.ld`:

| Binary | `.text` before → after (bound 65 536) | `.rodata` before → after (bound 32 768) | `.data` |
|---|---|---|---|
| `volume-app`, archive portal | 49 867 → 50 043 | 8 704 → 8 672 | 1 328 → 1 328 |
| `volume-app`, default | 34 581 → 34 613 | 4 384 → 4 360 | 1 088 → 1 088 |
| `volume-service` | 50 021 → 50 053 | 21 192 → 21 168 | 1 096 → 1 096 |

## Independent AI review

A read-only workflow ran three reviewer lenses (reader correctness and trust
boundary, test and mutant adequacy, compatibility and spec consistency) and
one adversarial verifier. It reviewed the uncommitted diff before the fixes
below. It was not a human audit.

| Finding | Verifier | Resolution |
|---|---|---|
| The design text said `DiskSource` adds no `MAX_ARCHIVE_V2` bound, but the code had one with no test or mutant (minor, reported by all three lenses) | real | The bound is kept, because `S09-ARCHIVE-V2.md` section 4.4 requires it of a range adapter. Section 5.1 now says so. A raw-reader test and the `disk-source-v2-bound` mutant pin it. |
| Mutant `disk-source-v2-version-records` did not compile (blocker) | refuted | It had already been fixed in the tree the verifier read. The final run caught it. |
| The spec named `impl<D: Disk>`, the code had `+ ?Sized` (nit) | — | The code now matches the spec (`impl<D: Disk>`). |
| `S09-ARCHIVE-V2.md` still listed the v2 range adapter as open (nit) | — | Removed from the open list. |

The reviewers also verified that:
- the v2 length formula equals `export_v2`'s output and the Python
  `stored_length_v2`;
- no read leaves `[0, sectors)`;
- the cached length and the cached sector stay correct after an error;
- the only production consumers are `walk` and `walk_v2` and their callers;
- the blanket impl has no coherence conflict in the repository.

## Not covered

- No new native stage; the v2 read path is exercised natively only in
  slice D.
- Hosted CI has not run on this head.
- The stack depth of the portals was not measured (slice B).
- Human audit: not done.
