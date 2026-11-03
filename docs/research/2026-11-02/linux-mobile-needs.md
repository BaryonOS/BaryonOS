# Linux-desktops en Android: behoeften en overstaprisico's

**Conclusie:** begrijpelijke gegevensrechten, betrouwbare vindbaarheid en aantoonbaar herstel bieden meer onderbouwde aanknopingspunten dan een nieuwe visuele stijl; bestaande compatibiliteit blijft een harde overstapvoorwaarde.

Status: extern bureauonderzoek, 2 november 2026; geen productimplementatie of gebruikersvalidatie van nieuw-os. Scope: GNOME, KDE Plasma, Ubuntu en Android; iOS uitsluitend waar vergelijkend onderzoek dat ondersteunt. ChromeOS is niet afzonderlijk onderzocht. Dit document vervangt geen repositoryanalyse of technische acceptatie.

## Methode en bewijsgrenzen

Onderzoeksvragen: waar verliezen mensen overzicht of controle; welke bestaande oplossingen werken onder welke voorwaarden; welke problemen zijn overdraagbaar naar nieuw-os? Zoekstrategie: actuele releasepagina's, oorspronkelijke productdocumentatie, oorspronkelijke GNOME/KDE-gebruikersmeldingen en SOUPS-onderzoek. Zoektermen omvatten `GNOME search files`, `KDE offline NAS search`, `Snap manage updates`, `Proton anti-cheat`, `Android permissions user study`, `privacy labels comprehension`, `Android backup` en `developer verification`. Geopende pagina-inhoud is gebruikt, niet alleen zoekresultaten of automatisch gegenereerde samenvattingen.

- **[BRON]** controleerbare specificatie, productmededeling of onderzoeksresultaat; een leveranciersclaim is geen onafhankelijke prestatietest.
- **[SIGNAAL]** oorspronkelijke gebruikerservaring; geen vastgestelde prevalentie of bewezen oorzaak.
- **[ANALYSE]** synthese van de onderzoeker; inclusief toepassingsgrenzen.
- **[HYPOTHESE]** ontwerpkeuze die nog gebruikersonderzoek en technische validatie vereist.

Twintig bronnen zijn geselecteerd. Meerdere GNOME-, KDE- of Googlepagina's tellen niet als onafhankelijke gebruikerspopulaties. De twee academische onderzoeken hebben verschillende onderzoeksteams; LM-15 bevat een Google-coauteur. LM-15 verscheen in 2024 maar verzamelde data in april–mei 2022: dit is historische gedragsinformatie, geen meting van Android 17. [LM-15] [LM-16] Geen representatieve Nederlandse of Europese steekproef, overstappercentage, interview of frequentierangorde is geconstrueerd.

## Linux: aspecten en consequenties

