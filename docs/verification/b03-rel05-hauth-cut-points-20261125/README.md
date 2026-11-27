# B03-REL05 M1-HAUTH: seeded power cut after every provisioning step (25 November 2026)

**Native TEST experiment, not acceptance evidence.** It closes the items that
[b03-rel05-hauth-lockout-cut-20261125](../b03-rel05-hauth-lockout-cut-20261125/README.md)
listed as NOT RUN: the cut points of the provisioning sequence before the
`LREC` define and before the ownerAuth change. It accepts no criterion and
grants no progress points.

- Source: commit `fa2d7f58` on `claude/b03-rel05-tpm-20261125` (local, not
  pushed under the lane rule), `tools/common.source_hash()`
  `2816266d485ebeb839a3e4ac659e8fe1876e63253fb2b72485099dccd9c2bb05`. All
  seventeen `run_tpm.py` cases are now bound to this source. No Rust source
  changed.
- Profile, stack and binaries are unchanged from the lockout-cut evidence:
  QEMU 10.2.1, `pc-q35-10.0`, TCG, local OVMF 2025.11, swtpm 0.10.1;
  `BOOTX64.EFI` `da716d06…` (`tpm-hauth-provision-test`), `b231b4b9…` (probe),
  `0065fdf6…` (default); `kernel.elf` `c6cef6ab…` (`absent`: `75f9483e…`).

## Provisioning sequence and cut points

The TEST loader provisions in this order (`boot/loader/src/tpm_hauth.rs`,
`bind()`), all under the empty ownerAuth until step 5:

| After step | Seed (`tools/tpm_seed.py`) | Case (`run_tpm.py --case …`) | Expected boot 0 |
|---|---|---|---|
| 1. define `HAUTH` | `hauth-defined` | `loader-hauth-cut-hauth-defined` (new) | re-provision |
| 2. write `HAUTH` | `hauth-written` | `loader-hauth-cut-hauth-written` (new) | re-provision |
| 3. define `LREC` | `cut` | `loader-hauth-cut` | re-provision |
| 4. write `LREC` (BOUND) | `lrec-written` | `loader-hauth-cut-lrec-written` (new) | re-provision |
| 5. ownerAuth = `HAUTH[:32]` | `lockout-cut` | `loader-hauth-lockout-cut` | set lockoutAuth only |

Steps 1 to 4 leave ownerAuth empty, so the loader takes `MODE:PROVISION`,
removes both handles (tolerating only `TPM_RC_HANDLE` `0x28B`), and provisions
fresh. The seeds share one chain of helpers, so each cut point is the previous
one plus one step. A cut after step 6 (lockoutAuth set) is the normal
provisioned state of `loader-hauth` boot 1.

## Method

- Runner: the three new cases are `PROVISIONED` two-boot cases with the
  `loader-hauth` expectations (boot 0 `MODE:PROVISION`, `LOCKOUT:SET`,
  PERMANENT bits 0; boot 1 `MODE:EXISTING`, `LOCKOUT:KEPT`, bits 5).
- The post-boot public-state checks moved from `run()` into
  `check_state(case, boot, entry)`, now unit-tested in `test_tpm`: a halt
  leaves the seeded state unchanged; the lockout-cut repair changes only
  lockoutAuthSet; and, new, every seeded boot that re-provisions must end with
  a freshly written `LREC` whose digest differs from the seeded one. The new
  rule also applies to `loader-hauth-cut`, `loader-hauth-owner-squat-empty`
  and `loader-hauth-da-counted`.
- `TpmSeedTests` checks each cut seed's public state on a fresh swtpm:
  PERMANENT bits, lockout counter 0, the frozen `HAUTH` Name (unwritten after
  step 1, written after step 2), `LREC` absent (`0x18B`) or written with the
  `trial.rs` layout.

## Commands and results

Run sequentially in one checkout at source `2816266d`; raw reports in the
ignored `build/rel05-batch-214140/`, summary in `build/rel05-batch.log`:

