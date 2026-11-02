# Overdraagbare mentale modellen en verwachtingscompatibiliteit

Onderzoeksdatum: 2 november 2026. Fase 4.5/4.6/4.8. Status: bronvergelijking en expliciete ontwerphypothesen; geen eigen interviews of geïmplementeerde interacties.

Leidend gebruikersprincipe: **“Familiar where familiarity reduces friction, innovative where innovation creates measurable value.”** Dit is de opdracht, geen onderzoeksresultaat.

## Bewijs en begrenzing

[BRON] Nielsen beschrijft onder meer herkenning, consistentie, gebruikerscontrole en foutpreventie als heuristieken. W3C's aanvullende cognitieve richtlijn adviseert vertrouwde, consistente bediening en duidelijke relaties tussen controles en inhoud. Dit zijn ontwerp-/toegankelijkheidsrichtlijnen, geen effectproef voor een nieuwe desktop. [J01][J03][J04]

[BRON] Mentale modellen verschillen per persoon en veranderen door ervaring. De bestaande vergelijkende OS-studie in het [Windows/macOS-dossier](2026-11-02/windows-macos-needs.md) gebruikte OS-afbeeldingen en een beperkte studentensteekproef; daarmee is geen kennis van een volledige installatie- of werkdag aangetoond. [J02]

[ANALYSE] Overeenkomende documentatie bewijst dat een concept in meerdere systemen voorkomt. Zij bewijst niet dat iedere gebruiker het kent, dezelfde betekenis eraan geeft of het zonder hulp kan gebruiken. Hieronder betekent **breed gedeeld** daarom: teruggevonden bij meerdere onderzochte platformen. De taakroutes, termen, sneltoetsen en bronversies staan in [de 28-taakvergelijking](os-interaction-patterns.md); installatieconcepten in [de installatiematrix](os-installation-comparison.md).

## Vergelijking van modellen

Feitelijke platformverschillen zijn ontleend aan de genoemde taakroutes; de overdrachtsgevolgen zijn [ANALYSE].

