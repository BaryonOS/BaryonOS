# G05 — volledige native M6c-acceptatie via de app-SDK

## Resultaat

**Geslaagd: 35 acceptatiegroepen, 31 echte QEMU-invocations en 14 koude
herstartparen.** Alle 30 M6c-VM's zijn gebouwd met de SDK-runtimebridge en
bevatten aantoonbaar dezelfde SDK-user-ELF. De resterende VM is de gewone
M1-bootregressie. De volledige bestaande capturecontrole heeft raw output,
exitstatussen, artifacthashes, echte schijfbytes, exacte dekking en alle
herstelkoppelingen opnieuw gecontroleerd. Geen acceptatie-eis is versoepeld.

Uitvoeringsbron: `3e41bbf93822f7044c6020259f4a752925e1966d`, branch
`codex/parallel-native-foundation-20261028`. Het v3-bronfingerprint is
`307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`.
De bron en HEAD bleven tijdens de volledige suite en capture ongewijzigd.
Dit archief is bewijs op die ontwikkelbron; opname in `main` en PR-status
worden afzonderlijk door de integrator vastgelegd.

## Wat nu native werkt

De bestaande catalogusrollen gebruiken de nieuwe allocatorvrije app-SDK voor
gewone canonieke IPC-send/receive, yield, ABI-informatie en geldige cap-close.
Daarmee werkt de echte native keten van catalogusaanvraag naar dienst en
virtio-opslag, teruggelezen antwoord en controle na een volledig nieuwe VM.
De fixture voert PUT/GET, CAS-revisiecontrole, RENAME, DELETE en LIST uit in
twee domeinen. In beide normale runs komen tien mutation-ACKs overeen met
tien onafhankelijk geparseerde journalrecords. Zes ongeldige aanvragen
veroorzaken geen opslag-I/O. De eindtoestand bevat per domein één aanwezig
128-byte-document op revisie 3 en één tombstone op revisie 2; namen en inhoud
zijn bytegewijs aan het bestaande scenario gebonden.

De SDK-bridge behoudt de bestaande rapporttellers en crashgate. Blok-I/O en
doelbewust ongeldige ABI-aanroepen blijven via het oorspronkelijke raw
fixturepad lopen, zodat de kernelgrens nog werkelijk wordt getest. De
catalogusproef is geen grafische Materiaal-app. De afzonderlijke B01-gebaseerde
SDK-voorbeeldapp is eerder native gecompileerd en op de host gecontroleerd;
deze suite start die voorbeeldapp niet.

## Uitgevoerde controles

| Onderdeel | Bewijs |
|---|---|
| Hostvoorwaarden | Strikte ontwikkelinventaris; 45 catalog-core/client/wire-tests en 48 externe-orakel/capture/harnastests, alle exit 0. |
| Normale opslag | Twee echte normale VM's op verschillende canaryimages met dezelfde kernel/loader en dezelfde expliciete documenttoestand; elk gevolgd door een aparte koude recovery-VM. |
| Foutgrenzen | `errors`, `full`, `corrupt`, `legacy`; errors en full elk gevolgd door koude recovery. |
| Onderbreking | De host beëindigt twee werkelijk levende QEMU-kindprocessen vóór en na rename op het vastgelegde cutpunt. Beide raw exits zijn `-9`, met iedere keer een nieuwe recovery-VM op exact de eerdere image. |
| WRITE/FLUSH en driverfouten | `write-error`, `flush-error`, `write-timeout`, `flush-timeout`, `reset-timeout`, `bad-descriptor`, `used-write`, `used-flush`; alle acht met `Unknown`, geen replay en afzonderlijke koude recovery volgens de bestaande acceptatiegrens. Reset-timeout controleert ook quarantaine. |
| Herstel | Veertien herstelparen controleren dezelfde volume-identiteit, een afzonderlijke VM, een expliciete verwijzing naar de vorige run, exacte eerdere schijfbytes, geen recovery-writes en dezelfde bevroren bron. |
| Normale boot | Eén gewone M1-VM na de SDK-suite, zonder M6c of SDK-claim. |
| Raw uitkomsten | 29 VM's eindigen met raw QEMU-exit 33; de twee echte crashcuts met -9. Geen timeout wordt als succes aangemerkt. |
| Capture | Originele `capture_m6c.py` slaagt: 35 groepen, 31 verschillende echte VM-invocations, alle verplichte artifactsets en disk-orakels hergecontroleerd. |
| SDK-selectie | Extra controle leest alle 30 echte M6c-kernels: `sdk_runtime=true`, dezelfde bron en de werkelijke SDK-user-ELF aanwezig in de kernelbytes. Alle vastgelegde SDK-/bridge-/dienstmodulehashes komen overeen met de bron. |

