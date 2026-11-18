# English reading guide to the completeness assessment

This guide summarises the sixteen original assessment records listed below. The assessment examined commit `380111b68688c981eec6abb8fab29174a29f5601`, recorded as `origin/main` on 2 November 2026. Its [source record](source-baseline.md) identifies that exact code and its [original introduction](README.md) states that no product code, product criteria or roadmap weights changed. This source binding and declared role distinguish the assessment from a current normative contract.

All original Dutch files remain unchanged. The sections below provide English access to their purpose, principal conclusions and limits; detailed source tables, quotations, commands and raw data remain in the linked originals. No original test is represented as newly executed. Recommendations retain their proposal status. Current [release requirements](../../FULL-TEST-RELEASE.md), [accepted decisions](../../DECISIONS.md), [project checkpoint](../../PROJECT-STATE.md) and [progress rules](../../PROGRESS.md) remain authoritative.

<a id="introduction"></a>

## Introduction and conclusion

Original: [README.md](README.md).

The assessed source had an independently developed x86_64/UEFI Rust kernel, QEMU tests and separate native services. Bounded evidence existed for memory protection, capability IPC, storage, updates, transport and a graphical session. The report classified it as a research/technical prototype. It did not establish a desktop release suitable for daily use. The sixty-two full release rows remained open until sign-off on a final integrated image; host, VM and separate-branch evidence retained their separate sources.

The assessment recorded all fifteen queried workflows on its exact source as successfully completed, including the native window-session/Windows QEMU workflow. This was workflow evidence, with no new hardware, human or full release acceptance. Its main blockers were an integrated release image, actual authentication/key recovery/encrypted user storage, native network/NAS authority, accessibility and physical-profile support, followed by independent evaluation and maintenance validation. External standards were scope references; the failed UEFI Forum HTTP request was explicitly unverified.

<a id="source-baseline"></a>

## Source baseline

Original: [source-baseline.md](source-baseline.md).

The clean assessment branch `simonvanieperen-cautious-happiness` selected `380111b68688c981eec6abb8fab29174a29f5601`, with parent `74a34d05ef5591b782a0da9da34e4669910d0722`. An initially pending workflow later completed successfully, giving fifteen successful queried workflows on the assessment head. PR #127 was integrated at that head; PR #125 was its recorded predecessor. The original retained a separate nine-PR development inventory, including stacked desktop work and storage/NAS experiments. Those branch results were not counted as additional main features.

The source record also identified an older WORK-QUEUE checkpoint as stale and respected its exclusive owner. Its PR, worktree and score observations are historical snapshots. Reproduction commands were read-only; no remote PR, branch, issue or release was changed by that audit.

<a id="completeness-matrix"></a>

## Completeness matrix

Original: [completeness-matrix.md](completeness-matrix.md); data: [matrix.json](matrix.json), [schema](matrix.schema.json), [validation record](verification.json).

A matrix row grouped a related facility or acceptance cluster. It was not a second product-progress percentage. The report grouped boot/kernel/IPC/services, drivers/storage/recovery, desktop/session/Materiaal, installer/apps, browser/media/accessibility, network/privacy/NAS/AI/quantum, and compatibility/release work. Multiple evidence levels in one row did not mean that every property had passed on one source or image.

The strongest bounded native foundations were the project's loader/kernel, paging, ring-3 processes, capability IPC, virtual block path and update/randomness gates. Other source and prototype work still needed integrated product evidence. The L02 QSIM job service had historical bounded native acceptance; the full integrated FR-L04 regression remained open. Browser engines, codecs, broad drivers, compatibility and physical-QPU work carried substantial continuing maintenance costs. None of the matrix counts estimated remaining development time.

<a id="release-criteria-map"></a>

## Release-criteria mapping

Original: [release-criteria-map.md](release-criteria-map.md).

The table mapped all sixty-two numbered `FR-*` release rows to audit groups, from `FR-K01` through `FR-R04`. Every mapped release gate was recorded as `OPEN`. A related component or audit-group result did not close a full user-task requirement. The mapping was navigation and changed no requirement, evidence boundary or owner. Current English requirement text is available in [FULL-TEST-RELEASE.md](../../FULL-TEST-RELEASE.md); the original mapping preserves the assessment-time wording and group identifiers.

<a id="evidence-register"></a>

## Evidence register

Original: [evidence-register.md](evidence-register.md).

