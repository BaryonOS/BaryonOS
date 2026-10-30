# Begrensde Ethernet/IPv4/UDP-, ARP- en DNS-protocollen

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

Driver, MTU/FCS/offloadbeleid, ontvangstbuffers, route-/subnetbeleid,
socket-/procesrechten, DHCP-clienttoestanden, native DNS-/ARP-integratie, TLS
en echte verbindingen blijven open.
Deze beperkte parser is geen volledige IPv4-hostimplementatie en wordt nog
niet door een native netwerkdienst aangeroepen. Er is geen nieuw
netwerk-, privacy-, hardware- of roadmapcriterium geaccepteerd.

## FR-N01: ARP en DNS als echte protocolbouwstenen

Taakcontract: bouw boven de bestaande codec een dependencyvrije, allocatorvrije
ARP-/DNS-basis voor de toekomstige native netwerkdienst. Alleen deze zelfstandige
workspace verandert; geen kernel-ABI, nieuwe bevoegdheid of rootlockfile. De
hoofdagent bezit NIC-/kernelintegratie. Er zijn geen hostnetwerkoperaties of
gesimuleerde verbindingssuccessen. Dit sluit N01/N02/N03 **niet** af.

| Interface | Gedrag en grens |
|---|---|
| `arp::parse` | Ethernet/IPv4 request/reply, bron-MAC gebonden aan Ethernet-header; replydoel in beide headers gelijk; 42 bytes zonder padding of volledig 60-byteframe, geen FCS. |
| `arp::build_request/build_reply` | Schrijft een echt RFC 826-frame met nulpadding; alle fouten vóór schrijven; reply alleen voor de geconfigureerde lokale interface. |
| `arp::NeighborSlot` | Eén expliciet gestarte aanvraag; alleen passend reply vóór de monotone deadline vult de slot. Annulering, exclusieve vervaltijd en gecontroleerde klokoverflow. |
| `dns::Name/Question/build_query` | Canonieke ASCII-hostnaam, IN/A of IN/AAAA, één vraag en RD; maximaal 255 wirebytes. Unicode/IDNA-conversie blijft een afzonderlijke invoerlaag. |
| `dns::parse_response` | Exact transaction-ID, vraagnaam/type/class, QR/opcode/flags, foutcode en alle gedeclareerde recordlengtes. TC is een expliciete fout, geen gedeeltelijk succes. |
| `dns::Response::resolve` | Alleen IN-antwoorden voor de gevraagde naam of haar gecontroleerde CNAME-keten; aanvullende glue en vreemde namen leveren geen adres. CNAME-cycli, tegenstrijdige alias/adresantwoorden en te grote sets worden geweigerd. |
| `dns::PendingQuery` | Bindt de aanvraag aan beide IPv4-adres-/poortparen, checksum en deadline. Geaccepteerde antwoorden consumeren de aanvraag; vreemde/malformed antwoorden niet. Annuleren en verlopen geven geen antwoord. |

DNS verwerkt maximaal 4.096 berichtbytes, 32 records over alle secties,
32 achterwaartse compressionpointers per naam, acht aliasstappen en acht unieke
adressen. Pointers naar de header, zichzelf, vooruit of buiten het bericht,
label/pointercycli en namen boven 255 wirebytes worden geweigerd. Een CNAME
mag niet buiten zijn RDLENGTH doorlezen. TTLs met de hoge bit gezet gelden
volgens RFC 2181 als nul; de cacheduur is maximaal 86.400 seconden en het
minimum van relevante alias-/adres-TTLs. Een nul-TTL mag niet worden gecachet.
Er wordt geen OPT/EDNS gevraagd; een ongevraagd OPT-record wordt expliciet
geweigerd, zodat een uitgebreide foutcode niet als gewone succesvolle DNS geldt.

Deze grenzen zijn een expliciet initieel codecprofiel, geen verlaagde
release-eisen. DNS-over-TCP na TC, EDNS, DNSSEC-validatie, volledige resolver-
cache en IPv6-transport blijven zichtbaar open. Een AAAA-record decoderen
bewijst geen werkende IPv6-stack.

ARP verwerkt probes met bron-IP nul als herkenbare verzoeken, maar voert nog
geen RFC 5227-adresconflictdetectie uit en antwoordt die niet via `build_reply`.
De resolver bewaart geen ongevraagde of gratuitous ARP-replies. ARP is
**niet geauthenticeerd**: een aanvaller op hetzelfde linksegment kan een passend
solicited reply vervalsen. Geen MAC/IP-paar verleent identiteit, TLS-vertrouwen
of apprechten. De netwerkdienst moet interface, netmasker, route en broadcast-
beleid afdwingen; DNS kan evenmin bestemmingscapabilities verlenen.

