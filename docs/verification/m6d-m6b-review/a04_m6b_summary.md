# A04 — onafhankelijke M6b-regressiecontrole

**Resultaat: geslaagd; audit exit 0.** Een afzonderlijke AI-agent controleerde de vastgelegde native regressiegate en de nog aanwezige oorspronkelijke runbestanden. De audit startte geen build, testgast of QEMU-proces en veranderde geen repositorybestanden.

## Bronbinding

- Vast broncheckpoint: `88e6c04be218b8b09ea4f0a226f6b9719da706a6`.
- Bronfingerprint v3: `8ca6d5d5291fde62f91fb47dca7df4d058424604449fb32b9e5899b1d4fbef93`.
- Suite-SHA-256: `96da4b3cbfd89f62ff715ad44ff67bc9644480f6d850f4badd6e9021c75e7e13`.
- Bewijsarchief: `docs/verification/m6d-m6b-regression`.
- De fingerprint is opnieuw berekend uit `git archive` van het exacte checkpoint. Alle 144 fingerprintbestanden zijn ook met de actuele checkout vergeleken. De vier betrokken Python-testbestanden zijn afzonderlijk aan hetzelfde checkpoint gebonden.

## Gecontroleerde dekking

| Onderdeel | Vastgesteld resultaat |
|---|---|
| Gategroepen | Exact 18, in de voorgeschreven volgorde |
| Unieke VM-aanroepen | Exact 14: 13 M6b en 1 gewone M1-regressie |
| Afsluitstatussen VM | 12 records met raw exit 33; 2 snijpunten met raw exit −9 en vastgelegde SIGKILL-voorwaarden |
| Hosttests | 22 Rust-tests en 30 Python-tests; drie hostcommando's hebben exit 0 en geen timeout |
| Native artefacten | Alle 121 verplichte artefacthashes opnieuw berekend |
| Synthetische schijven | 7 unieke volumes, volledig gereconstrueerd vanuit seed en journalbytes |
| Koude herstarts | 6 aparte VM-aanroepen op exact hetzelfde vorige volume; before = vorige after = nieuwe after |
| Duurzaam tekstbewijs | 58 niet-lege journalsectoren, inclusief de corrupte fixture, als volledige sectorhex |
| Bewijsinventaris | 164 oorspronkelijke run-/logbestanden met SHA-256 |

De zeven oorspronkelijke M6b-schijven hebben de verwachte toestand: twee normale runs en de foutvalidatierun eindigen op sequence 8; de volle journal op 32; de corrupte fixture blijft ongewijzigd en wordt door de onafhankelijke parser afgewezen; de VM-cut vóór publicatie heeft sequence 0; de VM-cut na flush heeft sequence 1. De zes koude herstarts behouden zowel schijfbytes als gereconstrueerde toestand. Normale runs op twee verschillende verse schijven gebruiken dezelfde kernel- en loaderbytes.

Voor iedere VM zijn de oorspronkelijke serial- en stderrbytes met het runrecord vergeleken, inclusief unieke vereiste markers en hun volgorde. De afsluitende M1-run bevat geen M6b-markers en is gebonden aan zijn eigen gewone M1-artefacten. De feature in ieder M6b-buildmanifest is exact de bedoelde `milestone-m6b` of `test-m6b-*`-variant. Alle vaste QEMU-argumenten, het QEMU-executablehash, volume-identiteiten, canaries buiten het journal en de before-/afterimages zijn gecontroleerd.

De journalcontrole gebruikt uitsluitend Python-standaardbibliotheekcode en importeert geen parser, oracle of capturemodule uit het project. Zij controleert de eigen headers, grenzen, padding, FNV-checksum, aaneengesloten sequence, objectrevisies, DELETE-voorganger en de verwachte fixturepayloads rechtstreeks in de schijfbytes. De JSON-claims van de gate zijn daarmee onafhankelijk getoetst aan de aanwezige bestanden.

## Reproductie

Uitgevoerd commando:

```sh
python3 -B /tmp/a04_m6b_audit.py \
  --root /workspace/scratch/2ffeac9e0e8f/nieuw-os \
  --archive docs/verification/m6d-m6b-regression \
  --output-prefix /tmp/a04_m6b
```

