# B03-REL05 M1-HAUTH: pre-seeded negative cases on swtpm (25 November 2026)

**Native TEST experiment, not acceptance evidence.** It runs the TEST loader
M1-HAUTH provisioning
([b03-rel05-hauth-bound-20261125](../b03-rel05-hauth-bound-20261125/README.md),
[b03-rel05-hauth-halt-20261125](../b03-rel05-hauth-halt-20261125/README.md))
on swtpm states that were prepared before the boot. It accepts no criterion
and grants no progress points.

- Source: commit `38453a5f` on `claude/b03-rel05-tpm-20261125` (local, not
  pushed under the CI-first lane rule), `tools/common.source_hash()`
  `bb0910022be5668c2971ade9c3531f342c1ff4bda5eaeed4d6ed0422fc95220f`. The
  first four seeded cases were introduced in `4e2eda89` (source `a9bd4cef`)
  and keep their stdout hashes at `38453a5f`.
- Profile and stack: identical to the halt evidence (QEMU 10.2.1,
  `pc-q35-10.0`, TCG, `-cpu qemu64`, 1 CPU, 256 MiB; local Ubuntu OVMF 2025.11;
  swtpm 0.10.1).
- Loader `BOOTX64.EFI` SHA-256 `5bbcc2fa7dd1b810c15ce3b4285e54795d98b975698765ea9196c991bb942271`;
  kernel SHA-256 `ba349c3b83755ec466809d2e509dcb4d825cf8946393c9303024b75ed2b55fda`.
  Both are identical to the halt evidence: this slice changes no Rust code.

## Method

`tools/tpm_seed.py` starts a short-lived swtpm with a command socket on the
case's disposable state directory, sends raw TPM 2.0 commands while PCR 13 is
at its reset value, and stops it with `CMD_SHUTDOWN` before the QEMU boot. That
position is the boot-path attacker T5, which stays out of scope until B04; the
seeds only make the loader's existing checks reachable. The helper prints no
command bytes. The foreign ownerAuth is a fresh TEST value and is discarded.

Before and after every boot of a seeded case, the runner records public facts
only: `TPMA_PERMANENT`, the public area (attributes and Name) of `HAUTH` and
`LREC` or the `NV_ReadPublic` response code, and the SHA-256 of a written
`LREC`, whose contents are not secret. A halt boot must leave these facts
unchanged.

| Case | Seed | Expected loader result |
|---|---|---|
| `loader-hauth-squat` | `HAUTH` defined by the platform hierarchy (`PLATFORMCREATE`, `AUTHREAD`, `AUTHWRITE`, `NO_DA`); empty platformAuth before firmware (C21) | `MODE:PROVISION`; the owner undefine is refused; halt before any random value, definition or hierarchy change |
| `loader-hauth-foreign-owner` | ownerAuth set to a foreign value; no `HAUTH` or `LREC` | `MODE:EXISTING`; `NV_ReadPublic(HAUTH)` fails; halt |
| `loader-hauth-owner-squat` | `HAUTH` defined by the owner hierarchy (`AUTHWRITE`, `AUTHREAD`, `NO_DA`, as in probe S1), then a foreign ownerAuth that keeps it | `MODE:EXISTING`; the `HAUTH` Name differs from the frozen written Name; halt `PUBLIC` |
| `loader-hauth-unbound` | boot 0 provisions as `loader-hauth`; then one flipped byte (offset 48, the binding) in the well-formed `LREC`, written back under its own policy | boot 1: `MODE:EXISTING`; `HAUTH` read and both Names pass; halt `UNBOUND` |
| `loader-hauth-cut` | power cut after defining `LREC`: `HAUTH` defined with the frozen template and owner-written with random bytes, `LREC` defined and unwritten, no hierarchy auth | boot 0 removes both leftovers and provisions; boot 1 verifies |

## Commands and results

Run sequentially in one checkout at source `bb091002` (uncommitted, identical
in content to `38453a5f`):

