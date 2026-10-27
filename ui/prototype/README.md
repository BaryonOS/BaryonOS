# Materiaal · interactieprototype P02

Dit is een lokaal HTML/CSS/JavaScript-prototype met synthetische gegevens. Het draait in de browser op de ontwikkelhost, nog niet op de eigen kernel. Het voert geen echte NAS-synchronisatie, pakketinstallatie, cryptografische controle of AI-inferentie uit.

## Starten

Vanuit de projectroot:

```powershell
python -m http.server 8765 --bind 127.0.0.1 --directory ui/prototype
```

Open [Materiaal lokaal](http://127.0.0.1:8765/). Stop de server met `Ctrl+C`. Bij een bezette poort kan een andere vrije loopbackpoort worden gekozen. Open de modules via deze lokale server; `file://` is geen ondersteund runpad.

De eerste start bestaat uit drie stappen. Daarna zijn de routes `#/materiaal`, `#/object/obj-001`, `#/toegang`, `#/gereedschap`, `#/apparaten` en `#/assistent` bereikbaar. `#/start` blijft via het prototypepaneel beschikbaar.

## Wat werkt in de simulatie

- Zoeken, collecties, lege resultaten, een nieuw document en bestaand materiaal openen.
- Tijdelijke toegang aanvragen, weigeren, toestaan en intrekken. Het sluiten of verlaten van het document beëindigt de actieve rechten. Vernieuwen van de pagina bewaart fictieve inhoud, maar hervat geen actieve rechten.
- Tuinplan bewerken en als nieuwe versie opslaan. Versieherstel toont eerst een vergelijking en voegt vervolgens een nieuwe versie toe. Bestaande versies blijven behouden.
- Een lokale bewerkingsdraft en cursorpositie bij het sluiten/hervatten van hetzelfde document. Het prototype bewaart één onafgemaakte bewerkingsdraft tegelijk.
- Voorbeeldinstallatie met geldige/ongeldige signatuur, verwijderen zonder materiaal te verwijderen en nul rechten bij installatie.
- Expliciete waarschuwing voor Webvenster met leesrecht en een fictieve internetbestemming. Er wordt geen verbinding gemaakt.
- NAS-koppeling via bevestigde voorbeeldidentiteit. Offline, gewijzigd netwerk en afwijkende identiteit pauzeren de gesimuleerde verbinding; lokaal opslaan blijft werken.
- Een vaste voorbeeldsamenvatting van uitsluitend geselecteerde context. Ontbrekend model, onvoldoende geheugen en annulering hebben eigen toestanden.

Het **Prototypepaneel** onderaan ieder werkscherm activeert foutscenario's. Het is expliciet testbediening en geen onderdeel van het beoogde OS-product.

## Bediening

- `Tab` / `Shift+Tab`: controls doorlopen. `Enter` activeert een gefocuste knop/link.
- `Ctrl+K` (of `Cmd+K`): naar het zichtbare zoekveld, als de browser de toets doorgeeft.
- `Ctrl+S` (of `Cmd+S`): een actieve bewerking opslaan. De knop **Opslaan** blijft beschikbaar.
- Pijl omhoog/omlaag: zoekresultaten doorlopen vanuit het zoekveld of een resultaat.
- `Escape`: toegangsaanvraag annuleren of herstelpreview sluiten. De lokale draft blijft behouden.
- Onder 1200 px opent **Informatie** de objectinspector. Onder 768 px opent **Menu** de hoofdnavigatie.

Native `dialog` sluit de achtergrond uit van toetsenbordbediening en begint op **Annuleer**. Na sluiten keert de focus terug naar de documentactie; na toestaan heet die actie **Opslaan**. De smalle weergave gebruikt dezelfde dialoog over het volledige scherm.

## Architectuur en grenzen

`fixtures.js` bevat uitsluitend fictieve data. `state.js` beheert overgangen als pure reducer; `broker.js` controleert mockoperaties opnieuw op object, handeling, intrekking, levensduur en revisie. `app.js` verzorgt de routes en semantische bediening. `tokens.css` volgt de vastgelegde productkleuren en `components.css` de responsive componentregels.

De mockbroker is geen echte beveiligingsgrens. In een toekomstige native port moeten kernel/services de capabilities afdwingen. De vertrouwde visuele dialoog kan in een browser worden nagebootst; daarvoor is later een compositor-/inputgrens nodig. Installeren is gesimuleerd en zegt niets over compatibiliteit of cryptografische authenticiteit.

Alleen `sessionStorage['nieuw-os-prototype-v1']` wordt gebruikt. **Demo opnieuw beginnen** wist alleen die sleutel. Bij ongeldige opgeslagen structuur wordt een schone fixturestaat geladen. Tooldefinities worden bij hervatten vanuit de vaste fixtures opgebouwd; opgeslagen data mogen geen gereedschapsrechten of markupvelden veranderen.

De app heeft geen netwerk-API, externe assets, fonts, framework of dependencies. De CSP blokkeert verbindingen, formulieren, plugins, inline script en externe resources. De eigen initiële HTML/CSS/JS-verzoeken naar de lokale server blijven nodig. Deze beperking geldt voor de prototypecode en bewijst niets over browserextensies of de latere OS-netwerklaag.

## Uitgevoerde lokale controles

Op 27 oktober 2026 met Node **v24.16.0**, vanuit de projectroot:

```powershell
node --test ui/prototype/tests/state.test.mjs
node --check ui/prototype/app.js
```

Resultaat: **22 tests geslaagd, 0 mislukt**, beide commando's exitstatus 0. De tests dekken onder andere scope/intrekking, verlopen rechten, gewijzigde opslagdata, installatie zonder rechten, NAS-foutgevallen met lokale opslag, ontbrekende AI-capaciteit, contextselectie, annulering met verouderde callbacks, behoud van versiegeschiedenis en stale-revisionafwijzing. De netwerkcheck is een bron- en CSP-check; daadwerkelijke browsernetwerkobservatie is een afzonderlijke acceptatiecontrole.

De daadwerkelijk gebruikte basistokens zijn met de sRGB-luminantieformule gecontroleerd op tekstcontrast (minimaal 4,5:1) en controlbordercontrast (minimaal 3:1). Dit dekt vaste kleurparen en is geen volledige toegankelijkheidsverklaring.

De hoofdagent verzorgt browseracceptatie en screenshots op 1440×900, 1024×768 en 390×844, plus reflow op 320 CSS px. De uitgevoerde browserchecks en screenshots worden in het integratieverslag vastgelegd. Screenreadergebruik, een echte gebruikersproef en native OS-toegankelijkheid zijn hiermee niet aangetoond.

De gerichte P02V-vervolgcontrole op 27 oktober 2026 herstelt skiplinknavigatie, focusbehoud op Menu/Informatie, zoeken op documenttype en de voorgeschreven 288px-inspector. De actuele Node-suite heeft **23 geslaagde tests**. Nieuw bewijs, bronhashes, screenshots en resterende toegankelijkheidsafhankelijkheden staan in [verification/P02V-2026-10-27.md](verification/P02V-2026-10-27.md). De eerdere rapporten blijven als historische uitvoeringen behouden.

## Visuele aansluiting op de gekozen specificatie

De bestaande tekstspecificatie `docs/implementation/PRODUCT-SPEC.md` is de ontwerpbron; er is geen nieuwe conceptselectie of rasterconcept toegevoegd. De implementatie gebruikt de voorgeschreven minerale achtergrond, groene primaire bediening, systeemtypografie, materiaallijst, 224/72 px navigatierail en volledige smalle dialoog. De hoofdtaak begint bij documenten; gereedschappen zijn ondergeschikt.

Tijdens browserreview is een gevonden overlap tussen zoekicoon en invoertekst hersteld door de inputpadding expliciet te begrenzen. Verdere visuele bevindingen horen bij de integratiecontrole. De lokale SVG-plattegrond is een fictief document uit de specificatie, geen externe illustratie of hardwarebewijs.

## Herstel en volgende stap

De server stoppen verandert geen systeembestanden. **Demo opnieuw beginnen** herstelt de fictieve beginstaat en verwijdert de fictieve bewerkingen van deze demo. De ZIP-bron en bestanden buiten `ui/prototype/` worden door het prototype niet gewijzigd.

Volgende productstap: gecontroleerde gebruiks- en schermlezerproeven, daarna vervanging van de mockinterfaces door echt afgedwongen kernel-/servicecontracten. Dit prototype geeft geen conformiteits- of beveiligingsgarantie.

## Uitgevoerde browsercontrole

Op27 oktober2026 zijn de kerntaken, modal-focuslus/annulering en vier viewportbreedtes werkelijk in de browser gecontroleerd. Netwerkregistratie zag zeven initiële loopbackassets en daarna nul aanvullende verzoeken tijdens de gecontroleerde flows; console zonder warnings/errors. Zie ../../docs/verification/browser-report.json en screenshots/. De eindcontrole gebruikte127.0.0.1:8776 na lege responses op8765. Echte200%-browserzoom, schermlezer en menselijke gebruikersproef zijn nog open; reflow op320 CSS px is afzonderlijk aangetoond.
