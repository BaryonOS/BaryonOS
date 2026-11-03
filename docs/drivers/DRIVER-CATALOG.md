# Drivercatalogus

Deze catalogus beschrijft code en registratie op de huidige workbranch. Een
registratie is geen succesvolle probe of werkende hardwareondersteuning.
QEMU-bewijs is geen bewijs voor fysieke hardware.

| Driver of bouwsteen | Hardware / IDs | Bron, versie en licentie | Implementatie en registratie | Tests / bewijsgrens |
|---|---|---|---|---|
| VirtIO block | QEMU modern PCI `1af4:1042`, profiel-BDF `00:05.0` | Nieuw OS: `drivers/virtio-blk`; geen externe drivercode | Eigen transport; type-0 snapshot wordt via `driver-core` gedecodeerd en de bestaande registratie gevalideerd; `discover()` selecteert via PCI-enumeratie en registry-match, niet via een vast BDF | Bestaande M5/M6-contracten en eerder profielbewijs; deze integratiestap niet onder QEMU getest |
| NVMe PCI-probe | PCI class `01:08:02`; geen vendor-ID-hardcodering | Nieuw OS: `drivers/nvme`; geen externe drivercode | Segment-0-enumeratie, unieke registry-match, BAR0 64-bit/non-prefetchable validatie en devicegebonden `PciBarGrant`; alleen `CAP`/`VS` read-only inspectie. Nog niet in kernelworkspace/volumeconsumer gekoppeld | Hosttestcode typechecked met Clippy, hostuitvoering geblokkeerd bij link; freestanding check geslaagd. Geen queue, block-I/O, QEMU- of fysieke test |
| VirtIO network | QEMU modern PCI `1af4:1041`, profiel-BDF `00:07.0` | Nieuw OS: `drivers/virtio-net`; geen externe drivercode | Eigen transport; type-0 snapshot wordt via `driver-core` gedecodeerd en de bestaande registratie gevalideerd; `discover()` selecteert via PCI-enumeratie en registry-match, niet via een vast BDF | Historisch model-/crosscompilebewijs staat in `drivers/virtio-net/verification/driver-acceptance.json`; deze integratiestap niet onder QEMU getest |
| VirtIO RNG | QEMU modern PCI `1af4:1044`; huidige fixture-BDF `00:06.0` | Nieuw OS: `drivers/virtio-rng`; geen externe drivercode | Eigen transport; segment-0-enumeratie, registry-match, gebonden PCI-configadapter, BAR-grants en factorygestuurde initialisatie; geen vaste BDF-eis | Registrymatchtestcode typechecked; hosttestuitvoering geblokkeerd door ontbrekende `libc_nonshared.a`; QEMU-runtime, fysieke RNG-kwaliteit en DMA-isolatie niet getest |
| VirtIO sound codec | VirtIO sound `1af4:1059`; een eerder QEMU-detectieprofiel gebruikte `00:08.0` | Nieuw OS: `drivers/virtio-snd`; geen externe drivercode | Pure codec-/descriptorvalidatie; geen geïntegreerde hostcontroller/transport, geen registryregistratie | Hosttests/no_std-code zijn geen audio-I/O; zie `drivers/virtio-snd/README.md` |
| USB descriptor core | USB descriptors, geen PCI-ID | Nieuw OS: `drivers/usb`; geen externe drivercode | Parserbouwsteen; geen xHCI of USB-hostcontroller, geen registryregistratie | Parserproeven bewijzen geen device-enumeratie of HID-input |
| PS/2 | PS/2 keyboard/mouse via legacy controllerpoorten | Nieuw OS: `drivers/ps2`; geen externe drivercode | Device-specifieke transportcode; geen PCI-match | Bestaande proeven blijven profielgebonden; geen fysieke hardwaretest in deze taak |
| RTC | CMOS/RTC-poorten | Nieuw OS: `drivers/rtc`; geen externe drivercode | Device-specifieke code; geen PCI-match | Bestaand bewijs blijft aan zijn verificationbestand gebonden; geen nieuwe run |
| ACPI power | Firmwaretabellen en legacy PM-registers | Nieuw OS: `drivers/acpi-power`; geen externe drivercode | Begrensde snapshotparser; geen generieke PCI-driver | Parserbewijs is geen platform-power-/slaapacceptatie |
| TPM CRB | QEMU CRB MMIO-aperture, geen PCI-match | Nieuw OS: `drivers/tpm-crb`; geen externe drivercode | Begrensde transport-/commandobouwsteen; aparte opt-in integratie | Geen product-TPM-/fysieke TPM-acceptatie |

Alle fysieke-hardwarestatussen zijn **niet getest**. Geen van de bovenstaande
entries krijgt door de registrymetadata een productvoortgangspunt.
