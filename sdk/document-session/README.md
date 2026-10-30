# Begrensde documentbewerksessie

De actuele volume-route staat in `nieuw_os_document_session::volume`. Zij
gebruikt de bestaande streamende volumeoperaties met het v7-uploadtoken voor
documenten tot 64 KiB,
met door de app beheerde buffers, stapsgewijs bewaren en annuleren. Zie
[contract en integratie](../../docs/implementation/VOLUME-DOCUMENT-SESSION.md).
De onderstaande catalogusroute blijft beschikbaar voor bestaande M6c-clients;
haar 128-bytegrens is geen nieuwe productkeuze voor Materiaal.

`nieuw-os-document-session` is een allocatorvrije `no_std`-library met
`forbid(unsafe_code)` boven de bestaande `nieuw_os_catalog_client::Session<T>`.
Zij beheert één reeds bestaand document en een lokale conceptbuffer van
maximaal **128 bytes**. Object-ID, naam, de bestaande catalogusprotocollen
en het opslagformaat blijven behouden. Inhoud is opaque bytes: deze laag
voert geen UTF-8-, tekst-, bestands- of schijfbeleid in.

## API voor G1/Materiaal en andere apps

| Aanroep | Gedrag |
|---|---|
| `DocumentSession::new(catalog, EditMode)` | Neemt één bestaande catalogussessie over; kent geen nieuwe bevoegdheid toe. |
| `open(object)` | Laadt een bestaand document via echte GET. Mislukking behoudt de vorige inhoud. Een vuil concept blokkeert de overstap. |
| `draft()` | Geeft een begrensde, onveranderlijke view. `Draft` kan expliciet worden gekopieerd; getters zijn `object`, `loaded_revision`, `name`, `contents`. |
| `replace_contents(bytes)` | Vervangt de lokale inhoud, maximaal 128 bytes; geen IPC of opslagactie. |
| `replace_range(start..end, bytes)` | Voegt in, vervangt of verwijdert binnen bytegrenzen. Alle controles vinden vóór wijziging plaats. Tekst-/grapheemgrenzen horen bij de aanroepende app. |
| `is_dirty()` | Vergelijkt met de laatst geladen of bevestigde bytes; terugzetten naar die bytes maakt het concept weer schoon. |
| `save()` | Stuurt bij een gewijzigd concept één PUT met exact de geladen CAS-revisie. Een geldige ACK verhoogt de lokale revisie en maakt het concept schoon. Zonder wijziging wordt niets verstuurd. |
| `is_conflicted()` | Een Stale-antwoord heeft de lokale basis ingehaald. Het concept blijft intact; opnieuw save doet geen nieuwe aanvraag zolang dit conflict niet expliciet is opgelost. |
| `is_uncertain()` | Een save kan al zijn uitgevoerd. Deze toestand is blijvend voor dit sessieobject; er wordt niet opnieuw verstuurd. |
| `discard_changes()` | Zet alleen lokale bytes terug naar de laatst geladen/ACKed toestand. Geen GET, PUT, revisieverversing of oplossing van onzekerheid. |
| `reopen()` | Expliciete GET voor het huidige object, mits er geen onverworpen wijzigingen of onzekere save zijn. |
| `list()` | Leest de catalogus van het toegekende domein via dezelfde eigenaar van de endpoints. Behoudt concept, geladen revisie en conflictstatus; gesloten of onzekere sessies verzenden niets. |
| `close()` | Sluit via G03 beide endpoints; het concept blijft leesbaar. Geen verborgen syscall in `Drop`. |

`EditMode::ReadOnly` is uitsluitend lokaal appbeleid dat edit/save weigert.
`ReadWrite` verleent geen schrijfrecht. De kernelgebonden endpointcapability
en de echte dienst blijven beslissen. Een canoniek `Access`-antwoord blijft
een fout en behoudt het concept. Intrekking van een echte endpoint werkt
ook als de app `ReadWrite` gebruikt.

Houd het `DocumentSession`-object in de appstaat, zodat fouten uit een
actiehandler het concept niet vernietigen. Een handler kan bijvoorbeeld
`replace_contents` en daarna `save` aanroepen via `&mut DocumentSession<T>`.
Een fout geeft de app een beslissing terug; er wordt niet automatisch
herladen, gemengd, herhaald of overschreven.

