# Installatiejourney: Ubuntu, Fedora Workstation en Kubuntu

**[BRON] De drie onderzochte Linux-installers verdelen opslagkeuzes, accountaanmaak en eerste-startinstellingen verschillend; één generieke Linux-installatiewizard zou die verschillen verbergen.** Dit is een documentatie- en broncodewalkthrough op **2026-11-02**, geen uitgevoerde installatie of gebruikerstest.

## Afbakening en broncontrole

Onderzoeksvraag: welke beslissingen maakt een nieuwe gebruiker tussen opstartmedium en eigen desktop, wanneer veranderen gegevens onomkeerbaar en welke stappen zijn optioneel? Zoekstrategie: officiële downloads → releasegebonden handleidingen → pakketversies/configuratie/broncode waar algemene handleidingen onvoldoende precies zijn. Alleen openbare tekst en bronpakketten gelezen; geen installatie-images gedownload, VM gestart of screenshots vervaardigd.

| Onderzochte route | Versie en motivatie | Versiegrens |
|---|---|---|
| Ubuntu Desktop, interactieve standaardinstallatie | **26.04.1 LTS**, actuele LTS, GNOME **50**. [LI01] | Ubuntu 26.10 bestaat eveneens; hier bewust de ondersteunde LTS. De handleiding is voor de 26.04-serie. [LI01][LI03] |
| Fedora Workstation, Live-ISO | **44**, actuele stabiele download, uitgebracht 2026-04-28; GNOME **50**. [LI06][LI07] | Pakketregister: Anaconda **44.30**, WebUI **68**, GNOME Initial Setup **50.1**. Dat zijn gecontroleerde F44-pakketversies, geen geïnspecteerde ISO-inhoud. [LI08][LI09][LI10] |
| Stock Kubuntu, gewone desktopinstallatie | **26.04.1 LTS**, ondersteuning tot april 2029; Plasma **6.6**. Vergelijkbaar met Ubuntu LTS, met afzonderlijke Calamares-interactie. [LI24][LI25] | Officieel image-manifest: Calamares **3.3.14-0ubuntu25**, instellingen **1:26.04.12**, prompt **26.04.3**, Plasma Welcome **6.6.5**. [LI26] |

Upstream GNOME 51/Plasma 6.8 en andere nieuwere desktopdocumentatie worden niet aan deze distributie-installers toegeschreven. `latest`-documentatie van Anaconda bevat al Fedora-45/46-wijzigingen; gebruikte uitvoeringscode is daarom op versie gepind. [LI11][LI12][LI18]

**Leeswijzer:** elke cel is **[BRON]** met LI-bronverwijzing, behalve expliciet **onbevestigd**. “Geen pagina” betekent uitsluitend geen pagina in de gecontroleerde route; het bewijst geen afwezigheid van een systeemfunctie. De nummers zijn het vergelijkingsregister, **niet** de schermvolgorde. Desktop betekent hier de eigen geïnstalleerde sessie; een live-desktop telt daarvoor niet.

## Vergelijkingsregister: 31 beslismomenten

