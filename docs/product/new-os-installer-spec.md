<a id="nieuw-os--volledige-installer-vanuit-herkenning"></a>

# Nieuw-os — complete installer built on familiarity

Date: 2 November 2026. **Status: proposal following comparative research; no installer has been built or released.** The current repository has its own UEFI/QEMU boot process and a bounded native session; it has no demonstrated physical installation, personal authentication or complete account/encryption/recovery chain. [Repository baseline](../research/2026-11-02/repository-baseline.md)

<a id="gebruikersprobleem-en-keuze"></a>

## User problem and design decision

[SOURCE] The [31-aspect comparison](../research/os-installation-comparison.md) shows shared concepts but different phase sequences: Setup/OOBE, Recovery/Setup Assistant and distribution-specific installers/first-start wizards. Some advanced storage routes write before a later final confirmation. A copied screen sequence is therefore neither a safe nor a proven universal funnel.

[DESIGN DECISION] Use one familiar route: first ensure the interface can be perceived and operated, then provide a safe trial and explain suitability, then select the destination and protection, and write only after a complete change plan. First boot verifies the actual outcome. Optional guidance and services follow once the necessary foundation is usable. This design decision requires testing; the sources do not establish it as the optimal sequence.

The installer serves experienced computer users without a mandatory manual and beginners without requiring knowledge of EFI, GPT, mount points or filesystems. Advanced provides technical control with the same safety boundaries. The project's own kernel remains the foundation; Calamares/Anaconda are research material, not an adopted platform architecture.

<a id="voorgestelde-funnel"></a>

## Proposed funnel

```text
Gecontroleerd opstartmedium
  → Welkom: taal + altijd bereikbare toegankelijkheid
  → Toetsenbord/invoer proberen
  → Proberen / Installeren / Herstel (alleen beschikbare routes)
  → Geschiktheid en benodigde verbinding
  → Installatiemethode + geïdentificeerde schijf
  → Voorstel gegevens-/bootwijzigingen
  → Lokaal account + bescherming en herstelmiddel
  → Lokale voorkeuren + privacycontrole
  → Volledig wijzigingsplan + expliciete schrijfbevestiging
  → Installeren met echte fase-/foutstatus
  → Herstart + juiste mediuminstructie
  → Eerste boot: feitelijke configuratie en toegang controleren
  → Desktop + optionele, hervatbare onboarding/migratie
```

Networking is not a separate mandatory gate when all authenticated core components are on the medium. Downloads, time synchronisation or later provider connections show their own purpose and network boundary. An incomplete medium cannot proceed through a fictitious offline path. The user can return to the trial or stop without modifying the disk.

<a id="stapcontract"></a>

## Step contract

Every row is a [PROPOSAL]. References `Ixx` are dimensions of the source matrix, not screen numbers from other systems. The comparison dossiers provide the precise sources and exceptions.

