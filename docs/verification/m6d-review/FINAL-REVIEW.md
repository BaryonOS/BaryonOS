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

## Addendum STATE-02 — gerichte bronreview van de definitieve native delta

Datum: 28 oktober 2026. Reviewer: afzonderlijke AI-agent `state_architecture`.
Dit addendum is een nieuwe, alleen-lezen bronreview; bovenstaande R09-audit en
de historische R08-receipt zijn niet opnieuw uitgevoerd of gewijzigd.
Reviewcheckout: `C:/Users/simon/OneDrive/Documenten/GitHub/nieuw-os-review`.
Reviewbasis van de checkout: `7484d87ec639fd8a29be327cad47c2f97739199d`.

**Conclusie:** geen nieuwe concrete blokkerende fout gevonden in de hieronder
afgebakende native delta. Dit geldt voor de vaste M6c/M6d-fixture en is geen
menselijke audit, algemene fouttolerantieclaim of nieuwe native testuitslag.

### Versies en daadwerkelijk gelezen scope

Gelezen delta: `b58ae2a56be6978eab2b5ac4bb5d12e3d615865f` naar
`b739183c48a06cb7798d0be8cc683430152533ac`, uitsluitend
`kernel/src/m2/m3/m6c.rs` en `userspace/m6c_probe.rs`. De omliggende
broker-/reply-/validatiepaden en de ongewijzigde native catalogusadapter zijn
als directe context gelezen. De twee native bestanden zijn bytegelijk op
`b739183`, uitvoeringscommit `88e6c04be218b8b09ea4f0a226f6b9719da706a6` en de
reviewcheckout; de integratiediff voor deze twee paden is leeg.

| Native bestand | SHA-256 van de beoordeelde Git-blob en checkoutbytes |
|---|---|
| `kernel/src/m2/m3/m6c.rs` | `6a4eed0fde2d2741e26274df0a01c7cf067b96e98959b58df526dcd9e319c3f9` |
| `userspace/m6c_probe.rs` | `1ba38587737ce9768ec4e54e174cafd5cb2b5df425685030bbcdb9c3da0337b2` |

De R08-receipt `source-review-checks.json` registreert voor die paden
respectievelijk `24685e4d5064d6ab6974cfb93ff6ea3406270ffcd2451766b6e3a4c154b1b03e`
en `9bd89d5157b6ad6aa761f3e08154bb33006e24b17b04f35419e56a37faf49989`.
Dat zijn andere bytes. De receipt bewijst dus de daarin vastgelegde lokale
reviewstand boven basis `b58ae2a`, geen exactheid van de latere checkpoint.
Dit addendum levert de ontbrekende expliciete bronreviewdekking voor de
native bestanden op `b739183`/`88e6c04`; het vult geen ontbrekende historische
uitvoering in.

### Onderzochte tegenvoorbeelden en resultaat

- **Onterechte Unknown of succesvolle ACK na een fout:** `syscall` koppelt de
  eerste onzekerheid aan de daadwerkelijk teruggegeven fout tijdens een
  voorbereide mutatie; een pending-poll (`-8`) wordt uitgesloten. `sent` eist
  daarna de canonieke lege Unknown-reply en passeert de commit-/ACK-tak niet.
  De gewone succesroute vereist nog beide daadwerkelijke completions.
- **Rechten of metadata lekken via de gesloten dienst:** requestvalidatie en
  endpointdomeincontrole gaan vóór de onzekerheidstak. `received` maakt voor
  een geldige latere aanvraag geen mutatiekandidaat; `sent` vergelijkt de
  volledige replybytes en de vastgehouden I/O-tellingen. De client controleert
  sender, tag, lengte, requestidentiteit, epoch en canonieke foutpayload.
- **Een timeout behandelen als bewijs dat niets is geschreven:** onderdrukte
  notify wordt gecombineerd met uitgestelde completionwaarneming tot de
  brokerdeadline. Dit levert onzekerheid, geen rollback. De levende toestand
  en de bestaande onafhankelijke schijfcontrole houden verschillende rollen.
- **Hergebruik zonder resetbevestiging:** `advance` herinitialiseert alleen na
  waargenomen status-reset. Timeout of mislukte herinitialisatie zet ook de
  policy in quarantaine; de afsluiting slaat een nieuwe reset daar over.
  Dezelfde vijf DMA-pagina's blijven kernelbezit. De delta voegt geen nieuwe
  userpointer, DMA-adres of capabilityrecht toe.
- **Een groen resultaat vóór ontvangst door beide apps:** de eerste werkelijk
  ontvangen Unknown opent uitsluitend de fixturebarrière voor app B. De twee
  apps proberen alle vijf geldige operaties, domeinvreemde aanvragen en een
  misvormd verzoek. De eindobserver vereist elf ontvangen Unknown-antwoorden,
  één eerdere ACK, de verwachte afwijzingen, beide voortgangstellers en geen
  extra blockcall/publicatie/succescompletion na onzekerheid.

Deze conclusies volgen uit bronlezing en vergelijking. De native gate en
ruwe imagecontrole blijven het eerder aan `88e6c04` gebonden R09-bewijs;
deze reviewer heeft daarvoor geen nieuwe QEMU-run of build uitgevoerd.
Dezelfde-VM-dienstherstart, corrupte succesvolle ABI-antwoorden, een service
die na Unknown alsnog nieuwe syscalls doet, willekeurige servicecode en
fysieke DMA-/stroomuitvalgedrag zijn hiermee niet als hersteld of getest
aangemerkt. De observer bevat fixtureasserties; zijn checks zijn geen algemeen
productieherstelmechanisme.

### Uitgevoerde controleopdrachten

Vanuit de hierboven genoemde reviewcheckout:

```text
git diff b58ae2a56be6978eab2b5ac4bb5d12e3d615865f..b739183c48a06cb7798d0be8cc683430152533ac -- kernel/src/m2/m3/m6c.rs userspace/m6c_probe.rs
git diff b739183c48a06cb7798d0be8cc683430152533ac..88e6c04be218b8b09ea4f0a226f6b9719da706a6 -- kernel/src/m2/m3/m6c.rs userspace/m6c_probe.rs
```

Beide commando's eindigden met exit 0; de tweede diff was leeg. Met
`python -X utf8 -c` zijn daarnaast de bytes uit `git show REF:PATH` voor
`b58ae2a`, `b739183`, `88e6c04` en `HEAD` via `hashlib.sha256` vergeleken met
de checkoutbytes en de twee R08-receipthashes; exit 0. De tabel hierboven
registreert de exacte beoordeelde inhoud. Er zijn geen hosttests, native
tests, volledige historische bewijsaudit of prestatiemetingen toegevoegd.

### Reproduceerbare hercontrole van de bronbinding

De bovenstaande verkorte `python -X utf8 -c`-vermelding is geen volledige
commandoreceipt. Daarom is de vergelijking op 28 oktober 2026 opnieuw
uitgevoerd met het bewaarde programma
`docs/verification/m6d-closeout/check_native_source.py`:

```powershell
python -X utf8 -B docs/verification/m6d-closeout/check_native_source.py
```

Exit 0. De volledige Git-argv's, werkmap en twaalf hashvergelijkingen staan in
`docs/verification/m6d-closeout/native-source-binding.json`. Het programma
vergelijkt beide bestanden op b739183, 88e6c04, 7484d87, 326d2f6 en HEAD met
de checkoutbytes. Dit is nieuwe controle van de bronbinding; de eerdere
R08-receipt en haar afwijkende hashes worden hiermee niet herschreven.
