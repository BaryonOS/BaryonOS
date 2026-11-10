# Complete OS installation journey — comparison matrix

Research date: **2 November 2026**. Phases 4.1/4.2. Status: official documentation and version-specific source code read; **no installations, user measurements or project screenshots performed**. The matrix compares decisions, not one assumed screen sequence.

## Routes, versions and evidence

| Platform | Selected route | Scope |
|---|---|---|
| Windows | Windows 11 **26H2**, Home/Pro personal, clean x64 USB installation | Current general download; no exact media build/hash. Upgrade from Windows and OEM/corporate OOBE described separately. [WI01] [WI02] |
| macOS | **27.2 Golden Gate**, Apple silicon, Recovery and Setup Assistant with an empty/new user environment | Released 26 October 2026; Recovery does not guarantee the latest patch. Intel/Tahoe 26.7.2 and installation preserving data are separate variants. [WI14] [WI15] [WI17] |
| Ubuntu | Desktop **26.04.1 LTS**, GNOME 50, interactive | Deliberately the current LTS, not interim 26.10. [LI01] |
| Fedora | Workstation **44**, Live ISO, GNOME 50 | Current stable download; Anaconda 44.30/WebUI 68 as package source. No F45/46 features adopted. [LI06] [LI07] [LI08] [LI09] |
| KDE distribution | **Kubuntu 26.04.1 LTS**, Plasma 6.6, stock Calamares | Image manifest and configuration/patches read; no generic KDE installer assumed. [LI26] [LI27] |
| Baryon OS | Main source `e08fb0e` and separately read native source `1f0dd9c` | Independent UEFI/QEMU boot exists; **no complete installer, physical installation or personal first login demonstrated**. Column **P** is a future proposal, not a working function. [Repository baseline](2026-11-02/repository-baseline.md) |

Complete source passages, original publication/access dates, 31-row registers and variant details: [Windows/macOS](2026-11-02/journey/windows-macos-installation.md) and [Linux installers](2026-11-02/journey/linux-installation.md). Those dossiers are authoritative where cells below are abbreviated. `Not established` means insufficiently precise evidence, not absence of a function. IDs 01–31 are dimensions; screens or required clicks are not being counted.

## OS installation flow matrix

**Source columns:** [SOURCE]. **Baryon OS P:** [PROPOSAL]; current absence is recorded above. Ubuntu/Fedora/Kubuntu are not available implementation foundations for the independent kernel.

