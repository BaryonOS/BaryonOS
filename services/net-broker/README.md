# TASK-N04-NET-BROKER - begrensde socketautoriteit

Contractvoorstel voor read-only afstemming met Security, nog niet de vaste
SDK-/IPC-interface. Basis `4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8`.
Rubric N04/A03/B06; release FR-N01/FR-N04/FR-K03. Native TCP-stack,
codec/client/listenbranches en hun brongebonden bewijs blijven behouden.

## Doel, eigenaar en bewijsgrens

NET bezit alleen deze nieuwe zelfstandige `services/net-broker/**`-leaf en
`.github/workflows/net-broker-quality.yml`, volgens manager `89f84098`.
Kernel, driver/deviceadapter, SDK en bestaande stackbestanden worden niet
gewijzigd. Security bezit identiteit/capability-/crypto-contracten; native
manifest/issuer/SDK-integratie wordt afzonderlijk met de betreffende eigenaren
afgestemd. Geen tweede transportstack, cryptolaag of wire-ABI.

Eerste complete stap: allocatorvrije autoriteitscore met vaste grant- en
verbindingstabellen boven de bestaande `nieuw-os-net-stack::Stack`.
Normale/negatieve hosttests, Clippy, formattering en baremetal-librarybuild
zijn de gates voor deze stap, geen native netwerkafdwinging of punten.
De huidige echte TCP-run faalt voor ring 3 bij de CR3-switch; REL bezit die
blokkade. Een authentieke caller-/issuerseam bestaat nog niet in deze leaf.

## Beveiligingsinvarianten

- Caller-identiteit ontstaat uitsluitend uit authentieke kernel-/
  capabilitymetadata plus vertrouwde bootstrapbinding aan dienstinstantie en
  generatie. Geen aanvraag-PID, IP, route, hostnaam, client- of
  `verified=true`-veld kan die identiteit leveren.
- De voorgenomen `Caller`, `PolicyAuthority` en `ServiceInstance` zijn opaque
  interne contexttypen zonder publieke constructor/parser/From/Default.
  Een productie-issuer ontbreekt expliciet: geen veilige native caller kan ze
  nu zelf fabriceren. Alleen private hosttestfixtures modelleren deze context;
  dat is geen geslaagde authenticatie of kernelbinding.
- Policy installeren/intrekken vereist afzonderlijke `PolicyAuthority` voor
  dezelfde dienstinstantie; een appgrant wordt nooit beleidsautoriteit.
  Een helper heeft zijn eigen callercontext en krijgt niet de grant van zijn
  opdrachtgever. Delegatie wordt in deze eerste core niet impliciet ondersteund.
- Maximaal acht grants met elk precies een unicast IPv4-adres en niet-nul
  TCP-poort; maximaal twee sockets, gelijk aan de bestaande stack. Geen wildcard,
  impliciet listen, naam-naar-IP-autorisatie of automatische bestemmingsuitbreiding.
  Een endpoint is een toegestane coordinatenbinding, geen TLS-/NAS-identiteit.
- Iedere grant/verbinding bindt caller, dienstinstantie en eigen monotone
  slotgeneratie. Hergebruik behoudt de generatieteller; overflow faalt zonder
  publicatie. Oude handles kunnen geen nieuw slot of andere app bedienen.
- Connect/send/receive/close/cancel toetsen de huidige grant en caller; send en
  receive hebben afzonderlijke rechten. Weigering komt voor backendactie of
  uitvoerwijziging. Receivefouten leveren geen nieuwe bytes. Connect kiest zelf
  een begrensde lokale poort; een app opent geen impliciete luisterpoort.
- Het service-eventloop levert echte monotone SDK-ticks, nooit aanvraagtijd.
  Absolute grant-/verbindingsdeadlines, rollback en overflow worden voor een
  backendstap gecontroleerd. De bestaande stack blijft eigenaar van TCP-timers,
  packetbuffers, retransmissie en backpressure; gelijke ticks tellen niet op.
- Send rapporteert alleen lokaal gequeue-de bytes, geen NIC-, peer- of
  applicatie-ACK. Een onzekere uitkomst wordt nooit stil herhaald.

