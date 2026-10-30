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
| `python3 -B tools/dev.py cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --test dhcp --locked --offline` | 0 | Op `30748d6`: 9 DHCP-tests geslaagd; log: `/tmp/nieuw-os-dhcp-307-targeted.log` |
| `git diff --check` | 0 | Op `30748d6` geen whitespacefouten |
| `python3 -B tools/dev.py cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline` | 0 | Op `30748d6`: 56 tests geslaagd; log: `/tmp/nieuw-os-dhcp-307-tests.log` |
| `env CARGO_TARGET_DIR=target/dhcp-codec python3 -B tools/dev.py cargo +1.90.0 clippy --manifest-path services/net-protocol/Cargo.toml --all-targets --locked --offline -- -D warnings` | 0 | Op `30748d6`: geen waarschuwingen; log: `/tmp/nieuw-os-dhcp-307-clippy.log` |
| `env CARGO_TARGET_DIR=target/dhcp-codec python3 -B tools/dev.py cargo +1.90.0 build --manifest-path services/net-protocol/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline` | 0 | Op `30748d6`: freestanding releasebuild geslaagd; log: `/tmp/nieuw-os-dhcp-307-target.log` |

Een eerdere volledige crate-run op `a3830f3` vond terecht dat
`build_renewal_request` DHCPDISCOVER in plaats van DHCPREQUEST bouwde. De
builder gebruikt nu een expliciete requestsoort; de volledige run op
`30748d6` slaagde 56/56 en de DHCP-groep controleert ook dat selecting
DHCPREQUEST broadcast gebruikt. Clippy en de freestanding targetbuild slaagden
eveneens op die exacte commit; er is nog geen native netwerkbewijs.

`python3 -B tools/preflight.py --strict` eindigde met exit 2: deze worktree
mist `qemu-system-x86_64` en OVMF. Dit was alleen tool-inventarisatie; geen
QEMU-boot, DHCP-peer, native lease, routeconfiguratie of internetverbinding
getest. De codec is geen FR-N01-aftekening. N01–N04 blijven open en de vaste
score blijft 34%.
