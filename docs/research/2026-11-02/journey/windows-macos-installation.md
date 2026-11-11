# Windows and macOS installation journeys

Installation on both platforms consists of different phases; disk changes, account choices and privacy choices therefore do not fit one assumed universal screen sequence.

## Scope and source checks

Research date: **2 November 2026**. Method: official release, installation, recovery, accessibility and administration documentation searched and read. This is document research, **not a performed installation, user test or project screenshot record**. `[SOURCE]` means described vendor behaviour; `[ANALYSIS]` is interpretation within this research; **insufficiently documented** means no sufficiently precise evidence for this release/route. N/A applies only to the selected route.

- **Windows:** Windows 11 **26H2**, Home/Pro for personal use, x64, generally available 29 September 2026. The download page names 26H2; no ISO was downloaded, so the exact media build and hash were not established. Main route: clean USB installation. Comparison route: upgrade/reinstallation from running Windows. 25H2 remains supported; 26H1 is a separate hardware-oriented release, not a general upgrade from 24H2/25H2. [WI01] [WI02]
- **macOS:** **27.2 Golden Gate**, Apple silicon, Recovery followed by Setup Assistant on a new/empty user volume. Apple's release overview and security register date 27.2 to **26 October 2026**, before the research date; this is the regular release entry, not a beta. The same register lists Tahoe **26.7.2** and Sequoia **15.8.2**. This establishes released security updates, not guaranteed future support duration. [WI14] [WI15]
- **Intel variant:** Tahoe **26.7.2** only on compatible models, such as MacBook Pro 16-inch 2019. Recovery may offer another previously installed or originally supplied version. Starting Recovery therefore does not guarantee installation of the latest patch named here. [WI16] [WI17]

**Source conflict:** Microsoft's general screen-reader guide contains older Setup screens and an offline/local-account route. This does not establish that a current 26H2 Home/Pro consumer installation works that way: current system requirements require internet and a Microsoft account during initial personal setup. The guide is used here only as bounded evidence for described keyboard interactions and additional OOBE options. [WI05] [WI06]

## Comparison register: 31 aspects

All cells are `[SOURCE]`, except explicitly marked analysis or evidence gaps. **This numbering is not a chronological screen sequence.** Windows “Setup” writes the OS; “OOBE” configures the first user experience. macOS “Recovery” restores/installs; “Setup Assistant” configures users and services. [WI03] [WI04] [WI17] [WI20]

