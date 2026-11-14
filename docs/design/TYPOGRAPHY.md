# BaryonOS typography

Status: research, measurements, specimen review and token specification
`type-1.0` complete. Selection: tie under the decision rule of section 6.2
(Atkinson Hyperlegible Next 73.00, Inter 72.45 of 100; the rule's
pre-registration cannot be verified from the repository). Inter 4.001 is
selected for `type-1.0` by the lead under the project owner's delegation of
14 November 2026, as a reversible judgement (section 7); the project owner can
redirect it. Native integration: font vendored
(section 9 step 1); an Inter atlas with nine records is generated behind the
`inter-atlas` cargo feature, off by default (host evidence only, no QEMU:
`docs/verification/native-inter-atlas-20261114/`); the default build still
draws Open Sans. Dates: 13–14 November 2026
(Europe/Amsterdam). Design study on branch `claude/baryon-os-ui-ux-design-152adb`.

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
Sources whose abstract was not available are not used for claims. The lookup
output is not stored in the repository.

| # | Finding | Source | Implication for BaryonOS |
|---|---|---|---|
| F1 | Among hundreds of crowdsourced adult readers, reading speed differed by 35 % between an individual's fastest and slowest font, without loss of comprehension; the high variability across fonts indicates that one font does not fit all. | Wallace et al. 2022, [10.1145/3502222](https://doi.org/10.1145/3502222) | Ship a default and a spacing setting; a user-selectable reading face is indicated but not in `type-1.0` (open item). Never market the default as best for everyone. |
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
6.6 px (0.17°), below the fluent range. Laptops at 125–150 % scale and closer
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

Fourteen families, all SIL Open Font License 1.1, variable: ten from
github.com/google/fonts at commit `29b4ec6d` with SHA-256 pins
(`ui/design/typography/fetch_candidates.py`), and Archivo, Instrument Sans,
Funnel Sans and Funnel Display, already vendored for the concept boards from
the same repository's branch `main` (`ui/design/fonts/`).
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

- **Coverage** of the 21 Latin-script official EU languages other than English
  (which needs ASCII only; from complete alphabet sets), Greek, Bulgarian
  Cyrillic and UI symbols, from each font's `cmap`.
- **Shape**: x-height/cap-height ratio, average lowercase advance, stroke contrast
  of `o`, aperture of `c` and `e`.
- **Pair distinctness** of 12 confusable pairs (I/l, l/1, I/1, O/0, rn/m, c/e, 6/8,
  5/S, B/8, a/o, cl/d, vv/w), rasterised with FreeType at 11, 12, 13, 14 and
  16 px (100 %) and at twice those sizes (200 %): the summed absolute pixel difference divided by the summed
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
| Saira (`zero`) | 0.510 | 0.719 / 0.305 | a/o 0.196 (O/0 0.172 without `zero`, 0.239 with) | 0.353 (without `zero`) | no / no | none |
| Sora | 0.534 | 0.285 / 0.158 | rn/m 0.160 | 0.328 | no / no | ≠, arrows |

The last column leaves out the thin space U+2009 and the narrow no-break space
U+202F: every passing family except Inter lacks U+202F, and Funnel Sans, Saira
and Sora also lack U+2009 (results file). At 200 %, Atkinson Hyperlegible Next
has the highest weakest-pair score (0.276). Inter's single-storey `a` alternate
(`cv11`) would lower a/o from 0.347 to 0.163 (a separate
`measure_fonts.distinctness` call with `cv11` at the section 4 sizes, not in the
stored results) and is therefore excluded. The aperture figure treats square forms as open, so it
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
  face is set at equal x-height (sizes from `roles()`, as in section 6.2).
  Output: `typography/native/`, which reproduces byte for byte when the font
  files of the seven gate-passing families (section 4) are gathered in one
  directory and passed as the only FONT_DIR. This is a build-time simulation,
  not a QEMU capture: among other differences, its compositing rounds down
  where `blend` in `ui/graphics/src/lib.rs` rounds to nearest (at most 1/255 per
  channel). `--blind` writes coded copies for review.

## 6. Specimen review

Four reviews, each by a separate reviewer agent that saw only coded images
(apart from the round-1 leak in section 6.1). All
four are AI judgement, not user testing; scores are 1–10 judgements, not
measurements. The unedited results, the code keys and the stimulus hashes are in
[`docs/verification/typography-review-20261113/`](../verification/typography-review-20261113/README.md).

### 6.1 Round 1: eight browser specimens and eight native sheets

All eight browser specimens were set at the same pixel size (UI 15 px, reading
16 px). Every face had `tnum` on whole technical lines, and Inter, Saira and
Archivo also `zero` (`index.html` as committed at `2cc2df1f`; the capture-time
version is not recorded). The native sheets came
from `native_specimen.py --blind` at commit `2cc2df1f`, which reproduces them
byte for byte, with digit codes.

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

Native legibility (one score per sheet row, covering 100 % and 200 %): Atkinson
Hyperlegible Next 7, Inter 6, Sora 6,
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
- **Unequal sizes.** At 15 px every face except Inter and Sora falls below the
  7.9 px floor of section 2: Exo 2 7.3 px, Atkinson Hyperlegible Next 7.4 px,
  Funnel Sans 7.5 px, Saira 7.65 px, Archivo 7.89 px. Their smaller apparent
  size flattered fit and density.
