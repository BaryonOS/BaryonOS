# INTERACTIVE-PROOF-1: opt-in invoerketen

Codebasis `99f8ef0d8b4f2f514a1246f7497f5562110a20ee`, branch
`simon/windows-qemu-test-version`. Deze begrensde stap wijzigt uitsluitend
de bestaande previewrunner, zijn directe hosttests en dit document.
Geen kernel-, build-, packaging-, status- of rubricwijziging.

## Contract en bewijsgrens

Zonder extra optie blijft de runner een boot/readiness/beeldproef met
standaard 45 s en maximaal 60 s. Met `--interactive-reference-run` is
maximaal 240 s toegestaan. De runner start het normale zevenprocessenprofiel
`native-system-materiaal`, nooit de automatische achtprocessenprobe.

De referentie moet een volledige, geslaagde normale native probe zijn.
`run_materiaal_probe.replay` hercontroleert de oorspronkelijke bestanden:
bronvingerafdruk v3, acht embedded ELF's, manifesten, gecorreleerde QMP,
volledige seriële fasen met schaal 1, echte 800x600-beelden, raw exit 33
en het onafhankelijke schijforakel. De zeven gewone apprecords, inclusief
SHA-256 en ELF-inspectie, moeten exact overeenkomen met de interactieve
build. Ook loader, machine, OVMF-code/template, VGA-ROM, QEMU-entry en
werkelijk gestart QEMU-programma zijn aan de referentie gebonden.
Een hash is hier herleidbaarheid, geen handtekening of menselijke audit.

Daarna doorloopt de runner dezelfde zes hardware-invoergroepen:
`ready`, echte muisklik naar `editor`, typen van `nieuw os werkt` naar
`dirty`, Ctrl+S naar `saved`, Esc naar `list`, Tab/Tab/Enter naar
`reopened`. De diagnostische sluitgroep met F12 is expliciet uitgesloten.
Er is geen controller- of snapshotafhankelijkheid in de interactieve gast.

Iedere fase vereist een volledige bytegelijke QEMU-PPM ten opzichte van
de gevalideerde native referentie. Dit omvat systeemranden, app, cursor en
tekstcaret; **er zijn geen pixelmaskers of uitgezonderde regio's**.
De gast krijgt pas de volgende invoer nadat het huidige beeld overeenkomt.
Herhaalde screendumps wachten uitsluitend op die exacte vergelijking,
binnen dezelfde deadline; kleuren, stabiele beelden of readiness vervangen
geen fasebewijs. Afwijkende geometry, ontbrekende fasen, gastfouten,
voortijdige exit en timeout falen zonder lossere terugval.

Na `reopened` volgt QMP `quit`, wachten op raw exit 0 en pas dan lezen
van de actieve wegwerpschijf. Het onafhankelijke `disk_oracle` eist precies
het getypte document, naam `Nieuw document`, revisie/generatie 1,
domeinklokken `[1, 0]`, geldige checksums/canaries en geen gewijzigd ander
object of collectie. Een levende of niet-nul geëindigde QEMU kan het
schijforakel niet bereiken. Referentie en bron worden aan het einde opnieuw
gecontroleerd. Rapporten bewaren referentie-/app-/framehashes, fasen,
QMP-trace, eindserialhash en voor-/naschijfhashes; lokale images blijven in
de eigen runmap.

Dit is geen controller-, snapshot-, procesopruimings-, koudherstel-,
handmatige UX-, Windows-gast- of volledige releaseacceptatie.
Exacte cursor-/caretgelijkheid is bewust een nog native te toetsen
haalbaarheidsaanname. Bij falen blijft de proef rood; geen ad-hoc masker.
De score krijgt met deze hostcode geen nieuwe punten: **34% -> 34%**.
De integrator beheert gedeelde voortgang en native bewijs afzonderlijk.

## Integratie en CLI

Integreer eerst deze code inhoudsbehoudend in de vaste integratiebron.
Bouw en toets daarna beide profielen op dezelfde actuele v3-bron.
Een toolswijziging verandert die vingerafdruk: oude referenties van
`344f497`/`c03070dd...` of `4bdc1a2`/`a6c4c723...` worden dus niet
stilzwijgend geaccepteerd. Gebruik geen nog lopende of onvolledige
`b02-gui`-run. Dit document voert geen bron-/imagevervanging uit.

Stel `REFERENCE_RUN` in op de absolute map van de **voltooide, geslaagde,
zelfde-bron normale probe**, met zijn eigen `result.json`, `inputs/`,
zes PPM's, QMP-trace, serial en schijfbeelden. Voor de normale interactieve
build en proef:

```sh
. .tools/nieuw-os/rtc-env.sh
export PATH="$PWD/.tools/nieuw-os:$PATH"
export NOS_BUILD_DIR="$PWD/build/native-test-version/interactive-proof-1"
export NOS_TARGET_DIR="$PWD/target/native-test-version/interactive-proof-1"
export CARGO_TARGET_DIR="$NOS_TARGET_DIR"
python3 -B -u tools/build.py --kernel-feature native-system-materiaal
python3 -B -u tools/run_materiaal_preview.py \
  --build-dir "$NOS_BUILD_DIR" \
  --interactive-reference-run "$REFERENCE_RUN" \
  --timeout 240 --machine pc-q35-8.2 \
  --bios-rom "$PWD/.tools/nieuw-os/emulator/usr/share/seabios/bios-256k.bin" \
  --vga-rom "$PWD/.tools/nieuw-os/emulator/usr/share/seabios/vgabios-stdvga.bin"
```

Resultaten: `$NOS_BUILD_DIR/preview-result.json` en
`$NOS_BUILD_DIR/preview-run/materiaal-<id>/result.json`.
Dit zijn toekomstige native commando's, **niet uitgevoerd in deze stap**.
Geen Windows-VM, ZIP of bundel aangemaakt.

## Uitgevoerde hostcontroles

De fixtures en QMP-mocks in de tests zijn uitsluitend synthetische
hosttegenvoorbeelden, geen gastuitvoering.

- `python3 -B -m unittest tools.tests.test_materiaal_preview tools.tests.test_package_windows_qemu tools.tests.test_materiaal_probe`
  — exit 0, 49 tests, 4,249 s; log
  `/tmp/nieuw-os-interactive-proof-1-targeted.log`.
- `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'`
  — exit 0, 127 tests, 5,579 s; log
  `/tmp/nieuw-os-interactive-proof-1-tools-tests.log`. Door mocks afgedrukte
  gast-/QEMU-tekst is geen uitvoering van nieuwe native VM's.
- `python3 -B -m py_compile tools/run_materiaal_preview.py tools/tests/test_materiaal_preview.py`
  — exit 0.
- `python3 -B tools/run_materiaal_preview.py --help` — exit 0.
- `git diff --check` — exit 0.

De negatieve gevallen omvatten andere bron/machine/apphash/loader,
ongeldig native referentiebewijs, ontbrekende fase, veranderde sluitgroep,
afwijkende geometry en gewijzigde pixels binnen of buiten de app.
Onjuiste ready-pixels geven geen muis-/tekstfallback. Geen invoer na de
deadline, geen F12, geen diskacceptatie met levende/niet-nul QEMU en
geen acceptatie van een lege, niet-opgeslagen schijf.
