# Devicegebonden PCI-configuratieadapter

**Broncommit:** `d9b97c4ad701b5c4410909000269e3114d2cfb80`  
**Ouder:** `1e813746efb1c8c1cf08528629871020ff8ad4d3`  
**Branch:** `copilot/universal-driver-framework`  
**Bewijsgrens:** testtargets typechecken en freestanding drivercrates bouwen;
hosttests zijn niet uitgevoerd door ontbrekende hostlinker `cc`. Geen kernelboot,
QEMU-runtime- of fysieke hardwareproef.

## Resultaat

`MatchedPciFunction` herleest het type-0-configuratieheader bij binding,
controleert dat de descriptor onveranderd is en dat de meegegeven
driverregistratie bij het device past. Config-space reads/writes naar een ander
BDF worden geweigerd. VirtIO block en netwerk gebruiken deze adapter voor
BAR-probing en busmasterbeheer na registryselectie. Negatieve tests dekken
verkeerd BDF, stale descriptor en niet-matchende registratie; ze zijn
gecompileerd maar niet uitgevoerd.

De adapter begrenst PCI-configuratie tot één functie. Hij is nog geen
kernel-owned BAR/MMIO-, IRQ- of DMA-grant, en maakt kernelcode niet
security-geïsoleerd van elkaar.

## Controles

| Controle | Commando | Exit | Duur | Resultaat |
|---|---|---:|---:|---|
| Formatter | `env PATH="/home/simon/.cargo/bin:$PATH" cargo +1.90.0 fmt --all -- --check` | 0 | 0,49 s | Geslaagd |
| Testtarget-typecheck | `env PATH="/home/simon/.cargo/bin:$PATH" cargo +1.90.0 check -p pci-core -p virtio-blk-driver -p virtio-net-driver --tests --offline` | 0 | 0,02 s | Testcode compileert; tests niet uitgevoerd |
| Clippy | `env PATH="/home/simon/.cargo/bin:$PATH" cargo +1.90.0 clippy -p pci-core -p virtio-blk-driver -p virtio-net-driver --all-targets --offline -- -D warnings` | 0 | 0,06 s | Geslaagd |
| Freestanding build | `env PATH="/home/simon/.cargo/bin:$PATH" cargo +1.90.0 build -p pci-core -p virtio-blk-driver -p virtio-net-driver --lib --release --target x86_64-unknown-none --offline` | 0 | 0,05 s | Geslaagd |
| Hosttests | `env PATH="/home/simon/.cargo/bin:$PATH" cargo +1.90.0 test -p pci-core -p virtio-blk-driver -p virtio-net-driver --offline` | 101 | 0,21 s | Geblokkeerd vóór uitvoering: `linker cc not found` |
| Kernel all-features check | `env PATH="/home/simon/.cargo/bin:$PATH" cargo +1.90.0 check -p os-kernel --all-features --offline --target x86_64-unknown-none` | 101 | 0,11 s | Buildscript-link geblokkeerd: `linker cc not found` |
| Patchcontrole | `git diff --check` | 0 | — | Geslaagd |

Primaire logs staan buiten Git onder
`/tmp/nieuw-os-pci-config-binding-2026-11-03/`.

| Log | SHA-256 |
|---|---|
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `check.log` | `6d77f64dfc8c5325b90fdbe05d8be367bdd578bcccec24f275c5ffee7d03dd6e` |
| `clippy.log` | `02db5a66fa0c60ca8618bd08aac556eac4f3ec863e8630a0d353dba6a0f6e126` |
| `build.log` | `90c8a1a69c0155761cff4bfe0e61bbee007961d2b96729d2127c2e0c291489dd` |
| `test.log` | `7035c09124160bffe8e8665454d84a8c8dd2ef24fed016374658f1364c2ac960` |
| `kernel-check.log` | `1b1a647adccbb63b9d14cc02d6f2efbcb7635b1cfa20af5747afc48884b334e4` |

SHA-256 van de aangepaste bronbestanden:

| Bestand | SHA-256 |
|---|---|
| `crates/pci-core/src/lib.rs` | `a645760c7d1b978ab110384c41b98924138f5460537ab03b311c3e05ba95c65f` |
| `drivers/virtio-blk/src/lib.rs` | `e6187435d32662b3070bfa2a07a75255be78b9554765f0e8961de2e6adca153d` |
| `drivers/virtio-net/src/lib.rs` | `56185bebe8de3fd4d415e2cfce375dd8c1f6033452ded16f0c0cf83df4842abf` |
| `drivers/virtio-net/src/pci.rs` | `53e43d1fcc2d787c0c1cf1eb17287a0925a51c4cb3eb8a23e3611c6da65e0bba` |

Er zijn geen hardwarecompatibiliteits- of runtimeclaims aan deze compile- en
typecheckresultaten ontleend.
