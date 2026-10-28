# G10 — actuele native SDK-opslag en koude recovery

## Resultaat en bron

**Geslaagd: twee echte QEMU-invocations, normal → koude recovery, beide met raw
exit 33 en zonder timeout.** De bron bleef bij alle vijf controles vóór, tussen
en na de runs exact:

- Commit: `dbd35b04d742b18d66b40fdd65d8a9b900656cac`.
- Git-tree: `421bb96102779d01d9cb4e6231d686aea417d63a`.
- Source-v3: `c5f3d9316433ba09ba4b886985ffa8b28643d67c72a9ebb0e2b0136362dc7a9f`.

De gerichte regressie voert de bestaande normale M6c-rollen en hun bestaande
recovery-case via de SDK-runtimebridge uit. Zij hergebruikt de projectbuild,
runner en de volledige bestaande **per-run** controle uit
`capture_m6c.check_run`. De eerdere volledige acceptatie van 35 groepen en
31 VM's blijft onder haar eigen bron `3e41bbf...` in `parallel-final-sdk-m6c`
bewaard; deze bundel claimt uitsluitend de twee nieuwe runs.

## Aangetoond native gedrag

De normale run voert PUT/GET, revisiecontrole, RENAME, DELETE en LIST uit in twee
domeinen. Tien bevestigde mutaties corresponderen met tien onafhankelijk uit
de echte schijfbytes geparseerde journalrecords. Zes ongeldige aanvragen
veroorzaken geen opslag-I/O. Per domein blijft één document van 128 bytes op
revisie 3 aanwezig en één tombstone op revisie 2. Namen en inhoud worden aan de
bestaande expliciete scenariofixtures gecontroleerd.

De eerste QEMU is volledig beëindigd voordat een nieuw QEMU-proces dezelfde
64-KiB-opslagimage krijgt. De recovery reconstrueert dezelfde namen, inhoud en
revisies; zij rapporteert nul mutation-ACKs en verandert geen imagebyte. De
controle bindt hetzelfde Volume-object, pad, volume-ID en bestand-inode, twee
verschillende QEMU-processen en runmappen, de expliciete vorige-runverwijzing
en exacte gelijkheid van eerdere after-image, recovery before-image,
recovery after-image en actuele volume-inhoud.

| Artifact | SHA-256 |
| --- | --- |
| SDK-user-ELF in beide werkelijke kernels | `2ef58d24b7ea8da9a857f58d580b78d65d698779fb2384c7eba524a08963a52d` |
| Normale kernel | `a7373fa01c3557595f15512f4919c4ff39ad8352673999ec9373786acdc3ea36` |
| Recovery-kernel | `98da09f727ff174d2a293b3084587bdf08c8f422b3958a9176cfbe60a4bdd394` |
| Initiële image | `ba61fd5a7f8b24580b2506d6f86d48fcf8b7d4ba8a9572d43b1409c888155d0f` |
| Image na normale mutaties én vóór/na recovery | `296a7f9bf4054341e0e0f683ee003f0cfd7a19471ace1ad86ca7bb4d03306f51` |

De SDK-selectie is vastgesteld via beide buildmanifests, de werkelijk
gecompileerde module-/linkerhashes en aanwezigheid van de volledige echte
user-ELF in beide kernelbestanden. Gewone canonieke IPC, yield, ABI-informatie
en geldige capability-close lopen door de SDK. Blok-I/O en opzettelijk ongeldige
ABI-aanvragen behouden het oorspronkelijke acceptance-fixturepad, zodat hun
kernelgrenzen daadwerkelijk worden bereikt.

## Controles en bewijsbestanden

`directed-result.json` bewaart de volledige runreports, vijf freezecontroles,
dertien herstelcontroles, SDK-binding en hashes van alle echte runartifacts.
`normal-result.json` en `recover-result.json`, de bijbehorende serial/stderr,
build- en volumemanifests zijn ongewijzigde kopieën. `build-commands.json`
bevat de daadwerkelijk uitgevoerde compilercommando's en exits uit de per-fase
logkopieën; `launcher-command.json` bindt de orchestration en ruwe loghashes.
Alle oorspronkelijke binaries, firmware en wegwerpimages blijven in de eigen
buildmap buiten Git. `archive-index.json` bindt deze tekstbundel en legt de
laatste hercontrole van echte artifacts vast.

