# NIC-MAC via `NetworkSource`

**Status:** hosttests, formatter/Clippy en beide freestanding profielbuilds
slagen op codecommit `1eeb5af8fff255eaef3714c43a0a933dd0027176`. De wijziging
is gepusht op `copilot/universal-driver-framework`. PR #153 stond bij de
laatste controle open met `CHANGES_REQUESTED`; CI was nog deels bezig.

## Resultaat en bewijsgrens

Syscall `0x74` levert precies zes MAC-bytes van de gematchte netwerkdriver,
maar alleen aan de eigenaar van dezelfde `NetworkSource`. De kernel controleert
de capability, de volledige schrijfbare outputrange en de exacte lengte vóór
de kopie. De ring-3-proef bouwt Ethernetframes met deze waarde en controleert
de ingestelde QEMU-fixture; de afzonderlijke netwerkdienst gebruikt de
driverwaarde dynamisch.

Dit is een compileer- en hosttestresultaat, geen runtimeproef. `qemu-system-x86_64`
ontbreekt lokaal. De MAC-query, het gedrag met echte packets en fysieke
hardware zijn dus niet uitgevoerd/getest. Het adres is geen geauthenticeerde
peeridentiteit. De score blijft **41/100 → 41/100**.

## Host- en kwaliteitspoorten

Hosttests gebruikten de reeds aanwezige tijdelijke compiler
`/tmp/nieuw-os-hostcc-2026-11-03/bin/cc`. Rust 1.91.1 is de versie die de
netwerkdienst-workspaces vereisen; de ontbrekende toolchain en componenten
werden voor deze controles beschikbaar gemaakt. Er zijn geen repository-
dependencies gewijzigd.