| No. | Aspect | Windows 11 26H2: USB → OOBE | macOS: Recovery → Setup Assistant |
|---:|---|---|---|
| 1 | Boot medium/recovery | Installation USB through firmware boot menu; manufacturer determines the key. Creating the USB erases its contents. [WI02] | Apple silicon: hold power → Options. Intel: Command-R; internet variants below. [WI18] |
| 2 | First visible installer | Language settings first. [WI03] | Startup options/Recovery first, possibly unlocking a volume and existing administrator; then utilities and Reinstall macOS. [WI19] |
| 3 | Language | Setup language; possibly repeated in OOBE. [WI03] [WI04] | Setup Assistant begins with language. After Apple silicon erasure, language also appears during reactivation. [WI20] [WI27] |
| 4 | Region/time zone | Region in OOBE; automatic time zone depends on location service. No universal separate time-zone screen established. [WI04] [WI12] | Exact region/time-zone screen and position insufficiently documented in the selected version-27 guide. [WI20] |
| 5 | Keyboard | Setup layout; possibly a second layout in OOBE. [WI03] [WI05] | Exact input-source/layout screen insufficiently documented. [WI20] |
| 6 | Accessibility | Guide describes Tab, arrows, Space and Enter. Complete screen-reader/audio support **before erasure** on 26H2 media unconfirmed. [WI05] | Setup Assistant offers accessibility choices; this is after installation. Complete Recovery screen-reader coverage **before erasure** unconfirmed. [WI19] [WI20] |
| 7 | Network | Personal Home/Pro: internet required during initial setup. A missing network driver may make this dependency problematic; the final sentence is [ANALYSIS]. [WI06] | Recovery download requires internet; activation also required after Apple silicon erasure. Some corporate/captive networks do not work for Recovery. [WI17] [WI27] |
| 8 | Try versus Install | No live-desktop trial described. [WI03] | N/A as a live-desktop trial: Recovery offers recovery utilities; the user selects reinstallation. [WI19] |
| 9 | Hardware check | Compatible CPU, TPM 2.0, UEFI/Secure Boot capability, RAM/storage requirements. Installation Assistant checks compatibility; no evidence that every USB check is identical. [WI02] [WI06] | Check model compatibility beforehand; Software Update offers suitable software. No universal checklist screen in Recovery established. [WI16] [WI29] |
| 10 | Storage device | Select target partition. [WI03] [ANALYSIS] Verify physical disk identity. | Installer selects target volume; where HD and HD–Data appear, Apple names HD. Unlocking may be required. [WI17] |
| 11 | Partitioning | Clean route: remove target partitions → unallocated space. [WI03] | Erasure/volume layout occurs separately in Disk Utility; the Apple silicon route uses APFS. An additional APFS volume is an alternative. [WI27] [WI26] |
| 12 | Retain/replace | Clean erases files/apps/settings/OEM customisations; in-place offers retention options. [WI03] | Reinstallation without erasure retains apps/personal data; erasure is a separate destructive action. [WI17] [WI27] |
| 13 | Dual boot | No guided dual-boot route in this clean installation. BCDBoot supports multiple Windows installations; Linux coexistence is not established by this. [WI13] | Additional compatible macOS on an APFS volume or external disk possible; Apple warns against retaining severely outdated versions long-term. [WI26] |
| 14 | Disk encryption | On suitable hardware, account setup activates Device Encryption; recovery key held with Microsoft/work/school account. No universal disk-encryption screen before installation. [WI07] | FileVault/personal recovery key: see explanation below. Hardware encryption alone does not provide the same password protection. Panel may appear; position varies. [WI25] [WI23] |
| 15 | User account | In OOBE; the first regular Setup account receives administrators-group membership. Organisation management is a separate variant. [WI04] [WI09] | Local computer account and password; first user is administrator. Apple Account is separate. [WI20] [WI22] |
| 16 | Device name/hostname | Device name described in the OOBE guide; exact 26H2 screen position not pinned. [WI05] | Separate hostname screen not established. [WI20] |
| 17 | Password/PIN/biometrics | Microsoft sign-in, then Hello/PIN; biometrics depend on hardware. Administrative tasks require UAC consent or administrator credentials. [WI05] [WI06] [WI08] | Computer password; Touch ID only where supported. Password remains relevant for administrative tasks. [WI20] [WI23] [WI29] |
| 18 | Online versus local | Standard personal Home/Pro requires a Microsoft account. Older offline instructions are not a universal current option. [WI06] [WI05] | Local computer account; Apple Account can be added later in System Settings. This does not remove network requirements. [WI21] [WI17] |
| 19 | Privacy choices | OOBE lists up to seven choices, not identical for everyone. Adjustable afterwards; exact defaults not independently documented. [WI04] [WI11] | Data and Privacy information, location and analytics are distinct subjects/panels; no evidence of identical visibility/defaults on every Mac. [WI23] |
| 20 | Telemetry | Required and optional diagnostics distinguished; optional off does not stop required diagnostics. [WI11] | Mac Analytics according to Apple requires consent; sharing with developers is separate. Can be disabled later; this does not establish that every Apple service processes no data. [WI24] |
| 21 | Location | Managed through Privacy & security → Location; precise OOBE default/panel variant insufficiently documented. [WI12] [WI04] | Location Services is among the final Setup Assistant choices. Exact default not established. [WI20] |
| 22 | Updates during installation | Critical OOBE patches downloaded after internet connection; a newer OS version may be offered. Not all updates therefore form one mandatory block. [WI10] | Recovery version may lag; Software Update afterwards. Initial setup may require a mandatory update. [WI17] [WI23] [WI29] |
| 23 | Drivers/firmware | Critical drivers during OOBE; check additional manufacturer drivers/Surface firmware separately. No universal OEM set guaranteed. [WI04] [WI02] | No separate driver-selection step described in this Recovery route. Firmware sequence per model insufficiently documented here. [WI17] [WI19] |
| 24 | Optional software | OneDrive, 365 offers/OEM registration may appear; guide describes skipping phone options/Game Pass. Depends on account/rollout. [WI04] [WI05] | Migration/as-new; Siri, Screen Time and Apple Intelligence, among others, during completion. No general package-selection screen established. [WI20] |
| 25 | Summary before installation | Data-loss warning → partition deletion → Ready to install. [WI03] [ANALYSIS] Deletion is already destructive. | Erasure confirms target name/format/volume group and Erase Mac; no combined final summary of all later account choices established. [WI27] |
| 26 | Progress | Installation/restarts described; precise indicator/ETA not documented. [WI03] | Progress bar, restarts and temporary black screen possible; no measured duration. [WI17] |
| 27 | Reboot | During installation and possibly OOBE; pages may recur after loss of network. [WI03] [WI04] | Possible after erasure/reactivation and during reinstallation. [WI27] [WI17] |
| 28 | Remove medium | Internal disk must boot; restoring changed boot order prevents Setup restarting. No universal removal prompt established. [WI02] | N/A for built-in Recovery: no USB installation medium in this route. [WI19] |
| 29 | First boot | Configuration wizard, not yet desktop. [WI03] | Setup Assistant may follow an empty installation; an installation preserving data does not necessarily repeat onboarding. [WI17] |
| 30 | Second configuration phase | OOBE: region/network/account/Hello/privacy/services; content and number of steps vary. [WI04] | Setup Assistant configures account, migration and features; MDM may skip panels or show additional management. [WI20] [WI23] |
| 31 | First actual desktop | After OOBE; documented initial quiet period restricts automatically appearing app windows, with Start as an exception. [WI04] | After the final Welcome, the Mac is ready for use; working apps, recovery and devices not tested here. [WI20] |

