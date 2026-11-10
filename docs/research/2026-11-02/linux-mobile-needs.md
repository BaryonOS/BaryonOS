# Linux desktops and Android: needs and migration risks

**Conclusion:** comprehensible data permissions, reliable discoverability, and demonstrable recovery provide better-supported opportunities than a new visual style; existing compatibility remains a strict migration prerequisite.

Status: external desk research, 2 November 2026; no Baryon OS product implementation or user validation. Scope: GNOME, KDE Plasma, Ubuntu, and Android; iOS only where comparative research supports it. ChromeOS was not investigated separately. This document does not replace repository analysis or technical acceptance.

## Method and evidence boundaries

Research questions: where do people lose awareness or control; which existing solutions work under which conditions; which problems transfer to Baryon OS? The search strategy used current release pages, original product documentation, original GNOME/KDE user reports, and SOUPS research. Search terms included `GNOME search files`, `KDE offline NAS search`, `Snap manage updates`, `Proton anti-cheat`, `Android permissions user study`, `privacy labels comprehension`, `Android backup`, and `developer verification`. The content of opened pages was used, not only search results or automatically generated summaries.

- **[SOURCE]** verifiable specification, product announcement, or research result; a vendor claim is not an independent performance test.
- **[SIGNAL]** original user experience; no established prevalence or demonstrated cause.
- **[ANALYSIS]** research synthesis, including applicability boundaries.
- **[HYPOTHESIS]** design choice that still requires user research and technical validation.

Twenty sources were selected. Multiple GNOME, KDE, or Google pages do not count as independent user populations. The two academic studies have different research teams; LM-15 includes a Google co-author. LM-15 was published in 2024 but collected data in April–May 2022: this is historical behavioural information, not a measurement of Android 17. [LM-15] [LM-16] No representative Dutch or European sample, migration percentage, interview, or frequency ranking was constructed.

## Linux: aspects and consequences

| Aspect | Verifiable finding and boundary | Implication for Baryon OS |
|---|---|---|
| Search, files, settings | **[SIGNAL]** A GNOME 46/Ubuntu 24.04 user reports differences between folder, shell, and indexed search results, with unclear symlink configuration. A KDE user reports that taskbar search freezes when a NAS is offline. Causes and prevalence have not been established. [LM-04] [LM-05] | **[HYPOTHESIS]** Show searched locations, offline sources, and permissions; one unreachable source must not block local results. Provide recovery without terminal diagnosis. |
| Everyday file management | **[SOURCE]** GNOME 51 improves selection preservation, readable status, and non-blocking file operations. [LM-01] | **[ANALYSIS]** Predictable selection and honest status are useful quality criteria. This does not demonstrate a current defect in every distribution. |
| Privacy and permission | **[SOURCE]** Flatpak restricts access by default, but manifests can grant broad permissions; portals support access through a deliberate file selection. GNOME Software clarifies file permissions. [LM-08] [LM-01] | **[HYPOTHESIS]** Request access for the specific task and show its effect and revocation. A dialogue without enforcement is insufficient. This corpus does not establish uniform Linux telemetry, advertising, or account practices. |
| Installation and updates | **[SOURCE]** Ubuntu App Center offers snap and deb packages, sometimes both for the same application. Snap documents automatic updates and deferral/scheduling, including indefinite deferral through `refresh --hold` from snapd 2.58. [LM-06] [LM-07] | **[ANALYSIS]** Package choice and update control already exist. **[HYPOTHESIS]** Explain provenance, support, and the planned change comprehensibly; do not conceal which implementation is installed. The assertion that Snap can never be paused is incorrect. |
| Multitasking and notifications | **[SOURCE]** Plasma 6.8 adds shortcuts for task ordering; GNOME 48 groups application notifications and introduces screen-time and break tools. [LM-03] [LM-02] | **[ANALYSIS]** Attention management and keyboard operation are existing competitive requirements. **[HYPOTHESIS]** Test task-oriented resumption and notifications per work context; a need for more window states has not been demonstrated. |
| Backup, synchronisation, pairing | **[SOURCE]** Pika Backup provides encrypted local/remote personal backups, not complete system recovery. KDE Connect provides file transfer, notification integration, and media control, among other functions. [LM-20] [LM-10] | **[ANALYSIS]** File transfer, synchronisation, and recovery are separate promises. **[HYPOTHESIS]** Show the last successful backup and recoverable categories; provide explicit device pairing. |
| Accessibility | **[SOURCE]** Plasma 6.8 integrates clicking by holding the pointer still, aimed at motor impairments. [LM-03] | **[ANALYSIS]** Inclusive input must be part of the shell. This addition does not demonstrate complete screen-reader, braille, or application coverage. That requires separate tests with users. |
| Drivers and legacy applications | **[SOURCE]** GNOME 51 removes legacy NVIDIA driver interfaces. Plasma 6.8 ends X11 sessions while continuing to support X11 applications through XWayland. [LM-01] [LM-03] | **[ANALYSIS]** Modernisation has explicit migration boundaries. **[HYPOTHESIS]** Establish a check for each GPU, display, and peripheral; avoid claiming that an entire device category works. |
| Games and anti-cheat | **[SOURCE]** Valve requires developer activation/configuration for EAC/BattlEye; Proton does not support kernel anti-cheat. Proton is a Linux compatibility layer. [LM-09] | **[ANALYSIS]** One blocked essential game can negate every other product benefit. Baryon OS does not inherit Proton support. **[HYPOTHESIS]** Check specific titles before migration; do not promise general Windows compatibility. |
| Ordinary and advanced users | **[ANALYSIS]** The documentation shows both graphical installation and advanced update commands; the search reports show diagnosis that requires technical knowledge. [LM-04] [LM-05] [LM-06] [LM-07] | **[HYPOTHESIS]** Provide one comprehensible default route with inspectable details and shortcuts. These two target groups are working segments, not personas measured here. |

