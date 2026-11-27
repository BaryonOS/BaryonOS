# B03-REL05 M1-HAUTH: cut between the ownerAuth and lockoutAuth changes (25 November 2026)

**Native TEST experiment, not acceptance evidence.** It closes the item that
[b03-rel05-pcr13-banks-20261125](../b03-rel05-pcr13-banks-20261125/README.md)
listed as NOT RUN: a power cut after the loader's ownerAuth change and before
its lockoutAuth change. It accepts no criterion and grants no progress points.

- Source: commit `94acc222` on `claude/b03-rel05-tpm-20261125` (local, not
  pushed under the lane rule), `tools/common.source_hash()`
  `6817b4e72714d5dda825935d05d59c6b38bbfc04f403110e0d681da49c3864a2`. All fourteen `run_tpm.py`
  cases are now bound to this source. No Rust source changed.
- Profile and stack: unchanged (QEMU 10.2.1, `pc-q35-10.0`, TCG, `-cpu qemu64`,
  1 CPU, 256 MiB; local Ubuntu OVMF 2025.11 `OVMF_CODE_4M.fd` SHA-256
  `874f15a7…6f758aa7`; TEST virtio-rng-pci where the case pins it;
  swtpm 0.10.1).
- Binaries are unchanged from the PCR 13 bank evidence: `BOOTX64.EFI`
  `da716d06…` (`tpm-hauth-provision-test`), `b231b4b9…` (probe loader),
  `0065fdf6…` (default loader); `kernel.elf` `c6cef6ab…` (`absent`:
  `75f9483e…`). The reports' artifact binding passed on every boot.

## Method

- Seed `lockout-cut` (`tools/tpm_seed.py`), on a fresh swtpm state at PCR 13 = 0:
  the `cut` steps with a fresh 64-byte TEST `HAUTH` secret kept in memory
  (define and write `HAUTH`, define `LREC`), then a BOUND `LREC` written through
  the loader's PCR 13 = 0 write policy, then
  `HierarchyChangeAuth(owner, HAUTH[:32])`. lockoutAuth stays empty. The
  record is `NOSLRC01`, 40 zero bytes and `SHA-256(HAUTH)[:16]`; `test_tpm`
  pins the magic, the binding length and the binding formula to
  `crates/update-core/src/trial.rs`. The secret is never printed or stored.
- Runner (`tools/run_tpm.py --case loader-hauth-lockout-cut`, two boots on one
  state): boot 0 must print `MODE:EXISTING`, `BOUND` and `LOCKOUT:SET` with
  `PERMANENT & 5 == 1` and none of `MODE:PROVISION` or `LOCKOUT:KEPT`; boot 1
  must follow the `loader-hauth` boot-1 path. Expected modes now come from
  `hauth_mode(case, boot)` and the `REPAIRED` table. After boot 0, the public
  state must equal the seeded state except for `TPMA_PERMANENT` bit 2
  (lockoutAuthSet): the same `HAUTH` and `LREC` Names, the same `LREC` digest
  and the same lockout counter, so the loader repaired the cut and did not
  re-provision.

## Commands and results

Run sequentially in one checkout at source `6817b4e7`; raw reports and stderr
in the ignored `build/rel05-batch-212521/`, summary in `build/rel05-batch.log`:

| Command (`python3 -B tools/run_tpm.py --case …`) | Exit | Raw QEMU exit per boot | Guest stdout SHA-256 per boot |
|---|---|---|---|
| `loader-hauth-lockout-cut` (new) | 0 | 33, 33 | `527f67582e40e7e71ff7cd78ba305094a48a7c2f1ae26758f1c229144db22bbf`, `08a3f8333b9c67151761aa0d7d03a363f647128417ff890b4c5793084c0493c2` |
| `loader-hauth` | 0 | 33, 33 | `7a4c0d5c…` (unchanged), `a3a5330c2f8beb83993d9a5b88663e538d8f7738e22cecb0ba2f6d98421f0370` (see Observed) |
| `loader-hauth-owner-squat-empty` | 0 | 33, 33 | `7a4c0d5c…`, `08a3f833…` (unchanged) |
| `loader-hauth-cut` | 0 | 33, 33 | `7a4c0d5c…`, `08a3f833…` (unchanged) |
| `loader-hauth-unbound` | 0 | 33, 35 | `7a4c0d5c…`, `37de5e2a…` (unchanged) |
| `loader-hauth-da-counted` | 0 | 33 | `14145a63…` (unchanged) |
| `loader-hauth-squat` | 0 | 35 | `81a89795…` (unchanged) |
| `loader-hauth-foreign-owner` | 0 | 35 | `65c97d20…` (unchanged) |
| `loader-hauth-owner-squat` | 0 | 35 | `fa928907…` (unchanged) |
| `loader-hauth-no-rng` | 0 | 35 | `8b6db745…` (unchanged) |
| `loader-probe` | 0 | 33 | `53362106…` (unchanged) |
| `loader-rng` | 0 | 33 | `f8d9e155…` (unchanged) |
| `normal` | 0 | 33 | `783d1b60…` (unchanged) |
| `absent` | 0 | 33 | `e6fa721b…` (unchanged) |