- **Feature confound.** Inter's `tnum` on whole lines also made the hyphen
  tabular, producing strings such as `Thuis - NAS` that the reviewer partly
  attributed to the face. Saira's zero reading as 8 came from the
  technical-line `zero` feature, not its default glyph; Archivo's slashed zero
  in the stress line likewise.
- **Coverage.** Twelve 100 % frames (nine dark, three light) were judged from
  composite strips only; most q2 and q3 quadrants at 200 % were not opened (the
  reviewer names D-q3, R-q2 and W-shell-dark-2x-q1 as viewed).

### 6.2 Rounds 2 and 2b: three finalists at equal x-height

D, K and R were within the range where the round-1 confounds could change the
order. They were rendered again under new codes (V Exo 2, Q Inter, J Atkinson
Hyperlegible Next) with:

- **Equal x-height, for three tiers.** UI size `ceil(7.9 px / x-height)`:
  Inter 15 px, Atkinson Hyperlegible Next 16 px, Exo 2 17 px; reading text one
  pixel larger; secondary text at the x-height of 13 px Inter (13, 14 and
  15 px). In the committed `index.html`, every other element kept one pixel size
  for all three faces: captions 12 px, subheads and search 18 px, headings
  28 px, the clock 56 px, the confusable rows 13 and 11 px, the Notities body
  16 px and the wordmark line 40 px. Reviewer claims drawn from those elements
  (I/l and O/0 in the confusable rows, zeros in the Notities prose and the
  clock) compare the faces at unequal x-height. The `index.html` version used
  for the captures is not recorded.
- **Shipping features only.** Inter `cv05 cv08 ss01`; tabular figures on digits
  only; no `zero` anywhere.
- **Blinding.** Sizes hidden; other browser tabs blank or unrelated; new codes.
- **Coverage.** All 12 full frames at 100 % and 22 same-region comparisons at
  200 % (the three faces stacked per region, order rotated).

Round 2 had a capture defect: three of J's four 100 % frames showed the content of
`J-apps-dark-1x.png`, so J's light and shell surfaces were judged from the 200 %
comparisons only, and the reviewer saw scaled-down images. The three frames
were recaptured, and a replicate review (round 2b, a fresh agent) was run on the
corrected set. The browser viewport matched each image at 1:1, but the 2b
reviewer reports that its screenshot tool scaled a1, a3, a5, s3 and s4 to about
0.6–0.8× and a7 to about 0.92× before it saw them, and that it did not zoom.
By its own account its I/l judgements are tentative, and for the downscaled
images it judged I/l cues and 5/S/8/B partly from the 1× frames and the
unscaled 2× images.

**Combination rule**, as written in this record: round 2b is primary and round
2 corroborates. If the rounds disagree on first place, or the leader's margin is
under 3 points out of 100, no further review round is run: the tie is broken by
measured inputs (sections 4, 6.2 and 9) and the owner's stated priority, and the
choice is listed as one the project owner can redirect. The rule was amended
after round 2b to add the native re-judge of section 6.3, a deviation. The rule,
its amendment, every score and the selection first appear together in commit
`6d5678d0`; no timestamped copy of either rule text exists in the repository,
so the order in which rule and scores were written cannot be verified from it.
The status line's "pre-registered" rests on the lead's account.

| Face (code) | Round | Readability | Accessibility | Brand | OS consistency | Wordmark fit | Blind part (of 90) |
|---|---|---:|---:|---:|---:|---:|---:|
| Inter (Q) | 2 | 8 | 7.5 | 5 | 8.5 | 4.5 | 65.75 |
| Inter (Q) | 2b | 8 | 7.5 | 5 | 8.5 | 4.8 | 65.75 |
| Atkinson Hyperlegible Next (J) | 2 | 7 | 7.5 | 5.5 | 6.5 | 5 | 60.25 |
| Atkinson Hyperlegible Next (J) | 2b | 7.8 | 8 | 5.8 | 7 | 5.3 | 65.40 |
| Exo 2 (V) | 2 | 7 | 6.5 | 8.5 | 5.5 | 8.5 | 62.75 |
| Exo 2 (V) | 2b | 6 | 5.8 | 8.5 | 5.5 | 8.5 | 57.85 |

Both reviewers' own rankings were Q, J, V, and both recommended Q as the single
text face but not as the source of the logotype, which both suggested drawing
with V as the reference. The weighted round-2 scores, however, order Q, V, J,
and round 2's reviewer called J's second place "provisional". Round 2
therefore corroborates first place only. Main points of round 2b:

- **Inter**: most compact; calm hierarchy across surfaces; 0 narrower than O;
  the tightest leading in long text; little relation to the mark.
- **Atkinson Hyperlegible Next**: the slashed zero removes O/0 ambiguity in
  identifiers but looks noisy in the clock, dates and prose; the most generous
  leading; more truncation than Inter in the file table, losing the time
  (`gisteren 16:…`, `vandaag 09:…`).
- **Exo 2**: closest to the mark; O and 0 nearly identical; the heaviest
  truncation, orphaned digits and crowding in dense rows.

