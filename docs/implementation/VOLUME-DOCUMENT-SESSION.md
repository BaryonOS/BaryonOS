# FR-A03 — streamende documentsessie voor volume v6

Implementatie boven de bestaande `volume-core`-dienst, zonder nieuwe kernel-ABI,
opslagengine, capabilities of schijfformaat. De oude `DocumentSession` blijft
byte-/API-compatibel met het catalogusprotocol van M6c. De nieuwe module is
`nieuw_os_document_session::volume`.

## Interface en eigenaarschap

`VolumeSession<T>::new(transport, epoch, service_pid, domain, send, receive,
poll_budget)` neemt één reeds verleend endpointpaar over. Gebruik native
`nieuw_os_catalog_client::NativeTransport`; dezelfde transporttrait maakt
gerichte hosttests mogelijk. Alleen de kernel en dienst bepalen toegang.
`ReadWrite`, objectnummers en metadata verlenen geen rechten.

`volume::DocumentSession::new(volume, EditMode, Buffers)` beheert één document.
De aanroeper levert drie even grote, afzonderlijke buffers: concept, laatst
bevestigde inhoud en leesstaging. Capaciteit is 1 tot en met het bestaande
`volume_core::MAX_LENGTH` (64 KiB). De sessie maakt geen documentgrote arrays
op de stack en gebruikt geen allocator. De native app moet deze buffers
binnen zijn toegewezen geheugen beheren. Een te groot document geeft een
zichtbare `ContentTooLarge`; de vorige inhoud blijft behouden. De 64-KiB-grens
komt uit het bestaande volume en blijft een open productlimiet voor de
volledige testrelease.

| Aanroep | Werkelijk gedrag |
|---|---|
| `open(object)` | STAT en begrensde READ-verzoeken; alle delen hebben dezelfde revisie. Pas na een volledige checksumcontrole worden concept en basis vervangen. |
| `create(object, name)` | STAT moet afwezigheid bevestigen. De tombstonerevisie blijft de CAS-basis; pas save maakt het object. Een leeg document is geldig. |
| `draft()` | Onveranderlijke geleende view met object, geladen revisie, naam en bytes. Bewaar benodigde bytes vóór het vrijgeven van de callerbuffers. |
| `replace_contents`, `replace_range`, `rename_draft` | Alleen lokale wijzigingen; valideer bereik, capaciteit en naam vóór mutatie. Inhoud is bytes, tekstgrenzen horen bij de editor. |
| `save()` | Ongewijzigd concept: geen IPC. Anders BEGIN, APPEND-stukken van hoogstens 160 bytes en COMMIT met de geladen revisie. Alleen een gevalideerde COMMIT-ACK maakt de lokale basis schoon. |
| `begin_save`, `save_next` | Zelfde pad met hoogstens één APPEND of COMMIT per `save_next`. De app verwerkt tussen deze stappen invoer, voortgang en annulering. Elke uitwisseling heeft een begrensd pollbudget. |
| `cancel_save()` | ABORT op dezelfde upload-ID. Geen COMMIT, geen verlies van concept. Bewaren na annuleren is een nieuwe expliciete gebruikersactie. |
| `metadata()` | Geeft huidige STAT inclusief type, labels, collectie en versiegeschiedenis. Dit vervangt nooit stil de geladen CAS-basis. |
| `list(out)` | Paged LIST van het eigen domein. Alleen een volledige lijst met dezelfde domeinklok wordt in `out` gepubliceerd. |
| `discard_changes()` | Herstelt alleen de laatst geladen/bevestigde lokale basis; een nieuw onbewaard object wordt lokaal losgelaten. Lost onzekerheid of een conflict niet stil op. |
| `close()` | Probeert beide endpointcloses éénmaal; concept blijft leesbaar en lokaal bewerkbaar. Stuurt geen COMMIT. Geen verborgen syscalls of retries in Drop. |

Tijdens een upload zijn inhoudswijzigingen en andere documentoperaties `Busy`,
zodat APPEND niet verschillende conceptversies mengt. Een normale annulering
geeft de bestaande volumereservering vrij. Bij een verloren verbinding kan
ABORT niet worden bevestigd; de bestaande dienst vervangt een verlaten upload
bij een volgende geautoriseerde BEGIN in hetzelfde domein. Close is geen
claim dat staging reeds is vrijgegeven.

## Fouten en veiligheidsgrens

- De module gebruikt `volume_core::wire::validate_request` en de bestaande
  `client::validate_reply`. Versie, epoch, request-ID, object, revisie,
  lengtes, padding en operaties worden gevalideerd. De buitenste Message320
  moet van de verwachte dienst komen, de juiste tag hebben en geen
  capabilities bevatten. Onverwachte ontvangen handles worden gesloten;
  daarna worden beide sessie-endpoints gesloten.
