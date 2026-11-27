# B03-REL05 M1-HAUTH: empty-ownerAuth owner squat and DA-counter sensitivity (25 November 2026)

**Native TEST experiment, not acceptance evidence.** It extends the pre-seeded
M1-HAUTH cases
([b03-rel05-hauth-seeded-20261125](../b03-rel05-hauth-seeded-20261125/README.md))
with two cases that the earlier evidence listed as NOT RUN. It accepts no
criterion and grants no progress points.

- Source: commit `06b7528a` on `claude/b03-rel05-tpm-20261125` (local, not
  pushed under the CI-first lane rule), `tools/common.source_hash()`
  `99347dfa09df6fd93465d747c6313597a3f4b8921622771ada9c397f27cc5f11`.
- Profile and stack: identical to the seeded evidence (QEMU 10.2.1,
  `pc-q35-10.0`, TCG, `-cpu qemu64`, 1 CPU, 256 MiB; local Ubuntu OVMF 2025.11
  `OVMF_CODE_4M.fd`; TEST virtio-rng-pci; swtpm 0.10.1).
- Loader `BOOTX64.EFI` SHA-256 `5bbcc2fa7dd1b810c15ce3b4285e54795d98b975698765ea9196c991bb942271`;
  kernel SHA-256 `ba349c3b83755ec466809d2e509dcb4d825cf8946393c9303024b75ed2b55fda`.
  Both are unchanged: this slice changes only `tools/`.

## Method

`tools/tpm_seed.py` gains two seeds, applied before boot 0:

| Case | Seed | Expected loader result |
|---|---|---|
| `loader-hauth-owner-squat-empty` | `HAUTH` defined by the owner hierarchy (`AUTHWRITE`, `AUTHREAD`, `NO_DA`, as in probe S1) under the empty ownerAuth | boot 0: `MODE:PROVISION`, the owner undefine removes the squat, provisioning proceeds as in `loader-hauth`; boot 1 verifies |
| `loader-hauth-da-counted` | an owner scratch index `0x0100B3DA` without `NO_DA` and with a fresh TEST authValue; one `NV_Read` with the empty password; then the index is undefined | the seed itself requires `TPM_RC_AUTH_FAIL` (`0x98E`) and a lockout counter (`TPM_PT_LOCKOUT_COUNTER`, `0x20E`) one higher than before; the loader provisions and its `DA` line must equal the seeded counter |

`observe()` now also records `TPM_PT_LOCKOUT_COUNTER`, so every seeded and
post-boot snapshot carries it; a halt boot must leave it unchanged with the
rest of the public state. `run_tpm.evaluate()` requires a zero counter on every
provisioned boot except the DA case, which requires a nonzero one. `run()`
compares the DA case's loader value with the counter read on the seeding
server. `tpm_seed.server()` now refuses a control-socket path longer than 107
bytes before it starts swtpm: a longer path previously failed only at the
connect, after swtpm had started, and left the process running.

## Commands and results

Run sequentially in one checkout at source `99347dfa` (commit `06b7528a`):