| # | Moment | Ubuntu 26.04.1 LTS | Fedora Workstation 44 Live | Kubuntu 26.04.1 LTS |
|---|---|---|---|---|
| 1 | Bootmedium/recovery | Start vanaf USB; firmwarekeuze kan nodig zijn. [LI02] | Live-USB, onder meer via Media Writer; recoveryroute buiten scope. [LI06] | Desktop-live-image; firmware/recoverydetails onbevestigd. [LI26][LI28] |
| 2 | Eerste zichtbare installer | Installer opent automatisch. [LI02] | In liveomgeving: WebUI met taal als eerste wizardstap; automatische start onbevestigd. [LI06][LI12] | Eigen prompt vóór Calamares. [LI28] |
| 3 | Taal | Eerste keuze. [LI02] | WebUI-taal; eerste-startwizard heeft eigen taalpagina. [LI12][LI19] | Taal in prompt, vervolgens Calamares-welcome. [LI28][LI27] |
| 4 | Regio/tijdzone | Na account, kaart/stad. [LI03] | Workstation verbergt installer-tijdzone; eerste-startwizard bevat die wel. [LI11][LI19] | Locale-stap vóór toetsenbord; tijdzonedetectie via GeoIP geconfigureerd. [LI27] |
| 5 | Toetsenbord | Vóór netwerk. [LI02] | GNOME-instellingen, meerdere layouts ondersteund; taalkeuze bepaalt Workstation-layout niet automatisch. Eerste start vraagt opnieuw. [LI18][LI19] | Eigen keyboard-stap na locale. [LI27] |
| 6 | Toegankelijkheid | Vroege toegankelijkheidsstap. [LI02] | Geen afzonderlijke wizardstap in deze configuratie; daadwerkelijke schermlezertoegang onbeproefd. [LI11][LI12] | Geen afzonderlijke stap in prompt/modulelijst; hulpmiddelen en keyboardbediening onbeproefd. [LI27][LI28] |
| 7 | Netwerk | Verbinden optioneel. [LI02] | Via liveomgeving/extra menu, geen F45-netwerkpagina; eerste-startwizard heeft netwerkstap. [LI18][LI19] | Wifi/Ethernet in prompt; online en offline softwarekeuzes verschillen. [LI28][LI27] |
| 8 | Try versus Install | Na netwerk; Try sluit installer, desktopshortcut opent opnieuw. [LI02] | Live-USB laat proberen vóór installatie toe. [LI06] | Afzonderlijke Try/Install-acties; Try opent liveomgeving. [LI28] |
| 9 | Hardwarecontrole | Live proberen test eigen hardware; proprietary drivers ontbreken daar. [LI02] | Opslag-/ruimtecontrole; geen volledige hardwarecompatibiliteitsgarantie. [LI15][LI18] | Welcome controleert RAM/opslag/root verplicht; internet/netvoeding niet verplicht. Geen hardwarecertificering. [LI27] |
| 10 | Opslagapparaat | Doelschijf kiezen. [LI03] | Een of meer doelschijven bij Installation method. [LI13] | Partition-module; precieze schijfselector niet visueel gecontroleerd. [LI27] |
| 11 | Partitionering | Automatisch of handmatig. [LI03] | Entire disk, vrije ruimte, mountpointtoewijzing of geavanceerde editor, afhankelijk van situatie. [LI13][LI29] | Wissen/vervangen/naast/handmatig conditioneel; ext4 standaard, ook btrfs/xfs geconfigureerd. [LI27][LI32][LI33] |
| 12 | Bestaand OS behouden/vervangen | Wissen of naast bestaand OS; BitLocker kan blokkeren. [LI03] | Hele schijf wissen; vrije ruimte behoudt bestaande partities. Reinstall bewaart home alleen bij herkenbare Fedora-layout. [LI13] | Partitie vervangen of hele schijf wissen; beschikbare routes hangen af van schijfsituatie. [LI32][LI33] |
| 13 | Dual boot | Alongside met ruimteverdeling. [LI03] | Use free space bij aanwezige partities; bootbaarheid van elke combinatie niet bewezen. [LI13] | Alongside vereist geschikte verkleinbare partitie; bij UEFI ook ESP. Geen algemene dual-bootgarantie. [LI32][LI33] |
| 14 | Disk-encryptie | LVM/passphrase of geschikte TPM-hardware; TPM-route toont recoverykey na installatie. [LI04][LI05] | LUKS-passphrase voor nieuwe partities, boot-layout zichtbaar; geen recoverycode-/TPM-stap bevestigd. [LI16] | Automatische layout: LUKS2 met onversleutelde `/boot`; handmatige layouts apart beoordelen. Geen TPM-/recoverycodeflow bevestigd. [LI27] |
| 15 | Gebruikersaccount | Naam, username, wachtwoord. [LI03] | Pas bij eerste start; lokaal account standaard admin, met aparte ouderlijk-toezichtvariant. [LI11][LI19][LI22] | Voor samenvatting; gebruiker in sudo-groep, apart rootwachtwoord uit. [LI27] |
| 16 | Apparaatnaam/hostname | Computernaam bij account. [LI03] | Hostnameveld in review uitsluitend BOOT_ISO, niet deze Live-route. [LI15] | Bewerkbare computernaam op gebruikerspagina; stockconfig behoudt de standaard. [LI27][LI31][LI33] |
| 17 | Wachtwoord/PIN/biometrie | Loginwachtwoord; TPM-disk-PIN is afzonderlijk. [LI03][LI05] | Eerste-startwachtwoord; geen PIN-/biometriepagina in gecontroleerde sequentie. [LI19] | Lokaal wachtwoord; autologin standaard uit. Geen PIN-/biometriestap; exacte kwaliteitsafdwinging onbeproefd. [LI27][LI33] |
| 18 | Online versus lokaal | Lokaal installatieaccount. [LI03] | Lokaal standaard; enterprise-aanmelding aparte netwerkafhankelijke variant. Geen verplichte consumentencloudaccount. [LI21] | Lokale usermodule; geen onlineaccountstap in configuratie. [LI27] |
| 19 | Privacykeuzes | Aanvullende eerste-startkeuzes. [LI03] | Privacy bij eerste start, met conditionele onderdelen. [LI19][LI20] | Geen privacywizard; GeoIP-tijdzoneverzoek afzonderlijk relevant. [LI27] |
| 20 | Telemetrie | Apparaatinformatie opt-in, standaard uit. [LI03] | Technische rapportagepagina afhankelijk van ABRT-dienst; upstreamswitch aanvankelijk aan. Distributiegedrag niet uitgevoerd. [LI20] | Geen telemetriepagina in modulelijst; geen bewijs van nul systeemtelemetrie. [LI27] |
| 21 | Locatie | Locatiediensten bij eerste start. [LI03] | Locatieswitch in eerste-startprivacy; upstream aanvankelijk aan. Provider niet vastgesteld. [LI20] | Tijdzone-GeoIP geconfigureerd naar `geoip.ubuntu.com/lookup`; geen aparte consentpagina aangetroffen. [LI27] |
| 22 | Updates tijdens installatie | Netwerk ondersteunt updates; updater na installatie. [LI03] | Geen updatekeuzestap in gecontroleerde Live-wizard. [LI12][LI14] | Optioneel; uitvoering na uitpakken binnen installatietaak. Offline keuze verborgen. [LI27] |
| 23 | Drivers/firmware | Keuze third-party drivers/mediaondersteuning. [LI03] | Eerste-startkeuze externe repositories kan apps/drivers ontsluiten; geen aangetoonde firmwarewizard. [LI23] | Huidige zichtbare driver/codeccheckbox onbevestigd; configuratieactie bewijst geen UI. [LI27] |
| 24 | Optionele software | Default of extended apps. [LI03] | Live-image heeft geen DNF-softwareselectiepagina; later optionele externe repositories. [LI14][LI23] | Minimal/Normal/Full; online extra apps. Full/extras verborgen indien offline. [LI27] |
| 25 | Samenvatting/schrijfgrens | Ready to install → Install. [LI03] | Review met bevestiging bij wissen/wijzigen. **Uitzondering:** Cockpit-editor schrijft onmiddellijk, zonder undo. [LI15][LI29] | Summary + bevestigingsprompt; daarna partition als eerste uitvoeringsmodule. [LI27] |
| 26 | Voortgang | Slideshow en details. [LI03] | Vier fasen plus status/foutmelding. [LI17] | Uitvoeringsmodules tot finished; precieze animatie/ETA onbevestigd. [LI27] |
| 27 | Herstart | Restart now. [LI03] | Exit to live desktop; vervolgens zelf herstarten. Niet de F45-rebootknop. [LI17][LI18] | Herstart selecteerbaar bij finished. [LI27] |
| 28 | Medium verwijderen | USB verwijderen op prompt, Enter. [LI03] | Precies verwijdermoment/-prompt onbevestigd in gecontroleerde UI. [LI17] | Precies verwijdermoment/-prompt onbevestigd. [LI27] |
| 29 | Eerste boot | Eventuele disk-unlock, login. [LI03][LI05] | GNOME Initial Setup vóór eigen gebruikerssessie. [LI12][LI19] | Login/autologinkeuze volgt installatieconfiguratie; exacte eerste schermen onbeproefd. [LI27] |
| 30 | Tweede fase/OOBE | Initial Setup met aanvullende voorkeuren. [LI03] | Ja: taal/keyboard/netwerk/privacy/tijdzone/repos/account/wachtwoord; conditionele stappen. [LI19][LI20][LI21][LI22][LI23] | Account al aangemaakt. Plasma Welcome aanwezig behalve minimal; vragen onbevestigd. OEM is aparte route. [LI26][LI27] |
| 31 | Eerste echte desktop | Geïnstalleerde GNOME 50-sessie. [LI01][LI03] | GNOME 50 na eerste-startafhandeling. [LI07][LI19] | Plasma 6.6; daadwerkelijk bereik van desktop niet getest. [LI25][LI27] |