The register distinguished `SOURCE_ONLY`, `HOST_TESTED`, `VM_TESTED`, `HARDWARE_TESTED`, `HUMAN_TESTED`, `INDEPENDENTLY_REVIEWED` and `RELEASE_VALIDATED`. An earlier VM run kept its own commit/profile; exact-head CI established only the workflow actually executed. The audit itself performed no product build, QEMU run, hardware trial, user trial, human security review or release validation.

Its source table covered boot, memory, IPC/processes, volume storage, crypto design, compositor/editor, browser prototype, accessibility preparation, native packet transport, host TLS, privacy contracts, signed updates, bounded SDK packages, QSIM and release validation. It preserved both supporting evidence and missing coverage. In particular, older GUI evidence had not completed Saved/Reopen; host browser/TLS evidence did not establish a native browser or product handshake; A01's bounded two-point SDK result did not complete package lifecycle; historical L02 evidence did not establish physical QPU use or quantum speedup.

<a id="dependency-map"></a>

## Dependencies

Original: [dependency-map.md](dependency-map.md).

The graph described prerequisites for full acceptance: trusted boot and kernel boundaries; identity/key/account continuity; durable, authenticated user data and independent recovery; service-backed desktop/apps/accessibility; native network/TLS and destination authority; paired encrypted NAS/synchronisation; bounded local AI; and a final supported release. Its arrows meant a prerequisite for acceptance, not an observation that the dependent feature already worked.

The five critical paths were storage admission through encrypted cold recovery and save/reopen; entropy/keyring through account/session/lock/revoke; drivers through config/DNS/TLS/app egress and updates; device identity through encrypted NAS reconciliation and independent recovery; and installer through accessible daily tasks and recovery. Test backends could support interface research but did not replace these native prerequisites.

<a id="release-profiles"></a>

## Release profiles

Original: [release-profiles.md](release-profiles.md).

The report classified `TECHNICAL_PREVIEW`, `EXPERIMENTAL_DESKTOP`, `DAILY_DRIVER_BETA` and `CONSUMER_RELEASE`. Their descriptions progressed from disposable developer/QEMU experiments, through a bounded native desktop, to authenticated persistent data, networking/apps/accessibility/maintenance and independently validated consumer delivery. It named no universal hardware or application compatibility guarantee. Full installation/migration and the narrower update-recovery route remained distinct.

**Source wording ambiguity:** the original opening uses `wijzigen` (“modify”) when discussing the stricter conjunctive release acceptance, without an explicit negation. The original introduction and criteria map explicitly state that the audit changes no criteria or acceptance requirements. That wording is retained in the original. This guide adopts no changed or weaker release gate: the current normative release contract controls. The profile table is an assessment classification, with no release approval supplied by its translation.

<a id="priorities"></a>

## Priorities

Original: [priorities.md](priorities.md).

The recorded P0/P1 order prioritised one integrated final image, data safety and recovery, actual session authority, native destination/TLS rights, encrypted paired NAS, transactional updates/recovery, a named physical profile, accessibility and human tasks, then the native editor/settings and maintained browser/formats/package lifecycle. Labels were gates, not time estimates. Data integrity, the Security Floor and prerequisites preceded visible polish.

Later proposal categories included reliable resumption/onboarding/formats/notifications/local AI, evidence-based Materiaal differentiation, and separately authorised research such as integrated QSIM regression, physical QPU, CHERI, additional ISAs and formal work. The report prohibited deferring a blocker merely to increase a percentage and introduced no competing implementation roadmap.

<a id="product-gaps"></a>

## Product gaps

Original: [product-gaps.md](product-gaps.md).

The report separated a functioning bounded kernel/system experiment, a complete daily-use product, and an evidence-based reason to switch platforms. Native foundations did not establish a complete device, account, service or storage platform. No single final image had completed the ordinary installation/session/apps/formats/network/NAS/AI/recovery journey. Materiaal's document/tool/history/rights model was a distinct proposal with prototypes and native components; representative user evidence of a switching advantage was absent.

It therefore withheld claims of a complete daily-use release, broad hardware/application support, product-wide authentication/encrypted storage/backup, general native internet/NAS, integrated local AI, complete accessibility, independent security sign-off or measured parity. Historical bounded native QSIM acceptance remained recognised with its separate integrated regression requirement.

<a id="technical-gaps"></a>

## Technical gaps

Original: [technical-gaps.md](technical-gaps.md).

