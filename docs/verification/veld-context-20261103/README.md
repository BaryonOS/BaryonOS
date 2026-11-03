# Veld-03 — echt documentcontextmenu

Productbron `9b1c82000d03885e9938c62eb2533fb207f04eab`,
fingerprint v3 `be9f05ecc64049506701670833d1d325bd2eb9fb1830c549cf693bac248f093d`.
Eigen kernel/UEFI in QEMU; dezelfde bron afzonderlijk op beide schalen.

**Native resultaat:** 100%: 44 fasen, 330,02 s; 200%: 44 fasen,
600,60 s. Beide runners exit 0, ruwe QEMU-exit 33. Shift+F10,
modale invoerweigering, pijlnavigatie, echte Lezen-keuze, geweigerde
mutaties, annuleren en terug naar Schrijven; daarna zichtbare muisknop en
Escape. Hele apppixels, geen menu- of hovermaskers. Alle oorspronkelijke
klembord-, procesisolatie-, crash/rebind-, bewaren/heropenen-,
schijfcanary- en cleanupgates slagen onverkort.

Na de bestaande geisoleerde `veld-env/env.sh`, sequentieel:

```sh
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
export NOS_BUILD_DIR="$PWD/build/veld-context-20261103/scale1"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 1 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
export NOS_BUILD_DIR="$PWD/build/veld-context-20261103/scale2"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context-hidpi"
python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 2 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

Alle vier opdrachten exit 0. Runnerwandtijden inclusief hostorakels:
344,46 en 626,18 s. Beide normale profielen ook gebouwd met exit 0:
`python3 -B -u tools/build.py --kernel-feature native-window-session`
en `python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi`,
met buildmappen `normal1`/`normal2` en dezelfde respectieve targetmappen.
Dit laatste is compilatiebewijs, geen extra runtimeclaim.

Hostcommando's, alle exit 0:

- `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml`
- `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml`
- `cargo +1.90.0 clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings`
- `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --all-targets -- -D warnings`
- `cargo +1.90.0 fmt --manifest-path ui/native/Cargo.toml`
- `cargo +1.90.0 fmt --manifest-path sdk/examples/materiaal/Cargo.toml -- --check`
- `python3 -B tools/test_window_session.py` — 16 tests

Twee nieuwe fixturefouten zijn voor deze runs hersteld: de toetsvariant
heet `Down`, niet `ArrowDown`; na FocusLost vereist verse muisinvoer een
nieuwe generatie. Geen productievalidatie versoepeld. De volledige hosttests
slagen zonder overgeslagen rastertest.

[100%-rapport](native-scale1.json), [200%-rapport](native-scale2.json) en
[loghashes](logs.sha256) binden de echte bron, pixels, fasen en schijf.
Geen gastnetwerk, hostdisk of productiedata gebruikt.

**Open grens:** hover blijft na pointerverlaten zonder actieve grab staan.
De muisorakels behouden de werkelijk achtergebleven knophover; dit is geen
oplossing voor pointer-leave. Geen nieuwe CI-uitvoering, fysieke hardwaretest,
menselijke gebruikerstest, schermlezertest of releasevalidatie uitgevoerd.
De gehele Veld-omgeving blijft in uitvoering. Score **41 -> 41**.
