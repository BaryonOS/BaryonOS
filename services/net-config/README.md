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
