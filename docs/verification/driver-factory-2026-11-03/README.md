# Factorygekoppelde PCI-matches

**Status:** gerichte hosttests, Clippy en formatcontrole geslaagd; kernelprofiel-
builds nog niet uitgevoerd.

- Geteste bron: `43da75947cb5b147c1435a81547a22b3cebfb04a`
- Branch: `copilot/universal-driver-framework`
- Remote branch SHA: gelijk aan de geteste bron.
- PR #153: open; head `43da75947cb5b147c1435a81547a22b3cebfb04a`,
  base `5c66ec57aaf8eb411a83b8d6e42320ddc379383c`.
- Hostcompiler: tijdelijk uitgepakt onder
  `/tmp/nieuw-os-hostcc-2026-11-03/`; geen systeeminstallatie of
  repositorydependency gewijzigd.

## Resultaten

Alle Cargo-controles gebruikten deze tijdelijke compileromgeving:

```sh
export PATH="/home/simon/.cargo/bin:/tmp/nieuw-os-hostcc-2026-11-03/bin:$PATH"
export LD_LIBRARY_PATH=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/x86_64-linux-gnu
export GCC_EXEC_PREFIX=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/gcc/x86_64-linux-gnu/15/
export COMPILER_PATH=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/libexec/gcc/x86_64-linux-gnu/15
export LIBRARY_PATH=/tmp/nieuw-os-hostcc-2026-11-03/linker-script:/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/x86_64-linux-gnu:/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/gcc/x86_64-linux-gnu/15
export CC=/tmp/nieuw-os-hostcc-2026-11-03/bin/cc
```

| Controle | Exact commando | Exit | Duur | Log | SHA-256 |
|---|---|---:|---:|---|---|
| Hosttests | `cargo test --locked -p driver-core -p virtio-blk-driver -p virtio-net-driver` | 0 | 0,62 s | `/tmp/nieuw-os-driver-factory-2026-11-03/host.log` | `41fc56e029c9fdd31136b4bbd67f031d795c2a172b40368ae342105142a1b34a` |
| Clippy | `cargo clippy --locked -p driver-core -p virtio-blk-driver -p virtio-net-driver --all-targets -- -D warnings` | 0 | 0,47 s | `/tmp/nieuw-os-driver-factory-2026-11-03/clippy.log` | `fea5f39bae42aa2b4af905271b14fecb21bd9bc2652068b26ada8636e10f82bf` |
| Format | `cargo fmt --all -- --check` | 0 | 0,52 s | `/tmp/nieuw-os-driver-factory-2026-11-03/fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| Whitespace | `git diff --check` | 0 | n.v.t. | geen uitvoer | n.v.t. |

De hosttests omvatten 11 `driver-core`-, 1 blockdriver- en 16 netwerkdrivertests.
De `fmt`-log is leeg omdat de controle geen uitvoer produceerde. De kernel-
profielbuilds `milestone-m5`, `milestone-m6a/b/c`, `native-volume`,
`native-update --image-version 1` en `native-system-net` moeten nog volgen.

## Implementatie en bewijsgrens

`DriverRegistry` ondersteunt nu factorygekoppelde statische registraties en
`best_match_with_factory` retourneert de initializer van de uniek
meest-specifieke match. VirtIO block en netwerk bewaren die function pointer
privé in hun discoveryresultaat. Hun bestaande `Device::initialize`-ingang
roept de geselecteerde factory aan nadat het kernelpad de bestaande MMIO-
mapping en DMA-pool heeft aangeleverd.

Dit is een initializerbinding voor twee ingebouwde driverfamilies, geen
centrale heterogene plug-and-playregistry, geen dynamisch laden en geen
driverisolatie. De families gebruiken verschillende initializer-/pool-ABI's.
De callback blijft kernelcode; busmaster-, DMA/IOMMU-, interrupt-, reset- en
cleanupsemantiek zijn niet uitgebreid. QEMU-runtime en fysiek hardware zijn
niet getest. Geen roadmappunt; score blijft 41/100.
