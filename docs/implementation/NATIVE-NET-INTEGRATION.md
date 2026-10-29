# TASK-NATIVE-NET — echte packets in het gezamenlijke systeem

30 oktober 2026, werkpakket voor FR-N01/FR-N02/FR-H03 en FR-K03.
Dit is een gerichte native integratiestap; de volledige netwerk-/TLS-/NAS-
acceptatie in FULL-TEST-RELEASE.md blijft verplicht.

## Grens en eigenaarschap

Het opt-in profiel `native-system-net` voegt een echte modern virtio-net op
PCI `00:07.0` toe aan de bestaande init-, sleutel-, opslag- en appprocessen.
Een afzonderlijke ring-3-netwerkproef krijgt als enige `NetworkSource`.
App/processupervisie blijft werken terwijl die proef packets verwerkt.
Het bestaande `native-system` verandert niet in een netwerkprofiel.

Kernelworker bezit kernel, capability-core en SDK-runtime. Netwerkworker bezit
de ring-3-proef en de afgebakende externe protocoltegenpartij. Root bezit
centrale build/runtools, gedeelde manifests/locks, integratie en eindbewijs.

Het nieuwe kernelobject heeft `NETWORK=2048`, geen derive/transfer/revoke;
sluiten en owner-exit beëindigen de autoriteit. Dit raw device-object is alleen
voor de netwerkdienst, niet de uiteindelijke app-/bestemmingsgrant. Gewone apps
krijgen geen raw packets, DMA of algemene netwerkbevoegdheid.

## Native ABI en apparaatgrens

ABI-minor 1.5 voegt typed SDK-operaties toe, zonder Message320 te wijzigen:

- `0x70`: send(cap, pointer, lengte, timeout 1..500 ticks) → ticket.
- `0x71`: poll_tx(cap, ticket) → verstuurde framebytes of EAGAIN.
- `0x72`: receive(cap, pointer, capaciteit) → volledig frame of EAGAIN.

Lengte maximaal 1514 bytes. Alle gebruikersmappings en rechten worden vóór
publicatie of bufferwijziging gecontroleerd, ook over paginagrenzen. Gegevens
worden gekopieerd naar/vanaf vier permanent gepinde driverpagina's. De
privileged code verwerkt geen ARP-, IP-, UDP- of DNS-beleid. De huidige kleine
broker plus virtio-driver blijft wel onderdeel van de onderzochte TCB; native
userspacedriver-/IOMMU-isolatie wordt niet hierdoor geclaimd.

Alleen het nieuwe profiel krijgt zijn begrensde UC/NX-MMIO-venster vanaf
`0x3f100000`. MAC `02:00:00:00:00:07` en VERSION_1/MAC/STATUS-keuzes worden
daadwerkelijk gecontroleerd. Een RX-wachtperiode is geen veilige DMA-reset.
Een onzekere TX wordt niet herhaald; geheugen blijft gepind tot reset-ACK of
quarantaine. Completion betekent devicecompletion, geen ontvangst door de peer.

## Tegenpartij en acceptatie

QEMU verbindt uitsluitend via een eigen loopbacksocket met de door de runner
gestarte testserver; geen TAP, bridge, routeraanpassing of default gastnetwerk.
De server spreekt het Ethernet-frameprotocol, levert testantwoorden en bewaart
zijn daadwerkelijke transcript. Hij voert geen OS-client, naamresolver of
rechtencontrole voor de gast uit.

De gast voert echte ARP, UDP-echo en DNS-A-resolutie uit met synthetische
192.0.2.0/24-adressen. Verkeerde afzender/target, verkeerd DNS-ID,
compressielus en een niet-beantwoorde query geven begrensde fouten. Het cache-
resultaat mag niet door verkeerde ARP-antwoorden worden overschreven.
De proef controleert ook captype, pointers, te grote frames, ticket nul en
gebruik na sluiten. De onafhankelijke peercontrole vergelijkt echte packets;
de systeemgate behoudt daarnaast zijn disk-, crypto- en cleanupvoorwaarden.

Verwachte aanvullende cijfers: ARP_REPLIES=1, UDP_ECHOES=1, DNS_ANSWERS=1,
DENIED=7, TX_COMPLETIONS=8, RX_FRAMES=7, ARP_REJECTS=2, DNS_REJECTS=2,
RX_TIMEOUTS=1, CACHE_INTACT=1, STALE_CAP=1 (prefix `NET_`). Die waarden zijn
vooraf vastgelegde eisen, geen reeds uitgevoerde resultaten.

## Open grenzen

Geen volledige N01/N02-aftekening: TCP, gebruikersconfiguratie, routes,
algemene resolver, TLS/PQC/tijd/certificaten, appbestemmingsbeleid,
netwerkherstel, NAS en internetgebruik moeten nog native worden geïntegreerd.
Het vaste VM-profiel bewijst geen wifi, Bluetooth, fysieke DMA/IOMMU of
algemene hardwareondersteuning. Bron-/image-/packet-/loghashes en negatieve
gevallen zijn verplicht vóór deze deelstap als native geslaagd telt.