| Command | Exit | Raw QEMU exit | Guest stdout SHA-256 |
|---|---|---|---|
| `python3 -B tools/run_tpm.py --case loader-hauth-squat` | 0 | 35 | `81a89795e06c320e0f0c3511f53eeed4cacb0d16816a436f4601035960c13c7a` |
| `python3 -B tools/run_tpm.py --case loader-hauth-foreign-owner` | 0 | 35 | `65c97d205e60848230afcf41059fb4de5d1261c7786584e7a57de8cd485af8a4` |
| `python3 -B tools/run_tpm.py --case loader-hauth-owner-squat` | 0 | 35 | `fa928907f2fb5812e84ab82209ba3417b0152019198367a855c3a91e04250ad0` |
| `python3 -B tools/run_tpm.py --case loader-hauth-unbound` boot 0 | 0 | 33 | `468b5eac8b5f8b62d480086a126613b3952523db558e76a98eaa9458aca38e53` |
| same run, boot 1 (after the seed) | – | 35 | `37de5e2a36866748c7edecbbec5a5d878e79cef1cf4ba5a6ac8e3d996939d3ac` |
| `python3 -B tools/run_tpm.py --case loader-hauth-cut` boot 0 | 0 | 33 | `468b5eac8b5f8b62d480086a126613b3952523db558e76a98eaa9458aca38e53` |
| same run, boot 1 | – | 33 | `f99667520dd46e805f969bd76f87cee7c44a950076c9f7a0f812e4cecae5279e` |
| `python3 -B tools/run_tpm.py --case loader-hauth` (boots 0, 1) | 0 | 33, 33 | `468b5eac…`, `f9966752…` (as above) |
| `python3 -B tools/run_tpm.py --case loader-hauth-no-rng` | 0 | 35 | `8b6db745637dde3820e7c2c7a42d680f75ca541adaa18fe2a5d222c7fbf2f0ff` |
| `python3 -B tools/run_tpm.py --case loader-probe` | 0 | 33 | `5336210692e0d50627d0084586c586e13fe46aa5aa4e98f2076223d199a498de` |
| `python3 -B tools/run_tpm.py --case loader-rng` | 0 | 33 | `f8d9e155bf8ce42c78ebfd5e34d89c400addec233b0f8aa0c3d3172098bd7cc9` |
| `python3 -B tools/run_tpm.py --case normal` | 0 | 33 | `783d1b60ee0dbb6219b2f5e89a432f0d1d24f257f68300acfa8f852f37dd1b6c` |
| `python3 -B tools/run_tpm.py --case absent` | 0 | 33 | `e6fa721b3827f35cf17c3c8704a2a156e01ab8161190135195ac25087c20c3c8` |
| `python3 -B -m unittest tools.tests.test_tpm` (`GIT_EDITOR` unset) | 0 | – | 10 tests |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` | 0 | – | 1181 tests, 15 skipped |
| `python3 -m compileall -q tools/run_tpm.py tools/tpm_seed.py tools/tests/test_tpm.py` | 0 | – | – |
| `ci.yml#host-quality` Python subset at `38453a5f`: `compileall` over the CI list; the 13 `tests/` `unittest discover` patterns (including the runner and QEMU harness patterns); `package_token_guard.py`; `validate_skill.py`; os-dev tests; `progress.py --check` | 0 each | – | 652 `tests/` tests (2 skipped), 18 os-dev tests; 43/100 consistent |

The guest stdout SHA-256 is computed over the report's UTF-8 `stdout`.

## Observed

- **Squat.** Lines: `BEGIN`, `PERMANENT:0x400`, `MODE:PROVISION`,
  `HALT:PCR13:CLOSED`, `FAIL:TPM_RC:0x149` (`TPM_RC_NV_AUTHORIZATION`: owner
  authority cannot delete a platform-created index). The seeded public state
  (`HAUTH` attributes `0x42040004`, `LREC` `0x18B`, PERMANENT `0x400`) is
  unchanged after the halt.
