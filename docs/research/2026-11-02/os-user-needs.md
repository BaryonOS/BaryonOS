# Gebruikersbehoeften — vergelijkende synthese van fase 2

Datum: 2 november 2026. Status: brononderzoek en analytische weging; geen representatieve klachtenenquête, productbesluit of implementatie. [Methode](README.md), [bronnenregister](sources.md), [repositorybasis](repository-baseline.md).

## Wat de bronnen gezamenlijk ondersteunen

[ANALYSE] De best verdedigbare onderzoeksrichting is minder onzekerheid tijdens dagelijkse taken: weten waar iets staat, het kunnen terugvinden, toegang en wijziging begrijpen, werk behouden en fouten herstellen. Meerdere bronnen geven hiervoor concrete scenario's. Geen bron bewijst dat één nieuw OS deze combinatie al beter levert, of dat hiermee massale overstap volgt.

[BRON] Gevestigde platformen ontwikkelen deze gebieden zelf: Microsoft erkent feedback over zoeken en onderbrekingen; Apple biedt Spotlight-acties en tiling; GNOME/KDE verbeteren bestandstoestand, toegankelijkheid en taakbediening; Android voegt onder meer meldingsgroepering en adaptieve vensters toe. Een ontbrekende of aangekondigde functie moet per versie, apparaat en regio worden beoordeeld. [WM-03][WM-19][WM-20][LM-01][LM-03][LM-11][LM-12]

De volledige aspectentabellen staan in [Windows/macOS](windows-macos-needs.md) en [Linux/Android](linux-mobile-needs.md). Bron-ID's in deze synthese linken naar hun oorspronkelijke bron; methode en beperkingen blijven in de dossiers staan.

## Herhaling, impact en relevantie

**Leesregel:** herhaling telt onafhankelijke bronfamilies binnen deze doelgerichte selectie, geen incidenten of gebruikers. “Sterk” kan een technische grens betekenen; de behoeftefrequentie blijft dan onbekend. Alle impact-, haalbaarheids- en onderscheidsoordelen hieronder zijn **[ANALYSE]**, geen gemeten voorkeur of definitieve fase-6-prioriteit.