- Een verzonden aanvraag wordt nooit opnieuw verzonden. EAGAIN mag alleen
  vóór enqueue opnieuw worden geprobeerd. Request-ID's lopen niet rond.
- `Stale` houdt het concept en de oude revisie vast en blokkeert volgende
  saves met `Conflict`. Alleen expliciet verwerpen en opnieuw openen haalt
  een nieuwe basis. Geen automatische merge of overschrijving.
- Bij verlies of manipulatie van de COMMIT-ACK, `Unknown`, of een andere
  onzekere COMMIT-uitkomst blijft het concept vuil en wordt de sessie
  permanent `Uncertain`. Ook lokale discard heft dat niet op. Behoud de
  gewenste conceptbytes en gebruik pas na onafhankelijk dienstherstel en
  nieuwe grants een nieuwe sessie; lees de feitelijke opslag vóór een
  bewuste volgende wijziging.
- Een canonieke `Io` tijdens data-/precommit-I/O is volgens het bestaande
  volumecontract een zekere abort. Die bewaart concept en vorige revisie;
  zij wordt niet ten onrechte als opgeslagen gemarkeerd. Intrekking door de
  echte capability-engine blokkeert volgende aanvragen.
- FNV-contentchecks detecteren toevallige beschadiging. Zij leveren geen
  encryptie, authenticiteit of antirollback; B02 blijft een afzonderlijke
  verplichte integratie. Deze library verruimt geen sleutel-/blokrechten.

## Verificatie en nog open acceptatie

`sdk/document-session/tests/volume.rs` gebruikt de echte capability-engine en
de bestaande `volume_core::service::Service` met een expliciet gelabelde
geheugenschijf. De tests behandelen 64-KiB-streaming, echte core-recovery,
annulering, ontbrekende/lege documenten, hernoemen, edits, metadata en LIST,
revisieconflicten, intrekking, ongeldige envelopes, half gelezen documenten,
zekere data-I/O-fouten en onzekere tabelwrites/verloren COMMIT-antwoorden.
De bestaande legacytests blijven afzonderlijk behouden.

Dit zijn hostintegratieproeven. Een geslaagde `x86_64-unknown-none`-librarybuild
bewijst uitsluitend targetcompatibiliteit. De hoofdintegratie moet dezelfde
sessie native in de echte gebruikersapp aanroepen, met gecontroleerde
image-/bronhashes, bedieningsbewijs, koude herstart en een onafhankelijk
schijforakel. Native Materiaal, overige programma's, grotere productopslag en
de volledige testrelease worden door deze deelstap niet afgetekend.

Alle nieuwe dependencies zijn bestaande lokale crates: `volume-core` hangt
al aan `catalog-core`. Er zijn geen nieuwe externe packages of buildscripts.
De integrator werkt uitsluitend de SDK-lockregistratie voor de toegevoegde
lokale dependency bij; geen protocol- of lockversieomzetting nodig.

### Uitgevoerde deelcontrole

Op de FR-A03-appbranch slagen 27 tests van deze crate (14 legacy, 12 volume-
integraties en één request-ID-grens), Clippy met `-D warnings`, formattering en
de release-librarybuild voor `x86_64-unknown-none`. De controles draaien
offline op Rust 1.90.0 met eigen targetmap. Omdat de hoofdagent de SDK-lockfile
bezit, is de SDK-boom eerst naar `/tmp/nieuw-os-apps-check/sdk` gekopieerd en
is uitsluitend daar de lokale dependencyregistratie gegenereerd. De geteste
library-/testbestanden zijn bytegelijk aan deze worktree. Exacte commando's,
exitstatussen en uitvoer staan in `/tmp/nieuw-os-a03-verification/`.

```sh
cargo +1.90.0 test --manifest-path /tmp/nieuw-os-apps-check/sdk/Cargo.toml --locked --offline -p nieuw-os-document-session
cargo +1.90.0 clippy --manifest-path /tmp/nieuw-os-apps-check/sdk/Cargo.toml --locked --offline -p nieuw-os-document-session --all-targets -- -D warnings
cargo +1.90.0 build --manifest-path /tmp/nieuw-os-apps-check/sdk/Cargo.toml --locked --offline -p nieuw-os-document-session --lib --release --target x86_64-unknown-none
cargo +1.90.0 fmt --manifest-path /tmp/nieuw-os-apps-check/sdk/Cargo.toml --all -- --check
```

Voor alle vier geldt
`CARGO_TARGET_DIR=/tmp/nieuw-os-apps-check-target`; bestaande lokale
`CARGO_HOME`/`RUSTUP_HOME` selecteren de gepinde projecttoolchain. De native
appaanroep, grafische bediening en koude QEMU-integratie zijn in deze
deelcontrole **niet uitgevoerd** en blijven bij de hoofdintegrator open.
