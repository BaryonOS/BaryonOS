# B03-REL05-P0 TPM feasibility probe (25 November 2026)

**Feasibility probe, not acceptance evidence.** This probe tests three
unverified assumptions of
[B03-REL05-TRIAL-BOOT-TPM.md](../../implementation/B03-REL05-TRIAL-BOOT-TPM.md)
(section 14: U1, U2, U7) against a disposable software TPM before any
implementation. It does not exercise the loader, the kernel, a physical TPM or
any product code. It accepts no criterion and grants no progress points.

- Source baseline: branch `claude/b03-rel05-tpm-20261125`, head `9068f045` (uncommitted probe).
- swtpm 0.10.1, libtpms 0.10.2-2ubuntu1 (Ubuntu package), QEMU 10.2.1
  (Debian 1:10.2.1+ds-1ubuntu3.3), machine `pc-q35-10.0`, TCG, `-cpu qemu64`.
- OVMF: `/usr/share/OVMF/OVMF_CODE_4M.fd` selected by `tools/common.firmware()`,
  Ubuntu package `ovmf` 2025.11-3ubuntu7.3, SHA-256
  `874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7`.
- Python 3.14.4. State, sockets and raw logs lived under the ignored
  `build/run/rel05-probe/`; only extracts and their hashes are committed.

## Method

`probe.py` (U1, U7) starts `swtpm socket --tpm2 --server type=unixio --ctrl
type=unixio --flags not-need-init,startup-clear` on a fresh state directory and
sends raw TPM 2.0 command bytes; it uses no tpm2-tools and computes no HMAC. The
locality is never changed through the control channel, so every command runs
at swtpm's default locality 0. The indices are defined exactly as spec 6.1
proposes: handles `0x0100B300`–`0x0100B303`, `nameAlg` SHA-256,
`POLICYWRITE | AUTHREAD | NO_DA` (+ `WRITEALL` on ordinary indices, `TPM_NT_COUNTER`
on `GEN`), sizes 64/64/64/8, and an `authPolicy` computed host-side with
`hashlib` by the spec formula
`H(H(0³² ‖ CC_PolicyPCR ‖ pcrSelect{SHA256,13} ‖ H(pcr13)) ‖ CC_PolicyCommandCode ‖ cc)`.
The sequence runs every PCR 13 = 0 leg, then extends PCR 13 (SHA-256 bank only)
with `H("BARYON-TRIAL-OPEN-V1")`, then every PCR 13 = `P_OPEN` leg, then U7.
Each refused write has a positive twin built by the same function. Every
session that a failed command left loaded is flushed (all `FlushContext`
calls returned success, so a failed authorisation does **not** consume the
session).

Policy-session authorisation area used for `NV_Write`/`NV_Increment`
(compare with the password form `40000009 0000 01 0000` in `tpm_fixture.rs` and
the CRB contract):

```
authSize=00000009 | sessionHandle=03000000 | nonceCaller=0000 (empty)
| sessionAttributes=00 (continueSession clear) | hmac=0000 (empty)
```

The response carries `nonceTPM` (16 B), attributes `00` and an empty HMAC, so
the caller has no response HMAC to verify. `StartAuthSession`:
`tpmKey = bind = TPM_RH_NULL`, 16-byte constant `nonceCaller`, empty salt,
`TPM_SE_POLICY`, symmetric `TPM_ALG_NULL`, `authHash` SHA-256. `PolicyPCR` is sent
with an **empty** `pcrDigest` (TPM uses current values).

`u2.py` (U2) boots the same OVMF with a second disposable swtpm via
`-chardev socket … -tpmdev emulator … -device tpm-crb`, no disk, no NIC, serial on
a Unix socket. With no boot option OVMF enters its Boot Manager menu; the probe
selects "EFI Internal Shell" and runs `dh -p 607F766C-7455-42BE-930B-E4D76DB2720F`
(the `EFI_TCG2_PROTOCOL` GUID from edk2 `MdePkg/Include/Protocol/Tcg2Protocol.h`).
swtpm ran with `--log level=20`; `swtpm_log_extract.py` decodes OVMF's TPM
command stream from that log.

## Verdicts

