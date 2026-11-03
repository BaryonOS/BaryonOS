# Driver Core-verificatie — 3 november 2026

Branch `copilot/universal-driver-framework`, basiscommit
`56a025bb326f7341ec89f40caf6a18345de9110f`. Dit is hostcompile-/crosscompile-
bewijs voor de eerste Driver Core-stap, geen VM-, hardware-, plug-and-play- of
releaseacceptatie.

Toolchain: Rust 1.90.0. Alle onderstaande succesvolle commando's gebruikten
`CARGO_TARGET_DIR=build/universal-driver-core` en eindigden met exit 0:

```sh
CARGO_TARGET_DIR=build/universal-driver-core $HOME/.cargo/bin/cargo +1.90.0 fmt -p driver-core -p virtio-blk-driver -p virtio-net-driver -- --check
CARGO_TARGET_DIR=build/universal-driver-core $HOME/.cargo/bin/cargo +1.90.0 check --tests -p driver-core -p virtio-blk-driver -p virtio-net-driver --offline --locked
CARGO_TARGET_DIR=build/universal-driver-core $HOME/.cargo/bin/cargo +1.90.0 clippy --all-targets -p driver-core -p virtio-blk-driver -p virtio-net-driver --offline --locked -- -D warnings
CARGO_TARGET_DIR=build/universal-driver-core $HOME/.cargo/bin/cargo +1.90.0 build --release -p driver-core -p virtio-blk-driver -p virtio-net-driver --target x86_64-unknown-none --offline --locked
```

`check --tests` compileert testcode maar voert die niet uit. De hosttest is
geprobeerd en geblokkeerd door ontbrekende hostlinker:

```sh
CARGO_TARGET_DIR=build/universal-driver-core $HOME/.cargo/bin/cargo +1.90.0 test -p driver-core -p virtio-blk-driver -p virtio-net-driver --offline
```

Exitstatus 101: `linker cc not found`. Volledige logs zijn lokaal beschikbaar
in `build/universal-driver-core/verification-2026-11-03.log` en
`build/universal-driver-core/host-tests-2026-11-03.log`; deze buildbestanden
zijn niet bedoeld voor Git.

SHA-256 van de belangrijkste bronbestanden op het moment van verificatie:

| Bestand | SHA-256 |
|---|---|
| `crates/driver-core/src/lib.rs` | `c04310443a6baf79ece27857230d3373437c5348ed0753fe6e5954ea21549a41` |
| `drivers/virtio-blk/src/lib.rs` | `f1858d43d95861363ab69fb1c3d9096b3f3b125dcbf491a0a1bb0fb206d8e7e8` |
| `drivers/virtio-net/src/lib.rs` | `e1addc9b425302e0018a729c3b92073f14a4c73f339921f72f95140116e6a475` |

Geen QEMU-run, fysieke hardwaretest of volledige CI uitgevoerd. De 41/100-
voortgangsscore is onveranderd; deze infrastructuur voldoet niet aan de
acceptatiecriteria voor generieke hardwareondersteuning.
