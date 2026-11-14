# BaryonOS typography

Status: research, measurements, specimen review, selection (Inter, a judgement the
project owner can redirect) and token specification `type-1.0` complete; native
integration not started. Dates: 13–14 November 2026 (Europe/Amsterdam). Design
study on branch `claude/baryon-os-ui-ux-design-152adb`.

This record keeps four kinds of statement apart and labels each:

- **Finding**: what a cited source reports, paraphrased from its abstract or text.
- **Measurement**: a reproducible number produced by a script in this repository.
- **Judgement**: an aesthetic or product decision; not evidence.
- **Assumption**: believed but not tested here.

No typeface is claimed to be best for every reader; finding F1 shows that no such
typeface exists. No user testing with participants has taken place.

## 1. Requirement

One recognisable typography system connecting the logo, the desktop, the OS
surfaces and the first-party apps. Consumer readability and accessibility take
priority over decorative futuristic styling. One family across wordmark and UI is
preferred by the project owner; a display/UI pairing is to be evaluated, not
assumed. The original wordmark (`ui/design/brand/reference/original-logo.webp`)
stays as reference and reversible fallback.

## 2. Research findings

Every source below was checked against OpenAlex (title, first author, year) on
13 November 2026, and each finding is limited to what the fetched abstract states.
Sources whose abstract was not available are not used for claims.

