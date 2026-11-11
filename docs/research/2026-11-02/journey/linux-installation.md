# Installation journey: Ubuntu, Fedora Workstation and Kubuntu

**[SOURCE] The three Linux installers examined distribute storage choices, account creation and first-start settings differently; one generic Linux installation wizard would conceal those differences.** This is a documentation and source-code walkthrough on **2026-11-02**, not a performed installation or user test.

## Scope and source checks

Research question: which decisions does a new user make between boot medium and personal desktop, when do data change irreversibly, and which steps are optional? Search strategy: official downloads → release-specific manuals → package versions/configuration/source code where general manuals are insufficiently precise. Only public text and source packages read; no installation images downloaded, VM started or screenshots produced.

| Route examined | Version and rationale | Version boundary |
|---|---|---|
| Ubuntu Desktop, standard interactive installation | **26.04.1 LTS**, current LTS, GNOME **50**. [LI01] | Ubuntu 26.10 also exists; the supported LTS is deliberately selected here. The guide covers the 26.04 series. [LI01] [LI03] |
| Fedora Workstation, Live ISO | **44**, current stable download, released 2026-04-28; GNOME **50**. [LI06] [LI07] | Package register: Anaconda **44.30**, WebUI **68**, GNOME Initial Setup **50.1**. These are checked F44 package versions, not inspected ISO contents. [LI08] [LI09] [LI10] |
| Stock Kubuntu, ordinary desktop installation | **26.04.1 LTS**, support until April 2029; Plasma **6.6**. Comparable to Ubuntu LTS, with separate Calamares interaction. [LI24] [LI25] | Official image manifest: Calamares **3.3.14-0ubuntu25**, settings **1:26.04.12**, prompt **26.04.3**, Plasma Welcome **6.6.5**. [LI26] |

Upstream GNOME 51/Plasma 6.8 and other newer desktop documentation are not attributed to these distribution installers. Anaconda's `latest` documentation already contains Fedora 45/46 changes; execution code used is therefore version-pinned. [LI11] [LI12] [LI18]

**Reading guide:** every cell is **[SOURCE]** with an LI reference, except where explicitly **unconfirmed**. “No page” means only no page in the checked route; it does not establish absence of a system function. Numbers form the comparison register, **not** the screen sequence. Desktop here means the user's own installed session; a live desktop does not count.

## Comparison register: 31 decision points

