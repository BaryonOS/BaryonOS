# S09 native archive-ustar stage: verification run 14 (15 November 2026)

**Scope.** This file records one verification pass (runner task S09-VERIFY, attempt 1) over the native `archive-ustar` stage of the M6e/S06 volume gate. That stage writes the approved D6(a) user export, POSIX ustar with pax `path` records, to the volume's fixture range from inside QEMU, and `tools/run_volume.py` checks the bytes with Python's `tarfile` reader. The file contains local host checks and one native gate run with source fingerprints. It is not S09 acceptance, and the criterion stays open.
- Design, refusals and limits: [S09-BACKUP-ARCHIVE.md](../../implementation/S09-BACKUP-ARCHIVE.md), section "User export to ustar".
- Previous native measurement of this stage: the same specification, gate fingerprint `0af1c15e…`. That run was not captured.

**Source.** Branch `feat/s09-ustar-name-check`, tested commit `18da1eb9cce655ad2798d8e8601faced3d37c092`. The source-v3 fingerprint (`common.source_hash()`, which excludes `docs/`) is `36cab7938fbbac4e8dbbc71e66e59f5c95a636651d69f5ca2fd1b378aacd54b8`. It was recorded before the gate and again after it, and the gate summary's `source_sha256` has the same value. Nothing was edited while the gate ran.

**Host checks (Rust 1.90.0, local, not CI).** Durations are wall-clock seconds. Durations under one second come from Cargo's incremental cache. A cached Clippy result replays the stored diagnostics, so the exit status still reflects the lints. `volume-core` Clippy and the `no_std` build were repeated after an mtime-only touch, with no content change, to force a fresh check.

| Command | Exit | s | Result |
|---|---|---|---|
| `cargo +1.90.0 test --locked -p volume-core` | 0 | 22.5 | PASS: 93 passed (lib 1, archive 38, crash 5, history 8, independent_copy 3, service 30, uploads 8) |
| `cargo +1.90.0 clippy --locked -p volume-core --all-targets -- -D warnings` | 0 | 0.2 / 0.4 fresh | PASS |
| `cargo +1.90.0 fmt -p volume-core -- --check` | 0 | 0.2 | PASS |
| `cargo +1.90.0 build --locked -p volume-core --target x86_64-unknown-none` | 0 | 0.1 / 0.2 fresh | PASS: freestanding `no_std` build |
| `cargo +1.90.0 test --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | 0 | 0.1 | PASS: 37 tests plus 3 doc tests |
| `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 0 | 1.2 | PASS: 53 tests |
| `cargo +1.90.0 fmt --all -- --check` | 0 | 0.8 | PASS |
| `cargo +1.90.0 clippy --manifest-path sdk/Cargo.toml --locked --all-targets -- -D warnings` (as in `ci.yml`) | 0 | 0.6 | PASS |
| `cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all -- --check` | 0 | 0.8 | PASS |
| `cargo +1.90.0 clippy --offline --locked --release --manifest-path sdk/examples/volume-docs/Cargo.toml --target x86_64-unknown-none --features native --bins -- -D warnings` | 0 | 0.4 | PASS |
| same, with `--features native,archive-portal` | **101** | 0.2 | **FAILED**: see below |
| `python3 -B tools/preflight.py --strict` | 0 | 0.8 | PASS |
| `cargo +1.90.0 clippy --locked --release -p os-kernel --target x86_64-unknown-none --target-dir build/s09-verify-kclippy --features test-volume-archive-ustar -- -D warnings` | **101** | 0.6 | **FAILED**: see below |

Archive-portal Clippy, FAILED (exit 101):
- One lint, `clippy::manual_is_multiple_of`, at `sdk/examples/volume-docs/src/app.rs:1056`: `check(length > 0 && length % SECTOR as u64 == 0, 837);`.
- The line was introduced in `8f4c7f3e`, the native archive-ustar stage commit. The fix is `length.is_multiple_of(SECTOR as u64)`.
- The command follows the precedent in [NATIVE-20261113.md](NATIVE-20261113.md). No workflow runs it, so CI does not catch this lint.
- The lint was not fixed here. This pass may change only evidence, and a source change would change the fingerprint and void the gate run below.

Kernel Clippy with the stage feature, FAILED (exit 101):
- The build ran with `NOS_VOLUME_APP_ELF` set to the gate's portal app and `NOS_VOLUME_SERVICE_ELF` set to the gate's service, as `tools/build.py` passes them. It reported 14 errors in `kernel/src/main.rs`, `serial.rs`, `m2/lock.rs`, `m2/paging.rs`, `m2/m3.rs` and `m2/m3/volume.rs`.
- Both `volume.rs` locations, lines 457 and 1113, date from `092ee525` (#47, 29 October 2026). This branch changes only `kernel/Cargo.toml` and `kernel/src/m2/m3/volume.rs` against `main`, and none of the 14 locations is in a changed hunk.
- No workflow runs kernel Clippy. The errors are older kernel debt, not caused by this stage.

**Native volume gate, run 14.**
- Command: `build/s09-ustar-notes/runner/heavy.sh python3 -B tools/test_volume.py`, a host-wide lock that allows one heavy gate at a time.
- Result: exit 0, `passed=true`, all 15 QEMU stages passed. Wall time was 164 s, of which the QEMU stages took 109.2 s.
- Environment: QEMU 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3), TCG, one CPU, local host. This was not a CI run.
- Stages in plan order, with raw QEMU exit and duration. A raw exit of 33 is a guest pass; -9 is the planned power cut.

