# Driver Core en selectie

Status: eerste broncodeversie op `copilot/universal-driver-framework`.
De registry valideert type-0 PCI-descriptors tijdens de bestaande vaste-BDF
VirtIO-probes. Algemene PCI-enumeratie, toekenning van devicebronnen en
generieke driverstart zijn nog niet geïmplementeerd.

## Huidige implementatie

`crates/driver-core` is een dependencyvrije `no_std`-crate zonder `unsafe`.
Hij definieert:

- gemeenschappelijke `DeviceId`, `DeviceClass`, `DriverId`, `DriverVersion`,
  `DriverState` en `DeviceMetadata`;
- eenvoudige traits voor block-, netwerk-, display- en inputdevices;
- toekomstige categorie-identifiers voor USB-hostcontrollers, audio,
  storagecontrollers en seriële devices;
- PCI-identiteit inclusief BDF, vendor/device, class/subclass/programming
  interface/revision en optionele subsystem-ID's;
- een bounds-safe decoder voor vaste 256-byte type-0 PCI-configuratiesnapshots;
- een vaste-capaciteit `DriverRegistry` die een uniek meest specifiek
  registratie-resultaat kiest en gelijke topmatches weigert.

De huidige `virtio-blk-driver` en `virtio-net-driver` publiceren een
`DRIVER_REGISTRATION` voor respectievelijk `1af4:1042` en `1af4:1041`.
Hun bestaande probes decoderen nu een type-0 header en bevestigen de identiteit
via de registry, zowel voor als na BAR-probing. Dit voorkomt dat hun fixed-BDF
transport een niet-geregistreerde of niet-type-0-functie verder initialiseert.
De probe blijft per driver op het bestaande BDF begrensd; de registry kiest
geen driver voor een algemene inventaris. Kernelpaden blijven eigenaar van
discovery, MMIO, bus mastering en DMA.

```text
PCI-probe op vaste BDF (bestaand, per driver)
    -> PciDeviceDescriptor
    -> DriverRegistry::best_match
    -> registratievalidatie of geen driver / ambiguïteit
    -> resourcegrant en driverstart (nog niet beschikbaar)
    -> subsysteminterface
```

ArceOS' categoriegerichte traits en `DriverProbe`/`init_drivers`-structuur
dienden als architectuurreferentie; er is geen ArceOS-code overgenomen.
De Nieuw OS-kern houdt matching, resourceautorisatie en starten gescheiden:
een PCI-match verleent geen toegang tot BAR's, interrupts of DMA.

## Nog benodigde architectuur

1. Breid type-0 descriptordecodering gecontroleerd uit naar de benodigde
   header-/bussoorten en voeg daarna pas een begrensde PCI-enumerator toe.
   Houd configuratieruimte op uitsluitend gevalideerde functies en veilige
   registers.
2. Ontwerp kernel-owned resource grants voor exact één PCI-function en
   expliciete BAR-subregio's, interruptbronnen, DMA-pools en resetrechten.
   Een portable driver ontvangt geen fysieke adressen, vrije config-space
   writes, kernelpointers, filesystem- of netwerkpolicytoegang.
3. Koppel registratie aan een driverfactory die uitsluitend reeds verleende
   resources kan gebruiken. Startfouten leveren geen `Ready`-metadata op;
   onbekende of ambigue devices worden niet gestart.
4. Pas daarna bestaande VirtIO-implementaties aan de subsystemtraits aan.
   Behoud hun huidige policy-, DMA-, timeout-, reset- en quarantainepaden.
5. Voeg USB-, MMIO- en niet-PCI-discovery toe met eigen descriptor- en
   resourcecontracten; de huidige PCI-registry beweert die bussen niet te
   ondersteunen.

Er is nog geen generieke resource/HAL-adapter of IOMMU-domein. De bestaande
virtio-blk/net-proeven blijven aan hun eigen QEMU-profiel gebonden. Dit
checkpoint claimt geen plug-and-play-boot, fysieke hardware, DMA-isolatie,
hotplug of herstel van een gecrashte driver.
