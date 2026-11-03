# DRV-NET-LATE-COMPLETION

Basis: `d10b5cdaf56774a39e3ef10463f674c6188a463a`, branch
`copilot/driver-next-net-reset`, uitgevoerd 3 november 2026.

## Brongebonden herhaling door integrator

De hieronder genoemde oorspronkelijke logs zijn historisch. De integrator
heeft alle vijf onderstaande commando's opnieuw uitgevoerd op exacte testbron
`ddf5207098d5088f28b52310220a80efe4569b7c`, vóór deze documentaanvulling,
met dezelfde PATH/CC en een eigen
`CARGO_TARGET_DIR="$PWD/build/pr-agent-a-host"`. Alle commando's exit 0;
17 hosttests geslaagd, 0 doctests. Geen nieuwe native uitvoering.

| Nieuw log | SHA-256 |
|---|---|
| `/tmp/pr-agent-a-170-test.log` | `aade44ce1d632eb8c73e7d23c35a2f94aed6673b49545881430cab9ef603f758` |
| `/tmp/pr-agent-a-170-fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `/tmp/pr-agent-a-170-clippy.log` | `17bc9a45344f69453acf4b773574fa3d10360eb51f9303b23c128540cd8aec33` |
| `/tmp/pr-agent-a-170-none.log` | `779d7a91e1a6938df1f3bac58900e78947d06951b1b1456f2944fc5052509674` |

## Oorspronkelijk verslag

Alleen `src/tests.rs` en dit bewijsbestand wijzigen. Geen productiecode,
kernel-/SDK-adapter, dependency of DMA-/resetcontract gewijzigd.

Twee nieuwe hostmodeltests controleren:

- Bij expliciete reset en TX-timeout: late RX/TX-completions terwijl reset
  geweigerd wordt; herhaald pollen/publiceren blijft Offline, output blijft
  intact, geen notificatie/replay en alle vier gepinde pagina's blijven
  bytegelijk, ook bij geweigerde initialisatie van een nieuwe driverinstantie.
- Na waargenomen resetbevestiging: nieuwe instantie op dezelfde pagina's,
  zonder oude instantie te behouden. Zowel normale als gewrapte used-index
  wordt gewist; geen oude completion of frame wordt geleverd. Nieuwe RX/TX
  slaagt pas na nieuwe devicecompletion; outputstaart en TX-padding blijven
  intact.

Omgeving: bestaande Rust 1.90.0 en C-wrapper, geen installatie.

```sh
export PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export CARGO_TARGET_DIR="$PWD/build/driver-net"
cargo +1.90.0 test --offline --locked -p virtio-net-driver
cargo +1.90.0 fmt -p virtio-net-driver --check
cargo +1.90.0 clippy -p virtio-net-driver --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build -p virtio-net-driver --lib --release --target x86_64-unknown-none --locked --offline
git diff --check
```

Alle commando's exit 0. Suite: 17 tests geslaagd, 0 doctests.
Logs onder `/tmp/nieuw-os-net-late-`, met SHA-256:

| Suffix | SHA-256 |
|---|---|
| `test.log` | `277533ba87a3dfb867d098ad34d1a5d4c08912a2f20caf305a99417858aa410c` |
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `clippy.log` | `38a1258aece5a01cc1c6684e8773a8cd1cbf8756494f280f5e7d22ce494928ac` |
| `none.log` | `83a5f5870d856f7095215acf28d5b7147b7c56d2ded5bad4265676af2b7c44ab` |

Dit is een geserialiseerd register-/DMA-hostmodel, geen echte NIC, concurrerende
DMA of IOMMU-isolatie. Alleen dit geserialiseerde hostmodel toont hier geen
replay, bytegelijke pagina's tijdens geweigerde reset en gewiste ringstatus
na bevestigde reset aan. Isolatie van buffers en capability-beschermd geheugen
tegen concurrerende of kwaadwillende device-DMA vereist afzonderlijke
IOMMU-/hardwareafscherming; CPU-capabilities alleen bewijzen dat niet.
Geen van die hardwaregaranties is hier getest. Het bestaande contract uit
`docs/implementation/VIRTIO-NET.md` blijft leidend: pagina's permanent gepind,
hergebruik alleen na status nul en bus mastering uit. QEMU-, fysieke en
menselijke controles zijn niet uitgevoerd. De freestanding build is geen
runtimebewijs. Geen nieuwe acceptatiepunten; centraal assessment bij integrator.
Rollback van beide bestanden raakt productiegedrag niet.
