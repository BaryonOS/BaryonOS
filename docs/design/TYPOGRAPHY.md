# BaryonOS typography

Status: research, measurements and native assessment complete; specimen review,
selection and token specification follow in sections 6–8. Date: 13 November 2026
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
  the shipping alternates baked in, at 100 % and 200 % in dark and light.
  Output: `typography/native/`. This is a build-time simulation, not a QEMU
  capture. `--blind` writes coded copies for review.

## 6. Specimen review

Pending: browser captures at 100 % and 200 % (dark and light), and a blind review
by an independent agent. The review is AI judgement, not user testing.

## 7. Selection and scoring

Pending (after section 6).

## 8. Typography token specification

Pending (after section 7).

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

Atkinson Hyperlegible Next and Exo 2 are within about 5 % of Inter.

**Feasible route** (no browser or webview in the native path):

1. Vendor the selected family's TTF with its OFL text and SHA-256 under
   `ui/design/fonts/`, and add a provenance entry.
2. Extend the generator to emit a second atlas beside the Open Sans atlas, never
   regenerating `atlas.rs` in place. Veld and the SDK examples keep Open Sans
   until each is migrated and its text-metric oracles are reviewed.
3. Bake the alternates (`cv05`, `cv08`, `ss01`) and tabular digits into the
   glyph selection, a 1:1 substitution needing no shaping. Tabular figures apply
   to digits only: in Inter, `tnum` also widens the hyphen.
4. Keep the token scale equal to the native records, so the browser and QEMU
   show the same sizes. Start with Latin-1 (fits like today), then measure the
   EU Latin atlas against the built ELF's read-only segment before adopting it.
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
