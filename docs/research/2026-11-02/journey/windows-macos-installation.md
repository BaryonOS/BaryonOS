# Installatiejourney Windows en macOS

De installatie bestaat bij beide platforms uit verschillende fasen; schijfwijzigingen, accountkeuzes en privacykeuzes horen daardoor niet in één veronderstelde universele schermvolgorde.

## Afbakening en broncontrole

Onderzoeksdatum: **2 november 2026**. Methode: officiële release-, installatie-, herstel-, toegankelijkheids- en beheerdocumentatie doorzocht en gelezen. Dit is documentenonderzoek, **geen uitgevoerde installatie, gebruikerstest of eigen screenshotregistratie**. `[BRON]` betekent beschreven leveranciersgedrag; `[ANALYSE]` is de duiding binnen dit onderzoek; **onvoldoende vastgelegd** betekent geen voldoende precies bewijs voor deze release/route. N/A geldt uitsluitend voor de gekozen route.

- **Windows:** Windows 11 **26H2**, Home/Pro voor persoonlijk gebruik, x64, algemene beschikbaarheid 29 september 2026. De downloadpagina noemt 26H2; geen ISO gedownload, dus exacte mediabuild en hash niet vastgesteld. Hoofdroute: schone installatie vanaf USB. Vergelijkingsroute: upgrade/herinstallatie vanuit draaiend Windows. 25H2 blijft ondersteund; 26H1 is een aparte hardwaregerichte release, geen algemene upgrade vanaf 24H2/25H2. [WI01] [WI02]
- **macOS:** **27.2 Golden Gate**, Apple silicon, Recovery en daarna Setup Assistant bij een nieuw/leeg gebruikersvolume. Apple's releaseoverzicht en beveiligingsregister dateren 27.2 op **26 oktober 2026**, vóór de onderzoeksdatum; dit is de reguliere releasevermelding, geen beta. Hetzelfde register vermeldt Tahoe **26.7.2** en Sequoia **15.8.2**. Dat bewijst uitgebrachte beveiligingsupdates, geen gegarandeerde toekomstige ondersteuningsduur. [WI14] [WI15]
- **Intel-variant:** Tahoe **26.7.2** uitsluitend op compatibele modellen, bijvoorbeeld MacBook Pro 16-inch 2019. Recovery kan een andere, eerder geïnstalleerde of oorspronkelijk meegeleverde versie aanbieden. “Recovery starten” garandeert dus niet installatie van de hier genoemde laatste patch. [WI16] [WI17]

**Bronconflict:** Microsofts algemene schermlezergids bevat oudere Setup-schermen en een offline/lokaal-accountpad. Dat is geen bewijs dat een actuele 26H2 Home/Pro-consumenteninstallatie zo werkt: de actuele systeemeisen vereisen internet en een Microsoft-account bij de eerste persoonlijke configuratie. De gids dient hier alleen als begrensd bewijs voor beschreven toetsenbordinteracties en aanvullende OOBE-opties. [WI05] [WI06]

## Vergelijkingsregister: 31 aspecten

Alle cellen zijn `[BRON]`, behalve expliciet gemarkeerde analyse of bewijsleemten. **Deze nummering is geen chronologische schermvolgorde.** Windows “Setup” schrijft het OS; “OOBE” configureert de eerste gebruikerservaring. macOS “Recovery” herstelt/installeert; “Setup Assistant” configureert gebruikers en diensten. [WI03] [WI04] [WI17] [WI20]

