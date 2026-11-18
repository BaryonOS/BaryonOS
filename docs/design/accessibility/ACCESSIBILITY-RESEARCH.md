# BaryonOS accessibility research and two-tier system proposal (UX-A11Y-R)

Date: 15 November 2026 (Europe/Amsterdam). Branch `claude/baryon-os-design-pipeline-20261115`.
Status: research synthesis and proposal. No design-system, prototype or implementation file was
changed. Integration into ds-1.x is the next task (UX-A11Y-DS). Nothing here is a user-tested
result, a human accessibility audit or a conformance claim.

Companion matrix: `ACCESSIBILITY-MATRIX.md` (92 need-adaptation rows, 110 sources). Structural
check: `python3 -I docs/verification/a11y-research-20261115/check.py .` (exit 0 on 15 November
2026). Inputs: the perception, cognition and interaction dossiers of three research workers. The
dossiers, the matrix generator and the scripts of repair rounds 1 and 2 are local build artefacts
that are not committed. Appendix A is the committed trace: the map from matrix rows to dossier row
ids, the hashes of the fetched standards, the cited WCAG criteria and EN 301 549 clauses, and the
repair history.

Owner basis: the accessibility decision of 15 November 2026 (message 2 takes precedence) and the
futuristic design ambition of 14 November 2026. In short: accessibility is part of the whole
BaryonOS identity; the regular themes stay the primary, light, futuristic default; essential
access and safety hold without configuration; personal settings are freely combinable variants of
the same identity, never diagnosis profiles and never a lesser OS.

## 1. Method

### 1.1 Questions

For every need the six owner questions were asked:

- **Q1** concrete barriers caused by a typical interface;
- **Q2** adaptations supported by research or recognised standards;
- **Q3** popular solutions with insufficient or contradicted evidence;
- **Q4** preferences that differ between individuals;
- **Q5** settings that serve several groups without needless complexity;
- **Q6** safety limits that are mandatory whatever the settings.

### 1.2 Sources and tools

- **Normative primary texts**, fetched on 15 November 2026:
  - WCAG 2.2 Recommendation of 12 December 2024 (`WCAG22`). The synthesis re-fetched it and parsed
    all 86 success criteria with their levels. The build script checks every `SC n.n.n` list and
    every `n.n.n (level)` token in every matrix cell against that list. SC numbers in this text
    were checked the same way.
  - Understanding documents for SC 2.3.1, 2.3.3 and 2.2.2 (`PER-U231`, `PER-U233`, `W3C-U222`).
  - WCAG 3.0 (`WCAG3`). Its status on the fetch date is a W3C Working Draft of 10 September 2026,
    not ready for conformance claims. It is not used as a normative basis, and no APCA threshold is
    adopted.
  - EN 301 549 V4.1.1 (2026-09) (`EN549`). The synthesis downloaded the free ETSI PDF and recorded
    its sha256. Every cited clause number was checked against its headings.
  - ITU-R BT.1702-3 (`PER-ITU1702`).
  - W3C notes: Making Content Usable (`COG-COGA`), Accessibility Requirements for People with Low
    Vision (`PER-W3CLV`), Natural Language Interface Accessibility User Requirements (`INT-NAUR`),
    WCAG2ICT (`INT-WCAG2ICT`), Media Queries Level 5 (`INT-MQ5`) and Core-AAM 1.2 (`INT-COREAAM`).
- **Paywalled ISO texts were not read in full.** Only their public scope is cited: ISO 9241-391
  (`PER-ISO9241-391`), ISO 9241-171:2025 (`INT-ISO9241-171`) and ISO 24495-1:2023
  (`COG-ISO24495`). IEEE 1789 was resolved by DOI but not read.
- **Peer-reviewed literature** was found by targeted searches in OpenAlex (`search_works`), PubMed
  and Semantic Scholar. Each reference was resolved with `verify_reference` or `lookup_reference`.
  Findings come only from abstracts or conclusions that were actually read; no full texts were
  read. The synthesis independently re-resolved the two highest-graded references,
  `COG-WOOD17` and `COG-GILL11`, by DOI.
- **Platform documentation** (Microsoft UI Automation and UISettings, Apple AppKit and UIKit,
  Android accessibility services, AT-SPI, GNOME help and AccessKit) documents practice only. It
  never raises an empirical grade above insufficient.
- **Prototype and native status** were checked against `ui/design/system/{tokens.css,
  components.css,index.html,gallery.js}`, `ui/graphics/src/theme.rs` and the contract documents
  named in the assignment. The synthesis re-read the cited lines for all three "works" cells and
  for every partial cell it rewrote; the other partial and absent cells are carried over from the
  dossiers. Code structure came from the hosted Graphify index
  (interaction cluster) or from literal reads.

### 1.3 Inclusion, grading and tiers

- **Inclusion.**
  - Normative clauses that apply to an operating-system user interface (WCAG read through
    WCAG2ICT; EN 301 549 clauses 4, 5 and 11).
  - Peer-reviewed studies with a readable abstract that test an adaptation or document a barrier.
  - Recognised-organisation consensus: W3C notes, and the WFD and WASLI statement.
  - Excluded: sources with no readable finding. The dossiers dropped three for that reason and
    corrected three DOIs that had been typed from memory.
- **Empirical scale** (fixed): strong, moderate, limited, insufficient, contradicted. The grade
  rates evidence that the *adaptation* helps the *named population*; evidence that only shows the
  barrier does not raise it.
- **Synthesis rules applied consistently across clusters:**
  1. A row whose sources are only normative texts, platform documentation or project contracts is
     graded insufficient. This changed c15 and c28; c14 was merged into an already insufficient
     row.
  2. W3C user-needs notes (COGA, the low-vision note `PER-W3CLV` and NAUR) count as
     recognised-organisation consensus. Where such a note recommends the adaptation, the row is
     limited. This raised p9, p18 and i26 from insufficient to limited.
  3. A different population makes a grade at most limited for the need in question.
  4. No deviation from the fixed scale. Read-aloud for dyslexia (c4) was kept at moderate as a
     declared conservative deviation until repair run 2; the scale has no deviation mechanism, so
     since repair round 1 of run 2 its core part is graded strong, scoped like the reminder row
     (section 1.4). Rows lowered later because they rest on one small study or on another
     population are listed in section 1.4.
  5. **Basis rule** (repair round 2, applied to every row graded limited or above). A grade of
     limited or above needs at least one cited source that (a) tests the adaptation, or the
     variable it manipulates, in any population or context, or (b) is recognised-organisation
     consensus that recommends it: the W3C user-needs notes, the WFD and WASLI statement, or
     published expert consensus such as `PER-HARDING05`. WCAG Understanding documents explain
     normative criteria and count as normative basis, not as consensus. A source that documents only
     the barrier, its prevalence or its mechanism never raises a grade. One study counts as moderate
     only when it is well controlled and not small; a single small study is limited. A row that
     combines several adaptations is graded by the evidence for its core part. Where the basis is
     not obvious, the matrix states it as an "Evidence caveat" in the Verification method cell. The
     sweep regraded seven rows to insufficient (accessibility tree, extra-dim, notifications never
     take focus, the 24 px target floor, labelled automatic captions, braille display support,
     trusted assistive-technology capability) and two rows from moderate to limited (section 1.4). It added two resolved sources (`PER-RUBIN89`,
     `PER-FERNANDES16`) and cited `PER-W3CLV`, `COG-WOOD17` and `INT-WFD2018` on rows where they
     apply.
  6. **Contradicted bar** (repair run 2). Contradicted uses the same bar as moderate: a review or
     meta-analysis of adequate studies, or at least one well-controlled study that is not small, in
     the population the claim is about, found no benefit or found harm. A claim stated as universal
     (for everyone) is contradicted by a well-controlled counterexample in a group it covers, and the
     item then names that narrower claim. A single small study, or a study in another population, never makes
     an item contradicted: the item is graded insufficient for its claim and the finding is reported
     as limited context. Grade cells hold one scale word; qualifiers go into the treatment or note
     text.
- **Tiers** (fixed):
  - *Baseline*: every regular theme, no configuration, never optional. Every safety limit is
    baseline.
  - *Setting*: an independently combinable axis. It may start automatically from a named device
    or user preference.

**Basis of the 51 rows graded limited** (matrix row numbers, counted from the first data row):

| Basis class | Rows |
|---|---|
| Tests the adaptation or its variable in the target population (small, single or multi-factor studies) | 6, 34, 35, 36, 37, 40, 41, 47, 48, 58, 59, 66 |
| Tests it in another population or context | 7, 8, 10, 12, 21, 29, 30, 42, 43, 45, 68, 74, 75, 85 |
| Uncontrolled evidence of use or preference in the target population (surveys, case series) | 16, 17, 70, 72, 78, 79 |
| Recognised-organisation consensus that recommends the adaptation | 9, 13, 19, 23, 24, 25, 27, 49, 50, 51, 52, 54, 55, 73, 80, 81, 82, 88, 92 |

Rows 3 and 76 were limited until repair round 1 of run 2 and are now insufficient. Row 3's palette
was evaluated only with a colour-vision-deficiency simulation (`PER-NUNEZ18`; the model was
validated with observers who have a colour-vision deficiency, `PER-MACHADO09`), not with people;
row 76's only peer-reviewed source (`INT-BURNHAM2008`) tests caption rate and text reduction, not
caption style. Rows 24 and 25 also cite laboratory and observational work on the flash and pattern guidelines
(`PER-WILKINS05`); row 27 rests on the consensus `PER-HARDING05`. Row 21 rests on a laboratory study that
varied stripe-pattern parameters (`PER-WILKINS84`); its numeric bound is an own comfort design rule
that a user test calibrates before it constrains identity artwork.

### 1.4 Grades above limited, against the fixed scale

| Row | Grade | Reason |
|---|---|---|
| Read-aloud for dyslexia and reading difficulties (`COG-WOOD17`) | strong | A random-effects meta-analysis in the target population (students with reading disabilities) found a small-to-moderate gain in reading comprehension (weighted effect size .35, 95 % CI .14 to .56); study-design moderators explained some variance. Strong applies only to the core part, read-aloud for reading comprehension. The studies concern students and school reading tasks; adults, word highlighting and rate control are not separately evidenced, and the operating-system form still needs a user test, as for the reminder row. Until repair run 2 this row was kept at moderate as a declared deviation, which the fixed scale does not provide for. |
| Reminders and step prompts for acquired brain injury (`COG-GILL11`, `COG-WILS01`) | strong | A systematic review of 91 studies of assistive technology for cognition in clinical populations (descriptive, no pooled effect) and an independent randomised crossover trial (n = 143, mostly traumatic head injury or stroke) of a paging system for everyday memory and planning. Strong applies only to the core part, reminders that prompt a scheduled action, and to this need only, not to ADHD or older adults. User creation on the device, snooze and done, and checked task steps are not tested by the cited studies (insufficient for those parts). The device form differs (pager versus operating system), so the operating-system implementation still needs a user test. |
| Personal speech models for disordered speech (`INT-GREEN2021`) | moderate | One well-controlled study (n = 432). Personalised recognition error was far lower than speaker-independent error. The outcome is recognition accuracy, not usability. |
| Dyslexia fonts (`COG-AZZ26`, `COG-KUSTER17`, `COG-WERY16`) | contradicted | A 2026 meta-analysis of 15 studies found a negligible effect. Controlled studies of Dyslexie and OpenDyslexic found no gain. |

Repair round 1 regraded upright sans-serif with weight emphasis (`COG-RELLO13`) from moderate to limited. The
support is one within-subject eye-tracking study (48 readers with dyslexia) that has not been replicated and that
compared font styles (sans serif and roman read faster than serif and italic), not emphasis; the fastest font
differs between individuals (`COG-WALL22`); and emphasis by weight 600 and the underline rule are own design rules. The caveats for reminders and personal speech models now also appear in
their matrix cells.

