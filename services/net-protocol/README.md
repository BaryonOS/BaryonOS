# Begrensd Ethernet/IPv4/UDP-protocol

`nieuw-os-net-protocol` is een zelfstandige Rust-workspace met één
allocatorvrije `no_std`-library, `forbid(unsafe_code)` en geen dependencies.
Zij valideert ontvangen bytes en codeert uitgaande frames, maar opent geen
device, socket, verbinding of netwerkrecht en verstuurt niets.

## Interface en invariant

```rust
use nieuw_os_net_protocol::{parse, Error};

fn payload_length(frame: &[u8]) -> Result<usize, Error> {
    let packet = parse(frame)?;
    Ok(packet.payload().len())
}
```

`parse(&[u8]) -> Result<UdpDatagram<'_>, Error>` geeft uitsluitend na alle
controles een onveranderlijke view terug. `frame`, `ethernet_header`,
`ipv4_packet`, `udp_packet`, `payload` en `ethernet_padding` lenen de originele
bytes; de parser verandert of kopieert de payload niet. MAC-/IPv4-adressen,
poorten, TTL, DF en de checksumstatus zijn via getters leesbaar. De private
resultaatvelden kunnen niet met ongevalideerde slices worden geconstrueerd.

De caller moet het buffer gedurende de borrow stabiel en onveranderlijk
houden. Een toekomstige DMA-driver moet dus eerst werkelijk bufferbezit
overdragen; deze library beheert geen DMA, MMIO of devicegeheugen.

## Geaccepteerde subset

| Laag | Gecontroleerd gedrag |
|---|---|
| Ethernet II | 14-byteheader, EtherType IPv4 (`0x0800`). VLAN C-/S-tags en andere EtherTypes geven expliciet `Unsupported`. |
| IPv4 | Versie 4, IHL precies 5, total_length inclusief de 20-byteheader binnen de beschikbare bytes, juiste headerchecksum, reserved flag nul. |
| Fragmentatie/options | MF of enige fragmentoffset wordt afgewezen als `Unsupported`; DF mag aan of uit. IHL boven 5 is expliciet unsupported, onder 5 ongeldig. |
| Bovenliggend protocol | Alleen protocol 17 (UDP). Andere protocollen worden niet als UDP geïnterpreteerd. |
| UDP | Minimaal acht bytes; de UDP-lengte moet exact gelijk zijn aan de gehele IPv4-payload. Een aanwezig checksumveld wordt met de IPv4-pseudoheader gecontroleerd. |
| UDP-checksum nul | `UdpChecksum::Absent`, nooit `Valid`. Een correct berekende nulwaarde wordt als `0xffff` op de wire herkend en gecontroleerd. |

De input begint bij de Ethernet-destination-MAC en bevat **geen FCS**.
De caller mag Ethernet-padding hebben gestript. Als padding aanwezig is,
accepteert dit eerste profiel alleen de complete Ethernet-minimumlengte
van 60 bytes zonder FCS. Andere trailers of gedeeltelijke padding worden
afgewezen. De paddingbytes blijven afzonderlijk zichtbaar en vallen buiten
de IP- en UDP-lengte/checksum. De parser behandelt hun inhoud als opaque;
zij worden nooit als UDP-payload teruggegeven.

Alle checksumwoorden zijn big-endian. De implementatie volgt de standaard
Internet checksum met end-around carry en een nulpad na een oneven laatste
octet. Vouwen na ieder woord houdt de accumulator begrensd. De IPv4-lengte
begrenst het lees-/checksumwerk tot 65.535 bytes; de extra Ethernet-offset
wordt gecontroleerd op overflow. De tests op deze maximale lengte zijn
synthetische parsergrenscontroles, geen claim dat de latere Ethernet-driver
zulke frames of een bepaalde jumbo-MTU ondersteunt.

`Error` onderscheidt afgebroken Ethernet/IP/UDP, unsupported features,
ongeldige IP-header-/totallengte/flags/checksum, ongeldige of niet-passende
UDP-lengte/checksum, onverwachte Ethernet-lengte en rekenkundige overflow.
De library repareert geen checksum, lengte, fragment of payload.

## Uitgaande pakketten (voorbereiding N01)

