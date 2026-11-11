# Product decision and build specification P2 — Materiaal

## Addendum ADR-058 — 30 October 2026

This project decision takes precedence over the rules below that the main screen does not imitate an existing desktop (§1) and that there will be no dock or taskbar (§5), and over the sentence in *Addendum P3/P4* that the experience must not interactionally replicate Windows or macOS, insofar as it concerns the common layout, controls and keyboard shortcuts. Visual replication (house style, icons, trade dress) remains excluded; the other P3/P4 requirements for finish and speed continue to apply. The final experience gets a recognisable desktop, taskbar, start menu with search, window controls, setup and sign-in screen according to [DESKTOP-SHELL.md](DESKTOP-SHELL.md) and [PROFILES-AND-USERS.md](PROFILES-AND-USERS.md). Materiaal remains the data, history and permissions model and appears in the Files app, recent items and permission dialogs. P2 below remains historical product evidence. An own visual identity remains mandatory; what is recognisable is the controls, not the house style.

## Research addendum full OS journey — 2 November 2026

The [31-aspect installation comparison](../research/os-installation-comparison.md),
[28 everyday tasks](../research/os-interaction-patterns.md) and
[mental models](../research/transferable-mental-models.md) were recorded
first. From them follow the [installer specification](../product/new-os-installer-spec.md),
[onboarding](../product/new-os-onboarding-spec.md) and
[measurable interaction principles](../product/interaction-principles.md).
**These are proposals for follow-up work, not an implementation assignment or
user validation. P2 remains frozen on the existing seven routes.**
Materiaal and the own kernel remain the product base; familiar basic actions
must remain predictable without a mandatory new metaphor.

### Bounded follow-up plan after current source check

| Component / problem | Solution and files involved | State and navigation | Verification and regression risk |
|---|---|---|---|
| J-P1: familiar term does not find unfamiliar own name | Local synonym registry and honest availability status according to ON-03. Existing references: `ui/prototype/state.js`, `app.js`, `broker.js` and development source `sdk/examples/window-session/src/scene.rs`; none of these product files has been changed now. A dedicated registry component/test is only scoped at execution | Shared intent IDs/labels; query inert; no permission change; origin does not determine a separate shell | Aliases per language, ambiguity/missing service, focus and performance baseline. Risk: falsely available function or action while typing |
| J-P2: wrong expectation and learning burden when switching | Optional local, skippable help according to ON-01/02/04–06. New guidance/preference component; any prototype task stays separate from P2 and clearly simulation | Postponed/skipped/completed, resume via Help; no second account/privacy wizard or extra primary navigation | Origin variants keep the same permissions/routes; keyboard/screen reader/scaling/reduced motion; task trial with/without hints. Risk: profiling or extra blockers |
| J-P3: data loss through an unclear installer boundary | First a read-only inventory/plan model; later installer broker, account/key/recovery services according to IN-01–07. Inspect exact native write files and IPC again first; do not broadly rewrite existing boot/storage code | Changing target/plan invalidates authorisation; one complete boundary before every mutation; first boot checks actual state | Disposable disk images, stale/hotplug/unknown/encrypted, interruption/boot/recovery and comprehensibility. Risk: irreversible loss; real writes only after separate native gates |

J-P1/P2 are candidates for a next bounded research trial,
not release features that have already been scheduled. J-P3 starts exclusively as a
non-writing plan trial; personal authentication, crypto, key recovery,
hardware/boot compatibility and accessibility from boot are still
separate preconditions. A renderer grants no permissions. The normal
route contains no mandatory provider account, OS telemetry or AI.

For each execution, first record concrete write files, current source commit,
capability boundary, performance baseline and applicable build/lint/test commands.
Source research does not prove a current native implementation. The
new criteria are test targets and yield no progress points; the
existing release criteria and storage/privacy contracts remain leading.

The P2 freeze concerns the historical prototype; it does not withdraw the recognisable
desktop direction from ADR-058 above. The new journey proposals
are not a silent change of that project decision.

## Addendum N01 — 27 October 2026