The eight work areas were an integrated source/image for native gates; complete service/fault/lifecycle/scheduler contracts; documented physical driver and DMA/reset limits; durable encrypted user storage and independent restoration; native network/config/DNS/TLS/authority; verified package-to-process lifecycle and quotas; native UI/accessibility/fonts/settings/media; and provenance/SBOM/reproducible release maintenance. The source treated browser engines, codecs, broad filesystems, compatibility, wireless devices and QPU hardware as continuing maintenance programmes, requiring evaluation of maintained components and their licensing, ABI, build scripts and isolation.

<a id="security-gaps"></a>

## Security gaps

Original: [security-gaps.md](security-gaps.md).

The assessment recognised explicit kernel/capability/usercopy contracts and scoped B01/B03 evidence. Its open product boundaries included B02 authenticated encrypted persistence/recovery, B04 boot/distribution trust and provenance, B05 independent human review, B06 native LP-01–31 enforcement, actual authentication/unlock/recovery and native TLS/incident ownership. Its stop conditions included untrusted pointers/lengths, privilege escalation, uncontrolled DMA, logged secrets, fail-open authentication/updates, unverifiable package execution and success-shaped I/O errors. This particular completeness audit performed no new code security review and did not classify those product gaps as newly discovered vulnerabilities.

<a id="accessibility-gaps"></a>

## Accessibility and internationalisation

Original: [accessibility-gaps.md](accessibility-gaps.md).

The recorded specifications and browser checks covered parts of keyboard/focus, scale, contrast and reduced motion. The source found no demonstrated full native assistive-technology chain, screen-reader/braille/switch workflow, 200% task trial or human usability acceptance. UTF-8 storage and a Dutch prototype did not establish font fallback, shaping, RTL, IME/dead keys, locale formatting, language switching or screen-reader language support.

Its proposed verification combined a native semantics/event API, keyboard/screen-reader setup-to-recovery tasks, contrast/zoom/non-colour and reduced-motion checks, explicit participant/task evidence, and Unicode/locale/input tests. WCAG 2.2 supplied web scope and interaction principles, without automatic native OS conformance.

<a id="compatibility-gaps"></a>

## Compatibility and migration

Original: [compatibility-gaps.md](compatibility-gaps.md).

No full product compatibility/migration route or accepted import/export matrix was established. The proposed order preserved export and independently verifiable backup, selected a small useful format set, defined fidelity/loss/metadata/malformed-input boundaries per format, used actual document fixtures, and assessed filesystems, sharing protocols and application compatibility separately. Games, anti-cheat and professional applications needed concrete tasks and support ownership. The recommendation to evaluate maintained browser engines before building one remained audit advice; the full release assignment required the Chromium/Gecko feasibility comparison.

<a id="visual-design-gaps"></a>

## Visual design and interaction

Original: [visual-design-gaps.md](visual-design-gaps.md).

The source recognised Materiaal's chosen object/tool/history/rights model, browser screens and native compositor/editor components. It still lacked one operated native first-boot-to-recovery journey, a complete loading/error/cancel/focus/long-text/scaling matrix, broad window/display behaviour, complete font/accessibility/motion coverage and representative human assessment. Screenshots supported examination but did not replace interaction. Neither a distinctive appearance nor decorative effects established originality, usability or product value.

<a id="risk-register"></a>

## Risk register

Original: [risk-register.md](risk-register.md).

The planning register covered hardware fragmentation, browser patching, application/game compatibility, updates/incident response, key recovery/migration, storage loss, accessibility, performance/energy, maintainership, user research, excessive scope, UI before durable services, kernel progress without a usable product, drifting documentation, permanent prototype assumptions and confusing open branches with integration. Every row paired uncertainty with detection, mitigation, a stop condition and an expertise owner. These were planning risks; the table did not assert that every loss, vulnerability or hardware fault had occurred. Required claims remained tied to the actual tested hardware, application/version, backend, workload and source.

<a id="recommended-roadmap"></a>

## Recommended sequence

Original: [recommended-roadmap.md](recommended-roadmap.md).

The assessment proposed a sequence from exact-source evidence hygiene, through coherent kernel/rights/data foundations, an accessible daily desktop, native connected/migration workflows, separately validated differentiating functions, and final supported release/maintenance qualification. Its proposed exits included cold save/reopen/recovery, negative authority/tamper tests, actual user tasks, identity/scope traffic evidence, explicit format-loss reporting, bounded local AI and separate QSIM/QPU evidence, followed by repeated duration/recovery and independently evaluated release evidence.

Those historical phase numbers are not the current approved Phase 0/governance plan. The original expressly required linking work to the existing queue, roadmap, release contract and checkpoint. This English guide preserves that reporting role and creates no new schedule, implementation authority, source acceptance or score change.
