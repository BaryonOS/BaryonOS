# G03 — native transport voor de bestaande catalogusclient

`nieuw-os-catalog-client` verbindt de bestaande
[`catalog_core::client`](../../services/catalog/src/client.rs) met de
[SDK-runtime](../runtime/README.md). De crate is allocatorvrij, `no_std` en
verbiedt `unsafe`. `NativeTransport` gebruikt de echte int80-wrappers en is
uitsluitend beschikbaar op `x86_64-unknown-none`. Er is geen nieuwe syscall,
capabilitysoort, IPC-versie of opslagformaat.

`Session::new` krijgt dienst-epoch, verwachte kernel-PID, domein en twee reeds
gedelegeerde endpoint-handles. De sessie neemt de verantwoordelijkheid voor
die handles over; gebruik ze niet buiten de sessie. Constructie verleent geen
rechten. De kernel en dienst blijven de werkelijke autoriteitsgrens.

`exchange` ondersteunt de bestaande `Operation::{List, Get, Put, Rename, Delete}`
en retourneert `Reply::{Catalog, Document, Mutation, NotFound, ServiceError}`.
Documentnaam/inhoud zijn begrensde slices van het gevalideerde B01-document.
PUT/RENAME/DELETE houden de bestaande expected-revisioncontrole. Een
`ServiceError`, waaronder `Unknown`, is nooit een mutatiebevestiging.

## Begrensd wachten en uitkomsten

De caller kiest een pollbudget van 1 tot en met 1.000.000 per fase. De sendfase
en receivefase hebben ieder dat maximum; tussendoor staat de app de CPU af.
Alleen `EAGAIN` vóór enqueue mag de send herhalen. Zodra enqueue is bevestigd,
wordt uitsluitend ontvangen; geen timeout, foutantwoord of cleanup leidt tot
opnieuw verzenden van de aanvraag.

| Delivery bij een fout | Betekenis |
|---|---|
| `NotEnqueued` | De adapter heeft geen aanvraag gepubliceerd; bijvoorbeeld een volle queue of reguliere negatieve syscallretour |
| `Enqueued` | De kernel heeft enqueue bevestigd; verwerking kan al hebben plaatsgevonden |
| `Unknown` | De syscallretour is ongeldig; de adapter kan enqueue niet uitsluiten |

De pure B01-client behandelt een reeds voorbereide mutatie conservatief als
mogelijk onzeker. De adapter bewaart daarnaast bovenstaande preciezere
transportuitkomst. Geen van deze fouten bewijst een schijfrollback, annulering
of afwezigheid van latere dienst-I/O.

Bij timeout, transportfout, verkeerde identiteit/correlatie, ongewenste
capabilities of een terminaal dienstantwoord sluit de sessie. Zij probeert
beide eigen endpoints te sluiten. Ontvangen capabilities worden pas gesloten
nadat B01 de **oorspronkelijke** envelope heeft gezien; hun aanwezigheid wordt
niet verborgen door vooraf `cap_count` te wissen. Alle cleanupoperaties worden
geprobeerd, ook als een eerdere close faalt; de eerste cleanupfout blijft in
het resultaat zichtbaar. Een gesloten sessie maakt geen nieuwe transportcall.

`close()` beëindigt een gezonde sessie expliciet. Er is geen verborgen Drop-
syscall of automatische herstart. Na onzekerheid vraagt een nieuwe sessie
onafhankelijk vastgestelde recovery en nieuwe grants; alleen de constructor
opnieuw aanroepen met oude handles is geen herstelstrategie.

## Controleren en voorbeeld

```sh
cargo test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-catalog-client -p nieuw-os-sdk-runtime
cargo clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-catalog-client --all-targets -- -D warnings
python3 -B sdk/examples/catalog-read/build.py
```

De [catalogusleesapp](../examples/catalog-read/README.md) gebruikt deze typed
client voor LIST→GET. De hosttests gebruiken het werkelijke pure capabilitymodel
voor queues, PID's, delegatie, intrekking en capcleanup, en de bestaande
cataloguscore voor CAS en documentresultaten. De commit in dat hostmodel is
uitsluitend een geheugenwijziging: dit is geen native WRITE/FLUSH- of
duurzaamheidsbewijs. De native G02-runtimeproef en toekomstige gewone appstart
houden hun afzonderlijke acceptatiegrenzen.

Broncontracten: [M6c](../../docs/implementation/M6C-NAMED-CATALOG.md),
[M6d](../../docs/implementation/M6D-IO-UNCERTAINTY.md) en
[ABI/capabilities](../../docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md).
Brongebonden controles staan onder
[`parallel-g03`](../../docs/verification/parallel-g03/).