| Model | Wat breed gedeeld is | Fundamenteel verschil / overdrachtsrisico | Gevolg voor nieuw-os — hypothese |
|---|---|---|---|
| Desktop | Een aangemelde werkomgeving met apps en systeemstatus | Een desktop hoeft geen opslagplaats voor pictogrammen te zijn; GNOME werkt sterk via overzicht | Eén stabiel thuispunt; bureaubladbestanden niet noodzakelijk maken |
| Venster | Afzonderlijk zichtbaar werkgebied met focus | App, venster en document zijn verschillende identiteiten; sluiten beëindigt niet overal een app | Geef Sluiten een voorspelbaar venster-/documenteffect; behandel Stop app afzonderlijk |
| Sluiten/minimaliseren/maximaliseren | Zichtbaarheid en grootte zijn bestuurbaar | macOS heeft andere positie, full-screen-/tilingbetekenis; GNOME biedt niet standaard alle titelbalkknoppen | Functies benoemen en alternatieven bieden; geen kleur of knoppositie als universele kennis aannemen |
| Dock/taakbalk/panel | Apps openen en/of naar lopend werk terugkeren | Vastgemaakte app en actief venster zijn niet hetzelfde; GNOME-dash verschilt van Ubuntu-dock | Starten, actief werk en geselecteerd venster van elkaar onderscheiden |
| Launcher/Start | Een vindbare ingang naar programma's | Startmenu, Activities, Launchpad/Spotlight en KDE Launcher verschillen in structuur | Eigen consistente launcher met tekst Zoeken en programma's; bekende termen als vindhulp |
| Systeemvak/statusgebied | Netwerk, geluid en systeemstatus zijn gegroepeerd | App-tray, menu-extra's en quick settings hebben verschillende grenzen | Onderscheid permanente systeemstatus, appstatus en meldingen |
| Quick settings/Control Centre | Veelgebruikte schakelaars nabij status | Een wifi-icoon kan een toggle of een lijst openen; klik op pictogram verschilt soms van pijltje | Aparte herkenbare actie voor aan/uit en voor netwerkkeuze; label de gevolgen |
| Notificatiecentrum | Tijdelijke berichten zijn later terug te vinden | Locatie, kalenderkoppeling, groepering en uitzonderingen verschillen | Eén overzicht met bron, tijd en actie; focus is ander beleid dan toegang of apparaatstatus |
| Bestandsbeheer | Benoemde bestanden, mappen, locaties en openen met een app | Finder, Explorer, Files en Dolphin delen niet alle metadata-/verplaatsingsconventies | Behoud naam/map/locatie/import/export naast Materiaal-collecties |
| Instellingen | Een herkenbaar systeemonderdeel voor voorkeuren | Categorieën en app-/systeemgrens verschillen; GNOME/KDE zijn geen enkele Linux-structuur | Zelfde instelling via categorie én zoekresultaat naar dezelfde toestand |
| Zoeken | Tekst invoeren om iets terug te vinden of te openen | Appzoeker, inhoudsindex, webresultaat en actie hebben verschillende scope | Resultaatsoort, locatie en bevoegdheid zichtbaar; geen actie uitvoeren bij alleen typen |
| Drag-and-drop | Een object naar een bestemming brengen | Kopie/verplaats/alias is context- en modificatortoetsafhankelijk | Vooraf resultaat tonen; knop-/contextmenu-alternatief; ongeldige bestemming verandert niets |
| Contextmenu | Relevante acties bij een object | Rechtermuisknop, Control-click, menutoets en touch verschillen | Zelfde commando's via menu en toetsenbord; geen essentiële actie alleen op rechtermuisknop |
| Sneltoetsen | Snelle handelingen voor selectie, navigatie en bewerken | Ctrl versus Command; appwisselen versus vensterwisselen; instellingen zijn aanpasbaar | Echte bindings tonen; aliases mogen geen niet-ondersteunde keymapping beloven |
| Workspaces/Spaces | Werk in meerdere groepen verdelen | Aantal, dynamiek, richting en full-screen-gedrag verschillen | Optioneel, zichtbaar benoemd werkruimtemodel; geen vereiste om één app te openen |
| Installeren | Programma beschikbaar maken en kunnen verwijderen | Store, installer, pakket, appbundle en rechten hebben andere betekenissen | Herkomst, ondersteuning, gevolgen en verwijderen samen uitleggen; installatie is geen algemene gegevensgrant |

## Migratieprofielen: verwachtingen, geen persoonskennis

[HYPOTHESE] Deze profielen zijn gespreks-/testsegmenten. Zij zeggen niets over een individuele deelnemer voordat diens werkelijke ervaring is gevraagd. Iemand kan meerdere OS'en gebruiken of veel onderdelen nooit gebruikt hebben.

| Herkomst | Plausibel meegenomen kennis op grond van bestaande routes | Te toetsen verwarring / begeleiding |
|---|---|---|
| Windows | Start, taakbalk/systeemvak, Explorer, Alt+Tab, randsnapping, rechtermuisknop, snelkoppelingen, Instellingen en verschillende installatiebronnen | Een apptegel is geen bestand; collectie is geen verplaatsing; sluiten is niet wissen. Verwijs Explorer naar bestands-/materiaalnavigatie en Taakbeheer naar het echte systeemoverzicht zodra dat bestaat |
| macOS | Dock/topmenubalk, Finder, Spotlight, Mission Control/Spaces, gestures, Quick Look, appbundles en Systeeminstellingen | Ctrl/Command en app/vensterwissel niet gelijkstellen. Leg afwijkende sluit-/maximaliseerbetekenis uit, zonder de menubalk of Finder na te bouwen |
| GNOME | Activities/overzicht, Super-zoeken, dash, dynamische workspaces, Files en quick settings | Geen verwachting van klassieke taakbalk afdwingen; zichtbare appwisseling bieden. Ontbrekende desktopiconen niet als fout presenteren |
| KDE Plasma | Launcher, panel/taakbeheer/systeemvak, KRunner, Dolphin, System Settings, configuratie en virtuele desktops | “Task Manager” kan een panelwidget of procesoverzicht betekenen; zoekresultaten moeten disambigueren. Configuratie mag de kernbetekenis niet veranderen |
| Ubuntu GNOME | GNOME-basis met distributiekeuzes zoals een zichtbaar dock en App Center | Ubuntu-ervaring niet automatisch op upstream GNOME projecteren; pakketbron en versie blijven relevant |
| Eerste computer / geen voorkeur | Geen OS-specifieke kennis veronderstellen | Bied labels, voorbeelden, standaardroutes en geleidelijke details; geen verplichte cursus of verborgen sneltoets |

