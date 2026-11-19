# BaryonOS design system ds-1.1

**ds-1.1 (15 November 2026, task UX-A11Y-DS).** Adds the two-tier
accessibility model (§21): a baseline that holds in every theme with no
setting, and independent, combinable axes with device mapping, a settings
place (Instellingen › Toegankelijkheid), reset and undo. It also reworks the
empty, loading and error states and the progress bar (§10), makes every loop
finite (§12) and removes the stripe fill. No class, token or keyboard
behaviour was removed or renamed (§18). The ds-1.0 text below stays valid
where §21 does not change it.

Status: **design (proposal)**, version `ds-1.1` (15 November 2026; `ds-1.0` of 14 November 2026), branch
`claude/baryon-os-ui-ux-design-152adb`, baseline commit `573840e0`. ds-1.0 is
derived from the Orbitaal v2 board
(`ui/design/concepts/orbitaal-v2/index.html`). Simulation evidence (browser
renders of the gallery) and native evidence are recorded separately; neither
is part of this document's claim. The project owner can redirect any part of
it. Inter 4.001 is a reversible lead selection (`docs/design/TYPOGRAPHY.md`
§7 and §8.1); the Saira wordmark is the project owner's to redirect
(`docs/design/BRAND.md` §4). This is a design artefact, not a working OS.
Repair round 1 (same day) applied an independent verifier's findings: new
values for `--color-line-strong`, `--color-text-subtle`, `--color-text-disabled`
and their neutral twins (§4.1), the line height of weight-600 text (§6), a
compact and a neutral specimen per theme in the gallery, and this document.
Its evidence is in `build/design-evidence/ds-1.0/repair-1/`.
Repair round 2 (same day) applied a second independent verifier's findings: a
base `[hidden]` rule (§10), a 1 px gap between the contrast-theme focus ring
and white marks (§10, §13, O-K2), the ask-state privacy instrument in the
200 % strip (§14), and corrections to §6, §10, §15 and §16. Its evidence is in
`build/design-evidence/ds-1.0/repair-2/`.
Task UX-D-C1 (14 November 2026) applied the owner decision on app permissions:
two prompt kinds with three equal choices, durations defined once, a
four-field grant key, revocable grant rows and the Appmachtigingen path
(§11.1); its evidence is in `build/design-evidence/ds-1.0/c1/writer/`. Repair
round 1 (15 November 2026) added the destination to the grant scope, the grant
row's reach badge and duration chip (`.ds-grant-reach`, `.ds-grant-until`, a
760 px list cap), one `alertdialog` on the modal `<dialog>` and a race-free
prompt resolver; evidence in `build/design-evidence/ds-1.0/c1/repair-1/`.
UX-D-C1 repair round 2 (15 November 2026; not the ds-1.0 repair round 2
above, whose evidence is `build/design-evidence/ds-1.0/repair-2/`) made the
modal prompt scroll inside the viewport at 200 % (§10, §14), named Intrekken
by all four key fields (§11.1), gave the "Altijd" chip its own glyph `altijd`
(§8), aligned "herstarten" with the shell's "opnieuw opstarten" and recorded
open items O-G12 to O-G14; evidence in
`build/design-evidence/ds-1.0/c1/repair-2/`. Boundary gate for UX-D-C1:
`git diff --stat HEAD -- <protected paths>` (uncommitted edits; empty) plus
`git log --oneline 46957bac..HEAD -- <protected paths>`, which lists only the
merge of main `e186ec1b` (#335, `tools/package_token_guard.py`, not a C1
change); `git diff --stat 46957bac` over the same paths therefore prints that
one file (`c1/repair-2/boundary-start.txt`, `boundary-end.txt`).

Requirements input: [WORKSPACE-REQUIREMENTS.md](suite/WORKSPACE-REQUIREMENTS.md)
(S0, accepted 14 November 2026). ds-1.0 applies its decisions D1 (colour rule
scope, no status hue), D2 (densities, 24 × 24 targets), D3 (neutral surfaces;
values PROVISIONAL), D4 (region names) and D9 (single-pointer selection). S0
values marked PROVISIONAL stay provisional here.

## 1. Purpose, scope and evidence level

ds-1.0 is the shared visual and interaction vocabulary for the BaryonOS shell
and first-party apps: tokens, icons, components and their states, keyboard
rules, and the trust indicators that only the shell may draw.

| File | Content |
|---|---|
| `ui/design/system/tokens.css` | All tokens: type, space and density, radii, motion (`:root`); colour, place, material and elevation per theme (`:root, [data-theme="dark"]`, `[data-theme="light"]`, `[data-theme="contrast"]`); neutral-surface twins and the `[data-surface="neutral"]` rule; compact density; reduced motion. Custom properties and `@font-face` only. |
| `ui/design/system/components.css` | `.ds-` component classes, states, the focus ring, the 200 % rule, reduced-motion and high-contrast overrides. |
| `ui/design/system/icons.js` | `ICONS`: 53 keys of inner SVG markup on a 24-unit grid; re-exported unchanged by `ui/design/prototype/icons.js`. |
| `ui/design/system/index.html`, `gallery.js` | Gallery ("BaryonOS ontwerpsysteem ds-1.0"): every component in the three themes, a compact-density strip and a neutral-surface panel per theme (S0 D2, D3), a 200 % strip, the icon set, the type scale and token swatches. Keyboard behaviour is wired in `gallery.js` and is a simulation. |

**Not part of ds-1.0:** `ui/prototype/tokens.css` is the frozen Materiaal P02
token file compiled into `ui/native/src/design.rs`
(`docs/design/NATIVE-FEASIBILITY.md` §2). ds-1.0 does not read, extend or
replace it.

| Evidence level | What exists for ds-1.0 | Where |
|---|---|---|
| Design | This document, the four system files above, the v2 board | `ui/design/system/`, `ui/design/concepts/orbitaal-v2/` |
| Simulation | Headless Chrome renders of the gallery and the icon sheet; scripted key and pointer events; computed token and contrast checks. All runs by AI agents. | `build/design-evidence/ds-1.0/` (ignored by Git, untracked) |
| Native | None. No ds-1.0 token or component is drawn by `ui/graphics` or `ui/native`. | §17 |

At the time of writing `ui/design/system/` and `ui/design/prototype/icons.js`
are new and uncommitted on the branch.

## 2. Principles

| Principle | Rule in ds-1.0 | Source |
|---|---|---|
| Familiar conventions (ADR-058) | Desktop, Orbit (dock), launcher, search, windows with controls on the right, and the default shortcuts follow Windows, macOS and Linux conventions; additions never replace the familiar path. | `AGENTS.md` (ADR-058); `docs/implementation/DESKTOP-SHELL.md` §1, §2 |
| Materiaal is the data and permissions model | History, rights and storage status come from Materiaal (inspector sections Versies, Toegang, Opslag); Materiaal is not a mandatory main screen. | `AGENTS.md`; DESKTOP-SHELL.md §2 (Files row); `docs/design/NAMING-AND-IA.md` §4 |
| Colour means place | Green = this device, blue = NAS, red = internet; never decoration, never alone. Place and state are always shape + icon + text (dot device, ring with dot NAS, open ring internet, square error). Scope: chrome, trust surfaces and the place badge; user content in work regions is unrestricted. | `docs/design/CONCEPTS.md` (Selection, adopted grafts); S0 D1 |
| No status hue | Error, warning and success use `text.primary`, an icon, a word and (for error) the square `.ds-status-marker`. ds-1.0 ships no red status colour. | S0 D1; `tokens.css` (`--color-danger`, `--color-warning`, `--color-success`) |
| Calm, premium dark and light | Two first-class themes with their own values (not inversions); dark is the default. High contrast is a full third theme. | `tokens.css`; CONCEPTS.md Selection |
| Security Floor | Trust indicators (place badge in trust surfaces, privacy instrument, permission prompt) are drawn only by the shell. A familiar or trusted look grants no capability; secrets are entered only on the session-service route. | Drawing, secret entry and "a trusted look grants no capability": DESKTOP-SHELL.md §1 ("Rights remain rights"); `docs/implementation/PROFILES-AND-USERS.md` §3 (signing in, locking and signing out) and §4 invariant 2. Security Floor layering: `docs/LOCAL-PRIVACY-CONTRACT.md` §4.1. `docs/design/BRAND.md` §5 |

## 3. Tokens

### 3.1 Naming and selectors

| Prefix | Meaning |
|---|---|
| `--color-*` | Neutral roles: surfaces, lines, text, focus, selection, states, status aliases |
| `--place-device`, `--place-nas`, `--place-internet` (`--place-internet-line`) | Place colours |
| `--material-*` | Kern (trusted surface), glass, launcher, inspector, Orbit, app tiles |
| `--elev-*` | Shadows (per theme) |
| `--font-*`, `--type-<role>-size/-line/-weight` | Type-1.0 |
| `--space-*` | 4 px grid steps and density roles |
| `--radius-*` | Corner radii by role |
| `--motion-duration-*`, `--motion-ease-*` | Motion |

| Selector | Effect |
|---|---|
| `:root, [data-theme="dark"]` | Dark (default) |
| `[data-theme="light"]` | Light |
| `[data-theme="contrast"]` | High contrast |
| `[data-surface="neutral"]` | Re-points 30 roles at the achromatic twins of the nearest theme (S0 D3, values PROVISIONAL) |
| `[data-density="compact"]` | Compact sizes (S0 D2, PROVISIONAL) |
| `@media (prefers-reduced-motion: reduce)` (unless `data-motion="full"`), `[data-motion="reduce"]` | Every `--motion-duration-*` = 0ms |
| `data-contrast`, `data-light-intensity`, `data-transparency`, `data-colour-perception`, `data-motion`, `data-stimulus`, `data-text-spacing`, `data-line-length`, `data-targets` (ds-1.1) | Tier-2 accessibility axes; values, owners, device mapping and precedence in §21 |
| `@media (prefers-contrast: more)`, `(prefers-reduced-transparency: reduce)`, `(forced-colors: active)` (ds-1.1) | Device mapping of the axes when no explicit value is set (§21.3) |

Every theme block defines the same 88 themed names (87 before repair round 1
added `--color-surface-surround`), so an element with its own
`data-theme` re-resolves every colour, place, material and elevation token
(`build/design-evidence/ds-1.0/checks.log`, check 3a; re-run in `build/design-evidence/ds-1.0/repair-1/check-report.txt`). The neutral rule
resolves against the nearest theme; a nested `data-theme` panel inside a
neutral region gets the plain theme values back, which keeps trust surfaces
unchanged (S0 D3). ds-1.1 adds no themed name: the axis blocks only re-point
existing names, and `docs/verification/a11y-ds-20261115/check.mjs` checks that
every axis value defines the names §21 promises for each theme.

### 3.2 Themed tokens

Values are copied by script from `tokens.css`
(`build/design-evidence/ds-1.0/repair-1/tools/gen_tables.py`, a copy of `doc/gen_tables.py`). The last column is the
Orbitaal v2 variable or rule the dark value comes from (row by row in
`build/design-evidence/ds-1.0/repair-1/tools/mapping.tsv`, the tokens stage's mapping plus the repair-1 rows). † marks a value derived by
the tokens stage because v2 defines none for that theme, or changed in repair
round 1 (§4.1); derivations are
listed in §13 (contrast) and in `build/design-evidence/ds-1.0/work/mapping.tsv` (light).

<!-- BEGIN token-tables -->
**Neutral colour roles** (29)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--color-bg` | `#070912` | `#E4E9F2` | `#000` | `--s0`; contrast: rule `.hc` (background) |
| `--color-surface-panel` | `#0D1020` | `#EDF0F7` | `#000` | `--s1` |
| `--color-surface` | `#12162A` | `#FBFCFE` | `#000` | `--s2` |
| `--color-surface-raised` | `#1A1F38` | `#E3E8F1` | `#000` | `--s3` |
| `--color-surface-inset` | `#080A16` | `#FFFFFF` | `#000` | `--sin` |
| `--color-surface-surround` | `#070912` † | `#E4E9F2` † | `#000` † | derived |
| `--color-line` | `#232A47` | `#D8DEE9` | `#FFFFFF` | `--ln` |
| `--color-line-strong` | `#7882A8` † | `#6B7590` † | `#FFFFFF` | derived from `--ln2` |
| `--color-text` | `#EEF1F8` | `#111726` | `#FFFFFF` | `--tx1` |
| `--color-text-muted` | `#A9B2CB` | `#424B62` | `#FFFFFF` | `--tx2` |
| `--color-text-subtle` | `#9BA2BC` † | `#515A73` † | `#F2F2F2` | derived from `--tx3` |
| `--color-text-disabled` | `#9BA2BC` † | `#515A73` † | `#F2F2F2` † | derived from `--tx3` |
| `--color-focus` | `#EEF1F8` | `#111726` | `#FFE34D` | `--focus` |
| `--color-selection` | `rgba(220,230,245,.085)` | `rgba(18,24,33,.065)` | `#333333` † | `--selbg` |
| `--color-hover` | `rgba(220,230,245,.07)` | `rgba(18,24,33,.05)` | `#1A1A1A` † | `--hov` |
| `--color-hover-strong` | `rgba(220,230,245,.12)` | `rgba(18,24,33,.09)` | `#262626` † | `--hov2` |
| `--color-rim-a` | `rgba(226,233,255,.46)` | `#FFFFFF` | `#FFFFFF` | `--rim-a` |
| `--color-rim-b` | `rgba(150,165,220,.10)` | `rgba(70,90,140,.16)` | `#FFFFFF` | `--rim-b` |
| `--color-rim-c` | `rgba(150,165,220,.20)` | `rgba(70,90,140,.30)` | `#FFFFFF` | `--rim-c` |
| `--color-silver` | `#C9D3E6` | `#5B667F` | `#FFFFFF` † | `--silver` |
| `--color-blade` | `#E6EBF5` | `#2B3245` | `#FFFFFF` | `--blade` |
| `--color-blade-glow` | `rgba(230,235,245,.35)` | `rgba(43,50,69,.18)` | `transparent` | `--blade-glow` |
| `--color-well` | `rgba(3,6,11,.42)` | `rgba(18,30,48,.045)` | `#000` | `--well` |
| `--color-scrim` | `rgba(3,5,9,.55)` | `rgba(222,229,238,.40)` | `#000000` † | `--scrim` |
| `--color-button-primary-bg` | `linear-gradient(180deg,#F4F6FA,#D9DFEA)` | `linear-gradient(180deg,#232B42,#121828)` | `#FFFFFF` † | `--pri-bg` |
| `--color-button-primary-bg-hover` | `linear-gradient(180deg,#FFFFFF,#E8ECF4)` | `linear-gradient(180deg,#2E3853,#1A2134)` | `#D9D9D9` † | `--pri-bg-h` |
| `--color-button-primary-text` | `#0A0D1A` | `#FFFFFF` | `#000000` † | `--pri-tx` |
| `--color-button-secondary-bg` | `rgba(220,230,245,.08)` | `#FFFFFF` | `#000000` † | `--sec-bg` |
| `--color-pill-bg` | `rgba(220,230,245,.06)` | `#FFFFFF` | `#000000` † | `--pill-bg` |

**Status (S0 D1: no status hue)** (3)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--color-danger` | `#EEF1F8` † | `#111726` † | `#FFFFFF` † | derived from `--tx1` |
| `--color-warning` | `#EEF1F8` † | `#111726` † | `#FFFFFF` † | derived from `--tx1` |
| `--color-success` | `#EEF1F8` † | `#111726` † | `#FFFFFF` † | derived from `--tx1` |

**Place** (4)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--place-device` | `#42D27F` | `#127A43` | `#7CF0A8` | `--green` |
| `--place-nas` | `#5AA2FF` | `#1061D6` | `#8EC5FF` | `--blue` |
| `--place-internet` | `#FF5C50` | `#CC3328` | `#FF8A80` | `--red` |
| `--place-internet-line` | `rgba(255,93,80,.5)` | `rgba(217,52,42,.5)` | `#FF8A80` † | `--ask-line` |

**Materials** (19)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--material-kern-bg` | `#0B0E1B` | `#EEF1F5` | `#000000` † | `--kern-bg` |
| `--material-kern-tx` | `#EEF1F8` | `#111726` | `#FFFFFF` † | `--kern-tx` |
| `--material-kern-tx2` | `#B3BCD3` | `#424B62` | `#FFFFFF` † | `--kern-tx2` |
| `--material-kern-line` | `#262E4C` | `#D8DEE9` | `#FFFFFF` † | `--kern-line` |
| `--material-kern-rim` | `linear-gradient(135deg,#EEF2F8,#9AA5BE 35%,#56607E 70%,#7C88A6)` | `linear-gradient(135deg,#FFFFFF,#C9D3E6 40%,#78819A 75%,#AEB8C6)` | `#FFFFFF` † | `--kern-rim` |
| `--material-kern-warn-bg` | `#22151C` | `#FBEDEB` | `#000000` † | `--kern-warn-bg` |
| `--material-kern-warn-line` | `rgba(255,108,96,.45)` | `rgba(204,51,40,.45)` | `#FF8A80` † | `--kern-warn-line` |
| `--material-kern-warn-ic` | `#FF7A6E` | `#CC3328` | `#FF8A80` † | `--kern-warn-ic` |
| `--material-glass` | `rgba(10,15,25,.60)` | `rgba(250,252,255,.62)` | `#000` | `--glass` |
| `--material-glass-line` | `rgba(205,216,232,.14)` | `rgba(20,32,50,.10)` | `#FFFFFF` | `--glass-line` |
| `--material-launch` | `linear-gradient(180deg,rgba(20,24,46,.86),rgba(11,13,27,.92))` | `linear-gradient(180deg,rgba(255,255,255,.80),rgba(240,243,250,.86))` † | `#000000` † | rule `.launch .lbg` |
| `--material-launch-opaque` | `#0F1328` | `#F4F6FB` | `#000000` † | `--launch-opaque` |
| `--material-insp-bg` | `linear-gradient(180deg,#181D38,#12162B 70%)` | `#FFFFFF` | `#000000` † | `--insp-bg` |
| `--material-orbit-bg` | `linear-gradient(180deg,rgba(32,38,68,.74),rgba(13,16,31,.84))` | `linear-gradient(180deg,rgba(255,255,255,.80),rgba(240,243,250,.86))` | `#000` | `--dock-bg` |
| `--material-tile-bg` | `radial-gradient(120% 120% at 28% 0%,#2D3858 0%,#131B2F 55%,#080A12 100%)` | `radial-gradient(120% 120% at 28% 0%,#2D3858 0%,#131B2F 55%,#080A12 100%)` | `#000` | `--tile-bg` (:root); contrast: rule `.hc .tile,.hc .nuc` (background) |
| `--material-tile-fg` | `#DCE3F0` | `#DCE3F0` | `#FFFFFF` † | `--tile-fg` (:root) |
| `--material-tile-rim` | `inset 0 1px 0 rgba(255,255,255,.2),inset 0 0 0 1px rgba(170,188,255,.14)` | `inset 0 1px 0 rgba(255,255,255,.2),inset 0 0 0 1px rgba(170,188,255,.14)` | `inset 0 0 0 2px #FFFFFF` | `--tile-rim` (:root); contrast: rule `.hc .tile,.hc .nuc` (box-shadow) |
| `--material-tile-sh` | `0 4px 10px rgba(0,0,0,.28)` | `0 4px 10px rgba(0,0,0,.28)` | `none` † | `--tile-sh` (:root) |
| `--material-tile-pc` | `#E6EBF5` | `#E6EBF5` | `#FFFFFF` † | `--tile-pc` (:root) |

**Elevation** (7)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--elev-window` | `22px 36px 64px rgba(0,0,0,.58),6px 12px 28px rgba(0,0,0,.42)` | `22px 36px 64px rgba(36,54,108,.22),6px 12px 28px rgba(36,54,108,.14)` | `none` † | `--wsh` |
| `--elev-window-inactive` | `14px 24px 48px rgba(0,0,0,.45)` | `14px 24px 48px rgba(36,54,108,.16)` | `none` † | `--wsh-off` |
| `--elev-orbit` | `0 18px 44px rgba(0,0,0,.55),0 2px 0 rgba(0,0,0,.3)` | `0 18px 44px rgba(36,54,108,.20),0 1px 0 rgba(255,255,255,.9) inset` | `none` | `--dock-sh` |
| `--elev-plate` | `0 18px 48px rgba(0,0,0,.28)` | `0 18px 48px rgba(36,54,108,.16)` † | `none` | rule `.plate-g` |
| `--elev-launcher` | `22px 36px 64px rgba(0,0,0,.6)` | `22px 36px 64px rgba(36,54,108,.22)` † | `none` † | rule `.launch .lbg` |
| `--elev-inspector` | `-14px 18px 40px rgba(0,0,0,.42)` | `-14px 18px 40px rgba(36,54,108,.16)` | `none` † | rule `.insp` |
| `--elev-trusted` | `0 34px 64px rgba(6,10,26,.42),0 10px 24px rgba(6,10,26,.30)` | `0 34px 64px rgba(6,10,26,.42),0 10px 24px rgba(6,10,26,.30)` | `none` † | rule `.kern` |

**Neutral-surface twins (S0 D3, PROVISIONAL)** (26)

| Token | Dark | Light | Contrast | Orbitaal v2 |
|---|---|---|---|---|
| `--color-neutral-surround` | `#090909` † | `#777777` † | `#000000` † | S0 D3 twin |
| `--color-neutral-surface-panel` | `#111111` † | `#F0F0F0` † | `#000` † | S0 D3 twin |
| `--color-neutral-surface` | `#171717` † | `#FCFCFC` † | `#000` † | S0 D3 twin |
| `--color-neutral-surface-raised` | `#212121` † | `#E8E8E8` † | `#000` † | S0 D3 twin |
| `--color-neutral-surface-inset` | `#0B0B0B` † | `#FFFFFF` † | `#000` † | S0 D3 twin |
| `--color-neutral-line` | `#2C2C2C` † | `#DEDEDE` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-line-strong` | `#858585` † | `#757575` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-text` | `#F1F1F1` † | `#171717` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-text-muted` | `#B2B2B2` † | `#4B4B4B` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-text-subtle` | `#A4A4A4` † | `#5B5B5B` † | `#F2F2F2` † | S0 D3 twin |
| `--color-neutral-selection` | `rgba(229,229,229,.085)` † | `rgba(24,24,24,.065)` † | `#333333` † | S0 D3 twin |
| `--color-neutral-hover` | `rgba(229,229,229,.07)` † | `rgba(24,24,24,.05)` † | `#1A1A1A` † | S0 D3 twin |
| `--color-neutral-hover-strong` | `rgba(229,229,229,.12)` † | `rgba(24,24,24,.09)` † | `#262626` † | S0 D3 twin |
| `--color-neutral-rim-a` | `rgba(233,233,233,.46)` † | `#FFFFFF` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-rim-b` | `rgba(167,167,167,.10)` † | `rgba(91,91,91,.16)` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-rim-c` | `rgba(167,167,167,.20)` † | `rgba(91,91,91,.30)` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-silver` | `#D2D2D2` † | `#666666` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-button-primary-bg` | `linear-gradient(180deg,#F6F6F6,#DFDFDF)` † | `linear-gradient(180deg,#2C2C2C,#181818)` † | `#FFFFFF` † | S0 D3 twin |
| `--color-neutral-button-primary-bg-hover` | `linear-gradient(180deg,#FFFFFF,#ECECEC)` † | `linear-gradient(180deg,#393939,#212121)` † | `#D9D9D9` † | S0 D3 twin |
| `--color-neutral-button-primary-text` | `#0E0E0E` † | `#FFFFFF` † | `#000000` † | S0 D3 twin |
| `--color-neutral-button-secondary-bg` | `rgba(229,229,229,.08)` † | `#FFFFFF` † | `#000000` † | S0 D3 twin |
| `--color-neutral-pill-bg` | `rgba(229,229,229,.06)` † | `#FFFFFF` † | `#000000` † | S0 D3 twin |
| `--elev-neutral-window` | `22px 36px 64px rgba(0,0,0,.58),6px 12px 28px rgba(0,0,0,.42)` † | `22px 36px 64px rgba(57,57,57,.22),6px 12px 28px rgba(57,57,57,.14)` † | `none` † | S0 D3 twin |
| `--elev-neutral-window-inactive` | `14px 24px 48px rgba(0,0,0,.45)` † | `14px 24px 48px rgba(57,57,57,.16)` † | `none` † | S0 D3 twin |
| `--color-neutral-focus` | `#F1F1F1` † | `#171717` † | `#FFE34D` † | S0 D3 twin |
| `--color-neutral-blade` | `#F1F1F1` † | `#171717` † | `#FFFFFF` † | S0 D3 twin |
<!-- END token-tables -->

`--color-surface-surround` is defined in every theme block (equal to
`--color-bg`; v2 has no surround) and re-pointed to `--color-neutral-surround`
under `data-surface="neutral"` (S0 D3; repair-1). Where the contrast value comes
from a v2 `.hc` rule rather than the dark source's variable, the Orbitaal v2
column names that rule ("contrast: rule …"); "derived from" names the v2
variable a † value departs from. The name changes `--dock-bg` → 
`--material-orbit-bg` and `--dock-sh` → `--elev-orbit` follow S0 D4 (avoid
"dock").

### 3.3 Type, space, radius and motion (theme-independent, `:root`)

| Token | Value | Source |
|---|---|---|
| `--font-family-ui` | `"Inter", system-ui, sans-serif` | TYPOGRAPHY.md §8.1 |
| `--font-features-ui` | `"cv05", "cv08", "ss01"` | §8.1 |
| `--font-numeric-digits` | `tabular-nums` | §8.1, §8.4 |
| `--font-weight-regular` / `-strong` | 400 / 600 | §8.2 |
| `--type-caption-*` | 13 / 18 / 400 | §8.2 |
| `--type-body-*` | 15 / 22 / 400 | §8.2 |
| `--type-body-dense-*` | 15 / 20 / 400 | §8.2 ("20 in dense rows"); S0 D2 |
| `--type-body-strong-*` | 15 / 22 / 600 | §8.2 |
| `--type-reading-*` | 16 / 26 / 400 | §8.2 |
| `--type-title-*` | 22 / 28 / 600 | §8.2 |
| `--type-display-*` | 48 / 56 / 400 | §8.2 |

| Token | Value | v2 source (tokens.css comment) |
|---|---|---|
| `--space-1` … `--space-6` | 4, 8, 12, 16, 20, 24 px | v2 `gap` and `padding` values |
| `--space-8`, `--space-10` | 32, 40 px | v2 `padding 24px 32px`, `.cap padding 0 40px` |
| `--space-target-min` | 24 px (compact 24) | S0 D2 |
| `--space-control-height` | 32 px (compact 24) | S0 D2, PROVISIONAL |
| `--space-control-inset` | 12 px (compact 8) | S0 D2, PROVISIONAL |
| `--space-gap` | 8 px (compact 4) | S0 D2, PROVISIONAL |
| `--space-panel` | 16 px (compact 8) | S0 D2, PROVISIONAL |

| Radius token | px | v2 rule | Radius token | px | v2 rule |
|---|---|---|---|---|---|
| `--radius-mark` | 2 | blades, bars | `--radius-tile` | 12 | `.tile` 40 px, `.kw`, `.tgl` |
| `--radius-keycap` | 6 | Enter keycap | `--radius-control-large` | 13 | `.btn.lg` |
| `--radius-tile-small` | 8 | `.tile.s28` | `--radius-card` | 14 | `.ob`, `.insp`, `.card` |
| `--radius-item` | 9 | `.tile.s32`, `.nb`, `.sr`, `.ni` | `--radius-window` | 16 | `.win`, `.plate` |
| `--radius-row` | 10 | `.row`, `.wc button` | `--radius-trusted` | 20 | `.kern` |
| `--radius-control` | 11 | `.btn`, `.inp` | `--radius-orbit` | 22 | `.orbit`, `.appic` |
| `--radius-pill` | 999px | `.pill` | `--radius-launcher` | 24 | `.launch .lbg` |
| `--radius-round` | 50% | nodes, dots, rings | | | |

Motion tokens are listed in §12. v2's off-grid spacings (2, 3, 5, 6, 9, 10,
11, 14, 18 px) and radii (1, 4, 7 px) are not tokenised; components snap to
the grid (`tokens.css` comments).

