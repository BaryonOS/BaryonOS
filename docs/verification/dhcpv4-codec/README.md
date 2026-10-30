# DHCPv4-codec — hostverificatie, deelcheckpoint

Datum: 30 oktober 2026
Implementatiecommit: `d16b44a71e5e959f7a7a9a2530e25daf400dd139`
Branch: `simonvanieperen-musical-happiness`
Bronfingerprint v3: `6cfc48b7bfc81a770cec47c17986a4c32a6b4c2accffe50e62aee5bc12be52f0`

## Exacte bronbestanden

| Bestand | SHA-256 |
|---|---|
| `services/net-protocol/src/dhcp.rs` | `c4085f4387de0c26eaa641eda751948c3eb3e87bba041830688463021bf49d5c` |
| `services/net-protocol/tests/dhcp.rs` | `8238b8fca7f2f78af53d122cabc78ca9824574fbe68b8c193505898356af5e3f` |
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
| `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check` | 0 | Formattering schoon |
| `python3 -B tools/dev.py cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --test dhcp --locked --offline` | 0 | 9 DHCP-tests geslaagd |
| `git diff --check` | 0 | Geen whitespacefouten |
| `python3 -B tools/dev.py cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline` | 101 | Op `a3830f3` waren 2 library- en 7 ARP-tests groen; DHCP vond 1 regressie in de renewal-builder. Log: `/tmp/nieuw-os-dhcp-tests-localcc.log`. |

Een eerdere volledige crate-run op `a3830f3` vond terecht dat
`build_renewal_request` DHCPDISCOVER in plaats van DHCPREQUEST bouwde. De
builder gebruikt nu een expliciete requestsoort; bovenstaande 9/9-proef
controleert dit herstel. De volledige crate-testgroep, Clippy en de
freestanding targetbuild moeten nog opnieuw op `d16b44a` worden uitgevoerd.

`python3 -B tools/preflight.py --strict` eindigde met exit 2: deze worktree
mist `qemu-system-x86_64` en OVMF. Dit was alleen tool-inventarisatie; geen
QEMU-boot, DHCP-peer, native lease, routeconfiguratie of internetverbinding
getest. De codec is geen FR-N01-aftekening. N01–N04 blijven open en de vaste
score blijft 34%.