| Aspect | Controleerbare bevinding en grens | Betekenis voor nieuw-os |
|---|---|---|
| Zoeken, bestanden, instellingen | **[SIGNAAL]** Een GNOME 46/Ubuntu 24.04-gebruiker meldt verschillen tussen map-, shell- en geïndexeerde zoekresultaten, met onduidelijke symlinkconfiguratie. Een KDE-gebruiker meldt vastlopende taakbalkzoekactie bij een offline NAS. Oorzaken en prevalentie zijn niet vastgesteld. [LM-04] [LM-05] | **[HYPOTHESE]** Toon doorzochte locaties, offline bronnen en rechten; laat één onbereikbare bron lokale resultaten niet blokkeren. Bied herstel zonder terminaldiagnose. |
| Dagelijks bestandsbeheer | **[BRON]** GNOME 51 verbetert selectiebehoud, leesbaarheidsstatus en niet-blokkerende bestandsoperaties. [LM-01] | **[ANALYSE]** Voorspelbare selectie en eerlijke status zijn bruikbare kwaliteitscriteria. Dit bewijst geen huidige fout in iedere distributie. |
| Privacy en toestemming | **[BRON]** Flatpak beperkt standaard toegang, maar manifests kunnen ruime rechten verlenen; portals ondersteunen toegang via een bewuste bestandskeuze. GNOME Software verduidelijkt bestandsrechten. [LM-08] [LM-01] | **[HYPOTHESE]** Vraag toegang voor de concrete taak en toon effect/intrekken. Een dialoog zonder afdwinging is onvoldoende. Dit corpus stelt geen uniforme Linux-telemetrie-, reclame- of accountpraktijk vast. |
| Installatie en updates | **[BRON]** Ubuntu App Center biedt snap en deb, soms beide voor dezelfde app. Snap documenteert automatische updates én uitstel/planning, inclusief onbeperkt uitstel via `refresh --hold` vanaf snapd 2.58. [LM-06] [LM-07] | **[ANALYSE]** Pakketkeuze en updatecontrole bestaan al. **[HYPOTHESE]** Leg herkomst, ondersteuning en geplande wijziging begrijpelijk uit; verberg niet welke uitvoering wordt geïnstalleerd. De stelling dat Snap nooit pauzeerbaar is, is onjuist. |
| Multitasking en notificaties | **[BRON]** Plasma 6.8 voegt sneltoetsen voor taakvolgorde toe; GNOME 48 groepeert appmeldingen en introduceert schermtijd- en pauzehulpmiddelen. [LM-03] [LM-02] | **[ANALYSE]** Aandachtsbeheer en toetsenbordbediening zijn bestaande concurrentievoorwaarden. **[HYPOTHESE]** Test taakgericht hervatten en meldingen per werkcontext; behoefte aan méér vensterstanden is niet aangetoond. |
| Back-up, synchronisatie, koppeling | **[BRON]** Pika Backup biedt versleutelde lokale/remote persoonlijke back-ups, geen compleet systeemherstel. KDE Connect biedt onder meer bestandsoverdracht, notificatiekoppeling en mediabediening. [LM-20] [LM-10] | **[ANALYSE]** Bestandsoverdracht, synchronisatie en herstel zijn afzonderlijke beloften. **[HYPOTHESE]** Toon laatste geslaagde back-up en herstelbare categorieën; bied expliciete apparaatkoppeling. |
| Toegankelijkheid | **[BRON]** Plasma 6.8 integreert klikken door de aanwijzer stil te houden, gericht op motorische beperkingen. [LM-03] | **[ANALYSE]** Inclusieve invoer moet onderdeel van de shell zijn. Deze toevoeging bewijst geen complete schermlezer-, braille- of appdekking. Dat vergt afzonderlijke tests met gebruikers. |
| Drivers en oude apps | **[BRON]** GNOME 51 verwijdert oude NVIDIA-driverinterfaces. Plasma 6.8 stopt X11-sessies, terwijl X11-apps via XWayland blijven ondersteund. [LM-01] [LM-03] | **[ANALYSE]** Modernisering heeft expliciete migratiegrenzen. **[HYPOTHESE]** Maak een controle per GPU, beeldscherm en randapparaat; vermijd de claim dat een hele apparaatcategorie werkt. |
| Games en anti-cheat | **[BRON]** Valve vereist ontwikkelaarsactivering/configuratie voor EAC/BattlEye; kernel-anticheat wordt in Proton niet ondersteund. Proton is een Linux-compatibiliteitslaag. [LM-09] | **[ANALYSE]** Een geblokkeerd essentieel spel kan elke andere productwinst tenietdoen. Nieuw-os erft Protonondersteuning niet. **[HYPOTHESE]** Controleer concrete titels vóór migratie; beloof geen algemene Windowscompatibiliteit. |
| Gewone en gevorderde gebruikers | **[ANALYSE]** De documentatie toont zowel grafische installatie als geavanceerde updatecommando's; de zoekmeldingen tonen diagnostiek die technische kennis vraagt. [LM-04] [LM-05] [LM-06] [LM-07] | **[HYPOTHESE]** Eén begrijpelijke standaardroute, met inspecteerbare details en sneltoetsen. Deze twee doelgroepen zijn werksegmenten, geen hier gemeten persona's. |

## Android en beperkte iOS-vergelijking