## Intrekking en transportownership - expliciet te beoordelen

De huidige stack heeft een enkele gedeelde TX-slot, maar geen externe
per-flow-ownership-/discard-API. NET gaat geen lokale vrijgave vervalsen via
`complete_transmission`: dat blijft uitsluitend een echte, passende
devicecompletion van de vertrouwde adapter.

Voorgestelde veilige eerste lifecycle: revoke/expiry/cancel retireert de
betrokken grant/verbinding direct, abort de socket en vereist vervolgens
transportreset/quarantaine voordat deze broker slots opnieuw gebruikt.
Een reeds gequeue-de of mogelijk gepubliceerde frame wordt niet opnieuw
aangeboden, hergebruikt of als voltooid geboekt. Een onzeker NIC-resultaat
quarantaineert eveneens de broker. Geen publieke `reset_ok=true`-omweg of
gefingeerde reset-ACK; herstel vereist een toekomstige echte driver-/issuergrens
en nieuwe dienstinstantie.

Deze eerste conservatieve quarantaine kan ook andere sockets op dezelfde
device-instantie pauzeren. Dat is een zichtbare voorlopige coregrens, geen
geaccepteerd dagelijks multi-appgedrag of rechtvaardiging voor N04-punten.
Fijnmazige drain per flow en native reset/cleanup zijn vervolgintegratie, niet
stil omzeild door een tweede stack of een onbewezen completion.

De core moet de authentieke grant intrekken voordat cleanup wordt geprobeerd.
Een cleanupfout kan dus geen oude autoriteit herstellen. Een succesresultaat
mag uitsluitend lokale retirement beschrijven, niet verwijderde pakketten bij
een peer. Alle open devicebufferlifetimes blijven bij de hardware-/drivergrens.

## Voorgenomen broninterface, geen wire-schema

- `Broker`: exclusieve eigenaar van bestaande `Stack` en twee reeds toegewezen
  caller-owned sockets/buffers; geen publieke mutable stack-bypass.
- `grant`/`revoke`: alleen met opaque beleidscontext, begrensd endpoint en
  rechten, echte monotone tijd en generatiebinding.
- `connect`/`send`/`receive`/`close`/`cancel`: opaque caller, grant en sockettoken;
  expliciete resultaten voor deny/stale/limit/clock/timeout/quarantine/backend.
- Poll/ingress/egress zijn uitsluitend de vertrouwde service/device-seam,
  geen appbericht. Een toekomstige adapter mag alleen echte devicecompletion
  doorgeven; `Unknown` levert geen ACK, herpublicatie of bufferrelease.
- Er is nog geen native RPC-dispatch, issuer, SDK-client of manifestgrant.
  Message320/sender-PID alleen wordt niet tot sterke identiteit gepromoveerd.

## Acceptatie voor deze core

Normaal geautoriseerde connect en rechten; deny zonder backendbijwerking,
cross-caller/dienstinstantie, stale/reused generatie, revoke op bestaande en
nieuwe sockets, helper zonder delegatie, grant-/socketuitputting, verkeerde
bestemming, timeout/rollback/overflow, backpressure en Unknown/no-retry.
Waar echte TCP-bytes nodig zijn blijft de bestaande stack de implementatie;
testtegenpartijen mogen alleen synthetische peers/modelmedia zijn.

Na interfaceafstemming: Rust 1.91.1 zoals de bestaande stack, dependency alleen
op die bestaande stack en zo nodig dezelfde al gecontroleerde smoltcp voor
hosttegenpartijtests. Geen nieuwe registryversie of featureverruiming.
Exacte commands/results/bronhashes worden onder `verification/` bewaard.
Een aparte kwaliteitsworkflow moet alle nieuwe coretests, Clippy, fmt en
`x86_64-unknown-none` uitvoeren; geen proxy voor native acceptatie.

Geen gate uitgevoerd en geen criteria afgetekend in dit contractvoorstel.
Scorevoorstel aan centrale manager: netwerk 0/10 -> 0/10, totaal 34 -> 34.
De manager blijft eigenaar van PROJECT-STATE, progress.json en roadmap.
