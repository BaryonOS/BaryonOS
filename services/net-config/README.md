# TASK-N01-NATIVE-CONFIG

Enige NET-schrijver volgens queuecommit
`16151b9c28a766c47f900e4a3e9f99b642cc3fe5`.
Leafbasis `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`; schrijfgebied uitsluitend
`services/net-stack/**`, dit nieuwe `services/net-config/**` en eigen bewijs
daaronder. Geen protocol-, root-, kernel-, SDK-, driver- of centralestatuswrites.
TLS #81 en broker #87 blijven afzonderlijke brongebonden opleveringen.

## Kleinste eerste resultaat

Adresconfiguratie mag niet alleen in een DHCP-codec blijven zitten: dezelfde
bestaande Stack moet on-geconfigureerd kunnen starten, een gecontroleerde
adres/prefix/default-route toepassen, en na intrekking niet onder de oude
configuratie doorzenden. De eerste rode lifecyclecase moet aantonen dat
gegevensverkeer zonder toegepaste configuratie of na withdrawal wordt geweigerd.
Dat is de eerste implementatiestap, niet de hele native acceptatie.

Daarop volgt de DHCP-lifecycleadapter met hergebruik van clientbron
`16adfe54f3e6539339aad60232f7145c4212e538` en de codec op mainca45, zonder
legacy-rootmerge of tweede client/transportstack. De pure clientbron staat nog
niet in deze basis; die mag niet stil als geintegreerd worden beschouwd.
Een eventuele verplaatsing/kopie van die eigen bestaande clientleaf moet exact
brongebonden en herkenbaar blijven, niet herontworpen als een tweede client.

## Gedeelde seams en grenzen

- De bestaande Stack heeft twee TCP-sockets, een IPv4-adres, vier buren,
  een route en een PacketDevice met een RX-/TX-slot. Geen dynamische
  adres/default-route/DNS-config-API bestaat op deze basis.
- Alle DHCP-, ARP- en TCP-egress moet hetzelfde PacketDevice en hetzelfde
  echte matching-completionpad gebruiken. Geen tweede producer die NIC-frames
  naast de Stack publiceert; geen synthetische completion om frames weg te gooien.
- De configuratieadapter is vertrouwde dienstcode, geen appgrant/issuer.
  Een DHCP-server/IP/route/DNS-antwoord is geen geauthenticeerde TLS-/NAS-peer.
  Configuratie verleent apps geen nieuwe proces-/bestemmingsrechten.
- DHCP-tijd is monotone seconden; Stack/SDK-tijd is 100 Hz. Converteer checked;
  afronden mag expiry niet verlengen. UTC/RTC is geen netwerktimerautoriteit.
- CheckingAddress mag alleen na echte RFC5227-probes naar bruikbare Bound
  promoveren. Geen publieke trustboolean of enkel DHCPACK als proefbewijs.
  Synthetische hostprobes blijven hostmodelbewijs, geen native botsingsdetectie.
- T1/T2/expiry, NAK, conflict en linkdown trekken adres/route/DNS coherent in.
  Reconnect behoudt geen oude configuratie, queued-RX, grants of sockets.
  DNS-serverconfig is routedata; antwoordcache/naamrechten blijven aparte
  resolver-/brokergrenzen. Geen standaard-DNS-fallback bij ontbrekende optie.
- Reeds gepubliceerde hardwareframes kunnen nog de wire op; beloof geen
  terughalen. Onzekere TX houdt buffers/ownership vast. Withdrawal moet
  nieuwe oude-configpublicatie stoppen zonder fake ACK/discard/replay.
  Ontbrekende echte device-reset/quiescence/epochbinding blijft expliciet open;
  configcode mag die niet met een nieuwe instantie of `reset_ok` vervangen.

## Acceptatie en bewijsgrens