| ID | Aspect | Windows | macOS | Ubuntu | Fedora | Kubuntu / KDE | Baryon OS |
|---|---|---|---|---|---|---|---|
| 01 | Boot medium/recovery | USB/firmware menu [WI02] | Power → Options; Intel Command-R [WI18] | USB/firmware [LI02] | Live USB/Media Writer [LI06] | Desktop live image [LI26] | Currently UEFI/QEMU; P: verified medium, no host disk |
| 02 | First screen | Setup language [WI03] | Recovery/unlocking/utilities [WI19] | Installer opens [LI02] | Live environment → WebUI language [LI12] | Dedicated prompt → Calamares [LI28] | P: welcome with language/assistive technology |
| 03 | Language | Setup; possibly again in OOBE [WI03] [WI04] | Setup Assistant; reactivation after erasure [WI20] [WI27] | First choice [LI02] | Installer and first start [LI12] [LI19] | Prompt and welcome [LI28] [LI27] | P: select once, changeable later |
| 04 | Region/time zone | Region in OOBE; no fixed time-zone page established [WI04] | Exact screen/position not established [WI20] | After account, map/city [LI03] | First start, not installer [LI19] | Locale before keyboard; GeoIP [LI27] | P: suggestion among local preferences, do not infer location consent |
| 05 | Keyboard | Setup and OOBE [WI03] [WI05] | Exact page insufficiently documented [WI20] | Before network [LI02] | Live settings; again on first start [LI19] | After locale [LI27] | P: test before password/unlocking |
| 06 | Accessibility | Keyboard described; pre-erasure screen reader unconfirmed [WI05] | Setup Assistant; Recovery coverage unconfirmed [WI20] | Dedicated early step [LI02] | No separate wizard page; operation untested [LI12] | No separate module; operation untested [LI27] | P: from welcome and during errors, not only after installation |
| 07 | Network | Internet required for personal OOBE [WI06] | Required for Recovery download/activation [WI17] [WI27] | Optional [LI02] | Live environment; first-start page [LI19] | Prompt; online/offline differences [LI28] | P: complete local core medium works offline; downloads separate |
| 08 | Try/Install | No live Try in selected route [WI03] | Recovery is not a live desktop [WI19] | Try or Install after network [LI02] | Live trial possible [LI06] | Dedicated Try/Install prompt [LI28] | P: try without disk mutation, installation explicit |
| 09 | Hardware check | Requirements; Assistant check not established as identical to USB [WI02] | Model compatibility beforehand [WI17] | Live trial; no proprietary drivers [LI02] | Storage/space check [LI15] | RAM/storage/root check; no hardware guarantee [LI27] | P: tested capabilities before disk selection, honest blockers |
| 10 | Target storage | Disk/partition [WI03] | Target volume, possibly unlock [WI17] | Target disk [LI03] | One/multiple disks [LI13] | Partition module [LI27] | P: user selects identified target; unknown ≠ empty |
| 11 | Partitioning | Clean route removes partitions [WI03] | Disk Utility separate; APFS [WI27] | Automatic/manual [LI03] | Entire disk/free space/mount points/editor [LI13] | Erase/replace/alongside/manual conditionally [LI27] [LI32] [LI33] | P: understandable consequences; technical layout in Advanced |
| 12 | Retain/replace | Clean erases; in-place retention options [WI03] | Reinstall preserves; erasure separate [WI17] | Erase or alongside [LI03] | Free space preserves; reinstall conditional [LI13] | Replace partition or entire disk [LI32] [LI33] | P: options according to demonstrated capability, source preservation explicit |
| 13 | Dual boot | Not guided in this clean route [WI03] | Additional macOS/APFS or external volume [WI26] | Alongside; BitLocker may block [LI03] | Free space; no universal boot guarantee [LI13] | Suitable shrinkable partition required [LI32] [LI33] | P: validated combination only, otherwise block |
| 14 | Encryption | Device Encryption on suitable device/account [WI07] | FileVault; hardware encryption is a different guarantee [WI25] | Passphrase/LVM or suitable TPM route [LI05] | New partitions with LUKS/passphrase [LI16] | Automatic LUKS2 layout, boot separate [LI27] | P: real personal protection and recovery before commit; gate open |
| 15 | User account | OOBE [WI04] | Setup Assistant local computer account [WI20] | Before summary [LI03] | First start [LI19] | Before summary [LI27] | P: local personal account; no current guest-login claim |
| 16 | Device name | OOBE guide; position not pinned [WI05] | No separate page established [WI20] | With account [LI03] | No review field in Live route [LI15] | Users page [LI27] | P: non-identifying editable name, no additional mandatory page |
| 17 | Password/PIN/biometrics | Account → Hello/PIN; biometrics conditional [WI05] | Computer password; Touch ID conditional [WI20] | Login password ≠ disk PIN [LI05] | First-start password [LI19] | Password; autologin off by default [LI27] | P: existing real methods, distinguish login from storage unlocking |
| 18 | Online/local | Personal Home/Pro requires MS account [WI06] | Apple Account possible later [WI21] | Local [LI03] | Local; enterprise variant [LI19] | Local [LI27] | P: local core; providers optional later |
| 19 | Privacy | OOBE choices vary [WI04] | Several separate panels [WI20] | Additional first start [LI03] | First start, conditional [LI19] | No separate privacy wizard established [LI27] | P: short understandable review; fixed Security Floor |
| 20 | Telemetry | Required/Optional distinguished [WI11] | Analytics with consent [WI24] | Device information opt-in [LI03] | Reporting page conditional; upstream enabled [LI20] | No page does not establish absence [LI27] | P: no OS telemetry according to existing contract |
| 21 | Location | Privacy & security; default unknown [WI12] | Location Services, default unknown [WI20] | First start [LI03] | Upstream switch on; provider unknown [LI20] | GeoIP time zone configured [LI27] | P: no silent location/GeoIP request; manual time zone possible |
| 22 | Installation updates | Critical OOBE patches; version offer [WI10] | Recovery may be older; update afterwards [WI17] | Online updates and updater afterwards [LI03] | No choice in Live wizard [LI12] | Optional, not offline [LI27] | P: verified payload; distinguish downloads from personal data |
| 23 | Drivers/firmware | Critical drivers; manufacturer separate [WI04] | No separate driver choice described [WI17] | Third-party choice [LI03] | External repositories on first start [LI19] | Visible driver/codec choice unconfirmed [LI27] | P: supported profile; no silent firmware action |
| 24 | Optional software | Services/OEM offers conditional [WI04] | Migration, Siri, AI etc. conditional [WI20] | Default/extended [LI03] | Live payload; repositories later [LI19] | Minimal/Normal/Full and online extras [LI27] | P: core set; additions after desktop, no sales funnel |
| 25 | Summary/write boundary | Ready to install after earlier partition actions [WI03] | Erasure confirmed separately, no complete summary [WI27] | Ready to install → Install [LI03] | Review; Cockpit writes directly without undo [LI29] | Summary/confirmation → execution [LI27] | P: complete change plan before every disk mutation |
| 26 | Progress | Installation/restarts; exact ETA unknown [WI03] | Bar/restarts/black screen [WI17] | Slideshow/details [LI03] | Four phases/status/errors [LI17] | Execution modules; indicator unknown [LI27] | P: real phase, honest uncertainty and cancellation boundary |
| 27 | Restart | During Setup/OOBE [WI04] | Possibly several [WI17] | Restart now [LI03] | Exit to live, then manual reboot [LI17] | Choice at finished [LI27] | P: explicit completion state, no invented success |
| 28 | Remove medium | Check boot order; prompt not established [WI02] | Not applicable to built-in Recovery [WI19] | Remove USB when prompted [LI03] | Exact prompt not established [LI17] | Exact prompt not established [LI27] | P: identify medium clearly and prevent booting back into it |
| 29 | First boot | OOBE [WI03] | Empty installation → Setup Assistant [WI17] | Unlock/login where applicable [LI03] | GNOME Initial Setup [LI19] | Login route, untested [LI27] | P: verify real installation/account state |
| 30 | Second phase | OOBE with account/privacy/services [WI04] | Setup Assistant; management may vary [WI23] | Initial Setup after desktop (31) [LI03] | Account/privacy/repositories etc. conditionally [LI19] | Plasma Welcome except minimal; details unknown [LI27] | P: necessary completion only; skippable guidance |
| 31 | Actual desktop | After OOBE [WI04] | After Welcome [WI20] | GNOME 50 [LI01] | GNOME 50 [LI06] | Plasma 6.6 [LI26] | Currently limited native session; P: demonstrated day-one tasks |

