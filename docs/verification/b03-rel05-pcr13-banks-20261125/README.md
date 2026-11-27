# B03-REL05 M1-HAUTH: PCR 13 read-back of every active bank (25 November 2026)

**Native TEST experiment, not acceptance evidence.** It closes the item that
[b03-rel05-hauth-da-squat-20261125](../b03-rel05-hauth-da-squat-20261125/README.md)
listed as NOT RUN: the read-back of the SHA-1, SHA-384 and SHA-512 banks of
PCR 13 after the loader's `CLOSED` extend. It accepts no criterion and grants
no progress points.

- Source: commit `2c9fdec9` on `claude/b03-rel05-tpm-20261125` (local, not
  pushed under the CI-first lane rule), `tools/common.source_hash()`
  `16ff840956209fc3ca7d6b06362041aaf6af3e2a3ed0eb5339a3064ed78defa0`. All
  thirteen `run_tpm.py` cases are now bound to this source.
- Profile and stack: unchanged from the seeded evidence (QEMU 10.2.1,
  `pc-q35-10.0`, TCG, `-cpu qemu64`, 1 CPU, 256 MiB; local Ubuntu OVMF 2025.11
  `OVMF_CODE_4M.fd` SHA-256 `874f15a7…6f758aa7`; TEST virtio-rng-pci where the
  case pins it; swtpm 0.10.1, binary SHA-256 `1ff9ce32…1d1844`).