| Aspect | Controleerbare bevinding en grens | Betekenis voor nieuw-os |
|---|---|---|
| Zoeken, bestanden, instellingen | **[BRON]** Androids Storage Access Framework biedt een systeemkiezer over lokale en cloudproviders, met toegang tot gekozen documenten zonder algemene opslagtoestemming. [LM-14] | **[ANALYSE]** Een uniforme toegangstaak is haalbaar zonder universeel leesrecht. Er is in dit corpus onvoldoende gebruikersbewijs om de kwaliteit van Androids globale zoekfunctie te rangschikken. |
| Begrijpelijke privacy | **[BRON]** LM-15 observeerde bij 132 deelnemers onder meer gebundelde intrekkingen van rechten, vooral voor weinig gebruikte apps/niet-essentiële functies. LM-16 vond in twee online studies (46 en 383 deelnemers) misverstanden over termen in privacylabels voor iOS/Android. [LM-15] [LM-16] | **[HYPOTHESE]** Beschrijf gevolgen in gewone taal, laat kernfunctionaliteit bij weigering waar mogelijk werken en bied een gerichte rechtencontrole. Meer schakelaars alleen bewijzen geen betere controle. |
| Reclame, account, telemetrie | **[BRON]** Google documenteert afzonderlijke advertentieonderwerpen, appaanbevelingen en advertentiemeting. Google-back-up gebruikt een account en noemt ook diagnostiekverzameling. [LM-18] [LM-13] | **[ANALYSE]** Deze diensten hebben verschillende gegevensstromen; één schakelaar dekt niet alle personalisatie. Dit bewijst geen identiek gedrag van alle Androidvarianten, algemene accountdwang of omvang van gebruikersweerstand. |
| Updates en installatie | **[BRON]** Pixel 8 en later krijgen zeven jaar OS-/beveiligingsupdates vanaf eerste beschikbaarheid in de Amerikaanse Google Store. Android-ontwikkelaarsverificatie start regionaal bij deelnemende winkels; internationale uitbreiding is gepland, met een aparte route voor ongeverifieerde apps. [LM-17] [LM-19] | **[ANALYSE]** Ondersteuningsduur moet per apparaat zichtbaar zijn. Verificatie is niet hetzelfde als verbod op alle sideloading en was volgens deze bron niet al EU-breed ingevoerd. **[HYPOTHESE]** Combineer begrijpelijke standaardbeveiliging met een expliciete geavanceerde installatieroute. |
| Multitasking en notificaties | **[BRON]** Android 16 groepeert meldingen en ondersteunt live voortgang voor passende apps; desktopvensters werden voor compatibele apparaten aangekondigd. Android 17 beschrijft zwevende appvensters en een vensterbalk op grote schermen, met gefaseerde uitrol. [LM-12] [LM-11] | **[ANALYSE]** Mobiele en desktopbediening groeien naar elkaar toe, maar productaankondigingen bewijzen geen universele beschikbaarheid of lagere taaklast. **[HYPOTHESE]** Behoud taakcontext bij formaatwisseling; test bediening met touch én toetsenbord. |
| Back-up, herstel en migratie | **[BRON]** Google waarschuwt dat niet alle appdata herstelt en dat een nieuwere Androidback-up niet op een oudere versie terug kan. Slechts een deel krijgt extra versleuteling via schermvergrendeling; Photos/MMS vallen daar niet onder. [LM-13] | **[HYPOTHESE]** Geef vóór migratie een categoriegewijs overzicht van wat overdraagbaar, ontbrekend of opnieuw in te stellen is. Een groene back-upschakelaar is geen volledig herstelbewijs. |
| Apparaten, toegankelijkheid | **[BRON]** Android 16 biedt betere bediening van LE Audio-hoortoestellen, afhankelijk van telefoon en fabrikant. KDE Connect biedt een bestaande desktop/telefoonroute. [LM-12] [LM-10] | **[ANALYSE]** Toegankelijkheid omvat protocollen en apparaten, niet alleen contrast. **[HYPOTHESE]** Test ondersteunde combinaties en toon ontbrekende functies vóór koppeling. |
| Games, randapparaten en expertise | **[ANALYSE]** Het corpus rechtvaardigt geen algemene claim over Android-game-, controller- of drivercompatibiliteit op nieuw-os. De verificatieroute onderscheidt wel standaardgebruik en gevorderde installatie. [LM-19] | **[HYPOTHESE]** Leg compatibiliteit per app/apparaat vast. Een snel begrijpelijke dagelijkse route en inspecteerbare geavanceerde bediening moeten dezelfde rechtenregels volgen. |

## Terugkerende patronen en prioriteit

**[ANALYSE]** Onderstaande rangorde is een onderzoeksweging, geen getelde gebruikersfrequentie. Europese relevantie betekent hier controleerbare gegevenskeuze, overdraagbaarheid en inclusie; er is geen gemeten Europese betalings- of overstapbereidheid afgeleid.

