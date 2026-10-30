# Begrensde native desktopcontrolcodecs

MULTIAPP-CODEC-1, zelfstandige `no_std`-library, zonder allocator, transport,
kernelhooks of hostbackend. Contract:
[NATIVE-MULTIAPP](../../docs/contracts/NATIVE-MULTIAPP.md).

`app::Ingress` accepteert uitsluitend werkelijke SDK `Received`-envelopes van
de verwachte InitPID, APPCTRL2-tag, caps0 en exact64bytes. Binding komt uit het
gecontroleerde bootrecord; de constructor verleent geen capability. Apprequest
en reply binden opcode, requestID, boot-, instance-, activation- en surface-
generatie. Alleen Pause→Resume mag dezelfde activationgeneratie hergebruiken.
Requests hebben frame/status0; Pause-completion heeft frame0, Resume-completion
een nieuw werkelijk Presented-frame. APP-statussen buiten0 zijn gereserveerd.

`switch::Preset` bindt uitsluitend Materiaal1 en Calculator2 aan twee vaste
identiteiten met hetzelfde bootrecord en verschillende PID/surface-epochs.
De switchrequest bevat geen PID/handle/programma-/bestandsrechten. Ingress
authenticeert de compositor. Success bindt het preset-surface-epoch; Busy1,
Denied2 en Failed3 mogen geen positieve resultaatmetadata bevatten.

`RequestIds` en ingress-replayguards lopen nooit rond. Geldige tellers liggen
in1..=u64::MAX-1. Een geauthenticeerd, anders geldig request met IDMAX sluit de
betreffende ingressguard; een verkeerd gebonden of andere afzender kan die
guard niet vergiftigen. Guards gelden per eigen kanaalpaar/instantie; ze zijn
geen vervanging van de volledige Init-/app-toestandmachine.

Een request maakt met `expect(peer, previous_frame)` (app) of
`expect(peer, previous_activation, previous_target_frame)` (switch) een
one-shot Pending. Geef de werkelijke eerdere watermarks van deze binding mee;
0 is alleen de eerste waarneming. Stale resultaten geven geen completion.
De eerste reply, ook een verkeerde, sluit Pending. `abandon()` na een verloren
of geannuleerde uitwisseling is definitief. Er is geen send, automatische
retry, receive-loop, timeoutverruiming of fabricated Received in deze crate.
De native adapter beheert één uitstaande request en de echte bounded klok/
transportafhandeling. Bij onverwachte ontvangen caps moet hij alle caps
werkelijk sluiten; de codec kan die bevoegdheid zelf niet uitvoeren.
Een gedeelde registry moet eerst op kernelafzender, tag en actieve fase
demultiplexen: geef een bericht van appB niet aan appA's Pending. Een foutieve
verwachte reply sluit die eigen lane; andere principals krijgen geen macht
om een andere appuitwisseling via de gedeelde ontvangstroute af te breken.

`paused`, `presented` en `completed` coderen een gebonden wire-uitkomst. **Zij
bewijzen zelf geen quiesce, policyFocusACK, blit of volledig frame.** Roep ze
uitsluitend aan na die echte adaptervoorwaarden; een door de codec geaccepteerd
hostbericht is geen native bewijs. Frame-/activationmetadata uit de app mag
geen compositorcallback vervangen. SYS-v1 Ready/GO/STOP en de bestaande
single-app-GUI blijven gescheiden en ongewijzigd.

Gerichte checks, in een eigen targetmap:

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/desktop-control/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/desktop-control/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/desktop-control/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --release --lib --target x86_64-unknown-none --manifest-path sdk/desktop-control/Cargo.toml
```

De Engine-tests leveren echte kernelmodelafzenders/captransfers; peerpayloads
blijven expliciete hostprotocoltestdata. Native consumers, launcher, pauzeren/
hervatten,100/200%-chrome, passende GOP-mode,2-appfocus, gegevensbehoud en
visuele/gebruikersacceptatie zijn nog niet door deze library uitgevoerd.
