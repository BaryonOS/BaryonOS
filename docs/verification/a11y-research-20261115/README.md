# UX-A11Y-R accessibility research and matrix: verification record (15 November 2026)

This record covers two documents, scoped by the project-owner decision of 15 November 2026 on accessibility (two tiers):

- `docs/design/accessibility/ACCESSIBILITY-RESEARCH.md` is a systematic literature review.
- `docs/design/accessibility/ACCESSIBILITY-MATRIX.md` is the matrix: 92 rows, each pairing a need with an adaptation, and 110 sources.

The evidence is literature, standards and an independent AI review. There was no participant testing, no human accessibility audit, no screen-reader run and no native run. An empirical grade summarises published studies; it is not a measured effect of BaryonOS.

| File | Content |
|---|---|
| `check.py` | Structural checker for the matrix. It covers headers, value sets, source ids and the needs the owner named. It enforces that safety rows are always baseline and that strong and moderate grades meet an evidence floor. It has a self-test. It does not judge evidence. |
| `check-output.json` | Output of `python3 -I -B docs/verification/a11y-research-20261115/check.py .` (exit 0) |
| `independent-check.json` | Final independent AI verification, round r2-1. Verdict: ACCEPT, with 0 blockers, 0 majors and 10 minors. Contains the citation audit (28 references), the normative checks (14) and the limitations. |
| `SHA256SUMS` | Hashes of the two documents and the files above |

## Matrix summary

These counts come from `check-output.json`.

| Field | Values |
|---|---|
| Tiers | 51 baseline, 41 setting; the 11 safety rows are all baseline |
| Empirical grade | 2 strong, 1 moderate, 51 limited, 37 insufficient, 1 contradicted |
| User test needed | 60 rows |

Rows per need named by the owner:

| Need | Rows |
|---|---|
| Colour vision | 5 |
| Low vision | 8 |
| Blindness | 8 |
| Light sensitivity | 5 |
| Photosensitive epilepsy | 5 |
| Vestibular | 4 |
| Dyslexia | 8 |
| Dyscalculia | 2 |
| ADHD | 3 |
| Autism | 3 |
| Motor | 18 |
| Hearing | 7 |

## Process

Work ran on a serial runner. In each run, a fresh writer did the work and a fresh verifier checked it without seeing the writer's reports. A run allowed at most two repair rounds. Workers did not stage or commit; the lead integrated afterwards.

| Run | Result |
|---|---|
| Research `wf_1c2c1a08-2d1` | Three cluster researchers (perception, cognition, interaction) wrote dossiers to the ignored `build/design-evidence/a11y/research/`. A planned shutdown interrupted the synthesis. |
| Run 1 `wf_4f6a5638-e4a` | The synthesis writer produced the three files. Verifier rounds 0–2 found 5, 2 and 1 majors, each a different defect, and no blockers. The run ended `blocked_after_2_repairs`; the remaining major concerned evidence grading. |
| Run 2 `wf_baf4e604-27b` | A repair writer edited the existing files, with one sweep for consistent grading and no regeneration of the matrix. Verifier r2-0 found 2 majors (both grading) and 6 minors. After repair round 1, verifier r2-1 accepted with 0 blockers, 0 majors and 10 minors. |

The final verifier ran nine checks:

1. Conformance with the owner decision.
2. Citation audit.
3. Normative accuracy against the W3C WCAG 2.2 text, EN 301 549 V4.1.1 and ITU-R BT.1702-3.
4. Evidence grading.
5. Tiers and axes.
6. Prototype and native status claims, checked by grep and line reads.
7. `check.py`, including mutation tests.
8. Repository boundaries.
9. Design ambition (labelled AI opinion).

It read the research document in full and the matrix as a parsed dump. It read in full every safety-tagged row and every row graded strong, moderate or contradicted.

## Open minors (not blocking, carried)

1. **Row 21:** the wording of the stripe-texture comfort bound conflicts between sections 2.2, 3.4 and 13. The reduced-motion progress stripes of ds-1.0 break that uncalibrated bound. Carried to UX-A11Y-DS, which redesigns the loading state.
2. **Row 53:** the strong grade for reminders after acquired brain injury needs a justification, or should be lowered to moderate.
3. **Section 4, colour-blind correcting glasses:** graded contradicted on PER-GOMEZ18, whose sample size is unconfirmed.
4. **`check.py`:** it does not pin the non-safety mandatory baseline rows, for example "no keyboard trap".
5. **Row 39:** the grade column takes on a second meaning here, because it grades the rejected dyslexia-font claim.
6. **Row 24:** it does not apply the ITU-R BT.1702-3 Guideline 1 clause on flashing sequences longer than 5 s. UX-A11Y-DS applies that clause in the design system.
7. **Section 3.5, Q2:** one empirical bullet has no source id.
8. **Row 26:** offering the per-item reveal for third-party flashing media needs an explicit project-owner decision as well as a clinical safety review. The reveal is currently off and outside the baseline guarantee.
9. **Rows 68–69:** INT-JACOB1991 resolves by metadata only; no grade depends on it.
10. **Section 12, priority 3:** the vestibular user test should also cover whether the exclusion of bounded large-field zoom transitions as a category can be relaxed (AI design opinion).

## Limitations

- **Citations:** the final audit re-resolved 28 of about 90 peer-reviewed references. These included every reference that is the sole support for a strong, moderate or contradicted grade or for a safety limit. Earlier rounds re-resolved samples of 29 and 33 references. The final round did not re-check the remaining references, the platform documentation or the W3C notes.
- **Standards:** the ISO and IEEE texts are paywalled and were not read in full. The harmonisation status of EN 301 549 V4.1.1 was not checked.
- **Prototype and native status:** the prototype was not rendered in this task. Native status reflects contract documents and code reads, not running code.
- **Ignored files:** raw dossiers, generator scripts and verifier scratch files are in the ignored `build/design-evidence/a11y/` directory. Run journals and their hashes are in the ignored design-pipeline checkpoint.