| Nr. | Aspect | Windows 11 26H2: USB → OOBE | macOS: Recovery → Setup Assistant |
|---:|---|---|---|
| 1 | Bootmedium/recovery | Installatie-USB, via firmwarebootmenu; fabrikant bepaalt toets. Aanmaken USB wist de USB-inhoud. [WI02] | Apple silicon: aan/uit ingedrukt → Options. Intel: Command-R; internetvarianten hieronder. [WI18] |
| 2 | Eerste zichtbare installer | Eerst taalinstellingen. [WI03] | Eerst opstartopties/Recovery, eventueel volume en bestaande beheerder ontgrendelen; daarna hulpprogramma's en Reinstall macOS. [WI19] |
| 3 | Taal | Setup-taal; eventueel opnieuw in OOBE. [WI03] [WI04] | Setup Assistant begint met taal. Na Apple-silicon-wissen ook taal tijdens heractivatie. [WI20] [WI27] |
| 4 | Regio/tijdzone | Regio in OOBE; automatische tijdzone hangt samen met locatieservice. Geen universeel afzonderlijk tijdzonescherm bewezen. [WI04] [WI12] | Exact regio-/tijdzonescherm en positie onvoldoende vastgelegd in de gekozen 27-handleiding. [WI20] |
| 5 | Toetsenbord | Setup-layout; eventueel tweede layout in OOBE. [WI03] [WI05] | Exact invoerbronnen-/layoutscherm onvoldoende vastgelegd. [WI20] |
| 6 | Toegankelijkheid | Gids beschrijft Tab, pijlen, spatie en Enter. Volledige schermlezer-/audio-ondersteuning **vóór wissen** op 26H2-media niet bevestigd. [WI05] | Setup Assistant biedt toegankelijkheidskeuzes; dat is ná installatie. Volledige Recovery-schermlezerdekking **vóór wissen** niet bevestigd. [WI19] [WI20] |
| 7 | Netwerk | Home/Pro persoonlijk: internet vereist in initiële configuratie. Ontbrekende netwerkdriver kan deze afhankelijkheid problematisch maken; laatste zin is [ANALYSE]. [WI06] | Recovery-download vereist internet; na Apple-silicon-wissen ook activatie. Sommige bedrijfs-/captive-netwerken werken niet voor Recovery. [WI17] [WI27] |
| 8 | Try versus Install | Geen live-desktopproef beschreven. [WI03] | N/A als live-desktopproef: Recovery biedt herstelhulpmiddelen; de gebruiker kiest herinstallatie. [WI19] |
| 9 | Hardwarecontrole | Compatibele CPU, TPM 2.0, UEFI/Secure Boot-capabel, RAM/opslagvereisten. Installation Assistant controleert compatibiliteit; geen bewijs dat iedere USB-controle identiek is. [WI02] [WI06] | Modelcompatibiliteit vooraf controleren; Software Update biedt passende software. Geen universeel checklistscherm in Recovery bewezen. [WI16] [WI29] |
| 10 | Opslagapparaat | Doelpartitie selecteren. [WI03] [ANALYSE] Verifieer fysieke schijfidentiteit. | Installer kiest doelvolume; bij HD en HD–Data noemt Apple HD. Ontgrendeling kan nodig zijn. [WI17] |
| 11 | Partitionering | Clean-route: doelpartities verwijderen → ongealloceerde ruimte. [WI03] | Wissen/volume-indeling gebeurt afzonderlijk in Disk Utility; Apple-silicon-route gebruikt APFS. Extra APFS-volume is alternatief. [WI27] [WI26] |
| 12 | Behouden/vervangen | Clean wist bestanden/apps/instellingen/OEM-aanpassingen; in-place biedt behoudopties. [WI03] | Herinstallatie zonder wissen bewaart apps/persoonlijke gegevens; wissen is aparte destructieve handeling. [WI17] [WI27] |
| 13 | Dual boot | Geen begeleid dual-bootpad in deze clean-route. BCDBoot ondersteunt meerdere Windows-installaties; Linux-co-existentie hiermee niet bewezen. [WI13] | Extra compatibel macOS op APFS-volume of externe schijf mogelijk; Apple waarschuwt tegen langdurig behoud van sterk verouderde versies. [WI26] |
| 14 | Disk-encryptie | Op geschikte hardware activeert accountconfiguratie Device Encryption; herstelkey bij Microsoft-/werk-/schoolaccount. Geen universeel schijfencryptiescherm vóór installatie. [WI07] | FileVault/persoonlijke herstelkey: zie nadere uitleg. Hardware-encryptie alleen is niet dezelfde wachtwoordbescherming. Paneel kan aanwezig zijn; positie varieert. [WI25] [WI23] |
| 15 | Gebruikersaccount | In OOBE; eerste reguliere Setup-account krijgt beheerdersgroepslidmaatschap. Organisatiebeheer is aparte variant. [WI04] [WI09] | Lokaal computeraccount en wachtwoord; eerste gebruiker is beheerder. Apple Account is afzonderlijk. [WI20] [WI22] |
| 16 | Apparaatnaam/hostname | Apparaatnaam beschreven in OOBE-gids; exacte 26H2-schermpositie niet gepind. [WI05] | Afzonderlijk hostnamescherm niet aangetoond. [WI20] |
| 17 | Wachtwoord/PIN/biometrie | Microsoft-aanmelding, vervolgens Hello/PIN; biometrie hardwareafhankelijk. Beheertaken vragen UAC-toestemming of beheerreferenties. [WI05] [WI06] [WI08] | Computerwachtwoord; Touch ID alleen waar ondersteund. Wachtwoord blijft relevant voor beheertaken. [WI20] [WI23] [WI29] |
| 18 | Online versus lokaal | Standaard Home/Pro-persoonlijk vereist Microsoft-account. Oudere offline-instructies zijn geen universele actuele optie. [WI06] [WI05] | Computeraccount lokaal; Apple Account kan later in System Settings worden toegevoegd. Netwerkvereisten verdwijnen daardoor niet. [WI21] [WI17] |
| 19 | Privacykeuzes | OOBE vermeldt maximaal zeven keuzes, niet voor iedereen dezelfde. Daarna aanpasbaar; exacte defaults niet onafhankelijk vastgelegd. [WI04] [WI11] | Data and Privacy-informatie, locatie en analytics zijn onderscheiden onderwerpen/panelen; geen bewijs voor identieke zichtbaarheid/defaults bij iedere Mac. [WI23] |
| 20 | Telemetrie | Verplichte en optionele diagnostiek onderscheiden; optioneel uit betekent niet dat verplichte diagnostiek stopt. [WI11] | Mac Analytics volgens Apple met toestemming; apart delen met ontwikkelaars. Later uitschakelbaar; dit bewijst niet dat alle Apple-diensten gegevensloos zijn. [WI24] |
| 21 | Locatie | Beheer via Privacy & security → Location; precieze OOBE-default/paneelvariant onvoldoende vastgelegd. [WI12] [WI04] | Location Services behoort tot afrondende Setup Assistant-keuzes. Exacte default niet vastgesteld. [WI20] |
| 22 | Updates tijdens installatie | Kritieke OOBE-patches worden na internetverbinding gedownload; nieuwere OS-versie kan aangeboden worden. Niet alle updates zijn dus één verplicht blok. [WI10] | Recovery-versie kan achterlopen; daarna Software Update. Initiële configuratie kan een verplichte update vereisen. [WI17] [WI23] [WI29] |
| 23 | Drivers/firmware | Kritieke drivers tijdens OOBE; aanvullende fabrikantdrivers/Surface-firmware apart controleren. Geen universele OEM-set gegarandeerd. [WI04] [WI02] | Geen losse driverkeuzestap beschreven in deze Recovery-route. Firmwareverloop per model hier onvoldoende vastgelegd. [WI17] [WI19] |
| 24 | Optionele software | OneDrive, 365-aanbod/OEM-registratie kunnen verschijnen; gids beschrijft overslaan van telefoonopties/Game Pass. Account/uitrolafhankelijk. [WI04] [WI05] | Migratie/as-new; onder meer Siri, Screen Time en Apple Intelligence bij afronding. Geen algemeen pakketselectiescherm aangetoond. [WI20] |
| 25 | Samenvatting vóór installatie | Verlieswaarschuwing → partitieverwijdering → Ready to install. [WI03] [ANALYSE] Verwijdering is al destructief. | Wissen bevestigt doelnaam/formaat/volumegroep en Erase Mac; geen gezamenlijke laatste samenvatting van alle latere accountkeuzes bewezen. [WI27] |
| 26 | Voortgang | Installeren/herstarts beschreven; precieze indicator/ETA niet vastgelegd. [WI03] | Voortgangsbalk, herstarts en tijdelijk zwart scherm mogelijk; geen gemeten doorlooptijd. [WI17] |
| 27 | Reboot | Tijdens installatie en eventueel OOBE; pagina's kunnen na netwerkverlies terugkomen. [WI03] [WI04] | Na wissen/reactivatie én tijdens herinstallatie mogelijk. [WI27] [WI17] |
| 28 | Medium verwijderen | Interne schijf moet starten; gewijzigde bootvolgorde herstellen voorkomt opnieuw Setup. Geen universeel verwijderprompt bewezen. [WI02] | N/A voor ingebouwde Recovery: geen USB-installatiemedium in dit pad. [WI19] |
| 29 | Eerste boot | Configuratiewizard, nog geen desktop. [WI03] | Na lege installatie kan Setup Assistant volgen; bij behoudinstallatie niet per definitie opnieuw onboarding. [WI17] |
| 30 | Tweede configuratiefase | OOBE: regio/netwerk/account/Hello/privacy/diensten; inhoud en aantal stappen variëren. [WI04] | Setup Assistant configureert account, migratie en voorzieningen; MDM kan panelen overslaan of extra beheer tonen. [WI20] [WI23] |
| 31 | Eerste echte desktop | Na OOBE; gedocumenteerde eerste quiet period beperkt automatisch opkomende appvensters, met Start als uitzondering. [WI04] | Na afrondende Welcome is Mac gebruiksklaar; werkende apps, herstel en apparaten hier niet getest. [WI20] |