Repair round 2 regraded two rows from moderate to limited. Pointer assistance rests on one
controlled study with 16 participants, 8 of them with motor impairments, in which the angle mouse
raised their throughput by 10.3 % over the default mouse (`INT-WOBBROCK2009`); `INT-GAJOS2008`
tested automatically generated layouts rather than pointer gain, and `INT-GROSSMAN2005` had no
participants described as motor-impaired. That is one small study, graded limited like
`COG-RELLO13`. Captions for deaf and hard-of-hearing users rest on a narrative review whose abstract
names deaf and hard-of-hearing viewers among the groups that benefit, but the share of reviewed
studies in that population was not established (`INT-GERNSBACHER2015`), so synthesis rule 3 caps the
grade at limited. The caption tier is unchanged: captions are baseline on normative grounds. Personal
speech models stay moderate because their one study is large and well controlled (n = 432).

Repair run 2 changed no matrix grade. It scoped the strong reminder grade to its core part (above);
added synthesis rule 6; regraded two research-text items from contradicted to insufficient ("all
notifications off" as the attention remedy, whose only study is in the general population, and touch
as easier for motor impairment, whose only study has 16 participants with motor impairments); kept dark
mode contradicted as a universal claim only; graded colour-correcting glasses contradicted for the
narrower claim that they improve colour-vision test results (`PER-GOMEZ18`); reduced every grade cell
in section 4 to one scale word; and gave the stripe-texture row (row 21) an explicit, testable own
bound. The `COG-FITZ19` abstract was re-read through the Semantic Scholar Graph API.

Repair round 1 of run 2 regraded three matrix rows and one research-text item. Read-aloud for
dyslexia (row 38) is strong for its core part (above). CVD-safe chart palettes (row 3) are
insufficient: the only basis is a simulation model, not a study with people. Adjustable caption
style (row 76) is insufficient: `INT-BURNHAM2008` varies caption rate and text reduction, not style,
and now supports the barrier and section 3.14 Q4. Dark mode as a universal legibility fix is
insufficient instead of contradicted: the sample size of `PER-PIEPENBROCK13` is not in the PubMed
abstract, and its full text could not be read (no PubMed Central copy, closed access in OpenAlex,
publisher page HTTP 403 on 15 November 2026), so synthesis rule 6 is not met. The stripe-texture
bound (row 21) is an untagged baseline comfort rule, no longer a safety row. Tiers are unchanged.

### 1.5 Limits of the method

- The search was targeted, not exhaustive. No PRISMA protocol was run, and no study was read in
  full text. Grades may rise or fall with a systematic search; switch scan rates, keyboard
  filters, focus indicators, live regions, accessible authentication and OS-level text-size
  settings are the largest gaps.
- No user test, human audit or screen-reader run was part of this work. AI review, including this
  synthesis, is not evidence of user effect.
- The harmonisation status of EN 301 549 V4.1.1 under the European Accessibility Act was not
  checked.

## 2. Consolidation and resolved contradictions

### 2.1 De-duplication

The dossiers contributed 103 rows; the matrix has 92: 103 − 13 merged + 1 added by the synthesis
(s0) + 1 added in repair round 1 (r1, the vestibular safety limit). Repair round 2 split no rows;
it gave the haptic parts of two rows their own status inside the Native status cell.

- **Twelve merges removed 13 rows:**

  | Kept row | Absorbed rows | Topic |
  |---|---|---|
  | p6 | i5 | Visible focus |
  | p12 | i30 | Accessibility tree |
  | p8 | i34 | Magnifier |
  | p24 | p25 | Motion axis |
  | p26 | c14 | Pause, stop and hide |
  | p17 | c6 | Tint and overlay |
  | p19 | i23 | Flash limit for all system UI, including alerts and scan highlights |
  | p29 | c29, i35 | Preference following |
  | p31 | i36 | Raw-value privacy |
  | p30 | c25 | Settings discoverability and combination |
  | c20 | i16 | Time limits |
  | c22 | i40 | Sign-in |

- **Row ids** used in this document (p0–p31, c0–c29, i0–i40) are the zero-based positions of rows
  in `perception.json`, `cognition.json` and `interaction.json`; s0 is the row added by the
  synthesis, and r1 the vestibular safety row added in repair round 1. The dossiers are not
  committed; Appendix A maps every matrix row number to its ids.
- **One row was added** (s0, text-size axis). Text size existed only as a 200 % baseline row, not
  as a user axis.
- **Duplicate source ids were collapsed:**
  - WCAG 2.2 (three ids) to `WCAG22`;
  - WCAG 3 (three ids) to `WCAG3`;
  - EN 301 549 (four ids, three versions) to `EN549`;
  - Lazar 2007, Szpiro 2016 and Griffiths 2016 (two ids each) to `LAZAR07`, `SZPIRO16` and
    `GRIFFITHS16`.
- **Two internal ids were added:** `W3C-U222` (fetched by the synthesis) and `BOS-SECFLOOR` (the
  project Security Floor). `BOS-SECFLOOR` is type other: it is a project contract, not evidence of
  user effect.

### 2.2 Contradictions and how they were resolved

1. **EN 301 549 version.** The perception cluster cited V3.2.1 and treated V4.1.1 as unverified;
   the cognition cluster read draft V4.1.0; the interaction cluster read V4.1.1.
   - The synthesis downloaded V4.1.1 (sha256 `636914c5…cb824`) and found every cited clause
     heading under the same number. Examples: 4.2.2, 4.2.3, 4.2.9, 4.2.11, 5.2, 5.7–5.9, 7.1.4,
     11.2.3.1, 11.5.2.13, 11.5.2.15, 11.6.2 and 11.7.
   - The notes of clause 4.2.2 were renumbered:
     - magnification and control of contrast, brightness and intensity is now Note 2;
     - depth perception is Note 3;
     - non-visual access for limited vision is Note 7.
   - A new Note 4 gives a best-practice minimum x-height of 8 CSS px for the smallest text.
   - Clause 11.7 now requires software to follow *any documented* platform accessibility feature,
     with an illustrative list in Note 6. The V3.2.1 reading that motion is not listed no longer
     applies: once BaryonOS documents a motion preference, apps must follow it. The older
     11.7 Note 3 (extra app values are allowed if one mode follows the system) is Note 5 in V4.1.1.
   - All matrix citations now refer to V4.1.1.
2. **What the device reduced-motion preference selects.**
   - The perception cluster mapped it to a *reduced* level that keeps small feedback.
   - The cognition cluster and the prototype set every duration to 0 ms.
   - Resolution: `prefers-reduced-motion: reduce` selects *reduced*. Reduced removes every
     interaction-triggered motion animation (translation, scaling, zoom, parallax, scroll-linked
     effects) and turns loops into their designed static forms. It keeps short cross-fades and
     colour changes.
   - Why this level: cross-fades are not motion animation under the WCAG definition, so the
     automatic level already meets SC 2.3.3 (AAA). Animated transitions can aid understanding of
     change (`PER-HEER07`), so fades are kept where they help.
   - *None* (0 ms everywhere) remains one step away.
   - Repair round 1 added a baseline vestibular safety limit for large-field motion (section 3.6 Q6),
     so protection from the most provocative motion no longer depends on a device preference that does
     not exist at first boot.
   - The prototype is therefore *partial*: its single level equals *none* (tokens.css lines
     427–431).
3. **Inconsistent grading of normative and consensus sources.** Some cognition rows treated WCAG
   itself as expert consensus. Under the rule that a normative requirement is never empirical
   proof, those rows are insufficient (c15, c28). In the other direction, W3C user-needs notes
   support limited in every cluster, which raised p9, p18 and i26 (section 1.3).
4. **Tier of the stripe-texture rule (p16).** The JSON said setting; the dossier prose said
   baseline comfort rule. It is baseline. Avoiding high-contrast stripe textures makes nothing
   duller, and their unpleasantness has laboratory support (`PER-WILKINS84`). Repair run 2 gave it an
   explicit, testable own bound for texture fills (more than five light-dark pairs at a contrast ratio
   above 1.5:1 is outside it; borders, separators and grids are not texture fills), so it is not a ban
   on texture motifs; the seizure limit for patterns is row 25. Since repair round 1 of run 2 it is an
   untagged baseline comfort design rule, not a safety limit: its bound is stricter than any seizure
   guideline and rests on discomfort evidence, so the epilepsy reason belongs to row 25 only. The
   1.5:1 ratio is calibrated by user test (section 12, priority 4) before it constrains identity
   artwork such as wallpapers.
5. **SC 2.2.2 in the default theme.**
   - The Understanding document (`W3C-U222`) accepts content that stops by itself within 5 s, as
     well as a pause or stop control.
   - A preload animation can be essential when no interaction is possible and progress must be
     shown (its Note 4).
   - Resolution: the ask halo animates for at most 5 s and then settles to its static double ring.
     Loaders count as essential only in that preload sense.
   - This is a baseline reason (SC 2.2.2, level A), not a stylistic reduction. The default still
     moves when something happens.
6. **"Screen flash" alerts.** The interaction dossier offered a screen-flash alert as a setting
   while its own Q3 named the photosensitivity risk. The option is redefined as a full-screen
   visual alert drawn as a sub-threshold luminance change. The flash limit lives only in the
   baseline photosensitivity row.
7. **Overlapping axes.** The clusters owned some properties twice:
   - transparency appeared under both a transparency axis and stimulus load;
   - tint appeared under both light intensity and a text-background colour choice;
   - the prototype contrast theme also switches polarity, transparency and motion.

   Resolution: every rendered property has exactly one owning axis (section 7). Contrast,
   transparency, motion and polarity become independent.
8. **Prototype status disagreements.**
   - Focus is *partial*. The ring works in all three themes (components.css lines 42–44), but
     focus-not-obscured and shell focus order are untested.
   - The motion setting is *partial*: one level only.
   - The pause, stop and hide rule is *partial*: default loops run indefinitely.
9. **Read-aloud grade.** It is strong for dyslexia for its core part (section 1.4) and limited for low vision,
   where the same meta-analysis (`COG-WOOD17`) counts only as other-population evidence (synthesis
   rule 3). The same axis keeps different grades per need because evidence does not transfer fully
   between populations.

## 3. Findings per need (Q1–Q6)

Source ids refer to the sources table of the matrix. "Own inference" marks reasoning that no source
read here supports directly.

### 3.1 Colour vision differences

- **Q1 Barriers.**
  - Red-green pairs collapse for protan and deutan observers: about 8 % of men and 0.4 % of women
    of European descent (`PER-BIRCH12`).
  - Hue-only states, and rainbow or red-green chart palettes (`PER-NUNEZ18`).
  - Achromatopsia removes hue and adds photophobia (`PER-HIRJI18`).
  - The BaryonOS place triad pairs green (this device) with red (internet) (tokens.css 124–126).
- **Q2 Supported.**
  - Normative: colour is never the only cue (WCAG 2.2 SC 1.4.1 A; SC 1.3.3 A; EN 301 549 clause
    4.2.3).
  - Empirical: a validated CVD simulation model (`PER-MACHADO09`) makes automated greyscale and CVD
    checks credible. Colormaps can be optimised for CVD with such a simulation (`PER-NUNEZ18`), but
    no study tested these palettes with readers who have a CVD, so CVD-safe chart palettes (matrix
    row 3) are insufficient and baseline on normative grounds.
  - No controlled UI study of redundant coding was found, so it is graded insufficient despite a
    firm normative basis.
- **Q3 Insufficient or contradicted.**
  - Colour-correcting glasses: contradicted for the narrower claim that they improve colour-vision
    test results or restore normal colour vision. One controlled study with what its abstract calls
    a high number of observers with CVD (no sample size given there) found neither (`PER-GOMEZ18`).
  - The user benefit of daltonisation was not established (`PER-RIBEIRO19`; insufficient).
  - Neither may be described as restoring colour vision.
- **Q4 Individual differences.** Type (protan, deutan, tritan), severity, and whether expressive
  colour should be kept or remapped.
- **Q5 Settings.** One `colour-perception` axis: a palette remap per type with intensity and
  preview, plus an optional compositor filter for third-party content. The lightness rule also
  serves low vision and greyscale displays.
- **Q6 Limits.** Redundant cues in every theme. Any remap keeps the contrast floors. A toggle never
  produces a flash (photosensitivity row).

### 3.2 Low vision and age-related vision