## Actual sequences side by side

- **Windows:** USB → language/keyboard → installation/data-loss choice/licence → target partitions, where deletion already writes destructively → ready → OS installation/restarts → OOBE with network/account/privacy/services → desktop. An upgrade from Windows has different retention options. [WI03] [WI04]
- **macOS:** Recovery/unlocking → optional separate erasure/reactivation → reinstall/target volume → progress/restarts → Setup Assistant for an empty environment → desktop. An installation preserving data does not require a new account phase. The exact complete panel sequence remains insufficiently documented. [WI17] [WI20] [WI27]
- **Ubuntu:** language → accessibility → keyboard → network → Try/Install → interactive installation/apps/drivers → storage → account → time zone → summary → writing → reboot/medium → unlock/login → desktop → additional Initial Setup. [LI02] [LI03]
- **Fedora Workstation:** live → language → installation method/targets → storage, where the optional Cockpit editor writes directly without undo → review → installer execution → return to live → restart → GNOME Initial Setup with conditional pages → personal desktop. [LI12] [LI19] [LI29]
- **Kubuntu:** language/network/Try–Install prompt → welcome → locale → keyboard → software choice → partition → users → summary → execution → finished/restart → desktop, optionally Welcome. [LI27] [LI28]

## A/B/C: retention, variation and friction

