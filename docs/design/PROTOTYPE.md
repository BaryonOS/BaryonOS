# BaryonOS connected prototype journey (UX-E)

## 1. Status and evidence level

The journey at `ui/design/prototype/index.html` is a **browser simulation** of BaryonOS in the Orbitaal design language (ds-1.1). It is design evidence, not a working operating system. Nothing on the page enforces, stores, synchronises, encrypts or recovers anything; all state lives in the page and is lost on reload.

- Slices built: **A (shell)**, stations 1-4 (base commit `ec459716`), and **B (data and consent)**, stations 5-8 (base commit `90a9d14b`; sections 12 and 13). Plan: `build/design-evidence/uxe/PLAN.md` (ignored build directory).
- Evidence classes are kept apart throughout this document:
  - **Objective**: checks with a stated mechanism and result (section 9).
  - **Design judgement (AI opinion)**: reasoned choices by the writing agent, marked *[judgement]*.
  - **[untested hypothesis]**: claimed effects on users, which need human task trials (U06) and are not demonstrated.

## 2. How to run

| Purpose | Command or URL |
|---|---|
| Journey | `http://127.0.0.1:8791/prototype/index.html` (the server on :8791 serves `ui/design`) |
| Theme for screenshots | `?theme=dark`, `?theme=light` or `?theme=contrast` |
| Test hook (sweep only) | `?test=1` exposes `window.__journey` (stations, go, setSim, show, setRoot); it is not part of the simulated product |
| Journey sweep (E10) | `http://127.0.0.1:8791/prototype/sweep.html` (`?only=1440`, `320` or `720`) in a focused window; headless: `node ui/design/prototype/tests/run-sweep.mjs OUT.json [only=320]` (tracked runner; Chrome `--headless=new` over CDP with focus emulation; without focus the sweep stops with `NO_FOCUS`; exit 0 only with zero failures; set `CHROME` when Chrome is not at `/opt/google/chrome/chrome`) |
| Unit and static tests | `node --test --experimental-test-isolation=none ui/design/prototype/tests/*.test.mjs` |
| Keyboard and pointer walk | Reproducible route: open the journey in a focused, headed browser and follow Sneltoetsen (F1) and the Rondleiding. The CDP scripts used for section 9 (`build/design-evidence/uxe/A-writer`, `A-repair-1`, `A-repair-2`) live in the ignored build directory and are local evidence only, not repository tooling |

## 3. Journey and stations

The page has two visibly distinct layers. The **prototype band** (header, dashed edge, "Simulatie · geen echt besturingssysteem") sits outside the OS and holds Naar BaryonOS (bypass), Rondleiding, Simulatie, Status and Sneltoetsen. The **OS frame** (`main`) holds the simulated system. Routes are `#/<station>`. The browser's Back and Forward buttons only open or focus; they never close a window, discard a draft or reset simulation state.

| n | Station | What it shows | Slice |
|---|---|---|---|
| 1 | Aanmelden | Lock screen (clock, date, no notification content), profile choice (Noor, Sem, Andere gebruiker), secret entered only on the session-service surface (Kern material, opposite theme, band naming the system as owner), recovery-key route with a new password, session start, Eerste start (five steps), system screen (Ctrl+Alt+End) | A |
| 2 | Bureaublad, Orbit en menu | Verder waar je was, Orbit (Menu, Zoeken, pinned and running apps, status, clock), Menu, Snelle instellingen, Meldingen list | A |
| 3 | Apps en vensters | Notities (Tuinplan, Ctrl+S), Rekenmachine, Bestanden (organised in slice B), Instellingen (Weergave, Toegankelijkheid); minimise, maximise, snap, close, window menu, switcher | A |
| 4 | Zoeken | Grouped results over apps, files, settings and actions, plus the on-device assistant row | A |
| 5-8 | Bestanden ordenen, Versies, Toestemming, Instellingen | See section 13 | B |
| 9-10 | Meldingen, Herstel | Shown in Rondleiding as "Volgt" entries (`aria-disabled`, focusable, with a reason line); never links or routes | C |

Locking keeps the session: windows and drafts stay. Signing out with an unsaved Notities draft opens a dialog with three equal answers, Opslaan, Niet opslaan and Annuleren, with focus on Annuleren.

## 4. States per station and simulation controls

Slice B stations: section 13.2.

| Station | Empty | Loading | Error | Success | Simulatie group |
|---|---|---|---|---|---|
| Aanmelden | Eerste start (no account yet) | Sessie starten (mark converges once, indeterminate bar; focus on the heading) | "Dat klopt niet…" with `aria-invalid`, focus kept, attempts left before a delay; the note beside it switches to "In deze simulatie wordt elke invoer afgewezen…" so the page never says that every input works next to a refusal. Choosing Sem shows "Alleen het profiel van Noor is in deze simulatie ingericht." as the note and, on submit, "Het profiel van Sem is in deze simulatie niet ingericht. Kies Noor om verder te gaan."; choosing Andere gebruiker hides the secret field (no PIN for an unknown account) and says that signing in with another account (account name and password) follows in a later version | Desktop | Aanmelden: Volgende invoer Klopt / Klopt niet; Eerste start nabootsen; Nu vergrendelen |
| Bureaublad | No recent files (`T.desktop.empty`) | n/a (no fetch) | n/a (no fetch); an unreachable Thuis-NAS is shown in the privacy instrument, the Orbit status and Snelle instellingen | Greeting, three-node privacy instrument, recent work on a trajectory with kind tiles and place badges | Bureaublad: Verder waar je was Gevuld / Leeg; Thuis-NAS: Bereikbaar / Niet bereikbaar |
| Apps en vensters | No windows: switcher says "Geen geopende vensters" | Skeleton body (Traag holds it) | "<App> kon niet starten. Er is niets gewijzigd." in the full `.ds-empty` anatomy (orbit with a device satellite, title, hint with its label, Opnieuw proberen) | App body; a reopened Notities without a grant shows "Tuinplan is niet geopend" (section 5) | Apps starten: Normaal / Traag / Mislukt |
| Zoeken | No query: suggestions | "Index wordt bijgewerkt…" with partial results; with no match "Nog niets gevonden… De bestandsindex wordt bijgewerkt" | Files unavailable; apps and settings still shown; "Bestanden kunnen nu niet worden doorzocht." replaces the spelling hint; Opnieuw proberen | Grouped results with a preview column (wide frames); no-result state with a spelling hint only when the index is ready | Zoeken: Index Klaar / Bijwerken / Niet beschikbaar |

The simulation controls appear only in the band's Simulatie panel: an `aside` labelled "Simulatie, geen echte actie". It opens with that note and has one `fieldset` with a `legend` per group. The Thuis-NAS group (Bereikbaar / Niet bereikbaar) acts out a network event; inside the OS frame, Snelle instellingen only reports the NAS state read-only, with its label and the pointer "Na te spelen in het paneel Simulatie" (repair 1 moved the former switch out of the frame). Errors state what happened, what is safe and what to do next, without apology or exclamation marks; an outage or a running index update is never presented as a spelling problem.

## 5. Simulated versus planned

