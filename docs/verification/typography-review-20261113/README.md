# Typography review record, 13 November 2026

Evidence for [TYPOGRAPHY.md](../../design/TYPOGRAPHY.md) sections 6 and 7. Every
review here is AI judgement by an independent reviewer agent, not user testing
and not a human design review. Scores are judgements, not measurements.

## Files

| File | Content |
|---|---|
| `round1.json` | Round 1: eight browser specimens (letter codes) and eight native rows (digit codes), one reviewer |
| `round2.json` | Round 2: three finalists at equal x-height, codes V, Q, J |
| `round2b.json` | Round 2b: replicate of round 2 after three J frames were recaptured, fresh reviewer, browser viewport at 1:1 (see Known defects) |
| `native2.json` | Native column re-judge: the three finalists rasterised as the native atlas would, digit codes |
| `STIMULI.sha256` | SHA-256 of the stimulus images as they were on disk when this record was written: round 1 (`blind/`), round 2b (`blind2/`) and the native re-judge (`native2/`). The images themselves are not committed |
| `finalist-checks.txt` | Output of `python3 -I ui/design/typography/finalist_checks.py` (coverage, zero glyphs, word-space gaps per role line and over all four specimen lines with the sites of ties and collapses, pair distinctness at the token sizes as a two-size mean and per size, I/l rasters) |

The JSON files are the reviewers' structured results as returned, unedited.
The stimuli were generated from `ui/design/typography/specimens/index.html`
and `ui/design/typography/native_specimen.py --blind` into the git-ignored
`ui/design/typography/candidates/{blind,blind2,native2}/`. The round-1 native
sheets (`blind/blind-*.png`) reproduce byte for byte with `native_specimen.py`
at commit `2cc2df1f` and the eight round-1 font files gathered in one
directory; the native re-judge sheets (`native2/`) with the script at
`a99f4d88` and the three finalist font files. The `index.html` version used
for the browser captures of either round is not recorded.

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

Without the native column Inter leads (65.75 against 65.40), so the total's
leader is decided by the native column: both the re-judge, whose reviewer states
that its scores are relative, not absolute, and the round-1 native values
(Atkinson 7, Inter 6; totals 72.40 and 71.75) put Atkinson first.

## Known defects of the record

- Round 1: a browser tab visible to the reviewer showed code M's URL; M is not
  blind. All faces were set at equal pixel size, not equal x-height.
- Round 1: W shares its UI face with R, so three of W's 2x quadrants are
  byte-identical to R's (visible as duplicate hashes); only W's wordmark line differs.
- Round 2: the reviewer reported that three of J's four 100 % frames showed the
  content of `J-apps-dark-1x.png`, a capture error by the lead. Those three
  frames were recaptured for round 2b and overwrote the defective files, so the
  defective images are not preserved; according to the file modification
  times, the other stimuli are the ones round 2 saw. Round 2's J judgements for
  light and shell rest on 200 % images only, and its reviewer called J's second
  place provisional.
- Round 2b: the browser viewport matched each image at 1:1, but the reviewer
  reports that its screenshot tool scaled a1, a3, a5, s3 and s4 to about
  0.6–0.8× and a7 to about 0.92× before it saw them, and that it did not zoom.
  By its own account its I/l judgements are tentative, and for the downscaled
  images it judged I/l cues and 5/S/8/B partly from the 1× frames and the
  unscaled 2× images.
- Rounds 2 and 2b: only the UI, reading and secondary text were at equal
  x-height. In the committed `index.html`, captions (12 px), subheads (18 px),
  headings (28 px), the clock (56 px), the confusable rows (13 and 11 px), the
  Notities body (16 px) and the wordmark line (40 px) kept one pixel size for
  all faces, so reviewer claims about glyphs in those elements compare the
  faces at unequal x-height. Reading text used a line height of 1.55 × size,
  so the leading differences follow from the size.
- Rounds 2 and 2b: round 2's reviewer said Q and V "strongly" resemble Inter and
  Exo 2; round 2b recognised them with moderate or low confidence. Those two
  codes were blind in name only.
- Rounds 2 and 2b: the file-table fit numbers in TYPOGRAPHY.md section 6.2 (a
  DOM measurement in Chrome) have no script or stored output; they are an
  unrecorded observation and are not used in the comparison.
- Combination rule: the rule, its amendment for the native re-judge, all scores
  and the selection first appear together in commit `6d5678d0`; no
  timestamped copy of either rule text exists here.
- The native re-judge reviewer noted that its own visual input downsampled the
  2x images; pixel-level claims were therefore checked separately
  (`finalist-checks.txt`, TYPOGRAPHY.md section 6.4). It guessed wrongly that
  Specimen 1 was Inter and tentatively and correctly recognised Specimen 3 as
  Exo 2.