## Android and limited iOS comparison

| Aspect | Verifiable finding and boundary | Implication for Baryon OS |
|---|---|---|
| Search, files, settings | **[SOURCE]** Android's Storage Access Framework provides a system picker across local and cloud providers, with access to selected documents without general storage permission. [LM-14] | **[ANALYSIS]** A uniform access task is feasible without universal read permission. This corpus contains insufficient user evidence to rank the quality of Android's global search function. |
| Comprehensible privacy | **[SOURCE]** LM-15 observed bundled permission revocations among 132 participants, particularly for rarely used applications/non-essential functions. LM-16 found misunderstandings of terms in iOS/Android privacy labels in two online studies with 46 and 383 participants. [LM-15] [LM-16] | **[HYPOTHESIS]** Describe consequences in plain language, keep core functionality working after refusal where possible, and provide a targeted permission review. More switches alone do not demonstrate better control. |
| Advertising, account, telemetry | **[SOURCE]** Google documents separate advertising topics, application suggestions, and advertisement measurement. Google Backup uses an account and also describes diagnostics collection. [LM-18] [LM-13] | **[ANALYSIS]** These services have different data flows; one switch does not cover all personalisation. This does not demonstrate identical behaviour across all Android variants, general account coercion, or the extent of user resistance. |
| Updates and installation | **[SOURCE]** Pixel 8 and later receive seven years of operating-system/security updates from first availability in the US Google Store. Android developer verification begins regionally in participating stores; international expansion is planned, with a separate route for unverified applications. [LM-17] [LM-19] | **[ANALYSIS]** The support period must be visible per device. Verification is not the same as a ban on all sideloading and, according to this source, had not already been introduced across the EU. **[HYPOTHESIS]** Combine comprehensible default security with an explicit advanced installation route. |
| Multitasking and notifications | **[SOURCE]** Android 16 groups notifications and supports live progress for suitable applications; desktop windows were announced for compatible devices. Android 17 describes floating application windows and a window bar on large screens, with a phased rollout. [LM-12] [LM-11] | **[ANALYSIS]** Mobile and desktop controls are converging, but product announcements do not demonstrate universal availability or lower task load. **[HYPOTHESIS]** Preserve task context when changing form factor; test operation with both touch and keyboard. |
| Backup, recovery, and migration | **[SOURCE]** Google warns that not all application data is restored and that a backup from a newer Android version cannot be restored to an older version. Only part of the data receives additional encryption through the screen lock; Photos/MMS are not included. [LM-13] | **[HYPOTHESIS]** Before migration, provide a category-by-category overview of what can be transferred, what is missing, and what requires reconfiguration. A green backup switch is not complete recovery evidence. |
| Devices and accessibility | **[SOURCE]** Android 16 provides improved control of LE Audio hearing aids, depending on the phone and manufacturer. KDE Connect provides an existing desktop/phone route. [LM-12] [LM-10] | **[ANALYSIS]** Accessibility includes protocols and devices, not only contrast. **[HYPOTHESIS]** Test supported combinations and show missing functions before pairing. |
| Games, peripherals, and expertise | **[ANALYSIS]** The corpus does not justify a general claim about Android game, controller, or driver compatibility on Baryon OS. The verification route does distinguish standard use from advanced installation. [LM-19] | **[HYPOTHESIS]** Record compatibility per application/device. A quickly comprehensible daily route and inspectable advanced controls must follow the same permission rules. |