De volledige suite duurde circa 258,6 seconden; de capture circa 0,52 seconde.
Dat zijn doorlooptijden van deze gecontroleerde ontwikkelrun, geen
opslaglatentie- of hardwareprestatieclaim. De uitvoering gebruikte Rust 1.90.0,
QEMU 7.2.22, het expliciete `pc-q35-7.2/qemu64/TCG/1CPU/256MiB`-profiel en een
eigen OVMF-CODE/VARS-kopie per run. Iedere opslagproef gebruikte een begrensde
synthetische image; er is geen hostschijf aan de gast aangeboden.

## Zelf reproduceren

Gebruik de genoemde uitvoeringscommit en de projecttoolchain met passende
QEMU 7.2/OVMF-firmware. Stel `OS_OVMF_CODE` en `OS_OVMF_VARS` in op een
bij elkaar horend firmwarepaar wanneer de bestaande firmwaredetectie dat
niet vindt. Voer vanaf de repositoryroot uit:

```sh
export NOS_SDK_RUNTIME=1
export NOS_QEMU_MACHINE=pc-q35-7.2
export NOS_BUILD_DIR="$PWD/build/parallel-final-sdk-m6c"
export NOS_TARGET_DIR=/tmp/nieuw-os-par-final-sdk-m6c-target
export CARGO_TARGET_DIR="$NOS_TARGET_DIR"
export CARGO_BUILD_JOBS=2
python3 -B tools/test_m6c.py
python3 -B tools/capture_m6c.py --destination docs/verification/parallel-final-sdk-m6c
```

Deze run gebruikte daarnaast de reeds ingerichte offline dependencycache
`CARGO_HOME=/tmp/nieuw-os-par-tools/cargo-home`; de gebruikte pad- en
firmwarekeuzes staan volledig in `driver.json`. Gebruik bij gelijktijdige
uitvoering een eigen build- en targetmap. De oorspronkelijke gate en capture
zijn met deze afzonderlijke mappen uitgevoerd. Hun exacte argv, exitstatus en
loghashes staan in `driver.json`.

`verify-sdk-binding.py` bevat de aanvullende bron-/artifactcontrole van G05.
Zij is bewust aan deze uitvoeringscommit en deze bewijsmap gebonden en leest
de echte bewaarde run-artifacts. De opgeslagen uitkomst staat in
`sdk-binding.json`. De SDK-user-ELF heeft SHA-256
`2ef58d24b7ea8da9a857f58d580b78d65d698779fb2384c7eba524a08963a52d`;
`sdk-user-manifest.json` legt haar bronmodules en segmentgrenzen vast.

## Archief en beperkingen

`suite.json` bevat de exacte geordende acceptatiegroepen. `guest-runs.json`
bewaart alle raw seriële uitvoer, exits, VM-commando's, bron- en artifacthashes,
schijforakels en herstelverwijzingen. `capture.json` bindt die bestanden en de
originele hostlogs. `sdk-binding.json`, `sdk-user-manifest.json`, `driver.json`
en deze toelichting zijn aanvullende SDK-bewijsbestanden; zij vervangen geen
bestand of controle uit de oorspronkelijke capture. De uitvoerbare binaries
en wegwerpimages blijven buiten Git in de aangegeven runmappen.

De vier-objectencatalogus, twee domeinen, 128-byte-inhoud en 32-recordjournal
blijven de bestaande begrensde opslagproef. `Unknown` betekent geen garantie
dat er niets op de schijf is veranderd: recovery reconstrueert de werkelijke
geldige records zonder de onzekere aanvraag opnieuw af te spelen. Er is geen
claim over fysieke power-loss-durability, grotere documenten, een algemene
appstarter, grafisch bewerken, G1/Materiaal of een menselijke audit.

**Roadmapdelta: 0 punten.** S03 en S07 blijven ondersteund door de volledige
regressie op deze SDK-integratie. A01 en U03 blijven open; de nieuwe native
SDK-IPC-keten is nuttige concrete voortgang maar sluit die bredere criteria
niet volledig af. De integrator beheert de ongewijzigde vaste score en de
uiteindelijke PR-/main-status.