The leading differences follow from the specimen's proportional line height for
reading text (1.55 × size: Inter 24.8 px, Atkinson Hyperlegible Next 26.35 px,
Exo 2 27.9 px), not from the faces. Round 2's reviewer said Q and V "strongly"
resemble Inter and Exo 2; round 2b recognised them with moderate or low
confidence. Those two codes were blind in name only.

**Fit observation, unrecorded** (apps view, DOM measurement in Chrome during
the review session; the shell view truncated nothing for any of the three):

| Face (size) | Truncated cells | Clipped width |
|---|---:|---:|
| Inter (15 px) | 5 | 160 px |
| Atkinson Hyperlegible Next (16 px) | 5 | 151 px |
| Exo 2 (17 px) | 7 | 263 px |

No script or stored output for these numbers exists in the repository, so they
are an observation, not a measurement, and section 7 does not use them. Equal
counts do not show which cells were cut, so they do not rebut round 2b's claim
that J loses the time in the file table. Round 2b and the native re-judge both
judged Exo 2 the widest of the three at equal x-height; in round 1 its fit was
judged at a smaller apparent size (section 6.1).

### 6.3 Native column, judged again

The round-1 native column came from sheets at equal pixel size. Because Inter
and Atkinson Hyperlegible Next ended within 3 points, the finalists' native
sheets were rendered again at equal x-height (`native_specimen.py --blind`,
digit codes ordered by a hash of the file name, reproduced byte for byte) and
judged once, blind, by a fresh reviewer agent. Its score replaces the round-1
value in the 10-point native column. This is a fourth AI review, run after the
tie was known, and so a deviation from the recorded rule's "no further review
round is run" (section 6.2); it is described as an input repair. No record of
when it was decided exists in the repository. It did not change the leader of
the total: with the round-1 native values (Atkinson Hyperlegible Next 7, Inter
6) the totals were 72.40 against 71.75.

Result: Atkinson Hyperlegible Next 7.6, Exo 2 7.0, Inter 6.7. The reviewer
states that "the scores are relative to each other, not absolute benchmarks",
and that its pixel-level judgements are approximate because its visual input
downsampled the images. Main defects named: Atkinson's tabular `1` leaves gaps
(`1 3 nov`) and its slashed zero can look like `Ø` at 100 %; Inter's I, l and |
are nearly indistinguishable (not reproduced, section 6.4), its word space
collapses in the 600 line, its O and 0 differ only in width, and its secondary
line looks lighter and smaller in light mode at 100 %; Exo 2's word space
collapses in the secondary line and it is the widest. The reviewer guessed,
wrongly, that Specimen 1 (Atkinson Hyperlegible Next) was Inter, and
tentatively and correctly recognised Specimen 3 as Exo 2.

### 6.4 Reviewer claims checked by measurement

`ui/design/typography/finalist_checks.py`; output in the verification folder
(`finalist-checks.txt`). The rasters come from Pillow/FreeType (assumption A5),
not from the Chrome captures the reviewers saw. Records at 200 % are cut at the
doubled optical size (section 9).

| Claim | From | Result |
|---|---|---|
| I and l differ only by height | 2b (Inter, Atkinson); 6.3 (all three) | Not reproduced for Inter and Atkinson Hyperlegible Next: with `cv05 cv08` Inter draws a serifed I and a tailed l, as Atkinson does by default. Consistent with the raster for Exo 2: a bare I and an l with a short tail. The cues are one or two pixels wide at 100 %. |
| Atkinson's slashed zero appears in the clock and prose | 2, 2b, 6.3 | Reproduced. The default and tabular zeros are both slashed, and the font has no `zero`, `ss` or `cv` feature to change that. |
| Atkinson's zeros are mixed, some slashed and some not | 2 | Not reproduced: the default and tabular zeros are both slashed. |
| Inter's word space collapses in the 600 line | 6.3 | Reproduced at 15 px: smallest word-space gap 2 px against 3 px inside words. With the space advance 2 px wider: 4 px. At 30 px: 7 px against 4 px. |
| Exo 2's word space collapses in the secondary line | 6.3 | Reproduced as a tie at 15 px: 2 px against 2 px. |
| Atkinson's word spacing | 6.3 (no defect named) | On each role's own line: smallest word-space gap at least 1 px wider than the largest gap inside words, at 100 % and 200 %. |

The same check over all four specimen lines for every role record, and for
Inter's 22sb, 44sb and 48r, flags both a collapse (a smaller gap across the
space than inside a word) and a tie (equal gaps), which the Exo 2 row above
counts as reproduced:

- **Inter**: 15sb collapses on the 600 line (2 px at `r t` against 3 px) and
  ties on the reading and secondary lines; 15r ties on the secondary line. The
  other records, 13r, 16r, 22sb, 26r, 30r, 30sb, 32r, 44sb and 48r, show none.
- **Atkinson Hyperlegible Next**: no smaller gap; ties at 16r, 16sb and 17r on the
  secondary line and at 14r on the reading line.
- **Exo 2**: no smaller gap; ties in five of its eight records (15r, 17r, 17sb,
  18r and 34r).
- With the space advance 2 px wider, no record of any face ties or collapses on
  these lines.

Several of the largest gaps inside words sit beside tabular digits or
punctuation (`,5`, `11`, `1|`, `-1`), so some ties come from the tabular-digit
rule rather than the space. Four lines are not a text corpus.

