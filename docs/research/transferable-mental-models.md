# Transferable mental models and compatibility of expectations

Research date: 2 November 2026. Phases 4.5/4.6/4.8. Status: source comparison and explicit design hypotheses; no original interviews or implemented interactions.

Guiding user principle: **“Familiar where familiarity reduces friction, innovative where innovation creates measurable value.”** This is the assignment, not a research result.

## Evidence and boundaries

[SOURCE] Nielsen's heuristics include recognition, consistency, user control, and error prevention. The W3C's supplemental cognitive guidance recommends familiar, consistent controls and clear relationships between controls and content. These are design and accessibility guidelines, not an effectiveness trial for a new desktop. [J01] [J03] [J04]

[SOURCE] Mental models differ between individuals and change through experience. The existing comparative operating-system study in the [Windows/macOS record](2026-11-02/windows-macos-needs.md) used operating-system images and a limited student sample; it does not demonstrate knowledge of a complete installation or working day. [J02]

[ANALYSIS] Matching documentation demonstrates that a concept occurs in multiple systems. It does not demonstrate that every user knows it, assigns the same meaning to it, or can use it without assistance. Consequently, **widely shared** below means that a concept was found on multiple platforms examined. Task routes, terms, keyboard shortcuts, and source versions are recorded in the [28-task comparison](os-interaction-patterns.md); installation concepts are recorded in the [installation matrix](os-installation-comparison.md).

## Comparison of models

Factual platform differences are derived from the cited task routes; transfer implications are [ANALYSIS].

| Model | What is widely shared | Fundamental difference / transfer risk | Implication for Baryon OS — hypothesis |
|---|---|---|---|
| Desktop | A signed-in work environment with applications and system status | A desktop need not be a storage surface for icons; GNOME relies heavily on the overview | Provide one stable home location; do not require desktop files |
| Window | A separately visible work area with focus | Application, window, and document are distinct identities; closing does not terminate an application everywhere | Give Close a predictable window/document effect; treat Quit application separately |
| Close/minimise/maximise | Visibility and size can be controlled | macOS uses a different position and different full-screen/tiling semantics; GNOME does not provide all title-bar buttons by default | Name functions and provide alternatives; do not assume colour or button position is universal knowledge |
| Dock/taskbar/panel | Open applications and/or return to work in progress | A pinned application and an active window are not the same; the GNOME dash differs from the Ubuntu dock | Distinguish launching, active work, and the selected window |
| Launcher/Start | A discoverable entry point to programs | The Start menu, Activities, Launchpad/Spotlight, and KDE Launcher differ in structure | Provide a consistent custom launcher with the text Search and programs; use familiar terms as discovery aids |
| System tray/status area | Network, sound, and system status are grouped | Application tray, menu extras, and quick settings have different boundaries | Distinguish persistent system status, application status, and notifications |
| Quick settings/Control Centre | Frequently used switches near status | A Wi-Fi icon may open a toggle or a list; clicking the icon sometimes differs from clicking an arrow | Provide separate, recognisable actions for on/off and network selection; label the consequences |
| Notification centre | Temporary messages can be found later | Location, calendar integration, grouping, and exceptions differ | Provide one overview with source, time, and action; focus is a different policy from access or device status |
| File management | Named files, folders, locations, and opening with an application | Finder, Explorer, Files, and Dolphin do not share every metadata and movement convention | Retain name/folder/location/import/export alongside Materiaal collections |
| Settings | A recognisable system component for preferences | Categories and application/system boundaries differ; GNOME and KDE are not one uniform Linux structure | Reach the same setting through both a category and a search result, leading to the same state |
| Search | Enter text to find or open something | Application search, content index, web result, and action have different scopes | Show result type, location, and authority; entering text alone must not execute an action |
| Drag and drop | Move an object to a destination | Copy/move/alias depends on context and modifier keys | Show the result in advance; provide a button/context-menu alternative; an invalid destination changes nothing |
| Context menu | Relevant actions for an object | Right-click, Control-click, menu key, and touch differ | Provide the same commands through menus and the keyboard; no essential action is available only through right-click |
| Keyboard shortcuts | Fast actions for selection, navigation, and editing | Ctrl versus Command; application switching versus window switching; settings are configurable | Show actual bindings; aliases must not promise unsupported key mappings |
| Workspaces/Spaces | Divide work into multiple groups | Number, dynamics, direction, and full-screen behaviour differ | Provide an optional, visibly named workspace model; it is not required to open one application |
| Installation | Make a program available and removable | Store, installer, package, application bundle, and permissions have different meanings | Explain provenance, support, consequences, and removal together; installation is not a general data grant |

