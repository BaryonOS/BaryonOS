# B03-REL05 U26: loader record `LREC` on swtpm (25 November 2026)

**Feasibility probe, not acceptance evidence.** Raw TPM 2.0 commands against
a disposable swtpm; no loader, kernel or firmware is involved. It checks the
public area and write behaviour of the M1-HAUTH loader record `LREC`
designed for U26 in
[COLD-FRESHNESS-ANCHOR.md](../../implementation/COLD-FRESHNESS-ANCHOR.md) and
[B03-REL05-TRIAL-BOOT-TPM.md](../../implementation/B03-REL05-TRIAL-BOOT-TPM.md)
sections 6.1 and 6.2. It accepts no criterion and grants no progress points.

- `LREC`: handle `0x0100B305`, ordinary, 64 bytes, attributes `0x02041008`
  (`ATTR_UPDATE_ORDINARY`), `authPolicy` `POLICY_LOADER_WRITE`
  `088d4acf1b7ad7d6a53a44d97668736524219be4e7148b45be4dcc3d00462816`; the
  same template as `TRIAL`/`FLOOR`, which U1/U7 proved at `0x0100B300`
  ([b03-rel05-probe-20261125](../b03-rel05-probe-20261125/README.md)).
- swtpm 0.10.1 (`TPM emulator version 0.10.1`), fresh state, flags
  `not-need-init,startup-clear`; no firmware, so PCR 13 starts at zero.
- The record bytes are a fixed test pattern; no secret is used.
- `probe.py` SHA-256 `3d109ca92143ca5b6ebfa97d13c656e8b2fd6a582c8b97e8a078a78f0af0d3cc`;
  `transcript-lrec.txt` SHA-256
  `f76f7220ffd5e89caff3d98a0debdba833b7224b2be0cc7cca1d74495b3454ec`.

## Command and result

| Command | Exit | Checks |
|---|---|---|
| `python3 -B docs/verification/b03-rel05-lrec-probe-20261125/probe.py <disposable dir>` | 0 | 16/16 PASS |

| Check | Observation |
|---|---|
| L0 | `POLICY_LOADER_WRITE` and both frozen Names recomputed from the public area |
| L1 | definition with the frozen public area; unwritten Name `000bad14c18b2e6f42a9e51a1cdfb8d6ef84d0dba6fcfd0598852bde1504447c4782` |
| L2 | owner `NV_Write` refused (`TPM_RC_NV_AUTHORIZATION`); index-password `NV_Write` refused (`TPM_RC_AUTH_UNAVAILABLE`, `0x12F`) |
| L3 | policy `NV_Write` of 64 bytes at PCR 13 = 0³² accepted; written Name `000b282efe987c36f93dc5464cfc70b668f196f74e7bcf62ace9c1a3757ef81f7a60`; `NV_Read` with `AUTHREAD` and empty `authValue` returns the record |
| L4 | after a PCR 13 extend the policy write is refused (`TPM_RC_POLICY_FAIL`) and the record is unchanged |
| L5 | with an empty ownerAuth (as after a clear) an owner undefine and redefine with the frozen public area succeed, the policy write is refused, and the index keeps the unwritten Name |

The Names equal the values pinned in
`crates/update-core/tests/trial_tpm_constants.rs` (`nv_names_frozen`).

## Reading

L5 is the basis of the trust rule: after the PCR 13 extend nobody can give an
index at `0x0100B305` the written Name, so a loader that trusts `LREC` only
with the written Name never trusts a record defined at run time. The first
L2 expectation in the probe draft (`TPM_RC_NV_AUTHORIZATION` for the
index-password write) was wrong; the TPM reports `TPM_RC_AUTH_UNAVAILABLE`,
which is also a refusal, and the probe now pins that code.

## Limits

One run on swtpm 0.10.1 without firmware. Not run: OVMF, the hosted swtpm
0.7.3, the loader code that reads and writes `LREC`, power-loss atomicity of
the write (U22), physical TPMs.