The guest stdout SHA-256 is computed over the report's UTF-8 `stdout`. Every
swtpm backend exited with status 0, and no swtpm process remained after the
batch.

| Gate | Exit | Result |
|---|---|---|
| `python3 -B -m unittest tools.tests.test_tpm -v` (`GIT_EDITOR` unset) | 0 | 12 tests, none skipped; both `TpmSeedTests` ran on swtpm 0.10.1 |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` (`GIT_EDITOR` unset) | 0 | 1183 tests, 15 skipped |
| `python3 -B -m compileall -q tools/tpm_seed.py tools/run_tpm.py tools/tests/test_tpm.py` | 0 | – |
| `python3 -B tools/progress.py --check` | 0 | 43/100, consistent |

No Rust gate was rerun: the slice changes only `tools/` and documentation, and
the loader and kernel binaries hash as before.

## Observed

- Boot 0 of `loader-hauth-lockout-cut` printed, in order (PCR 13 bank lines
  omitted; they equal the CLOSED values of the PCR 13 bank evidence):

```text
LOADER-HAUTH:BEGIN
LOADER-HAUTH:PERMANENT:0000000000000401
LOADER-HAUTH:MODE:EXISTING
LOADER-HAUTH:BOUND
LOADER-HAUTH:LOCKOUT:SET
LOADER-HAUTH:PCR13:CLOSED
LOADER-HAUTH:READ:REFUSED
LOADER-HAUTH:OWNER:REFUSED
LOADER-HAUTH:DA:0000000000000000:0000000000000000
LOADER-HAUTH:RESIDUE:NONE
LOADER-HAUTH:PASS
```

- Seeded public state: `TPMA_PERMANENT` `0x401`, lockout counter 0, `HAUTH`
  Name `000bbd7d…9ce2` (`NAME_HAUTH_WRITTEN`), `LREC` Name `000b282e…7a60`
  (`NAME_LREC_WRITTEN`). After boot 0 and after boot 1: `0x405`, with the same
  Names, the same `LREC` digest and counter 0. The `LREC` digest differs per run
  because the seed secret is fresh.
- Boot 1 is byte-identical to boot 1 of `loader-hauth-cut` and
  `loader-hauth-owner-squat-empty` (`08a3f833…`).
- `loader-hauth` boot 1 differs from that same output only in two kernel
  `IPCWAIT:TICKS` lines (161 to 162 and 201 to 202), timer-tick counts of the
  IPC-wait probe under TCG. All `LOADER-HAUTH:` and `TPMCRB:` lines are
  identical, and the runner does not pin tick values. The `08a3f833…` hash of
  the earlier evidence is therefore a typical value, not a fixed one.

## Limits

- The TEST loader does not compare ownerAuth with `HAUTH[..32]` (an open item
  reported to the project owner). This case therefore shows that the loader
  sets lockoutAuth on a state with ownerAuth set and a bound `LREC`; it would
  behave the same with any other non-empty ownerAuth and a consistent
  `HAUTH`/`LREC` pair. It is not evidence that a foreign ownerAuth is refused.
- The cut state is built by the host seed, not by interrupting a loader boot.
  Its `HAUTH` and `LREC` follow the frozen templates and the `trial.rs`
  encoding, so the loader cannot tell them apart from its own writes; a real
  interruption (fault injection) was not run.
- The seed reads and writes at PCR 13 = 0, the boot-path position of threat T5,
  which stays out of scope until B04.
- Results hold for OVMF 2025.11 and swtpm 0.10.1 only.

## NOT RUN

- The other cut points of the provisioning sequence (`HAUTH` defined but not
  written; `HAUTH` written before the `LREC` define; `LREC` written before the
  ownerAuth change) and a fault-injected halt after an NV write.
- The TCG event log for PCR 13; a firmware or TPM with fewer active banks.
- swtpm 0.7.3, the hosted OVMF 2024.02 image and hosted CI for the loader-*
  cases.
- Physical TPM or firmware; independent human audit.