| ID | Verdict (on swtpm 0.10.1 / libtpms 0.10.2) | Decisive evidence |
|---|---|---|
| U1 | **PROVEN** | `PolicyGetDigest` equals the host constant and `NV_ReadPublic.authPolicy` for all four indices. With empty nonce and empty HMAC: `NV_Write` TRIAL/FLOOR and `NV_Increment` GEN at PCR 13 = 0 → `0x000`; `NV_Write` CONFIRM at `P_OPEN` → `0x000`; read-back matches. Negatives: CONFIRM at PCR 13 = 0, TRIAL/FLOOR/GEN at `P_OPEN`, and a NV_Write-bound session used for `NV_Increment` → `0x99D` `TPM_RC_POLICY_FAIL` (session 1). Session satisfied at PCR 13 = 0, then PCR extended, then write → `0x128` `TPM_RC_PCR_CHANGED`. |
| U7 | **PROVEN** | `PCR_Reset(13)` at locality 0 → `0x907` `TPM_RC_LOCALITY`; PCR 13 still equals `P_OPEN` afterwards. Controls: `PCR_Reset(16)` and `PCR_Reset(23)` → `0x000`. |
| U2 | **PARTIAL: presence PROVEN; semantics DEFERRED to S6** | Shell `dh -p <GUID>` printed "Handle dump by protocol 'Tcg2' / 40: Tcg2": handle 0x40 carries `EFI_TCG2_PROTOCOL` in the Shell, i.e. before any ExitBootServices. The firmware command stream shows `Startup`, `SelfTest`, PCR_Extend into PCRs 0–7 only, each listing all four active banks (sha1, sha256, sha384, sha512); PCR 13 is never extended. `HashLogExtendEvent` hashing semantics, its bank coverage when called by the loader, and CRB idle state at EBS were **not** tested. |

## Incidental observations (not verdicts)

- U3: `StartAuthSession` with a 0-byte or 15-byte `nonceCaller` → `0x1D5`
  `TPM_RC_SIZE` (parameter 1); 16 bytes is accepted. A constant non-secret
  nonce worked for the unbound, unsalted session.
- U5: `PCR_Extend` listing only the SHA-256 bank succeeded on this swtpm,
  whose fresh state has four banks active (OVMF's `GetCapability(TPM_CAP_PCRS)`
  reported sha1/sha256/sha384/sha512 on). `PolicyPCR` with empty `pcrDigest` used
  the current value (U1 positives depend on it).
- U8: OVMF issues `HierarchyChangeAuth(TPM_RH_PLATFORM)` with a 64-byte new
  value before handover, so platform auth is not empty after boot.
- A 32-byte all-zero HMAC on an otherwise valid policy session is rejected
  with `0x9A2` `TPM_RC_BAD_AUTH`; the HMAC must be empty, not merely ignored.
- The wrong-command-code case returns `TPM_RC_POLICY_FAIL`, not
  `TPM_RC_POLICY_CC`, because the session digest already differs from the
  index's `authPolicy`.
- A failed policy authorisation leaves the session loaded; S6/S7 must flush
  it (or rely on `continueSession` clear only for the success path).
- `BdsDxe` waits for a key at "Press any key" when no boot option exists; the
  probe sends one keystroke.

## Spec impact

No refutation. No change to section 6.1 or the state machine is required by
U1/U7. For S6, the remaining U2 check must be done by the loader itself:
locate `EFI_TCG2_PROTOCOL`, call `GetCapability`/`GetActivePcrBanks`, call
`HashLogExtendEvent(PCR 13, "BARYON-TRIAL-OPEN-V1")`, then read PCR 13
(SHA-256) through `SubmitCommand` and compare with `P_OPEN`
(`501c196a…3696c0`); also confirm the CRB is idle after ExitBootServices.
Because four banks are active, the kernel's SEAL extend should either list all
active banks or the spec should state explicitly that only the SHA-256 bank of
PCR 13 is policy-relevant (the probe shows a SHA-256-only extend is accepted).

## Files

- `probe.py` — U1/U7 raw-command probe; `transcript-u1u7.txt` — its full output
  (every command and response in hex, decoded response codes, summary).
- `u2.py` — U2 OVMF/QEMU probe; `swtpm_log_extract.py` — swtpm log decoder;
  `transcript-u2.txt` — u2 stdout, cleaned serial console, decoded firmware
  command stream, SHA-256 of raw logs kept under `build/run/rel05-probe/`.
  The raw swtpm log is not committed: it contains the platform auth value
  OVMF chose for that disposable TPM.
- `SHA256SUMS`.

Reproduce: `python3 -B probe.py <workdir>` and
`python3 -B u2.py <workdir> <repo-root>` with `<workdir>` a disposable directory
under the checkout's `build/run/` (short path; Unix socket name limit). The
sandbox in which this ran denied signals to child processes, so both scripts
stop swtpm through the control channel (`CMD_SHUTDOWN`) and QEMU through the
monitor (`quit`).
