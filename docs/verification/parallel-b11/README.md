# B11 — allocatorvrije netwerkparser

## Oplevering

Nieuwe zelfstandige workspace `services/net-protocol`, basis
`dbd35b04d742b18d66b40fdd65d8a9b900656cac`, branch
`codex/parallel-native-net-protocol-20261028`. Geen root-Cargo-, build-,
syscall-, capability-, IPC-, opslag- of kernelwijziging.

De pure `no_std`-library bevat geen dependency, allocator, unsafe of I/O.
`parse` geeft een borrowed UDP-datagram of een expliciete fout terug.
De EtherType-, IPv4-header/lengte/fragment- en UDP-lengte/checksumgrenzen
zijn gecontroleerd. Het ontbreken van een IPv4-UDP-checksum blijft als
`UdpChecksum::Absent` zichtbaar. Adressen en poorten blijven onbetrouwbare
packetdata; er worden geen rechten of afzenderidentiteiten vastgesteld.

De precieze API, beperkte paddingvorm, foutvarianten, nog ontbrekende driver/
MTU/routing/FCS/socket-/procesrechten en geraadpleegde primaire RFC-bronnen
staan in `services/net-protocol/README.md`.

## Werkelijk uitgevoerde controles

`report.json` bindt alle bron-/manifest-/lock-/testbestanden aan SHA256 en
legt de exacte commando's, uitvoertijden, exitstatussen en loghashes vast.
Alle controles zijn op 28 oktober 2026 werkelijk uitgevoerd met Rust1.90:

| Controle | Uitkomst |
|---|---|
| `cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline` | Exit 0; 18 tests: twee checksumtests en zestien packettests |
| `cargo +1.90.0 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check` | Exit 0 |
| `cargo +1.90.0 clippy --manifest-path services/net-protocol/Cargo.toml --all-targets --locked --offline -- -D warnings` | Exit 0 |
| `cargo +1.90.0 build --manifest-path services/net-protocol/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline` | Exit 0; werkelijke library voor de native target |

Eigen targetmap: `/tmp/nieuw-os-par-b11-target`; twee Cargo-buildjobs,
offline cache `/tmp/nieuw-os-par-tools/cargo-home`. De native rlib heeft
SHA256 `1d4f76e683388cabdd078163636d0bd3486c7da64537d23f08c7e310443a08ab`.
Dit is cross-compilatie, geen uitgevoerde gastfunctie of VM.

## Gerichte foutgrenzen

De vijf vaste bekende packetvectoren bevatten een oneven payload, even
payload, lege payload, ontbrekende checksum en een checksum die berekend
nul is en daarom als `0xffff` wordt verzonden. `vectors.py` leidt de
constante bytes onafhankelijk af met een afzonderlijke hostberekening en
controleert de referentiesom uit RFC1071 §3. `vectors.json` bewaart deze
afleiding; de Rusttests herberekenen de verwachte fixtures niet met de
library onder test.

De 18 methoden controleren onder meer alle **234** afgebroken prefixes van
de vijf volledige vectors, **160** enkele bitfouten in de IPv4-header,
**120** enkele bitfouten in UDP-header/payload en **64** wijzigingen in beide
pseudoheaderadressen nadat de gewone IP-checksum opnieuw correct is gemaakt.
Zij controleren daarnaast iedere IHL/versiewaarde, VLAN/andere protocollen,
fragmentoffset/MF/reserved flags, toegelaten DF, minimale/maximale/foutieve
IP- en UDP-lengtes, oneven checksumdelen, padding, extra trailers en behoud
van alle inputbytes. De synthetische IP-maximumbuffer van 65.535 bytes test
rekenkundige grenzen; hij bewijst geen fysieke jumbo-MTU.

## Grenzen en roadmap

Er zijn **nul netwerkoperaties en nul VMs** uitgevoerd. Geen driver,
verbinding, algemene native netwerkservice, cryptografische bescherming of
procesgebonden socketrecht is geïmplementeerd. De Internet checksum is
uitsluitend de standaard foutdetectie, geen eigen cryptografie.

Roadmapvoorstel: **0 punten wijziging**. Dit resultaat is een kleine
uitvoerbare library-afhankelijkheid voor een latere native driver/service.
