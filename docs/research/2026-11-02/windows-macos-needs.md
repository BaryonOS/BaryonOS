<a id="windows-en-macos-gebruikersbehoeften-en-overstaprisicos"></a>
# Windows and macOS: user needs and migration risks

Research date: **2 November 2026**. Phase 2; documentary research, no product change. Repository baseline: `main` / `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`.

**Conclusion [ANALYSE]: the strongest starting points are reliable retrieval, understandable file locations and preservation of working interactions; these sources do not establish general willingness to migrate.**

<a id="onderzoeksvragen-en-methode"></a>
## Research questions and method

Which everyday actions fail or require recovery work? What control actually exists, which improvements have been announced, and which dependencies make migration difficult?

Targeted searches combined Windows/macOS with `search`, `settings`, `OneDrive Desktop missing`, `Spotlight dictionary`, `window restoration`, `VoiceOver`, `NVDA`, `updates`, `privacy`, `iPhone Mirroring`, `Office` and `Creative Cloud requirements`. Priority was given to 2024–2026, original reports, maintained documentation and original studies. The selection comprises twenty sources that were read: vendor documentation, two Windows user reports, one macOS user report, AppleVis's own tests, NV Access maintenance notes and an academic experiment. Search-result snippets do not count as evidence that has been read.

Labels:

- **[BRON]**: a bounded statement from documentation or research; vendor statements remain vendor statements.
- **[SIGNAAL]**: a specific experience/report, without representative frequency data.
- **[ANALYSE]**: interpretation or design implication of this research.
- **[HYPOTHESE]**: a user need or effect that remains to be tested.

This is a targeted qualitative inventory, not a systematic review, survey or interview study. Multiple Microsoft pages are not independent source families. No complaint count, vote count or vendor wording has been converted into a population percentage. A problem in a particular version is not automatically present on the access date. Announced Insider functionality is not evidence of general availability.

## Windows

| Aspect | Verified finding | Implication for nieuw-os |
|---|---|---|
| Search and files | **[BRON]** Microsoft's quality programme describes more reliable searches for apps, files and settings, a clearer distinction between local and web results, and a faster, more stable File Explorer. This is a vendor interpretation of feedback received and an improvement plan. [WM-03] | **[ANALYSE]** Make search scope and result type visible; measure whether the correct local document is retrieved. A universal search field alone does not solve ranking and reliability. |
| Settings and personalisation | **[BRON]** Microsoft describes how customising Start requires multiple settings and how disabling recommended content can also affect recent files and Jump Lists. Independent switches and taskbar positions are presented as improvements, partly in Experimental. [WM-04] | **[ANALYSE]** One understandable choice must not have a hidden side effect elsewhere. Show the effect and recovery route alongside the setting. |
| Account, telemetry and advertising | **[BRON]** Windows 11 Home and Pro for personal use require internet access and a Microsoft account during initial setup. [WM-01] Windows distinguishes required and optional diagnostics; disabling the optional stream allows normal use. Personalisation/offer choices are separate; deleting diagnostics does not stop future transmission. Organisational policy can restrict choices. [WM-02] | **[ANALYSE]** Account use, diagnostics and offers are different permissions. **[HYPOTHESE]** Local onboarding and separate, persistent choices may support trust. The extent of resistance has not been measured here. |
| Updates and installation | **[BRON]** Microsoft announces more control over deferral, fewer interruptions and shutdown/restart without immediate installation. Hardware requirements include TPM 2.0 and Secure Boot capability. [WM-03] [WM-01] | **[ANALYSE]** Show timing, required restarts and consequences in advance. Hardware suitability should be known before migration. This dossier contains insufficient independent comparison of Store, installer and package-management experiences. |
| Multitasking | **[SIGNAAL]** A PowerToys user reports that saved Workspaces start minimised after version 0.88.0 instead of in their saved positions. The issue is open; it concerns a utility, not established general Windows behaviour. [WM-07] | **[HYPOTHESE]** Advanced users benefit more from reliable workspace restoration than from additional window effects. Test reconnecting displays and restoration after restart separately. |
| Notifications and AI | **[BRON]** Microsoft aims for quieter Widgets, fewer update interruptions and fewer unnecessary Copilot entry points. [WM-03] | **[ANALYSE]** Give system notifications an actionable purpose and respect disabling them. **[HYPOTHESE]** Assistance with actions on request may be more useful than persistent assistant promotion; there is no evidence that all users reject AI. |
| Backup, synchronisation and devices | **[SIGNAAL]** A user reports missing desktop icons after disabling OneDrive folder backup. [WM-06] **[BRON]** Current Windows help now describes a choice between keeping files only locally or only in OneDrive; cloud files may need to be downloaded first. [WM-05] | **[ANALYSE]** The signal concerns an incomprehensible change of location, not established deletion. Show local presence, synchronisation and recovery copies separately. The old complaint does not prove that the current stop dialogue causes the same fault. Phone linking has been insufficiently researched here. |
| Accessibility | **[BRON]** NV Access documents a fix in NVDA 2026.1.1 for a regression in which `Ctrl+F` in File Explorer no longer correctly announced only the search field. [WM-08] | **[ANALYSE]** Focus, name and announcement together constitute the task. Test actual screen-reader interaction during updates; an existing accessibility feature does not establish a regression-free workflow. |

