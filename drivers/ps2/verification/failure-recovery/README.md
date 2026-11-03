# DRV-PS2-FAILURE-RECOVERY — 3 november 2026

**Bewijsniveau:** uitgevoerde hosttests met gescripte i8042-antwoorden en
freestanding librarybuild. Geen nieuwe QEMU-, fysieke of menselijke proef.

## Bron en grens

- Basis: main `78ff7c03b015ad16fe665ae7df9fe239d078e08d`.
- Testbron: `5e344582da0d05b79b35bb9c5425034396315a67`.
- Branch: `copilot/driver-next-ps2`.
- Alleen `drivers/ps2/tests/controller.rs` en dit bewijsbestand gewijzigd.
  De productiecode, bestaande `Ports`/`Controller`/`Sample`-interfaces,
  kerneladapter en gebruikersinteractie zijn ongewijzigd.
- De test volgt de al geïmplementeerde resetvolgorde uit
  [`NATIVE-DISPLAY-INPUT.md`](../../../../docs/implementation/NATIVE-DISPLAY-INPUT.md#begrensde-ps2-reset).
  Dit is geen nieuwe driver, protocolvariant of autoriteitsmodel.

## Uitgevoerd

De normale resetfixture genereert aantoonbaar dertien antwoorden: controller-
mode, keyboard ACK/BAT en configuratie-ACK's, mouse ACK/BAT/ID/configuratie
en de laatste controller-modecontrole en scan-enable-ACK's. Op ieder antwoord
wordt afzonderlijk een gewijzigde byte, pariteitsfout of ontbrekend antwoord
geïnjecteerd: **13 × 3 = 39 fout-/herstelroutes** in één parametrische test.

Iedere route start met een geslaagde reset en controleert vervolgens:

1. De tweede reset bereikt het injectiepunt en faalt zonder `Ready`.
2. De fout blijft op hetzelfde ticket staan; lezen vóór herstel wordt geweigerd
   en beide operaties voeren geen nieuwe poort-I/O uit.
3. Een expliciete nieuwe reset krijgt een ander ticket; het oude ticket kan
   die reset niet voortzetten en input blijft geblokkeerd.
4. Eventueel achtergebleven FIFO-antwoorden worden precies als `discarded`
   geteld. Herstel publiceert epoch 2, correcte controller-mode, een lege FIFO
   en daarna ongewijzigde keyboard-/mousebytes met sequences 1 en 2.

De bestaande `finish`-helper controleert bij iedere poll maximaal zestien
poortoperaties en begrenst het aantal pollticks. Fouten tijdens reset leveren
geen extra succesvolle epoch op.

## Commando's en resultaten

Uitgevoerd vanuit de branchworktree met de bestaande omgeving:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export CARGO_TARGET_DIR="$PWD/build/driver-next"
```

Geen toolchain of systeempackage geïnstalleerd; geen externe dependency
toegevoegd.

| Commando | Exit | Resultaat |
|---|---:|---|
| `cargo +1.90.0 test --offline --locked --manifest-path drivers/ps2/Cargo.toml` | 0 | 1 unit- en 8 controllertests; 0 doctests. Inclusief alle 39 nieuwe routes |
| `cargo +1.90.0 fmt --manifest-path drivers/ps2/Cargo.toml -- --check` | 0 | Formattercheck |
| `cargo +1.90.0 clippy --offline --locked --manifest-path drivers/ps2/Cargo.toml --all-targets -- -D warnings` | 0 | Strict Clippy, inclusief testcode |
| `cargo +1.90.0 build --offline --locked --manifest-path drivers/ps2/Cargo.toml --release --target x86_64-unknown-none --lib` | 0 | Bestaande `no_std`-driverlibrary |
| `git diff --check` | 0 | Geen whitespacefouten |

| Log | SHA-256 |
|---|---|
| `/tmp/nieuw-os-ps2-failure-test.log` | `345814e8bb5b7de6dc303a180126b1b486eff4d81f27d4ec90738e97b445ed83` |
| `/tmp/nieuw-os-ps2-failure-fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `/tmp/nieuw-os-ps2-failure-clippy.log` | `b45a02057d58a51634f2dbb468b1822388d5a46adad6ada7c4b2068736eaf1fc` |
| `/tmp/nieuw-os-ps2-failure-none.log` | `a3c5b94617a2ce9f081ec3948a5145308e60be55ac4d789807ba157972792ec7` |

## Open grenzen

De fixture simuleert controller-/device-antwoorden, niet een werkelijke
poortadapter of fysieke controller. DMA-/IRQ-autoriteit is niet nodig voor
deze test. `DRIVER-DEVICE-EXECUTION-INTERFACE` blijft afzonderlijk nodig voor
NVMe-I/O en xHCI-enumeratie; hier wordt geen alternatief geïmplementeerd.
FR-H01/H02 of andere voortgangscriteria worden niet opnieuw geaccepteerd.
De hoofdeigenaar onderhoudt centrale status en score; deze bestanden zijn
hier niet gewijzigd.
