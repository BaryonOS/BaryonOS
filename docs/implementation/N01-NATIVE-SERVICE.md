# N01 — geïsoleerde native netwerkdienst

## Taak en bewijsgrens

Basis `0fe648b87d1f4959087d3f80a251a7d6fb64dd66`, branch
`codex/n01-native-service-20261102`. Doel: herbruikbare, afzonderlijke ring-3
netwerkdienst met twee onafhankelijke IPC-apps. Alleen de dienst krijgt NIC en
entropy. TCP gebruikt de bestaande `net-stack` met smoltcp0.14.0; UDP de
bestaande net-protocol encoder/parser en gesolliciteerde ARP-cache. Geen nieuwe
registrycrate of bestaande kernel als basis. Kernel blijft Rust1.90; deze
appworkspace gebruikt de al toegestane Rust1.91.1, zonder rlib-uitwisseling.

Niet inbegrepen: TLS/PQC-handshake(N03), DHCP/dagelijkse internetconfiguratie(N02),
algemene dynamische policy-issuer of netwerkintrekking(N04), fysieke wifi of
nieuwe syscalls. Een IP/MAC-adres is geen cryptografische peeridentiteit. De
normale proof gebruikt uitsluitend synthetische packets via de QEMU-testpeer.

Bij deze eerste codecheckpoint: service en SDK compileren op de host, native
bins controleren op x86_64-unknown-none. De volledige hostnegatieven, gelinkte
ELF-build en geïntegreerde native gate zijn nog in uitvoering, niet geslaagd
verklaard. Root integreert de kernel/build/runners; dit document kent geen punten toe.

## Vertrouwensgrens en interfaces

- Kernelbootstrap bindt twee werkelijke app-PIDs en per app een apart RECV en
  vast SEND-replyendpoint. Onbetrouwbare requests leveren geen PID/replycap.
- Iedere app mag uitsluitend zijn door bootstrap gekozen IPv4 en vijf TCP-poorten,
  plus één UDP-poort gebruiken. Apps krijgen geen NetworkSource/Entropy.
- Eén TCP-socket per app,4096B RX/TX, maximaal208B data per IPC. UDP heeft één
 208B ontvangstslot per app; bij backpressure wordt de oude datagram niet overschreven.
- Alle datagrammen zijn onbeschermde transportbytes. Standaard geen listen/server,
  IPv6, IP-fragmentatie, hostsocket of stille fallback.
- TCP-verbindingtokens zijn generatie+slot en worden altijd tegen de werkelijke
  afzender én service-epoch gecontroleerd. Ze zijn geen zelfstandige capabilities.
- SDK en service herhalen geen eenmaal gequeue-de IPC-mutatie na een verloren
  antwoord. Een netwerk-TX-slot wordt pas vrijgegeven na de echte NIC-completion.
- Serviceverwerking is begrensd: één packetstap en maximaal één request/antwoord
  per app per beurt; een volle antwoordqueue van A blokkeert B niet.

NET1-tag is `u64::from_le_bytes(*b"NETRPC01")`, payload exact256B:
`NET1`@0, versionu16=1@4, operationu16@6 (`|0x8000` bij reply), requestu64@8,
epochu64@16, connectionu64@24, statusu32@32, argumentu32@36, IPv4@40,
portu16@44, reserved0@46, data208B@48. Alle ongebruikte bytes moeten0 zijn.
Ops1OpenTcp,2SendTcp,3ReceiveTcp,4CloseTcp,5State,6SendUdp,7ReceiveUdp,8Retire.
Open.argument is timeout1..6000ticks(100Hz); Send.argument datalengte≤208.
State geeft Closed0,Connecting1,Established2,PeerClosed3,Closing4,TimedOut5.
Retire sluit alleen de eigen appautoriteit; geen app kan de dienst globaal stoppen.

SDK: `Session::new(Transport,epoch,service_pid,SendEndpoint,ReceiveEndpoint,budget)`;
`open/state/send/receive/close/send_udp/receive_udp/retire`. Bufferfouten worden
vóór publicatie afgewezen. Antwoordpeer/tag/geen-caps/request/epoch/op/token worden
gecontroleerd. Net zoals de bestaande catalogclient is Transport een vertrouwde
adapter; zelf geconstrueerde Rustwaarden bewijzen geen kernelafzender.

## Native startupcontract

Alle startupvelden zijn u64 op offsets in de bestaande private1024B-startpagina.
De service valideert afzonderlijke handles en verschillende service/app-PIDs.