## Recurring patterns and priority

**[ANALYSIS]** The ranking below is a research weighting, not a counted user frequency. European relevance here means auditable data choice, portability, and inclusion; no measured European willingness to pay or migrate was derived.

| Pattern | Basis and strength | Impact, differentiation, and feasibility |
|---|---|---|
| Understand permissions without losing the task | Two original studies from different teams; portals/document pickers show implementation patterns. Moderately strong, with old behavioural data. [LM-08] [LM-14] [LM-15] [LM-16] | Everyday relevance; differentiation lies in comprehensible consequences and actual enforcement. Small UI explanations are conceivable; complete platform permissions require native evidence. |
| See why something is not found | Independent GNOME and KDE reports; a product specification confirms attention to file status. Qualitatively recurring, frequency unknown. [LM-01] [LM-04] [LM-05] | High impact once an essential task fails. Search scope/offline status are bounded concepts; indexing and source isolation are additional system functions. |
| Recoverable data and reliable migration | Linux and Android documentation identify specific recovery boundaries. Strong evidence for boundaries, no measured prevalence of demand. [LM-13] [LM-20] | Potentially decisive for migration. An honest migration overview differentiates only when export and recovery actually work. |
| Fewer interruptions | GNOME and Google are developing grouping/attention functions. Independent product responses, no independent effect measurement. [LM-02] [LM-12] | Functionally relevant; low novelty. A task-oriented focus model remains a hypothesis to test. |
| Retain essential applications and devices | Valve and desktop documentation provide technical conditions. Strong for the named boundaries, population size unknown. [LM-01] [LM-03] [LM-09] | Very high migration impact for affected users; high cost. Defer compatibility promises until support is demonstrable. |

**[HYPOTHESIS] Preferred direction:** first design comprehensible task access and discoverability, including refusal, offline state, and recovery. Then test migration and focus flows. This does not authorise replacement of existing product decisions: combine it with repository research and the complete priority matrix.

**[ANALYSIS] Technical boundary:** these Linux, Flatpak, Android, and Proton facilities are not directly usable libraries for an independent kernel. Porting, drivers, platform APIs, storage, and security enforcement require separate investigation. A host prototype can investigate comprehensibility but does not demonstrate a native operating-system function. No progress points or native acceptance claims are attached to this research.

Missing evidence: representative migration reasons; systematic screen-reader tests; OEM-specific advertising/telemetry; a printer/audio/GPU matrix; Android search and synchronisation friction; comparable task measurements for non-technical users. Follow-up: genuine task observations and migration inventories with consent, separated by target group and accessibility need. Do not invent interviews or test results.

<a id="bronregister"></a>

## Source register

All sources were reviewed substantively on **2026-11-02**. `n.d.` means that no publication or modification date was visible in the opened source. Dynamic documentation may change. Product-version dates are not publication dates for an entire website.

