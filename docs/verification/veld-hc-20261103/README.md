# Veld-05 — app-lokaal hoogcontrast

**Tussenstand voor native acceptatie.** Implementatiebasis
`16c26c1c4b2d20e4644633439edf1dfe848bc140`,
branch `simonvanieperen-veld-hc-20261103`. Geen opname van de onverwachte
oude remote-delta `dcb97175`, geen scoreimport van General.
De controller-/renderleaf blijft ongewijzigd na expliciete filevrijgave.

Ctrl+Alt+H wisselt de presentatie van de gefocuste Materiaal- of
Rekenmachine-app, niet de shell of een persistent systeemprofiel.
Materiaal valideert eerst de bestaande input. Alleen Press togglet;
selectie, document, caret, menu, fout/status en berekening blijven behouden.
Geen opslag-RPC of nieuwe bevoegdheid. Gedeeld rolpalet,
zwarte selectietekst op cyaan en dubbele knopfocusgrens.
Zie [ADR-VELD-004](../../DECISIONS.md).

## Uitgevoerd: hostbewijs

Na de bestaande geisoleerde `veld-env/env.sh`:

```sh
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
for manifest in ui/graphics/Cargo.toml ui/native/Cargo.toml sdk/examples/materiaal/Cargo.toml sdk/examples/calculator/Cargo.toml; do
  cargo +1.90.0 fmt --manifest-path "$manifest"
  cargo +1.90.0 test --offline --locked --manifest-path "$manifest"
  cargo +1.90.0 clippy --offline --locked --manifest-path "$manifest" --all-targets -- -D warnings
done
python3 -B tools/test_window_session.py
```

Alle opdrachten exit0; runner17 tests, calculator19 model/render-tests.
Log: sessie-artifact `files/veld-env/logs/veld-hc-app-final-host.log`.
Echte Materiaal-inputdienstketen toetst Press/Repeat/Release, geen opslag-RPC,
behoud van document/caret/status en pixelgelijk standaardframe na terugschakelen.
Calculator toetst vier viewports, volledige schadegrens/no-op, selectie,
berekening en foutbehoud, exacte modifiercombinatie en rolcontrasten.
Native UI behoudt de bestaande72 standaardframefingerprints en zes
viewports. Pythontegenproeven wijzigen hoek-/middenpixels in alle vier
nieuwe fasen op beide schalen; er is geen kleur- of appbodymasker.

Eerste nieuwe fouttest verwachtte onterecht een geslaagde evaluatie bij
`7/0`; zij bewaart nu de bestaande succes- en fouttellers.
De nieuwe luminantietest vereiste expliciete u32-kanaaltypen.
Productgedrag is voor beide testfouten niet veranderd; volledige keten herhaald.

## Nog niet uitgevoerd: native acceptatie

Bevries eerst dit productcheckpoint. Voorgeschreven sequentiele opdrachten:

```sh
export NOS_BUILD_DIR="$PWD/build/veld-hc-20261103/scale1"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 1 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
export NOS_BUILD_DIR="$PWD/build/veld-hc-20261103/scale2"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context-hidpi"
python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 2 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

Daarna normale builds met eigen `normal1`/`normal2`-buildmappen,
dezelfde respectieve targetmappen en features `native-window-session`
en `native-window-session-hidpi`.
Verwacht48 volledige fasen per schaal, echte QMP-hotkeys bij open documentmenu
en bij Rekenmachine, behouden berekening en exact herstel van het standaardframe.
Alle oorspronkelijke crash/rebind-, klembord-, opslag-, canary- en cleanupgates
blijven verplicht. Een build is geen runtimebewijs.

Nog geen native HC-, nieuwe CI-, fysieke hardware-, menselijke gebruiks-,
schermlezer- of releasevalidatie. Globale instellingen/persistence, shell-HC,
reduced-motion en overige complete Veld-functies blijven open.
Score **41 -> 41**, zonder criterium- of gewichtswijziging.
