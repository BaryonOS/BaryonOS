# Productbesluit en bouwspecificatie P2 — Materiaal

## Aanvulling ADR-058 — 30 oktober 2026

Gebruikersbesluit, leidend boven de regels hieronder dat het hoofdscherm geen bestaande desktop nabootst (§2) en dat er geen dock of taakbalk komt (§5). De eindervaring krijgt een herkenbaar bureaublad, taakbalk, startmenu met zoeken, vensterbediening, inrichting en inlogscherm volgens [DESKTOP-SHELL.md](DESKTOP-SHELL.md) en [PROFILES-AND-USERS.md](PROFILES-AND-USERS.md). Materiaal blijft het gegevens-, geschiedenis- en rechtenmodel en verschijnt in de Bestanden-app, recente items en toestemmingsdialogen. P2 hieronder blijft historisch productbewijs. Eigen visuele identiteit blijft verplicht; herkenbaar is de bediening, niet de huisstijl.

## Aanvulling N01 — 27 oktober 2026

De gebruikerswijziging is leidend boven onderstaande oorspronkelijke scope van één eigen NAS. Apparaten heeft nu drie flows: **Mijn NAS**, **Ruimte ontvangen** en **Ruimte aanbieden**. Internet is afzonderlijk toegestaan voor een bevestigde NAS-identiteit. Ontvangen ruimte kan bij vrienden/familie staan; de ontvanger bevestigt dat de host zijn NAS-kopie kan verwijderen en krijgt het advies een onafhankelijke kopie te bewaren. De beheerder ziet alleen capaciteit/gebruik/status en mag intrekken+wissen zonder gasttoestemming of inhoudssleutel.

Datatoegang geeft geen beheerrol. In de mock is expliciete lokale beheeractivatie vereist; een internetroute verleent geen extern beheer. Intrekking verhoogt de generatie, blokkeert oude uploads en geeft capaciteit vrij; een nieuwe uitnodiging krijgt een nieuw ruimte-ID. Documentnamen en inhoud horen alleen in de echte client versleuteld te worden: de browserflow simuleert dit en verwerkt geen sleutels, verbindingen of echte NAS-bestanden.

De 39 state-tests en syntaxiscontroles zijn uitgevoerd; grafische browsercontrole van deze nieuwe schermen is nog open. Exacte crypto-/dienstproef en native integratiegrenzen: [NAS-SHARING-AND-INTERNET.md](NAS-SHARING-AND-INTERNET.md), [privacycontract revisie 3](../LOCAL-PRIVACY-CONTRACT.md), [UI-handleiding](../../ui/prototype/README.md).

## Aanvulling consumentenonderzoek — 28 oktober 2026

De productkeuze **Materiaal** blijft ongewijzigd en P2 blijft bevroren op de zeven bestaande schermen. De aanvullende literatuurverkenning in [CONSUMER-VALUE-2026-10-28.md](../research/CONSUMER-VALUE-2026-10-28.md) voegt geen nieuw bewezen marktvoordeel toe, maar geeft een gerichte toetsvolgorde voor de onderscheidende ervaring.

Onderzoek na de bestaande P2-acceptatie één samenhangende demonstratie (**DV-01**) waarin dezelfde ondersteunde materiaalidentiteit leidend blijft: terugvinden, beperkte toegang voor gereedschap A, bevestigd opslaan, A beëindigen, hetzelfde materiaal met gereedschap B openen, geschiedenis/herstel als nieuwe revisie, toegang van A intrekken en een nieuwe beschermde actie van A weigeren.

**Productbewijs:** taaksucces, tijd/hulp, onbedoelde kopieën, begrip van opslagstatus en rechten, herstelsucces en vrijwillige keuze om dezelfde taak opnieuw via Materiaal uit te voeren.

**Native systeembewijs:** echte capability-/dienstafdwinging, bevestigd werk dat binnen het opslagcontract het beëindigen van een gereedschap overleeft, revocation voor toekomstige acties en geen valse successtatus na onzekere opslag. Een browsermock bewijst deze eigenschappen niet.

DV-01 is een vervolghypothese en acceptatiescenario. Het verruimt P2 niet stilzwijgend en verandert geen bestaande native bewijsclaim of voortgangspunt.

