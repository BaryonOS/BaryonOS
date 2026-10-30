# DHCPv4-codec — hostverificatie, deelcheckpoint

Datum: 30 oktober 2026
Implementatiecommit: `30748d6402ee9bf84a5606a8260455b0c05d9e9a`
Branch: `simonvanieperen-musical-happiness`
Bronfingerprint v3: `f5c4968216a35c4fab9f85d3c9c34729c5e667cf5a0066561f067ba5c452789c`

## Exacte bronbestanden

| Bestand | SHA-256 |
|---|---|
| `services/net-protocol/src/dhcp.rs` | `8de7696583921428b6bfe7daf2d6bb5211387c0becce04b5e948ab473f6fa493` |
| `services/net-protocol/tests/dhcp.rs` | `5389b6565a801e790440fdbef68b43fe19609b262b9771470a107ff6dc9344bf` |
| `services/net-protocol/src/lib.rs` | `9c3e877322677f0a6ec14716e01f8530340c75390708d806bb71844dc490333c` |

## Hosttests en omgeving

De projecttoolomgeving had geen `cc`. Zonder omgevingswijziging installeerde
ik niets globaal: GCC 13.4.0 en de benodigde Ubuntu 26.04 `libc`/GCC-pakketten
zijn alleen naar `/tmp/nieuw-os-dhcp-hostcc/` gedownload en daar uitgepakt.
De C-linkerprobe slaagde en de DHCP-testgroep is uitgevoerd met Rust 1.90.0.
Deze tijdelijke toolchain is geen Nieuw-OS-runtimeafhankelijkheid.

Met `LD_LIBRARY_PATH`, `GCC_EXEC_PREFIX`, `COMPILER_PATH`, `LIBRARY_PATH`,
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER`, `RUSTFLAGS` en
`CARGO_TARGET_DIR=target/dhcp-hostcc` ingesteld op die tijdelijke sysroot:

| Commando | Exit | Resultaat |
|---|---:|---|
| `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check` | 0 | Op `30748d6` formattering schoon |
| `python3 -B tools/dev.py cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --test dhcp --locked --offline` | 0 | Op `30748d6`: 9 DHCP-tests geslaagd; log is de tooluitvoer van deze sessie |
| `git diff --check` | 0 | Op `30748d6` geen whitespacefouten |
| `python3 -B tools/dev.py cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline` | 0 | Op voorganger `d16b44a`: 56 tests geslaagd; log: `/tmp/nieuw-os-dhcp-tests-final.log` |
| `env CARGO_TARGET_DIR=target/dhcp-codec python3 -B tools/dev.py cargo +1.90.0 clippy --manifest-path services/net-protocol/Cargo.toml --all-targets --locked --offline -- -D warnings` | 101 | Op voorganger `d16b44a` meldde Clippy `too_many_arguments` (8/7) in `build_request`; commit `30748d6` verwijderde de losse broadcast-parameter. Clippy moet op de nieuwe bron worden herhaald; log: `/tmp/nieuw-os-dhcp-clippy-final.log` |

Een eerdere volledige crate-run op `a3830f3` vond terecht dat
`build_renewal_request` DHCPDISCOVER in plaats van DHCPREQUEST bouwde. De
builder gebruikt nu een expliciete requestsoort; de volledige run op
`d16b44a` slaagde 56/56 en de 9/9-proef op `30748d6` controleert ook dat
selecting DHCPREQUEST broadcast gebruikt. De volledige crate-testgroep, Clippy
en de freestanding targetbuild moeten nog op `30748d6` worden uitgevoerd.

`python3 -B tools/preflight.py --strict` eindigde met exit 2: deze worktree
mist `qemu-system-x86_64` en OVMF. Dit was alleen tool-inventarisatie; geen
QEMU-boot, DHCP-peer, native lease, routeconfiguratie of internetverbinding
getest. De codec is geen FR-N01-aftekening. N01–N04 blijven open en de vaste
score blijft 34%.