## Compatibility of expectations — voorstel

1. **Eén concept, één effect.** Dezelfde knop, zoekactie en sneltoets voeren dezelfde operatie uit. Herkomstkeuze verandert geen opslag, rechten, authenticatie, sluitbeleid of netwerktoegang.
2. **Vertrouwde woorden helpen vinden.** Ondersteun gerichte synoniemen voor programma's, bestandsbeheer en systeemfuncties. Toon de eigen naam, korte betekenis en beschikbaarheid. Een alias naar een afwezig onderdeel levert geen nepresultaat op.
3. **Begeleid verschil op het moment van handelen.** Bijvoorbeeld bij de eerste Sluiten-actie of het eerste zoeken naar Finder. Een kort wegklikbaar kaartje is een hypothese; herhaalde hints en taakonderbrekingen worden gemeten.
4. **Laat basistaken zelfstandig bestaan.** Zoeken, mappen, menu's en zichtbare knoppen blijven bruikbaar zonder AI, gestures, herkomstprofiel of voorkennis van Materiaal.
5. **Onderzoek begrip, niet alleen herkenning.** Vraag vóór uitvoeren welk effect iemand verwacht; vergelijk dat met werkelijk resultaat. Tel stille misverstanden, herstelwerk en hulp naast taaksucces.

Uitwerking van aliases en lokale voorkeuren: [onboardingspecificatie](../product/new-os-onboarding-spec.md). Dit is nog geen implementatie.

## Familiarity budget

De categorieën zijn een **projectweging**, geen wetenschappelijke score. Standaard is KEEP FAMILIAR. EVOLVE vereist een benoemd probleem en meetbare verwachting. REINVENT/NEW vereisen aanvullend aantoonbare meerwaarde boven een vertrouwde variant én behoud van basistaken.

| Onderdeel | Categorie | Motivatie / bewijsgrens |
|---|---|---|
| Vooruit/Terug/Annuleren, duidelijk focus en labels | KEEP FAMILIAR | Herkenning/controle uit richtlijnen; exacte taakroute nog testen |
| Naam/map/locatie, kopiëren/verplaatsen en nieuwe map | KEEP FAMILIAR | Breed aanwezige taakconcepten; collecties zijn aanvullend |
| Vensterfocus, sluiten, appwissel | KEEP FAMILIAR | Betekenis voorspelbaar houden; positie/stijl kan eigen zijn |
| Statusgebied, geluid, netwerk en vergrendelen | KEEP FAMILIAR | Direct bereikbare systeemhandelingen; niet achter assistent |
| Contextmenu en zichtbare alternatieven | KEEP FAMILIAR | Meer dan één invoermethode; gelijke opdrachten |
| Schijf kiezen, bewaren/wissen en bevestigen | KEEP FAMILIAR | Een onomkeerbaar gevolg verbergen levert geen zinvolle vernieuwing |
| Instellingen met categorieën én zoeken | EVOLVE | Minder navigatiewerk als hypothese; één gedeelde instelling |
| Zoekresultaten met bekende aliassen en zichtbare scope | EVOLVE | Eerder onderzoek toont concrete zoekfrictie; kwaliteit nog testen |
| Installer met veilige preview en expliciet wijzigingsplan | EVOLVE | Zelfde bekende beslissingen met betere gevolgweergave; geen verwijdering van noodzakelijke keuze |
| Privacykeuzes en toegangsherstel | EVOLVE | Gevolg, duur en intrekken combineren; herhaalde toestemming vermijden |
| Verplichte leveranciersaccount als poort naar lokale taken | REINVENT | Bewust verwijderen uit nieuw-os-funnel volgens bestaand privacycontract; geen universeel OS-patroon of gemeten effectclaim |
| Materiaal als blijvende identiteit boven gereedschappen | NEW voor overstappers | Bestaand projectconcept; nieuwe metafoor alleen geleidelijk introduceren naast mappen/openen |
| Objectgerichte toegangs-/herstelinspectie | NEW voor overstappers | Optionele verdieping, geen extra leerpoort vóór ieder bestand |

