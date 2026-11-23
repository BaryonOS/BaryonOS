# Accessibility matrix baseline contract successor — 19 November 2026

This checkpoint replaces the pinned baseline contract of
[`../a11y-checker-20261118/`](../a11y-checker-20261118/README.md) after the reviewed
UX-A11Y-R amendment A1 (flashing media and photosensitivity, owner decisions of
15 November 2026 and owner decision B of 19 November 2026). The 18 November contract,
checker archive and evidence remain unchanged as history.

[`baseline-contract.json`](baseline-contract.json) records the Need and Adaptation
identities of all 53 baseline rows of the A1 matrix (SHA-256
`fcc050d75e5af812c07d6621f1dc2da1dcbe69283b593b28d970cd4b9626d044`), built on
source commit `8210db5c29e1a78e82c18ff3ee9c8c43d0467634`. Compared with the 51
identities of 18 November:

- one identity changed: the photosensitive-epilepsy safety row for unsolicited
  content (matrix row 26). A1 withdrew the per-item reveal and states that
  protection follows the unsolicited placement (owner decision B). The row keeps tier
  `baseline` and axis `none (baseline)`;
- two identities were added: the A1 baseline rows on informative notices for
  deliberately started media and games, and on the separation of reduced motion
  from flash attenuation;
- no baseline row was removed, demoted or reclassified.

`tools/check_a11y_matrix.py` pins this file by SHA-256
`9bdab7b19ac0c10c5e6c1d45e2bd513ce8c7868cf01bd98bc9ce07e5992a00c9`. The pinned
historical checker `docs/verification/a11y-research-20261115/check.py`
(`cf1b65d1…`) is unchanged. The baseline count asserted in
`tools/tests/test_a11y_matrix.py` changes from 51 to 53.

Verification on the working tree before commit, with raw logs kept in ignored `build/design-evidence/a11y/a1r/`:

| Command | Exit | Result |
|---|---|---|
| `python3 -B tools/check_a11y_matrix.py .` | 0 | ok, 96 rows, 121 sources, 0 errors |
| `python3 -B -m unittest tools.tests.test_a11y_matrix` | 0 | 15 tests OK |
| `python3 -m compileall -q tools/check_a11y_matrix.py` | 0 | — |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` | 1 | 545 tests run, 6 errors in `test_agent_isolation`/`test_agent_pipeline`, all `Git environment overrides require a verified controller adapter` from the session's Git environment variables; unrelated to this change, not a pass |

Independent verification of A1: workflow run `wf_bc36fa86-cf9`, status accepted
after one repair round, 0 blockers, 0 majors and 5 minors. A sample of 17 citations
was re-resolved with 0 mismatches. That verification is an AI review, not a human
audit. This structural gate does not validate research quality, UI conformance or
any native criterion. Hosted checks on the pushed head remain a separate
requirement. Score unchanged.