- **Q1 Barriers.**
  - Muted text and text over glass. Older adults lose contrast sensitivity (`PER-OWSLEY11`).
  - Fixed sizes that clip.
  - Magnifiers that lose focus: 11 low-vision participants needed constant gestures
    (`SZPIRO16`).
  - Glare from large bright surfaces (own inference).
- **Q2 Supported.**
  - SC 1.4.3 (AA, 4.5:1; large text 3:1), 1.4.11 (AA, 3:1), 1.4.4 (AA, 200 %), 1.4.10 (AA,
    320 CSS px reflow), 1.4.6 (AAA, 7:1) and 2.4.7 (AA).
  - EN 301 549 clauses 4.2.2 (Notes 2 and 4), 11.5.2.13 (focus tracking) and 11.7.
  - Empirical support is limited. For contrast, one small laboratory study found that most of 19
    low-vision readers needed higher text contrast than normally sighted readers (`PER-RUBIN89`);
    it does not test the 4.5:1 or 3:1 thresholds, and `PER-OWSLEY11` documents the barrier only.
    For scaling and magnification the evidence comes from devices (the Cochrane review found
    insufficient evidence for any one magnifier type, `PER-VIRGILI18`) and from an observational
    study (`SZPIRO16`). The contrast, text-size and brightness settings rest on the W3C low-vision
    draft (`PER-W3CLV`).
- **Q3 Insufficient or contradicted.**
  - Dark mode as a universal legibility gain: insufficient (section 4). One controlled study of
    normally sighted younger and older adults found positive polarity better for acuity and
    proofreading in both groups (`PER-PIEPENBROCK13`); its sample size could not be established
    (abstract only), so it is limited context against the universal claim, not a contradiction
    (synthesis rule 6). Dark polarity remains a legitimate individual choice: five of 19 low-vision
    readers read white-on-black faster (`PER-RUBIN89`).
  - APCA while WCAG 3 is a draft.
  - Prism and filter spectacles (`PER-VIRGILI18`).
- **Q4 Individual differences.** Polarity, contrast level, text size, magnification mode, and
  speech versus magnification. Low-vision participants preferred visual over aural access
  (`SZPIRO16`). Five of 19 low-vision readers, four of them with cloudy ocular media, read
  white-on-black text faster (`PER-RUBIN89`).
- **Q5 Settings.** Separate `polarity`, `contrast`, `transparency`, `text-size`, `magnifier` and
  `read-aloud` axes. Polarity and contrast start from device preferences.
- **Q6 Limits.** Contrast floors measured over the worst-case backdrop of each glass material,
  visible focus, and 200 % with reflow, in every theme and combination.

### 3.3 Blindness and screen-reader use

- **Q1 Barriers.** Confusing layout feedback, AT conflicts and missing labels were the top
  frustrations of 100 blind users, who lost on average 30.4 % of their time (`LAZAR07`).
- **Q2 Supported.**
  - SC 4.1.2 (A), 1.3.1 (A), 4.1.3 (AA) and 2.4.1 (A).
  - EN 301 549 clauses 11.5.2.1–11.5.2.17.
  - Core-AAM role mappings (`INT-COREAAM`).
  - Empirical evidence for the accessibility tree itself is insufficient: `LAZAR07` documents
    barriers, not a tested design, so the tree is baseline on normative grounds. Heading navigation
    and the system screen reader are limited, on an uncontrolled survey of 1,539 screen-reader users
    (`INT-WEBAIM2024`).
- **Q3 Insufficient.**
  - Landmarks as the main structure: survey respondents start with headings far more often
    (`INT-WEBAIM2024`, limited).
  - Live-region-heavy designs.
- **Q4 Individual differences.** Rate, verbosity, punctuation, earcons, and speech versus braille.
- **Q5 Settings.** One accessibility tree serves the screen reader, braille, voice control, switch
  scanning, the magnifier and automated audits.
- **Q6 Limits.**
  - The screen reader is available from first boot, at sign-in and on the trusted screen.
  - Speech synthesis runs on the device without network permission
    (`docs/LOCAL-PRIVACY-CONTRACT.md`).
  - Secure fields are never read out unless the user's AT policy allows echo.

### 3.4 Light sensitivity, migraine and dimmed display

- **Q1 Barriers.**
  - Photophobia accompanies many disorders (`PER-DIGRE12`), and light worsens migraine headache
    (`PER-NOSEDA16`).
  - High-contrast stripes are unpleasant and induce illusions (`PER-WILKINS84`).
  - The trusted Kern surface takes the opposite theme, so a bright panel appears inside the dark
    theme (tokens.css 128).
- **Q2 Supported.**
  - EN 301 549 clause 4.2.2 Note 2 (control of brightness and intensity). No WCAG criterion covers
    luminance comfort.
  - Empirical support for the light-intensity setting is limited and rests on consensus: the W3C
    low-vision draft lists setting the overall brightness of a display as a user need separate from
    text contrast (`PER-W3CLV` 3.1.1 and 3.1.2), which also supports separate contrast and
    light-intensity axes. Ambient green light worsened migraine headache less than other colours
    (`PER-NOSEDA16`); that study varied colour, not luminance, so it does not test the cap.
- **Q3 Insufficient or weak.**
  - Blue-light spectacles: a Cochrane review of 17 trials found low-certainty evidence of no
    short-term difference in visual fatigue, with most trials at high risk of bias (`PER-SINGH23`);
    insufficient, not contradicted (synthesis rule 6).
  - Coloured overlays cannot be endorsed (`GRIFFITHS16`).
  - Precision tints for migraine are weak (limited): one small double-masked randomised crossover
    trial in 17 patients found a marginal reduction in headache frequency, which the authors call
    suggestive rather than conclusive (`PER-WILKINS02`). BaryonOS offers tint for comfort only, with
    no treatment claim.
- **Q4 Individual differences.** Maximum luminance, polarity, tint, and tolerance of maximum
  contrast (`PER-W3CLV`).
- **Q5 Settings.**
  - `light-intensity` caps maximum luminance in steps.
  - An extra-dim step lowers the software white level, so dimming does not rely on backlight
    modulation alone (own hypothesis, graded insufficient: `PER-WILKINS10` concerns LED lighting
    flicker, and `PER-IEEE1789` was not read in full).
  - `tint` is a comfort preference without claims (insufficient: no cited source tests a display
    comfort tint; matrix row 22).
  - Under light intensity, trusted surfaces keep their distinction by edge, material and label
    rather than by luminance inversion.
- **Q6 Limits.** Contrast floors stay. Flash and pattern limits apply to every toggle.
- **Baseline comfort rule (not a safety limit).** Stripe, grid and dot texture fills (wallpapers,
  backgrounds, ornaments, loader and progress-track fills) with more than five light-dark pairs stay at
  or below a 1.5:1 contrast ratio (own comfort design rule, row 21); component structure such as
  borders, separators and grids stays under the contrast floors instead. The bound is provisional: it
  constrains identity artwork such as wallpapers only after the user test of section 12 (priority 4)
  has calibrated it. The reduced-motion progress stripes (17.4:1 to 21:1 against the track) fall
  inside the bound on tracks wider than 80 px.

### 3.5 Photosensitive epilepsy

- **Q1 Barriers.**
  - Repeated luminance flashes, saturated-red transitions and regular stripes can provoke
    seizures.
  - Photosensitivity occurs in about 0.3–3 % of people (`PER-FISHER05`).
  - In the 1997 broadcast incident, most patients who seized had no earlier unprovoked seizure
    (`PER-ISHIGURO04`), so users often cannot know they should enable a setting.
- **Q2 Supported (normative).**
  - WCAG 2.2 SC 2.3.1 (A): nothing flashes more than three times in any one-second period unless
    it is below the general and red flash thresholds. A general flash is an opposing pair of
    relative-luminance changes of 10 % or more where the darker state is below 0.80. The area limit
    is 0.006 sr, about 25 % of a 10-degree field. The saturated-red definition comes from
    ISO 9241-391 via `PER-U231`.
  - EN 301 549 clauses 4.2.9 and 11.2.3.1.
  - ITU-R BT.1702-3: Guideline 1 sets screen-area and absolute-luminance flash rules, including an
    HDR rule; Guideline 2 sets pattern limits.
- **Q2 Supported (empirical, limited).**
  - Expert consensus of the Epilepsy Foundation of America (`PER-HARDING05`).
  - A stimulus model estimated to protect at least two thirds of susceptible patients
    (`PER-WILKINS05`).
  - Observational reductions in incidence after broadcast guidelines.
- **Q3 Insufficient.** A "reduce motion" or "safe mode" switch as protection. "Epilepsy-safe"
  labels without measurement.
- **Q4 Individual differences.** None are used: the limit is the same for everyone.
- **Q5 Settings.** None replaces the baseline. Reduced motion lowers exposure further for those who
  choose it.
- **Q6 Limits (all baseline).**
  - The flash limit for all system UI, including theme and setting switches, alerts, scan
    highlights and dwell feedback.
  - Pattern limits for wallpapers and effects.
  - The absolute-luminance rules once HDR output exists.
  - Measuring native redraw flicker before release (`PROFILES-AND-USERS.md` 44–51).
  - Future research: a compositor flash guard for third-party content. It is never a setting and can
    never be switched off for system-drawn surfaces.
  - Outside the baseline safety guarantee: a deliberate, warned, non-persistent reveal of one
    third-party media item for one viewing is recorded as a candidate exception, because some content
    carries its meaning in the flashing. It is not offered until a safety review with clinical expertise
    approves it, and it never applies to system-drawn surfaces. A warning alone does not protect:
    in the 1997 broadcast incident, most patients who seized had no earlier unprovoked seizure
    (`PER-ISHIGURO04`).

### 3.6 Vestibular disorders and motion intolerance

- **Q1 Barriers.**
  - Parallax and scroll-linked motion (`PER-U233`).
  - Moving or complex visual stimuli worsen PPPD (`PER-STAAB17`).
  - Visual vertigo in visually dependent patients (`PER-BRONSTEIN95`).
  - Large-field vection is linked to motion sickness, although that link is debated
    (`PER-KESHAVARZ15`).
  - At first boot no motion preference exists yet (natively, the profile store is the device
    preference), so a protection that waits for a preference does not cover boot or first-run setup.
- **Q2 Supported.**
  - SC 2.3.3 (AAA) makes this a setting rather than baseline; technique C39
    (`prefers-reduced-motion`).
  - SC 2.2.2 (A) for auto-moving content.
  - Empirical support is limited and comes entirely from other contexts and populations:
    dynamically narrowing the moving field of view reduced VR sickness in 30 seated participants
    (`PER-FERNANDES16`). `PER-BRONSTEIN95`, `PER-STAAB17` and `PER-KESHAVARZ15` document the barrier
    and its mechanism only.
  - WCAG 2.2 places SC 2.3.3 under Guideline 2.3, Seizures and Physical Reactions. As an AAA criterion
    it mandates no baseline, so the large-field limit in Q6 is an own tiering decision that follows the
    owner's rule that dangerous visual effects are prevented without configuration.
- **Q3 Insufficient.**
  - "Safe" durations or amplitudes for UI motion.
  - A blanket static default, which would remove the understanding benefit of animated
    transitions (`PER-HEER07`) without a baseline reason.
- **Q4 Individual differences.** Large-field versus small motion, amplitude, and daily variation.
- **Q5 Settings.** The graded `motion` axis (full, reduced, none) also serves attention and
  stimulus-load needs.