| ID/direct source | Author/organisation; date; type | Claim used; limitation |
|---|---|---|
| [LM-01 — Introducing GNOME 51][LM-01] | GNOME Project; 2026-09-16; release | File status, settings, permissions, and driver transition; product description, not a user test. |
| [LM-02 — Introducing GNOME 48][LM-02] | GNOME Project; 2025-03-19; release | Notification grouping and well-being functions; no demonstrated effect size. |
| [LM-03 — Plasma 6.8][LM-03] | KDE Community; 2026-10-14; release | Dwell-click, task controls, and Wayland/XWayland; availability depends on the distribution. |
| [LM-04 — Search Everywhere can't find files][LM-04] | Adam Monsen, comments by Peter Eisenmann/Mathieu Bridon; 2025-12-13–16; user forum | Unclear search scope; GNOME 46, one case, no current GNOME 51 regression demonstrated. |
| [LM-05 — CIFS share freezes taskbar when NAS offline][LM-05] | Marc_Sch, comment by claydoh; 2025-12-10; user forum | Search blocking reported with an offline NAS; cause not reproduced independently. |
| [LM-06 — Install an application][LM-06] | Canonical; updated 2026-06-17; Ubuntu 24.04 guide | Snap/deb in App Center; shows choice structure, does not measure confusion. |
| [LM-07 — Manage updates][LM-07] | Canonical; updated 2026-10-19; Snap documentation | Automatic updates, scheduling, and hold; a CLI capability does not demonstrate GUI discoverability. |
| [LM-08 — Sandbox Permissions][LM-08] | Flatpak Project; n.d.; developer documentation | Sandbox and portals; broad grants remain possible, no general application audit. |
| [LM-09 — Steam Hardware and Proton][LM-09] | Valve; n.d.; Steamworks documentation | Linux layer and anti-cheat conditions; not every game was tested and not applicable to every kernel. |
| [LM-10 — KDE Connect][LM-10] | KDE; n.d.; project description | File, notification, and device functions; no evidence of complete platform parity or backup. |
| [LM-11 — Check out what's new in Android 17][LM-11] | Seang Chau / Google; 2026-06-16; release announcement | Application windows and phased rollout; no independent task measurement. |
| [LM-12 — Android 16: Productivity, security and more features on Android][LM-12] | Seang Chau / Google; 2025-06-10; release announcement | Notifications, hearing aids, and windows; distinguishes hardware/application conditions and components that were then future work. |
| [LM-13 — Back up or restore data on your Android device][LM-13] | Google; n.d.; support documentation | Account, recovery limitations, encryption, and diagnostics; Google service, not all Android variants. |
| [LM-14 — Access documents and other files from shared storage][LM-14] | Google Android Developers; updated 2026-10-28; API documentation | Document picker and restricted access; no user research or evidence of complete local/cloud parity. |
| [LM-15 — “I do (not) need that Feature!”: Understanding Users' Awareness and Control of Privacy Permissions on Android Smartphones][LM-15] | Sarah Prange et al.; Bundeswehr Munich/Innsbruck/LMU universities and Google Munich; SOUPS 2024-08; field study | 132 analysed participants over two weeks; data from April–May 2022, predominantly young Europeans, self-selection, and possible research effects. |
| [LM-16 — User Understandings of Technical Terms in App Privacy Labels][LM-16] | Ishika Keswani et al., Pomona College; SOUPS 2025-08; two online studies | Comprehension problems among 46/383 participants; abstract consulted, no behavioural effect or representative EU result established. |
| [LM-17 — Learn when you'll get software updates on Google Pixel phones][LM-17] | Google; n.d.; support policy | Support period by Pixel generation; not the entire Android ecosystem. |
| [LM-18 — Manage your ad privacy settings on Android][LM-18] | Google; n.d.; support documentation | Separate advertising settings; does not measure resistance, and disabling them does not cover all tracking. |
| [LM-19 — Android developer verification][LM-19] | Google Android Developers; n.d.; current policy/timeline | Starts 2026-09-30 at participating stores in Brazil, Indonesia, Singapore, and Thailand; global expansion planned for 2027, rollout remains subject to change. |
| [LM-20 — Pika Backup][LM-20] | Pika contributors/GNOME, maintainers Sophie Herold/Fina Wilke; n.d., version 0.8.4 dated 2026-08-04; product documentation | Personal backups and recovery boundary; not standard in every Linux desktop. |

[LM-01]: https://release.gnome.org/51/
[LM-02]: https://release.gnome.org/48/
[LM-03]: https://kde.org/announcements/plasma/6/6.8.0/
[LM-04]: https://discourse.gnome.org/t/search-everywhere-cant-find-files/33037
[LM-05]: https://discuss.kde.org/t/cifs-share-freezes-taskbar-when-nas-offline/42466
[LM-06]: https://ubuntu.com/desktop/docs/en/24.04/how-to/software/install-an-application/
[LM-07]: https://snapcraft.io/docs/how-to-guides/manage-snaps/manage-updates/
[LM-08]: https://docs.flatpak.org/en/latest/sandbox-permissions.html
[LM-09]: https://partner.steamgames.com/doc/steamhardware/proton
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
