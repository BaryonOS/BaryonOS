# B03-REL05 M1-HAUTH: PCR 13 CLOSED entry in the firmware event log (26 November 2026)

**Native TEST experiment, not acceptance evidence.** It closes the item that
[b03-rel05-hauth-cut-points-20261125](../b03-rel05-hauth-cut-points-20261125/README.md)
and [b03-rel05-pcr13-banks-20261125](../b03-rel05-pcr13-banks-20261125/README.md)
listed as NOT RUN: the TCG event log for PCR 13. It accepts no criterion and
grants no progress points.

- Source: commit `477674bf` on `claude/b03-rel05-tpm-20261125` (local, not
  pushed under the lane rule), `tools/common.source_hash()`
  `42e48d3c9a5ee13b237c463ca6f755b9ec77bd3daad138eaffe697422f832950`. All
  seventeen `run_tpm.py` cases are now bound to this source.
- Profile and stack are unchanged: QEMU 10.2.1, `pc-q35-10.0`, TCG, local
  Ubuntu OVMF 2025.11 (`OVMF_CODE_4M.fd` `874f15a7…`), swtpm 0.10.1.
- Binaries: `BOOTX64.EFI` (`tpm-hauth-provision-test`)
  `d6659d59cecdd2c549b07ffae5e7a686d2ba262dcdb5c7ebf147b0fd7fd5204e` (was
  `da716d06…`). Unchanged: the probe loader `b231b4b9…`, the default loader
  `0065fdf6…`, `kernel.elf` `c6cef6ab…` (`absent`: `75f9483e…`); the new
  `tpm-crb` module is not linked into the kernel. The reports' artifact
  binding passed on every boot.

## Firmware interface

Taken from the EDK2 sources (tianocore/edk2 `master`, read 26 November 2026):
`MdePkg/Include/Protocol/Tcg2Protocol.h`,
`MdePkg/Include/IndustryStandard/UefiTcgPlatform.h` and
`SecurityPkg/Tcg/Tcg2Dxe/Tcg2Dxe.c`. The normative TCG EFI Protocol and PC
Client Platform Firmware Profile documents were not read.

- `GetEventLog(This, EventLogFormat: UINT32, *EventLogLocation,
  *EventLogLastEntry, *EventLogTruncated: BOOLEAN)` is the second member of
  `EFI_TCG2_PROTOCOL`; format 2 (`EFI_TCG2_EVENT_LOG_FORMAT_TCG_2`) is the
  crypto-agile log. The loader's `Tcg2` table now types that member; its layout
  is unchanged.
- In `Tcg2Dxe.c`, `EventLogLastEntry` is the start of the last logged event (0
  when none was logged). After the first call, later events are also copied to
  the final events table; the main log keeps growing.
- The log is packed and little-endian: a SHA-1-format `TCG_PCR_EVENT` with
  `EV_NO_ACTION` and the `Spec ID Event03` algorithm table, then
  `TCG_PCR_EVENT2` entries (PCR, type, digest count, per digest `hashAlg` and
  the digest, event size, event data).

## Method

- Loader (`Tpm::closed_event` in `boot/loader/src/tpm_hauth.rs`): runs on
  the pass path, after the PCR 13 bank read-back and before the policy read
  of `HAUTH`. It sends no TPM command, so the CRB residue scan still anchors
  on the last `0x20E` response. It fails closed on a non-`SUCCESS` status, a
  truncated log, `EventLogLocation` 0, `EventLogLastEntry` at or before
  the location, or more than 1 MiB before the last entry. It parses two
  firmware slices: [`EventLogLocation`, `EventLogLastEntry`), and the
  bytes at `EventLogLastEntry` of the exact length the expected entry
  must have (16 bytes plus 2 and the digest size per header algorithm plus
  22 bytes of event data; at most 566 bytes).
- Parser (`drivers/tpm-crb/src/event_log.rs`, safe and `no_std` under
  `forbid(unsafe_code)`, hashes nothing):
  - The header must have PCR 0, `EV_NO_ACTION` and an all-zero SHA-1 digest,
    the signature `Spec ID Event03`, 1 to 8 distinct algorithms with digest
    sizes 1 to 64, and an event size that fits exactly.
  - Every later entry must be whole and use only header algorithms, none
    may name PCR 13, and the walk must end exactly at `EventLogLastEntry`.
  - The last entry must be exactly PCR 13, `EV_IPL` and
    `BARYON-TRIAL-CLOSED-V1`, with one digest per header algorithm in header
    order and no trailing byte.