Every simulated function carries one of four labels, either beside it as a chip (`.jr-feas`: neutral colours, icon plus text) or in the per-station Status table (band, Status). Both are built from `STATIONS[*].feas` in `journey/state.js`, which the sweep also reads; the visible row text lives in `strings.js` (`T.journey.more.statusRows`) since repair 1.

| Label | Meaning |
|---|---|
| Werkt nu in dit prototype | Behaves as described within this page |
| Technisch haalbaar, nog niet gebouwd | Specified (DESKTOP-SHELL, PROFILES-AND-USERS); not built natively |
| Toekomstig onderzoek | Open research question |
| Gesimuleerd | Acted out with sample data; no real effect |

Copy rule, enforced by `journey-static.test.mjs`: no journey string claims encryption, synchronisation, protection, enforcement or recovery as fact unless the sentence says "gesimuleerd" or "gepland". Existing strings that over-claim are not shown bare:

- `DATA.notifications[1]` ("versleuteld") appears in Meldingen beside a "Gesimuleerd" chip.
- The privacy status ("Op dit apparaat") carries a "Gesimuleerd" chip.
- NAS place badges in Bestanden carry a "Gesimuleerd" chip.
- `T.bestanden.storageNas` and `T.instellingen.opslag.nasEncrypted` are not used in slice A.

Consent (slice A, superseded in slice B): slice A relied on the `data.js` page-load seed (Notities, readWrite, Tuinplan). Since slice B the owner file-open model of 23 November 2026 applies; see section 12. The bootstrap ends the page-load seed through `grants.logout()` (no user session exists before sign-in), and files gain access only by an explicit user open.

## 6. Familiar conventions (ADR-058) and shortcuts

| Convention | Choice | Source |
|---|---|---|
| Dock | Orbit, a floating capsule at the bottom centre: Menu, Zoeken, apps, status, clock | DESKTOP-SHELL §2, NAMING-AND-IA §5 |
| Launcher | Menu with search field, Vastgezet, Verder waar je was, Alle apps; footer with user, Vergrendelen, Aan/uit | DESKTOP-SHELL §2 |
| Window controls | **Right side**: Minimaliseren, Maximaliseren / Vorige grootte, Sluiten; app icon at the left opens the window menu; double-click on the title maximises | DESKTOP-SHELL §2, DESIGN-SYSTEM §9 |
| Arranging | Window menu: Minimaliseren, Maximaliseren, Links/Rechts vastzetten, Sluiten (keyboard alternative to dragging, WCAG 2.5.7) | DESKTOP-SHELL §2 |
| Status and clock | Status (three nodes: device, NAS, internet; internet is off unless a running app holds a connect grant, section 13.6) opens Snelle instellingen, which reports the NAS state read-only; clock opens Meldingen, whose heading takes focus so a held or repeated Enter cannot reach Alles wissen | NAMING-AND-IA §5 |
| Sign-in | Lock screen, any key or click shows sign-in, last user preselected, "Pincode of wachtwoord vergeten?" leads to the recovery key | PROFILES-AND-USERS §3 |
| Secure attention | System screen with Vergrendelen, Andere gebruiker, Afmelden, Taakbeheer (feasible, not built) | PROFILES-AND-USERS §3 |

The browser reserves several native keys, so the prototype maps them as follows (listed in Sneltoetsen, F1):