## Werkelijke routevolgorde en varianten

### Windows: schone USB-installatie

1. Voorbereiding: back-up, passende editie/licentie, hardwarecontrole; USB maken en daarvan starten. [WI02] [WI06]
2. **Setup:** taal → toetsenbord → installatiekeuze/verliesbevestiging → zo nodig productsleutel en editie → licentie → doelpartities → Ready to install → installeren/herstarts. Een ingebouwde productsleutel kan de sleutelvraag overslaan. [WI03]
3. **OOBE:** taal/regio/layout → netwerk/kritieke updates → toepasselijke licentie-/updatepagina's → account/herstel → Hello → privacy → toepasselijke diensten/aanbiedingen → desktop. Dit is Microsoft's typische, niet uitputtende volgorde; apparaatnaam en aanvullende opties komen uit de minder releaseprecieze gids. [WI04] [WI05]

**Upgrade/in-place:** start `setup.exe` vanuit het bestaande Windows en bevestig UAC; kies behoud van apps/bestanden/instellingen, alleen bestanden of niets. Daarna installatie/herstarts. Dit is een andere route dan USB booten en doelpartities wissen. Installation Assistant is een aanvullend x64-upgradepad. [WI03] [WI02]

**OEM/organisatie:** voorgeïnstalleerde pc begint bij OOBE; bedrijfsaanmelding/beheer kan de account- en toestemmingsroute wijzigen. Geen generieke offline-bypass of organisatieprocedure als standaard consumentenroute overgenomen. [WI05] [WI06]

