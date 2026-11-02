# OS-ontwerp 2011–2026 en onderzoekshorizon 2031–2036

**Hoofdconclusie — analyse:** toekomstgericht OS-ontwerp wordt vooral bruikbaar door samenhang tussen taken, invoermethoden, apparaten en gebruikerscontrole; een nieuwe visuele stijl is op zichzelf geen bewijs van betere bruikbaarheid.

**Status:** fase 4, bureauonderzoek, geraadpleegd op **2026-11-02**. Geen gebruikerstest van nieuw-os, productbesluit of implementatie. Productvisie, selectie en technische uitwerking horen bij fasen 5–8. De eenentwintig DT-bronrecords hieronder scheiden productdocumentatie, normen, empirisch onderzoek en institutionele verslaggeving.

## Vragen en zoekstrategie

Onderzoeksvragen: welke interactiepatronen veranderden sinds 2011; wat ondersteunt taakuitvoering; welke claims zijn vooral esthetisch; welke ontwikkelingen verdienen onderzoek voor de komende vijf tot tien jaar?

Gericht gezocht naar originele releaseberichten en richtlijnen van Microsoft, Apple, Android en GNOME, W3C-documenten en oorspronkelijke studies over notificaties, leesbaarheid en AI-bediening. Zoektermen omvatten productnaam plus `release`, `design`, `adaptive`, `privacy dashboard`, `reduced motion`, en de exacte onderzoekstitels. Historische aankondigingen zijn onderscheiden van beschikbaarheid. Datums zijn gecontroleerd op de bronpagina; voor GNOME ook in de officiële releasekalender. Er is geen systematische literatuurreview of representatieve inventarisatie van alle OS-varianten uitgevoerd.

**Bewijsgrens:** fabrikantdocumentatie bewijst dat een functie is beschreven of aangekondigd, niet dat deze mensen sneller of tevredener maakt. De studies hebben verschillende populaties en uitkomstmaten. Ze leveren geen marktbrede ranglijst of specifiek bewijs voor Europese nieuw-os-gebruikers.

## Tijdlijn: gedateerde feiten, geen verhaal van onvermijdelijke vooruitgang

| Datum en status | Gedocumenteerde ontwikkeling | Betekenis voor onderzoek — analyse |
|---|---|---|
| 2011-06-01 publicatie / 06-02 Computex-demonstratie | Windows 8-preview richt zich op touchapparaten naast bestaande pc-functionaliteit. [DT01] | Eén systeem krijgt uiteenlopende invoercontexten; gelijke bediening is niet automatisch geschikte bediening. |
| 2013-06-10, aankondiging | iOS 7 vereenvoudigt typografie en vormgeving, maar gebruikt ook translucentie, lagen en beweging; Control Center groepeert snelle instellingen. [DT02] | De verschuiving richting flat design betekende geen volledige afschaffing van diepte. |
| 2014-06-25, aankondiging | Material Design introduceert samenhangende oppervlakken, elevatie, schaduwen en beweging. [DT03] | Een ontwerpsysteem kan relaties uitleggen; decoratieve schaduwen alleen doen dat niet. |
| 2014-10-16, Yosemite beschikbaar | Handoff, apparaatoverschrijdende AirDrop, Today-widgets en uitgebreider Spotlight; sommige Continuity-functies vereisen iOS 8.1 en geschikte hardware. [DT04] | Taakcontinuïteit wordt onderdeel van de desktopervaring, met ecosysteemvoorwaarden. |
| 2017-05-11, aankondiging | Microsoft presenteert Fluent voor verschillende apparaten en invoer via onder meer aanraking, pen, stem en blik. [DT05] | Het ontwerpvraagstuk verschuift naar consistente betekenis over verschillende interacties. |
| 2021-03-24, release | GNOME 40 ordent werkruimtes horizontaal, met bijpassende toetsen, muisbediening en gebaren. [DT07] | Ruimtelijke samenhang en alternatieve invoer zijn concretere onderzoekseenheden dan animatiestijl. |
| 2021-06-24, aankondiging | Windows 11 beschrijft Snap Layouts, Snap Groups en afzonderlijke Desktops. [DT08] | Vensters organiseren wordt zichtbaarder aangeboden; deze patronen zijn daarmee niet uitgevonden. |
| 2021-10-19, Pixel-uitrol | Android 12 koppelt systeemkleuren aan wallpaper, vernieuwt widgets en biedt een privacydashboard en camera-/microfoonindicatoren. [DT06] | Personalisatie en zichtbare gegevenstoegang worden systeemfuncties. |
| 2023-06-05, aankondiging | Vision Pro/visionOS introduceert Apples ruimtelijke vensters met oog-, hand- en spraakbediening. [DT09] | Nieuwe hardware maakt andere interacties mogelijk; desktopvervanging is hiermee niet bewezen. |
| 2023-10-05, aanbeveling | WCAG 2.2 verschijnt, met onder meer extra criteria rond focus en doelgrootte; geraadpleegde bijgewerkte versie: 2024-12-12. [DT10] | Toegankelijkheid krijgt toetsbare voorwaarden naast visuele voorkeuren. |
| 2024-06-10, aankondiging | Apple Intelligence verbindt generatieve functies met persoonlijke context en lokale én serververwerking. [DT11] | AI wordt geïntegreerd in bestaande taken; dit bericht bewijst geen volledige latere levering van alle aangekondigde functies. |
| 2025-06-09, aankondiging | macOS Tahoe combineert Liquid Glass, Continuity/Live Activities en Spotlight-acties. [DT13] | Zoeken ontwikkelt zich verder tot handelen; glas is een afzonderlijke visuele laag. |
| 2026-03-18, release | GNOME 50 voegt Reduced Motion toe en verbetert Orca, waaronder globale instellingen en taalwisseling. [DT14] | Nieuwe releases blijven fundamentele toegankelijkheid verbeteren. Dit is een mijlpaal, geen claim dat 50 de nieuwste GNOME-versie is. |