## Aanvulling P3/P4 — premium native productiviteit en eigen visuele taal

Gebruikersbesluit 28 oktober 2026: privacyvriendelijkheid mag **niet** zichtbaar
aanvoelen als een functionele of esthetische concessie. De uiteindelijke
Nieuw-OS-ervaring moet qua afwerking, snelheid, typografie en samenhang kunnen
concurreren met hoogwaardige moderne consumentensoftware, zonder Windows,
macOS, Microsoft Office of andere suites visueel of interactioneel na te
bouwen.

Dit verandert de bevroren P2-taakflows niet. P2 blijft een prototype om
Materiaal te toetsen. Onderstaande eisen sturen P3 (designsysteem/appcontract)
en P4/M7 (native vertaling).

### Productiviteitsmodel

- **Materiaal blijft centraal.** Een document, tabel, presentatie, afbeelding of
  collectie is het blijvende object; gereedschappen zijn verwisselbare
  bewerkingsmogelijkheden.
- **Geen Office-ribbonkopie als basis.** Contextuele gereedschappen verschijnen
  wanneer het geselecteerde object of onderdeel ze nodig heeft. De inhoud
  domineert het scherm.
- **Herkenning zonder imitatie.** Bekende basisacties zoals openen, opslaan,
  zoeken, undo, kopiëren en exporteren blijven voorspelbaar; onderscheid komt
  uit objectcontinuïteit, geschiedenis, contextuele tools, motion en
  capabilitybewust delen.
- **Eén productfamilie.** Tekst, tabellen, presentaties, PDF/media en latere
  eigen gereedschappen gebruiken dezelfde spacing, typografie, focus,
  selectie, versiegeschiedenis, delen, herstel en foutpatronen.
- **Interoperabiliteit is een migratie-eis.** DOCX/XLSX/PPTX/PDF-import/export
  wordt als praktisch compatibiliteitsdoel onderzocht en per formaat/feature
  getest. Een bestand kunnen openen is geen claim van volledige
  round-trippariteit.

### Visuele/interactionele ambitie: quiet futurism

De gewenste uitstraling is modern en vooruitstrevend zonder decoratieve
sciencefiction: scherpe typografie, precieze grids, subtiele diepte,
responsieve motion en contextuele bediening. Privacy- en beveiligingsstatus
moeten rustig in dezelfde visuele taal passen en geen beheerconsole worden.

Belangrijke principes:

- materiaal/content krijgt visuele prioriteit boven permanente chrome;
- transitions volgen objectcontinuïteit: hetzelfde materiaal voelt als
  hetzelfde object wanneer een ander gereedschap het opent;
- motion ondersteunt plaats, status en causaliteit en respecteert
  reduced-motion;
- iconen en acties zijn direct herkenbaar op functie, met een eigen uitvoering;
- technische details zijn beschikbaar maar niet de standaardtaal voor gewone
  gebruikers;
- fouten tonen gevolg en eerstvolgende herstelactie in plaats van cryptische
  codes als primaire laag.

De bestaande P2-kleuren/tokens zijn prototypekeuzes, geen definitieve merk- of
OS-huisstijl.

### Native renderdoel

De uiteindelijke kernapps krijgen een gedeelde native UI-/renderlaag als
productdoel. Die laag moet GPU-versnelling, high-DPI, tekst, vectoren,
animaties, toegankelijkheid en consistente input/focus ondersteunen zodra de
native graphicsbasis dat toelaat.

Een verzameling los vormgegeven webviews is niet de gewenste eindarchitectuur
voor de eigen kernapps. Het exacte compositor-, GPU-, text shaping- en
renderontwerp blijft een afzonderlijk technisch besluit en wordt pas
vastgezet na de relevante G/H-mijlpalen. Het browserprototype is dus
ontwerpgereedschap, geen stil gekozen runtime.

### Lokale AI als gereedschap, niet als verplichte chatbot

Systeem-AI blijft lokaal volgens het privacycontract en wordt waar nuttig
contextueel aangeboden, bijvoorbeeld op een expliciete tekstselectie:
verkorten, verduidelijken, samenvatten, vergelijken of vertalen. AI krijgt
alleen de geselecteerde context/capabilities en de gewone workflow blijft
bruikbaar wanneer AI ontbreekt.

