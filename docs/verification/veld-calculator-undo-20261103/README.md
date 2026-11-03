# VELD-09: herstelbare rekenmachinecorrecties

Status: **QUEUED-FOR-INTEGRATION**, geen OPEN-PR-LEASE en dus geen nieuwe PR.
Basis: actuele main `6d523964ab85638d4dd981bcdb19580d18646fab`.
Eigen branch: `veld/calculator-undo-20261103`. Geen VELD-08-import;
`scene.rs`/`init.rs` blijven ongewijzigd en tijdelijk bij General.
Geen PR zonder OPEN-PR-LEASE; deze milestone gaat na acceptatie naar
QUEUED-FOR-INTEGRATION.

## Gebruikersinteractie en invarianten

Ctrl+Z herstelt een formulewijziging; Ctrl+Y of Ctrl+Shift+Z herhaalt die.
Een plakactie of selectievervanging is een enkele transactie. Backspace,
Delete, C en Escape zijn terug te draaien. Herstel brengt de oorspronkelijke
cursor en selectie terug en annuleert een nog ingedrukte rekenknop.
Navigeren, focuswisselen, rekenen, afgewezen invoer en lege verwijderacties
maken geen nieuwe teksttransactie. Een nieuwe wijziging wist de redo-tak.

Maximaal 32 wijzigingen, uitsluitend in vluchtig, app-eigen geheugen:
geen allocator, dienst, capability, opslag, app-ABI of nieuwe dependency.
De oudste wijziging vervalt wanneer de begrensde geschiedenis vol is.
Herstel rekent nooit automatisch en verklaart een oud antwoord nooit actueel;
Enter blijft nodig. C wist nog steeds het resultaat; undo herstelt daarvan
alleen de formule. Dit is geen beveiligde wisactie van appgeheugen.
Toetsherhaling, releases, Alt/Super en Ctrl+Shift+Y voeren geen undo/redo uit.
De bestaande statusregel toont de beschikbare sneltoetsen; fouten en
afrondingsuitleg blijven voorgaan. Geen uitsluitend kleurafhankelijke bediening.

## Schrijfclaim

Alleen `sdk/examples/calculator/src/{model,render}.rs`,
`tests/undo.rs`, een nieuwe test in calculator `tests/chain.rs`,
de opt-in `--calculator-undo`-route in `tools/run_window_session.py`,
bijbehorende tegenproeven in `tools/test_window_session.py` en dit bewijs.
De oude defaultgate blijft behouden. Deze deelgate kan uitsluitend
`calculator_undo_gate=true`, niet volledige `native_gate=true`, opleveren.
Geen algemene notification/core-API, General-, Maturity-, Driver- of
Professional Platform-writes. Geen centrale status-/scorewrites.

## Uitgevoerd voor native checkpoint

Met de bestaande `veld-env/env.sh` en
`CARGO_TARGET_DIR="$VELD_ENV/host-target"`:

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml --test undo
cargo +1.90.0 fmt --manifest-path sdk/examples/calculator/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml --all-targets -- -D warnings
python3 -B sdk/examples/calculator/build.py
python3 -B sdk/examples/calculator/build.py clippy
```

De eerste testpoging faalde verwacht met exit 101: undo/redo ontbraken.
Log `$VELD_ENV/logs/veld09-undo-red.log`. Geen bestaande productregressie
geclaimd. Na implementatie acht nieuwe tests exit 0, inclusief alle gewijzigde
pixels versus damage op 320x240@1, 320x484@1 en 640x608@2.
Log `veld09-undo-targeted.log`.

Volledige calculatorhostsuite exit 0: 50 tests, waarvan een bestaande
opt-in fixture zonder invoer geen native bewijs levert. De nieuwe echte
capability-/INP1-/SUR1-ketentest controleert beide schalen, veertien exacte
key-/ACK-events, actuele framegeneraties en expliciete herevaluatie.
Suitewandtijd 5,33 s; Clippy exit 0. Log `veld09-undo-host.log`.
Freestanding crossbuild, ELF-inspectie en target-Clippy exit 0:
`veld09-undo-target.log`. Dit bewijst compilatie, geen OS-uitvoering.

## Uitgevoerd: gerichte native acceptatie

```sh
export NOS_BUILD_DIR="$PWD/build/veld09-undo-20261103"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --calculator-undo --timeout 600 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

Op bron `f8c08ffe33a411253c97e118758828c52c423472`:
build exit 0, 3,66 s; runner exit 0, 46,14 s, tien fasen;
ruwe QEMU-exit 33 en normale cleanupmarkers.
`passed=true`, `calculator_undo_gate=true`, bewust `native_gate=false`.
Bronfingerprint:
`04dfc32c7585c6e9469f159ad19414466c483082ee1db993b8aa7402c8205fe8`.
Log `$VELD_ENV/logs/veld09-undo-native-attempt1.log`.
Het compacte `native.json` behoudt manifest, images, firmware, serial,
frames, raw-reportpad/hash en een canonieke QMP-tracesamenvatting.
De ruwe artifacts staan in de daarin genoemde lokale buildrun.
`logs.sha256` bindt alle zes ontwikkellogs, inclusief de rode compileertest.
De hashcontrole en `git diff --check` zijn exit 0.

De nieuwe route typt werkelijk 2+3, rekent, corrigeert en herstelt met alle
drie sneltoetsen. Zij vergelijkt volledige appbeelden bij roundtrips,
controleert stale/current en evaluatietellers, weigert repaint buiten de
calculatorbody en vereist normale cleanup met ruwe QEMU-exit 33.
Dit is geen volledige herstel-/opslagacceptatie of herhaling van VELD-08.
Geen nieuwe volledige schaalmatrix: deze appdelta heeft hostpixeldekking op
beide schalen; de gerichte VM-route gebruikt het bestaande 100%-profiel.

`python3 -B tools/test_window_session.py`: exit 0, twaalf runnertegenproeven
waarvan twee nieuwe tegen onjuiste combinaties en gewijzigde model-/bindingsvelden.

Nog niet uitgevoerd: gewijzigde-mainintegratie, CI,
fysieke hardware-, menselijke gebruiks-, onafhankelijke audit- en releasegates.
Geen scoreverhoging voorgesteld. Rollback: uitsluitend deze scopedelta reverten.
De milestone is bevroren; nieuwe code volgt uitsluitend op een aparte
actuele-mainbranch, niet als tweede featurebatch hier.