## Welke veranderingen zijn functioneel?

Onderstaande beoordelingen zijn **analyse**, gebaseerd op de vermelde bronnen; het zijn geen reeds gekozen nieuw-os-functies.

| Thema | Mogelijk praktisch effect | Grens of risico |
|---|---|---|
| Flat en Material | Rustige typografie, herkenbare groepen en consequente lagen kunnen een duidelijke hiërarchie ondersteunen. [DT02][DT03] | Minder ornament is geen gemeten taakverbetering. Onzichtbaar geworden bediening moet apart worden onderzocht. |
| Glas en translucentie | Voorgrondbediening en achterliggende inhoud kunnen visueel onderscheiden worden. Apples eigen richtlijn reserveert glas voor navigatie, waarschuwt tegen gestapeld glas en biedt verminderde transparantie, meer contrast en minder beweging. [DT12] | Achtergrondafhankelijk contrast, visuele ruis en renderkosten vereisen meting; een glaseffect bewijst geen moderniteit. |
| Dynamische kleur en dark mode | Kleurpersonalisatie en licht/donker kunnen voorkeuren en gebruiksomstandigheden ondersteunen. [DT06][DT18] | Een accentkleur mag de betekenis van fouten of focus niet onduidelijk maken. Dark mode is niet universeel beter leesbaar of energiezuiniger. [DT10][DT18][DT19] |
| Touch, gebaren en adaptieve layouts | GNOME koppelt gebaren aan alternatieve toetsen/muisacties. Android adviseert aanpassing aan beschikbare breedte, hoogte, invoer en houding, met herschikken, onthullen en andere presentatievormen. [DT07][DT15] | Een verkleinde desktop is nog geen mobiele interface. Verborgen gebaren en alleen slepen bieden niet iedereen toegang. [DT10][DT20] |
| Command palettes en universeel zoeken | Een gedeelde ingang voor vindbare objecten en expliciete acties kan navigatiestappen beperken; Spotlight laat zulke acties zien. [DT13] | Dit is een onderzoekshypothese voor nieuw-os: resultaatdekking, rechten, voorspelbaarheid en ontdekbaarheid moeten worden getest. Een commandopalet hoeft geen taalmodel te gebruiken. |
| Virtuele desktops, snapping en tiling | Werkruimtes en vensterindelingen bieden concrete manieren om taken te groeperen en meerdere bronnen te vergelijken. [DT07][DT08] | De bronnen bewijzen geen algemene superioriteit van automatisch tilen. Kleine vensters, focusvolgorde, herstel en beginnersbegrip blijven toetsvragen. |
| Widgets, context en ambient computing | Widgets en Live Activities maken status beschikbaar zonder de volledige app te openen. [DT04][DT13] | Hier betekent ambient: informatie op de achtergrond beschikbaar stellen. Permanent bewegende of profilerende informatie kan juist aandacht opeisen; relevantie moet meetbaar zijn. |
| AI en natuurlijke taal | Taken kunnen vanuit context worden ondersteund; richtlijnen benadrukken begrijpelijke grenzen, correctie, afwijzen en globale controle. [DT11][DT16] | Vrije taal is ambigu. Lokaal uitvoeren voorkomt op zichzelf geen foutieve acties, overmatige rechten of onduidelijke gegevensverwerking. |
| Continuity en privacydashboards | Handoff maakt taakoverdracht mogelijk; Android toont recent sensorgebruik en biedt toestemmingbeheer vanuit het overzicht. [DT04][DT06] | Integratie binnen één ecosysteem bewijst geen open interoperabiliteit. Een dashboard is alleen nuttig als weergave en daadwerkelijke rechten overeenkomen. |
| Spatial computing | Ruimtelijke plaatsing en multimodale invoer bieden een andere werkruimte. [DT09] | Hardware, ergonomie en toegankelijke alternatieven zijn randvoorwaarden. Er is hier geen vergelijkend bewijs verzameld voor productiviteitswinst op een gewone desktop. |

