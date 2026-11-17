<a id="gebruikersbehoeften--vergelijkende-synthese-van-fase-2"></a>
# User needs — comparative synthesis of phase 2

Date: 2 November 2026. Status: source research and analytical assessment; no representative complaints survey, product decision or implementation. [Method](README.md), [source register](sources.md), [repository baseline](repository-baseline.md).

<a id="wat-de-bronnen-gezamenlijk-ondersteunen"></a>
## What the sources jointly support

[ANALYSE] The most defensible research direction is less uncertainty during everyday tasks: knowing where something is, being able to find it, understanding access and modification, preserving work and recovering from errors. Multiple sources provide specific scenarios for this. No source establishes that one new OS already delivers this combination better, or that it will lead to mass migration.

[BRON] Established platforms are developing these areas themselves: Microsoft acknowledges feedback on search and interruptions; Apple offers Spotlight actions and tiling; GNOME/KDE improve file state, accessibility and task controls; Android adds notification grouping and adaptive windows, among other features. A missing or announced feature must be assessed per version, device and region. [WM-03] [WM-19] [WM-20] [LM-01] [LM-03] [LM-11] [LM-12]

The complete aspect tables are in [Windows/macOS](windows-macos-needs.md) and [Linux/Android](linux-mobile-needs.md). Source IDs in this synthesis link to their original source; methods and limitations remain in the dossiers.

<a id="herhaling-impact-en-relevantie"></a>
## Recurrence, impact and relevance

**Reading rule:** recurrence counts independent source families within this targeted selection, not incidents or users. “Strong” can refer to a technical boundary; the frequency of the need then remains unknown. All assessments of impact, feasibility and differentiation below are **[ANALYSE]**, not measured preferences or definitive phase-6 priorities.

| Problem / possible need | Recurrence and strength of research | Impact when it occurs | European relevance | Feasibility in nieuw-os | Possible differentiation |
|---|---|---|---|---|---|
| Reliable retrieval with visible scope | Microsoft feedback, Apple support/user report, independent GNOME and KDE reports. Moderate for friction; prevalence unknown. [WM-03] [WM-09] [WM-10] [LM-04] [LM-05] | High when work appears unfindable | Language, data location and offline sources | Scope/status bounded; a complete index of content that respects access rights is a major service | Moderate: primarily coherence and reliability |
| Understanding file location, synchronisation and backup | Microsoft/Apple semantics plus GNOME/Google recovery requirements. Strong for specific technical boundaries; only one individual signal of confusion, so weak/open for user understanding. [WM-05] [WM-06] [WM-13] [WM-14] [LM-13] [LM-20] | High: loss, unintended deletion or mistaken recovery expectations | Portability, local/NAS/provider choice | Explanation and status small; full sync, recovery and export large | High if the whole chain demonstrably works |
| Understanding and revoking permissions without losing the task | Two original studies by different teams, plus portals/pickers. Moderate; populations and measurement years limited. [LM-08] [LM-14] [LM-15] [LM-16] | High for sensitive tasks | Understandable control over data; no compliance claim | Existing capability foundation, but inventory/issuer/revocation chain requires native integration | Moderate/high by connecting task, duration and consequence |
| Preserving working keyboard/assistive routes | NV Access, AppleVis, GNOME and W3C are different maintenance/guideline families. Strong for specific regressions/requirements, not a frequency count. [WM-08] [WM-11] [DT14] [DT20] | Blocking for affected users | Inclusion, multilingualism and different input methods | App semantics exist; an OS-wide screen reader, shaping and input are substantial work | Quality prerequisite; not an exclusively new idea |
| Retaining essential apps and devices | Adobe, Microsoft, LibreOffice, Valve and a migration case. Strong for the stated compatibility boundaries. [WM-16] [WM-17] [EA-09] [EA-10] [EA-13] | Blocking for an indispensable task | Work, education, games, accessibility and service life | Very large; an independent kernel does not inherit the Linux/Windows ecosystem | Prerequisite before differentiation becomes relevant |
| Being able to predict updates and interruptions | Windows feedback, Snap scheduling, Pixel support policy and two different notification studies. Moderate; different outcome measures. [WM-03] [LM-07] [LM-17] [DT17] [DT21] | Moderate/high, depending on the task | Control and long-term maintenance | UI policy limited; recoverable updates require the existing slot/verification chain | Low for a switch; higher for coherence |
| Reliably resuming workspace and task context | One PowerToys regression plus existing workspace/snap features. Weak as a broad wish, a specific case. [WM-07] [DT07] [DT08] | Moderate/high for multitaskers | Different occupations/input methods; not specifically European | A two-app foundation exists; dynamic windows/session restoration require extension | Moderate if document context remains central |
| Understanding installation provenance and consequences | Apple notarisation, Ubuntu package choice, Flatpak and Android verification. Strong for mechanisms; weak for the general frequency of confusion. [WM-15] [LM-06] [LM-08] [LM-19] | High for unsafe or non-working installation | Vendor choice, inspectability and maintenance | Package/update cores present; complete installation/start chain still open | Quality prerequisite with open distribution |
| Optional contextual assistance with correctable actions | HCI guideline validation and existing AI product direction; no nieuw-os or EU market validation. Weak/moderate. [DT11] [DT16] | Task-dependent; errors can have high impact | Selected context and local processing fit the project contract | Actual local inference and a bounded action broker remain major dependencies | Unproven; compare with ordinary controls first |