### macOS: Apple silicon Recovery

1. Uitschakelen → aan/uit vasthouden → Options/Continue → eventueel bestaand volume/beheerder ontgrendelen → Recovery. [WI18] [WI19]
2. **Behoudroute:** rechtstreeks Reinstall macOS. **Wisroute:** eerst Disk Utility en doelvolume/volumegroep bevestigen. Apple's wisroute verwijdert gegevens, ontkoppelt Bluetooth en kan Apple Account-verificatie vragen. Daarna herstart → taal → internetactivatie → terug naar Recovery. [WI17] [WI27]
3. Reinstall macOS → doelvolume → installatie met voortgang/herstarts. Recovery bepaalt de aangeboden versie; eventueel later bijwerken. [WI17]
4. Op leeg/nieuw systeem: **Setup Assistant** start met taal; behandelt migratie/as-new, toegankelijkheid, verbinding en computeraccount, Apple Account en afrondende voorkeuren. De handleiding legt geen volledige vaste volgorde tussen alle panelen vast. Welcome rondt af. [WI20]

**Intel:** Command-R voor Recovery van de laatst geïnstalleerde versie; Option-Command-R kan de nieuwste compatibele macOS aanbieden; Shift-Option-Command-R de oorspronkelijke of dichtstbij beschikbare versie. Disk Utility-wissen heeft een apart Intel-pad, met aanbevolen formaat en volumegroep; geen Apple-siliconprocedure blind overnemen. [WI17] [WI18] [WI28]