**Patronen om kritisch te onderzoeken:** uitsluitend op vaste schermformaten ontwerpen, essentiële bediening achter één gebaar verbergen, esthetiek zonder contrastcontrole en AI zonder herstelpad botsen met actuele adaptieve, toegankelijkheids- en AI-richtlijnen. [DT10][DT15][DT16][DT20] Dat maakt ze geen bewezen marktbrede afvallers. Glas, dark mode en virtuele desktops zijn evenmin vanzelf verouderd of toekomstbestendig: taak, uitvoering en doelgroep bepalen hun waarde.

## Empirisch bewijs en beperkingen

1. **Notificaties — Fitz e.a.** Een gerandomiseerde veldstudie met 237 smartphonegebruikers in India duurde twee weken. Drie dagelijkse bundels gingen samen met minder gerapporteerde stress en meer ervaren controle dan gebruikelijke meldingen. Elk uur bundelen leverde weinig verandering; volledig onderdrukken verhoogde angst en FoMO. [DT17] **Interpretatie:** timing en voorspelbaarheid verdienen onderzoek, niet alleen een aan/uitknop. **Grens:** Android, zelfrapportages en één steekproef; geen bewijs voor een universele driemaal-daagsinstelling of Europese desktops. De abstracttekst is intern tegenstrijdig over productiviteit; daarover wordt hier geen effectclaim overgenomen.

2. **Tegenbevinding notificaties — Dekker e.a.** Een vooraf geregistreerde RCT analyseerde 205 Androidgebruikers van 18–30 jaar na een baselineweek en één interventieweek. Meldingen uitschakelen gaf geen statistisch significant effect op schermtijd, controlefrequentie, ervaren controle of afleiding; wel minder ervaren gewoontegedrag en meer FoMO. [DT21] Het productiviteitsresultaat bevat een interactie door daling in de controlegroep, dus geen eenvoudige nulclaim. **Grens:** korte duur, leeftijdsselectie en uitsluiting bij onvoldoende naleving (minimaal 75% notificatiereductie). **Interpretatie:** uitschakelen verschilt van bundelen; deze studie weerlegt Fitz' bundelresultaat niet, maar ondersteunt evenmin de algemene aanname dat minder meldingen vanzelf beter welzijn opleveren. Geen significant effect is geen bewijs van afwezigheid van ieder effect.

3. **Leesbaarheid — Piepenbrock e.a.** Het oorspronkelijke abstract rapporteert betere visuele scherpte en correctieprestaties bij donkere tekens op een lichte achtergrond, bij zowel jongere als oudere deelnemers. [DT18] **Interpretatie:** dark mode moet een onderzochte keuze blijven. **Grens:** abstract gecontroleerd, volledige methoden niet geraadpleegd; specifieke taken, geen bewijs dat alle gebruikers of omgevingen lichte weergave verkiezen.

4. **AI-controle — Amershi e.a.** Achttien interactierichtlijnen werden iteratief ontwikkeld en onder meer geëvalueerd door 49 ontwerp-/HCI-beoefenaars op twintig AI-producten. Daaronder vallen correctie, onzekerheid, afwijzen en controle over systeemgedrag. [DT16] **Interpretatie:** bruikbare toetsvragen voor eventuele OS-automatisering. **Grens:** richtlijnvalidatie, geen gerandomiseerde effectmeting bij eindgebruikers en geen evaluatie van huidige generatieve OS-agenten.

