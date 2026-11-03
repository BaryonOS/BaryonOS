# PCI type-0-validatie — verificatie 3 november 2026

Branch `copilot/universal-driver-framework`; voorafgaande gepushte bron
`ca2cbcf78a43bcc62ff32bd62836f881401227d5`. Deze stap voegt een decoder voor
conventionele type-0 PCI-configuratiesnapshots toe, gebruikt die vanuit de
bestaande vaste-BDF VirtIO block-/netwerkprobes en neemt `driver-core` op in
de root CI-hosttest- en Clippy-commando's. Er is geen algemene
PCI-enumerator/resourcegrant of generieke driverstart gebouwd.

## Geslaagde lokale gates

Rust 1.90.0; alle onderstaande commando's gebruikten
`CARGO_TARGET_DIR=build/universal-driver-core` en eindigden exit 0:

```sh
CARGO_TARGET_DIR=build/universal-driver-core $HOME/.cargo/bin/cargo +1.90.0 fmt -p driver-core -p virtio-blk-driver -p virtio-net-driver -- --check
CARGO_TARGET_DIR=build/universal-driver-core $HOME/.cargo/bin/cargo +1.90.0 check --tests -p driver-core -p virtio-blk-driver -p virtio-net-driver --offline --locked
CARGO_TARGET_DIR=build/universal-driver-core $HOME/.cargo/bin/cargo +1.90.0 clippy --all-targets -p driver-core -p virtio-blk-driver -p virtio-net-driver --offline --locked -- -D warnings
CARGO_TARGET_DIR=build/universal-driver-core $HOME/.cargo/bin/cargo +1.90.0 build --release -p driver-core -p virtio-blk-driver -p virtio-net-driver --target x86_64-unknown-none --offline --locked
```

De volledige uitvoer staat in
`build/universal-driver-core/integration-gates-2026-11-03.log`.
`check --tests` compileert de unit-/modeltests maar voert ze niet uit.

## Niet uitgevoerd

De hosttests zijn geprobeerd:

```sh
CARGO_TARGET_DIR=build/universal-driver-core $HOME/.cargo/bin/cargo +1.90.0 test -p driver-core -p virtio-blk-driver -p virtio-net-driver --offline --locked
```

Exit 101: `linker cc not found`. Log:
`build/universal-driver-core/integration-host-tests-2026-11-03.log`.

`python3 -B tools/preflight.py --strict` eindigde exit 2. Het rapport staat in
`build/universal-driver-core/preflight-integration-2026-11-03.json`; de lokale
omgeving mist de gepinde Rust-tools in `PATH`, `x86_64-unknown-none`,
QEMU, OVMF en `cc`. De directe Rust 1.90.0 binaries werken voor de genoemde
compilegates. Geen QEMU-run, fysieke hardwaretest of volledige CI uitgevoerd.

SHA-256 van de belangrijkste bronbestanden vóór checkpointcommit:

| Bestand | SHA-256 |
|---|---|
| `crates/driver-core/src/lib.rs` | `fbfc70d660071053adbd336fc984a9b286a9225e5eb7d9a0c28e1703595daef3` |
| `drivers/virtio-blk/src/lib.rs` | `024c4c28dd6dfec273b9443f43fa473295c1e40751a4a61812047ec628331774` |
| `drivers/virtio-net/src/pci.rs` | `b80e2e0fba65b4876b1b4509508741f4a925eb68e7284837bd2a67f734ae5cd2` |
| `drivers/virtio-net/src/tests.rs` | `964e0baf178d0c2b56d1d3c5496d4a55e8e1a309cb518a104465fcb648e126d9` |

Er zijn geen voortgangspunten toegekend: descriptordecodering en
fixed-profile-validatie bewijzen geen generieke hardwareondersteuning.