| # | Point | Ubuntu 26.04.1 LTS | Fedora Workstation 44 Live | Kubuntu 26.04.1 LTS |
|---|---|---|---|---|
| 1 | Boot medium/recovery | Boot from USB; firmware selection may be needed. [LI02] | Live USB, including through Media Writer; recovery route outside scope. [LI06] | Desktop live image; firmware/recovery details unconfirmed. [LI26] [LI28] |
| 2 | First visible installer | Installer opens automatically. [LI02] | In live environment: WebUI with language as first wizard step; automatic start unconfirmed. [LI06] [LI12] | Dedicated prompt before Calamares. [LI28] |
| 3 | Language | First choice. [LI02] | WebUI language; first-start wizard has its own language page. [LI12] [LI19] | Language in prompt, then Calamares welcome. [LI28] [LI27] |
| 4 | Region/time zone | After account, map/city. [LI03] | Workstation hides installer time zone; first-start wizard includes it. [LI11] [LI19] | Locale step before keyboard; time-zone detection through GeoIP configured. [LI27] |
| 5 | Keyboard | Before network. [LI02] | GNOME settings, multiple layouts supported; language choice does not automatically determine Workstation layout. First start asks again. [LI18] [LI19] | Dedicated keyboard step after locale. [LI27] |
| 6 | Accessibility | Early accessibility step. [LI02] | No separate wizard step in this configuration; actual screen-reader access untested. [LI11] [LI12] | No separate step in prompt/module list; assistive technology and keyboard controls untested. [LI27] [LI28] |
| 7 | Network | Connection optional. [LI02] | Through live environment/additional menu, no F45 network page; first-start wizard has a network step. [LI18] [LI19] | Wi-Fi/Ethernet in prompt; online and offline software choices differ. [LI28] [LI27] |
| 8 | Try versus Install | After network; Try closes installer, desktop shortcut reopens it. [LI02] | Live USB permits trying before installation. [LI06] | Separate Try/Install actions; Try opens the live environment. [LI28] |
| 9 | Hardware check | Live trial tests the user's hardware; proprietary drivers are absent there. [LI02] | Storage/space check; no complete hardware-compatibility guarantee. [LI15] [LI18] | Welcome requires RAM/storage/root checks; internet/mains power not required. No hardware certification. [LI27] |
| 10 | Storage device | Select target disk. [LI03] | One or more target disks at Installation method. [LI13] | Partition module; exact disk selector not checked visually. [LI27] |
| 11 | Partitioning | Automatic or manual. [LI03] | Entire disk, free space, mount-point assignment or advanced editor, depending on situation. [LI13] [LI29] | Erase/replace/alongside/manual conditionally; ext4 default, btrfs/xfs also configured. [LI27] [LI32] [LI33] |
| 12 | Retain/replace existing OS | Erase or alongside existing OS; BitLocker may block. [LI03] | Erase entire disk; free space preserves existing partitions. Reinstall retains home only with a recognisable Fedora layout. [LI13] | Replace partition or erase entire disk; available routes depend on disk state. [LI32] [LI33] |
| 13 | Dual boot | Alongside with space allocation. [LI03] | Use free space where partitions exist; bootability of every combination not established. [LI13] | Alongside requires a suitable shrinkable partition; also an ESP with UEFI. No general dual-boot guarantee. [LI32] [LI33] |
| 14 | Disk encryption | LVM/passphrase or suitable TPM hardware; TPM route displays recovery key after installation. [LI04] [LI05] | LUKS passphrase for new partitions, boot layout visible; no recovery-code/TPM step confirmed. [LI16] | Automatic layout: LUKS2 with unencrypted `/boot`; assess manual layouts separately. No TPM/recovery-code flow confirmed. [LI27] |
| 15 | User account | Name, username, password. [LI03] | Only at first start; local account admin by default, with separate parental-control variant. [LI11] [LI19] [LI22] | Before summary; user in sudo group, separate root password disabled. [LI27] |
| 16 | Device name/hostname | Computer name with account. [LI03] | Hostname field in review only for BOOT_ISO, not this Live route. [LI15] | Editable computer name on user page; stock configuration retains the default. [LI27] [LI31] [LI33] |
| 17 | Password/PIN/biometrics | Login password; TPM disk PIN is separate. [LI03] [LI05] | First-start password; no PIN/biometrics page in checked sequence. [LI19] | Local password; autologin off by default. No PIN/biometrics step; exact quality enforcement untested. [LI27] [LI33] |
| 18 | Online versus local | Local installation account. [LI03] | Local by default; enterprise sign-in a separate network-dependent variant. No mandatory consumer cloud account. [LI21] | Local user module; no online-account step in configuration. [LI27] |
| 19 | Privacy choices | Additional first-start choices. [LI03] | Privacy at first start, with conditional components. [LI19] [LI20] | No privacy wizard; GeoIP time-zone request separately relevant. [LI27] |
| 20 | Telemetry | Device information opt-in, off by default. [LI03] | Technical reporting page depends on ABRT service; upstream switch initially on. Distribution behaviour not executed. [LI20] | No telemetry page in module list; no evidence of zero system telemetry. [LI27] |
| 21 | Location | Location services at first start. [LI03] | Location switch in first-start privacy; upstream initially on. Provider not established. [LI20] | Time-zone GeoIP configured to `geoip.ubuntu.com/lookup`; no separate consent page found. [LI27] |
| 22 | Updates during installation | Network supports updates; updater after installation. [LI03] | No update-choice step in checked Live wizard. [LI12] [LI14] | Optional; execution after unpacking within installation task. Choice hidden offline. [LI27] |
| 23 | Drivers/firmware | Choice of third-party drivers/media support. [LI03] | First-start choice of external repositories may provide apps/drivers; no firmware wizard established. [LI23] | Current visible driver/codec checkbox unconfirmed; configuration action does not establish UI. [LI27] |
| 24 | Optional software | Default or extended apps. [LI03] | Live image has no DNF software-selection page; optional external repositories later. [LI14] [LI23] | Minimal/Normal/Full; additional online apps. Full/extras hidden offline. [LI27] |
| 25 | Summary/write boundary | Ready to install → Install. [LI03] | Review with confirmation for erasure/modification. **Exception:** Cockpit editor writes immediately, without undo. [LI15] [LI29] | Summary + confirmation prompt; then partition as the first execution module. [LI27] |
| 26 | Progress | Slideshow and details. [LI03] | Four phases plus status/error message. [LI17] | Execution modules through finished; exact animation/ETA unconfirmed. [LI27] |
| 27 | Restart | Restart now. [LI03] | Exit to live desktop; then restart manually. Not the F45 reboot button. [LI17] [LI18] | Restart selectable at finished. [LI27] |
| 28 | Remove medium | Remove USB when prompted, Enter. [LI03] | Exact removal time/prompt unconfirmed in checked UI. [LI17] | Exact removal time/prompt unconfirmed. [LI27] |
| 29 | First boot | Disk unlock if applicable, login. [LI03] [LI05] | GNOME Initial Setup before the personal user session. [LI12] [LI19] | Login/autologin choice follows installation configuration; exact initial screens untested. [LI27] |
| 30 | Second phase/OOBE | Initial Setup with additional preferences after reaching the desktop (31). [LI03] | Yes: language/keyboard/network/privacy/time zone/repositories/account/password; conditional steps. [LI19] [LI20] [LI21] [LI22] [LI23] | Account already created. Plasma Welcome present except minimal; questions unconfirmed. OEM is a separate route. [LI26] [LI27] |
| 31 | First actual desktop | Installed GNOME 50 session. [LI01] [LI03] | GNOME 50 after first-start completion. [LI07] [LI19] | Plasma 6.6; actually reaching the desktop not tested. [LI25] [LI27] |