Pair distinctness at the token sizes (shipping features). The proxy here
renders 8-bit coverage with kerning and proportional digits, so it only
approximates the native atlas, which is 4-bit, unkerned and tabular.

| Face | Small size: mean, weakest | UI size: mean, weakest | Two-size mean, weakest |
|---|---|---|---|
| Inter | 13 px: 0.452, c/e 0.195 | 15 px: 0.421, c/e 0.175 | 0.436, c/e 0.185 |
| Atkinson Hyperlegible Next | 14 px: 0.444, a/o 0.252 | 16 px: 0.426, rn/m 0.039 | 0.435, rn/m 0.236 |
| Exo 2 | 15 px: 0.369, I/1 0.098 | 17 px: 0.466, O/0 0.235 | 0.418, 5/S 0.242 |

The weakest pair swings strongly between adjacent sizes: Atkinson's rn/m is
0.039 at 16 px, under the section 4 gate of 0.12, but not among its three
weakest pairs at 14 px. For Inter against Atkinson Hyperlegible Next the proxy
is therefore inconclusive.

## 7. Selection and scoring

Weights from the brief: readability and recognition 35, accessibility and
language 20, brand 20, OS consistency 15, native, performance and licensing 10. A
1–10 score contributes score × weight / 10. Round 2b supplies the first four
criteria (the blind part, of 90). The whole fifth criterion went to the native legibility judgement of
section 6.3; the other two parts it names, atlas size and licence, are equal
for the two leading faces (tables below). Wordmark fit is reported but not
weighted: the logotype is drawn separately (BRAND.md).

| Face | Blind part (of 90) | Native legibility, judgement (of 10) | Total | Round 2 blind part |
|---|---:|---:|---:|---:|
| Atkinson Hyperlegible Next | 65.40 | 7.6 | 73.00 | 60.25¹ |
| Inter | 65.75 | 6.7 | 72.45 | 65.75 |
| Exo 2 | 57.85 | 7.0 | 64.85 | 62.75 |

¹ Rests on defective stimuli; its reviewer called J's place "provisional"
(section 6.2).

Atkinson Hyperlegible Next leads the total by 0.55 points, under the 3-point
threshold; without the native column, the blind part puts Inter first (65.75
against 65.40). The total's leader is therefore decided by the native
legibility column: one AI judgement, whose reviewer wrote that "the scores are
relative to each other, not absolute benchmarks", and which counts against
Inter an I/l defect that section 6.4 did not reproduce. Replicate variation is
larger than these gaps: Exo 2, which had no capture defect, moved between
rounds 2 and 2b from 7 to 6 in readability and from 62.75 to 57.85 in blind
part. The tie branch of section 6.2 applies.

**Tie-break, part 1: the inputs the recorded rule names** (sections 4, 6.2
and 9).

| Input | Inter | Atkinson Hyperlegible Next | Favours |
|---|---|---|---|
| Pair distinctness, section 4 (measurement, 11–14 and 16 px) | weakest c/e 0.281, mean 0.466 (shipping set) | weakest rn/m 0.169, mean 0.429; at 200 % the highest weakest pair of the passing families (0.276) | Inter at 100 %; Atkinson at 200 % |
| Language coverage, section 4 (measurement) | 21 Latin-script EU languages other than English, Greek, Cyrillic, arrows | 21 Latin-script EU languages other than English; no Greek, Cyrillic or arrows | Inter; natively ↑↓ are in the proposed first atlas, Greek and Cyrillic only after a separate atlas-budget decision (section 9) |
| Fit, section 6.2 | No recorded measurement | No recorded measurement | Not used |
| Glyph data, section 8 role set, Latin-1 (section 9, measurement) | 261.6 KiB | 265.5 KiB | Equal |
| Licence (section 9) | OFL 1.1, no Reserved Font Name | OFL 1.1, no Reserved Font Name | Equal |

**Part 2: the native re-judge, a declared deviation.** The rule was amended
after round 2b to add an input-repair re-judge of the native column (section
6.3). No timestamped copy of either rule text exists in the repository. Its
result, 7.6 against 6.7, is already in the total above and is not used again
here.

**Part 3: other inputs.**

| Input | Inter | Atkinson Hyperlegible Next | Status |
|---|---|---|---|
| Readability and accessibility, weighted (2b, judgement) | 43.0 | 43.3 | Already in the total, not a tie-breaker |
| OS consistency (2b, judgement) | 8.5 | 7 | Already in the total, not a tie-breaker |
| Brand (2b, judgement) | 5 | 5.8 | Already in the total, not a tie-breaker |
| Native legibility (6.3, judgement) | 6.7 | 7.6 | Already in the total, not a tie-breaker |
| Wordmark fit (2b, judgement, unweighted) | 4.8 | 5.3 | Not used: the logotype is drawn (BRAND.md v2) |
| Pair distinctness at the token sizes (6.4, measurement; approximation: 8-bit, kerned, proportional digits) | 13 px c/e 0.195; 15 px c/e 0.175 | 14 px a/o 0.252; 16 px rn/m 0.039 | Inconclusive for this pair: the weakest pair swings with the size |
| Native word space on the four measured lines (6.4, measurement) | 15sb collapses by 1 px and ties twice; 15r ties once; all 11 role-set records checked | Ties at 14r, 16r, 16sb and 17r, no smaller gap; 8 of its 11 role-set records checked (not 24sb, 48sb, 52r) | Inconclusive: section 6.4 counts a tie as a collapse, and a 2 px wider space clears every checked record of both faces |
| Zero (6.4, measurement) | Unslashed; slashed `zero` alternate in the font, not applied anywhere yet; natively a second glyph | Slashed everywhere, cannot be switched off | Inter in clock and prose; Atkinson in identifiers |