| Probleem / mogelijke behoefte | Herhaling en onderzoekssterkte | Impact bij optreden | Europese relevantie | Haalbaarheid in nieuw-os | Mogelijk onderscheid |
|---|---|---|---|---|---|
| Betrouwbaar terugvinden met zichtbare scope | Microsoft-feedback, Apple-support/gebruikersmelding, onafhankelijke GNOME- en KDE-meldingen. Middel voor frictie; prevalentie onbekend. [WM-03][WM-09][WM-10][LM-04][LM-05] | Hoog als werk onvindbaar lijkt | Taal, gegevenslocatie en offline bronnen | Scope/status afgebakend; volledige toegangsveilige inhoudsindex is grote dienst | Middel: vooral samenhang en betrouwbaarheid |
| Bestandslocatie, synchronisatie en back-up begrijpen | Microsoft/Apple-semantiek plus GNOME/Google-herstelvoorwaarden. Sterk voor concrete technische grenzen; slechts één individueel verwarringssignaal, dus zwak/open voor gebruikersbegrip. [WM-05][WM-06][WM-13][WM-14][LM-13][LM-20] | Hoog: verlies, onbedoeld wissen of verkeerde herstelverwachting | Overdraagbaarheid, lokaal/NAS/providerkeuze | Uitleg en status klein; volwaardige sync, herstel en export groot | Hoog indien hele keten aantoonbaar werkt |
| Rechten begrijpen en intrekken zonder taakverlies | Twee originele studies met verschillende teams, plus portals/kiezers. Middel; populaties en meetjaren beperkt. [LM-08][LM-14][LM-15][LM-16] | Hoog voor gevoelige taken | Begrijpelijke gegevensregie; geen complianceclaim | Bestaande capabilitybasis, maar inventaris/issuer/revocationketen vergen native integratie | Middel/hoog door taak, duur en gevolg te verbinden |
| Werkende toetsenbord-/hulpmiddelenroutes behouden | NV Access, AppleVis, GNOME en W3C zijn verschillende onderhouds-/richtlijnfamilies. Sterk voor concrete regressies/voorwaarden, geen frequentietelling. [WM-08][WM-11][DT14][DT20] | Blokkerend voor getroffen gebruikers | Inclusie, meertaligheid en verschillende invoer | Appsemantiek bestaat; OS-brede screenreader, shaping en invoer zijn groot werk | Kwaliteitsvoorwaarde; geen exclusief nieuw idee |
| Essentiële apps en apparaten behouden | Adobe, Microsoft, LibreOffice, Valve en migratiecase. Sterk voor genoemde compatibiliteitsgrenzen. [WM-16][WM-17][EA-09][EA-10][EA-13] | Blokkerend bij een onmisbare taak | Werk, onderwijs, games, toegankelijkheid en levensduur | Zeer groot; zelfstandige kernel erft geen Linux/Windows-ecosysteem | Voorwaarde vóór onderscheid relevant wordt |
| Updates en onderbrekingen kunnen voorspellen | Windows-feedback, Snap-planning, Pixel-supportbeleid en twee verschillende notificatiestudies. Middel; verschillende uitkomstmaten. [WM-03][LM-07][LM-17][DT17][DT21] | Middel/hoog, afhankelijk van taak | Controle en langdurig onderhoud | UI-beleid beperkt; herstelbare updates vereisen bestaande slot-/verificatieketen | Laag voor een schakelaar; hoger voor samenhang |
| Werkplek en taakcontext betrouwbaar hervatten | Eén PowerToys-regressie plus bestaande workspace-/snapfeatures. Zwak als brede wens, concrete casus. [WM-07][DT07][DT08] | Middel/hoog voor multitaskers | Verschillende beroepen/invoer; niet specifiek Europees | Twee-appbasis bestaat; dynamische vensters/sessieherstel vragen uitbreiding | Middel indien documentcontext leidend blijft |
| Herkomst en gevolgen van installatie begrijpen | Apple-notarisatie, Ubuntu-pakketkeuze, Flatpak en Android-verificatie. Sterk voor mechanismen; zwak voor algemene verwarringsfrequentie. [WM-15][LM-06][LM-08][LM-19] | Hoog bij onveilige of niet-werkende installatie | Leverancierskeuze, inspecteerbaarheid en onderhoud | Pakket-/updatecores aanwezig; volledige install/startketen nog open | Kwaliteitsvoorwaarde met open distributie |
| Optionele contextuele hulp met corrigeerbare acties | HCI-richtlijnvalidatie en bestaande AI-productrichting; geen nieuw-os- of EU-marktvalidatie. Zwak/middel. [DT11][DT16] | Taakafhankelijk; fouten kunnen hoog uitvallen | Geselecteerde context en lokale verwerking passen bij projectcontract | Echte lokale inference en beperkte actiebroker nog grote afhankelijkheden | Onbewezen; eerst vergelijken met gewone bediening |

Geen bovenstaande rij krijgt een objectief “frequentiepercentage”. Een gegenereerde volgorde zou de doelgerichte bronselectie voor een representatieve meting aanzien. Voor fase 6 zijn nog nodig: een gekozen doelgroep, echte taakobservaties, kostenraming, onderhouds-/privacy-/toegankelijkheidseffect en afhankelijkheden per voorstel.

## Dekking van de gevraagde onderwerpen

| Onderwerp | Gecontroleerde dekking | Wat onbekend blijft |
|---|---|---|
| Ontbrekende functies / omslachtige routes / aantal stappen | Zoekscope, indexherstel, Start-afhankelijkheden, installuitzonderingen en sync-stoproutes [WM-03][WM-04][WM-09][WM-15][LM-04] | Geen uniforme, live taakmeting of vergelijkbaar stappenaantal over vier OS'en |
| Controle, privacy, account, reclame en telemetrie | Windows-installatie/diagnose, Android-advertentie- en back-upstromen, originele privacybegripsstudies [WM-01][WM-02][LM-13][LM-15][LM-16][LM-18] | Geen algemene weerstandsscore; geen uniforme Android-OEM- of Linuxtelemetriepraktijk vastgesteld |
| Updates | Windows-verbeterplan, Snap hold/planning, Pixelduur, native nieuw-os-updatebouwstenen [WM-03][LM-07][LM-17][RB-01] | Plannen zijn geen brede uitrol; geen dagelijks update-effectonderzoek voor nieuw-os |
| Zoeken, instellingen en bestandsbeheer | Windows/macOS- en GNOME/KDE-dossiers, afzonderlijke huidige oplossingen | Uniforme representatieve tevredenheidsvergelijking ontbreekt |
| Multitasking en vensters | Workspace-regressie, GNOME-workspaces, Windows Snap, macOS-tiling, Android-vensters [WM-07][WM-20][DT07][DT08][LM-11] | Geen bewijs dat één model voor beginners en gevorderden het beste is |
| Notificaties | Groepering/platformfuncties plus bundel- en uitschakelstudies [LM-02][LM-12][DT17][DT21] | Desktopoverdracht, urgente uitzonderingen en optimale frequentie niet vastgesteld |
| Back-ups en synchronisatie | OneDrive, iCloud, Time Machine, Pika en Android-herstelvoorwaarden [WM-05][WM-13][WM-14][LM-13][LM-20] | Geen eigen volledige herstelproef; sync is geen onafhankelijke back-up |
| Apparaatkoppeling | KDE Connect, Apple Mirroring-regio/account, EU-portabiliteitsroutes [LM-10][WM-12][EA-11] | Nieuwe routes/API's bewijzen geen algemene huidige beschikbaarheid of nieuw-os-integratie |
| Toegankelijkheid | Regressies, semantiek, invoeralternatieven, WCAG/WCAG2ICT en font-/schaalgrenzen [WM-08][WM-11][LM-03][DT10][DT20][RB-01] | Menselijke en systeembrede assistieve acceptatie open |
| Installatie en pakketbeheer | Meerdere bestaande distributiemodellen en rechtenkiezers [WM-15][LM-06][LM-08][LM-19] | Onvoldoende onafhankelijk onderzoek om alle modellen op gebruiksgemak te rangschikken |
| Gewone versus gevorderde gebruikers | Onderzoekssegmenten, update-/zoekdiagnostiek, sneltoetsen en werkplekherstel [WM-04][WM-07][LM-04][LM-07] | Geen geïnterviewde persona's of veronderstelde voorkeuren uit demografie |
| Software uit andere platformen overnemen | Bestaande referentiepatronen hieronder; originele individuele wensen waar aanwezig | Geen representatieve lijst “gebruikers willen functie X van OS Y” |