**Budgetregel — hypothese:** geen nieuwe metafoor nodig voor de onomkeerbare schijfkeuze of voor de 28 basistaken. Introduceer Materiaal in één optionele voorbeeldtaak; stel de tweede nieuwe inspectiemetafoor uit tot die nodig is. Dit is een beheersregel om leerlast te begrenzen, geen bewezen numerieke bovengrens. Geen extra REINVENT/NEW toevoegen zonder variantvergelijking en regressieproef van bekende taken.

## Bronregister voor deze analyse

Alle bronnen geraadpleegd op 2026-11-02. De platformhandleidingen en versies staan in de twee gekoppelde vergelijkingen; de bronfamilies hieronder onderbouwen het analysemodel.

| ID | Oorspronkelijke bron / datum | Gebruikt en beperking |
|---|---|---|
| J01 | Jakob Nielsen, *10 Usability Heuristics*, 1994-04-24; herzien 2024-01-30 | Consistentie, herkenning, controle, foutpreventie. Heuristieken, geen huidige OS-gebruikersmeting |
| J02 | Megan Chan, *Mental Models*, 2024-01-26; herzien 2026-09-01, herwerking Nielsen 2010 | Individuele verwachtingen en onjuiste modellen; voorbeelden/richtlijn, geen migratie-RCT |
| J03 | W3C COGA, *Use a Familiar Hierarchy and Design*, bronnotitie 2021-04-29; webpresentatie 2022-01 | Cognitieve toegankelijkheid en herkenning; aanvullende guidance, geen WCAG-conformiteitscriterium |
| J04 | W3C COGA, *Help Users Understand What Things are and How to Use Them*, zelfde notitie/webperiode | Duidelijke controles, relatie en consistentie; webcontext moet voor native getoetst worden |
| J05 | Microsoft Support, *Change or reset your PIN in Windows*, datum niet vermeld | PIN is niet hetzelfde als Microsoft-accountwachtwoord; geen implementatievoorbeeld voor nieuw-os |
| J06 | Nieuw-os privacycontract revisie 4, 2026-10-28, en bronbasis main e08fb0e/ontwikkelbron1f0dd9c | Lokale werking, geen leveranciersaccountdwang, echte rechten en open native grenzen; projectvoorwaarden, geen externe gebruikersdata |

[J01]: https://www.nngroup.com/articles/ten-usability-heuristics/
[J02]: https://www.nngroup.com/articles/mental-models/
[J03]: https://www.w3.org/WAI/WCAG2/supplemental/patterns/o1p02-familiar-design/
[J04]: https://www.w3.org/WAI/WCAG2/supplemental/objectives/o1-understandable/
[J05]: https://support.microsoft.com/en-us/windows/security/change-or-reset-your-pin-in-windows
[J06]: ../LOCAL-PRIVACY-CONTRACT.md
