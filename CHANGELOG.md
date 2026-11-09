# Changelog

## 2026-11-09

### Documentation and governance

- The README is reduced to project introduction, architecture overview, status, quick start, and navigation; historical development updates are archived.
- PROJECT-STATE and PROGRESS are separated into current status, criteria register, and historical snapshots.
- CONTRIBUTING and SECURITY now define concrete contribution, merge, and reporting paths.
- The official MPL-2.0 text, licensing scope, covered-file register, and third-party index are added.
- Active core links point to `simonvanieperen/BaryonOS`.

## 2026-11-03

Gebaseerd op commitbereik
`ad2b981fe1f2bcccb03cb776b2281d32529d2f00..380111b68688c981eec6abb8fab29174a29f5601`:
14 commits, van 30 oktober t/m 2 november 2026. De eindcommit is inbegrepen.

### Functies

- Materiaal en Rekenmachine zijn verder geïntegreerd in de native
  venstersessie, met klembord, bijgewerkt tekenen en herstel na een
  compositorstoring.
- Sleutelstatus blijft behouden na een koude herstart; het sleutelbeheer is
  uitgebreid met rotatie en intrekking.
- Een RTC-klok kan optioneel de UTC-tijd aan een ring-3-dienst leveren.
- De native route bevat een TPM-proef en een bediende testversie van Materiaal;
  dit zijn ontwikkelproeven, geen release- of hardwareacceptatie.

### Bugfixes

- Herstelde native bediening voor de opdrachtenkiezer, proportionele
  aanwijzerposities en het volledig opnieuw tekenen van dockknoppen.
- Een TCP-verbindingsdeadline geldt niet langer voor een al gevestigde
  verbinding; gesloten verbindingen worden correct als gesloten gemeld.
- Herstelde betrouwbare afhandeling van NAS-bewerkingen met onzekere uitkomst
  en isoleerde de configuratie van de native probe.
- Het wachtwoordveld valideert invoer vooraf en geeft de ingevoerde waarde niet
  terug aan de bediening.

### Onderhoud

- De preflight meldt nu duidelijk wanneer de hosttest-linker ontbreekt.
- CI en verificatiearchieven zijn uitgebreid voor componentintegratie en
  native desktop-, netwerk- en opslagcontroles.
- Het geluidsstuurprogramma kan begrensde PCM-formaatinformatie verwerken;
  dit is geen bewijs van volledige geluidsuitvoer.

### Incompatibele wijzigingen

- Geen vastgesteld in deze commitreeks.