## Werkelijke volgorde en variantgrenzen

- **Ubuntu:** boot → taal → toegankelijkheid → keyboard → netwerk → Try/Install. [LI02] Daarna in registernummers: interactief → **24/23 → 10–14 → 15–18 → 4 → 25 → 26 → 27/28 → 29 → 30 → 31**. [LI03] Hardwarebacked encryptie voegt geschiktheidscontrole, mogelijke firmwareactie en recoverykey toe. [LI05]
- **Fedora Workstation Live:** liveomgeving → WebUI-taal → installatiemethode/doelschijven → opslagconfiguratie of mountpointtoewijzing → review → uitvoering → terug naar liveomgeving → herstart → GNOME Initial Setup → eigen desktop. Accounts en tijdzone zijn bewust uit de installer verborgen; softwareselectie is voor DNF-payloads, niet deze Live-route. [LI11][LI12][LI13][LI14][LI17] De gecontroleerde GNOME-paginatabel is taal → keyboard → netwerk → privacy → tijdzone → repositories → account → wachtwoord → eventueel ouderlijk toezicht → afronding. Distributie-instellingen kunnen pagina's overslaan. [LI19]
- **Kubuntu:** prompt met taal/netwerk/Try–Install → Calamares **welcome → locale → keyboard → pkgselect → partition → users → summary → uitvoering → finished**. De precieze volgorde tussen keuzes op het eerste promptscherm ligt niet vast. [LI27][LI28] Minimal verwijdert Plasma Welcome; OEM doorloopt later een andere gebruikersconfiguratie. Dat zijn geen verplichte stappen van de gewone installatie. [LI27]

