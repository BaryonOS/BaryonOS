# S2 consumer-to-professional maturity model — verification record (18 November 2026)

Document: `docs/design/suite/MATURITY-MODEL.md`
(sha256 `6a610de33b09ca8b874afef91a8d940cf4340f7fcf467b1ca2fe78e0127356a6`).
The model defines 5 levels, 15 dimensions (75 cells), progression rules RULE-1 to
RULE-12, a section 6 mapping of all 216 S1 rows, and 46 proposed acceptance criteria.
Evidence level: design research and proposal. No user test was run, no ADR exists,
and no native run was made. S1 evidence levels are unchanged:
14 prototyped (host), 202 proposed, 0 passed (native).

| File | Content |
|---|---|
| `check.py` | Consistency checker written during authoring. The `check <repo>` mode validates S2 against S1: names, evidence levels, exactly-once coverage, the dimension × level cells, Basis citations, criterion coverage, the section 8.2 lists, cross-references and source URLs. The `gen` mode rewrites the section 6 block; it is an authoring helper and was not used for verification. |
| `check-output.txt` | `python3 -I -B docs/verification/suite-s2-maturity-model-20261118/check.py check .` run by the lead from the worktree root (exit 0, PASS) |
| `independent-check.json` | Final independent AI verification (round 2): 0 blockers, 0 majors, 4 minors; 9 external sources checked, none unsupported |
| `SHA256SUMS` | Hashes of the files above and of the document |

Process: one writer, an independent verifier per round, and at most 2 repairs
(workflow `wf_0bcaef43-719`). The verifier findings per round, as blockers/majors/minors,
were 1/4/1, then 0/1/6, then 0/0/4. Every blocker and major was repaired and re-verified.
The 4 remaining minors (D05.L5 coverage wording, AC-X.2 versus the RULE-7 recent-commands
group, the strength of the E4 claim, and the S1 read-depth statement) are open items for
the next revision of S2.

Limits:
- The verification is an AI review, not a human audit or user test.
- The checker cannot decide whether a criterion fully exercises a cell.
- 25 cells have no criterion and 16 are tested only in part (section 8.2, item 14).
- The authoring mutation tests (7/7 in round 1, 5/5 in round 2) remain under the
  ignored `build/design-evidence/suite-s2/` and are not stored here.

Open owner items named by the document, all undecided:
- O1: product performance budgets.
- O2: grant scope for scripts, batch jobs and plugins.
- P-NAMES-1: level names stay provisional.
- The S1 XC09 permission prompt wording differs from ADR-DESIGN-OWNER-20261115 D2.
