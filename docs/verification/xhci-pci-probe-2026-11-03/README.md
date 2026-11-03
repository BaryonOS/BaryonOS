# xHCI PCI-capabilityprobe — 3 november 2026

**Bewijsniveau:** broncode, hosttests en freestanding compilecheck.
Er is geen controller gestart, USB-device geënumereerd, QEMU-run of fysieke
hardwaretest uitgevoerd.

## Hostregressie en fixtureherstel

De bestaande userspace-toolchain is hergebruikt zonder installatie. Op
`5ea1c3494cc94482c993241fbda306becd4b4074` is eerst de gemelde regressie
gereproduceerd: 13 geslaagd, 2 gefaald (exit 101). De twee positieve fixtures
plaatsten hun extended-capabilityheader op `0x100`, binnen het gecontroleerde
operationalgebied `[0x40, 0x480)`. Alleen hun pointer en verwachte uitkomst zijn
verplaatst naar `0x800` (xECP `0x200` DWORDs), buiten die regio en vóór de
doorbells op `0x1000`. De productievalidatie is ongewijzigd.

Alle onderstaande commando's zijn uitgevoerd vanuit de geïsoleerde worktree
met `PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH`,
`CC=/tmp/nieuw-os-c-toolchain/bin/cc` en
`CARGO_TARGET_DIR="$PWD/build/xhci-reviewfix"`.

| Commando | Exit | Resultaat en log |
|---|---:|---|
| `cargo +1.90.0 test --offline --locked --manifest-path drivers/xhci/Cargo.toml` | 0 | Na fixtureherstel werkelijk uitgevoerd: 15/15 hosttests geslaagd, 0 doctests; `/tmp/nieuw-os-xhci-fixture-after.log`, SHA-256 `7bb788a04ef5afbf1b5cb6c07fa43507a0033ab7348ec39d715df02cdeb545cd` |
| `cargo +1.90.0 fmt --manifest-path drivers/xhci/Cargo.toml -- --check` | 0 | `/tmp/nieuw-os-xhci-fixture-fmt.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `cargo +1.90.0 clippy --manifest-path drivers/xhci/Cargo.toml --all-targets --offline --locked -- -D warnings` | 0 | `/tmp/nieuw-os-xhci-fixture-clippy.log`, SHA-256 `ed5c9bb286d443957ec8c71237292c9f6bd7529740f24695b9d85cf92b0f02d3` |
| `git diff --check` | 0 | Geen whitespacefouten |

De eerdere falende uitvoering staat in `/tmp/nieuw-os-xhci-fixture-before.log`,
SHA-256 `2fc006c1966954ebec1064919ae01db9528aa904e2ef66367865d22ed8c1cf77`.
Deze hosttests bewijzen parser-/mockgedrag, geen werkende xHCI-controller.
Onderstaande receipt blijft het historische compilebewijs vóór dit herstel;
de eerdere linkerblocker is voor de hosttests hierboven opgeheven.

## Workspace-integratie van prerequisites

Vervolg op fixtureherstelcommit `2731f279661fb5a8d87399d0129d0f1c91cd5f27`:
de ontbrekende rootworkspaceleden `driver-core` en `pci-core` en uitsluitend
hun lokale lockfile-entries zijn toegevoegd. Geen externe dependency,
versiewijziging of afzonderlijke workspace toegevoegd. Met dezelfde hierboven
vermelde omgeving zijn daarna alle volgende commando's uitgevoerd (exit 0):

| Commando | Resultaat |
|---|---|
| `cargo +1.90.0 test --offline --locked -p driver-core -p pci-core` | 11 driver-core- en 8 pci-core-hosttests geslaagd; 0 doctests |
| `cargo +1.90.0 test --offline --locked --manifest-path drivers/xhci/Cargo.toml` | 15 xHCI-hosttests geslaagd; 0 doctests |
| `cargo +1.90.0 fmt -p driver-core -p pci-core -- --check` | Geslaagd |
| `cargo +1.90.0 fmt --manifest-path drivers/xhci/Cargo.toml -- --check` | Geslaagd |
| `cargo +1.90.0 clippy --offline --locked -p driver-core -p pci-core --all-targets -- -D warnings` | Geslaagd |
| `cargo +1.90.0 clippy --offline --locked --manifest-path drivers/xhci/Cargo.toml --all-targets -- -D warnings` | Geslaagd |
| `cargo +1.90.0 check --offline --locked -p driver-core -p pci-core --target x86_64-unknown-none` | Geslaagd |
| `cargo +1.90.0 check --offline --locked --manifest-path drivers/xhci/Cargo.toml --target x86_64-unknown-none` | Geslaagd |
| `git diff --check` | Geslaagd |

Logs en SHA-256:

| Log | SHA-256 |
|---|---|
| `/tmp/nieuw-os-driver-workspace-test.log` | `0cef1097c4c093bcc24016feccd4270dd3c6e4f1ac4f78b362c8f017e3c4d75e` |
| `/tmp/nieuw-os-driver-workspace-xhci-test.log` | `c3dc03bd1a7280b43a9d69bd2794615c9192971fc23ab5eef2d72086ca5cc1bd` |
| `/tmp/nieuw-os-driver-workspace-fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `/tmp/nieuw-os-driver-workspace-clippy.log` | `7e01dad433bc36975e7922e925ef767a88a267116a1a6e73e70d7c4427fff5d4` |
| `/tmp/nieuw-os-driver-workspace-none.log` | `2f6a1328c42ad74b9a072d733413a1d90d9eaa953e8dda0ce584c095b4b6e7f4` |