| Serviceoffset | Betekenis |
|---:|---|
|32|`u64::from_le_bytes(*b"NETSVC01")`|
|40|service-epoch>0|
|48|NetworkSource|
|56/64/72|appA RECV / replySEND / werkelijke PID|
|80/88/96|appB RECV / replySEND / werkelijke PID|
|104/112|EntropySource / servicePID|
|120/128|eerste TCP-poort A/B; steeds vijf toegestane poorten|
|136/144|exacte UDP-poort A/B|
|152/160|optionele controllerRECV / werkelijke controllerPID; beide0=afwezig|

Linkprofiel is het bestaande NIC-profiel MAC02:00:00:00:00:07,
192.0.2.2/24,peer192.0.2.1. De library ondersteunt gevalideerde andere
on-linkconfiguraties; de native proof legt deze expliciete fixtureconfig vast.
De service verkrijgt8bytes via de echte EntropySource voor de smoltcp-seed;
geen vaste proefseed als productie-ISN-claim. MAC/ARP geven geen authenticatie.

Appoffset32=NETAPP01,40epoch,48SEND,56RECV,64servicePID,72slot0/1,
80werkelijke eigenPID,88eigen TCP-startpoort,96eigen UDP-poort. Beide apps
gebruiken hetzelfde ELF; het echte bootstrap-slot kiest hun eigen testdata.

Optionele controller-tagNETCTL01, payload16B: `NETSTOP1`@0,epochu64@8.
Alleen de gebonden controller mag dit versturen, zonder caps. De service stopt
nieuwe requests, sluit sockets en wacht op echte TCP/NIC-drain vóór exit0.
Normale apps beëindigen zichzelf met Retire; als beide apps retired zijn,
alle pending replies zijn verstuurd en alle sockets/TX klaar zijn, sluit de
dienst zijn eigen handles en eindigt hij met exit0.

## Native peer en acceptatievolgorde

AppA TCPbasis8080,appB8090; per app betekenen offsets:
0normaleHTTP,1SYNweigering(RST),2RST na request,3eersteSYN weggooien en daarna
HTTP,4alleSYNs negeren tot echte300tickstimeout. Native apps oefenen die vijf
gevallen zelfstandig en achtereenvolgens; hun processen lopen gelijktijdig.
HTTP-requests zijn exact `GET /app-a HTTP/1.1\r\nHost: fixture.invalid\r\nConnection: close\r\n\r\n`
of `/app-b`. Antwoord exact `HTTP/1.1 200 OK\r\nContent-Length: 19\r\nConnection: close\r\n\r\nNieuw OS service A\n`
of B. UDP9000/9001 echoot exact `Nieuw OS UDP app A` of B naar45000/45001.
ARP moet werkelijk de peer-MAC oplossen; geen vaste MAC in de service.

Apps controleren ook verkeerde TCP-poort, verbindingstoken van andere slot,
oude tokens na reopen, malformed reservedbytes vóór de SDK-start, en geweigerde
rawNIC-aanroep met een IPC-handle. Deze laatste weigering verleent geen hardwarecap.

## Observatie en bouwen

Servicewoorden400ready,408loop,416werkelijkeTXcompletions,424RXframes,
432requests,440rejected,448UDPgequeued,456UDPontvangen,464TCPopens,
472TCPsendbytes,480TCPrecvbytes,488closecalls,496retiredmask,504fatal,
512malformedframes,520UDPdrops,528cleanexit. De native adapter publiceert alle
waarden uit werkelijke service-/devicehandelingen. Geen plaintextpayloadlogs.

Appwoorden400ready,408HTTPcomplete(2),416UDPcomplete(1),424scopeDenied(1),
432staleDenied(≥5),440malformedDenied(1),448sendchunks,456laatsteHTTPresponsebytes,
464gracefulclose(2),472reopens(4),480rawNICDenied(1),488reset(1),496timeout(1),
504retransmit(1),512refused(1),528done. Fatalcode staat op240. Root moet deze
woorden vóór procescleanup vastleggen, zoals bestaande probes; een dode
adresruimte mag niet later voor bewijs worden teruggelezen.

`python3 sdk/examples/network-service/build.py` bouwt beide echte ELFs met
Rust1.91.1/matching rust-lld/locked-offline/W^X-linkerscript. Uitvoer:
NOS_NETWORK_SERVICE_ELF en NOS_NETWORK_CLIENT_ELF. Targets zijn uitsluitend
x86_64-unknown-none; compilation/ELFinspectie is geen native uitvoering.

Bronoriëntatie: OSDev Network Stack,W002538,revisie23577(16april2019).
De bestaande projectdrivers/ABI en de gecontroleerde smoltcp/net-protocol-code
blijven leidend; geen corpusvoorbeelden overgenomen of uitgevoerd.