**A — shared concepts [SOURCE/ANALYSIS]:** language/input, an installation source and target, decisions about existing data, progress, restart/transition and a personal user session recur in several routes. Familiarity is plausible, but user knowledge has not been measured. Not every route asks for a separate account, encryption or network screen choice.

**B — platform-specific [SOURCE]:** Windows product edition/licence/Hello and mandatory personal MS account; Apple Recovery/activation/FileVault/Apple Account; Linux live Try, distribution-specific package/repository choices and partitioning options. The convention itself is not an automatic requirement for Baryon OS. [WI03] [WI06] [WI17] [WI25] [LI02] [LI19] [LI27]

**C — potentially avoidable friction [ANALYSIS, not measured]:** duplicate language/input without clear handover, account/privacy questions across several phases, offers unrelated to installation needs, technical layout terminology on a beginner route, hidden early write boundaries and a network requirement that cannot resolve a missing driver. An additional recovery check can be valuable; fewer screens is not an independent quality objective. Guided choices, defaults and consolidation require testing.

## Terminology and decisions

| Understandable main term (proposal) | Technical explanation only where needed | What must never disappear |
|---|---|---|
| Use this disk | Brand, capacity, recognisable stable characteristic; partition layout in Advanced | Which physical target and data will be affected |
| Replace everything on this disk | Format/recreate partitions | “All files and existing systems on … will be removed” before writing |
| Keep the existing system | Free space/resize/boot manager in details | Which parts actually remain untouched; residual and boot risks |
| Protect data | Distinguish encryption, keys, unlocking and recovery | What losing recovery materials means; no guaranteed undo |
| Personal account on this device | Authentication versus provider linking | Who has access; do not label a public guest as a private account |

**Safe to suggest [PROPOSAL]:** allow language/layout selection or provide an editable suggestion, generate a non-identifying device name, calculate a tested internal layout **after** target/method selection, privacy-preserving defaults and time zone without a location request. Automatic calculation is not authorisation to write. Detection may be uncertain; the UI shows this.

**Never silently automatic [PROPOSAL]:** selecting an erasure target, treating an unknown disk as empty, shrinking/removing/converting partitions, changing boot configuration, weakening encryption, exporting a recovery key, linking providers, modifying firmware or giving decryption keys to a third party. Advanced is subject to the same write boundary. If disk identity, plan or conditions change, confirmation expires.

## Implication for the Baryon OS flow

[DESIGN DECISION] First usable language/accessibility/input and a safe trial option; then real suitability and storage selection; account/protection before one complete write confirmation; after reboot, only necessary verification and optional task help. The sources do not prescribe this precise sequence. The [installer specification](../product/new-os-installer-spec.md) explains, step by step, what is combined, deferred or omitted and which backend gates remain open.

Legal/technical handling: only public text, configuration and source references used, no reproduced OS screenshots or downloaded installation images. Screen descriptions are source-bound summaries; unknown details are not mock images. Source versions may change: record and recheck media/builds before execution.

## Direct sources for the abbreviated matrix

Complete titles, dates and limitations appear in the two source dossiers; every ID below refers to the same original source.

