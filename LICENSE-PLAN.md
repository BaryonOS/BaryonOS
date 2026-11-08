# Openbare ontwikkeling en licentiecontrole

## Bevestigd projectbesluit — 8 november 2026

Ik heb **gekozen voor openbare ontwikkeling van Nieuw OS**.
Het bestaande MPL-2.0-plan is het **bevestigde uitgangspunt voor de
licentiecontrole**, met MPL-2.0 als beoogde licentie voor de eigen OS-code.
Dit vervangt de eerdere onbesliste ontwikkelrichting; het besluit past de
licentie nog niet toe en tekent de controle niet af.

Besluit, consequenties en herzieningscriteria:
[ADR-PUBLIC-DEVELOPMENT-20261108](docs/DECISIONS.md#adr-public-development-20261108--openbare-ontwikkeling-met-mpl-20-als-controlebasis).
De actuele uitvoeringsstand staat in [PROJECT-STATE.md](docs/PROJECT-STATE.md).

| Onderdeel | Status |
|---|---|
| Openbare ontwikkeling als projectrichting | Bevestigd op 8 november 2026 |
| MPL-2.0 als controlebasis voor eigen OS-code | Bevestigd uitgangspunt; nog geen toegepaste projectlicentie |
| Licenties voor documentatie, assets en bijdragen | Afzonderlijk vast te stellen en te controleren |
| Rechten, componentinventaris, compatibiliteit en notices | Controle nog open |
| Repository daadwerkelijk openbaar maken | Afzonderlijke uitvoeringsstap na de publicatiecontroles |

## Betekenis van de MPL-2.0-richting

Openbare ontwikkeling maakt inspectie en samenwerking mogelijk. MPL-2.0
staat gebruik, kopiëren, wijzigen en verspreiden, ook commercieel, toe onder
haar voorwaarden. Het auteursrecht blijft bij de rechthebbenden; MPL is geen
kopieerverbod of exclusiviteitsgarantie. [1–2]

Het copyleft geldt op bestandsniveau: verspreide wijzigingen aan gedekte
bestanden blijven onder MPL vallen. Nieuwe afzonderlijke bestanden zonder
MPL-code hoeven daardoor niet automatisch MPL te worden. Bij verspreiding
van binaries moet de bijbehorende gedekte broncode voor ontvangers
beschikbaar zijn en moet duidelijk zijn hoe zij die verkrijgen. Behoud de
licentie- en auteursrechtvermeldingen. Alleen private wijzigingen brengen
geen algemene publicatieplicht mee. Merkrechten worden niet automatisch
meegegeven. [1–2]

## Bestaande controles vóór publieke publicatie

De vijf voorwaarden uit het oorspronkelijke plan blijven open totdat er
controleerbaar bewijs is; alleen het richtingbesluit hierboven is bevestigd.

1. **Bepaal rechten en reikwijdte.** Controleer rechthebbenden en herkomst van
   eigen code en bijdragen, inclusief overgenomen voorbeelden en
   AI-ondersteunde wijzigingen. Werk vanuit MPL-2.0 voor de eigen OS-code;
   stel de voorwaarden voor documentatie, assets, modellen en bijdragen
   afzonderlijk vast. Een algemeen MPL-label over de hele repository is
   hiervoor onvoldoende.
2. **Pas de gekozen licentie correct toe.** Neem na controle de ongewijzigde
   officiële licentietekst op als `LICENSE`; leg expliciet vast welke bestanden
   zij dekt en behoud externe licenties afzonderlijk. Dit document is geen
   vervanging voor die toepassing.
3. **Leg notices en bijdragen vast.** Regel passende SPDX-identificatie,
   auteursrechtvermeldingen, bronverwijzingen, wijzigingen aan externe code
   en een bijdragenprocedure die de benodigde rechten duidelijk maakt.
4. **Controleer de werkelijke componenten.** Beoordeel alle gebruikte
   workspaces/lockfiles, directe en transitieve dependencies, vendored code,
   patches en meegeleverd materiaal; onderscheid hostgereedschap van
   gedistribueerde broncode en binaries. Controleer licentiecompatibiliteit
   en broncode-/noticesverplichtingen per distributievorm. Voeg geen eigen
   'alleen Europees', 'niet voor AI' of 'niet-commercieel' verbod toe en noem
   dat daarna MPL-2.0 of open source.
5. **Rond de inventaris af vóór publicatie.** Bewaar een commitgebonden
   dependency-/licentie-inventaris met de uitkomst per component en de
   benodigde notices/bronlevering. Los ontbrekende rechten of onverenigbare
   voorwaarden aantoonbaar op, bijvoorbeeld door toestemming, vervanging
   of uitsluiting van het betreffende materiaal. Release-SBOM en
   productacceptatie volgen hun eigen bestaande criteria.

Gebruik [DEPENDENCIES.md](docs/DEPENDENCIES.md) en de bestaande
componentregisters als basis; één rootlockfile dekt niet alle onderdelen.

### Concreet aandachtspunt: meegeleverde kennisbronnen

De repository bevat onder `.agents/skills/os-dev/` extern bronmateriaal.
Het bijbehorende [bron- en licentieregister](.agents/skills/os-dev/BRONNEN-EN-LICENTIES.md)
meldt gemengde OSDev Wiki-licenties, CC BY-NC-SA 3.0 US voor het
Little OS Book-proza/PDF en afgeleid materiaal, en afzonderlijk als publiek
domein beschreven codevoorbeelden. Dit zijn bestaande bronvermeldingen,
geen afgeronde juridische controle van ieder fragment.

De MPL-richting voor eigen OS-code herlicentieert dit materiaal niet.
Controleer vóór openbaarmaking per onderdeel de publicatiemogelijkheid,
attributie en toepasselijke beperkingen, of houd het buiten de te publiceren
inhoud. De controle betreft ook de Git-geschiedenis die openbaar wordt;
alleen verwijderen uit de laatste versie kan oudere kopieën laten bestaan.
Deze opdracht geeft geen toestemming voor een geschiedeniswijziging.

## Grens tussen richting, publicatie en productrelease

De openbare ontwikkelrichting is besloten. De concrete publicatiestap volgt
na de bestaande licentie- en overige publicatiecontroles; dit document
verandert de GitHub-zichtbaarheid niet. De afzonderlijke repository-/CI-
controle vervangt de licentiecontrole evenmin.

Een openbare ontwikkelbron mag nog onvoltooide functies hebben, mits de
status en beperkingen kloppen. Het openbaar maken van broncode tekent dus
geen native test, veiligheidseigenschap, hardwareondersteuning of voltooide
productrelease af. De criteria in [FULL-TEST-RELEASE.md](docs/FULL-TEST-RELEASE.md)
blijven van kracht; R03 uit de voortgangsrubric blijft open.

## Bronnen

Projectbesluit: Simon van Ieperen, 8 november 2026. Openbare ontwikkeling
met het bestaande MPL-2.0-plan als uitgangspunt voor de licentiecontrole.
Verdere projectafweging: [PRODUCT-AND-SOVEREIGNTY.md](docs/PRODUCT-AND-SOVEREIGNTY.md)
en [EUROPE-AND-RELEASE.md](docs/implementation/EUROPE-AND-RELEASE.md).

1. Mozilla. (z.d.). *Mozilla Public License, version 2.0*, §§ 1.4, 1.7, 1.10,
   2.1–2.3, 3.1–3.4 en Exhibit A. Geraadpleegd op 8 november 2026, van
   https://www.mozilla.org/en-US/MPL/2.0/
2. Mozilla. (2024, 30 januari). *MPL 2.0 FAQ*, vragen 1, 4–12, 18 en 22.
   https://www.mozilla.org/en-US/MPL/2.0/FAQ/