De UI mag lokale verwerking zichtbaar maken, maar gebruikt geen opdringerige
“AI overal”-interface als vervanging voor goede basisbediening.

### Privacyuitleg zonder popupmoeheid

Er zijn twee verschillende UI-elementen:

1. een noodzakelijke capabilitybeslissing voor nieuwe toegang;
2. optionele educatieve privacyuitleg met concreet voorbeeld of kritische vraag.

De gebruiker kan de educatieve uitleg globaal of per app uitschakelen. Dat
verleent geen nieuwe rechten en wijzigt bestaande grants niet. Een normale
handeling binnen een reeds geldige grant geeft niet telkens opnieuw dezelfde
uitleg.

Tijdens gaming/focus worden niet-urgente educatieve meldingen niet midden in
de activiteit getoond. Een bewust geactiveerde nieuwe gevoelige functie mag
wel een compacte noodzakelijke grantvraag openen.

### P3/P4-acceptatievragen

- **PX-01:** tekst-, tabel- en presentatieprototypes voelen als één productfamilie
  zonder permanente Office-ribbonimitatie.
- **PX-02:** gebruikers kunnen bekende basistaken uitvoeren zonder uitleg van
  het onderliggende capabilitymodel.
- **PX-03:** dezelfde objectversie, locatie en historie blijven herkenbaar
  wanneer een ander gereedschap wordt gebruikt.
- **PX-04:** privacyuitleg uitzetten verandert geen grantstate.
- **PX-05:** toetsenbord, 200% zoom, screenreaderroute en reduced-motion zijn
  onderdeel van hetzelfde designsysteem.
- **PX-06:** realistische documenten worden getest op renderkwaliteit, latency,
  herstel en import/export; screenshots alleen tellen niet als acceptatie.
- **PX-07:** visuele vergelijking controleert originaliteit en samenhang zonder
  te claimen dat subjectieve “mooier dan Office” al bewezen is.

Status: **besluit voor de eerste implementatie**, 26 oktober 2026. De bronnenvergelijking en conceptkeuze zijn nu uitgevoerd. Gebruikersonderzoek, grafische implementatie en native OS-integratie zijn nog niet uitgevoerd. Codex hoeft vóór P2 geen nieuwe brede conceptstudie te starten. Alleen een concreet gevonden conflict of gebruikerscorrectie heropent de keuze.

## 1. Gekozen productlogica

**Materiaal is de gekozen richting.** De gebruiker begint bij een document, foto of verzameling die van de gebruiker blijft. Apps zijn verwisselbare gereedschappen die een specifieke bewerking uitvoeren, met een expliciete reikwijdte en duur. De objectgeschiedenis, gegevenslocatie en toegangsrechten blijven zichtbaar buiten de afzonderlijke app.

Dit is een werknaam voor het interactiemodel, geen definitieve OS- of merknaam. Het ontwerp is eigen synthese, geen bewezen wereldprimeur. Een oorspronkelijke productervaring vraagt geen onbekend bestandssysteem voor de eerste proef.

De gebruiker kan ook een app rechtstreeks openen via Gereedschap; daarmee blijft de bediening toegankelijk voor mensen die vanuit een programma denken. Het hoofdscherm wordt geen appstartscherm of decoratieve imitatie van een bestaande desktop. De eerste navigatievraag is: **waar wil ik aan werken?**

## 2. Drie concepten op dezelfde vijf taken

Onderstaande vergelijking is ontwerpwerk, geen gemeten gebruikersproef.

| Taak | Materiaal: objecten centraal | Werk: activiteiten centraal | Atelier: werkoppervlak centraal |
|---|---|---|---|
| Installeer een bewerker | Voeg een gereedschap toe; het verschijnt als passende actie bij objecttypes | Voeg een gereedschap toe aan een activiteitprofiel | Plaats een gereedschap als aanpasbaar paneel op het werkoppervlak |
| Maak/bewerk een document | Open Tuinplan; kies bewerking; maak nieuwe versie van hetzelfde object | Open activiteit Tuin vernieuwen; start de volgende stap en sla resultaat op | Sleep Tuinplan naar bewerkingspaneel; stel omgeving vrij in |
| Vind iets terug | Zoek op titel/type/collectie; locatie en laatste versie direct zichtbaar | Zoek op activiteit, doel of periode; zoek daarna het bijbehorende resultaat | Zoek op paneel/werkblad of open de materiaallade |
| Onderbreek en hervat | Hervat object, selectie en bewerking; rechten opnieuw controleren | Hervat activiteit met laatste stap, vensters en conceptresultaten | Herstel volledige ruimtelijke paneelindeling |
| Herstel een fout | Open objectgeschiedenis; herstel versie als nieuwe revisie | Herstel de vorige activiteitstap, met gevolgen voor gekoppelde resultaten | Herstel paneelconfiguratie of bewerkingsgeschiedenis van geselecteerd materiaal |