### Conflict

Bij `Error::Conflict` blijven concept en geladen revisie ongewijzigd. Bewaar
desgewenst eerst `let retained = *editor.draft().unwrap();`. Laat de app
expliciet kiezen tussen behouden/vergelijken of verwerpen. Alleen een
expliciete `discard_changes()` gevolgd door `reopen()` haalt een nieuwe
basis op. De app mag daarna bewust gekozen bytes opnieuw aanbrengen; de
volgende save gebruikt dan de nieuw geladen revisie. Deze library beslist
niet automatisch dat het oude concept over de nieuwe versie moet worden
geschreven.

### Onzekerheid en verbroken transport

Een verloren antwoord nadat de aanvraag is geplaatst, een ongeldige
respons of `Unknown` mag geen gewone herhaalactie worden. De lokale bytes
en oude revisie blijven behouden; G03 sluit de sessie en G07 blokkeert
verdere saves. `discard_changes` wist die onzekerheid niet. Bewaar het
concept, sluit de oude sessie en gebruik pas na onafhankelijk herstel een
nieuwe catalogussessie met nieuw gevestigde grants. Laad de daadwerkelijke
opslagtoestand vóór een eventuele nieuwe gebruikerskeuze.

Een aantoonbare EAGAIN-timeout vóór enqueue houdt hetzelfde concept intact,
maar beweert niet dat een mutatie is uitgevoerd. Ook die transportverbinding
is gesloten; er volgt geen automatische reconnect. Lokale conceptbewerking
blijft mogelijk terwijl opslag niet beschikbaar is.

Ontbrekende objecten geven `NotFound` met de bestaande revisie; deze laag
maakt niet stilzwijgend een leeg document. Aanmaken, hernoemen, verwijderen,
zoeken en GUI-interactie blijven afzonderlijke apphandelingen via de
bestaande catalogusinterfaces.

## Controles en grenzen

De hosttests gebruiken de echte capability-engine voor Message320-IPC,
G03-transport, B01-correlatie, catalogusvalidatie, `State::prepare/commit`
en volledige `Recovery` over de 32 bestaande journalrecords. Het hostmodel
ordent append-WRITE, geslaagde FLUSH, commit en ACK. Een verloren reply
wordt pas na de echte corecommit geïnjecteerd; een nieuw servicemodel bouwt
zijn toestand uitsluitend uit de opgeslagen recordbytes op. Beschadiging
van zo'n record wordt door de echte Recovery afgewezen.

Dit zijn **hostmodeltests**. De library wordt daarnaast voor
`x86_64-unknown-none` gebouwd. G11 voegt een afzonderlijke opt-in native proef
toe: de bestaande M6c-gebruikersprocessen openen een bestaand document,
bewerken de 128 inhoudsbytes, bewaren met de geladen revisie en openen
opnieuw. Een aparte koude VM leest via dezelfde library de echte image terug.
De poort vereist `NOS_DOCUMENT_SESSION=1` én `NOS_SDK_RUNTIME=1`; alleen de
normale M6c-case en recovery zijn toegestaan. De bestaande kernelobserver,
capabilitygrenzen, 16/3 verzoeken per client en externe disk-oracle blijven
behouden. Actuele uitgevoerde resultaten en hun exacte bronbinding staan in
`docs/PROJECT-STATE.md`; compilatie of deze beschrijving is geen VM-bewijs.
Deze deterministische proef heeft geen grafische bediening of algemene
appstart. G1/Materiaal behoudt een afzonderlijke implementatie en acceptatie.

```sh
CARGO_TARGET_DIR=/tmp/nieuw-os-g07-target cargo test --manifest-path sdk/Cargo.toml --workspace --locked --offline
CARGO_TARGET_DIR=/tmp/nieuw-os-g07-target cargo clippy --manifest-path sdk/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
CARGO_TARGET_DIR=/tmp/nieuw-os-g07-target cargo build --manifest-path sdk/Cargo.toml -p nieuw-os-document-session --lib --release --target x86_64-unknown-none --locked --offline
```

Geen externe crate is toegevoegd: alle dependencies zijn de bestaande lokale
SDK-, catalogus- en voor tests capability-crates. Het source-/testbewijs en
de oorspronkelijke librarygrens staan in `docs/verification/parallel-g07/`.
