# Volume-admissie — native acceptatie

**De bestaande volumedienst weigert onbekende en nulgevulde volumes tijdens
service-startup; 10 toelatingsproeven en de bestaande 9-VM-crashgate slagen.**

Datum: 2026-11-02 (Europe/Amsterdam). Uitvoeringsbasis `74a34d05ef5591b782a0da9da34e4669910d0722`;
geteste broncommit `94f12da0b9740a1a0ced1f08b4a30787f82ae79f` op
`codex/volume-format-admission-20261102`.
Source fingerprint v3: `2cffba892608c907b717158a83b8a1c66c7d79dc5843d2d03eabc4ff1831c028`.
[Concept-PR #133](https://github.com/simonvanieperen/nieuw-os/pull/133).

## Besluit en wijziging

Recovery selecteert uitsluitend een volledig gedecodeerde en verzegelde eigen
NOSVOL02-tabelkopie. De bestaande layout codeert de expliciete lege toestand
als canonieke generatie 0 in slot 0. De offline fixturemaker schrijft deze
toestand vooraf naar een nieuw wegwerpbestand. Recovery leidt nooit meer
lege opslag af uit ontbrekende of gedeeltelijke metadata. Aangrenzende
generaties worden overflowveilig vergeleken.

Bestaande complete generaties blijven geldig, ook generatie 1 met een oude
nulcompanion. Een onderbroken eerste commit blijft herstelbaar vanuit de
geldige generatie-0-basis. Alle gedeeltelijke eerste-commitprefixen zijn ook
gericht op de host getest. Uniform sealed maar semantisch corrupte metadata
blijft een harde fout. Zie [ADR-058](../../DECISIONS.md#adr-058--positieve-volume-identiteit-vóór-toelating-als-lege-opslag).

De eigen kernel, IPC en capabilities zijn niet verruimd. Er is geen
runtimeformatteerpad of nieuwe bestandssysteemdriver toegevoegd. Alleen
reguliere wegwerpimages zijn gebruikt; bronimages worden uitsluitend gelezen
en iedere VM krijgt een eigen kopie. Geen fysieke disks, mounts, modules,
netwerk of systeemwijde pakketinstallatie.

## Native toelatingsmatrix — 10/10

QEMU 11.1.0, TCG single-thread, één CPU, 256 MiB; Rust 1.90.0 en de eigen
UEFI-loader/kernel. Kernel-/loaderhashes worden tegen de bouwmanifesten
gecontroleerd. De bron, afzonderlijke Python-testinputs en runner zijn vooraf
bevroren en na iedere run hercontroleerd.

| Proef | Raw QEMU-exit | READ-trace | WRITE-trace | Seconden |
| --- | ---: | ---: | ---: | ---: |
| own-normal | 33 | 48000 | 4963 | 52.42 |
| own-cold-recover | 33 | 37756 | 0 | 25.20 |
| btrfs-readonly | 35 | 328 | 0 | 2.34 |
| btrfs-writable | 35 | 328 | 0 | 2.34 |
| zero-readonly | 35 | 328 | 0 | 2.02 |
| zero-writable | 35 | 328 | 0 | 2.08 |
| zero-prefix-payload-tail-readonly | 35 | 328 | 0 | 2.06 |
| zero-prefix-payload-tail-writable | 35 | 328 | 0 | 2.03 |
| openzfs-leaf-readonly | 35 | 328 | 0 | 2.59 |
| openzfs-leaf-writable | 35 | 328 | 0 | 2.91 |

Eigen normal eindigt in generatie 84. Cold recovery start in een nieuwe VM
met de exacte normale eindbytes en behoudt generatie en beide domeindigests.
De normale controle bewijst werkende WRITE-tracering en werkelijke writes.

Alle acht negatieve proeven eindigen expliciet met `Corrupt = -18` en
`SERVICE_STARTUP`, raw exit 35, zonder timeout, READY/PASS of appfoutmarkers.
Geen opslag beschikbaar gesteld; geen opslagaanvragen afgehandeld. De
afwezigheid van appdiagnostiek bewijst **niet** dat appcode nooit is gestart.
Alle volledige bron- en proefimagehashes blijven gelijk. Nul WRITE-traces
geldt voor alle virtio-blk-apparaten samen; READ-aantallen omvatten ook het
read-only boot-ESP en zijn geen data-disk-only tellingen.

Btrfs en het OpenZFS-mirrorlid zijn exacte kopieën van de eerdere native
backendtoets. De OpenZFS-input bevat DMU-testobjectsets, geen normaal ZPL-
bestandscorpus; zelfstandig degraded import is niet getest. De twee
synthetische negatieven zijn 1 MiB: volledig nul en nulheaders met payload
op sector 80 (binnen het toegestane bereik) en 900 (erbuiten).

## Bestaande crash- en herstelgate — 9/9

De ongewijzigde `tools/test_volume.py`-gate is via de externe wrapper
uitgevoerd. Alleen de QEMU-werkmap van `run_volume` wijst naar het externe
dossier; bronfingerprint, hosttests en builds gebruiken de echte source.
Alle drie voorafgaande hostchecks eindigen met exit 0: 45 volume-coretests,
32 SDK-runtimetests en 31 Python-volumetests. Het definitieve
`volume-gate.json` heeft `status=completed`, `passed=true`, negen stages en
dezelfde bron-/testinputhashes. Wrapperexit: 0.

| Stage | Geval | Afkomst | Raw QEMU-exit | Seconden |
| ---: | --- | --- | ---: | ---: |
| 1 | normal | — | 33 | 48.22 |
| 2 | recover | normal | 33 | 23.20 |
| 3 | cut-data | — | 1 | 2.52 |
| 4 | recover-cut | cut-data | 33 | 2.87 |
| 5 | cut-table | — | 1 | 2.89 |
| 6 | recover-cut | cut-table | 33 | 2.78 |
| 7 | cut-txn | — | 1 | 3.36 |
| 8 | recover-cut | cut-txn | 33 | 5.55 |
| 9 | corrupt | — | 33 | 3.67 |

De drie cutgevallen zijn daadwerkelijke hostkills op het voorgeschreven
schrijfmoment; hun raw exit is geen succesexit van de guest. De gate
controleert de verwachte markers, bytes/canaries, oracletoestand en nieuwe
cold-recovery-VM op dezelfde image. Proceskills in QEMU met vastgelegde
cachemodus bewijzen geen fysieke stroomuitval of hardwareduurzaamheid.

## Hostbewijs en review

De nieuwe rawzero-regressie is vastgelegd als rood op de oude code en groen
na de fix. `volume-core`: 45 tests en Clippy `-D warnings`; document-session:
28 tests; Materiaal-app: 24 tests; onafhankelijk Python-orakel en harnassen:
141 tests. Formatteringschecks en lockfilebehoud slagen. Exacte commando's,
exitstatussen en volledige uitvoer staan onder `host/` en `host-oracle/`.

Eerdere fouten blijven bewaard: ontbrekende bestaande locked crate in de
offline cache (daarna lokaal opgehaald, zonder lockfilewijziging), Windows-
platformvereisten van Linux-tests (dezelfde suites slagen onder WSL), en een
externe runnerpreflight met bare-rustc-resolutiefout vóór build/VM. Alleen
die executable-resolutie is hersteld en de nieuwe runnerhash vastgelegd.

Een tweede AI-agent beoordeelde bron en runner: geen blokkerende bevinding;
downgradecompatibiliteit en de grens van appmarkerclaims zijn verwerkt.
Een andere AI-agent controleerde de echte bestanden, serial, traces, hashes,
manifesten en cold-paarbinding; zie `independent-native-review/`. Dit is
AI-teamcontrole, geen onafhankelijke menselijke audit.

## Bewijs lezen en reproduceren

- `TASK.md`: taakcontract en als zodanig gelabeld hostcheckpoint.
- `native/freeze.json`, `environment.json`, `builds/`: bron/tool/bouwbinding.
- `native/matrix.json`, `native/runs/*/result.json`: exacte commando's en exits.
- `native/runs/*/serial.txt`: ongewijzigde guestuitvoer.
- `native/runs/*/stderr-trace.txt.gz`: volledige raw QEMU-trace, verliesloos
  gzip; decomprimeer met standaard Python `gzip.decompress`.
- `native/regression/invocation.json` en `build/volume-gate.json`: volledige
  bestaande gate, inclusief hostuitvoer en verwijzingen naar runrapporten.
- `native-import.json`: koppeling van iedere archiefkopie aan oorspronkelijke
  bytes/SHA256. `ARTIFACTS.json`: hashes van de hele compacte bewijsbundel;
  gziplogs bevatten ook de ongecomprimeerde hash.
  De bijgewerkte reviewverifiers verschillen van de historische import;
  de oude receipts en importhashes blijven bewijs van de eerdere controle.
  De volledige rootcontrole blijft open tot de import opnieuw bytegelijk is
  en de externe bron en oorspronkelijke bestanden beschikbaar zijn.

Images, binaries, Cargo-targets en caches blijven buiten Git. De gearchiveerde
runner is een snapshot van het externe dossier, met bronpaden relatief aan
dat dossier; voer hem niet blind uit vanuit deze archiefmap. Zijn README en
invocationreceipts leggen de werkelijk uitgevoerde commando's vast. Lokaal
dossier: `C:/Users/simon/Documents/Codex/2026-11-02/volume-format-admission/`.

## Compatibiliteit, herstel en resterende grens

Oudere kernel/orakelcode weigert nieuwe sealed-gen0-lege images vóór de eerste
complete commit. Code en fixturetooling horen bij rollback bij elkaar. Revert
de patch alleen als samenhangende versie; herstel nooit impliciete nulvolume-
toelating en herformatteer geen bestaande data om compatibiliteit af te dwingen.
Een echte formatter/installer of vrije-mediatoegang vraagt een apart contract.

FNV/seals leveren formaatvalidatie en foutdetectie, geen cryptografische
authenticatie tegen bewust vervalste eigen metadata. Native Btrfs/OpenZFS-
ondersteuning, vreemde bestandslezing/herstel, fysieke powercuts en productie-
geschiktheid zijn hiermee niet aangetoond. De eerdere backendpoort blijft
open. Deze gerichte bugfix verandert geen rubriccriterium: **34/100 → 34/100** bij bron `94f12da`; de huidige projectscore is **41/100**.