**Afweging en keuze.** Materiaal sluit het directst aan bij eigenaarschap, appuitwisselbaarheid en begrijpelijke bevoegdheden. Werk helpt bij langlopende activiteiten, maar introduceert al vroeg een extra organisatielaag. Atelier biedt sterke aanpasbaarheid, maar vraagt meer inrichting en kan onrustig of inconsistent worden. P2 implementeert Materiaal, met eenvoudige collecties; geen automatische combinatie van alle drie concepten. Activiteiten en vrijere werkoppervlakken blijven latere hypotheses, niet extra P2-scope.

### Vergelijking met bekende patronen

- Windows Snap ordent appvensters en kan groepen herstellen; macOS Spaces verdeelt vensters over meerdere bureaubladen. Die patronen tonen ruimtelijke organisatie, maar bepalen niet ons objecteigenaarschap of rechtenmodel (Microsoft, z.d.; Apple, z.d.).
- Flatpak-portalen demonstreren dat een concrete bestandskeuze toegang kan verlenen zonder brede bestandstoegang. Het principe van een systeemportaal is dus bekend. Ons voorstel maakt gegevenslocatie, objecthistorie en tijdelijke gereedschapstoegang onderdeel van één doorlopende ervaring; dit is een ontworpen combinatie, geen claim dat bestaande systemen zulke onderdelen nooit hebben (Flatpak Project, z.d.).
- WCAG 2.2 levert toetsbare criteria voor het webprototype. Gebruik die voor P2; een geslaagd webprototype verklaart het toekomstige native OS niet automatisch toegankelijk of wettelijk conform (W3C, 2024).

## 3. P2 is bevroren op zeven schermen

Alle schermen moeten daadwerkelijk navigeerbaar zijn. Geen knop zonder werking of expliciete niet-beschikbaarstatus. Gebruik uitsluitend fictieve fixturedata.

| ID / route | Inhoud en primaire handeling | Verplichte toestanden |
|---|---|---|
| S01 `#/start` | Drie korte stappen: taal/bediening; lokaal profiel; opslagkeuze. Hoofdactie 'Begin lokaal'. NAS kan later. | nieuw, stap terug, voltooid; geen internetaccount of echte wachtwoordinvoer |
| S02 `#/materiaal` | Mijn materiaal: voortzetten, collecties, zoekveld, objectlijst. Hoofdactie 'Nieuw document'. | gevuld, zoekresultaat, nul resultaten, lege collectie |
| S03 `#/object/obj-001` | Tuinplan in lees-/bewerkstand, gereedschapkeuze, versiegeschiedenis, gegevenslocatie. Hoofdactie 'Bewerken'. | lezen, verzoek in behandeling, bewerken, opslaan, opgeslagen, herstelpreview |
| S04 `#/toegang` | Vertrouwd systeemoppervlak met precies wie/wat/handeling/duur. Vanuit object of gereedschapdetails; desktop als dialog, smal als volledig scherm. | aanvraag, toegestaan, geweigerd, ingetrokken; gecombineerde lees-/internetwaarschuwing |
| S05 `#/gereedschap` | Fictieve lokale catalogus, pakketdetails en geïnstalleerde gereedschappen. Hoofdactie 'Installeer voorbeeld'. | niet geïnstalleerd, controle, geïnstalleerd, foutieve handtekening, verwijderd |
| S06 `#/apparaten` | Dit apparaat en één eigen NAS. Koppeling via zichtbare identiteitscontrole; datasets en syncstatus. | niet gekoppeld, code vergelijken, gekoppeld, NAS offline, netwerk veranderd, identiteit afwijkend |
| S07 `#/assistent` | Lokale assistent met expliciet geselecteerde context. Hoofdactie 'Vat selectie samen'. | model aanwezig, model ontbreekt, onvoldoende geheugen, bezig, antwoord, geannuleerd |