| Action | Native | Prototype |
|---|---|---|
| Menu | OS key | Ctrl+Esc |
| Zoeken | OS key and typing | Ctrl+K |
| Switch windows | Alt+Tab | Alt+` (Shift for back; releasing Alt chooses); matched on `KeyboardEvent.code` Backquote |
| Close window | Alt+F4 | Alt+W (Alt+F4 when it reaches the page) |
| Snap, maximise, restore | OS key+arrows | Alt+Shift+arrows |
| Window menu | Alt+Space | Shift+F10 on the title or the title-bar app icon |
| Regions | F6 | F6 / Shift+F6 (active window, Orbit, band) |
| Lock | OS key+L | Ctrl+Alt+L |
| System screen | Ctrl+Alt+Delete | Ctrl+Alt+End |

Ctrl+W, Ctrl+Tab and Ctrl+L are deliberately unbound, because the browser takes them. A keydown with `repeat` never activates a command (ds-1.1 B8; `journey/keys.js`).

## 7. Identity, metaphors and motion

- Trusted surfaces (session-service surface, system screen) use Kern material in the opposite theme and re-theme at once on every theme change. The Triade mark (`ui/design/brand`, referenced, never edited) appears on the lock screen, the session start and the Menu button; the contrast theme uses the mono-ink mark. Place colours appear only as `.ds-place`/`.ds-node`, always with shape, icon and text.
- **Session start** *[judgement]*: the Triade mark converges once (`--motion-duration-boot`, 1400 ms) to show that the workspace is assembling. Under reduced motion: the resting mark, the text and the indeterminate bar.
- **Window grows from its Orbit item and folds back into it on minimise** *[judgement]*. This shows where the window went (`--motion-duration-base`, orbital easing). Under reduced motion: an instant change, the Orbit item's state bar and a polite live message ("… geminimaliseerd, terug te vinden in Orbit").
- Popovers rise 12 px (`--motion-duration-base`); there is no decorative motion elsewhere. Theme changes are not animated by the journey; ds components keep their short `--motion-duration-fast` colour transitions (about 0.14 s), which is not a full-screen cross-fade. Axis changes go through a 400 ms queue (`journey/throttle.js`), so holding an arrow key cannot flash the screen.
- **Bureaublad composition (repair 1)** *[judgement]*: following the Orbitaal v2 board, the desktop shows a display-size greeting, the three-node privacy instrument (`.ds-privacy`: Dit apparaat lit, Thuis-NAS lit or off, Internet off unless a running app holds a connect grant (B repair 3, section 13.6); shape, icon and text) and recent work as a trajectory, newest first, with kind tiles (Tekst, Tekening, Lijst, Rekenblad). The lit blade dot on the trajectory marks where the user was last. A neutral blade-light field (no place colour) sits behind the desk. In stack mode the instrument is hidden and the Orbit status carries the same three nodes. All of it is static, so the reduced-motion path is identical.
- **Search preview column** *[judgement]*: on wide frames the highlighted result is described beside the list (kind, collection, place with shape, icon and text, last change); in stack mode the column is hidden.
- Whether these metaphors improve understanding is an **[untested hypothesis]** for U06. The Orbitaal identity of the desktop after repair 1 is a design judgement, not a verified finding.

## 8. Components

- Reused ds-1.1 components: `.ds-btn`, `.ds-input`, `.ds-field`, `.ds-fieldset`, `.ds-seg`, `.ds-radio`, `.ds-check`, `.ds-switch`, `.ds-row`, `.ds-place`, `.ds-node`, `.ds-orbit`, `.ds-taskitem`, `.ds-taskitem-badge`, `.ds-window*`, `.ds-menu*`, `.ds-dialog*`, `.ds-empty*`, `.ds-skeleton*`, `.ds-progress`, `.ds-banner`, `.ds-kbd`, `.ds-live`, `.ds-vh`.
- Journey-only parts live in `ui/design/prototype/journey.css` with the `.jr-` prefix. They are candidates for O-K1 and are not appended to `components.css`: band and panels, feasibility chip, lock screen, session surface, user list, step list, desktop head and plate (`.jr-desk-head`, `.jr-plate`), trajectory list (`.jr-cont-list`, `.jr-cont-item`), kind tile (`.jr-thumb`), search preview (`.jr-search-body`, `.jr-preview`), neutral desk field (`.jr-field`), window layout, launcher and popovers, switcher, settings layout. Repair 1 reuses `.ds-privacy` (three-node instrument) and the full `.ds-empty` anatomy (orbit, satellite, title, hint, actions) through one helper, `emptyState` in `journey/apps.js`.
- `.ds-*` rules are used only in a `.jr-*` or `:where()` context. `button.ds-row` drops the user-agent button face at zero added specificity, so the ds hover and selection rules still apply.
- Gallery regression: `gallery.js`, `index.html`, `components.css` and `icons.js` are byte-identical to the baseline hashes in `build/design-evidence/uxe/gallery-baseline/SOURCES.sha256`. A dark gallery render at 1440x2400 differs from the baseline in 0 pixels. `strings.js` differs only by the appended `journey` block; outside the block the file hashes to the `ec459716` value (`journey-static.test.mjs`).
- The permission renderer has not been extracted yet. That happens in slice B.

## 9. Accessibility checks performed (slice A; slice B in section 13.5)

| Check | Mechanism | Result |
|---|---|---|
| Unit and static tests | `node --test --experimental-test-isolation=none ui/design/prototype/tests/*.test.mjs` | 49 tests: 29 contract (unchanged) + 20 journey, including the held-key guard (`swallowRepeat`) and the Simulatie groups; 0 fail |
| DS checker | `node docs/verification/a11y-ds-20261115/check.mjs` | exit 0, 0 failures |
| Matrix checker | `python3 -B tools/check_a11y_matrix.py <worktree>` | exit 0 |
| Journey sweep (E10) | `sweep.html` driven over CDP (`ui/design/prototype/tests/run-sweep.mjs`) with `Emulation.setFocusEmulationEnabled`. The sweep refuses to run when `document.hasFocus()` is false (`NO_FOCUS` crash), because 2.4.11 and `:focus-within` results depend on focus; earlier unfocused runs reported 349 spurious `obscured` cells. Covers 1440x900, 320x640 and 720x450 (200 % of 1440x900) × dark, light, contrast, plus C1 and C2 at 1440 and 320, and Schaal 200 % (`data-scale="2"`) at 1440. Every built station and state plus the extra views Menu, Snelle instellingen, Meldingen, system screen, Herstelcode route, Instellingen, Bestanden, Snelle instellingen with Thuis-NAS unreachable, Notities without a grant and all band panels open. Rules: overflow (only `.jr-table-wrap` may scroll sideways), unexpected horizontal scroll inside any panel or popover, sibling overlap of text-bearing in-flow parts (per line box for inline text), clipping, feasibility labels, undefined `var(--x)`, region order, target size, obscured focusables, occlusion of the element holding focus after each state, a dispatched repeated Enter and Space on every control that must come back `defaultPrevented`, text contrast, animation length, live regions, consent coverage (granted Tuinplan body in the success cell, never without a grant), and since repair 2: `pointer-blocked` (a control whose centre hits a layer that paints nothing is a failure; a control behind a painted surface such as a window or popover is the warning `behind-surface`, and `covered-until-focus` / `raised-on-focus` record controls that a focus lifts or raises), `clip-y` (a hidden or clipped box whose content runs past its bottom edge) and `lost` (a control or text element whose centre lies outside an overflow-hidden or -clip ancestor with no scroll container in between; this also covers the OS frame, desktop, lock screen and windows that the clip rules exempt), `null-text` (a stringified `null`, `undefined` or `NaN` in text or accessible names), and a settle wait so that the Apps en vensters error cell measures the error body, not the loading skeleton | The upgraded sweep run against the unrepaired slice reported 17 hscroll, 58 overflow, 279 overlap, 21 focus-obscured and 17 658 held-key failures (`A-repair-1/sweep-before.json`). Repair 1 final run: 350 cells, 0 failures. Repair 2: with the pre-repair desktop column restored, the `lost` rule reports 504 failures at 320x640 (rows 3 and 4 of Verder waar je was outside `#jr-desk`, `A-repair-2/sweep-red-320.json`); an interrupted earlier repair-2 run recorded 132 `pointer-blocked`, 10 `null-text` and 10 `clip-y` failures against the unrepaired slice. Final run: 378 cells, 0 failures, warnings only of the kinds `behind-surface`, `covered-until-focus` and `raised-on-focus` (`A-repair-2/sweep-final.json`) |
| Keyboard walk | CDP `Input.dispatchKeyEvent` (verifier walk `kbd.mjs`, rerun in A-repair-1) | Completes: any key wakes sign-in, Enter signs in (focus on the session-start heading, then Menu), Ctrl+K, search with arrows and Enter, Alt+Shift snap and restore, Alt+W, F6 cycle (now including the band's Naar BaryonOS), F1, Ctrl+Alt+L. Repro `A-repair-1/repro.mjs` with real auto-repeat (`autoRepeat: true`): held Enter on the lock-screen button opens sign-in once and no longer submits the empty secret; held Enter on the clock opens Meldingen once and clears nothing (focus on the Meldingen heading); held Enter on Rekenmachine 7 gives "7"; a held quick-settings switch toggles once; Escape after Ctrl+K or Ctrl+Esc from the Notities text field returns focus to that field, and after Alt+` to the window element it came from; Eerste start nabootsen with an unsaved draft asks Opslaan / Niet opslaan / Annuleren first, and a reopened Notities then shows no Tuinplan text; the Menu search field at 320x640 is not occluded |
| Reduced motion | CDP `Emulation.setEmulatedMedia` prefers-reduced-motion. A synchronous probe opens an app and the Zoeken popover, then lists running animations: `none`. The same probe without emulation lists `jr-rise` | pass |
| Theme emulation | Explicit `data-theme` on the root (sweep, `?theme=`); device preference via `matchMedia` at boot | — |
| Screen reader structure | CDP `Accessibility.getFullAXTree` at the desktop: banner, main "BaryonOS (simulatie)", navigation "Orbit, dock", headings per window | saved as `ax-desktop.txt`; **no screen reader was run** |
| Live regions | One polite station region (`#jr-live`); no `assertive`; `role="alert"` only in a user-caused app start error | sweep rule 8 |

Defects found and fixed in repair 2 (verifier round 1), checked with a real mouse over CDP `Input.dispatchMouseEvent` including `mouseWheel` (`A-repair-2/probe-r2.mjs`) at 1440x900 dark, 320x640 light and 720x450 contrast:

- The transparent window layer covered the whole desktop, so a mouse click on a Verder waar je was row did nothing. The layer now passes the pointer through (`pointer-events: none`) and only its windows take it. Every row now opens its document by mouse when no window lies over it; a row under an open window is covered by that window, as on any desktop.
- The session-service card clipped its band, title, submit button and recovery link at 320x640 and 720x450 (`overflow: hidden` on a height-bounded card). The card now grows with its content (`overflow: clip` for the radius only) and the sign-in layout scrolls; the band, title, label, submit and recovery link are reached by wheel and clicked by mouse.
- On the desktop, rows 3 and 4 lay under the Orbit and beyond the desk edge at 320x640, unreachable by any input. Verder waar je was now scrolls in its own column, which ends above the Orbit.
- The search preview showed a literal "nullnull" for apps, settings and actions; it now shows the kind only.
- Choosing Sem or Andere gebruiker still showed "In deze simulatie werkt elke invoer." beside the refusal; the note and the refusal now explain the single prepared profile, and Andere gebruiker no longer offers a PIN field.
- The window-close dialog repeated its heading as its description; it now has the heading "Niet opgeslagen wijzigingen" and a body that says what Opslaan, Niet opslaan and Annuleren do.
- A search action (for example Licht thema) was announced as the result count; `#jr-live` now says "Licht thema ingesteld.". Browser Back from Bureaublad to Aanmelden locks the session as before, and the station message now adds "Vergrendeld. Je apps blijven open." instead of replacing it.
- A focused window title drew a box that read as a text field; it now shows an underline in the focus colour.

Defects found and fixed in repair 1 (verifier round 0):

- Auto-repeated Enter or Space activated controls (sign-in on one held press, Alles wissen, repeated digits, switches). A window capture-phase guard (`swallowRepeat` in `journey/keys.js`, the gallery's B8 rule extended to single-line fields) now swallows them.
- Popovers were sized from the viewport (`100cqh` on an inline-size container) and rose under the band at 320 px; they are now bounded by the OS frame, and the focused field is kept in view after the rise.
- Focus returned to an Orbit button after a shortcut popover; it now returns to the field the user came from. Focus fell to the page during session start and after clearing all notifications; both now land on a heading.
- A new window took focus while still scaled into its Orbit item, so the page scrolled to the wrong place at Schaal 200 %; focus now precedes the grow animation.
- Place badges ran over dates in rows (fixed list columns); journey rows now wrap. Rondleiding rows were 26 px too wide (content-box); fixed.
- The launch-failure state lacked the `.ds-empty` orbit, so its title stood one word per line and the label sat under the button.
- First load: the band now starts with Naar BaryonOS (bypass block, WCAG 2.4.1), and the open Rondleiding is limited to 30 % of the viewport height so the lock-screen clock and prompt are in view at 1440x900 on first load (page height 928 px, measured with `A-repair-1/fold.mjs`).
- Spelling advice during a search outage; a sign-in note that contradicted the refusal; a NAS switch acting out a network event inside the OS frame; Notities showing Tuinplan after its grant ended (also after a band sign-out, which kept the draft); sign-out from the band now asks about an unsaved draft and ends temporary grants (`grants.logout()`).

Defects found and fixed during the slice:

- User-agent button face on row buttons (contrast 2.5:1).
- Desktop rows focusable behind windows. The card is now lifted while it has focus.
- Elements behind an open popover could take focus. A popover now closes when focus moves outside it.
- Window-menu icon below 44 px under `data-targets=44`.
- Orbit items squeezed at 320 px with 44 px targets.
- Stack mode overridden by inline window positions.
- A literal "null" in Orbit.
- Search names squeezed into the icon column.
- Double-handled Shift+F10.
- Niet opslaan did not discard the draft.
- Popover focus returned to Menu instead of the opener: the Orbit is re-rendered, so the trigger is now kept as a key.
- Rondleiding or Back while locked opened windows behind the lock screen; the target is now kept as pending.
- The Rondleiding panel pushed the shell off-screen; it now collapses after the first sign-in.
- Meldingen dismiss buttons shrank in their row.

Not run: a real-key check (a person or `xdotool` at a headed browser; CDP bypasses browser accelerators), a screen reader with real assistive technology, human task trials (U06) and native QEMU.

## 10. Representative apps and the S7/S8 host seam

These are planned for slice C.

## 11. Open items

- **Real-key check not run.** It is not known whether Ctrl+Esc, Ctrl+K, Alt+`, Alt+W, Alt+Shift+arrows, F6, Ctrl+Alt+L and Ctrl+Alt+End reach the page in a headed browser on each host OS.
- **Browser-reserved keys** (Alt+Tab, Alt+F4, Meta, Ctrl+Alt+Delete) are stand-ins. Native behaviour follows DESKTOP-SHELL and WORKSPACE-REQUIREMENTS D5.
- **Stale existing shortcut strings** (`T.shortcuts.close` Ctrl+W, `T.shortcuts.switcher`/`T.window.switcherHint` Ctrl+Tab, `T.shortcuts.lock` Ctrl+L) are unchanged and unused. Correcting them needs a lead-owned string revision.
- **Kern for the session surface** extends DESIGN-SYSTEM §5 ("permission prompts and other shell questions") to the session-service surface, as a reasoned extension that needs owner confirmation.
- **Single session**: Sem and Andere gebruiker get an explanatory message, and Andere gebruiker has no account-name step yet; concurrent sessions and that step are feasible, not built.
- **Window size**: windows use one fixed size, so Rekenmachine shows empty space below its keypad. Sizing from the app registry's `minSize` or content is not built *[judgement: low priority for slice A]*.
- **Back to Aanmelden locks**: Back from Bureaublad locks the session (announced, nothing lost). Whether people expect Back to stay on the desktop is an [untested hypothesis].
- **Desktop column shadow**: the scrolling Verder waar je was column clips the outer edge of the plate shadow by design; visible only as a slightly shorter shadow *[judgement]*.
- **Journey-only components** are O-K1 candidates. Promoting them to `.ds-*` needs gallery cards and `check.mjs` coverage.
- **Pointer dragging and resizing of windows** are not built. The keyboard and window-menu alternatives exist.
- **Contrast of gradient fills** (primary buttons) is skipped by the sweep and covered per component by the gallery's `?measure=1`.
- **Reload** loses all state (stated in the Status panel).
- **Sweep mechanism**: the sweep needs real or emulated document focus and refuses to run without it. Its held-key rule proves the guard with dispatched events; native activation under auto-repeat is proven only for the cases in `repro.mjs`.
- **Opening a document again** after its grant ended is an explicit user open (Bestanden, Zoeken, Verder waar je was, Menu recent) and seeds a new temporary read (section 12); the UX-C prompt is reserved for requests the user did not hand over.
- **Secret-rule test** greps `session.js` source; a DOM-free behaviour test that drives `initSession` with a stub document and spies on bus, store and console is not written yet.
- **Desktop identity (E6)** after repair 1 is a design judgement; no user comparison and no owner review has taken place.

## 12. Slice B: owner file-open model (E11)

Source: owner decision of 23 November 2026 (`build/design-pipeline-checkpoint/owner-decision-file-open-20261123.md`, binding for slices B and C). It supersedes the build plan rule that seeded `readWrite` when a file was opened. Evidence level: prototype behaviour in this page ("Werkt nu in dit prototype"); enforcement by BaryonOS itself is "Technisch haalbaar, nog niet gebouwd". The first label is a visible chip next to the Notities mode chip; both labels and the model sentence sit in the "Over bestandstoegang" disclosure beside it, in the grants view and in the Status table of station 7 (repair 1 replaced the dense two-label sentence above the text).

### 12.1 Model as built (repair 1)

| Rule | Implementation |
|---|---|
| (a) An explicit open grants temporary read for that one document, without a prompt | `journey/fileaccess.js` `shell.open(app, doc, ev)` calls `grants.seedTemporary({kind: 'ongoing', app, right: 'read', data: doc, destination: null, scope: 'document'})` only when `ev` passes the broker's `isUserGesture` predicate. The default predicate (`realInput`) accepts only a real `Event` with `isTrusted === true`; a plain object such as `{isTrusted: true}`, an event made by script or a missing argument is refused. The page instance (`journey/access.js`) adds one exception: the `?test=1` token `TEST_OPEN`, which is `null` on the normal page. Every user open route passes its input event to `openFile(id, ev)`: Bestanden Openen, Bestanden Openen met (a shell-drawn dialog that lists the apps), a Zoeken result, Verder waar je was, Menu > Verder waar je was (the repair-1 regression fix) and the new shell-drawn Openen... chooser in the Notities window bar and in its empty state. |
| Opening never grants write | Notities shows the text read-only (`readonly` textarea, mode chip "Alleen lezen") until an edit action. |
| (b) Write only after an explicit user edit or save action on a trusted surface | Openen..., Bewerken and Opslaan are drawn by the shell in the window bar, outside the app content. Since B repair 2 a window bar that holds these actions has two rows inside the frame: the title row (window menu, title, window controls) and the shell's action row beneath a divider; the bar grows with its content and does not shrink (`flex-shrink: 0`), so the actions never hang over the desktop or into the app content at any width, theme or tier-2 setting (sweep check 10, `barprobe.mjs`). Their click handlers pass the real input event to `shell.edit`, which applies the same `isUserGesture` predicate inside the broker; since B repair 3 the shell's `editAction` also refuses any event whose `currentTarget` is not a control inside the frame's action row (`.jr-frame-acts`). Ctrl+S never escalates (lead decision 3 of 23 November 2026, strict reading of E11 b): on a document the app holds read-only, a real Ctrl+S keypress, whatever element has focus, moves focus to the shell-drawn Bewerken control and announces politely that the document is read-only and that Bewerken enables changes; only activating Bewerken (or Opslaan) grants `readWrite`. Once `readWrite` is held, Ctrl+S saves without a prompt. The decision is the pure function `saveShortcut(level)` in `journey/fileaccess.js`. A pointer or keyboard activation of the shell-drawn Opslaan control on a read-only document still escalates and saves, because Opslaan is a save control on the trusted surface named in E11 (b) (design decision). Bewerken is a one-way action, not a toggle: once granted it is replaced by the state chip "Bewerken aan" until the file is closed. |
| App-originated escalation refused | `apps.js` and `stations.js` import only `appAccess(app)` (the frozen app facet); the shell facet is imported only by `shell.js` and `journey.js`. `requestEdit()` returns `{ok: false, reason: 'trusted-surface-only'}`; `write()` fails closed without a `readWrite` grant for that document. |
| (c) Revocation | Closing one document (switching the single Notities window to another document, or Intrekken in the grants view) calls `contract.js` `grants.endTemporary` on the five-field key of each document right (`read`, `readWrite`) while the app keeps running; the lead's E11 (c) amendment of 23 November 2026 reads "grants.revoke on the five-field key" as `grants.endTemporary` for temporary grants, and `grants.revoke` stays for remembered decisions only. Closing the Notities window ends the app session (`wm` emits `appClosed`, the broker calls `grants.appClosed`); sign-out calls `logout`. After each, a later write fails closed. Instellingen > Privacy en beveiliging lists the session file grants as `.ds-grant` rows (five-field key, "Tot sluiten") with Intrekken. A document read that the user allowed in the prompt (Boodschappen, station 7) is listed in the same view under "Bestanden die een app na je toestemming mag lezen" with "Tot Notities sluit" and, since B repair 3, its own Intrekken action, which ends exactly that key through the same shell path; the next read by Notities then opens the prompt again. Ending a document never touches a remembered decision, the app's temporary denials (R2) or any other key. |
| (d) Separate authorization | Another file goes through `contract.js` `ask()` and the `.ds-ask` prompt: Notities "Lijst invoegen uit Boodschappen" asks for a temporary read of Boodschappen (station 7). That grant does not make Boodschappen an opened file (`shell.level` stays `null`), so it cannot be escalated to write. A broader scope, a remembered grant or any network right also goes through `ask()`. No grant is transferable between apps. Nothing this path creates persists past close, app exit or sign-out; remembered decisions are created only by the user in the prompt, are owned by Appmachtigingen and are left as they are. An explicit user open is the person's own act, so it grants read even when a remembered Weigeren exists for the same key: that remembered decision answers requests made by the app, not the person's open (design decision, B repair 2). |

### 12.2 Contract method `grants.endTemporary` (resolved in B repair 3)

The earlier limitation (repairs 1 and 2: ending one document ended the app's contract session, its R2 temporary denials included, and re-seeded the other grants) is resolved. The lead added one method to the protected consent contract in commit 44ef6030: `grants.endTemporary(key)` ends exactly one five-field temporary grant and its compact-form record, emits `{op: 'end-temporary', key}`, and leaves remembered decisions, temporary denials (R2) and every other key unchanged (tests in `tests/contract-end-temporary.test.mjs`). `contract.js` stays read-only for the journey; the HEAD version is the accepted contract. The broker's `end(app, doc)` is now one `endTemporary` call per document right and no longer calls `grants.appClosed` or re-seeds anything; app close, sign-out and a session switch keep their own contract calls. The former `todo` test (the same app keeps its R2 denial when one of its documents closes) now passes, and no test in the suite is `todo` or skipped. In the rendered walk, an automatic identical re-request after a document switch stays a quiet denial (W9b).

A second limit: in this browser page the separation between the shell facet and the app facet is a module convention checked by static tests, not isolation. App modules (`apps.js`, `stations.js`) can still call `ctx.openFile(id, ev)` with a real input event that landed in app content (Bestanden Openen is such a call), so the broker proves that a person acted, not that the action happened on a shell-drawn surface; in this prototype Bestanden stands in for the system file manager; the gesture rule is checked by behaviour tests and the rendered walk. Native isolation of app code from shell code is part of the planned enforcement (Technisch haalbaar, nog niet gebouwd).

### 12.3 Checks performed (repair 1)

| Check | Mechanism | Result |
|---|---|---|
| Broker unit tests | `node --test` over `tests/journey-fileaccess.test.mjs` on fresh `createPermissions()` instances. Positive paths use an injected gesture predicate (a symbol that stands in for a real input event); a separate test asserts that the default predicate refuses `undefined`, `null`, `{}`, `{isTrusted: true}`, `new Event('click')` and an object with the `Event` prototype for both `open` and `edit` | pass |
| Wiring | static checks: no `openFile(...)` call without the event argument in `shell.js`; no `fileAccess` in `apps.js` or `stations.js`; the broker receives the gesture (`fileAccess.shell.open('notities', id, ev)`); no `seedTemporary` outside the `?test=1` block of `journey.js` | pass |
| Rendered integration walk, three themes | `build/design-evidence/uxe/B-repair-1/walk.mjs`: Chrome headless over CDP; `Input.dispatchMouseEvent` and `Input.dispatchKeyEvent` produce trusted input; in-page `.click()`, `new MouseEvent()` and forged objects are the negative cases. Results in `walk-<theme>.json` | 25 of 25 checks pass in dark, light and contrast (0 failures) |

The walk covers: no test hook and `TEST_OPEN === null` without `?test=1`; no grant after sign-in; each open route (Verder waar je was, Menu recent by pointer and by Enter, Bestanden Openen, Openen met, the Openen... chooser by keyboard, Escape on the chooser, a Zoeken result) gives exactly one temporary read without a prompt; forged `{isTrusted: true}` objects, a script `MouseEvent` and script `.click()` on Bewerken and Openen... are refused; pointer Bewerken gives `readWrite` on that file only and two Ctrl+S saves show no prompt; the Boodschappen request (prompt focus, remember unchecked, equal answers, Escape fail-closed, Toestaan inserts into the draft without amplification); Intrekken in the grants view followed by a failing write; closing the window; sign-out.

### 12.4 Checks performed (B repair 2)

| Check | Mechanism | Result |
|---|---|---|
| Side effects of close | `tests/journey-fileaccess.test.mjs`: a remembered Toestaan and a remembered Weigeren on (Notities, read, Tuinplan) survive open, edit, close and revoke, and still answer the next request without a prompt; another app's temporary denial survives a Notities document close; the same app's R2 denial after a document close is a `todo` test | pass, 1 todo; `build/design-evidence/uxe/B-repair-2/side-after.txt` (verifier probe `side.mjs`): `rememberedAfterClose: ["deny"]`, `tempDenyAfterClose: 0` (the open item above) |
| Frame actions inside the bar | `barprobe.mjs` (CDP pointer open and Bewerken at 1280, 1440 and 1920 px) | every action inside the bar, title not clipped (`barprobe-after.txt`; before: `barprobe-before.txt`) |
| Rendered E11 walk | verifier probe `walk.mjs` re-run in dark, light and contrast (trusted CDP pointer and key input) | all checks pass except W9 and W9b (the R2 limitation of that round, resolved in B repair 3, section 12.2) in each theme; W8 (a remembered Weigeren survives opening and closing that document) now passes (`walk-<theme>.txt`, `walk-<theme>.json`) |
| Keyboard walk | verifier probe `kbd.mjs` at 1440 x 900 dark, 320 x 640 light and 720 x 450 contrast | 1440: 0 failures (K4b and K6, inspector tabs, failed before the repair); 320 and 720: K13 (Foto maken not reached by Tab, because stack mode shows one window and other windows are reached with Alt+Tab or the taskbar) and K16 (focus reaching the document body at the end of the Tab cycle in headless Chrome) fail exactly as in the verifier's own B-verify-1 run |
| Station 7 task 1 from the route state | `task1.mjs` from `#/toestemming`: pointer (dark; Notities raised from the taskbar, Openen..., Tuinplan, Lijst invoegen uit Boodschappen) and keyboard (contrast) | the route opens Notities empty with no grant; the open gives exactly one temporary read without a prompt; the request opens `.ds-ask` with focus inside; Escape fails closed (`task1-pointer.txt`, `task1-keyboard.txt`) |
| Sweep coverage | the sweep now fails a view and theme in which it measured fewer cells than stations' states plus extras | 770 of 770 cells, 0 failures; two further 1440 runs 330 of 330 each |

## 13. Slice B: stations 5 to 8

Slice B is built for stations 5 to 8. Station 9 (Meldingen) was moved to slice C in repair 1, because this slice's task scopes B to stations 5 to 8; it stays a "Volgt" entry in the Rondleiding (never a route), and the Meldingen list of slice A stays available from the Orbit clock.

### 12.5 Checks performed (B repair 3)

| Check | Mechanism | Result |
|---|---|---|
| Broker and wiring tests | `node --test --experimental-test-isolation=none ui/design/prototype/tests/*.test.mjs`. New: closing a document emits only `end-temporary` events and leaves every other key equal; Intrekken on a prompt-allowed read ends it, keeps the opened document's read, another app's connect grant and an R2 denial, creates nothing remembered, and the next read prompts again; `saveShortcut` never returns an escalation for `null` or `read`; a static check that the window frame's Ctrl+S handler reaches neither `editAction`, `saveAction` nor `shell.edit`, and that `editAction` requires a frame control | 72 tests, 72 pass, 0 todo, 0 skipped, exit 0 (`build/design-evidence/uxe/B-repair-3/node-test.txt`) |
| Rendered walk of the earlier verifier | `walk.mjs` (copied from B-verify-2), trusted CDP pointer and key input, dark | 0 failures; W9 and W9b (R2 denial survives a document switch; the automatic re-request stays quiet) now pass (`walk-dark.out`, `walk-dark.json`) |
| Repair-3 probe | `r3probe.mjs`: real Ctrl+S (`Input.dispatchKeyEvent`, Ctrl modifier) with focus in the read-only text, on the window title, on Opslaan and on Openen...; auto-repeat Ctrl+S; Enter on the focused Bewerken; Ctrl+S after `readWrite`; app identity on window title, Orbit (or the switcher in stack mode) and prompt; Orbit status, desktop privacy instrument and quick settings before, during and after a connect grant; Intrekken by keyboard on a prompt-allowed read and the next request | 0 failures in dark, light and contrast at 1440 x 900 and in dark at 320 x 640 (`r3probe-<theme>-<width>.json`, screenshots `r3-*.png`) |
| Keyboard walks | `kbd.mjs` and `e11walk.mjs` from B-verify-2 | 1440 dark: 0 failures; 320 dark: K13 and K16 as in every earlier run (section 12.4); e11walk: all checks pass except K7a, which fails as in B-verify-2 because it queries a selector (`.jr-acc-row`) that the grants view has not used since repair 1; the keyboard Intrekken step K7 that follows passes |
| Reflow | `misc.mjs` from B-verify-2 at 320 x 640, 720 x 450 and 320 x 640 contrast with T1 | the B repair 3 Versies layout regression it found (the widened diff column applied inside narrow windows) is fixed; not fixed: the rotating loading satellite of `.ds-empty-orbit` reports element overflow mid-rotation in the three busy cells (320, 200 %, 320 contrast + T1). `gorbit.mjs` scrubs the rotation: the gallery's own busy orbits reach scrollWidth 88 against clientWidth 68 at 1440 px, exactly as the journey at 320 px (88/68; 70/68 at rest); the orbit stays inside the window body (x 41 to 113 in a 0 to 320 body) and the page does not scroll (scrollWidth 320). The overflow belongs to the accepted component, so the journey leaves it unchanged (open item) |

### 13.1 Stations

| n | Station | Route setup (opens or focuses, never closes or resets) | What it shows |
|---|---|---|---|
| 5 | Bestanden ordenen | opens or focuses Bestanden | Sidebar with Alle bestanden, the four collections and the places Dit apparaat and Thuis-NAS; find field; Groeperen op Plaats, Collectie or Soort; per file the place badges (`.ds-place`: colour plus node shape, icon and text), Openen and Openen met...; inspector with the tabs Versies, Opslag and Collectie. Opslag holds "Bewaren op" (Dit apparaat, Thuis-NAS, Dit apparaat en Thuis-NAS, and, once the user has chosen an external provider, "Dit apparaat en opslag.example, versleuteld (gesimuleerd)") and the route strip, which ends at the blue NAS node or at the red internet node of the chosen provider (colour plus ring shape, globe icon and the provider name). Before a provider is chosen, the external option is disabled with the reason and a button to Opslag en NAS. Collectie moves the file to a collection and sends it to the Prullenbak with Ongedaan maken. |
| 6 | Versies en terugzetten | Bestanden with Tuinplan selected, inspector tab Versies | Versions newest first, "Huidige versie", Vergelijken (lines marked with the words Toegevoegd and Verwijderd, not by colour) and Deze versie terugzetten (`files.restore`, a new current version; Ongedaan maken calls `files.undoRestore`). A restore made in Bestanden reaches an open Notities window through the `files` bus event. |
| 7 | Toestemming | opens or focuses Notities (empty: no document, no grant), Foto's and Webvenster, placed side by side in wide frames (B repair 3: tiles instead of a cascade; placement only); the Rondleiding entry lists the three requests to try ("Drie verzoeken om te proberen") | The requesting web app is presented as the third-party app **Webvenster** on every journey surface: window title, Orbit, switcher, Menu, Zoeken, notifications, Appmachtigingen and the prompt provenance ("Webvenster · geïnstalleerd 12 nov. uit Haven"). This is a journey-only mapping of the app id `browser` through the prompt renderer's own `APP` map (`journey/access.js` `appName`); the gallery's `APP` map, its output and the string `T.apps.browser` ("Browser", the first-party app in NAMING-AND-IA) are unchanged. Three in-app requests through `contract.js` `ask()` and the gallery's prompt renderer: Notities "Lijst invoegen uit Boodschappen" (document read), Webvenster "Verbinden met tuinplanner.example" (connect, domain scope) and Foto's "Foto maken" (camera, while open, with the app's reason). Each answer leads to an honest quiet note in the app's own window; no request ever starts from the OS frame. |
| 8 | Instellingen | opens or focuses Instellingen at Privacy en beveiliging, in the main window position in wide frames | Appmachtigingen as the gallery's `.ds-grants` list (remembered allows and denials, "Precies:" key chips, Wijzigen and Vergeten through `grants.change` and `grants.revoke`), the session file grants in the same `.ds-grant` anatomy (plus, when present, the document reads allowed in the prompt, each with Intrekken), Opslag en NAS (usage bars, pairing and encryption lines with "Gesimuleerd", and Externe opslag: what a provider gets and never gets, "opslag.example kiezen" and Loskoppelen, which returns every external copy to this device) and Updates with its states. The settings navigation wraps long names instead of truncating them (B repair 2). Weergave and Toegankelijkheid come from slice A. |

The prompt renderer was factored out of `ui/design/system/gallery.js` into `ui/design/prototype/ask-view.js` by moving the code verbatim (`renderAsk`, `lines`, `keyChips`, `words`, `what`, `itemName`, `places`, `badge`, `viewOf`, `syncAsk`, the constants, and a `grantRows(list)` function holding the former inline Appmachtigingen markup). `journey/ask.js` mirrors the gallery's live controller in `#jr-ask`.

### 13.2 States and simulation controls

| Station | Empty | Loading | Error | Success | Simulatie group |
|---|---|---|---|---|---|
| Bestanden ordenen | collection Cursus pottenbakken (empty, with a hint) | "Bestanden laden van dit apparaat en Thuis-NAS…" (`.ds-empty` busy, static orbit in reduced motion) | Thuis-NAS not reachable: what happened, that files on this device keep working, Opnieuw proberen and Opslag en NAS openen; NAS-only files say so | grouped list with inspector | Bestanden: Gevuld, Laden, Fout, Leeg (the Thuis-NAS group of slice A gives the same error) |
| Versies | one version only ("Dit bestand heeft nog maar een versie…") | skeleton rows "Versies laden…" | older versions on Thuis-NAS unreachable; the version on this device stays listed and safe; Opnieuw proberen | stack of versions with compare | Versies: Gevuld, Laden, Fout, Leeg |
| Toestemming | Webvenster before a request ("…heeft nog geen verbinding met tuinplanner.example") | after Toestaan: "Verbinden met tuinplanner.example…" with an indeterminate bar | refusal: nothing sent, nothing shared, Opnieuw vragen; site not responding (simulated): nothing sent, try later | labelled simulated site | Toestemming: Site laden Normaal, Traag, Mislukt; buttons "Volgende vraag kan niet worden getoond" (the next `present` throws: `via: 'error'`) and "App vraagt automatisch opnieuw" (repeats the last request without a gesture: R2 quiet denial) |
| Instellingen | Appmachtigingen with no remembered choice | Updates "Controleren…" | "Controleren is niet gelukt. Je huidige versie blijft werken. Probeer het later opnieuw." | "BaryonOS is bijgewerkt" | Instellingen: Updates Bijgewerkt, Controleren, Fout |

Each group sits in the band's Simulatie panel under the visible note "Simulatie, geen echte actie".

### 13.3 Simulated versus planned (slice B rows of the Status panel)

| Function | Label |
|---|---|
| Find, group and move in this page; Openen and Openen met (read only) | Werkt nu in dit prototype |
| Keeping a file on Thuis-NAS and its route; external storage chosen by the user and its route (LOCAL-PRIVACY-CONTRACT revision 4, section 2.4 and P05; nothing is encrypted or sent); versions on the NAS; usage, pairing and encryption lines; camera image and site content; Updates | Gesimuleerd |
| The native Files app (DESKTOP-SHELL D5); history for every file (DESKTOP-SHELL §4); enforcement of prompts, Appmachtigingen and file access by BaryonOS | Technisch haalbaar, nog niet gebouwd |
| Restoring a version without loss; the prompt and decision model; Appmachtigingen list; tier-2 axes | Werkt nu in dit prototype |

### 13.4 Metaphors and motion *[judgement]*

- **Where data travels** (Opslag tab): choosing Thuis-NAS moves a small node along a dashed route from the device node to the NAS node once (`--motion-duration-slow`, three passes), while the text reads "Wordt bijgewerkt…"; it ends with the NAS node lit and "Op dit apparaat en Thuis-NAS (gesimuleerd)". With the NAS offline the node waits halfway with a hollow NAS node and "Wacht op Thuis-NAS (gesimuleerd)". Reduced motion: the duration tokens are 0 ms, so the end state shows at once and the polite announcement carries the change.
- **A stack of drafts** (Versies): versions are offset sheets; a restored version lifts onto the stack (`--motion-duration-base`) and the former current version stays right beneath it with "Bewaard", which shows that restoring never deletes. Reduced motion: no lift; the chips carry the meaning.
- **The key turns** (Bewerken): Bewerken carries the access key glyph; when granted, the chip "Bewerken aan" slides out of where the button was. Reduced motion: instant.
- The prompt keeps only its existing rise; there is no motion toward either answer (§11.1).
- [untested hypothesis] the route strip makes "where is my file" easier to answer, and the stack makes restore feel safe.

### 13.5 Accessibility checks performed (slice B)

| Check | Mechanism | Result |
|---|---|---|
| Journey sweep (E10) | `node ui/design/prototype/tests/run-sweep.mjs` (Chrome headless over CDP, focus emulation). It visits every built station and state, including the slice B states and the extras `note-edit`, `note-revoked`, `note-nodoc`, `chooser`, `open-with` and `opslag`, in the dark, light and contrast themes and the tier-2 combinations C1, C2 and S2 (`data-scale="2"`), at 1440 x 900, 320 x 640 and 720 x 450 CSS px (the last is 1440 x 900 at 200 %) | B repair 3: 798 cells, 0 failures, 10954 warnings (`build/design-evidence/uxe/B-repair-3/sweep.json`; new extras `quick-net`, the system status while Webvenster holds a connect grant, and `tile-edit`, Notities with Bewerken aan inside its station-7 tile; `tile.mjs` measures that frame row at 1160, 1280 and 1440 px: every action inside the bar, nothing truncated, no tile overlap). B repair 2: 770 cells, 0 failures; 11568 warnings of the non-failing kinds behind-surface, raised-on-focus and covered-until-focus (`build/design-evidence/uxe/B-repair-2/sweep-full.json`, which also lists every visited cell). Before the repair the three new checks found 961 failures at 1440 x 900 alone (`sweep-before.json`) |
| Sweep change | inside a cell with an open modal dialog (`dialog:modal`), only the dialog is checked: `showModal()` makes the rest inert | documented in `journey/sweep.js` |
| Sweep checks added in B repair 2 | (10) frame containment: a box inside a window stays inside it, and a box inside a window bar inside the bar, unless a scroll box or popover lies between; (11) truncated labels: an ellipsis that leaves less than two thirds of a label or less than four em fails, a milder one is a warning (this covers `.ds-window-title`, which the vertical clip check exempts); (12) tab focus: after ArrowRight and ArrowLeft in every visible `[role=tablist]`, focus is on the newly selected tab; new extras `external`, `opslag-ext` and `prompt-doc` | each check flagged its known defect before the fix (Notities frame actions, Instellingen navigation and window title, Bestanden inspector tabs) and passes after it |
| Prompt in the journey | walk W12 and W13: focus on the question, "Keuze onthouden" unchecked, both answers `secondary`, Escape is a fail-closed cancel with focus back in the app | pass in three themes |
| Gallery unchanged after the extraction | pixel compare of the gallery at 1440 x 2400 against `gallery-baseline` (ImageChops bounding box) and `galcmp.mjs` (outerHTML of the permission and Appmachtigingen cards plus 15 prompt openings per theme against the HEAD copy in `B-verify-0/base`) | identical in dark, light and contrast |
| Contract tests | `node --test --experimental-test-isolation=none ui/design/prototype/tests/*.test.mjs` | B repair 3: 72 tests, 72 pass, 0 `todo`, 0 skipped, exit 0; the 29 contract tests and the 3 `endTemporary` contract tests are unchanged |
| Design-system gates | `node docs/verification/a11y-ds-20261115/check.mjs`; `python3 -B tools/check_a11y_matrix.py <worktree>` | both exit 0 |

Mechanisms for media features: theme and tier-2 axes through the journey's own root attributes (`setRoot`), reduced motion through `data-motion="reduce"` in C1; 200 % through `data-scale="2"` and the 720 x 450 CSS px cell. A real screen reader was not used; landmarks, names and live regions are checked by the sweep and the walk only.

### 13.6 Open items (slice B)

- **Externe opslag** can be chosen (simulated, one fictitious provider) and used per file; a provider account, credentials, several providers, and the pairing sheet of NAS-SHARING-AND-INTERNET.md are not built. Keeping a file at the provider is a user action in the shell's own Bestanden, not an app request, so it does not open the `.ds-ask` prompt; the prompt renderer has no app name for Bestanden and its send-copy consequences describe a readable copy, not opaque ciphertext. Whether the native shell needs a separate confirmation per file is an open design question.
- **Label density** in the Bestanden list (one "Gesimuleerd" chip per file row) is unchanged *[AI opinion: it reads as administrative]*; one label per group would need a sweep rule change.
- **Rename, Favoriet and Delen** in Bestanden are not built; Openen, Openen met, Collectie, Naar Prullenbak with Ongedaan maken, Bewaren op and versions are.
- **System status** (B repair 3, lead decision 5): while a running app holds a connect grant (temporary, or a remembered Toestaan), the Orbit status item ("Status: ..., internet aan voor Webvenster, alleen tuinplanner.example (gesimuleerd)", internet ring drawn heavier than the "off" ring, so the state is not colour alone), the desktop privacy instrument (internet column lit, "Internet: Webvenster, alleen tuinplanner.example") and the quick settings (with "Gesimuleerd") say so; it clears when the grant ends (window close, app exit, sign-out, or Vergeten or Wijzigen of a remembered decision in Appmachtigingen; a temporary connect grant has no Intrekken row and ends only with the app session; after such a change the open Webvenster and Foto's windows re-render from the grants, so the address-bar chip and the camera state clear while the window stays open). The status derives from the same grants the Webvenster window reads. Camera use is still shown only in the Foto's window, not in the Orbit status (open).
- **Loading orbit overflow** (`.ds-empty-orbit`, not fixed): the satellite's single rotation counts as element overflow while it turns, in the gallery as in the journey (section 12.5). A fix changes the rotating element of the accepted component in `components.css` (existing rules are append-only for this task) and is left to the design-system owner; the journey adds no override.
- **Window placement**: the station 7 tiles and the station 8 main position apply in frames of at least 72rem; between 48 and 72rem windows still cascade, and below 48rem stack mode shows one window.
- **Visual review** of the new surfaces by a person: the writing agent could not view images in this run (a local hook blocked image reads), so the slice B layout evidence is the sweep's geometry, overflow, contrast and focus checks plus DOM measurements, not a visual inspection. In B repair 2 the writer inspected downscaled screenshots of the repaired surfaces (`build/design-evidence/uxe/B-repair-2/shot-*.png`: Notities bar after Bewerken, Bestanden with the external route, Opslag en NAS with Externe opslag in the contrast theme, the station 7 task list); a review by a person is still open.
- **Multi-window apps** (Webvenster, Foto's) can have several windows; a route revisit focuses the existing one, and the app session ends with its last window.
