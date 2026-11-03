# Volledige OS-installatiejourney — vergelijkingsmatrix

Onderzoeksdatum: **2 november 2026**. Fase 4.1/4.2. Status: officiële documentatie en versiegebonden broncode gelezen; **geen installaties, gebruikersmetingen of eigen screenshots uitgevoerd**. De matrix vergelijkt beslissingen, niet één veronderstelde schermvolgorde.

## Routes, versies en bewijs

| Platform | Gekozen route | Afbakening |
|---|---|---|
| Windows | Windows 11 **26H2**, Home/Pro persoonlijk, schone x64-USB-installatie | Actuele algemene download; geen exacte mediabuild/hash. Upgrade vanuit Windows en OEM/bedrijfs-OOBE apart beschreven. [WI01] [WI02] |
| macOS | **27.2 Golden Gate**, Apple silicon, Recovery en Setup Assistant bij lege/nieuwe gebruikersomgeving | Release 26 oktober 2026; Recovery garandeert niet de nieuwste patch. Intel/Tahoe 26.7.2 en behoudinstallatie aparte varianten. [WI14] [WI15] [WI17] |
| Ubuntu | Desktop **26.04.1 LTS**, GNOME 50, interactief | Bewust actuele LTS, niet interim 26.10. [LI01] |
| Fedora | Workstation **44**, Live-ISO, GNOME 50 | Actuele stabiele download; Anaconda 44.30/WebUI 68 als pakketbron. Geen F45/46-functies overgenomen. [LI06] [LI07] [LI08] [LI09] |
| KDE-distributie | **Kubuntu 26.04.1 LTS**, Plasma 6.6, stock Calamares | Image-manifest en configuratie/patches gelezen; geen generieke KDE-installer verondersteld. [LI26] [LI27] |
| nieuw-os | Mainbron `e08fb0e` en apart gelezen native bron `1f0dd9c` | Eigen UEFI/QEMU-boot is aanwezig; **geen volwaardige installer, fysieke installatie of persoonlijke eerste-login aangetoond**. Kolom **P** is een toekomstvoorstel, geen werkende functie. [Repositorybasis](2026-11-02/repository-baseline.md) |

Volledige bronpassages, oorspronkelijke publicatie-/raadpleegdatums, 31-rijregisters en variantdetails: [Windows/macOS](2026-11-02/journey/windows-macos-installation.md) en [Linux-installers](2026-11-02/journey/linux-installation.md). Die dossiers zijn leidend bij verkorte cellen hieronder. `Niet vastgesteld` betekent onvoldoende precies bewijs, niet dat een functie ontbreekt. IDs 01–31 zijn dimensies; er is geen telling van schermen of benodigde klikken.

## OS installation flow matrix

**Bronkolommen:** [BRON]. **nieuw-os P:** [VOORSTEL]; huidige afwezigheid is hierboven vastgelegd. Ubuntu/Fedora/Kubuntu zijn geen beschikbare implementatiebasis voor de eigen kernel.