Eerst gerichte rode en groene lifecyclecases, daarna normale en negatieve
DHCP/route/DNS-proeven: malformed/stale/NAK/conflict/overflow/backpressure,
T1/T2/expiry/linkdown/reconnect en echte modelpakketbytes met een enig
completionpad. Nieuwe dependencies alleen indien noodzakelijk en binnen de
bestaande gecontroleerde pins/features. Rust1.91.1 zoals de bestaande Stack.

Locked hosttests, Clippy/all-targets/-Dwarnings, fmt en
`x86_64-unknown-none`-librarybuild op exacte bron; relevante bestaande
codec-/stacktests behouden. Logs, exitstatussen, bron-/artifacthashes onder
eigen `verification/`, checkpoints gepusht voor lange gates.
Native issuer/caller/driver/devicefence en de eerdere TCP-CR3-bootfout blijven
REL/SDK/HW-afhankelijkheden; geen eigen writeclaim daarvoor.
Geen hosttool voert de ontbrekende native DHCP-/TCP-/TLS-client uit.

Dit document is het eerste taakcontract, nog geen implementatie of gate.
Netwerk 0/10 -> 0/10, totaal 34 -> 34 als voorstel aan centrale eigenaar;
geen centrale voortgangsbestanden wijzigen.

## Eerste implementatiestap: echte Stack-adreslifecycle

De bestaande statische `Config`-constructor blijft beschikbaar; nieuw zijn
`LinkConfig`, `AddressConfiguration`, `Stack::new_unconfigured`,
`configuration`, `apply_configuration` en `withdraw_configuration`.
Dit zijn trusted-service-broninterfaces, geen bewijs van een DHCP-probe.
Nog geen DHCP-/DNS-/RFC5227-clientadapter of nieuwe protocolbron toegevoegd.

De basisinterfacegate was terecht compile-rood (exit101, geen runtime).
Daarna vond een echte runtimecase dat alleen `socket.abort()` onvoldoende
was: na reapply van hetzelfde adres kon de oude RST alsnog ARP/egress
veroorzaken. Die rode proef met de exacte WIP-patch is apart bewaard.
Withdrawal verwijdert nu eerst adres/routes en fenceert publicatie; alle
sockets worden lokaal geabort. De bestaande smoltcp SocketSet remove/add
reset de neighbor-backoffmetadata zonder buffers vrij te geven en met behoud
van de bestaande proceslokale handles;
de gepinde upstream dispatch reset een tuple zonder lokaal bronadres
VOOR zij kan uitzenden. Daarna is de oude TCP-/RX-toestand weg. Er wordt
geen klok vooruitgezet, pakket gepubliceerd of completion gefingeerd.

Een bestaand PacketDevice-TX-slot blijft behouden en onzichtbaar voor nieuwe
publicatie. Reapply geeft Busy totdat een echte passende completion dat slot
heeft voltooid; een ongepubliceerd/Unknown-slot kan hiermee niet automatisch
worden hersteld. Andere echte reset/quiescence/epochbewijzen blijven open.
Klokrollback/overflow bij withdrawal levert expliciet een fout op, maar
adres/routes/socketautoriteit worden desondanks eerst ingetrokken.

Validatie weigert niet-unicast/subnet-/broadcastadressen en gateways buiten
de subnet, gelijk aan eigen IP of niet-unicast/subnet-/broadcast. Nieuwe
configuratie tijdens een levende socket wordt geweigerd zonder mutatie.
Ongelezen ingress en de oude neighborcache worden niet doorgegeven.
ARP-rate-limiting blijft de echte upstream backoff; een reconnect hoeft
daarom niet op dezelfde tick een nieuw pakket te produceren.

Gerichte hostuitvoering: zeven nieuwe configuratiecases en alle vijftien
bestaande transporttests groen. De default-routecase observeert een echte
ARP voor de gateway bij een offlink TCP-bestemming, geen hostsocket.
Volledige brongebonden Clippy/fmt/none/codec-gates op bron2c0e37a zijn
exit0 opgeslagen onder `verification/address-lifecycle/`.
Geen native DHCP/IP-config-/appacceptatie of punten.