**Upgrade zonder wissen:** System Settings → General → Software Update → compatibele update/upgrade → beheerderwachtwoord → voortgang/herstarts. Dit is geen verplichte nieuwe Setup Assistant of nieuwe accountcreatie. [WI29]

**Dual boot:** extra macOS-volume vereist aparte installatie en eerste accountconfiguratie; iedere installatie heeft eigen updates. Windows op Intel via Boot Camp en alternatieve OS-installatie op Apple silicon vallen buiten het hier geverifieerde pad. [WI26]

## Beslissingen met gevolgen en bewijsgrenzen

- **Encryptie/herstel:** Windows Device Encryption is hardware- en accountafhankelijk; bij een lokaal account gaat automatische activering volgens Microsoft niet aan. Herstelinformatie hoort bij het gekoppelde account. Apple beschrijft FileVault **vanaf macOS 26.4** standaard ingeschakeld met afmeldmogelijkheid, een herstelkey van 24 tekens en opslag in de sleutelhanger/Passwords. Met iCloud Keychain synchroniseert deze; zonder iCloud moet de getoonde key veilig bewaard worden. Op Apple silicon/T2 blijft opslag ook zonder FileVault versleuteld, maar zonder dezelfde wachtwoordbinding. Dit zijn verschillende herstel- en vertrouwensmodellen; exacte paneeldefaults per testapparaat blijven te controleren. [WI07] [WI25]
- **Beheerrechten:** beheerderslidmaatschap betekent bij Windows niet dat iedere app verhoogd draait: UAC maakt onderscheid tussen gewone uitvoering en elevatie. Ook Mac maakt een eerste beheerder aan; die keuze is niet gelijk aan toestemming voor iedere clouddienst. [WI08] [WI22]
- **[ANALYSE] Destructief controlepunt:** een samenvatting ná partitieverwijdering kan de eerdere verwijdering niet ongedaan maken. Voor een latere nieuw-os-specificatie is het relevante toetsmoment daarom vóór de eerste opslagmutatie, niet uitsluitend vóór het kopiëren van het OS. Geen productbesluit of implementatie in dit dossier.
- **[ANALYSE] Nog nodig:** afgebakende VM-installaties met gepinde media/build, schermregistratie en toetsenbord-/schermlezertest vóór wissen. Met name macOS regio/layout/hostname, Windows actuele locatie-defaults, firmwarefouten en offline-netwerkherstel blijven open. Leveranciersdocumentatie bewijst geen probleemfrequentie, taakduur of bruikbaarheid voor echte deelnemers.

## Bronregister

Alle bronnen hieronder zijn **primaire leveranciersdocumentatie**, geraadpleegd **2026-11-02**. “Geen datum” betekent geen publicatiedatum vastgesteld, geen claim dat de pagina nieuw is. Dynamische pagina's zijn geen bevroren installatie-opname. Enkele Apple-URL's gaven tussentijds HTTP 429; alleen daadwerkelijk leesbare pagina-inhoud/PDF-passages onderbouwen claims. Zoekfragmenten en externe kopieën zijn niet als zelfstandig bewijs gebruikt.

