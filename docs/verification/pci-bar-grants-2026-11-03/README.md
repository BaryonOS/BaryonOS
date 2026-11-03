# Devicegebonden PCI BAR-grants

**Status:** hosttests, lint en freestanding profielbuilds geslaagd; geen
runtime- of hardwareacceptatie.

- Geteste bron: `b67f8739b48378bd30e57bbb97d86de109fa9b95`
- Branch: `copilot/universal-driver-framework`
- Remote branch SHA: gelijk aan de geteste bron.
- PR #153: open; head `b67f8739b48378bd30e57bbb97d86de109fa9b95`,
  base `5c66ec57aaf8eb411a83b8d6e42320ddc379383c`.
- Hostcompiler: tijdelijk uitgepakt onder
  `/tmp/nieuw-os-hostcc-2026-11-03/`; geen systeeminstallatie of
  repositorydependency gewijzigd.

## Resultaten

Alle onderstaande commando's gebruikten de tijdelijke compileromgeving:

```sh
export PATH="/tmp/nieuw-os-hostcc-2026-11-03/bin:/home/simon/.cargo/bin:$PATH"
export LD_LIBRARY_PATH=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/x86_64-linux-gnu
export GCC_EXEC_PREFIX=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/gcc/x86_64-linux-gnu/15/
export COMPILER_PATH=/tmp/nieuw-os-hostcc-2026-11-03/root/usr/libexec/gcc/x86_64-linux-gnu/15
export LIBRARY_PATH=/tmp/nieuw-os-hostcc-2026-11-03/linker-script:/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/x86_64-linux-gnu:/tmp/nieuw-os-hostcc-2026-11-03/root/usr/lib/gcc/x86_64-linux-gnu/15
export CC=/tmp/nieuw-os-hostcc-2026-11-03/bin/cc
```

| Controle | Exact commando | Exit | Duur | Log | SHA-256 |
|---|---|---:|---:|---|---|
| Hosttests | `cargo test --locked -p pci-core -p virtio-blk-driver -p virtio-net-driver` | 0 | 0,16 s | `/tmp/nieuw-os-pci-bar-grant-2026-11-03/receipt-run/host-tests.log` | `9590c5150bdbbf0eba21b4e1f866623ecc79f03def79762ea895a4eafb12c117` |
| Clippy | `cargo clippy --locked -p pci-core -p virtio-blk-driver -p virtio-net-driver --all-targets -- -D warnings` | 0 | 0,09 s | `/tmp/nieuw-os-pci-bar-grant-2026-11-03/receipt-run/clippy.log` | `02db5a66fa0c60ca8618bd08aac556eac4f3ec863e8630a0d353dba6a0f6e126` |
| Format | `cargo fmt --all -- --check` | 0 | 0,56 s | `/tmp/nieuw-os-pci-bar-grant-2026-11-03/receipt-run/fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| M5 | `python3 -B tools/build.py --kernel-feature milestone-m5` | 0 | 1,36 s | `m5.log` | `7b2d3625be3d79f4248ee1c7c8d028e0f87fb6d30c8bb17cfe6d2fad31fd1d41` |
| M6a | `python3 -B tools/build.py --kernel-feature milestone-m6a` | 0 | 1,59 s | `m6a.log` | `b3dbda6545678cdfcf0fa6bc5332123c8e5f22310c5b289485e62b0011ad2b4b` |
| M6b | `python3 -B tools/build.py --kernel-feature milestone-m6b` | 0 | 1,59 s | `m6b.log` | `4ac15cdb0096e0c4b2f86cd9e333c3a2c4d4978127e5fd82193787969d2311b6` |
| M6c | `python3 -B tools/build.py --kernel-feature milestone-m6c` | 0 | 1,72 s | `m6c.log` | `25d41afdbbb7c6cc9f36c7f42d964056cf213f2d53f58738944edb0dceb0381f` |
| Volume | `python3 -B tools/build.py --kernel-feature native-volume` | 0 | 0,31 s | `volume.log` | `7a1effbe2ec2ad09384caf91655901068e2cb346dd29b69d546e7bbc71d3d05c` |
| Update | `python3 -B tools/build.py --kernel-feature native-update --image-version 1` | 0 | 0,31 s | `update.log` | `bb4a7b34e6eeba6972bdeee18b6ae49635da849fc8a4734cf0ae15a3bd21793d` |
| Netwerk | `python3 -B tools/build.py --kernel-feature native-system-net` | 0 | 0,62 s | `net.log` | `0c60eda38675045b898a85fa43a7730bcb36b2edcfddaff1d75f032a20ab8947` |

De hosttests omvatten 8 `pci-core`-, 1 blockdriver- en 16 netwerkdrivertests.
De `fmt`-log is leeg omdat de controle geen uitvoer produceerde. Volledige
profiel-logpaden zijn `/tmp/nieuw-os-pci-bar-grant-2026-11-03/receipt-run/`.

## Bewijsgrens

`PciBarGrant` wordt alleen door een gematchte PCI-functie gemaakt en bindt
deviceadres, driver-ID, BAR-index en bereik. De BAR-probe laat PCI-I/O-decode,
memory-decode en bus mastering uit. De kernel accepteert alleen een pagina
binnen die grant voor block/net. Dit geldt niet voor de afzonderlijke RNG-route
en maakt de drivercode niet geïsoleerd van de kernel. IRQ-toewijzing,
interrupt lifecycle, afzonderlijke busmastergrant, DMA/IOMMU-bescherming,
factorygestuurde activatie en meerdere actieve instanties blijven open.

QEMU-runtimeproeven en fysieke hardwaretests zijn niet uitgevoerd. Dit bewijs
verhoogt de roadmap niet; de score blijft 41/100.