## Vervolg: bestaande client hergebruiken, geen tweede NIC-producer

Dit gebied bevat nu een eigen no_std-crate met alleen bestaande lokale
Stack-/codecdependencies en exact dezelfde registrylocks/features als Stack.
De DHCP-client en elf bestaande pakkettests zijn overgenomen uit bron16adfe5;
oorspronkelijke client-SHA256:
`1f9e64b272665d1ac43d63ae54fdc539f21d236ef91b636b3c2da39f695fd04c`.
Op bronf6d9f1f geen algoritmewijziging: alleen fmt/imports aangepast voor deze crate.
De vroegere private `arp::unicast_ipv4`-predicate is exact als private
compatibiliteitshelper opgenomen, zonder protocol-API/write te verruimen.
De oorspronkelijke branch blijft behouden; geen oude rootwijzigingen gemerged.
`dhcp_client` is nog dezelfde pure client, niet de native dienstadapter.
Zijn `confirm_address_available` mag een uiteindelijke adapter uitsluitend
zelf aanroepen na echte eigen probe-/completionwaarneming, niet via IPC.

Nieuwe `time`-helpers converteren gecontroleerd tussen 100Hz en DHCP-seconden,
met dezelfde bovengrens als Stack en rollbackdetectie ook binnen een seconde.
Floor bij requestseconden kan expiry vroeger maken, nooit later. Geen UTC/RTC.

`Stack::queue_control_frame` is uitsluitend een trusted-service-broninterface,
geen appgrant of rawsocket-IPC. De toekomstige configuratiedienst valideert
doel/type/identiteit van DHCP/ARP voordat zij dit aanroept. Frames gebruiken
hetzelfde PacketDevice-TX-slot, oplopende id en matching completion als TCP.
Busy/ongeldige lengte/uitgeputte id/klokfout vervangt geen bestaande bytes.
Withdrawal verbergt ook controleframes maar behoudt storage tot echte completion.
Unknown/unpublished recovery is nog niet toegevoegd.

Dit tussencheckpoint alleen leverde nog geen coordinator; inmiddels volgt
hieronder de eigenlijke source-implementatie. Native driver/callerkoppeling
ontbreekt nog. N01 en netwerk0/10 blijven open.

## DHCP-/probe-/DNS-coordinator: hostcore, geen native proces

`service::Service` bezit de bestaande Stack en DHCP-client. `start`,
`send_dhcp`, `receive_frame`, `poll`, `complete_transmission`, `transmission`,
`configuration`, `phase`, `begin_renewal`, `link_down` en
`transport_uncertain` zijn trusted-service-broninterfaces, GEEN nieuwe IPC,
appcapability, kernelissuer, exclusieve devicelease of authentieke peerbinding.
Geen publieke probe-succesboolean of constructor voor een succesattest.

ACK laat adres/routes/DNS nog afwezig. `poll` publiceert drie echte RFC5227
ARP-probebytes via hetzelfde PacketDevice; alleen passende backendcompletions
verzetten de probevoortgang. Tussenpozen komen uit een gevalideerde
`ProbeTiming`: init0..100ticks, daarna100..200ticks. De native caller moet deze
uniform uit geautoriseerde entropy trekken; vaste hostwaarden zijn geen RNG.
Na de derde completion volgen200ticks wachttijd en twee announcements,
200ticks uiteen. Pas na completion van de eerste announcement wordt het
adres/default-route toegepast, de client Bound en DNS samen zichtbaar.

De bestaande gepinde smoltcp wireviews encoderen/decodeerden ARP; geen nieuwe
dependency of protocolcratewrite. RFC5227-proberesponses met target-IP0 worden
wel als conflict gezien. De oudere neighborcodec weigert die terecht in haar
beperkte resolverprofiel en is niet als volledige conflictdetector gebruikt.
Eigen echo wordt niet als vreemde probe geteld. Vreemde claim tijdens de
volledige wachtperiode of tijdens Bound trekt de configuratie in; geen
permanente adresverdediging. Na conflict minstens10s wachten, na tien
conflicten minstens60s (RFC5227rate-limit); state blijft in deze service.
Een nieuw object is geen native apparaat-/crash-/persistentielimietbewijs.

