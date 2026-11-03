# xHCI PCI-capabilityprobe — 3 november 2026

**Bewijsniveau:** broncode, testcode/typecheck en freestanding compilecheck.
De hosttests zijn niet uitgevoerd. Er is geen controller gestart, USB-device
geënumereerd, QEMU-run of fysieke hardwaretest uitgevoerd.

## Bron en resultaat

- Branch: `copilot/universal-driver-framework`
- Implementatiecommit: `8c7ccdac31823837dfbcd28871a78f64b26b082e`
- De standalone `no_std`-crate matcht PCI class `0c:03:30`, valideert een
  devicegebonden 32-/64-bit non-prefetchable BAR0-grant van minimaal 4 KiB en
  leest alleen `CAPLENGTH`, `HCIVERSION`, `HCSPARAMS1`, `HCCPARAMS1`, `DBOFF`
  en `RTSOFF`.
- Operational-, doorbell-, runtime- en optionele extended-capabilityranges
  worden op uitlijning, overflow, BAR-bounds en onderlinge overlap
  gecontroleerd. De capabilitypointer wordt niet gevolgd.
- De crate schrijft geen controllerregisters, neemt het device niet over van
  firmware, schakelt geen bus mastering in en reserveert of deelt geen DMA,
  interrupts, rings of USB classdriverresources uit.
- Tien hosttestgevallen zijn aanwezig en door Clippy gecompileerd/typechecked,
  maar de testbinary werd niet gelinkt of uitgevoerd.

## Uitgevoerde controles

| Commando | Exit | Resultaat |
|---|---:|---|
| `CARGO_TARGET_DIR=build/xhci-probe $HOME/.cargo/bin/cargo +1.90.0 fmt --manifest-path drivers/xhci/Cargo.toml -- --check` | 0 | Formattercheck geslaagd; log `/tmp/nieuw-os-xhci-fmt-final.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `CARGO_TARGET_DIR=build/xhci-probe $HOME/.cargo/bin/cargo +1.90.0 clippy --manifest-path drivers/xhci/Cargo.toml --all-targets --offline -- -D warnings` | 0 | Strict Clippy geslaagd, inclusief compilatie/typecheck van testcode; log `/tmp/nieuw-os-xhci-clippy-final.log`, SHA-256 `cee673063f8c1ecb12d76b0f678dafba9f167a477af2dfbe2608fcaeeb835162` |
| `CARGO_TARGET_DIR=build/xhci-probe $HOME/.cargo/bin/cargo +1.90.0 check --manifest-path drivers/xhci/Cargo.toml --target x86_64-unknown-none --offline --locked` | 0 | Freestanding librarycheck geslaagd; log `/tmp/nieuw-os-xhci-none-check-final.log`, SHA-256 `cb427590cbbc0193da184b9733bb39691a0201e0c0c96fad0ca01c25308871ab` |
| `CARGO_TARGET_DIR=build/xhci-probe $HOME/.cargo/bin/cargo +1.90.0 test --manifest-path drivers/xhci/Cargo.toml --offline --locked` | 101 | Hosttests niet uitgevoerd: linker `cc` ontbreekt; log `/tmp/nieuw-os-xhci-host-test-final.log`, SHA-256 `12550087d79810ff0634b56d6c98e42c9444f70042a519404d9f6200593b6c06` |

Geen toolchain- of systeeminstallatie/workaround uitgevoerd. Er zijn geen
externe dependencies toegevoegd. Dit resultaat sluit geen FR-H01/H02-gate en
levert geen voortgangspunt op; xHCI-ownership, DMA, interrupts, transfer-rings,
USB-enumeratie, HID en driverherstel blijven open.

## Primaire bron

- Intel, *eXtensible Host Controller Interface for Universal Serial Bus
  (xHCI) Requirements Specification*, revision 1.2c, October 2025, §§5.2–5.3:
  <https://cdrdv2-public.intel.com/868295/xHCI__Rev1.2c.pdf>.
