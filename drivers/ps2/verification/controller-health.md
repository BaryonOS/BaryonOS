# DRV-PS2-CONTROLLER-HEALTH

Basis `6d523964ab85638d4dd981bcdb19580d18646fab`, branch
`copilot/driver-ps2-time-boundaries`, 3 november 2026.

De bestaande resetconsument controleert nu eerst controllerzelftest
AA->55 en beide poorttests AB/A9->00. Foute/ontbrekende/pariteitsantwoorden
verhinderen alle devicecommands en Ready. De voor zelftest gelezen modebits
blijven behouden; na zelftest worden beide poorten opnieuw uitgezet en wordt
restinput begrensd gedraind, omdat zelftest controllerstate kan veranderen.
Het profiel blijft dezelfde dual-port QEMU i8042; geen single-portfallback,
USB-/IRQ-/DMA-interface, rechten, kerneladapter of gebruikersinteractie toegevoegd.
Ticket, epoch, samplebytes, deadlines en per-call-I/O-budget blijven gelijk.

Bronnen, gelezen 3 november 2026:

- QEMU Project, tag v10.2.1, `hw/input/pckbd.c`, `kbd_write_command`:
  AA geeft 55, AB en A9 geven 00; alle drie via controller/non-AUX-antwoord.
  https://github.com/qemu/qemu/blob/v10.2.1/hw/input/pckbd.c
- OSDev: I8042 PS/2 Controller, W003160, lokale snapshot; stappen controller-
  en poorttest, met waarschuwing dat zelftest state kan resetten.
  Dit is een communityroute, geen bewijs voor fysieke hardware.

Geen upstreamcode overgenomen. Alleen `src/lib.rs`, `tests/controller.rs`
en dit bewijsbestand wijzigen. Oud queued PS2-/RTC-/USB-/TPM-/ACPI-werk is niet
overgenomen of gewijzigd.

## Uitgevoerd

```sh
export PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export CARGO_TARGET_DIR="$PWD/build/ps2-health"
cargo +1.90.0 test --offline --locked --manifest-path drivers/ps2/Cargo.toml
cargo +1.90.0 fmt --manifest-path drivers/ps2/Cargo.toml -- --check
cargo +1.90.0 clippy --offline --locked --manifest-path drivers/ps2/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --manifest-path drivers/ps2/Cargo.toml --release --target x86_64-unknown-none --lib
git diff --check
```

Definitief alle exit 0: 1 unit- en 10 controllertests, 0 doctests.
Nieuwe tests weigeren alle 765 ongeldige healthantwoordwaarden, zes
ontbrekende/pariteitsgevallen plus herstel met nieuw ticket, en zelftest die
mode reset met behoud van opgeslagen bits/IRQ-off/nieuwe raw sample.
Eerste hostrun faalde op een oude fixture die na een vast aantal polls
injecteerde; vervangen door injectie op de daadwerkelijke deviceantwoordactie,
niet door productiecontrole te versoepelen. Eerste strict Clippy weigerde
een OR-pattern; hersteld naar range en alle gates herhaald.

Logs onder `/tmp/nieuw-os-ps2-health-`, SHA-256:

| Suffix | SHA-256 |
|---|---|
| `test.log` | `adced8286c2badf30499a11440e522aa7315c992b2e313385e9fd9c4fa493cf0` |
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `clippy.log` | `4a404fe71a821a65670df6440675909a9b9c0e89da724101fa0f7fd5fc0d5fd5` |
| `none.log` | `d6d692ad44185969848b3d1759af0eac908128df53611e01b77a7db55a604282` |

`python3 -B -u tools/build.py --kernel-feature native-display-probe` is op de
gewijzigde bron geslaagd, exit 0, met onderstaande native omgeving. Daarna:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/sm-oom-qemu-root/usr/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export LD_LIBRARY_PATH=/tmp/sm-oom-qemu-root/usr/lib/x86_64-linux-gnu
export OS_OVMF_CODE=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_CODE_4M.fd
export OS_OVMF_VARS=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_VARS_4M.fd
export NOS_QEMU_MACHINE=pc-q35-8.2
export NOS_BUILD_DIR="$PWD/build/ps2-health-native"
export NOS_TARGET_DIR="$PWD/target/ps2-health-native"
export CARGO_TARGET_DIR="$PWD/target/ps2-health-host"
export CARGO_NET_OFFLINE=true
python3 -B -u tools/run_display_probe.py --machine pc-q35-8.2 --qmp-transport pipe --vga-rom '/mnt/c/Program Files/qemu/share/vgabios-stdvga.bin'
```

Exit 0, werkelijk QEMU raw 33, passed true, 2.6429 s. Beide resets worden
Ready met nieuwe epochs/tickets, 16 echte samples verdeeld over twee
levensduren, alle input/isolation/cleanup/pixelcontroles groen. Zes processen
beindigd, nul overgebleven; drie slotreuses. Het harnas levert QMP-invoer en
observeert pixels, niet de eigen driverlogica of succesmarkers.
Gecompileerde/uitgevoerde bron `76369c7c9598cc3d515427bad25a2a710e976651`,
fingerprint v3 `47ccf9da3c02084f783e14ed1830ffb5261666c1064ab25310062e89ccc63933`.
Alle source/artifact/transcript/QMP/pixelbindings true.
Geen nieuwe failure-injectie in QEMU; fouten hierboven blijven hostbewijs.

| Bestand | SHA-256 |
|---|---|
| `/tmp/nieuw-os-ps2-health-native-build.log` | `e2b33a057f3c35cd4455a1594ee6d0d249f1ea042f277186c8616e93d79b3bc9` |
| `/tmp/nieuw-os-ps2-health-native-run.log` | `82709a31bd94d500797011a88b67866f69e8ece705e1d3334d7bea7db78c45d0` |
| `build/ps2-health-native/run/display-input-7b29103c6d/result.json` | `3f641c7ef7070e4cc9aadb82f5b74ad8c44fbeec265400ac61e5fca698729b55` |

Geen fysieke/humanacceptatie of scorepunten. Centrale beoordeling bij integrator.

Integratie: #166 bevat een apart fixture met dertien oorspronkelijke
device/modeantwoorden; dit pad voegt drie controllerantwoorden toe. Pas bij
integratie de injectietelling expliciet aan; oud testbewijs dekt deze bron niet.
Rollback van deze drie bestanden herstelt het oorspronkelijke resetpad.