[WI01]: https://learn.microsoft.com/en-us/windows/release-health/windows11-release-information
[WI02]: https://www.microsoft.com/en-us/software-download/windows11
[WI03]: https://support.microsoft.com/en-us/windows/deployment/install-upgrade/reinstall-windows-with-the-installation-media
[WI04]: https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/customize-oobe-in-windows-11
[WI05]: https://support.microsoft.com/en-us/accessibility/windows/use-a-screen-reader-to-set-up-windows
[WI06]: https://www.microsoft.com/en-us/windows/windows-11-specifications
[WI07]: https://support.microsoft.com/en-us/windows/security/encryption/device-encryption-in-windows
[WI10]: https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/windows-updates-during-oobe-in-windows-11
[WI11]: https://support.microsoft.com/en-us/windows/privacy/diagnostics-feedback-and-privacy-in-windows
[WI12]: https://support.microsoft.com/en-us/windows/privacy/windows-location-service-and-privacy
[WI14]: https://support.apple.com/en-us/109033
[WI15]: https://support.apple.com/en-us/100100
[WI17]: https://support.apple.com/en-us/102655
[WI18]: https://support.apple.com/en-us/102603
[WI19]: https://support.apple.com/guide/mac-help/macos-recovery-a-mac-apple-silicon-mchl82829c17/27/mac/27
[WI20]: https://support.apple.com/en-euro/guide/macbook-air/apd831707cb3/2026/mac/27
[WI21]: https://support.apple.com/guide/macbook-air/set-up-your-accounts-apd1c52f9d30/2026/mac/27
[WI23]: https://support.apple.com/en-euro/guide/deployment/depdeff4a547/web
[WI24]: https://support.apple.com/en-euro/guide/mac-help/mh27990/27/mac/27
[WI25]: https://help.apple.com/pdf/security/en_US/apple-platform-security-guide.pdf
[WI26]: https://support.apple.com/en-ie/118282
[WI27]: https://support.apple.com/en-us/102506
[LI01]: https://ubuntu.com/download/desktop
[LI02]: https://ubuntu.com/desktop/docs/en/26.04/tutorial/try-ubuntu-desktop/
[LI03]: https://ubuntu.com/desktop/docs/en/26.04/tutorial/install-ubuntu-desktop/
[LI05]: https://ubuntu.com/desktop/docs/en/26.04/how-to/encrypt-your-disk-with-tpm/
[LI06]: https://fedoraproject.org/workstation/download/
[LI07]: https://fedoramagazine.org/announcing-fedora-linux-44/
[LI08]: https://packages.fedoraproject.org/pkgs/anaconda-webui/anaconda-webui/index.html
[LI09]: https://packages.fedoraproject.org/pkgs/anaconda/anaconda/
[LI12]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/steps.js
[LI13]: https://github.com/rhinstaller/anaconda-webui/blob/68/docs/installation-steps.rst
[LI15]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/review/ReviewConfiguration.jsx
[LI16]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/storage/storage-configuration/DiskEncryption.jsx
[LI17]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/installation/InstallationProgress.jsx
[LI19]: https://github.com/GNOME/gnome-initial-setup/blob/50.1/gnome-initial-setup/gnome-initial-setup.c
[LI20]: https://github.com/GNOME/gnome-initial-setup/blob/50.1/gnome-initial-setup/pages/privacy/gis-privacy-page.c
[LI26]: https://cdimage.ubuntu.com/kubuntu/releases/26.04.1/release/kubuntu-26.04.1-desktop-amd64.manifest
[LI27]: https://archive.ubuntu.com/ubuntu/pool/universe/c/calamares-settings-ubuntu/calamares-settings-ubuntu_26.04.12.tar.xz
[LI28]: https://archive.ubuntu.com/ubuntu/pool/universe/k/kubuntu-installer-prompt/kubuntu-installer-prompt_26.04.3.tar.xz
[LI29]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/storage/cockpit-storage-integration/CockpitStorageIntegration.jsx
[LI32]: https://raw.githubusercontent.com/calamares/calamares/v3.3.14/src/modules/partition/gui/ChoicePage.cpp
[LI33]: https://archive.ubuntu.com/ubuntu/pool/universe/c/calamares/calamares_3.3.14-0ubuntu25.debian.tar.xz