## Migration profiles: expectations, not knowledge about individuals

[HYPOTHESIS] These profiles are discussion and test segments. They say nothing about an individual participant until that participant's actual experience has been elicited. A person may use multiple operating systems or may never have used many components.

| Origin | Plausibly transferred knowledge based on existing routes | Confusion/guidance to test |
|---|---|---|
| Windows | Start, taskbar/system tray, Explorer, Alt+Tab, edge snapping, right-click, shortcuts, Settings, and different installation sources | An application tile is not a file; a collection is not a move operation; closing is not deletion. Map Explorer to file/Materiaal navigation and Task Manager to the actual system overview once it exists |
| macOS | Dock/top menu bar, Finder, Spotlight, Mission Control/Spaces, gestures, Quick Look, application bundles, and System Settings | Do not equate Ctrl with Command or application switching with window switching. Explain different close/maximise semantics without reproducing the menu bar or Finder |
| GNOME | Activities/overview, Super search, dash, dynamic workspaces, Files, and quick settings | Do not impose an expectation of a classic taskbar; provide visible application switching. Do not present absent desktop icons as an error |
| KDE Plasma | Launcher, panel/task manager/system tray, KRunner, Dolphin, System Settings, configuration, and virtual desktops | “Task Manager” may mean a panel widget or process overview; search results must disambiguate. Configuration must not change the core meaning |
| Ubuntu GNOME | The GNOME base with distribution choices such as a visible dock and App Center | Do not project Ubuntu experience automatically onto upstream GNOME; package source and version remain relevant |
| First computer/no preference | No operating-system-specific knowledge assumed | Provide labels, examples, default routes, and progressive detail; no mandatory course or hidden shortcut |

## Compatibility of expectations — proposal

1. **One concept, one effect.** The same button, search action, and keyboard shortcut perform the same operation. Origin selection does not change storage, permissions, authentication, close policy, or network access.
2. **Familiar words support discovery.** Support targeted synonyms for programs, file management, and system functions. Show the custom name, a short meaning, and availability. An alias for an absent component does not produce a fake result.
3. **Guide differences at the moment of action.** Examples include the first Close action or the first search for Finder. A short dismissible card is a hypothesis; repeated hints and task interruptions are measured.
4. **Keep basic tasks independently usable.** Search, folders, menus, and visible buttons remain usable without AI, gestures, an origin profile, or prior knowledge of Materiaal.
5. **Investigate comprehension, not recognition alone.** Before execution, ask which effect a participant expects and compare it with the actual result. Count silent misunderstandings, recovery work, and assistance alongside task success.

Aliases and local preferences are elaborated in the [onboarding specification](../product/new-os-onboarding-spec.md). This is not yet an implementation.

## Familiarity budget

The categories are a **project weighting**, not a scientific score. The default is KEEP FAMILIAR. EVOLVE requires a named problem and a measurable expectation. REINVENT/NEW additionally require demonstrable value beyond a familiar variant and preservation of basic tasks.

