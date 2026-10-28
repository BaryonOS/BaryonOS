# I12 — gezamenlijke artifactcontrole van zes actuele native VM-runs

## Uitkomst

**Geslaagd, geen concrete blocker gevonden.** De afzonderlijke controle leest de
zes bewaarde VM-resultaten en hun oorspronkelijke artifacts, vergelijkt deze
met de archieven en berekent de hashes opnieuw. Zij start geen VM, herhaalt geen
acceptatiesuite en wijzigt geen bestaande validator of testartifact.

Uitvoering: 28 oktober 2026, 16:28:49–16:28:50 UTC.

- Broncommit: `dbd35b04d742b18d66b40fdd65d8a9b900656cac`.
- Tree: `421bb96102779d01d9cb4e6231d686aea417d63a`.
- Source fingerprint v3:
  `c5f3d9316433ba09ba4b886985ffa8b28643d67c72a9ebb0e2b0136362dc7a9f`.
- **6 unieke oorspronkelijke VM-mappen**, alle met echte gerapporteerde ruwe
  QEMU-exitcode **33**, complete seriële uitkomsten en geen timeout.
- **216 unieke bestanden opnieuw gehasht**; ieder gelezen bestand is aan het
  eind opnieuw op dezelfde bytes gecontroleerd.
- HEAD, tree en bronvingerafdruk zijn vóór en na de controle exact gelijk.
- Nieuwe VM-uitvoeringen door I12: **0**.

`audit.json` bevat alle 216 daadwerkelijk gelezen absolute paden, lengtes en
SHA-256-hashes. De controle vertrouwt niet uitsluitend op bestaande
`passed: true`-velden: zij vergelijkt de ruwe seriële uitvoer, bestanden,
schijfbeelden en pixels rechtstreeks.

## Gecontroleerde runs

| Onderdeel | Unieke oorspronkelijke VM-map | Ruwe exit | Gecontroleerd gedrag |
| --- | --- | ---: | --- |
| B08 G0 | `graphics-g0-15e24b60b7` | 33 | Vijf echte beelden, vier invoerveranderingen, backspaceherstel en volledige eindmarkers |
| I11 qsim | `native-qsim-e3745ad261` | 33 | Exact 28 regels, twee gescheiden processen, daadwerkelijke counters en framevrijgave |
| ROOT M6b schrijven | `m6b-followup-root-normal-82d158c1f7` | 33 | Journalreeks 8, acht mutatie-ACK's, zes verzoeken zonder I/O afgewezen |
| ROOT M6b koude start | `m6b-followup-root-cold-6c3bee8bf3` | 33 | Dezelfde schijf en toestand, nul mutatie-ACK's |
| G10 SDK schrijven | `m6c-g10-normal-b82538dd76` | 33 | Catalogusreeks 10, tien mutatie-ACK's, twee domeinrevisies van 5 |
| G10 SDK koude start | `m6c-g10-cold-recovery-14f12deae4` | 33 | Dezelfde schijf, documentinhoud en namen; nul mutatie-ACK's |

Alle zes commandoregels gebruiken expliciet `pc-q35-7.2`, `qemu64`, TCG,
één CPU, 256 MiB RAM en geen gastnetwerk. De controle volgt de werkelijke
firmware- en ESP-paden uit die commandoregels. Per run worden de kernel,
UEFI-loader, firmware en overige opgegeven artifacts opnieuw gehasht en
gekoppeld aan het bijbehorende featuremanifest. Oorspronkelijke result-,
serial-, stderr- en manifestbestanden worden rechtstreeks vergeleken met hun
archiefkopieën.

De G0-runner heeft geen afzonderlijk `timed_out`-veld. Daar bestaat het bewijs
uit ruwe exit 33, `error: null`, de volledige normale markerreeks en de
vastgelegde eindige uitvoertijd. De overige vijf resultaten bevatten expliciet
`timed_out: false`.

## G0: pixels en invoer

De vijf PNG's hebben RGB-modus en afmetingen 800×600. Onafhankelijke decodering
met Pillow, gevolgd door samenvoegen van de oorspronkelijke PPM-header en alle
RGB-bytes, levert voor ieder beeld **exact het oorspronkelijke PPM-bestand**
op. De originele PPM's worden zelf gelezen en gehasht.

