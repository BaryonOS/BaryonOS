# Veld-04 — echte pointer-leave

Productbron `3799ec63b9be6a2a5dbe67bd2cb38af703f101a0`,
fingerprint v3 `af315f77251383dfcafca5f25f2d4fd4087371f675ef3ab1112659032b3ad234`.
Zelfstandige kernel/UEFI in QEMU; dezelfde bron afzonderlijk op beide schalen.

**Native resultaat:** 100%: 44 fasen, 344,97 s; 200%: 44 fasen,
609,74 s. Beide runners exit 0, ruwe QEMU-exit 33. Werkelijke body-exit na
de menuklik wist de hover, behoudt het geopende menu en document. Hele
appbeelden zonder maskers; de hostfixture vereist aantoonbaar verschillende
hoverpixels voor/na leave. Alle oude gereedschap-, venster-, klembord-,
isolatie-, crash/rebind-, bewaren/heropenen-, schijfcanary- en cleanupgates
slagen onverkort.

Na de bestaande geisoleerde `veld-env/env.sh`, sequentieel:

```sh
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
export NOS_BUILD_DIR="$PWD/build/veld-pointer-20261103/scale1"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 1 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
export NOS_BUILD_DIR="$PWD/build/veld-pointer-20261103/scale2"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context-hidpi"
python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 2 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

Alle vier opdrachten exit 0; runnerwandtijd inclusief hostorakels 358,38 en
633,42 s. Ook normale builds exit 0:
`python3 -B -u tools/build.py --kernel-feature native-window-session` en
`python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi`,
met buildmappen `normal1`/`normal2`, respectieve bovenstaande targetmappen.
Deze normale builds zijn compilatiebewijs, geen extra runtimeclaim.

Volledige hostcontrole, alle opdrachten exit 0:

```sh
for manifest in services/input-core/Cargo.toml sdk/input-client/Cargo.toml ui/native/Cargo.toml sdk/examples/materiaal/Cargo.toml sdk/examples/calculator/Cargo.toml sdk/examples/window-session/Cargo.toml sdk/examples/input-service/Cargo.toml; do
  cargo +1.90.0 fmt --manifest-path "$manifest"
  cargo +1.90.0 test --offline --locked --manifest-path "$manifest"
  cargo +1.90.0 clippy --offline --locked --manifest-path "$manifest" --all-targets -- -D warnings
done
python3 -B tools/test_window_session.py
cargo +1.90.0 fmt --manifest-path sdk/examples/image-viewer/Cargo.toml
cargo +1.90.0 fetch --locked --manifest-path sdk/examples/image-viewer/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/image-viewer/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/image-viewer/Cargo.toml --all-targets -- -D warnings
```

Runner16 tests, viewer15 tests. Alle bestaande Event-consumenten bijgewerkt;
viewer is hostbewijs, geen native derde app. De eerste offline viewertest
miste reeds gelockte `zune-core`; alleen bestaande packages opgehaald,
geen manifest/lockfile of afhankelijkheidsversie veranderd.
Negatieve tests weigeren niet-nulle leave-payload, verkeerde afzender,
correlatie/type, oude generatie/replay en klembordautoriteit. Leave
verandert de focusgeneratie niet; actieve drag blijft Resync gebruiken.

Eerste targetbuild op `3f72f2f3` stopte voor VM-start met E0282:
queue-index niet afleidbaar door de nieuwe eerdere matcharm. Expliciete
optionele usize-indextypen herstellen dit; echte targetbuild en daarna
beide volledige native gates herhaald. Eerdere hostfixture-/lintfouten
(assertietraits, generational handles, importpad, clone-on-copy en
invoegpositie) hersteld zonder productvalidatie te versoepelen.

[100%-rapport](native-scale1.json), [200%-rapport](native-scale2.json) en
[loghashes](logs.sha256) binden bron, firmware, pixels, fasen en schijf.
Geen gastnetwerk, hostdisk of productiedata gebruikt.

**Grenzen:** INP1-type4 wordt alleen aan de twee gezamenlijk gebouwde
native apps geleverd; oudere derdenapps vereisen protocolonderhandeling.
Geen nieuwe CI-uitvoering, fysieke hardwaretest, menselijke gebruikerstest,
schermlezertest of releasevalidatie uitgevoerd. Hoogcontrast en de overige
complete-omgevingsfuncties blijven uitvoering. Score **41 -> 41**.