Een echte rode runtimecase vond bovendien dat upstream smoltcp zero-source
peerprobes tijdens Bound niet beantwoordt. De regression en exacte WIP-patch
tegen4c335d1 zijn bewaard. De configuratieadapter antwoordt nu uitsluitend op
een gevalideerde vreemde probe voor het werkelijk toegepaste eigen adres,
zonder0 als neighbor te cachen, via hetzelfde TX-slot/completionpad.
Busy houdt het huidige frame vast; de ontvangende loop bewaart haar ingress
tot zij opnieuw kan aanbieden. Geen fake completion of onzichtbare discard.

T1 meldt RenewalDue; de loop levert een verse XID aan `begin_renewal`.
Renewal gebruikt de server-MAC uit de geaccepteerde directe DHCPuitwisseling,
alleen als ongeauthenticeerde routeringshint. T2 verandert naar broadcast
Rebinding; expiry/NAK/linkdown verwijderen adres/route/DNS en verbergen oude
TX. DNS is de serverconfiguratie, geen resolver/cache of appnaamrecht.
Bij ongewijzigd adres/prefix/gateway vernieuwt de lease/DNS zonder TCPreset.
Een gewijzigde route/prefix retiret de oude sockets; pas na oude TXcompletion
wordt de nieuwe epoch actief. Dat kan verbindingen onderbreken en is geen
claim van naadloze routewijziging. Clockfault of Unknown is terminaal:
geen automatische retry/release/herstel of hervatten van een oude lease.

Twaalf gerichte hostcases zijn groen, inclusief probe-/announcementbytes,
stale completion, TXbackpressure, late conflicten, gewone peerprobeantwoorden,
T1/T2/NAK, gewijzigde gateway/DNS,10conflicten,expiry,linkdown en klokoverflow.
Nog niet: volledige brongebonden eindgates op deze coordinator, native proces,
echte NICcompletion/DMAfence/driverreset/CR3, gekoppelde broker-policy,
native RNG/retryscheduler/auto-reconnect of volledige DHCPreleaseacceptatie.
Op bron03277a7 had de codec geen DHCPDECLINE-builder: concrete extra claim
was toen gevraagd maar niet overgenomen. Die grens is hieronder expliciet
vrijgegeven. RFC2131sectie3.1 vereist DECLINE;10s wachten is daar SHOULD en
wordt hier conservatief toegepast.

De gerichte vervolgcase op bron678081c vond een echt wireverschil: selecting
REQUEST had `secs=5`, DISCOVER `secs=1`. De eigen verplaatste client bewaart nu
de `secs` van de laatst gecodeerde DISCOVER voor selecting REQUEST
(RFC2131sectie3.1stap3), zonder de conservatieve leaseklok naar voren/achter
te schuiven. De nieuwe regression controleert ook echte expiry op de
oorspronkelijke REQUESTtijd+80s. Dit is een afzonderlijk vastgelegde
algoritmecorrectie boven de exacte16ad-import, GEEN wijziging van donorbranch
of de gedeelde protocolcrate. De rode testpatch/log tegenover678081c blijven
brongebonden bewaard.

Een aparte read-only AI-codecheck op bronf16062b vond een concrete
vertraagde-renewalcase: na acceptatie van een nieuwe lease kan T1 al voorbij
zijn en retourneert de pure client meteen RenewalDue in plaats van Bound.
Alleen Bound afhandelen liet oude gateway/DNS zichtbaar terwijl de client
intern al de nieuwe lease gebruikte. Gerichte runtime-reproductie was rood;
de coordinator past nu geaccepteerde renewal-ACKs in beide fasen coherent toe.
De case bewaart de nieuwe T1 en expiry; geen leaseverlenging of successfallback.
Dit is een begrensde tweede AI-review, geen menselijke audit/nativeacceptatie.