- **Q6 Limits (baseline).**
  - Every motion has a designed static equivalent (components.css 355–364 already designs
    these).
  - Auto-moving content stops within 5 s or can be paused.
  - A vestibular safety limit for large-field motion, in every regular theme, at boot and before
    any preference exists:
    - excluded by category: parallax backgrounds, scroll-linked movement of backgrounds or large
      surfaces, auto-panning, auto-zooming or drifting wallpapers, and fly-through or zoom-through
      transitions of the whole screen;
    - bounded: a non-essential transition that moves, scales or rotates a surface larger than about a
      quarter of the display (window open and close, workspace or place switch, any surface
      movement within the boot convergence) runs once, never loops and lasts at most 320 ms. These bounds are own design
      rules, not evidence-derived safe values (Q3), and a user test must calibrate them;
    - small purposeful motion stays (the 12 px rise, blade sweeps, the ask halo), so the default stays
      animated;
    - the limit restricts movement (translation, scaling, rotation) only. Light, opacity, glow and
      colour that change in place over large areas are not movement and stay free within the flash
      limits, so the regular themes keep large-field expression. A light front or gradient that
      travels across more than about a quarter of the display counts as movement. Direction for the
      boot convergence (`--motion-duration-boot`, 1400 ms, tokens.css line 80): keep its full length
      as a convergence of light, in which large surfaces brighten and fade in place in one monotonic
      rise, with no front sweeping across the display; elements smaller than a quarter of the
      display, such as the blades, may move into the Orbit for the whole sequence, and any movement
      of a large surface stays within 320 ms;
    - first-run setup follows any device preference already present and offers the motion choice on
      its first screen; the boot sequence itself stays inside the bound.

### 3.7 Dyslexia and reading difficulties

- **Q1 Barriers.**
  - Crowded text (`COG-ZORZI12`).
  - Long lines increase regressions (`COG-SCHNEPS13`).
  - Italic and serif styles reduced performance (`COG-RELLO13`).
  - Text-only output where listening would help (`COG-WOOD17`).
  - Layouts that clip when spacing grows (SC 1.4.12).
- **Q2 Supported.**
  - Read-aloud (strong for its core part, reading comprehension: a meta-analysis of students with
    reading disabilities, `COG-WOOD17`; adults, word highlighting and rate control are not separately
    evidenced).
  - Upright sans-serif without italics (limited: one unreplicated study of font styles,
    `COG-RELLO13`). Emphasis by weight 600 and the underline rule are own design rules, not tested
    by the study; both are already the type-1.0 rule.
  - Paired spacing: letter, word, line and paragraph spacing together, following the SC 1.4.12
    values of 0.12, 0.16, 1.5 and 2 times the font size (limited). Letter spacing alone slowed
    reading (`COG-GALL20`).
  - A measure of at most 80 characters, ragged right (limited; SC 1.4.8 AAA).
  - A reading ruler (limited; `COG-NIK23`).
- **Q3 Contradicted or insufficient.**
  - Dyslexia fonts are contradicted (`COG-AZZ26`, `COG-KUSTER17`, `COG-WERY16`).
  - Overlays are insufficient (`GRIFFITHS16`, `COG-SUTTLE18`).
  - Bionic reading is insufficient and inconsistent (`COG-BEEL25`, `COG-WANG26`).
- **Q4 Individual differences.** The fastest font differs per reader by up to 35 %
  (`COG-WALL22`). Font preference does not predict performance (`COG-KUSTER17`). Spacing benefit
  varies (`COG-SCHNEPS13`).
- **Q5 Settings.** `text-spacing` (one switch for the four values), `line-length`, `read-aloud`
  and `line-focus`. These also serve low vision, second-language readers and fatigue.
- **Q6 Limits.**
  - Every layout survives the SC 1.4.12 override.
  - No string claims that a font or colour treats dyslexia.
  - Read-aloud runs on the device without network permission.

### 3.8 Dyscalculia and low numeracy (added: older adults)

- **Q1 Barriers.**
  - A core deficit in understanding numerosity (`COG-BUTT11`).
  - Numbers without a unit, reference or complement ("8 GB used" without "of 10 GB").
  - Mixed formats.
  - Quantities shown by colour alone.
- **Q2 Supported (limited, from health risk communication).**
  - Icon arrays improved risk understanding in older adults with low numeracy (`COG-GALE09`).
  - Visual aids helped less numerate people most (`COG-GR17`).
  - Consistent formats and tabular digits are already specified (TYPOGRAPHY.md 8.4).
- **Q3 Insufficient.** No study of OS numeric displays was found. Training is not an OS remedy
  (`COG-BUTT11` calls it promising, not proven).
- **Q4 Individual differences.** Exact figures, rounded words or pictures.
- **Q5 Settings.** `numbers` adds proportion bars, icon arrays and rounded words. It never removes
  the exact value, and it combines with read-aloud.
- **Q6 Limits.** A decision-relevant number always carries its unit and its reference or
  complement, is never shown by colour alone, and keeps its exact value available.

### 3.9 ADHD and attention differences

- **Q1 Barriers.**
  - Interruptions: in a general-population study, having alerts on raised self-reported
    inattention (`COG-KUSH16`).
  - Looping content beside reading.
  - Toasts that take focus or vanish.
- **Q2 Supported (limited; no ADHD-sample interface study).**
  - In a randomised field experiment in the general population (n = 237, no ADHD sample
    described), batching notifications three times a day raised self-reported attentiveness and
    lowered stress (`COG-FITZ19`; limited, other population).
  - SC 2.2.2 (A) and 2.2.4 (AAA); BaryonOS makes the interruption control baseline.
  - ADHD technology research is mostly deficit-framed with little co-design (`COG-SPIEL22`), which
    supports properties over profiles.
  - Notifications that never take focus and never auto-dismiss are a mandatory access limit on
    normative grounds; their empirical grade is insufficient, because `COG-KUSH16` (general
    population) bears only on suppressing alerts.
- **Q3 Insufficient.** "ADHD modes" that strip the interface. "All notifications off" as the
  attention remedy: no study tested it for attention or ADHD (insufficient). Context only (limited,
  other population): in the same general-population experiment, the group that received no
  notifications gained few of the benefits of batching and reported more anxiety and fear of
  missing out (`COG-FITZ19`).
- **Q4 Individual differences.** Stimulation versus calm, immediate versus batched alerts, and
  motion tolerance.
- **Q5 Settings.** `interruptions` (immediate, batched, quiet; per app and per contact), `motion`,
  `line-focus` and resume support.
- **Q6 Limits.**
  - Notifications never take keyboard focus.
  - An actionable item is never dismissed before it is stored in the notification centre.
  - Security and consent signals are never batched or hidden (cross-cutting row,
    `BOS-SECFLOOR`).

### 3.10 Autism and sensory-processing differences

- **Q1 Barriers.**
  - Sensory differences run in both directions: under-responsivity, over-responsivity and
    sensation seeking (`COG-BENS08`, meta-analysis of 14 studies).
  - Small eye-tracking studies found more fixations on irrelevant elements and a non-equivalent
    experience with high visual complexity (`COG-ERAS18`, `COG-ERAS20`).
  - Interfaces that rearrange themselves.
- **Q2 Supported.**
  - Clear regions and bounded complexity (limited).
  - Predictable navigation and no change of context on focus (SC 3.2.3 AA, 3.2.4 AA, 3.2.5 AAA;
    COGA).
- **Q3 Insufficient.** A single "low-stimulus autism theme". It conflicts with the two-directional
  differences.
- **Q4 Individual differences.** Visual intensity, sound and motion each differ independently.
- **Q5 Settings.**
  - `stimulus` (rich, regular, calm) adjusts glow, ornament, ambient effects and simultaneous
    panels while keeping layout, shapes, place colours and identity.
  - The regular theme is the middle step, not the maximum. `rich` only adds intensity (more glow,
    ambient depth and simultaneous detail) for people who seek stimulation. Every flagship identity
    feature belongs to the regular default, and no identity element is reserved for `rich`.
- **Q6 Limits.** Adaptive suggestions appear in one labelled place and never reorder the familiar
  path. Sound is never the only signal.

### 3.11 Memory, cognitive load and acquired brain injury (added)

- **Q1 Barriers.**
  - Recall demands, data-losing time-outs, irreversible actions, puzzle-like authentication and
    loss of place (COGA).
  - In 1 204 adults aged 18–91, cognitive abilities and computer anxiety mediated the link between
    age and technology use (`COG-CZAJA06`).
- **Q2 Supported.**
  - Reminders that prompt a scheduled action: strong for memory and planning after brain injury
    (`COG-GILL11`, `COG-WILS01`). The other parts of the reminder row (user creation on the device,
    snooze and done, checked task steps) are untested (insufficient).
  - Normative, limited through COGA:
    - SC 2.2.1 (A, turn off, ten times or a 20 s warning; exceptions for limits over 20 hours);
    - SC 3.3.4 (AA, reversible or confirmed actions);
    - SC 3.3.8 (AA, no cognitive function test);
    - SC 3.3.7 (A, no re-entry of known information);
    - SC 3.2.4 (AA, consistent labels).
- **Q3 Insufficient.** Brain-training features. "Simple modes" as a replacement OS, which the owner
  also excludes.
- **Q4 Individual differences.** Reminder modality, amount of step guidance, and whether a resume
  summary is wanted.
- **Q5 Settings.** `memory-support` (reminders; resume). Returning focus to the previous context is
  baseline.
- **Q6 Limits.** No data loss on time-out or lock. Destructive actions are reversible or confirmed
  in plain words.

### 3.12 Plain language, low literacy, second language and intellectual disability (added)

- **Q1 Barriers.** Jargon (NAS, capability, encryption), long sentences, and errors shown as codes
  or colour, including in permission decisions.
- **Q2 Supported.**
  - ISO 24495-1 (public scope), SC 3.1.3–3.1.5 (AAA), 3.3.1 (A) and 3.3.3 (AA), and COGA.
  - Empirical support is limited: text format affected understanding, but the evidence base is
    weak (`COG-SUTH16`), and easy-read evidence is of low quality (`COG-WILS26`).
- **Q3 Insufficient.** Easy-read versions as proven. Readability formulas, which were not
  assessed.
- **Q4 Individual differences.** Language level, wanting explanations, and reading speed.
- **Q5 Settings.** `language-level` (explanation lines, symbols, easier variants) with read-aloud
  and `numbers`.
- **Q6 Limits.** Permission and security texts are in plain language in every theme. Explanations
  never shorten a permission prompt below its required content.

### 3.13 Motor impairments and alternative input

- **Q1 Barriers.**
  - Small or dense targets, drags and chorded actions; distinct cursor-movement patterns in older
    adults and people with Parkinson's disease (`INT-KEATES2005`, `INT-FINDLATER2017`).
  - Pointer-only shell functions.
  - Fixed dwell and scan rates.
  - Icon-only controls without speakable names.
- **Q2 Supported.**
  - SC 2.5.8 (AA, 24 × 24 CSS px or spacing), 2.5.5 (AAA, 44 × 44), 2.5.1 (A), 2.5.2 (A), 2.5.7
    (AA), 2.1.1 (A), 2.1.2 (A), 2.1.4 (A), 2.4.11 (AA) and 2.5.3 (A).
  - EN 301 549 clauses 5.7 (key repeat), 5.8 (double-strike acceptance), 5.9 (no simultaneous
    actions), 5.2 (activating accessibility features) and 5.1.7 (access without speech).
  - Ability-based and pointer-gain adaptation (limited: one controlled study each,
    `INT-GAJOS2008` for generated layouts and `INT-WOBBROCK2009` for pointer gain with 8
    motor-impaired participants; section 1.4). The 24 px floor itself is normative; no study read
    tests it.
  - User-adjustable dwell, which sped up novices (`INT-MAJARANTA2009`, limited).
  - Multimodal gaze and switch input: fewer errors, but four individual response patterns
    (`INT-FAGER2022`, limited).
  - Camera-based pointers (`INT-BETKE2002`, limited).
- **Q3 Insufficient.**
  - Touch as inherently easier: insufficient (section 4). One small controlled study (16
    participants with upper-body motor impairments, 16 without) found three times as many tapping
    errors on touch as with a mouse for the motor-impaired group, although touch was faster
    (`INT-FINDLATER2017`): a limited signal against the error part of the claim, too small to
    contradict it (synthesis rule 6).
  - A universal 44 px default as the only remedy.
  - One fixed scan rate.
  - Gaze as general input for everyone.
  - Off-the-shelf assistants for disordered speech (`INT-GREEN2021`, `INT-BALLATI2018`).
- **Q4 Individual differences.** Movement profile, filter windows, scan mode, dwell time, and
  push-to-talk versus wake word.