This project decision takes precedence over the original scope below of one own NAS. Devices now has three flows: **Mijn NAS** (My NAS), **Ruimte ontvangen** (Receive space) and **Ruimte aanbieden** (Offer space). Internet is separately permitted for a confirmed NAS identity. Received space can be hosted by friends/family; the recipient confirms that the host can delete their NAS copy and is advised to keep an independent copy. The administrator sees only capacity/usage/status and may revoke+wipe without guest consent or content key.

Data access confers no administrator role. In the mock, explicit local administrator activation is required; an internet route grants no remote administration. Revocation increments the generation, blocks old uploads and frees capacity; a new invitation gets a new space ID. Document names and content are to be encrypted only in the real client: the browser flow simulates this and processes no keys, connections or real NAS files.

The 39 state tests and syntax checks have been run; graphical browser checking of these new screens is still open. Exact crypto/service trial and native integration boundaries: [NAS-SHARING-AND-INTERNET.md](NAS-SHARING-AND-INTERNET.md), [privacy contract revision 3](../LOCAL-PRIVACY-CONTRACT.md), [UI guide](../../ui/prototype/README.md).

## Addendum consumer research — 28 October 2026

The product choice **Materiaal** remains unchanged and P2 remains frozen on the seven existing screens. The supplementary literature exploration in [CONSUMER-VALUE-2026-10-28.md](../research/CONSUMER-VALUE-2026-10-28.md) adds no new proven market advantage, but gives a targeted test order for the distinctive experience.

After the existing P2 acceptance, investigate one coherent demonstration (**DV-01**) in which the same supported material identity remains leading: finding it again, limited access for tool A, confirmed save, ending A, opening the same material with tool B, history/recovery as a new revision, revoking A's access and refusing a new protected action by A.

**Product evidence:** task success, time/help, unintended copies, understanding of storage status and permissions, recovery success and voluntary choice to perform the same task again via Materiaal.

**Native system evidence:** real capability/service enforcement, confirmed work that, within the storage contract, survives the termination of a tool, revocation for future actions and no false success status after uncertain storage. A browser mock does not prove these properties.

DV-01 is a follow-up hypothesis and acceptance scenario. It does not silently broaden P2 and changes no existing native evidence claim or progress point.

## Addendum P3/P4 — premium native productivity and own visual language

Project decision of 28 October 2026: privacy-friendliness must **not** visibly
feel like a functional or aesthetic concession. The eventual
Nieuw OS experience must be able to compete in finish, speed, typography and coherence
with high-quality modern consumer software, without visually or interactionally
replicating Windows, macOS, Microsoft Office or other
suites.

This does not change the frozen P2 task flows. P2 remains a prototype for testing
Materiaal. The requirements below steer P3 (design system/app contract)
and P4/M7 (native translation).

### Productivity model

- **Materiaal remains central.** A document, table, presentation, image or
  collection is the lasting object; tools are interchangeable
  editing capabilities.
- **No Office ribbon copy as the base.** Contextual tools appear
  when the selected object or part needs them. The content
  dominates the screen.
- **Recognition without imitation.** Familiar basic actions such as open, save,
  search, undo, copy and export remain predictable; distinction comes
  from object continuity, history, contextual tools, motion and
  capability-aware sharing.
- **One product family.** Text, tables, presentations, PDF/media and later
  own tools use the same spacing, typography, focus,
  selection, version history, sharing, recovery and error patterns.
- **Interoperability is a migration requirement.** DOCX/XLSX/PPTX/PDF import/export
  is investigated as a practical compatibility target and tested per format/feature.
  Being able to open a file is no claim of full
  round-trip parity.

### Visual/interaction ambition: quiet futurism

The desired look is modern and forward-looking without decorative
science fiction: sharp typography, precise grids, subtle depth,
responsive motion and contextual controls. Privacy and security status
must fit calmly into the same visual language and not become an administration console.

Key principles:

- material/content gets visual priority over permanent chrome;
- transitions follow object continuity: the same material feels like
  the same object when another tool opens it;
- motion supports place, status and causality and respects
  reduced motion;