No row above receives an objective “frequency percentage”. A generated ordering would mistake the targeted source selection for a representative measurement. Phase 6 still requires a chosen target group, actual task observations, cost estimates, maintenance/privacy/accessibility effects and dependencies for each proposal.

<a id="dekking-van-de-gevraagde-onderwerpen"></a>
## Coverage of the requested subjects

| Subject | Verified coverage | What remains unknown |
|---|---|---|
| Missing features / cumbersome routes / number of steps | Search scope, index recovery, Start dependencies, installation exceptions and routes for stopping sync [WM-03] [WM-04] [WM-09] [WM-15] [LM-04] | No uniform live task measurement or comparable step count across four OSes |
| Control, privacy, account, advertising and telemetry | Windows installation/diagnostics, Android advertising and backup flows, original studies of privacy understanding [WM-01] [WM-02] [LM-13] [LM-15] [LM-16] [LM-18] | No general resistance score; no uniform Android OEM or Linux telemetry practice established |
| Updates | Windows improvement plan, Snap hold/scheduling, Pixel support duration, native nieuw-os update building blocks [WM-03] [LM-07] [LM-17] [RB-01] | Plans are not broad rollout; no research into everyday update effects for nieuw-os |
| Search, settings and file management | Windows/macOS and GNOME/KDE dossiers, separate current solutions | No uniform representative satisfaction comparison |
| Multitasking and windows | Workspace regression, GNOME workspaces, Windows Snap, macOS tiling, Android windows [WM-07] [WM-20] [DT07] [DT08] [LM-11] | No evidence that one model is best for beginners and advanced users |
| Notifications | Grouping/platform features plus batching and disabling studies [LM-02] [LM-12] [DT17] [DT21] | Transfer to desktop use, urgent exceptions and optimal frequency not established |
| Backups and synchronisation | OneDrive, iCloud, Time Machine, Pika and Android recovery requirements [WM-05] [WM-13] [WM-14] [LM-13] [LM-20] | No complete recovery trial conducted by this research; sync is not an independent backup |
| Device linking | KDE Connect, Apple Mirroring region/account, EU portability routes [LM-10] [WM-12] [EA-11] | New routes/APIs do not establish general current availability or nieuw-os integration |
| Accessibility | Regressions, semantics, input alternatives, WCAG/WCAG2ICT and font/scale boundaries [WM-08] [WM-11] [LM-03] [DT10] [DT20] [RB-01] | Human and system-wide assistive acceptance remain open |
| Installation and package management | Multiple existing distribution models and permission pickers [WM-15] [LM-06] [LM-08] [LM-19] | Insufficient independent research to rank all models by usability |
| Ordinary versus advanced users | Research segments, update/search diagnostics, shortcuts and workspace restoration [WM-04] [WM-07] [LM-04] [LM-07] | No interviewed personas or preferences assumed from demographics |
| Adopting software from other platforms | Existing reference patterns below; original individual wishes where present | No representative list of “users want feature X from OS Y” |

