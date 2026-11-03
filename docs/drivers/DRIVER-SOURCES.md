# Driverbronnen en herkomst

Geen upstream-driverbron is in deze stap vendored, als dependency toegevoegd
of naar Nieuw OS gekopieerd. De twee huidige VirtIO-registraties verwijzen
naar bestaande Nieuw OS-drivercode. De volgende upstreambronnen zijn alleen
beoordeeld als ontwerpreferentie of adoptiekandidaat.

| Bronversie | Gecontroleerde eigenschappen | Besluit |
|---|---|---|
| [ArceOS `modules/axdriver`](https://github.com/arceos-org/arceos/tree/8d2e6f97efc4a67359b99db5e519a55b7067b524/modules/axdriver), commit `8d2e6f9` | Per-devicecategorie devicecontainers; static model beperkt het aantal instanties, `dyn` model gebruikt trait objects; bus probing via `DriverProbe` en `init_drivers`. | Alleen architectuurreferentie. Niet overnemen: vaste Nieuw OS-capabilities, resource ownership, operationele failure contracts en eigen drivercode blijven leidend. |
| [ArceOS `axdriver_crates`](https://github.com/arceos-org/axdriver_crates/tree/983e30d1df6dfc18293a0eb7eedf6b0b0aea0bb2), commit `983e30d` | `axdriver_base`, block/net/display traits, PCI-wrapper en VirtIO-wrappers; workspace versie 0.2.0, editie 2024, `no_std`. Workspace biedt licentiekeuze GPL-3.0-or-later OR Apache-2.0 OR MulanPSL-2.0. | Category-traitvorm bestudeerd, geen code gekopieerd. Geen package gekozen; de volledige licentie- en dependencykeuze is niet nodig voor dit interne, dependencyvrije core. |
| [`rcore-os/virtio-drivers`](https://github.com/rcore-os/virtio-drivers/tree/c76e09d7143f3070f81776db108ceaff7d80a8ba), commit `c76e09d` | Versie 0.13.0, MIT, Rust 2024 edition, `no_std`; upstream README noemt block/net/GPU/input/sound/RNG. `Hal` is een `unsafe`-trait voor DMA-allocatie, fysieke/virtuele vertaling en MMIO; PCI README-profiel noemt memory-mapped CAM. README markeert `VIRTIO_F_VERSION_1` als TODO. | **SKIP voor huidige transportadoptie.** De huidige Nieuw OS-transport gebruikt x86 PCI mechanism 1 en een eigen moderne VirtIO 1.x-policy; upstream heeft hier geen passende directe PCI/HAL-adapter en de versie-1 status moet worden opgelost. Geen dependency toegevoegd. Later opnieuw vergelijken na een concreet resourcegrant- en transportcontract. |
| [rust-osdev `xhci`](https://github.com/rust-osdev/xhci/tree/f2254c86d5ba12754259bf0ae483c6c8dc821aad), commit `f2254c8` | `no_std` xHCI register-/context-/ringtypes, geen complete hostcontroller. MIT/Apache-2.0. | Alleen referentie voor een toekomstige xHCI-port; nog niet geïntegreerd. |
| [rust-osdev `pci_types`](https://github.com/rust-osdev/pci_types/tree/eff856b6bab81adbe1bda0ab750009f3786cb8d3), commit `eff856b` | PCI-types en capabilitystructuren, geen config-space backend of veilige busenumeratie. MIT/Apache-2.0. | Geen dependency; eerst vaststellen of bestaande kernelcode dit typewerk werkelijk dupliceert. |

Deze metadata is afkomstig uit de genoemde README-/Cargo-manifesten en
bronbestanden op de vastgezette commits, bekeken op 3 november 2026. SHA's
van external source archives zijn niet vastgelegd, omdat geen externe code is
binnengehaald of gebouwd. Voor elke toekomstige dependency moeten commit,
bronhash, licentie, MSRV, feature graph, build scripts, `unsafe`-grenzen en
offline build opnieuw worden vastgelegd.
