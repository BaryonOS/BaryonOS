# Veld-07 — echt systeemmeldingenpaneel

**Hostgeteste tussenstand; native open.** Basis
`ea0cd6d19235c5f8575f21d527e5bcc8526056ee`, branch
`simonvanieperen-veld-hc-20261103`. Alleen shell-/fixture-/runner-/CI-writes;
General-/Settingsbestanden blijven ongemoeid.

Het paneel toont de huidige echte compositor-klembordstatus, geen
verzonnen appmelding. Alt+N en zichtbare bovenbalkactie; focus,
Enter/Delete/Escape en echte dismiss-/sluitknoppen. Dismiss wist uitsluitend
presentatie. Exclusieve overlayinput, geen clickthrough/stale releases.
Nieuwe statusovergang kan opnieuw verschijnen; dezelfde statuspoll niet.
Geen algemene appnotificatie-API, historie of persistentie.
[ADR-VELD-006](../../DECISIONS.md).

## Hostbewijs

Na de bestaande geisoleerde `veld-env/env.sh`:

```sh
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
cargo +1.90.0 fmt --manifest-path sdk/examples/window-session/Cargo.toml -- --check
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml --all-targets -- -D warnings
python3 -B tools/test_window_session.py
```

Alle exit0. Drie nieuwe hostproeven, vijf displays; alle echte statusrollen,
hele incremental/referencepixels, keyboardfocus en muisknoppen, modifiergrenzen,
inputannulering zonder clickthrough/stale releases. Runner19 tegenproeven,
inclusief het hele ondoorzichtige paneelinterieur op beide schalen.
Log `files/veld-env/logs/veld-notice-final-host.log`.

## Native nog niet uitgevoerd

Na checkpoint, sequentieel:

```sh
export NOS_BUILD_DIR="$PWD/build/veld-notice-20261103/scale1"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 1 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
export NOS_BUILD_DIR="$PWD/build/veld-notice-20261103/scale2"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context-hidpi"
python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 2 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

Daarna normal1/normal2-buildmappen en dezelfde respectieve targetmappen:
`python3 -B -u tools/build.py --kernel-feature native-window-session` en
`python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi`.
Verwacht60 fasen per schaal; vier extra fasen: echte gekopieerde melding,
dismiss/lege toestand, muisopenen en muissluiten. Het hele ondoorzichtige
paneelinterieur en de volledige bovenbalk na sluiten worden vergeleken.
De oorspronkelijke hele appbodyorakels blijven ongemaskerd; de latere echte
numerieke paste moet clipboarddata na dismiss behouden.
Ook tekst/F6/clear-chord tijdens modaliteit mag geen appinput of clear geven.
Alle oorspronkelijke isolatie-/HC-/venster-/herstel-/schijfgates behouden.

Nog geen native paneel-, nieuwe CI-, fysieke of menselijke acceptatie.
Score **41 -> 41**; complete gebruikersomgeving blijft open.
