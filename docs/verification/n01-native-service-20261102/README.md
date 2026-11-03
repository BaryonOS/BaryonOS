# N01 native netwerkdienst — 2 november 2026

Werkelijke uitvoering op broncommit
`f152aec0615285065689f0a7e2855e5858fdb350`, branch
`codex/n01-native-service-20261102`. De gate bouwde de eigen kernel met Rust
1.90.0 en twee native ELFs met Rust 1.91.1. Twee appprocessen gebruikten
uitsluitend IPC naar een afzonderlijk netwerkdienstproces. Alleen die dienst
kreeg de virtuele NIC en entropy. Geen bestaande OS-kernel, Internetverbinding,
hostshare, fysieke schijf, passthrough of hostinstellingen gewijzigd.

`python3 tools/run_network_service.py` slaagde met QEMU-exit 33 na 8,413 s.
De geverifieerde v3-bronhash is
`56ed10aade50703e08c8422251e323b4400bf37196320d24befe0b46883bbce8`.
De originele volledige machine-uitvoer staat in [native-report.json](native-report.json);
SHA-256 `689edbbb1cefe310b9250e35f20b83f31749ce60960c05d72ebdeec7411209cd`.
De afzonderlijk opgeslagen [serial.txt](serial.txt) bevat de echte proces- en
brokerobservaties. [host-verification.json](host-verification.json) bindt de
exacte hostcommando's en loghashes aan dezelfde bron.

| Werkelijk gemeten | Uitkomst |
|---|---:|
| TCP-openingen, verdeeld over twee apps | 10 |
| HTTP-antwoorden exact ontvangen | 4 |
| SYN-weigeringen / resets na request / retransmissies / timeouts | elk 2 |
| TCP-bytes verzonden / ontvangen | 390 / 308 |
| Gecontroleerd gesloten HTTP-verbindingen | 4 |
| UDP-echo's met eerste bericht behouden | 2 |
| Malformed/checksum/bestemmingspoort geweigerd | 4 |
| Tweede datagram gedropt door vol ontvangstslot | 2 |
| Werkelijke NIC-TX-completions / RX | 45 / 35 |
| Verboden raw-NIC-aanroepen vanuit apps | 2 |
| Geweigerde RPC's (scope, oude/vreemde token, malformed) | 22 |
| Entropy werkelijk geleverd | 8 bytes |
| Processen netjes geëindigd / NIC-resetacknowledgement | 3 / 1 |

De native gate vergelijkt echte frames opnieuw met een begrensde causale
peer-state-machine; hij neemt geen guestmarkers over als serverbewijs.
TX/RX-tellingen stemmen overeen tussen service, kernelbroker en socketpeer.
De kernel legt appresultaten vast vóór geheugenopruiming en controleert daarna
dat capabilities, wachtrijen, procesgeheugen en paginatabellen zijn opgeruimd.
Bron, inspectieresultaten en ingebedde ELFs, QEMU, firmware, ESP en opgeslagen
serial/stderr/peertranscript zijn vóór en na uitvoering gecontroleerd.

Gerichte hosttests: protocol 4/4, SDK 6/6 en service 7/7, alle drie ook strict
clippy met `-D warnings`. De servicetest vult twee echte smoltcp-TCP-buffers
tot 4096 bytes per app, constateert backpressure, laat de peer 8192 bytes
ontvangen en echoën en sluit beide verbindingen. Dat is een gecontroleerd
hostmodel; de native UDP-proef levert het afzonderlijke echte backpressurebewijs.
Python: nieuwe peer/harness-negatieven 11/11; bestaande TCP-peerregressies 16/16.

Bewijsgrens: één QEMU/TCG-profiel, synthetische Ethernetpeer en vaste
bootstrapbestemmingen. Dit bewijst geen TLS, cryptografische peeridentiteit,
DHCP, algemene dynamische policy-issuer/intrekking, fysieke DMA-veiligheid,
volledige Internetfunctionaliteit of algemene veiligheidssuperioriteit.
Een onverwachte permanente appkanaalfout retireert alleen dat slot; een
onherstelbare gedeelde devicefout beëindigt de dienst fail-closed. Automatisch
dienstherstel en fouttolerantie voor iedere mogelijke procescrash zijn geen
gateclaim.
De hoofdagent herhaalt de gate op de geïntegreerde bron vóór scoreacceptatie.

## Kanaaluitval en aanvullende foutbegrenzing

`python3 tools/run_network_service.py --case client-exit` slaagde op dezelfde
bron met exit 33 na 7,538 s. A krijgt een echte verbinding naar de synthetische
blackhole, publiceert State, sluit de werkelijke reply-RECV en publiceert een
laatste controle-Statesverzoek. De dienst ziet Closed op de uitgaande reply,
retireert alleen A, sluit diens kanalen en behoudt de gedeelde NIC-transmissie
tot echte completion. A eindigt op monotone tick 32. B voltooit daarna alle
vijf TCP-scenario's en UDP met malformed/backpressure op tick 470.

Deze tweede route meet zes TCP-openingen, 195 verzonden/154 ontvangen bytes,
twee normale sluitingen, één UDP-echo, twee malformed-weigeringen, één vol-slot
drop, 13 RPC-weigeringen en 25 TX-/18 RX-frames. Werkelijke peer-, service- en
brokertellingen kloppen; alle capability-/frameopruiming en NIC-resetack slagen.
[client-exit-report.json](client-exit-report.json), SHA-256
`e7fad081a1a0c5c307852e6ef1dd111ff6a0c3c2bf3432c198ff3860eb569ec7`, en
[client-exit-serial.txt](client-exit-serial.txt) bewaren het bewijs.

De afzonderlijke hostnegatieven laten A zijn volledige eigen namespace van
12.768 TCP-poorten uitputten. A krijgt daarna Status::Exhausted; B kan nog
steeds openen en zijn eigen verbinding opvragen. Een tweede test controleert
dat A's kanaalfout een al gepubliceerde transmissie nooit vals voltooit of
het slot vrijgeeft. De native runner bewaart oorspronkelijke stdout/stderr
als bytes en herleest die vóór strikte UTF-8-decodering. Ongeldige bytes,
outputoverflow en timeout worden geweigerd; CRLF blijft bytegetrouw bewaard.

Een afzonderlijke AI-review vond deze drie randgevallen in de eerste kandidaat
en controleerde de fixes opnieuw met zeven service- en elf harness-tests.
Daarbij bleven geen blockers over binnen deze scope. Dit is geen menselijke
veiligheidsaudit. De kortere oude normale VM-proef op 3a5e1e3 is uitsluitend
ontwikkelbaseline; de opgeslagen eindrapporten horen bij f152aec.