## 4. Themes and contrast pairs

Method: `build/design-evidence/ds-1.0/repair-1/tools/contrast_pairs.py` (copy of the documentation stage's `doc/contrast_pairs.py`), run with
`python3 -I` on `tokens.css` (WCAG 2.2 relative luminance, ratios floored to
two decimals; translucent tokens composited in 8-bit sRGB over the named base;
gradients judged at every stop, worst stop reported). Output:
`build/design-evidence/ds-1.0/repair-1/tools/contrast-pairs.md`. Thresholds: text 4.5:1
(7:1 in the contrast theme), UI components and focus 3:1, disabled text 3:1
(TYPOGRAPHY.md §8.3). `--color-line` is listed for information only: ds-1.0
uses it for dividers and for the edges of surfaces (window, menu, dialog,
toast, place badge, info banner) that are identified by fill, icon or text
rather than by that edge. The final base verifier measured these edges from
computed styles at 1440 px: 1.27:1 dark (`#232A47` on `#12162A`), 1.31:1 light
(`#D8DEE9` on `#FBFCFE`), place-badge edges 1.03:1; each of these components
is identified by its fill, icon or text instead (place badge: node shape,
icon and label), so the edges are not WCAG 1.4.11 boundaries. The new grant
row edge measures 1.27:1 dark and 1.32:1 light (identified by its text and
Intrekken button) and 21:1 in contrast (UX-D-C1 harness).

**Backdrop assumption.** Glass, Orbit material and the translucent launcher are
composited over `--color-bg`. Their contrast over the real wallpaper is not
known (§19).

<!-- BEGIN contrast-tables -->
#### dark — at rest

| Foreground | Need | bg `#070912` | panel `#0D1020` | surface `#12162A` | raised `#1A1F38` | inset `#080A16` |
|---|---|---|---|---|---|---|
| `--color-text` `#EEF1F8` | 4.5 | 17.57 | 16.70 | 15.82 | 14.31 | 17.43 |
| `--color-text-muted` `#A9B2CB` | 4.5 | 9.38 | 8.91 | 8.44 | 7.64 | 9.30 |
| `--color-text-subtle` `#9BA2BC` | 4.5 | 7.84 | 7.45 | 7.06 | 6.39 | 7.77 |
| `--color-text-disabled` `#9BA2BC` | 3.0 | 7.84 | 7.45 | 7.06 | 6.39 | 7.77 |
| `--color-line-strong` `#7882A8` | 3.0 | 5.25 | 4.99 | 4.73 | 4.28 | 5.21 |
| `--color-line` `#232A47` | info | 1.41 | 1.34 | 1.27 | 1.15 | 1.40 |
| `--color-focus` `#EEF1F8` | 3.0 | 17.57 | 16.70 | 15.82 | 14.31 | 17.43 |
| `--color-blade` `#E6EBF5` | 3.0 | 16.62 | 15.79 | 14.96 | 13.54 | 16.48 |
| `--place-device` `#42D27F` | 3.0 | 10.17 | 9.67 | 9.16 | 8.29 | 10.09 |
| `--place-nas` `#5AA2FF` | 3.0 | 7.61 | 7.23 | 6.85 | 6.20 | 7.55 |
| `--place-internet` `#FF5C50` | 3.0 | 6.52 | 6.20 | 5.87 | 5.31 | 6.47 |

#### light — at rest

| Foreground | Need | bg `#E4E9F2` | panel `#EDF0F7` | surface `#FBFCFE` | raised `#E3E8F1` | inset `#FFFFFF` |
|---|---|---|---|---|---|---|
| `--color-text` `#111726` | 4.5 | 14.67 | 15.67 | 17.41 | 14.54 | 17.88 |
| `--color-text-muted` `#424B62` | 4.5 | 7.13 | 7.62 | 8.47 | 7.07 | 8.69 |
| `--color-text-subtle` `#515A73` | 4.5 | 5.63 | 6.01 | 6.68 | 5.58 | 6.86 |
| `--color-text-disabled` `#515A73` | 3.0 | 5.63 | 6.01 | 6.68 | 5.58 | 6.86 |
| `--color-line-strong` `#6B7590` | 3.0 | 3.76 | 4.02 | 4.47 | 3.73 | 4.59 |
| `--color-line` `#D8DEE9` | info | 1.10 | 1.18 | 1.31 | 1.09 | 1.35 |
| `--color-focus` `#111726` | 3.0 | 14.67 | 15.67 | 17.41 | 14.54 | 17.88 |
| `--color-blade` `#2B3245` | 3.0 | 10.47 | 11.19 | 12.43 | 10.38 | 12.76 |
| `--place-device` `#127A43` | 3.0 | 4.43 | 4.73 | 5.25 | 4.38 | 5.39 |
| `--place-nas` `#1061D6` | 3.0 | 4.65 | 4.97 | 5.52 | 4.61 | 5.67 |
| `--place-internet` `#CC3328` | 3.0 | 4.23 | 4.52 | 5.02 | 4.19 | 5.15 |

#### contrast — at rest

| Foreground | Need | bg `#000000` | panel `#000000` | surface `#000000` | raised `#000000` | inset `#000000` |
|---|---|---|---|---|---|---|
| `--color-text` `#FFFFFF` | 7.0 | 21.00 | 21.00 | 21.00 | 21.00 | 21.00 |
| `--color-text-muted` `#FFFFFF` | 7.0 | 21.00 | 21.00 | 21.00 | 21.00 | 21.00 |
| `--color-text-subtle` `#F2F2F2` | 7.0 | 18.75 | 18.75 | 18.75 | 18.75 | 18.75 |
| `--color-text-disabled` `#F2F2F2` | 3.0 | 18.75 | 18.75 | 18.75 | 18.75 | 18.75 |
| `--color-line-strong` `#FFFFFF` | 3.0 | 21.00 | 21.00 | 21.00 | 21.00 | 21.00 |
| `--color-line` `#FFFFFF` | info | 21.00 | 21.00 | 21.00 | 21.00 | 21.00 |
| `--color-focus` `#FFE34D` | 3.0 | 16.34 | 16.34 | 16.34 | 16.34 | 16.34 |
| `--color-blade` `#FFFFFF` | 3.0 | 21.00 | 21.00 | 21.00 | 21.00 | 21.00 |
| `--place-device` `#7CF0A8` | 3.0 | 14.88 | 14.88 | 14.88 | 14.88 | 14.88 |
| `--place-nas` `#8EC5FF` | 3.0 | 11.58 | 11.58 | 11.58 | 11.58 | 11.58 |
| `--place-internet` `#FF8A80` | 3.0 | 9.19 | 9.19 | 9.19 | 9.19 | 9.19 |

#### Worst case under hover, strong hover and selection fills (every base)

| Foreground | Need | dark | light | contrast |
|---|---|---|---|---|
| `--color-text` | 4.5 / 7 | 10.37 (raised+hover-strong `#31374F`) | 12.13 (raised+hover-strong `#D0D5DE`) | 12.63 (bg+selection `#333333`) |
| `--color-text-muted` | 4.5 / 7 | 5.54 (raised+hover-strong `#31374F`) | 5.90 (raised+hover-strong `#D0D5DE`) | 12.63 (bg+selection `#333333`) |
| `--color-text-subtle` | 4.5 / 7 | 4.63 (raised+hover-strong `#31374F`) | 4.65 (raised+hover-strong `#D0D5DE`) | 11.28 (bg+selection `#333333`) |
| `--color-text-disabled` | 3.0 | 4.63 (raised+hover-strong `#31374F`) | 4.65 (raised+hover-strong `#D0D5DE`) | 11.28 (bg+selection `#333333`) |
| `--color-line-strong` | 3.0 | 3.10 (raised+hover-strong `#31374F`) | 3.11 (raised+hover-strong `#D0D5DE`) | 12.63 (bg+selection `#333333`) |
| `--color-focus` | 3.0 | 10.37 (raised+hover-strong `#31374F`) | 12.13 (raised+hover-strong `#D0D5DE`) | 9.83 (bg+selection `#333333`) |
| `--color-blade` | 3.0 | 9.81 (raised+hover-strong `#31374F`) | 8.66 (raised+hover-strong `#D0D5DE`) | 12.63 (bg+selection `#333333`) |
| `--place-device` | 3.0 | 6.00 (raised+hover-strong `#31374F`) | 3.66 (raised+hover-strong `#D0D5DE`) | 8.95 (bg+selection `#333333`) |
| `--place-nas` | 3.0 | 4.49 (raised+hover-strong `#31374F`) | 3.84 (raised+hover-strong `#D0D5DE`) | 6.96 (bg+selection `#333333`) |
| `--place-internet` | 3.0 | 3.85 (raised+hover-strong `#31374F`) | 3.50 (raised+hover-strong `#D0D5DE`) | 5.53 (bg+selection `#333333`) |

#### Component and material pairs (translucent materials composited over `--color-bg`)

| Pair | Need | dark | light | contrast |
|---|---|---|---|---|
| Primary button text on its fill | 4.5 / 7 | 14.45 | 14.04 | 21.00 |
| Primary button text on hover fill | 4.5 / 7 | 16.34 | 11.62 | 14.87 |
| Text on secondary button over raised | 4.5 / 7 | 11.63 | 17.88 | 21.00 |
| Secondary button edge over raised | 3.0 | 3.48 | 4.59 | 21.00 |
| Secondary button edge over surface | 3.0 | 3.91 | 4.59 | 21.00 |
| Input edge on inset | 3.0 | 5.21 | 4.59 | 21.00 |
| Tooltip / badge text (inset on text fill) | 4.5 / 7 | 17.43 | 17.88 | 21.00 |
| Kern text on Kern material | 4.5 / 7 | 16.99 | 15.78 | 21.00 |
| Kern secondary text on Kern material | 4.5 / 7 | 10.11 | 7.67 | 21.00 |
| Kern text on internet notice | 4.5 / 7 | 15.58 | 15.68 | 21.00 |
| Internet notice icon on notice | 3.0 | 6.93 | 4.52 | 9.19 |
| Tile glyph on tile (every stop) | 3.0 | 8.95 | 8.95 | 21.00 |
| Text on launcher, opaque fallback | 4.5 / 7 | 16.23 | 16.53 | 21.00 |
| Text on launcher, translucent over bg | 4.5 / 7 | 15.82 | 15.92 | 21.00 |
| Text on inspector (every stop) | 4.5 / 7 | 14.60 | 17.88 | 21.00 |
| Muted text on inspector | 4.5 / 7 | 7.79 | 8.69 | 21.00 |
| Text on glass over bg | 4.5 / 7 | 17.18 | 16.36 | 21.00 |
| Muted text on glass over bg | 4.5 / 7 | 9.17 | 7.95 | 21.00 |
| Text on Orbit material over bg | 4.5 / 7 | 14.45 | 15.92 | 21.00 |

#### MF1: Kern material against the page window it covers (UI, 3:1)

| Page theme | Kern theme | Kern bg | Window | Ratio |
|---|---|---|---|---|
| dark | light | `#EEF1F5` | `#12162A` | 15.78 |
| light | dark | `#0B0E1B` | `#FBFCFE` | 18.71 |
| contrast | contrast | `#000000` | `#000000` | 1.00 (edge carried by 2 px white `--material-kern-rim`) |

#### Neutral surface (`data-surface="neutral"`, PROVISIONAL) — worst case at rest over panel, surface, raised, inset; state and button fills: §4.1 (0 failures)

| Foreground | Need | dark | light | contrast |
|---|---|---|---|---|
| `--color-text` | 4.5 / 7 | 14.25 (raised) | 14.63 (raised) | 21.00 (inset) |
| `--color-text-muted` | 4.5 / 7 | 7.59 (raised) | 7.11 (raised) | 21.00 (inset) |
| `--color-text-subtle` | 4.5 / 7 | 6.45 (raised) | 5.54 (raised) | 18.75 (inset) |
| `--color-text-disabled` | 3.0 | 6.45 (raised) | 5.54 (raised) | 18.75 (inset) |
| `--color-line-strong` | 3.0 | 4.36 (raised) | 3.76 (raised) | 21.00 (inset) |
| `--color-focus` | 3.0 | 14.25 (raised) | 14.63 (raised) | 16.34 (inset) |
| `--color-blade` | 3.0 | 14.25 (raised) | 14.63 (raised) | 21.00 (inset) |
| `--place-device` | 3.0 | 8.24 (raised) | 4.40 (raised) | 14.88 (inset) |
| `--place-nas` | 3.0 | 6.17 (raised) | 4.62 (raised) | 11.58 (inset) |
| `--place-internet` | 3.0 | 5.28 (raised) | 4.20 (raised) | 9.19 (inset) |
<!-- END contrast-tables -->

#### Axis combinations (ds-1.1)

The tier-2 colour axes (§21.2) re-point existing names in each theme:
`data-contrast="increased"` (text and lines, dark and light),
`data-transparency="opaque"` (glass, Orbit, launcher), `data-colour-perception`
(three place colours and the Kern warning colours) and `data-stimulus` (blade
glow and the window, Orbit, plate, launcher and inspector elevation);
`data-light-intensity="dimmed"` multiplies the rendered sRGB channels by 0.85
(about 30 % less luminance) and, per theme, strengthens `--color-text-subtle`,
`--color-text-disabled`, `--color-line-strong` (and in dark
`--color-text-muted` and `--place-internet`), because the filter alone breaks
the floors on hover and selection stacks (repair 1). The increased-contrast and
colour-perception values hold at × 0.85 themselves.
`docs/verification/a11y-ds-20261115/check.mjs` evaluates every pair of this
section on every surface alone and under `--color-hover`,
`--color-hover-strong`, `--color-selection` and `--color-button-secondary-bg`
(place colours also on `--color-pill-bg` over those), composited and quantised
to 8 bits, at × 1 and × 0.85, for every combination of the five colour axes in
the three themes, on the default and the neutral surface (`data-surface`), plus
the device-preference cases at both light intensities (588 cases, 187 768
pair evaluations; text 4.5:1, 7:1 under increased contrast
and in the contrast theme; lines, focus, blade and places 3:1): 0 failures.
Tightest margins as `check.mjs` prints them (3:1 required, all dimmed, the
dark cases on the neutral surface): dark tritan `--place-nas` on raised +
hover-strong + pill 3.01:1, dark `--place-internet` on the same stack 3.02:1,
dark `--color-line-strong` on raised + hover-strong 3.03:1 and contrast protan
`--place-internet` on bg + selection 3.03:1. In rendered pixels the run-1
verifier measured the first stack at 3.44:1 and the line-strong case at 3.03:1. Record: `docs/verification/a11y-ds-20261115/README.md`.

### 4.1 S0 D3 enumeration and the repair-1 token change

The enumeration in the S0 D3 form (bases panel, surface, raised, inset; at rest
and under hover, strong hover, selection, secondary-button and pill fills; text
4.5:1, `line-strong`, focus and place 3:1) failed 29 plain and 17 neutral pairs
on the tokens stage's values, all `--color-line-strong` or `--color-text-subtle`.
One failure was at rest: dark `--color-line-strong` `#5D6890` on
`--color-surface-raised` `#1A1F38`, 2.96:1, the edge of inputs, progress bars and
secondary and disabled buttons. Lowering state alphas cannot clear an at-rest
failure, so repair round 1 did the other task S0 D3 assigns to this stage: it
raised `--color-line-strong` and `--color-text-subtle` (with
`--color-text-disabled`, which equals subtle) in dark and light, and their
neutral twins. State alphas keep their v2 values.

| Token | Theme | Before (v2 or S0 twin) | ds-1.0 after repair-1 | Worst D3 pair after |
|---|---|---|---|---|
| `--color-line-strong` | dark | `#5D6890` (`--ln2`) | `#7882A8` | 3.10 (raised + hover-strong `#31374F`) |
| `--color-line-strong` | light | `#78819A` (`--ln2`) | `#6B7590` | 3.11 (raised + hover-strong `#D0D5DE`) |
| `--color-text-subtle`, `--color-text-disabled` | dark | `#8C95B2` (`--tx3`) | `#9BA2BC` | 4.63 (raised + hover-strong) |
| `--color-text-subtle`, `--color-text-disabled` | light | `#58627D` (`--tx3`) | `#515A73` | 4.65 (raised + hover-strong) |
| `--color-neutral-line-strong` | dark / light | `#747474` / `#818181` | `#858585` / `#757575` | 3.12 / 3.13 |
| `--color-neutral-text-subtle` | dark / light | `#969696` / `#626262` | `#A4A4A4` / `#5B5B5B` | 4.63 / 4.62 |

Method (`build/design-evidence/ds-1.0/repair-1/tools/tune.py`): HSL lightness in steps of 0.25 %, hue and
saturation kept, smallest change that brings every D3 pair to at least 3.10 or
4.60 after flooring (0.10 above the threshold). A twin is the achromatic grey of
the same WCAG luminance as the new plain value, moved one 8-bit step where it
still fell short. The contrast theme is unchanged.

| Set | Dark | Light | Contrast |
|---|---|---|---|
| Plain values | 0 (was 20) | 0 (was 9) | 0 |
| Neutral twins (PROVISIONAL) | 0 (was 10) | 0 (was 7) | 0 |

<!-- BEGIN failure-list -->

<!-- END failure-list -->

The tables above and these counts are regenerated from the edited `tokens.css`
(`build/design-evidence/ds-1.0/repair-1/tools/contrast-pairs.md`); `build/design-evidence/ds-1.0/repair-1/tools/check_tokens.py` reports 0 hard
failures and 0 D3 open items (`build/design-evidence/ds-1.0/repair-1/check-report.txt`). Consequences:

- These values depart from the reviewed v2 board and are marked † in §3.2. This is a ds-1.0 design decision, not a revision of v2.
- The dark secondary-button edge is now 3.48:1 over raised and 3.91:1 over a
  window (O-C2 closed).
- Hierarchy: muted and subtle text are closer than in v2 (1.20:1 dark, was
  1.41:1; 1.27:1 light, was 1.43:1). Judgement: acceptable, because subtle text
  is metadata and never the only cue; review is open (O-C3).
- The neutral twin values stay PROVISIONAL (S0 D3).
- S0 D3 asks for the run under `docs/verification/`. This round could not write
  there, so the run is only in ignored evidence and the D3 check is not yet
  stored as accepted (O-C1).

## 5. Materials

| Material | Tokens | Use | Not for |
|---|---|---|---|
| Kern (trusted surface) | `--material-kern-bg/-tx/-tx2/-line/-rim`, `--material-kern-warn-*`, `--radius-trusted`, `--elev-trusted` | Permission prompts and other shell questions only. Opaque. Takes the **opposite** theme by setting `data-theme` on the Kern element (MF1); the contrast theme keeps contrast with a 2 px white rim. | Any app surface; any translucent treatment |
| Scrim | `--color-scrim` | Behind a modal dialog (`::backdrop`, `.ds-scrim`) and behind a trusted question (CONCEPTS.md, adopted grafts). Opaque `#000000` in contrast. | Decoration |
| Glass | `--material-glass`, `--material-glass-line` | Shell plates over the wallpaper only (v2 `.plate-g`). | Window interiors, text-heavy panels, anything over moving content |
| Orbit | `--material-orbit-bg`, `--elev-orbit`, `--radius-orbit` | The Orbit capsule; `.ds-orbit` adds `backdrop-filter: blur(24px)`, removed in contrast. | — |
| Launcher | `--material-launch`, `--material-launch-opaque`, `--elev-launcher`, `--radius-launcher` | Launcher panel; `--material-launch-opaque` is the required fallback over windows (MF11), because natively blur is feasible only over the static pre-blurred wallpaper. | — |
| Well | `--color-well` | Recessed areas inside a surface (v2 `--well`); in ds-1.0 only the privacy instrument's track. | Primary content |
| Inspector | `--material-insp-bg`, `--elev-inspector` | The inspector tethered to a selected row (MF8). | — |
| App tile | `--material-tile-bg/-fg/-rim/-sh/-pc`, `--radius-tile*` | Every app icon, first- or third-party, identical in both themes (MF9). Used by `.ds-ask-app`. | System controls (bare glyphs) |

Glass, launcher and inspector materials, `--elev-plate`, `--elev-launcher` and
`--elev-inspector` are tokens only: no ds-1.0 component uses them yet.

**Rule against indiscriminate glass.** Translucency is limited to the three
shell materials that sit on the wallpaper (glass plate, Orbit, launcher). Every
translucent material has an opaque value in the contrast theme, and the
launcher has an opaque fallback. Window bodies, trusted surfaces, menus,
dialogs and toasts are opaque. Contrast of translucent materials depends on
the backdrop and is unverified over real wallpaper pixels (§19).

## 6. Typography

ds-1.0 implements the `type-1.0` roles (deviations: `font-display: swap`
and the digits rule below); the specification is
[TYPOGRAPHY.md](TYPOGRAPHY.md) §8 and is not restated here.

- Roles and tokens: `--type-{caption,body,body-dense,body-strong,reading,title,display}-{size,line,weight}` (§3.3). Classes `.ds-t-caption`, `.ds-t-body`, `.ds-t-body-dense`, `.ds-t-body-strong`, `.ds-t-reading`, `.ds-t-title` and `.ds-t-display` apply them.
- Inter 4.001 is loaded by `@font-face` from `ui/design/fonts/Inter[opsz,wght].ttf` with `font-display: swap`; features `cv05 cv08 ss01` always on.
- Weights 400 and 600 only; tracking 0; no other sizes or line heights. Weight 600 at 15 px is always `body-strong` 15/22/600: buttons, the selected tab, the selected row name, lit privacy-instrument labels and the instrument's place line. The 20 px line is used only at weight 400 (`body-dense`, S0 D2); a selected compact row keeps its 24 px height with the 22 px line. Unlit instrument labels use `body` 15/22/400 so lit and unlit labels share one line box.
- Sweep: the gallery's own check (`gallery.js`, `?measure=1`) compares size, line height and weight together since repair round 1; the earlier check compared size and weight only and missed 15/20/600 on `.ds-btn`, `.ds-tab`, `.ds-row-name`, `.ds-privacy-lbl` and the instrument's place line. Repair-1 run: 1 314 visible text elements (200 % strip excluded), none outside the seven roles (`build/design-evidence/ds-1.0/repair-1/measure.json`).
- Tabular figures: `--font-numeric-digits` is set by four selectors: `.ds-num` on numeral spans (times, sizes, dates, percentages), and directly on `.ds-row-meta` (file size), `.ds-taskitem-badge` (count) and `.ds-toast-time` (time), three elements that hold only a short value; it is never set on a line of prose; the progress percentage is a `.ds-num` span and `.ds-progress-label` sets no figure style. TYPOGRAPHY.md §8.1 and §8.4 ask for tabular digits everywhere because the native atlas cannot select by context; the browser and native behaviours therefore differ for digits in prose (open item O-T1).
- Disabled text uses `--color-text-disabled` (= `--color-text-subtle`), at least 3:1 (§4), and is always paired with a non-colour cue (dashed edge, reason line).

## 7. Layout

| Aspect | ds-1.0 value | Source |
|---|---|---|
| Grid | 4 px; every `--space-*` is a multiple of 4 | `tokens.css`; tokens stage check 3d |
| Minimum target | 24 × 24 CSS px for every hit area (`--space-target-min`) | S0 D2; WCAG 2.2 SC 2.5.8 |
| Control height | 32 px standard, 24 px compact (PROVISIONAL); v2 used 36 px | S0 D2; v2 `.btn`, `.inp` |
| Large button | 44 px (`data-size="large"`, Kern and dialogs) | v2 `.btn.lg` |
| Window title bar | 52 px (`.ds-window-bar`) | v2 `.fm` grid: first row 52 px, holding the full-width `.bar` |
| Window controls | 40 × 32 px each (`.ds-window-ctl`) | v2 `.wc button` |
| Orbit | 64 px high, 48 × 48 items, 22 px from the bottom edge in v2 | v2 `.orbit`, `.ob` |
| Files window first column (sidebar) | 248 px | v2 `.fm` (`grid-template-columns: 248px 1fr`) |
| v2 artboard | 1440 × 900 | v2 `.ab` |
| v2 window examples | 916 × 640, 960 × 690, 1240 × 724 | v2 `.win` inline styles |
| v2 launcher | 860 × 608 | v2 `.launch` inline style |
| Permission prompt | 600 px (`.ds-ask`, UX-C run 2); v2 shows 600 px | `components.css`; v2 `.kern` inline style |
| Menu / toast / dialog | min 300 px / 440 px / 440 px | `components.css` |

Radii: §3.3. Elevation: `--elev-*` in §3.2; every elevation is `none` in the
contrast theme, where edges carry separation.

## 8. Iconography

| Rule | Value | Source |
|---|---|---|
| Grid | 24 units, `viewBox="0 0 24 24"` | `icons.js` header; `ui/design/prototype/contract.js` `icon()` |
| Stroke | 1.6, round caps and joins, `fill: none`, `stroke: currentColor` (48 px empty-state icons: 1.2) | `components.css` `.icon`, `.ds-empty > .icon` |
| Particle | One solid particle at a joint in app icons, `class="pc"`, `fill="currentColor"`; place colour only through `--ds-c` in `.ds-place`, lit or asking privacy columns and `.ds-ask-row` | `icons.js` header; `components.css` |
| Naming | v2 sprite ids without `i-`; app ids, file kinds and places from `ui/design/prototype/data.js`; fallback `dot` | `icons.js` |
| Markup | No inline style, no `var()`, no colour literals | tokens stage check 2 |

Icon keys (54; the gallery's `?measure=1` reports 54 keys, all used, none
unresolved, `c1/repair-2/gallery-measure.json`):

| Group | Keys |
|---|---|
| ds-1.0 addition, not in v2 (1) | `altijd` (UX-D-C1 repair-2): a closed loop with the particle on the top of its right lobe (r 2.1, as on `internet`'s edge; on the crossing it merged with the strokes at 16 px), drawn in the v2 stroke style for the "Altijd, tot je het intrekt" grant chip, so `lock` stays reserved for the Kern band and a revocable grant does not read as "locked". Kept in a separate block after the generated object in `icons.js`. That the loop reads as "until revoked" is AI design judgement, untested (O-G13) |
| v2 sprite (42) | `zoeken` `bestanden` `notities` `instellingen` `haven` `device` `nas` `versies` `toegang` `internet` `power` `lock` `assistent` `min` `max` `close` `back` `fwd` `plus` `check` `recent` `desk` `doc` `image` `download` `trash` `warn` `draw` `list` `pdf` `loc` `nofiles` `window` `chev` `coll` `huis` `archief` `web` `klok` `apps` `user` `a11y` |
| Drawn for ds-1.0 (4) | `rekenmachine`, `muziek`, `sheet`, `dot` (outlined diamond; a solid dot would read as "device") |
| Aliases (7) | `browser` = `web`, `fotos` = `image`, `note` = `doc`, `drawing` = `draw`, `desktop` = `desk`, `documents` = `doc`, `downloads` = `download` |

**Tiles versus glyphs (CONCEPTS.md MF9).** Every app icon, first- or
third-party, sits on the one app tile (`--material-tile-*`), identical in both
themes; system controls (menu, search, window controls) are bare glyphs. In
ds-1.0 only `.ds-ask-app` draws the tile; `.ds-taskitem` currently shows app
icons as bare glyphs, unlike v2's Orbit (open item O-K3).

## 9. Window chrome and desktop components

Implemented in `components.css` and shown in the gallery:

| Element | Class | Behaviour and states | DESKTOP-SHELL §2 |
|---|---|---|---|
| Window | `.ds-window`, `.ds-window-bar`, `.ds-window-title`, `.ds-window-body` | `data-state="active"` (top light blade, `--elev-window`), `inactive` (subtle title and controls, `--elev-window-inactive`), `maximised` (no radius, edge or shadow) | Title bar with minimise, maximise, close |
| Window controls | `.ds-window-ctls`, `.ds-window-ctl` | Right-hand order Minimaliseren, Maximaliseren (Vorige grootte when maximised), Sluiten; hover, pressed, inner focus ring; Sluiten carries a tooltip "Sluiten" with the keycap "Alt+F4" | Buttons on the right |
| Orbit | `.ds-orbit`, `.ds-orbit-sep` | Capsule; separator before third-party apps (CONCEPTS.md grafts); accessible name "Orbit, dock" (NAMING-AND-IA.md §4, S0 D4) | Orbit (dock) |
| Orbit item | `.ds-taskitem`, `.ds-taskitem-badge` | `data-state` `running` (short bar), `active` (blade on Orbit's top edge + bar, `aria-current`), `attention` (wide bar + numbered keycap badge; the count is in the accessible name) | Pinned and open apps with indicator |

Not in ds-1.0 (tokens exist for some): launcher (`--material-launch*`),
search results, the Alt+Tab switcher, system tray and clock, notification
centre, quick settings, desktop icons, window move, resize and snap handles.
These are open item O-K1.

Names: NAMING-AND-IA.md §1 keeps DESKTOP-SHELL.md §7 (Horizon, Orbit, Zenit,
Nebula, Signaal, Kern, Haven) binding until the project owner decides P-NAMES-1
(§8), while the design system on this branch uses the proposed names (Menu,
Zoeken, Bestanden, Instellingen). ds-1.0 token names use "Kern" for the v2
trusted-surface material; in DESKTOP-SHELL.md §7 "Kern" is the Settings app
(open item O-N1).

## 10. Component states

States are real pseudo-classes and ARIA attributes. `data-demo="hover|pressed|focus"`
forces a state on inert gallery specimens only. "—" means the component has no
such state in `components.css`; missing states that a product use would need
are listed in O-S1. Keyboard behaviour for listbox, tabs, menu, dialogs and the
tooltip is implemented by `ui/design/system/gallery.js` (simulation), not by
`components.css`.

**Focus ring (all interactive components).** `outline: 2px solid var(--color-focus)`,
offset `2px` outside for `.ds-btn`, `.ds-input`, `.ds-switch`, checkbox and
radio inputs and `.ds-tabpanel`; offset `-2px` inside for `.ds-row`,
`.ds-menu-item`, `.ds-tab`, `.ds-window-ctl` and `.ds-taskitem`. No ring on
pointer focus (`:focus:not(:focus-visible)`). In the contrast theme the ring is
`#FFE34D`, 1.28:1 against white, so two rules keep a 1 px black or `#333333`
gap between the ring and white marks: `.ds-tab` takes offset `-5px` (clear of
the 3 px selected underline and the white tab-list line), and a focused row's
blade moves to `left: 3px`, inside the ring (§13, O-K2).

**Hidden.** The base rule `[hidden] { display: none !important; }` in
`components.css` keeps the HTML `hidden` attribute effective on every `.ds-`
class that sets `display`. Before repair round 2 only `.ds-menu`,
`.ds-tabpanel` and `.ds-toast` had a guard, so the hidden "Melding opnieuw
tonen" `.ds-btn` rendered and took a Tab stop in each theme panel.

**Anatomy (part classes).** Every `.ds-` class in `components.css` is listed
here; the state tables below use the root names.

| Component | Root | Parts |
|---|---|---|
| Type role | — | `.ds-t-caption`, `.ds-t-body`, `.ds-t-body-dense`, `.ds-t-body-strong`, `.ds-t-reading`, `.ds-t-title`, `.ds-t-display` (§6) |
| Utilities | — | `.ds-num` (tabular digits on a numeral span), `.ds-muted` (`--color-text-muted`), `.ds-vh` (visually hidden), `.ds-status-marker` (square error marker) |
| Button | `.ds-btn` | `.ds-spinner` (busy) |
| Field | `.ds-field` | `.ds-field-label`, `.ds-input`, `.ds-input-wrap` (positions the leading search icon), `.ds-field-hint`, `.ds-field-error` |
| Switch | `.ds-switch` | `.ds-switch-track` (the 44 × 24 track; decorative, `aria-hidden`) |
| Checkbox, radio | `.ds-check`, `.ds-radio` | `.ds-fieldset` (group with legend) |
| Segmented choice (ds-1.1 repair 2) | `.ds-seg` (in a `.ds-fieldset`) | `.ds-radio` tiles with an optional static sample |
| List | `.ds-list` | `.ds-list-head` (caption column header, `aria-hidden`), `.ds-row`, `.ds-row-name`, `.ds-row-meta` (size, tabular), `.ds-row-date` |
| Place | `.ds-place` | `.ds-node` (shape: dot, ring with dot, open ring) |
| Privacy instrument | `.ds-privacy` | `.ds-privacy-track`, `.ds-privacy-col`, `.ds-privacy-lbl`, `.ds-privacy-text` |
| Permission prompt | `.ds-ask` | `.ds-ask-band`, `.ds-ask-main`, `.ds-ask-body`, `.ds-ask-proto`, `.ds-ask-title`, `.ds-ask-from`, `.ds-ask-app`, `.ds-ask-path`, `.ds-ask-stop`, `.ds-ask-link`, `.ds-ask-held`, `.ds-ask-holds`, `.ds-ask-facts`, `.ds-ask-again`, `.ds-ask-reason`, `.ds-ask-explain`, `.ds-ask-keybox`, `.ds-ask-more`, `.ds-ask-outcomes`, `.ds-ask-decide`, `.ds-ask-lifetime`, `.ds-ask-remember`, `.ds-ask-state`, `.ds-ask-choices`, `.ds-ask-confirm`, `.ds-ask-note`, `.ds-key` (UX-C run 2) |
| Window | `.ds-window` | `.ds-window-bar`, `.ds-window-title`, `.ds-window-ctls`, `.ds-window-ctl`, `.ds-window-body` |
| Orbit | `.ds-orbit` | `.ds-taskitem`, `.ds-taskitem-badge`, `.ds-orbit-sep` |
| Menu | `.ds-menu-anchor`, `.ds-menu` | `.ds-menu-item`, `.ds-menu-sep`, `.ds-menu-sub`, `.ds-menu-why` (reason line of an unavailable item), `.ds-chev-down` (trigger chevron) |
| Tabs | `.ds-tabs` | `.ds-tablist`, `.ds-tab`, `.ds-tabpanel` |
| Toast | `.ds-toast` | `.ds-toast-title`, `.ds-toast-body`, `.ds-toast-time`, `.ds-toast-actions` |
| Banner | `.ds-banner` | — |
| Dialog | `.ds-dialog`, `.ds-scrim` | `.ds-dialog-body`, `.ds-dialog-title`, `.ds-dialog-text`, `.ds-dialog-actions` |
| Progress | `.ds-progress-field` | `.ds-progress-label`, `.ds-progress`, `.ds-progress-bar` |
| Empty, loading, error | `.ds-empty`, `.ds-skeleton` | `.ds-empty-orbit`, `.ds-empty-sat`, `.ds-empty-title`, `.ds-empty-place`, `.ds-empty-hint`, `.ds-empty-actions` (orbit, satellite and place: ds-1.1); `.ds-skeleton-row`, `.ds-skeleton-line` |
| Tooltip | `.ds-tip-anchor` | `.ds-tip` |
| Keyboard key | `.ds-kbd` | `.ds-kbd-combo` |

**Density and neutral surface.** `data-density="compact"` and
`data-surface="neutral"` (S0 D2, D3; values PROVISIONAL) change tokens; the only
component rule is `[data-density="compact"] .ds-place` (badge 20 px high). The gallery shows both per theme: compact buttons,
field, switch, check, radio, tabs and rows render 24 px high (targets at least
24 × 24; window controls keep 40 × 32), and the neutral panel renders a field,
secondary buttons (rest, hover, disabled), rows (selected, hover, rest) and a
progress bar on a raised surface, with a focusable canvas on
`--color-surface-surround`. Inside the neutral host every computed colour is
achromatic except place colours (135 elements, 0 tinted; `build/design-evidence/ds-1.0/repair-1/measure.json`).

### Button `.ds-btn` (`.ds-spinner`)

| State | Treatment |
|---|---|
| Default | `data-variant` `primary` (light fill in dark, dark fill in light), `secondary` (`--color-button-secondary-bg` + 1 px `--color-line-strong`), `quiet` or none (transparent), `danger` (2 px `--color-danger` edge and text, trash icon; no red, S0 D1). `data-size="large"` 44 px; `data-shape="icon"` square. |
| Hover | Primary: `--color-button-primary-bg-hover`; others: `--color-hover-strong` |
| Pressed (`:active`, `aria-pressed="true"`) | 1 px down; non-primary: `--color-selection` fill and `--color-text-muted` edge; primary: inset 1 px edge |
| Focus-visible | Outer ring |
| Disabled (`:disabled`, `aria-disabled="true"`) | Transparent, 1 px dashed `--color-line-strong`, `--color-text-disabled`; hover and pressed suppressed; no opacity |
| Loading (`aria-busy="true"`) | Spinner arc shown, pointer events off; static arc under reduced motion |
| Narrow (ds-1.1 run-2 repair 2) | A label wider than its row wraps between words, balanced and centred, in a button that grows (`min-height: max(--space-control-height, --space-target-min)` instead of a fixed `height`; `data-size="large"` likewise); one line keeps the control height, and a single word wider than the line breaks inside the word. The `.ds-ask` actions keep one line (task UX-C) |
| Selected, error, empty | — |

### Text and search field `.ds-field`, `.ds-input`

| State | Treatment |
|---|---|
| Default | `--color-surface-inset` fill, 1 px `--color-line-strong`; label `.ds-field-label`, hint `.ds-field-hint`; placeholder `--color-text-subtle` |
| Hover | Edge `--color-text-muted` |
| Focus-visible | Outer ring |
| Error (`aria-invalid="true"`) | 2 px `--color-danger` edge; `.ds-field-error` with square marker, warn icon and message |
| Disabled | Dashed edge, transparent, `--color-text-disabled` |
| Search variant | `.ds-field[data-variant="search"]`: pill radius, leading search icon |
| Pressed, selected, loading, empty | — |

### Switch `.ds-switch` (`role="switch"`)

| State | Treatment |
|---|---|
| Off / on (`aria-checked`) | Outlined track with knob left / filled `--color-text` track, knob right (shape and position, not colour) |
| Hover | 4 px `--color-hover-strong` halo |
| Focus-visible | Outer ring |
| Disabled | Dashed `--color-line-strong` track, `--color-text-disabled` label |
| Pressed, error, loading | — |

### Checkbox and radio `.ds-check`, `.ds-radio`, `.ds-fieldset`

| State | Treatment |
|---|---|
| Unchecked | 20 px, 2 px `--color-text-muted` edge; checkbox rounded square (`--radius-keycap`, distinct from the error square), radio open ring |
| Checked (`:checked`) | Checkbox: filled with tick (`::after`); radio: filled disc with knocked-out centre (distinct from the NAS ring-with-dot) |
| Hover | 4 px halo |
| Focus-visible | Outer ring on the input |
| Disabled | Dashed edge, `--color-text-disabled` label |
| Segmented choice (ds-1.1 repair 2) | `<div class="ds-seg">` inside a `.ds-fieldset`: the same native radios (one Tab stop, arrow keys), each a transparent cover over its tile, so the whole tile is the hit area (at least `max(--space-control-height, --space-target-min)`); an optional static sample (`aria-hidden`) above the label; selected: a 2 px `--color-line-strong` inset edge, a `--color-blade` underline and body-strong text, so the state does not rest on colour; hover `--color-hover`; focus-visible: the 2 px `--color-focus` ring on the tile (`:has(> input:focus-visible)`); tiles wrap (`auto-fit`, at least 104 px) and grow with their text; under `forced-colors: active` the underline becomes a 2 px border in the system text colour (emulated in headless Chromium only) |
| Indeterminate, pressed, error | — |

### List row `.ds-list`, `.ds-row` (`role="option"` in a listbox)

| State | Treatment |
|---|---|
| Default | Icon, name, place badge, size and date (`.ds-num` tabular) |
| Hover | `--color-hover` |
| Selected (`aria-selected="true"`) | `--color-selection` fill, name weight 600, tapered `--color-blade` grows from the row centre (`--motion-duration-selection`); contrast adds a 2 px white inset edge |
| Focus-visible | Inner ring |
| Single pointer (D9) | A plain click selects one row |
| Pressed, disabled, error, loading | — |

### Place badge `.ds-place`, node `.ds-node`

| State | Treatment |
|---|---|
| `data-place` `device` / `nas` / `internet` | Node `data-shape` dot / ring with dot / open ring, icon with place-coloured particle, text (gallery: "Dit apparaat", "Thuis-NAS", "Internet") |
| Node `data-state="off"` | Outline only, `--color-text-subtle` |
| Node `data-state="ask"` | 2 px ring plus a pulsing halo (`--motion-duration-halo`); static in contrast and under reduced motion |
| Contrast | Device dot gets a black and white halo |
| Not focusable (static label) | |

### Privacy instrument `.ds-privacy` (content and rules: §11)

| State | Treatment |
|---|---|
| Root `data-state="local"` / `"internet"` | No root rule; expressed by the columns: only Dit apparaat lit / all three lit with lit connector blades |
| Narrow container or wider text (ds-1.1 repair 2) | The frame wraps (`flex-wrap`, `min-height: 48px`, `max-width: 100%`): the text block moves below the track and keeps its own 48 px row; column labels wrap between words. The 3 × 120 px track itself does not reflow below about 400 px (O-A4) |
| Root `data-state="ask"` | Edge `--place-internet-line`; internet column `ask` with node halo; 2 px edge in contrast. The edge is a supplementary cue: at 50 % alpha it reaches 2.11:1 to 2.35:1 against the adjacent fills in dark and light (below 3:1; contrast theme `#FF8A80` on black, 9.20:1). The text "Webvenster vraagt internet", the `role="status"` label and the haloed internet node are the required indicators; the edge must never be the only cue. |
| Column `data-state="lit"` / `"ask"` / `"off"` | Label weight 600 and `--color-text`, place-coloured particle / same plus the asking node / outline node in `--color-text-subtle` |
| Hover, pressed, focus, disabled, error | — (not focusable, `role="status"`) |

### Permission prompt `.ds-ask` (content and rules: §11.1; UX-C run 2, replaces the C1 three-choice prompt)

| State | Treatment |
|---|---|
| Shown | Kern material in the opposite theme, rim via `padding-box`/`border-box` layers, `--elev-trusted`, 600 px wide (`max-width: 100%`); rises once over `--motion-duration-ask` (240 ms; none under reduced motion or in contrast). No other motion: the path is static and nothing moves toward either answer before a decision |
| Anatomy | `.ds-ask-band` (lock + "Vraag van BaryonOS · getoond boven alle apps (gesimuleerd)"), then `.ds-ask-main` = scrolling `.ds-ask-body` + fixed `.ds-ask-decide`. Body: `.ds-ask-proto` prototype line, `.ds-ask-title` question, `.ds-ask-from` app tile + provenance, `.ds-ask-facts` (at most three shell facts, each led by an `aria-hidden` `.ds-ask-fact-icon` naming its consequence class; the text stays the carrier), optional `.ds-ask-reason`, `.ds-ask-path` (a vertical route: `.ds-ask-stop` pills stacked top to bottom, joined by downward `.ds-ask-link` chevrons in the gap; `[data-held]` dashed = data the app already holds; `[data-place]` edge in the place colour), `.ds-ask-holds`, `.ds-ask-explain` ("Vergelijk:" + `<details>` "Waarom niet ‘…’?"), `<details class="ds-ask-keybox">` with the five-field `.ds-key`. Decide row: `.ds-ask-lifetime` (native checkbox `.ds-check.ds-ask-remember` "Keuze onthouden" + `.ds-ask-state[data-on]`), `.ds-ask-outcomes` (one sentence per answer), `.ds-ask-choices` |
| Choices | Two `.ds-btn` `data-variant="secondary"` `data-size="large"`, equal width in a two-column grid, fixed order Toestaan then Weigeren; no primary, no preselection. Each choice's `aria-describedby` is its outcome sentence |
| Remember | `<input type="checkbox">` inside a `<label>`: unchecked at every opening, Space toggles it natively, checked state exposed natively; `aria-describedby` = the state sentence, which the gallery rewrites on `change` together with both outcome sentences. Dashed state chip + `klok` = until the app closes; solid 2 px chip + `altijd` = remembered; 600 weight in the contrast theme |
| Title | `tabindex="-1"`, initial focus (set as `autofocus` before `showModal()`); 2 px focus ring; Enter or Space on it does nothing. `.ds-ask` is an inline-size container (`ask`): when the Kern is at least 560 px wide and the viewport at least 40rem tall, the question takes the display role (48/56, `text-wrap: balance`), so the one question dominates as in concept b; a narrower Kern, a shorter viewport (200 % of a 1440 x 900 window) and `[data-scale="2"]` keep the title role (repair-1, `run2-repair-1/out/repair.json`) |
| Modal | One `alertdialog`: the `<dialog>` carries `role="alertdialog"`, `aria-labelledby` (question) and `aria-describedby` (facts); the inner `.ds-ask` gets `role="none"`. `showModal()` makes the background inert. Escape (`cancel`) and closing without a choice cancel fail-closed (§11.1) |
| Outcomes (repair-1) | `.ds-ask-outcomes` sits in `.ds-ask-decide` between the lifetime row and the choices, so each answer's lifetime and suppression is read next to the answer (MF-1); in a viewport of at most 40rem or with `[data-scale="2"]` the live prompt moves it to the end of the scrolling body (O-G16) |
| Fit (UX-C run 2) | The `<dialog>` keeps the UA `max-height` and `overflow: visible`; `.ds-ask-main` is a flex column in which only `.ds-ask-body` scrolls (`overflow-y: auto`, `overscroll-behavior: contain`); `.ds-ask-decide` never scrolls, so the checkbox, the state sentence and both answers stay in view and a focused control is never covered (measured at 200 % in a 720 x 450 CSS-px viewport, with both tier-2 combinations and at 320 CSS px; build/design-evidence/uxc-run2/run2-writer/matrix-summary.txt) |
| Confirm step `[data-step="confirm"]` | Only for a combined document-and-internet right with Toestaan + remember: `.ds-ask-confirm` replaces `.ds-ask-main` in the same dialog, focus on its heading; "Terug naar de vraag" and "Onthouden en toestaan" are equal secondary buttons |
| Compact `[data-compact]` | MF-5 repeat form: question, provenance, "eerder in deze sessie toegestaan", the facts and the decide row stay; path, explanation and key move into one `<details class="ds-ask-more">` "Alle details" |
| App reason `.ds-ask-reason` | Dashed frame after the facts: "<App> zegt:" + the app's plain text (inserted with `textContent`) + "Tekst van de app, niet van BaryonOS" |
| Quiet note `.ds-ask-note` | `role="status"`, dashed edge, never takes focus, no timer: temporary denial with the retry count and "Zelf opnieuw kiezen voor <App>", covered requests, cancellations, a prompt that could not be shown. A static, inert covered-request specimen (`[data-ask-covered]`) shows the acknowledgement for the seeded remembered grant: no prompt, only the note |
| One at a time | `contract.js` presents one prompt at a time: a distinct request that arrives while a prompt is open waits until that prompt is answered or cancelled; it never cancels the open prompt (OQ-4). Identical requests share the open prompt |

### App permissions `.ds-grants`, `.ds-grant`, `.ds-key` (content and rules: §11.1)

| State | Treatment |
|---|---|
| Rest | One row per remembered decision, Toestaan and Weigeren alike (`data-decision="allow"` or `"deny"`): app tile, name "<App> · <what>", "Precies:" + the five-field `.ds-key` chips (the same markup as the prompt's key), reach as `.ds-place` badges, decision chip `.ds-grant-until[data-decision]` (solid 2 px for allow, double 3 px for deny, `altijd` icon, text "Toegestaan, onthouden" / "Geweigerd, onthouden"). Flex row that wraps, so the actions drop below at narrow widths |
| Change | Secondary `.ds-btn` "Wijzigen naar Toestaan" or "Wijzigen naar Weigeren"; absent where the model refuses the change (an irreversible external send or a combined right cannot become Toestaan outside the prompt) |
| Forget | Secondary `.ds-btn` "Vergeten"; accessible names list all five key fields; removal and change are announced in `.ds-live`; focus stays in the list |
| Usage | No usage metadata: the page keeps no usage log, so none is shown (never invented) |
| Empty | `.ds-grant-empty`: "Geen onthouden keuzes. Apps vragen het je wanneer ze iets nodig hebben." |
| Contrast | 2 px white edge |

### Window `.ds-window`, `.ds-window-ctl`

| State | Treatment |
|---|---|
| `data-state="active"` / `"maximised"` | Top light blade (`::before`), `--elev-window` / no radius, edge or shadow |
| `data-state="inactive"` | Title, controls and title icon in `--color-text-subtle`; `--elev-window-inactive` |
| Control hover / pressed | `--color-hover-strong` and `--color-text` / `--color-selection` |
| Control focus-visible | Inner ring |
| Control disabled | — |

### Orbit item `.ds-taskitem`

| State | Treatment |
|---|---|
| `data-state="running"` / `"active"` / `"attention"` | 12 × 3 px bar in `--color-text-muted` / bar plus 36 px blade above the item / 24 px bar in `--color-text` plus `.ds-taskitem-badge` |
| Hover / pressed | `--color-hover-strong` / `--color-selection` |
| Focus-visible | Inner ring |
| Disabled | — |

### Menu `.ds-menu-anchor`, `.ds-menu`, `.ds-menu-item`

| State | Treatment |
|---|---|
| Trigger | `aria-haspopup="menu"`, `aria-expanded`; `.ds-chev-down` |
| Open | Rises 12 px over `--motion-duration-base`; none under reduced motion or contrast; `hidden` when closed |
| Item hover / focus-visible | `--color-hover-strong` + inner ring |
| Item disabled (`aria-disabled="true"`) | `--color-text-disabled`, no hover fill, visible reason line `.ds-menu-why` |
| Separator, shortcut, submenu marker | `.ds-menu-sep`; `.ds-kbd` hints; `.ds-menu-sub` chevron as a static visual specimen. The item carries no `aria-haspopup` or `aria-expanded`, because no submenu opens; ArrowRight moves to the next item. Submenus are not specified (O-S4). |
| Pressed, checked items | — |

### Tabs `.ds-tabs`, `.ds-tablist`, `.ds-tab`, `.ds-tabpanel`

| State | Treatment |
|---|---|
| Default / hover | `--color-text-muted` / `--color-hover` fill and `--color-text` |
| Selected (`aria-selected="true"`) | Weight 600 + 2 px `--color-blade` underline (3 px in contrast) |
| Focus-visible | Inner ring on the tab; outer ring on the panel |
| Pressed, disabled | — |
| Narrow box, 200 % or wider text spacing (ds-1.1 run 2) | The tab list wraps (`flex-wrap: wrap`) onto a next row instead of widening the page; `.ds-tabs` has one `minmax(0, 1fr)` column, so the panel keeps the box width. The selected tab keeps its blade underline, focus rings stay inset (−2 px; −5 px in contrast) and no overflow container clips them; arrow keys, Home and End are unchanged (`build/design-evidence/a11y-ds/run2-writer/tabs.json`) |

### Toast `.ds-toast`

| State | Treatment |
|---|---|
| Shown (`role="status"`) | Icon, title, body, time, actions ("Ongedaan maken" quiet, close icon); rises over `--motion-duration-base` |
| Hidden | `hidden`; the gallery offers "Melding opnieuw tonen" and moves focus to it |
| Narrow (ds-1.1 repair 2) | The actions wrap; in a toast of about 270 px the close button moves to its own line. Since run-2 repair 2 a button that is still wider than its line wraps its label; repair 2's `flex-shrink: 0` (keep the label width) was removed, because at 160 CSS px (`data-scale="2"` on 320 px) with wider spacing it pushed "Ongedaan maken" 13 px of the unzoomed viewport (6.5 CSS px at zoom 2) past the actions row; 0 since (`run2-repair-2/lbl2-before-none.json`, `lbl2-now-none.json`) |

### Banner `.ds-banner`

| `data-tone` | Treatment |
|---|---|
| `info` | `--color-line` edge, `dot` icon, word "Info." |
| `success` | Base edge (`--color-line-strong`), `check` icon, word "Gelukt." (its rule repeats the base edge style) |
| `warning` | No dedicated rule: base edge, `warn` icon, word "Let op." |
| `error` | 2 px `--color-danger` edge, square marker, `warn` icon, word "Fout." |

### Dialog `.ds-dialog`, `.ds-scrim`

| State | Treatment |
|---|---|
| Open (native `<dialog>` `showModal()`) | `::backdrop` `--color-scrim`; focus starts on the safest action (Annuleren); rises over `--motion-duration-ask`; a held Enter or Space opens it once and never closes it again through Annuleren (§15, held key) |
| Static specimen | `.ds-scrim` over an inert stage |
| Narrow (ds-1.1 repair 2) | A plain dialog is a size container; at most 320 px wide it stacks its actions at full width and a long label wraps inside its button (the `data-variant="ask"` shell is not a container) |
| `data-variant="ask"` | Transparent shell hosting a `.ds-ask`; the body scrolls inside the viewport under a fixed band (Permission prompt, Modal fit) |

### Progress `.ds-progress-field`, `.ds-progress`, `.ds-progress-bar`

| State | Treatment |
|---|---|
| Determinate | `role="progressbar"`, `aria-valuenow`, width from `--ds-value`, tabular percentage |
| Leading edge (ds-1.1) | The fill (`--color-text-muted`) ends in a 3 px light blade (`--color-blade`, glow `--color-blade-glow`), the active-window signature; the label carries the place node |
| Indeterminate (`data-state="indeterminate"`) | A tapered blade segment sweeps three times (3 × `--motion-duration-loader` = 4.2 s) and rests in the centre; the visible word "Bezig" carries the state. Under reduced motion the segment rests in the centre from the start. No stripes (§21.1 B7) |

### Empty, loading and error `.ds-empty`, `.ds-skeleton`

| State | Treatment |
|---|---|
| Frame (ds-1.1) | `.ds-empty` is a grid of a 72 px orbit (`.ds-empty-orbit`: 2 px ring in `--color-line-strong` around the place icon, the place node on `.ds-empty-sat`, drawn 1.34 times its size and trailed by a tapered 3 px, 110° tail in its place colour (run-2 repair 1), `.ds-empty-sat::before`, so the node is the dominant mark; the arc travels with the node) and text (`.ds-empty-title`, `.ds-empty-place` with the place badge, `.ds-empty-hint` up to 60 characters, `.ds-empty-actions`), crowned by a light blade |
| Empty | Title names the collection; `.ds-empty-place` says where it lives; hint and one action |
| Loading (`data-state="busy"`; no `aria-busy`, so the status region is not suppressed) | Visible title with `role="status"` ("Bestanden laden…") and its source place; the node orbits three times (4.2 s) and rests at the top; a caption "Nog bezig · m:ss" (`.ds-empty-tick`, `data-elapsed`, every 5 s, not live) keeps it visibly alive afterwards; `.ds-skeleton` rows (solid `--color-hover-strong` lines, place node per row, `aria-hidden`); in contrast a solid 1 px `--color-line-strong` outline. No shimmer |
| Error (`data-tone="error"`, `role="alert"`) | 2 px `--color-danger` edge, no crown, a 2 px ring broken once at the top (no dashed texture, row 21) with the hollow node (`data-state="off"`) in the gap, square marker in the title, where the work is safe, retry (primary) and a quiet action |
| Narrow (container at most 320 px) | The orbit stacks above the text; place badges and buttons wrap between words with `height: auto`, a `min-height` and 4 px block padding (a word breaks only if it is wider than the line); skeleton rows take the stacked `.ds-row` form |

### Tooltip `.ds-tip-anchor`, `.ds-tip` (`role="tooltip"`)

| State | Treatment |
|---|---|
| Hidden / shown | Shown on `:hover` and `:focus-within` of the anchor; inverted (`--color-text` fill, `--color-surface-inset` text); no fade in contrast |
| Dismissed | Escape sets `data-dismissed` until focus or pointer leaves |
| Static | `data-state="open"` |
| Edge (`data-align="start"` or `"end"`, ds-1.1 run-2 repair 1) | A tip on the first or last control of a bar aligns with that control's outer edge (`left: 0` or `right: 0`, no centring transform). The gallery's Versies tip uses `start` and the Sluiten tip of the window bar `end`; centred, the Versies tip started at −31.4 CSS px at 1440 px and at −94.6 CSS px at 320 px with wider text spacing (ds-1.0 defect, run-2 verifier) |
| Width | `max-content` up to 15rem, then the text wraps between words; a keycap does not break. The tip that is asked for (`:hover`, `:focus-within`) paints above a static one (`z-index` 31 over 30) |

### Keyboard key `.ds-kbd`, `.ds-kbd-combo`

Caption 13/18, 24 px high, keycap radius, 1 px `--color-text-subtle` inset
edge, `--color-text-muted` text. Not focusable.

## 11. Trust indicators

| Indicator | Class | Content | Behaviour |
|---|---|---|---|
| Place badge | `.ds-place` | Node shape + icon + storage label (NAMING-AND-IA.md §6) | Static; inside trust surfaces and in file rows |
| Privacy instrument | `.ds-privacy` (`.ds-privacy-track`, `-col`, `-lbl`, `-text`) | Three columns Dit apparaat, Thuis-NAS, Internet; each column `data-state` `lit`, `ask` or `off`; connectors are light blades between lit columns | `role="status"` with a full sentence `aria-label`. Root `data-state`: `local`, `ask` (internet-place edge `--place-internet-line`, a supplementary cue under 3:1 in dark and light, §10; halo on the internet node) and `internet` (all lit). Only `ask` has a root rule; `internet` is derived, v2 has no such state. |
| Permission prompt | `.ds-ask` (`-band`, `-main`, `-body`, `-proto`, `-title`, `-from`, `-path`, `-stop`, `-facts`, `-reason`, `-explain`, `-keybox`, `-outcomes`, `-decide`, `-lifetime`, `-state`, `-choices`, `-confirm`), `.ds-key` | Band "Vraag van BaryonOS · getoond boven alle apps (gesimuleerd)"; prototype line; one plain question; provenance; up to three shell facts; path; lifetime state sentence; one outcome sentence per answer | Native `<dialog>` via `showModal()`, `role="alertdialog"`; Toestaan and Weigeren plus the native checkbox "Keuze onthouden" (§11.1); initial focus on the question; Escape cancels fail-closed; the result is reported in a quiet `role="status"` note |
| App permissions | `.ds-grants`, `.ds-grant` (`-app`, `-name`, `-key`, `-reach`, `-until`, `-actions`, `-empty`), `.ds-key`, `.ds-place`, `.ds-live` | One row per remembered decision (allow or deny): "Precies:" with the five key fields, reach, decision chip; "Wijzigen naar …", "Vergeten" | Changes and removals announced in a polite `role="status"` region (§11.1) |

Rules and their sources:

- **Drawn only by the shell.** DESKTOP-SHELL.md §1: permission dialogs are drawn by the trusted shell in a layer that apps cannot draw over, and §4 asks for permission dialogs stating who, what and for how long. DESKTOP-SHELL.md §3 lists the protected system layer as **missing** natively, so this is a requirement, not a property of any build. In the browser the `.ds-` classes are available to any page; nothing in ds-1.0 enforces exclusivity.
- **Opposite theme and opacity.** Kern takes the opposite theme (MF1) and is always opaque; a scrim dims apps while a trusted question is open (CONCEPTS.md grafts).
- **Not proof of authenticity.** An app can imitate pixels; passwords, PINs and administrator passwords are entered only on the session-service surface (DESKTOP-SHELL.md §1; BRAND.md §5).
- **Separate, visible, revocable permissions.** LOCAL-PRIVACY-CONTRACT.md §4: installation grants nothing; LAN and internet are granted separately; combined permissions are displayed; an app allowed to read a file and contact a server may send that content. The prompt's internet notice states exactly this boundary.
- **Necessary decision versus explanation.** LOCAL-PRIVACY-CONTRACT.md §4.1: explanations may be turned off; "Privacyuitleg uit" never means "alles toestaan", and a valid grant need not be asked again for each normal repetition.
- **Location honesty.** LOCAL-PRIVACY-CONTRACT.md §1: an unreachable storage location is visibly paused and files are not forwarded elsewhere; the gallery's warning banner and error empty state use the NAMING-AND-IA.md §6 sentence, which expresses this behaviour without stating the pause and the non-forwarding explicitly (O-N3).

The Kern band shows the `lock` icon where BRAND.md §5 places the compact mark
(O-B1).

### 11.1 Permission model (owner decisions of 15 and 18-19 November 2026, task UX-C run 2)

**Status.** Design and browser prototype only (`ui/design/prototype/contract.js`,
the gallery). Remembered decisions, temporary grants, temporary denials, their
storage, change and revocation are **not enforced by BaryonOS**; every prompt
and specimen carries the line "Prototype: onthouden keuzes worden nog niet door
BaryonOS afgedwongen." Native persistence needs an accepted ADR first (draft:
`docs/verification/uxc-run2-20261119/PROPOSED-ADR.md`). The Kern surface simulates
the shell-only protected layer, which is missing natively (DESKTOP-SHELL.md §3);
the band claim "getoond boven alle apps" is marked "(gesimuleerd)". Nothing here
changes native code, `docs/implementation/**` or `docs/LOCAL-PRIVACY-CONTRACT.md`.

**Superseded.** The 14 November model of task UX-D-C1 (three equal choices
"Toestaan tot afsluiten" / "Eenmalig toestaan", "Altijd toestaan", "Weigeren";
a four-field key) is superseded. Its verification record
`docs/verification/ds-1.0-c1-20261115/` is not evidence for this model, and its
contract test no longer matches `contract.js` (open item).

**Sources.** Owner decision of 15 November 2026 (O-G12, O-G4, O-G2, items 1-8),
approved readings R2 and R3, owner decisions of 18 and 19 November 2026
(OQ-1..OQ-6, MF-1..MF-6; verbatim in the coordinator checkpoint
`owner-decision-uxc-continuation-20261118.md`), concept b "Eén vraag" of UX-C
run 1 with grafts from concepts a and c.

**The model.** Two answers, Toestaan and Weigeren, plus one native checkbox
"Keuze onthouden" that applies to both and is unchecked at every opening (R3).

| Answer | Remember off | Remember on |
|---|---|---|
| Toestaan, ongoing right | Temporary grant until the logical app session ends (O-G12, OQ-5) | Remembered allow for the exact five-field key |
| Toestaan, single action | This action only; nothing stored (O-G4) | Remembered allow, except for an irreversible external send (OQ-1) |
| Weigeren | Temporary denial in volatile state until the app closes; automatic identical re-requests are refused without a prompt; a new deliberate gesture may ask again (R2) | Remembered deny for the exact key |
| Escape or closing | Cancel: the same fail-closed outcome as a temporary denial; nothing granted, nothing stored, even with the checkbox checked (owner decision 1) | same |

- **Key (OQ-4, MF-3).** Every decision is keyed on exactly app, right, data,
  destination and scope. Data is one exact item, never a class or pattern
  (`contract.js` rejects `*` and `?`). Changing any one field is a new question.
  No per-app or per-right anti-nag throttling exists, so a distinct legitimate
  request is never denied silently. Camera and microphone are separate rights.
- **No repeat prompt (owner item 6).** A remembered decision and a running
  temporary grant answer without a prompt. Identical concurrent requests share
  one prompt and store one decision. Distinct concurrent requests are shown one
  after another; a new request never cancels or silently denies an open one.
- **Prompt that cannot be shown.** When the shell cannot draw a prompt (for
  example a right without copy, or a render error), the request is denied for
  that request only (`via: 'error'`): nothing is granted or stored, no
  temporary denial is recorded and no R2 suppression follows, because the user
  never saw a question. Every right in `RIGHTS` (`read`, `readWrite`,
  `save-copy`, `connect`, `send-copy`, `read-connect`, `camera`, `microphone`,
  `location`) has question, verb phrase, facts and comparison copy.
- **OQ-1.** For an irreversible single-action external send, remembered Toestaan
  is not offered: with the checkbox checked the state sentence says that only a
  refusal can be remembered and Toestaan sends this once; the model never stores
  it. A remembered Weigeren is keyed to the exact request. For a combined
  document-and-internet right (`read-connect`), a remembered Toestaan needs a
  named destination, one exact item, a bounded scope (`document` or `item`) and
  the confirm step; otherwise only a temporary grant results. A remembered
  Toestaan answers a later request only when remembering Toestaan is offered for
  that request: an irreversible single-action external send is always asked,
  even when a Toestaan is remembered for the same five fields, and kind and
  irreversibility never become key fields. Because one decision is stored per
  five-field key, a remembered Weigeren given on that prompt replaces the earlier
  remembered Toestaan for the same five fields (access only narrows; owner
  decision A of 19 Nov 2026).
- **OQ-5 end events (prototype).** A temporary grant ends on app termination,
  logout or a user-session switch; hiding or minimising a window ends nothing;
  there is no timeout. Temporary denials end when the app closes.
- **OQ-6.** An optional app reason is shown only when it is a string of 1-120
  characters without control, format (bidirectional) or line-separator
  characters; otherwise it is not shown at all, never truncated. Markup
  characters are ordinary text: the gallery inserts the reason with
  `textContent`, so `<b>` appears literally. It sits after the BaryonOS facts,
  attributed ("<App> zegt:") and framed apart from system text.
- **Change and revocation (owner item 7).** Appmachtigingen lists remembered
  allows and remembered denials with "Precies:" and the five key fields; "Vergeten"
  removes exactly the matching key; "Wijzigen" obeys the same OQ-1 rules.
- **Per-request check.** A suppressed prompt never replaces the policy service's
  capability, object-scope and authorisation check on every request.

**Equal choice (MF-4).** Equal visual weight (same component, variant, size,
width and weight), clear wording, comparable pointer effort (adjacent equal
targets), no preselected answer, initial focus on a neutral element (the
question heading), Escape as a safe cancellation, and one fixed platform-wide
order: Toestaan, then Weigeren. The order has a keyboard asymmetry that is
stated, not hidden: Tab from the checkbox reaches Toestaan first, and Shift+Tab
from the heading reaches Weigeren first. An extra confirmation for
irreversible or combined cases is allowed and used only for the combined right.

**Lifetime in words (MF-1).** The state sentence always names the current
lifetime ("Niet onthouden: geldt tot je <App> sluit." for an ongoing right;
"Niet onthouden: Toestaan geldt alleen voor deze handeling, Weigeren tot je
<App> sluit." for a single action, so the sentence never claims that a
single-action Toestaan lasts until the app closes (O-G4, R2); "Onthouden voor
precies deze vraag, tot je het wijzigt in Appmachtigingen."), and one outcome
sentence per answer states its real lifetime and suppression; both outcomes
change when the checkbox toggles. A single-action Weigeren says that automatic
identical re-requests are refused until the app closes, never "alleen deze keer".

**Progressive disclosure and word budget (MF-6).** One question, up to three
essential shell facts, the explicit lifetime statement, both outcomes and the
everyday comparison line ("Vergelijk: ...", an explanation shown openly in the
first layer for transparency) are visible at decision time; only "Waarom niet
‘…’?" and the five-field key are in disclosures. Measured visible words at decision time (gallery live
prompt, explanations on, remember off, no app reason): read 136, single action
120, send 129, combined 133, camera with reason 142 (target about 120, review
above 156; `run2-repair-1/out/words2.log`; the single-action state sentence grew
by six words in repair-1). No material consequence was
removed to meet the budget; the read case stays above the target because its
three facts (later network risk, copies survive revocation, the NAS
administrator can delete but not read) are each material.

**Compact repeat prompt (MF-5).** Shown only when the request is identical in
all five key fields to one the user temporarily allowed earlier in the session,
with the same circumstance (`context`), and low risk: right `read` or
`save-copy`, no destination, not irreversible, not combined. External sends,
irreversible actions, any internet destination, camera, microphone, combined
rights and any changed permission or circumstance always get the full prompt.
The eligibility record is volatile prototype state, grants nothing and lasts
until logout or a session switch (a design decision: it must outlive the app
session, or an ongoing right could never reach the compact form). Any later
answer other than a temporary Toestaan for that key (a Weigeren, a cancellation
by Escape or closing, or a remembered decision), and any change or revocation
of a remembered decision for that key, deletes the record: the user's latest
answer is a changed circumstance, so the next identical request gets the full
prompt. The compact
form keeps the facts, the lifetime and both outcomes visible, puts the path,
explanation and key behind one "Alle details" disclosure and keeps the checkbox
unchecked.

**Three evidence classes for the owner decisions of 18-19 November.**

| Class | Items | Status |
|---|---|---|
| Approved UX behaviour, built as prototype | OQ-1, OQ-2 (Escape), OQ-4, OQ-6, MF-1..MF-6, OQ-5 end events | In `contract.js`, the gallery and `ui/design/prototype/tests/contract.test.mjs`; not enforced by BaryonOS |
| ADR changes awaiting native implementation | OQ-3 (suppression of identical unsolicited re-prompts across relaunches until logout, volatile, with native gesture attribution), OQ-5 lifecycle and revocation semantics (security-context revalidation) | Proposed in `PROPOSED-ADR.md`; R2 (until the app closes) stays the built behaviour; the gallery labels the until-logout behaviour "Voorgesteld, nog niet gebouwd" |
| Unverified research hypotheses | Comprehension of the question, path, facts and comparison; the ~120-word target; the compact form's effect on irritation and effort; detection of a "new deliberate user gesture" | [untested hypothesis] until the user study runs; gesture detection is "Toekomstig onderzoek" (the prototype treats a press of an opener as the gesture) |

**Feasibility labels.** The gallery shows each element beyond the baseline with
"Werkt nu in dit prototype", "Technisch haalbaar, nog niet gebouwd" or
"Toekomstig onderzoek" as visible text in the prompt card's status table; the simulation controls that act out app and
session events ("App vraagt automatisch opnieuw", "Venster verbergen", "App
sluiten", "Afmelden", "Andere gebruiker") carry a visible "Simulatie, geen
echte actie" note, a named group and their own status row.

**Strings.** All prompt copy lives once in `ui/design/prototype/strings.js`
(`trusted.*`); the gallery writes every specimen and the live prompt from one
renderer, so no specimen text can drift from the live prompt.

## 12. Motion

| Token | ms | Use in ds-1.0 | Board source |
|---|---|---|---|
| `--motion-duration-fast` | 140 | Hover fills, button press, input edge, window control and task item fills, tooltip fade | B1 menu close |
| `--motion-duration-selection` | 160 | Row blade grows from the centre; switch track and knob | B1 selection |
| `--motion-duration-base` | 220 | Menu and toast rise (12 px); determinate progress width | B1 menu open |
| `--motion-duration-ask` | 240 | Permission prompt and dialog rise, no overshoot | B1 system question |
| `--motion-duration-focus` | 260 | Not used by a component (focus blade sweep) | B1 |
| `--motion-duration-slow` | 320 | Active Orbit blade transform | B1 active blade |
| `--motion-duration-boot` | 1400 | Not used by a component (boot convergence) | B1 |
| `--motion-duration-loader` | 1400 | Spinner rotation (3 turns); indeterminate sweep and empty-state orbit (3 × 1400 = 4.2 s, once) | v2 `.ld` |
| `--motion-duration-halo` | 2400 | Internet node halo while asking (2 pulses, 4.8 s) | B1 |
| `--motion-duration-loader-pulse` | 2800 | Not used since ds-1.1 (the skeleton shimmer was removed) | v2 `.ld .bind` |
| `--motion-duration-stagger` | 80 | Not used by a component (boot blade stagger) | v2 `.bt-b2` |

Easing: `--motion-ease-orbital-out` `cubic-bezier(.16, .84, .24, 1)` for every
transition and rise; `--motion-ease-halo` `cubic-bezier(.2, .7, .2, 1)`;
`--motion-ease-linear` (spinner); `--motion-ease-in-out` (progress
sweep). Motion uses transform and opacity, plus background colour, border
colour and width.

**Reduced motion** (`prefers-reduced-motion: reduce` unless `data-motion="full"`,
or `[data-motion="reduce"]`): all 11 durations become 0ms (`tokens.css`); halo,
spinner, progress sweep, empty-state orbit and the rise of menu, toast, dialog
and prompt are removed; the indeterminate segment rests in the centre with the
word "Bezig"; the empty-state node rests at the top; the spinner stays a
static arc. `data-motion="full"` is the user's deliberate override of the
device preference and restores every duration. The gallery exposes a switch
for `data-motion` and the Beweging group in §21.5.

**Finite loops (ds-1.1, §21.1 B8).** No animation is infinite: spinner 3 ×
1.4 s, halo 2 × 2.4 s, indeterminate sweep and empty-state orbit one run of
4.2 s. Every loop ends within 5 s: this is the chosen way to meet WCAG 2.2.2
(pause, stop, hide) without a pause control. Auto-starting moving content
that lasts more than 5 s beside other content needs a pause, stop or hide
mechanism (WCAG 2.2.2); other safe animation is not restricted. No
period is shorter than 1.4 s, so nothing approaches three flashes per second
(WCAG 2.3.1, a separate flash rule). The ITU-R BT.1702-3 Guideline 1 remark
that a flashing sequence longer than 5 s might be a risk even when it complies
is an additional risk consideration for design review, not a hard limit
(ACCESSIBILITY-RESEARCH.md §3.5). The gallery `?measure=1` run sweeps every computed animation and
reports any that ends after 5 s or repeats faster than once per second
(35 animations, none flagged).

## 13. High contrast

`[data-theme="contrast"]` is a complete theme (opaque surfaces and fills; the decorative row, window, Orbit and privacy-connector blades keep their faded gradient ends over black, with a solid centre at 21:1; black surfaces, white
text and lines, focus `#FFE34D`). Values come from v2 `.hc` where v2 defines
them (v2 applies `class="dk hc"`); the 24 names `.hc` leaves undefined are
derived as opaque, achromatic values anchored on `HIGH_CONTRAST` in
`ui/graphics/src/theme.rs` (black `0x000000`, white `0xffffff`):

| Derived token | Value | Reason |
|---|---|---|
| `--color-selection` / `--color-hover` / `--color-hover-strong` | `#333333` / `#1A1A1A` / `#262626` | Opaque fills; selection is carried by a white edge and weight 600 because `#333333` is only 1.66:1 on black |
| `--color-scrim` | `#000000` | Trusted surfaces stand alone on black with a 2 px white edge |
| `--color-button-primary-bg` / hover / text | `#FFFFFF` / `#D9D9D9` / `#000000` | 21.00:1 and 14.87:1 |
| `--color-button-secondary-bg`, `--color-pill-bg` | `#000000` | Edge from the line token |
| `--color-silver`, Kern text, text-2, line, rim; tile fg and particle | `#FFFFFF` | Solid rim instead of the metal gradient |
| Kern bg, Kern warn bg, inspector, launcher, launch-opaque | `#000000` | Opaque |
| Kern warn line and icon, `--place-internet-line` | `#FF8A80` | v2 `.hc --red` (internet place) |
| `--material-tile-sh`, every `--elev-*` | `none` | No shadows |
| `--color-surface-surround` (repair-1) | `#000` | Equal to `--color-bg` (v2 `.hc{background:#000}`); v2 has no surround. Under `data-surface="neutral"` the surround and surfaces are all black, so the canvas region carries a 1 px `--color-line-strong` edge (S0 D3; gallery `.gal-surround`) |

Rejected anchors: native `accent_soft` cyan `0x00ffff` as selection (a
non-place hue next to NAS blue) and native muted `0xb8b8b8` (v2 `.hc` defines
`#FFFFFF` and `#F2F2F2`). Source: tokens stage decisions,
`build/design-evidence/ds-1.0/work/mapping.tsv`.

Component overrides in contrast (`components.css`): Orbit and privacy edges
2 px, Orbit blur off; selected row 2 px white inset edge; selected tab 3 px
underline; focused tab ring offset `-5px` and focused row blade at `left: 3px`
(repair round 2); device node black and white halo; ask halo static; no entrance
animation for menu, prompt, dialog and toast; no tooltip fade; skeleton lines
a solid 1 px `--color-line-strong` outline (dashed until ds-1.1); Kern edge 2 px.

Thresholds and results: text ≥ 7:1 (lowest 11.28:1, `#F2F2F2` on
`#333333`), UI ≥ 3:1 (lowest 5.53:1, `--place-internet` on `#333333`), no
translucent colour or gradient (§4; tokens stage check 3e). Constraint: focus
`#FFE34D` is 1.28:1 next to white and 1.09:1 next to `#D9D9D9`, so every ring
needs a black separator where it would touch a white fill; the outer ring's
2 px offset over the black base provides it. Inner rings next to white marks
were measured in repair round 2 with pixel profiles across the ring at device
scale 1 (`build/design-evidence/ds-1.0/repair-2/shots/ring-base-profile.txt`, `ring-final-profile.txt`,
`junction-base-final.json`; a junction pixel is a grey or white pixel that
touches a pure ring pixel at under 3:1; it does not count yellow-white blends
such as `#FFEF9C`). Before the §10 rules the selected-tab underline and the
tab-list line touched the ring (27 junction pixels, lowest 1.26:1) and the
white row blade covered the ring's left side. After them, the selected
standard tabs Versies, Toegang and Opslag and the compact tab Versies have 0
junction pixels and a 1 px black gap above the underline; the standard
selected row's left-edge profile reads ring, 1 px `#333333`, blade (the
junction count is 0 for that row before and after, so the profile is the
evidence). v2 defines no contrast-theme focus offset or blade position
(`.hc` rules of the board); both values are repair-2 derivations. Still open (O-K2): in compact rows the
20 px place badge's white edge touches the ring (258 pixels, lowest 1.06:1,
unchanged by the fix), the compact blade's rounded end blends with the ring
corner (2 pixels), a descender in a compact tab label (Toegang) touches the
inset ring (12 pixels), and the Orbit badge was not measured.

**Increased contrast and device mapping (ds-1.1, §21).** `data-contrast="increased"`
is a contrast level inside dark and light: text #FFFFFF / #05080F, muted and
subtle text at least 7:1, lines and rims at least 3:1, 2 px borders on
secondary buttons, inputs, place badges, banners, toasts, windows, empty states
and progress tracks. `prefers-contrast: more` applies the same values when no
explicit value is set. The contrast theme stays the maximum level; there increased
contrast only thickens those edges to 2 px, because its text and lines already
exceed the increased values. `forced-colors: active` keeps the place node shapes
(`forced-color-adjust: none` on `.ds-node`); everything else follows the user
agent palette (O-A5).

## 14. Scaling

| Scale | Browser | Native |
|---|---|---|
| 100 % | All sizes are logical CSS px | `type-1.0` records 13r, 15r, 15sb, 22sb, 48r (with `inter-atlas`) |
| 200 % | One mechanism shared by gallery, prototype and checks: an element (normally `<html>`) with `data-scale="2"` gets `[data-scale="2"] { zoom: 2 }` (`components.css`). The gallery renders a strip at 200 % (`#scale-host`) with buttons (primary, secondary, danger), a switch, a search field, one selected list row with a place badge, place badges, keycaps, a checkbox and, since repair round 2, the privacy instrument in the ask state. Menu, tabs, permission prompt, dialog, toast, window chrome and the Orbit are not in the strip (§19). | Doubled geometry and the 200 % atlas records 26r, 30r, 30sb, 44sb; display 48r magnified 2× (`ui/graphics/src/text.rs`; `docs/verification/native-inter-atlas-20261114/README.md`) |
| 125 %, 150 % | — | Open (TYPOGRAPHY.md §8.6) |

CSS `zoom` is not a device scale factor or a reflow. The components stage
checked one strip (computed zoom 2; a 32 px button measured 64 px tall).
Repair round 2 measured the strip with the instrument added: computed zoom 2,
83 rendered elements (SVG parts included), `#scale-host` scroll width equal to
its 680 px client width, no element beyond the strip's edges and no clipped
scroller (`build/design-evidence/ds-1.0/repair-2/flows.json`, `build/design-evidence/ds-1.0/repair-2/shots/scale-strip.png`).
UX-D-C1 added both permission prompts and the grant rows to the strip: stacked
choices 1018 x 88 px each, none clipped inside the prompt
(`build/design-evidence/ds-1.0/c1/writer/harness-result.json`). Those are the
inline specimens. The modal prompt (`#dlg-ask`, top layer) spilled out of the
viewport at 200 % (verify-1 step 10; reproduced in
`c1/repair-2/kbd-pre-fix.txt`: "Altijd toestaan" at 955–1043 and Weigeren at
1059–1147 in a 1000 px viewport). Since C1 repair-2 its body scrolls under a
fixed band (§10 Permission prompt, Modal fit): with `<html data-scale="2">`,
real Tab presses and windows of 1440 x 1000 and 960 x 667 in dark, light and
contrast, every choice is in view after its Tab, the band stays in view and
nothing clips horizontally (`c1/repair-2/kbd-real-keys.jsonl`). The page
behind still scrolls horizontally at 200 % from pre-existing gallery
elements (verify-1 step 13), not from the prompt or the grant rows.

**Reflow and text spacing (ds-1.1, matrix rows 7 and 34).** A list narrower
than 440 px stacks each row's cells under the name (container query on
`.ds-list`; the rule follows the base `.ds-row` rule because an `@container`
rule adds no specificity); menus are at most the viewport width minus 32 px.
`data-text-spacing="wider"` applies the WCAG 1.4.12 values (letter 0.12 em,
word 0.16 em, line height at least 1.5, paragraph 2 em) and widens the fixed
row columns. Harness `build/design-evidence/a11y-ds/writer/reflow.html` (every
gallery card at a given viewport): at 1024 px both spacings add no overflow
beyond a tooltip anchored at the right edge; at 320 px the rows, states,
progress, buttons, fields and menus reflow, but the privacy instrument (three
120 px columns), the tab list, the list header, two long button labels and
that tooltip still overflow, with and without wider spacing (O-A4). Run 2
replaced that harness with a full-gallery sweep (§21.9,
`build/design-evidence/a11y-ds/run2-writer/sweep.mjs`): the tab list now wraps,
the gallery reflows through container queries (which, unlike media queries,
also react to `[data-scale="2"]`), and the page no longer overflows at 1440,
720 or 640 px or at `[data-scale="2"]` on 1440 px, with either spacing. At
320 CSS px the list header, the 3 × 120 px privacy track and the `.ds-ask`
and `.ds-grant` specimens (task UX-C) still overflow (§21.9). Since run-2
repair 2 a `.ds-btn` label wraps instead of passing its button, and the
gallery's two-column specimens stack below 480 px of grid width, so no other
control label passes its box at 320 CSS px. The tooltips fit since run-2
repair 1 (§10, §21.9).

## 15. Keyboard rules

Default shortcuts (copied from DESKTOP-SHELL.md §2, "Default keyboard
shortcuts"):

| Key | Action |
|---|---|
| Ctrl+C / Ctrl+X / Ctrl+V | Copy / cut / paste |
| Ctrl+Z / Ctrl+Y | Undo / redo |
| Ctrl+S, Ctrl+A, Ctrl+F, Ctrl+P | Save, select all, find, print |
| Alt+Tab, Alt+F4 | Switch, close window |
| OS key | Zenit with Signaal |
| OS key+E / +I / +L / +D | Files / Settings / lock / show desktop |
| Ctrl+Alt+Delete | Trusted system screen (lock, switch, sign out, task manager) |
| Ctrl+Shift+Esc | Task manager |

S0 D5 adds namespaces (OS-owned keys never reach apps; Ctrl+Alt+Delete only to
the session service) and proposes Ctrl+N/O/W, Shift+Ctrl+Z and Ctrl+plus,
minus and 0 as an amendment to DESKTOP-SHELL.md (S0 open item 5); ds-1.0 does
not implement a keymap.

| Rule | ds-1.0 |
|---|---|
| Focus order | DOM order; Tab and Shift+Tab move between components |
| Groups, one tab stop each | Listbox rows and tabs use a roving `tabindex`; radios, including the segmented choice `.ds-seg`, are native groups |
| Arrow keys (gallery simulation) | Listbox: ArrowUp, ArrowDown, Home, End move and select (Left and Right ignored), Space selects; tabs: ArrowLeft, ArrowRight (also Up and Down), Home, End select and focus (automatic activation); menu: ArrowDown or ArrowUp on the trigger opens on the first or last item, arrows and Home and End move; Right and Left act as Down and Up, since no item opens a submenu. Arrow movement wraps at the ends. |
| Activation | Space and Enter on native buttons, switch and menu items |
| Held key (ds-1.1 run 3, tier 1, B8) | One activation per physical key press: an auto-repeated Enter or Space (`KeyboardEvent.repeat`) never activates a button, switch, menu item, tab or `summary` again (gallery.js, the first check in the document `keydown` handler). Without it, holding Enter on the delete-dialog opener moved focus to Annuleren, the next repeat closed the dialog and the one after reopened it: 56 open/close changes in 2 s, 28 per second, over 43.8 % of a 1024 x 768 screen in light as measured by the run-2 verifier (WCAG 2.3.1 limits flashes to three per second). The menu trigger, the toast close/show pair and every switch flipped the same way at 33 per second. Measured with CDP auto-repeat every 30 ms in dark, light and contrast, standard and dimmed plus reduced motion: before 54 to 66 changes per 2 s hold, after exactly 1 for Enter and Space in all 76 cases; four separate presses still give four changes (`build/design-evidence/a11y-ds/run3-writer/i1-hold-*.json`, `i1-taps-after.log`; independently verified in UX-A11Y-DS run 3, 19 November 2026). Arrow-key repeat (listbox, tabs, menus, the settings radios, already throttled to one preview change per 400 ms) and typing in text fields are unchanged |
| Escape | Closes the menu and returns focus to its trigger; closes dialogs and returns focus to the opener; answers a permission prompt with deny; hides a tooltip |
| Modal focus | `showModal()` makes the page behind the dialog inert; Tab cycles through the dialog controls and the browser UI, never page content; Escape always closes it, so there is no keyboard trap. Tab out of an open menu closes it. |
| Initial focus | Permission prompt: the title (`tabindex="-1"`); Enter and Space at open do nothing; Tab reaches the choices in visual order (§11.1; real key presses in `build/design-evidence/ds-1.0/c1/writer/kbd-real-keys.txt`). Delete dialog: Annuleren (the safe choice, NAMING-AND-IA.md §6) |
| Focus ring | §10: 2 px `--color-focus`, outer or inner. Real keyboard focus (repair-2, 176 Tab stops; repair-1 counted 179, three of them the hidden "Melding opnieuw tonen" buttons that rendered before the `[hidden]` rule): minimum 4.00:1, the light ring `#171717` on the neutral surround `#777777` (S0 D3 predicts 4.00); elsewhere at least 9.83:1 (contrast theme, `#FFE34D` inside a selected row `#333333`); these ratios are computed from styles by `m_focus.js`, not pixel-sampled in repair round 2 (§13 has the contrast-theme ring profiles). The 17 stops over gradient or translucent fills (Kern prompt buttons, Orbit items) were not pixel-sampled again; their markup and focus tokens are unchanged since the verifier's pass. |
| Shortcut display | `.ds-kbd` keycaps; S0 D5 names this `.ds-keycap` (O-S2) |

Evidence: the components stage sent real key and pointer events through the
DevTools protocol in the dark panel only (`build/design-evidence/ds-1.0/components/keys.json`,
`checks.log`). Repair round 1 sent real keys in all three
themes to the new compact and neutral specimens (tabs ArrowRight and Home;
listbox ArrowDown and End; switch Space; radio ArrowRight; Tab to the canvas)
and made one forward Tab pass over the whole gallery (179 stops, every stop
`:focus-visible` with a ring; `build/design-evidence/ds-1.0/repair-1/dn-keys.json`, `build/design-evidence/ds-1.0/repair-1/tools/kbd-raw.json`).
Repair round 2 repeated the pass after the `[hidden]` rule (176 stops, every
stop `:focus-visible` with a ring, no "Melding opnieuw tonen" stop;
`build/design-evidence/ds-1.0/repair-2/tools/kbd-raw.json`) and sent real keys in all three themes to the toast
(Tab to Melding sluiten, Enter hides the toast and focuses the now rendered
"Melding opnieuw tonen", Enter shows the toast, hides the button and returns
focus to Melding sluiten) and to the Versies tooltip (Tab shows it, Escape
hides it with focus kept, leaving and re-entering shows it again;
`build/design-evidence/ds-1.0/repair-2/flows.json`). The second verifier had already seen the light tooltip hide
on Escape.

**Settings group (ds-1.1).** Real key presses (CDP `Input.dispatchKeyEvent`) on
`?only=a11y`: one Tab stop per axis group in reading order, then "Standaard
herstellen"; Arrow Down and Up change the value at once and the preview
follows; Space on "Standaard herstellen" resets every axis, fills the
`role="status"` region and shows "Ongedaan maken"; Tab reaches it, Space
restores the previous values, hides it and returns focus to "Standaard
herstellen"; Shift+Tab walks back through every group. In the states card,
Tab reaches "Verplaatsen naar collectie", "Opnieuw proberen" and "Opslag en
NAS openen" with a 2 px solid focus ring
(`build/design-evidence/a11y-ds/writer/kb.json`).

## 16. Asset and font licensing

| Asset | Licence and provenance | Source |
|---|---|---|
| Inter 4.001 (`Inter[opsz,wght].ttf`) | SIL OFL 1.1; google/fonts commit `29b4ec6d`; SHA-256 `29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031`; licence `OFL-Inter.txt` `5b9321a4…` | `ui/design/fonts/FONTS.md` |
| Saira (wordmark v2) | SIL OFL 1.1; SHA-256 `9d050fc5a01c85f74c4257c207d50b55d1e40c37308c642f974a2c5003231dde`; licence `OFL-Saira.txt` `f2665d47…`; traced to outline paths at build time, no run-time dependency; used on brand surfaces only | FONTS.md; BRAND.md §4 (the Saira hash is pinned in FONTS.md) |
| BaryonOS mark | Project mark; generated deterministically by `ui/design/brand/generate_mark.py`; `LICENSING.md` ("Trademarks"): "MPL-2.0 does not grant general rights to the **Baryon OS** name, logos, or other project marks." | BRAND.md intro, §6; `LICENSING.md` ("Trademarks") |
| Icons | Drawn for the project (v2 sprite plus four ds-1.0 drawings); no third-party icon set | `icons.js` header; tokens stage report |

FONTS.md states that the OFL forbids selling the fonts by themselves and that
bundling them with BaryonOS is permitted; no conclusion beyond that is drawn
here. Native shipping of Inter still needs the `ui/native` licence and package
entries (FONTS.md; inter-atlas record open items 6 and 7).

## 17. Native mapping

Native code is not changed by ds-1.0. The plan is a **new, additive** `Palette`
const for Orbitaal in `ui/graphics/src/theme.rs`
(`docs/design/NATIVE-FEASIBILITY.md` §4 step 1). `MATERIAAL` and
`theme::wallpaper` stay untouched because `ui/native`, Veld, materiaal,
calculator and the compositor share them (NATIVE-FEASIBILITY.md §2).
`HIGH_CONTRAST` is the anchor of the contrast theme, not its equal (see the
table). The default native build still draws Open Sans; Inter is drawn only
with the cargo feature `inter-atlas`, off by default (`ui/graphics/Cargo.toml`;
`text.rs` header).

| `Palette` field / constant | Proposed ds-1.0 source (dark / light) | `HIGH_CONTRAST` today vs ds contrast |
|---|---|---|
| `desktop_start`, `desktop_end`, `glow` | No token: the v2 wallpaper is artwork; nearest is `--color-bg` for `desktop_start` | `0` / `#000` |
| `surface` | `--color-surface` | `0` / `#000` |
| `background` | `--color-bg` (`--color-surface-panel` has no field) | `0` / `#000` |
| `text` | `--color-text` | `0xffffff` / `#FFFFFF` |
| `muted` | `--color-text-muted` | `0xb8b8b8` / `#FFFFFF` (differs) |
| `accent` | `--color-focus` (no accent hue in ds-1.0) | `0xffff00` / `#FFE34D` (differs) |
| `accent_soft` | `--color-selection`, composited to an opaque value per base | `0x00ffff` / `#333333` (cyan rejected) |
| `border` | `--color-line` (one field; `--color-line-strong` has none) | `0xffffff` / `#FFFFFF` |
| `SHELL_TEXT`, `SHELL_MUTED` | `--color-text`, `--color-text-muted` over the wallpaper | — |
| `SHELL_LINE` | `--material-glass-line` or `--color-line-strong` (undecided) | — |
| `DIVIDER` | `--color-line` | — |
| `SHADOW` | Colour of `--elev-*` (black in dark; `rgba(36,54,108,·)` in light) | none |
| `HIGH_CONTRAST_DANGER` (`0xffb4b4`) | No counterpart: S0 D1 has no status hue | — |

Text roles (`ui/graphics/src/text.rs`; inter-atlas record table in
`docs/verification/native-inter-atlas-20261114/README.md`; Open Sans sizes from
NATIVE-FEASIBILITY.md §1):

| ds role | `text.rs` API | `inter-atlas` record (100 / 200 %) | Default Open Sans record |
|---|---|---|---|
| caption 13/18 | `Size::Small` | 13r / 26r | 12 regular |
| body 15/22 | `Size::Body`, `Weight::Regular` | 15r / 30r | 16 regular |
| body-dense 15/20 | as body | no 20 px line record | — |
| body-strong 15/22 | `Size::Body`, `Weight::Semibold` | 15sb / 30sb | 16 semibold |
| reading 16/26 | none | not built (16r / 32r) | — |
| title 22/28 | `Size::Title` | 22sb / 44sb | 32 semibold |
| display 48/56 | `Font::display` | 48r, magnified 2× at 200 % | 64 px record |

Icons: `ui/graphics/src/icon.rs` has 20 stroked polyline icons on a 24-unit
grid, with no curves, fills or particles. Nearest counterparts only:

| ds key | `icon.rs` | ds key | `icon.rs` |
|---|---|---|---|
| `zoeken` | `Search` | `close`, `min`, `max` | `Close`, `Minimize`, `Maximize` |
| `bestanden` | `Folder` | `back`, `fwd` | `ChevronLeft`, `ChevronRight` |
| `instellingen` | `Settings` | `check`, `plus` | `Check`, `Add` |
| `power`, `lock`, `user`, `apps` | `Power`, `Lock`, `User`, `Apps` | `internet` | `Network` (nearest) |

No native counterpart: the remaining 39 keys (aliases and the ds-1.0 addition `altijd` included), among them `device`, `nas`,
`notities`, `haven`, `versies`, `toegang`, `warn`, `trash`, `dot` and every file
kind. Native icons without a ds key: `Material`, `Key`, `Fingerprint`,
`Refresh`, `Shield`.

No native counterpart yet: place colours (no `Palette` field), materials and
gradients (no gradient primitive; per-pixel computation only), backdrop blur
(only over the static pre-blurred wallpaper, MF11), dashed edges, every motion
token (no animation clock), the reading and dense type records, the mark and
the wordmark (BRAND.md §4), and 125 % and 150 % scale (NATIVE-FEASIBILITY.md §1;
TYPOGRAPHY.md §8.6).

**Axes (ds-1.1).** Native code is not changed by ds-1.1. Planned mapping:

| Axis | Native mechanism (plan) |
|---|---|
| Contrast, transparency, colour perception, stimulus | Additional `Palette` variants in `ui/graphics/src/theme.rs`, selected per axis value; same names, no new roles; an axis value applies to the surface that carries the theme, never to a part of it (placement, §21.3) |
| Light intensity | Output scaling in the compositor after composition, applied last |
| Motion | A global duration scale (0 or 1) read by every animation |
| Text spacing, line length, text size | `text.rs` spacing and measure parameters; text size needs atlas records (TYPOGRAPHY.md §8.6, §9; O-A1) |
| Target size | Layout minimum read from the settings store |
| Device preferences | The settings store is the single source; a fresh install starts from the defaults |

## 18. Versioning and change rules

| Change | Version step |
|---|---|
| Rename or remove a token, class, attribute, theme selector or icon key; change a token's role; change keyboard behaviour; a value change that drops a listed pair below its threshold | Major (`ds-2.0`) |
| Add tokens, components, variants or icons; replace PROVISIONAL values within the thresholds; reserved S0 names becoming real (ds-1.1 per S0) | Minor (`ds-1.1`) |
| Documentation fixes; value corrections that keep every pair | Patch (`ds-1.0.1`) |

Repairs made before the project owner accepts ds-1.0, such as repair rounds 1
and 2, keep the version `ds-1.0`.

**ds-1.1 (15 November 2026, task UX-A11Y-DS) is a minor step.** It adds axis
attributes and values, the media-feature mappings, the sub-classes
`.ds-empty-orbit`, `.ds-empty-sat`, `.ds-empty-place`,
the `.ds-t-reading` measure and the settings group; it renames or removes no
class, token, attribute value or keyboard behaviour and keeps every listed pair
above its threshold. Repair 1 of ds-1.1 (same version, not yet released): the
loading `.ds-empty` is marked `data-state="busy"` instead of `aria-busy`, adds
`.ds-empty-tick`, stacks below 320 px of inline size; the light-intensity
compensation tokens, the six stimulus elevation tokens, opaque contrast-theme
warning lines and re-derived colour-perception palettes were added; the
reading measure is 58ch and the short measure 37ch, both counted in
characters; the settings preview applies at most once per 400 ms. Repair 2 of ds-1.1 (same version): adds the
segmented choice `.ds-seg` and the place arc on `.ds-empty-sat`; the error ring breaks once instead of
dashing; `.ds-empty` labels and `.ds-privacy` wrap; the settings group is grouped, follows the chosen
theme and gets an Actief strip with a reset per axis; dialog and toast actions keep their button width
and wrap, a narrow plain dialog stacks its actions, and `.ds-fieldset` takes `min-width: 0`; nothing is
renamed or removed. Run-2 repair 1 (same version): `.ds-tip` gains the
`data-align` values `start` and `end` and wraps at 15rem; the `.ds-empty` orbit
ring is 2 px with a tapered 110° place tail; the gallery's combinations panel
gives the reading text a full-width strip; nothing is renamed or removed. Removed were internal keyframes (`ds-shimmer`,
`ds-slide`) and the reduced-motion stripe fill of the indeterminate bar,
replaced by a centred blade segment with the same word and role;
`--motion-duration-loader-pulse` stays defined but unused. Run-2 repair 2
(same version): `.ds-btn` wraps a label that does not fit its row
(`white-space: normal`, `min-height` instead of `height`, also for
`data-size="large"`), `.ds-ask .ds-btn` keeps one line, and dialog and toast
buttons no longer keep their label width (`flex-shrink: 0` removed); nothing is
renamed or removed. The axis blocks in
`tokens.css` were written by hand, like repair 1, and are verified by
`docs/verification/a11y-ds-20261115/check.mjs` (exit 0 only when every check
passes); the generator must take them over before the next value change (O-P1,
O-A9).

Rules: `tokens.css` is regenerated, not hand-edited
(`build/design-evidence/ds-1.0/work/gen_tokens.py`, see O-P1). Exception:
repair round 1 edited it by hand because it could write only `ui/design/system/`
and its evidence folder; the generator must take over the repair-1 values before
the next value change (O-P1). Every value keeps
its v2 source or a recorded derivation; the contrast script in §4 is re-run on
every colour change; `ui/prototype/tokens.css` is never touched. Architecture
changes are recorded in `docs/DECISIONS.md`. The project owner can redirect any
decision; the design pipeline may change values only within this document's
rules.

## 19. Not verified

- No human review, no user or task trial (U06), no screen-reader or
  assistive-technology test. All checks were run by AI agents.
- No native rendering of any ds-1.0 token, icon or component; no QEMU run.
- Contrast of glass, Orbit material and the translucent launcher over real
  wallpaper pixels; §4 assumes `--color-bg` as backdrop.
- `data-density="compact"` and `data-surface="neutral"` are rendered for a
  subset of components only (§10); menu, dialog, toast, window chrome and the
  privacy instrument were not rendered in either mode.
- Keyboard sequences and modals in the light and contrast themes (repair
  round 2 exercised only the toast and tooltip paths there); Firefox and Safari (the checkbox tick uses `::after`
  on `<input>`, which Firefox does not render).
- Colour-vision deficiency simulation of the place colours.
- 200 % only as CSS `zoom` on one strip; no device scale factor; no reflow at
  320 CSS px; menu, tabs, dialog, toast, window chrome and the Orbit are not
  in the strip (both permission prompts and the grant rows are, since UX-D-C1).
  The modal permission prompt was measured at 200 % with `zoom` on `<html>`
  only since C1 repair-2 (two window sizes, three themes); the delete dialog,
  menus and toasts were not measured at 200 %.
- UX-D-C1 (permission model): the contract self-check, the gallery
  self-measure, a harness for labels, sizes, clipping, contrast and flows, real
  key presses through the chrome-devtools MCP, and renders
  (`build/design-evidence/ds-1.0/c1/writer/`). The writer could not view the
  renders (image reads were blocked in this session); a visual review is
  outstanding (O-G6). The crops use y-ranges measured in a 1425 px-wide iframe
  applied to a 1440 px render, so their edges are approximate. The two contrast-ring rules for the Orbit bar and the
  menu offset were applied without a new pixel profile.
- UX-D-C1 repair-1 (`build/design-evidence/ds-1.0/c1/repair-1/`): contract
  test (48 assertions), a writer harness with fixed 80 ms waits (the verify-0
  failure mode) and a same-task race (answer one prompt and open the next
  before the first `close` event), reach and duration chips (contrast, 24 px
  height, no clipping at 1440 px and with the list at 320 px, unique revoke
  names), real Enter, Escape, Tab, Shift+Tab and Enter-on-a-choice presses and Chrome's accessibility tree through
  the chrome-devtools MCP (`real-keys-and-a11y.txt`). Expected screen-reader
  output on open, from that tree and not from a screen reader: "Webvenster wil
  internet gebruiken, alertdialog" (once), the description "Dit programma kan
  de toegestane inhoud naar een internetdienst sturen.", then the focused
  heading; each choice reads its name and its duration line. NVDA, JAWS,
  VoiceOver and Orca were not run. The writer viewed the dark and contrast
  grant cards in screenshots; light and 200 % renders of the new rows were
  checked by measurement only.
- UX-D-C1 repair-2 (`build/design-evidence/ds-1.0/c1/repair-2/`): real
  Enter, Tab, Shift+Tab and Escape presses through the chrome-devtools MCP on
  the modal prompt at 200 % (`kbd-real-keys.jsonl`, 12 runs, all pass; one
  run repeats the ongoing prompt because the asynchronous `close` event
  returned focus to its opener, recorded as such; the first four runs predate
  the `overflow: clip` and `scrollbar-color` lines, and only the last capture
  run uses the final scrollbar track (`var(--material-kern-bg)`); neither
  change alters measured geometry. The runs' in-view flags compare each
  focused choice with the window; their logged rectangles also lie between
  the band's bottom and the dialog's bottom, and a final check with real Tab
  presses on the final CSS found every choice inside the `.ds-ask-body`
  scrollport); a 100 % check (body does
  not scroll, `--elev-trusted` shadow present, dialog `overflow: visible`);
  real Enter on an Intrekken button (four distinct names, the announcement
  names all four fields); `revoke-names-test.txt` (6/6), the verify-1 and
  repair-1 contract tests (36/36, 48/48), the verify-1 consistency check (no
  change) and the verify-1 measure harness (re-run on the final CSS and
  icons: no difference from verify-1, 0 failures, normal and reduced motion);
  the gallery self-measure (54 icon keys, none unresolved; run before the
  final particle position, the key count re-checked on the final file); real ArrowDown, PageDown and End with focus on the
  title scroll the body to its maximum. The writer viewed, in chrome-devtools
  screenshots, the 200 % prompt in all three themes with focus on Weigeren,
  the 100 % dark-page prompt, the final-CSS 200 % prompt at 960 x 667 (light
  page, End pressed: scrollbar inside the rounded edge, band and last line in
  view) and the grant chips with the final `altijd` glyph in light (100 % and
  200 %) and contrast (100 %); the dark chip was viewed only with earlier
  particle positions. Saved captures: `r2-*.png` in the same folder (the
  `-final` ones use the final CSS and icons). The `altijd` and `klok` glyphs
  were compared side by side in those screenshots, not in a pixel crop.
- Animation was not watched or timed; reduced motion was checked by computed
  styles.
- Contrast theme: the compact place badge, the compact blade end, compact tab
  descenders and the Orbit badge beside inner focus rings (O-K2).
- Repair round 1 re-ran the gallery self-measure, the computed and
  pixel-sampled contrast sweep (0 computed failures, 0 controls under 3:1; the
  pixel sampler flags 13 glyphs, Orbit tile icons and contrast-theme place
  nodes, against 9 of the same kinds in the verifier's run; not analysed further), the keyboard checks above and clipped renders
  (`build/design-evidence/ds-1.0/repair-1/`). The other rendered and keyboard results are the tokens and
  components stages'. Renders and `SHA256SUMS` in `build/design-evidence/ds-1.0/`
  and `components/` predate repair-1 and are stale for the changed tokens,
  typography and gallery.
- Repair round 2 re-ran the gallery self-measure (0 errors, 53 icon keys
  resolved, 224 targets with none under 24 px, 1 311 visible text elements
  with none outside the seven roles), the forward Tab pass, the toast and
  tooltip key paths, the contrast-ring profiles and clipped renders
  (`build/design-evidence/ds-1.0/repair-2/`). Repair-1 renders and `SHA256SUMS` are stale for the changed
  `components.css` and `index.html`.

Evidence paths cited here under `build/design-evidence/ds-1.0/` are ignored by
Git; the lead must move what is needed to `docs/verification/` (O-P2).

**ds-1.1 (UX-A11Y-DS).** See §21.9. In short: no user has tested any axis or
the reworked states; colour-perception palettes rest on simulation only; light
intensity on an sRGB model only; forced colours only emulated; the wallpaper
and the boot sequence were not analysed for flashes or patterns; reflow at
320 px still fails for the list header and the privacy track (ds-1.0) and for
the `.ds-ask` and `.ds-grant` specimens (task UX-C) (§14, §21.9); nothing is
native yet.

## 20. Open items

| ID | Item |
|---|---|
| O-C1 | Store the repair-1 S0 D3 run (`build/design-evidence/ds-1.0/repair-1/check-report.txt`, `tools/contrast-pairs.md`, `tools/tune.py`, `tools/mapping.tsv`) under `docs/verification/` (S0 D3, S0 open item 2); this round had no write access there. |
| O-C2 | Closed in repair-1: dark secondary-button edge 3.48:1 over raised, 3.91:1 over a window (§4.1). |
| O-C4 | S0 D3's twin table (`docs/design/suite/WORKSPACE-REQUIREMENTS.md`) still lists the earlier twins `#747474`, `#969696`, `#818181` and `#626262`; ds-1.0 now uses `#858585`, `#A4A4A4`, `#757575` and `#5B5B5B` (§4.1). The drift from v2 is not yet in `docs/DECISIONS.md`. |
| O-C3 | Muted versus subtle text separation fell to 1.20:1 (dark) and 1.27:1 (light) with the repair-1 values; confirm the hierarchy in a human review. |
| O-T1 | Tabular digits: `.ds-num` spans only in the browser versus every digit in TYPOGRAPHY.md §8.1 and §8.4. |
| O-S1 | Missing states: pressed on row, tab, menu item, switch, checkbox and radio; disabled on row, tab, Orbit item and window control; loading only on the button and skeleton; checkbox indeterminate; checked menu items; error on switch, checkbox and radio; no dedicated warning-banner rule. |
| O-S2 | S0 reserved names not implemented: regions (`.ds-workspace`, `.ds-menu-bar`, `.ds-toolbar`, `.ds-tool-rail`, `.ds-canvas`, `.ds-inspector`, `.ds-panel`, `.ds-panel-zone`, `.ds-timeline`, `.ds-data-grid`, `.ds-status-bar`, `.ds-document-tabs`, `.ds-command-palette`) and components (`.ds-panel-header`, `.ds-track`, `.ds-ruler`, `.ds-playhead`, `.ds-cell`, `.ds-number-field`, `.ds-layer-row`, `.ds-expander`, `.ds-export-dialog`, `.ds-object-chip`, `.ds-version-indicator`); S0 names `.ds-keycap` and `.ds-place-badge` where ds-1.0 has `.ds-kbd` and `.ds-place`. |
| O-S3 | Attributes and hooks beyond the fixed naming list: `data-kind`, `data-until`, `data-initial-focus`, `data-remember`, `data-forget`, `data-change`, `data-decision`, `data-held`, `data-compact`, `data-step`, `.ds-live`, `.ds-key` (UX-C run 2), `data-size`, `data-shape`, `data-demo`, `data-choice`, `--ds-c`, `--ds-value`; sub-classes such as `.ds-status-marker`, `.ds-node`, `.ds-orbit`, `.ds-list`, `.ds-input`. |
| O-S4 | Submenus: `.ds-menu-sub` is a visual marker only; opening, ARIA (`aria-haspopup`, `aria-expanded`) and keys (ArrowRight opens, ArrowLeft and Escape close) are not specified. |
| O-K1 | Launcher, search results, switcher, tray, notification centre, quick settings and window move, resize and snap are not designed as components. |
| O-K2 | UX-D-C1 applied the base verifier's two one-line fixes (`.ds-taskitem::after` bottom 4 px and `.ds-menu` 5 px offset in the contrast theme); not re-profiled. Contrast theme, partly closed in repair round 2: tabs and the standard selected row now keep a 1 px gap between the inner ring and white marks (§13); the focused row's blade moves 3 px inward, a repair-2 derivation not in v2. Open: the compact place badge edge (258 junction pixels, lowest 1.06:1), the compact blade end (2 pixels) and compact tab descenders (12 pixels) touch the ring; the Orbit badge is not measured. Compact values are PROVISIONAL (S0 D2). |
| O-K3 | `.ds-taskitem` shows app icons as bare glyphs; MF9 puts every app on the tile. |
| O-N1 | "Kern" names the trusted-surface material in ds-1.0 and the Settings app in DESKTOP-SHELL.md §7; P-NAMES-1 awaits the project owner. |
| O-N2 | Since UX-C run 2 the prompt copy comes from `strings.js` `trusted.*` (§11.1); NAMING-AND-IA.md §6 still lists the C1 labels and needs an update outside this task. |
| O-N3 | The NAS-unreachable sentence implies but does not state "paused" and "not forwarded elsewhere" (LOCAL-PRIVACY-CONTRACT.md §1); adding "en wordt niet naar een andere plek gestuurd" needs a copy decision. |
| O-N4 | The gallery names the browser app "Webvenster" (v2 board), since C1 repair-2 from `strings.js` `apps.webvenster`; `strings.js` also keeps `apps.browser` "Browser", so the product name of the browser app is still undecided. The prompts' "what" and "done" fragments in `gallery.js` are marked gallery sample data, not `strings.js` copy (O-D1). |
| O-G1 | A project ADR is required before native remembered decisions (allow and deny), their storage, change and revocation; draft `docs/verification/uxc-run2-20261119/PROPOSED-ADR.md` (Proposed). Native capability and security contracts are unchanged. |
| O-G2 | Closed by the owner decisions of 15 and 19 November 2026: Weigeren can be remembered for the exact key; no per-app or per-right quieting (OQ-4). OQ-3 (suppression until logout across relaunches) is a proposed ADR change, not built. |
| O-G3 | Matching stays exact on the five-field key (OQ-4); whether a broader remembered decision may cover a narrower request needs a policy-service decision in the ADR. |
| O-G4 | Closed (owner decision 15 November 2026): no separate "Eenmalig toestaan"; a single action without remembering is allowed for that action only. |
| O-G5 | UX-C run 2 built concept b in the design system; comprehension, the ~120-word target (read case measures 136 visible words), the compact form's effect and gesture detection are untested hypotheses until the user study runs. The old C1 contract test (`docs/verification/ds-1.0-c1-20261115/contract-test.mjs`) tests the superseded model and fails against the new `contract.js`; it stays as a historical record. |
| O-G15 | A `readWrite` request on an item stored on the NAS has four material consequences (can change or erase the content; time-bounded network wording; revocation and copies; the NAS administrator line). All four stay visible, which exceeds the MF-6 limit of three essential facts; the owner decides whether one may move or the limit admits a fourth for this right (UX-C run 2 repair-1). |
| O-G16 | Repair-1 placed the per-answer outcome sentences in the fixed decide row, directly above Toestaan and Weigeren, in viewports taller than 40rem (measured in view at open for all five cases at 1440 x 1000, 1440 x 900 and 1280 x 720; `run2-repair-1/out/outcomes.json`). In a viewport of at most 40rem (200 % zoom of a 1440 x 900 window, 320 x 640) or with `[data-scale="2"]` they move to the end of the scrolling body so that the question and all controls stay reachable; there they can be scrolled out of view while both answers stay visible (still linked by `aria-describedby`). Whether a shorter outcome form for short viewports is preferable is an open design question. |
| O-G6 | The C1 renders (`build/design-evidence/ds-1.0/c1/`) show the superseded prompt; UX-C run 2 renders and measurements are in `build/design-evidence/uxc-run2/run2-writer/`. |
| O-B1 | Kern band uses `lock`; BRAND.md §5 places the compact mark on trusted surfaces. Since C1 repair-2 `lock` appears only on the Kern band and the prompt openers in the gallery, not on grant chips. |
| O-B2 | BRAND.md §5 describes red, green and blue as accents for highlights and app differentiation, and the mark's node colours differ from the place colours (BRAND.md §4); both sit against "colour means place" and need a project-owner decision. |
| O-I1 | No info or restore icon: the info banner uses `dot` and "Vorige grootte" uses `window`. |
| O-P1 | `tokens.css` names its generator under ignored `build/`; the generator and mapping must move into the repository for the file to be reproducible. `work/gen_tokens.py` and `work/mapping.tsv` predate repair-1, which edited `tokens.css` by hand; the current mapping is `build/design-evidence/ds-1.0/repair-1/tools/mapping.tsv`. |
| O-P2 | Move the tokens, components and §4 contrast evidence from `build/design-evidence/ds-1.0/` to `docs/verification/`. |
| O-P3 | DESKTOP-SHELL.md §7 still describes a mint "Lokaal" privacy light; CONCEPTS.md MF2 removes the mint tint, and ds-1.0 uses the three-node instrument. |
| O-D1 | Gallery sample data not in `ui/design/prototype/data.js` (file sizes, "Vakantiefoto's Lissabon"; the seeded "Altijd" grant Notities · collectie Tuin, created through `ask()` at gallery load; the two prompts' "what" and "done" fragments in `gallery.js`). |
| O-G7 | The Apple HIG Privacy and Chromium Blog rows in §11.1 are not re-verified (both render client-side; the verifier retrieved only the Chromium date). Apple's "ask once, then redirect to Settings" behaviour named in the task brief is not cited. Quote both from a retrieved copy, or from Apple's API documentation, before relying on them. Since C1 repair-2, option B cites only Android as support. |
| O-G9 | Interpretation for owner confirmation: the destination of a sending right is part of the scope ("reikwijdte") of the four-field key, carried as `to` in the contract (§11.1). The alternative is a fifth key field; the behaviour is the same. |
| O-G10 | Contrast theme exception, pre-existing since `46957bac`: the decorative row, window, Orbit and privacy-connector blades keep transparent gradient ends over black (§13), against the "no translucency" rule. Justification: their solid centre carries the cue at 21:1 and the ends are decoration; either confirm the exception or make the ends opaque in `[data-theme="contrast"]` and re-profile the focus-ring gaps (O-K2). |
| O-G11 | The grant row's reach badge and duration chip are a design proposal for UX-E (§11.1); their scanning benefit is AI judgement, untested with users (U06). |
| O-G12 | Owner question, label not changed: "Toestaan tot afsluiten" can be read as "until shutdown". The BaryonOS power menu itself uses "Afsluiten" for shutting down (`strings.js` `shell.shutdown`, NAMING-AND-IA.md Power row: "Aan/uit: Slaapstand, Opnieuw opstarten, Afsluiten"), and Windows NL uses the same word (ADR-058). The contract ends the grant earlier, when the app closes; the duration line "Geldt tot je <App> sluit" states this. Under owner rule M3 (no label may suggest another validity) the owner is asked to confirm "afsluiten" or clarify it, for example "Toestaan tot je de app sluit". The verify-1 note that BaryonOS has no internal clash is not correct. |
| O-G13 | The `altijd` glyph (closed loop) is a ds-1.0 addition, not in the reviewed v2 sprite (§8); it replaces `lock` on the "Altijd" grant chip. Brand review and a user check of its reading are pending. The final form (particle r 2.1 on the top of the right lobe) was viewed at 16 px in light and contrast and at 32 px in light; at 16 px the particle reads as a small dot on the loop, slight but present (AI judgement). |
| O-G14 | UX-E design proposal, not built: grants grouped per app with a combined-reach privacy strip (§11.1). Needs its own keyboard, focus-after-revoke and screen-reader design. |
| O-G8 | `docs/verification/suite-s1-capability-matrix-20261114/` may hold a 215-row summary; the matrix now has 216 rows (XC09) and the stored summary was not regenerated (no write access). |
| O-A1 | Text size 125 % and 150 % as an axis (matrix row 13): more atlas records against the 512 KiB read-only segment of `ui/native` (TYPOGRAPHY.md §8.6, §9); outside the UX-A11Y-DS write set (§21.8 E1). |
| O-A2 | Motion middle level "reduced" with cross-fades kept (rows 30, 46): a third `data-motion` value, one token block and about ten component rules (§21.8 E2). |
| O-A3 | Visual-intensity axis beyond glow (row 47): gradients, density and wallpaper under `data-stimulus="calm"`. |
| O-A4 | Reflow at 320 px (row 7) and text spacing (row 34): since run 2 the tab list wraps and fits at 320 px with and without wider spacing, and place pills wrap in the stacked list form. At 320 CSS px the page still overflows by up to 147 px: list header 178.8 px, privacy track 128.8 px (3 × 120 px); `.ds-ask` and `.ds-grant` (UX-C) 137.1 and 38.9 px. The compact density text, the switch, radio and field labels and the asks openers were fixed in run-2 repair 2 (`.gal-2` stacking, `.ds-btn` wrapping; §21.9). Tooltips: fixed in run-2 repair 1 by edge alignment and a 15rem wrap; no open tip leaves the viewport at 320 CSS px or wider (§21.9; earlier classes in `build/design-evidence/a11y-ds/run2-writer/final-classes.txt`). |
| O-A5 | A designed forced-colors mode beyond the place-node shapes (row 86), tested on a real platform. |
| O-A6 | Deeper light-intensity steps (row 19): a per-theme dimmed token family (about 90 values per theme) because a uniform filter below 0.85 breaks the floors (§21.8 E3). |
| O-A7 | Split the contrast theme bundle (no translucency, glow, halo or entrance rise) into the transparency, stimulus and motion axes (row 88). |
| O-A8 | Nested axis reset inside a subtree via `@container style()` queries (§21.3). |
| O-A9 | Move the hand-written axis blocks into the token generator (O-P1) and store the UX-A11Y-DS evidence under `docs/verification/` (O-P2). |
| O-A10 | User tests for every axis, the settings group and the reworked states, with people who use these adaptations; flash and pattern analysis of the wallpaper and the boot sequence (rows 24, 25). |
| O-A11 | Compositor colour filter for third-party content (row 5, §21.8 E4): a colour-matrix stage after composition in the native compositor. |
| O-A12 | Compositor flash filter (row 26, §21.8 E5): research on frame-to-frame luminance analysis first, then a compositor stage. Amendment A1 of ACCESSIBILITY-RESEARCH.md §3.5: it protects unsolicited content without a "show anyway" control or per-item reveal; the protection follows the unsolicited placement, so a start, open, enlarge, forward or hand-off action from it never lifts the protection before the item is assessed safe: it ends only the static-first hold where the guard runs for the item, and otherwise (for example DRM, HDR, live or own-pipeline rendering) the static or otherwise safe presentation stays (owner decision B of 19 November 2026); deliberately started games and media play normally, with the optional `flash-attenuation` axis (matrix row 93), separate from `motion`; no per-user record of hazardous media or viewing history. No claim that it detects or prevents every hazardous flash. |
| O-A13 | Native assistive technology (§21.8 E6): screen reader with accessibility tree and trusted AT role, magnifier, read-aloud, captions, keyboard filters, switch, eye, head and voice control; one native subsystem and ADR each. |
| O-A14 | Native mapping of the axes (§17, §21.8 E7): one `Palette` variant per colour-axis value in `ui/graphics/src/theme.rs`, a global duration scale, compositor output scaling for light intensity and layout minimums from the settings store. |
| O-A15 | Dimmed light theme (row 19): uniform channel scaling greys the light theme; a deeper, slightly warmer dimmed light palette as a token family (AI design judgement; a tint is not a comfort claim, row 34) needs its own pair checks (part of E3). Raised again by the repair-2 verifier (the × 0.85 filter turns light surfaces a uniform mid-grey, AI design judgement); still open. |
| O-A16 | Settings presentation: done in repair 2 as grouped segmented tiles with a static sample per value and an Actief strip (§21.5). Open: a sample that shows the value combined with the other active axes, and the design of the Instellingen shell around the group (AI design judgement). |
| O-A18 | Neutral surface under dimmed light or increased contrast: the axis blocks write tinted text and line roles over the neutral re-pointing; give each block neutral twins that win on `data-surface="neutral"` without raising specificity above the owner blocks. |
| O-A17 | Gallery page overflow at 640 CSS px (200 % of 1280 px). Fixed in run 2: the gallery grid, columns, header, stage and 200 % strip reflow (container queries, so `data-scale="2"` also triggers them), and the tab list wraps; page overflow is 0 at 1440, 720 (DSF 2) and 640 px and at `data-scale="2"` on 1440 px in all 24 axis combinations and three themes (`build/design-evidence/a11y-ds/run2-writer/sweep.json`). The 320 CSS px case stays open under O-A4. |
| O-A19 | A ds-1.0 `.ds-btn` shrinks below its label in a row that does not wrap: `min-width: var(--space-target-min)` replaces the automatic minimum size, so at 720 and 640 CSS px and at 200 % labels spilled sideways: 183 hits (one per label per configuration) in the stored repair-2 run `build/design-evidence/a11y-ds/repair-2/fit-3.txt`, 180 labels 2 to 66 px past their button and the Schaal fieldset 2 px past its card. Repair 2 cleared the gallery cases: the button-state matrix keeps its columns at their content width and scrolls inside its card (a two-dimensional layout, WCAG 1.4.10 exception), `.ds-toast-actions` wraps, buttons in `.ds-dialog-actions` and `.ds-toast-actions` keep their label width (`flex-shrink: 0`), a plain dialog of at most 320 px stacks its actions at full width with wrapping labels, and `.ds-fieldset` takes `min-width: 0`; the harness now reports 0 (`fit-final.txt`; an intermediate run with 18 hits was not kept). Run-2 repair 2 changed the `.ds-btn` rule itself: a label that does not fit its row wraps between words in a button that grows (`white-space: normal`, `min-height` instead of `height`), so `flex-shrink: 0` on dialog and toast buttons was removed. This followed the run-2 verifier's finding that the asks card's opener buttons spilled at 320 CSS px ("Vraag voor één handeling openen" 6.3 px, 39.0 px with wider spacing; "Doorlopende vraag openen" 6.8 px with wider spacing; also at 640 px with `data-scale="2"`), which no earlier record listed. Evidence: `run2-repair-2/lbl.mjs` (before `lbl-before.json`, after `lbl-after4.json`). Open: `.ds-ask .ds-btn` keeps one line (task UX-C); at 160 CSS px (`data-scale="2"` on 320 px) its "Toestaan tot afsluiten" passes its button with wider spacing by 16.3 px of the unzoomed viewport (about 8.2 CSS px at zoom 2; 17.3 and 8.7 in the contrast theme). |
| O-A20 | `docs/verification/a11y-ds-20261115/README.md` still quotes the repair-1 hashes and the claim "0 clipped elements" for `.ds-empty`; it is outside this repair's write set. |
| O-A21 | Design quality (AI opinion, run-2 verifier; not user evidence): Instellingen › Toegankelijkheid still reads close to an existing OS accessibility page (a preview pinned beside the edited group, or orbit and blade language in the Actief strip, were not attempted); the `.ds-empty` composition remains icon left, text right; run-2 repair 1 only strengthened the ring and tail, rendered at rest and in reduced motion, not mid-loop. Second run-2 verifier (AI opinion): the group is a stack of segmented controls with a generic "Aa" tile sample, and the preview changes only once a value is chosen. Proposed, not built (it would touch the 400 ms flash limiter and the stored keyboard evidence): while a segment is hovered (keyboard focus already selects), the preview shows that value through the same limiter and returns when the pointer leaves, never for Thema, which would restyle the panel under the pointer; the "Aa" samples become Orbitaal specimens (a place node, a list row, the ask band). Stimulus: in the dark theme "Rustig" and "Rijk" are hard to tell from "Standaard" in the preview (pixels differing by more than 8 per channel: dark 0.9 % calm and 6.0 % rich, light 6.2 % and 10.7 %; `run2-repair-2/stim.mjs`, `stim-diff.txt`), because shadows barely show on the dark base. The stimulus layer lives in the protected `tokens.css`; a proposed dark-rich halo (window and Orbit halo alpha 0.2 → 0.34 with a 4 to 6 px spread, injected for measurement only) raises dark rich to 12.6 %, and calm has no token-only lever that keeps the blade signature, so a stronger calm needs a design decision (for example no glow and a shorter blade on inactive surfaces). |
| O-A22 | Row 21 (§21.6): the contrast-theme skeleton (six pairs on a vertical scan, up to nine along one placeholder row at 21:1), the stacked place-node column (nine pairs, every theme) and the `.ds-empty` orbit frame (nine vertical and six horizontal pairs, up to 8.5:1 dark and 21:1 contrast; run-2 verifier) pass only through the icon and component-structure clause. Accept, or reduce (for example outline only the outer frame, or fewer placeholder rows); a lead or project-owner decision. |
| O-A23 | Tooltip placement is chosen by the author (`data-align`); there is no automatic flip near an edge, and §17 has no native placement rule yet. Recorded, not fixed: the compact-density "Vergelijken" label passes its button by 2.0 px at 320 px with wider spacing, and `.ds-row-name` truncates without a `title` (§21.9). |
| O-A24 | Text inside specimens, not gallery captions or notes, that still sits in a neighbouring specimen's modelled shadow fringe (alpha 0.02 to 0.1; run 3, `build/design-evidence/a11y-ds/run3-writer/i2-shadow-after.json`): the Standaard, Rustig and Rijk labels beside the stimulus samples (their own `--elev-plate`), the menu trigger Bestand under its open menu, the swatch labels of the elevation and material reference (each tile's shadow reaches the next tile), the place labels of the 200 % strip beside its ask, and two settings-preview hints under rich. All pass the run-3 rendered contrast sweep (0 failures in 19 configurations); whether a specimen's own composition should also keep its shadow off neighbouring labels is a design decision, not a contrast defect. Run 3 repair 1 moved the elevation reference's effect on the next panel's heading, its subtitle and the swatch value captions out of this item. These are gallery text, and the reference now clips its own reach (§21.9). The tile-to-tile labels inside the reference remain here. |
| O-A25 | The gallery's reach rules (run 3, repairs 1 and 2, gallery `<style>` in `index.html`) are px copies of the `--elev-*` geometry, and no automated check enforces them. The elevation reference clips its own reach, so a token change there can only crop it. The ask row, Orbit stage, settings preview, menu and window and toast rows have no such clip. A token change there could put a caption back in a shadow without failing any gate. Proposal for the lead: add a static check to `check.mjs`. For each theme block and stimulus block in `tokens.css`, it would parse the outer layers of each `--elev-*` value and compute the reach: up is blur plus spread minus the y offset, and down is the y offset plus spread plus blur. It would then assert that each reserved literal in the gallery `<style>` is at least the largest reach it covers. These literals are the `.gal-asks` margins (`--elev-trusted`), the Orbit stage padding at the bottom and the rich top (`--elev-orbit`), and `--gal-reach-up`, `--gal-reach-down` and `--gal-pop-down` per stimulus value (`--elev-window`, `--elev-window-inactive`; below a card less `--gal-air`). They also include the rich top padding of the settings preview (`--elev-window`) and `--gal-elev-up` and `--gal-elev-down` (all nine reference tiles). `build/design-evidence/a11y-ds/run3-repair-2/reachcheck.mjs` is a rendered reference for the same rule. Repair 2 closed the two class instances this item listed (the Menu card heading and the accessibility panel subtitle). It also closed three more: the Orbit-item heading, which the run-3 verifier reported, and two from its own wider sweep, the Banner heading at 720 px and the Melding met acties heading at 44 px targets. The clipped reach check finds 0 overlaps in 51 configurations. The run-3 verifier's unclipped geometry still counts the stage dialog over the Voortgang note under rich at 1000 to 1200 px; the stage clips that shadow (`overflow: hidden`), and the pixel delta there is 0 (§21.9). What stays open is only the missing automated check. |

## 21. Accessibility: two tiers (ds-1.1)

ds-1.1 (15 November 2026, task UX-A11Y-DS) turns the accessibility research
(`docs/design/accessibility/ACCESSIBILITY-RESEARCH.md`, matrix
`docs/design/accessibility/ACCESSIBILITY-MATRIX.md`) into two tiers. Tier 1 is
the baseline: it holds in dark, light and contrast with no setting. Tier 2 is a
set of independent axes, one data attribute per axis, defined for every theme
and combinable with every other axis. Settings describe properties of the
interface, never a diagnosis or a person (matrix row 88). Every statement below
is design and browser-simulation evidence; no user has tested any axis, and
an AI judgement is not user evidence. Nothing here is a medical claim.

### 21.1 Tier 1: the baseline (no setting)

| ID | Baseline guarantee | Implementation | Matrix row |
|---|---|---|---|
| B1 | Text at least 4.5:1, UI parts, focus and blade at least 3:1, in every theme and every axis combination, on every surface and under every hover, selection and button overlay, at full and dimmed light; 7:1 for text under increased contrast. Run 2 measured the gallery caption `p.gal-note` below the ask specimens at 3.47:1 in light, inside the ask card's shadow; run 3 and its repair 2 keep every specimen shadow's reach free of gallery captions and notes (clipped reach check: 0 overlaps in 51 configurations; the stage dialog's shadow, which an unclipped model counts at 1000 to 1200 px under rich, is clipped by its stage, pixel delta 0; §21.9) | `tokens.css` theme and axis blocks (dimmed compensation per theme); `docs/verification/a11y-ds-20261115/check.mjs` (6) evaluates every §4 pair on every overlay stack at sRGB × 1 and × 0.85 | 6 |
| B2 | A 2 px focus ring that keeps 3:1 in every combination | `components.css` focus rules; `check.mjs` focus pairs | 61 |
| B3 | Every hit area at least 24 × 24 CSS px, including rows, menu items, tabs and window controls | `max(<size>, var(--space-target-min))` in `components.css` | 57 |
| B4 | Lists reflow: a list narrower than 440 px stacks each row's cells; menus fit a 320 px viewport | `@container (max-width: 440px)` on `.ds-list`; `.ds-menu` `min-width: min(300px, calc(100vw - 32px))` | 7 (partial, §14) |
| B5 | Components tolerate the WCAG 1.4.12 spacing values; the wider-text axis applies exactly those values | `[data-text-spacing="wider"]`; wider row columns | 34 (partial, §14) |
| B6 | Reading surfaces cap the measure at 80 characters and align to the start | `.ds-t-reading { max-width: 58ch; text-align: start }`: in Inter `1ch` is the width of "0", so the measure was counted per rendered line: 58ch holds 70–78 characters (80ch held 100–108) | 36 |
| B7 | No stripe, grid or dot texture is used as a fill | the ds-1.0 reduced-motion stripes are removed | 21 |
| B8 | Nothing flashes more than three times per second above the WCAG general and red flash thresholds (WCAG 2.3.1; ITU-R BT.1702-3 Guideline 1); every system loop is finite and ends within 5 s, the chosen way to meet WCAG 2.2.2 without a pause control (only auto-starting moving content that lasts more than 5 s beside other content needs a pause, stop or hide mechanism; other safe animation is not restricted); a compliant flashing sequence longer than 5 s is reviewed as an additional risk consideration (ITU-R BT.1702-3 Guideline 1 remark), not failed as a hard limit | §12 durations and iteration counts; the gallery `?measure=1` loop sweep; the settings panel and its preview change at most once per 400 ms, so a held arrow key makes at most 3 changes, 1.5 flash pairs, in any second (measured with a held arrow key at 33 Hz: at most 3 panel-background transitions, one flash pair, in any 1-s window; this evidence covers the settings panel only). A held Enter or Space activates a control once per physical key press (§15, held key; run 2 measured 28 open/close changes per second on the delete dialog) | 24, 25, 31 |
| B9 | Every motion has a static form that carries the same information | §12 reduced motion | 32 |
| B10 | Device preferences apply automatically: colour scheme, contrast, reduced transparency, reduced motion; forced colours keep place-node shapes | contrast, transparency, motion and forced colours: media blocks in `tokens.css` and `components.css`; colour scheme: script, because a theme is a whole token set (gallery `deviceTheme()` in `gallery.js`; natively the settings store) (§21.3) | 86 (partial) |
| B11 | Colour never carries meaning alone: a place is colour, node shape, icon and label | §5, §10 place badge | 1 |
| B12 | Settings are independent axes with one place, a reset per axis, a reset for all and an undo | §21.5 | 88 (partial) |

### 21.2 Tier 2: the axes

Each axis is one attribute. Dutch labels describe what the user perceives ("Rood lijkt donker", "Rood en groen lijken op elkaar", "Blauw en geel lijken op elkaar"); the technical name (protan, deutan, tritan) is in the hint. An absent attribute means "follow the device, else
the design default"; any explicit value is the user's deliberate choice and
wins over the device. Values are defined in all three themes unless the
"Owns" column says otherwise. One owner per token (research §7.2), with one
documented exception: the light-intensity compensation block also writes
text, line and internet-place roles that contrast and colour perception own.
It precedes their blocks with the same specificity, so where both are set the
owner's value wins, and that value holds the floors at × 0.85 itself
(`check.mjs` (6) and self-test mutations R1 and R2).

<!-- axes:start -->
| Axis (Dutch label) | Attribute and non-default values | Default | Device preference | Owns | Matrix rows | Status |
|---|---|---|---|---|---|---|
| Theme, polarity (Thema) | `data-theme="dark"`, `data-theme="light"`, `data-theme="contrast"` | dark | `prefers-color-scheme` (gallery: `deviceTheme()` in `gallery.js`) | every themed token | 10 | works in the prototype |
| Contrast (Contrast) | `data-contrast="increased"` | standard | `prefers-contrast: more` | text and line tokens: `--color-text*`, `--color-line*`, rim and glass lines, Kern secondary text and line, neutral twins | 9, 23 | works; in the contrast theme, which already meets it, it only thickens the edges to 2 px |
| Light intensity (Lichtsterkte) | `data-light-intensity="dimmed"` | standard | none (no media feature exists) | an output filter `brightness(.85)` (about 30 % less luminance) on the element and on modal dialogs and their `::backdrop` (top layer), applied once and last; plus a compensation block per theme for `--color-text-subtle`, `--color-text-disabled`, `--color-line-strong` and, in dark, `--color-text-muted`, `--place-internet` and its line, placed before the contrast and colour-perception blocks so that those win where set | 19, 23 | partial: one step |
| Transparency (Transparantie) | `data-transparency="opaque"` | translucent | `prefers-reduced-transparency: reduce` | `--material-glass`, `--material-orbit-bg`, `--material-launch`; Orbit backdrop blur | 11, 47 | works; no-op in the contrast theme |
| Colour perception (Kleuren onderscheiden) | `data-colour-perception="protan"`, `data-colour-perception="deutan"`, `data-colour-perception="tritan"` | off | none | `--place-device`, `--place-nas`, `--place-internet`, `--place-internet-line`, Kern warning icon and line (opaque in the contrast theme) | 4 | partial: palettes only, verified by simulation; each option shows its palette as a static sample; no intensity control, no per-user storage |
| Motion (Beweging) | `data-motion="reduce"`, `data-motion="full"` | full | `prefers-reduced-motion: reduce` | every `--motion-duration-*` | 30, 46 | partial: no middle level |
| Stimulus (Prikkels) | `data-stimulus="calm"`, `data-stimulus="rich"` | standard | none | `--color-blade-glow`; in dark and light `--elev-window`, `--elev-window-inactive`, `--elev-orbit`, `--elev-plate`, `--elev-launcher`, `--elev-inspector` (calm: a short contact shadow; rich: deeper depth with a cool halo); never `--elev-trusted` | 47 | partial: glow and depth; no-op in the contrast theme |
| Text spacing (Ruimere tekst) | `data-text-spacing="wider"` | standard | none | `--type-*-line` and letter, word and paragraph spacing | 35 | works |
| Line length (Regellengte) | `data-line-length="short"` | standard (58ch, measured 70–78 characters) | none | `.ds-t-reading` measure (37ch, measured 44–50 characters) | 37 | works |
| Target size (Klikvlakken) | `data-targets="32"`, `data-targets="44"` | 24 | none | `--space-target-min` | 58 | works |
<!-- axes:end -->

The explicit default values (`standard`, `translucent`, the theme the device
would pick) are offered in the settings group so that a user can overrule a
device preference; they need no CSS of their own because every device block is
guarded by `:not([attr], [attr] *)`. Text size 125 % and 150 % is not an axis
yet (§21.8, O-A1).

### 21.3 Device mapping and precedence

Precedence, highest first: (1) an explicit value on the axis, set on `<html>`
or on the element that carries the `data-theme` (placement below); (2) the
device preference; (3) the design default. Selector
pattern: an axis rule is `theme:is([A="v"], [A="v"] *)`, so it applies to the
element and to nested themed elements (a Kern surface in the opposite theme
keeps the axis); a device rule is `@media (feature) { theme:not([A], [A] *) }`,
whose value equals the attribute rule (checked by `check.mjs`). Exceptions:

- The contrast theme already meets increased contrast and opaque materials:
  opaque changes nothing there and increased contrast only thickens edges to
  2 px. It still bundles, as in ds-1.0, the
  removal of translucency, glow, the halo and the entrance rises; that bundle
  is a documented exception to independent axes (O-A7).
- `data-motion="full"` overrules `prefers-reduced-motion` and restores every
  duration; `data-motion="reduce"` sets every duration to 0 ms (§12).
- Light intensity is applied last, as an output filter over everything below
  it, including Kern and modal dialogs, which render in the top layer and get
  the filter once (pixel sample: 0.85 of the undimmed surface). The modal's
  `::backdrop` is a separate top-layer box and gets the same rule (run 3; before
  it, opening a modal under dimmed brightened the light page from 194,198,206 to
  205,211,218; now the composited scrim is 192,197,204 against the expected
  192,197,205, and the computed `::backdrop` filter is `brightness(0.85)` in all
  three themes). The gallery has no popover; its menus, tooltips and toasts are
  in flow and inherit the root filter. No dialog opener sits under a non-root
  light-intensity scope (the settings preview and the combination specimens hold
  none, run 3 probe `i3-scope.mjs`), so a modal follows the root value. Its token
  compensation block precedes the contrast and colour-perception blocks with
  the same specificity, so an explicit or device increased contrast and a
  re-mapped palette keep their own (stronger) values under dimmed light.
- In the contrast theme, Transparantie and Prikkels change nothing and
  Contrast only thickens edges. The settings group then shows a note in each
  of those groups
  (`.gal-inert`, linked with `aria-describedby`), so no choice looks effective
  while it is inert.
- Nested limit: an axis set on an ancestor cannot be switched back to the
  device or default value inside a subtree, because the ancestor rule also
  matches descendants. The product sets axes once, on `<html>`; the gallery
  sets them on specimens. Upgrade path: `@container style()` queries (O-A8).
- Placement (run-2 repair 2): an axis attribute sits on `<html>` or on the
  same element as a `data-theme`. The axis rules key on the themed element, so
  an axis on an unthemed element below a themed ancestor is unreliable (run-2
  verifier): on `<div data-theme="light"><div data-contrast="increased">` the
  inner element keeps `--color-text-muted` #424B62, so the axis silently does
  nothing, and an unthemed `data-light-intensity="dimmed"` inside a dark panel
  applies the `brightness(.85)` filter without the compensation tokens
  (`--color-text-subtle` #9BA2BC instead of #ACB2C7; ratios still 5.84, 5.38
  and 5.00:1 on bg, surface and raised). The product sets axes on `<html>` and
  the gallery sets them beside `data-theme`, so nothing renders wrong today.
  `check.mjs` does not test nested placement; a check is proposed to the lead
  (the file is protected).
- `forced-colors: active`: the user agent's palette wins; `.ds-node` and its
  ring keep their shapes (`forced-color-adjust: none`) so a place stays
  distinguishable without colour. No full forced-colors design exists (O-A5).

### 21.4 Combinations

All axes combine. The gallery panel "Combinaties" renders a greedy pairwise
covering array over the values listed in `AXES` in `gallery.js` (16 rows,
213 pairs, 0 uncovered), each row in dark, light and contrast (48 specimens).
Since run-2 repair 1 the reading text of each specimen sits in its own strip
across the three specimen columns (`.gal-combo-read`, 48 strips, with the theme
and attributes of their specimen): a specimen column is narrower than the 37ch
short measure, so the line-length pairs rendered no difference there (run-2
verifier: 42-43 characters per line with and without `data-line-length`).
Characters per full line (`build/design-evidence/a11y-ds/run2-repair-1/ll.txt`):
76 standard and 47-48 short at 1440 px and at 720 px (DSF 2); with
`data-text-spacing="wider"` 54-57 and 31-37. At 320 px the strip is narrower
than both measures (33-35 characters), so there the axis has no effect.
`AXES` holds the empty default and every non-default value of the nine attribute
axes. The explicit default values (`data-contrast="standard"`,
`data-transparency="translucent"`, `data-motion="full"`) and the device
preferences are not rows of this panel, so the panel does not cover their pairs. `check.mjs` evaluates the contrast pairs for every
combination of the five colour axes in the three themes, plus the device-mapped
cases. The gallery's `?measure=1` run reports the cascade check: the Kern
surface inside a light specimen with increased contrast resolves to the dark
theme with `--color-text` #FFFFFF.

### 21.5 Settings place, reset and undo

The settings group lives at **Instellingen › Toegankelijkheid**
(`docs/design/NAMING-AND-IA.md`; gallery panel "Instellingen ›
Toegankelijkheid", `#panel-a11y`). Repair 2 replaced the stack of radio rows:

- **Grouped by what changes.** Four groups with a heading each: Zien (Thema,
  Contrast, Lichtsterkte, Transparantie, Kleuren onderscheiden), Beweging en
  prikkels (Beweging, Prikkels), Lezen (Ruimere tekst, Regellengte) and
  Bedienen (Klikvlakken).
- **One segmented choice per axis** (`.ds-seg`, §10): native radios, so each
  axis stays one fieldset, one Tab stop and arrow-key selection. Every tile
  carries a static sample of its own value, rendered with the real axis
  attribute in the current theme (for example the blade with and without its
  glow for Prikkels, a word in wider spacing for Ruimere tekst, the hit-area
  outline for Klikvlakken, the three place nodes in each palette for Kleuren
  onderscheiden). The samples are `aria-hidden` and do not move. Under each
  choice one line says what it does and its status in the words of §21.7
  ("Werkt in dit prototype." or "Deels in dit prototype: …").
- **Actief.** A strip at the top lists one chip per axis that is not on its
  first value ("Contrast: Verhoogd ×"); a chip returns that axis alone,
  announces it in the `role="status"` region and moves focus to the next chip
  (or to "Standaard herstellen" when none is left). With nothing changed the
  strip says so in words.
- **Standaard herstellen** returns every axis to its default (device
  preferences apply again), announces this and shows "Ongedaan maken", which
  restores the previous values, hides itself and returns focus to "Standaard
  herstellen".
- **The group follows the theme.** The chosen Thema applies to the settings
  surface itself and to the preview; the other axes apply to the preview,
  because an axis set on an ancestor would also override the samples (§21.3,
  nested limit). `?theme=` preselects Thema.

Keyboard: one Tab stop per group; arrow keys move the selection immediately,
while the settings surface and preview follow at most once per 400 ms (B8: a
held arrow key auto-repeats at about 33 Hz and would otherwise cycle dark,
light and contrast at 7.4 flash pairs per second); Space and Enter activate
chips, reset and undo. Checked with real key presses in dark, light and
contrast: one stop per group, a 2 px ring on the focused tile, chips appear
and leave with focus kept in the strip. "Volg apparaat" follows a change of
`prefers-color-scheme` while the page is open. The preview is illustrative and
`inert`. The group lists, in plain text, what is feasible natively but not in
this prototype.

### 21.6 Reworked states (carried major and minor)

- **Empty, loading, error** (`.ds-empty`): one Orbitaal frame. The collection's
  place is a node on a 2 px orbit ring around the place icon, drawn larger than
  a list node and trailed by a tapered 110° tail (3 px) in its place colour (the
  blade signature on the orbit), and the frame is crowned by a light blade. Ring
  and tail were strengthened in run-2 repair 1 because the run-2 verifier found
  them faint at 100 % (AI design judgement; the icon-left composition stays). Loading
  (`data-state="busy"`): node and arc orbit three times (4.2 s) and rest at
  the top; the title is visible, says what loads from where and is the
  `role="status"` region, outside any `aria-busy` container so that it is not
  suppressed. After the loop a caption "Nog bezig · m:ss" keeps a long
  operation visibly alive in 5-s steps, in the moving and the static form
  alike; it is text, not motion, and not a live region. Error: the ring breaks
  once at the top (one gap, no dashed texture, row 21), the hollow node sits in
  the gap, the title carries the square error marker and the text says where
  the work is safe. The static form shows the same frame with the node at rest.
  Below 320 px of inline size (container query) the orbit stacks above the
  text, place badges and buttons wrap between words in boxes that grow with
  them, and skeleton rows take the stacked `.ds-row` form.
   In the contrast theme each skeleton line is a 1 px outline: a vertical scan
   through the three rows crosses six light-dark pairs at 21:1, one more than
   row 21 allows for a texture. The skeleton passes only through row 21's
   clause that exempts component structure: the outlines stand in for the list
   rows that will load, but they appear only in the loader, so counting them as
   list-row structure is a design judgement (AI), not a measured result. In
   every theme the column of the three stacked place nodes (one per placeholder
   row) crosses six to nine pairs over about 8 CSS px, nine across its central
   4 CSS px; it relies on the same clause. The dark and
   light fills stay at 1.35:1 and 1.20:1, below the 1.5:1 bound, and the
   progress tracks give three to four pairs (run-2 repair-1 recount at DSF 2,
   rest and reduced motion alike: `run2-repair-1/tex.mjs`, `texcount.py`,
   `tex-count.txt`). Exempt counts, recorded so that a later reviewer can see
   why they pass (run-2 verifier, `run2-verify-1/tex.py` on its DSF-2
   captures, rest and reduced motion alike; re-run unchanged in
   `run2-repair-2/tex-verify1-rerun.txt`): the skeleton node column gives 9
   vertical pairs in every theme (largest ratio 6.85:1 dark, 5.52:1 light,
   11.58:1 contrast); the contrast-theme skeleton outlines give 6 pairs on a
   vertical scan and up to 9 on a horizontal scan along one placeholder row
   (21:1), the dark and light skeletons 3 horizontal pairs; the orbit frame
   (ring, place icon and satellite node) gives 9 vertical and 6 horizontal
   pairs, up to 8.53:1 dark, 8.47:1 light and 21:1 contrast, in the rest, busy
   and error frames alike; the indeterminate progress gives 2 to 3 pairs. The
   skeleton fills measure 1.35:1 dark, 1.26:1 dark dimmed and 1.20:1 light. The
   node column, the outlines and the orbit frame are exempt as icon or
   component structure, not as textures (none of them is a periodic fill);
   whether that reading of the clause holds is settled under O-A22. Measured with Range
  rects of every label against its box, on both axes: 0 labels outside their
  box in the gallery's `[data-scale="2"]` strip (`?measure=1`,
  `scaleTextFit`), and 0 in the states, privacy and settings specimens at 720
  and 640 CSS px (device scale 2) and in a 680 px host with and without
  `data-text-spacing="wider"`, in dark, light and contrast. A single word
  wider than its line still breaks inside the word rather than spill; the
  gallery grid now places these frames side by side only from 300 px each. Class names are kept
  (`.ds-empty`, `.ds-skeleton`); sub-classes `.ds-empty-orbit`,
  `.ds-empty-sat`, `.ds-empty-place` are new.
- **Progress** (`.ds-progress`): the fill ends in the light blade, the
  signature of the active window; the label carries the place node.
  Indeterminate: a tapered blade segment sweeps three times (4.2 s) and rests
  in the centre; the word "Bezig" carries the state and becomes "Nog bezig ·
  m:ss" in 5-s steps (`data-elapsed`). No stripes. The track is
  10 px; in the light theme the fill is `--color-text`, because the muted fill
  read weakly on white (AI design judgement). Since run 2 that rule sits in
  `@scope ([data-theme="light"]) to ([data-theme]:not([data-theme="light"]))`,
  so a dark or contrast subtree inside a light one keeps the muted fill of its
  own theme (rendered with a light root: dark #A9B2CB, light #111726, contrast
  #FFFFFF). `@scope` needs Chromium 118 or Safari 17.4; a browser without it
  shows the muted ds-1.0 fill in light too.

These are design proposals; their effect on users has not been studied.

### 21.7 Status per adaptation

Consistent with the matrix "Prototype status" column. Works in the prototype:
contrast levels (row 9), polarity following the device (10), opaque materials
(11), finite loops (31),
static motion forms (32), wider text (35), reading measure and line length
(36, 37), target size axis (58), the combinable contrast and light axes (23),
no texture fills (21; the contrast-theme skeleton relies on the structure
clause, §21.6), upright Inter in weights 400 and 600 (40), 24 px minimum
targets (57). Partial: colour-perception palettes (4, palettes only, simulation), light
intensity (19, one step), graded motion (30, 46, no middle level), visual
intensity (47, glow and depth), reflow (7) and
text spacing (34), device following (86, forced colours only for nodes),
independent settings (88, contrast-theme bundle), flash limits (24, 25; boot
sequence and wallpaper not analysed), focus in every combination (61, focus
obscuring untested), text size (13, only the 200 % page zoom of
`[data-scale="2"]`). Not in the prototype: text size 125 % and 150 % (13),
screen reader, magnifier, read-aloud, captions, keyboard filters, switch, eye,
head and voice control, a line-focus ruler, numbers with visual aids,
notification bundling, a compositor colour filter (5) and a compositor flash
filter (26).

### 21.8 Extensions recorded, with cost

| ID | Extension | Why not in ds-1.1 | Cost estimate |
|---|---|---|---|
| E1 | Text size 125 % and 150 % (row 13) | Needs type-atlas records per role at two more scales; `docs/design/TYPOGRAPHY.md` is outside this task's write set | More atlas records against the 512 KiB read-only segment of `ui/native` (atlas size estimates in TYPOGRAPHY.md §9, settings in §8.6); a typography contract change |
| E2 | Motion middle level "reduced" (cross-fades kept, rows 30 and 46) | Needs a third value and a cross-fade form per component | One token block, about ten component rules, checker update |
| E3 | Deeper light-intensity steps (row 19) | A uniform filter below 0.85 breaks the contrast floors (0.8 already fails) | A per-theme dimmed token family: about 90 values per theme plus pair checks |
| E4 | Compositor colour filter for third-party content (row 5) | Needs the native compositor | Native compositor work; no browser equivalent |
| E5 | Compositor flash filter (row 26) | Research question | Research plus compositor work |
| E6 | Native assistive technology: screen reader, magnifier, read-aloud, captions, keyboard filters, switch, eye, head and voice control | Native only | Per feature a native subsystem with its own ADR: an accessibility tree in the shell, toolkit and SDK plus a trusted AT role (screen reader); compositor scaling (magnifier); an on-device speech model and audio path (read-aloud, captions); filter stages in the input service (keyboard filters, switch scanning); on-device recognition (voice) |
| E7 | Native mapping of the axes to `ui/graphics` palettes | Native code is outside this task | One palette variant per axis value (§17) |

### 21.9 Not verified

- Run 3 (owner-approved bounded run, 18 November 2026) fixed the three run-2
  final-verifier defects, each as a class; the writer measured them, independent
  verification is pending. Evidence, scripts and renders are in
  `build/design-evidence/a11y-ds/run3-writer/`:
  - B8, held key: a held Enter on the delete-dialog opener opened and closed the
    modal 56 times in 2 s (28 per second; the run-2 verifier measured 43.8 % of a
    1024 x 768 screen in light, not re-measured); the menu trigger, the toast close/show pair and every switch
    flipped at 33 per second. Now one activation per physical key press (§15):
    exactly 1 change per 2 s hold of Enter or Space on each of these openers
    and on the motion switch, in dark, light and contrast, standard and dimmed
    with reduced motion (76 cases, `i1-hold-before.json`, `i1-hold-after.json`),
    and at 200 % in light (`i1i3-extra-after.json`); separate presses still act
    every time (`i1-taps-after.log`); the settings chips and Standaard herstellen
    change no more under a held Enter than under one press (the undo button,
    shown only after a reset, was not held).
  - B1, captions in a shadow: no gallery caption or note (`.gal-note`,
    `.gal-card > h3`) sits inside a specimen's shadow. Each elevated specimen
    keeps its shadow's reach free (offset + spread + blur radius, past which the
    Gaussian is under 2.3 % of the shadow alpha; gallery `<style>` in
    `index.html`): the ask row 30 px above and 98 px below, the Orbit stage
    64 px of bottom padding, the window and toast rows 28 px above (10 calm,
    64 rich). Modelled shadow alpha over every caption and note line: at most
    0.013 in 15 configurations, 200 % (`data-scale="2"`) and a 720 px layout
    included, against 0.39 before (`i2-shadow-*.json`). Rendered contrast of
    every text in every panel, the method of the run-2 verifier with bands of at
    most 4000 px: 0 failures in 19 configurations (axes, device increased
    contrast, 200 %, 720 px); the caption that measured 3.47:1 (light panel),
    3.09 (dimmed), 5.93 and 4.79 (increased, increased plus dimmed) now
    measures, first and second line of the light-panel caption, 8.40 / 8.47,
    7.07 / 7.13, 14.36 / 14.48 and 10.95 / 11.04; its dark-panel instance
    measures 8.45, 7.58, 14.32 and 10.54 (re-measured after repair 2 with the
    run-3 verifier's caption method, `run3-repair-2/cap-a.log`). The before sweep
    (`i2-contrast-before.json`) took each panel as one tall clip; its 303 and 14
    failures near 1.1:1 under 200 % plus increased plus dimmed and 720 px plus rich
    plus increased plus dimmed are blank-capture artefacts, not defects. The
    Orbit stage padding and the room above the window and toast rows go beyond
    the reported caption; they follow the same measured rule and are reasoned
    design judgement that the lead may trim.
  - Repair 1 (writer-measured, not yet independently verified; evidence in
    `build/design-evidence/a11y-ds/run3-repair-1/`): the run-3 verifier found
    the same class at the elevation reference under rich. Its tiles' shadows
    reached the next panel's heading and subtitle (`.gal-panel > h2`, its
    `<small>`) and the swatch value captions above them (`.gal-sw figcaption`).
    These count as gallery text. The reference (`.gal-elev`) now keeps its
    tiles' reach inside its own padding and clips there (`overflow-y: clip`):
    30 px above and 100 below, 64 and 128 under rich, no room in the contrast
    theme. A later token change can therefore only crop the reference; it cannot
    reach that text. Results:
    - The geometric reach check (`reachcheck.mjs`, 12 configurations including
      calm, neutral surface, wider text, compact, increased plus dimmed, 200 %
      and 720 px) finds 0 gallery-text overlaps from the reference, and every
      tile's reach fits its clip box.
    - A repair-1 copy of the run-3 verifier's pixel shadow diff (change of at
      least 3 channel levels, 8 configurations, `shadowdiff-final.log`) reports
      only the tile labels (O-A24). The verifier's own run before the fix
      reported 19 texts under rich (`run3-verify-0/sd-1.log`).
    - The light subtitle measures 7.14:1 under rich (was 6.63) and 9.37 under
      rich plus increased plus dimmed (was 8.70). The caster delta is now 0.
    - The rendered caption contrast sweep gives 0 caption failures in the
      verifier's 24 configurations (`cap-*.log`).
    After repair 1 the class was not closed everywhere. Under rich, the modelled shadow alpha
    (run-3 writer method, `i2-shadow-repair1.json`) is 0.044 over the dark Menu
    card heading and 0.020 over the accessibility panel subtitle, against the
    0.013 yardstick. The first comes from a window in the card above; the run-3
    model wrongly counted it as covered. The second comes from the preview
    window. Disabling the window shadows changes those line boxes by 1 to 2
    channel levels. Contrast stays at 15.97:1 (dark Menu), 17.12:1 (light Menu,
    17.42 without the shadow) and 9.38:1 (`reach-residual.log`). Not fixed in
    repair 1; repair 2 closed them.
  - Repair 2 (19 November 2026, writer-measured, not yet independently
    verified; evidence in `build/design-evidence/a11y-ds/run3-repair-2/`):
    the run-3 verifier's geometric test (reach = offset + spread + blur
    against every caption line box) still found the class. The
    writer's own wider baseline sweep found two more casters (`reachcheck-before.json`).
    Before the fix, these instances were open:
    - the window row in the Vensterrand card over the Menu card heading and its code
      (3 px deep standard, 31 dark and 15 light rich);
    - the settings preview window over the accessibility panel subtitle (rich);
    - the Orbit over the Orbit-item heading and its code (rich, also at 200 % and 720 px);
    - the toast over the Banner heading (rich, 720 px);
    - the open menu over the Melding met acties heading at 44 px targets. Its box
      also ended 14 px past its own card there.

    Each of these specimens now keeps its own reach inside its own card, stage or preview (gallery
    `<style>` in `index.html`):
    - Below a card's last specimen, the reach first crosses at least 50 px of
      gallery air (`--gal-air`: card padding of at least `--space-3` on both
      cards, two 1 px borders, the `--space-6` grid gap). The window row keeps the
      rest of its down-reach: 50 px standard, 0 calm, 78 rich. The toast and the
      open menu keep the rest of theirs (`--elev-window-inactive`): 22, 0 and 38 px.
    - In the gallery, the open menu is in flow under its button (like the
      stage dialog), with the same 300 px width. A taller menu now grows its
      card; at 44 px targets the card grows by 61 px. Closing that menu returns
      the card to its 420 px minimum (`menunarrow-after.log`). At 320, 360 and
      640 px with 200 %, the menu's right edge, the page's scroll width and the
      button's position measure the same as before (`menunarrow-*.log`). The
      run-2 minor on menu fit at 200 % stays open. The gallery's keyboard
      self-test gives the same menu result before and after (`selftest.log`).
    - The Orbit stage has 28 px of top padding under rich.
    - The settings preview has 40 px of top padding under rich. The rule also
      matches the preview itself, because the Prikkels choice sets
      `data-stimulus` there.

    Results:
    - The writer's clipped reach check (`reachcheck.mjs`) finds 0
      gallery-text overlaps in 51 configurations. These cover dark, light and
      contrast roots; the standard, calm and rich stimulus values; 32 and 44 px
      targets; wider text, short lines, increased plus dimmed, opaque, the
      neutral surface and compact density; 200 %, also at 720 px; widths of 360
      to 1440 px around both container breakpoints; and the settings form path
      with the sticky preview scrolled beside the form.
    - The verifier's unclipped test (`reachgeo.mjs`, same configurations)
      finds 0 overlaps except under rich at widths of 1000 to 1200 px. There it
      counts the stage dialog's reach over the neighbouring Voortgang note,
      but the stage clips that shadow (`overflow: hidden`). The pixel delta
      over that note is 0 at 1000, 1100 and 1200 px, before and after this
      repair (`reachpx.log`, `reachpx-1200.log`).
    - Disabling each former caster changes its former caption by 0 channel
      levels, against 0 to 18 before (`reachpx.log`). Before the fix, the Orbit-item heading and its code, the Banner heading
      at 720 px and the light Menu heading under standard were geometric
      instances only (0 levels); the other instances changed by 1 to 18. The light Melding met
      acties heading at 44 px targets plus rich measured 15.11:1 with the menu
      shadow and 17.42 without; it now measures 17.42.
    - The rendered caption sweep (the verifier's method, 16 runs including light
      root, 200 % and 720 px) finds 0 caption failures (`cap-*.log`).
  - Light intensity x top layer: the `::backdrop` rule (§21.3); composited
    scrim in light dimmed 192,197,204 against the expected 192,197,205 (was
    205,211,218, brighter than the closed page); dark and contrast unchanged
    in pixels, computed `::backdrop` filter `brightness(0.85)` under dimmed in
    all three themes and at 200 % (`i3-backdrop-*.json`, `i1i3-extra-after.json`).
  Text inside specimens (not captions or notes) can still sit in a
  neighbouring specimen's shadow fringe; it passes the rendered sweep (O-A24).
- No user has tried any axis, the settings group or the reworked states.
- Colour-perception palettes are verified by simulation only (Machado et al.
  2009, severity 1.0, CIE76 ΔE at least 40 between the three places, at least
  3:1 on every surface); no person with a colour-vision difference judged them.
- Light intensity is modelled as sRGB channels × 0.85; display luminance and
  comfort are not measured. Rendered pixels can differ from the quantised
  model. The tightest computed margin that `check.mjs` prints is 3.01:1 (dark
  `--place-nas`, neutral surface, tritan, dimmed); rendered, that stack
  measured 3.44:1 and the tightest rendered case was 3.03:1 (run-1 verifier).
- Under `data-surface="neutral"`, the dimmed and increased-contrast blocks
  write tinted text and line values over the neutral twins (contrast holds;
  the D3 achromatic rule does not) (O-A18).
- The button spinner (`aria-busy` on `.ds-btn`) has no elapsed-time cue.
- The flash measurement samples the preview background and its attributes per
  frame in headless Chromium; the filter of Lichtsterkte is bounded by the
  change count, not sampled in pixels.
- Run-2 layout sweep (`build/design-evidence/a11y-ds/run2-writer/sweep.mjs`,
  table `sweep.json`, classes `final-classes.txt`): every gallery panel forced
  to dark, light and contrast × `data-text-spacing` default and wider ×
  `data-targets` default, 32, 44 × `data-line-length` default and short ×
  `data-contrast` default and increased (24 combinations) × scale 1 at 1440,
  720 (DSF 2, 200 % of 1440), 640 and 320 px × `data-scale="2"` at 1440, 720,
  640 and 320 px (576 configurations; reduced motion emulated over CDP so loops
  are at rest; browser cache disabled). In the 288 configurations at scale 1
  on 1440, 720 or 640 px and at `data-scale="2"` on 1440 px, wider text
  spacing included, the page does not overflow and no element box or text
  passes its parent, the viewport or a clipping container by more than 1 CSS
  px; the run-1 major (`.ds-tabs`, +95 px and +48 px) is fixed. The pre-fix
  baseline is a 96-configuration subset (`sweep-base-quick-summary.json`).
- At 320 CSS px (scale 1 at 320 px, and `data-scale="2"` at 640 px) the page
  still overflows by up to 147 CSS px in every configuration, and by up to
  111 CSS px at 360 CSS px (`data-scale="2"` at 720 px). Worst horizontal
  overflow per element at 320 CSS px: the list header 178.8 px (ds-1.0, fixed
  132/64/112 px columns; the header is `aria-hidden` and the rows stack); the
  privacy track 128.8 px (ds-1.0, three 120 px columns). Run 2 also left, in
  the compact density specimen, the row date text 41.6 px and a place pill
  text 34.2 px past their boxes, a switch label 29.3 px, a radio label 29.0 px,
  the field 7.3 px and, with `data-text-spacing="wider"`, the button label
  "Vergelijken" 2.0 px; the run-2 verifier added the asks card's opener
  buttons (outside `.ds-ask`): "Vraag voor één handeling openen" 6.3 px (39.0
  px wider spacing) and "Doorlopende vraag openen" 6.8 px (wider spacing) past
  their 257 px box, the same at 640 px with `data-scale="2"`. Run-2 repair 2
  removed both causes: the gallery rule that stacks `.gal-2` preceded the
  `.gal-2` rule and never applied, so those specimens stayed two 117 px
  columns at 320 px (it now follows it and stacks below 480 px of grid width),
  and `.ds-btn` wraps a label that does not fit (Button, O-A19). After the
  repair no control label passes its box at 320 CSS px outside the UX-C
  specimens (`run2-repair-2/lbl.mjs`, `lbl-after4.json`; 30 configurations:
  dark, light, contrast × both spacings × 1440, 720 at DSF 2, 640, 320 and
  `data-scale="2"` on 640 px).
  In the Instellingen preview at 640 px (scale 1), `.ds-row-name` cuts
  "Vakantiefoto's Lissabon" (168 px in 86 px) and "Boodschappen" (106 in 86)
  with an ellipsis and no `title`; this is the ds-1.0 truncation of the row
  name in an inert illustrative preview, and the names wrap under wider
  spacing (O-A4). `.ds-ask` (137.1 px) and
  `.ds-grant` (38.9 px) belong to task UX-C and were not changed.
  `data-scale="2"` at 320 px (160 CSS px, below the WCAG 1.4.10 width of
  320 CSS px) overflows by up to 596 px of the unzoomed viewport, which is
  298 CSS px at zoom 2 (the other figures here are CSS px), and is recorded in the sweep only
  (O-A4, O-A17).
- `forced-colors` was only emulated in headless Chromium; no Windows high
  contrast mode or other platform was used.
- The tab list wraps since run 2 and fits at 320 px in all three themes
  (`run2-writer/tabs.json`, 18 cases).
- Tooltips (run-2 repair 1, `build/design-evidence/a11y-ds/run2-repair-1/tips.mjs`,
  `tips-base.json`, `tips-after.json`): every `.ds-tip` in the gallery forced
  open (`data-state="open"`, the state hover and focus show), all panels in
  dark, light and contrast, text spacing default and wider, at the sweep's
  eight viewports (48 configurations, 9 tips each). Before: 207 tip boxes
  outside the viewport, the Versies tip from −31.4 CSS px (1440 px) to −94.6
  (320 px, wider) and the Sluiten tip up to 11.3 px past the right edge (320
  px, wider). After: 0 of 378 tip boxes outside the viewport or a clipping
  container at 320 CSS px or wider; the Versies tip starts at 31.4 CSS px at
  320 px and wraps to two lines (240 px) under wider spacing. At
  `data-scale="2"` on 320 px (160 CSS px) 36 of 54 tip boxes are outside, as
  is the page (O-A17). Real hover was rendered at 1440 and 320 px
  (`run2-repair-1/shots/after-tips-*-hover.png`).
- The gallery's `?measure=1` focus block printed `fv=false none` for every
  kind in headless runs, because a page without system focus never matches
  `:focus-visible`; its `ring:base` figures were the text colour against the
  base, not a ring. Since run-2 repair 1 it draws the ring through
  `[data-demo~="focus"]` when `:focus-visible` does not match, measures inset
  rings against the element's own fill and prints "no ring rendered" when no
  outline is drawn: all 87 measured kind-and-host cases draw a ring, minimum
  4.00:1 (light `.gal-canvas`), primary button 15.82, 17.42 and 16.35:1 in
  dark, light and contrast (`run2-repair-1/gallery-measure.json`). These are
  computed from styles, not pixel-sampled.
- Re-sweep after run-2 repair 1 (`run2-repair-1/sweep.mjs`, unchanged copy of
  the run-2 sweep; `sweep-final-r1-summary.json`, `classes-final-r1.txt`): 576
  configurations; against the run-2 writer's `sweep-final4-summary.json` no hit
  class is new and no worst value grew by more than 1 px; 6 classes are gone
  (the old combination reading text at `data-scale="2"` on 320 px); page
  overflow is unchanged (288 configurations, all at 360 CSS px or narrower).
  In the 288 configurations at 1440, 720 and 640 px and at `data-scale="2"` on
  1440 px there is no page overflow and no viewport, text or clipped-text hit.
  The sweep, like the run-2 sweep, skips elements that are hidden until shown:
  the tooltips now have their own probe, but closed menus, dialogs and toasts
  were not opened at 320 px with wider spacing (not verified). Since run-2
  repair 2 the delete dialog is opened by `run2-repair-2/lbl2.mjs` (see the
  re-sweep below); menus remain not verified.
- Re-sweep after run-2 repair 2 (`run2-repair-2/sweep.mjs`, an unchanged copy;
  `sweep-final-r2-summary.json`, `classes-final-r2.txt`; plus an unchanged copy
  of the run-2 verifier's probe, `vsweep.mjs`, `vsweep.txt`): 576
  configurations; against `sweep-final-r1-summary.json` no hit class is new
  and no worst value grew by more than 1 px; 69 hit class strings at 320 CSS px
  or wider are gone. Those strings include coordinates: without them they are
  51, and with the three themed panels merged 16 classes, one of which (the
  compact row-date number box) was also gone at 360 CSS px; the others are at
  320 CSS px (the switch, radio and check labels, the compact row dates, place
  pills and names, the field, "Vergelijken" and both asks openers). Page overflow is
  unchanged: 288 configurations, all at 360 CSS px or narrower, 99 and 147 CSS
  px at 320 px. Button heights (`lbl.mjs`, `hdiff.py`): with standard spacing
  none of the 281 `.ds-btn` changes height at 1440, 720 or 640 px; with wider
  spacing the compact buttons measure 24.5 px instead of 24 px (a 22.5 px line
  plus a 1 px edge; before, the line overflowed its box by 0.25 px) and
  "Verplaatsen naar collectie" in the empty state wraps to two lines at 1440
  px (its single line was about 0.5 px wider than the box); at 320 CSS px the
  opener labels wrap (`run2-repair-2/shots/askrow-320-*.png`). A first
  threshold of 600 px stacked `.gal-2` at 640 px too and left the compact list
  a 460 px row with a 52 px name (4 px with wider spacing); it was rejected for
  480 px, which keeps the two columns at 640 px and wider. Moving the 900 px
  block also revived its `.gal-type > div` rule: below 900 px of grid width the
  Typeschaal rows stack the token name above its sample (one 526 px column at
  640 px, one 598 px column at 1440 px with `data-scale="2"`, no element
  scrolling; `run2-repair-2/typeshot.mjs`, `shots/type-640.png`).
- Follow-up probe after run-2 repair 2 (`run2-repair-2/lbl2.mjs`, `hagg.py`;
  1440, 640, 360 and 320 px and `data-scale="2"` on 640 and 320 px; the before
  run swaps in `components.before.css` after load, `index.html` is the current
  one in both runs; axis attributes on `<html>` and on every `data-theme`
  element). With `data-contrast="increased"` the compact secondary buttons
  "Versies" and "Delen" measure 26 px instead of 24 px (a 22 px line plus two
  2 px edges; before, the fixed height let the line pass its content box by
  1 px at each edge) and 26.5 px with wider spacing. With `data-targets="44"`
  only the wrapping labels change height. At 160 CSS px (`data-scale="2"` on
  320 px) "Thema: Donker" and "Thema: Licht" in the settings group, "Standaard
  herstellen" and, with wider spacing, the toast's "Ongedaan maken" wrap (the
  theme chips 64 to 108 px of the unzoomed viewport, 32 to 54 CSS px). The
  delete dialog, opened in each view with standard and wider spacing, has no
  label past its button and no button past its actions row, before or after;
  at 320 px with wider spacing its actions stack at full width and
  "Definitief verwijderen" wraps to two lines
  (`shots/dialog-now-none-320-wider.png`).
- The wallpaper and the boot sequence were not analysed for flashes or
  patterns.
- The reset announcement, the chips and the settings group were not tried with
  a screen reader; whether the tile samples help people choose is untested.
- The Range-rect harness covers the theme panels, the states, privacy and
  settings specimens and the 200 % strip; the `.ds-ask` actions (task UX-C)
  keep a one-line label (`.ds-ask .ds-btn`) and can still pass their button
  below 320 CSS px (O-A19).
- Native code was not changed; nothing here holds natively yet.

### 21.10 Sources

- W3C, WCAG 2.2 (5 October 2023): SC 1.4.1, 1.4.3, 1.4.6, 1.4.10, 1.4.11,
  1.4.12, 2.2.2, 2.3.1, 2.4.7, 2.5.5, 2.5.8.
- W3C, Media Queries Level 5: `prefers-color-scheme`, `prefers-contrast`,
  `prefers-reduced-motion`, `prefers-reduced-transparency`, `forced-colors`.
- W3C, CSS Conditional Rules Level 5 and CSS Containment Level 3: container
  queries.
- ITU-R BT.1702-3, Guideline 1 (flash limits; its remark on flashing sequences
  longer than 5 s is an additional risk consideration, not a hard limit).
- G. M. Machado, M. M. Oliveira, L. A. F. Fernandes, "A physiologically-based
  model for simulation of color vision deficiency", IEEE TVCG 15(6), 2009.
- Project: `docs/design/accessibility/ACCESSIBILITY-RESEARCH.md` (§7.2 token
  ownership), `docs/design/accessibility/ACCESSIBILITY-MATRIX.md`,
  `docs/design/NAMING-AND-IA.md`.