## Actual sequence and variant boundaries

- **Ubuntu:** boot → language → accessibility → keyboard → network → Try/Install. [LI02] Then by register numbers: interactive → **24/23 → 10–14 → 15–18 → 4 → 25 → 26 → 27/28 → 29 → 31 → 30**. [LI03] The desktop precedes the additional Initial Setup app; the table groups aspects, not one universal screen sequence. Hardware-backed encryption adds a suitability check, possible firmware action and recovery key. [LI05]
- **Fedora Workstation Live:** live environment → WebUI language → installation method/target disks → storage configuration or mount-point assignment → review → execution → return to live environment → restart → GNOME Initial Setup → personal desktop. Accounts and time zone are deliberately hidden from the installer; software selection is for DNF payloads, not this Live route. [LI11] [LI12] [LI13] [LI14] [LI17] The checked GNOME page table is language → keyboard → network → privacy → time zone → repositories → account → password → optional parental controls → completion. Distribution settings may skip pages. [LI19]
- **Kubuntu:** prompt with language/network/Try–Install → Calamares **welcome → locale → keyboard → pkgselect → partition → users → summary → execution → finished**. The exact order among choices on the first prompt screen is not fixed. [LI27] [LI28] Minimal removes Plasma Welcome; OEM later follows different user configuration. These are not mandatory steps of ordinary installation. [LI27]

**[ANALYSIS] Three relevant design questions for Baryon OS:** (1) can users try the keyboard, network and assistive technology before installation; (2) is every destructive route recognisable, including exceptional advanced editors; (3) is it clear what occurs before writing, after restart or optionally later? The sources establish existing mechanisms, not lower error probability or user preference. **[HYPOTHESIS]** A continuous progress list with separate storage confirmation may make these transitions more understandable; this requires user tests.

**Limitations:** no visual/keyboard/screen-reader observations, installation duration, error frequencies or end-to-end success claims. A source module may be conditionally hidden; such conditions are included above where established. Fedora package registers may be newer than the original ISO. A recent Framework F44 guide was checked but not used as route evidence: it still contains Ubuntu boot text, old installer terminology and an outdated location provider. [LI30] Linux installers are comparison material; their drivers, package managers and encryption are not automatically available implementations for the independent Baryon OS kernel.

## Source register

All sources accessed **2026-11-02**. Date = visible publication/update, otherwise unknown. Source code is implementation evidence for the stated version, not runtime validation. IDs with multiple files below refer to separately verifiable primary passages.

