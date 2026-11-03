# Veld-02 — venstertoestanden en echt venstermenu

Productbron `0c66173bed205b2d2b04368e39d4a59b9f5a2b9c`,
fingerprint v3 `78d83b650f7796832e313aed23ccdd748b8b569e47c5099f1bba0b94322422bc`.
Baseline264ad5b0; geen resultaten van andere branches als eigen bewijs.

**Native resultaat:** exit 0, 36 fasen, 277,54 s, ruwe QEMU-exit33.
Echte Alt+Space-/pijlbediening en menupixels, muisminimaliseren,
Alt+F9, volledige lege shell bij twee geminimaliseerde vensters, geweigerde
X-invoer aan beide apps en werkbalkherstel van het volledige echte documentbeeld.
Alle vorige documentgereedschap-, klembord-, isolatie-, verplaatsings-,
echte compositorcrash/rebind-, bewaren/heropenen-, disk- en cleanupgates slagen.
Geen extra apppixelmaskers; menu-orakels betreffen benoemde shellgebieden,
nul-appbuffers blijven uitdrukkelijk geen appbewijs.

Na de bestaande geïsoleerde `veld-env/env.sh`:

```sh
export NOS_BUILD_DIR="$PWD/build/veld-window-20261103/probe"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103"
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

Beide Python-opdrachten exit0; build2,58 s, runner288,40 s inclusief fixtures.
QEMU10.2.1/`pc-q35-10.0`, TCG single-thread, eigen kernel/UEFI, geen gastnetwerk,
uitsluitend synthetische wegwerpdisk. [Rapport](native-report.json) bindt
bron, fasen, firmware, apporakels, disk en volledig lokaal rapport;
[loghashes](logs.sha256) binden de volledige lokale commando-uitvoer.

Hostcommando's, alle exit0:

- `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml` — 61 tests
- `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml --all-targets -- -D warnings`
- `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml`
- `cargo +1.90.0 fmt --manifest-path sdk/examples/window-session/Cargo.toml -- --check`
- `cargo +1.90.0 fmt --manifest-path ui/native/Cargo.toml -- --check`
- `python3 -B tools/test_window_session.py` — 13 tests

De eerste integratiefmt vond een verkeerd ingevoegde matchpositie; hersteld
vóór bouw. Geen mislukte native poging voor deze bron.

**Grenzen:** Normal/Minimized, geen resize/maximaliseren of procesafsluiting.
Verbergen behoudt dezelfde appbuffers, epochs, documenteigenaar en rechten;
nieuwe overlays/verborgen focus annuleren oude INP1-invoer en klembordtickets.
100% native; 200% en kleine viewport voor deze stap alleen exacte hostpixels.
Geen nieuwe CI-, fysieke, menselijke, schermlezer- of releaseacceptatie.
Score41 blijft41. Documentcontextmenu, native200%-gate en overige Veld-functies
blijven uitvoering, niet vervangen door documentatie.