| ID | Titel/organisatie en oorspronkelijke datum of versie | Gelezen passage; beperking |
|---|---|---|
| WI01 | [Windows 11 release information][WI01] — Microsoft; actuele releasetabel, 26H2 GA 2026-09-29 | Ondersteuning en 26H1-uitzondering; geen ISO-identificatie. |
| WI02 | [Download Windows 11][WI02] — Microsoft; 2026 Update/26H2, geen paginadatum | Assistant, media en bootvolgorde; enkele generieke Setup-termen. |
| WI03 | [Reinstall Windows with the installation media][WI03] — Microsoft; Windows 11, geen datum | Clean/in-place, wissen en Ready to install; geen gepinde mediabuild. |
| WI04 | [Customize the out-of-box experience][WI04] — Microsoft; bijgewerkt 2026-03-13 | Typische OOBE, herstarts, quiet period; expliciet niet uitputtend. |
| WI05 | [Use a screen reader to set up Windows][WI05] — Microsoft; geen datum/releasepin | Toetsenbord, naam/PIN/extra's; gemengde oudere schermen en offlinepad. |
| WI06 | [Windows 11 specifications][WI06] — Microsoft; actuele systeemeisen, geen datum | Hardware, internet/account, Hello; geen volledig installatiescript. |
| WI07 | [Device Encryption in Windows][WI07] — Microsoft; Windows 11/10, geen datum | Automatische activering en sleutelopslag; geschikte hardware vereist. |
| WI08 | [How User Account Control works][WI08] — Microsoft; 2026-04-23 | Tokens/elevatie; beveiligingsmechanisme, geen OOBE-volgorde. |
| WI09 | [Local accounts][WI09] — Microsoft; 2026-04-15 | Setup-account en ingebouwde Administrator; organisatievarianten afzonderlijk. |
| WI10 | [Windows updates during OOBE in Windows 11][WI10] — Microsoft; 2026-03-13 | Kritieke patches/versieaanbod; geen waarneming actuele uitrol. |
| WI11 | [Diagnostics, feedback, and privacy in Windows][WI11] — Microsoft; Windows 11/10, geen datum | Required/Optional; geen actuele paneeldefaults. |
| WI12 | [Windows location service and privacy][WI12] — Microsoft; Windows 11-deel, geen datum | Tijdzone/locatiebeheer; geen exact OOBE-scherm. |
| WI13 | [BCDBoot Command-Line Options][WI13] — Microsoft; Windows 11-documentatie, geen datum vastgesteld | Meerdere Windows-installaties; geen Linux-dual-bootgarantie. |
| WI14 | [Find out which macOS your Mac is using][WI14] — Apple; 2026-10-26 | Laatste versienummers; geen toekomstig supportcontract. |
| WI15 | [Apple security releases][WI15] — Apple; release-items 2026-10-26 | 27.2/26.7.2/15.8.2; dynamische lijst. |
| WI16 | [macOS Tahoe 26 is compatible with these computers][WI16] — Apple; versie 26, geen datum vastgesteld | Ondersteunde modellen; geen toets van randapparatuur. |
| WI17 | [How to reinstall macOS][WI17] — Apple; 2026-09-14 | Behoud, versiekeuze, doelvolume/voortgang; geen volledige Setup Assistant. |
| WI18 | [Mac startup key combinations][WI18] — Apple; 2025-03-13 | Silicon/Intel-startmethoden; keyboard/firmware kunnen verschillen. |
| WI19 | [Use macOS Recovery on a Mac with Apple silicon][WI19] — Apple; versie 27 | Recovery starten/hulpprogramma's; geen bewezen assistieve volledige route. |
| WI20 | [Set up your MacBook Air][WI20] — Apple; 2026/macOS 27 | Setup Assistant-overzicht; geen volledige vaste paneelvolgorde. |
| WI21 | [Set up your accounts][WI21] — Apple; 2026/macOS 27 | Apple Account later toevoegen; geen belofte volledig offline installatie. |
| WI22 | [Add a user or group on Mac][WI22] — Apple; macOS 27 | Eerste gebruiker beheerder; later accountbeheer. |
| WI23 | [Manage Setup Assistant for Apple devices][WI23] — Apple; 2026-09-17 | Paneellijst, MDM/internet; beheerdocument, geen consumentendefault. |
| WI24 | [Share analytics information from your Mac with Apple][WI24] — Apple; macOS 27 | Toestemming en uitschakelen; leveranciersverklaring over gegevens. |
| WI25 | [Apple Platform Security][WI25] — Apple; augustus 2026, pp. 91, 104–106 | FileVault/default/herstel; technische beschrijving, geen schermopname. |
| WI26 | [Use more than one version of macOS on Mac][WI26] — Apple; 2026-09-15 | APFS/extern volume en beperking oud OS; uitsluitend macOS. |
| WI27 | [Use Disk Utility to erase a Mac with Apple silicon][WI27] — Apple; 2026-09-14 | Wissen/volumegroep/reactivatie; geen behoudinstallatie. |
| WI28 | [Use Disk Utility to erase an Intel-based Mac][WI28] — Apple; 2025-12-08 | Intel-wisroute; verwijst voor toepasselijke resets naar Erase Assistant. |
| WI29 | [Update macOS on Mac][WI29] — Apple; 2026-09-14 | Compatibele upgrade, beheerwachtwoord/herstarts; geen nieuwe lege installatie. |

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