## Bestaande patronen die als referentie kunnen dienen

Dit zijn **[HYPOTHESE] ontwerp-/toetsscenario's**, geen bewezen overnamewensen of opdracht om een OS te kopiëren.

| Referentiepatroon | Bron en bestaande functie | Te onderzoeken gebruikersresultaat |
|---|---|---|
| Vindbare acties naast zoekresultaten | macOS Spotlight-acties [WM-19] | Minder omwegen, met zichtbare context en begrensde gevolgen |
| Begrijpelijke, selectieve gegevenstoegang | Flatpak-portals en Android-documentkiezer [LM-08][LM-14] | Eén gekozen object gebruiken zonder algemene bestandsrechten |
| Zichtbare herstelgeschiedenis | Time Machine en Materiaal als verschillende technische modellen [WM-14][RB-01] | Weten wat herstelbaar is en een fout ongedaan maken zonder onbedoelde overschrijving |
| Vensters groeperen en hervatten | Windows Snap/GNOME-workspaces/macOS-tiling [DT07][DT08][WM-20] | Twee bronnen vergelijken en de juiste taak terugvinden; ook toetsenbordbediening |
| Expliciete gekoppelde apparaten | KDE Connect/Handoff [LM-10][DT04] | Werk op een tweede apparaat voortzetten zonder stille gegevensverruiming |
| Overzicht van feitelijke toegang | Android-privacydashboard [DT06] | Begrijpen welke app toegang gebruikte en wat intrekken precies stopt |

## Ontwerpvoorwaarden voor de volgende fasen

[ANALYSE] Europese waarden worden concreet door begrijpelijke gegevensbestemming, providerwissel/export, lokale kernwerking, herstel, toegankelijke taal/invoer en onderhoud. Europese herkomst alleen bewijst geen van die eigenschappen. Kosten, veiligheid en bruikbaarheid moeten op het werkelijke product worden getoetst. De Eurobarometer is daarvoor een houdingmeting over diensten, geen marktomvang voor nieuw-os. [EA-01–EA-04]

[HYPOTHESE] Voor de latere visie is één keten kansrijk: **vinden → begrijpen waar en met welke rechten → handelen → bewaren → herstellen/hervatten**. Toets die ook zonder AI. Behoud namen, export en vertrouwde basisbediening wanneer die migratie vergemakkelijken. Leg bij ieder concept vast welk deel bronbevinding, ontwerpskeuze, technisch experiment of latere ambitie is.

Nog niet doen op basis van dit onderzoek: decoratieve glaseffecten als waardeclaim, een verplichte assistent, algemene app-/drivercompatibiliteit beloven, een nieuw account/cloudvereiste invoeren, of kleine bestaande proefgrenzen als volwaardige productcapaciteit presenteren. Dit zijn afwijzingen van onvoldoende onderbouwde claims, geen verlaging van de bestaande productambitie.

## Verwijzingen

