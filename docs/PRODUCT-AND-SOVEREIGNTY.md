# Product vision, privacy and European independence

Status: design proposal, 26 October 2026. This document describes desired properties; none has yet been implemented or certified. Architecture and technical development phases are in `ARCHITECTURE.md` and `ROADMAP.md`.

## 1. One clear product promise

An independent, open operating system through which people can use their devices and data with local accounts, a personal NAS or voluntarily offered NAS space with friends/family for end-to-end encrypted synchronisation, through LAN or explicitly permitted internet access, without a vendor cloud for OS data, with understandable permissions and recoverable changes.

The initial technical experiment runs on the specified virtual machine; product design and interaction research begin at the same time. Design interfaces to support different devices later. A phone, PC and small embedded device have different interactions, energy requirements and security boundaries. One product family therefore need not have an identical interface or installation file.

'As secure as macOS', 'as usable as Windows' and 'as configurable as Linux' are inspirations, not measurable specifications. Eventually compare concrete tasks on specific versions and devices. A claim that this OS surpasses an existing OS is unsupported without comparative measurements.

## 2. Originality that users notice

Novelty need not mean a new programming language, cryptography or incompatible file format. The following proposes a coherent product of its own; it does not claim that these ideas have never been applied.

| Product idea | Desired behaviour | Initial evidence |
| --- | --- | --- |
| Task-centred workspace | A user groups documents, apps and relevant settings around an activity. | A prototype in which participants resume a task without instructions from its creator. |
| Understandable consent | For example, an app receives only the chosen document for this action; the user can inspect and revoke that access. | The user can identify what was shared; the negative access test passes. |
| Safe customisation | Theme, shell and apps are replaceable within defined permissions. Critical changes have a recovery route. | Incorrect configuration is rolled back without losing user files. |
| Local operation as the norm | Search, settings and local documents work without an account or network. | Task test after installation with networking disabled. |
| Understandable errors | The message states the problem, consequences and one executable recovery action. | The user recovers from the error without terminal knowledge. |

The OS receives its own visual identity and interaction model. Originality is an acceptance criterion from M0. A conventional desktop with different colours is insufficient as the final concept. *Addition of 30 October 2026 (ADR-058):* familiar interaction—desktop, taskbar, start menu and login—is now required; originality resides in independent code, visual identity, the permission model and additional functions, not unfamiliar basic operation. A prototype may investigate new ideas before the kernel executes them; record product evidence and system evidence separately.

### Three concept directions to investigate
These directions are hypotheses, not an already selected product or a proven world first. Develop genuinely different ways of working, not three themes on the same screen layout.

| Concept | Primary organising model | Distinctive research question |
|---|---|---|
| **Materiaal** | People start with a document, photo, collection or device; installed apps offer appropriate operations. | Can one object use different tools with temporary permissions for each operation, without repeatedly moving files between apps? |
| **Werk** | People open an activity with a goal, current step, results and recovery timeline; apps appear where needed. | Can work resume understandably after interruption, including on a second personal device through an explicitly paired personal NAS? |
| **Atelier** | People arrange direct control surfaces: multiple tools, material and feedback visible together, suited to the task and screen. | Can beginners start immediately while advanced users substantially customise operation without fragmented settings or broken accessibility? |

Use the same five task trials for each concept: installing, creating/editing something, retrieving it, interrupting/resuming and recovering from an error. Compare with a conventional windowed desktop, app launcher and file manager: state exactly what changes, what deliberately remains familiar and what benefit users experience. Then select one coherent direction; do not automatically combine every idea.

### High-quality finish as a product requirement
Develop a dedicated design system in parallel: typography, spacing, colour, icons, materials, focus, motion rules, error states and consistent language. Also assess installation, first use, settings, updates, search, file selection and recovery; quality applies throughout the experience.

Define measurable interaction budgets, test smooth animation and immediate feedback, and provide reduced motion. Treat keyboard, touch and assistive technologies as full interaction methods. The SDK and UI toolkit must help external developers achieve that quality through documentation, examples and test tools.