| Component | Category | Rationale/evidence boundary |
|---|---|---|
| Forward/Back/Cancel, clear focus and labels | KEEP FAMILIAR | Recognition/control from guidelines; exact task route still requires testing |
| Name/folder/location, copy/move, and new folder | KEEP FAMILIAR | Widely present task concepts; collections are additive |
| Window focus, closing, application switching | KEEP FAMILIAR | Keep meaning predictable; position/style may be distinct |
| Status area, sound, network, and locking | KEEP FAMILIAR | Directly accessible system actions; not hidden behind an assistant |
| Context menu and visible alternatives | KEEP FAMILIAR | More than one input method; equivalent commands |
| Disk selection, preserve/erase, and confirmation | KEEP FAMILIAR | Hiding an irreversible consequence is not meaningful innovation |
| Settings with categories and search | EVOLVE | Less navigation work as a hypothesis; one shared setting |
| Search results with familiar aliases and visible scope | EVOLVE | Earlier research shows specific search friction; quality still requires testing |
| Installer with safe preview and explicit change plan | EVOLVE | The same familiar decisions with better presentation of consequences; no removal of a necessary choice |
| Privacy choices and access recovery | EVOLVE | Combine consequence, duration, and revocation; avoid repeated permission requests |
| Mandatory vendor account as a gate to local tasks | REINVENT | Deliberately removed from the Baryon OS funnel under the existing privacy contract; no universal operating-system pattern or measured-effect claim |
| Materiaal as a persistent identity above tools | NEW for people migrating | Existing project concept; introduce the new metaphor only gradually alongside folders/opening |
| Object-centred access/recovery inspection | NEW for people migrating | Optional detail, not an additional learning gate before every file |

**Budget rule — hypothesis:** no new metaphor is required for the irreversible disk choice or for the 28 basic tasks. Introduce Materiaal in one optional example task; defer the second new inspection metaphor until it is needed. This is a governance rule intended to limit learning burden, not a demonstrated numerical upper bound. Do not add another REINVENT/NEW item without a variant comparison and a regression test of familiar tasks.

<a id="bronregister-voor-deze-analyse"></a>

## Source register for this analysis

All sources were accessed on 2026-11-02. Platform manuals and versions are recorded in the two linked comparisons; the source families below support the analysis model.

| ID | Original source/date | Use and limitation |
|---|---|---|
| J01 | Jakob Nielsen, *10 Usability Heuristics*, 1994-04-24; revised 2024-01-30 | Consistency, recognition, control, and error prevention. Heuristics, not a current operating-system user measurement |
| J02 | Megan Chan, *Mental Models*, 2024-01-26; revised 2026-09-01, reworking Nielsen 2010 | Individual expectations and incorrect models; examples/guidance, not a migration RCT |
| J03 | W3C COGA, *Use a Familiar Hierarchy and Design*, source note 2021-04-29; web presentation 2022-01 | Cognitive accessibility and recognition; supplemental guidance, not a WCAG conformance criterion |
| J04 | W3C COGA, *Help Users Understand What Things are and How to Use Them*, same note/web period | Clear controls, relationships, and consistency; the web context requires testing for native use |
| [J05] | Microsoft Support, *Change or reset your PIN in Windows*, no date stated | A PIN is not the same as a Microsoft account password; not an implementation example for Baryon OS |
| [J06] | Baryon OS privacy contract revision 4, 2026-10-28, and source baseline main `e08fb0e`/development source `1f0dd9c` | Local operation, no mandatory vendor account, genuine permissions, and open native boundaries; project conditions, not external user data |

[J01]: https://www.nngroup.com/articles/ten-usability-heuristics/
[J02]: https://www.nngroup.com/articles/mental-models/
[J03]: https://www.w3.org/WAI/WCAG2/supplemental/patterns/o1p02-familiar-design/
[J04]: https://www.w3.org/WAI/WCAG2/supplemental/objectives/o1-understandable/
[J05]: https://support.microsoft.com/en-us/windows/security/change-or-reset-your-pin-in-windows
[J06]: ../LOCAL-PRIVACY-CONTRACT.md
