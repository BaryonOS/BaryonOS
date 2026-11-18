<a id="fase-4--installatie-eerste-gebruik-en-herkenning"></a>

# Phase 4 — installation, first use and familiarity

Researched on **2 November 2026**. An extension of [the phase 2–4 research](../README.md), documentation only. No installer/onboarding was built. No installation images were downloaded, no real disks were used, no OS sessions were completed and no participants were tested.

<a id="onderzoeksvragen-en-methode"></a>

## Research questions and method

Which installation decisions are shared, where do actual routes differ, which first-day actions are transferable, and which expectations should nieuw-os support? Which innovation solves a concrete problem without requiring familiar basic actions to be relearned?

Search route: current official downloads/releases → version-bound installation/recovery guide → desktop task manuals → fixed source-code tag/configuration when the guide is too general, outdated or contradictory. Windows/macOS, Linux installers and daily controls were researched in parallel and merged centrally. Search clusters: Windows 26H2 media/Setup/OOBE, macOS Recovery/Setup Assistant/FileVault, Ubuntu 26.04 installation, Fedora Live Anaconda hidden pages, Kubuntu Calamares manifest/patches, GNOME 51 and Plasma 6.8 task routes, mental models and W3C cognitive accessibility.

Preference was given to supported Ubuntu/Kubuntu LTS releases and the current stable Fedora Workstation. Upstream GNOME 51/Plasma 6.8 daily controls are not attributed to distributions containing GNOME 50/Plasma 6.6. Dynamic support pages are not a pinned ISO capture. Source code demonstrates configuration/logic, not end-to-end success.

Labels: **[BRON]** verifiable description, **[ANALYSE]** interpretation, **[HYPOTHESE]/[VOORSTEL]/[ONTWERPKEUZE]** direction still to be tested, **[REPOSITORY]** precisely bounded code/evidence baseline. Conceptual step counts are document analysis, not measured clicks, times, error probabilities or demonstrated familiarity. Unestablished details remain visibly unknown.

<a id="leesroute-en-geleverde-artefacten"></a>

## Reading route and delivered artifacts

| Document | Contribution |
|---|---|
| [Installation comparison](../../os-installation-comparison.md) | 31 aspects × Windows/macOS/Ubuntu/Fedora/Kubuntu plus an explicitly future nieuw-os column; actual sequence, A/B/C, safe automation |
| [Windows/macOS source dossier](windows-macos-installation.md) | 31 aspects, clean/in-place/Recovery/Intel variants, WI source register |
| [Linux source dossier](linux-installation.md) | 31 aspects, distribution profiles/versions/patches, LI source register |
| [Daily interaction](../../os-interaction-patterns.md) | 28 tasks × Windows/macOS/GNOME/KDE/nieuw-os, counting method, terms/icons/keys, UI source register |
| [Transferable mental models](../../transferable-mental-models.md) | Shared concepts, prior-platform profiles, expectation compatibility, familiarity budget, J source register |
| [Switching friction](../../switching-friction.md) | Prior-platform-specific hypotheses, compatibility and installation risks; connection with phase 3 |
| [Installer](../../../product/new-os-installer-spec.md) | Full justified funnel, step contracts, boundary before every mutation, open backend gates and IN acceptance |
| [Onboarding](../../../product/new-os-onboarding-spec.md) | One UI, optional guidance, locally inert alias register, honest missing status and ON acceptance |
| [Interaction principles](../../../product/interaction-principles.md) | Measurable proposal criteria, accessibility and research/verification plan |
| [Existing product plan](../../../implementation/PRODUCT-SPEC.md) | Bounded supplement following this comparison; no P2 extension or implementation in this assignment |
| [Document check](verification.json) | Matrix coverage, source/link consistency, unchanged product tree/rubric, exact executed checks |

Source registers are kept with the dossier responsible for the claim and include title, organization, date/version, access date, direct URL and limitations. The central matrix uses the same WI/LI IDs and URLs. Existing user/adoption sources EA/WM/LM/DT remain in [the original register](../sources.md). The same vendor text is not counted again as an independent user voice.

<a id="synthese-keuzes-en-verworpen-richtingen"></a>

## Synthesis, choices and rejected directions

[ANALYSE] Familiar concepts are more broadly transferable than exact screen positions or key combinations. In particular, app versus window, search scope, status area, workspaces, package sources and account/disk consequences differ. This raises questions to test, not a nieuw-os success claim.

[VOORSTEL] Preserve ordinary verbs and visible consequences; improve discoverability with local synonyms and place disk mutations behind one complete authorization boundary. Optionally ask about prior experience for temporary help, not for four shells. Introduce Materiaal gradually within the same basic controls.

Rejected for this direction: visually copying an existing OS, four different interfaces, prior platform as authorization, a search query that directly executes system actions, treating unknown disks as empty, dummy encryption/authentication, an AI/provider account as a required entry gate, and fewer steps as the sole quality measure. Rationale and acceptance are recorded in the linked specifications. These proposals change no implemented security guarantees.

<a id="beperkingen-en-volgende-bewijsstap"></a>

## Limitations and next evidence step

No complete release-bound installation capture, user research, keyboard/screen-reader/touch trial or physical hardware validation. Some macOS panel positions, defaults and exact KDE step counts have not been established. The KDE applet/display entry points were additionally read at tag v6.8.0, but the full backend/shell route was not executed. The native reference remains exactly `1f0dd9c`, separate from this branch's main baseline.

Preferred next step: first recheck current code/service gates, then a bounded read-only plan/alias experiment and safe task observation. Before actual writes: capability/auth/encryption/recovery evidence with disposable images and independent review. A human audit and representative adoption/retention measurement remain separately necessary. The fixed product score remains **34/100** on this main baseline; additional documents earn no acceptance points.