- icons and actions are immediately recognisable by function, with their own execution;
- technical details are available but not the default language for ordinary
  users;
- errors show the consequence and the next recovery action instead of cryptic
  codes as the primary layer.

The existing P2 colours/tokens are prototype choices, not a definitive brand or
OS house style.

### Native render target

The eventual core apps get a shared native UI/render layer as a
product goal. That layer must support GPU acceleration, high-DPI, text, vectors,
animations, accessibility and consistent input/focus as soon as the
native graphics base allows it.

A collection of separately styled webviews is not the desired final architecture
for the own core apps. The exact compositor, GPU, text shaping and
render design remains a separate technical decision and is only
fixed after the relevant G/H milestones. The browser prototype is therefore a
design tool, not a silently chosen runtime.

### Local AI as a tool, not as a mandatory chatbot

System AI remains local according to the privacy contract and is offered contextually
where useful, for example on an explicit text selection:
shorten, clarify, summarise, compare or translate. AI receives
only the selected context/capabilities and the ordinary workflow remains
usable when AI is absent.

The UI may make local processing visible, but does not use an intrusive
“AI everywhere” interface as a substitute for good basic controls.

### Privacy explanation without pop-up fatigue

There are two different UI elements:

1. a necessary capability decision for new access;
2. optional educational privacy explanation with a concrete example or critical question.

The user can turn off the educational explanation globally or per app. That
grants no new permissions and does not change existing grants. A normal
action within an already valid grant does not show the same explanation
again each time.

During gaming/focus, non-urgent educational notifications are not shown in the middle of
the activity. A deliberately activated new sensitive function may
still open a compact necessary grant request.

### P3/P4 acceptance questions

- **PX-01:** text, table and presentation prototypes feel like one product family
  without permanent Office ribbon imitation.
- **PX-02:** users can perform familiar basic tasks without explanation of
  the underlying capability model.
- **PX-03:** the same object version, location and history remain recognisable
  when another tool is used.
- **PX-04:** turning off privacy explanation does not change grant state.
- **PX-05:** keyboard, 200% zoom, screen reader route and reduced motion are
  part of the same design system.
- **PX-06:** realistic documents are tested for render quality, latency,
  recovery and import/export; screenshots alone do not count as acceptance.
- **PX-07:** visual comparison checks originality and coherence without
  claiming that a subjective “more beautiful than Office” has already been proven.

Status: **decision for the first implementation**, 26 October 2026. The source comparison and concept choice have now been carried out. User research, graphical implementation and native OS integration have not yet been carried out. Codex does not need to start a new broad concept study before P2. Only a concretely established conflict or an explicit correction by the project owner reopens the choice.

## 1. Chosen product logic

**Materiaal is the chosen direction.** The user starts from a document, photo or collection that remains the user's. Apps are interchangeable tools that perform a specific edit, with an explicit scope and duration. The object history, data location and access permissions remain visible outside the individual app.

This is a working name for the interaction model, not a definitive OS or brand name. The design is an own synthesis, not a proven world first. An original product experience does not require an unknown filesystem for the first trial.

The user can also open an app directly via Gereedschap (Tools); this keeps the controls accessible for people who think in terms of a program. The main screen does not become an app launcher or a decorative imitation of an existing desktop. The first navigation choice concerns **the material being worked on**.

## 2. Three concepts on the same five tasks

The comparison below is design work, not a measured user trial.

| Task | Materiaal: objects central | Werk (Work): activities central | Atelier: work surface central |
|---|---|---|---|
| Install an editor | Add a tool; it appears as a matching action for object types | Add a tool to an activity profile | Place a tool as an adjustable panel on the work surface |
| Create/edit a document | Open Tuinplan; choose an edit; create a new version of the same object | Open activity Tuin vernieuwen (Renew garden); start the next step and save the result | Drag Tuinplan to the editing panel; set up the environment freely |
| Find something again | Search by title/type/collection; location and latest version immediately visible | Search by activity, goal or period; then look for the associated result | Search by panel/worksheet or open the material drawer |
| Interrupt and resume | Resume object, selection and edit; check permissions again | Resume activity with last step, windows and draft results | Restore full spatial panel layout |
| Recover from an error | Open object history; restore version as a new revision | Restore the previous activity step, with consequences for linked results | Restore panel configuration or editing history of selected material |

