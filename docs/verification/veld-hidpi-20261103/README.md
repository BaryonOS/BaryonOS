# Veld-03 — native 200%-bediening

Productbron `48655bce7ccf2ac707ee77de8014ebc4029c1b89`,
fingerprint v3 `fccedfe2712dc3785798145b0acee1710fba485775115f0f321c7239ac9fd72e`.
Eigen kernel/UEFI in QEMU, 1920x1080, schaal 2. Geen omgelabeld 100%-bewijs.

**Resultaat:** build exit 0; volledige native gate exit 0, 36 fasen,
469,36 s, ruwe QEMU-exit 33. Volledige apppixels voor Schrijven/Lezen,
geweigerde bewerking, bewaren/heropenen; echte venstermenu-, minimaliseer-
en herstelbediening. Alle oorspronkelijke klembord-, procesisolatie-,
crash/rebind-, schijfcanary- en cleanupgates behouden en geslaagd.

Na de bestaande geisoleerde `veld-env/env.sh`:

```sh
export NOS_BUILD_DIR="$PWD/build/veld-hidpi-20261103/probe"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-hidpi"
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 2 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

Beide opdrachten exit 0; wandtijd build 1,84 s, runner 493,07 s inclusief
hostorakels. QEMU10.2.1, `pc-q35-10.0`, TCG single-thread, geen gastnetwerk,
uitsluitend synthetische wegwerpdisk. Het [rapport](native-report.json)
bindt bron, fasen, pixels en disk; [loghashes](logs.sha256) binden de uitvoer.

Eerste poging op `a95a8b97` bouwde wel, maar de runner stopte voor VM-start
met exit 1: `%` in argparse-help onder Python3.14. Helptekst hersteld,
echte CLI-helptest toegevoegd; bovenstaande volledige herhaling slaagt.
Die eerste poging blijft apart als mislukt vastgelegd.

**Grenzen:** deze VM-test bewijst geen fysieke hardwareondersteuning,
menselijke bruikbaarheid, schermlezertoegang, CI-uitvoering of release.
Het documentcontextmenu is nog niet geintegreerd of native getest.
De gehele Veld-omgeving blijft in uitvoering. Score **41 -> 41**:
geen volledig nieuw criterium aanvaard.
