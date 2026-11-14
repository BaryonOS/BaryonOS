# S1 professional capability matrix — verification record (14 November 2026)

Document: `docs/design/suite/CAPABILITY-MATRIX.md` (215 rows: 14 prototyped (host),
201 proposed, 0 passed (native)). Evidence level: source analysis and host
evidence cited per row; no native run and no participant testing.

| File | Content |
|---|---|
| `check.py` | Validates the matrix tables and counts shared dependencies |
| `check-output.json` | `python3 -I docs/verification/suite-s1-capability-matrix-20261114/check.py . docs/design/suite/CAPABILITY-MATRIX.md` (exit 0) |
| `evidence-inventory.md` | Evidence inventory written by the researcher (predates the repair; see matrix open item 13) |
| `independent-check.json` | Independent AI verification of the repaired matrix: ACCEPT, 0 blockers, 0 majors, 5 minors |
| `SHA256SUMS` | Hashes of the files above |

The independent check is an AI review, not a human audit. Its five minor
findings remain open.