The rule does not select a face. Of the inputs it names, section 4's
distinctness favours Inter at 100 % and Atkinson Hyperlegible Next at 200 %,
language coverage favours Inter (natively, for Greek and Cyrillic, only after a
budget decision), fit is unrecorded, and atlas size and licence are equal. Of
the other measurements, word spacing and the token-size distinctness are
inconclusive and the zero divides by context. The judgements are already in the
total. The proposal below is therefore a product judgement; the lead selection
that follows it is reversible, and the project owner can redirect it.

**Proposed for UI and reading text: Inter 4.001** with `cv05 cv08 ss01`
(judgement; selected by the lead on 14 November 2026, below). Reasons:

- Language coverage (measurement, section 4): Greek, Cyrillic and arrows are in
  the font, so a browser page set in Inter can cover all 24 official EU
  languages with one face. Natively this holds only after a separate
  atlas-budget decision (section 9, step 5).
- An unslashed zero in prose and the clock (measurement, section 6.4). The
  slashed zero there was both reviewers' main objection to Atkinson Hyperlegible
  Next.

The project owner's recorded priorities do not decide the tie. Readability
first does not, because the readability and accessibility judgements differ by
0.3 weighted points, less than the variation between replicate rounds. Nor does
the instruction of 14 November 2026, "Prioritize readability and elegance
equally": readability is tied as above, and no criterion of the rule scores
elegance as such. The
unified-family preference (the brief: "The user prefers a unified family across
wordmark and UI") is not decided by the UI face alone: the brand judgements
slightly favour Atkinson Hyperlegible Next (5.8 against 5.0), and BRAND.md v2
has since chosen a Saira wordmark by blind review
([logotype review](../verification/logotype-review-20261114/README.md)). With
either UI face the wordmark and the UI are therefore a pairing, which BRAND.md
records as a departure from this preference that the project owner can
redirect.

**Lead selection, 14 November 2026: Inter 4.001 for `type-1.0`
(reversible).** The project owner's continuation instruction of 14 November
2026 lets the lead resolve ordinary typography decisions, "including Inter
versus Atkinson", as "a justified, reversible decision" that preserves "the
recorded tie and uncertainty". The lead design agent (an AI agent) therefore
selects Inter 4.001 with `cv05 cv08 ss01`. The tie above stands. The selection
is a judgement, not a measured advantage; it is not the project owner's
approval (the merge of #322 is not read as one) and not a test of human
preference. Reasons, beyond the two above:

- The scores do not separate the faces, so the cost of each choice decides.
  The `type-1.0` sizes and features, the only built native atlas (section 9),
  the finalist checks and the Orbitaal v2 board are specific to Inter. Choosing
  Atkinson Hyperlegible Next would redo the scale at 16/17/14 px and the atlas
  for no measured gain.
- With readability tied, the elegance half of the owner's priority is judged on
  the recorded rows that bear on calm everyday text: the unslashed zero in prose
  and the clock, the time kept in the file table, and the higher
  OS-consistency judgement (8.5 against 7). This weighting is the lead's
  judgement.
- Accepted costs, carried as open items: the lower native-legibility and
  accessibility judgements (6.7 against 7.6; 7.5 against 8), soft 13 and 15 px
  native text, O and 0 that differ only in width until the `zero` code glyph
  exists, and the deviations of the built atlas (section 8.2).

Reversal: the project owner can redirect to Atkinson Hyperlegible Next at any
time; that redoes `type-1.0` at 16/17/14 px, regenerates the atlas and reruns
the finalist checks. Reverting the commit that records this selection restores
the proposal status. The native default does not change with it: the
`inter-atlas` feature stays off by default until QEMU evidence, the oracle
review and the `ui/native` licence and package entries justify a change
(section 10).

**Choosing Inter or Atkinson Hyperlegible Next: gains and costs**, on the same
rows:

| Row | Choosing Inter | Choosing Atkinson Hyperlegible Next |
|---|---|---|
| Weighted total (judgement) | Cost: 72.45 | Gain: 73.00 |
| Blind part, without the native column (judgement) | Gain: 65.75 | Cost: 65.40 |
| Native legibility (judgement) | Cost: 6.7 | Gain: 7.6 |
| Readability (2b, judgement) | Gain: 8 | Cost: 7.8 |
| Accessibility (2b, judgement) | Cost: 7.5 | Gain: 8 |
| OS consistency (judgement) | Gain: 8.5 | Cost: 7 |
| Brand and wordmark fit (judgement) | Cost: 5 and 4.8 | Gain: 5.8 and 5.3 (both low) |
| Language coverage (measurement) | Gain: Greek, Cyrillic and arrows in the font, usable by a browser; natively ↑↓ in the first atlas, Greek and Cyrillic after a budget decision | Cost: none of them; a second face or icons needed |
| Zero (measurement) | Gain: unslashed in prose and the clock. Cost: no slashed zero for identifiers until `zero` is applied and a native code glyph is decided | Gain: O and 0 separated in identifiers. Cost: slashed in prose and the clock; cannot be switched off |
| Native word space, measured lines (measurement) | Cost: correction required for 15sb and 15r (one collapse, three ties) | Cost: correction required for 14r, 16r, 16sb and 17r (four ties); three records unchecked |
| File-table truncation (2b, judgement) | Gain: keeps the time (`gisteren 16:30`) | Cost: loses the time (`gisteren 16:…`); not rebutted (section 6.2) |
| Other native defects named in 6.3 (judgement) | Cost: secondary line lighter and smaller in light mode at 100 %; O and 0 differ only in width | Cost: tabular `1` leaves gaps (`1 3 nov`); zero can look like `Ø` at 100 % |
| Pair distinctness (measurement) | Inconclusive; section 4 favours Inter at 100 % | Inconclusive; section 4 favours Atkinson at 200 % (0.276); 16 px rn/m 0.039, under the section 4 gate |
| Long-text leading (judgement) | Tightest in the specimen, a result of its proportional line height; that the 26 px leading of `type.reading` answers it is an untested assumption | Most generous in the specimen, for the same reason |
| Type scale | `type-1.0` as written | `type-1.0` redone at 16/17/14 px for UI, reading and small text |

**Atkinson Hyperlegible Next as an optional reading face: considered, not
adopted.** Both reviewers made it conditional on limiting its slashed zero to
technical contexts, and section 6.4 measured that the zero cannot be switched
off. A user-selectable reading face (F1) remains an open item.

**Exo 2 is not selected for text.** It had the lowest accessibility in rounds
2 and 2b, and of the three finalists also in round 1 (7 against 8 and 8, on the
stimuli section 6.1 discloses), and the lowest or tied-lowest readability (tied with Atkinson Hyperlegible Next in round 2).
The reviewers of rounds 2 and 2b both judged its O and 0 almost
indistinguishable, and it carries the square-grotesque risk of assumption A1.
Section 6.4's proxy, with proportional digits, also places O/0 among its
weakest pairs at 17 px (0.235); with the tabular digits the tokens specify,
that pair was not measured. It is recorded as the face with the most brand
character (brand and wordmark fit 8.5 in rounds 2 and 2b) and was a candidate
in the logotype review.

**Logotype**, decided in BRAND.md v2, not here. A blind AI review of seven
candidates, under a rule committed before judging, compared route (a), a
wordmark set in Inter, route (b), a geometric wordmark in a squared or wide
display face, as rounds 2 and 2b suggested, and route (c), the v1 Archivo
wordmark as control. It selected route (b): Saira 600 at 125 % width
([logotype review](../verification/logotype-review-20261114/README.md)).
BRAND.md records it as a departure from the one-family preference that the
project owner can redirect. The original raster wordmark stays the reference
and reversible fallback.

## 8. Typography token specification

Version `type-1.0` (14 November 2026). Applies to the shell (Horizon, Orbit,
Zenit, Signaal, Nebula, Kern, Haven), the first-party apps and the prototype.
Values are CSS px at 100 % scale. The native renderer is to use the same pixel
sizes, so the browser and QEMU show the same scale. An Inter atlas with nine of
these records exists behind the `inter-atlas` feature, off by default (host
evidence only, no QEMU; deviations in section 8.2); the default build still
draws the Open Sans records of section 9. The sizes and features are
specific to Inter; a redirect (section 7) changes them.

### 8.1 Families

| Token | Value | Use |
|---|---|---|
| `font.family.ui` | Inter 4.001 (`ui/design/fonts/`); lead selection of 14 November 2026, reversible; the project owner can redirect it (section 7) | All interface and reading text |
| `font.family.wordmark` | Outlined logotype traced from Saira 600 at 125 % width (BRAND.md v2, section 7) | Wordmark only; never typed text |
| `font.features.ui` | `cv05 cv08 ss01` | Always on |
| `font.features.digits` | `tnum` on 0–9 only | Every digit: the native atlas cannot select by context, so prose digits are tabular too |
| `font.features.code` | `zero` (slashed) | Planned for paths, identifiers and hashes; not applied in any prototype yet; native support open (section 9) |
| Fallback (browser) | `system-ui, sans-serif` | Prototype only |
| Fallback (native) | U+FFFD from the same atlas | Characters outside the atlas. Deviation: Inter 4.001 has no U+FFFD glyph, so the `inter-atlas` fallback slot holds `.notdef` (section 10) |

### 8.2 Roles

| Role token | Size / line height | Weight | Tracking | Native record (100 % / 200 %) | Use |
|---|---|---|---|---|---|
| `type.caption` | 13 / 18 | 400 | 0 | 13r / 26r | Non-essential metadata only: timestamps, secondary counts, column hints |
| `type.body` | 15 / 22 (20 in dense rows) | 400 | 0 | 15r / 30r | All UI text: menus, buttons, list rows, settings, fields, notifications |
| `type.body-strong` | 15 / 22 | 600 | 0 | 15sb / 30sb | Buttons, selected navigation, row titles, labels that carry state |
| `type.reading` | 16 / 26 | 400 | 0 | 16r / 32r | Documents and long text (Notities, help, licence) |
| `type.title` | 22 / 28 | 600 | 0 | 22sb / 44sb | Window, page and dialog titles; Signaal group heads |
| `type.display` | 48 / 56 | 400 | 0 | 48r / 48r magnified 2× | Greeting, clock, boot and sign-in only |

The caption token (13 px) equals the specimen's secondary-text size, not its
12 px captions; the body and reading sizes are the UI and reading sizes
reviewed in rounds 2, 2b and 6.3. The title (22 px) and display (48 px) token
sizes were not reviewed: the specimen's headings were 28 px and its clock 56 px
with −0.01 em tracking, for every face (section 6.2). Line heights were not
reviewed at the token values either: the specimen used 1.43 × size for UI text and 1.55 × size for reading
text (the Notities body: 16 px at 1.6 for every face).

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
- Native word space, on the four measured specimen lines
  (`finalist-checks.txt`, section 6.4): 15sb collapses on the 600 line and ties
  on the reading and secondary lines, and 15r ties on the secondary line. A
  space advance 2 px wider removes every tie and collapse on those lines, in
  every record. The other `type-1.0` records (13r, 16r, 22sb, 26r, 30r, 30sb,
  32r, 44sb, 48r) show none on those lines. On the measured lines, counting a
  tie as section 6.4 does, the correction is required for 15sb and 15r and for
  no other record; whether a larger text corpus finds more is open.
- Deviations of the built `inter-atlas` (host evidence only, no QEMU;
  `docs/verification/native-inter-atlas-20261114/`). It holds 13r, 15r, 15sb,
  22sb and 48r, and 26r, 30r, 30sb and 44sb at 200 %. The reading records 16r
  and 32r and the dense 20 px body line height are not built, so
  `type.reading` has no native record yet. The space is widened by 2 px in
  15sb and by 4 px in 30sb; 15r, for which the measured lines require the
  correction, is not widened, and 30sb, for which they do not, is.

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
- Paths and identifiers in `type.body`, with the code feature once it is
  implemented (section 8.1); truncate in the middle (`Thuis-NAS › … › Tuin`),
  never in the file name.

### 8.5 Localisation

- The first native atlas holds Latin-1, `–—‘’“”•…€` and `›↑↓` (a superset of
  the prototype copy's characters); EU Latin follows after the read-only budget
  check; Greek and Cyrillic need a project decision (section 9). Browser
  coverage follows the face.
- Allow 30–40 % text expansion (German, Finnish): buttons grow, they do not
  truncate.
- Truncation uses `…` (U+2026), with the full text in a tooltip and the
  accessible name.

### 8.6 Accessibility settings (WCAG 1.4.4, 1.4.8, 1.4.12)

- Text size: 100 % and 200 % natively today; 125 % and 150 % are open (section
  9, item 7).
- Ruimere tekst (wider spacing): letter spacing 0.12 em, word spacing 0.16 em,
  line height 1.5, paragraph spacing 2 em, without loss of content (F9, F10).
  Specified, not yet implemented in any prototype; natively `text.rs` has no
  letter- or word-spacing parameter.
- No reading-face setting in `type-1.0`; see section 7 and the open items.

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
| Full proposed scale (20 records)¹ | 558.4 KiB | 1028.3 KiB | 1451.5 KiB |
| Reduced set (9 records): 12r 15r 15sb 22sb 48r, and 24r 30r 30sb 44sb at 200 % (display magnified) | 215.6 KiB | 396.6 KiB | 559.4 KiB |
| Section 8 role set (11 records): 13r 15r 15sb 16r 22sb 48r, and 26r 30r 30sb 32r 44sb | 261.6 KiB | 481.0 KiB | 679.1 KiB |

¹ The script skips glyphs larger than the generator's 96 px bound (3 for
Latin-1, 6 for the larger sets), so these totals are low and this record set
could not be generated as is.

The same role set for Atkinson Hyperlegible Next (14r 16r 16sb 17r 24sb 52r,
and 28r 32r 32sb 34r 48sb) needs 265.5 KiB for Latin-1 and 503.0 KiB for EU
Latin; it has no Greek or Cyrillic glyphs. Only the UI and small sizes follow
the equal-x-height rule; reading is UI + 1 px as for Inter (x-height 8.4 px
against Inter's 8.7 px), and title and display are scaled approximately.
Reproduce with `atlas_estimate.py FONT [features] weight:px,...`. The estimates
cover the 194-character Latin-1 set; the ten further characters of the
proposed first atlas (`–—‘’“”•›↑↓`) are not estimated. Every record is cut at
its pixel size on the optical-size axis (clamped to Inter's 14–32), so the
200 % records use the doubled optical size, while a browser at 200 % keeps the
optical size of the CSS size (assumption from CSS Fonts 4, not tested here).
Cutting 200 % records at the 100 % optical size is an open choice.

**Feasible route** (no browser or webview in the native path):

1. Vendor the chosen family's TTF with its OFL text and SHA-256 under
   `ui/design/fonts/`, and add a provenance entry. Done for Inter 4.001, the
   selected face (`ui/design/fonts/FONTS.md`).
2. Extend the generator to emit a second atlas beside the Open Sans atlas, never
   regenerating `atlas.rs` in place. Veld and the SDK examples keep Open Sans
   until each is migrated and its text-metric oracles are reviewed. Done for
   Inter behind the `inter-atlas` feature, off by default
   (`ui/graphics/scripts/generate_inter_atlas.py`; host evidence only, no QEMU:
   `docs/verification/native-inter-atlas-20261114/`); `atlas.rs` and the Open
   Sans outputs are unchanged.
3. Bake the alternates (`cv05`, `cv08`, `ss01`) and tabular digits into the
   glyph selection, a 1:1 substitution needing no shaping. Tabular figures apply
   to digits only: in Inter, `tnum` also widens the hyphen. The character set is
   Latin-1, `–—‘’“”•…€`, U+FFFD and `›↑↓`, a superset of the characters the
   prototype copy uses (`finalist-checks.txt`). Apply the word-space correction
   of section 8.2. The slashed code zero needs a second glyph that the atlas
   cannot select without shaping: a separate code record or an explicit
   `text.rs` call, which is an open interface decision.
4. Keep the token scale equal to the native records, so the browser and QEMU
   show the same sizes. Start with Latin-1 (261.6 KiB of glyph data for the
   section 8 role set, against 188.5 KiB today) and measure it against the built
   ELF's read-only segment; the Open Sans atlas leaves `ui/native` only if
   nothing there references it any more. Measured for the nine built records
   (host evidence only): the linked `materiaal-app` read-only segment is
   293 248 B with the default Open Sans atlas and 353 736 B with
   `inter-atlas` instead, of the 512 KiB limit. The feature replaces the Open
   Sans atlas rather than adding to it; both together would need about
   627 398 B (arithmetic, not measured), over the limit, so the two are never
   linked into one binary. Measure EU Latin the same way before adopting it.
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
8. **Rendering quality** (F15, A4): that 16 coverage levels without hinting
   soften stems at 12–15 px, and that 200 % largely removes this, is an
   assumption, not measured. The `inter-atlas` record's AI viewing of host
   images (not a measurement) reports 13 and 15 px text soft and 22 px and
   every 200 % record crisp. A gamma-correct blend check and
   an 8-bit coverage option are open items; each would cost bytes.

**Licensing.** All candidates are under SIL OFL 1.1, which permits use,
embedding, modification and redistribution with the licence text, but not
selling the fonts on their own. None of the copyright lines for Inter, Atkinson
Hyperlegible Next, Exo 2 or Sora declares a Reserved Font Name, so a subset or
rasterised derivative may keep the family name. The OFL text must accompany the
OS (licence screen and provenance), as Apache-2.0 does today for Open Sans. A
rasterised atlas is treated as a derivative of the font software for this
purpose: this is a conservative reading, not legal advice.

## 10. Open items

- Confirmation or redirect by the project owner of the lead's reversible Inter
  selection (section 7). Beyond section 1 (readability and accessibility over decorative
  styling), the project owner's continuation instruction of 14 November 2026
  states "Prioritize readability and elegance equally"; section 7 applies it,
  and it does not decide the tie.
- The status line's "pre-registered" cannot be verified from the repository
  (section 6.2). Future decision rules are committed before the reviews run.
- Code zero: Inter's `zero` for identifiers is planned, not implemented in any
  prototype; the native code glyph and its `text.rs` interface are undecided.
- A user-selectable reading face (F1): none in `type-1.0`; Atkinson
  Hyperlegible Next was not adopted (section 7).
- Fit: commit a DOM measurement script with stored output, or keep fit out of
  the comparison (section 6.2).
- Word-space correction: required for 15sb and 15r on the measured lines; check
  every record with a representative text corpus, and Atkinson Hyperlegible
  Next's unchecked 24sb, 48sb and 52r if it is chosen (section 8.2).
- Browser specimen: re-render with every element at equal x-height and with
  equal absolute leading, including the 26 px `type.reading` leading (section
  6.2); record the `index.html` commit for each capture.
- Native 200 % records: cut at the doubled or at the 100 % optical size
  (section 9).
- Read-only budget: measured for each atlas alone (section 9, step 4); EU Latin
  and the missing records still need the same measurement.
- Built `inter-atlas` deviations (section 8.2; host evidence only, no QEMU):
  15r is not widened although the measured lines require it; the reading
  records 16r and 32r and the dense 20 px line height are not built; the
  fallback slot holds `.notdef` because Inter 4.001 has no U+FFFD glyph (draw
  an original replacement glyph, or change section 8.1). The atlas record's
  own open items list the rest, including the missing `ui/native` licence and
  package entries and the Open Sans pixel-hash oracle.
- Wider spacing (Ruimere tekst): not implemented in any prototype, and
  `text.rs` has no spacing parameter (section 8.6).
- Tooling outside this record: `measure_fonts.py` to measure `cv11` and 15 px;
  `atlas_estimate.py` to cover the ten extra characters and the row tables;
  pair distinctness with tabular digits and 4-bit coverage as the atlas draws
  them; `native_specimen.py` to round like `blend`; the OpenAlex lookup output
  to be stored.