| ID | Title, organisation; date/version | Checked passage and limitation |
|---|---|---|
| [LI01] | Download Ubuntu Desktop, Canonical; unknown, 26.04.1 | LTS download and GNOME 50; also a separate 26.10 section. Live download page. |
| [LI02] | Try Ubuntu Desktop, Canonical; 2026-10-13, 26.04 | Boot, early choices, returning from Try and open drivers. Not a complete hardware test. |
| [LI03] | Install Ubuntu Desktop, Canonical; 2026-10-15, 26.04 | Interactive route, disk/account, summary, restart and Initial Setup. No project execution. |
| [LI04] | Advanced disk setup features, Canonical; 2026-10-13, 26.04 | LVM/passphrase, hardware-backed and advanced options. Hardware conditions differ. |
| [LI05] | Encrypt your disk with TPM, Canonical; 2026-10-13, 26.04 | Suitability, entire disk, recovery key, later disk PIN. No login-PIN evidence. |
| [LI06] | Download Fedora Workstation, Fedora Project; 2026-04-28 release, 44 | Current stable Live download and Media Writer/Try. No installation screens. |
| [LI07] | Announcing Fedora Linux 44, Jef Spaleta/Fedora Magazine; 2026-04-28 | Workstation GNOME 50; KDE variant separate. No current F45 release claim. |
| [LI08] | anaconda-webui, Fedora Packages; unknown | F44 stable 68-1.fc44. Package register, not ISO manifest. |
| [LI09] | anaconda, Fedora Packages; unknown | F44 stable 44.30-2.fc44. Same limitation. |
| [LI10] | gnome-initial-setup, Fedora Packages; unknown | F44 stable 50.1-1.fc44. Same limitation. |
| [LI11] | fedora-workstation.conf, Anaconda/Red Hat; tag anaconda-44.30, date unknown | `hidden_webui_pages`: accounts/date-time. Workstation profile only. |
| [LI12] | steps.js, Anaconda WebUI/Red Hat; tag 68, date unknown | Step list and filters; comment explains account deferral. Combine with profile. |
| [LI13] | Installation Flow Overview, Anaconda WebUI; tag 68, date unknown | Storage scenarios and Reinstall/Free space conditions. Do not blindly apply the generic list to Workstation. |
| [LI14] | software/index.js, Anaconda WebUI; tag 68, date unknown | Page hidden when payload is not DNF. No inspected ISO payload. |
| [LI15] | ReviewConfiguration.jsx, Anaconda WebUI; tag 68, date unknown | Write confirmation; hostname only `isBootIso`. No evidence that every editor defers writes. |
| [LI16] | DiskEncryption.jsx, Anaconda WebUI; tag 68, date unknown | LUKS passphrase, boot keyboard, new partitions only. No recovery procedure. |
| [LI17] | InstallationProgress.jsx, Anaconda WebUI; tag 68, date unknown | Four phases, success/error and Live exit versus Boot ISO reboot. Outcome not executed. |
| [LI18] | Anaconda release notes, Anaconda team; live documentation, date unknown | F44 WebUI keyboard; F45 network page/reboot as differences. F45/46 are not F44 features. |
| [LI19] | gnome-initial-setup.c, GNOME/Red Hat; tag 50.1, date unknown | `page_table`, vendor skip, conditional parental controls. Upstream may differ from distribution patches. |
| [LI20] | gis-privacy-page.c, GNOME/Red Hat; tag 50.1, date unknown | Location/reporting switch initialisation, ABRT dependency. No claim about provider or telemetry scope. |
| [LI21] | gis-account-page.c, GNOME/Red Hat; tag 50.1, date unknown | Local default, enterprise/network variant. No mandatory consumer account. |
| [LI22] | gis-account-page-local.c, GNOME/Red Hat; tag 50.1, date unknown | Admin default and parental-control exception. Build conditions remain relevant. |
| [LI23] | gis-software-page.c, GNOME/Red Hat; tag 50.1, date unknown | Only with `fedora-third-party`; optional repositories. Not choosing may cause a repeated question later. |
| [LI24] | Kubuntu downloads, Kubuntu; unknown, 26.04.1 | Current LTS and support through April 2029. 26.10 separate interim release. |
| [LI25] | Kubuntu 26.04 release notes, Kubuntu; 2026-04-23 | Plasma 6.6, Qt 6.10.2; not a complete installation walkthrough. |
| [LI26] | kubuntu-26.04.1-desktop-amd64.manifest, Ubuntu/Kubuntu; date unknown | Exact image package versions. Text manifest, ISO not downloaded. |
| [LI27] | calamares-settings-ubuntu_26.04.12, Kubuntu/Lubuntu; changelog Aaron Rainbolt, 2026-04-21 | `kubuntu/settings.conf`, modules welcome/partition/users/pkgselect_context; common locale/finished/pkgselect. Source archive read in memory; UI presence not inferred from an isolated action. |
| [LI28] | kubuntu-installer-prompt_26.04.3, Kubuntu; changelog Aaron Rainbolt, 2026-08-20 | `installerprompt.ui/.cpp`: language, network, Try/Install, return to live. No evidence about subsequent Calamares screens. |
| [LI29] | CockpitStorageIntegration.jsx, Anaconda WebUI; tag 68, date unknown | Explicit warnings: immediate changes, no undo. Advanced side route. |
| [LI30] | Fedora 44 Workstation Installation on Framework Laptop 16, Matt Hartley et al./Framework; 2026-09-30 | Steps 6–8 contain internal inconsistencies; page states unpublished status. Source criticism only, not a normative route. |
| [LI31] | page_usersetup.ui, Calamares; v3.3.14, date unknown | `textBoxHostname`; visibility in [UsersPage.cpp](https://raw.githubusercontent.com/calamares/calamares/v3.3.14/src/modules/users/UsersPage.cpp) and `EtcHostname` default in [users/Config.cpp](https://raw.githubusercontent.com/calamares/calamares/v3.3.14/src/modules/users/Config.cpp). Combine with Kubuntu configuration and patches. |
| [LI32] | ChoicePage.cpp, Calamares; v3.3.14, date unknown | `setupActions`: shrinkability/replaceability, ESP, table type; [partition/Config.cpp](https://raw.githubusercontent.com/calamares/calamares/v3.3.14/src/modules/partition/Config.cpp) has a manual default. Four routes are not always visible. |
| [LI33] | calamares_3.3.14-0ubuntu25.debian.tar.xz, Ubuntu/Kubuntu; archive date 2026-03-18 | Patch series does not change the named hostname/choice logic; `fix-checkbox-warning-confusion.patch` changes password logic. Therefore no enforcement claim solely from configuration keys. |

[LI01]: https://ubuntu.com/download/desktop
[LI02]: https://ubuntu.com/desktop/docs/en/26.04/tutorial/try-ubuntu-desktop/
[LI03]: https://ubuntu.com/desktop/docs/en/26.04/tutorial/install-ubuntu-desktop/
[LI04]: https://ubuntu.com/desktop/docs/en/26.04/reference/advanced-disk-setup-features/
[LI05]: https://ubuntu.com/desktop/docs/en/26.04/how-to/encrypt-your-disk-with-tpm/
[LI06]: https://fedoraproject.org/workstation/download/
[LI07]: https://fedoramagazine.org/announcing-fedora-linux-44/
[LI08]: https://packages.fedoraproject.org/pkgs/anaconda-webui/anaconda-webui/index.html
[LI09]: https://packages.fedoraproject.org/pkgs/anaconda/anaconda/
[LI10]: https://packages.fedoraproject.org/pkgs/gnome-initial-setup/gnome-initial-setup/
[LI11]: https://github.com/rhinstaller/anaconda/blob/anaconda-44.30/data/profile.d/fedora-workstation.conf
[LI12]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/steps.js
[LI13]: https://github.com/rhinstaller/anaconda-webui/blob/68/docs/installation-steps.rst
[LI14]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/software/index.js
[LI15]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/review/ReviewConfiguration.jsx
[LI16]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/storage/storage-configuration/DiskEncryption.jsx
[LI17]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/installation/InstallationProgress.jsx
[LI18]: https://anaconda-installer.readthedocs.io/en/latest/release-notes/index.html
[LI19]: https://github.com/GNOME/gnome-initial-setup/blob/50.1/gnome-initial-setup/gnome-initial-setup.c
[LI20]: https://github.com/GNOME/gnome-initial-setup/blob/50.1/gnome-initial-setup/pages/privacy/gis-privacy-page.c
[LI21]: https://github.com/GNOME/gnome-initial-setup/blob/50.1/gnome-initial-setup/pages/account/gis-account-page.c
[LI22]: https://github.com/GNOME/gnome-initial-setup/blob/50.1/gnome-initial-setup/pages/account/gis-account-page-local.c
[LI23]: https://github.com/GNOME/gnome-initial-setup/blob/50.1/gnome-initial-setup/pages/software/gis-software-page.c
[LI24]: https://kubuntu.org/download/
[LI25]: https://www.kubuntu.org/news/kubuntu-26-04-release-notes/
[LI26]: https://cdimage.ubuntu.com/kubuntu/releases/26.04.1/release/kubuntu-26.04.1-desktop-amd64.manifest
[LI27]: https://archive.ubuntu.com/ubuntu/pool/universe/c/calamares-settings-ubuntu/calamares-settings-ubuntu_26.04.12.tar.xz
[LI28]: https://archive.ubuntu.com/ubuntu/pool/universe/k/kubuntu-installer-prompt/kubuntu-installer-prompt_26.04.3.tar.xz
[LI29]: https://github.com/rhinstaller/anaconda-webui/blob/68/src/components/storage/cockpit-storage-integration/CockpitStorageIntegration.jsx
[LI30]: https://guides.frame.work/Guide/Fedora+44+Workstation+Installation+on+the+Framework+Laptop+16/731
[LI31]: https://raw.githubusercontent.com/calamares/calamares/v3.3.14/src/modules/users/page_usersetup.ui
[LI32]: https://raw.githubusercontent.com/calamares/calamares/v3.3.14/src/modules/partition/gui/ChoicePage.cpp
[LI33]: https://archive.ubuntu.com/ubuntu/pool/universe/c/calamares/calamares_3.3.14-0ubuntu25.debian.tar.xz