## Bron en resultaat

- Branch: `copilot/xhci-reviewfix`
- Basis: main `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e`
- Bevroren reviewreferentie: `f2a8f6eecb2cf2be5399b331223da152d8975d80`
- Reviewfixbroncommit: `5995dacb9d28225a389a71d2c392e220f702fab4`
- De standalone `no_std`-crate matcht PCI class `0c:03:30`, valideert een
  devicegebonden 32-/64-bit non-prefetchable BAR0-grant van minimaal 4 KiB en
  leest alleen `CAPLENGTH`, `HCIVERSION`, `HCSPARAMS1`, `HCCPARAMS1`, `DBOFF`
  en `RTSOFF`.
- De volledige operationalregisterregio inclusief `max_ports` portregisters,
  de doorbell-array, runtime-registers en de eerste extended-capabilityheader
  worden op uitlijning, overflow, BAR-bounds en overlap met capability- of
  andere geadverteerde gebieden gecontroleerd. Alle drie `HCSPARAMS1`-limieten
  (`max_slots`, `max_interrupters`, `max_ports`) moeten niet nul zijn. De
  extended-capabilityketen wordt niet gevolgd.
- De crate schrijft geen controllerregisters, neemt het device niet over van
  firmware, schakelt geen bus mastering in en reserveert of deelt geen DMA,
  interrupts, rings of USB classdriverresources uit. PCI BAR-sizing schrijft
  tijdelijk de PCI-configuratie-BARs, herstelt hun oorspronkelijke waarden en
  laat I/O-decode, memory-decode en bus mastering uitgeschakeld; xHCI-MMIO
  wordt alleen gelezen.
- Vijftien hosttestgevallen zijn aanwezig en door Clippy gecompileerd/typechecked,
  maar de testbinary werd niet gelinkt of uitgevoerd. Negatieve gevallen
  bestrijken nul-limieten, overlap van portregisters en extended-capability-
  headers, overlap met de capabilityruimte, `max_ports`-BAR-bounds,
  uitgeschakelde PCI-decode/bus mastering na discovery en een niet-reagerende
  PCI-functie.

## Uitgevoerde controles

| Commando | Exit | Resultaat |
|---|---:|---|
| `CARGO_TARGET_DIR=build/xhci-reviewfix $HOME/.cargo/bin/cargo +1.90.0 fmt --manifest-path drivers/xhci/Cargo.toml -- --check` | 0 | Formattercheck geslaagd; log `/tmp/nieuw-os-xhci-reviewfix-fmt.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `CARGO_TARGET_DIR=build/xhci-reviewfix $HOME/.cargo/bin/cargo +1.90.0 clippy --manifest-path drivers/xhci/Cargo.toml --all-targets --offline -- -D warnings` | 0 | Strict Clippy geslaagd, inclusief compilatie/typecheck van testcode; log `/tmp/nieuw-os-xhci-reviewfix-clippy.log`, SHA-256 `fdc5ca451b3817065e58c2902bdbb45f75609908eb7dd770b133231909607567` |
| `CARGO_TARGET_DIR=build/xhci-reviewfix $HOME/.cargo/bin/cargo +1.90.0 check --manifest-path drivers/xhci/Cargo.toml --target x86_64-unknown-none --offline --locked` | 0 | Freestanding librarycheck geslaagd; log `/tmp/nieuw-os-xhci-reviewfix-none-check.log`, SHA-256 `1b54b439d4653b115688021ef7dbdacf9b7049b72bc76d4f864334e32f4beae6` |
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