| # | Stage | Raw | s | | # | Stage | Raw | s |
|---|---|---|---|---|---|---|---|---|
| 0 | normal | 33 | 41.8 | | 8 | cut-data | -9 | 2.9 |
| 1 | recover | 33 | 24.5 | | 9 | recover-cut | 33 | 2.2 |
| 2 | archive-export | 33 | 3.9 | | 10 | cut-table | -9 | 3.1 |
| 3 | archive-restore | 33 | 5.3 | | 11 | recover-cut | 33 | 2.2 |
| 4 | archive-corrupt | 33 | 3.6 | | 12 | cut-txn | -9 | 2.8 |
| 5 | archive-conflict | 33 | 4.9 | | 13 | recover-cut | 33 | 2.2 |
| 6 | archive-full | 33 | 3.5 | | 14 | corrupt | 33 | 1.8 |
| 7 | archive-ustar | 33 | 4.3 | | | | | |

- Build warnings in the gate log: `hex` is never used (`kernel/src/serial.rs:184`), and `level`, `owner` and `lock_pair` are never used (`kernel/src/m2/lock.rs:146`). The log has none in `kernel/src/m2/m3/volume.rs`.

**Archive-ustar stage measurements** (`7-archive-ustar/result.json`):
- kernel feature `test-volume-archive-ustar`; parent case `archive-export`.
- Guest counters:
  - volume archive path: 1,440 reads, 368 writes and 2 flushes, and `USTAR_BYTES` 96,256;
  - broker: 3,552 reads, 368 writes and 2 flushes;
  - service: 358 sectors read, generation 84;
  - 6 app device denials and 6 service rejections.
- Guest markers: `VOLUME:CASE:ARCHIVE_USTAR:PASS`, `VOLUME:RESOURCES:PASS` and `VOLUME:PASS`.
- Host oracle (`tarfile`):
  - ustar stream: 96,256 bytes, sha256 `13bdbe8dea0bbe10a0023d5e178c612369adc503bf0840fcddf6aeee4e5bbaea`;
  - members: `Privé/Notitie ✓`, `Werk/Leeg`, `Concept`, `Herstart` and `Archief`;
  - archive: 91,801 bytes, sha256 `d75aa5c812a33b1c1b29447390e20311fb1406ee38e7b7eed0ec8b2d35e3bde1`, I/O `[1440, 368, 2]`.
- These figures equal the earlier `0af1c15e…` measurement in the specification.
- Images:
  - kernel `369f21616af101ad1a3291fc350868440e1c647e8f82ffe2e688297d51d24ec5`;
  - portal app `73a34262b3ae634f6274cb8bbf946f56585eb9914ef9373e17192daa793d3fc2`;
  - service `f26097715a8189ffe3d1d97f4b7f7f049e5d99ff480fe00e7c68a1c094760075`.
- Disk: before `19e2fdb3081ed07d54e5f42d152919606aac51ac07cd9dd079769ec011146bb3`, after `8ca0687d95bdec36d317e6268bf47df9574c4a8f192048c7d83c0ded30fcf762`.

**Captured evidence.** [run14-20261115/](run14-20261115/) was written by `tools/capture_volume.py` and holds one directory per stage (`result.json`, `serial.txt`, `stderr.txt`) and `volume-gate.json`. `SHA256SUMS` covers 46 files, and `sha256sum -c` exited 0. `volume-gate.json` has sha256 `c903b97e88769d91761d21006fbdd715062cea06ee92984c08d3b799c8c27304`. Disk images are not stored.
- The serial logs use mixed CRLF/LF line endings. The first evidence commit, `5376afc1`, stored them under the repository-wide `text=auto eol=lf` rule. That rule rewrote their line endings, so `sha256sum -c` against the committed files exited 1.
- [.gitattributes](.gitattributes) now marks `run14-20261115/**/serial.txt` as `-text`, following the m6b rule in the root `.gitattributes`. The follow-up commit stores the captured bytes unchanged.
- Run 13's committed serial logs ([run13-20261114/](run13-20261114/)) fail their own `SHA256SUMS` in the same way, with exit 1. They were left untouched here. The byte-exact originals exist only in this worktree's working copy.

**Other commands.** These ran from the worktree root. Durations were not recorded for any of them except the gate.

| Command | Exit | s |
|---|---|---|
| `python3 -B -c "import sys; sys.path.insert(0,'tools'); from common import source_hash; print(source_hash())"` before the gate | 0 | not recorded |
| `build/s09-ustar-notes/runner/heavy.sh python3 -B tools/test_volume.py` | 0 | 164 |
| the same fingerprint command after the gate | 0 | not recorded |
| `python3 -B tools/capture_volume.py --destination docs/verification/s09-archive/run14-20261115` | 0 | not recorded |
| `sha256sum -c SHA256SUMS --quiet` in the run folder, working copy | 0 | not recorded |
| the same, on the files committed in `5376afc1` (`git archive`) | 1 | not recorded |
| the same, on the files committed in `97dd1b9d` (`git archive`) | 0 | not recorded |

**Not run.**
- KVM, real hardware and CI on this head;
- extraction on Windows or macOS, and bsdtar or Windows `tar` as readers;
- byte-for-byte comparison with an independent pax writer;
- an independent human audit;
- updates to `docs/PROJECT-STATE.md` and the progress register, which are outside this pass's write scope.

Not fixed: the two Clippy failures above.
