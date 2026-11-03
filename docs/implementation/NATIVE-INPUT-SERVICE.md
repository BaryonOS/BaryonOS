# TASK-NATIVE-INPUT-PREP — invoerdienstcore vóór native koppeling

Taakcontract, 30 oktober 2026, basis `309ff53`. Eigenaar: `native_ui_apps`;
hoofdagent bezit kernel-/driver-/compositorintegratie. Alleen nieuwe bestanden
onder `services/input-core/` en dit document. Geen syscall, kernelrecht,
workspace-membership of bestaand SDK-/UI-interface wordt gewijzigd.

## Werkelijke uitgangspositie

`ui-core` levert geteste Set-1-/PS2-ID-0-decoders en fysieke-down-/focusrouting.
G0 levert uitsluitend kernelpolling en framebufferdiagnostiek. Een native
InputSource-broker, beschermde ring-3-ingress, compositor en inputdienst-loop
ontbreken nog. Deze core hergebruikt `ui-core` en de bestaande SDK-enveloppen;
hij verricht geen I/O en bewijst geen werkende native invoerdienst of GUI.

## Autoriteit, levensduur en foutgedrag

- Bootstrap bindt service-epoch, bron-PID, compositor-PID, supervisor-PID en
  maximaal acht client-ID/PID-paren. Vier verschillende RECV-endpoints passen
  binnen het bestaande kernelquotum: bron, focus, lifecycle en gedeelde apps.
  Alleen vooraf verleende SEND-caps bereiken de betreffende ingress.
- `Received.sender()` komt uitsluitend van echte `native::receive`; een
  `Received::from_bytes` op de host bewijst geen herkomst. De core controleert
  werkelijk ontvangen endpoint én kernel-PID. Payload, tag en geraden ID's
  verlenen geen bevoegdheid. Een app kan geen input injecteren, focus wijzigen,
  een ander client-PID kiezen of procesdood melden.
- De compositor wijzigt alleen focus; de supervisor meldt client-/bron-/
  compositordood op grond van echte kernelobservatie. Een gestorven client-ID
  wordt in deze service-lifetime niet opnieuw gebonden. Bron-/compositordood
  blokkeert invoer totdat de afgesproken beschermde herinitialisatie is voltooid.
- Bij focuswisseling, invoerverlies of een volle eventqueue wordt oude input
  ongeldig. Een afzonderlijke cancel-watermark krijgt voorrang boven events;
  zij wist alleen generaties tot en met de genoemde grens. Bekende ingedrukte
  toetsen/knoppen gaan via de bestaande router niet over naar een nieuwe focus.
- Apps lezen met PEEK en bevestigen met ACK van precies het aangeboden
  eventnummer. Een mislukte IPC-send consumeert geen event. Oude ACKs kunnen
  een nieuwe cancel of nieuw event niet wegnemen. De native adapter controleert
  de antwoordgeldigheid direct vóór send en verwerkt dat zonder tussenliggende
  focus-/lifecyclemutatie in dezelfde servicethread.
- De bron configureert daadwerkelijk Set 1 en muis-ID 0 vóór SYNC. Samples
  dragen monotone bronvolgnummers en werkelijke i8042-status/data. Een gat,
  controllerfout of defecte packetstroom sluit samples tot expliciete SYNC na
  drain/reinitialisatie. Geen byteheuristiek claimt herstel na ongemerkt verlies.
- Tellers lopen niet rond. Globale event-/focus- of beschermde bronteller-
  uitputting sluit de core; een uitgeput app-requestnummer blokkeert uitsluitend
  die client. Anders kon een app met één MAX-verzoek alle invoer stoppen.
  Bij een gestopte core sluit de adapter de verbindingen en meldt annulering. Onverwacht
  ontvangen gedelegeerde caps worden geweigerd en moeten door de adapter met
  `native::close_received_caps` worden gesloten; de core bezit geen syscall-I/O.

## Wire v1

Eigen protocolpayload van 96 bytes binnen bestaande Message320, little-endian;
geen Rust-structcast. Header: `INP1`, versie u16=1, opcode u16, request-ID u64,
service-epoch u64, arg0 u64 en arg1 u64. Overige requestbytes zijn nul.
Tag `0x4e49_4e50_5554_0001`; geen overgedragen capabilities.

Bron: SYNC (profiel 1), SAMPLE (statusbyte/data-byte). Focus: FOCUS (client-ID,
0 wist focus). Lifecycle: CLIENT_DEAD, SOURCE_DEAD, COMPOSITOR_DEAD. Apps:
PEEK en ACK(eventnummer). Rollen kunnen elkaars opcodes niet gebruiken.
Verkeerde epoch, noncanonieke velden, onbekende opcode, ontbrekende/extra bytes
en oude requestnummers worden geweigerd vóór een bevoegdheidswijziging.

