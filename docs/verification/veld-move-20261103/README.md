# Veld-06 — toetsenbordverplaatsing

**Native deelacceptatie geslaagd.** Productbron
`0d232c7028ba0696c177a568d07e0e50f8d6f269`, basis
`4328426434edd01d4f7bb8a40c845b4e3bcae70e`, branch
`simonvanieperen-veld-hc-20261103`. Geen opname van General-werk of
de oorspronkelijke remote-delta. Schrijfgrens: window-session scene/main,
eigen hosttests, shellfixtures en bestaande runner/CI.

Alt+F7 opent echte begrensde verplaatsing van het zichtbare gefocuste venster.
Pijlen16/Shift+pijlen1 logische pixels, Enter bevestigt, Escape/muisklik/
inputannulering herstelt. Geen appinput tijdens de modus, geen clickthrough,
geen resize, buffer-/epoch-/rechtenwijziging. Counteroverflow is atomair.
Zie [ADR-VELD-005](../../DECISIONS.md).

## Hostbewijs

Na de bestaande geisoleerde `veld-env/env.sh`:

```sh
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
cargo +1.90.0 fmt --manifest-path sdk/examples/window-session/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml --all-targets -- -D warnings
python3 -B tools/test_window_session.py
```

Alle exit0. Vijf nieuwe hosttests, vijf displays; volledige incremental- en
referencepixels, coarse/fine, grenzen/repeat, modifierexactheid, annulering,
geen clickthrough of stale releases, geen gedeeltelijke overflowmutatie.
18 runnertegenproeven; nieuwe proef eist alle shell- en apppixels van
de verplaatste body in alle acht fasen, beide schalen.
Log `files/veld-env/logs/veld-move-final-host.log`.

## Native uitgevoerd

Bevroren bron0d232c70, fingerprint v3
`2f1ade603fd2ad556ceec23dd6e99c8886e0d300a96624367fa88c853fa52caa`.
Na productcheckpoint, sequentieel uitgevoerd:

```sh
export NOS_BUILD_DIR="$PWD/build/veld-move-20261103/scale1"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 1 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
export NOS_BUILD_DIR="$PWD/build/veld-move-20261103/scale2"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context-hidpi"
python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 2 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

Daarna eigen normal1/normal2-buildmappen en dezelfde respectieve targetmappen:
`python3 -B -u tools/build.py --kernel-feature native-window-session` en
`python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi`.
De twee probe-builds en twee runners eindigden elk met exitstatus 0.
Bij 100%: 56 fasen, 412,07 s; runnerwandtijd 432,36 s.
Bij 200%: 56 fasen, 726,07 s; runnerwandtijd 764,34 s. Beide ruwe QEMU-exit33;
alle toepasselijke gates true, inclusief `keyboard_window_move_gate` en HC.
Beide normale builds exit0,2,84 s en2,52 s; compilatie, geen extra runtimeclaim.
56 volledige fasen per schaal. Iedere nieuwe fase vereist het hele
shellbeeld plus de echte volledige appbody op de verplaatste coordinaten.
Ctrl+C/tekst/F6 tijdens de modus mogen geen appkey leveren; Enter behoudt,
Escape en daarna Ordenen herstellen het volledige framebuffer exact.
Alle oorspronkelijke isolatie-/klembord-/HC-/herstel-/schijfgates behouden.

[100%-rapport](native-scale1.json), [200%-rapport](native-scale2.json) en
[loghashes](logs.sha256). De compacte rapporten behouden alle runnergegevens
behalve de omvangrijke QMP-tracelijst; `raw_report` bewaart pad/hash van het
volledige oorspronkelijke rapport, `qmp_trace_summary` aantal/hash van die lijst.
Het checksum-manifest gebruikt bestandsnamen relatief aan de log-artifactroot.
In de uitvoeromgeving is die root `$VELD_ENV/logs`; de ruwe logs zijn
sessie-artifacts, niet meegeleverd in de checkout. Met de geleverde logbundle:
`cd <log-artifactroot> && sha256sum -c <checkout>/docs/verification/veld-move-20261103/logs.sha256`.
Een schone checkout zonder die bundle kan de ruwe logs niet controleren.
Geen gastnetwerk, hostdisk of productiedata gebruikt.
Geen nieuwe CI-, fysieke of menselijke acceptatie.
Score **41 -> 41**; complete gebruikersomgeving blijft open.
