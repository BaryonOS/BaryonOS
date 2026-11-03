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
gewijzigde bron geslaagd, exit 0. Native run nog niet uitgevoerd op dit
checkpoint; de gevonden bestaande VGA-ROM wordt expliciet aan de runner
geleverd. Geen fysieke/humanacceptatie of scorepunten. Centrale beoordeling
blijft bij integrator.

Integratie: #166 bevat een apart fixture met dertien oorspronkelijke
device/modeantwoorden; dit pad voegt drie controllerantwoorden toe. Pas bij
integratie de injectietelling expliciet aan; oud testbewijs dekt deze bron niet.
Rollback van deze drie bestanden herstelt het oorspronkelijke resetpad.