| Step | Purpose / familiar comparison | Information for beginners | Advanced and safe default | Combine, defer or omit |
|---|---|---|---|---|
| S00 Boot | Separate source and destination; all studied routes have boot/recovery (I01) | Medium name/version, trial status and limited hardware support | Only verified payload/boot chain; no automatic repair of host storage | Do not hide firmware configuration in “Volgende” |
| S01 Welcome | Language before important text; Ubuntu provides accessibility early (I03/06) | Language, larger/higher-contrast display, speech/input where actually provided | No network detection of nationality; preserve selected assistive tools | Accessibility always reachable, no mandatory questionnaire |
| S02 Input | Keyboard before secrets; Windows/Ubuntu offer an early choice (I05) | Type a sample, choose a layout and see a test result without storage | Other layouts/IMEs only if actually supported; no password as test text | Changeable later; do not ask again at first boot |
| S03 Intent | Linux live-Try is a familiar safe experiment (I08) | “Proberen verandert de interne schijf niet”; installation and recovery separate | Trial session has no storage mutation right; recovery only if actually provided | May be offered at welcome, but sensitive steps follow usable input |
| S04 Readiness | Requirements/hardware differ across all routes (I09/23) | Whether display/input/storage work; known limitations and space/power | Precisely tested profile; warning ≠ guarantee. Unsafe/unknown write destination blocks installation | Show before disk selection; do not silently install drivers/firmware yet |
| S05 Connection if needed | Optional in Ubuntu; route-dependent in Windows/macOS (I07/22) | Why a connection is needed, which download and what alternative | Complete offline core; verified downloads, no external fallback or logging of captive-portal secrets | Skip if unnecessary; normal wifi switching later in settings |
| S06 Method/destination | All routes distinguish destination and preservation/replacement (I10–13) | “Bestaand systeem behouden”, “Deze schijf vervangen” or a demonstrated recovery route; disk name/capacity/identity | No destination to erase preselected; unknown layout is not free space | Show method and disk together; technical editor behind Advanced |
| S07 Change proposal | Automatic and manual layouts coexist (I11/25) | Before/after: which data/systems remain, disappear or move; boot consequences | Calculate internal layout after explicit choice; no writes during preview | Ordinary users need not choose a partition type; Advanced retains the same plan model |
| S08 Personal account | Account is a shared concept, its position differs (I15/18) | Name and actual sign-in method on this device; provider separate | Local account; no unauthenticated guest presented as personal. Do not derive privileges from a web form | Propose a non-identifying device name, editable in details; no mandatory online account |
| S09 Protection/recovery | Encryption and recovery models differ substantially (I14/17) | What is protected, how unlocking works and what losing the recovery material means | Personal data encrypted as a product objective; offer only with a real crypto/key/authentication chain; otherwise block | Combine with account if explanation remains clear; do not confuse PIN, login password and encryption key |
| S10 Local preferences | Timezone/region are not the same step everywhere (I04) | Timezone and optional display region; no location needed | Editable suggestion without silent GeoIP; device name visible | Do not duplicate language/keyboard; timezone can be combined with preferences |
| S11 Privacy check | OOBE/first-start choices differ (I19–21/24) | Local storage, no OS telemetry, optional later connections; security floor remains enabled | No provider escrow, no cloud/AI required; no privacy choice that disables isolation | Brief check, no collection of extra service questions; apps/AI/phone later |
| S12 Summary/authorise | A clear write boundary exists in some routes but is not uniform (I25) | Concrete destination, preserved/erased data, boot changes, protection/recovery and network download | No authorisation for an uncertain/stale plan; explicit action with consequence in button text | One complete boundary before any mutation, including through Advanced |
| S13 Install | All routes show execution and transition (I26) | Actual phase, errors, cancellation/recovery boundary; no false ETA | Installer broker executes only the authorised plan; do not log secrets | No offers, quiz or new account choice during copying |
| S14 Reboot/medium | Routes range from automatic reboot to live exit (I27/28) | Completion, which medium to remove, where the next boot comes from | No reboot with unfinished writes; error shows recovery path | Do not show a USB instruction for built-in recovery when no USB is present |
| S15 First boot | A new configuration phase is common (I29/30) | Actual account access and installation status; necessary error handling | Check state rather than blindly trusting a wizard flag; no reformat on failure | No repeated language/privacy questions; optional guidance may be skipped to reach the desktop |
| S16 Desktop | Success means a usable installed session (I31) | Where apps, search, material, settings and help are located, insofar as provided | Honest trial/personal/managed status; no fictitious network/apps | [Onboarding](new-os-onboarding-spec.md) voluntary; import separate and preserves the source |

<a id="schijfveiligheid-en-herstelbaarheid"></a>

## Disk safety and recoverability

**No mutation before S12.** Inventory is read-only and must not perform filesystem repair, automatic mounting with writes, journal replay on source volumes, resize, boot-entry changes, encryption conversion or formatting described as “voorbereidend” (preparatory). Advanced must not offer an editor that writes directly around the confirmation boundary either. This stricter boundary is a project safety decision based on the exceptions found, not a claim that all existing installers work this way.

The change plan contains at least: stable device identity plus a human-recognisable name/capacity, source/destination role, all parts to preserve/erase/shrink, existing OS/data detection with uncertainty labels, boot changes, encryption/recovery status, required space and any download. Unknown or encrypted data are explicitly identified as unknown. The plan states exact consequences; a confirmation checkbox does not replace understandable information.

Any change to destination identity, media, plan, encryption status or prerequisites invalidates authorisation. The broker revalidates identity and the authorised plan immediately before the first write and at relevant phases. Hotplug, corrupt planning or an incorrect destination handle causes a block, not an automatic replacement destination disk. A UI renderer has no general disk right and cannot fabricate evidence of consent. The future broker/capability contract requires separate design and negative tests.

**Dual boot is conditional:** offer it only for precisely tested boot, space, filesystem, resize and encryption combinations with a recovery route. Otherwise explain that preservation on this disk is unsupported; an explicitly selected other disk or stopping remains possible. The installer must not silently select the only offered erasure route. A backup is required beforehand when changing existing data; detecting a backup does not prove that recovery works.

