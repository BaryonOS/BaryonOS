# Sessieroute — hostbeelden en visuele review

**Datum:** 2 november 2026. **Rendererbron vóór polish:** `7d1858f`. **Na polish:** `20fc36e`.
**Actuele artifactbron:** `703cc0705471062ffb1483a16f42a2e580a6fdf7`.
**Bewijsniveau:** hostgerenderde echte Rustcode, synthetische data. Geen HTML,
getekende vervangende app, QEMU-screenshot, echte authenticatie of hardwareproef.
Score blijft 41/100; dit bewijs sluit geen nieuwe gate.

## Herkomst en reproduceerbaarheid

`ui/native/tests/session_board.rs` koppelt de echte window-session `Scene` en
`Shell`, de echte Materiaal `Controller`/`Frame` en calculator `Model`/renderer.
De Materiaalfixture gebruikt de bestaande capability-engine, volumeservice en
geordende RAM-sectoren. Drie synthetische documenten zijn werkelijk geschreven;
het getoonde document is werkelijk heropend. De calculator evalueert werkelijk
`1250*0,21`. Auth toont bewust de beschikbare gesloten dienstgrens en verzamelt
geen persoonlijke credentials; desktopstart doorloopt de aparte openbare VM-gast.
Instellingen en capability-aanvraag zijn bestaande shelltoestanden, geen grants.
De vaste UTC-waarde is uitsluitend displayfixture, geen authenticatietijd.

Het board toont acht echte renderoutputs op halve primaire resolutie. PNG is
verliesloos uit Rust-PPM geconverteerd, zonder retouche of gemaskeerde gebieden.
De pointer staat in de actuele captures bewust op ongebruikte achtergrond;
het bewaarde vóór-board houdt zijn oorspronkelijke pointerpositie. Bronlabel
komt uit de host-worktree; deze fixture kan geen QEMU-uitvoering vaststellen.

```sh
source /workspace/.nieuw-os-env/activate.sh
mkdir -p build/session-board-fresh
OS_SESSION_BOARD_DIRECTORY="$PWD/build/session-board-fresh" CARGO_TARGET_DIR="$PWD/build/session-board" cargo test --manifest-path ui/native/Cargo.toml --test session_board --release --offline --locked
CARGO_TARGET_DIR="$PWD/build/session-board" cargo clippy --manifest-path ui/native/Cargo.toml --test session_board --release --offline --locked -- -D warnings
rustfmt --edition 2021 --check ui/native/tests/session_board.rs
git diff --check
```

Gebruik een nieuwe lege artifactmap: de fixture weigert bestaande PPM's te
overschrijven. De twee nieuwe devdependencies zijn uitsluitend bestaande lokale
window-session/calculator-crates. Geen nieuwe registryversies of runtimehardware.
Logs: `/workspace/.nieuw-os-env/session-board-{render,clippy}.log`.

**Uitgevoerd:** één rendertest met 40 toestanden (acht per resolutie), exit 0;
Clippy met `-D warnings`, formattering en whitespacecheck exit 0. Eerste compile
miste een directe type-import; de fixture hergebruikt nu het echte geometrytype
zonder extra dependency. Een eerste Clippy vond een te brede helperinterface;
die is verkleind en opnieuw gecontroleerd. Geen fout is als groen gerapporteerd.

## Beelden

- [Acht toestanden vóór polish](screenshots/session-board-before-polish-host.png)
- [Actueel board](screenshots/session-board-host.png)
- [Twee vensters primair](screenshots/1280x720-s1-two-windows.png)
- [Kleine werkruimte](screenshots/1024x768-s1-two-windows.png)
- [200% werkruimte](screenshots/1920x1080-s2-two-windows.png)
- [200% capability-aanvraag](screenshots/1920x1080-s2-capability.png)