## macOS

| Aspect | Verified finding | Implication for nieuw-os |
|---|---|---|
| Search and settings | **[BRON]** Apple documents a local Spotlight index, progress and potentially hours/days of indexing after changes. Recovery may require excluding and re-adding a folder or disk. [WM-09] **[SIGNAAL]** A macOS 26.0 user reports dictionary results that are difficult to reach and a filter yielding no results. [WM-10] | **[ANALYSE]** Indexing status and recovery must be understandable. **[HYPOTHESE]** Predictable categories support frequent searches; the dictionary report is not evidence of general search failure or current fault status. |
| Files and synchronisation | **[BRON]** With iCloud Desktop/Documents, deletion on one device also applies elsewhere. Disabling it leaves files in iCloud and creates new local folders; having another cloud service manage the same folders simultaneously is unsupported. [WM-13] | **[ANALYSE]** Distinguish 'remove local download', 'delete everywhere' and 'stop synchronisation'. These semantics warrant confirmation showing the destination, rather than just an on/off switch. Do not infer macOS prevalence from the Windows report. |
| Backup | **[BRON]** Time Machine supports automatic backups to external storage, optional encryption, a visible last backup and file recovery; when storage is full, the oldest backups are removed. [WM-14] | **[ANALYSE]** A useful reference pattern: show the latest recoverable version and storage limit. Synchronisation and versioned backup must be recognisably different. |
| Control, privacy and account | **[BRON]** Apple states that the Spotlight index remains on the device. [WM-09] iPhone Mirroring requires the same Apple account with two-factor authentication. [WM-12] | **[ANALYSE]** One index being local does not establish that all search/AI services are local. This source set does not quantify macOS resistance to advertising, objections to telemetry or mandatory accounts during initial setup; no general conclusion follows on those subjects. |
| Updates and app installation | **[BRON]** macOS also permits apps from outside the App Store, with signature/notarisation checks; an exception for an unverified app requires explicit steps in Privacy & Security. [WM-15] **[SIGNAAL]** AppleVis describes specific VoiceOver regressions alongside improvements at Tahoe's introduction. [WM-11] | **[ANALYSE]** Show provenance and the reason for blocking. Reduce explanation problems without removing security checks. An update must also preserve the existing assistive workflow. |
| Multitasking | **[BRON]** Apple announced window tiling through screen edges, corners, menus and the keyboard in Sequoia (2024). [WM-20] | **[ANALYSE]** 'macOS has no snapping' is not a defensible starting point. Assess restoration, discoverability and keyboard parity; this dossier provides no representative evidence that users prefer one tiling model. |
| Notifications and actions | **[BRON]** Tahoe describes Spotlight actions, filters and quick keys, Live Activities and categories for notifications from unknown senders. [WM-19] | **[ANALYSE]** Actions close to search are a useful design pattern, not an established gain in satisfaction. Independent user research on daily notification load and Focus complexity is absent here. |
| Device linking and accessibility | **[BRON]** Apple's current support information still lists iPhone Mirroring as unavailable in the EU. [WM-12] **[SIGNAAL]** AppleVis tested usable VoiceOver screen sharing between Macs, among other features, but reported an inaccessible Mirroring interface in the initial Tahoe release. [WM-11] | **[ANALYSE]** Region, device requirements and accessibility are part of the feature. The accessibility report is a historical release observation; the fix status in November 2026 has not been established. |

