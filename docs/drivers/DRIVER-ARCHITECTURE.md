# Driver Core en selectie

Status: eerste generieke PCI-discovery en VirtIO-selectie zijn geïmplementeerd
op `copilot/universal-driver-framework`. De actuele code enumereert
segment 0, matcht PCI-descriptors en laat VirtIO block/net hun apparaat op
basis van de registry selecteren. Een devicegebonden PCI-configadapter en
kernel-owned BAR/MMIO-grants voor block/net zijn toegevoegd. De bredere
resourcebroker en factorygestuurde activatie ontbreken nog.

## Huidige implementatie

`crates/driver-core` is een dependencyvrije `no_std`-crate zonder `unsafe`.
Hij definieert:

- gemeenschappelijke `DeviceId`, `DeviceClass`, `DriverId`, `DriverVersion`,
  `DriverState` en `DeviceMetadata`;
- categorie-identifiers en subsystemtraits voor block-, netwerk-, display- en
  inputdevices, plus identifiers voor USB-hostcontrollers, audio, storage- en
  seriële devices;
- PCI-identiteit inclusief BDF, vendor/device, class/subclass/programming
  interface/revision en optionele subsystem-ID's;
- een bounds-safe decoder voor vaste 256-byte type-0 PCI-configuratiesnapshots;
- een vaste-capaciteit `DriverRegistry` die een uniek meest specifiek
  registratie-resultaat kiest en gelijke topmatches weigert.

`crates/pci-core` enumereert conventionele PCI-bussen op segment 0 met
configuratiemechanisme 1. Hij bezoekt functies 1-7 alleen wanneer functie 0
het multifunction-bit aangeeft, negeert niet-aanwezige functies en rapporteert
alleen type-0-endpoints aan de caller. De scanner leest configuratieruimte
voordat de descriptor aan de registry wordt aangeboden. De huidige scan
ondersteunt geen ECAM, segmenten buiten 0, bridge-topologie of hotplug.

De VirtIO block- en netwerkdrivers registreren hun PCI-ID's en selecteren hun
apparaat met `DriverRegistry::best_match`, in plaats van een vooraf vastgelegde
BDF. BAR-probing en verdere toegang blijven beperkt tot de geselecteerde
functie. Als meer dan één passend device voorkomt terwijl het kernelpad nog
maar één instantie ondersteunt, wordt dat als fout geweigerd.

`MatchedPciFunction` bindt config-space-toegang aan een opnieuw gevalideerde
descriptor en `DriverMatch`. Reads en writes naar een ander BDF worden
geweigerd; stale descriptors en registraties die niet bij het device passen,
worden niet gebonden. Dit is een concrete devicegrens voor PCI-configuratie.
`MatchedPciFunction` kan na matching de memory-BARs peilen. Daarbij blijven
I/O-decode, memory-decode en bus mastering uitgeschakeld; de probe herstelt
BAR-registers ook bij fouten. Een `PciBarGrant` bindt het geselecteerde BDF,
de driver-ID, BAR-index en gevalideerde base/lengte. Block en netwerk krijgen
zo'n grant vóór mapping. `Paging::map_pci_bar` weigert een ander device, een
andere driver of BAR-index en laat uitsluitend volledige 4-KiB-pagina's
binnen die BAR mappen. De bestaande PAT-UC-, CPU-MAXPHYADDR- en
supervisor-NX-controles blijven actief. Dit is een begrensde kernelinterne
grant, geen userspace capability of driverisolatie. IRQ-toewijzing,
busmaster-toestemming na initialisatie en DMA-buffers blijven buiten deze
broker. Alle bestaande block-kernelprofielen alloceren hun DMA-pool pas na
een succesvolle discovery- en registrymatch; de netwerkroute volgde deze
volgorde al.

```text
PCI segment-0 scan
    -> type-0 DeviceDescriptor
    -> DriverRegistry::best_match
    -> apparaatgebonden configadapter en BAR-grants
    -> kernel valideert grant en maakt beperkte UC/NX-mapping
    -> bestaande VirtIO-probe en initialisatie
    -> bestaande block-/netwerk-I/O
```

ArceOS' categoriegerichte traits en `DriverProbe`/`init_drivers`-structuur
dienden als architectuurreferentie; er is geen ArceOS-code overgenomen. De
Nieuw OS-kern houdt matching, resourceautorisatie en starten gescheiden: een
PCI-match verleent op zichzelf geen toegang tot BAR's, interrupts of DMA.

## Nog benodigde architectuur

1. Breid de kernel-owned resourcegrens uit naar expliciete BAR-subregio's voor
   de overige PCI-drivers. De block-/netwerkpaden hebben nu een matched BAR-
   grant, maar de mappingcode en deviceinitialisatie zijn nog niet één
   algemene resourcebroker. Integreer interrupt lifecycle en DMA met de
   maturitybranch-semantiek; maak geen concurrerende capability- of
   IOMMU-modellen.
2. Koppel registryresultaten aan concrete driverfactories die alleen verleende
   resources ontvangen. Startfouten leveren geen `Ready`-metadata op;
   onbekende of ambigue devices worden niet gestart. De bestaande block- en
   netwerkinitialisatie is nog geen factorygestuurde plug-and-play-activatie.
3. Behoud de bestaande VirtIO-policy-, DMA-, timeout-, reset- en
   quarantainelogica bij het verplaatsen naar die resourcegrens.
4. Voeg alleen de subsysteminterfaces toe die concrete drivers nodig hebben.
   USB vraagt een afzonderlijke enumeratie-, class-matching- en
   resourcegrens; de huidige PCI-registry beweert USB niet te ondersteunen.
5. Breid discovery gecontroleerd uit naar bridge-topologie, ACPI/ECAM,
   aanvullende segmenten en later hotplug, met tests voor iedere nieuwe
   toegangspad.

Alleen PCI BAR-grants voor de bestaande block-/netwerkpaden zijn geïntegreerd;
er is nog geen algemene BAR/MMIO-, interrupt- of DMA-resourcebroker en geen
factorygestuurde driverregistratie. De huidige PCI-code is nog geen volledige
plug-and-play-keten. Deze status claimt geen fysieke hardware, DMA-isolatie,
hotplug, USB, GPU-acceleratie of herstel van een gecrashte driver.
