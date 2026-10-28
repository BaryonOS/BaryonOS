# R09 — onafhankelijke integratie- en bewijsaudit M6d

## Uitkomst

**Geslaagd, geen resterende materiële bevindingen binnen de onderzochte M6d-scope.** De onafhankelijke controle op de voltooide native bundel is daadwerkelijk uitgevoerd met exitcode 0. De audit heeft geen VM gestart, geen build uitgevoerd en geen repositorybestand of testdisk gewijzigd.

Uitvoeringsbron: `88e6c04be218b8b09ea4f0a226f6b9719da706a6`.

Zelfstandig opnieuw berekende bronfingerprint v3: `8ca6d5d5291fde62f91fb47dca7df4d058424604449fb32b9e5899b1d4fbef93`.

De integratie sinds checkpoint `b739183` wijzigde geen native kernel-, userspace- of drivercode. De eerder onderzochte onzekerheidsafhandeling blijft intact. De geïntegreerde M6c-hostlogbinding bewaart nu de SHA-256-hashes uit het gatetijdstip en controleert die opnieuw vóór en ná het kopiëren. De M6b-archiefbestemming heeft dezelfde begrenzing tot één directe submap van `docs/verification`.

## Zelfstandig gecontroleerd bewijs

| Onderdeel | Gecontroleerde uitkomst |
|---|---:|
| Gategroepen, in de vereiste vaste volgorde | 35 |
| Afzonderlijke echte VM-runmappen | 31 |
| M6c-invocations / gewone M1-regressie | 30 / 1 |
| Normale raw-exit 33 / externe raw-exit -9 | 29 / 2 |
| Koude herstelstarts met exacte voorafgaande bytes | 14 |
| Native onzekerheidsscenario's | 8 |
| Unieke synthetische 64-KiB-volumes | 16 |
| Verplichte artifacthashes gecontroleerd | 274 |
| Alle aanwezige runbestanden opnieuw gehasht | 368 |
| Hashes in de tekstarchiefbundel | 12 |
| Bronbestanden bytegelijk aan de opgegeven commit | 144 |
| Hosttestbestanden bytegelijk aan die commit | 4 |
| Geldige v2-records over unieke volumes | 78 |
| Geëxporteerde niet-lege sectoren, inclusief corrupt/legacy | 80 |

De hostlogtrio's zijn gekoppeld aan hun exacte commando, werkmap, afzonderlijke bestanden en gatetijdstiphashes. De werkelijke logs tonen 30 Rust-tests en 48 Python-tests met een succesvolle exit. De finale strikte preflight is geslaagd. De eerdere preflight met ontbrekend firmwarezoekpad heeft een lege `results`-lijst, bleef apart bewaard en telt niet als native run of geslaagde gate.

De audit controleert zelf de journalbytes, recordversie, nulpadding, checksum, volgorde, voorgangerrevisie, rename-inhoudsbehoud, namen, objectgrenzen, synthetische canaryseeds en bytes buiten het journal. Geen projectparser, runner, capturehelper of bronhashhelper wordt geïmporteerd of uitgevoerd.

## Exacte onzekerheidsuitkomsten

Alle acht scenario's behouden in de live service **sequence 1, ACK 1, 11 Unknown-antwoorden, nul succesvolle catalogusantwoorden en nul aanvullende service-blockcalls, publicaties of succesvolle completions na onzekerheid**. Het bevestigde PUT-record heeft request-ID 10. Waar de RENAME fysiek aanwezig is, heeft die exact request-ID 11.

| Foutscenario | Werkelijk gelezen journal-sequence | Request-ID's | Quarantaine |
|---|---:|---|---|
| `write-error` | 2 | 10, 11 | Nee |
| `flush-error` | 2 | 10, 11 | Nee |
| `write-timeout` | 1 | 10 | Nee |
| `flush-timeout` | 2 | 10, 11 | Nee |
| `reset-timeout` | 1 | 10 | Ja |
| `bad-descriptor` | 1 | 10 | Nee |
| `used-write` | 2 | 10, 11 | Nee |
| `used-flush` | 2 | 10, 11 | Nee |

Deze fysieke uitkomsten zijn direct uit de eigen rawimages gelezen. Ze zijn niet afgeleid uit een succesvlag van het harnas. De vijf gevallen met journal-sequence 2 laten zien waarom een reeds geschreven RENAME toch geen succesvolle live ACK mag krijgen. Iedere bijbehorende nieuwe VM leest vervolgens exact het bestaande journal, zonder een nieuw record te schrijven.

