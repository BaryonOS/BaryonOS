# Driverbeveiligingsgrenzen

## Wat de nieuwe core wel en niet afdwingt

`driver-core` is een safe Rust `no_std`-crate met `#![forbid(unsafe_code)]`.
De registry verwerkt uitsluitend een PCI-identiteitsdescriptor die door een
vertrouwde discoverylaag moet worden aangeleverd. Een match zegt alleen:
"deze registratie claimt deze PCI-identiteit". De match kent geen geheugen,
MMIO, I/O-poorten, IRQ's, DMA, resetmacht of capability toe en start geen code.

Een lege match geeft `None`; een gelijke hoogste specificiteit geeft
`AmbiguousMatch`. Beide gevallen moeten geen driverstart veroorzaken.
Subvendor-/subdevice-selectors zijn alleen geldig naast een exact
vendor/device-paar. Class selectors volgen de class→subclass→programming-
interface-hiërarchie. Registryvolte, dubbele driver-ID's en ongeldige
selectors leveren expliciete fouten.

`PciDeviceDescriptor::from_type0_config` leest alleen een vaste snapshot van
256 bytes, controleert BDF-grenzen, afwezigheid en header type, en decodeert
geen BAR- of resourcebevoegdheid. VirtIO block, netwerk en RNG gebruiken de
segment-0-enumerator en hun registratie om één endpoint te selecteren; hun
descriptor en match worden opnieuw gevalideerd na BAR-probing.

## Resourcegrens

Een volledige generieke resource/HAL-adapter is nog niet geïmplementeerd. De
block-, netwerk- en RNG-paden gebruiken wel begrensde `PciBarGrant`-waarden
voor hun eigen registrymatch. Het kernelcontract heeft geen algemene
capability voor willekeurige PCI-functions, BAR's, interrupts of DMA; de
bestaande virtio-drivers behouden hun concrete profielpaden en gepinde
buffers. Die paden worden niet vervangen door ruwe `phys_to_virt`, raw
pointers of een permissive `unsafe Hal`-bridge.

Een volgende resource-interface moet uitsluitend kernel-uitgegeven opaque
grants accepteren. Vereist zijn:

- BAR-subregiobounds, toegangsbreedte/alignment en cachetypecontrole;
- PCI-configuratie alleen voor de geautoriseerde function en veilige registers;
- interruptobjecten gebonden aan de gekozen devicefunctie;
- DMA-bufferownership, richting, lengte en lifetime; reset-quarantaine tot
  een geverifieerde quiescence;
- optionele IOMMU-domain achter hetzelfde contract, zonder onbeheerde DMA als
  tijdelijke fallback;
- geen algemene fysieke/geheugentoegang, filesystem, credentials of
  netwerkpolicy.

Bus mastering voor RNG wordt via zijn aan de gematchte functie gebonden
configadapter ingeschakeld na mapping en wordt bij reset uitgeschakeld; dit is
nog geen afzonderlijke kernel-uitgegeven busmastergrant of algemene
resourcebroker. De huidige QEMU-drivercode en hostmodellen beschermen niet
tegen een kwaadaardig device en bewijzen geen fysieke IOMMU-afdwinging. Alle
devicevelden blijven onbetrouwbaar; grootte, overflow, alignment, ringindices,
status, timeouts en failure cleanup moeten vóór gebruik worden gecontroleerd.
Een driverfout mag geen `Ready`-toestand of valse completion opleveren.

## Integratiegrenzen

De kernel/capability-engine blijft eigenaar van apparaatresources en apps
krijgen geen rechten doordat een driver in de registry staat. De nieuwe
category traits zijn in-memory servicecontracten, geen IPC-ABI of
cross-process capability. De bestaande Block/Network/Display/Input-capabilities
blijven ongewijzigd. Geen usermode-driverloader, hotplugpad, automatisch
driverherstel of generieke "UNSUPPORTED DEVICE"-bootdiagnostiek is al gekoppeld.