5. **Energie — Dash en Hu.** Purdue beschrijft metingen van zes apps op vier OLED-telefoons; de besparing van donkere weergave hangt sterk af van helderheid en apparaat. [DT19] **Grens:** het oorspronkelijke ACM-artikel was niet rechtstreeks toegankelijk; de institutionele toelichting en auteursvermelding zijn gecontroleerd. Daarom geen overgenomen batterijpercentages en geen vertaling naar lcd, QEMU of nieuw-os-energiegebruik.

Beweging is hier onderbouwd met toegankelijkheidsrichtlijnen, niet met een nieuw verzamelde experimentele studie. WCAG 2.2 onderscheidt onder meer tekstcontrast, toetsenbordbediening, focus, alternatieven voor slepen en doelgrootte. Uitschakelbare niet-essentiële interactieanimatie is criterium 2.3.3 op **AAA**, niet AA. [DT10] WCAG2ICT geeft informatieve toepassing op niet-websoftware; dit document claimt geen native OS-conformiteit. [DT20]

## Horizon 2031–2036: expliciete hypotheses

Dit zijn **onderzoekshypotheses**, geen voorspellingen, productbeloften of implementatiebesluiten.

| Hypothese | Waarom plausibel | Hoe toetsen of weerleggen? |
|---|---|---|
| Gecombineerde object-/actiebediening groeit verder naast vensters. | Spotlight-acties en contextuele AI bestaan als productrichting. [DT11][DT13] | Vergelijk taakduur, fouten en ontdekbaarheid met menu's; verwerp wanneer leren en corrigeren meer kosten dan navigeren. |
| Lokale AI wordt bruikbaarder voor afgebakende taken, met gewone bediening als herstelroute. | Lokale verwerking is al onderdeel van productarchitecturen; controle en correctie zijn onderzochte richtlijnen. [DT11][DT16] | Meet resultaatkwaliteit, latency, geheugen, energie, offline werking en correctiekosten op doelhardware. Geen cloudfallback veronderstellen. |
| Adaptiviteit verschuift van apparaatcategorie naar taakruimte en invoer. | Android beschrijft aanpassing op component-/paneelniveau; GNOME combineert invoerwijzen. [DT07][DT15] | Test dezelfde taak bij meerdere schalen, vensterformaten en invoermethoden, inclusief assistieve technologie. |
| Aandacht en energie worden expliciete ontwerpbegrotingen. | Notificatie-effecten verschillen per ingreep; weergave-energie is contextafhankelijk. [DT17][DT21][DT19] | Meet onderbrekingen en herstelduur, plus daadwerkelijk energiegebruik; CPU-idle is geen batterijmeting. |
| Continuïteit wordt belangrijker dan één identieke interface op elk apparaat. | Taakoverdracht en Live Activities verbinden verschillende interfaces. [DT04][DT13] | Onderzoek hervatten met offline apparaten, ingetrokken rechten en alternatieve providers. Open interoperabiliteit volgt niet vanzelf uit huidige ecosystemen. |
| Spatial blijft taakafhankelijk naast conventionele schermen. | Een ruimtelijk platform bestaat, maar deze bronnen bewijzen geen algemene vervanging. [DT09] | Vergelijk concrete ruimtelijke taken op comfort, toegankelijkheid, kosten en prestaties; houd afwijzing als mogelijke uitkomst open. |

## Afzonderlijke repository-inferentie

