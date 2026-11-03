# Changelog

## 2026-11-03

Gebaseerd op de 14 commits van `main..380111b` (30 oktober t/m 2 november
2026).

### Functies

- Materiaal en Rekenmachine zijn verder geïntegreerd in de native
  venstersessie, met klembord, bijgewerkt tekenen en herstel na een
  compositorstoring.
- Sleutelstatus blijft behouden na een koude herstart; het sleutelbeheer is
  uitgebreid met rotatie en intrekking.
- Een RTC-klok kan optioneel de UTC-tijd aan een ring-3-dienst leveren.

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

### Incompatibele wijzigingen

- Geen vastgesteld in deze commitreeks.