Privacy should be natural to operate: visible data location, permissions attached to concrete actions, understandable NAS pairing and recovery without a vendor account. AI assists locally and remains optional. Assess a design without AI as well, so the ordinary product stands on its own merits.

### European standards as a design brief
Baryon OS's European orientation means privacy, control, accessibility, transparency, open collaboration and portability. These values must be perceptible in the product and testable in its behaviour. Specific legal or technical standards will later be linked to evidence in a requirements register for each applicable area; European orientation here does not refer to a single established certification.

An unknown technical possibility receives a research question and a bounded experiment. Do not lower the product ambition because the initial implementation is small; a demonstrated limit leads to an explicit design decision.

## 3. Privacy as observable behaviour

These are product requirements for future implementation:

- The base installation works with a local account. Personal storage consists of local data, optional E2EE storage on a personal NAS/offered NAS space, and optional opaque client-side encrypted objects at an explicitly selected external storage provider. LAN remains the default for personal NAS; internet is explicit for each peer/provider. No mandatory vendor cloud, silent relay/fallback, provider key escrow or fallback server. A provider account may be required for the voluntarily selected provider but grants no Baryon OS content key. Local tasks continue to work without a NAS/provider. The NAS host manages quotas and may erase its copy without guest consent; that grants no decryption key or document access.
- No external OS telemetry or automatic external crash uploads. Diagnostics remain local, bounded and inspectable. A user may explicitly back them up with client-side encryption to a permitted NAS or opaque provider storage; the storage backend receives no plaintext diagnostic content through this route.
- Crash exports show which data will be included before transmission. No automatic memory dumps potentially containing documents or keys.
- No advertising ID, system advertising, mandatory platform account or preselected data sharing.
- Network traffic can be explained per service. Updates and time synchronisation have visible settings and must not be silently bundled as telemetry.
- AI is optional and removable, but inference occurs exclusively on the device. Prompts, history, embeddings and results remain local; any storage backup is end-to-end encrypted to an explicitly paired personal NAS. No cloud fallback or outsourced inference. A separate model downloader has no access to personal context; the inference service has no network permission.
- Offline use must not trigger forced cloud registration. Users are clearly informed about missed security updates.
- Export and migration must be possible with documented formats. Revoking access cannot undo copies previously made by another party.

Downloadable third-party apps remain possible with explicit file, NAS, device and network permissions. An app with both read and internet permissions may transmit authorised data externally; make that combination visible and provide no absolute promise against leakage. Details and checks: [LOCAL-PRIVACY-CONTRACT.md](LOCAL-PRIVACY-CONTRACT.md).

Evidence: test network traffic on a clean installation, repeat after updates and after enabling/disabling optional services. While the prototype lacks networking, absence of traffic does not prove that the future networked version respects privacy.

## 4. Lightweight and fast: measurement before marketing

For every measurement, record the commit, build profile, host processor, allocated CPUs/RAM, QEMU version, acceleration type, display setting, dataset and measurement procedure. Do not directly compare TCG emulation with hardware acceleration.

| Property | Provisional test target | Condition |
| --- | --- | --- |
| Startup | First a reproducible baseline, then an agreed budget for each phase. | Measure firmware, loader, kernel and usable session separately. |
| Memory | Measure kernel, services, shell and cache separately; treat an unexplained increase above 20% as a review point. | This percentage is a project proposal, not a proven standard or absolute rejection threshold. |
| Responsiveness | Simple local actions provide visible feedback within 100 ms in the agreed reference setup. | Design goal for the later shell; no claim about existing code or all hardware. |
| Idle state | No unintended busy-wait loops; record periodic background tasks. | CPU percentages are meaningful only with a reproducible measurement setup. |
| Energy | Measure sleep, wake and actual energy use on supported physical hardware. | A small kernel or good QEMU score does not prove long battery life. |

Set a budget for each phase that fits the functions then available. A superiority claim such as faster than Windows requires the same tasks, comparable functionality and a published measurement protocol.

