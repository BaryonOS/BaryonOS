# USB descriptor and interface-matching core

## Taakcontract

Doel van deze USB-bouwsteen: een kleine dependencyvrije `no_std`-parser die
de grenzen van achtereenvolgende USB-descriptors controleert en standaard
interface-descriptors decodeert voor klassematching. De bytes komen later van
een begrensde control-transfer; deze core voert zelf geen apparaat-I/O uit.

De USB 2.0-descriptorstructuur begint met `bLength` en `bDescriptorType`.
`bLength` omvat de descriptorheader zelf. De parser accepteert alleen lengtes
vanaf twee bytes die volledig binnen de aangeleverde slice vallen. Een
onvolledige header, te kleine lengte of afgekapt descriptor levert één expliciete
fout op; daarna stopt de iterator en probeert hij niet in onbetrouwbare bytes te
resynchroniseren. Onbekende descriptor-types blijven ongewijzigd beschikbaar
voor een latere policylaag.

## Grenzen en invarianten

- De caller valideert control-transferstatus en begrenst de ontvangen bytes
  voordat deze core de slice krijgt. De core kent geen devicepointer, DMA-buffer,
  controllerstatus of capability.
- De core alloceert niet, houdt geen globale toestand bij en gebruikt geen
  `unsafe`. Descriptors lenen hun ruwe bytes uit de input-slice.
- De interface-decoder accepteert alleen descriptor type 4 met de standaard
  lengte van negen bytes. Hij geeft nummer, alternate setting,
  endpointaantal en class/subclass/protocol terug. `InterfaceClassMatch`
  vergelijkt class en optionele subclass-/protocolvelden; `None` betekent
  wildcard.
- Dit is geen USB-enumeratie, configuratievalidatie, registry, HID-driver,
  mass-storage-driver, USB-hostcontroller of xHCI-driver. Een match verleent
  geen resource en start geen code. Descriptorvolgorde en apparaatbeleid
  blijven voor afzonderlijke lagen.
- Een framingfout beëindigt de iteratie; gedeeltelijk gevalideerde opvolgende
  descriptors worden niet gepubliceerd.
- Het verwijderen van dit standalone component verandert geen kernel-, IPC-,
  capability- of bestaande driverinterface.

## Acceptatie en bewijsgrens

Hosttests bestrijken lege input, geldige en onbekende types, descriptorgrenzen,
lengtes 0/1/2/255, ontbrekende headerbytes, afgekorte descriptors, terminale
foutafhandeling, interfacevelden, class/subclass/protocol-wildcards en
ongeldige interface-descriptortypen/-lengtes. Controleer daarnaast
formattering, Clippy en een `x86_64-unknown-none`-librarybuild met de
vastgezette Rust 1.90.0-toolchain.

Deze bouwsteen levert geen H02-punt op: er is nog geen USB-hostcontroller die
een apparaat enumereert of gebeurtenissen aan een proces levert. xHCI-ringen,
MMIO-/DMA-beleid, time-outs/reset, hotplug, rechten, QEMU-apparaten en fysieke
referentiehardware blijven afzonderlijke vervolgstappen.

## Primaire bron

- USB Implementers Forum, *USB 2.0 Specification*, descriptorbeschrijving
  §9.5 en interface descriptor §9.6.5. De officiële documentpagina is
  bijgewerkt op 3 juni 2025 en koppelt de
  specificatie met bijbehorende errata/ECN's:
  <https://www.usb.org/document-library/usb-20-specification>.
  De pagina linkt het officiële archief
  <https://www.usb.org/sites/default/files/usb_20_20250603.zip>.
  Geen specificatietekst of implementatiecode is overgenomen.

## Uitgevoerde controle — 30 oktober 2026

Op broncommit `f8e7490b1dd8fdeb3cd8e8eb8578535bc49c5cc6` zijn de hosttests
(7/7), Clippy, formattering en de release-librarybuild voor
`x86_64-unknown-none` geslaagd. Exacte commando's, logs en de hash van de
gebouwde library staan in
[`verification/usb-descriptor-core-f8e7490/`](../verification/usb-descriptor-core-f8e7490/).
Er is geen native QEMU-USB-run of fysieke hardwaretest uitgevoerd.
