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

## Reviewherstel: aangeboden adres, basis 6a20b35

De reviewbevinding over ontbrekende `yiaddr`-controle is tegen de code
bevestigd. `parse_reply` weigert nu een niet-unicast `yiaddr` bij OFFER/ACK
met `InvalidAddress`. NAK met `yiaddr=0.0.0.0` blijft toegestaan. De nieuwe
test controleert nul-, loopback-, multicast- en broadcastadressen voor beide
berichttypen en apart het nuladres bij NAK.

Alle onderstaande commando's zijn uitgevoerd op uitsluitend deze codec-diff,
zonder de apart bewaarde DHCP-client-WIP:

| Commando | Exit | Uitkomst |
|---|---:|---|
| `python3 -B tools/dev.py cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline` | 0 | 57 tests, waarvan 10 DHCP |
| `python3 -B tools/dev.py cargo +1.90.0 clippy --manifest-path services/net-protocol/Cargo.toml --all-targets --locked --offline -- -D warnings` | 0 | Geen waarschuwingen |
| `python3 -B tools/dev.py cargo +1.90.0 build --manifest-path services/net-protocol/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline` | 0 | Freestanding build |
| `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check` | 0 | Schoon |
| `git diff --check` | 0 | Schoon |

Zelfde lokale GCC-sysroot als boven; hosttargetmap `target/dhcp-hostcc`,
lint-/targetmap `target/dhcp-codec`. Logs:
`/tmp/nieuw-os-dhcp-yiaddr-{tests,clippy,target}.log`. De echte native pair op
`9ddc6b6` is afzonderlijk vastgelegd onder
[`../full-release-network-dev-9ddc6b6/`](../full-release-network-dev-9ddc6b6/README.md);
die run wordt niet als native test van de latere reviewfix voorgesteld.

## Integratiereview: minimale BOOTP-lengte, basis 1bb8acf

RFC 1542, W. Wimer, oktober 1993, §2.1
(<https://www.rfc-editor.org/rfc/rfc1542.txt>) noemt 300 octets UDP-data als
minimale BOOTP-lengte voor de consistentiecontrole. RFC 2131 §2 behoudt
BOOTP-relaygebruik; variabele DHCP-opties zijn geen reden om kortere
uitgaande berichten als algemeen interoperabel te behandelen. De builder
gaf inderdaad 266/278/266 bytes terug. Nu vult dezelfde centrale builder
DISCOVER, selecting REQUEST en renewal REQUEST tot minimaal 300 bytes aan
met nulbytes na END, zonder de options of parsergrenzen te verruimen.

De nieuwe regressie faalde eerst op de ongewijzigde builder: `Ok(266)` waar
`OutputTooSmall` voor een 299-bytebuffer werd verwacht, exit 101
(`padding-red.txt`, 0,45 s). Na herstel controleert hij alle drie builders:
299 bytes faalt zonder enige mutatie; exact 300 bytes slaagt; END blijft op
de exacte optiegrens; padding is nul; bytes vanaf offset 300 blijven intact
bij een grotere buffer.

| Getest bronbestand | SHA-256 |
|---|---|
| `services/net-protocol/src/dhcp.rs` | `c51ad2d93e317cd36513668ddba67e13653e57ccb4e67a7e18c240d5b92d681b` |
| `services/net-protocol/tests/dhcp.rs` | `f8a11b61fba4c9bbcf095f3c5cc2fb8581572c81639c7cf00b4c852b8b9b4bf2` |

Op deze geïsoleerde codec-diff, zonder clientcore, met bovenstaande lokale
sysroot en `CARGO_TARGET_DIR=target/dhcp-hostcc`:

| Exact commando | Exit | Uitkomst / duurzaam log |
|---|---:|---|
| `python3 -B tools/dev.py cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --test dhcp --locked --offline all_requests_pad_to_300_bytes_without_touching_the_tail` | 101 | Verwachte rode reproductie vóór fix; `padding-red.txt` |
| `python3 -B tools/dev.py cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline` | 0 | 58/58, waarvan 11 DHCP; `padding-tests.txt`, 0,49 s |
| `python3 -B tools/dev.py cargo +1.90.0 clippy --manifest-path services/net-protocol/Cargo.toml --all-targets --locked --offline -- -D warnings` | 0 | `padding-clippy.txt`, 0,37 s |
| `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check` | 0 | `padding-fmt.txt` |
| `python3 -B tools/dev.py cargo +1.90.0 build --manifest-path services/net-protocol/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline` | 0 | `padding-none.txt`, 0,30 s |
| `git diff --check` | 0 | Schoon |

Geen native DHCP/relay getest of FR-N01-acceptatie geclaimd. Netwerk blijft
0/10; totaal 34/100. Centrale status/assessment zijn nu van teamleider
`89f84098`; deze brongebonden reviewfix levert geen nieuwe punten op.