De elf QMP-verzoeken en antwoorden hebben overeenkomende unieke ID's. De
daadwerkelijke toetsenreeks is `a`, `b`, backspace, `c`, escape. De pixelverschillen
zijn achtereenvolgens `[48, 192, 62, 212]` en driemaal `[60, 192, 74, 212]`.
Het beeld na backspace is over alle pixels gelijk aan het eerdere `a`-beeld.
De originele en gearchiveerde seriële uitvoer zijn gelijk en bevatten beide
de complete eindregels `G0:PASS` en `M1:PASS`.

## Opslag: directe koude koppeling

Voor ieder paar geldt byte voor byte:

`schrijven.after == koud.before == koud.after == actuele volume`.

De recovery verwijst naar de juiste voorgaande run en dezelfde volume-identiteit.
De oorspronkelijke 64-KiB-schijf wordt bovendien onafhankelijk gereconstrueerd
uit de vastgelegde seed, label en lege journalruimte. Alle bytes buiten de
journalruimte zijn gelijk gebleven.

| Volume | SHA-256 van bewaarde en koud teruggelezen schijf |
| --- | --- |
| M6b | `05675190d5f8608227a1f01bbbcedc89045b1c8a98e87b9d158cba3366a19129` |
| SDK-M6c | `296a7f9bf4054341e0e0f683ee003f0cfd7a19471ace1ad86ca7bb4d03306f51` |

Een begrensde decoder in `audit.py` leest voor deze twee normale fixtures de
werkelijke records. Zij controleert versie, volgorde, objectrevisies,
opcodes, lengte, padding en checksums, en voor M6c ook namen en de
onveranderde inhoud bij hernoemen. De afgeleide toestand wordt vergeleken met
de **volledige, exact geordende seriële reeks** van beide runs. FNV dient hier
uitsluitend als bestaande integriteitschecksum; het is geen authenticatieclaim.

Bij M6b zijn de reeks en ACK-tellingen 8 en 8→0; bij SDK-M6c 10 en 10→0.
De twee bewaarde SDK-user-ELFs worden opnieuw gehasht en byte-exact in de
daadwerkelijk gestarte kernels teruggevonden. Hun opgenomen SDK-bronmodules
zijn opnieuw vergeleken met het actuele bevroren bronbestand.

## Qsim: app en artifactbinding

De exacte reeks van 28 regels wordt afzonderlijk gecontroleerd, inclusief
PID 1 en 2, elk 253 checkpoints, 7.216 nuttige arenabytes en maximale
werkkloof 448. Timerinterrupts, syscalls en vrijgegeven frames zijn strikt
positieve u64-velden met het vereiste formaat.

Het werkelijke qsim-user-ELF is opnieuw gehasht en byte-exact in de gestarte
kernel aangetroffen. Het buildmanifest, scalarbuildrapport, de bewaarde
disassembly en alle 18 qsim-broninputs zijn aan elkaar gekoppeld. De strikte
I11-resultaten en de oorspronkelijke artifactindex blijven volledig van
toepassing. De verklaring van het verschil in loadermetadata staat in de
I11-bundel; I12 verandert die artifacts niet.

## Uitvoeren en grenzen

De bewaarde controle is uitgevoerd vanuit de repository met:

```sh
python3 -B docs/verification/parallel-followup-integration/audit.py
```

De controle vereist de hierboven genoemde bevroren HEAD en bronvingerafdruk,
Pillow en de bewaarde oorspronkelijke runmappen. Zij weigert afwijkende bron,
ontbrekende artifacts of veranderde bytes. Publicatie van een latere
documentatiecommit verandert niet de bronbinding van deze opgeslagen uitvoering.

Deze bundel sluit uitsluitend de gecombineerde artifactcontrole van deze zes
reeds uitgevoerde VM-runs af. De volledige eerdere M3-, M6b- en M6c-suites zijn
niet opnieuw gedraaid. Hosttests en native librarycompilaties uit de G10-bundel
worden niet als extra VM geteld. Er wordt geen fysieke hardware-, algemene
quantumdienst- of G1-documentbewerkingsacceptatie geclaimd. Roadmapwijziging door
deze controle: **0 punten**.
