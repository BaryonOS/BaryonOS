# I13 — herbruikbare native DocumentSession-acceptatiepoort

## Opgeleverd

`tools/test_document_session.py` voert uitsluitend de twee bestaande native
M6c-cases `normal` en `recover` uit, met de G11-DocumentSession-probe expliciet
geselecteerd. De poort hergebruikt `build.build`, `run_m6c.run_m6c` en de
ongewijzigde volledige per-runcontrole `capture_m6c.check_run`.

De implementatie is gebouwd op broncommit
`dbd35b04d742b18d66b40fdd65d8a9b900656cac`. Centrale integratie met G11 en de
nieuwe selectie in `build.py` is een benoemde afhankelijkheid. **In I13 zijn
geen VMs gestart.** De resultaten hieronder zijn hostcontroles van het harnas
en een afzonderlijke controle van echte G11-crosscompile-artifacts.

Schrijfscope: de nieuwe poort, `tests/test_document_session_harness.py` en
deze bewijsmap. Er zijn geen kernel-, observer-, syscall-, opslagformaat- of
gedeelde buildwijzigingen door deze taak gemaakt.

## Bindende controles

- Zowel `NOS_DOCUMENT_SESSION=1` als `NOS_SDK_RUNTIME=1` is vooraf expliciet
  vereist. De poort schakelt deze opties niet zelf in.
- De gebruiker kiest een lege private buildsubmap en expliciete overeenkomende
  native/Cargo-targetpaden. Eerdere resultaten worden niet overschreven.
- HEAD en source fingerprint v3 worden vóór en na iedere build en VM en na de
  volledige eindcontrole vergeleken. Optioneel opgegeven freezehashes moeten
  al vóór het maken van output overeenkomen.
- Zowel buildmanifest als usermanifest moeten `document_session: true` en
  `sdk_runtime: true` bevatten. Alleen `milestone-m6c` en
  `test-m6c-recover`, zonder extra loaderfeature, zijn toegestaan.
- De vaste bronselectie bindt de bestaande probe en runtime, de nieuwe
  DocumentSession-probe, de vier cratebronbomen en hun manifesten, en de
  SDK-workspace en lockfile. Ontbrekende, extra of gewijzigde bronhashes
  worden afgewezen.
- De vier daadwerkelijke rlibs, het app-ELF en zijn geïnspecteerde geheugenlayout
  worden opnieuw gehasht. Het exacte app-ELF moet in de daadwerkelijk gestarte
  kernel zitten. Kernel en loader moeten bij het fasegebonden manifest horen.
- De vijf echte rustc-logrecords binden Rust 1.90, target, crate, bronbestand,
  outputpad en bestaande native ABI-opties. De probecompile moet beide cfgs
  en exact de vier geselecteerde externe libraries bevatten.
- Vóór de recovery-build worden usermanifest, app-ELF, vier rlibs en relevante
  compilerlogs van de eerste fase gekopieerd naar een eigen snapshot. Na de
  twee VMs worden beide snapshots en beide volledige oorspronkelijke
  VM-resultaten opnieuw gecontroleerd. App- en libraryhashes moeten tussen
  de twee gevallen gelijk blijven.
- Beide echte runs moeten raw exit 33 en geen timeout hebben. De bestaande
  externe schijforacle en markercontrole blijven verplicht.
- De koude run moet een andere VM-invocation zijn, verwijzen naar precies de
  normale voorganger en dezelfde volume-identiteit gebruiken. De normale
  eindschijf, het koude begin- en eindbeeld en het actuele volume moeten
  byte-exact gelijk zijn. Sequence 10 en mutatie-ACK's 10→0 zijn verplicht.

De bestaande ongewijzigde kernelobserver controleert vóór `M6C:PASS` exact
16 verzoeken per client in de normale case en 3 in de recoverycase. De poort
bindt dit aan de geselecteerde kernelcase, de succesvolle volledige
observeruitkomst en de observerbronhash. Deze aantallen worden uitdrukkelijk
gelabeld als door de observer afgedwongen aantallen, niet als nieuw toegevoegde
seriële counters.

## Uitgevoerde controles

### Twintig gerichte hosttests

```sh
python3 -B -m unittest discover -s tests -p test_document_session_harness.py -v
```

Exitstatus 0; **20 tests geslaagd**. De originele stdout/stderr, commandoregel,
basiscommit en bronhashes staan in `host-checks.json`.