<a id="bestaande-patronen-die-als-referentie-kunnen-dienen"></a>
## Existing patterns that can serve as references

These are **[HYPOTHESE] design/test scenarios**, not established wishes to adopt features or an instruction to copy an OS.

| Reference pattern | Source and existing feature | User outcome to investigate |
|---|---|---|
| Discoverable actions alongside search results | macOS Spotlight actions [WM-19] | Fewer detours, with visible context and bounded consequences |
| Understandable, selective data access | Flatpak portals and Android document picker [LM-08] [LM-14] | Using one selected object without general file permissions |
| Visible recovery history | Time Machine and Materiaal as different technical models [WM-14] [RB-01] | Knowing what is recoverable and undoing an error without unintended overwriting |
| Grouping and resuming windows | Windows Snap/GNOME workspaces/macOS tiling [DT07] [DT08] [WM-20] | Comparing two sources and finding the correct task again; keyboard operation included |
| Explicitly linked devices | KDE Connect/Handoff [LM-10] [DT04] | Continuing work on a second device without silently broadening data access |
| Overview of actual access | Android privacy dashboard [DT06] | Understanding which app used access and exactly what revocation stops |

<a id="ontwerpvoorwaarden-voor-de-volgende-fasen"></a>
## Design prerequisites for the next phases

[ANALYSE] European values become concrete through understandable data destinations, provider switching/export, local core operation, recovery, accessible language/input and maintenance. European origin alone establishes none of those properties. Cost, security and usability must be tested on the actual product. For that purpose, the Eurobarometer measures attitudes towards services, not the market size for nieuw-os. [EA-01–EA-04]

[HYPOTHESE] One chain is promising for the later vision: **find → understand where and with which permissions → act → save → recover/resume**. Test it without AI as well. Preserve names, export and familiar basic controls when they ease migration. For each concept, record which part is a source finding, design choice, technical experiment or later ambition.

Not yet justified by this research: using decorative glass effects as a value claim, a mandatory assistant, promising general app/driver compatibility, introducing a new account/cloud requirement, or presenting small existing experimental boundaries as full product capabilities. These are rejections of insufficiently substantiated claims, not a reduction of the existing product ambition.

<a id="verwijzingen"></a>
## References

The WM/LM/DT/EA IDs refer to the registers in this directory. The following directly linked selections make the synthesis independently navigable; limitations remain in the relevant dossier.

