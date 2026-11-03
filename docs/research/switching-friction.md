# Overstapfrictie tijdens installatie en de eerste gebruiksdag

Onderzocht: 2 november 2026. Fase 4.6. Status: gedocumenteerde platformverschillen plus toetsbare hypotheses; geen eigen gebruikersproef. De bredere adoptie-, app-, hardware- en Europese barrières staan in [fase 3](2026-11-02/european-adoption-barriers.md). De oorspronkelijke Eurobarometer blijft daar exact begrensd: dienstoverstap en ongespecificeerde meerkosten, geen OS-adoptie.

## Onderzoeksvraag en methode

Welke bestaande kennis helpt bij nieuw-os, waar ontstaat een verkeerde verwachting en welk productbewijs is nodig vóór een veilige overstap? De bronnenroute loopt van [de installatiematrix](os-installation-comparison.md) en [28 dagelijkse taken](os-interaction-patterns.md) naar [mentale modellen](transferable-mental-models.md). Platformdocumentatie beschrijft beschikbare bediening. Zij meet niet hoeveel mensen die bediening kennen, prefereren of succesvol overdragen. De onderstaande profielen zijn daarom **onderzoekssegmenten, geen verzonnen personen of gemeten populaties**.

De fase-3-bronnen CMA en vzbv onderbouwen het bestaan van leer-, ecosysteem- en compatibiliteitsbarrières binnen hun eigen populaties. Zij leveren geen effectgrootte voor de hier voorgestelde onboarding. Zie de methodologische beperkingen en primaire bronnen [EA-05–EA-11 in het adoptiedossier](2026-11-02/european-adoption-barriers.md#directe-bronverwijzingen).

## Herkomstprofielen

De eerste kolom geeft gedocumenteerde conventies uit de taakvergelijking. Alle verwachtingen, prioriteiten en interventies in de overige kolommen zijn [ANALYSE]/[HYPOTHESE]. Geen herkomstprofiel verandert de kerninterface, beveiliging of rechten.

| Herkomst | Meegenomen conventies | Mogelijke misverwachting | Te toetsen ondersteuning |
|---|---|---|---|
| Windows | Start/zoekveld, taakbalk, systeemvak, Verkenner, contextmenu, Alt+Tab, snapping en aparte Instellingen; installatie via Store én andere distributieroutes | Een gesloten venster betekent afgesloten app; gedownload `.exe`-bestand werkt; bureaublad is vanzelf de opslagplek; “Taakbeheer” is de naam van ieder systeemoverzicht | Zoekaliassen, zichtbare vensterstatus, uitleg van installeren versus openen en een werkelijke bestandsimportproef; eerst launcher, vensterwissel en gegevenslocatie toetsen |
| macOS | Dock, Finder, Spotlight, globale menubalk, Mission Control/Spaces, Quick Look, gebaren, appbundels en Systeeminstellingen | Venster sluiten laat app altijd draaien; Command-toets bestaat op ieder toetsenbord; menubalkacties staan buiten het venster; een `.app` kan worden overgenomen | Toon de werkelijke sluitbetekenis en beschikbare sneltoets op de actie zelf; contextuele uitleg van Materiaal, zoekaliassen en voorbeeld “zoeken → openen → terug” |
| GNOME | Overzicht met Super, appzoeker, activiteiten/workspaces, bovenbalk, Bestanden en Software | Minimaliseren/maximaliseren zijn altijd verborgen; dynamische workspace verschijnt automatisch; extensies en GNOME-pakketten werken | Toon nieuw-os-vensterbediening; één optionele uitleg van workspacecontract wanneer beschikbaar; gewone taken zonder extensie of terminal toetsen |
| KDE Plasma | Toepassingenstarter, paneel, systeemvak, Dolphin, KRunner, Systeeminstellingen, Discover, brede aanpasbaarheid | Paneel/widgets en alle sneltoetsen zijn overal identiek configureerbaar; “Task Manager” is eenduidig een processenvenster | Ambigue zoekterm laat betekenis kiezen; standaardroute blijft stabiel; gevorderde voorkeuren apart van noodzakelijke bediening toetsen |
| Ubuntu Desktop | GNOME-basis met Ubuntu-aanpassingen, onder meer dock en eigen software-/installatieroutes | Iedere GNOME-route of pakketnaam is identiek aan Ubuntu; Ubuntu-instructies gelden voor de eigen nieuw-os-kernel | Gebruik distributie én desktop als achtergrondinformatie; geen onbeperkte “Linuxmodus”; herkenbare werkwoorden en zichtbare pakketbron |
| Eerste computer / geen voorkeur | Geen platformconventie verondersteld | Pictogrammen, dubbelklikken, slepen, schijfwissen of onlineaccounts zijn vanzelfsprekend | Tekstlabels, veilige proefactie, toetsenbordhulp en expliciete gevolgen; meten zonder af te leiden uit leeftijd, nationaliteit of opleiding |

De macOS-route verschilt bovendien tussen appwisselen en vensterwisselen; GNOME en KDE hebben verschillende workspace- en launchercontracten. Herkomst “Linux” is daarom onvoldoende voor een exacte sneltoetsbelofte. Versie, instellingen, hardware en hulpmiddelen kunnen ieder profiel verder veranderen. [Taakroutes en beperkingen](os-interaction-patterns.md)

## Risicoregister voor de overstap

Impact en prioriteit zijn ontwerpbeoordelingen, geen gemeten incidentfrequenties. **Blokkerend** betekent dat een route niet als ondersteunde migratie mag worden aangeboden voordat het vereiste bewijs bestaat.

| Risico | Gevolg | Ontwerp-/engineeringeis | Benodigd bewijs |
|---|---|---|---|
| Verkeerde doelschijf of onbekende versleuteling | Onherstelbaar bronverlies; blokkerend | Niet automatisch wissen, verkleinen, converteren of ontsleutelen; identiteit en wijzigingsplan vóór autorisatie tonen | Disposable diskbeelden met meerdere, volle, onbekende en versleutelde schijven; hotplug/stale-plan-tests; menselijke begripstoets |
| Ontbrekende herstel-/encryptiesleutel | Gebruiker verliest toegang; blokkerend | Sleutelbeheer en herstel werkelijk leveren; geen nepkeuze “versleuteld” of impliciete providerescrow | Cryptografisch/servicebewijs, herstel met nieuwe sessie en verlies-/onderbrekingsgevallen; beveiligingsreview |
| Verlies van noodzakelijk programma | Werk of spel onuitvoerbaar; blokkerend voor die gebruiker | Compatibiliteit per app, versie en workflow controleren; geen generieke Windows/macOS/Linux-compatibiliteitsclaim | Uitvoering op eigen OS, inclusief plug-ins, authenticatie, offlinegebruik en relevante anti-cheat; [fase-3-appbronnen](2026-11-02/european-adoption-barriers.md#barrièrematrix) |
| Bestanden openen maar onjuist terugschrijven | Layout-, macro- of gegevensverlies | Origineel behouden; voorbeeldimport en heen/terugcontrole; niet “volledig gemigreerd” melden op basis van kopie alleen | Gesproken taakcriteria plus bestandstypen, aantallen, hashes waar passend, rechten en wijzigingsverslagen |
| Ontbrekende driver of hulpmiddel | Geen netwerk, audio, beeld, invoer of zelfstandige bediening; blokkerend | Exact ondersteund profiel tonen vóór installatie; geen automatisch fysiek experiment | Geteste hardware-/hulpmiddelenmatrix; VM-bewijs expliciet van fysieke ondersteuning scheiden |
| Ander zoekwoord voor dezelfde functie | Hulp nodig of verkeerde actie | Lokale synoniemen naar echte doelen met eigen naam; ambiguïteit zichtbaar | Per herkomst/taal term→doel-proeven, onbekende term en ontbrekend doel; nul automatische uitvoering |
| Andere betekenis van sluiten/minimaliseren | Werkstatus verkeerd ingeschat | Eén zichtbaar venster-/appcontract; onbevestigd werk beschermen | Werk bewaren, sluiten, heropenen en crashherstel als afzonderlijke taken |
| Telefoon, cloud of bedrijfsidentiteit ontbreekt | Verlies van gedeelde workflow; mogelijk blokkerend | Integratie niet impliciet claimen; lokale kern bruikbaar; optionele koppeling met scope | Echte export/import-/ontkoppelproeven, meerdere accounts, beheerbeleid en offlinepad |
| “Snel geholpen” door onzichtbare automatisering | Rechten, netwerk of systeemactie buiten verwachting | Hint en zoekresultaat voeren niets uit; gevolgen vóór gerichte actie | Negatieve grant-, netwerk- en broker-tests; toestemming niet afleiden uit herkomstkeuze |
| Onbekende onderhouds-/supportroute | Proberen leidt niet tot blijven gebruiken | Grenzen, herstel en eigenaar vindbaar; beleid begrijpelijk | Taak “updateprobleem oplossen” met echte terugweg en bereikbaar supportkanaal |

De eigen [repositorybasis](2026-11-02/repository-baseline.md) bewijst geen fysieke installatie, appcompatibiliteit, algemene bestandsoverdracht, persoonlijke authenticatie of volledige toegankelijkheid. Ook de nieuwere native bron heeft die volledige migratieketen niet aangetoond. Deze risico's worden niet opgelost door een betere uitleg alleen.

## Verwachtingscompatibiliteit als productstrategie

[VOORSTEL] Behoud de betekenis van openen, selecteren, kopiëren, plakken, sluiten, teruggaan en instellingen zoeken. Laat de bestaande termen Finder, Explorer en System Monitor als zoekingang dienen, terwijl één nieuw-os-naam zichtbaar blijft. Laat een onbekende of nog niet geleverde functie eerlijk als ontbrekend zien. De [onboardingspecificatie](../product/new-os-onboarding-spec.md) legt ambiguïteit en grenzen vast.

[VOORSTEL] Vraag “Waar kom je vandaan?” alleen optioneel, na een bruikbare sessie. Gebruik het antwoord voor tijdelijke begeleiding en relevante woorden. De gebruiker kan overslaan, later veranderen of wissen. Het antwoord verleent geen capabilities, wijzigt geen account-/cloudbeleid en selecteert geen ander desktopmodel.

[ANALYSE] Een zichtbare installatie- en herstelroute verlaagt een concreet risico; of zij werkelijk meer vertrouwen of overstappen veroorzaakt moet nog worden gemeten. De voorkeursproef vergelijkt dezelfde taken en interface met en zonder contextuele begeleiding. Een voorkeursoordeel na een screenshot telt niet als bewezen day-one usability.

## Validatie vóór een migratieclaim

1. Selecteer Windows-, macOS-, GNOME-, KDE- en Ubuntu-gebruikers op werkelijk uitgevoerde taken; leg versie, toetsenbord en hulpmiddelen vast. Geef “geen voorkeur” een eigen route. Verzamel geen wachtwoorden of echte herstelgeheimen.
2. Laat deelnemers met synthetische gegevens eerst de installatiekeuze uitleggen, daarna de relevante 28 taken uitvoeren. Codeer verwacht versus feitelijk effect, hulp, verkeerde bestemming, terugstappen en herstel.
3. Toets professioneel noodzakelijke apps, formaten en hardware vóór een overstapaanbod. Een lage kliklast compenseert geen ontbrekende werkfunctie.
4. Rapporteer ieder herkomstsegment apart. Een kleine verkennende steekproef levert problemen en hypothesen op, geen representatieve succespercentages voor Europa.
5. Meet proberen, geslaagde migratie, vrijwillig blijven gebruiken en reden tot terugkeer afzonderlijk. Geen enkel antwoord op een hypothetische dienstvraag vervangt die waarnemingen.

Meetdoelen, veiligheidscriteria en nog open bewijs staan in [interaction-principles.md](../product/interaction-principles.md). Er zijn in deze onderzoeksfase geen deelnemers geworven en geen installaties of gebruikersmetingen uitgevoerd.