De map bevat lock/users/auth/empty-desktop/two-windows/search/settings/capability
op 1280×720, 1920×1080, 1440×900, 1024×768 en 1920×1080 op schaal 2. Elk bestand
noemt maat, schaal en toestand. Het board is een contactblad, geen negende
OS-toestand. 100%-beelden op andere resoluties zijn geen fysieke displaytests.

## Eerste visuele beoordeling

AI-beeldreview van het hele primaire board en individuele primaire, kleine,
1920- en 200%-beelden. Dit is geen menselijke gebruikersstudie. Alle 40 beelden
hebben daarnaast geometriechecks voor zichtbare controls/systeembalk/footer;
geen volledige automatische OCR- of schermlezertest.

**Vooruitgang:** tijd/status hebben een vaste plek, launcher/appnamen zijn
leesbaar, actieve Materiaalrand en inactieve calculatorrand verschillen,
venstercontrols zijn herkenbaar en documentinhoud vervangt een leeg testvlak.
Witte/warme vensters tegenover inktkleurige systeemlaag maken hiërarchie
zichtbaar zonder Vista-glas of neon. Bekende labels verlagen het giswerk.

**Concreet gevonden vóór polish:**

1. Zoeken accepteert toetsenbordinput maar toont alleen een tekstregel en
   resultatenknoppen, geen herkenbaar invoerveld. Dit is een bedieningsgat.
2. Locktijd gebruikt dezelfde fysieke atlasmaat op 100% en 200%; op 200% is de
   tijd relatief kleiner dan de omliggende typografie. Datum staat op 100% te
   dicht bij de grote tijd.
3. Calculator-meta “9 decimalen” wordt op 200% rechts afgesneden doordat oude
   monospacemeting niet bij proportionele labels past.
4. De algemene panelen houden onder weinig controls veel lege ruimte. Users,
   auth en settings lijken daardoor nog te veel op dezelfde demo-opstelling.
5. 1920×1080 op 100% plaatst de apps ver uit elkaar en laat veel middenruimte;
   calculatoronderruimte is nog groot. Deze vensterindeling blijft open.

Geen “premium consumentenproduct”-eindoordeel op basis van deze tussenstand.
Documentbody gebruikt nog bewust monospace voor juiste caret-/selectiemetrics;
volledige shaping/IME en OS-brede schermlezer ontbreken. De capability-aanvraag
noemt programma/object/handeling/bestemming/duur, maar mist nog een echte broker
voor gevolg/intrekking. De UI maakt dit zichtbaar, zonder grant te simuleren.

Nominale kleurcontrastberekening (sRGB-luminantie, geen conformiteitsclaim):
primaire tekst `#20262D` op canvas `#F2F1EC` 13,49:1; secundair `#5C6570` 5,23:1;
focus `#275DAD` op wit 6,45:1; lichte systeemtekst op `#18242F` 14,29:1.
Antialiasing, hover, disabled-toestanden en alle gebruikte combinaties vragen
nog aparte beoordeling; deze vier paren bewijzen geen algemene toegankelijkheid.


## Gecontroleerde verbetering na beeldreview

Productbron `20fc36e` wijzigt alleen `ui/graphics/src/text.rs`,
`sdk/examples/window-session/src/shell.rs` en
`sdk/examples/calculator/src/render.rs`. Geen nieuw fontasset, parser,
privileged code, rechten of grotere readonlysegmentgrens.

- Zoeken heeft nu een zichtbaar focusbaar veld met invoerplaceholder, query en
  caret. Click in het veld start geen app; Enter kiest het eerste actuele
  resultaat; Tab/Shift+Tab wisselt tussen veld en resultaten. De query blijft
  begrensd; lange invoer blijft binnen het veld.
- Locktijd en datum gebruiken fontmetrics voor hun verticale afstand. De
  displayface gebruikt dezelfde bestaande 64-pixelatlas op 100%, met maximaal
  2× coverage-uitbreiding op 200%; glyphbounds blijven maximaal 192×192.
  Dit is geen nieuw optisch gerasterd high-DPI-font. Gewone bodyfonts houden
  hun eigen 100/200%-rasters en gedrag.