**[ANALYSE] Drie relevante ontwerpvragen voor nieuw-os:** (1) kunnen gebruikers vóór installeren toetsenbord, netwerk en hulpmiddelen proberen; (2) is elk destructief pad herkenbaar, inclusief afwijkende geavanceerde editors; (3) is duidelijk wat vóór schrijven, na herstart of optioneel later gebeurt? De bronnen bewijzen aanwezige mechanismen, geen lagere foutkans of gebruikersvoorkeur. **[HYPOTHESE]** Een doorlopende voortgangslijst met afzonderlijke opslagbevestiging kan die overgangen begrijpelijker maken; dit vereist gebruikerstests.

**Beperkingen:** geen visuele/keyboard-/schermlezerwaarnemingen, installatieduur, foutfrequenties of end-to-end-succesclaims. Een bronmodule kan conditioneel verborgen zijn; zulke voorwaarden zijn hierboven opgenomen waar vastgesteld. Fedora-pakketregisters kunnen verder zijn dan het oorspronkelijke ISO. Een recente Framework-F44-handleiding is gecontroleerd maar niet als routebewijs gebruikt: deze bevat nog Ubuntu-boottekst, oude installerterminologie en een verouderde locatieprovider. [LI30] Linux-installers zijn vergelijkingmateriaal; hun drivers, package managers en encryptie zijn geen automatisch beschikbare implementaties voor de zelfstandige nieuw-os-kernel.

## Bronnenregister

