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
Volledige brongebonden Clippy/fmt/none/codec-gates volgen na dit checkpoint.
Geen native DHCP/IP-config-/appacceptatie of punten.