**Cancellation before writing:** zero disk changes and erase temporary secrets. **After writing:** no promised undo; stop only at safe phase boundaries and report actually completed/incomplete steps and recovery options. Source loss after explicit erasure cannot be recovered with a “Terug” button. Interruption testing and recovery media are release gates. Download errors leave a valid source/installation intact where technically demonstrated; do not initialise a new disk to mask an error.

<a id="encryptie-accounts-en-persoonsgegevens"></a>

## Encryption, accounts and personal data

Personal data protection follows [the local privacy contract](../LOCAL-PRIVACY-CONTRACT.md), not a newly invented cryptographic layer. Maintainable cryptography, entropy, key isolation, authentication, storage integrity, update/package trust and recovery must actually exist. A missing chain blocks the claim “veilig persoonlijk geïnstalleerd” (English: securely installed for personal use). No dummy checkbox, plaintext fallback or automatic provider key escrow.

The normal route distinguishes login, unlocking and recovery. Biometrics/TPM/PIN require demonstrably supported hardware and a safe fallback; do not suggest physical presence or biometrics on the basis of a VM. Recovery material is stored outside the volume to be erased or protected by it, with explanation and a safe availability check; it is not sent to a provider, placed in telemetry or included in logs/screenshot evidence. Remove secrets from input buffers/progress state under a memory/service contract to be tested separately.

<a id="state-navigatie-techniek-en-uitvoeringsplan"></a>

## State, navigation, technology and implementation plan

Proposed state model: read-only inventory → draft plan → fully validated plan → explicitly authorised plan → execution → verifiable completion, error or recovery. Going back before execution removes dependent authorisations; retaining language or layout does not retain disk consent. After writing, safe phase/recovery actions replace the ordinary wizard action Terug.

Future components: boot/payload verifier, readiness probe, read-only storage inventory, plan builder, account/key service, summary renderer, bounded installer broker, progress/recovery coordinator and first-boot validator. These are required components, **not existing services**. Exact native files and IPC contracts are determined only after current code inspection and security design. Do not perform a broad rewrite or modify existing evidence or the QEMU storage model to make this documentation appear operational.

Preferred sequence for a later implementation plan:

1. Pure, non-writing plan/state experiment with synthetic disks; focus on invalid and ambiguous destinations. A UI prototype remains labelled as a simulation.
2. Real storage inventory and capability boundary with disposable images; no initialisation of unknown volumes. Negative tests and interruptions first.
3. Real account/crypto/recovery chain and accessible input from boot. Do not claim personal privacy in advance.
4. Authorised execution and first-boot recovery on the fixed virtual profile; then complete regression gates and a human task trial.
5. Physical installation only after separate hardware evidence and an explicit instruction. This research assignment does not grant that permission.

Performance impact: inventory/downloads must not block input or assistive tools; cancellable probes, bounded memory/log growth and no animation dependent on every disk read. Measure response and phase costs on the actual profile; no substantiated time budget or installation duration has been established.

<a id="acceptatie-en-regressierisico"></a>

## Acceptance and regression risk

| ID | Required before supported execution | Check |
|---|---|---|
| IN-01 | No mutation before complete plan confirmation, including in Advanced | Broker/state negative tests, read-only source monitoring, write audit on disposable images |
| IN-02 | Correct destination identity; change invalidates confirmation; unknown ≠ empty | Multiple identical disks, encrypted/unknown/full, hotplug, stale/corrupt plan, unauthorised handle |
| IN-03 | Beginners understand preservation/erasure without technical layout terms | Task trial with synthetic data: ask participants to explain consequences beforehand; serious misunderstanding blocks release |
| IN-04 | Keyboard, language, assistive tools and focus work before secrets and disk selection | Keyboard/screenreader/contrast/scale/reduced-motion trial from boot, not only on the desktop |
| IN-05 | Actual progress, safe cancellation boundary, recovery after interruption and correct first boot | Fault injection at each phase, failed payload/update/boot, recovery-material trial; no fictitious success |
| IN-06 | Local account and protection are real; optional providers do not change the security floor | Authentication, isolation, key, grant and network negative tests; human security review remains separate |
| IN-07 | Source preservation and boot/dual-boot claims hold for each offered route | Before/after inventory, applicable hashes/metadata, boot/recovery of every preserved system |

Risks: stale device identity, disk inventory that already writes, incorrectly stored recovery material, differences between UI consent and broker rights, misleading first-boot status, missing drivers/assistive tools and regression in existing virtual storage recovery. Build, lint/type/test commands follow the current code involved; they were not executed here because only documentation is being added. Safety evidence, usability and physical support remain separate acceptance levels.
