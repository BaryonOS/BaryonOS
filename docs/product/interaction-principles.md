<a id="nieuw-os--interactieprincipes-en-meetbare-ux-criteria"></a>

# Nieuw-os — interaction principles and measurable UX criteria

Date: 2 November 2026. Status: **proposed product and research criteria; no achieved UX score**. This document connects the [installation comparison](../research/os-installation-comparison.md), [daily tasks](../research/os-interaction-patterns.md), [mental models](../research/transferable-mental-models.md) and [switching barriers](../research/switching-friction.md) to behaviour that requires later testing.

<a id="ontwerpgrondslag"></a>

## Design foundation

**Familiar where familiarity reduces friction; innovative where innovation creates measurable value.** [ASSIGNMENT] The project's own identity lies in the coherence of material, actions, permissions and recovery. It requires no new meaning for opening, closing a window, copying or erasing a disk.

[SOURCE] Nielsen's heuristics recommend recognition, consistent controls, user control and error prevention. W3C's supplemental cognitive guidance recommends familiar controls. These are design guidelines, not evidence that this nieuw-os solution performs better. WCAG 2.2 is a web standard; the selected contrast, focus and input principles help the prototype and inform a native accessibility contract. They do not prove OS-wide conformance. [J01/J03 in the model register](../research/transferable-mental-models.md#bronregister-voor-deze-analyse), [W3C WCAG 2.2](https://www.w3.org/TR/WCAG22/) (12 December 2024; accessed 2 November 2026).

<a id="criteria-en-traceerbaarheid"></a>

## Criteria and traceability

The numbers below are **preselected test targets**, not research-derived universal thresholds or results already achieved. They will be revised after a pilot, with reasons recorded. An average does not compensate for a barrier or data loss. `IN-xx` refers to [the installer](new-os-installer-spec.md), `ON-xx` to [onboarding](new-os-onboarding-spec.md); TASK 01–28 refer to the task comparison.

| ID | Principle and proposed criterion | Supporting basis | Test and boundary |
|---|---|---|---|
| UX-01 | All 28 supported basic tasks are reachable without a manual; target for later confirmation: ≥90% independent task completion per prior-platform segment | Recognition/consistency [J01/J03]; documented TASK 01–28 | Observe first attempt, help, errors and result per task; report missing functions separately. A small pilot does not provide a reliable population percentage |
| UX-02 | Every destructive installation action shows destination and consequences before authorisation; zero unauthorised disk mutations in the recorded safety matrix | Installation steps 10–14/25; project safety decision IN-01/02 | Machine trials with disposable images plus asking participants to explain what is preserved/erased. “100% geslaagde testgevallen” (English: all test cases passed) does not constitute an absolute safety claim |
| UX-03 | Beginner path requires no input of EFI, GPT, mount point, filesystem or partition table | Differences in technical installer choices; cognitive guidance | IN-03 task trial and text audit; Advanced remains explicitly reachable with the same consequence check |
| UX-04 | Every provided setting is discoverable through a category and at least one ordinary search term | TASK 06, settings/search mental models | Coverage check of actually provided settings; human search task without first providing the setting's nieuw-os name |
| UX-05 | Every recorded prior-platform alias leads to the correct available destination, a clear choice or an honest missing status; query performs zero actions | Prior-platform profiles; ON-03 | Register trials per language and service version; include ambiguous “Task Manager”, misspelled term and missing service |
| UX-06 | The same action has the same outcome and access control through keyboard, mouse, touch or accessibility API | J01; WCAG input principles; TASK 13/16–19 | Complete route with keyboard and appropriate assistive tools; no mandatory drag/hover/gesture; verify physical touch and screenreader separately |
| UX-07 | Language, input and necessary assistive tools work before secrets and disk selection | Installation steps 3/5/6/14/17 | IN-04, focus order, screenreader, keyboard test and return without logging secrets |
| UX-08 | Only necessary information blocks installation; local core use requires no provider account, cloud, telemetry or AI | Existing privacy contract; source differences in account/network/OOBE | Offline path where payload is complete, refuse/skip, zero external fallback; missing real local services remain release blockers |
| UX-09 | Progress, remaining choice and cancellation boundary are understandable at all times | J01 status/control; installation 25–30 | IN-05; report phase and error, no invented percentage or time. Interruption after writing promises no undo |
| UX-10 | Basic controls use KEEP FAMILIAR or EVOLVE; REINVENT/NEW requires a problem, evidence and separate testing before inclusion | Familiarity budget and task differences | Design review per main route, beyond screenshots; the project's own Materiaal model must not be an entry requirement for a simple basic task |
| UX-11 | Text contrast ≥4.5:1, large text and meaningful UI boundaries/states ≥3:1; visible focus, reflow/scaling and reduced motion | Existing P2 contract, WCAG 2.2; project component choices | Measure actual states. 200% browser zoom/320 CSS px apply to the prototype; native text scale, focus and semantics require their own evidence. No complete WCAG claim from colour measurements |
| UX-12 | Guidance is skippable, resumable and locally deletable; prior-platform choice changes zero privileges or core routes | J02 individual models; ON-01/02/04 | State trials, comparison of all profiles, retention/deletion trial; do not infer prior platform from use |
| UX-13 | Data import leaves the source intact and distinguishes copied, verified and not imported | Switching/format barriers; existing recovery principles | Synthetic source, differing files/permissions/formats, interrupt/resume; no success based solely on a progress bar |
| UX-14 | Operation and evidence status are visible: a missing backend does not report a successful system action | Repository baseline; existing product/broker boundary | Actual service outcome and negative paths; prototype fixtures remain recognisable as simulation |

The 90% ambition in UX-01 is a product objective for a later, sufficiently large confirmatory study, not a requirement to claim a percentage from a handful of participants. Time to result, number of conceptual steps, wrong destination, help and recovery are also reported. Speed alone does not determine the winner: one extra understandable check before erasure may be necessary.

<a id="familiarity-budget-toepassen"></a>

## Applying the familiarity budget

The complete classification is in [transferable-mental-models.md](../research/transferable-mental-models.md#familiarity-budget). Application to each change:

- **KEEP FAMILIAR:** meaning of files/folders, selection, window controls, status, back/cancel and explicit destination-disk selection. Deviation requires more than a style preference.
- **EVOLVE:** settings plus search, familiar aliases, installer change plan and contextual permissions. Existing purpose and verbs remain recognisable.
- **REINVENT:** remove a mandatory provider-account gateway for the local core under the existing privacy contract. This does not mean alternative authentication has already been built.
- **NEW:** Materiaal/tool concept and inspection of object permissions/recovery only where they improve a real task; separate explanation and testing, without a new metaphor for erasure.

This is a qualitative budget. No source supports a magic percentage of innovation. Not recommended: four shells by prior platform, a new unlabelled icon for a critical choice, provider login for basic use, automatic selection of a disk to erase, unrestricted AI actions and purely decorative futuristic effects.

<a id="onderzoeks--en-verificatieplan"></a>

## Research and verification plan

**Technical suitability first:** document actually supported account, storage, network, update and assistive-tool routes. Removing a task does not turn a missing function into a usability success. Test unknown disks, changing device identity, corrupt planning, power interruption and recovery with disposable VM images before anyone could be permitted to use a real disk.

**Exploratory pilot:** proposal for 8–12 participants with varied prior platforms and assistive tools, deliberately recruited for relevant tasks. This number is a planning choice; it does not provide representative or statistically reliable per-OS findings. Provide synthetic data, a safe VM/prototype environment and no real secrets. Record prior-platform version and daily tasks without treating prior platform as a skill score.

**Tasks and outcomes:** first ask what the participant expects, then let the participant perform the task. Define a consistent initial state and desired end result in advance. Measure unaided task completion, wrong destination, serious error, recovery, need for help and time. Code understanding of installation (“welke gegevens verdwijnen?”; English: which data will disappear?) separately from the ability to press a button. Do not compare a source route and nieuw-os using a click ranking: account, safety and hardware prerequisites differ.

**Confirmation:** only after the pilot, design a larger sample and analysis by prior platform/task, with uncertainty margins, predefined outcome definitions and appropriate power/precision calculations. Test hints in the same interface with and without guidance. Test preservation/improvement of existing routes and voluntary migration/retention separately; a preference vote is not evidence of switching.

**Technical regression:** later implementation requires state/broker tests, build/type/lint checks where applicable, real keyboard/focus/screenreader trials, contrast and scaling/reduced motion, responsive layout and recovery after interruption. At present, only document consistency, source routes, matrix coverage and repository scope have been checked; there is no user measurement or new native acceptance.

<a id="besluit-en-open-risicos"></a>

## Decision and open risks

Preferred direction: one consistent core with familiar basic actions, contextual help and explicit consequences. Supporting guidelines make this defensible to test; they do not yet demonstrate new product value. The main risks remain missing compatibility/services, insufficient installer/encryption safety, still-incomplete native accessibility, source-version drift and a small or selective later sample. New implementations remain subject to the gates in [the existing product plan](../implementation/PRODUCT-SPEC.md).