## 5. Accessibility and European usability

Product requirements: full keyboard operation, clear focus, scalable text, sufficient contrast, reduced motion, screen-reader support and signals that are not exclusively audible. Avoid security questions relying solely on colour, sound or rapid responses.

WCAG 2.2 provides testable accessibility criteria for web content. Apply them directly to any web interfaces and use them as inspiration for the native shell. The native environment also needs its own accessibility API, semantic UI structure and practical tests with assistive tools. This is not a declaration of legal conformity (W3C, 2024).

Plan translation from the first UI: external text files, Unicode, different keyboard layouts, date and number formats, and text expansion after translation. Investigate needs with Dutch and other European users; there is no single uniform European user. Select one clear audience for the first trial, such as privacy-conscious desktop users, and test five core tasks with multiple participants. Results from a small trial are directional, not representative of all Europe.

## 6. Independence has multiple layers

| Layer | Meaning for this project |
| --- | --- |
| During use | OS data local, E2EE on a paired personal NAS or, after explicit selection, opaque client-side encrypted objects at an external provider; internet explicit for each peer/provider. Host/provider receives no content key. AI inference only on-device. No mandatory vendor cloud, silent relay/fallback, provider key escrow or OS telemetry. Updates/model downloads may use a separate downloader; third-party apps only with assigned permissions. |
| During development | Codex can assist writing and research; source code, decisions, tests and build instructions remain available outside the conversation. |
| During builds | A third party must be able to build from source without a Codex account. Check all download and build dependencies. |
| Publication | Code remains exportable through Git; release files, issues and documentation receive a portable copy. |
| Governance | Publish decision rules, maintenance roles, funding dependencies and the security reporting process. |
| Hardware | Firmware and devices may retain external dependencies. An independent kernel does not automatically make hardware auditable. |

Codeberg is a platform operated by a non-profit association in Berlin. Forgejo is the self-hostable software on which Codeberg runs. This makes them relevant options for European control; it does not guarantee that every processing activity or dependency is European (Codeberg, n.d.; Forgejo, n.d.). Self-hosting requires administration, updates, backups and recovery tests. For a beginner, a managed platform is generally initially preferable to an additional server project.

A GitHub mirror can facilitate participation. Define one primary repository and avoid competing issue and decision systems. Check current platform conditions and support for the chosen Codex workflow before use; this document guarantees no particular integration.

## 7. Open source and governance

A visible repository without an appropriate licence is insufficient. Use an established, suitable OSI-approved licence and record the choice before publishing code publicly. Check compatibility of reused code and record provenance, version, licence and any modifications. Do not accept AI contributions without checking provenance either.

The European objective concerns control and usability, not a ban on non-European users. A licence excluding particular nationalities, businesses or fields of use is incompatible with the Open Source Definition (Open Source Initiative, n.d.). Establish privacy and product values through governance, default behaviour and technical interfaces.

Development may continue while a licensing decision is prepared; before that decision, do not present the project as already legally published as open source. This starter package does not declare the future product compliant with the GDPR, product legislation or procurement requirements. Have applicability assessed when the audience, distribution model and service provision are concrete.

Minimum governance before a public trial: a named maintainer, contribution procedure, licensing/provenance register, security contact, release process, supported versions and a process for reporting defects without publicly exposing sensitive details.

## Sources

Codeberg. (n.d.). *What is Codeberg?* Retrieved October 26, 2026, from https://docs.codeberg.org/getting-started/what-is-codeberg/

Forgejo. (n.d.). *Forgejo: Beyond coding. We forge.* Retrieved October 26, 2026, from https://forgejo.org/

Open Source Initiative. (n.d.). *The Open Source Definition*. Retrieved October 26, 2026, from https://opensource.org/osd

Open Source Initiative. (n.d.). *OSI approved licenses*. Retrieved October 26, 2026, from https://opensource.org/licenses

World Wide Web Consortium. (2024, December 12). *Web Content Accessibility Guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/