- **Q5 Settings.** `targets`, `pointer-assist`, `keyboard-filters`, `input-timing` (one shared axis
  for double-click, hover, message duration, key repeat, filter windows, dwell and scan rate) and
  `alternative-input` (switch, dwell, head pointer, voice).
- **Q6 Limits.**
  - A 24 px minimum.
  - No keyboard trap.
  - Single-key shortcuts can be remapped or turned off.
  - Keyboard filters can never lock a user out: their state is visible, they have a modifier-free
    off path, and they reset on the trusted screen.
  - Consent and destructive actions need a deliberate confirmation that a resting gaze or a stray
    switch press cannot complete.
  - Speech is never the only input.

### 3.14 Hearing: deaf and hard of hearing, sign-language users

- **Q1 Barriers.** Speech in media without text. Alerts carried only by sound. Written language as
  a second language for Deaf signers.
- **Q2 Supported.**
  - Captions and transcripts (SC 1.2.2 A, 1.2.4 AA; EN 301 549 clauses 7.1.1–7.1.4) are limited
    (`INT-GERNSBACHER2015`, a narrative review whose deaf and hard-of-hearing share was not
    established) and baseline on normative grounds.
  - Visual and haptic alerts were wanted by almost all of 201 DHH respondents (`INT-FINDLATER2019`,
    survey).
  - Sign video (SC 1.2.6 AAA; `INT-BRAGG2019`); the WFD and WASLI statement favours skilled human
    signers (`INT-WFD2018`).
- **Q3 Insufficient.**
  - Automatic captions as a full equivalent: word error rate tracked DHH usability judgements
    poorly (`INT-KAFLE2017`).
  - Screen flash as the universal alert.
  - Adjustable caption style (size, colour, background, outline, position): insufficient. No resolved
    study of these parameters with deaf and hard-of-hearing viewers was found; `captions` keeps the
    style controls as a setting without an effect claim (matrix row 76).
  - Signing avatars as equivalent access: the WFD and WASLI statement cautions against them
    (`INT-WFD2018`).
- **Q4 Individual differences.** Caption size, colour and position; full captions versus keywords;
  visual versus haptic alerts; device choice. The comprehension effects of caption rate (130 to 230
  words per minute) and text reduction (84 to 100 % of the text) differed by reading level, hearing
  status and age; reading level dominated, and more proficient readers benefited more from rate
  changes (`INT-BURNHAM2008`).
- **Q5 Settings.** `captions` (one preference for every player, with style), `alert-modality` and
  `sign-language`. These also serve noisy or silent places and second-language listeners.
- **Q6 Limits.**
  - Never sound-only.
  - Automatic captions are labelled and generated on the device.
  - No avatar is presented as equivalent access.
  - Visual alerts obey the flash limit.

### 3.15 Speech impairments and deafblindness (added)

#### Speech impairments

- **Q1 Barriers.**
  - Speaker-independent recognition often fails on dysarthric or atypical speech (`INT-GREEN2021`,
    `INT-BALLATI2018`).
  - Speech-only flows and short response windows exclude non-speaking users and slow speakers
    (`INT-NAUR`).
- **Q2 Supported.**
  - Normative: EN 301 549 V4.1.1 clauses 5.1.7 and 6.4; W3C NAUR (Group Draft Note): a text or
    keyboard alternative, tolerance of atypical speech, adjustable response time and visible text
    output.
  - Empirical: personalised models cut recognition error sharply (`INT-GREEN2021`, moderate). The
    outcome is word-error rate, not usability or on-device training.
- **Q3 Insufficient.** Off-the-shelf voice assistants as an adequate route for disordered speech:
  assistants differed widely in recognising dysarthric speech (`INT-BALLATI2018`).
- **Q4 Individual differences.** Speech rate, intelligibility and fatigue differ per person and, in
  progressive conditions, over time. How many enrolment phrases a person can record also differs.
  Both are own inference, not measured in the sources read.
- **Q5 Settings.** `alternative-input` (voice) with an optional personal on-device model;
  `input-timing` for response windows; keyboard, pointer, switch and text equivalents through the
  same semantic tree. These axes also serve motor and fatigue needs.
- **Q6 Limits.**
  - Speech is never the only input.
  - Voice data and enrolment recordings stay on the device and are deletable
    (`docs/LOCAL-PRIVACY-CONTRACT.md`).
  - A misheard phrase can never complete consent or a destructive action.

#### Deafblindness and braille

- **Q1 Barriers.** Users who can neither see the screen nor hear speech depend entirely on
  refreshable braille (`INT-JAISWAL2018`). Setup and sign-in with audio-only or visual-only feedback
  lock them out.
- **Q2 Supported.**
  - Normative: EN 301 549 V4.1.1 clauses 5.1.3 and 11.5.2.1; WCAG 2.2 SC 4.1.2 (A) for the semantics
    that braille output reads.
  - Empirical: restricting tactile cues, for example removing sliding contact, lowered letter
    recognition (`INT-RUSSOMANNO2015`, limited). That supports the display-type row only. Braille
    display support itself is graded insufficient: no study of operating-system braille support was
    found, and `INT-JAISWAL2018` documents barriers.
- **Q3 Insufficient.** Single-cell or vibration-only output as equivalent to a multi-cell display for
  reading; `INT-RUSSOMANNO2015` points the other way. No study of operating-system braille workflows
  was found in this search.
- **Q4 Individual differences.** Braille code and grade (contracted or uncontracted, language table),
  display size in cells, reading speed, and the degree of remaining hearing or vision: dual sensory
  loss is a range, not one group (`INT-JAISWAL2018`).
- **Q5 Settings.** `screen-reader` with braille output and input. For partial loss, the same axes as
  low vision and hearing (`magnifier`, `text-size`, `contrast`, `captions`) and a haptic
  `alert-modality`. No separate deafblind profile.
- **Q6 Limits.**
  - First-run setup and sign-in can be completed by braille alone.
  - No audio-only or visual-only feedback in setup.
  - Permission prompts and security signals reach the braille display in full.
  - The screen reader is reachable from first boot.

### 3.16 Reduced stereoscopic depth perception (added)

- **Q1 Barriers.** About 7 % of adults under 60 are stereoblind, and older adults may have a higher
  prevalence (`PER-CHOPIN19`). Futuristic depth effects (stereo, depth-of-field blur, depth by
  parallax, shadow-only layering) can carry hierarchy that these users do not perceive.
- **Q2 Supported.** Normative: EN 301 549 V4.1.1 clause 4.2.2 Note 3 and WCAG 2.2 SC 1.3.3 (A). No
  interface study was found, so the adaptation is graded insufficient.
- **Q3 Insufficient.** No popular remedy specific to reduced depth perception was identified in this
  search. Depth- or stereo-heavy "spatial" interfaces presented as more intuitive for everyone have no
  support in the sources read.
- **Q4 Individual differences.** Stereo ability differs widely between individuals rather than being
  simply present or absent, and measured prevalence depends on the test used (`PER-CHOPIN19`).
- **Q5 Settings.** No separate axis is needed. `contrast` (border weight), `transparency: opaque` and
  `motion: reduced` remove depth cues that remain decorative (own inference). Depth by motion parallax
  would conflict with vestibular comfort, so it is never the only cue and never a large-field effect.
- **Q6 Limits.** No meaning by depth alone: layering, focus and trusted surfaces also differ by edge,
  contrast, label or position. Depth stays as expression.

## 4. Popular solutions with insufficient or contradicted evidence

| Solution | Grade | Treatment in BaryonOS |
|---|---|---|
| Dyslexia fonts (Dyslexie, OpenDyslexic) | contradicted | Never a default or a recommended aid. A reading face may later be a plain preference without claims (font/atlas extension). |
| Coloured overlays and tints for reading | insufficient | Only the `tint` comfort preference, without a reading or health claim. |
| Blue-light filters for eye strain | insufficient | A Cochrane review of 17 trials of blue-light filtering spectacles found low-certainty evidence of no short-term difference in visual fatigue, with most trials at high risk of bias (`PER-SINGH23`), so the claim is unsupported rather than contradicted. No health claim. A warm tint is comfort only. |
| Precision tints for migraine | limited | One small double-masked randomised crossover trial (n = 17) with a marginal effect (`PER-WILKINS02`). Not offered as treatment: the `tint` axis stays comfort-only. |
| Colour-blind "correcting" glasses | contradicted | Contradicted for the narrower claim that they improve colour-vision test results or restore normal colour vision: one controlled study with what its abstract calls a high number of observers with CVD (no sample size given there) found neither (`PER-GOMEZ18`). Never offered as a cure. |
| Colour-blind "correcting" software filters (daltonisation) as a cure | insufficient | No user-effect study was found; `PER-RIBEIRO19` surveys algorithms only. `colour-perception` is labelled as an aid, not a cure. |
| Dark mode as a universal legibility fix | insufficient | No adequate study settles the universal claim. Limited context against it: one controlled study of normally sighted younger and older adults found a positive-polarity advantage for acuity and proofreading in both age groups (`PER-PIEPENBROCK13`). Its sample size and design details are not in the PubMed abstract, and the full text could not be read (no PubMed Central copy, closed access in OpenAlex, publisher page HTTP 403 on 15 November 2026), so it cannot carry a contradicted grade (synthesis rule 6). Among 19 low-vision readers, five read white-on-black faster (`PER-RUBIN89`), which supports individual choice. Both polarities are full-quality options; neither is presented as healthier. |
| APCA or other new contrast metrics | insufficient | Normative basis: none. WCAG 3 is a Working Draft whose contrast metric is still to be determined (`WCAG3`), so WCAG 2.2 ratios stay the floor and no APCA threshold is adopted. |
| A reduce-motion switch as seizure protection | insufficient | Flash limits are baseline for everyone. |
| "Bionic reading" | insufficient | Results are inconsistent (`COG-BEEL25`, `COG-WANG26`). Not proposed. |
| Diagnosis modes ("ADHD mode", "autism theme", "epilepsy mode") | insufficient | No study found; rejected by the owner's decision. Axes only; presets are named combinations. |
| "All notifications off" as the attention remedy | insufficient | No study tested it for attention or ADHD. Context only (limited, other population): in one randomised field experiment in the general population (n = 237, no ADHD sample described), participants who received no notifications gained few of the benefits of batching and reported more anxiety and fear of missing out (`COG-FITZ19`). Batched and quiet modes both stay available; neither is presented as an ADHD treatment. |
| Touch as easier for motor impairment | insufficient | One small controlled study in the target population (16 participants with upper-body motor impairments) found three times as many tapping errors on touch as with a mouse, although touch was faster overall (`INT-FINDLATER2017`): a limited signal against the error part of the claim, too small to contradict it (synthesis rule 6). Touch stays a full input method with per-user targets and pointer assistance. |
| Automatic captions as equivalent | insufficient | Labelled as automatic; author captions win. |
| Signing avatars as equivalent | insufficient | The World Federation of the Deaf and WASLI advise against presenting them as equivalent (`INT-WFD2018`). Research only, with the Deaf community. |
| Off-the-shelf voice assistants for disordered speech | insufficient | Personal on-device models (future research). |
| Easy-read and brain-training as proven | insufficient | Plain language is baseline; no cure claims. |

## 5. Conflicts between needs and how settings resolve them

1. **High contrast versus light sensitivity.**
   - Older and low-vision users may need maximum contrast; light-sensitive users find bright,
     maximum-contrast surfaces painful (`PER-W3CLV`).
   - `contrast` and `light-intensity` are separate axes. Increased contrast with dimmed luminance
     gives, for example, off-white text on near-black that stays at or above the contrast floors.
