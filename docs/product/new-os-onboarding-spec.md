<a id="nieuw-os--onboarding-vanuit-bestaande-kennis"></a>

# Nieuw-os — onboarding built on existing knowledge

Date: 2 November 2026. **Status: design specification for later research and implementation; neither implemented nor validated by users.** Supporting analysis: [mental models](../research/transferable-mental-models.md), [switching friction](../research/switching-friction.md) and [28 daily tasks](../research/os-interaction-patterns.md). This addition changes no current P2 route or accepted native evidence.

<a id="probleem-en-keuze"></a>

## Problem and design decision

[ANALYSIS] People may search for a familiar function under another name or expect different window-control behaviour. The underlying platform differences are documented; their extent in nieuw-os has not yet been measured. [PROPOSAL] One interface and one system contract, with optional guidance based on prior experience. A choice of prior platform is a cue for explanation, not a reliable assessment of skill.

No four desktops, mandatory tour, marketing carousel, cloud-provider preference or AI chat as a gateway to ordinary actions. The [interaction principles](interaction-principles.md) determine how the effect is tested.

<a id="eerste-sessie-en-overdracht-uit-de-installer"></a>

## First session and installer handover

1. **Session status:** show whether this is a trial, personal or managed session. Do not describe a current public VM/guest session as securely signed in. A future personal account first requires real authentication and storage isolation.
2. **Immediately usable:** the desktop is reachable once the necessary configuration is complete. Show at most a non-blocking offer “Wegwijs in nieuw-os”, with “Later” and “Overslaan”. Accessibility controls remain available from the first screen.
3. **Optional question:** “Met welk systeem werk je meestal?” Answers: Windows, macOS, Linux, eerste computer / geen voorkeur. For Linux, GNOME, KDE Plasma or Ubuntu is an optional follow-up question; “Weet ik niet” remains valid. Multiple experiences can be specified later in help settings; no migration history is inferred.
4. **Brief task help:** offer opening/searching, switching windows, saving/finding material and settings/permissions as separate, skippable exercises. Use recognisably synthetic practice material. Do not erase or overwrite a real document to demonstrate a lesson.
5. **Migration separate:** checking apps/devices and importing data has its own resumable route. Show source, destination, trial result and limitations. Request source permissions only at import time; preserve the source. An answer about prior platform is not permission for disk access.
6. **Optional services later:** offer phone, NAS, cloud and local AI only for a relevant task, with existing privacy/capability gates. No step is required to use the local foundation.

[PROPOSAL] A signed/authenticated installation handover record contains only the version number, completed necessary steps, selected language/input/accessibility and references to the account/storage configuration actually created. It contains no password, unlock key, recovery code or provider token. A renderer must not fabricate completed security steps. First boot validates the actual state; if incomplete, it shows the concrete recovery route and does not silently write to the disk again.

Region, encryption, privacy and password are not requested again because a tour starts. The future state model distinguishes `niet aangeboden`, `uitgesteld`, `bezig`, `overgeslagen` and `afgerond`; skipping is a valid final state. “Later” remains reachable through Help/Instellingen, without notifications that repeatedly interrupt the user.

<a id="zoekcompatibiliteit-zonder-nieuwe-naamsystemen"></a>

## Search compatibility without new naming systems

[PROPOSAL] A local, versioned synonym register maps familiar terms to existing nieuw-os destinations. Search shows the nieuw-os name of the result, a short description, availability and optionally “Ook bekend als …”. It uses neither the internet nor a language model. All supported terms are available to everyone; prior platform affects at most the initial explanation, not discoverability or access rights.

This is a register contract, not an existing API. Each item requires: intent ID, nieuw-os label, language, aliases, type (`app`, `bestand`, `instelling`, `actie`, `hulp`), destination ID, availability, potential ambiguity and required next step. IDs grant no rights.

| Familiar search term | Intended concept / proposed nieuw-os result | Required boundary |
|---|---|---|
| Verkenner, Explorer, File Explorer, Finder, Bestanden, Files, Nautilus, Dolphin | Materiaal — finding files and documents | The current Materiaal route is limited; no claim of general disk/network management. Show only actually available sources and operations |
| Task Manager, Taakbeheer, Activity Monitor, Activiteitenweergave, System Monitor, Systeemmonitor | Future system overview | Do not currently show as a working app. In KDE, “Task Manager” can also mean the taskbar widget: show meanings without automatically terminating a process |
| Settings, Instellingen, System Preferences, Systeemvoorkeuren, System Settings, Systeeminstellingen | Future unified settings | Open only existing categories; explicitly report a missing setting |
| Spotlight, KRunner, Start, application launcher | Search / open apps | “Start” may also be an action. Show context and category; no visual copy required |
| Wifi, Wi-Fi, WLAN | Network — wireless connection | Offer only when a real network service and management exist; no password in the search index or query history |
| Sound, geluid, audio output, geluidsuitvoer | Sound — output device | Do not switch a connection without a visible choice; show selected device and effect |
| Display, beeldscherm, resolution, resolutie | Display | Only supported modes; future confirmation/fallback for an unreadable display required |
| Do Not Disturb, Niet storen, Focus | Notifications and focus | Muting ordinary notifications differs from hiding system errors or recording/privacy indicators |
| Windows Update, Software Update, Discover, Software | Apps/updates as distinct results | Store, package installation and system update are different intents; do not automatically select a package source or update |
| Quick Look, preview, voorbeeld | Materiaal — preview where available | Do not promise support for unknown formats; source quotation (Dutch): “niet uitvoerbare inhoud openen als app”. Ambiguity note (English): the source wording is ambiguous; clarification is required before implementation. |

