# B03-REL05 M1-HAUTH: TEST loader BOUND provisioning on swtpm (25 November 2026)

**Native TEST experiment, not acceptance evidence.** It runs the host model's
`loader()` block (`crates/update-core/tests/anchor_chain_model.rs`) up to
BOUND in the real UEFI loader against a disposable swtpm, then a second boot on
the same TPM state. It accepts no criterion and grants no progress points.
First provisioning is **unauthorised and TEST-only**: S3b (EK, signed
`ENROLL`), genesis, ACCEPTED, the UEFI copy and U28 are not built.

- Source: commit `1a46fa21` on `claude/b03-rel05-tpm-20261125` (local, not
  pushed under the CI-first lane rule), `tools/common.source_hash()`
  `225dde0f273e801c68168c3e480b3bb5af3409a1fcc8dab21fa34ec807cbe6cb`.
- Profile: loader feature `tpm-hauth-provision-test` (implies the read-only
  `tpm-loader-probe-test`), unchanged `native-tpm-crb` kernel, the
  `loader-rng` QEMU line (TEST `virtio-rng-pci` for `EFI_RNG_PROTOCOL`, U25).
  QEMU 10.2.1, `pc-q35-10.0`, TCG, `-cpu qemu64`, 1 CPU, 256 MiB; OVMF
  `OVMF_CODE_4M.fd` SHA-256 `874f15a7c9110e78…` (Ubuntu 2025.11); swtpm 0.10.1.
- Loader `BOOTX64.EFI` SHA-256
  `f6be0dc809dbb166ca407f74305049a4de4052d41e6db51a800e3aab4abfe059`; kernel
  SHA-256 `ba349c3b83755ec466809d2e509dcb4d825cf8946393c9303024b75ed2b55fda`
  (identical to the S6 step-1 and step-4 evidence).

## Loader order (`boot/loader/src/tpm_hauth.rs`)

1. `TPM_PT_PERMANENT`; PCR 13 (SHA-256) must be zero.
2. ownerAuth unset only: owner `NV_UndefineSpace` of `HAUTH` and `LREC`
   tolerating exactly `0x28B` (`TPM_RC_HANDLE`, handle 2); define `HAUTH`,
   compare the unwritten Name `NAME_HAUTH`; owner-write 64 bytes from
   `EFI_RNG_PROTOCOL`; define `LREC`; policy-write `LoaderRecord` BOUND with
   `hauth_binding`; `HierarchyChangeAuth(owner, HAUTH[..32])`.
3. Every boot: compare `NAME_HAUTH_WRITTEN`, read `HAUTH` under
   `PolicyPCR(13 = 0³²) + PolicyCommandCode(NV_Read)`, compare
   `NAME_LREC_WRITTEN`, read `LREC` and require BOUND with the binding of the
   value just read; lockoutAuth unset only: `HierarchyChangeAuth(lockout, HAUTH[32..])`.
4. Every path that reached the TPM: `HashLogExtendEvent(13, "BARYON-TRIAL-CLOSED-V1")`
   (`EV_IPL`, flags 0); any earlier failure then stops the boot (exit 0x11).
5. After the extend: SHA-256 PCR 13 must equal `P_CLOSED`; a fresh policy read
   of `HAUTH` must fail with exactly `0x99D` (`TPM_RC_POLICY_FAIL`, session
   flushed); an empty-ownerAuth owner undefine of `HAUTH` must fail with
   exactly `0x9A2` (`TPM_RC_BAD_AUTH`); the CRB data buffer must still begin
   with that 10-byte error response (the scan reads the real buffer) and
   must contain neither 32-byte half of the secret.

Command and response buffers of every frame and the secret are wiped with
volatile writes when dropped. Markers carry results only.

## Commands and results

Run sequentially in one checkout at `1a46fa21`:

| Command | Exit | Raw QEMU exit | Guest stdout SHA-256 |
|---|---|---|---|
| `python3 -B tools/run_tpm.py --case loader-hauth` boot 0 | 0 | 33 | `037bc4642e37455657929e7c32bb9f7931ddf9afab1c4f0b1ede5d2ce4ddd4fb` |
| same run, boot 1 (same swtpm state) | – | 33 | `9480c6e57931ed6de877df6d9d255b017ae6e28950b4cf66a53ff8b5115d8d80` |
| `python3 -B tools/run_tpm.py --case loader-probe` | 0 | 33 | `5336210692e0d50627d0084586c586e13fe46aa5aa4e98f2076223d199a498de` |
| `python3 -B tools/run_tpm.py --case loader-rng` | 0 | 33 | `f8d9e155bf8ce42c78ebfd5e34d89c400addec233b0f8aa0c3d3172098bd7cc9` |
| `python3 -B tools/run_tpm.py --case normal` | 0 | 33 | `783d1b60ee0dbb6219b2f5e89a432f0d1d24f257f68300acfa8f852f37dd1b6c` |
| `python3 -B tools/run_tpm.py --case absent` | 0 | 33 | `e6fa721b3827f35cf17c3c8704a2a156e01ab8161190135195ac25087c20c3c8` |
| `python3 -B -m unittest tools.tests.test_tpm` (`GIT_EDITOR` unset) | 0 | – | 7 tests |
| `cargo +1.90.0 test --locked --manifest-path drivers/tpm-crb/Cargo.toml` | 0 | – | 25 + 20 tests |
| `cargo +1.100.0 test --locked -p update-core`; `clippy … -p update-core --all-targets -- -D warnings` | 0 | – | – |

Boot-specific loader lines (identical in three separate runs, each with a new
random secret, so no output depends on the secret):

| Line | Boot 0 | Boot 1 |
|---|---|---|
| `LOADER-HAUTH:PERMANENT` | `0x400` | `0x405` (ownerAuthSet, lockoutAuthSet) |
| mode | `MODE:PROVISION` | `MODE:EXISTING` |
| lockout | `LOCKOUT:SET` | `LOCKOUT:KEPT` |

Both boots print `BOUND`, `PCR13:CLOSED`, `READ:REFUSED`, `OWNER:REFUSED`,
`RESIDUE:NONE`, `PASS`, and the unchanged kernel then completes
`TPMCRB:PASS` and the IPC-wait markers. The four existing cases keep the
guest stdout hashes pinned in
[b03-rel05-loader-rng-20261125](../b03-rel05-loader-rng-20261125/README.md),
although the probe module was refactored to share its TCG2 helpers.

## Reading

- U2 (SHA-256 bank of PCR 13): firmware `HashLogExtendEvent` hashes
  `DataToHash` and extends PCR 13 so that the SHA-256 bank equals the frozen
  `P_CLOSED`; the loader needs no hash code of its own for the extend.
- `0x28B` is the "not defined" answer on a fresh TPM (the only tolerated
  non-success; boot 0 passed). `0x9A2` rather than `0x98E` (`TPM_RC_AUTH_FAIL`)
  indicates that the refused owner command is not counted towards dictionary-
  attack lockout (TPM 2.0 Part 1, hierarchy authValues); the lockout hierarchy
  is never tried with a wrong value.
- `RESIDUE:NONE` is observed at the loader's last TPM command, before
  `ExitBootServices`, in the QEMU CRB buffer only.

## NOT RUN

- Native negative cases: squatted or unwritten `HAUTH`/`LREC`, an unbound
  `LREC`, a platform squat, foreign ownerAuth, and a power cut between
  provisioning steps (covered only by the host model).
- Read-back of the SHA-1, SHA-384 and SHA-512 banks of PCR 13 and of the TCG
  event log entry (delegated to firmware, not observed).
- Wiping of the SHA-256 working state inside `hauth_binding`, of firmware or
  TCG2 driver copies, and of QEMU/swtpm process memory; the CRB buffer after
  firmware's own `ExitBootServices` commands.
- S3b, genesis, ACCEPTED, the UEFI copy, U22 write atomicity, U28, S3/U27,
  REL-05 section 7.9, native S5 and N's joint tests.
- Hosted CI: `native-system.yml` runs only `normal` and `absent`
  (`.github/**` is protected; the hosted OVMF 2024.02 image is not on this host).
- Physical TPM or firmware; independent human audit.