2. **Motion for understanding and identity versus vestibular safety.**
   - The default keeps purposeful motion (`PER-HEER07`; the owner's futuristic ambition).
   - The device preference selects *reduced*, which has no motion animation but keeps fades; *none*
     is one step further.
   - A baseline vestibular safety limit bounds large-field motion in every theme, at boot and before
     any preference exists (section 3.6 Q6); small purposeful motion stays, so the default remains
     animated. *Reduced* then removes the remaining large-field motion first. The bounds and that
     ordering are own inference and need a user test.
   - The limit constrains movement only. Light, opacity, glow and colour that change in place over
     large areas stay free within the flash limits, so the regular themes keep large-field
     expression; a light front travelling across more than about a quarter of the display counts as
     movement (section 3.6 Q6, with a direction for the 1400 ms boot convergence).
3. **Dark versus light polarity.**
   - Evidence favours positive polarity for reading in normally sighted adults
     (`PER-PIEPENBROCK13`); photophobia and night use favour dark.
   - `polarity` follows the device. Both are full-identity variants with the same materials, place
     colours and motion language.
   - The product default when no device preference exists is a UX-A11Y-DS design decision. A light
     default is compatible with the evidence read.
4. **Stimulation seeking versus avoidance** (`COG-BENS08`). `stimulus` has steps in both directions
   around a regular middle; calm does not remove identity, and rich adds intensity only. Every
   flagship identity feature is in the regular default.
5. **Glass translucency versus contrast.**
   - Contrast floors are measured over the worst-case backdrop at baseline.
   - `transparency: opaque` is independent of contrast.
   - Increased contrast may add an opaque backing under text runs. That is contrast's own property;
     it does not switch the transparency axis off for other surfaces.
6. **Trusted Kern surface in the opposite theme versus light sensitivity.**
   - The inverted Kern panel is a bright patch for light-sensitive users.
   - Under dimmed light intensity, trust stays recognisable by edge, material, label and position,
     not by luminance inversion. Its presence never depends on a setting.
7. **Colour-rich identity versus colour-vision differences.**
   - The place triad keeps its colours. Shape, icon and label carry the meaning at baseline.
   - `colour-perception` remaps only for users who choose it.
8. **Visual alerts versus photosensitivity.** The full-screen visual alert is a sub-threshold
   luminance change; the flash limit is baseline.
9. **Interruption batching versus security.** Permission prompts, trust indicators and data-loss
   warnings are never batched, delayed or simplified (`BOS-SECFLOOR`). WCAG SC 2.2.4 also exempts
   emergencies.
10. **Adaptive, intent-aware behaviour versus predictability.** Suggestions live in one labelled
    place and never reorder Orbit, menus or settings (SC 3.2.3 AA, COGA).
11. **Larger targets versus density.** The 24 px baseline keeps the default dense and refined.
    `targets` and invisible pointer assistance (motor-space enlargement) serve those who need more.
12. **Time limits for security versus input timing.**
    - Security time-outs lock the session but never discard entered data.
    - A time limit is extendable unless it is essential (SC 2.2.1).
13. **Read-aloud, captions and voice versus privacy.** All run on the device without network
    permission (`docs/LOCAL-PRIVACY-CONTRACT.md`).

## 6. The two-tier model

### 6.1 Tier 1: universal baseline (every regular theme, no configuration, never optional)

None of these items makes the default theme duller, static or simpler without an access or safety
reason. The vestibular limit removes only large-field motion; the default keeps its small, purposeful
motion.

- **Perception.**
  - Redundant non-colour cues; information survives greyscale; CVD-safe chart palettes.
  - Text 4.5:1 and UI and focus 3:1 over the worst-case glass backdrop, in every theme and setting
    combination.
  - Visible, unobscured focus.
  - 200 % resize with reflow.
  - No meaning by depth alone.
  - Stripe-texture comfort bound (row 21, untagged comfort design rule): texture fills with more than
    five light-dark pairs stay at or below a 1.5:1 contrast ratio; component structure stays under
    the contrast floors. It constrains identity artwork only after user-test calibration.
- **Safety.** These limits hold under every theme, setting combination, toggle and surface the system
  draws, at boot and before any preference exists. In the matrix each is a row whose Adaptation
  starts with "Safety:" (11 rows); `check.py` fails when a required limit has no tagged row or a
  tagged row is not baseline. The other Q6 limits (for example speech never the only input, never
  sound-only, no keyboard trap, no data loss on time-out) are mandatory access limits: baseline rows
  without the tag, held by the rule that a baseline row's axis is `none (baseline)`.
  - Flash limits (SC 2.3.1; BT.1702-3 Guideline 1).
  - Pattern limits (BT.1702-3 Guideline 2). The stripe-texture comfort bound (row 21) is not a
    safety limit; it is listed under Perception.
  - Absolute-luminance rules once HDR output exists.
  - Native redraw flicker measured before release.
  - A compositor flash guard for third-party content (future research) that is never a setting and
    never switched off for system-drawn surfaces. A per-item reveal for third-party media is outside
    this guarantee and is not offered until a clinical safety review approves it.
  - A vestibular safety limit for large-field motion, boot included (section 3.6 Q6). It limits
    movement only; light, opacity, glow and colour that change in place over large areas stay free
    within the flash limits, while a light front travelling across the display counts as movement.
  - Auto-moving content stops within 5 s or can be paused.
  - Every motion has a static equivalent.
  - Keyboard-filter safety, deliberate consent under alternative input, and security signals intact
    under every setting (also listed under Interaction and Platform).
- **Reading and cognition.**
  - Layouts survive text-spacing overrides.
  - A measure of at most 80 characters on reading surfaces.
  - Upright sans-serif with weight emphasis.
  - Plain-language system and permission text.
  - Numbers with unit and reference.
  - Predictable layout.
  - Recognition over recall.
  - Reversible or confirmed destructive actions.
  - Errors in words with a suggested fix.
  - Focus returns to context.
- **Interaction.**
  - Targets of at least 24 px.
  - Full keyboard operation with no trap.
  - Remappable single-key shortcuts.
  - Single-pointer alternatives to gestures.
  - Accessible names that contain the visible label.
  - No unextendable time limit; no data loss on time-out.
  - Filter safety.
  - Deliberate confirmation for consent under alternative input.
  - Speech never the only input.
  - Never sound-only.
  - Captions and transcripts for first-party media.
- **Platform.**
  - One accessibility tree with focus and change events.
  - Automatic following of device and user preferences.
  - Apps receive rendered adaptation rather than raw values.
  - Findable, combinable and resettable axes.
  - Security signals intact under every setting.
  - A trusted AT role.
  - Accessible sign-in, lock and trusted screens, with the screen reader (with braille), magnifier
    and motion choice reachable from first boot.
  - Read-aloud, captions, voice and any other inference run on the device without network
    permission.
  - No setting, preset or help text claims to treat or cure a condition.
  - EN 301 549 V4.1.1 clause 11 as the platform checklist.

### 6.2 Tier 2: settings

Settings are additional, independently combinable axes (section 7). They are variants of the same
BaryonOS identity: materials, place colours, Orbit, typography and the light-blade language remain
recognisable in every combination. A setting may start automatically from a device or user
preference, and the user is told which preference was followed.

## 7. Setting axes

### 7.1 Axis list

"Owns" names the rendered properties the axis controls. No other axis may change those properties.

| Family | Axis | Values (default first) | Automatic source | Owns |
|---|---|---|---|---|
| Vision | `polarity` | product default; light; dark | `prefers-color-scheme` (prototype); profile value (native) | background and foreground polarity |
| Vision | `contrast` | standard; increased (text ≥ 7:1) | `prefers-contrast: more`; `forced-colors: active` (system colour roles) | token contrast ratios, border weight, text backing plates |
| Vision | `light-intensity` | standard; dimmed steps; extra-dim | manual; optional user-chosen schedule; ambient sensor only by user choice | maximum luminance and software white level |
| Vision | `tint` | none; warm; user text and background pair | manual only | colour temperature and text-background pair (comfort, no claims) |
| Vision | `colour-perception` | off; protan; deutan; tritan (with intensity); optional compositor filter | manual (no standard device preference) | hue mapping of palettes and content |
| Vision | `transparency` | translucent; opaque | `prefers-reduced-transparency` where offered | material opacity and blur |
| Vision | `text-size` | 100 %; 125 %; 150 %; 200 % and larger | device text-size preference where present | text scale and reflow |
| Vision | `magnifier` | off; full-screen; lens (level, tracking) | shortcut; device magnifier preference | scaled composition |
| Vision | `screen-reader` | off; on (rate, verbosity, braille) | shortcut; first-boot offer | speech and braille output |
| Motion and stimulus | `motion` | full (within the baseline large-field bound); reduced; none | `prefers-reduced-motion: reduce` selects reduced (natively the profile value); before any value exists, at first boot, only the baseline bound applies and first-run setup offers the choice | movement animation and loops |
| Motion and stimulus | `stimulus` | regular; calm; rich | manual | glow, ornament, ambient effects, simultaneous panels; `rich` only adds intensity, and every flagship identity feature is in the regular default |
| Motion and stimulus | `interruptions` | immediate; batched; quiet (per app and contact) | device focus or do-not-disturb state | notification delivery timing (never security signals) |
| Reading | `text-spacing` | standard; wider (four SC 1.4.12 values together) | manual | letter, word, line and paragraph spacing |
| Reading | `line-length` | standard (≤ 80 characters); short (about 40–50) | manual | reading-surface measure |
| Reading | `typeface` | Inter; a later contract face | manual (font/atlas extension) | reading face (no claim) |
| Reading | `read-aloud` | on demand; word-highlight style | shortcut | on-device speech of selection or focus |
| Reading | `line-focus` | off; band; underline | manual | reading ruler |
| Reading | `numbers` | exact; with visual aids | manual | additional numeric representations |
| Reading | `language-level` | standard; with explanations | manual | explanation lines and easier variants |
| Hearing | `captions` | off or on everywhere; style | device caption preference | caption display and style |
| Hearing | `alert-modality` | sound with visual twin; plus banner, edge light-blade, full-screen visual, haptic | manual per device | alert presentation channels |
| Hearing | `sign-language` | show tracks when provided | manual | sign video area |
| Input | `targets` | 24 px; 32; 44; larger | manual or optional calibration | hit-area size and spacing |
| Input | `pointer-assist` | off; gain reduction; area cursor | manual or optional calibration | pointer transform |
| Input | `keyboard-filters` | each off: sticky, slow, bounce, repeat, mouse keys | confirmed shortcut | key event filtering |
| Input | `input-timing` | platform defaults with live preview | manual | double-click, hover, message duration, repeat, filter windows, dwell, scan rate |
| Input | `alternative-input` | off; switch; dwell; head pointer; voice | manual | additional input sources |
| Memory | `memory-support` | off; reminders; resume | manual | on-device reminders and resume |

The matrix records the evidence row for each axis. Axes without user-effect evidence are offered as
preferences without claims.

### 7.2 Combination rules

1. **One owner per property.** Because no axis may change a property another axis owns, choosing
   increased contrast no longer switches off motion, translucency or light polarity, as the
   prototype contrast theme does today (components.css 247 and 372–381).
2. **A fixed colour pipeline.** Colour is computed in the order polarity → contrast →
   colour-perception → tint → light-intensity. Contrast floors are re-verified after every stage.
   When a cosmetic stage (tint) would break a floor, its strength is limited and the user sees why;
   the floor never yields. Safety rows are evaluated last and always win.
3. **Device and user preferences are inputs, not locks.** Settings shows "Following your device:
   reduced motion", with a per-axis override. Proposal: natively, BaryonOS is itself the platform,
   so its profile store would act as the device preference that apps (and any future browser
   engine) see; this is part of the profiles extension in section 10.
4. **Presets are shortcuts only.** A convenience preset (for example "Calm reading": calm stimulus,
   wider spacing, short lines, reduced motion) is a named combination. It is never named after a
   diagnosis, every member stays individually adjustable, and changing one member does not reset
   the others.

### 7.3 Finding and resetting

- **Where the settings are.**
  - Settings > Accessibility, grouped in the families above.
  - A quick-settings tile.
  - The trusted sign-in and lock screens. Before any account exists, first-run setup offers
    "Display, reading and input" ahead of account creation. Its first screen follows any device
    preference already present and offers the motion choice before any large-field animation; the
    boot sequence itself stays within the baseline vestibular limit (section 3.6 Q6).
- **Shortcuts.** The screen reader, magnifier and increased contrast have shortcuts. A shortcut
  that enables a keyboard filter asks for confirmation.
- **Reset.** Every axis has its own reset, and "Reset all accessibility settings" is undoable.
  Resets are reachable from the trusted screen with keyboard, switch or screen reader. A reset
  never disables a safety row, because safety rows are not settings.

## 8. How apps respect system-wide settings, and preference privacy

- **System rendering first.**
  - SDK components, tokens, text layout and the compositor apply every axis. Apps built on the SDK
    follow the user without code and without reading raw values.
  - This meets EN 301 549 V4.1.1 clause 11.7 for system components. Under clause 11.7 Note 5, an
    app may have extra values for a setting as long as one mode follows the system.
  - Clause 11.6.2 forbids apps from disrupting platform accessibility features.
- **Preference values are personal data.**
  - They can reveal health or disability information. Media Queries 5 warns that preference
    features can identify users (`INT-MQ5`), and EN 301 549 clause 4.2.11 requires accessibility
    users to keep the same privacy as others.
  - Proposal: values are stored with the user's encrypted profile data and, if synchronised at
    all, only to the user's own paired devices under the end-to-end rules of
    `docs/LOCAL-PRIVACY-CONTRACT.md`.
  - They never appear in logs. The project rule of no OS telemetry (AGENTS.md) also applies.
- **Any app-facing preference API is an extension, not a change.**
  - It is a separate capability with consent per property.
  - It returns coarse values only (for example "reduce motion: yes").
  - It never reveals whether assistive technology is running, and it is revocable.
  - Proposal: if a BaryonOS browser exposes preference media features to web content, it exposes
    only those the user has not restricted.
- **On-device processing.** Read-aloud, captions, speech recognition, summarising, the "where was
  I" resume summary and any camera-based pointer run on the device without network permission
  (`docs/LOCAL-PRIVACY-CONTRACT.md`).

## 9. Screen reader, keyboard, magnification and alternative input

- **One semantic tree.**
  - The accessibility service holds a system-wide tree with name, role, value, states, relations
    and actions, plus focus, selection, window and live-region events (EN 301 549 clauses
    11.5.2.1–11.5.2.17).
  - It serves the screen reader, braille, voice control (speakable names, numbered overlays),
    switch scanning (action lists), dwell, the magnifier (focus and caret rectangles) and
    automated audits.
  - An app-local `SemanticNode` tree exists in `ui/native/src/render.rs` with host tests only
    (U04 open).
  - AccessKit's push model (a full tree, then incremental updates) is a reference design. It is not
    an adopted dependency.
- **Keyboard.**
  - Every shell and app function is keyboard-operable without timing, with no trap.
  - Shortcuts follow `DESKTOP-SHELL.md`, use modifiers, and single-key shortcuts are remappable.
  - Focus is never fully hidden by Orbit or toasts.
  - Keyboard operation is the substrate for switch, voice and on-screen keyboards.
- **Magnification.**
  - A compositor magnifier, not page zoom.
  - It follows focus, caret, pointer and system prompts, and brings permission prompts into view.
  - Magnified movement follows the `motion` axis, and the magnifier is available at sign-in.
- **Alternative input.**
  - Switch scanning is ordered by the semantic tree (Orbit, window, content, notifications).
  - Dwell has visible progress and a rest area.
  - Head and camera pointers are on-device only, with a visible indicator.
  - Voice control uses on-device recognition with push-to-talk or wake confirmation.
  - All of these inject input only through the input service under a trusted AT role, and none can
    approve a capability prompt without a deliberate confirmation.

## 10. Contract extensions needed (recorded, nothing changed)

Each item below is a separate item for its contract owner. None was made in this run. Section 10.1
extends contracts that exist in the repository; section 10.2 lists contracts that do not exist yet.
In the matrix, a native-status cell that needs one of the latter reads "new contract: <name> (none
exists yet)".