- Markers: the loader prints one `LOADER-HAUTH:EVENTLOG:DIGEST:<alg>:<hex>`
  line per logged digest, requires SHA-256 to equal `DIGEST_CLOSED`, then
  prints `LOADER-HAUTH:EVENTLOG:CLOSED`. `run_tpm.py` requires the digest
  lines to equal `H(CLOSED)` for exactly the banks in `LOADER-TPM:BANKS`, in
  bank order. Together with the `PCR13:BANK` check, replaying the single
  PCR 13 entry from zero gives the value read from every active bank.
- Host tests: four new `tpm-crb` tests for a valid log and for negative
  headers, prefixes and last entries (truncation, `EventSize` overflow,
  unknown or too many algorithms, an earlier PCR 13 entry, an entry crossing
  `EventLogLastEntry`, wrong PCR, type or data, a missing or reordered
  digest, a trailing byte). Six new negative `test_tpm` transcripts, an
  inactive-bank case, and a pin of `DIGEST_CLOSED` to `trial_tpm.rs`.

## Commands and results

Run sequentially in one checkout at source `42e48d3c`; raw reports and stderr
in the ignored `build/rel05-batch-085222/`, summary in `build/rel05-batch.log`:

| Command (`python3 -B tools/run_tpm.py --case …`) | Exit | Raw QEMU exit per boot | Guest stdout SHA-256 per boot (prefix) |
|---|---|---|---|
| `loader-hauth-cut-hauth-defined` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` |
| `loader-hauth-cut-hauth-written` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` |
| `loader-hauth-cut-lrec-written` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` |
| `loader-hauth-lockout-cut` | 0 | 33, 33 | `d2eb65a3…`, `94a42322…` |
| `loader-hauth` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` |
| `loader-hauth-cut` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` |
| `loader-hauth-owner-squat-empty` | 0 | 33, 33 | `eda7e5a3…`, `94a42322…` |
| `loader-hauth-unbound` | 0 | 33, 35 | `eda7e5a3…`, `37de5e2a…` (boot 1 unchanged) |
| `loader-hauth-da-counted` | 0 | 33 | `6c5e18bd…` |
| `loader-hauth-squat` | 0 | 35 | `81a89795…` (unchanged) |
| `loader-hauth-foreign-owner` | 0 | 35 | `65c97d20…` (unchanged) |
| `loader-hauth-owner-squat` | 0 | 35 | `fa928907…` (unchanged) |
| `loader-hauth-no-rng` | 0 | 35 | `8b6db745…` (unchanged) |
| `loader-probe` | 0 | 33 | `53362106…` (unchanged) |
| `loader-rng` | 0 | 33 | `f8d9e155…` (unchanged) |
| `normal` | 0 | 33 | `783d1b60…` (unchanged) |
| `absent` | 0 | 33 | `e6fa721b…` (unchanged) |

New full hashes: `loader-hauth` boot 0
`eda7e5a37ebcd568446927d72de3d1179a9235b3e0acda6ff984d509ff85ccc4`, boot 1
`94a42322d341c900309f0e31b42bed6d3a39f1fe79d078b3076eb830032675a9`;
`loader-hauth-lockout-cut` boot 0
`d2eb65a36dc8f8d824831063addb3f0aa5d912963a3510e8990b175d38532366`;
`loader-hauth-da-counted`
`6c5e18bdf260bd200004a96ad1f48e91434539226671bac0e91025b9ecb8c795`. The
hashes marked unchanged equal those of the cut-point evidence. Every swtpm
backend exited with status 0, and no swtpm process remained after the batch.

| Gate | Exit | Result |
|---|---|---|
| `cargo +1.100.0 test --offline --locked --manifest-path drivers/tpm-crb/Cargo.toml` | 0 | includes the four new `event_log` tests |
| `cargo +1.100.0 clippy --offline --locked --manifest-path drivers/tpm-crb/Cargo.toml --all-targets -- -D warnings` | 0 | – |
| `cargo +1.100.0 fmt --manifest-path drivers/tpm-crb/Cargo.toml -- --check` | 0 | – |
| `cargo +1.100.0 build --offline --locked --release --manifest-path drivers/tpm-crb/Cargo.toml --target x86_64-unknown-none` | 0 | – |
| `cargo +1.100.0 fmt --manifest-path boot/loader/Cargo.toml -- --check` | 0 | – |
| `cargo +1.100.0 clippy --locked --manifest-path boot/loader/Cargo.toml --target x86_64-unknown-uefi` with `tpm-hauth-provision-test`, `tpm-loader-probe-test` and no feature | 101 each | only the two existing errors at `main_default.rs:818` |
| `python3 -B -m unittest tools.tests.test_tpm -v` (`GIT_EDITOR` unset) | 0 | 13 tests, none skipped; both `TpmSeedTests` ran on swtpm 0.10.1 |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` (`GIT_EDITOR` unset) | 0 | 1184 tests, 15 skipped |
| `python3 -B -m compileall -q tools/run_tpm.py tools/tests/test_tpm.py` | 0 | – |
| `python3 -B tools/test.py` (boot acceptance) | 0 | 10 acceptance groups, normal artifacts restored |
| `python3 -B tools/progress.py --check` | 0 | 43/100, consistent |