- Calculator-meta wordt met echte proportionele `Font.measure` uitgelijnd.
  “9 decimalen” past nu op 200% volledig binnen het appvenster.
- Gebruikerskeuze, auth en snelle instellingen hebben een inhoudsafhankelijke
  compactere paneelhoogte. Controls blijven binnen de veilige schermruimte.

Het actuele board en alle 40 individuele beelden zijn opnieuw uit de echte
Rust-renderers gemaakt. De nieuwe primaire boardcompositie en individuele
zoek-, lock-, 1440-auth- en 200%-calculatorbeelden zijn visueel bekeken. De
bovenstaande vier problemen zijn zichtbaar verbeterd. Het 1920-middengat,
monospaced documentediting, beperkte fontcoverage/shaping, ontbrekende echte
auth-/keyringbroker en OS-brede toegankelijkheid blijven open. Deze slice
heeft herkenbare OS-bediening, maar dit is geen volwaardig dagelijks product.

Historische aanvullende checks op productbron `20fc36e`, alle exit 0:

```sh
CARGO_TARGET_DIR="$PWD/build/graphics-polish" cargo test --manifest-path ui/graphics/Cargo.toml --offline --locked
CARGO_TARGET_DIR="$PWD/build/window-polish" cargo test --manifest-path sdk/examples/window-session/Cargo.toml --offline --locked
CARGO_TARGET_DIR="$PWD/build/calculator-polish" cargo test --manifest-path sdk/examples/calculator/Cargo.toml --offline --locked
CARGO_TARGET_DIR="$PWD/build/graphics-polish" cargo clippy --manifest-path ui/graphics/Cargo.toml --all-targets --offline --locked -- -D warnings
CARGO_TARGET_DIR="$PWD/build/window-polish" cargo clippy --manifest-path sdk/examples/window-session/Cargo.toml --all-targets --offline --locked -- -D warnings
CARGO_TARGET_DIR="$PWD/build/calculator-polish" cargo clippy --manifest-path sdk/examples/calculator/Cargo.toml --all-targets --offline --locked -- -D warnings
CARGO_TARGET_DIR="$PWD/build/session-polish-scalar" cargo build --manifest-path sdk/examples/window-session/Cargo.toml --target x86_64-unknown-none --offline --locked
```

Daarnaast slaagt de volledige ui/native-hostsuite in releaseprofiel: 63 tests
(waaronder de nieuwe matrixfixture), exit 0. Commando:
`CARGO_TARGET_DIR="$PWD/build/native-polish-host" cargo test --manifest-path ui/native/Cargo.toml --release --offline --locked`.

Graphics: 9 tests; window-session: 49 tests, inclusief alle 7 bestaande
shelltests en 2 nieuwe zoek-focusproeven; calculator: 30 tests. Nieuwe fonttests
controleren displaymetrics, glyphgrens, tiles en guardpixels. De no_std-build
is geen native uitvoering. Logs staan buiten Git onder
`/workspace/.nieuw-os-env/session-polish-*.log`.

## Finale artifactherkoppeling op 703cc07

De bestaande fixture is opnieuw gebouwd en uitgevoerd op de ongewijzigde
codefreeze `703cc0705471062ffb1483a16f42a2e580a6fdf7`, detached worktree
`/workspace/nieuw-os-session-final-board`. Er zijn geen bronbestanden, tests,
manifesten, dependencies of locks aangepast. Dit is een nieuwe uitvoering,
geen herlabeling van de oude PNG's.

```sh
source /workspace/.nieuw-os-env/activate.sh
mkdir -p build/session-final-images
OS_SESSION_BOARD_DIRECTORY="$PWD/build/session-final-images" CARGO_TARGET_DIR="$PWD/build/session-final-target" cargo test --manifest-path ui/native/Cargo.toml --test session_board --release --offline --locked
```