| ID | Aspect | Windows | macOS | Ubuntu | Fedora | Kubuntu / KDE | nieuw-os |
|---|---|---|---|---|---|---|---|
| 01 | Bootmedium/recovery | USB/firmwaremenu [WI02] | Aan/uit → Options; Intel Command-R [WI18] | USB/firmware [LI02] | Live-USB/Media Writer [LI06] | Desktop-live-image [LI26] | Nu UEFI/QEMU; P: gecontroleerd medium, geen hostschijf |
| 02 | Eerste scherm | Setup-taal [WI03] | Recovery/ontgrendeling/hulpmiddelen [WI19] | Installer opent [LI02] | Liveomgeving → WebUI-taal [LI12] | Eigen prompt → Calamares [LI28] | P: welkom met taal/hulpmiddelen |
| 03 | Taal | Setup; mogelijk opnieuw OOBE [WI03] [WI04] | Setup Assistant; na wissen heractivatie [WI20] [WI27] | Eerste keuze [LI02] | Installer én eerste start [LI12] [LI19] | Prompt en welcome [LI28] [LI27] | P: eenmaal kiezen, later wijzigbaar |
| 04 | Regio/tijdzone | Regio OOBE; geen vaste tijdzonepagina bewezen [WI04] | Exact scherm/positie niet vastgesteld [WI20] | Na account, kaart/stad [LI03] | Eerste start, niet installer [LI19] | Locale vóór keyboard; GeoIP [LI27] | P: voorstel bij lokale voorkeuren, geen locatieconsent afleiden |
| 05 | Toetsenbord | Setup en OOBE [WI03] [WI05] | Exacte pagina onvoldoende vastgelegd [WI20] | Vóór netwerk [LI02] | Live-instellingen; opnieuw eerste start [LI19] | Na locale [LI27] | P: testen vóór wachtwoord/ontgrendeling |
| 06 | Toegankelijkheid | Keyboard beschreven; pre-wipe-screenreader onbevestigd [WI05] | Setup Assistant; Recoverydekking onbevestigd [WI20] | Vroege eigen stap [LI02] | Geen aparte wizardpagina; werking onbeproefd [LI12] | Geen aparte module; werking onbeproefd [LI27] | P: vanaf welkom en bij fouten, niet pas na installatie |
| 07 | Netwerk | Internet voor persoonlijke OOBE vereist [WI06] | Nodig voor Recovery-download/activatie [WI17] [WI27] | Optioneel [LI02] | Liveomgeving; eerste-startpagina [LI19] | Prompt; online/offline verschillen [LI28] | P: lokaal compleet kernmedium offline; downloads apart |
| 08 | Try/Install | Geen live-Try in gekozen route [WI03] | Recovery is geen live-desktop [WI19] | Try of Install na netwerk [LI02] | Live proberen mogelijk [LI06] | Eigen Try/Install-prompt [LI28] | P: proberen zonder schijfmutatie, installeren expliciet |
| 09 | Hardwarecontrole | Eisen; Assistant-check niet identiek aan USB bewezen [WI02] | Modelcompatibiliteit vooraf [WI17] | Liveproef; geen proprietary drivers [LI02] | Opslag-/ruimtecontrole [LI15] | RAM/opslag/root-check; geen hardwaregarantie [LI27] | P: geteste mogelijkheden vóór schijfkeuze, blokkades eerlijk |
| 10 | Doelopslag | Schijf/partitie [WI03] | Doelvolume, mogelijk unlock [WI17] | Doelschijf [LI03] | Een/meerdere schijven [LI13] | Partition-module [LI27] | P: gebruiker kiest geïdentificeerd doel; onbekend ≠ leeg |
| 11 | Partitionering | Clean-route verwijdert partities [WI03] | Disk Utility apart; APFS [WI27] | Automatisch/handmatig [LI03] | Hele schijf/vrij/mountpoints/editor [LI13] | Wissen/vervangen/naast/handmatig conditioneel [LI27] [LI32] [LI33] | P: begrijpelijke gevolgen; technische layout in Advanced |
| 12 | Behouden/vervangen | Clean wist; in-place behoudopties [WI03] | Reinstall bewaart; wissen apart [WI17] | Wissen of alongside [LI03] | Vrije ruimte behoudt; reinstall conditioneel [LI13] | Partitie vervangen of hele schijf [LI32] [LI33] | P: opties per bewezen capaciteit, bronbehoud expliciet |
| 13 | Dual boot | Niet begeleid in deze clean-route [WI03] | Extra macOS/APFS of extern volume [WI26] | Alongside; BitLocker kan blokkeren [LI03] | Vrije ruimte; geen universele bootgarantie [LI13] | Geschikte verkleinbare partitie vereist [LI32] [LI33] | P: uitsluitend gevalideerde combinatie, anders blokkeren |
| 14 | Encryptie | Device Encryption geschikt apparaat/account [WI07] | FileVault; hardwareversleuteling is andere garantie [WI25] | Passphrase/LVM of geschikte TPM-route [LI05] | Nieuwe partities met LUKS/passphrase [LI16] | Automatische LUKS2-layout, boot apart [LI27] | P: echte persoonlijke bescherming en recovery vóór commit; gate open |
| 15 | Gebruikersaccount | OOBE [WI04] | Setup Assistant lokaal computeraccount [WI20] | Vóór summary [LI03] | Eerste start [LI19] | Vóór summary [LI27] | P: lokaal persoonlijk account; geen huidige guest-loginclaim |
| 16 | Apparaatnaam | OOBE-gids; positie niet gepind [WI05] | Geen aparte pagina aangetoond [WI20] | Bij account [LI03] | Geen reviewveld in Live-route [LI15] | Users-pagina [LI27] | P: niet-identificerende bewerkbare naam, geen extra verplichte pagina |
| 17 | Wachtwoord/PIN/biometrie | Account → Hello/PIN; biometrie conditioneel [WI05] | Computerwachtwoord; Touch ID conditioneel [WI20] | Loginwachtwoord ≠ disk-PIN [LI05] | Eerste-startwachtwoord [LI19] | Wachtwoord; autologin standaard uit [LI27] | P: bestaande echte methoden, login en opslagontgrendeling onderscheiden |
| 18 | Online/lokaal | Persoonlijk Home/Pro MS-account vereist [WI06] | Apple Account later mogelijk [WI21] | Lokaal [LI03] | Lokaal; enterprisevariant [LI19] | Lokaal [LI27] | P: lokale kern; providers later optioneel |
| 19 | Privacy | OOBE-keuzes variabel [WI04] | Meerdere aparte panelen [WI20] | Aanvullend eerste start [LI03] | Eerste start, conditioneel [LI19] | Geen aparte privacywizard aangetoond [LI27] | P: korte begrijpelijke controle; veiligheidsvloer vast |
| 20 | Telemetrie | Required/Optional onderscheiden [WI11] | Analytics met toestemming [WI24] | Apparaatinformatie opt-in [LI03] | Rapportagepagina conditioneel; upstream aan [LI20] | Geen pagina bewijst geen afwezigheid [LI27] | P: geen OS-telemetrie volgens bestaand contract |
| 21 | Locatie | Privacy & security; default onbekend [WI12] | Location Services, default onbekend [WI20] | Eerste start [LI03] | Upstream-switch aan; provider onbekend [LI20] | GeoIP-tijdzone geconfigureerd [LI27] | P: geen stil locatie-/GeoIP-verzoek; tijdzone handmatig mogelijk |
| 22 | Installatie-updates | Kritieke OOBE-patches; versieaanbod [WI10] | Recovery kan ouder; update naderhand [WI17] | Online updates en updater naderhand [LI03] | Geen keuze in Live-wizard [LI12] | Optioneel, niet offline [LI27] | P: geverifieerde payload; download onderscheiden van persoonlijke data |
| 23 | Drivers/firmware | Kritieke drivers; fabrikant apart [WI04] | Geen losse driverkeuze beschreven [WI17] | Third-party-keuze [LI03] | Externe repos bij eerste start [LI19] | Zichtbare driver/codeckeuze niet bevestigd [LI27] | P: ondersteund profiel; geen stille firmwareactie |
| 24 | Optionele software | Diensten/OEM-aanbod conditioneel [WI04] | Migratie, Siri, AI e.a. conditioneel [WI20] | Default/extended [LI03] | Live-payload; repos later [LI19] | Minimal/Normal/Full en online extras [LI27] | P: kernset; uitbreidingen na desktop, geen verkooptrechter |
| 25 | Summary/schrijfgrens | Ready to install ná eerdere partitieacties [WI03] | Wissen apart bevestigd, geen alles-summary [WI27] | Ready to install → Install [LI03] | Review; Cockpit schrijft direct zonder undo [LI29] | Summary/bevestiging → uitvoering [LI27] | P: compleet veranderplan vóór élke schijfmutatie |
| 26 | Voortgang | Installatie/herstarts; exacte ETA onbekend [WI03] | Balk/herstarts/zwart scherm [WI17] | Slideshow/details [LI03] | Vier fasen/status/fouten [LI17] | Uitvoeringsmodules; indicator onbekend [LI27] | P: echte fase, eerlijke onzekerheid en annuleergrens |
| 27 | Herstart | Tijdens Setup/OOBE [WI04] | Mogelijk meerdere [WI17] | Restart now [LI03] | Exit naar live, daarna zelf reboot [LI17] | Keuze bij finished [LI27] | P: expliciete gereedstatus, geen verzonnen succes |
| 28 | Medium verwijderen | Bootvolgorde controleren; prompt niet bewezen [WI02] | N.v.t. ingebouwde Recovery [WI19] | USB op prompt verwijderen [LI03] | Exacte prompt niet vastgesteld [LI17] | Exacte prompt niet vastgesteld [LI27] | P: medium herkenbaar noemen en terugboot voorkomen |
| 29 | Eerste boot | OOBE [WI03] | Lege installatie → Setup Assistant [WI17] | Unlock/login indien van toepassing [LI03] | GNOME Initial Setup [LI19] | Loginroute, onbeproefd [LI27] | P: echte installatie-/accounttoestand verifiëren |
| 30 | Tweede fase | OOBE met account/privacy/diensten [WI04] | Setup Assistant; beheer kan variëren [WI23] | Initial Setup na de desktop (31) [LI03] | Account/privacy/repos e.a. conditioneel [LI19] | Plasma Welcome behalve minimal; details onbekend [LI27] | P: alleen noodzakelijke afronding; begeleiding overslaanbaar |
| 31 | Werkelijke desktop | Na OOBE [WI04] | Na Welcome [WI20] | GNOME 50 [LI01] | GNOME 50 [LI06] | Plasma 6.6 [LI26] | Nu beperkte native sessie; P: aantoonbare day-one-taken |

