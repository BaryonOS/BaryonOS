# Afzonderlijke publicatiekandidaat — 8 november 2026

**Deze gecontroleerde bronmomentopname is voorbereid voor een onafhankelijke
nieuwe repository; de oorspronkelijke Nieuw-OS-repository en haar geschiedenis
blijven privé. Er is nog geen nieuwe repository gepubliceerd.**

De basis is vóór opslag bijgewerkt naar main `5b2189824c32145ad5c110429517dfcea3f2720f` (PR #250): acht documentatiebestanden, alle exact op Git-blob-ID gecontroleerd. Productcode is sinds de volledig gecontroleerde snapshot `363b486b` ongewijzigd. Het bevestigde openbare richtingbesluit is behouden.

## Waarom deze scope

De oorspronkelijke bron bevat een geïntegreerde OSDev-/boekcollectie.
Voor concreet behouden proza uit een wiki-import uit 2006 is geen
herpublicatiegrant aangetoond. De upstream wiki vermeldt gemengde
licenties en vroeg afzonderlijke verklaringen voor oudere bijdragen.
Een gewone verwijdercommit zou de oude kopieën in Git- en PR-refs
bereikbaar laten. Deze kandidaat neemt die collectie en oude Git-objecten
daarom niet over.

De praktische uitsluitingsgrens is `.agents/skills/os-dev/`: 78 bestanden,
16.304.389 bronbytes. Dat is geen afkeuring van ieder bestand. Het boek,
eigen bronhulpmiddelen en metadata kunnen later afzonderlijk onder hun
passende voorwaarden worden teruggebracht. De leerroute verwijst nu naar
de primaire online bronnen.

## Exacte samenstelling

| Onderdeel | Bron |
|---|---|
| Eigen OS-code, tests, bestaande bewijsbestanden en documentatie | main `363b486b06023d3d578f8eca2b23ff49045a1757` |
| 21 hosted-workflows en nieuwe hostcache | PR #249, `d3d8e3e0a3b0b455b1832bdc53220352c2b20960` |
| Bewijscollector, lokale action, tests en bijbehorende documentatie | PR #251, `5c291cff3b9c28e4948aeb36adda3acc6ee6a8ab` |
| Uitsluitingen | De 78 skillbestanden en de private auditworkflow |
| Aanvullingen | Officiële MPL-2.0-tekst, exact padregister, licentiescope, componentregister en dit publicatiecheckpoint |

Alle 6.524 bestanden van de main-basissnapshot en de 45 geselecteerde
overlaybronbestanden uit #249/#251 zijn gecontroleerd tegen hun Git-blob-ID's. De
componentinventaris verifieert 112 exacte registryversies en alle 361
checksumvermeldingen in 62 lockfiles. Bestaande vendor-/fontlicenties
blijven behouden; [LICENSING.md](../LICENSING.md) en
[THIRD-PARTY.md](../THIRD-PARTY.md) leggen de afbakening vast.

De [integratieprovenance](verification/publication-candidate-20261108.json)
beschrijft de tussenstand na de CI-/broncollectieaanpassingen. De latere
licentie- en checkpointintegratie wordt afzonderlijk in het eindmanifest
vastgelegd in het [eindmanifest](verification/publication-final-20261108.json),
zodat de tussenstand niet als eindhash wordt gebruikt.

## Uitgevoerde controles

- 15 host-unittests voor de bewijscollector: geslaagd.
- Actionlint voor alle 21 workflows: geslaagd; optionele externe
  shellcheck/pyflakes niet gebruikt.
- 470 statische beleids-/integratiecontroles: geslaagd; alle 24 jobs
  gebruiken `ubuntu-24.04`, alle 32 externe action-aanroepen in de workflows zijn
  volledig SHA-gepind en 15 aanroepen gebruiken de beoordeelde lokale action.
- Voortgangsgenerator: **41/100**, zonder wijziging van criteria of bewijs.
- De actuele bron, inclusief gecomprimeerde bewijsbestanden en beelden,
  is in de oorspronkelijke publicatiecontrole onderzocht. De laatste
  kandidaatcontrole is aan het eindmanifest gebonden: 37 Gitleaks-kandidaten
  zijn verklaard (35 bestaande, ongewijzigde locaties en twee
  workflow-SHA-256-vermeldingen uit #251); geen onopgeloste secretbevinding.

Er is geen nieuwe Rust-build, QEMU-acceptatie of fysieke hardwaretest op
deze samengestelde boom uitgevoerd. De bronpublicatie accepteert daardoor
geen nieuw productcriterium. Historische testresultaten blijven gebonden
aan hun oorspronkelijke bron; R03 en de volledige productrelease blijven open.

## Publicatie-uitvoering na scopekeuze

Het voorgestelde doel is `simonvanieperen/nieuw-os-public`, met een nieuwe
geschiedenis die geen oude commits, branches of PR-refs erft. Bestaande
issues, wiki, discussies, Actions-runs, logarchieven en caches worden niet
geïmporteerd. De oude repository blijft privé en intact.

De nieuwe repository krijgt geen eigen runnerregistraties en geen
overgenomen secrets. Actions gebruikt leestokens, toegestane SHA-gepinde
actions en goedkeuring voor externe bijdragen. De bestaande relevante
host-/QEMU-gates worden in de nieuwe branchregels aangehouden. De echte
CI-uitvoering en zichtbaarheid worden na afloop gecontroleerd; een
voorbereide configuratie is geen groen testresultaat.

Dit verandert de bestemming uit de oorspronkelijke opdracht en vraagt
daarom een expliciete scopekeuze. Er wordt geen toestemming verondersteld
om de oorspronkelijke geschiedenis te herschrijven of te verwijderen.

## Bronnen

OSDev Wiki-bijdragers. (2011). *OSDev Wiki:License*.
https://wiki.osdev.org/OSDev_Wiki:License

Combuster. (2011, 8 juli). *OSDev Wiki Relicensing statements*
[Oorspronkelijke beheerdersverklaring].
https://forum.osdev.org/viewtopic.php?t=23838

Mozilla. (z.d.). *Mozilla Public License, version 2.0*.
https://www.mozilla.org/en-US/MPL/2.0/
