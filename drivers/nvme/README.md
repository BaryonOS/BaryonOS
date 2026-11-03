# NVMe PCI-probe

Deze crate is de eerste hardwarestap voor NVMe-opslag: segment-0-PCI-
enumeratie, class matching (`01:08:02`), validatie van de 64-bit BAR0 en een
begrensde read-only inspectie van `CAP` en `VS`. De probe weigert meerdere
passende controllers; een onbekende functie wordt niet gestart.

De kernel moet de BAR-grant supervisor-only en uncached mappen en memory
decoding gecontroleerd inschakelen voordat `Discovery::map_bar0` en
`ControllerInfo::inspect` worden gebruikt. PCI BAR-probing herstelt de
BAR-registers en laat memory decoding en bus mastering uit. De crate schakelt
bus mastering niet zelf in en schrijft niet naar de controllerregisters.

Dit is **geen** bruikbaar blockdevice: er zijn nog geen admin-/I/O-queues,
Identify-commands, namespace-selectie, read/write/flush, reset- of
timeoutafhandeling. Deze volgende stap vereist een kernel-owned, devicegebonden
DMA-grant en expliciete busmaster-/interruptlifecycle. De crate maakt daarvoor
geen concurrerend algemeen DMA-model en is nog niet aan de kernelworkspace of
volumeconsumer gekoppeld.

## Controles

Vanaf de repositoryroot, met Rust 1.90.0:

```sh
cargo +1.90.0 fmt --manifest-path drivers/nvme/Cargo.toml -- --check
cargo +1.90.0 clippy --manifest-path drivers/nvme/Cargo.toml --all-targets --offline --locked -- -D warnings
cargo +1.90.0 check --manifest-path drivers/nvme/Cargo.toml --target x86_64-unknown-none --offline --locked
cargo +1.90.0 test --manifest-path drivers/nvme/Cargo.toml --offline --locked
```

Tests, exacte uitslagen en de bewijsgrens staan in
[`docs/verification/nvme-pci-probe-2026-11-03/`](../../docs/verification/nvme-pci-probe-2026-11-03/README.md).

## Bronnen

- PCI-SIG, *PCI Code and ID Assignment Specification*, revisie 1.13,
  17 november 2021: <https://pcisig.com/specification-overview/pci-express-base>.
- NVM Express, *NVM Express Base Specification*, revisie 2.2,
  11 maart 2025, §3.1 (controllerregisters):
  <https://nvmexpress.org/wp-content/uploads/NVM-Express-Base-Specification-Revision-2.2-2025.03.11-Ratified-1.pdf>.

De selectie en registerdecode zijn op deze specificaties gebaseerd; de
community-uitleg [OSDev: NVMe, W004432, revisie 30943](https://wiki.osdev.org/NVMe)
is alleen een aanvullende routekaart.