## Werkelijke volgorde naast elkaar

- **Windows:** USB → taal/keyboard → installatie-/verlieskeuze/licentie → doelpartities, waarbij verwijderen al destructief schrijft → ready → OS installeren/herstarts → OOBE met netwerk/account/privacy/diensten → desktop. Een upgrade vanuit Windows heeft andere behoudopties. [WI03] [WI04]
- **macOS:** Recovery/ontgrendeling → eventueel apart wissen/reactiveren → herinstalleren/doelvolume → voortgang/herstarts → bij lege omgeving Setup Assistant → desktop. Behoudinstallatie is geen verplichte nieuwe accountfase. Exacte totale paneelvolgorde blijft onvoldoende vastgelegd. [WI17] [WI20] [WI27]
- **Ubuntu:** taal → toegankelijkheid → keyboard → netwerk → Try/Install → interactieve installatie/apps/drivers → opslag → account → tijdzone → summary → schrijven → reboot/medium → unlock/login → desktop → aanvullende Initial Setup. [LI02] [LI03]
- **Fedora Workstation:** live → taal → installatiemethode/doelen → opslag, waarbij de optionele Cockpit-editor direct schrijft zonder undo → review → installeruitvoering → terug naar live → herstart → GNOME Initial Setup met conditionele pagina's → eigen desktop. [LI12] [LI19] [LI29]
- **Kubuntu:** taal/netwerk/Try–Install-prompt → welcome → locale → keyboard → softwarekeuze → partition → users → summary → uitvoering → finished/herstart → desktop, eventueel Welcome. [LI27] [LI28]