De assistent gebruikt in P2 een deterministische voorbeeldrespons, nadrukkelijk gelabeld als simulatie. Er wordt geen model gedownload of inference geclaimd. Quantum krijgt hier geen decoratieve knop: de echte Q1-proef heeft een afzonderlijk uitvoercontract en wordt pas na werkende functionaliteit in de shell opgenomen.

### Vaste fixturedata

- Apparaat: 'Dit apparaat'; type desktop.
- Eigen NAS: 'Thuis-NAS'; gesimuleerde identiteit `NAS-DEMO-01`; geen bestaand IP-adres of echt apparaat.
- Objecten: `obj-001` Tuinplan (tekst, drie versies), `obj-002` Plattegrond (SVG uit lokale fixture), `obj-003` Boodschappen (tekst); collecties 'Vandaag', 'Huis' en 'Archief'.
- Gereedschappen: 'Notities' vooraf beschikbaar; 'Beeldbewerker' als installeerbaar fixturepakket; 'Webvenster' als fictief gereedschap met afzonderlijk internetrecht. Geen echte applicatiecompatibiliteit claimen.
- Foutbediening: een apart, duidelijk gemarkeerd prototypepaneel activeert NAS offline, verkeerde identiteit, weinig geheugen en ongeldige pakketsignatuur. Dit paneel behoort niet tot de uiteindelijke OS-ervaring.

## 4. Overgangen en gegevensregels

De prototypecontroller beslist over overgangen; schermcomponenten muteren geen willekeurige globale state.

| Event / beginstaat | Nieuwe staat en zichtbaar resultaat | Verboden effect |
|---|---|---|
| `finishOnboarding` | S02; lokaal profiel in fixturestate | Accountregistratie of netwerkverzoek |
| `requestEdit(obj-001, Notities)` | S04: 'Notities mag Tuinplan lezen en wijzigen tot je dit document sluit' | Automatische toegang tot collectie, NAS of netwerk |
| `grantSession` | S03 bewerken; status 'Notities · alleen dit document' | Eenmalige toestemming omzetten in permanente toegang |
| `denyGrant` | S03 lezen; inhoud ongewijzigd | Verlies van document of gedwongen appinstallatie |
| `saveDraft` | Nieuwe revisie, daarna 'Op dit apparaat opgeslagen'; syncstatus apart | 'Veilig op NAS' tonen voordat sync bevestigd is |
| `closeObject` | S02; editgrant beëindigd; cursorpositie als lokale hervatmetadata | Een actieve capability kopiëren naar een ander apparaat |
| `restoreRevision(v1)` | Eerst vergelijkingspreview; bevestigen maakt v4 met inhoud v1 | V2/v3 wissen of herstel direct naar NAS pushen zonder gespecificeerd syncgedrag |
| `pairNas` | Codevergelijking; bevestiging koppelt fixture-identiteit | Koppelen uitsluitend op hostnaam of hetzelfde wifi |
| `nasOffline` | Lokale opslag blijft bruikbaar; 'Wacht op eigen NAS' | Externe cloudfallback of blokkeren van lokaal bewerken |
| `nasIdentityMismatch` | Sync gestopt; uitleg en 'Verbinding opnieuw controleren' | Knop om onbevestigde identiteit stilzwijgend te vertrouwen |
| `networkChanged` | Sync pauzeert; lokaal werken blijft mogelijk | NAS via extern internet bereikbaar proberen te maken |
| `requestSummary` | S07 met lijst geselecteerde objecten en lokale simulatie | Algemene documentindex lezen of externe AI starten |
| `modelMissing / lowMemory` | Expliciete lokale beperking; 'Gebruik zonder assistent' blijft beschikbaar | Cloudknop of impliciete externe fallback |
| `installFixture` | Gesimuleerde handtekeningcontrole; app toegevoegd; nul grants | Installeren gelijkstellen aan toegang verlenen |
| `revokeGrant` | Toekomstige beschermde acties worden geweigerd in mockbroker | Beloven reeds gemaakte kopieën te wissen |