**Uitkomst:** exit 0; acht toestanden op vijf profielen (40 beelden) plus het
board. Alle PPM's zijn opnieuw verliesloos naar RGB-PNG geconverteerd. Elke
teruggelezen PNG is per pixel met de bijbehorende verse PPM vergeleken.
De RGB-pixels van alle 40 matrixbeelden zijn exact gelijk aan de vorige
matrix; het contactblad heeft een nieuw correct bronlabel. Dit ondersteunt
de rastercontinuïteit na de cover-/readinesswijzigingen, geen claim over
native displaylatency of runtime-input. Het oorspronkelijke vóór-board uit
`7d1858f` is byte voor byte behouden en tegen de eerste artifactcommit getoetst.

Opnieuw visueel bekeken: het hele primaire board, primaire twee-vensterbeeld,
200%-lock/zoeklaag en de kleine 1024×768-werkruimte. Geen nieuwe clipping,
onbedoelde control-overlap of onleesbare hoofdactie waargenomen. Geometriechecks
in de fixture slagen op alle 40 toestanden. De eerder genoemde resterende
tekortkomingen (monospaced editing, optische displayfontkwaliteit, brede
vensterafstand op 1920 en ontbrekende echte accountdiensten/toegankelijkheid)
blijven open. AI-beeldreview is geen menselijke gebruikerstest.

De historische Clippy-/cratetestresultaten hierboven blijven bij hun eigen
bron; voor deze pure artifacttaak is de bestaande matrixfixture opnieuw
uitgevoerd. Root rapporteert de finale brede en native gates afzonderlijk.
Log: `/workspace/.nieuw-os-env/session-final-board-render.log`.

## Artifactbinding

Het machineleesbare [hashrapport](visual-artifact-hashes.json) bindt alle
verse PPM-/PNG-/RGB-hashes, vijf profielen, acht toestanden, bronbestanden,
rendercommando, exitstatus en bewaarde before-board. De source-v3-vingerafdruk
is vóór/na genereren gecontroleerd:

`e4ad9f922d202ee4249b6b1dd02a02d724f6775c616ad1c46dde48bf7ee2271c`.

De vijf kernbronnen voor compositor, sessiebediening, klokfont en calculator,
plus fixture/atlas/authcontract, zijn afzonderlijk gebonden:

| Bronbestand | SHA-256 |
|---|---|
| `sdk/examples/window-session/src/scene.rs` | `ead69d27c7e8f08cbfaec3ba78654ef82b120758cfb5936b98d01c8ded1c58b2` |
| `sdk/examples/window-session/src/main.rs` | `7ce27ec096ad7bedf2e3d11a000ef70415476bb2b35a430f802d8384e3a05b22` |
| `sdk/examples/window-session/src/shell.rs` | `8f1b6905ad98f12e006896b0c0e453c225ab7285e3030a5098325c4da349ba6f` |
| `ui/graphics/src/text.rs` | `a06ba18c2459211d63293e644a37516676db1e711fb093950205af91161bb6e9` |
| `sdk/examples/calculator/src/render.rs` | `31ccfc62c37cf3587040fcde9633a43591cc86037b4ccc75bb09539dceabccfd` |
| `ui/native/tests/session_board.rs` | `28dec36d58c525988e1f8b314d298782e49ab8c5d2691e5e035d50e365963a71` |
| `ui/graphics/assets/open-sans-atlas.rle` | `a4e3efa8027d5eba26bea742f7350bda34380cab9d77030d21481cb06c5f2e07` |
| `ui/native/src/session.rs` | `0388858099523e1b538ea5fca846b6003cd0256d31d62070d27072238071cc13` |

Het bronlabel is een host-artifactlabel. Dit document geeft geen QEMU-,
FIDO2-, biometrie-, hardware- of nieuwe scoreclaim.
