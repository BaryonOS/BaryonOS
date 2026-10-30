# Begrensde ring-3 TCP-engine

`Stack` gebruikt de bestaande vastgepinde `smoltcp = 0.14.0` zonder allocator,
hostsocket of nieuwe dependencies. Dit is protocolcode, geen kernelcapability,
netwerkbroker, geauthenticeerd peertransport of TLS-stack. `Client` is alleen
een process-lokale socketindex bij zijn eigen engine; nooit een IPC-token.

## Passief luisteren

`listen(client, local_port, timeout_ticks)` bindt één bestaande, gesloten
socket aan het **geconfigureerde eigen IPv4-adres**, niet aan een wildcard.
De poort mag niet nul zijn en de duur is 1..6000 ticks (100 Hz, maximaal
60 seconden). De caller moet eerst `poll` met de echte monotone klok
aanroepen. De acceptdeadline wordt vanaf die laatst gepollde tijd berekend,
met overflowcontrole. `poll` sluit LISTEN/SYN-RECEIVED op de exacte grens,
**vóór** een nieuw ingresspakket verwerkt wordt. Dezelfde tijd herhalen
verlengt niets; klokrollback geeft een expliciete fout.

Er is geen backlog, automatische relisten of verborgen nieuwe socket.
Maximaal twee sockets delen vaste caller-owned opslag; per socket zijn
RX/TX elk maximaal 8192 bytes. Een tweede wachtende listener op dezelfde
poort geeft `Busy` zonder de eerste te vervangen. Na accept kan de andere
socket die poort bedienen; na afloop moet de caller expliciet opnieuw
luisteren. `state` onderscheidt wachten, halfopen, established en gesloten.
Een ongebruikte of halfopen listener verliest zijn socket bij de deadline;
een tijdig gevestigde verbinding niet. Voor reads/writes en de volledige
operatie moet de caller aparte absolute deadlines handhaven. De upstream
TCP-timeout is **geen** algemene idle-/operatiedeadline.

`remote_endpoint` geeft alleen ruwe IPv4-/poortcoördinaten terug, geen
identiteit. Bij LISTEN/CLOSED is dat `None`, ook wanneer upstream nog een
RST afhandelt. De caller moet vóór applicatiegegevens echte TLS-peercontrole,
caller-/appbinding, bestemmingsrecht en intrekkingsgeneratie afdwingen.
Deze library implementeert die beleidsgrens niet. `send`/`receive` blijven
begrensd en kunnen gedeeltelijk of nul bytes verwerken; dat is geen
devicecompletion of bevestiging door de remote applicatie.

`close` stopt een listener of sluit een verbinding ordelijk. `cancel`
aborteert expliciet; upstream kan nog een RST queueën. Cancel/timeout maken
een al gepubliceerde DMA-buffer niet vrij: de native caller moet exacte
TX-completion afwachten of de broker laten resetten/quarantainen.
Intrekking moet ook queued TX bij de daadwerkelijke broker blokkeren; deze
engine is op zichzelf geen N04-afdwinging.

## Bewijs en grenzen

Host/modeltests gebruiken synthetische Ethernetframes en expliciete modeltijd:
exacte request-/responsebytes met 8-bytebuffers en backpressure, endpoint-
inspectie, orderly EOF, ontbrekende handshake-ACK, verkeerd doelpoort,
late SYN, dubbele listeners, cancel/relisten, exacte deadline, stilstaande
klok, rollback en overflow. Bestaande actieve TCP-scenario's blijven groen.
Beide modelendpoints gebruiken dezelfde smoltcp-engine; dit is geen
onafhankelijke interoperabiliteitsproef.

Bron- en commandobewijs: [`verification/listen/README.md`](verification/listen/README.md).
Er is nog **geen native passieve TCP/NAS/TLS-acceptatie**. De bestaande
`sdk/examples/tcp-probe` is een actieve client en kan alleen als gerichte
native regressie van deze gedeelde engine worden gebruikt, niet als
luisterproef. Root/kernel/manifest/buildintegratie blijft bij de releaseowner.

Primaire library-API: smoltcp 0.14.0,
<https://docs.rs/smoltcp/0.14.0/smoltcp/socket/tcp/struct.Socket.html>
(`listen`, `set_timeout`, `abort`, geen backlog). Bestaande dependency-
licenties staan in [`THIRD-PARTY.md`](THIRD-PARTY.md); manifests/lockfile
zijn niet gewijzigd. Geen RTC-, hosttijd- of netwerkbron wordt tot
vertrouwde certificaattijd gepromoveerd.