De WM-/LM-/DT-/EA-ID's verwijzen naar de registers in deze map. De volgende direct gekoppelde selecties maken de synthese zelfstandig navigeerbaar; beperkingen staan steeds in het betreffende dossier.

[WM-01]: https://www.microsoft.com/en-us/windows/windows-11-specifications
[WM-02]: https://support.microsoft.com/en-us/windows/privacy/diagnostics-feedback-and-privacy-in-windows
[WM-03]: https://blogs.windows.com/windows-insider/2026/03/20/our-commitment-to-windows-quality/
[WM-04]: https://blogs.windows.com/windows-insider/2026/05/15/improving-windows-quality-making-taskbar-and-start-more-personal/
[WM-05]: https://support.microsoft.com/en-gb/onedrive/back-up-your-folders-with-onedrive?nochrome=true
[WM-06]: https://learn.microsoft.com/en-us/answers/questions/5557469/why-did-onedrive-remove-my-desktop-icons
[WM-07]: https://github.com/microsoft/PowerToys/issues/37634
[WM-08]: https://api.nvaccess.org/releases/2026.2/documentation/changes.html
[WM-09]: https://support.apple.com/en-us/102321
[WM-10]: https://discussions.apple.com/thread/256145579
[WM-11]: https://applevis.com/blog/macos-tahoe-new-features-changes-improvements-bugs-blind-deafblind-low-vision-users
[WM-12]: https://support.apple.com/en-us/120421
[WM-13]: https://support.apple.com/en-us/109344
[WM-14]: https://support.apple.com/en-us/104984
[WM-15]: https://support.apple.com/en-us/102445
[WM-16]: https://helpx.adobe.com/download-install/apps/system-requirements/creative-cloud-requirements.html
[WM-17]: https://support.microsoft.com/en-us/office/system-requirements/system-requirements-for-microsoft-365-for-business-education-and-government-use
[WM-19]: https://www.apple.com/os/pdf/All_New_Features_macOS_Tahoe_Sept_2025.pdf
[WM-20]: https://www.apple.com/newsroom/2024/06/macos-sequoia-takes-productivity-and-intelligence-on-mac-to-new-heights/
[LM-01]: https://release.gnome.org/51/
[LM-02]: https://release.gnome.org/48/
[LM-03]: https://kde.org/announcements/plasma/6/6.8.0/
[LM-04]: https://discourse.gnome.org/t/search-everywhere-cant-find-files/33037
[LM-05]: https://discuss.kde.org/t/cifs-share-freezes-taskbar-when-nas-offline/42466
[LM-06]: https://ubuntu.com/desktop/docs/en/24.04/how-to/software/install-an-application/
[LM-07]: https://snapcraft.io/docs/how-to-guides/manage-snaps/manage-updates/
[LM-08]: https://docs.flatpak.org/en/latest/sandbox-permissions.html
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
[DT04]: https://www.apple.com/newsroom/2014/10/16OS-X-Yosemite-Available-Today-as-a-Free-Upgrade/
[DT06]: https://blog.google/products-and-platforms/platforms/android/android-12/
[DT07]: https://release.gnome.org/40/
[DT08]: https://blogs.windows.com/windowsexperience/2021/06/24/introducing-windows-11/
[DT10]: https://www.w3.org/TR/2024/REC-WCAG22-20241212/
[DT11]: https://www.apple.com/newsroom/2024/06/introducing-apple-intelligence-for-iphone-ipad-and-mac/
[DT14]: https://release.gnome.org/50/
[DT16]: https://www.microsoft.com/en-us/research/wp-content/uploads/2019/01/Guidelines-for-Human-AI-Interaction-camera-ready.pdf
[DT17]: https://doi.org/10.1016/j.chb.2019.07.016
[DT20]: https://www.w3.org/TR/2025/NOTE-wcag2ict-22-20251211/
[DT21]: https://pure.uva.nl/ws/files/245971989/Beyond_the_Buzz.pdf
[EA-09]: https://help.libreoffice.org/latest/en-US/text/shared/guide/ms_user.html
[EA-10]: https://partner.steamgames.com/doc/steamhardware/proton
[EA-11]: https://digital-markets-act.ec.europa.eu/factsheet-how-dma-making-smartphones-better-interoperability-and-data-portability-case-studies-2026-05-11_en
[EA-13]: https://www.schleswig-holstein.de/DE/landesregierung/ministerien-behoerden/I/Presse/PI/2025/cds/251204_cds_open-source
[EA-01–EA-04]: european-adoption-barriers.md#oorspronkelijke-eurobarometer-gecontroleerd
[RB-01]: repository-baseline.md