## A/B/C: behoud, variant en frictie

**A — gedeelde concepten [BRON/ANALYSE]:** taal/invoer, een installatiebron en doel, keuze over bestaande gegevens, voortgang, herstart/overgang en een eigen gebruikerssessie komen in meerdere routes terug. Herkenbaarheid is plausibel, maar gebruikerskennis is niet gemeten. Niet iedere route vraagt een aparte account-, encryptie- of netwerkschermkeuze.

**B — platformspecifiek [BRON]:** Windows producteditie/licentie/Hello en verplichte persoonlijke MS-account; Apple Recovery/activatie/FileVault/Apple Account; Linux live-Try, distributiegebonden pakket-/repositorykeuzes en partitioneringsmogelijkheden. De conventie zelf is geen automatisch voorschrift voor nieuw-os. [WI03] [WI06] [WI17] [WI25] [LI02] [LI19] [LI27]

**C — mogelijke vermijdbare frictie [ANALYSE, niet gemeten]:** dubbele taal/invoer zonder duidelijke overdracht, account-/privacyvragen over meerdere fasen, aanbiedingen zonder installatiebehoefte, technische layouttermen op een beginnerspad, verborgen vroege schrijfgrenzen en een netwerkverplichting die een ontbrekende driver niet kan oplossen. Een extra herstelcontrole is juist waardevol; minder schermen is geen zelfstandig kwaliteitsdoel. Begeleide keuze, defaults en samenvoegen moeten worden getest.