## Actual route sequence and variants

### Windows: clean USB installation

1. Preparation: backup, appropriate edition/licence, hardware check; create USB and boot from it. [WI02] [WI06]
2. **Setup:** language → keyboard → installation choice/data-loss confirmation → product key and edition if necessary → licence → target partitions → Ready to install → installation/restarts. An embedded product key may skip the key question. [WI03]
3. **OOBE:** language/region/layout → network/critical updates → applicable licence/update pages → account/recovery → Hello → privacy → applicable services/offers → desktop. This is Microsoft's typical, non-exhaustive sequence; device name and additional options come from the less release-specific guide. [WI04] [WI05]

**Upgrade/in-place:** start `setup.exe` from existing Windows and confirm UAC; choose to retain apps/files/settings, files only or nothing. Installation/restarts follow. This is a different route from booting USB and erasing target partitions. Installation Assistant is an additional x64 upgrade route. [WI03] [WI02]

**OEM/organisation:** a preinstalled PC starts at OOBE; corporate sign-in/management may change the account and consent route. No generic offline bypass or organisation procedure is adopted as the standard consumer route. [WI05] [WI06]

### macOS: Apple silicon Recovery

1. Shut down → hold power → Options/Continue → optionally unlock existing volume/administrator → Recovery. [WI18] [WI19]
2. **Retention route:** directly Reinstall macOS. **Erasure route:** first Disk Utility and confirm target volume/volume group. Apple's erasure route removes data, disconnects Bluetooth and may request Apple Account verification. Then restart → language → internet activation → return to Recovery. [WI17] [WI27]
3. Reinstall macOS → target volume → installation with progress/restarts. Recovery determines the version offered; update later if necessary. [WI17]
4. On an empty/new system: **Setup Assistant** begins with language; covers migration/as-new, accessibility, connection and computer account, Apple Account and final preferences. The guide does not specify a complete fixed order among all panels. Welcome completes it. [WI20]

**Intel:** Command-R for Recovery of the last installed version; Option-Command-R may offer the latest compatible macOS; Shift-Option-Command-R the original or closest available version. Disk Utility erasure has a separate Intel route, with recommended format and volume group; do not blindly apply the Apple silicon procedure. [WI17] [WI18] [WI28]

**Upgrade without erasure:** System Settings → General → Software Update → compatible update/upgrade → administrator password → progress/restarts. This does not require a new Setup Assistant or creation of a new account. [WI29]

**Dual boot:** an additional macOS volume requires separate installation and initial account setup; each installation has its own updates. Windows on Intel through Boot Camp and alternative OS installation on Apple silicon are outside the route verified here. [WI26]

## Consequential decisions and evidence boundaries