**Trade-off and choice.** Materiaal aligns most directly with ownership, app interchangeability and understandable permissions. Werk helps with long-running activities, but introduces an extra organisational layer early on. Atelier offers strong customisability, but requires more setup and can become restless or inconsistent. P2 implements Materiaal, with simple collections; no automatic combination of all three concepts. Activities and freer work surfaces remain later hypotheses, not extra P2 scope.

### Comparison with familiar patterns

- Windows Snap arranges app windows and can restore groups; macOS Spaces distributes windows across multiple desktops. Those patterns show spatial organisation, but do not determine the object ownership or permissions model of Nieuw OS (Microsoft, n.d.; Apple, n.d.).
- Flatpak portals demonstrate that a concrete file choice can grant access without broad file access. The principle of a system portal is therefore known. The proposed design makes data location, object history and temporary tool access part of one continuous experience; this is a designed combination, not a claim that existing systems never have such components (Flatpak Project, n.d.).
- WCAG 2.2 provides testable criteria for the web prototype. Use them for P2; a successful web prototype does not automatically declare the future native OS accessible or legally compliant (W3C, 2024).

## 3. P2 is frozen on seven screens

All screens must actually be navigable. No button without function or explicit unavailable status. Use exclusively fictitious fixture data.

| ID / route | Content and primary action | Mandatory states |
|---|---|---|
| S01 `#/start` | Three short steps: language/controls; local profile; storage choice. Main action 'Begin lokaal' (Start locally). NAS can come later. | new, step back, completed; no internet account or real password entry |
| S02 `#/materiaal` | My material: continue, collections, search field, object list. Main action 'Nieuw document' (New document). | filled, search result, zero results, empty collection |
| S03 `#/object/obj-001` | Tuinplan in read/edit mode, tool choice, version history, data location. Main action 'Bewerken' (Edit). | reading, request pending, editing, saving, saved, recovery preview |
| S04 `#/toegang` | Trusted system surface with exactly who/what/action/duration. From object or tool details; on desktop as a dialog, on narrow screens as full screen. | request, granted, denied, revoked; combined read/internet warning |
| S05 `#/gereedschap` | Fictitious local catalogue, package details and installed tools. Main action 'Installeer voorbeeld' (Install example). | not installed, checking, installed, invalid signature, removed |
| S06 `#/apparaten` | This device and one own NAS. Pairing via visible identity check; datasets and sync status. | not paired, compare code, paired, NAS offline, network changed, identity mismatch |
| S07 `#/assistent` | Local assistant with explicitly selected context. Main action 'Vat selectie samen' (Summarise selection). | model present, model missing, insufficient memory, busy, answer, cancelled |

In P2 the assistant uses a deterministic example response, explicitly labelled as simulation. No model is downloaded and no inference is claimed. Quantum gets no decorative button here: the real Q1 trial has a separate execution contract and is only included in the shell after working functionality.

### Fixed fixture data

- Device: 'Dit apparaat' (This device); type desktop.
- Own NAS: 'Thuis-NAS' (Home NAS); simulated identity `NAS-DEMO-01`; no existing IP address or real device.
- Objects: `obj-001` Tuinplan (text, three versions), `obj-002` Plattegrond (SVG from local fixture), `obj-003` Boodschappen (text); collections 'Vandaag', 'Huis' and 'Archief'.
- Tools: 'Notities' available beforehand; 'Beeldbewerker' as an installable fixture package; 'Webvenster' as a fictitious tool with a separate internet permission. Do not claim real application compatibility.
- Error controls: a separate, clearly marked prototype panel activates NAS offline, wrong identity, low memory and invalid package signature. This panel is not part of the eventual OS experience.

## 4. Transitions and data rules

The prototype controller decides on transitions; screen components do not mutate arbitrary global state.

