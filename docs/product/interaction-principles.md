# Nieuw-os — interactieprincipes en meetbare UX-criteria

Datum: 2 november 2026. Status: **voorgestelde product- en onderzoekscriteria; geen behaalde UX-score**. Dit document verbindt de [installatievergelijking](../research/os-installation-comparison.md), [dagelijkse taken](../research/os-interaction-patterns.md), [mentale modellen](../research/transferable-mental-models.md) en [overstapbarrières](../research/switching-friction.md) met later te toetsen gedrag.

## Ontwerpgrondslag

**Herkenbaar waar herkenning frictie verlaagt; vernieuwend waar vernieuwing meetbare waarde creëert.** [OPDRACHT] De eigen identiteit zit in de samenhang van materiaal, acties, rechten en herstel. Zij vereist geen nieuwe betekenis voor openen, venster sluiten, kopiëren of schijfwissen.

[BRON] Nielsen's heuristieken adviseren herkenning, consistente bediening, gebruikerscontrole en foutpreventie. W3C's aanvullende cognitieve richtlijn adviseert vertrouwde bediening. Dit zijn ontwerprichtlijnen, geen bewijs dat deze nieuw-os-oplossing beter presteert. WCAG 2.2 is een webstandaard; de gekozen contrast-, focus- en invoerprincipes helpen het prototype en zijn input voor een native toegankelijkheidscontract. Zij bewijzen geen OS-brede conformiteit. [J01/J03 in het modellenregister](../research/transferable-mental-models.md#bronregister-voor-deze-analyse), [W3C WCAG 2.2](https://www.w3.org/TR/WCAG22/) (12 december 2024; geraadpleegd 2 november 2026).

## Criteria en traceerbaarheid

Getallen hieronder zijn **vooraf gekozen toetsdoelen**, niet uit onderzoek afgeleide universele grenzen of al behaalde resultaten. Ze worden na een pilot met reden herzien. Een gemiddelde compenseert geen blokkade of dataverlies. `IN-xx` verwijst naar [de installer](new-os-installer-spec.md), `ON-xx` naar [onboarding](new-os-onboarding-spec.md); TASK 01–28 verwijzen naar de taakvergelijking.

| ID | Principe en voorgesteld criterium | Onderbouwing | Toets en grens |
|---|---|---|---|
| UX-01 | Alle 28 ondersteunde basistaken zijn zonder handleiding bereikbaar; doel bij latere bevestiging: ≥90% zelfstandige taakvoltooiing per herkomstsegment | Herkenning/consistentie [J01/J03]; gedocumenteerde TASK 01–28 | Observeer eerste poging, hulp, fouten en resultaat per taak; rapporteer ontbrekende functies apart. Kleine pilot levert geen betrouwbaar populatiepercentage |
| UX-02 | Iedere destructieve installatieactie toont doel en gevolgen vóór autorisatie; nul ongeautoriseerde schijfmutaties in de vastgelegde veiligheidsmatrix | Installatiestappen 10–14/25; eigen veiligheidskeuze IN-01/02 | Machineproeven met disposable images plus deelnemers laten uitleggen wat behouden/gewist wordt. “100% geslaagde testgevallen” is geen absolute veiligheidsclaim |
| UX-03 | Beginnerspad bevat geen verplichte invoer van EFI, GPT, mount point, filesystem of partitietabel | Verschillen technische installerkeuzes; cognitieve richtlijn | IN-03-taakproef en tekstaudit; Advanced blijft expliciet bereikbaar met dezelfde gevolgencontrole |
| UX-04 | Iedere geleverde instelling is vindbaar via categorie én ten minste één gewone zoekterm | TASK 06, mentale modellen instellingen/zoeken | Dekkingscheck van werkelijk geleverde instellingen; menselijke zoekopdracht zonder vooraf de eigen naam te geven |
| UX-05 | Iedere vastgelegde herkomstalias leidt naar het juiste beschikbare doel, duidelijke keuze of eerlijke ontbrekendstatus; query voert nul acties uit | Herkomstprofielen; ON-03 | Registerproeven per taal en serviceversie; ook ambigue “Task Manager”, verkeerd gespelde term en ontbrekende service |
| UX-06 | Dezelfde actie heeft dezelfde uitkomst en toegangscontrole via toetsenbord, muis, touch of toegankelijkheids-API | J01; WCAG-invoerprincipes; TASK 13/16–19 | Volledige route met keyboard en passende hulpmiddelen; geen verplichte drag/hover/gesture; fysieke touch en screenreader apart verifiëren |
| UX-07 | Taal, invoer en benodigde hulpmiddelen werken vóór geheimen en schijfkeuze | Installatiestappen 3/5/6/14/17 | IN-04, focusvolgorde, schermlezer, toetsenbordtest en terugkeer zonder geheimen te loggen |
| UX-08 | Alleen noodzakelijke informatie blokkeert installatie; lokaal kerngebruik vereist geen provideraccount, cloud, telemetrie of AI | Bestaand privacycontract; bronverschillen account/netwerk/OOBE | Offlinepad waar payload compleet is, weigeren/overslaan, nul externe fallback; ontbrekende echte lokale services blijven releaseblokkerend |
| UX-09 | Op ieder moment zijn voortgang, resterende keuze en annuleergrens begrijpelijk | J01 status/controle; installatie 25–30 | IN-05; rapporteer fase en fout, geen verzonnen percentage of tijd. Onderbreking na schrijven belooft geen undo |
| UX-10 | Basisbediening gebruikt KEEP FAMILIAR of EVOLVE; REINVENT/NEW krijgt probleem, bewijs en aparte toets vóór opname | Familiarity budget en taakverschillen | Ontwerpreview per hoofdroute, niet alleen screenshots; eigen Materiaal-model mag geen toegangseis voor simpele basistaak zijn |
| UX-11 | Tekstcontrast ≥4,5:1, grote tekst en betekenisvolle UI-grenzen/staten ≥3:1; zichtbare focus, reflow/schaal en reduced motion | Bestaand P2-contract, WCAG 2.2; eigen componentkeuzes | Werkelijke toestanden meten. 200% browserzoom/320 CSS px gelden voor prototype; native tekstschaal, focus en semantiek eigen bewijs. Geen volledige WCAG-claim uit kleurmetingen |
| UX-12 | Begeleiding is overslaanbaar, hervatbaar en lokaal wisbaar; herkomstkeuze verandert nul privileges of kernroutes | J02 individuele modellen; ON-01/02/04 | Toestandsproeven, vergelijking alle profielen, retentie-/wisproef; geen herkomst uit gebruik afleiden |
| UX-13 | Gegevensimport laat bron intact en onderscheidt gekopieerd, gecontroleerd en niet overgenomen | Overstap-/formaatbarrières; bestaande recoveryprincipes | Synthetische bron, afwijkende bestanden/rechten/formats, onderbreken/hervatten; geen succes op basis van alleen een voortgangsbalk |
| UX-14 | Werking en bewijsstatus zijn zichtbaar: ontbrekende backend geeft geen succesvolle systeemhandeling | Repositorybasis; bestaande product-/brokergrens | Echte service-uitkomst en negatieve paden; prototypefixtures als simulatie blijven herkennen |

De 90%-ambitie in UX-01 is een productdoel voor een latere voldoende grote bevestigende studie, geen eis om met een handvol deelnemers een percentage te claimen. Tijd tot resultaat, aantal conceptuele stappen, verkeerd doel, hulp en herstel worden eveneens gerapporteerd. Snelheid alleen bepaalt de winnaar niet: één extra begrijpelijke controle vóór wissen kan noodzakelijk zijn.

## Familiarity budget toepassen

De volledige onderverdeling staat in [transferable-mental-models.md](../research/transferable-mental-models.md#familiarity-budget). Toepassing per wijziging:

- **KEEP FAMILIAR:** betekenis van bestanden/mappen, selecteren, vensterbediening, status, terug/annuleren en expliciete doelschijfkeuze. Afwijking vereist meer dan stijlvoorkeur.
- **EVOLVE:** instellingen plus zoeken, herkenbare aliassen, installerwijzigingsplan en contextuele rechten. Bestaand doel en werkwoorden blijven herkenbaar.
- **REINVENT:** een verplichte provideraccountpoort voor de lokale kern verwijderen volgens het bestaande privacycontract. Hiermee is geen alternatieve authenticatie al gebouwd.
- **NEW:** Materiaal-/toolconcept en inspectie van objectrechten/herstel alleen waar zij een echte taak verbeteren; aparte uitleg en toets, zonder nieuwe metafoor voor wissen.

Dit is een kwalitatief budget. Geen bron onderbouwt een magisch percentage vernieuwing. Niet aanbevolen: vier shells per herkomst, een nieuw pictogram zonder label voor een kritieke keuze, providerlogin voor basisgebruik, automatisch kiezen van een te wissen schijf, vrije AI-acties en uitsluitend decoratieve futuristische effecten.

## Onderzoeks- en verificatieplan

**Eerst technische geschiktheid:** documenteer werkelijk ondersteunde account-, opslag-, netwerk-, update- en hulpmiddelroutes. Een ontbrekende functie wordt geen usability-succes door de taak te verwijderen. Test onbekende schijf, wisselende device-identiteit, corrupte planning, stroomonderbreking en recovery met disposable VM-beelden voordat iemand een echte schijf zou mogen gebruiken.

**Verkennende pilot:** voorstel 8–12 deelnemers met uiteenlopende herkomst en hulpmiddelen, bewust geworven op relevante taken. Dit aantal is een planningkeuze; het levert geen representatieve of per-OS statistisch betrouwbare uitspraken op. Bied synthetische gegevens, een veilige VM/prototypeomgeving en geen echte geheimen. Registreer achtergrondversie en dagelijkse taken zonder afkomst als vaardigheidsscore te behandelen.

**Taken en uitkomsten:** vraag eerst wat de deelnemer verwacht, laat vervolgens uitvoeren. Houd vooraf de beginstaat en het gewenste eindresultaat gelijk. Meet taakvoltooiing zonder hulp, verkeerd doel, ernstige fout, herstel, hulpbehoefte en tijd. Codeer installatiebegrip (“welke gegevens verdwijnen?”) apart van het kunnen indrukken van een knop. Vergelijk bronroute en nieuw-os niet met een klikranglijst: account-, veiligheids- en hardwarevoorwaarden verschillen.

**Bevestiging:** ontwerp pas na de pilot een grotere steekproef en analyse per herkomst/taak, met onzekerheidsmarges, vooraf vastgelegde uitkomstdefinities en passende power-/precisieberekening. Toets hints bij dezelfde interface met en zonder begeleiding. Toets behoud/verbetering van bestaande routes en vrijwillige migratie/retentie apart; een voorkeurstem is geen bewijs van overstappen.

**Technische regressie:** latere implementatie vereist state-/brokertests, build/type-/lintchecks waar passend, echte keyboard-/focus-/screenreaderproeven, contrast en schaal/reduced motion, responsive layout en onderbrekingsherstel. Nu zijn uitsluitend documentconsistentie, bronroutes, matrixdekking en repositoryscope gecontroleerd; er is geen gebruikersmeting of nieuwe native acceptatie.

## Besluit en open risico's

Voorkeursrichting: één consistente kern met herkenbare basisacties, contextuele hulp en expliciete gevolgen. Ondersteunende richtlijnen maken dit verdedigbaar om te testen; zij bewijzen de nieuwe productwaarde nog niet. Belangrijkste risico's blijven ontbrekende compatibiliteit/services, onvoldoende installer-/encryptieveiligheid, nog onvolledige native toegankelijkheid, bronversiedrift en een kleine of selectieve latere steekproef. Nieuwe implementaties blijven achter de gates in [het bestaande productplan](../implementation/PRODUCT-SPEC.md).