- Binaries (from the reports' artifact binding, which passed on every boot):

| Build | `BOOTX64.EFI` SHA-256 | `kernel.elf` SHA-256 |
|---|---|---|
| `tpm-hauth-provision-test` (all `loader-hauth*` cases) | `da716d066c9383987f986819b272937fbf55b624b9001d01817749932625fb8b` | `c6cef6abb2facba1846027248098b04120d873fb2c243736ea551442255ce29a` |
| `tpm-loader-probe-test` (`loader-probe`, `loader-rng`) | `b231b4b9…` | `c6cef6ab…` |
| default loader, `normal` | `0065fdf6…` | `c6cef6ab…` |
| default loader, `absent` (`test-tpm-crb-absent`) | `0065fdf6…` | `75f9483e…` |

  The kernel binary changed because it links the `drivers/tpm-crb` codec,
  which gained a request variant; its guest output did not change.

## Method

- `drivers/tpm-crb` codec: `Request::PcrReadBank { pcr, alg }` for
  `ALG_SHA1` (`0x0004`), `ALG_SHA256` (`0x000B`), `ALG_SHA384` (`0x000C`) and
  `ALG_SHA512` (`0x000D`); the response must carry exactly one selection of
  that bank and PCR and one digest of the bank's size (up to 64 bytes). Any
  other algorithm, PCR 24 and above, an empty (unallocated) answer, a wrong
  bank or PCR and every truncation are refused (host test
  `pcr_read_bank_decodes_each_bank_and_refuses_other_algorithms`).
- Loader (`tpm-hauth-provision-test`, `after()`): after
  `LOADER-HAUTH:PCR13:CLOSED` and before the two `0x20E` reads, it calls
  `GetActivePcrBanks` and prints one public line per bank,
  `LOADER-HAUTH:PCR13:BANK:<alg as 16 hex digits>:<digest hex>`, or `INACTIVE`
  without a TPM read when the firmware bit is clear. A failed read halts the
  loader like every other `after()` failure. The `0x20E` answer stays the last
  TPM response before the residue scan. `tpm_probe.rs` now shares the
  `GetActivePcrBanks` call through `active_banks()`; the probe loader's
  output is unchanged.
- Runner (`run_tpm.py`): `closed_banks(mask)` computes, for each bank bit of
  the firmware mask `LOADER-TPM:BANKS`, `H(0ⁿ ‖ H("BARYON-TRIAL-CLOSED-V1"))`
  in that bank's hash (`hashlib`). Every provisioned boot must print exactly
  these four lines in order; any other case must print none.

## Commands and results

Run sequentially in one checkout at source `16ff8409` (commit `2c9fdec9`);
raw logs in the ignored `build/rel05-batch-195302/` and `build/rel05-batch.log`:

| Command (`python3 -B tools/run_tpm.py --case …`) | Exit | Raw QEMU exit per boot | Guest stdout SHA-256 per boot |
|---|---|---|---|
| `loader-hauth` | 0 | 33, 33 | `7a4c0d5cbf9d0f36772066a01d7a34f8568ad2fb40b44f8c47f1fec28234a6a7`, `08a3f8333b9c67151761aa0d7d03a363f647128417ff890b4c5793084c0493c2` |
| `loader-hauth-owner-squat-empty` | 0 | 33, 33 | `7a4c0d5c…`, `08a3f833…` (identical to `loader-hauth`) |
| `loader-hauth-cut` | 0 | 33, 33 | `7a4c0d5c…`, `08a3f833…` (identical to `loader-hauth`) |
| `loader-hauth-unbound` | 0 | 33, 35 | `7a4c0d5c…`, `37de5e2a36866748c7edecbbec5a5d878e79cef1cf4ba5a6ac8e3d996939d3ac` (halt boot unchanged) |
| `loader-hauth-da-counted` | 0 | 33 | `14145a637e4c744119a778ea63f6d847aabd4d7e5df11a423eb7d86c082fdc54` |
| `loader-hauth-squat` | 0 | 35 | `81a89795e06c320e0f0c3511f53eeed4cacb0d16816a436f4601035960c13c7a` (unchanged) |
| `loader-hauth-foreign-owner` | 0 | 35 | `65c97d205e60848230afcf41059fb4de5d1261c7786584e7a57de8cd485af8a4` (unchanged) |
| `loader-hauth-owner-squat` | 0 | 35 | `fa928907f2fb5812e84ab82209ba3417b0152019198367a855c3a91e04250ad0` (unchanged) |
| `loader-hauth-no-rng` | 0 | 35 | `8b6db745637dde3820e7c2c7a42d680f75ca541adaa18fe2a5d222c7fbf2f0ff` (unchanged) |
| `loader-probe` | 0 | 33 | `5336210692e0d50627d0084586c586e13fe46aa5aa4e98f2076223d199a498de` (unchanged) |
| `loader-rng` | 0 | 33 | `f8d9e155bf8ce42c78ebfd5e34d89c400addec233b0f8aa0c3d3172098bd7cc9` (unchanged) |
| `normal` | 0 | 33 | `783d1b60ee0dbb6219b2f5e89a432f0d1d24f257f68300acfa8f852f37dd1b6c` (unchanged) |
| `absent` | 0 | 33 | `e6fa721b3827f35cf17c3c8704a2a156e01ab8161190135195ac25087c20c3c8` (unchanged) |

The guest stdout SHA-256 is computed over the report's UTF-8 `stdout`. The
swtpm backend exited with status 0 on every boot, and no swtpm process remained
after the batch.

| Gate | Exit | Result |
|---|---|---|
| `cargo +1.100.0 test --offline --locked --manifest-path drivers/tpm-crb/Cargo.toml` | 0 | 21 codec, 25 core tests |
| `cargo +1.100.0 clippy … --all-targets -- -D warnings` (same manifest) | 0 | – |
| `cargo +1.100.0 fmt --manifest-path drivers/tpm-crb/Cargo.toml -- --check` | 0 | – |
| `cargo +1.100.0 build --offline --locked --release --lib --target x86_64-unknown-none` (same manifest, the native-foundation scalar `RUSTFLAGS`) | 0 | up to date in its target directory |
| `cargo +1.90.0 clippy --locked --manifest-path boot/loader/Cargo.toml --target x86_64-unknown-uefi` with `tpm-hauth-provision-test`, `tpm-loader-probe-test` and no feature | 101 each | only the two pre-existing errors at `main_default.rs:818` |
| `cargo +1.90.0 fmt --manifest-path boot/loader/Cargo.toml -- --check` | 0 | – |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` (`GIT_EDITOR` unset) | 0 | 1181 tests, 15 skipped; `test_tpm` 10 OK with `TpmSeedTests` on swtpm 0.10.1 |
| `python3 -m compileall -q tools/run_tpm.py tools/tests/test_tpm.py` | 0 | – |
| `python3 -B tools/progress.py --check` | 0 | 43/100, consistent |
| the seven `ci.yml` `tests/` unittest patterns (`test_ci_evidence*`, `test_ci_workflows`, `test_document_session_*`, `test_security_alerts_summary`, `test_system_*`, `test_tokenguard*`, `test_volume_*`) | 0 each | 545 tests |

## Observed

- Firmware bank mask `LOADER-TPM:BANKS:000000000000000f`. Every provisioned
  boot, on both the provisioning and the verifying boot, printed:

```text
LOADER-HAUTH:PCR13:BANK:0000000000000004:1bf28bd4e4419b916cfc277f43efece61f2eb0d3
LOADER-HAUTH:PCR13:BANK:000000000000000b:6c3bbe55b4efb9b75ad347f6c2e1d14a23235dfda8027599614e8ef0faae57a2
LOADER-HAUTH:PCR13:BANK:000000000000000c:f6e96e2d97a86845ec25896a5212fe00d14c1cee8dd5b2a329c78ae922e2f015531c3114bcaf034178ba8d638432e5b4
LOADER-HAUTH:PCR13:BANK:000000000000000d:8848751d5cc117033812a900b6e1b2ab8d551aca98bc5732bc039ce32c0b30d81d51cedad182186cb385d39162db021b83bf0514cf4cedd9b4cc1636d2145bca
```

  Each value equals the bank's hash of its zero digest concatenated with the
  bank's hash of `BARYON-TRIAL-CLOSED-V1`, so OVMF 2025.11's
  `HashLogExtendEvent` extended `CLOSED` once into each of the four active
  banks of PCR 13, and nothing else reached PCR 13 before the read-back. The
  SHA-256 line equals `P_CLOSED` in `crates/update-core/src/trial_tpm.rs`.
- Provisioned boots 0 and 1 remain byte-identical across `loader-hauth`,
  `loader-hauth-owner-squat-empty` and `loader-hauth-cut` (and boot 0 of
  `loader-hauth-unbound`). `loader-hauth-da-counted` differs from
  `loader-hauth` boot 0 in exactly one line, the `DA` line (`…01:…01`).
- Halt boots and the kernel-only and probe cases keep their earlier stdout
  hashes: the read-back adds lines only after `CLOSED` on paths that pass.

## Limits

- The loader's `INACTIVE` branch did not run natively: OVMF 2025.11 activates
  all four banks. It is covered by runner unit tests only, and the codec's
  refusal of an unallocated bank by a host test only.
- Only the SHA-256 expectation is checked against an independent constant
  (`P_CLOSED` in `trial_tpm.rs`, also checked in `test_tpm`). The SHA-1,
  SHA-384 and SHA-512 expectations come from the runner's own formula, and the
  unit transcripts are generated by that same `closed_banks()`, so the native
  read-back is the only independent confirmation of those three values.
- An inactive bank is taken from firmware (`GetActivePcrBanks`), not from the
  TPM's PCR allocation. A bank that the TPM allocates but the firmware does not
  report would be printed `INACTIVE` without a read.
- The TCG event log for PCR 13 was not read.
- Results hold for OVMF 2025.11 and swtpm 0.10.1 only.

## NOT RUN

- The TCG event log for PCR 13; a firmware or TPM with fewer active banks.
- A halt in the provisioning path after an NV write (fault injection) and a
  power cut between the ownerAuth and lockoutAuth changes.
- swtpm 0.7.3, the hosted OVMF 2024.02 image and hosted CI for the loader-*
  cases.
- Physical TPM or firmware; independent human audit.
