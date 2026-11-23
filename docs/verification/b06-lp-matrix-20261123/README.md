# LP evidence reference matrix, revision 20261123

Created 23 November 2026 for task B06-T4 (criterion B06, Native privacy contract).
This revision supersedes `docs/verification/b06-lp-matrix-20261115/` as the default
of `python3 -B tools/lp_matrix.py --check`. The earlier revision is unchanged and is
still validated with
`python3 -B tools/lp_matrix.py --check --matrix docs/verification/b06-lp-matrix-20261115/matrix.json`.

**Evidence limit.** This is a structural mapping of previously executed and
archived native QEMU runs. No new native run was performed for this revision, no
historical record was modified or re-audited beyond the checks listed below, and
B06 remains open. N04 also remains open in `docs/progress.json`. B03 is accepted.
A green `--check` validates references, digests and the native record fields. It
does not establish record authenticity, complete-scenario coverage or privacy
compliance.

## Meaning of `native` in this revision

The 20261115 README defines `none` as "no complete-scenario evidence mapped". In
this revision, `native` means: **the core denial in the LP row's "Required result"
column was exercised in an archived, passing, source-bound native QEMU run**. It
does **not** mean that the complete LP scenario is covered. The `limit` string in
the `--check` output states the same qualifier. A row is not mapped as `native` when
the evidence covers only one of several distinct artifact classes or paths that
the row names (see LP-12). Every part of the row
that the run did not exercise is named in the row's `remaining` text. The matrix
schema (version 1) has only `native`, `host` and `none`; there is no partial status.
The existing `evidence` (path plus SHA-256) and `remaining` fields therefore hold the
evidence pointer and the residual text. No field was added, so the validator was not
loosened.

## Changes compared with 20261115

Only two of 31 rows changed: LP-10 became `native`, and LP-12 stayed `none` with a
new residual text that records the partial B03 update-path evidence. The other 29
rows are byte-identical in content (checked by `tools/tests/test_lp_matrix.py`).
Counts: native 0 → 1, host 0 → 0, none 31 → 30.

### LP-10: permission revoked, old handle retained → `native`

Contract: "Future operations denied; earlier external copies noted separately".

Evidence: `docs/verification/n04-native-revoke-20261105/revoke-report.json`,
SHA-256 `5ee8097baf6932245c8eee9e66fd1e3e0069c0d77f05aab1c00fed11f6249652` (matches
that directory's `SHA256SUMS`). Record fields: `passed: true`, `timed_out: false`,
`raw_qemu_exit: 33`, `case: revoke`, `kernel_feature: test-net-service-revoke`,
`source_commit: a2a89daf21ffe3b407919955563bc5d1e72b540c`, `source_sha256:
b7c23420…`. The record was archived on `main` by squash `1db01c4a` (#215).

Why the core denial is exercised:
- App A holds a live TCP connection inside its own destination grant (port 8080)
  when an authenticated policy process revokes the grant.
- The `tools/run_network_service.py` revoke oracle requires app A offset 432 = 5
  (five denied uses after revocation: send, receive, new open, UDP send, UDP
  receive) and offset 504 < 500 ticks (Closed observed promptly). The record has
  `APP_A` 432 = 5, 424 = 2, 544 = 2 and 504 = 2.
- Causal order on one boot-monotonic clock, as the runner requires:
  request 57 ≤ policy 57 ≤ service revoke 58 ≤ app A Closed 59.
- The host peer transcript records `8080` with phase `revoked` and
  `revoke_rsts: 1`. The N04 README states that no further SYN, data, FIN or UDP
  from app A reached the peer after revocation.

Residual (also in `remaining`):
- Only network socket handles are covered. No document, NAS, key or other file
  handle is tested after revocation.
- The requirement "earlier external copies noted separately" is not exercised.
  There is no user-visible notice about data that was already sent.
- There is a single synthetic localhost Ethernet peer under QEMU TCG. No physical
  hardware is involved.
- The N04 README describes the record as an evidence proposal, not N04 acceptance.

### LP-12: modified model/package/update → stays `none`

Contract: "No execution without successful authenticity/integrity verification" for
a modified model, package or update.

The row names three distinct artifact classes with separate verifiers. Native
evidence exists for only one of them, so the row is not mapped as `native`:
- Modified update: archived B03 native runs (accepted criterion) exercise the
  denial for the OS update path.
- Modified package (third-party app): no native verifier evidence; A02 is open.
- Modified model: no native model loader exists (B06-T3 is blocked). The B06-T1
  ledger entry also states that native passes for AI rows stay open.

Partial update-path evidence, recorded in `remaining` with its digests (each
matches `docs/verification/b03/SHA256SUMS`; each record has `passed: true`,
`timed_out: false`, raw exit 33, source fingerprint `07994d47…`):
- `docs/verification/b03/3-bad-signature/result.json`
  (`1a3773d0011ab0bf77055b4f427b706e0e65e7a0f5e32e19ef4b3733325e2ac5`):
  one ML-DSA signature byte of the slot-B kernel flipped; the loader emits
  `BOOT:SLOT:B:REJECTED:SIGNATURE` and boots verified slot A. Slot B content itself
  is intact in this stage.
- `docs/verification/b03/4-bad-digest/result.json`
  (`d6aefeb0226f35257c50775c420a1a80cd0592a2748d41af677c63295b46b43f`):
  `SLOT:B:REJECTED:DIGEST`, the most literal "modified update" record.
- `docs/verification/b03/5-bad-elf/result.json`
  (`1c9a13e7e49658944dce426bf43e262bf12e0dc8a85232793e1c850c57ea802d`):
  `SLOT:B:REJECTED:FORMAT`.
- `docs/verification/b03/10-inbox-signature/result.json`
  (`72f185ad3de94992c7abe967377532349838ed01143568d8e6e743898305ffad`):
  update service `INBOX_REJECT` 3 with zero writes.

These runs used QEMU TCG with the TEST signing keys in `tools/update-keys/`, with no
physical hardware or production signing. The record has no commit field;
`docs/verification/B03-ACCEPTANCE.md` binds it to branch `ccr-43832ef7-gi2qlz`, base
`main` `f63a0c1`, archived on `main` by squash `9f017354` (#61).

## Rows considered and left `none`

No other row was evaluated for remapping in this task. The 29 unchanged rows keep
their 20261115 residual text.

## Validation commands

- `python3 -B -m unittest discover -s tools/tests -p test_lp_matrix.py -v`
- `python3 -B tools/lp_matrix.py --check`
- `python3 -B tools/lp_matrix.py --check --matrix docs/verification/b06-lp-matrix-20261115/matrix.json`
- `python3 -m compileall -q tools/lp_matrix.py`

The tests include negative cases. A native row that points at a record with
`raw_qemu_exit` 35 or `passed: false` is rejected. This covers the archived
`b03/9-no-valid-slot` raw-exit-35 record and mutated copies of the B03
`3-bad-signature` record with recomputed digests, placed in an isolated copy of the
repository where LP-12 is promoted to `native` only for that synthetic test.