De twee runs gebruikten Rust 1.90.0, QEMU 7.2.22 en het expliciete profiel
`pc-q35-7.2/qemu64/TCG/1CPU/256MiB`, zonder gastnetwerk. Iedere VM had eigen
firmware-/runbestanden. De totale launcherdoorlooptijd was circa 19,07 seconden;
dit is geen opslagprestatieclaim. De bestaande waarschuwing over ongebruikte
`Catalog::EMPTY` trad op bij de fixturecompilatie; de compilercommando's eindigden
met exit 0.

Afzonderlijk zijn **39 SDK-hosttests** uitgevoerd — 13 runtime, 14 catalogusclient
en 12 documentbewerkingssessie — plus releasecompilatie van alle SDK-libraries voor
`x86_64-unknown-none`. Beide commando's slagen. Deze uitvoering gebruikte een
immutable snapshot van `f52ef0a64b75f29396510c96ca57eaac9ea07d0d`; alle **38**
geselecteerde library-/dependencybestanden zijn bytegelijk aan de G10-freeze.
`sdk-library-source.json`, `sdk-library-checks.json` en
`sdk-library-freeze-binding.json` leggen die toepasselijkheid exact vast.
Hostmodellen en native librarycompilatie blijven afzonderlijk bewijs; zij tonen
geen native start van de typed document-session of een GUI.

## Reproduceren

Gebruik de genoemde commit in een verse checkout en een nog lege eigen G10-
buildmap. De launcher weigert een bestaand `directed-result.json` te overschrijven.
Activeer de passende Rust/QEMU/OVMF-toolchain en voer vanuit de repositoryroot
uit:

```sh
export NOS_SDK_RUNTIME=1
export NOS_QEMU_MACHINE=pc-q35-7.2
export NOS_BUILD_DIR="$PWD/build/parallel-followup-sdk-m6c"
export NOS_TARGET_DIR=/tmp/nieuw-os-followup-sdk-m6c-target
export CARGO_TARGET_DIR="$NOS_TARGET_DIR"
export CARGO_BUILD_JOBS=2
python3 -u -B docs/verification/parallel-followup-sdk-m6c/run_directed.py \
  --root "$PWD" \
  --expected-head dbd35b04d742b18d66b40fdd65d8a9b900656cac \
  --expected-source c5f3d9316433ba09ba4b886985ffa8b28643d67c72a9ebb0e2b0136362dc7a9f
```

De launcher in deze bundel is bytegelijk aan het uitgevoerde bestand in de
buildmap. Omdat de bewijsbundel na de uitvoeringscommit wordt toegevoegd, kan
men dit losse launcherbestand ook naast een checkout van de genoemde commit
gebruiken. De source-check betreft de projectinputs, de launcher wordt met een
afzonderlijke hash gebonden. Gebruik de genoemde targetmap niet gelijktijdig
voor een andere build. Deze omgeving gebruikte bovendien de offline cache
`CARGO_HOME=/tmp/nieuw-os-par-tools/cargo-home` en het firmwarepaar uit
`/tmp/nieuw-os-par-tools/env.sh`; de concrete instellingen staan in het rapport.

## Beperkingen

Deze run bewijst een native service-/opslagpad met echte VM-uitvoering en echte
imagebytes, aangestuurd door bestaande synthetische acceptatieprogramma's.
De begrenzing blijft vier objecten, twee domeinen, maximaal 128 inhoudsbytes en
een journal van 32 records. De aanvullende foutinjecties, crashcuts en algemene
M1-regressie zijn in G10 niet opnieuw uitgevoerd. Er wordt geen grafische
Materiaal-editor, algemene appstarter, grotere documenten, fysieke
power-loss-durability of menselijke audit geclaimd. Roadmapbeoordeling en
integratie-/PR-status blijven bij de hoofdagent.
