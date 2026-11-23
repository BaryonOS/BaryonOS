# BaryonOS connected prototype journey (UX-E)

## 1. Status and evidence level

The journey at `ui/design/prototype/index.html` is a **browser simulation** of BaryonOS in the Orbitaal design language (ds-1.1). It is design evidence, not a working operating system. Nothing on the page enforces, stores, synchronises, encrypts or recovers anything; all state lives in the page and is lost on reload.

- Slice built: **A (shell)**, stations 1-4. Base commit `ec459716`. Plan: `build/design-evidence/uxe/PLAN.md` (ignored build directory).
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
| 3 | Apps en vensters | Notities (Tuinplan, Ctrl+S), Rekenmachine, Bestanden (read-only list), Instellingen (Weergave, Toegankelijkheid); minimise, maximise, snap, close, window menu, switcher | A |
| 4 | Zoeken | Grouped results over apps, files, settings and actions, plus the on-device assistant row | A |
| 5-10 | Bestanden ordenen, Versies, Toestemming, Instellingen, Meldingen, Herstel | Shown in Rondleiding as "Volgt" entries (`aria-disabled`, focusable, with a reason line); never links or routes | B, C |

Locking keeps the session: windows and drafts stay. Signing out with an unsaved Notities draft opens a dialog with three equal answers, Opslaan, Niet opslaan and Annuleren, with focus on Annuleren.

## 4. States per station and simulation controls

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

Consent (contract OQ-5, repair 1): `data.js` seeds one temporary grant (Notities, readWrite, Tuinplan, until the app closes). `wm.js` emits `appClosed` when the last Notities window closes, and `contract.js` ends the grant. Notities shows the Tuinplan text only while `grants.temporary()` or a remembered Toestaan covers it (or an unsaved draft of the same app session exists); otherwise it fails closed with "Tuinplan is niet geopend" and the label Gesimuleerd. Slice A makes no new permission request and never re-seeds a grant in the product path; reopening a document through the UX-C prompt is slice B work (station 7). The sweep's test hook restores the page-load seed only for its Apps en vensters success cell, so both bodies are measured.

## 6. Familiar conventions (ADR-058) and shortcuts

| Convention | Choice | Source |
|---|---|---|
| Dock | Orbit, a floating capsule at the bottom centre: Menu, Zoeken, apps, status, clock | DESKTOP-SHELL §2, NAMING-AND-IA §5 |
| Launcher | Menu with search field, Vastgezet, Verder waar je was, Alle apps; footer with user, Vergrendelen, Aan/uit | DESKTOP-SHELL §2 |
| Window controls | **Right side**: Minimaliseren, Maximaliseren / Vorige grootte, Sluiten; app icon at the left opens the window menu; double-click on the title maximises | DESKTOP-SHELL §2, DESIGN-SYSTEM §9 |
| Arranging | Window menu: Minimaliseren, Maximaliseren, Links/Rechts vastzetten, Sluiten (keyboard alternative to dragging, WCAG 2.5.7) | DESKTOP-SHELL §2 |
| Status and clock | Status (three nodes: device, NAS, internet off) opens Snelle instellingen, which reports the NAS state read-only; clock opens Meldingen, whose heading takes focus so a held or repeated Enter cannot reach Alles wissen | NAMING-AND-IA §5 |
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
- **Bureaublad composition (repair 1)** *[judgement]*: following the Orbitaal v2 board, the desktop shows a display-size greeting, the three-node privacy instrument (`.ds-privacy`: Dit apparaat lit, Thuis-NAS lit or off, Internet off; shape, icon and text) and recent work as a trajectory, newest first, with kind tiles (Tekst, Tekening, Lijst, Rekenblad). The lit blade dot on the trajectory marks where the user was last. A neutral blade-light field (no place colour) sits behind the desk. In stack mode the instrument is hidden and the Orbit status carries the same three nodes. All of it is static, so the reduced-motion path is identical.
- **Search preview column** *[judgement]*: on wide frames the highlighted result is described beside the list (kind, collection, place with shape, icon and text, last change); in stack mode the column is hidden.
- Whether these metaphors improve understanding is an **[untested hypothesis]** for U06. The Orbitaal identity of the desktop after repair 1 is a design judgement, not a verified finding.

## 8. Components

- Reused ds-1.1 components: `.ds-btn`, `.ds-input`, `.ds-field`, `.ds-fieldset`, `.ds-seg`, `.ds-radio`, `.ds-check`, `.ds-switch`, `.ds-row`, `.ds-place`, `.ds-node`, `.ds-orbit`, `.ds-taskitem`, `.ds-taskitem-badge`, `.ds-window*`, `.ds-menu*`, `.ds-dialog*`, `.ds-empty*`, `.ds-skeleton*`, `.ds-progress`, `.ds-banner`, `.ds-kbd`, `.ds-live`, `.ds-vh`.
- Journey-only parts live in `ui/design/prototype/journey.css` with the `.jr-` prefix. They are candidates for O-K1 and are not appended to `components.css`: band and panels, feasibility chip, lock screen, session surface, user list, step list, desktop head and plate (`.jr-desk-head`, `.jr-plate`), trajectory list (`.jr-cont-list`, `.jr-cont-item`), kind tile (`.jr-thumb`), search preview (`.jr-search-body`, `.jr-preview`), neutral desk field (`.jr-field`), window layout, launcher and popovers, switcher, settings layout. Repair 1 reuses `.ds-privacy` (three-node instrument) and the full `.ds-empty` anatomy (orbit, satellite, title, hint, actions) through one helper, `emptyState` in `journey/apps.js`.
- `.ds-*` rules are used only in a `.jr-*` or `:where()` context. `button.ds-row` drops the user-agent button face at zero added specificity, so the ds hover and selection rules still apply.
- Gallery regression: `gallery.js`, `index.html`, `components.css` and `icons.js` are byte-identical to the baseline hashes in `build/design-evidence/uxe/gallery-baseline/SOURCES.sha256`. A dark gallery render at 1440x2400 differs from the baseline in 0 pixels. `strings.js` differs only by the appended `journey` block; outside the block the file hashes to the `ec459716` value (`journey-static.test.mjs`).
- The permission renderer has not been extracted yet. That happens in slice B.

## 9. Accessibility checks performed (slice A)

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
- **Opening a document again** after its grant ended needs the UX-C prompt (slice B, station 7). Until then the reopened Notities fails closed.
- **Secret-rule test** greps `session.js` source; a DOM-free behaviour test that drives `initSession` with a stub document and spies on bus, store and console is not written yet.
- **Desktop identity (E6)** after repair 1 is a design judgement; no user comparison and no owner review has taken place.