De S04-waarschuwing bij lees- én internetrecht luidt: 'Dit programma kan de toegestane inhoud naar een internetdienst sturen.' Toon objecten, bestemming/reikwijdte en duur. Kies standaard 'Annuleer'; een expliciete tweede actie kan de gevraagde beperkte combinatie toestaan. Dit is bewust geautoriseerd delen, geen absolute niet-lekkengarantie.

## 5. Visuele specificatie v1

Richting: rustig, precies en verzorgd; lichte minerale achtergrond, diepe groentinten, grote leesbare titels, scherpe uitlijning en ruim gebruik van witruimte. Geen gekopieerde dock, Windows-taakbalk, glaslagen of decoratieve statistieken. De objectlijst en echte gebruikersinhoud dragen het scherm.

| Token | Waarde |
|---|---|
| `color.background` | `#F4F5F1` |
| `color.surface` | `#FFFFFF` |
| `color.text` | `#162B2B` |
| `color.textMuted` | `#5D6A67` |
| `color.accent` / primary button | `#0A5E55`, tekst wit |
| `color.danger` | `#A33625`, altijd ook tekst/icoon |
| `color.warningText/background` | `#8A5200` op `#FFF3DB` |
| `color.controlBorder` | `#768783`; decoratieve scheidingslijnen mogen lichter zijn |
| `font.family` | `system-ui, "Segoe UI", sans-serif`; geen externe fontrequest |
| `font.size/lineHeight` | body 16/24; metadata 14/20; label 14/20; sectie 20/28; paginatitel 32/40 |
| `font.weight` | 400 body, 500 labels, 600 titels; geen bodytekst dunner dan 400 |
| `space` | 4, 8, 12, 16, 24, 32, 48 px |
| `radius` | 8 controls, 14 surfaces, 18 dialogs; pill uitsluitend voor statuslabels |
| `control.minHeight` | 44 px; icon-only doelgebied 44×44 px |
| `focus` | 3 px accent-outline, 3 px offset; volledig zichtbaar |
| `shadow.dialog` | 0 16px 48px rgba(22,43,43,.16); niet als enige begrenzing |
| `motion` | 120 ms state/hover; 180 ms panel; 220 ms route; easing cubic-bezier(.2,0,0,1) |
| `reducedMotion` | overgang zonder verplaatsing; maximaal 50 ms fade; geen essentiële informatie afhankelijk van animatie |

Berekend met de sRGB/WCAG-relatieve-luminantieformule: hoofdtekst/achtergrond **13,56:1**, secundaire tekst/achtergrond **5,15:1**, witte tekst/accent **7,65:1**, gevaartekst/wit **6,75:1**, waarschuwingtekst/-achtergrond **5,81:1**, controlborder/wit **3,77:1**. Dit zijn berekeningen van vaste kleurparen, nog geen controle van alle gerenderde toestanden.

**Layout:** desktop vanaf 1200 px: navigatierail 224 px, hoofdinhoud flexibel met max-breedte 1120 px, optionele inspector 288 px binnen de hoofdindeling. Op 768–1199 px: rail 72 px met labels via tooltip én toegankelijke naam; inspector als opvraagbaar paneel. Onder 768 px: één kolom met compacte bovennavigatie en expliciet menu; dialogs volledig scherm. Op 320 CSS px geen horizontale paginascroll; lange titels wrappen. De documentinhoud kan eigen overflow hebben wanneer de inhoud dat vereist.

**Componentregels:** lijstregels zijn 72 px minimaal, titel 16 px, metadata 14 px; één primaire actie per paneel. Status heeft tekst plus icoon. Skeleton alleen bij werkelijk geplande asynchrone state; geen nepwachttijden boven 300 ms om technologie te suggereren. Foutmeldingen benoemen oorzaak voor zover bekend, gevolgen en één eerstvolgende handeling.

## 6. Bedienings- en toegankelijkheidscontract