Deze koppeling gebruikt de fase-1-broninspectie; externe trends bewijzen geen werking van nieuw-os. De onderzoeksworktree staat op main `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. De afzonderlijk bekeken ontwikkelbron is vastgezet op `1f0dd9c93959e3b1335656da1e548525a288133e`; deze inhoud is niet in de onderzoeksworktree geïntegreerd.

De [native UI op main](https://github.com/simonvanieperen/nieuw-os/blob/e08fb0e48c1bb0781e98357f588c2f73b6ae8500/ui/native/README.md) is een eigen Rust-pad; hostweergave is geen bewijs voor native systeemfuncties. De nieuwere [venstersessie](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/sdk/examples/window-session/src/scene.rs) en het [U01-acceptatieverslag](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/docs/verification/mature-session-20261102/U01-ACCEPTANCE.md) maken die oudere basis niet actueel. Hun afgebakende native bewijs mag evenmin worden verbreed tot volledige desktopvolwassenheid.

Onderzoeksvragen voor de volgende fasen:

- Hoe verhouden adaptieve layouts zich tot het huidige logische minimumformaat en de discrete schalen in de venstersessie?
- Welke betekenis, rechten en resultaatdekking ontbreken tussen de beperkte [app-lokale zoekfunctie](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/native/src/desktop.rs) en taakgerichte systeemzoekfunctie?
- Welke export en assistieve integratie zijn nodig boven op de [app-lokale semantiek](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/native/README.md), en hoe beperken de huidige [font-/Unicodegrenzen](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/graphics/README.md) Europese meertaligheid?
- Welke meetbare taakverbetering rechtvaardigt extra context, beweging of automatisering binnen het bestaande [privacycontract](https://github.com/simonvanieperen/nieuw-os/blob/e08fb0e48c1bb0781e98357f588c2f73b6ae8500/docs/LOCAL-PRIVACY-CONTRACT.md)? Apples hybride AI-architectuur vormt geen precedent voor cloudinferentie in nieuw-os.

## Bronnenregister

Alle records: **raadpleegdatum 2026-11-02**. A = primaire aankondiging/release; G = fabrikantgids; N = norm/richtlijn; E = oorspronkelijk onderzoek; I = verslag van de uitvoerende instelling. Links staan rechtstreeks bij de titels via de DT-verwijzingen.

| ID / titel | Auteur, publicatiedatum; type | Ondersteunde claim en beperking |
|---|---|---|
| [DT01 — Microsoft Previews Windows 8][DT01] | Microsoft; 2011-06-01, evenement 06-02; A | Touch/pc-preview; geen gebruikseffectmeting. |
| [DT02 — Apple Unveils iOS 7][DT02] | Apple; 2013-06-10; A | Vormgeving, lagen, Control Center; marketingclaims niet overgenomen. |
| [DT03 — Google I/O: Design, Develop, Distribute][DT03] | Monica Tran / Google; 2014-06-25; A | Introductie Material/elevatie; geen vergelijkend UX-bewijs. |
| [DT04 — OS X Yosemite Available Today as a Free Upgrade][DT04] | Apple; 2014-10-16; A | Continuity, widgets, Spotlight; ecosysteem-/hardwarevoorwaarden. |
| [DT05 — Build 2017: Sparking the next wave of creativity][DT05] | Terry Myerson / Microsoft; 2017-05-11; A | Fluent en invoerdiversiteit; historische aankondiging. |
| [DT06 — A dozen things to love in Android 12][DT06] | Dave Burke / Google; 2021-10-19; A | Dynamic color, widgets, privacydashboard; Pixel-uitrol, geen universele OEM-pariteit. |
| [DT07 — Introducing GNOME 40][DT07] | GNOME; 2021-03-24; A | Werkruimtes en invoeralternatieven; datum uit [releasekalender](https://release.gnome.org/calendar/), geen vergelijkende taakstudie. |
| [DT08 — Introducing Windows 11][DT08] | Panos Panay / Microsoft; 2021-06-24; A | Snapping/groepering/desktops; niet hun historische oorsprong. |
| [DT09 — Introducing Apple Vision Pro][DT09] | Apple; 2023-06-05; A | Ruimtelijke interface; geen bewijs voor desktopvervanging. |
| [DT10 — WCAG 2.2][DT10] | W3C AG Working Group; eerst 2023-10-05, geraadpleegde versie 2024-12-12; N | Toetsbare webcriteria; niet automatisch native conformiteit. |
| [DT11 — Introducing Apple Intelligence][DT11] | Apple; 2024-06-10; A | Context en lokale/servermodellen; aankondiging, geen volledige actuele beschikbaarheidsaudit. |
| [DT12 — Meet Liquid Glass][DT12] | Apple Developer; WWDC 2025, dag niet op pagina vermeld; G | Hiërarchie, geen gestapeld glas, toegankelijkheidsvarianten; fabrikantgids, geen effectonderzoek. |
| [DT13 — macOS Tahoe 26 makes the Mac more capable…][DT13] | Apple; 2025-06-09; A | Spotlight-acties, Liquid Glass, Continuity; productclaims niet als effectmeting behandeld. |
| [DT14 — Introducing GNOME 50][DT14] | GNOME; 2026-03-18; A | Reduced Motion en Orca; datum uit officiële releasekalender; geen gebruikersstudie. |
| [DT15 — Adapt layouts][DT15] | Android Developers / Google; bijgewerkt 2026-10-22; G | Vensterruimte, invoer, paneelaanpassing; platformspecifieke richtlijn. |
| [DT16 — Guidelines for Human-AI Interaction][DT16] | Saleema Amershi e.a.; CHI 2019, 2019-05-04–09; E | Achttien richtlijnen, 49 beoefenaars; geen eindgebruikerseffectproef. |
| [DT17 — Batching smartphone notifications can improve well-being][DT17] | Nicholas Fitz, Kostadin Kushlev, Ranjan Jagannathan, Terrel Lewis, Devang Paliwal, Dan Ariely; online 2019-07-16, tijdschrift 2019-12; E | RCT/timing; India, korte duur, zelfrapportage; [oorspronkelijke tekst](https://static1.squarespace.com/static/57a40c19414fb54f51f8095f/t/614a55faa7b89e25f4e48ad1/1632261627146/2019%2BFitz%2BBatching.pdf) gelezen. |
| [DT18 — Positive display polarity is advantageous for both younger and older adults][DT18] | Cosima Piepenbrock, Susanne Mayr, Iris Mund, Axel Buchner; online 2013-05-08; E | Leespolariteit; alleen oorspronkelijke abstract gecontroleerd. |
| [DT19 — Dark mode may not save your phone’s battery life as much as you think…][DT19] | Kayla Wiles / Purdue; 2021-07-28; I | Eigen onderzoek Dash/Hu, [MobiSys-paper](https://doi.org/10.1145/3458864.3467682); fulltext geblokkeerd, beperkt tot contextafhankelijkheid. |
| [DT20 — Guidance on Applying WCAG 2 to Non-Web ICT][DT20] | W3C; Mary Jo Mueller, Phil Day, Daniel Montalvo (red.); 2025-12-11; N | Toepassing op software; informatieve Group Note, geen zelfstandige normatieve verplichting. |
| [DT21 — Beyond the Buzz: Investigating the Effects of a Notification-Disabling Intervention on Smartphone Behavior and Digital Well-Being][DT21] | Cynthia A. Dekker, Susanne E. Baumgartner, Sindy R. Sumter, Jakob Ohme; online 2024-03-28, tijdschrift 2025; E | Uitschakelinterventie; abstract, methode en resultaten gelezen; korte duur, leeftijds-/nalevingsselectie, geen bundelproef. |

[DT01]: https://news.microsoft.com/source/2011/06/01/microsoft-previews-windows-8/
[DT02]: https://www.apple.com/newsroom/2013/06/10Apple-Unveils-iOS-7/
[DT03]: https://android-developers.googleblog.com/2014/06/google-io-design-develop-distribute.html
[DT04]: https://www.apple.com/newsroom/2014/10/16OS-X-Yosemite-Available-Today-as-a-Free-Upgrade/
[DT05]: https://blogs.windows.com/windows-insider/2017/05/11/build-2017-sparking-the-next-wave-of-creativity-with-the-windows-10-fall-creators-update/
[DT06]: https://blog.google/products-and-platforms/platforms/android/android-12/
[DT07]: https://release.gnome.org/40/
[DT08]: https://blogs.windows.com/windowsexperience/2021/06/24/introducing-windows-11/
[DT09]: https://www.apple.com/newsroom/2023/06/introducing-apple-vision-pro/
[DT10]: https://www.w3.org/TR/2024/REC-WCAG22-20241212/
[DT11]: https://www.apple.com/newsroom/2024/06/introducing-apple-intelligence-for-iphone-ipad-and-mac/
[DT12]: https://developer.apple.com/videos/play/wwdc2025/219/
[DT13]: https://www.apple.com/newsroom/2025/06/macos-tahoe-26-makes-the-mac-more-capable-productive-and-intelligent-than-ever/
[DT14]: https://release.gnome.org/50/
[DT15]: https://developer.android.com/design/ui/mobile/guides/layout-and-content/adapt-layout
[DT16]: https://www.microsoft.com/en-us/research/wp-content/uploads/2019/01/Guidelines-for-Human-AI-Interaction-camera-ready.pdf
[DT17]: https://doi.org/10.1016/j.chb.2019.07.016
[DT18]: https://pubmed.ncbi.nlm.nih.gov/23654206/
[DT19]: https://www.purdue.edu/newsroom/archive/releases/2021/Q3/dark-mode-may-not-save-your-phones-battery-life-as-much-as-you-think,-but-there-are-a-few-silver-linings.html
[DT20]: https://www.w3.org/TR/2025/NOTE-wcag2ict-22-20251211/
[DT21]: https://pure.uva.nl/ws/files/245971989/Beyond_the_Buzz.pdf
