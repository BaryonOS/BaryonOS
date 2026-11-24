# Native settings over the volume service (U05-N1)

This `no_std` example persists the typed settings of `sdk/settings-core`
through the M6e v6 volume service. Two app instances run, one per volume
domain. Each holds only a send/reply endpoint pair to the service and receives
no block, file, network or administrative right. This is a bounded native
persistence path, not U05 acceptance. It has no UI, account scope, per-user
encryption or rollback protection.

## Adapter (`src/lib.rs`)

`VolumeMedia` implements `SettingsMedia`. Settings slot `i` is volume object
`i` of the app's own domain: objects 0/1 for domain 0 and 8/9 for domain 1,
named `settings-a`/`settings-b`. The domain and epoch come from kernel startup
words, never from the app.

- **read:** STAT, then READ of the current version. The reported length is
  untrusted: anything other than 96 bytes is an I/O error. The bytes must also
  match the FNV sum that STAT reported. `NotFound` reads as an all-zero slot.
  Masking consequence: if both objects are deleted, the store looks blank and
  `initialize` becomes possible again. Deletion requires the domain's own
  endpoint, so this is not a cross-domain path, but this adapter does not
  detect it.
- **write:** STAT for the expected revision, then BEGIN, one 96-byte APPEND
  and COMMIT against that revision. Any failure before an acknowledged COMMIT
  sends a best-effort ABORT and poisons the adapter. A reply whose outcome is
  `Unknown` is never resent.
- **flush:** Ok exactly when no write has failed or stayed uncertain. The
  service flushes before it acknowledges a COMMIT. A poisoned adapter refuses
  every later write and flush. `settings-core` therefore retires its state and
  never claims a commit that is not acknowledged.

Report words written by the app: 480 = generation, 488 = canonical 8-byte
settings encoding as a little-endian `u64`, 496 = status, then 40 = 1 and exit 0.

| Status (word 496) | `settings_core::Error` |
|---|---|
| 0 | success |
| 1 | InvalidValue |
| 2 | RecoveryRequired |
| 3 | NotInitialized |
| 4 | NotBlank |
| 5 | Corrupt |
| 6 | UnsupportedVersion |
| 7 | NoPrevious |
| 8 | GenerationExhausted |
| 9 | Media(Io) |
| 10 | Media(Full) |
| 11 | ReadbackMismatch |

## Native cases

Both cases first repeat the volume-docs block-denial probes. Kernel startup
word 112 selects the case.

- **23 `settings` (writable).** Recover. On blank storage, `initialize(S1)`
  then `set(S2)`. On an existing record, verify S2, then `revert` (a new
  generation holding S1). Then check that `Settings::new` refuses scale 301
  and 74 without changing the generation, and that a fresh store recovers the
  same state. The domain-1 instance also checks that STAT of domain-0 object
  0 returns `Access`.
- **24 `settings-recover` (read-only mount).** Recover only. A refusal is
  reported in word 496 and is not an app failure. The kernel mounts the image
  read-only and fails on any broker write or flush.

Per-domain values: domain 0 uses S1 = Light/100 %/motion and
S2 = Dark/150 %/reduced motion. Domain 1 uses S1 = System/125 %/motion and
S2 = Light/200 %/reduced motion.

## Runner and oracle

`tools/run_settings.py` performs six cold QEMU boots on one image chain.
`tests/test_settings_oracle.py` checks the NOSSET01 decoder, the recovery
selection and the damage helpers on synthetic images without QEMU. The damage
helpers edit settings bytes and then repair the volume's FNV sums in both table
copies, so settings-core's SHA-256 and version checks are what is exercised.

Step 4 differs from the original brief. The brief expected the newest-slot
fallback to give gen 1 = S1, but step 3's revert has already overwritten gen 1
(slot 0 holds gen 3 = S1 and slot 1 holds gen 2 = S2). The reachable and
checked result is therefore gen 2 = S2.

## Commands

```sh
cargo +1.100.0 test --locked --manifest-path sdk/examples/settings-volume/Cargo.toml --target-dir "$PWD/target"
cargo +1.100.0 clippy --locked --manifest-path sdk/examples/settings-volume/Cargo.toml --target-dir "$PWD/target" --all-targets -- -D warnings
cargo +1.100.0 fmt --manifest-path sdk/examples/settings-volume/Cargo.toml --all -- --check
python3 -B sdk/examples/settings-volume/build.py            # x86_64-unknown-none ELF
python3 -B sdk/examples/settings-volume/build.py clippy
NOS_BUILD_DIR=build/s python3 -B tools/build.py --kernel-feature test-volume-settings
NOS_BUILD_DIR=build/r python3 -B tools/build.py --kernel-feature test-volume-settings-recover
python3 -B tools/run_settings.py --settings-build build/s --recover-build build/r
```

The stages are deliberately absent from `tools/test_volume.py` PLAN, so the
workflow stage-count pins stay unchanged. No CI workflow runs this example yet.

## Shared wiring

The shared wiring lives in `kernel/Cargo.toml` (features
`test-volume-settings` and `test-volume-settings-recover`) and in
`kernel/src/m2/m3/volume.rs`. In that file, cases 23/24 are added to `case()`,
24 is added to `read_only()`, and the markers `SETTINGS`/`SETTINGS_RECOVER`
are new. Slot B's words 488/496 are printed only for cases 23/24.
`tools/build.py` adds `VOLUME_SETTINGS_CASES` and the settings-app ELF
override. `run_settings.py` refuses to run and names the missing edit when any
part is absent.