[WM-01]: https://www.microsoft.com/en-us/windows/windows-11-specifications
[WM-02]: https://support.microsoft.com/en-us/windows/privacy/diagnostics-feedback-and-privacy-in-windows
[WM-03]: https://blogs.windows.com/windows-insider/2026/03/20/our-commitment-to-windows-quality/
[WM-04]: https://blogs.windows.com/windows-insider/2026/05/15/improving-windows-quality-making-taskbar-and-start-more-personal/
[WM-05]: https://support.microsoft.com/en-gb/onedrive/back-up-your-folders-with-onedrive?nochrome=true
[WM-06]: https://learn.microsoft.com/en-us/answers/questions/5557469/why-did-onedrive-remove-my-desktop-icons
[WM-07]: https://github.com/microsoft/PowerToys/issues/37634
[WM-08]: https://api.nvaccess.org/releases/2026.2/documentation/changes.html
[WM-09]: https://support.apple.com/en-us/102321
[WM-10]: https://discussions.apple.com/thread/256145579
[WM-11]: https://applevis.com/blog/macos-tahoe-new-features-changes-improvements-bugs-blind-deafblind-low-vision-users
[WM-12]: https://support.apple.com/en-us/120421
[WM-13]: https://support.apple.com/en-us/109344
[WM-14]: https://support.apple.com/en-us/104984
[WM-15]: https://support.apple.com/en-us/102445
[WM-16]: https://helpx.adobe.com/download-install/apps/system-requirements/creative-cloud-requirements.html
[WM-17]: https://support.microsoft.com/en-us/office/system-requirements/system-requirements-for-microsoft-365-for-business-education-and-government-use
[WM-19]: https://www.apple.com/os/pdf/All_New_Features_macOS_Tahoe_Sept_2025.pdf
[WM-20]: https://www.apple.com/newsroom/2024/06/macos-sequoia-takes-productivity-and-intelligence-on-mac-to-new-heights/
[LM-01]: https://release.gnome.org/51/
[LM-02]: https://release.gnome.org/48/
[LM-03]: https://kde.org/announcements/plasma/6/6.8.0/
[LM-04]: https://discourse.gnome.org/t/search-everywhere-cant-find-files/33037
[LM-05]: https://discuss.kde.org/t/cifs-share-freezes-taskbar-when-nas-offline/42466
[LM-06]: https://ubuntu.com/desktop/docs/en/24.04/how-to/software/install-an-application/
[LM-07]: https://snapcraft.io/docs/how-to-guides/manage-snaps/manage-updates/
[LM-08]: https://docs.flatpak.org/en/latest/sandbox-permissions.html
[LM-10]: https://kdeconnect.kde.org/
[LM-11]: https://blog.google/products-and-platforms/platforms/android/android-17-features/
[LM-12]: https://blog.google/products-and-platforms/platforms/android/android-16/
[LM-13]: https://support.google.com/android/answer/2819582?hl=en
[LM-14]: https://developer.android.com/training/data-storage/shared/documents-files
[LM-15]: https://www.usenix.org/system/files/soups2024-prange.pdf
[LM-16]: https://www.usenix.org/conference/soups2025/presentation/keswani
[LM-17]: https://support.google.com/pixelphone/answer/4457705?hl=en
[LM-18]: https://support.google.com/android/answer/13720755?hl=en
[LM-19]: https://developer.android.com/developer-verification
[LM-20]: https://apps.gnome.org/PikaBackup/
[DT04]: https://www.apple.com/newsroom/2014/10/16OS-X-Yosemite-Available-Today-as-a-Free-Upgrade/
[DT06]: https://blog.google/products-and-platforms/platforms/android/android-12/
[DT07]: https://release.gnome.org/40/
[DT08]: https://blogs.windows.com/windowsexperience/2021/06/24/introducing-windows-11/
[DT10]: https://www.w3.org/TR/2024/REC-WCAG22-20241212/
[DT11]: https://www.apple.com/newsroom/2024/06/introducing-apple-intelligence-for-iphone-ipad-and-mac/
[DT14]: https://release.gnome.org/50/
[DT16]: https://www.microsoft.com/en-us/research/wp-content/uploads/2019/01/Guidelines-for-Human-AI-Interaction-camera-ready.pdf
[DT17]: https://doi.org/10.1016/j.chb.2019.07.016
[DT20]: https://www.w3.org/TR/2025/NOTE-wcag2ict-22-20251211/
[DT21]: https://pure.uva.nl/ws/files/245971989/Beyond_the_Buzz.pdf
[EA-09]: https://help.libreoffice.org/latest/en-US/text/shared/guide/ms_user.html
[EA-10]: https://partner.steamgames.com/doc/steamhardware/proton
[EA-11]: https://digital-markets-act.ec.europa.eu/factsheet-how-dma-making-smartphones-better-interoperability-and-data-portability-case-studies-2026-05-11_en
[EA-13]: https://www.schleswig-holstein.de/DE/landesregierung/ministerien-behoerden/I/Presse/PI/2025/cds/251204_cds_open-source
[EA-01–EA-04]: european-adoption-barriers.md#oorspronkelijke-eurobarometer-gecontroleerd
[RB-01]: repository-baseline.md