| Event / initial state | New state and visible result | Forbidden effect |
|---|---|---|
| `finishOnboarding` | S02; local profile in fixture state | Account registration or network request |
| `requestEdit(obj-001, Notities)` | S04: 'Notities mag Tuinplan lezen en wijzigen tot je dit document sluit' (Notities may read and change Tuinplan until you close this document) | Automatic access to collection, NAS or network |
| `grantSession` | S03 editing; status 'Notities · alleen dit document' (Notities · this document only) | Converting one-time permission into permanent access |
| `denyGrant` | S03 reading; content unchanged | Loss of document or forced app installation |
| `saveDraft` | New revision, then 'Op dit apparaat opgeslagen' (Saved on this device); sync status separate | Showing 'Veilig op NAS' (Safe on NAS) before sync is confirmed |
| `closeObject` | S02; edit grant ended; cursor position as local resume metadata | Copying an active capability to another device |
| `restoreRevision(v1)` | First a comparison preview; confirming creates v4 with content of v1 | Erasing V2/v3 or pushing the restore directly to the NAS without specified sync behaviour |
| `pairNas` | Code comparison; confirmation pairs fixture identity | Pairing solely on host name or the same Wi-Fi |
| `nasOffline` | Local storage remains usable; 'Wacht op eigen NAS' (Waiting for own NAS) | External cloud fallback or blocking local editing |
| `nasIdentityMismatch` | Sync stopped; explanation and 'Verbinding opnieuw controleren' (Check connection again) | Button to silently trust an unconfirmed identity |
| `networkChanged` | Sync pauses; local work remains possible | Trying to make the NAS reachable via the external internet |
| `requestSummary` | S07 with list of selected objects and local simulation | Reading a general document index or starting external AI |
| `modelMissing / lowMemory` | Explicit local limitation; 'Gebruik zonder assistent' (Use without assistant) remains available | Cloud button or implicit external fallback |
| `installFixture` | Simulated signature check; app added; zero grants | Equating installation with granting access |
| `revokeGrant` | Future protected actions are refused in the mock broker | Promising to erase copies already made |

The S04 warning for read and internet permission reads: 'Dit programma kan de toegestane inhoud naar een internetdienst sturen.' (This program can send the permitted content to an internet service.) Show objects, destination/scope and duration. Default to 'Annuleer' (Cancel); an explicit second action can permit the requested limited combination. This is deliberately authorised sharing, not an absolute no-leak guarantee.

## 5. Visual specification v1

Direction: calm, precise and polished; light mineral background, deep green tones, large readable titles, sharp alignment and generous use of white space. No copied dock, Windows taskbar, glass layers or decorative statistics. The object list and real user content carry the screen.

| Token | Value |
|---|---|
| `color.background` | `#F4F5F1` |
| `color.surface` | `#FFFFFF` |
| `color.text` | `#162B2B` |
| `color.textMuted` | `#5D6A67` |
| `color.accent` / primary button | `#0A5E55`, text white |
| `color.danger` | `#A33625`, always also text/icon |
| `color.warningText/background` | `#8A5200` on `#FFF3DB` |
| `color.controlBorder` | `#768783`; decorative dividers may be lighter |
| `font.family` | `system-ui, "Segoe UI", sans-serif`; no external font request |
| `font.size/lineHeight` | body 16/24; metadata 14/20; label 14/20; section 20/28; page title 32/40 |
| `font.weight` | 400 body, 500 labels, 600 titles; no body text thinner than 400 |
| `space` | 4, 8, 12, 16, 24, 32, 48 px |
| `radius` | 8 controls, 14 surfaces, 18 dialogs; pill exclusively for status labels |
| `control.minHeight` | 44 px; icon-only target area 44×44 px |
| `focus` | 3 px accent outline, 3 px offset; fully visible |
| `shadow.dialog` | 0 16px 48px rgba(22,43,43,.16); not as the only boundary |
| `motion` | 120 ms state/hover; 180 ms panel; 220 ms route; easing cubic-bezier(.2,0,0,1) |
| `reducedMotion` | transition without movement; maximum 50 ms fade; no essential information dependent on animation |