## TASK-N01-DHCP-DECLINE-SEAM

Expliciete extra toewijzing:
queue `dea97f9b57c6fda9549a48072119663fe155a563`,
blob `53ac59d53fa7daece42b07925d6ecffe852012f6`.
Uitsluitend `services/net-protocol/src/dhcp.rs` en `tests/dhcp.rs` boven
de bestaande Stack/configclaim; geen andere protocol-/root-/SDKwrites.

`dhcp::build_decline(out,xid,mac,address,server)` hergebruikt de bestaande
private requestencoder/validators. Message4, BOOTREQUEST, zero secs/flags/ciaddr,
requestedIP/serveridentifier en hetzelfde clientidentifier. Alleen statische
message56 toegevoegd; GEEN parameterlist55, maxsize57, leasetime51 of andere
REQUEST-onlyopties. Vooraf alle controles, geen outputwijziging bij fout.
300UDPdatabytes met zeropadding en onaangeraakte tail buiten de retourlengte.

Bij echt geobserveerd conflict trekt de coordinator eerst oude adres/route/DNS
en TXpublicatie in. Zij bewaart alleen de oude lease/XID voor DECLINE.
`poll` queueet het nieuwe broadcastframe pas zodra het bestaande PacketDevice
werkelijk vrij is; geen completion/discard van het retired frame.
Ook DECLINE gebruikt dezelfde id/storage/completion. Nieuwe acquisitie wacht
minstens10s (na10conflicten60s) EN op echte DECLINEcompletion. Niet-gepubliceerd
of EAGAIN blijft hetzelfde pendingframe; Unknown blijft terminaal.
Linkdown/clockfault behoudt geen geautoriseerde oude lease of herstelshortcut.

Drie nieuwe codeccases controleren exacte packet/header/options/tail,
ongeldige adressen/MAC en elk van de300tekorte bufferlengtes all-or-nothing.
Vijftien coordinatorcases groen, inclusief conflict->retired TX->DECLINE->
matchingcompletion->verse acquisitie en bestaande netwerk-lifecycle.
De eerste APIgate was compile101/E0425 (ontbrekende functie), GEEN runtime.
Volledige brongebonden eindgates op deze source volgen na het checkpoint.
Geen nieuwe manifest/lock/dependency/entropybron of netwerkgrant, geen
native completion/fence/app/netwerkacceptatie en geen nieuwe punten.

## Transport onder de actieve lease

De coordinator bezit de Stack; zonder begrensde transportdoorgifte zou een
native integrator toch een tweede Stack of ongecontroleerde mutable toegang
nodig hebben. `add_transport_client`, `connect`, `send`, `receive`, `close`,
`cancel` en `state` delen daarom de bestaande buffers/handles/Stack.
Er wordt GEEN mutable Stack/device uitgeleend. Iedere operatie verwerkt eerst
de werkelijke monotone tick/lease-expiry; gegevensoperaties vereisen ook de
werkelijk toegepaste actieve configuratie. Status/cancel geven na withdrawal
alleen de bestaande gesloten toestand terug en herstellen geen lease.
Dit blijft een trusted-service-broninterface, geen appgrant of brokerpolicy.

Twee nieuwe gekoppelde hostcases tonen echte ARP plus TCP-SYN uit dezelfde
Stack na DHCP/probes; controlbackpressure bewaart dat SYN-slot, expiry
sluit de socket/verbergt TX en blokkeert send/receive/close zonder outputwrite.
Een ongewijzigde renewal behoudt een levende socket; een nieuwe gateway
retiret die en toont de nieuwe DNS/route pas na oudecompletion.
Dit bewijst geen native verbinding, applicatieantwoord of rechtenafdwinging.