<a id="terugkerende-patronen-en-doelgroepen"></a>
## Recurring patterns and target groups

The ordering below concerns **strength of evidence and estimated task impact**, not measured frequency among users.

| Pattern | Evidence and limitation | Priority recommendation [ANALYSE] |
|---|---|---|
| Retrieval must remain predictable | Windows quality programme plus Apple support and an original Spotlight report: different sources/platforms, no population measurement. [WM-03] [WM-09] [WM-10] | High: design local search, result type, location and indexing status together. |
| 'My file' has multiple storage states | OneDrive report and two vendor descriptions show location/deletion semantics; macOS confusion is a hypothesis here. [WM-05] [WM-06] [WM-13] | High: explicit destination and offline availability, safe stop/recovery route. |
| New versions can disrupt existing actions | PowerToys report, NVDA fix note and AppleVis tests are separate maintenance/user contexts. [WM-07] [WM-08] [WM-11] | High: test task regressions, including focus and announcements; no quality claim based solely on appearance. |
| More choice can acquire hidden dependencies | Start personalisation is specifically documented; a broader effect on settings is interpretation. [WM-04] | Medium/high: changes with visible consequences, few coupled toggles. |
| Compatibility can outweigh interface improvement | Official Office and Adobe platform requirements; no measured likelihood of migration. [WM-16] [WM-17] | Prerequisite for migration: test users' own work documents and application chain. |

**[HYPOTHESE] Target-group segments, not interviewed personas:** non-technical users may benefit particularly from recoverable choices and recognisable locations; advanced users from stable shortcuts, filters, automation and saved workspaces. The reports provide scenarios, not evidence that all members of those groups share the same preference. Screen-reader users are not an optional later target group: their tasks belong in the same acceptance criteria.

**[BRON]** The experiment by Łazaruk et al. compares visual search in OS images and subjective ratings among 68 computer-science students aged 19–23, mostly men. Preference and measured performance do not necessarily coincide. **[ANALYSE]** This supports caution with 'feels modern, therefore works better'; it does not establish a general migration barrier or a superior OS. Complete daily workflows on three live systems were not tested. [WM-18]

<a id="professionele-workflows-en-aanbeveling"></a>
## Professional workflows and recommendation

**[BRON]** The verified Microsoft 365 desktop requirements list Windows and macOS. Adobe also lists supported Windows/macOS versions for Creative Cloud, with different requirements per application; nieuw-os is not listed as a supported desktop platform. [WM-16] [WM-17] **[ANALYSE]** This establishes missing vendor support in these matrices, not the impossibility of compatibility in principle. Browser use, file import and a compatibility layer must not be assumed to provide a complete replacement.

**[HYPOTHESE]** A migration trial must test document editing and round-trip exchange: Office formatting, macros, collaboration and plugins; for creative workflows, project files, fonts, colour management and peripherals, among other aspects. These are proposed test scenarios, not measured defects of nieuw-os.

