<a id="overstapfrictie-tijdens-installatie-en-de-eerste-gebruiksdag"></a>

# Switching friction during installation and the first day of use

Researched: 2 November 2026. Phase 4.6. Status: documented platform differences and testable hypotheses; no original user study. The broader adoption, app, hardware and European barriers are covered in [phase 3](2026-11-02/european-adoption-barriers.md). The original Eurobarometer remains precisely bounded there: switching services and unspecified additional costs, not OS adoption.

<a id="onderzoeksvraag-en-methode"></a>

## Research question and method

Which existing knowledge helps with nieuw-os, where do incorrect expectations arise, and what product evidence is needed before a safe switch? The source route runs from [the installation matrix](os-installation-comparison.md) and [28 daily tasks](os-interaction-patterns.md) to [mental models](transferable-mental-models.md). Platform documentation describes available controls. It does not measure how many people know, prefer or successfully transfer those controls. The profiles below are therefore **research segments, not invented people or measured populations**.

The phase-3 sources CMA and vzbv support the existence of learning, ecosystem and compatibility barriers within their own populations. They provide no effect size for the onboarding proposed here. See the methodological limitations and primary sources [EA-05–EA-11 in the adoption dossier](2026-11-02/european-adoption-barriers.md#directe-bronverwijzingen).

<a id="herkomstprofielen"></a>

## Prior-platform profiles

The first column presents documented conventions from the task comparison. All expectations, priorities and interventions in the remaining columns are [ANALYSE]/[HYPOTHESE]. No prior-platform profile changes the core interface, security or permissions.

| Prior experience | Transferred conventions | Possible incorrect expectation | Support to test |
|---|---|---|---|
| Windows | Start/search field, taskbar, system tray, Verkenner (File Explorer), context menu, Alt+Tab, snapping and separate Instellingen (Settings); installation through the Store and other distribution routes | Closing a window means quitting the app; a downloaded `.exe` file works; the desktop is the default storage location; “Taakbeheer” (English: “Task Manager”) is the name of every system overview | Search aliases, visible window state, explanation of installing versus opening and an actual file-import test; test the launcher, window switching and data location first |
| macOS | Dock, Finder, Spotlight, global menu bar, Mission Control/Spaces, Quick Look, gestures, app bundles and Systeeminstellingen (System Settings) | Closing a window always leaves the app running; every keyboard has a Command key; menu-bar actions are outside the window; an `.app` can be transferred | Show the actual meaning of closing and the available shortcut on the action itself; contextual explanation of Materiaal, search aliases and the example “zoeken → openen → terug” (English: “search → open → back”) |
| GNOME | Overzicht (Overview) with Super, app search, activities/workspaces, top bar, Bestanden (Files) and Software | Minimize/maximize are always hidden; a dynamic workspace appears automatically; extensions and GNOME packages work | Show nieuw-os window controls; one optional explanation of the workspace contract when available; test ordinary tasks without an extension or terminal |
| KDE Plasma | Toepassingenstarter (Application Launcher), panel, system tray, Dolphin, KRunner, Systeeminstellingen (System Settings), Discover, extensive customization | Panels/widgets and all shortcuts are identically configurable everywhere; “Task Manager” unambiguously means a process window | An ambiguous search term offers a choice of meaning; the default route remains stable; test advanced preferences separately from necessary controls |
| Ubuntu Desktop | GNOME base with Ubuntu modifications, including a dock and its own software/installation routes | Every GNOME route or package name is identical to Ubuntu; Ubuntu instructions apply to the independent nieuw-os kernel | Use both distribution and desktop as background information; no unrestricted “Linuxmodus” (English: “Linux mode”); familiar verbs and a visible package source |
| First computer / no preference | No platform convention assumed | Icons, double-clicking, dragging, disk erasure or online accounts are self-explanatory | Text labels, a safe trial action, keyboard help and explicit consequences; measure without inferring from age, nationality or education |

The macOS route also differs between app switching and window switching; GNOME and KDE have different workspace and launcher contracts. Prior experience with “Linux” is therefore insufficient for an exact shortcut promise. Version, settings, hardware and assistive technologies can further change each profile. [Task routes and limitations](os-interaction-patterns.md)

<a id="risicoregister-voor-de-overstap"></a>

## Switching risk register

Impact and priority are design assessments, not measured incident frequencies. **Blocking** means that a route must not be offered as a supported migration before the required evidence exists.

| Risk | Consequence | Design/engineering requirement | Required evidence |
|---|---|---|---|
| Wrong target disk or unknown encryption | Irrecoverable loss of source data; blocking | No automatic erasure, shrinking, conversion or decryption; show identity and the change plan before authorization | Disposable disk images with multiple, full, unknown and encrypted disks; hotplug/stale-plan tests; human comprehension test |
| Missing recovery/encryption key | User loses access; blocking | Provide actual key management and recovery; no fake “versleuteld” (English: “encrypted”) choice or implicit provider escrow | Cryptographic/service evidence, recovery with a new session and loss/interruption cases; security review |
| Loss of a necessary program | Work or play becomes impossible; blocking for that user | Check compatibility per app, version and workflow; no generic Windows/macOS/Linux compatibility claim | Execution on the independent OS, including plug-ins, authentication, offline use and relevant anti-cheat; [phase-3 app sources](2026-11-02/european-adoption-barriers.md#barrièrematrix) |
| Files open but are written back incorrectly | Layout, macro or data loss | Preserve the original; sample import and round-trip check; do not report “volledig gemigreerd” (English: “fully migrated”) based on copying alone | Spoken task criteria plus file types, counts, hashes where appropriate, permissions and change reports |
| Missing driver or assistive technology | No network, audio, display, input or independent operation; blocking | Show the exact supported profile before installation; no automatic physical experiment | Tested hardware/assistive-technology matrix; explicitly separate VM evidence from physical support |
| Different search term for the same function | Help needed or incorrect action | Local synonyms point to real targets with their own names; visible ambiguity | Term→target trials per prior platform/language, unknown term and missing target; zero automatic execution |
| Different meaning of closing/minimizing | Work state misjudged | One visible window/app contract; protect unconfirmed work | Saving work, closing, reopening and crash recovery as separate tasks |
| Missing phone, cloud or organizational identity | Loss of shared workflow; potentially blocking | Do not implicitly claim integration; usable local core; optional linking with a defined scope | Actual export/import/unlinking trials, multiple accounts, management policy and offline route |
| “Snel geholpen” (English: “quickly helped”) by invisible automation | Unexpected permissions, network use or system action | Hints and search results execute nothing; consequences shown before a targeted action | Negative grant, network and broker tests; do not infer permission from the prior-platform choice |
| Unknown maintenance/support route | Trying does not lead to continued use | Discoverable limits, recovery and responsible owner; understandable policy | Task “updateprobleem oplossen” (English: “resolve an update problem”) with an actual way back and a reachable support channel |

The project's [repository baseline](2026-11-02/repository-baseline.md) does not prove physical installation, app compatibility, general file transfer, personal authentication or full accessibility. The newer native source has not demonstrated that complete migration chain either. Better explanations alone do not resolve these risks.

<a id="verwachtingscompatibiliteit-als-productstrategie"></a>

## Expectation compatibility as a product strategy

[VOORSTEL] Preserve the meaning of opening, selecting, copying, pasting, closing, going back and finding settings. Let the existing terms Finder, Explorer and System Monitor serve as search entry points while one nieuw-os name remains visible. Honestly show an unknown or undelivered function as missing. The [onboarding specification](../product/new-os-onboarding-spec.md) defines ambiguity and boundaries.

[VOORSTEL] Make the question “Waar kom je vandaan?” (English: “Where are you coming from?”) optional and ask it only after a usable session. Use the answer for temporary guidance and relevant terms. The user can skip it, change it later or erase it. The answer grants no capabilities, changes no account/cloud policy and selects no different desktop model.

[ANALYSE] A visible installation and recovery route reduces a concrete risk; whether it actually increases trust or switching still requires measurement. The preferred experiment compares the same tasks and interface with and without contextual guidance. A preference judgment after a screenshot does not count as demonstrated day-one usability.

<a id="validatie-vóór-een-migratieclaim"></a>

## Validation before a migration claim

1. Select Windows, macOS, GNOME, KDE and Ubuntu users based on tasks they have actually performed; record version, keyboard and assistive technologies. Give “geen voorkeur” (English: “no preference”) its own route. Collect no passwords or actual recovery secrets.
2. Using synthetic data, ask participants first to explain the installation choice and then perform the relevant 28 tasks. Code expected versus actual effects, assistance, wrong destinations, backtracking and recovery.
3. Test professionally necessary apps, formats and hardware before offering a switch. A low click burden does not compensate for missing work functionality.
4. Report each prior-platform segment separately. A small exploratory sample identifies problems and hypotheses, not representative success rates for Europe.
5. Measure trying, successful migration, voluntary continued use and reasons for returning separately. No answer to a hypothetical service question replaces these observations.

Measurement targets, safety criteria and outstanding evidence are recorded in [interaction-principles.md](../product/interaction-principles.md). No participants were recruited and no installations or user measurements were performed during this research phase.
