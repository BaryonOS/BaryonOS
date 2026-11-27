# B03-REL05 M1-HAUTH: halt path and DA counter on swtpm (25 November 2026)

**Native TEST experiment, not acceptance evidence.** It extends the TEST loader
M1-HAUTH BOUND provisioning
([b03-rel05-hauth-bound-20261125](../b03-rel05-hauth-bound-20261125/README.md),
bound to `1a46fa21`) with two observations:

- the dictionary-attack lockout counter around the refused owner command;
- one native halt case after the PCR 13 `CLOSED` extend.

It accepts no criterion and grants no progress points.

- Source: commit `89d00679` on `claude/b03-rel05-tpm-20261125` (local, not
  pushed under the CI-first lane rule), `tools/common.source_hash()`
  `1438acd9c5e0172f498a6b6f26dd27f1674b4c27cd055e694d261b9766c21332`.
- Profile and stack: identical to the BOUND evidence (QEMU 10.2.1,
  `pc-q35-10.0`, TCG, `-cpu qemu64`, 1 CPU, 256 MiB; local Ubuntu OVMF 2025.11;
  swtpm 0.10.1).
- Loader `BOOTX64.EFI` SHA-256 `5bbcc2fa7dd1b810c15ce3b4285e54795d98b975698765ea9196c991bb942271`; kernel SHA-256 `ba349c3b83755ec466809d2e509dcb4d825cf8946393c9303024b75ed2b55fda`.

## Changes measured

1. **DA counter.** `TPM_PT_LOCKOUT_COUNTER` is `0x20E`: `PT_VAR + 14`, where
   `PT_VAR = PT_GROUP * 2 = 0x200` (libtpms `stable-0.10`,
   `src/tpm2/TpmTypes.h`, read 25 November 2026). The loader reads it before
   and after the refused empty-ownerAuth owner `NV_UndefineSpace`, prints
   `LOADER-HAUTH:DA:<before>:<after>`, and halts with `DA_COUNTED` if the value
   changed.
2. **Residue scan anchor.** The scan now follows that final `GetCapability`
   read. Before scanning, the loader checks that the CRB buffer begins with
   that response: the 10-byte header (size 27, success), then capability 6,
   count 1 and property `0x20E`. Byte 10 (`moreData`) is not compared.
3. **Halt path.** On a failure after a successful `HashLogExtendEvent`, the
   loader prints `LOADER-HAUTH:HALT:PCR13:{CLOSED|OTHER|UNREADABLE}` before the
   original `LOADER-HAUTH:FAIL:` line and `exit(0x11)`.
4. **`loader-hauth-no-rng`.** This case uses the same loader and kernel without
   the TEST virtio-rng device. On a fresh swtpm the loader:
   - undefines the leftovers (`0x28B` each);
   - finds no `EFI_RNG_PROTOCOL`;
   - extends `CLOSED`;
   - halts before any secret, NV definition or hierarchy change.

   The runner requires all of the following:
   - the exact `LOADER-HAUTH:` line sequence, ending in `HALT:PCR13:CLOSED`
     and `FAIL:RNG_ABSENT`;
   - raw exit 35, the value `tools/test.py` pins for loader failures;
   - no `KERNEL:`, `M1:PASS`, `M2:`, `TPMCRB:` or `IPCWAIT:` line.

## Commands and results

Run sequentially in one checkout at `89d00679`:

| Command | Exit | Raw QEMU exit | Guest stdout SHA-256 |
|---|---|---|---|
| `python3 -B tools/run_tpm.py --case loader-hauth` boot 0 | 0 | 33 | `468b5eac8b5f8b62d480086a126613b3952523db558e76a98eaa9458aca38e53` |
| same run, boot 1 (same swtpm state) | – | 33 | `f99667520dd46e805f969bd76f87cee7c44a950076c9f7a0f812e4cecae5279e` |
| `python3 -B tools/run_tpm.py --case loader-hauth-no-rng` | 0 | 35 | `8b6db745637dde3820e7c2c7a42d680f75ca541adaa18fe2a5d222c7fbf2f0ff` |
| `python3 -B tools/run_tpm.py --case loader-probe` | 0 | 33 | `5336210692e0d50627d0084586c586e13fe46aa5aa4e98f2076223d199a498de` |
| `python3 -B tools/run_tpm.py --case loader-rng` | 0 | 33 | `f8d9e155bf8ce42c78ebfd5e34d89c400addec233b0f8aa0c3d3172098bd7cc9` |
| `python3 -B tools/run_tpm.py --case normal` | 0 | 33 | `783d1b60ee0dbb6219b2f5e89a432f0d1d24f257f68300acfa8f852f37dd1b6c` |
| `python3 -B tools/run_tpm.py --case absent` | 0 | 33 | `e6fa721b3827f35cf17c3c8704a2a156e01ab8161190135195ac25087c20c3c8` |
| `loader-hauth` again, separate run (fresh secret) | 0 | 33, 33 | identical to both boots above |
| `python3 -B -m unittest tools.tests.test_tpm` (`GIT_EDITOR` unset) | 0 | – | 8 tests |
| `python3 -B tools/build.py --loader-feature tpm-hauth-provision-test --kernel-feature native-tpm-crb` | 0 | – | – |
| `cargo +1.100.0 fmt --manifest-path boot/loader/Cargo.toml -- --check` | 0 | – | – |

The guest stdout SHA-256 is computed over the report's UTF-8 `stdout`. `cargo
clippy` on the loader (`x86_64-unknown-uefi`) reports only the two existing
`not_unsafe_ptr_arg_deref` errors in `main_default.rs`, none in
`tpm_hauth.rs` or `tpm_probe.rs`; the loader is not part of the CI clippy set.

## Reading

- Both `loader-hauth` boots print `LOADER-HAUTH:DA:0000000000000000:0000000000000000`.
  The refused empty-ownerAuth owner command (`0x9A2`) left the lockout counter
  at 0 in this state, as the host probe in
  [b03-rel05-hauth-probe-20261125](../b03-rel05-hauth-probe-20261125/probe.py)
  also saw after one wrong ownerAuth. The counter stayed 0 throughout, so this
  run does not prove that `0x20E` moves on a DA-counted failure (see NOT RUN).
- Apart from the new `DA` line, the `loader-hauth` lines are identical to the
  BOUND evidence. That line is the only reason the two stdout hashes differ.
- `loader-hauth-no-rng` observes the halt order natively: firmware TCG2 is
  present, the leftover undefines pass, `EFI_RNG_PROTOCOL` is absent, PCR 13
  (SHA-256) equals `P_CLOSED` after the extend, and the loader stops with raw
  exit 35 before `ExitBootServices`; the kernel never runs. No secret exists
  at that point.
- The four existing cases keep the guest stdout hashes pinned in
  [b03-rel05-loader-rng-20261125](../b03-rel05-loader-rng-20261125/README.md).

## NOT RUN

- Native negative cases that need a pre-seeded swtpm state: a squatted or
  unwritten `HAUTH`/`LREC`, an unbound `LREC`, a platform squat, foreign
  ownerAuth, and a power cut between provisioning steps.
- A halt after `HAUTH` or `LREC` was written. The only halt observed fails
  before any NV or hierarchy change.
- A sensitivity check that a DA-counted failure moves the `0x20E` value. That
  needs a wrong authorisation on a DA-protected object in a disposable state.
- Read-back of the SHA-1, SHA-384 and SHA-512 banks and of the event log.
- Hosted OVMF 2024.02 and hosted CI for the loader-* cases.
- Physical TPM or firmware; independent human audit.