- **Encryption/recovery:** Windows Device Encryption depends on hardware and account; according to Microsoft, automatic activation does not occur with a local account. Recovery information belongs to the linked account. Apple describes FileVault as enabled by default **from macOS 26.4**, with an opt-out, a 24-character recovery key and storage in Keychain/Passwords. With iCloud Keychain it synchronises; without iCloud the displayed key must be stored safely. On Apple silicon/T2, storage remains encrypted without FileVault, but without the same password binding. These are different recovery and trust models; exact panel defaults still require checking per test device. [WI07] [WI25]
- **Administrator rights:** administrator membership in Windows does not mean that every app runs elevated: UAC distinguishes ordinary execution from elevation. Mac also creates a first administrator; that choice is not equivalent to consent for every cloud service. [WI08] [WI22]
- **[ANALYSIS] Destructive checkpoint:** a summary after partition deletion cannot undo the earlier deletion. For a later Baryon OS specification, the relevant test point is therefore before the first storage mutation, not only before copying the OS. No product decision or implementation in this dossier.
- **[ANALYSIS] Still required:** bounded VM installations with pinned media/build, screen recording and keyboard/screen-reader testing before erasure. In particular, macOS region/layout/hostname, current Windows location defaults, firmware failures and offline network recovery remain open. Vendor documentation establishes neither problem frequency nor task duration or usability for real participants.

## Source register

All sources below are **primary vendor documentation**, accessed **2026-11-02**. “No date” means no publication date established, not a claim that the page is new. Dynamic pages are not a frozen installation record. Some Apple URLs intermittently returned HTTP 429; only actually readable page content/PDF passages substantiate claims. Search excerpts and external copies were not used as independent evidence.

| ID | Title/organisation and original date or version | Passage read; limitation |
|---|---|---|
| WI01 | [Windows 11 release information][WI01] — Microsoft; current release table, 26H2 GA 2026-09-29 | Support and 26H1 exception; no ISO identification. |
| WI02 | [Download Windows 11][WI02] — Microsoft; 2026 Update/26H2, no page date | Assistant, media and boot order; some generic Setup terms. |
| WI03 | [Reinstall Windows with the installation media][WI03] — Microsoft; Windows 11, no date | Clean/in-place, erasure and Ready to install; no pinned media build. |
| WI04 | [Customize the out-of-box experience][WI04] — Microsoft; updated 2026-03-13 | Typical OOBE, restarts, quiet period; explicitly non-exhaustive. |
| WI05 | [Use a screen reader to set up Windows][WI05] — Microsoft; no date/release pin | Keyboard, name/PIN/extras; mixed older screens and offline route. |
| WI06 | [Windows 11 specifications][WI06] — Microsoft; current system requirements, no date | Hardware, internet/account, Hello; not a complete installation script. |
| WI07 | [Device Encryption in Windows][WI07] — Microsoft; Windows 11/10, no date | Automatic activation and key storage; suitable hardware required. |
| WI08 | [How User Account Control works][WI08] — Microsoft; 2026-04-23 | Tokens/elevation; security mechanism, not OOBE sequence. |
| WI09 | [Local accounts][WI09] — Microsoft; 2026-04-15 | Setup account and built-in Administrator; organisation variants separate. |
| WI10 | [Windows updates during OOBE in Windows 11][WI10] — Microsoft; 2026-03-13 | Critical patches/version offer; no observation of current rollout. |
| WI11 | [Diagnostics, feedback, and privacy in Windows][WI11] — Microsoft; Windows 11/10, no date | Required/Optional; no current panel defaults. |
| WI12 | [Windows location service and privacy][WI12] — Microsoft; Windows 11 section, no date | Time-zone/location management; no exact OOBE screen. |
| WI13 | [BCDBoot Command-Line Options][WI13] — Microsoft; Windows 11 documentation, no date established | Multiple Windows installations; no Linux dual-boot guarantee. |
| WI14 | [Find out which macOS your Mac is using][WI14] — Apple; 2026-10-26 | Latest version numbers; no future support contract. |
| WI15 | [Apple security releases][WI15] — Apple; release entries 2026-10-26 | 27.2/26.7.2/15.8.2; dynamic list. |
| WI16 | [macOS Tahoe 26 is compatible with these computers][WI16] — Apple; version 26, no date established | Supported models; no peripheral assessment. |
| WI17 | [How to reinstall macOS][WI17] — Apple; 2026-09-14 | Retention, version selection, target volume/progress; not full Setup Assistant. |
| WI18 | [Mac startup key combinations][WI18] — Apple; 2025-03-13 | Silicon/Intel startup methods; keyboard/firmware may differ. |
| WI19 | [Use macOS Recovery on a Mac with Apple silicon][WI19] — Apple; version 27 | Starting Recovery/utilities; no demonstrated complete assistive route. |
| WI20 | [Set up your MacBook Air][WI20] — Apple; 2026/macOS 27 | Setup Assistant overview; no complete fixed panel sequence. |
| WI21 | [Set up your accounts][WI21] — Apple; 2026/macOS 27 | Adding Apple Account later; no promise of a completely offline installation. |
| WI22 | [Add a user or group on Mac][WI22] — Apple; macOS 27 | First user administrator; later account management. |
| WI23 | [Manage Setup Assistant for Apple devices][WI23] — Apple; 2026-09-17 | Panel list, MDM/internet; administration document, not a consumer default. |
| WI24 | [Share analytics information from your Mac with Apple][WI24] — Apple; macOS 27 | Consent and disabling; vendor statement about data. |
| WI25 | [Apple Platform Security][WI25] — Apple; August 2026, pp. 91, 104–106 | FileVault/default/recovery; technical description, not a screen record. |
| WI26 | [Use more than one version of macOS on Mac][WI26] — Apple; 2026-09-15 | APFS/external volume and old-OS limitation; macOS only. |
| WI27 | [Use Disk Utility to erase a Mac with Apple silicon][WI27] — Apple; 2026-09-14 | Erasure/volume group/reactivation; not an installation preserving data. |
| WI28 | [Use Disk Utility to erase an Intel-based Mac][WI28] — Apple; 2025-12-08 | Intel erasure route; refers applicable resets to Erase Assistant. |
| WI29 | [Update macOS on Mac][WI29] — Apple; 2026-09-14 | Compatible upgrade, administrator password/restarts; not a new empty installation. |