Calculated with the sRGB/WCAG relative luminance formula: body text/background **13.56:1**, secondary text/background **5.15:1**, white text/accent **7.65:1**, danger text/white **6.75:1**, warning text/background **5.81:1**, control border/white **3.77:1**. These are calculations of fixed colour pairs, not yet a check of all rendered states.

**Layout:** desktop from 1200 px: navigation rail 224 px, main content flexible with max width 1120 px, optional inspector 288 px within the main layout. At 768–1199 px: rail 72 px with labels via tooltip and an accessible name; inspector as an on-demand panel. Below 768 px: one column with compact top navigation and explicit menu; dialogs full screen. At 320 CSS px no horizontal page scroll; long titles wrap. The document content may have its own overflow when the content requires it.

**Component rules:** list rows are at least 72 px, title 16 px, metadata 14 px; one primary action per panel. Status has text plus icon. Skeleton only for genuinely planned asynchronous state; no fake waiting times above 300 ms to suggest technology. Error messages name the cause as far as known, the consequences and one next action.

## 6. Controls and accessibility contract

- Use semantic HTML, real buttons/inputs, heading hierarchy, landmarks and a skip link. Native `dialog` for S04 on desktop; focus returns to the opener. Background not operable during modality.
- Tab/Shift+Tab through all controls; Enter performs the primary selected action; Escape closes only a non-destructive overlay and keeps a local draft; no loss on accidental Escape.
- `Ctrl+K` opens search/actions within the prototype if the browser event is available; the visible search button remains the reliable route. A browser may reserve system shortcuts.
- `Ctrl+S` in S03 saves the fixture version and reports the new status; the visible Save button remains.
- Arrow keys move focus in an open results list; do not intercept document editing keys when text focus is active.
- `Alt+Links` is not a requirement, because of browser history; the own Back button always works.
- No drag action without a button alternative; no hover-only functions.
- Use a polite live region for storage status, paired NAS and revoked access. Error messages get context without continuously interrupting announcements.
- WCAG 2.2 AA is the target for P2: normal text at least 4.5:1, large text at least 3:1; meaningful UI boundaries/states at least 3:1. The project target is 44 px controls and visible 3 px focus. These are chosen stricter component rules, not a claim that all WCAG criteria have already been met.
- Test browser zoom 200%, keyboard and at least one screen reader in the available environment; report missing equipment or checks not carried out.

## 7. Exact P2 implementation path and ownership

**Choose dependency-free HTML/CSS/JavaScript ES modules.** No React, package installation, CDN, external font, remote image or analytics. This choice builds an interaction prototype on the development host; it does not fix any native compositor technology and does not replace the own kernel.

Files that P2 may create:

- `ui/prototype/index.html`: semantic document, route shell and CSP.
- `ui/prototype/tokens.css`: tokens only.
- `ui/prototype/components.css`: layout/components/responsive/reduced motion.
- `ui/prototype/app.js`: router, render coordination and events.
- `ui/prototype/state.js`: pure reducer and selectors.
- `ui/prototype/fixtures.js`: deterministic synthetic data.
- `ui/prototype/broker.js`: explicit mock of object, grant, NAS, install and AI interfaces.
- `ui/prototype/README.md`: run path, simulation boundary and checks.
- `ui/prototype/tests/state.test.mjs`: any meaningful state/negative mock tests with the built-in Node test module when Node is available.

One builder owns `app.js/state.js/broker.js`. A visual builder may only change `tokens.css/components.css` after agreements on component names. The verifier works read-only. P2 changes no kernel, boot or crypto code.

Run path from the repository root:

```bash
python3 -m http.server 8765 --bind 127.0.0.1 --directory ui/prototype
```
Open `http://127.0.0.1:8765/` in the local test browser. If port 8765 is in use, choose another free loopback port and document it. No public deployment or LAN binding.