**Preferred direction [ANALYSE]:** prioritise one coherent chain: 'find → understand location/access → open → recover safely'. Include keyboard and screen-reader tests in that chain. Preserve familiar basic actions and add targeted actions. The repository analysis and next prioritisation phase must determine which parts can become real services and which can only be prototype interaction; this document provides no implementation or compatibility evidence.

Still needed: task tests with different experience levels and disabilities; independent data on notifications, package management and phone integration; verification on current OS builds; professional migration trials. European relevance is concrete in regional feature restrictions and autonomy requirements, but this dossier does not measure European willingness to pay or migrate. No interviews, user tests or new benchmarks were conducted.

<a id="bronregister"></a>
## Source register

**All sources accessed on 2026-11-02.** 'n.d.' means that no reliable publication/modification date was visible, not that the content is timeless. URL and title are directly linked. Dates for dynamic support pages are the displayed publication dates; the underlying feature release may be earlier.

| ID | Source, organisation and date | Type; supports | Limitation |
|---|---|---|---|
| WM-01 | [Windows 11 specifications](https://www.microsoft.com/en-us/windows/windows-11-specifications), Microsoft, n.d. | Official requirements; account/internet during Home/Pro personal setup and hardware requirements. | No measurement of resistance; not all editions/forms of use. |
| WM-02 | [Diagnostics, feedback, and privacy in Windows](https://support.microsoft.com/en-us/windows/privacy/diagnostics-feedback-and-privacy-in-windows), Microsoft, n.d. | Technical explanation; diagnostic categories, offers, deletion and policy. | Vendor description, not an independent privacy audit. |
| WM-03 | [Our commitment to Windows quality](https://blogs.windows.com/windows-insider/2026/03/20/our-commitment-to-windows-quality/), Pavan Davuluri/Windows Insider Team, Microsoft, 2026-03-20. | Roadmap/feedback summary; search, Explorer, AI, updates, quiet. | Vendor selection; promises/previews, no measured reach. |
| WM-04 | [Making taskbar and Start more personal](https://blogs.windows.com/windows-insider/2026/05/15/improving-windows-quality-making-taskbar-and-start-more-personal/), Diego Baca/Microsoft, 2026-05-15. | Product analysis/Insider announcement; coupled Start choices. | No evidence that everything has been generally rolled out. |
| WM-05 | [Back up your folders with OneDrive](https://support.microsoft.com/en-gb/onedrive/back-up-your-folders-with-onedrive?nochrome=true), Microsoft, n.d. | Guide; current Windows choice when stopping, download requirement. | Describes intended behaviour, not a user test. |
| WM-06 | [Why did OneDrive remove my desktop icons?](https://learn.microsoft.com/en-us/answers/questions/5557469/why-did-onedrive-remove-my-desktop-icons), Keagan Nelson/Microsoft Q&A, 2025-09-16. | Original report; experienced disappearance after stopping folder sync. | Individual, not reproduced; answer does not establish data loss. |
| WM-07 | [PowerToys issue #37634](https://github.com/microsoft/PowerToys/issues/37634), overbarg/Microsoft project, 2025-02-25; issue updated 2026-10-27. | Original bug report; minimised Workspaces after 0.88.0. | Open status is not evidence of the fault in every current version. |
| WM-08 | [What's New in NVDA](https://api.nvaccess.org/releases/2026.2/documentation/changes.html), NV Access, version archive 2026.2, section 2026.1.1; day not visible. | Maintenance notes; fixed File Explorer search regression. | Windows plus assistive-tool combination; no broad OS score. |
| WM-09 | [About Spotlight indexing and search results](https://support.apple.com/en-us/102321), Apple, 2026-08-25. | Support; local index, duration/status and recovery. | No fault frequency; a local index says nothing about all connected services. |
| WM-10 | [Dictionary search with Spotlight broken](https://discussions.apple.com/thread/256145579), SteinHaugan/Apple Support Community, 2025-09-23. | Original experience on macOS 26.0. | One report, no current reproduction or prevalence data. |
| WM-11 | [macOS Tahoe: features, improvements and bugs for blind, DeafBlind and low vision users](https://applevis.com/blog/macos-tahoe-new-features-changes-improvements-bugs-blind-deafblind-low-vision-users), AppleVis Editorial Team, 2025-09-15. | Community's own tests; VoiceOver improvements and regressions. | Release snapshot; limited braille coverage, low-vision additions not tested by the authors. |
| WM-12 | [iPhone Mirroring: Use your iPhone from your Mac](https://support.apple.com/en-us/120421), Apple, 2026-10-14. | Support; account/device requirements and EU availability. | Availability can change; no explanation of all causes. |
| WM-13 | [Add your Desktop and Documents files to iCloud Drive](https://support.apple.com/en-us/109344), Apple, 2026-09-14. | Guide; synchronisation, deletion, stopping and competing providers. | Operation, no measurement of confusion. |
| WM-14 | [Back up your Mac with Time Machine](https://support.apple.com/en-us/104984), Apple, 2026-10-16. | Guide; backup status, encryption, recovery and retention. | No recovery trial conducted by this research. |
| WM-15 | [Safely open apps on your Mac](https://support.apple.com/en-us/102445), Apple, 2026-05-27. | Security/installation guide; external apps and explicit exception. | No evidence that prompts are generally incomprehensible. |
| WM-16 | [Technical requirements for Creative Cloud apps](https://helpx.adobe.com/download-install/apps/system-requirements/creative-cloud-requirements.html), Adobe, updated 2026-10-13. | Support matrix; supported desktop platforms and app-specific differences. | No compatibility test or assessment of alternative workflows. |
| WM-17 | [System requirements for Microsoft 365 for business, education and government use](https://support.microsoft.com/en-us/office/system-requirements/system-requirements-for-microsoft-365-for-business-education-and-government-use), Microsoft, n.d. | Support matrix; Windows/macOS desktop clients. | No evidence of equivalent web or alternative software. |
| WM-18 | [The Force of Habit: Comparing Graphical User Interfaces of Popular Operating Systems Using Eye-Tracking Analysis](https://www.researchgate.net/publication/395804148_The_Force_of_Habit_Comparing_Graphical_User_Interfaces_of_Popular_Operating_Systems_Using_Eye-Tracking_Analysis), Łazaruk et al., Lublin University of Technology/IEEE Access, published 2025-09-24; DOI 10.1109/ACCESS.2025.3614018. | Original article, authors' copy; visual search experiment and subjective assessment. | Narrow student sample, image stimuli; no live migration/workflow test. |
| WM-19 | [New features available with macOS Tahoe](https://www.apple.com/os/pdf/All_New_Features_macOS_Tahoe_Sept_2025.pdf), Apple, September 2025. | Official feature overview; Spotlight actions, filters, notifications and Live Activities. | Marketing/functionality, not user research; regional requirements. |
| WM-20 | [macOS Sequoia takes productivity and intelligence on Mac to new heights](https://www.apple.com/newsroom/2024/06/macos-sequoia-takes-productivity-and-intelligence-on-mac-to-new-heights/), Apple, 2024-06-10. | Historical primary announcement; tiling through edges, corners and the keyboard. | Announcement from 2024, no current comparative task test. |

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
[WM-18]: https://www.researchgate.net/publication/395804148_The_Force_of_Habit_Comparing_Graphical_User_Interfaces_of_Popular_Operating_Systems_Using_Eye-Tracking_Analysis
[WM-19]: https://www.apple.com/os/pdf/All_New_Features_macOS_Tahoe_Sept_2025.pdf
[WM-20]: https://www.apple.com/newsroom/2024/06/macos-sequoia-takes-productivity-and-intelligence-on-mac-to-new-heights/
