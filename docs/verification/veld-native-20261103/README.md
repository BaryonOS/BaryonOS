# Veld: eerste native uitvoering — 3 november 2026

Basis `d6f8e98178d73d6aeda0abe84ca404dfde03123c`,
branch `simonvanieperen-fictional-disco`. Dit checkpoint bewaart de eerste
code en hostcontroles vóór de langere native build/VM-controle. Geen
volledige Veld-, U02–U06-, fysieke, menselijke of releaseacceptatie.

## Omgeving en uitgevoerde hostcontroles

De oorspronkelijke `python3 tools/preflight.py --strict` faalde met exit 2:
Rust/linker/QEMU/OVMF ontbraken. Daarna uitsluitend binnen de sessie-artifactmap
een geïsoleerde omgeving opgebouwd; geen systeeminstallatie of hostschijfmutatie.
Officiële Rust 1.90.0, beide eigen-OS-targets, rustfmt/Clippy, lokaal uitgepakte
distro-QEMU/firmware, bron-/pakket-/binaryhashes en echte linkchecks:
`files/veld-env/{report,validation,rust-packages,distro-packages,binary-hashes,crate-hashes}.json`.
Strikte preflight met `--strict-host-tests`: exit 0.
Node ontbrak eveneens; Node 22.22.0 ondersteunt de bestaande CI-optie
`--test-isolation=none` niet (exit 9). Officiële Node 24.11.0 met gecontroleerde
SHA-256 werkt; beide downloads bleven buiten Git.

Commandoprefix voor Rust/Python:
`source /home/simon/.copilot/session-state/7f2fb1d137004b68b9bef4b8ca72f938/files/veld-env/env.sh`;
hosttarget `CARGO_TARGET_DIR="$VELD_ENV/host-target"`.
De volledige logs staan in `$VELD_ENV/logs/veld-*`.

| Commando | Resultaat |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path ui/graphics/Cargo.toml` | exit 0; gradient/schaduw-tilegelijkheid, guards, ongeldige/extreme geometrie |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml` | exit 0 |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml` | exit 0; inclusief 9 workspace-v2-tests, exact volledig/incrementeel beeld bij 800×500, 1280×720 en 1920×1080/200% |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml` | exit 0 |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml` | exit 0 |
| `cargo +1.90.0 clippy --offline --locked --manifest-path <manifest> --all-targets -- -D warnings` | exit 0 voor alle vijf bovengenoemde manifests |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/window-session/Cargo.toml -- --check` | exit 0; appmanifests eveneens gecontroleerd |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/surface-protocol/Cargo.toml --features client` | exit 0 |
| `python3 -B tools/test_window_session.py` | exit 0; 12 tests, inclusief vervalste workspacepixels en diagnostic-only weigering |
| `python3 -B -m unittest tools.tests.test_package_windows_qemu` | exit 0 |
| `$VELD_ENV/node/node-v24.11.0-linux-x64/bin/node --test --test-isolation=none ui/prototype/tests/*.test.mjs` | exit 0; 39 tests, inclusief echte tokencontrastparen |

Tijdens ontwikkeling zijn echte fouten gevonden en hersteld: een verkeerd
geneste methode/test, een Clippy-lusbevinding, niet bereikbare zoekknop en
onvolledige dockdamage bij focuswisseling. Eerdere niet-geregistreerde tests
zijn niet als uitgevoerd bewijs gebruikt; de uiteindelijke testlijst telt
de nieuwe tests expliciet mee. Optionele fixture-/previewtests zonder
omgevingsvariabele zijn geen visuele of native acceptatie.

## Native en overige grenzen

Eerste native uitvoering op `093d26eb`: build exit 0 (11,34 s), echte
QEMU-workspaceproeven tot en met Finder-selectie en documentinvoer uitgevoerd.
De totale gate gaf **exit 1** na 61,20 s: de oude beginbeeldvergelijking werd
ten onrechte over de nieuwe, bewust focuswijzigende shelltaken heen gebruikt.
Een verse pixelanalyse vond 296 gewijzigde calculator-caretpixels; de
workspace-selectiecapture kon bovendien vóór volledige chrome-repaint vallen.
Dit is geen geaccepteerd native resultaat.

De runner wacht nu na de shelltaken op de volledige exact gedefinieerde
chrome-complementgebieden én het oorspronkelijke volledige Materiaalbeeld.
Dat vastgelegde stabiele beeld is daarna de basis voor dezelfde ongewijzigde
vergelijking van **alle** pixels buiten de appbody; er zijn geen extra
uitzonderingspixels of maskers toegevoegd. Bestaande volledige apporakels,
crash-, schijf- en cleanupgates blijven verplicht. Gerichte scene-/runnertests
en Clippy slagen opnieuw; de volledige native herkansing volgt nog.

De eerste buildpoging naar de artifactmap buiten de checkout is terecht
geweigerd (exit 1, `FPU artifact escaped checkout`). Geen guard aangepast;
de build gebruikt daarna de genegeerde lokale mappen
`NOS_BUILD_DIR="$PWD/build/veld-native-20261103/probe"` en
`NOS_TARGET_DIR="$PWD/target/veld-native-20261103"`.
CI is aangepast, niet als geslaagd geteld.
Een host-renderoracle is geen VM-test. QEMU is geen fysiek GPU-, driver-,
energie- of timingbewijs. Geen nieuwe gebruikerstest, schermlezerproef,
onafhankelijke menselijke review of volledige releasecontrole uitgevoerd.
Voortgang **41 → 41**; geen criteriumstatus, gewicht of historisch bewijs gewijzigd.