## Uitgevoerde commando's en overdrachtsbestanden

```text
python3 -B /tmp/r09_audit_m6d.py --self-test
python3 -B /tmp/r09_audit_m6d.py --repo /workspace/scratch/2ffeac9e0e8f/nieuw-os --out /tmp/r09-m6d-audit
python3 -B /tmp/r09_audit_m6d.py --reconstruct /tmp/r09-m6d-audit/m6d-actual-journal-records.json
```

Alle drie commando's zijn uitgevoerd en eindigden met exitcode 0. De script-eigen 16 checks gebruiken expliciet kunstmatige hostinputs en tellen niet als native VM-bewijs. Ze controleren onder andere afwijzing van een opnieuw gechecksumd fout request-ID, changed-content rename, verkeerde padding, replaypublicatie, dubbele Unknown, extra ACK en stale live-state-output.

Het zelfstandige reconstructiecommando draaide vanuit `/tmp` zonder `--repo`. Het heeft alle 16 begin- en eindimagehashes opnieuw berekend uit uitsluitend seed plus niet-lege sectorhex, inclusief de ongeldige legacy/corrupt-fixtures. Daarvoor zijn geen grote images, firmware of kernelbinaries in Git nodig. Dit commando is later vanuit een tekstarchief opnieuw uitvoerbaar.

Over te nemen tekstbestanden:

- `/tmp/r09_audit_m6d.py`: zelfstandig controlescript; SHA-256 `5c88448e100acadd49c515d71e7d1516752b86ca657bbcf14177f11739cd0c85`.
- `/tmp/r09-m6d-audit/m6d-final-native-audit.json`: volledig auditrapport met bronbinding, individuele runs, cold-paren en grenzen.
- `/tmp/r09-m6d-audit/m6d-actual-journal-records.json`: seeds, sectorhex, begin-/eindhashes en onafhankelijk gedecodeerde toestand.
- `/tmp/r09-m6d-audit/m6d-all-artifact-hashes.json`: alle 368 werkelijk gelezen runbestanden.
- `/tmp/r09-m6d-audit/m6d-audited-source-hashes.json`: per-bestandbinding aan de broncommit.
- `/tmp/r09-m6d-audit/m6d-image-reconstruction.json`: eerste zelfstandige reconstructiecontrole binnen de audit.
- `/tmp/r09-m6d-audit/m6d-audit-self-test-run.json`: echte commandoreceipt, stdout/stderr en exit van de 16 hostchecks.
- `/tmp/r09-m6d-audit/m6d-text-only-reconstruction-run.json`: echte commandoreceipt, stdout/stderr en exit van de latere reconstructie zonder repo-argument.

## Grenzen van de conclusie

De status- en used-length-foutinjecties veranderen een lokale completionsnapshot nadat een echte used-ring-completion is waargenomen. Zij bewijzen de native foutafhandeling; zij zijn geen geclaimde fysieke devicefout. De timeoutscenario's verbergen waarneming tot een echte deadline en mogen daarom een al gepubliceerde write niet als afwezig veronderstellen. Het descriptorgeval wordt vóór publicatie afgewezen en vereist wél uitsluitend het eerdere PUT-record.

Quarantaine en geen hergebruik worden in de geobserveerde native run gecontroleerd. Dit bewijst geen gedrag op fysieke hardware. De cold-recoverycases zijn afzonderlijke OS/VM-starts, geen automatisch serviceherstartmechanisme in dezelfde VM. De externe process-kills zijn geen fysieke stroomuitvalproeven. FNV blijft een diagnostische checksum.

De audit leest bestaande native bewijsbestanden en bindt ze aan de bron; zij is geen tweede uitvoering van de 31 VM's. De geen-replay/geen-extra-I/O-conclusie steunt mede op de brongebonden native assertions en ruwe gastmarkers, niet op onafhankelijke hardwaretracing of een formeel bewijs. De QEMU-launcher, firmware en gastartefacten zijn gehasht; de volledige hostomgeving en alle gedeelde bibliotheken zijn niet geattesteerd.

De afzonderlijke M6b-regressie wordt door A04 beoordeeld en valt buiten dit rapport. De voortgangsscore en acceptatie van roadmapregels blijven de verantwoordelijkheid van de hoofdtaak; deze audit onderbouwt uitsluitend de expliciet begrensde M6d-functionaliteit.