## Observed

- On both boots of every passing `loader-hauth` case, the parser accepted the
  log: no PCR 13 entry before the last one, and the last entry was the
  loader's `EV_IPL` `CLOSED` event. The logged digests were, in this order:

```text
LOADER-HAUTH:EVENTLOG:DIGEST:0000000000000004:6b034108a5b39e1641e482b6ee73011208d8c645
LOADER-HAUTH:EVENTLOG:DIGEST:000000000000000b:045f890d159cf579eac941ebf1a97ed642a3c1a1867effeaa417079b1a5221b2
LOADER-HAUTH:EVENTLOG:DIGEST:000000000000000c:1ba76fe2edd0771a4ac63c321630f003f017039ef505593e2b6b33c60a6db4a216e427e2d079a760b1efc98d131dea34
LOADER-HAUTH:EVENTLOG:DIGEST:000000000000000d:af3c17a8cc27d00170d18d7b767af1ae975d91b7bfc99cad83fd4d907abe679856f8c989b87ba788d7183ef53a0bee6e84d5059a92eac3093b05966c696cdd99
LOADER-HAUTH:EVENTLOG:CLOSED
```

- Each digest is `H(BARYON-TRIAL-CLOSED-V1)` in its bank (checked by the
  runner with `hashlib`). The SHA-256 digest is `DIGEST_CLOSED`, so the
  firmware hashes the event data as the spec assumed (U2).
- Compared with the cut-point batch, `loader-hauth` differs on both boots only
  by these five lines; this batch showed no `IPCWAIT:TICKS` jitter. Halting
  boots and the cases without HAUTH are byte-identical to the earlier evidence.

## Limits

- Pass path only. Halting boots extend `CLOSED` but stop before the log is
  read, so the log after a halt is not checked.
- At `EventLogLastEntry` the loader reads the exact length of the expected
  entry. A different last entry is refused after a bounded read of at most
  566 bytes of firmware memory, which is in the TCB (U2).
- The prefix walk checks structure and the absence of PCR 13 entries. It does
  not check the digests of other PCRs or replay them.
- Only the crypto-agile log was read: not the SHA-1-format log, and not the
  final events table (events logged after `GetEventLog`, such as
  `ExitBootServices`). No consumer of either exists yet.
- In EDK2, the first `GetEventLog` call makes firmware also copy later events
  into the final events table; the kernel output is unchanged.
- OVMF 2025.11 with four active banks and swtpm 0.10.1 only. Fewer active
  banks are covered by the runner's mask logic and host tests, not natively.
- The interface facts come from the EDK2 sources, not from the normative TCG
  documents.
- `drivers/tpm-crb/src/event_log.rs` and `drivers/tpm-crb/tests/event_log.rs`
  are not listed in `MPL-2.0-FILES.txt` (neither is `tests/codec.rs`). The
  register is an owner or lead decision, so this is reported, not changed.

## NOT RUN

- The event log on a halt path; a firmware or TPM with fewer active banks;
  the SHA-1-format log; the final events table.
- swtpm 0.7.3, the hosted OVMF 2024.02 image and hosted CI for the loader-*
  cases. `native-system.yml` runs the `tpm-crb` tests and `tools.tests.test_tpm`
  on the first lane push.
- A fault-injected interruption and a torn NV write; physical TPM or
  firmware; independent human audit.
