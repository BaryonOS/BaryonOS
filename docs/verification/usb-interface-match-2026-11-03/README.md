# USB interface-descriptormatching — 3 november 2026

**Bewijsniveau:** broncode, testcode/typecheck en freestanding compilecheck.
De USB-tests zijn op deze bron niet uitgevoerd; er is geen xHCI-controller,
QEMU-USB-apparaat of fysieke hardware getest.

## Bron en resultaat

- Branch: `copilot/universal-driver-framework`
- Implementatiecommit: `e6e38d1e524776241b4485bef278e16b5a2865ad`
- Standcheckpoint: `d4b3b45e4c05da4bf9378a3a31a1178b06bf60f1`
- De nieuwe parser decodeert standaard interface-descriptors van exact negen
  bytes en matcht class/subclass/protocol; weggelaten subclass- en
  protocolvelden werken als wildcards.
- Drie nieuwe testgevallen dekken decode, selectie en foutieve type/lengtes.
  De bestaande framingtests blijven ongewijzigd.
- Geen USB-transfer, driverregistratie, resourcegrant of device-I/O.

## Uitgevoerde controles

| Commando | Exit | Resultaat |
|---|---:|---|
| `CARGO_TARGET_DIR=build/usb-descriptor-core $HOME/.cargo/bin/cargo +1.90.0 fmt --manifest-path drivers/usb/Cargo.toml -- --check` | 0 | Formattering in orde; log `/tmp/nieuw-os-usb-fmt-e6e38d1.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `CARGO_TARGET_DIR=build/usb-descriptor-core $HOME/.cargo/bin/cargo +1.90.0 clippy --manifest-path drivers/usb/Cargo.toml --all-targets --offline --locked -- -D warnings` | 0 | Strikte Clippy geslaagd; library en testcode zijn gecompileerd/typechecked; log `/tmp/nieuw-os-usb-clippy-e6e38d1.log`, SHA-256 `7774487222b8cb6656f3eacf8f98299787f21ada91abbe1aa41929db2ac2b87d` |
| `CARGO_TARGET_DIR=build/usb-descriptor-core $HOME/.cargo/bin/cargo +1.90.0 check --manifest-path drivers/usb/Cargo.toml --target x86_64-unknown-none --offline --locked` | 0 | Freestanding librarycheck geslaagd; log `/tmp/nieuw-os-usb-none-check-e6e38d1.log`, SHA-256 `fc41732cca2aab2a3d40c0ba0880ff78f5b7ca67603d05a91a14e8071545a0e0` |
| `CARGO_TARGET_DIR=build/usb-descriptor-core $HOME/.cargo/bin/cargo +1.90.0 test --manifest-path drivers/usb/Cargo.toml --offline --locked` | 101 | Hosttests niet uitgevoerd: linker `cc` ontbreekt; log `/tmp/nieuw-os-usb-host-test-e6e38d1.log`, SHA-256 `2e86699ba2b0df61ebd7a024b1bcc87fdc7a3474c73fd982487629bb169e5e97` |

Geen toolchain- of systeeminstallatie/workaround uitgevoerd. Er zijn geen
nieuwe dependencies toegevoegd. Dit resultaat sluit geen FR-H01/H02-gate en
levert geen voortgangspunt op; xHCI, USB-enumeratie, HID, mass storage, DMA,
interrupts, hotplug en runtimeherstel blijven open.