P2 stores exclusively fictitious state in `sessionStorage` under `nieuw-os-prototype-v1`; a refresh can resume. A visible 'Demo opnieuw beginnen' (Restart demo) erases only that key. No real user files, passwords, NAS login or form uploads. The demo header label 'Interactiemodel · simulatie' (Interaction model · simulation) remains visible on every screen.

CSP in the HTML document: `default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'none'; connect-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'`. No inline scripts, eval, workers or service worker. All imagery is local SVG/fixture. The first loopback requests for HTML/CSS/JS are necessary; after that the app makes no network requests. Browser/extension traffic outside the app falls outside this prototype claim.

## 8. Logical interface for prototype and later port

This is a UI contract, not a replacement for the kernel ABI. Object IDs are UI references and do not themselves grant permissions.

```text
Object { id, title, mediaType, collectionId, revision, localState, nasState }
GrantRequest { toolId, objectIds[], operations[], lease: "session", internet: false|destination }
Grant { id, toolId, objectIds[], operations[], expiresOn: "objectClose", state }
NasState = unpaired | verifying | online | offline | networkChanged | identityMismatch
AiState = ready | missingModel | lowMemory | running | cancelled | result
```

Mock broker operations: `listObjects`, `openObject`, `requestGrant`, `grant`, `deny`, `revoke`, `saveRevision`, `restoreAsNewRevision`, `pairNas`, `setNasCondition`, `installFixture`, `summarizeFixture`. Use return codes `DENIED`, `NOT_FOUND`, `STALE_REVISION`, `NAS_UNAVAILABLE`, `IDENTITY_MISMATCH`, `MODEL_MISSING`, `RESOURCE_LIMIT`, `INVALID_PACKAGE`. No screen may translate a failure into a success status.

A later native adapter replaces the mock; interfaces for capabilities, revocation, versions and error handling must then be connected to real services. The UI itself is never the security enforcement layer.

## 9. Acceptance for the next builder

1. **P2-01:** all seven routes and mandatory states exist; five tasks from §2 fully walked through with fixtures.
2. **P2-02:** denied and revoked permissions block the corresponding mock operations; installation creates zero grants.
3. **P2-03:** NAS offline/wrong identity/network transition shows correct recovery behaviour; local saving keeps working and no external fallback is offered.
4. **P2-04:** AI without model/memory shows exclusively local alternatives; the example answer is recognisably simulation.
5. **P2-05:** restoring an older revision creates a new revision; the earlier history remains intact.
6. **P2-06:** all controls by keyboard, focus restoration and 200% zoom; contrast evaluation of actually used states.
7. **P2-07:** screenshots at 1440×900, 1024×768 and 390×844; additionally reflow at 320 CSS px. No clipping, hidden primary actions or unexpected overlap.
8. **P2-08:** inspect app network traffic during all flows: exclusively initial own loopback assets; no fetch, websocket, beacon, analytics, external font or remote image.
9. **P2-09:** every error/empty/loading screen uses the same component rules and concrete text. No decorative working-OS or hardware claims.
10. **P2-10:** deliver real test results and screenshots plus a list of checks not carried out. A screenshot does not prove accessibility, system security or NAS pairing.

P2 is **ready to build at document level**. No users have tested these concepts and no interface has been built yet. A first real user trial follows after P2; adjust the chosen direction then based on concrete observations, rather than starting a general brainstorm again beforehand.

## Sources (APA 7)

Apple. (n.d.). *Work in multiple spaces on Mac*. Retrieved 26 October 2026, from https://support.apple.com/en-sg/guide/mac-help/mh14112/mac

Flatpak Project. (n.d.). *Sandbox permissions*. Retrieved 26 October 2026, from https://docs.flatpak.org/en/latest/sandbox-permissions.html

Microsoft. (n.d.). *Snap your windows*. Retrieved 26 October 2026, from https://support.microsoft.com/en-us/windows/experience/snap-your-windows

World Wide Web Consortium. (2024, December 12). *Web Content Accessibility Guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/

Token values, concept choice, task scenarios, interfaces, screens and acceptance tests are own project decisions. The sources are only used for the named existing patterns and accessibility criteria.
