# Driver Core en selectie

Status: generieke segment-0-PCI-discovery, registratie en factorygekoppelde
initialisatie zijn geïmplementeerd voor VirtIO block, netwerk en RNG. Alle drie
selecteren een functie via `pci-core` en hun eigen registry, met config-space
gebonden aan de match. Block, netwerk en RNG krijgen kernel-owned BAR-grants
vóór mapping; de bestaande transports behouden hun eigen I/O- en DMA-logica.
Daarnaast bestaan losse, niet-geïntegreerde NVMe- en xHCI-probecrates die PCI
matchen en alleen controllerregisters read-only inspecteren. Een gedeelde
heterogene registry, interruptbroker en generieke DMA/IOMMU-resourcebroker
ontbreken nog.

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

M5-kernelprofielen doen vóór device-initialisatie een extra read-only scan.
Functies zonder match in de voor dat profiel geregistreerde drivers worden op
COM1 gemeld als `UNSUPPORTED:PCI`, met segment, bus/BDF, vendor/device,
class/subclass/programming interface/revision en subsystem-ID's wanneer
beschikbaar. Niet-type-0-functies worden alleen als aantal gemeld; een
onvolledige configuratiescan wordt expliciet gemeld en levert geen
unsupported-records op. Dit is een diagnostische matchinventaris, geen bewijs
dat een gematchte driver succesvol is geïnitialiseerd of dat alle PCI-driver-
families in iedere profielregistry zijn opgenomen. De RNG-driver wordt nu ook
via segment-0-enumeratie en een geregistreerde factory gestart; de huidige
QEMU-fixture gebruikt BDF `00:06.0`, maar de driver vereist die locatie niet.

De VirtIO block-, netwerk- en RNG-drivers registreren hun PCI-ID's en
selecteren hun apparaat met `DriverRegistry::best_match_with_factory`, in
plaats van een vooraf vastgelegde BDF. BAR-probing en verdere toegang blijven
beperkt tot de geselecteerde functie. Als meer dan één passend device voorkomt
terwijl het kernelpad nog maar één instantie ondersteunt, wordt dat als fout
geweigerd.
De activatiepaden registreren daarnaast per match een statische
initializer-functie. `best_match_with_factory` retourneert metadata plus die
initializer; de door discovery gemaakte waarde bewaart hem privé. Na kernel-
toekenning van bestaande BAR-mappings en DMA-pagina's roept
`Device::initialize` de geselecteerde factory aan. Familiespecifieke opties
en de bestaande reset-/quarantainelogica blijven bij de concrete initializer.
Er wordt geen drivercode dynamisch geladen, heapobject gemaakt of extra
resource-authority gecreëerd; resources blijven kernel-toekenningen.

`drivers/nvme` en `drivers/xhci` gebruiken ieder een lokale een-entry PCI-
registry en een devicegebonden BAR-grant voor een beperkte read-only probe.
De NVMe-crate leest `CAP`/`VS` volgens NVM Express Base Specification 2.4,
§§3.1.1–3.1.2. De xHCI-crate matcht de USB-hostcontrollerclass volgens PCI-SIG
PCI Code and ID Assignment Specification 1.10, §1.2, en controleert revision
en registeroffsets volgens de Intel xHCI Requirements Specification 1.2c,
§§5.2–5.3. Geen van beide maakt deel uit van de kernel-activatieketen: ze
schakelen bus mastering niet in en allociëren geen DMA. Alleen de USB
descriptor-core matcht interface class/subclass/protocol volgens USB 2.0
Specification §9.6.5; ook die selectie start nog geen classdriver.

Primaire bronnen: NVM Express, *NVM Express Base Specification*, revision 2.4,
31 juli 2026, <https://nvmexpress.org/specification/nvm-express-base-specification/>;
PCI-SIG, *PCI Code and ID Assignment Specification*, revision 1.10,
28 november 2017, <https://pcisig.com/PCIExpress/Specs/Base/CodeandIDAssignment_1.10>;
Intel, *eXtensible Host Controller Interface for Universal Serial Bus (xHCI)
Requirements Specification*, revision 1.2c, oktober 2025,
<https://cdrdv2-public.intel.com/868295/xHCI__Rev1.2c.pdf>; USB Implementers
Forum, *USB 2.0 Specification*, descriptorbeschrijving §9.6.5,
<https://www.usb.org/document-library/usb-20-specification>.

`MatchedPciFunction` bindt config-space-toegang aan een opnieuw gevalideerde
descriptor en `DriverMatch`. Reads en writes naar een ander BDF worden
geweigerd; stale descriptors en registraties die niet bij het device passen,
worden niet gebonden. Dit is een concrete devicegrens voor PCI-configuratie.
`MatchedPciFunction` kan na matching de memory-BARs peilen. Daarbij blijven
I/O-decode, memory-decode en bus mastering uitgeschakeld; de probe herstelt
BAR-registers ook bij fouten. Een `PciBarGrant` bindt het geselecteerde BDF,
de driver-ID, BAR-index en gevalideerde base/lengte. Block, netwerk en RNG
krijgen zo'n grant vóór mapping. `Paging::map_pci_bar` weigert een ander device, een
andere driver of BAR-index en laat uitsluitend volledige 4-KiB-pagina's
binnen die BAR mappen. De bestaande PAT-UC-, CPU-MAXPHYADDR- en
supervisor-NX-controles blijven actief. Dit is een begrensde kernelinterne
grant, geen userspace capability of driverisolatie. IRQ-toewijzing,
busmaster-toestemming als afzonderlijke grant en DMA-bufferbeheer blijven
buiten deze broker. Block- en RNG-kernelprofielen alloceren hun DMA-pool pas
na een succesvolle discovery- en registrymatch; de netwerkroute volgde deze
volgorde al.

```text
PCI segment-0 scan
    -> type-0 DeviceDescriptor
    -> DriverRegistry::best_match_with_factory
    -> apparaatgebonden configadapter en BAR-grants
    -> kernel valideert grant en maakt beperkte UC/NX-mapping
    -> geselecteerde VirtIO-factory
    -> kernel valideert devicegebonden BAR-grants en DMA-eigendom
    -> bestaande block-, netwerk- of RNG-I/O
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
2. Breng de per-familie factories samen in een gedeelde registry met een
   getypeerde resourcecontext, en koppel activatiestatus/cleanup aan de
   lifecycle. De huidige statische factoryfuncties activeren alleen de
   bestaande block-, netwerk- en RNG-initializers; zij vormen nog geen algemene,
   heterogene plug-and-playregistry.
3. Behoud de bestaande VirtIO-policy-, DMA-, timeout-, reset- en
   quarantainelogica bij het verplaatsen naar die resourcegrens.
4. Voeg alleen de subsysteminterfaces toe die concrete drivers nodig hebben.
   USB vraagt een afzonderlijke enumeratie-, class-matching- en
   resourcegrens; de huidige PCI-registry beweert USB niet te ondersteunen.
5. Breid discovery gecontroleerd uit naar bridge-topologie, ACPI/ECAM,
   aanvullende segmenten en later hotplug, met tests voor iedere nieuwe
   toegangspad.

PCI BAR-grants en statisch gekoppelde initializers voor block-, netwerk- en
RNG-paden zijn geïntegreerd; er is nog geen algemene BAR/MMIO-, interrupt-
of DMA-resourcebroker. De huidige PCI-code is nog geen volledige
plug-and-play-keten. Deze status claimt geen fysieke hardware, DMA-isolatie,
hotplug, USB, GPU-acceleratie of herstel van een gecrashte driver.
