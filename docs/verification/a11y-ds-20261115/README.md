# UX-A11Y-DS: accessibility axes check (ds-1.1)

Task UX-A11Y-DS, repair round 1, 15 November 2026. Branch
`claude/baryon-os-design-pipeline-20261115`; base commit `47c2b78d`
(uncommitted working tree at the time of the run). Design artefact evidence,
not a working OS; no user test, and AI review is not user evidence.

## Command

```
node docs/verification/a11y-ds-20261115/check.mjs .
node docs/verification/a11y-ds-20261115/check.mjs . --self-test
```

Node 18 or later, no dependencies. Exit 0 only when every check passes.

## What it checks

1. The dark, light and contrast theme blocks define the same colour, place,
   material and elevation tokens.
2. Every axis value defines the tokens that `DESIGN-SYSTEM.md` §21.2 promises;
   each device-preference block equals its attribute block; an explicit
   default value overrules the device; `data-motion` full and reduce.
3. Every `var()` is defined somewhere, and (8) every `var()` used by
   `components.css` resolves in each theme's base cascade with no axis set.
4. No colour literal outside `tokens.css`: every declaration value in
   `components.css` and the gallery `<style>` (custom properties included);
   hex, colour functions, the 148 CSS named colours and the system colours,
   in any case and inside gradients, borders, shadows and `color-mix()`.
5. Every axis value in the §21.2 table has a CSS selector.
6. Contrast, for every combination of the five colour axes in the three themes,
   on the default and the neutral surface, plus the device-preference cases at
   both light intensities (588 cases):
   every foreground of §4 on every surface alone and under hover,
   hover-strong, selection and the secondary-button fill (place colours also
   on the pill over those), plus primary button, Kern, notice icon, tile,
   inspector, glass, Orbit, neutral twins and the progress track. Translucent
   colours are composited and quantised to 8 bits like rendered pixels;
   dimmed light is sRGB × 0.85 (the CSS `brightness(.85)` filter), quantised.
   Floors: text 4.5:1 (7:1 under increased contrast and in the contrast
   theme), lines, focus, blade and places 3:1.
7. Colour-perception palettes keep the three places at least ΔE 40 apart
   under Machado et al. (2009) simulation at severity 1.0.
9. The contrast theme has no translucent colour under any axis
   (`transparent`, meaning no paint, is allowed).

`--self-test` applies 26 mutations: the 21 of the repair-1 verifier
(`verify-0/mutate.py`: T1–T4 missing tokens, L1–L8 colour literals, P1–P9
missing or weak pairs), re-targeted by selector and token name, plus five
repair-1 cases (dimmed compensation removed, increased contrast weakened at
× 0.85, a translucent warning line in the contrast theme, an undefined
`var()`, a promised selector without CSS). Each must fail.

## Recorded output (repair 1)

Last lines of `check.mjs .` and the self-test count (full output in
`build/design-evidence/a11y-ds/repair-1/check-final.txt` and
`check-selftest.txt`):

```
info  contrast: 588 theme x axis cases (incl. 12 device-preference cases, each at both light intensities), 187768 pair evaluations
summary: 0 failure(s)
self-test: 26 of 26 mutations caught (exit 0)
```

Before the repair the same checker reported 1364 failures on the previous
tokens (for example dark `--color-line-strong` on raised + hover-strong
2.670:1 and dark increased `--color-text-subtle` 6.653:1, both dimmed; light
deutan `--place-device` 2.764:1 undimmed).

## Browser measurements (headless Chromium, design evidence)

Scripts and raw output: `build/design-evidence/a11y-ds/repair-1/` (not in
Git). Chromium flags and the CDP emulation used are in each script.

| Measurement | Result |
|---|---|
| Reading measure, Inter reading role, characters per rendered line (`measure-ll.mjs`) | 80ch: 100–108; 58ch: 70–78 (new baseline); 45ch: 53–61; 37ch: 44–50 (new short value) |
| 200 % zoom, CDP device-scale 2 at 720 and 640 CSS px, plus `data-scale="2"` specimens (`scale3.mjs`) | `.ds-empty`: 13 instances, 0 clipped elements; narrowest text column 102 px (was 10–39 px). The page still overflows 64 px at 640 CSS px (page chrome, open) |
| Held ArrowDown on Thema, Contrast and Lichtsterkte, 33 Hz for 2 s, with and without reduced motion (`flashkey2.mjs`) | at most 3 preview changes and 1 flash pair in any 1 s (was 7.4 flash pairs per second) |
| Prikkels pixel difference of the Instellingen preview (`a11yshots.mjs`, `stimulus-diff.txt`) | dark calm 6.5 %, rich 13.1 %; light calm 11.2 %, rich 13.9 % of the area changes by at least 3/255; contrast theme 0 % (stated in the settings group) |

## Not verified

No screen reader, magnifier or user test; no physical display luminance;
colour-perception palettes by simulation only.

## Hashes of the checked files (SHA-256)

```
c4513523d43435c64691749d7d8b9771c3657fa8db9cdcb01835aa08fe96ee39  ui/design/system/tokens.css
0e1fff4e7f1cb479531bc93a29451677a0adae2ed9cb2c43ce480a23809690c6  ui/design/system/components.css
d491e1368ea444e461c251085fa8b2506c9b1812da909c3358702de9a7a9652b  ui/design/system/index.html
f57d6f8e1c50d6a576696ddde1278ef957fbccf46fb6823914f312287718ae6c  ui/design/system/gallery.js
134301990d319dc14e0a500c638863aad68c33059116704e37e15c31fb690122  ui/design/system/icons.js
1d803ec1ce5929c4c16249a5525f3542e341bd7c49f98e43003b5793918f2d7e  docs/design/DESIGN-SYSTEM.md
1fc0fff85a3e698c90709e50f7d8ae158bb087166e18d2b0ea3de99d912391c7  docs/design/accessibility/ACCESSIBILITY-MATRIX.md
8258238463a2c789ceadec61e2650ef7efd3ff57741ff1b47c7c584953542675  docs/verification/a11y-ds-20261115/check.mjs
```
