# Getypeerde INP1-appclient

Allocatorvrije `no_std`-client voor de bestaande 96-byte INP1-appinterface.
Dit is een bibliotheek voor het echte kanaal naar de geïsoleerde inputservice;
zij decodeert geen PS/2-hardwarebytes en verleent geen focus- of invoerrechten.
`NativeTransport` gebruikt bestaande native SDK-IPC. In de productbron wordt
geen `Received` gemaakt uit zelf verzonnen afzendergegevens.

## Interface en eigenaarschap

`Session::new(transport, Binding { epoch, service_pid, client_id, width, height,
send, receive }, poll_budget)` neemt bij succes één exclusief endpointpaar
over. De bootstrap moet dat paar al hebben verleend. Geef geen geleende of
gedeelde handles door. Een mislukte constructor neemt de handles niet over.
IDs/PID/geometry in `Binding` zijn verwachte correlatiegegevens; alleen kernel
en service-ingress bepalen werkelijke autoriteit. Width/height zijn de globale
GOP-geometrie, niet de lokale apprechthoek.

`peek()` geeft `Peek::Empty` of `Peek::Offer(offer)`. Een aanbod heeft getypeerde
`epoch()`, `sequence()`, `generation()` en `event()`-getters. `Event` gebruikt
rechtstreeks de bestaande input-core-/ui-coretypes. De app verwerkt een aanbod
eenmaal en roept `ack(&offer)` aan. Tot die exacte ACK weigert de sessie een
nieuwe PEEK; ook een tweede ACK van hetzelfde aanbod wordt lokaal geweigerd.

`AckOutcome::Acknowledged` bevestigt een werkelijke, gevalideerde service-ACK.
Een canonieke `StaleAck` bij bijvoorbeeld focusverlies geeft
`AckOutcome::Superseded`. De oude aanbieding is dan achterhaald. Dat resultaat
verhoogt geen ACK-teller en leest of bevestigt geen nieuwe annulering. De app
kan vluchtige bediening annuleren en met een volgende PEEK het nieuwe echte
annuleringsbericht ontvangen. `Cancel.generation` is een through-watermark:
die mag ouder zijn dan de bekende focusgeneratie. De controller beslist welke
oude toestand daarmee vervalt. Alleen key-/mousegeneraties moeten niet dalen;
iedere eventsequence moet strikt stijgen binnen de gebonden service-epoch.

`close()` en `cancel()` sluiten uitsluitend dit eigen kanaalpaar, elk handle
éénmaal; beide closes worden geprobeerd als de eerste faalt. `Drop` probeert
dezelfde opruiming zonder impliciete event-ACK. Gebruik expliciet `close()` om
een closefout te zien. Geen van deze functies is een globale focusopdracht.
De vertrouwde supervisor/broker moet bij het aflopen van een binding ook de
service-lifecycle afhandelen; een lokale close claimt die native koppeling niet.

## Protocol- en foutcontract

De client hergebruikt INP1-versie/tag/opcodes/requests en status-/eventtypen uit
de bestaande servicecrate. Replies worden opnieuw strikt gevalideerd:
kernel-afzender, tag, geen capabilities, exacte lengte, magic/versie/opcode,
requestID/epoch/clientID, status, alle gereserveerde bytes, enumwaarden,
modifier-/knopbits, globale coördinaten, bewegingsgrenzen en overflowvelden.
De 107 ondersteunde physical-keyvarianten zijn expliciet benoemd; er zijn geen
unsafe enumcasts of decodertrucs. Onbekende berichten worden nooit Empty,
een event of een ACK. Onverwacht ontvangen echte caps worden afzonderlijk
opgeruimd voordat het eigen kanaalpaar sluit.

RequestIDs beginnen bij 1, worden nooit hergebruikt en falen vóór wrap. Alleen
een bekende kernel-EAGAIN vóór enqueue mag dezelfde nog niet gepubliceerde
aanroep herhalen. Na enqueue volgt alleen receive/yield, nooit opnieuw send.
Polling is eindig en begrensd door `MAX_POLL_BUDGET`.

Een verloren of ongeldige reply sluit de sessie. `Error::Interrupted` bewaart
`Delivery::NotEnqueued`, `Enqueued` of `Unknown`, plus transport-, protocol- en
opruimfout waar beschikbaar. Vooral een verloren ACK kan al zijn uitgevoerd;
de client meldt dat niet als succes en speelt de gebeurtenis niet opnieuw af.
Een nieuwe sessie na onzekerheid vereist expliciete herstelafspraken, nieuwe
grants en reconciliatie door de app/broker. Er is geen automatische reset of
cloud-/hostfallback.

## Uitgevoerd bewijs en open grenzen

Deze standalone crate bouwt offline met een eigen lockfile. De enige nieuwe
typeafhankelijkheid is de door de UI-eigenaar geleverde veilige publieke
`MouseButtons::checked`-constructor, broncommit `5f887648`.

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=build/input-client-target python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/input-client/Cargo.toml
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=build/input-client-target python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path sdk/input-client/Cargo.toml --all-targets -- -D warnings
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=build/input-client-target RUSTFLAGS='-C no-redzone=yes -C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2' python3 -B tools/dev.py cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/input-client/Cargo.toml --target x86_64-unknown-none --message-format=json-render-diagnostics
```

Er slagen 13 tests: één bereikbare interne uitputtingsgrens en twaalf gerichte
gesprekken via de echte capabilityengine en inputservice, inclusief echte
queue-EAGAIN, focusannulering tussen PEEK/ACK, verloren PEEK/ACK-replies,
ongeldige afzender/tag/wire, replay, foutieve muisvelden, echte ontvangen caps
en een mislukte close. De hostadapter gebruikt testdata; dit is geen native
toetsenbord-, compositor- of Materiaal-acceptatie. Clippy/fmt en de scalar
none-targetbuild slagen. De concrete bron-/artefacthashes staan in
`verification/pre-native.json`.

De native appcaller, echte inputdienst-IPC in de samengevoegde image,
supervisor-lifecycle en QMP-/bedieningsbewijs blijven afzonderlijk vereist.
Dit SDK-fundament rondt de volledige testrelease niet af.
