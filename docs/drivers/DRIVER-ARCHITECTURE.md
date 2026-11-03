# Driver Core en selectie

Status: eerste broncodeversie op `copilot/universal-driver-framework`.
De registry en interfaces bouwen; algemene PCI-enumeratie, toekenning van
devicebronnen en driverstart zijn nog niet geïmplementeerd.

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
- een vaste-capaciteit `DriverRegistry` die een uniek meest specifiek
  registratie-resultaat kiest en gelijke topmatches weigert.

De huidige `virtio-blk-driver` en `virtio-net-driver` publiceren een
`DRIVER_REGISTRATION` voor respectievelijk `1af4:1042` en `1af4:1041`.
Dat is alleen matchmetadata. De bestaande QEMU-transporten en kernelpaden
blijven eigenaar van discovery, MMIO, bus mastering en DMA; ze gebruiken de
registry nog niet om de driver te starten.

```text
PCI-enumerator (nog niet generiek)
    -> PciDeviceDescriptor
    -> DriverRegistry::best_match
    -> unieke DriverRegistration of geen driver / ambiguïteit
    -> resourcegrant en driverstart (nog niet beschikbaar)
    -> subsysteminterface
```

ArceOS' categoriegerichte traits en `DriverProbe`/`init_drivers`-structuur
dienden als architectuurreferentie; er is geen ArceOS-code overgenomen.
De Nieuw OS-kern houdt matching, resourceautorisatie en starten gescheiden:
een PCI-match verleent geen toegang tot BAR's, interrupts of DMA.

## Nog benodigde architectuur

1. Vervang de afzonderlijke vaste-BDF-detecties pas na regressietests door één
   begrensde PCI-enumerator die veilige headers en subsystemvelden verzamelt.
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