De negatieve gevallen omvatten ontbrekende en verkeerd getypeerde opt-ins,
ontbrekende en verwisselde librarybronnen, ontbrekende en verwisselde rlibs,
een vervangen app-ELF met bijgewerkte metadata, een ongeldig ELF, ontbrekende
compilerselectie, een vreemd `--extern`-pad, een verkeerd compilerbronbestand,
gewijzigde source/profilebinding, symlinks, dubbelzinnige of te grote JSON,
een verkeerde koude voorganger, hergebruik van dezelfde invocation,
veranderde schijfbytes, een nieuwe ACK tijdens recovery en gewijzigde
bron/HEAD tussen de runs.

Een afzonderlijke orchestratietest verandert de bronvingerafdruk direct na
de gesimuleerde build. Het resultaat wordt afgewezen **vóór de eerste
VM-aanroep**; de native runner wordt in die test niet aangeroepen.

Alle testbronnen, ELF-vormfixtures, compilerrecords en schijfbeelden zijn
expliciet hostfixtures. De tests fabriceren geen geaccepteerde gastseriallogs
en gelden niet als native OS-bewijs.

### Echte G11-compilecompatibiliteit, zonder VM

De immutable G11-bron
`91bf66ca2cdc00b7d2e220cd5afefd270d8fe062` en artifacts onder
`wt-g11/build/parallel-g11-document` zijn uitsluitend gelezen. De nieuwe
poorthelpers accepteren de werkelijk vastgelegde **20 broninputs**, **vier
rlibs**, **vijf rustc-commandorecords** en het geïnspecteerde app-ELF.

Werkelijk G11-app-ELF:
`38f5dee1b903b9ad86cbb6d44f801fb49d07c92567d501cd818d4c76ad04c831`.

`g11-compile-compatibility.json` bevat de bron- en artifacthashes. Deze controle
bevestigt de overeenkomst tussen G11 en het poortcontract; zij bewijst nog
geen kernelembedding, IPC-gedrag of koude herstart. Die onderdelen worden pas
door de daaropvolgende twee echte native runs afgesloten.

## Native uitvoeren na centrale freeze

Kies voor iedere nieuwe uitvoering een lege eigen buildmap en eigen targetmap.
Met de eerder gepinde Rust/QEMU/OVMF-omgeving kan de centrale integratie daarna
worden gecontroleerd met:

```sh
source /tmp/nieuw-os-par-tools/env.sh
export CARGO_HOME=/tmp/nieuw-os-par-tools/cargo-home
export CARGO_BUILD_JOBS=2
export NOS_DOCUMENT_SESSION=1
export NOS_SDK_RUNTIME=1
export NOS_BUILD_DIR="$PWD/build/document-session-native-1"
export NOS_TARGET_DIR=/tmp/nieuw-os-document-session-native-1-target
export CARGO_TARGET_DIR="$NOS_TARGET_DIR"
export NOS_QEMU_MACHINE=pc-q35-7.2
python3 -B tools/test_document_session.py \
  --expected-head FULL_FROZEN_COMMIT \
  --expected-source FULL_FROZEN_SOURCE_V3
```

De twee freezeparameters moeten samen aanwezig of samen afwezig zijn. Zonder
expliciete hashes bevriest de poort de actuele bron zelf vóór de eerste build.
De native uitvoering is voor deze taak nog **niet uitgevoerd** en mag pas na
de centrale integratie en bronfreeze starten.

Het begrensde JSON-resultaat staat onder
`$NOS_BUILD_DIR/document-session-gate/result.json`. Dezelfde map bevat de
beide compiler/artifact-snapshots en fase-uitvoer; de bestaande runner bewaart
de echte VM-serial, firmware-, kernel- en schijfartifacts onder de eigen
`$NOS_BUILD_DIR/run`-mappen. Mislukte stappen behouden hun logs en worden niet
als geslaagde native uitvoering geteld.

## Grenzen en roadmap

Dit is een gerichte poort voor twee native DocumentSession-fixtures; het is
geen volledige M6c-suite, G1/Materiaal-GUI-acceptatie, groter documentformaat,
fysieke stroomuitvaltest of hardwareclaim. Het bestaande maximum van 128
inhoudsbytes en de bestaande opslag-/capabilitygrenzen blijven gelden.

Roadmapvoorstel voor I13 zelf: **0 punten wijziging**. De poort en hosttests
maken de eerstvolgende native gebruikershandeling controleerbaar; zij
vervangen het nog uit te voeren native bewijs niet.
