# Logotype review record, 14 November 2026

Results of the review fixed in [PREREGISTRATION.md](PREREGISTRATION.md)
(committed in `1ace2981` before the judges ran). Every score here is AI
judgement by an independent reviewer agent, not user testing and not a human
design review.

## Files

| File | Content |
|---|---|
| `PREREGISTRATION.md` | Candidates, criteria, weights and decision rule, committed before judging |
| `STIMULI.sha256` | SHA-256 of the 43 files given to the judges (copied outside the repository; regenerate with `python3 -I ui/design/brand/logotype_study.py OUT --blind` at `1ace2981`) |
| `judge1.json`, `judge2.json` | The two judges' structured results as returned, unedited. Judge 1 reviewed L→S, judge 2 S→L |

Both judges opened all 43 files and judged small sizes from the 1:1 crops.

## Code key

L: oxanium-twotone · M: inter-twotone · N: inter-heavy-wide · P: exo2-twotone ·
Q: archivo-v1 · R: saira-wide-twotone · S: inter-weight

## Scores

Mean of the two judges per criterion; weighted total = Σ score × weight / 10
(L 30, F 25, C 15, H 15, D 15).

| Code | Variant | Route | L | F | C | H | D | Total | Judge 1 / 2 |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| R | saira-wide-twotone | b | 7.0 | 7.5 | 8.5 | 5.5 | 8.0 | **72.75** | 75.5 / 70.0 |
| P | exo2-twotone | b | 7.5 | 7.0 | 6.0 | 7.0 | 6.0 | 68.50 | 70.0 / 67.0 |
| N | inter-heavy-wide | a | 9.0 | 7.5 | 3.5 | 8.5 | 2.5 | 67.50 | 65.5 / 69.5 |
| L | oxanium-twotone | b | 6.5 | 6.5 | 7.0 | 6.0 | 6.5 | 65.00 | 68.5 / 61.5 |
| M | inter-twotone | a | 8.5 | 6.5 | 3.0 | 9.0 | 2.0 | 62.75 | 61.5 / 64.0 |
| S | inter-weight | a | 8.0 | 6.0 | 2.0 | 8.5 | 3.5 | 60.00 | 60.0 / 60.0 |
| Q | archivo-v1 | c | 6.5 | 5.5 | 3.5 | 5.5 | 7.5 | 58.00 | 56.0 / 60.0 |

Rankings: judge 1 R, L, P, N, S, Q, M; judge 2 R, P, N, L, M, Q, S. Both
named R as closest to the original wordmark.

## Applying the rule

- No code has a mean L of 5 or lower.
- B = R (72.75). A = N (67.50), the best route-(a) code.
- total(B) − total(A) = 5.25, not below 3.0, so the margin condition fails.
  (L and F would have passed: 9.0 ≥ 6.5 and 7.5 ≥ 7.0.)
- **Selected: R, Saira 600 at 125 % width, "OS" in the secondary text colour,
  tracking 0, a word space between "Baryon" and "OS".** Both judges' top code
  is R, so rule 5 does not apply.

The outcome is route (b). The project owner's brief states a preference for a
unified family across wordmark and interface; this result is a pairing (Saira
wordmark, Inter interface) and is therefore recorded as a departure that the
project owner can redirect. Redirecting to route (a) would select N
(Inter 700, +0.03 em). The pre-registration's rationale paragraph described the
route-(a) margin as a cost judgement "not a stated preference of the project
owner"; that sentence understated the brief, which does state the unified-family
preference. The rule itself was not changed.

## What the judges flagged on R

- At 14 px on the dark theme the secondary "OS" recedes (both judges), although
  its contrast is 9.38:1 (`#A9B2CB` on `#070912`, computed with the WCAG
  formula); the cause is stroke weight at that size, not colour contrast.
- The wide setting makes small lockups long, and the boot line wordmark is much
  wider than the mark above it (both).
- Beside 15 px Inter interface text it reads as a separate brand voice
  (H 5.5, the lowest of the leaders).
- Lighter and looser than the original's tight bold setting (judge 2); a risk of
  a dated technical feel unless spacing is tuned at large sizes (judge 1). The
  judges read the 125 % width as "tracking"; the tracking was 0.

These are open refinement items for the wordmark (BRAND.md section 4). None
was tuned after the judging; the generated assets use the judged setting.

## Known limits of the record

- Judges recognised Inter (M, N, S) with high confidence, Exo 2 (P) with
  moderate confidence, and R only as "Saira-like" or "Saira or Exo kind" with
  low or low-to-moderate confidence. Route (a) was therefore not blind.
- Three variants per route; the route with more variants had more chances to
  produce the top code. Route (b) won with one variant, R.
- The "Over BaryonOS" panel and the boot line are study surfaces, not the
  Orbitaal board; integration into Orbitaal v2 is checked separately.
- Two AI judges; agreement on the winner is reported, not tested
  statistically. The judges disagree by up to 7 points on single codes (L).
- The reference original is one dark raster at one size, so continuity was
  judged at large size only.
