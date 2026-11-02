# Sessieroute — hostbeelden en visuele review

**Datum:** 2 november 2026. **Rendererbron vóór polish:** `7d1858f`.
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
De pointer staat voor de captures bewust op ongebruikte achtergrond. Bronlabel
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