De scriptuitvoering eindigde met exit 0 en status `passed_actual_evidence_audit` op 2026-10-28 om 09:04 UTC. Het script ondersteunt een verplaatste repository via `--root` en remapt de in het bewijs opgenomen oorspronkelijke repositorypaden. Voor een volledige heruitvoering zijn het exacte git-checkpoint en de oorspronkelijke `build/run`- en logbestanden nodig. Het sectorbestand maakt de zeven schijfhashes afzonderlijk reconstrueerbaar vanuit de vastgelegde seeds en journalbytes.

## Reikwijdte

Dit is een controle van aanwezig uitvoeringsbewijs. De verifier heeft de eerdere processen niet zelf waargenomen en kan geen host uitsluiten die alle bewijsmateriaal vervalst. De M6b-gate legt zelf geen hostloghashes vast op het moment van de test; deze audit vergelijkt de actuele oorspronkelijke hostlogs met de gearchiveerde bytes en legt hun hashes nu vast.

Het resultaat geldt voor de vaste synthetische QEMU/TCG-fixtures. Het bewijst geen fysieke stroomuitvalbestendigheid of productierijpe opslag. FNV dient als diagnostische checksum, niet als authenticatie of versleuteling. De nieuwe native M6d-injecties met `Unknown` vallen buiten deze M6b-regressieaudit en worden apart beoordeeld. Dit is een afzonderlijke AI-review, geen menselijke of formele audit.

## Overdrachtsbestanden

| Bestand | Bytes | SHA-256 |
|---|---:|---|
| `a04_m6b_audit.py` | 22802 | `3fbed4fa4d853a45e503c3ddbfdefe46624fa49508fd403145e7f7a968698897` |
| `a04_m6b_audit.json` | 13851 | `a5f981e5581c731d320fef9745aa7fb2f1d6c34759b37c1ecb077a1a51173e3c` |
| `a04_m6b_sectors.json` | 69066 | `73a19ffa223b77bcdfcf8c6123f283d933a7e117f8df00e7a8ab191cf09de3db` |
| `a04_m6b_source_hashes.json` | 14137 | `e995727cc610cd883455a4d2d7cb36d3104972cb3bc6ccf27604a3419115e54f` |
| `a04_m6b_raw_hashes.json` | 21633 | `e1aac2a56d69ef5e13eefdfff1e7ee91c2371b480ab33ddfe8772b500cff0753` |

### Addendum — latere scriptversie, 28 oktober 2026

De tabel hierboven beschrijft de oorspronkelijke A04-overdracht. Het later
bijgewerkte, ingecheckte `a04_m6b_audit.py` is **27056 bytes** en heeft SHA-256
`17fc9ca5744f9a8da5d407a621fec0500ba5c50cc7b989e77d6d82ee89145f4f`.
Die scriptversie heeft **22 geslaagde hosttests**, opgenomen in de definitieve
Python-hostsuite van 173 tests; dit zijn overlappende aantallen. De volledige
historische rawaudit is niet opnieuw uitgevoerd met deze gewijzigde versie.
Zie [Windowsafsluiting](../M6D-CLOSEOUT.md) en het actuele
[hashregister](../m6d-summary.json). Het in de tabel genoemde
[a04_m6b_sectors.json](a04_m6b_sectors.json) is ingecheckt; zijn oorspronkelijke
bytes en de historische tabelwaarden blijven behouden.

### Oorspronkelijke uitvoeringsbron beschikbaar

Het script dat het historische `a04_m6b_audit.json` produceerde staat nu ook als
[a04_m6b_audit.original.py](a04_m6b_audit.original.py) in dit archief. Het is
bytegelijk aan `a04_m6b_audit.py` op commit
`7484d87ec639fd8a29be327cad47c2f97739199d`: 22802 bytes, SHA-256
`3fbed4fa4d853a45e503c3ddbfdefe46624fa49508fd403145e7f7a968698897`.
Die hash is exact `audit_script_sha256` in het oorspronkelijke JSON-record.
Ook zijn zes `method`-regels horen bij deze versie. Het bestaande resultaat
bewijst geen uitvoering van de latere strengere `validate_qemu`- en
`validate_raw_inventory`-controles. De actuele scriptversie en zijn afzonderlijke
hosttests blijven beschreven in het eerdere addendum; er is geen nieuwe
volledige rawaudit uitgevoerd. Het JSON-uitvoeringsrecord is niet gewijzigd.
