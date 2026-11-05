# DRV-RTC-IO-FAILURE — 3 november 2026

Basis main `78ff7c03b015ad16fe665ae7df9fe239d078e08d`; testbron
`cc0b28103ed8357ed111f370647ab11c4c4fae54`; branch
`copilot/driver-next-rtc`. Alleen tests en dit bewijsbestand zijn gewijzigd.
Productiecode, kerneladapter, NMI-beleid en tijdautoriteit zijn ongewijzigd.

## Uitgevoerd hostbewijs

De bestaande stabiele RTC-read doet twee snapshots van dertien registers,
elk via select/settle/read: 78 operaties, gevolgd door indexherstel in twee
operaties. De nieuwe fixture injecteert expliciete I/O-fouten op die bestaande
interface, zonder hostklok of hardware-I/O.

- Alle 78 snapshotoperaties met zowel toegestane als gemaskeerde NMI:
  156 routes. Na de fout wordt de oorspronkelijke index/NMI-selectie hersteld,
  zonder register C (interruptacknowledgement) te lezen. Een foutieve toekomstige
  read verhoogt de eerder vastgelegde tijdvloer niet; herstel naar dezelfde
  eerdere datum slaagt, een oudere datum blijft `Rollback`.
- Iedere snapshotfout gecombineerd met select- of settle-failure bij herstel:
  312 routes. Beide fouten blijven expliciet beschikbaar; de driver raakt
  `Poisoned` en herhaalde reads doen aantoonbaar geen enkele I/O-operatie.
- Beide hersteloperaties na volledig geslaagde snapshots, onder beide
  NMI-beleiden: 4 routes. Er wordt geen tijdwaarde gepubliceerd wanneer
  indexherstel faalt; ook daarna blijft I/O geblokkeerd.

**Totaal: 472 foutscenario's in drie parametrische tests.** De werkelijk
uitgevoerde suite bevat 1 unit- en 13 clocktests, 0 doctests. Dit is bewijs van
het gescripte drivergedrag, niet van fysieke CMOS-hardware, NMI-concurrency,
een werkende kerneladapter of geauthenticeerde TLS-/certificaattijd.
`Quality::UnauthenticatedRtc` blijft verplicht.

## Commando's

Omgeving: bestaande toolchain, geen installatie of nieuwe dependency:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export CARGO_TARGET_DIR="$PWD/build/driver-rtc"
```

| Commando | Exit/resultaat |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path drivers/rtc/Cargo.toml` | 0; 14 tests geslaagd |
| `cargo +1.90.0 fmt --manifest-path drivers/rtc/Cargo.toml -- --check` | 0 |
| `cargo +1.90.0 clippy --offline --locked --manifest-path drivers/rtc/Cargo.toml --all-targets -- -D warnings` | 0 |
| `cargo +1.90.0 build --offline --locked --manifest-path drivers/rtc/Cargo.toml --release --target x86_64-unknown-none --lib` | 0 |
| `git diff --check` | 0 |

| Log | SHA-256 |
|---|---|
| `/tmp/nieuw-os-rtc-io-test.log` | `f305d37beb607b530b76b5cf9497906fbdfa21ad42dadd7d0e3791791e95c2f1` |
| `/tmp/nieuw-os-rtc-io-fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `/tmp/nieuw-os-rtc-io-clippy.log` | `1cbd813410500d6a5b2152c28d1f399376d2485c4a97c566278c5abdb1c16b8f` |
| `/tmp/nieuw-os-rtc-io-none.log` | `179ef24f73b9149d52f6ed145fd53b866d7fcdd38fa859e869514e9d767f2f23` |

Geen QEMU-/fysieke/human gate uitgevoerd of voortgangscriterium gesloten.
Centrale status/assessment blijft bij de hoofdeigenaar. De afzonderlijke
device-executiondependency voor NVMe/xHCI is hier niet geconsumeerd of vervangen.