| Command (`python3 -B tools/run_tpm.py --case …`) | Exit | Raw QEMU exit per boot | Guest stdout SHA-256 per boot (prefix) |
|---|---|---|---|
| `loader-hauth-cut-hauth-defined` (new) | 0 | 33, 33 | `7a4c0d5c…`, `08a3f833…` |
| `loader-hauth-cut-hauth-written` (new) | 0 | 33, 33 | `7a4c0d5c…`, `08a3f833…` |
| `loader-hauth-cut-lrec-written` (new) | 0 | 33, 33 | `7a4c0d5c…`, `08a3f833…` |
| `loader-hauth-lockout-cut` | 0 | 33, 33 | `527f6758…`, `08a3f833…` |
| `loader-hauth` | 0 | 33, 33 | `7a4c0d5c…`, `08a3f833…` |
| `loader-hauth-cut` | 0 | 33, 33 | `7a4c0d5c…`, `08a3f833…` |
| `loader-hauth-owner-squat-empty` | 0 | 33, 33 | `7a4c0d5c…`, `08a3f833…` |
| `loader-hauth-unbound` | 0 | 33, 35 | `7a4c0d5c…`, `37de5e2a…` |
| `loader-hauth-da-counted` | 0 | 33 | `14145a63…` |
| `loader-hauth-squat` | 0 | 35 | `81a89795…` |
| `loader-hauth-foreign-owner` | 0 | 35 | `65c97d20…` |
| `loader-hauth-owner-squat` | 0 | 35 | `fa928907…` |
| `loader-hauth-no-rng` | 0 | 35 | `8b6db745…` |
| `loader-probe` | 0 | 33 | `53362106…` |
| `loader-rng` | 0 | 33 | `f8d9e155…` |
| `normal` | 0 | 33 | `783d1b60…` |
| `absent` | 0 | 33 | `e6fa721b…` |

Full hashes are in the earlier evidence directories; every prefix above
matches them. Every swtpm backend exited with status 0, and no swtpm process
remained after the batch.

| Gate | Exit | Result |
|---|---|---|
| `python3 -B -m unittest tools.tests.test_tpm -v` (`GIT_EDITOR` unset) | 0 | 13 tests, none skipped; both `TpmSeedTests` ran on swtpm 0.10.1 |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` (`GIT_EDITOR` unset) | 0 | 1184 tests, 15 skipped |
| `python3 -B -m compileall -q tools/tpm_seed.py tools/run_tpm.py tools/tests/test_tpm.py` | 0 | – |
| `python3 -B tools/progress.py --check` | 0 | 43/100, consistent |

## Observed

- Boot 0 of each new case is byte-identical to `loader-hauth` boot 0: the
  loader prints nothing for the removal of leftovers, so a cut at steps 1 to 4
  is indistinguishable in its output from a fresh TPM. Boot 1 is identical to
  `loader-hauth` boot 1.
- Seeded and resulting public state (Names abbreviated; `0x18B` = index not
  defined):

| Case | Seeded `HAUTH` | Seeded `LREC` | After boot 0 |
|---|---|---|---|
| `…-cut-hauth-defined` | Name `000b8bc4…` (`NAME_HAUTH`, unwritten) | `0x18B` | PERMANENT `0x405`, `HAUTH` `000bbd7d…`, `LREC` `000b282e…` written |
| `…-cut-hauth-written` | `000bbd7d…` (written) | `0x18B` | as above |
| `…-cut-lrec-written` | `000bbd7d…` | `000b282e…`, digest `7cdb3f7c…` | as above, `LREC` digest `f04718ca…` (replaced) |

- In this batch, `loader-hauth` boot 1 hashed `08a3f833…` again; the
  `a3a5330c…` of the lockout-cut batch was the `IPCWAIT:TICKS` jitter recorded
  there.

## Limits

- The cut states are built by the host seed, not by interrupting a loader boot
  (no fault injection). They follow the frozen templates and the `trial.rs`
  encoding, so the loader cannot tell them apart from its own writes.
- A cut inside a single TPM command (a torn NV write) is not modelled; NV write
  atomicity (U22) remains a separate open item.
- Re-provisioning after a cut at steps 1 to 4 discards the earlier secret. That
  is intended while no genesis or ACCEPTED pair exists; with genesis, U13c
  record staging (an open OWNER question) decides what a cut may discard.
- The TEST loader does not compare ownerAuth with `HAUTH[..32]` (open, reported
  to the project owner). Seeds run at PCR 13 = 0, the T5 position (out of scope
  until B04). Results hold for OVMF 2025.11 and swtpm 0.10.1 only.

## NOT RUN

- A fault-injected interruption of a real loader boot; a torn NV write.
- The TCG event log for PCR 13; a firmware or TPM with fewer active banks.
- swtpm 0.7.3, the hosted OVMF 2024.02 image and hosted CI for the loader-*
  cases.
- Physical TPM or firmware; independent human audit.
