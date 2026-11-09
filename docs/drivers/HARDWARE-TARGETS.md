# Hardwaredoel: van QEMU via VMware naar een echte pc

Status 5 november 2026. Dit document legt het **einddoel** vast waar al het
virtuele driverwerk naartoe werkt: Nieuw OS draait lokaal op een gewone
x86-64-desktop of -laptop met UEFI. Virtueel testen is de weg ernaartoe, niet
het eindpunt. Dit document kent zelf geen roadmappunten toe.

## Drie profielen, in deze volgorde

| Profiel | Wat | Rol | Wie draait het |
|---|---|---|---|
| `VM-FR` | QEMU `pc-q35-8.2`, OVMF, zie [FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md) | Automatische regressie in CI; negatieve proeven en foutinjectie | CI en agents |
| `VM-VMW` | VMware Workstation Pro 17 op Simons Windows-pc, UEFI, wegwerp-VM met alleen de gebouwde ISO | Tweede, onafhankelijke emulatie van **echte** apparaatmodellen (xHCI, NVMe, e1000e, HD Audio, SATA). Vangt QEMU-specifiek gedrag | Simon, met script en verwachte COM1-uitvoer |
| `HW-FR` | Eén concreet benoemde fysieke desktop of laptop (nog te kiezen) | Acceptatie op echte hardware; IOMMU, energie en batterij | Simon, met dezelfde image en herstelroute |

Een criterium dat "fysieke" hardware vraagt, blijft open tot `HW-FR` het
bewijs levert. `VM-VMW` vervangt dat niet, maar bewijst wel dat een driver niet
alleen op QEMU leunt. De VMware-startproef staat in
[VMWARE-WORKSTATION-TRIAL](../implementation/VMWARE-WORKSTATION-TRIAL.md).

## Minste drivers, breedste dekking

Kies per apparaatfamilie eerst de standaardinterface die in **alle drie**
profielen voorkomt. Eén driver dekt dan QEMU, VMware en de meeste echte pc's.

| Familie | Driver (standaard) | VM-FR | VM-VMW | HW-FR | Stand |
|---|---|---|---|---|---|
| Beeld | UEFI GOP-framebuffer | ja | ja | ja | aanwezig (G0/U01) |
| Invoer | PS/2 (i8042) | ja | ja | vaak (laptoptoetsenbord) | aanwezig, gepold |
| Invoer/USB | xHCI + USB-HID bootprotocol | `qemu-xhci` | ja | ja | in uitvoering (H02) |
| Opslag | NVMe | `nvme` | ja | ja | probe-crate aanwezig; volgende stap |
| Opslag | AHCI/SATA | `ich9-ahci` | ja | oudere pc's | later |
| Netwerk | Intel e1000e (82574/I219-familie) | `e1000e` | `e1000e` | Intel-ethernet | volgende stap (H04) |
| Audio | Intel HD Audio | `intel-hda` | `hdaudio` | ja | volgende stap (H03) |
| Energie | ACPI (FADT/DSDT, S5, later S3, `_BST`) | ja | ja | ja | S5 in PR #213 (H05) |
| Entropie | RDRAND/RDSEED | ja | ja | ja | virtio-rng aanwezig |
| DMA-grens | Intel VT-d (DMAR) | QEMU `intel-iommu` optioneel; guest-isolatie niet aangetoond | vIOMMU afhankelijk van VM-configuratie; niet aangetoond | alleen met geschikte VT-d-hardware | open (H06) |

VirtIO (blk/net/rng/snd) blijft de snelle CI-route, maar telt niet als dekking
voor echte hardware.

Bewust **niet** in de eerste ronde: wifi en Bluetooth (vereisen chipspecifieke
firmwareblobs en een licentiebesluit), GPU-versnelling (GOP is de basis;
"benodigde GPU-functionaliteit" in H03 moet nog worden afgebakend), Realtek-
ethernet, USB-hubs en isochrone USB. Die keuzes zijn een besluit van Simon
zodra `HW-FR` is gekozen.

## Hoe drivers zonder kernelwijziging bijkomen

De kernel krijgt één generieke, capability-begrensde driverinterface
(BAR-mapping, DMA-pagina's, vrijgave bij crash; zie branch `hw/driver-grant`).
Daarna draait elke driver hierboven als userspacedienst. Zonder ingerichte
guest-IOMMU en expliciete DMA-toewijzing is guest-DMA-isolatie niet bewezen;
de emulatorgrens beperkt DMA niet automatisch tot de buffers van een driver.
Op `HW-FR` vraagt die isolatie geschikte VT-d-hardware en implementatie (H06).

Hergebruik vóór zelf schrijven: register-/typecrates van rust-osdev (`xhci`,
`pci_types`, `acpi`) na de controle in [DEPENDENCIES](../DEPENDENCIES.md);
zie [driveronderzoek](../research/2026-11-03/os-repos-and-hardware-drivers.md).

## Wat Simon moet aanleveren voor HW-FR

1. Merk/model van de doelmachine en de uitvoer van een PCI-lijst (bijvoorbeeld
   vanuit een Linux-live-USB: `lspci -nn`), zodat elke apparaat-ID in de
   tabel hierboven een driver of een bewuste "niet ondersteund" krijgt.
2. Toestemming voor een USB-stick als bootmedium; geen interne schijf.
3. Voor COM1-only markers, zoals `UNSUPPORTED:PCI`, is een seriële verbinding
   nodig. Verbind de seriële poort van de doelmachine met een tweede computer
   via een geschikte seriële of USB-serieeladapter aan die tweede computer.
   Een USB-serieeladapter op de doelmachine zelf ontvangt niet automatisch
   COM1-uitvoer; zonder seriële poort vereist USB-uitvoer een ondersteunde
   USB-serialdriver. **Schermroute: onbekend/nog niet aangetoond.** Of dezelfde
   diagnostische markers op het scherm verschijnen moet afzonderlijk worden
   getest voordat een fysieke proef zonder seriële verbinding kan slagen.