| # | Finding | Source | Implication for BaryonOS |
|---|---|---|---|
| F1 | Among hundreds of crowdsourced adult readers, reading speed differed by 35 % between an individual's fastest and slowest font, without loss of comprehension; the high variability across fonts indicates that one font does not fit all. | Wallace et al. 2022, [10.1145/3502222](https://doi.org/10.1145/3502222) | Ship a default, plus a user-selectable reading face and spacing setting. Never market the default as best for everyone. |
| F2 | In a driving-simulator menu task, a humanist typeface reduced total glance time by 10.6 % compared with a square-grotesque typeface among men; the effect was smaller or absent among women; error rates were 3.1 % lower for both. | Reimer et al. 2014, [10.1080/00140139.2014.940000](https://doi.org/10.1080/00140139.2014.940000) | Square-grotesque styling is a measured risk for UI text read at a glance. |
| F3 | In a glance-like lexical decision task, the humanist style was significantly more legible than the square grotesque, typeface interacted with age, and black-on-white text was more legible than white-on-black. | Dobres et al. 2016, [10.1080/00140139.2015.1137637](https://doi.org/10.1080/00140139.2015.1137637) | As F2; older readers are affected more under hard conditions. |
| F4 | Eight popular sans-serif typefaces differed measurably in glance legibility; typography is "more than a matter of taste" in glance contexts. | Sawyer et al. 2020, [10.1080/00140139.2020.1714748](https://doi.org/10.1080/00140139.2020.1714748) | Typeface choice is a usability decision and is measured here, not only judged. |
| F5 | Variants of frequently misrecognised letters differed in legibility at equal size, weight and personality; narrow letters benefited from widening, and x-height characters from using more of the ascender and descender area. | Beier & Larson 2010, [10.1075/idj.18.2.03bei](https://doi.org/10.1075/idj.18.2.03bei) | Distinct forms for confusable characters are a design lever; measured as pair distinctness in section 4. |
| F6 | Bolder letters improved recognition at small visual angles but not at large ones; very thin strokes and extreme boldness reduced recognition at every tested size. | Beier & Oderkerk 2019, [10.1016/j.actpsy.2019.102904](https://doi.org/10.1016/j.actpsy.2019.102904) | No light or thin weights for text, including large clock numerals; semibold rather than black for small emphasis. |
| F7 | The fluent print-size range spans x-heights of about 0.2° to 2° of visual angle (1.4–14 mm at 40 cm). | Legge & Bigelow 2011, [10.1167/11.5.8](https://doi.org/10.1167/11.5.8) | Lower bound for essential text size; see the calculation below. |
| F8 | For fluent readers of ordinary text in adequate light, reading rate is limited by letter spacing (crowding), not by letter size. | Pelli et al. 2007, [10.1167/7.2.20](https://doi.org/10.1167/7.2.20) | No negative tracking at text sizes; tighten only at display sizes. |
| F9 | Extra-large letter spacing improved text reading on the fly in a large unselected sample of Italian and French children with dyslexia. | Zorzi et al. 2012, [10.1073/pnas.1205566109](https://doi.org/10.1073/pnas.1205566109) | A spacing setting, not a special font. Evidence concerns children and extra-large spacing only. |
| F10 | In 20 readers with age-related macular degeneration, letter spacing and letter width had a small but measurable effect on reading acuity near the critical print size. | Beier et al. 2021, [10.1075/idj.19033.bei](https://doi.org/10.1075/idj.19033.bei) | Supports the same spacing setting for low-vision users; small effect. |
| F11 | Children with dyslexia did not read faster or more accurately in the Dyslexie font than in Arial, and preference did not predict performance. | Kuster et al. 2017, [10.1007/s11881-017-0154-6](https://doi.org/10.1007/s11881-017-0154-6) | No dedicated "dyslexia font"; preference is not evidence of performance. |
| F12 | For 48 readers with dyslexia, sans-serif, monospaced and roman (upright) styles improved reading performance over serif, proportional and italic styles. | Rello & Baeza-Yates 2013, [10.1145/2513383.2513447](https://doi.org/10.1145/2513383.2513447) | Avoid italics for UI emphasis; use weight. |
| F13 | The usual advantage of dark text on a light background disappeared when overall display luminance was equalised; performance followed luminance, not polarity itself. | Buchner et al. 2009, [10.1080/00140130802641635](https://doi.org/10.1080/00140130802641635) | The light theme's advantage comes from luminance; the dark theme must not be dimmer than necessary. |
| F14 | A positive-polarity advantage (dark on light) was found for both younger and older adults in acuity and proofreading tasks. | Piepenbrock et al. 2013, [10.1080/00140139.2013.790485](https://doi.org/10.1080/00140139.2013.790485) | Read with F13: reading-heavy work favours the light theme; dark remains a fully supported choice. |
| F15 | Smoother character rendering (ClearType) led to faster reading, fewer fixations and shorter fixations than non-ClearType rendering. | Slattery & Rayner 2009, [10.1002/acp.1623](https://doi.org/10.1002/acp.1623) | Rasterisation quality is part of legibility; the native 16-level coverage is assessed in section 9. |
| F16 | Ultra-condensed type changed eye-movement patterns, but reading speed did not differ across four widths of one family. | Minakata & Beier 2021, [10.1016/j.apergo.2021.103523](https://doi.org/10.1016/j.apergo.2021.103523) | Avoid extreme widths for text; normal-width faces carry no speed penalty. |

**Standards.** WCAG 2.2 (W3C Recommendation, 12 December 2024,
<https://www.w3.org/TR/WCAG22/>), success criteria checked against the text:

- 1.4.3 Contrast (Minimum, AA): at least 4.5:1 for text, 3:1 for large-scale text
  (at least 18 pt, or 14 pt bold, which is 24 px or about 18.7 px bold); text in
  inactive components and logotypes is exempt.
- 1.4.4 Resize Text (AA): resizable to 200 % without loss of content or function.
- 1.4.8 Visual Presentation (AAA): blocks of text no wider than 80 characters, not
  justified, line spacing at least 1.5 and paragraph spacing at least 1.5 times
  the line spacing. These must be available as a mechanism, not necessarily as
  the default.
- 1.4.10 Reflow (AA): no two-dimensional scrolling at 320 CSS px width.
- 1.4.11 Non-text Contrast (AA): at least 3:1 for component boundaries and states.
- 1.4.12 Text Spacing (AA): no loss when line height is 1.5, paragraph spacing 2,
  letter spacing 0.12 and word spacing 0.16 times the font size.
- 2.4.13 Focus Appearance (AAA): the indicator is at least a 2 px perimeter with
  3:1 change in contrast.

**Calculation (F7, assumptions labelled).** Assumption: a desktop monitor at
about 96 px per inch (0.2646 mm per px) at 100 % scale, viewed from 60 cm.
0.2° at 60 cm subtends 2.09 mm, which is 7.9 px. A face with an x-height of
0.546 em (Inter) reaches that at 14.5 px; 15 px gives 8.2 px (0.21°); 12 px gives
6.6 px (0.16°), below the fluent range. Laptops at 125–150 % scale and closer
viewing shift these numbers; the calculation sets a floor, not a target.
Conclusion: essential text at least 15 px for an x-height near 0.55 em (larger
for faces with smaller x-heights), 12 px only for non-essential metadata.

**Aesthetic preference, recorded separately.** The project owner prefers one family
across wordmark and UI, a geometric relationship to the mark and a premium,
distinctive feel. These are judgements and weigh in the brand criteria of
section 7; they are not presented as findings.

**Untested assumptions.**

- A1: F2/F3 compared one humanist and one square-grotesque typeface. That the
  disadvantage generalises to other square-grotesque or techno faces (Saira,
  Oxanium and, partly, Exo 2) is an assumption.
- A2: The pair-distinctness proxy (section 4) correlates with perceptual
  confusability. It is a pixel-difference measure, not a recognition experiment.
- A3: Light text on dark backgrounds looks heavier (irradiation) and may need
  lighter weights or wider tracking. A design convention, not tested here.
- A4: 16-level (4-bit) coverage without hinting is legible enough at 12–15 px.
  Rendered, not measured with readers.
- A5: Rasterisation through Pillow/FreeType approximates the browser closely
  enough for specimen comparison. Chrome additionally applies kerning and its
  own gamma handling.

## 3. Candidates and sources

Fourteen families, all SIL Open Font License 1.1, variable, from
github.com/google/fonts at commit `29b4ec6d` with SHA-256 pins
(`ui/design/typography/fetch_candidates.py`), plus Archivo, Instrument Sans and
Funnel Sans/Display already vendored for the concept boards (`ui/design/fonts/`).
Required by the brief: Sora, Exo 2, Space Grotesk, Inter (readability benchmark),
Atkinson Hyperlegible Next (accessibility benchmark). Added: Saira (has a width
axis that could carry a wordmark-like wide cut), Manrope, Lexend, Geist,
Oxanium, Archivo, Instrument Sans, Funnel Sans/Display. Open Sans 1.10, the
current native face, could not be measured: its source files are not installed on
this host.

## 4. Measurements

Script: `ui/design/typography/measure_fonts.py`; results:
[typography/measurements.md](typography/measurements.md) and `.json`.
Reproduce:

```bash
python3 -I ui/design/typography/fetch_candidates.py
```

```bash
python3 -I ui/design/typography/measure_fonts.py ui/design/typography/candidates ui/design/fonts docs/design/typography/measurements
```

- **Coverage** of 21 EU Latin-script official languages (from complete alphabet
  sets), Greek, Bulgarian Cyrillic and UI symbols, from each font's `cmap`.
- **Shape**: x-height/cap-height ratio, average lowercase advance, stroke contrast
  of `o`, aperture of `c` and `e`.
- **Pair distinctness** of 12 confusable pairs (I/l, l/1, I/1, O/0, rn/m, c/e, 6/8,
  5/S, B/8, a/o, cl/d, vv/w), rasterised with FreeType at 11–16 px (100 %) and
  22–32 px (200 %): the summed absolute pixel difference divided by the summed
  ink, 0 when the bitmaps are identical. A proxy for F5 under assumption A2.
- **OpenType features**: `tnum` (tabular figures), `zero` (slashed zero) and
  character variants, measured both singly and as the full shipping set.

**Hard gates** (fail any one and the family is out for UI text):

| Gate | Threshold | Fails |
|---|---|---|
| Licence | OFL 1.1 or equivalent permissive | none |
| EU Latin | 21 of 21 languages | Oxanium (Maltese) |
| Confusable pairs | every pair ≥ 0.12 at 100 % and 200 %, alternates allowed | Manrope (I/l 0.000), Geist (I/l 0.061), Space Grotesk (I/l 0.000 at 200 %), Oxanium (6/8, 5/S 0.098), Instrument Sans (I/l 0.111 at 200 %) |
| Tabular figures | `tnum` present | Lexend, Oxanium |
| Weights | at least four, or variable | none |

Passing all gates: Archivo, Atkinson Hyperlegible Next, Exo 2, Funnel Sans,
Inter, Saira, Sora. Saira, and partly Exo 2, carry the square-grotesque risk of
F2/F3 (assumption A1).

**Key numbers for the passing families** (100 %; full tables in the results file):

| Family | x-height (em) | aperture c / e | weakest pair | mean | Greek / Cyrillic | missing UI symbols |
|---|---|---|---|---|---|---|
| Inter, shipping set `cv05 cv08 ss01` | 0.546 | 0.333 / 0.153 | c/e 0.281 | 0.466 | yes / yes | none |
| Inter, default glyphs | 0.546 | 0.333 / 0.153 | 6/8 0.249 | 0.445 | yes / yes | none |
| Atkinson Hyperlegible Next | 0.496 | 0.493 / 0.251 | rn/m 0.169 | 0.429 | no / no | arrows |
| Exo 2 | 0.487 | 0.702 / 0.293 | B/8 0.163 | 0.415 | no / yes | arrows |
| Archivo | 0.526 | 0.245 / 0.114 | c/e 0.146 | 0.395 | no / no | none |
| Funnel Sans | 0.500 | 0.308 / 0.159 | I/l 0.158 | 0.367 | no / no | ≤ ≥ ≠, arrows |
| Saira (`zero`) | 0.510 | 0.719 / 0.305 | O/0 0.239 with `zero` (0.172 without) | 0.353 | no / no | none |
| Sora | 0.534 | 0.285 / 0.158 | rn/m 0.160 | 0.328 | no / no | ≠, arrows |

At 200 %, Atkinson Hyperlegible Next has the highest weakest-pair score (0.276).
Inter's single-storey `a` alternate (`cv11`) would lower a/o from 0.347 to 0.163
and is therefore excluded. The aperture figure treats square forms as open, so it
does not distinguish humanist from square-grotesque styles.

## 5. Specimens

- **Browser specimens**: `ui/design/typography/specimens/index.html`, identical
  Dutch BaryonOS content in neutral colours, so the typeface is the only variable.
  Parameters:
  - `?font=<id>` selects the UI face; `&wm=<id>` sets the wordmark face separately.
  - `&theme=dark|light`.
  - `&view=apps|shell`. The `apps` view holds signal search, a files table and
    path, long text, settings toggles, a form with an error, a context menu,
    confusable characters, a permission dialog, a notification and Orbit. The
    `shell` view holds the Horizon desktop with "Verder waar je was", a Notities
    document, the Zenit launcher, quick settings, notifications, an empty state
    and Orbit.
  - `&blind=<code>` hides the family name for review.

  States shown: regular, semibold, selected, disabled, focus and error. Every
  panel carries its binding surface name, with the P-NAMES-1 proposal in
  parentheses.
- **Native specimens**: `ui/design/typography/native_specimen.py` rasterises
  candidates the way `ui/graphics/scripts/generate_font.py` and `text.rs` draw
  them: per glyph, 16 coverage levels, floored 26.6 advances, no kerning, with
  the shipping alternates baked in, at 100 % and 200 % in dark and light. Each
  face is set at equal x-height (sizes from `roles()`, as in section 6.2), and
  the sheets reproduce byte for byte from the same font files. Output:
  `typography/native/`. This is a build-time simulation, not a QEMU capture.
  `--blind` writes coded copies for review.

## 6. Specimen review

Four reviews, each by a separate reviewer agent that saw only coded images. All
four are AI judgement, not user testing; scores are 1–10 judgements, not
measurements. The unedited results, the code keys and the stimulus hashes are in
[`docs/verification/typography-review-20261113/`](../verification/typography-review-20261113/README.md).

### 6.1 Round 1: eight browser specimens and eight native sheets

All eight browser specimens were set at the same pixel size (UI 15 px, reading
16 px). Inter and Saira had `tnum` and `zero` switched on for whole technical
lines. The native sheets came from `native_specimen.py --blind` with digit codes.

| Code | Face | Readability | Accessibility | Brand | OS consistency | Wordmark fit |
|---|---|---:|---:|---:|---:|---:|
| D | Exo 2 | 7 | 7 | 8 | 8 | 7 |
| K | Atkinson Hyperlegible Next | 8 | 8 | 4 | 8 | 3 |
| W | Inter UI, Saira wordmark line | 7 | 8 | 6 | 6 | 9 |
| R | Inter | 7 | 8 | 4 | 6 | 3 |
| N | Sora | 6 | 7 | 6 | 5 | 5 |
| M | Funnel Sans | 6 | 7 | 4 | 8 | 4 |
| T | Archivo | 6 | 6 | 5 | 6 | 5 |
| H | Saira | 4 | 5 | 5 | 5 | 9 |

Native legibility at 100 %: Atkinson Hyperlegible Next 7, Inter 6, Sora 6,
Archivo 5, Funnel Sans 5, Saira 5, Exo 2 5, Space Grotesk 4. Ranking: D, K, W.
Best wordmark line: H. The reviewer judged a pairing (a legible UI face plus a
squared display face for the brand line) safer, with D as the exception if one
family is required.

Disclosures, which limit how much weight round 1 can carry:

- **Blinding leak.** A browser tab list visible to the reviewer showed the URL of
  code M (`font=funnel`). The reviewer reported it and stated that M's scores rest
  on the images only. M is not blind.
- **Recognition.** Low-confidence resemblances, not used for scoring: D
  "Exo-style", K and R/W "Inter-style". The guess for K was wrong.
- **Unequal sizes.** At 15 px, Exo 2 (x-height 0.487 em) and Atkinson Hyperlegible
  Next (0.496 em) fall below the 7.9 px floor of section 2. Their smaller apparent
  size flattered fit and density.
- **Feature confound.** Inter's `tnum` on whole lines also made the hyphen
  tabular, producing strings such as `Thuis - NAS` that the reviewer partly
  attributed to the face. Saira's zero reading as 8 came from the global `zero`
  feature, not its default glyph.
- **Coverage.** Most dark frames were judged from composite strips; three of the
  200 % quadrants were opened.

### 6.2 Rounds 2 and 2b: three finalists at equal x-height

D, K and R were within the range where the round-1 confounds could change the
order. They were rendered again under new codes (V Exo 2, Q Inter, J Atkinson
Hyperlegible Next) with:

- **Equal x-height.** UI size `ceil(7.9 px / x-height)`: Inter 15 px, Atkinson
  Hyperlegible Next 16 px, Exo 2 17 px; reading text one pixel larger; secondary
  text at the x-height of 13 px Inter (13, 14 and 15 px).
- **Shipping features only.** Inter `cv05 cv08 ss01`; tabular figures on digits
  only; no `zero` anywhere.
- **Blinding.** Sizes hidden; other browser tabs blank or unrelated; new codes.
- **Coverage.** All 12 full frames at 100 % and 22 same-region comparisons at
  200 % (the three faces stacked per region, order rotated).

Round 2 had a capture defect: three of J's four 100 % frames showed the content of
`J-apps-dark-1x.png`, so J's light and shell surfaces were judged from the 200 %
comparisons only, and the reviewer saw scaled-down images. The frames were
recaptured, and a replicate review (round 2b, a fresh agent, every image viewed
at 1:1) was run on the corrected set.

**Combination rule.** Recorded by the lead before round 2b's scores were read;
the timing is documented in the session log, not in this repository. Round 2b is
primary and round 2 corroborates. If the rounds disagree on first place, or the
leader's margin is under 3 points out of 100, no further review round is run: the
tie is broken by measured inputs (sections 4, 6.2 and 9) and the owner's stated
priority, and the choice is listed as one the project owner can redirect.

| Face (code) | Round | Readability | Accessibility | Brand | OS consistency | Wordmark fit | Blind part (of 90) |
|---|---|---:|---:|---:|---:|---:|---:|
| Inter (Q) | 2 | 8 | 7.5 | 5 | 8.5 | 4.5 | 65.75 |
| Inter (Q) | 2b | 8 | 7.5 | 5 | 8.5 | 4.8 | 65.75 |
| Atkinson Hyperlegible Next (J) | 2 | 7 | 7.5 | 5.5 | 6.5 | 5 | 60.25 |
| Atkinson Hyperlegible Next (J) | 2b | 7.8 | 8 | 5.8 | 7 | 5.3 | 65.40 |
| Exo 2 (V) | 2 | 7 | 6.5 | 8.5 | 5.5 | 8.5 | 62.75 |
| Exo 2 (V) | 2b | 6 | 5.8 | 8.5 | 5.5 | 8.5 | 57.85 |

Both rounds ranked Q, J, V and recommended Q as the single text face but not as
the source of the logotype; round 2 suggested drawing the logotype with V's forms
as a basis. Main points of round 2b:

- **Inter**: most compact; calm hierarchy across surfaces; 0 narrower than O;
  the tightest leading in long text; little relation to the mark.
- **Atkinson Hyperlegible Next**: the slashed zero removes O/0 ambiguity in
  identifiers but looks noisy in the clock, dates and prose; the most generous
  leading; more truncation than Inter in the file table (the measurement below
  shows equal cell counts).
- **Exo 2**: closest to the mark; O and 0 nearly identical; the heaviest
  truncation, orphaned digits and crowding in dense rows.

Both reviewers tentatively recognised Q as Inter and V as an Exo 2-like face, so
those two codes were blind in name only.

**Measured fit** (apps view, DOM measurement in Chrome; the shell view truncates
nothing for any of the three):

| Face (size) | Truncated cells | Clipped width |
|---|---:|---:|
| Inter (15 px) | 5 | 160 px |
| Atkinson Hyperlegible Next (16 px) | 5 | 151 px |
| Exo 2 (17 px) | 7 | 263 px |

At equal x-height Exo 2 is the widest of the three; its round-1 advantage in fit
came from setting it below its legibility floor.

### 6.3 Native column, judged again

The round-1 native column came from sheets at equal pixel size, so it cannot
break a tie. Because Inter and Atkinson Hyperlegible Next ended within 3 points,
the finalists' native sheets were rendered again at equal x-height
(`native_specimen.py --blind`, digit codes ordered by a hash of the file name,
reproduced byte for byte) and judged once, blind, by a fresh reviewer agent. Its
score replaces the round-1 value in the 10-point native column. This fills a
confounded input; it is not a third specimen round. The rule was recorded before
the review was launched.

Result: Atkinson Hyperlegible Next 7.6, Exo 2 7.0, Inter 6.7. Main defects named:
Atkinson's tabular `1` leaves gaps (`1 3 nov`) and its slashed zero can look like
`Ø` at 100 %; Inter's word space collapses in the 600 line and its O and 0 differ
only in width; Exo 2's word space collapses in the secondary line and it is the
widest. The reviewer guessed that Specimen 1 (Atkinson Hyperlegible Next) was
Inter, which was wrong.

### 6.4 Reviewer claims checked by measurement

`ui/design/typography/finalist_checks.py`; output in the verification folder.

| Claim | From | Result |
|---|---|---|
| I and l differ only by height | 2b (Inter, Atkinson); 6.3 (all three) | Not reproduced. With `cv05 cv08` Inter draws a serifed I and a tailed l, as Atkinson Hyperlegible Next does by default; Exo 2 has a bare I and an l with a short tail. The cues are one or two pixels wide at 100 %. |
| Atkinson's slashed zero appears in the clock and prose | 2, 2b, 6.3 | Reproduced. The default and tabular zeros are both slashed, and the font has no `zero`, `ss` or `cv` feature to change that. |
| Inter's word space collapses in the 600 line | 6.3 | Reproduced at 15 px: smallest word-space gap 2 px against 3 px inside words. With the space advance 2 px wider: 4 px. At 30 px: 7 px against 4 px. |
| Exo 2's word space collapses in the secondary line | 6.3 | Reproduced as a tie at 15 px: 2 px against 2 px. |
| Atkinson's word spacing | 6.3 (no defect named) | Smallest word-space gap at least 1 px wider than the largest gap inside words, in every role at 100 % and 200 %. |
| Round 2's zero evidence `13 november` | 2 | Not applicable: the string contains no zero. |

Pair distinctness at the token sizes (small and UI size, shipping features):
Inter mean 0.436, weakest c/e 0.185; Atkinson Hyperlegible Next mean 0.435,
weakest rn/m 0.236; Exo 2 mean 0.418, weakest 5/S 0.242 and O/0 0.249.

## 7. Selection and scoring

Weights from the brief: readability and recognition 35, accessibility and
language 20, brand 20, OS consistency 15, native, performance and licensing 10. A
1–10 score contributes score × weight / 10. Round 2b supplies the first four
columns and section 6.3 the native column. Wordmark fit is reported but not
weighted: the logotype is drawn separately (BRAND.md).

| Face | Blind part (of 90) | Native (of 10) | Total | Round 2 blind part |
|---|---:|---:|---:|---:|
| Atkinson Hyperlegible Next | 65.40 | 7.6 | 73.00 | 60.25 |
| Inter | 65.75 | 6.7 | 72.45 | 65.75 |
| Exo 2 | 57.85 | 7.0 | 64.85 | 62.75 |

Atkinson Hyperlegible Next leads by 0.55 points, under the 3-point threshold,
while round 2b's blind part puts Inter first. The tie branch of section 6.2
applies; no further review round was run. The inputs, both ways:

| Input | Inter | Atkinson Hyperlegible Next | Favours | In the first native atlas? |
|---|---|---|---|---|
| Readability and accessibility, weighted (2b) | 43.0 | 43.3 | Equal within 0.3 | Judgement |
| OS consistency (2b) | 8.5 | 7 | Inter | Judgement |
| Native legibility (6.3) | 6.7 | 7.6 | Atkinson | Simulated atlas output |
| Pair distinctness, mean / weakest (6.4) | 0.436 / 0.185 | 0.435 / 0.236 | Equal / Atkinson | Yes |
| Fit: truncated cells / clipped width (6.2) | 5 / 160 px | 5 / 151 px | Equal | Browser measurement |
| Native word space, 600 at UI size (6.4) | Collapses at 15 px; a 2 px wider space fixes it | No collapse | Atkinson | Inter needs the correction in one record |
| Zero (6.4) | Unslashed and narrower than O; slashed alternate for code | Slashed everywhere, cannot be switched off | Inter in clock and prose; Atkinson in identifiers | Atkinson's slash: yes. Inter's code zero needs a second glyph: not yet |
| Language coverage (4) | 21 EU Latin languages, Greek, Cyrillic, ↑↓ | 21 EU Latin languages | Inter | Latin-1 first for both; `›↑↓` can be added for Inter only; Greek and Cyrillic are a later budget decision |
| Optical size axis | `opsz` 14–32 | None | Inter | Yes: each record is cut at its own optical size |
| Glyph data, section 8 role set, Latin-1 (9) | 261.6 KiB | 265.5 KiB | Equal | Estimate |
| Licence | OFL 1.1, no Reserved Font Name | OFL 1.1, no Reserved Font Name | Equal | Yes |
| Brand / wordmark fit (2b) | 5 / 4.8 | 5.8 / 5.3 | Atkinson; both low | Logotype is drawn |

The rule does not select a face. The measured legibility inputs (weakest pair,
native legibility, word spacing) lean towards Atkinson Hyperlegible Next. The
measured language coverage and zero behaviour lean towards Inter. The weighted
readability and accessibility scores are equal. The selection below is therefore
a product judgement, and the project owner can redirect it.

**Selected for UI and reading text: Inter 4.001** with `cv05 cv08 ss01`
(judgement). Reasons:

- One family can cover all 24 official EU languages; with Atkinson Hyperlegible
  Next, Greek and Cyrillic would need a second face. This is the owner's
  one-family preference applied to languages, not only to the wordmark.
- The desktop clock, dates and prose (the most frequent numerals) keep a calm,
  unslashed zero. Identifiers get a slashed zero through the code feature in the
  browser now, and natively once a code glyph is added (section 9).
- An optical size axis and the highest OS-consistency judgement in both rounds.

Costs accepted: a weaker weakest pair (c/e 0.185), a lower native legibility
judgement (6.7), one atlas correction (word space at 15 px 600) and denser long
text, answered by the 26 px leading of `type.reading`.

**Atkinson Hyperlegible Next: optional reading face** (Leeslettertype, F1) for
`type.reading` at its equal-x-height size of 17 px. In the browser prototype now;
natively it needs four more records (about 86 KiB, Latin-1) and stays
prototype-only until the read-only budget is measured on the built ELF.

**Redirect to Atkinson Hyperlegible Next** would mean: sizes 16/17/14 px for UI,
reading and small text; no Greek, Cyrillic or arrows (a fallback face or icons
needed); a slashed zero everywhere, including the clock; no word-space
correction.

**Exo 2 is not selected for text.** It had the lowest readability and
accessibility in both rounds, the most truncation (7 cells, 263 px), O/0 at 0.249
and the square-grotesque risk of assumption A1. It is recorded as the face with
the most brand character (brand and wordmark fit 8.5 in both rounds) and is an
input to the logotype.

**Logotype routes**, decided in BRAND.md v2, not here:

- (a) An optically refined outline drawn from Inter. Unified with the UI and
  allowed by the brief; reviewers rated Inter's "Baryon OS" line lowest for fit
  with the mark (4.5–4.8).
- (b) A geometric outline drawn with Exo 2 or Saira forms as reference, as
  round 2 suggested. A departure from the one-family preference, recorded as such.
- (c) The original wordmark, kept as reference and reversible fallback.

## 8. Typography token specification

Version `type-1.0` (14 November 2026). Applies to the shell (Horizon, Orbit,
Zenit, Signaal, Nebula, Kern, Haven), the first-party apps and the prototype.
Values are CSS px at 100 % scale. The native renderer uses the same pixel sizes
(section 9), so the browser and QEMU show the same scale.

### 8.1 Families

| Token | Value | Use |
|---|---|---|
| `font.family.ui` | Inter 4.001 (`ui/design/fonts/`) | All interface and reading text |
| `font.family.wordmark` | Outlined logotype, route chosen in BRAND.md v2 (section 7) | Wordmark only; never typed text |
| `font.family.reading-alt` | Atkinson Hyperlegible Next, user setting | Optional reading face (F1); prototype-only natively until measured |
| `font.features.ui` | `cv05 cv08 ss01` | Always on |
| `font.features.digits` | `tnum` on 0–9 only | Clocks, tables, sizes, dates |
| `font.features.code` | `zero` (slashed) | Paths, identifiers, hashes; native support open (section 9) |
| Fallback (browser) | `system-ui, sans-serif` | Prototype only |
| Fallback (native) | U+FFFD from the same atlas | Characters outside the atlas |

### 8.2 Roles

| Role token | Size / line height | Weight | Tracking | Native record (100 % / 200 %) | Use |
|---|---|---|---|---|---|
| `type.caption` | 13 / 18 | 400 | 0 | 13r / 26r | Non-essential metadata only: timestamps, secondary counts, column hints |
| `type.body` | 15 / 22 (20 in dense rows) | 400 | 0 | 15r / 30r | All UI text: menus, buttons, list rows, settings, fields, notifications |
| `type.body-strong` | 15 / 22 | 600 | 0 | 15sb / 30sb | Buttons, selected navigation, row titles, labels that carry state |
| `type.reading` | 16 / 26 | 400 | 0 | 16r / 32r | Documents and long text (Notities, help, licence) |
| `type.title` | 22 / 28 | 600 | 0 | 22sb / 44sb | Window, page and dialog titles; Signaal group heads |
| `type.display` | 48 / 56 | 400 | 0 | 48r / 48r magnified 2× | Greeting, clock, boot and sign-in only |

Caption, body and reading sizes are the sizes reviewed in rounds 2, 2b and 6.3.
Title and display were not part of the specimen review.

Rules:

- Two weights only, 400 and 600 (F6). No light or thin weights, also not for
  large numerals, and no 700 in UI text.
- No italic for emphasis (F12); use 600. No all-caps labels: Dutch `IJ`,
  diacritics and word shapes suffer.
- Tracking 0 at every text size (F8). The display role tightens through the
  face's optical size, not through letter spacing.
- Essential text is never below 15 px (calculation under F7); 13 px is limited to
  metadata whose loss does not block a task.
- Reading text: at most 80 characters per line (WCAG 1.4.8), target 60–72; never
  justified.
- Hierarchy uses size and weight first, colour second. Secondary text uses the
  `text.secondary` colour token, never a lighter weight.
- Native word space: the 15 px 600 record widens the space advance by 2 px
  (section 6.4). No other record needs it.

### 8.3 States

| State | Treatment |
|---|---|
| Normal | Role weight, `text.primary` |
| Selected | 600 and the selection indicator; never colour alone |
| Disabled | Same weight, `text.disabled` token (exempt from 1.4.3 but kept at least 3:1) |
| Focus | Focus ring per the design system (2 px, 3:1, WCAG 2.4.13); text unchanged |
| Error | `status.error` text plus an icon and a message; never colour alone |

### 8.4 Numbers, dates and paths (nl-NL)

- Numbers `1.234,56`; currency `€ 1.234,56`; sizes `18 MB`, `1,2 GB` (decimal
  comma, space before the unit).
- Dates `13 nov. 2026` in lists, `vrijdag 13 november 2026` in headers,
  `13-11-2026` in tables; time `09:41` (24-hour).
- All digits tabular, so columns and clocks do not jitter.
- Paths and identifiers in `type.body` with the code feature; truncate in the
  middle (`Thuis-NAS › … › Tuin`), never in the file name.

### 8.5 Localisation

- The first native atlas holds Latin-1, `–—‘’“”•…€` and `›↑↓`; EU Latin follows
  after the read-only budget check; Greek and Cyrillic need a project decision
  (section 9). Browser coverage follows the face.
- Allow 30–40 % text expansion (German, Finnish): buttons grow, they do not
  truncate.
- Truncation uses `…` (U+2026), with the full text in a tooltip and the
  accessible name.

### 8.6 Accessibility settings (WCAG 1.4.4, 1.4.8, 1.4.12)

- Text size: 100 % and 200 % natively today; 125 % and 150 % are open (section
  9, item 7).
- Ruimere tekst (wider spacing): letter spacing 0.12 em, word spacing 0.16 em,
  line height 1.5, paragraph spacing 2 em, without loss of content (F9, F10).
- Leeslettertype (reading face): Inter or Atkinson Hyperlegible Next for
  `type.reading` (F1). Preference is not evidence of performance (F11).

## 9. Native rendering and licensing assessment

**Current native text** (`ui/graphics/src/text.rs`, `ui/graphics/scripts/generate_font.py`):

- **Glyphs**: a fixed Open Sans 1.10 atlas generated offline: 194 characters
  (Latin-1, …, €, U+FFFD), with Regular 12/16/24/32 and Semibold 16/32/64.
- **Display**: the clock face is the 64 px record, magnified 2× at 200 %.
- **Encoding**: 16 coverage levels in run-length rows; 188.5 KiB of glyph data.
- **Not supported**: no runtime font parser, kerning, shaping, hinting or
  subpixel rendering. Glyphs are bounded to 96 × 96 px and unknown characters
  draw U+FFFD.
- **Shared users**: the atlas is shared by `ui/native`, Veld and the SDK examples
  (calculator, window-session, presentation, compositor).
- **Binding constraint**: `ui/native` has an explicit 512 KiB read-only segment
  limit (`ui/native/README.md`).

**Atlas size estimates** for Inter with the shipping alternates, using the same
encoding (glyph data only; the glyph and row tables add more). Measured with
`ui/design/typography/atlas_estimate.py`:

| Records | Latin-1 (194) | EU Latin (350) | EU + Greek + Cyrillic (487) |
|---|---|---|---|
| Today's seven records | 177.7 KiB | 327.1 KiB | 459.0 KiB |
| Full proposed scale (20 records) | 558.4 KiB | 1028.3 KiB | 1451.5 KiB |
| Reduced set (9 records): 12r 15r 15sb 22sb 48r, and 24r 30r 30sb 44sb at 200 % (display magnified) | 215.6 KiB | 396.6 KiB | 559.4 KiB |
| Section 8 role set (11 records): 13r 15r 15sb 16r 22sb 48r, and 26r 30r 30sb 32r 44sb | 261.6 KiB | 481.0 KiB | 679.1 KiB |

The same role set for Atkinson Hyperlegible Next at its own equal-x-height sizes
(14r 16r 16sb 17r 24sb 52r, and 28r 32r 32sb 34r 48sb) needs 265.5 KiB for
Latin-1 and 503.0 KiB for EU Latin; it has no Greek or Cyrillic glyphs. Its
optional reading records alone (17r 17sb 34r 34sb) need 86.2 KiB for Latin-1.
Reproduce with `atlas_estimate.py FONT [features] weight:px,...`.

**Feasible route** (no browser or webview in the native path):

1. Vendor the selected family's TTF with its OFL text and SHA-256 under
   `ui/design/fonts/`, and add a provenance entry. Done for Inter 4.001
   (`ui/design/fonts/FONTS.md`).
2. Extend the generator to emit a second atlas beside the Open Sans atlas, never
   regenerating `atlas.rs` in place. Veld and the SDK examples keep Open Sans
   until each is migrated and its text-metric oracles are reviewed.
3. Bake the alternates (`cv05`, `cv08`, `ss01`) and tabular digits into the
   glyph selection, a 1:1 substitution needing no shaping. Tabular figures apply
   to digits only: in Inter, `tnum` also widens the hyphen. The character set is
   Latin-1, `–—‘’“”•…€`, U+FFFD and `›↑↓`, which the prototype copy uses.
   Widen the space advance of the 15 px 600 record by 2 px (section 6.4). The
   slashed code zero needs a second glyph that the atlas cannot select without
   shaping: a separate code record or an explicit `text.rs` call, which is an
   open interface decision.
4. Keep the token scale equal to the native records, so the browser and QEMU
   show the same sizes. Start with Latin-1 (261.6 KiB of glyph data for the
   section 8 role set, against 188.5 KiB today) and measure it against the built
   ELF's read-only segment; the Open Sans atlas leaves `ui/native` only if
   nothing there references it any more. Measure EU Latin the same way before
   adopting it.
5. Greek and Cyrillic do not fit the current budget alongside Latin. They need a
   separate, separately reviewed atlas or a revised segment limit, which is a
   project decision.
6. **Kerning**: the native path draws without kerning, whereas browsers apply
   it. Measure the visible difference on common pairs and, if needed, bake a
   small pair table. This changes `text.rs` metrics and needs oracle review.
7. **Accessibility scaling**: native today supports 100 % and 200 % only. A text
   size setting at 125–150 % needs more records or a sandboxed on-device
   rasteriser. Font parsing is a trust boundary: system fonts only, signed, and
   fuzzed. This is a research item, not part of this branch.
8. **Rendering quality** (F15, A4): 16 coverage levels without hinting soften
   stems at 12–15 px. 200 % largely removes this. A gamma-correct blend check and
   an 8-bit coverage option are open items; each would cost bytes.

**Licensing.** All candidates are under SIL OFL 1.1, which permits use,
embedding, modification and redistribution with the licence text, but not
selling the fonts on their own. None of the copyright lines for Inter, Atkinson
Hyperlegible Next, Exo 2 or Sora declares a Reserved Font Name, so a subset or
rasterised derivative may keep the family name. The OFL text must accompany the
OS (licence screen and provenance), as Apache-2.0 does today for Open Sans. A
rasterised atlas is treated as a derivative of the font software for this
purpose: this is a conservative reading, not legal advice.