Antwoorden binden opcode+replybit, request-ID en service-epoch. Een PEEK bevat
Empty of één key/mouse/cancel, met eventnummer en focusgeneratie. Key-ID volgt
de expliciete `PhysicalKey`-u8-representatie van dit broncontract; compatibele
uitbreiding vraagt behoud van bestaande nummers of een nieuwe wireversie.
Pijltjes zijn fysieke toetsen; Unicode-/tekstlayout en hit testing blijven
afzonderlijke compositor-/tekstlaaginterfaces.
Aanvulling Veld-04: reply-eventtype4 is `PointerLeave`, uitsluitend een
door de compositor waargenomen uitgang uit een toegewezen appbody. Bytes
41..47 en64..95 zijn nul; er reizen geen buitenbodycoordinaten, knopbits,
toetsen of capabilities mee. Eventnummer, generatie, afzender en correlatie
blijven verplicht. Dit is geen cancel-watermark, focuswisseling of
klembordgebaar. De hardwarebron en servicecore genereren dit event niet;
de twee-appcompositor verstuurt het via de bestaande gebonden appqueues.
De twee vast gebootstrapte apps en SDK worden gezamenlijk herbouwd.
Oudere decoders weigeren type4; compatibiliteit met oudere derdenapps is
hiermee niet bewezen en vereist later expliciete protocolonderhandeling.
Status 0 is succesvol en 1 is Empty; negatieve statussen 2–18 zijn de expliciete
`Error`-nummers. `error_reply` maakt alleen een negatieve reply voor een al
gebonden afzender en canonieke huidige-epoch-request. Overige protocolfouten
worden geweigerd en lokaal gemeld; een appfout mag niet de gedeelde app-ingress
van andere clients sluiten. De adapter sluit onverwachte caps en kan alleen
de betrokken clientgrant intrekken. Een werkelijk gestopte core beëindigt zijn
hele epoch en laat alle betrokken clients input annuleren.
Er wordt geen succes of lege input als vervanging voor een fout uitgegeven.
Empty en gewone commandobevestigingen beschrijven de toestand bij afhandeling;
zij verlenen geen actuele focus. Alleen replies met input-/cancel-eventnummer
worden bij focus-/lifecyclewijziging ongeldig. Een later ontvangen Empty wist
geen inputstaat of nog wachtende cancel-watermark.

## Acceptatie van deze voorbereiding

Hosttests gebruiken de echte capability-engine voor kernel-envelopherkomst en
weigering van onbeschermde routes, naast vaste malformed-wirecases. Verder:
focuswissel/death, oude ACKs, queueoverflow/cancel, streamgaten, controllerfouten,
PS/2-muis/keyboard, no-focus-resetregressie, telleruitputting en begrensd geheugen.
Compileer de library daadwerkelijk voor `x86_64-unknown-none`; draai Clippy
en fmt in een eigen targetmap. Hostmodel en targetbuild krijgen geen native
U01/U02-punten.

Native vervolgpoort blijft open: beschermde sourcecap+driverloop, kernel-
provenance en echte procesdeathobservatie, vier ingresses en replyroutes,
QMP-toetsen/muis in twee geïsoleerde clients, EAGAIN/herstart en compositorfocus.
Een eventueel opnieuw opgestarte dienst krijgt een nieuwe epoch en nieuwe
endpointgeneraties; oude lokale tokens zijn geen herbruikbare autoriteit.

## Uitgevoerd bewijs — 30 oktober 2026

Op basis `309ff53` plus deze nieuwe core zijn daadwerkelijk geslaagd (exit 0):

```sh
cargo +1.90.0 test --manifest-path services/input-core/Cargo.toml --locked --offline -- --nocapture
cargo +1.90.0 clippy --manifest-path services/input-core/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build --manifest-path services/input-core/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline
cargo +1.90.0 fmt --manifest-path services/input-core/Cargo.toml --check
git diff --check
```

**20 hosttests** (19 grens-/lifetime-/wirecases plus telleruitputting) slagen.
De echte capability-engine levert in die hostproeven afzender-PID en scoped
receive, blokkeert vervalste sender-envelope en echte queuebackpressure. Dit
blijft een hostmodel van die kernelbouwsteen, geen native invoerdienstbewijs.
Gemeten `size_of::<Service>()` is 4256 bytes op x86-64, onder het lokale
8-KiB-corebudget; de library gebruikt geen heap of externe dependencies.
Productie-afhankelijkheden zijn uitsluitend bestaande lokale ui-core en SDK;
capability-core is alleen dev-dependency. Shared Cargo-/kernel-/SDK-/UI-bestanden
zijn niet gewijzigd.
Een afzonderlijke agent herhaalde de oorspronkelijke 18 tests, Clippy en
freestandingbuild onafhankelijk op `e228f0d`, zonder concrete blocker. De
reviewverduidelijkingen over clientlokale telleruitputting en Empty als
observatie zijn daarna in twee extra regressies vastgelegd; de IPC-queueproef
eist nu precies kernel-EAGAIN. Productiecode en wiregedrag zijn daarbij niet
gewijzigd. Dit is een AI-review binnen de genoemde scope, geen menselijke audit.

Compiler en bron-/log-/targethashes staan in
[`services/input-core/verification/host-target.json`](../../services/input-core/verification/host-target.json).
Volledige lokale logs: `build/input-core-checks/`; targetmap:
`target/input-core/`. Toolselectie gebruikte bestaande
`LOCALAPPDATA=/workspace/nieuw-os/.tools` via `tools/common.py::environment()`.
Een initiële testcompile vereiste een expliciete u64-annotatie in de testhelper;
de bovenstaande controles zijn daarna op de uiteindelijke bron herhaald.

Nog **niet uitgevoerd**: native hardware-ingress, echte sourcecap, driver- en
compositorproces, procesdeathnotificaties, QMP-toets/muis, focus op echte surfaces,
native dienstherstart en producttoegankelijkheid. De overige voorwaarden uit
FULL-TEST-RELEASE blijven open. De bronbasis blijft de eigen microkernel en de
bestaande [inputdecoderafspraken](NATIVE-INPUT.md) en Message320-contracten.
