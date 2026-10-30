# USB descriptor framing core — f8e7490

**Reikwijdte:** dependencyvrije `no_std`-library voor het valideren van
USB-descriptorgrenzen in een caller-bounded byte slice. Onbekende typen blijven
leesbaar als borrowed bytes. Er is geen hostcontroller, transfer, device-I/O,
enumeratie, HID, xHCI-, hotplug-, DMA- of fysiek bewijs.

- Broncommit: `f8e7490b1dd8fdeb3cd8e8eb8578535bc49c5cc6`.
- Toolchain: lokaal gepinde Rust 1.90.0, zonder systeembrede installatie.
- Hosttargetmap: `target/usb-host`.
- Freestanding targetmap: `target/usb-target`.
- Release-library SHA-256:
  `2d8d28d4a3951efb400fdce10f0342b879c592c39017bbb2861e6b7713acf1af`.

| Commando | Exit/resultaat |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path drivers/usb/Cargo.toml` | 0 — 7 tests |
| `cargo +1.90.0 clippy --offline --locked --manifest-path drivers/usb/Cargo.toml --all-targets -- -D warnings` | 0 |
| `cargo +1.90.0 fmt --manifest-path drivers/usb/Cargo.toml --all -- --check` | 0 |
| `cargo +1.90.0 build --offline --locked --release --lib --target x86_64-unknown-none --manifest-path drivers/usb/Cargo.toml` | 0 |

`host-test.log`, `clippy.log`, `format.log` en `target-build.log` bewaren de
uitvoer; `target-artifact.sha256` bindt de gebouwde libraryhash. Bij Cargo
builds wees `CARGO_TARGET_DIR` naar de hierboven vermelde aparte mappen; `PATH`,
`RUSTUP_HOME` en `CARGO_HOME` wezen naar `.tools/nieuw-os`. De gegenereerde
librarybinary is niet in Git opgenomen.