Translation variants and capitalisation are explicitly normalised. Diacritic variants may improve discoverability; they must not merge two security-sensitive destinations. Substrings, typos and ambiguous terms receive a list with text labels. A query is always inert: no shell interpretation, URL opening, permission request or change during typing. Actions show consequences and require separate activation; system privileges remain with the broker/service.

Availability is determined by the actual service/capability state. A planned component appears at most as clearly marked help with “nog niet beschikbaar”, not among executable results as though it works. The current fixed Materiaal search terms do not implement this OS-wide register. [Repository baseline](../research/2026-11-02/repository-baseline.md)

<a id="hulp-per-herkomst-hetzelfde-gedrag"></a>

## Guidance by prior platform, same behaviour

| Context | Temporary explanation | What remains the same |
|---|---|---|
| Windows | Where apps/windows are located, how to switch and work side by side, where files are stored | Same launcher, window buttons, shortcut labels, permissions and storage rules |
| macOS | Difference between app and window state, location of actions, actual Command/Ctrl mapping on this keyboard | Do not simulate a global menu bar if none exists; no unverified gesture promise |
| GNOME | Overview/search, window controls and workspace behaviour if provided | Do not implicitly adopt extensions or dynamic workspaces |
| KDE/Ubuntu | Project settings/software-source contract, synonyms and panel status | Do not claim package compatibility or unrestricted panel configuration |
| Geen voorkeur | Text labels, selecting/opening, going back and a safe practice action | Full access to the same controls, without lower privilege or a childish tone |

The hints are a [HYPOTHESIS]; retain or remove them based on task observation. A user can dismiss hints by topic. Do not repeat a dismissed hint after a reboot or software update unless a relevant control actually changes and the change is explained.

<a id="privacy-toegankelijkheid-en-prestaties"></a>

## Privacy, accessibility and performance

Store optional experience information and completed lessons locally for each actually supported profile. Provide viewing, editing and deletion. Do not infer a server profile, telemetry, synchronisation or consent from the choice. Do not claim multi-user privacy without real profile isolation. The existing [local privacy contract](../LOCAL-PRIVACY-CONTRACT.md) remains authoritative for networking, NAS and AI.

Every exercise works with a keyboard and named controls; focus returns to the initiating action after closing. No timer, mandatory dragging or reliance solely on colour/animation. Show the actual, reconfigurable shortcut if provided. Help must not reset text scale, contrast or reduced motion. The local synonym register has a bounded index and makes no network request; response time and memory use receive budgets only after an actual baseline.

<a id="betrokken-onderdelen-en-acceptatie-vóór-uitvoering"></a>

## Components involved and acceptance before implementation

Future components: session status, first-boot coordinator, guidance panel, local preference store, search register and help/migration route. Current relevant locations are `docs/implementation/PRODUCT-SPEC.md`, `ui/prototype/` as an explicit simulation and the separate native shell/service code in the development source. No files in those product paths are modified in this phase. A concrete native file layout requires a new inspection of current source; this specification does not imply that a service already exists.

| ID | Acceptance proposal | Verification |
|---|---|---|
| ON-01 | First use, skipping and resuming work without an answer about prior platform | State/route trial including reboot and incomplete handover; no duplicate security question |
| ON-02 | Identical core controls, search destinations and privileges for all prior-platform choices | Compare routes and capability requests; changing prior platform changes no grants |
| ON-03 | Every recorded alias finds the correct available destination or an honest missing/selection status | Register tests by language; ambiguity, missing service, collisions, typos and inert typing |
| ON-04 | Guidance deletes itself on request without changing data or settings of other tasks | Profile/retention and deletion trial with synthetic data |
| ON-05 | No barrier for keyboard, assistive tool, scaling or reduced motion | Manual focus/screenreader/touch trial in an actually supported environment; report missing evidence |
| ON-06 | Hints reduce incorrect expectations without adding new basic steps | Exploratory task trial by prior platform; compare help, wrong destination and task completion against a route without hints |

Risks: incorrect categorisation, unwanted profiling, outdated aliases, ambiguous platform terms and help text that conceals a missing system function. Preferred implementation: first a bounded local register/help experiment, then real native integration once the stated services and accessibility exist. Do not start with an automatic desktop mode or a model that freely interprets actions.