Naast de bestaande parser kan de library met `build_udp_ipv4(tx, payload, output)`
nu een IPv4/UDP-datagram in een door de aanroeper beheerde buffer coderen.
`UdpIpv4Tx` bevat MAC- en IP-adressen, poorten, TTL, identificatie en
expliciete effectieve IPv4-MTU. De encoder plaatst IPv4- en UDP-checksums
(ook de RFC 768-nul-als-`0xffff`-regel), zet DF, schrijft exact de
60-byte-Ethernet-minimumlengte waar nodig, en geeft de te verzenden lengte terug.

**Geen enkele fout schrijft gedeeltelijke data:** een te klein buffer, een
te groot payload, TTL nul of overschreden MTU wordt vooraf afgewezen.
Een succesvolle aanroep laat alles na de geretourneerde lengte ongemoeid.
De netwerkdienst moet MTU, afzenderadressen, bestemmingsrechten, route en
naburige MAC-identiteit zelf afdwingen. Deze functie doet geen ARP,
fragmentatie, transmissie, devicebeheer, IP/TCP-stack of privilegecontrole.

Vaste onafhankelijk afgeleide bestaande RX-vectoren worden nu ook als TX-orakel
gebruikt. Negatieve tests bestrijken MTU, te kleine output, payloadoverschrijding,
nul-TTL, bytebehoud bij fout, maximaal IPv4-formaat en checksumcorruptie.
De tests zijn hosttests; pas een afzonderlijke kernel-/NIC-/QEMU-gate kan de
native netwerkvereisten van N01 onderbouwen. De voortgangsmeter blijft
ongewijzigd.

## Autoriteit en nog ontbrekende functies

De Internet checksum is foutdetectie, **geen authenticatie of cryptografie**.
Pakketadressen en poorten leveren geen betrouwbare afzenderidentiteit en
verlenen geen capability. Ook een `Valid` checksum mag geen socket-, app-,
NAS- of bestemmingsrecht bepalen. De nul-checksumstatus moet zichtbaar
blijven voor een toekomstige expliciete servicekeuze.

Driver, MTU/FCS/offloadbeleid, ontvangstbuffers, ARP, routes/TTL-beleid,
socket-/procesrechten, TCP, DNS, TLS en echte verbindingen blijven open.
Deze beperkte parser is geen volledige IPv4-hostimplementatie en wordt nog
niet door een native netwerkdienst aangeroepen. Er is geen nieuw
netwerk-, privacy-, hardware- of roadmapcriterium geaccepteerd.

## Uitvoering en bewijs

```sh
CARGO_TARGET_DIR=/tmp/nieuw-os-net-parser-target \
  cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline
CARGO_TARGET_DIR=/tmp/nieuw-os-net-parser-target \
  cargo +1.90.0 clippy --manifest-path services/net-protocol/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo +1.90.0 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check
CARGO_TARGET_DIR=/tmp/nieuw-os-net-parser-target \
  cargo +1.90.0 build --manifest-path services/net-protocol/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline
```

De vaste synthetische packetvectoren worden niet door de library onder test
gegenereerd. Hun afzonderlijke Python-afleiding, RFC-checksumreferentie,
exacte uitvoering en bron-/artifact-hashes staan in
`docs/verification/parallel-b11/`. Hosttests en compilatie voor de native
target blijven afzonderlijk gelabeld; er wordt geen VM of netwerk gebruikt.

## Primaire bronnen

Geraadpleegd 28 oktober 2026:

- J. Postel / ISI, *User Datagram Protocol*, RFC 768, 28 augustus 1980,
  Fields: <https://www.rfc-editor.org/rfc/rfc768.html>.
- DARPA / ISI, *Internet Protocol*, RFC 791, september 1981, §3.1:
  <https://www.rfc-editor.org/rfc/rfc791.html>.
- R. Braden, D. Borman en C. Partridge, *Computing the Internet Checksum*,
  RFC 1071, september 1988, §§1 en 3:
  <https://www.rfc-editor.org/rfc/rfc1071.html>. De vaste voorbeeldsom is
  `0xddf2`; de complement-checksum is `0x220d`.
- C. Hornig, *A Standard for the Transmission of IP Datagrams over Ethernet
  Networks*, RFC 894, april 1984, Frame Format:
  <https://www.rfc-editor.org/rfc/rfc894.html>, met geverifieerde errata
  570/5141: <https://www.rfc-editor.org/errata_search.php?rfc=894>.
- IANA, *IEEE 802 Numbers*, versie 3 november 2024:
  <https://www.iana.org/assignments/ieee-802-numbers/>; labels van de twee
  expliciet geweigerde VLAN-tagtypen.
