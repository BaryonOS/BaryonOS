# xHCI PCI-capabilityprobe — 3 november 2026

**Bewijsniveau:** broncode, testcode/typecheck en freestanding compilecheck.
De hosttests zijn niet uitgevoerd. Er is geen controller gestart, USB-device
geënumereerd, QEMU-run of fysieke hardwaretest uitgevoerd.

## Bron en resultaat

- Branch: `copilot/xhci-reviewfix`
- Basis: main `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e`
- Bevroren reviewreferentie: `f2a8f6eecb2cf2be5399b331223da152d8975d80`
- Reviewfixbroncommit: `3a63f4512f6f16c2f5ebca7b2e2e21221f87f7f4`
- De standalone `no_std`-crate matcht PCI class `0c:03:30`, valideert een
  devicegebonden 32-/64-bit non-prefetchable BAR0-grant van minimaal 4 KiB en
  leest alleen `CAPLENGTH`, `HCIVERSION`, `HCSPARAMS1`, `HCCPARAMS1`, `DBOFF`
  en `RTSOFF`.
- De volledige operationalregisterregio inclusief `max_ports` portregisters,
  de doorbell-array, runtime-registers en de eerste extended-capabilityheader
  worden op uitlijning, overflow, BAR-bounds en overlap met andere
  geadverteerde gebieden gecontroleerd. Alle drie `HCSPARAMS1`-limieten
  (`max_slots`, `max_interrupters`, `max_ports`) moeten niet nul zijn. De
  extended-capabilityketen wordt niet gevolgd.
- De crate schrijft geen controllerregisters, neemt het device niet over van
  firmware, schakelt geen bus mastering in en reserveert of deelt geen DMA,
  interrupts, rings of USB classdriverresources uit. PCI BAR-sizing schrijft
  tijdelijk de PCI-configuratie-BARs, herstelt hun oorspronkelijke waarden en
  laat I/O-decode, memory-decode en bus mastering uitgeschakeld; xHCI-MMIO
  wordt alleen gelezen.
- Dertien hosttestgevallen zijn aanwezig en door Clippy gecompileerd/typechecked,
  maar de testbinary werd niet gelinkt of uitgevoerd. Negatieve gevallen
  bestrijken nul-limieten, overlap van portregisters en extended-capability-
  headers, uitgeschakelde PCI-decode/bus mastering na discovery en een
  niet-reagerende PCI-functie.

## Uitgevoerde controles

| Commando | Exit | Resultaat |
|---|---:|---|
| `CARGO_TARGET_DIR=build/xhci-reviewfix $HOME/.cargo/bin/cargo +1.90.0 fmt --manifest-path drivers/xhci/Cargo.toml -- --check` | 0 | Formattercheck geslaagd; log `/tmp/nieuw-os-xhci-reviewfix-fmt.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `CARGO_TARGET_DIR=build/xhci-reviewfix $HOME/.cargo/bin/cargo +1.90.0 clippy --manifest-path drivers/xhci/Cargo.toml --all-targets --offline -- -D warnings` | 0 | Strict Clippy geslaagd, inclusief compilatie/typecheck van testcode; log `/tmp/nieuw-os-xhci-reviewfix-clippy.log`, SHA-256 `0b5c81ddcec9e1e2acb0d7e44f73a0dfdf9bf25cad47b4eec678146db3adbe57` |
| `CARGO_TARGET_DIR=build/xhci-reviewfix $HOME/.cargo/bin/cargo +1.90.0 check --manifest-path drivers/xhci/Cargo.toml --target x86_64-unknown-none --offline --locked` | 0 | Freestanding librarycheck geslaagd; log `/tmp/nieuw-os-xhci-reviewfix-none-check.log`, SHA-256 `02db5a66fa0c60ca8618bd08aac556eac4f3ec863e8630a0d353dba6a0f6e126` |
| `CARGO_TARGET_DIR=build/xhci-reviewfix $HOME/.cargo/bin/cargo +1.90.0 test --manifest-path drivers/xhci/Cargo.toml --offline --locked` | 101 | Hosttests niet uitgevoerd: linker `cc` ontbreekt; log `/tmp/nieuw-os-xhci-reviewfix-host-test.log`, SHA-256 `5794eb392edef4fa5869665314b6e8c20e7f6d7fa83025c80c151481c5bca224` |
| `git diff --check` | 0 | Geen whitespacefouten in de reviewfix |

Geen toolchain- of systeeminstallatie/workaround uitgevoerd. Er zijn geen
externe dependencies toegevoegd. Dit resultaat sluit geen FR-H01/H02-gate en
levert geen voortgangspunt op; xHCI-ownership, DMA, interrupts, transfer-rings,
USB-enumeratie, HID en driverherstel blijven open.

## Primaire bron

- PCI-SIG, *PCI Code and ID Assignment Specification*, revision 1.10,
  28 November 2017, §1.2:
  <https://pcisig.com/PCIExpress/Specs/Base/CodeandIDAssignment_1.10>.
- Intel, *eXtensible Host Controller Interface for Universal Serial Bus
  (xHCI) Requirements Specification*, revision 1.2c, October 2025, §§5.2–5.3:
  <https://cdrdv2-public.intel.com/868295/xHCI__Rev1.2c.pdf>.

De centrale `README.md` en `docs/progress.json` zijn niet gewijzigd: de
integratieopdracht reserveert deze centrale voortgangsbestanden voor hun
eigenaar. Deze probe-/reviewfix sluit geen bestaand voortgangscriterium.
