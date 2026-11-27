# B03-REL05 M1-HAUTH: unenrolled halt instead of re-provisioning (27 November 2026)

**Native TEST experiment, not acceptance evidence.** It removes the TEST
loader's re-provisioning after a TPM clear, which section 7.9 of
[B03-REL05-TRIAL-BOOT-TPM.md](../../implementation/B03-REL05-TRIAL-BOOT-TPM.md)
recorded as a violation of its first two binding conditions. It accepts no
criterion and grants no progress points; REL-05 stays open.

- Source: branch `claude/b03-rel05-hauth-unenrolled-20261127`, commit
  `54a8da66` merged with `main` `2d0d2d51` as `fda3e745` (local at the time
  of the run). `tools/common.source_hash()`
  `5b392b72791b7f95b9a22755e8dfa10696808b5a93b7c86447ef645bc15f28ed`; all
  nineteen `run_tpm.py` cases are bound to it. An earlier batch of the same
  nineteen cases on `54a8da66` alone (before the merge) also exited 0 and is
  superseded by this one.
- Profile and stack: QEMU 10.2.1, `pc-q35-10.0`, TCG, local Ubuntu OVMF
  2025.11 (`OVMF_CODE_4M.fd` copy `0d362a7d…`), swtpm 0.10.1.
- Binaries: the verify-only loader (`tpm-hauth-verify-test`) `BOOTX64.EFI`
  `5e7b72821d8e31e2ffc30b6c2d867d44ecfb17e463192cad66f59dc048a070d9`; the
  enrollment loader (`tpm-hauth-provision-test`, verify plus provisioning)
  `1ebbb6ea248d8e51f5a0ef42b021a80a8c6d504fa2aec6b2a78963f66757d452`.
  Unchanged: the probe loader `b231b4b9…` and the default loader `0065fdf6…`.
  `kernel.elf` is `8eb93c60…` (`absent`: `35775f1d…`); it changed only through
  the `main` merge. The reports' artifact binding passed on every boot.

## Change

- `boot/loader`: the TEST loader is split at compile time. The new feature
  `tpm-hauth-verify-test` contains the M1-HAUTH verify path. When the
  permanent properties show `ownerAuth` unset, it returns
  `TPM_UNENROLLED` before any TPM write. The common halt path then extends
  PCR 13 with `CLOSED` and prints `LOADER-HAUTH:HALT:PCR13:CLOSED`,
  `LOADER-HAUTH:FAIL:TPM_UNENROLLED:0000000000000000` and
  `LOADER-HAUTH:RECOVERY:REQUIRED`, and the loader exits with raw QEMU status
  35. The feature `tpm-hauth-provision-test` (verify plus `provision()`) keeps
  the BOUND provisioning; it is the TEST stand-in for the signed `ENROLL` step
  S3b.
- `tools/run_tpm.py`: `loader(case, boot)` uses the enrollment loader only on
  boots that pin `MODE:PROVISION`, and the verify-only loader on every other
  HAUTH boot (separate ESP copies per profile). New cases:
  - `loader-hauth-unenrolled`: seed `fresh` (no change, a never-provisioned
    TPM); one boot that must halt `TPM_UNENROLLED` with the recovery marker
    and leave the recorded public state unchanged.
  - `loader-hauth-cleared`: boot 0 provisions; then seed `clear` issues
    `TPM2_Clear` through the platform hierarchy, and the runner requires the
    cleared public state to equal that of a new TPM; boot 1 must halt
    `TPM_UNENROLLED` with the recovery marker and leave that state unchanged.
- `tools/tpm_seed.py`: seeds `fresh` and `clear` (`TPM_CC_Clear` `0x126`).
  `tools/tests/test_tpm.py`: only provisioning boots run the enrollment
  loader; a clear leaves the public state of a new TPM (live swtpm).

The halt marker is `TPM_UNENROLLED`, not `TPM_ANCHOR`. Section 10.1 allows
`TPM_ANCHOR` only with a verified identity, which this TEST loader does not
establish, and a cleared TPM is indistinguishable from a new one. The
markers are the TEST loader's `LOADER-HAUTH:*` lines, not the `BOOT:*`
markers of the specification.

## Commands and results

Run sequentially in one checkout at source `5b392b72`
(`build/b03-hauth-unenrolled/run.sh`, 11:14:22 to 11:16:40 UTC); raw reports
in the ignored `build/tpm-<case>-report.json`, per-case logs and
`summary.txt` (ends with `DONE`) in `build/b03-hauth-unenrolled/`:

| Command (`python3 -B tools/run_tpm.py --case …`) | Exit | Raw QEMU exit per boot | Guest stdout SHA-256 per boot (prefix) | Loader per boot |
|---|---|---|---|---|
| `loader-hauth-unenrolled` | 0 | 35 | `4473d948…` (new) | verify |
| `loader-hauth-cleared` | 0 | 33, 35 | `eda7e5a3…`, `4473d948…` (new) | enrollment, verify |
| `loader-hauth` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` | enrollment, verify |
| `loader-hauth-cut` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` | enrollment, verify |
| `loader-hauth-cut-hauth-defined` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` | enrollment, verify |
| `loader-hauth-cut-hauth-written` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` | enrollment, verify |
| `loader-hauth-cut-lrec-written` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` | enrollment, verify |
| `loader-hauth-owner-squat-empty` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` | enrollment, verify |
| `loader-hauth-lockout-cut` | 0 | 33, 33 | `d2eb65a3…`, `94a42322…` | verify, verify |
| `loader-hauth-unbound` | 0 | 33, 35 | `eda7e5a3…`, `37de5e2a…` | enrollment, verify |
| `loader-hauth-da-counted` | 0 | 33 | `6c5e18bd…` | enrollment |
| `loader-hauth-squat` | 0 | 35 | `81a89795…` | enrollment |
| `loader-hauth-no-rng` | 0 | 35 | `8b6db745…` | enrollment |
| `loader-hauth-foreign-owner` | 0 | 35 | `65c97d20…` | verify |
| `loader-hauth-owner-squat` | 0 | 35 | `fa928907…` | verify |
| `loader-probe` | 0 | 33 | `53362106…` | probe |
| `loader-rng` | 0 | 33 | `f8d9e155…` | probe |
| `normal` | 0 | 33 | `783d1b60…` | default |
| `absent` | 0 | 33 | `e6fa721b…` | default |

New full hash: both halting boots
`4473d948ba30a37576e90a749bf13aff24abeca6d4305b86608ec8e7a46584ef`. Every
other stdout hash equals the corresponding one in
[b03-rel05-hauth-eventlog-20261126](../b03-rel05-hauth-eventlog-20261126/README.md):
the split changed no output on any boot that provisions or verifies.

| Gate | Exit | Result |
|---|---|---|
| `cargo +1.100.0 fmt --manifest-path boot/loader/Cargo.toml -- --check` | 0 | – |
| `cargo +1.100.0 clippy --locked --manifest-path boot/loader/Cargo.toml --target x86_64-unknown-uefi` with no feature, `tpm-hauth-verify-test`, `tpm-hauth-provision-test` and `tpm-loader-probe-test` | 101 each | only the two existing errors at `main_default.rs:818` (the same line on `main`) |
| `python3 -B -m unittest tools.tests.test_tpm` (`GIT_EDITOR` unset) | 0 | 16 tests, none skipped, swtpm 0.10.1 |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` (`GIT_EDITOR` unset) | 0 | 1355 tests, 15 skipped |
| `python3 -B -m compileall -q tools/run_tpm.py tools/tpm_seed.py tools/build.py tools/tests/test_tpm.py` | 0 | – |
| `python3 -B tools/test.py` (boot acceptance) | 0 | 10 acceptance groups, normal artifacts restored |

## Observed

- Both halting boots print, in this order: `LOADER-HAUTH:BEGIN`,
  `LOADER-HAUTH:PERMANENT:0000000000000400` (`TPMA_PERMANENT`: only
  `tpmGeneratedEPS` set; `ownerAuthSet` clear), `LOADER-HAUTH:HALT:PCR13:CLOSED`,
  `LOADER-HAUTH:FAIL:TPM_UNENROLLED:0000000000000000` and
  `LOADER-HAUTH:RECOVERY:REQUIRED`, then the loader exits with raw QEMU status
  35 and no kernel runs.
- The never-provisioned and the cleared TPM produce byte-identical guest
  output. The runner also found their recorded public state (`permanent`,
  lockout counter, `HAUTH` and `LREC` presence) equal before the boot. Nothing
  the loader reads distinguishes them.
- After each halt, the recorded public state equals the seeded state: no NV
  index was defined or written and no hierarchy authorisation changed. The
  PCR 13 `CLOSED` extend is not a persistent write.

## Limits

- Section 7.9 condition 2 is not met. The choice between first provisioning
  and a clear is made by the build feature and the harness (only boots that
  pin `MODE:PROVISION` get the enrollment loader), not by the device. The
  stand-in is not authenticated; S3b (a signed `ENROLL` record) is not built.
- Condition 1 is not met for the loader either: this TEST loader covers only
  the `ownerAuth`-unset class. The trial path (S6), the `BOOT:*` markers and
  the user-visible recovery text do not exist, and the other anchor-loss
  classes of section 10.1 are not implemented in the loader.
- `TPM2_Clear` is assumed to keep the endorsement seed; the EK was not read
  back before and after the clear.
- The halt path was not checked against the firmware event log (as in the
  event-log evidence, the log is read on the pass path only).
- OVMF 2025.11 with four active banks and swtpm 0.10.1 only.

## NOT RUN

- swtpm 0.7.3 (hosted `native-system.yml` runs `tools.tests.test_tpm` with it,
  including the new `clear` seed test), the hosted OVMF 2024.02 image and
  hosted CI for the loader-* cases.
- A physical TPM or firmware; a firmware-mediated clear (physical-presence
  interface); an EK read-back across the clear; independent human audit.