## Terminologie en beslissingen

| Begrijpelijke hoofdterm (voorstel) | Technische uitleg alleen waar nodig | Wat nooit mag verdwijnen |
|---|---|---|
| Deze schijf gebruiken | Merk, capaciteit, herkenbaar stabiel kenmerk; partitielayout in Advanced | Welk fysiek doel en welke gegevens geraakt worden |
| Alles op deze schijf vervangen | Formateren/partities opnieuw aanmaken | “Alle bestanden en bestaande systemen op … worden verwijderd” vóór schrijven |
| Bestaand systeem behouden | Vrije ruimte/resize/bootmanager in detail | Welke delen werkelijk ongemoeid blijven; rest- en bootrisico's |
| Gegevens beschermen | Encryptie, sleutels, ontgrendelen en herstel onderscheiden | Wat verlies van een herstelmiddel betekent; geen gegarandeerde undo |
| Persoonlijk account op dit apparaat | Authenticatie versus providerkoppeling | Wie toegang heeft; geen openbare gast als privéaccount aanduiden |

**Veilig te suggereren [VOORSTEL]:** taal/layout laten kiezen of bewerkbaar voorstellen, een niet-identificerende apparaatnaam genereren, een geteste interne layout berekenen **na** doel-/methodekeuze, privacybeschermende defaults en tijdzone zonder locatieverzoek. Automatisch berekenen is geen autorisatie om te schrijven. Herkenning kan onzeker zijn; de UI toont dat.

**Nooit stil automatisch [VOORSTEL]:** doel kiezen om te wissen, onbekende schijf als leeg behandelen, partities verkleinen/verwijderen/converteren, bootconfiguratie veranderen, encryptie afzwakken, een herstelkey exporteren, providers koppelen, firmware aanpassen of ontsleutelsleutels aan een derde geven. Ook Advanced valt onder dezelfde schrijfgrens. Verandert schijfidentiteit, plan of conditie, dan vervalt de bevestiging.

## Gevolg voor de nieuw-os-funnel

[ONTWERPKEUZE] Eerst bruikbare taal/toegankelijkheid/invoer en een veilige proefmogelijkheid; daarna echte geschiktheid en opslagkeuze; account/protectie vóór één complete schrijfbevestiging; na reboot alleen noodzakelijke verificatie en optionele taakhulp. De bronnen schrijven deze precieze volgorde niet voor. De [installerspecificatie](../product/new-os-installer-spec.md) motiveert per stap wat wordt gecombineerd, uitgesteld of weggelaten en welke backendgates nog openstaan.

Juridische/technische omgang: alleen openbare tekst, configuratie en bronverwijzingen gebruikt, geen gereproduceerde OS-screenshots of gedownloade installatie-images. Schermbeschrijvingen zijn brongebonden samenvattingen; onbekende details zijn geen mockbeelden. Bronversies kunnen wijzigen: vóór uitvoering media/builds vastleggen en opnieuw controleren.

## Directe bronnen voor de verkorte matrix

Volledige titels, datum en beperkingen staan in de twee brondossiers; iedere ID hieronder verwijst naar dezelfde oorspronkelijke bron.

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