[WI01]: https://learn.microsoft.com/en-us/windows/release-health/windows11-release-information
[WI02]: https://www.microsoft.com/en-us/software-download/windows11
[WI03]: https://support.microsoft.com/en-us/windows/deployment/install-upgrade/reinstall-windows-with-the-installation-media
[WI04]: https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/customize-oobe-in-windows-11
[WI05]: https://support.microsoft.com/en-us/accessibility/windows/use-a-screen-reader-to-set-up-windows
[WI06]: https://www.microsoft.com/en-us/windows/windows-11-specifications
[WI07]: https://support.microsoft.com/en-us/windows/security/encryption/device-encryption-in-windows
[WI08]: https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/how-it-works
[WI09]: https://learn.microsoft.com/en-us/windows/security/identity-protection/access-control/local-accounts
[WI10]: https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/windows-updates-during-oobe-in-windows-11
[WI11]: https://support.microsoft.com/en-us/windows/privacy/diagnostics-feedback-and-privacy-in-windows
[WI12]: https://support.microsoft.com/en-us/windows/privacy/windows-location-service-and-privacy
[WI13]: https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/bcdboot-command-line-options-techref-di?view=windows-11
[WI14]: https://support.apple.com/en-us/109033
[WI15]: https://support.apple.com/en-us/100100
[WI16]: https://support.apple.com/en-us/122867
[WI17]: https://support.apple.com/en-us/102655
[WI18]: https://support.apple.com/en-us/102603
[WI19]: https://support.apple.com/guide/mac-help/macos-recovery-a-mac-apple-silicon-mchl82829c17/27/mac/27
[WI20]: https://support.apple.com/en-euro/guide/macbook-air/apd831707cb3/2026/mac/27
[WI21]: https://support.apple.com/guide/macbook-air/set-up-your-accounts-apd1c52f9d30/2026/mac/27
[WI22]: https://support.apple.com/guide/mac-help/add-a-user-or-group-mchl3e281fc9/mac
[WI23]: https://support.apple.com/en-euro/guide/deployment/depdeff4a547/web
[WI24]: https://support.apple.com/en-euro/guide/mac-help/mh27990/27/mac/27
[WI25]: https://help.apple.com/pdf/security/en_US/apple-platform-security-guide.pdf
[WI26]: https://support.apple.com/en-ie/118282
[WI27]: https://support.apple.com/en-us/102506
[WI28]: https://support.apple.com/en-us/102639
[WI29]: https://support.apple.com/en-us/108382