- Gebruik semantische HTML, echte buttons/inputs, kophiërarchie, landmarks en een skiplink. Native `dialog` voor S04 op desktop; focus keert terug naar de opener. Achtergrond tijdens modaliteit niet bedienbaar.
- Tab/Shift+Tab door alle controls; Enter voert primaire geselecteerde handeling uit; Escape sluit alleen niet-destructieve overlay en bewaart een lokale draft; geen verlies bij toevallig Escape.
- `Ctrl+K` opent zoeken/acties binnen het prototype als het browser-event beschikbaar is; zichtbare zoekknop blijft de betrouwbare route. Een browser kan systeemshortcuts reserveren.
- `Ctrl+S` slaat in S03 de fixtureversie op en meldt de nieuwe status; zichtbare Opslaan-knop blijft bestaan.
- Pijltoetsen verplaatsen focus in een geopende resultatenlijst; geen documentbewerktoetsen onderscheppen wanneer tekstfocus actief is.
- `Alt+Links` is geen vereiste, vanwege browserhistorie; eigen Terug-knop werkt altijd.
- Geen sleepactie zonder knoppenalternatief; geen hover-only functies.
- Gebruik een polite live region voor opslagstatus, gekoppelde NAS en ingetrokken toegang. Foutmeldingen krijgen context zonder continu onderbrekende aankondigingen.
- WCAG 2.2 AA is doel voor P2: gewone tekst minstens 4,5:1, grote tekst minstens 3:1; betekenisvolle UI-grenzen/staten minstens 3:1. Projectdoel is 44 px bediening en zichtbare 3 px focus. Dit zijn gekozen strengere componentregels, geen claim dat alle WCAG-criteria al gehaald zijn.
- Test browserzoom 200%, toetsenbord en ten minste één schermlezer in de beschikbare omgeving; rapporteer ontbrekende apparatuur of niet uitgevoerde checks.

## 7. Exact P2-implementatiepad en eigenaarschap

**Kies dependencyvrije HTML/CSS/JavaScript-ES-modules.** Geen React, package-installatie, CDN, extern font, remote afbeelding of analytics. Deze keuze bouwt een interactieprototype op de ontwikkelhost; zij legt geen native compositor-technologie vast en vervangt de eigen kernel niet.

Bestanden die P2 mag maken:

- `ui/prototype/index.html`: semantisch document, routeshell en CSP.
- `ui/prototype/tokens.css`: uitsluitend tokens.
- `ui/prototype/components.css`: layout/componenten/responsive/reduced motion.
- `ui/prototype/app.js`: router, rendercoördinatie en events.
- `ui/prototype/state.js`: pure reducer en selectoren.
- `ui/prototype/fixtures.js`: deterministische synthetische data.
- `ui/prototype/broker.js`: expliciete mock van object-, grant-, NAS-, install- en AI-interfaces.
- `ui/prototype/README.md`: runpad, simulatiegrens en checks.
- `ui/prototype/tests/state.test.mjs`: eventuele betekenisvolle state-/negatieve mocktests met ingebouwde Node-testmodule wanneer Node beschikbaar is.

Eén builder bezit `app.js/state.js/broker.js`. Een visuele builder mag alleen `tokens.css/components.css` wijzigen na afspraken over componentnamen. Verifier werkt alleen-lezen. P2 wijzigt geen kernel-, boot- of cryptocode.

Runpad vanuit de repositoryroot:

```bash
python3 -m http.server 8765 --bind 127.0.0.1 --directory ui/prototype
```
Open `http://127.0.0.1:8765/` in de lokale testbrowser. Als poort 8765 bezet is, kies een andere vrije loopbackpoort en documenteer die. Geen publieke deployment of LAN-binding.

P2 bewaart uitsluitend fictieve toestand in `sessionStorage` onder `nieuw-os-prototype-v1`; refresh kan hervatten. Een zichtbare 'Demo opnieuw beginnen' wist alleen die sleutel. Geen echte gebruikersbestanden, wachtwoorden, NAS-login of formulieruploads. Het demoheaderlabel 'Interactiemodel · simulatie' blijft zichtbaar op ieder scherm.

CSP in het HTML-document: `default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'none'; connect-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'`. Geen inline scripts, eval, workers of service worker. Alle imagery is lokale SVG/fixture. De eerste loopbackrequests voor HTML/CSS/JS zijn nodig; daarna doet de app geen netwerkverzoeken. Browser-/extensieverkeer buiten de app valt buiten deze prototypeclaim.

## 8. Logische interface voor prototype en latere port

