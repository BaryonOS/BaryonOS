# Factorygekoppelde PCI-matches

**Status:** gerichte hosttests, Clippy, formatcontrole en zeven
freestanding kernelprofielbuilds geslaagd; geen QEMU-runtime- of
hardwareacceptatie.

- Implementatiecommit: `43da75947cb5b147c1435a81547a22b3cebfb04a`
- Geteste bron voor de profielbuilds: `9b2d6fb6b71eade4ffb2386480645564229c958e`
- Branch: `copilot/universal-driver-framework`
- Remote branch SHA: gelijk aan de geteste bron.
- PR #153: open; head `9b2d6fb6b71eade4ffb2386480645564229c958e`,
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
| Hosttests | `cargo test --locked -p driver-core -p virtio-blk-driver -p virtio-net-driver` | 0 | 0,17 s | `/tmp/nieuw-os-driver-factory-2026-11-03/receipt-run/host-final.log` | `ee4dbacc2f8000cc7592db03f305c4a7aae08656abb3f92da3ef524464d6b7af` |
| Clippy | `cargo clippy --locked -p driver-core -p virtio-blk-driver -p virtio-net-driver --all-targets -- -D warnings` | 0 | 0,08 s | `/tmp/nieuw-os-driver-factory-2026-11-03/receipt-run/clippy-final.log` | `02db5a66fa0c60ca8618bd08aac556eac4f3ec863e8630a0d353dba6a0f6e126` |
| Format | `cargo fmt --all -- --check` | 0 | 0,56 s | `/tmp/nieuw-os-driver-factory-2026-11-03/receipt-run/fmt-final.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| Whitespace | `git diff --check` | 0 | n.v.t. | geen uitvoer | n.v.t. |
| M5 | `python3 -B tools/build.py --kernel-feature milestone-m5` | 0 | 1,72 s | `m5.log` | `09a1bf0d7f45dc9efc76e13aa84c82943b978efb90fcd15466fccc2f80f9b299` |
| M6a | `python3 -B tools/build.py --kernel-feature milestone-m6a` | 0 | 1,62 s | `m6a.log` | `a8d5e1f0fef1b164b01de8cc04948040a42f5eb7f72ba627034a6a9229e8cfb3` |
| M6b | `python3 -B tools/build.py --kernel-feature milestone-m6b` | 0 | 1,55 s | `m6b.log` | `937137d84530fde369d3e40801ba4ee71f5b894a3f8c810f79335195b279eddd` |
| M6c | `python3 -B tools/build.py --kernel-feature milestone-m6c` | 0 | 1,64 s | `m6c.log` | `dc2212399fc96cda11608245327620dad49c2e489432298938d884c4bef54beb` |
| Volume | `python3 -B tools/build.py --kernel-feature native-volume` | 0 | 1,07 s | `volume.log` | `f7f0bf38cf16ed7c402bd3340ccb50dcf352e5bc2d1fb60b648646abe4228ecd` |
| Update | `python3 -B tools/build.py --kernel-feature native-update --image-version 1` | 0 | 1,03 s | `update.log` | `2256a949f4afc8f9b420d861d8f7fa3c77f3fd136268b67bab7f254b3ded6b95` |
| Netwerk | `python3 -B tools/build.py --kernel-feature native-system-net` | 0 | 1,89 s | `net.log` | `0ef22242b01d1dfeb03a00c7e1c036edd1b40a9ba65cbc1ab0b571669967b683` |

De hosttests omvatten 11 `driver-core`-, 1 blockdriver- en 16 netwerkdrivertests.
De `fmt`-log is leeg omdat de controle geen uitvoer produceerde. De
profielbuilds melden in hun logs dat de kernels zijn gebouwd en de ELF-segmenten
geïnspecteerd. De tijdelijke updateversie `1` is alleen compile-invoer, geen
release. Logpaden zijn relatief aan
`/tmp/nieuw-os-driver-factory-2026-11-03/receipt-run/`.

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
cleanupsemantiek zijn niet uitgebreid. `qemu-system-x86_64` ontbreekt lokaal;
runtime en fysiek hardware zijn niet getest. Geen roadmappunt; score blijft
41/100.
