# PCI-runtime en VirtIO BDF-migratie

**Bron:** `9984b1806a99e9bf0b2df0de7935d4a0d72a5219`  
**Branch:** `copilot/universal-driver-framework`  
**Remote:** `refs/heads/copilot/universal-driver-framework` wees bij controle
naar dezelfde commit.  
**Bewijsgrens:** hosttypechecks, Clippy en freestanding releasebuilds; geen
uitgevoerde hosttest, kernelboot, QEMU-runtime- of fysieke hardwareproef.

## Resultaten

| Controle | Commando | Exit | Uitkomst |
|---|---|---:|---|
| Formatter | `PATH=/home/simon/.cargo/bin:$PATH cargo +1.90.0 fmt --all -- --check` | 0 | Geslaagd |
| Driver typecheck | `PATH=/home/simon/.cargo/bin:$PATH cargo +1.90.0 check -p pci-core -p virtio-blk-driver -p virtio-net-driver --tests --offline` | 0 | Testtargets typechecken; tests niet uitgevoerd |
| Clippy | `PATH=/home/simon/.cargo/bin:$PATH cargo +1.90.0 clippy -p pci-core -p virtio-blk-driver -p virtio-net-driver --all-targets --offline -- -D warnings` | 0 | Geslaagd |
| Freestanding build | `PATH=/home/simon/.cargo/bin:$PATH cargo +1.90.0 build -p pci-core -p virtio-blk-driver -p virtio-net-driver --lib --release --target x86_64-unknown-none --offline` | 0 | Geslaagd |
| Hosttests | `PATH=/home/simon/.cargo/bin:$PATH cargo +1.90.0 test -p pci-core -p virtio-blk-driver -p virtio-net-driver --offline` | 101 | Geblokkeerd: `linker cc not found` |
| Kernel all-features check | `PATH=/home/simon/.cargo/bin:$PATH cargo +1.90.0 check -p os-kernel --all-features --offline --target x86_64-unknown-none` | 101 | Geblokkeerd tijdens buildscript-link: `linker cc not found` |
| Strikte preflight | `PATH=/home/simon/.cargo/bin:$PATH python3 -B tools/preflight.py --strict` | 2 | Rust 1.90.0 en beide targets aanwezig; QEMU, OVMF en host-`cc` ontbreken |
| Patchcontrole | `git diff --check` | 0 | Geslaagd vóór documentatiecheckpoint |

De stdout/stderr-logs staan buiten Git onder `/tmp/nieuw-os-pci-runtime-*`.
SHA-256:

| Log | SHA-256 |
|---|---|
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `check.log` | `736e2582f563605dd272e5fb977840b0c0767377d27b6940c440caf75eec7157` |
| `clippy.log` | `02db5a66fa0c60ca8618bd08aac556eac4f3ec863e8630a0d353dba6a0f6e126` |
| `release.log` | `92434e198f27984c035c376ec77a7cf9e6ebdde782f8ccb1336ae3329b66ddb7` |
| `host-test.log` | `7dd419c8752dc7e37c5f1afdd3664cd931d473996962cacba382a0d4d34de0d7` |
| `kernel-check.log` | `c34935627356ebb233efebe109f25aaa8ccbe06bfa6ce3104e18b629f147b66b` |
| `preflight.json` | `a34827a96c009c4e040e22ca9a4c46dca22a954dc2b3f74a9179a25168e542f0` |

Belangrijkste bronvingerafdrukken op de testbron:

| Bestand | SHA-256 |
|---|---|
| `crates/pci-core/src/lib.rs` | `a9cb7074116b17abc31a401df2d376f5d49d7a5ba3250744adc24d22a4e7a0bb` |
| `drivers/virtio-blk/src/lib.rs` | `bdf228b8661237a34b601d782c7755fcd44e0b8d64b49d7ffe5e9fc3a3e4f843` |
| `drivers/virtio-net/src/lib.rs` | `a4165e22aeeab4e37c82c35e9edaaff4de9ec784daa74c0a0f495e32cafd5bb7` |
| `drivers/virtio-net/src/pci.rs` | `8198a2aa54e6c6801bb89cf9db9a7b932cd2f9e278d277224c1d4162bbece56e` |
| `kernel/src/m2/m3/network.rs` | `aeb33c38cb2ee9e819074add26884f03f2164eecb7245e86c98d21a7af10fdeb` |

## Implementatiegrens

De scanner gebruikt PCI configuration mechanism 1 op segment 0, scant alle
conventionele busnummers omdat de huidige bootinformatie geen ACPI-
roottopologie aanlevert, respecteert multifunction-bitten en geeft alleen
type-0-endpoints door. De beide VirtIO-drivers matchen op hun registratie en
houden het geselecteerde BDF aan voor latere config-/busmastertoegang. Per
familie wordt meer dan één match geweigerd omdat de bestaande kernelbroker één
transportinstantie beheert.

Kernelcode blijft eigenaar van DMA-pagina's en BAR/MMIO-mappings. Dit resultaat
introduceert geen generieke resourcebroker, driver factoryregistry,
IRQ-lifecycle, DMA/IOMMU-model, bridgevalidatie, ECAM, multi-devicebroker,
USB-driver of fysieke hardwareondersteuning.
