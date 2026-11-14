# Typography review record, 13 November 2026

Evidence for [TYPOGRAPHY.md](../../design/TYPOGRAPHY.md) sections 6 and 7. Every
review here is AI judgement by an independent reviewer agent, not user testing
and not a human design review. Scores are judgements, not measurements.

## Files

| File | Content |
|---|---|
| `round1.json` | Round 1: eight browser specimens (letter codes) and eight native rows (digit codes), one reviewer |
| `round2.json` | Round 2: three finalists at equal x-height, codes V, Q, J |
| `round2b.json` | Round 2b: replicate of round 2 on recaptured stimuli, fresh reviewer, 1:1 viewing |
| `native2.json` | Native column re-judge: the three finalists rasterised as the native atlas would, digit codes |
| `STIMULI.sha256` | SHA-256 of the stimulus images as they were on disk when this record was written: round 1 (`blind/`), round 2b (`blind2/`) and the native re-judge (`native2/`). The images themselves are not committed |
| `finalist-checks.txt` | Output of `python3 -I ui/design/typography/finalist_checks.py` (coverage, zero glyphs, word-space gaps, pair distinctness at token sizes, I/l rasters) |

The JSON files are the reviewers' structured results as returned, unedited.
The stimuli were generated from `ui/design/typography/specimens/index.html`
and `ui/design/typography/native_specimen.py --blind` into the git-ignored
`ui/design/typography/candidates/{blind,blind2,native2}/`.

## Code keys

The keys were withheld from the reviewers and are published here after the
reviews were complete. Native digit codes follow from the script: rows are
ordered by the SHA-256 of the font file name, and that ordering was recomputed
when this record was written.

| Round | Code: face |
|---|---|
| 1, browser | D: Exo 2 · H: Saira · K: Atkinson Hyperlegible Next · M: Funnel Sans · N: Sora · R: Inter · T: Archivo · W: Inter UI with a Saira wordmark line |
| 1, native | 1: Archivo · 2: Atkinson Hyperlegible Next · 3: Funnel Sans · 4: Sora · 5: Space Grotesk · 6: Saira · 7: Inter · 8: Exo 2 |
| 2 and 2b | V: Exo 2 (17 px) · Q: Inter (15 px) · J: Atkinson Hyperlegible Next (16 px) |
| Native re-judge | 1: Atkinson Hyperlegible Next · 2: Inter · 3: Exo 2 |

## Score calculation

Weights from the brief: readability and recognition 35, accessibility and
language 20, brand 20, OS consistency 15, native, performance and licensing 10.
A 1–10 score contributes score × weight / 10. The reviewers' wordmark-fit score
is reported but not weighted; the logotype is decided in BRAND.md.

| Face | Round 2b R / A / B / O | Blind part (of 90) | Native re-judge | Total (of 100) | Round 2 blind part |
|---|---|---:|---:|---:|---:|
| Atkinson Hyperlegible Next | 7.8 / 8 / 5.8 / 7 | 65.40 | 7.6 | 73.00 | 60.25 |
| Inter | 8 / 7.5 / 5 / 8.5 | 65.75 | 6.7 | 72.45 | 65.75 |
| Exo 2 | 6 / 5.8 / 8.5 / 5.5 | 57.85 | 7.0 | 64.85 | 62.75 |

## Known defects of the record

- Round 1: a browser tab visible to the reviewer showed code M's URL; M is not
  blind. All faces were set at equal pixel size, not equal x-height.
- Round 1: W shares its UI face with R, so three of W's 2x quadrants are
  byte-identical to R's (visible as duplicate hashes); only W's wordmark line differs.
- Round 2: the reviewer reported that three of J's four 100 % frames showed the
  content of `J-apps-dark-1x.png`, a capture error by the lead. The frames were
  recaptured for round 2b, which overwrote the defective files, so round 2's
  stimuli are not preserved and its J scores rest on 200 % images only.
- Rounds 2 and 2b: the reviewers tentatively recognised Q as Inter and V as an
  Exo 2-like face; those two codes were blind in name only.
- The native re-judge reviewer noted that its own visual input downsampled the
  2x images; pixel-level claims were therefore checked separately
  (`finalist-checks.txt`, TYPOGRAPHY.md section 6.4).
