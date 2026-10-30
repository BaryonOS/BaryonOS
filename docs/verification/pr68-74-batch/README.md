# Review en integratie van PR68 tot en met PR74

Datum: 30 oktober 2026. Integratiebranch: `chatgpt/pr68-74-review-20261030`.
De implementatiebron staat in de [voortgangsbeoordeling](../../PROGRESS.md).

## Aanleiding en bronselectie

De oorspronkelijke PRs bevatten overlappende wijzigingen in afhankelijkheden,
workflows en centrale documentatie. De samengestelde kandidaat verwerkt alleen
de gevraagde PRs en noodzakelijke combinatiecorrecties. Het volledige
[source-manifest](source-manifest.json) legt de vaste main-bron, de bronheads,
het behoud van de inhoud en de uiteindelijke bestandshashes vast. Het manifest
is opgesteld vóór het toevoegen van deze uitleg en de centrale statusbestanden;
het beschrijft de implementatie- en bewijsbestanden, niet zijn eigen hash.

PR69 is een reeds gesloten negatieve rulesettest en wordt niet geïntegreerd.
PR70 was al gemerged. PR71 is tijdens deze review gemerged als `484ac33d`, met
exact dezelfde tree als de door ons bijgewerkte bron `479415e`. De bestaande
NAS-code blijft daarmee behouden. De afzonderlijk onderzochte replayprofiel-
wijziging is niet opgenomen: de actuele main heeft geen native NAS-runner die
die aanvullende wijziging voor deze afhandeling noodzakelijk maakt.

## Gerichte reparaties en controle

- PR68: de semantische venstergrenzen waren al fysiek en werden bij 200% schaal
  nogmaals vermenigvuldigd. De regressietest faalde op de oorspronkelijke
  renderer en slaagt met de correctie. De bestaande 20 UI-hosttests, Clippy
  met `-D warnings` en formatteringscontrole slagen op de vastgelegde bronnen.
  Zie [logs en manifest](../pr68-semantic-window-bounds/README.md).
- PR71: de Python-tests voor replay-handoff draaien in de CI, de test voor de
  repositorygrens gebruikt een eigen tijdelijke map en de oude CI-beschrijving
  is expliciet historisch gemaakt. De oorspronkelijke fout en de geslaagde
  negen tests zijn vastgelegd in [het reviewbewijs](../pr71-review-fixes/README.md).
- PR72: de runtimebeschrijving benoemt de werkelijke `zeroize`-afhankelijkheid.
  De samengestelde kandidaat bewaart de bestaande envelope-opruiming en
  actualiseert de nieuwe zelfstandige lockbestanden waar deze nodig zijn.
- PR73: de validatie van succesvolle antwoorden is gecombineerd met de al
  aanwezige afwijzing van gereserveerde bytes. Beide controles en hun
  regressietests blijven aanwezig. De oude centrale statusbestanden worden
  bewaard in het integratiearchief, terwijl de actuele status op main voortbouwt.
- PR74: de 18 native B03-stappen zijn op bron `26f45321` groen uitgevoerd.
  Het gedeelde workflowbestand behoudt zowel die 18-stapsproef als de dependency-
  voorbereiding van PR72. De gecombineerde kandidaat krijgt eigen CI.

De gepubliceerde PR bevat de resultaten van de aanvullende controle van de
gezamenlijke kandidaat. Oude logs worden niet hernoemd alsof zij een nieuwe
head zouden bewijzen. Een eventuele gewijzigde bron vraagt passende nieuwe
controle voordat die wordt gemerged.

## Bewijsgrens en voortgang

Hosttests en freestanding builds zijn geen nieuwe native productacceptatie.
Exacte vorige-boot NAS-replay blijft afhankelijk van de private tokenexport
en startupbinding; deze afhandeling claimt geen geslaagde native NAS-, E2EE-
of toegankelijkheidsacceptatie. De vaste rubric en alle acceptatiescores
blijven ongewijzigd: **34/100**. De datum, taak en implementatiebron van de
voortgangsbeoordeling zijn wel bijgewerkt.

De kandidaat wordt pas gemerged na de actuele branchregels, passende CI,
reviewafhandeling en een hercontrole van inhoudsbehoud tegen de originele
heads. Terugdraaien gebeurt via een afzonderlijke revert-PR van de eventuele
integratiesquash, met opnieuw de geldende controles; historische bewijsbestanden
blijven beschikbaar in hun broncommits en het expliciete archief.