| Command | Exit | Raw QEMU exit | Guest stdout SHA-256 |
|---|---|---|---|
| `python3 -B tools/run_tpm.py --case loader-hauth-owner-squat-empty` boot 0 | 0 | 33 | `468b5eac8b5f8b62d480086a126613b3952523db558e76a98eaa9458aca38e53` |
| same run, boot 1 | – | 33 | `f99667520dd46e805f969bd76f87cee7c44a950076c9f7a0f812e4cecae5279e` |
| `python3 -B tools/run_tpm.py --case loader-hauth-da-counted` | 0 | 33 | `6a6a8601956848a701bf69f53ab359452f3c5590382331d8337324fd3ce77bfd` |
| the eleven existing cases (`loader-hauth-squat`, `-foreign-owner`, `-owner-squat`, `-unbound`, `-cut`, `loader-hauth`, `-no-rng`, `loader-probe`, `loader-rng`, `normal`, `absent`) | 0 each | as pinned | unchanged from the seeded evidence (`81a89795`, `65c97d20`, `fa928907`, `468b5eac`/`37de5e2a`, `468b5eac`/`f9966752`, `468b5eac`/`f9966752`, `8b6db745`, `53362106`, `f8d9e155`, `783d1b60`, `e6fa721b`) |
| `python3 -B -m unittest tools.tests.test_tpm` (`GIT_EDITOR` unset) | 0 | – | 10 tests; `TpmSeedTests` ran on swtpm 0.10.1 |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` (`GIT_EDITOR` unset) | 0 | – | 1181 tests, 15 skipped |
| `python3 -m compileall -q tools/run_tpm.py tools/tpm_seed.py tools/tests/test_tpm.py` | 0 | – | – |

The guest stdout SHA-256 is computed over the report's UTF-8 `stdout`. The
swtpm backend exited with status 0 on every boot, and no swtpm process remained
after the batch.

## Observed

- **Owner squat under the empty ownerAuth.** Seeded state: PERMANENT `0x400`,
  `HAUTH` attributes `0x02040004` with Name
  `000bfc25ca0b…45ab40055` (the probe S1 squat Name), `LREC` `0x18B`, lockout
  counter 0. Boot 0 is byte-identical to `loader-hauth` boot 0, so the owner
  undefine removed the squat silently and provisioning repeated. After boot 0,
  `HAUTH` has the frozen written Name `000bbd7dcf06…6f513f739ce2`
  (`NAME_HAUTH_WRITTEN`), `LREC` has `NAME_LREC_WRITTEN`, and PERMANENT is
  `0x405`. Boot 1 is byte-identical to `loader-hauth` boot 1. This confirms
  the code-path expectation recorded as unobserved in the seeded evidence.
- **DA-counter sensitivity.** The seed moved `0x20E` from 0 to 1 with
  `TPM_RC_AUTH_FAIL`, and the scratch index was removed (`NV_UndefineSpace`
  succeeded). The loader's provisioning boot differs from `loader-hauth`
  boot 0 in exactly one line: `LOADER-HAUTH:DA:…01:…01` instead of
  `…00:…00`. The loader's own reads before and after the refused owner command
  (`TPM_RC_BAD_AUTH` `0x9A2`) both return 1, and the post-boot snapshot also
  reads 1.
- On swtpm 0.10.1 the counter persisted across the swtpm restart between the
  seed and the boot, and no non-orderly-startup increment was observed: every
  other provisioned boot reads 0, including second boots after a provisioning
  boot that changed lockoutAuth.
- All seeded halt cases keep their public state, now including the lockout
  counter (0), unchanged across the halt.

## Limits

- The DA case shows that the loader's `0x20E` read reflects a DA-counted
  failure that happened before the boot. It does not show the counter moving
  between the loader's own two reads: the refused owner command is a hierarchy
  authorisation, which TPM 2.0 exempts from dictionary-attack counting, so
  "unchanged" around it is the expected result rather than a measured property
  of a DA-protected command.
- The counter persistence and the absence of a startup increment are observed
  on swtpm 0.10.1 only. `TpmSeedTests` checks the counter only on the seeding
  server itself, not across a restart, because the CI image runs swtpm 0.7.3.
- The empty-ownerAuth squat case shows replacement of an owner squat during a
  first provisioning. It says nothing about a platform squat (halts,
  `loader-hauth-squat`) or about an owner squat that an attacker plants after
  ownerAuth is set (needs that ownerAuth).
- Open item, unchanged: the TEST loader does not check that the ownerAuth
  equals `HAUTH[..32]`.

## NOT RUN

- Read-back of the SHA-1, SHA-384 and SHA-512 banks of PCR 13 and of the event
  log; the expected values are known (`H(0^n || H("BARYON-TRIAL-CLOSED-V1"))`;
  the SHA-256 value equals `P_CLOSED`), but the codec reads only the SHA-256
  bank.
- A halt in the provisioning path after an NV write (fault injection) and a
  power cut between the ownerAuth and lockoutAuth changes.
- swtpm 0.7.3, the hosted OVMF 2024.02 image and hosted CI for the loader-*
  cases.
- Physical TPM or firmware; independent human audit.