| Patroon | Onderbouwing en sterkte | Impact, onderscheid en haalbaarheid |
|---|---|---|
| Rechten begrijpen zonder de taak te verliezen | Twee oorspronkelijke studies, verschillende teams; portals/documentkiezer tonen uitvoeringspatronen. Middelsterk, met oude gedragsdata. [LM-08] [LM-14] [LM-15] [LM-16] | Dagelijkse relevantie; onderscheid zit in begrijpelijke gevolgen en echte afdwinging. Kleine UI-uitleg is voorstelbaar; volledige platformrechten vereisen native bewijs. |
| Zien waarom iets niet gevonden wordt | Onafhankelijke GNOME- en KDE-meldingen; productspecificatie bevestigt aandacht voor bestandsstatus. Kwalitatief terugkerend, frequentie onbekend. [LM-01] [LM-04] [LM-05] | Hoge impact zodra een essentiële taak faalt. Zoekscope/offlinestatus zijn afgebakende concepten; indexering en bronisolatie zijn aanvullende systeemfuncties. |
| Herstelbare gegevens en betrouwbare migratie | Linux- en Androiddocumentatie benoemen concrete herstelgrenzen. Sterk bewijs voor grenzen, geen gemeten wensprevalentie. [LM-13] [LM-20] | Mogelijk beslissend voor overstap. Een eerlijk migratieoverzicht onderscheidt alleen als export en herstel werkelijk werken. |
| Minder onderbrekingen | GNOME en Google ontwikkelen groepering/aandachtsfuncties. Onafhankelijke productreacties, geen onafhankelijke effectmeting. [LM-02] [LM-12] | Functioneel relevant; nieuwheid laag. Een taakgericht focusmodel blijft een te testen hypothese. |
| Essentiële apps en apparaten behouden | Valve- en desktopdocumentatie geven technische voorwaarden. Sterk voor genoemde grenzen, onbekende populatieomvang. [LM-01] [LM-03] [LM-09] | Zeer hoge overstapimpact voor getroffen gebruikers; hoge kosten. Compatibiliteitsbelofte uitstellen tot aantoonbare ondersteuning. |

**[HYPOTHESE] Voorkeursrichting:** ontwerp eerst begrijpelijke taaktoegang en vindbaarheid, inclusief weigering, offline toestand en herstel. Beproef daarna migratie- en focusflows. Dit is geen toestemming om bestaande productkeuzes te vervangen: combineer het met repositoryonderzoek en de volledige prioriteitenmatrix.

**[ANALYSE] Technische grens:** deze Linux-, Flatpak-, Android- en Protonvoorzieningen zijn geen direct inzetbare libraries voor een zelfstandige kernel. Portering, drivers, platform-API's, opslag en beveiligingsafdwinging moeten apart worden onderzocht. Een hostprototype kan begrijpelijkheid onderzoeken, maar bewijst geen native OS-functie. Aan dit onderzoek zijn geen voortgangspunten of native acceptatieclaims verbonden.

Ontbrekend bewijs: representatieve overstapredenen; systematische schermlezertests; OEM-specifieke reclame/telemetrie; printer/audio/GPU-matrix; Android-zoek- en syncfrictie; vergelijkbare taakmetingen voor niet-technische gebruikers. Vervolg: echte taakobservaties en migratie-inventarisaties met consent, gescheiden naar doelgroep en toegankelijkheidsbehoefte. Geen fictieve interviews of testresultaten invullen.

## Bronregister

Alle bronnen inhoudelijk geraadpleegd op **2026-11-02**. `z.d.` betekent dat geen publicatie-/wijzigingsdatum op de geopende bron zichtbaar was. Dynamische documentatie kan wijzigen. Datums van productversies zijn geen publicatiedatums van een hele website.