- **Foreign ownerAuth.** Lines: `BEGIN`, `PERMANENT:0x401`, `MODE:EXISTING`,
  `HALT:PCR13:CLOSED`, `FAIL:TPM_RC:0x18B` (`TPM_RC_HANDLE`, handle 1, from
  `NV_ReadPublic` of the undefined `HAUTH`). State unchanged.
- **Owner squat under a foreign ownerAuth.** Lines: `BEGIN`,
  `PERMANENT:0x401`, `MODE:EXISTING`, `HALT:PCR13:CLOSED`,
  `FAIL:PUBLIC:0x0100B304`: the `HAUTH` public area (attributes `0x02040004`)
  does not yield `NAME_HAUTH_WRITTEN`. The seeded public state (`HAUTH`
  attributes and Name, `LREC` `0x18B`, PERMANENT `0x401`) is unchanged after
  the halt.
- **Unbound `LREC`.** Boot 0 is byte-identical to `loader-hauth` boot 0. The
  seed changes only byte 48 of `LREC`; the written Name stays
  `NAME_LREC_WRITTEN`. Boot 1 lines: `BEGIN`, `PERMANENT:0x405`,
  `MODE:EXISTING`, `HALT:PCR13:CLOSED`, `FAIL:UNBOUND`. There is no `BOUND`
  line and no lockout change, and the `LREC` digest is the seeded one after the
  halt. This is a halt after `HAUTH` and `LREC` were written, on the
  verification path.
- **Cut.** The seeded `HAUTH` has the Name `NAME_HAUTH_WRITTEN`. Boot 0 is
  byte-identical to `loader-hauth` boot 0, so both leftover undefines succeeded
  silently and provisioning repeated. After it, PERMANENT is `0x405` and `LREC`
  is written with a new digest. Boot 1 is byte-identical to `loader-hauth`
  boot 1.
- The six existing cases keep the guest stdout hashes pinned in the halt
  evidence, so the runner refactor does not change their observations. The
  loader evidence of `1438acd9` thereby carries over to source `bb091002`.
- `tools/tests/test_tpm.py::TpmSeedTests` (skipped without swtpm) checks the
  seeds against the frozen Names parsed from
  `crates/update-core/src/trial_tpm.rs`.

## Limits

- The halt cases show where the TEST loader stops, not that a product loader
  recovers. The host model instead clears a foreign hierarchy under a newer
  record or reports `NeedsRecovery`; S3b, genesis, ACCEPTED and the UEFI copy
  are not built.
- The loader never checks that the ownerAuth equals `HAUTH[..32]`. A foreign
  owner that also plants a well-formed `HAUTH` (owner write) and a matching
  `LREC` (policy write at PCR 13 = 0) would be reported `BOUND`. That needs
  loader-phase TPM access (T5) or a previous OS that never extends PCR 13. It
  is reported as an open item, not tested.
- An owner-created squat under an empty ownerAuth goes through the same owner
  undefine as the cut leftovers and is then replaced by provisioning; that
  variant was not run as a separate case. Under a foreign ownerAuth it halts
  `PUBLIC` (`loader-hauth-owner-squat`).
- The seeds run on swtpm with an empty platformAuth before firmware. Whether
  firmware leaves the platform hierarchy reachable on real systems (U24) is
  not observed.

## NOT RUN

- A halt in the provisioning path after `HAUTH` or `LREC` was written; it
  needs fault injection between commands, not a pre-seeded state.
- A power cut between the ownerAuth and lockoutAuth changes. Its pre-seeded
  form needs a host-held HAUTH secret and a host `LoaderRecord` encoder,
  which the helper does not implement.
- A DA-counter sensitivity check; read-back of the SHA-1, SHA-384 and SHA-512
  banks and of the event log.
- Hosted OVMF 2024.02 and hosted CI for the loader-* cases.
- Physical TPM or firmware; independent human audit.