### 10.1 Extensions of existing contracts

1. **Font and atlas** (`docs/design/TYPOGRAPHY.md`):
   - text-size steps beyond type-1.0;
   - spacing parameters in `text.rs` (letter, word, line, paragraph);
   - a layout measure in characters;
   - caption sizes;
   - any later reading face.

   All of this must fit the mutually exclusive Open Sans and Inter atlases under the 512 KiB
   read-only limit, or the limit itself needs a recorded extension.
2. **Compositor** (`NATIVE-COMPOSITOR.md`):
   - scaled composition for the magnifier;
   - output luminance and white-level scaling;
   - colour-matrix filters;
   - opacity per material;
   - a flash budget and frame-level check;
   - the third-party flash guard, with no off switch for system-drawn surfaces;
   - a back buffer or page flip against redraw flicker;
   - AT overlays (scan highlight, dwell progress, numbered labels);
   - a protected system surface that notifications cannot cover;
   - the animation clock that applies `motion`, and the vestibular safety limit for system surfaces
     (category exclusions and the area and duration bounds, boot included);
   - in that clock, the 5 s stop or settle of auto-moving content and a registered static equivalent
     per animation. `NATIVE-COMPOSITOR.md` defines no animation clock, and native code has no
     animation yet (`ui/native/src/design.rs` line 71).
3. **Accessibility service** (CAPABILITY-MATRIX dependency "Accessibility service"):
   - a system-wide tree with events and actions;
   - availability before sign-in;
   - focus and caret rectangles.
4. **Input service** (`NATIVE-INPUT-SERVICE.md`):
   - a filter stage (sticky, slow, bounce, repeat);
   - a pointer transform (gain, area capture);
   - switch, gaze and dwell sources;
   - injection only through the trusted AT role.

   The current ban on app input injection and focus changes stays for ordinary apps.
5. **Capability contract.**
   - A trusted assistive-technology role:
     - granted only from trusted UI;
     - secure fields hidden;
     - no self-approval of capability prompts;
     - revocation always reachable.

     Accessibility input and output paths are a documented attack surface (`INT-JANG2014`).
   - A separate app-facing preference capability (section 8).
6. **Profiles** (`PROFILES-AND-USERS.md`):
   - a per-user store for every axis;
   - applied at sign-in and on the trusted screen;
   - a pre-sign-in route, including a motion value before any account exists;
   - the first-run order: the motion choice before any large-field animation;
   - session state kept on lock;
   - AT state isolated between users.
7. **Privacy contract** (`docs/LOCAL-PRIVACY-CONTRACT.md`). Preference values and voice data are
   personal data. Models are on-device only, and enrolment data is local and deletable.

8. **Native palette plan** (`docs/design/NATIVE-FEASIBILITY.md` section 4 step 1;
   `ui/graphics/src/theme.rs`):
   - a contrast-floor rule for every `Palette` (today `MATERIAAL` and `HIGH_CONTRAST`; the
     Orbitaal palette is planned), checked over the worst-case backdrop of each material;
   - palettes per combination of polarity, contrast, light intensity and colour-perception aid.

   `GRAPHICS-G0.md` sets no palette or contrast rule today.

### 10.2 New contracts (none exists yet)

9. **Audio transport.** Audio capture and output. `drivers/virtio-snd` has a host codec only; its
   transport contract is only proposed.
10. **Media engine.** Caption, transcript and sign-video tracks for first-party and app media.
11. **Shell notification service.** Delivery modes, the security exemption, and timing values from
    `input-timing`. `DESKTOP-SHELL.md` mentions notifications but defines no service.
12. **Strings catalogue with variants.** Plain-language and explanation variants of system text.
    `TYPOGRAPHY.md` 8.5 covers localisation only. Today system strings are inline literals, almost
    all in `ui/native/src/render.rs`.
13. **SDK theming.** Axes are delivered to apps as tokens and components, not as raw values.
14. **On-device speech engine.** Synthesis for the screen reader and read-aloud, and recognition for
    voice control and automatic captions, without network capability (`docs/LOCAL-PRIVACY-CONTRACT.md`).
15. **Haptic output drivers.** Haptic alerts where hardware exists.
16. **Undo and version-history service.** Trash, undo and version history for destructive actions in
    the shell and Materiaal (matrix row 51). `DESKTOP-SHELL.md` lists Ctrl+Z and Ctrl+Y but defines
    no undo service.

The matrix rows that lead with feasible-now (21 rows) need no extension for their main part. Two
of them name further parts with their own status: the haptic part of never sound-only and of the
alert modality mix needs the haptic output contract, and the programmatic part of never sound-only
needs the accessibility service. The rows are mainly rendering, layout and wording rules: redundant
cues, lightness contrast, the stripe-texture bound, target sizes, keyboard and shortcut handling in the shell, and
plain-language rewording of the inline system strings. Among native code,
the existing per-app high-contrast flag already selects `theme::HIGH_CONTRAST`
(`ui/graphics/src/theme.rs`).

## 11. Innovative interaction principles (labelled)

| Principle | Purpose | Status |
|---|---|---|
| Light-blade alert and scan language: the window focus blade (components.css line 239, `.ds-window:is([data-state="active"], [data-state="maximised"])::before`) extended to scan highlights and visual sound alerts, within flash limits | Gives alerts and scanning a BaryonOS identity without sound or flashing | works now for window focus; technically feasible for alerts |
| Calm-but-present security: under any stimulus or motion setting, trust surfaces change form (no glow, no motion) but never presence | Keeps consent understandable in every combination | works now for motion (static ask ring); technically feasible for other axes |
| Invisible pointer assistance: gain reduction and area capture enlarge targets in motor space, so visuals are unchanged | Serves tremor without a denser or coarser default | technically feasible |
| Ability-based calibration: an optional one-minute motor calibration proposes target, timing and dwell values that stay adjustable | Personal fit without a diagnosis | technically feasible; user test required |
| One input-timing axis with live preview | One control serves switch, dwell, voice and fatigue | technically feasible |
| Paired spacing as one control | Prevents letter spacing growing alone | technically feasible (text.rs extension) |
| Numbers in three forms (exact, rounded word, proportion glyph), drawn by the system | Understandable numbers in every app | technically feasible; user effect to test |
| Adaptive layer outside the familiar path | Intent-aware help without unpredictability | technically feasible; user effect is future research |
| Semantic group scanning with optional gaze pre-selection | Faster switch access | future research |
| Compositor flash guard for third-party content | Protection without a setting; never switched off for system-drawn surfaces; a warned, non-persistent per-item reveal for third-party media is outside the baseline guarantee and not offered until a clinical safety review approves it | future research |
| "Where was I" on-device resume summary | Recovery after interruption | future research (on-device inference and privacy review open) |
| On-device personal voice model | Voice control for disordered speech | future research |

## 12. User testing with people with lived experience

Automated checks verify conformance only, not user benefit: contrast sweeps per axis combination,
CVD and greyscale simulation, frame-level flash and pattern analysis, reflow at every text size,
text-spacing overrides, accessibility-tree audits and keyboard walk-throughs. The matrix marks 60 of
92 rows "user test needed: yes". Priorities:

1. Screen-reader and braille walk-throughs of first-run setup, sign-in, Orbit, windows, permission
   prompts and settings.
2. Magnifier focus tracking.
3. Graded motion levels, the baseline large-field bounds (area and duration), boot and first-run
   setup with vestibular participants.
4. Contrast × light-intensity combinations with low-vision and light-sensitive participants, and the
   calibration of the stripe-texture comfort bound (row 21) with participants with migraine or pattern
   sensitivity.
5. The colour-perception remap by CVD type and severity.
6. Read-aloud with highlight, spacing, line length and the reading ruler with readers with dyslexia.
7. Numeric aids with low-numeracy and older users.
8. Interruption modes and stimulus steps with ADHD and autistic participants.
9. Targets, pointer assistance, keyboard filters, switch, dwell and voice with motor-impaired users.
10. Captions, alert modalities and sign video with DHH and Deaf signers.
11. Plain-language permission prompts with people with intellectual disabilities.

How to test:

- **Recruitment.** Recruit by the functional property tested (for example "uses a screen reader
  daily"), not by diagnosis. Involve disabled people in the design of the studies, not only as
  participants (`COG-SPIEL22`; Deaf community involvement for sign research, `INT-WFD2018`).
- **Format.** Accessible consent and materials, paid participation, and the participant's own AT
  where possible.
- **Measures.**
  - Task success, time and errors.
  - Comfort and symptom ratings, without treatment claims. Photosensitive participants are never
    exposed to flashing stimuli.
  - Preference, recorded as preference rather than efficacy.
  - Whether the result still feels like BaryonOS.
- **Privacy.** Health-related data stays minimal, on-device or offline, and is deleted after
  analysis.
- **Audit.** An independent human accessibility audit against EN 301 549 V4.1.1 clause 11 remains
  open. Conformance is claimed only per tested clause with stored evidence. A safety review with
  clinical expertise of the third-party flash-guard reveal is also open; it never exposes
  photosensitive participants to flashing stimuli. Until it approves the reveal, the reveal is not
  offered.

## 13. Prototype findings for UX-A11Y-DS (not changed here)

- **No automatic adoption of colour scheme or contrast.** `index.html` line 2 hard-codes
  `data-theme="dark"`. Only motion follows the device (tokens.css 427–431).
- **The contrast theme bundles polarity, transparency and motion.** Split it into independent
  axes.
- **Reduced motion is a single level equal to *none*.** Add *reduced*, which keeps fades.
- **Default loops never stop.** The ask halo, spinner, indeterminate progress and skeleton shimmer
  need the 5 s settle or the preload exemption.
- **The reduced-motion progress fallback is an 8 px full-contrast stripe**
  (components.css 359, 363), 17.4:1 to 21:1 against the track; on tracks wider than 80 px it falls
  inside the stripe-texture bound of row 21. Replace it with a solid track and a text status.
- **200 % is CSS `zoom`, not reflow** (components.css 14). Add the 125 % and 150 % steps and a
  real reflow test.
- **The internet place red may count as saturated red** under linearised values. Any future red
  pulse must be counted as a red flash.
- **The striped and indeterminate states and the boot sequence have no frame-level flash
  analysis.**
- **The boot convergence (`--motion-duration-boot`, 1400 ms, tokens.css line 80) must stay within
  the baseline vestibular limit.** No component uses it yet, and the prototype has no first-run
  motion choice.

## 14. Limitations

- This is a targeted, abstract-level review, not a systematic review. Several grades rest on one
  study or on populations other than the need (ADHD, dyscalculia, second-language readers).
- No operating-system-level study was found for most adaptations. "Insufficient" usually means
  "not studied", not "shown not to work".
- The prototype was checked by literal reads and selectors, not by an accessibility-tree run,
  screen reader or frame analysis. Native status comes from contract documents, not from running
  code.
- Paywalled ISO texts were not read in full. EN 301 549 V4.1.1 was read for the cited clauses only.
- The axis set and its combination rules are a design proposal. Its learnability, and whether the
  number of axes feels like needless complexity, needs testing.
- AI review, including this synthesis and its check script, is not evidence of user effect. The
  check validates structure and value sets only.
- The area and duration bounds of the vestibular safety limit (about a quarter of the display,
  320 ms) are own design rules, not evidence-derived thresholds; they need calibration with
  vestibular participants.
- Repair rounds 1 and 2 edited the generated matrix and this text with local scripts; the matrix was
  not regenerated from the dossiers. The scripts, snapshots and dossiers are not committed, so
  Appendix A is the committed trace.
- The basis sweep of repair round 2 is a judgement by the synthesis worker against the rule in
  section 1.3, made from abstracts and source notes, not full texts. It is not an independent
  review, and check.py can only enforce its structural floor (strong or moderate needs a resolved
  peer-reviewed source).
- The grading sweep of repair run 2 (synthesis rule 6, the scoped reminder grade, the stripe-texture
  bound) is likewise a judgement from abstracts by one worker, not an independent review. The
  1.5:1 stripe-texture ratio is an own design rule that needs a user test.
- Repair round 1 of run 2 found no study of caption display style with deaf and hard-of-hearing
  viewers and could not read the full text of `PER-PIEPENBROCK13`; both items are graded
  insufficient until such evidence is read.

## Appendix A. Traceability

**A.1 Matrix rows to dossier row ids.** Matrix row number, then the dossier ids it carries (p, c and
i are zero-based positions in the perception, cognition and interaction dossiers; a plus sign joins
merged rows; s0 was added by the synthesis and r1 in repair round 1):

1 p0; 2 p1; 3 p2; 4 p3; 5 p4; 6 p5; 7 p7; 8 p8+i34; 9 p9; 10 p10; 11 p11; 12 p13; 13 s0; 14 p12+i30;
15 i31; 16 i32; 17 i33; 18 i37; 19 p14; 20 p15; 21 p16; 22 p17+c6; 23 p18; 24 p19+i23; 25 p20; 26
p21; 27 p22; 28 p23; 29 r1; 30 p24+p25; 31 p26+c14; 32 p27; 33 p28; 34 c0; 35 c1; 36 c2; 37 c3; 38
c4; 39 c5; 40 c7; 41 c8; 42 c9; 43 c10; 44 c11; 45 c12; 46 c15; 47 c17; 48 c18; 49 c19; 50 c16; 51
c21; 52 c23; 53 c24; 54 c26; 55 c27; 56 c28; 57 i0; 58 i1; 59 i2; 60 i3; 61 p6+i5; 62 i4; 63 i6; 64
i7; 65 i8; 66 i9; 67 i10; 68 i11; 69 i12; 70 i13; 71 i14; 72 i15; 73 c20+i16; 74 i17; 75 i18; 76
i19; 77 i20; 78 i21; 79 i22; 80 i24; 81 i25; 82 i26; 83 i27; 84 i28; 85 i29; 86 p29+c29+i35; 87
p31+i36; 88 p30+c25; 89 c13; 90 i38; 91 i39; 92 c22+i40.

**A.2 Standards as fetched on 15 November 2026.**

- WCAG 2.2 Recommendation HTML (https://www.w3.org/TR/WCAG22/), 512,457 bytes, sha256
  `6e3c5fe397257cae509a2fb4752b73062cf8cbeb92c2cec618989b17e4cf7057`. The synthesis parsed 86 success
  criteria with their levels. Every criterion cited in the matrix exists in that list with the level
  the matrix gives: 1.2.2 (A), 1.2.4 (AA), 1.2.6 (AAA), 1.3.1 (A), 1.3.3 (A), 1.4.1 (A),
  1.4.3 (AA), 1.4.4 (AA), 1.4.6 (AAA), 1.4.8 (AAA), 1.4.10 (AA), 1.4.11 (AA), 1.4.12 (AA),
  1.4.13 (AA), 2.1.1 (A), 2.1.2 (A), 2.1.4 (A), 2.2.1 (A), 2.2.2 (A), 2.2.4 (AAA), 2.2.6 (AAA),
  2.3.1 (A), 2.3.3 (AAA), 2.4.1 (A), 2.4.3 (A), 2.4.7 (AA), 2.4.11 (AA), 2.4.13 (AAA), 2.5.1 (A),
  2.5.2 (A), 2.5.3 (A), 2.5.5 (AAA), 2.5.7 (AA), 2.5.8 (AA), 3.1.3 (AAA), 3.1.4 (AAA), 3.1.5 (AAA), 3.2.1 (A),
  3.2.3 (AA), 3.2.4 (AA), 3.2.5 (AAA), 3.2.6 (A), 3.3.1 (A), 3.3.3 (AA), 3.3.4 (AA), 3.3.6 (AAA),
  3.3.8 (AA), 3.3.9 (AAA), 4.1.2 (A), 4.1.3 (AA) (50 criteria).
- EN 301 549 V4.1.1 (2026-09) PDF from etsi.org, 3,971,655 bytes, sha256
  `636914c5c18dc58cf7716b8935502f1471de05caf5e1e6ae32469eed929cb824`. Every clause number cited in
  the matrix exists among the headings of that version (for ranges, both ends were checked): 4.2.1,
  4.2.2, 4.2.3, 4.2.9, 4.2.11, 5.1.3, 5.1.4, 5.1.7, 5.2, 5.3, 5.6.2, 5.7, 5.8, 5.9, 6.4, 7.1.1,
  7.1.2, 7.1.4, 11.1.2.2, 11.1.2.4, 11.1.3.1, 11.1.4.1, 11.1.4.3, 11.1.4.4, 11.1.4.10, 11.1.4.11,
  11.1.4.12, 11.2.1.1, 11.2.1.2, 11.2.1.4, 11.2.2.1, 11.2.2.2, 11.2.3.1, 11.2.4.3, 11.2.4.7,
  11.2.4.11, 11.2.5.1, 11.2.5.3, 11.2.5.7, 11.2.5.8, 11.3.3.8, 11.4.1.2, 11.4.1.3, 11.5.2.1,
  11.5.2.4, 11.5.2.5, 11.5.2.13, 11.5.2.14, 11.5.2.15, 11.5.2.17, 11.6.2, 11.7 (52 clauses).
- Understanding SC 2.2.2 (`W3C-U222`): sha256 prefix `7780b8cdbca2f3`.

**A.3 Repair history.**

- Repair round 1 (15 November 2026): added the vestibular safety row (r1) and the `Safety:` tags,
  regraded `COG-RELLO13` to limited, and corrected native and prototype cells.
- Repair round 2 (15 November 2026): applied the basis rule (section 1.3, rule 5) to every row graded
  limited or above, with the result in the table after section 1.3; regraded rows 14, 20, 44, 57,
  77, 84 and 91 to insufficient and rows 59 and 75 from moderate to limited; added `PER-RUBIN89` and
  `PER-FERNANDES16`; made native status precise for rows 6, 31, 32, 42, 52, 54, 56, 78 and 79;
  gave the wider-text setting its English axis name; stated that the vestibular limit leaves light,
  opacity and colour free and that `rich` only adds intensity; and added the peer-reviewed floor
  rule to check.py.
- Repair run 2 (15 November 2026): fixed the findings of the run 1 verifier without regenerating the
  matrix. Added synthesis rule 6 (contradicted bar); regraded the section 4 items "all notifications
  off" and "touch as easier" from contradicted to insufficient, kept dark mode contradicted as a
  universal claim only, split colour-correcting glasses (contradicted for a narrower claim) from
  software filters (insufficient), and reduced section 4 grade cells to one scale word; scoped the
  strong grade of row 53 to scheduled reminder prompts after brain injury; gave row 21 an explicit own
  bound (more than five pairs at most 1.5:1), pointed its seizure part to row 25, moved it to the
  other-context basis class and set its user test to yes; removed a duplicate clause in row 24; added
  evidence notes to rows 4, 5, 22, 26, 28, 34 to 37, 39, 41 and 45; re-read the `COG-FITZ19` abstract;
  corrected the section 11 selector; and added a named-file existence check to check.py. No matrix
  grade, tier or row count changed.
- Repair round 1 of run 2 (15 November 2026): fixed the findings of the run 2 verifier without
  regenerating the matrix. Regraded row 38 (read-aloud) from moderate to strong for its core part and
  removed the declared deviation (rule 4); rows 3 (CVD-safe palettes, simulation only) and 76
  (caption style; `INT-BURNHAM2008` tests rate and text reduction, now cited for the barrier and
  section 3.14 Q4) from limited to insufficient; and the section 4 item "dark mode as a universal
  legibility fix" from contradicted to insufficient (full text of `PER-PIEPENBROCK13` not readable,
  sample size unknown). Untagged row 21 as a baseline comfort rule with a calibration condition and
  removed its safety topic from check.py; put the row 26 reveal outside the baseline guarantee, off
  until a clinical safety review; named the file or document in the native status of rows 33 (`ui/graphics/src/theme.rs`), 51 and 82 and
  added the undo and version-history service (section 10.2 item 16); marked row 85 as guidance on the
  screen-reader axis; and reconciled the row 10 barrier with `PER-RUBIN89`. No tier or row count
  changed. The number of limited rows fell from 53 to 51 and of insufficient rows rose from 35 to 37;
  strong rose from 1 to 2 and moderate fell from 2 to 1. Re-read the
  abstracts of `COG-WOOD17` and `INT-BURNHAM2008` (OpenAlex API) and `PER-PIEPENBROCK13` (PubMed
  23654206), and confirmed the `INT-BURNHAM2008` metadata by DOI (lookup_reference).