| ID / directe bron | Auteur/organisatie; datum; type | Gebruikte claim; beperking |
|---|---|---|
| [LM-01 — Introducing GNOME 51][LM-01] | GNOME Project; 2026-09-16; release | Bestandsstatus, instellingen, rechten en drivertransitie; productbeschrijving, geen gebruikerstest. |
| [LM-02 — Introducing GNOME 48][LM-02] | GNOME Project; 2025-03-19; release | Meldingsgroepering en welzijnsfuncties; geen bewezen effectgrootte. |
| [LM-03 — Plasma 6.8][LM-03] | KDE Community; 2026-10-14; release | Dwell-click, taakbediening en Wayland/XWayland; beschikbaarheid hangt van distributie af. |
| [LM-04 — Search Everywhere can't find files][LM-04] | Adam Monsen, reacties Peter Eisenmann/Mathieu Bridon; 2025-12-13–16; gebruikersforum | Onduidelijke zoekscope; GNOME 46, één casus, geen huidige GNOME 51-regressie bewezen. |
| [LM-05 — CIFS share freezes taskbar when NAS offline][LM-05] | Marc_Sch, reactie claydoh; 2025-12-10; gebruikersforum | Zoekblokkering gemeld bij offline NAS; oorzaak niet onafhankelijk gereproduceerd. |
| [LM-06 — Install an application][LM-06] | Canonical; bijgewerkt 2026-06-17; Ubuntu 24.04-handleiding | Snap/deb in App Center; toont keuzestructuur, meet geen verwarring. |
| [LM-07 — Manage updates][LM-07] | Canonical; bijgewerkt 2026-10-19; Snap-documentatie | Automatische update, planning en hold; CLI-mogelijkheid bewijst geen vindbaarheid in GUI. |
| [LM-08 — Sandbox Permissions][LM-08] | Flatpak Project; z.d.; ontwikkelaarsdocumentatie | Sandbox en portals; brede grants blijven mogelijk, geen algemene appaudit. |
| [LM-09 — Steam Hardware and Proton][LM-09] | Valve; z.d.; Steamworks-documentatie | Linuxlaag en anti-cheatvoorwaarden; niet elke game getest en niet van toepassing op elke kernel. |
| [LM-10 — KDE Connect][LM-10] | KDE; z.d.; projectbeschrijving | Bestand-, notificatie- en apparaatfuncties; geen bewijs voor volledige platformpariteit of back-up. |
| [LM-11 — Check out what's new in Android 17][LM-11] | Seang Chau / Google; 2026-06-16; releasebericht | Appvensters en gefaseerde uitrol; geen onafhankelijke taakmeting. |
| [LM-12 — Android 16: Productivity, security and more features on Android][LM-12] | Seang Chau / Google; 2025-06-10; releasebericht | Meldingen, hoortoestellen, vensters; hardware/appvoorwaarden en toen toekomstige onderdelen onderscheiden. |
| [LM-13 — Back up or restore data on your Android device][LM-13] | Google; z.d.; supportdocumentatie | Account, herstelbeperkingen, versleuteling en diagnostiek; Google-dienst, niet alle Androidvarianten. |
| [LM-14 — Access documents and other files from shared storage][LM-14] | Google Android Developers; bijgewerkt 2026-10-28; API-documentatie | Documentkiezer en beperkte toegang; geen gebruikersonderzoek of bewijs van volledige lokale/cloudpariteit. |
| [LM-15 — “I do (not) need that Feature!”: Understanding Users' Awareness and Control of Privacy Permissions on Android Smartphones][LM-15] | Sarah Prange e.a.; universiteiten Bundeswehr München/Innsbruck/LMU en Google München; SOUPS 2024-08; veldstudie | 132 geanalyseerde deelnemers, twee weken; data april–mei 2022, overwegend jonge Europeanen, zelfselectie en mogelijke onderzoekseffecten. |
| [LM-16 — User Understandings of Technical Terms in App Privacy Labels][LM-16] | Ishika Keswani e.a., Pomona College; SOUPS 2025-08; twee online studies | Begripsproblemen bij 46/383 deelnemers; abstract geraadpleegd, geen gedragseffect of representatieve EU-uitkomst vastgesteld. |
| [LM-17 — Learn when you'll get software updates on Google Pixel phones][LM-17] | Google; z.d.; supportbeleid | Supportperiode per Pixelgeneratie; niet het gehele Android-ecosysteem. |
| [LM-18 — Manage your ad privacy settings on Android][LM-18] | Google; z.d.; supportdocumentatie | Gescheiden advertentie-instellingen; meet geen weerstand, uitzetten omvat niet alle tracking. |
| [LM-19 — Android developer verification][LM-19] | Google Android Developers; z.d.; actueel beleid/tijdlijn | Start 2026-09-30 bij deelnemende winkels in Brazilië, Indonesië, Singapore, Thailand; globale uitbreiding 2027 gepland, uitrol blijft veranderlijk. |
| [LM-20 — Pika Backup][LM-20] | Pika-contributors/GNOME, maintainers Sophie Herold/Fina Wilke; z.d., versie 0.8.4 van 2026-08-04; productdocumentatie | Persoonlijke back-ups en herstelgrens; niet standaard in iedere Linuxdesktop. |

[LM-01]: https://release.gnome.org/51/
[LM-02]: https://release.gnome.org/48/
[LM-03]: https://kde.org/announcements/plasma/6/6.8.0/
[LM-04]: https://discourse.gnome.org/t/search-everywhere-cant-find-files/33037
[LM-05]: https://discuss.kde.org/t/cifs-share-freezes-taskbar-when-nas-offline/42466
[LM-06]: https://ubuntu.com/desktop/docs/en/24.04/how-to/software/install-an-application/
[LM-07]: https://snapcraft.io/docs/how-to-guides/manage-snaps/manage-updates/
[LM-08]: https://docs.flatpak.org/en/latest/sandbox-permissions.html
[LM-09]: https://partner.steamgames.com/doc/steamhardware/proton
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