| Controle | Exact commando | Exit | Duur | Log | SHA-256 |
|---|---|---:|---:|---|---|
| VirtIO-net hosttests | `cargo test --locked -p virtio-net-driver` | 0 | 126 ms | `/tmp/nieuw-os-nic-mac-2026-11-03/virtio-net-tests.log` | `985bd6c847bd27f31f855919791f4353d91345aa15ff53b3163f8cf419172f42` |
| SDK-runtime hosttests | `cargo test --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | 0 | 330 ms | `/tmp/nieuw-os-nic-mac-2026-11-03/sdk-runtime-tests.log` | `89be6d85651b336b6ebde85ca1b8b17390c8484faff85408554ca00f81fa5cfc` |
| Netwerkdienst-hosttests | `cargo +1.91.1 test --locked --manifest-path services/network-service/Cargo.toml` | 0 | 1.567 s | `/tmp/nieuw-os-nic-mac-2026-11-03/network-service-tests.log` | `2e3d4450525a6482458ae144de5b5d7732139f0b7a42d872ee19e78fd9b2aa58` |
| Net-stack-hosttests | `cargo +1.91.1 test --locked --manifest-path services/net-stack/Cargo.toml` | 0 | 1.482 s | `/tmp/nieuw-os-nic-mac-2026-11-03/net-stack-tests.log` | `cd4972c74f557c509af12c1deca7b0249de7bfcee89f6e0b698ce47d23ee4505` |
| Python-netwerkfixtures | `python3 -B -m unittest discover -s tests -p 'test_net_*.py'` | 0 | 100 ms | `/tmp/nieuw-os-nic-mac-2026-11-03/python-network-tests.log` | `b45cdc1a639291fb5ed5a9855bbc7652459ddd7595d2df36572d5495ffc63735` |
| Root-format + VirtIO Clippy | `cargo fmt --all -- --check && cargo clippy --locked -p virtio-net-driver --all-targets -- -D warnings` | 0 | 651 ms | `/tmp/nieuw-os-nic-mac-2026-11-03/root-quality.log` | `02db5a66fa0c60ca8618bd08aac556eac4f3ec863e8630a0d353dba6a0f6e126` |
| SDK-format + runtime Clippy | `cargo fmt --manifest-path sdk/Cargo.toml --all -- --check && cargo clippy --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime --all-targets -- -D warnings` | 0 | 891 ms | `/tmp/nieuw-os-nic-mac-2026-11-03/sdk-quality.log` | `0a2c94b072ffa4cc22f6bca6a91dfddfffe319bf7fa1a7f40f5daeec1ae4b17a` |
| Netwerkdienst-format + Clippy | `cargo +1.91.1 fmt --manifest-path services/network-service/Cargo.toml --all -- --check && cargo +1.91.1 clippy --locked --manifest-path services/network-service/Cargo.toml --all-targets -- -D warnings` | 0 | 1.234 s | `/tmp/nieuw-os-nic-mac-2026-11-03/service-quality.log` | `4a5310c5c79033d02b9537736afd2858f5e76e0d5fa913d33131c01ccb85b5c6` |
| Net-probe format | `cargo fmt --manifest-path sdk/examples/net-probe/Cargo.toml --all -- --check` | 0 | <1 s | geen | n.v.t. |
| Net-probe target-Clippy | `cargo clippy --offline --locked --manifest-path sdk/examples/net-probe/Cargo.toml --features native --bin net-probe --target x86_64-unknown-none -- -D warnings` | 0 | 322 ms | `/tmp/nieuw-os-nic-mac-2026-11-03/net-probe-clippy.log` | `f85d524e855e358cd1a5cab8edd1a53c7583e1ef633c3688f63ca9e0a5478b11` |
| Ring-3-service target-Clippy | `cargo +1.91.1 clippy --offline --locked --manifest-path sdk/examples/network-service/Cargo.toml --features native --bins --target x86_64-unknown-none -- -D warnings` | 0 | 1.133 s | `/tmp/nieuw-os-nic-mac-2026-11-03/network-service-bin-clippy.log` | `b4e82f71bb2e7cccae97e2425dccbbe39fb87abff2c6e403c7ae9554f8d34cf9` |

De tellers in de Rustlogs zijn 16 VirtIO-driver-, 33 SDK-runtime-, 10
netwerkdienst- en 26 net-stacktests. De Python-suite bevat 17 tests: samen
102 geslaagde tests.

## Freestanding builds

De buildrunner gebruikte Rust uit `/home/simon/.cargo/bin`. Voor
`native-net-service` was daarnaast de bestaande tijdelijke hostcompiler op
`PATH` gezet en via `CC` aangewezen; de eerste poging zonder die omgeving faalde
met `linker cc not found`, de herhaling slaagde.

| Profiel | Exact commando | Exit | Duur | Log | SHA-256 |
|---|---|---:|---:|---|---|
| Ring-3-probe en kernel | `python3 -B tools/build.py --kernel-feature native-system-net` | 0 | 1.887 s | `/tmp/nieuw-os-nic-mac-2026-11-03/native-system-net-build.log` | `ae01842edccfacc0fcdb01d6530e77e9cf905dc81c712ec068cddb9637fd3c5a` |
| Ring-3-netwerkdienst en kernel | `python3 -B tools/build.py --kernel-feature native-net-service` | 0 | 3.334 s | `/tmp/nieuw-os-nic-mac-2026-11-03/native-net-service-build.log` | `b5982f985ebcbf4729a62a636f71cf9f518416a36e11ba22b64f3151a2ab8483` |

SHA-256 van de gegenereerde, door de runner geïnspecteerde ring-3-ELFs:

- `native-system-net` `net-probe`: `f5a9ceb8c0fa2e92d85001974e2c71f7c05a22d6815235bb567a1a2d76eff382`
- `native-net-service` `network-service`: `2bf4be4e13d645108c3bd0481d789f116d037af8f841e1362ffa4494c57fe53a`
- `native-net-service` `network-client`: `1169224ccd2c61ee5f3191a833ddfde50f8273805738c0a98e1fe5e87ddc4e37`

De eerste probe-build na implementatie faalde door een lokale `local`
naamoverschaduwing; de variable is hernoemd en de uiteindelijke
`native-system-net`-build slaagde. De eerste onjuiste Cargo-selectors,
ontbrekende host-`cc` en standaard Rust 1.90 voor de 1.91-workspaces zijn
gecorrigeerd; zij zijn geen geslaagde controles.

De oorspronkelijke native netwerkacceptatiegate is niet uitgevoerd. De
fixture-counterverwachting is bijgewerkt van 7 naar 10 denial-events vanwege
drie nieuwe negatieve MAC-querygevallen, maar hosttests simuleren geen echte
NIC- of kernelruntime.