Acceptatie voor deze bouwsteen: onafhankelijke vaste request/reply-/DNS-wire-
fixtures, truncatie op iedere offset, ongeldige headers/identiteiten, verkeerde
doelen, expiratie, annulering, overflow, DNS-pointer- en aliasaanvallen,
RDLENGTH-grenzen, TTL en endpoint/ID-weigering; daarnaast Clippy, fmt en echte
freestanding targetcompilatie. Uitvoering en hashes:
[`verification/fr-n01-report.json`](verification/fr-n01-report.json).
Dit zijn hosttests en een targetbuild; **geen NIC-, VM- of internetbewijs**.

Eerstvolgende native koppeling: modern `virtio-net-pci` met gepinde RX/TX-
queues, expliciete bufferownership en rechten; een geïsoleerde netwerkdienst
gebruikt deze codecs met geconfigureerde interface en monotone tijd. De kernel
mag alleen begrensde I/O/capabilitygrenzen bemiddelen. Een eigen testpeer kan
op een afgebakend VM-netwerk ARP beantwoorden en DNS-fouten leveren. Pas echte
frames door de Nieuw-OS-client tellen voor native netwerkacceptatie.

Specificatiebasis: RFC 826 (ARP-velden en opcodes), RFC 1035 §§3.1, 4.1 en
4.1.4 (DNS-vragen, records en compressie), RFC 2181 §8 (TTL). Lokale
kennisoriëntatie: OSDev *Address Resolution Protocol*, W002539, revisie 11170
(5 februari 2011). Geen tutorialcode overgenomen; geen dependency toegevoegd.

## DHCPv4 — pakketcodec voor toekomstige configuratiedienst

`dhcp` voegt vaste, allocatorvrije BOOTP/DHCP-berichten toe. `build_discover`,
`build_selecting_request` en `build_renewal_request` coderen respectievelijk
DISCOVER, de gekozen OFFER-aanvraag en een bestaande lease-aanvraag. De
aanroeper levert MAC, transaction-ID en verstreken seconden; deze crate
genereert geen entropy, verstuurt geen frame en beheert geen timer.

`parse_reply` begrenst de ontvangen BOOTP/DHCP-datagramlengte op 576 bytes en
controleert BOOTREPLY, Ethernet-type/-lengte, transaction-ID, client-MAC,
cookie, optiongrenzen, einde/padding, berichttype, server-ID en leasevelden.
De eerste profielsubset leest subnetmasker, router(s), DNS, leaseduur en
optionele T1/T2; lijsten zijn begrensd op vier adressen. Ongeldige masks,
adressen, lease-timers, dubbele relevante opties en DHCP option-overload
worden geweigerd. RFC 3396-concatenatie en option-overload zijn nog niet
ondersteund; dit kan compatibiliteit met geldige uitgebreide serverantwoorden
beperken en is geen volledige DHCP-client.

De parser controleert geen UDP/IP-bron- of bestemmingsendpoint; de caller moet
dat vóór gebruik doen. Een passende transaction-ID of DHCP-server-ID is geen
serverauthenticatie: DHCP kan route/DNS-configuratie niet veilig valideren
tegen een kwaadwillende linkgenoot. De parser activeert geen lease, route of
DNS-server. Ontdekken, kiezen, ARP-adresconflictcontrole, DHCPDECLINE/NAK-
herstel, leasepersistentie, T1-renewal, T2-rebinding, linkovergang en
reconnect zijn nog niet geïmplementeerd. De kernel-ABI en capabilitygrens
veranderen niet; DHCP krijgt geen zelfstandige bevoegdheid.

Dit is een host-/freestanding-codecbouwsteen voor FR-N01, geen N01-aftekening
of Internetbewijs. Pas een native test met een protocolconforme DHCP-peer,
echte virtio-netframes, toegepaste configuratie, leaseverlenging/-verval,
verkeerde serverantwoorden en koude/hot-linkherstel kan netwerkvoortgang
onderbouwen.

## Uitvoering en bewijs

```sh
CARGO_TARGET_DIR=/tmp/nieuw-os-net-parser-target \
  cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline
CARGO_TARGET_DIR=/tmp/nieuw-os-net-parser-target \
  cargo +1.90.0 check --manifest-path services/net-protocol/Cargo.toml --tests --locked --offline
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
- R. Droms, *Dynamic Host Configuration Protocol*, RFC 2131, maart 1997,
  §§3.1, 3.2, 4.1 en 4.4.5: <https://www.rfc-editor.org/rfc/rfc2131.html>.
- S. Alexander en R. Droms, *DHCP Options and BOOTP Vendor Extensions*,
  RFC 2132, maart 1997, §§3.3, 3.9, 3.11 en 9:
  <https://www.rfc-editor.org/rfc/rfc2132.html>.
