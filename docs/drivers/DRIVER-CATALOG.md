# Drivercatalogus

Deze catalogus beschrijft code en registratie op de huidige workbranch. Een
registratie is geen succesvolle probe of werkende hardwareondersteuning.
QEMU-bewijs is geen bewijs voor fysieke hardware.

| Driver of bouwsteen | Hardware / IDs | Bron, versie en licentie | Implementatie en registratie | Tests / bewijsgrens |
|---|---|---|---|---|
| VirtIO block | QEMU modern PCI `1af4:1042`, profiel-BDF `00:05.0` | Nieuw OS: `drivers/virtio-blk`; geen externe drivercode | Eigen transport; `driver-core`-registratie toegevoegd, nog niet door bootprobe gestart | Bestaande M5/M6-contracten en eerder profielbewijs; in deze taak alleen freestanding buildcheck, geen VM-run |
| VirtIO network | QEMU modern PCI `1af4:1041`, profiel-BDF `00:07.0` | Nieuw OS: `drivers/virtio-net`; geen externe drivercode | Eigen transport; `driver-core`-registratie toegevoegd, nog niet door bootprobe gestart | Historisch model-/crosscompilebewijs staat in `drivers/virtio-net/verification/driver-acceptance.json`; native PCI-test was daar niet uitgevoerd; huidige native I/O niet opnieuw getest |
| VirtIO RNG | QEMU modern PCI `1af4:1044`, profiel-BDF `00:06.0` | Nieuw OS: `drivers/virtio-rng`; geen externe drivercode | Eigen transport; nog niet in `DriverRegistry` geregistreerd | Geen nieuwe test uitgevoerd; geen fysiek RNG-/DMA-bewijs |
| VirtIO sound codec | VirtIO sound `1af4:1059`; een eerder QEMU-detectieprofiel gebruikte `00:08.0` | Nieuw OS: `drivers/virtio-snd`; geen externe drivercode | Pure codec-/descriptorvalidatie; geen geïntegreerde hostcontroller/transport, geen registryregistratie | Hosttests/no_std-code zijn geen audio-I/O; zie `drivers/virtio-snd/README.md` |
| USB descriptor core | USB descriptors, geen PCI-ID | Nieuw OS: `drivers/usb`; geen externe drivercode | Parserbouwsteen; geen xHCI of USB-hostcontroller, geen registryregistratie | Parserproeven bewijzen geen device-enumeratie of HID-input |
| PS/2 | PS/2 keyboard/mouse via legacy controllerpoorten | Nieuw OS: `drivers/ps2`; geen externe drivercode | Device-specifieke transportcode; geen PCI-match | Bestaande proeven blijven profielgebonden; geen fysieke hardwaretest in deze taak |
| RTC | CMOS/RTC-poorten | Nieuw OS: `drivers/rtc`; geen externe drivercode | Device-specifieke code; geen PCI-match | Bestaand bewijs blijft aan zijn verificationbestand gebonden; geen nieuwe run |
| ACPI power | Firmwaretabellen en legacy PM-registers | Nieuw OS: `drivers/acpi-power`; geen externe drivercode | Begrensde snapshotparser; geen generieke PCI-driver | Parserbewijs is geen platform-power-/slaapacceptatie |
| TPM CRB | QEMU CRB MMIO-aperture, geen PCI-match | Nieuw OS: `drivers/tpm-crb`; geen externe drivercode | Begrensde transport-/commandobouwsteen; aparte opt-in integratie | Geen product-TPM-/fysieke TPM-acceptatie |

Alle fysieke-hardwarestatussen zijn **niet getest**. Geen van de bovenstaande
entries krijgt door de registrymetadata een productvoortgangspunt.
