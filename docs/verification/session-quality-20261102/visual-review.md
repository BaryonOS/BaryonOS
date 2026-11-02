# Sessieroute — hostbeelden en visuele review

**Datum:** 2 november 2026. **Rendererbron vóór polish:** `7d1858f`. **Na polish:** `20fc36e`.
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

Aanvullende checks op de productbron, alle exit 0:

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

## Artifactbinding

Hashes hieronder binden de uitgevoerde renderoutputs; de visueel bekeken
selectie staat hierboven expliciet beschreven. De before-board blijft
op `7d1858f`; de actuele matrix is productbron `20fc36e`. Niet naar een latere
integratorcommit herlabelen zonder nieuwe render-/integratiecontrole.

| Bestand | SHA-256 |
|---|---|
| `screenshots/1024x768-s1-auth.png` | `2c48c6b94c32d462962fb99fb4c89c2e0c7e72c68fb6e543bc71ae3c176da7fe` |
| `screenshots/1024x768-s1-capability.png` | `b446d581177ccce2ab60db9604af8833ea70f1ffdfe2d9ab0f603fbfa17fe76f` |
| `screenshots/1024x768-s1-empty-desktop.png` | `6bb1a2e3f0e0021d686a1c13834f96fbd4f6bf088e86d66a03f3d325a9159886` |
| `screenshots/1024x768-s1-lock.png` | `7400bb0d8563fee1fdf3de9c55538a8bbe7bb0af8542afefadb7e963e569a4ea` |
| `screenshots/1024x768-s1-search.png` | `d7d2c6868d4c4bcc8be53a63e8f885611d0d9c4844460c592eb1985f1a4b20df` |
| `screenshots/1024x768-s1-settings.png` | `ee2efd5c4066d24e42fa65740a57e03fec19e1319f786f529a72f7a0fa2b1c92` |
| `screenshots/1024x768-s1-two-windows.png` | `0c3aa62d05e743a68192d431dfde7f0d3f571c01f5d14afe0c10c5c8ba187002` |
| `screenshots/1024x768-s1-users.png` | `8688252ac4b87c45bd7a41ed214541f3bec5d470f4fee21244ed85815d3c39fa` |
| `screenshots/1280x720-s1-auth.png` | `e74c5737c80488b016c189dfee1a72003c74c3527448f805c19862a35df9d673` |
| `screenshots/1280x720-s1-capability.png` | `329101c9f4052544e90effec6318b40d65a1fd6889a1d9c6489c94be6a02e69c` |
| `screenshots/1280x720-s1-empty-desktop.png` | `fd71108d586b991d4d1ededfde10f06ae88dc9ebb08fd1813da4666dc8b35527` |
| `screenshots/1280x720-s1-lock.png` | `51755b6a8719139b65f8176af1d189562063d6adb5a45e34002073f141cb823c` |
| `screenshots/1280x720-s1-search.png` | `1127ed5dd6036fc5437d107838740f1ba08061a20b03a9ba87e39f23e662a164` |
| `screenshots/1280x720-s1-settings.png` | `b71bca734175055f5172a85b22330618ce06abe0e4ae2843760dffba767a9a20` |
| `screenshots/1280x720-s1-two-windows.png` | `b843b3eb53849b5eef5273b44e338059f38340bcdadf2f09e4c1422341bb4614` |
| `screenshots/1280x720-s1-users.png` | `55adc0d524ba68699591c82f28aec771d4a0d9b713bc347221975d80de886c4a` |
| `screenshots/1440x900-s1-auth.png` | `514ae0109bb2533997b1fde73917e27c7605257e38f695b54cd475c063fc53b3` |
| `screenshots/1440x900-s1-capability.png` | `914d0567c5c379d50301bcc43ef7f277b18c6bbf836e8345e99e2d81b5316a32` |
| `screenshots/1440x900-s1-empty-desktop.png` | `dfe6babeda0b69219778f78f32ba83a4a97b84d3d23a8b5c89d0c6d5da46b184` |
| `screenshots/1440x900-s1-lock.png` | `b7afd1b2c4c8999fdad56591b3fdda322cc95d04a3d62b863596b7197d26c391` |
| `screenshots/1440x900-s1-search.png` | `b61e0b1dbe792229d7d38585395f1e74852e44c7339dd361c7de07309e5ff2a6` |
| `screenshots/1440x900-s1-settings.png` | `6020c574a3491391935f1a2c2f5154298625a1a13e571fef6a16fee50b185cfd` |
| `screenshots/1440x900-s1-two-windows.png` | `59e585a4eb94abcaa25b4287d710956718ad58972bd04a37fbe9507074414e2f` |
| `screenshots/1440x900-s1-users.png` | `277dd48eb32555d5eea9702301bb5d1ecca24b33dd0631d5fa4a17b52eb27ab1` |
| `screenshots/1920x1080-s1-auth.png` | `94b385e4b745a34ca150f63edf750aeaf2d0d9bbce1e485d2bdd76d61d18f925` |
| `screenshots/1920x1080-s1-capability.png` | `68adf530b941507a0b9b5c7069d81bab97b70061d40bc43c50cc24a0444e259e` |
| `screenshots/1920x1080-s1-empty-desktop.png` | `ed8ad7620bb99628de411e095f596e2357c9fc97a182c5192303577866b02034` |
| `screenshots/1920x1080-s1-lock.png` | `877560a1ed054235366cb441a3b7b9af2e1849ae170fd0f55f6c46763771884c` |
| `screenshots/1920x1080-s1-search.png` | `a71d8636856476a3f2fa559ee3eaab61a82c35e31002e155a5d1c924212f50d0` |
| `screenshots/1920x1080-s1-settings.png` | `e036da7d29feb3914a41318ca1bc6149d7a391c73e7bc56198334cadd8fe2826` |
| `screenshots/1920x1080-s1-two-windows.png` | `a6a3c282d100a34ff6e62b167f7cc67aaff090e4f84850d4b3b8d47db92579df` |
| `screenshots/1920x1080-s1-users.png` | `5418be795497f32a05601e4ab1daa1ca974ba2932ca51549da68245c615d1c17` |
| `screenshots/1920x1080-s2-auth.png` | `86fc74d3a4d60d8af302bc946f2696c9b951e38f86024219151d5b13a30dd9d2` |
| `screenshots/1920x1080-s2-capability.png` | `78405992eb4ceecb3c24d856af9cf71e8cc87a567df7a595d20d8a9fd5036086` |
| `screenshots/1920x1080-s2-empty-desktop.png` | `9d55c78fad81db06dbdf958e39c0b03c8c6b088d8e58d5464768dd51272831b1` |
| `screenshots/1920x1080-s2-lock.png` | `87dd9b21da8d5ef89432085fe410fd085f29a8ce8c86279f3b737422ce12b12a` |
| `screenshots/1920x1080-s2-search.png` | `bba4f9d34a432a11a9990d9d60f91b0f66b61efdedd5dd20c9d84f07f55c05c1` |
| `screenshots/1920x1080-s2-settings.png` | `ed1af4273090947f4b06dd94d0ffa5d13b5827ef8bdec5b5920e2e01cb9e7f0c` |
| `screenshots/1920x1080-s2-two-windows.png` | `e5622270360e745bdb5aefe969bb490d8f81b5aa386812e1a37da061ac729e0f` |
| `screenshots/1920x1080-s2-users.png` | `9ec3c2a80b8968fb58c78c249be7786ee95f30a01983417b78eca1d6f08f69ce` |
| `screenshots/session-board-before-polish-host.png` | `af7e6655087c0d4fd94a35431747be1b8cf19ce38c3197bc33942686f697d56b` |
| `screenshots/session-board-host.png` | `1187ca3956ffa5d101df1d5af1bacf84e627a05f2731fea7010e3c1faddf0906` |

| Bronbestand | SHA-256 |
|---|---|
| `ui/native/tests/session_board.rs` | `28dec36d58c525988e1f8b314d298782e49ab8c5d2691e5e035d50e365963a71` |
| `ui/graphics/src/text.rs` | `a06ba18c2459211d63293e644a37516676db1e711fb093950205af91161bb6e9` |
| `sdk/examples/window-session/src/shell.rs` | `6dff0dd3c85bf44ad8bc61fd09432d701da0e9de6b2bbe0f923314580e702a1e` |
| `sdk/examples/calculator/src/render.rs` | `31ccfc62c37cf3587040fcde9633a43591cc86037b4ccc75bb09539dceabccfd` |
| `ui/graphics/assets/open-sans-atlas.rle` | `a4e3efa8027d5eba26bea742f7350bda34380cab9d77030d21481cb06c5f2e07` |