Alle bronnen geraadpleegd **2026-11-02**. Datum = zichtbare publicatie/update, anders onbekend. Broncode is implementatiebewijs voor de genoemde versie, geen runtimevalidatie. IDs met meerdere bestanden verwijzen hieronder naar afzonderlijk controleerbare primaire passages.

| ID | Titel, organisatie; datum/versie | Gecontroleerde passage en beperking |
|---|---|---|
| [LI01] | Download Ubuntu Desktop, Canonical; onbekend, 26.04.1 | LTS-download en GNOME 50; ook aparte 26.10-sectie. Levende downloadpagina. |
| [LI02] | Try Ubuntu Desktop, Canonical; 2026-10-13, 26.04 | Boot, vroege keuzes, Try-terugkeer en open drivers. Geen complete hardwaretest. |
| [LI03] | Install Ubuntu Desktop, Canonical; 2026-10-15, 26.04 | Interactieve route, schijf/account, summary, herstart en Initial Setup. Geen eigen uitvoering. |
| [LI04] | Advanced disk setup features, Canonical; 2026-10-13, 26.04 | LVM/passphrase, hardwarebacked en geavanceerde opties. Hardwarecondities verschillen. |
| [LI05] | Encrypt your disk with TPM, Canonical; 2026-10-13, 26.04 | Geschiktheid, volledige schijf, recoverykey, latere disk-PIN. Geen login-PIN-bewijs. |
| [LI06] | Download Fedora Workstation, Fedora Project; 2026-04-28 release, 44 | Actuele stabiele Live-download en Media Writer/Try. Geen installatieschermen. |
| [LI07] | Announcing Fedora Linux 44, Jef Spaleta/Fedora Magazine; 2026-04-28 | Workstation GNOME 50; KDE-variant afzonderlijk. Geen actuele F45-releaseclaim. |
| [LI08] | anaconda-webui, Fedora Packages; onbekend | F44 stable 68-1.fc44. Pakketregister, geen ISO-manifest. |
| [LI09] | anaconda, Fedora Packages; onbekend | F44 stable 44.30-2.fc44. Zelfde beperking. |
| [LI10] | gnome-initial-setup, Fedora Packages; onbekend | F44 stable 50.1-1.fc44. Zelfde beperking. |
| [LI11] | fedora-workstation.conf, Anaconda/Red Hat; tag anaconda-44.30, datum onbekend | `hidden_webui_pages`: accounts/date-time. Alleen Workstation-profiel. |
| [LI12] | steps.js, Anaconda WebUI/Red Hat; tag 68, datum onbekend | Stappenlijst en filters; comment legt accountuitstel uit. Combineer met profiel. |
| [LI13] | Installation Flow Overview, Anaconda WebUI; tag 68, datum onbekend | Opslagscenario's en voorwaarden Reinstall/Free space. Generieke lijst niet blind op Workstation toepassen. |
| [LI14] | software/index.js, Anaconda WebUI; tag 68, datum onbekend | Pagina verborgen wanneer payload niet DNF is. Geen geïnspecteerde ISO-payload. |
| [LI15] | ReviewConfiguration.jsx, Anaconda WebUI; tag 68, datum onbekend | Schrijfbevestiging; hostname alleen `isBootIso`. Geen bewijs dat elke editor uitgesteld schrijft. |
| [LI16] | DiskEncryption.jsx, Anaconda WebUI; tag 68, datum onbekend | LUKS-passphrase, boot-keyboard, uitsluitend nieuwe partities. Geen herstelprocedure. |
| [LI17] | InstallationProgress.jsx, Anaconda WebUI; tag 68, datum onbekend | Vier fasen, succes/fout en Live-exit versus Boot-ISO-reboot. Uitkomst niet uitgevoerd. |
| [LI18] | Anaconda release notes, Anaconda-team; levende documentatie, datum onbekend | F44 WebUI-keyboard; F45 netwerkpagina/reboot als verschillen. F45/46 zijn geen F44-features. |
| [LI19] | gnome-initial-setup.c, GNOME/Red Hat; tag 50.1, datum onbekend | `page_table`, vendor skip, ouderlijk toezicht conditioneel. Upstream kan afwijken van distributiepatches. |
| [LI20] | gis-privacy-page.c, GNOME/Red Hat; tag 50.1, datum onbekend | Locatie-/rapportageswitch initialisatie, ABRT-afhankelijkheid. Geen provider-/telemetrieomvangclaim. |
| [LI21] | gis-account-page.c, GNOME/Red Hat; tag 50.1, datum onbekend | Lokaal als default, enterprise/netwerkvariant. Geen verplichte consumentenaccount. |
| [LI22] | gis-account-page-local.c, GNOME/Red Hat; tag 50.1, datum onbekend | Admin-default en ouderlijk-toezichtuitzondering. Buildvoorwaarden blijven relevant. |
| [LI23] | gis-software-page.c, GNOME/Red Hat; tag 50.1, datum onbekend | Alleen met `fedora-third-party`; optionele repositories. Niet kiezen kan latere herhaalde vraag geven. |
| [LI24] | Kubuntu downloads, Kubuntu; onbekend, 26.04.1 | Actuele LTS en ondersteuning april 2029. 26.10 afzonderlijke interimrelease. |
| [LI25] | Kubuntu 26.04 release notes, Kubuntu; 2026-04-23 | Plasma 6.6, Qt 6.10.2; geen volledige installatiewalkthrough. |
| [LI26] | kubuntu-26.04.1-desktop-amd64.manifest, Ubuntu/Kubuntu; datum onbekend | Exacte imagepakketversies. Tekstmanifest, ISO niet gedownload. |
| [LI27] | calamares-settings-ubuntu_26.04.12, Kubuntu/Lubuntu; changelog Aaron Rainbolt, 2026-04-21 | `kubuntu/settings.conf`, modules welcome/partition/users/pkgselect_context; common locale/finished/pkgselect. Bronarchief in geheugen gelezen; UI-aanwezigheid niet uit losse actie afgeleid. |
| [LI28] | kubuntu-installer-prompt_26.04.3, Kubuntu; changelog Aaron Rainbolt, 2026-08-20 | `installerprompt.ui/.cpp`: taal, netwerk, Try/Install, terugkeer naar live. Geen bewijs over latere Calamares-schermen. |
| [LI29] | CockpitStorageIntegration.jsx, Anaconda WebUI; tag 68, datum onbekend | Expliciete waarschuwingen: wijzigingen direct, geen undo. Geavanceerde zijroute. |
| [LI30] | Fedora 44 Workstation Installation on Framework Laptop 16, Matt Hartley e.a./Framework; 2026-09-30 | Stappen 6–8 bevatten interne inconsistenties; pagina vermeldt ongepubliceerde status. Alleen bronkritiek, geen normatieve route. |
| [LI31] | page_usersetup.ui, Calamares; v3.3.14, datum onbekend | `textBoxHostname`; zichtbaarheid in [UsersPage.cpp](https://raw.githubusercontent.com/calamares/calamares/v3.3.14/src/modules/users/UsersPage.cpp) en `EtcHostname`-default in [users/Config.cpp](https://raw.githubusercontent.com/calamares/calamares/v3.3.14/src/modules/users/Config.cpp). Combineer met Kubuntu-config en patches. |
| [LI32] | ChoicePage.cpp, Calamares; v3.3.14, datum onbekend | `setupActions`: verkleinbaarheid/vervangbaarheid, ESP, tabeltype; [partition/Config.cpp](https://raw.githubusercontent.com/calamares/calamares/v3.3.14/src/modules/partition/Config.cpp) heeft manual-default. Vier routes niet altijd zichtbaar. |
| [LI33] | calamares_3.3.14-0ubuntu25.debian.tar.xz, Ubuntu/Kubuntu; archiefdatum 2026-03-18 | Patchreeks verandert genoemde hostname/keuzelogica niet; `fix-checkbox-warning-confusion.patch` wijzigt wachtwoordlogica. Daarom geen afdwingingsclaim uitsluitend uit configuratiesleutels. |

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