Dit is een UI-contract, geen vervanging voor het kernel-ABI. Object-ID's zijn UI-referenties en verlenen zelf geen rechten.

```text
Object { id, title, mediaType, collectionId, revision, localState, nasState }
GrantRequest { toolId, objectIds[], operations[], lease: "session", internet: false|destination }
Grant { id, toolId, objectIds[], operations[], expiresOn: "objectClose", state }
NasState = unpaired | verifying | online | offline | networkChanged | identityMismatch
AiState = ready | missingModel | lowMemory | running | cancelled | result
```

Mockbrokeroperaties: `listObjects`, `openObject`, `requestGrant`, `grant`, `deny`, `revoke`, `saveRevision`, `restoreAsNewRevision`, `pairNas`, `setNasCondition`, `installFixture`, `summarizeFixture`. Gebruik returncodes `DENIED`, `NOT_FOUND`, `STALE_REVISION`, `NAS_UNAVAILABLE`, `IDENTITY_MISMATCH`, `MODEL_MISSING`, `RESOURCE_LIMIT`, `INVALID_PACKAGE`. Geen scherm mag mislukking vertalen naar een geslaagde status.

Een latere native adapter vervangt de mock; interfaces voor capabilities, revocation, versies en foutafhandeling moeten dan met echte services worden verbonden. De UI zelf is nooit de security-enforcementlaag.

## 9. Acceptatie voor de eerstvolgende builder

1. **P2-01:** alle zeven routes en verplichte toestanden bestaan; vijf taken uit §2 volledig doorlopen met fixtures.
2. **P2-02:** geweigerde en ingetrokken rechten blokkeren de betreffende mockoperaties; installatie maakt nul grants.
3. **P2-03:** NAS offline/verkeerde identiteit/netwerkovergang toont correct herstelgedrag; lokaal opslaan blijft werken en geen externe fallback wordt aangeboden.
4. **P2-04:** AI zonder model/geheugen toont uitsluitend lokale alternatieven; voorbeeldantwoord is herkenbaar simulatie.
5. **P2-05:** oudere revisie herstellen maakt een nieuwe revisie; de eerdere geschiedenis blijft intact.
6. **P2-06:** alle bediening met toetsenbord, focusherstel en 200% zoom; contrastevaluatie van werkelijk gebruikte toestanden.
7. **P2-07:** screenshots op 1440×900, 1024×768 en 390×844; aanvullend reflow op 320 CSS px. Geen clipping, verborgen primaire acties of onverwachte overlap.
8. **P2-08:** inspecteer app-netwerkverkeer tijdens alle flows: uitsluitend initiële eigen loopbackassets; geen fetch, websocket, beacon, analytics, extern font of remote afbeelding.
9. **P2-09:** ieder fout-/leeg-/loading-scherm gebruikt dezelfde componentregels en concrete tekst. Geen decoratieve werkende-OS- of hardwareclaims.
10. **P2-10:** lever echte testresultaten en screenshots plus lijst niet uitgevoerde controles. Een screenshot bewijst geen toegankelijkheid, systeemveiligheid of NAS-koppeling.

P2 is **bouwklaar op documentniveau**. Geen gebruikers hebben deze concepten getest en er is nog geen interface gebouwd. Een eerste echte gebruikersproef volgt na P2; pas de gekozen richting dan aan op concrete observaties, niet vooraf opnieuw een algemene brainstorm starten.

## Bronnen (APA 7)

Apple. (z.d.). *Work in multiple spaces on Mac*. Geraadpleegd op 26 oktober 2026, van https://support.apple.com/en-sg/guide/mac-help/mh14112/mac

Flatpak Project. (z.d.). *Sandbox permissions*. Geraadpleegd op 26 oktober 2026, van https://docs.flatpak.org/en/latest/sandbox-permissions.html

Microsoft. (z.d.). *Snap your windows*. Geraadpleegd op 26 oktober 2026, van https://support.microsoft.com/en-us/windows/experience/snap-your-windows

World Wide Web Consortium. (2024, 12 december). *Web Content Accessibility Guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/

Tokenwaarden, conceptkeuze, taakscenario's, interfaces, schermen en acceptatietests zijn eigen projectbesluiten. De bronnen worden alleen gebruikt voor de genoemde bestaande patronen en toegankelijkheidscriteria.

