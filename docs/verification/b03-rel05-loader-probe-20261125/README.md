# B03-REL05 S6 step 1: loader TCG2/RNG probe (25 November 2026)

**Native TEST probe, not acceptance evidence.** The `tpm-loader-probe-test`
loader calls firmware `EFI_TCG2_PROTOCOL` and `EFI_RNG_PROTOCOL` before
`ExitBootServices`, then boots the unchanged `native-tpm-crb` kernel. It sends
only read-only TPM commands (no NV, PCR or hierarchy change), prints no RNG
bytes and holds no keys. It answers U19 and parts of U2 and U25 of
[B03-REL05-TRIAL-BOOT-TPM.md](../../implementation/B03-REL05-TRIAL-BOOT-TPM.md)
section 14 for this QEMU profile only. It accepts no criterion and grants no
progress points.

- Source: branch `claude/b03-rel05-tpm-20261125`, parent `bc5c25e9`, run on
  the uncommitted step-1 tree with `tools/common.source_hash()`
  `7a8c250bfc11233a2ce17e89c572a8bea7dea853543f331e776e7da3cedf3c5d`.
- swtpm 0.10.1 (fresh state per run, `startup-clear`), QEMU 10.2.1, machine
  `pc-q35-10.0`, TCG, `-cpu qemu64`, 1 CPU, 256 MiB, no RNG device.
- OVMF: Ubuntu package `ovmf` 2025.11-3ubuntu7.3, `OVMF_CODE_4M.fd` SHA-256
  `874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7`.
- Loader `BOOTX64.EFI` SHA-256
  `5f6038d6f7c83cc0b2ef2e99811f9bcefb82515cb0427c9fa11e0de8faee6fba`;
  kernel SHA-256
  `ba349c3b83755ec466809d2e509dcb4d825cf8946393c9303024b75ed2b55fda`.
- Raw logs and reports lived under the ignored `build/`; only this extract is
  committed. Full guest stdout SHA-256:
  `5336210692e0d50627d0084586c586e13fe46aa5aa4e98f2076223d199a498de`.

## Commands and results

Run sequentially in one checkout:

| Command | Exit | Raw QEMU exit |
|---|---|---|
| `python3 -B tools/run_tpm.py --case loader-probe` | 0 | 33 |
| `python3 -B tools/run_tpm.py --case normal` | 0 | 33 |
| `python3 -B tools/run_tpm.py --case absent` | 0 | 33 |

Loader lines of the `loader-probe` serial output, in order, before
`BOOT:EBS_OK`:

```text
LOADER-TPM:TCG2:PRESENT
LOADER-TPM:BANKS:000000000000000f
LOADER-TPM:PERMANENT:0000000000000400
LOADER-TPM:PCR13:ZERO
LOADER-TPM:PASS
LOADER-TPM:RNG:ABSENT
```

The same run then reached `M2:PASS`, `TPMCRB:RELEASED:PASS` and
`TPMCRB:PASS` with `TPMCRB:FAMILY` `0x322E3000` and no quarantine.

## Reading

- U19: r-efi 7.2.0 defines no `EFI_TCG2_PROTOCOL`; the loader's own
  `repr(C)` table (`boot/loader/src/tpm_probe.rs`) located the protocol and
  called `GetActivePcrBanks` and `SubmitCommand` successfully.
- U2 (part): `SubmitCommand` carried `GetCapability(TPM_PT_PERMANENT)` and
  `PCR_Read(13)` built and decoded by the tpm-crb codec. Active banks are
  SHA-1, SHA-256, SHA-384 and SHA-512 (`0xf`), matching the four banks of the
  earlier probe. SHA-256 PCR 13 is zero in the loader phase. The kernel CRB
  driver completes its property query after the loader used TCG2, so the CRB
  is usable after `ExitBootServices` in this profile. Not examined:
  `HashLogExtendEvent`, extension of every bank, and PCR 13 after the loader.
- `TPMA_PERMANENT` `0x400`: only `tpmGeneratedEPS`; owner, endorsement and
  lockout authorisation are unset, no `disableClear`, not in lockout.
- U25: `EFI_RNG_PROTOCOL` is **absent** in this configuration (`-cpu qemu64`,
  no RNG device). The cause (no RDRAND for the firmware RNG driver, no
  virtio-rng device) is inferred, not probed. Under the specified rule M1-HAUTH
  provisioning halts here; the profile needs an entropy source before
  provisioning can run (S6 step 4).

## Limits

One local run per case on one host; hosted CI does not run the
`loader-probe` case. No physical TPM, no other firmware build, no S3, no
provisioning or record code, no fault injection.
