# capability-core — M4/M5 authority engine

Dependencyvrije `no_std`-bibliotheek met `forbid(unsafe_code)`. Alle proces-,
handle-, node-, endpoint- en queueopslag heeft een vaste capaciteit. Er is geen
allocator, klok, I/O of userpointer in deze crate. Kernelinitialisatie gebruikt
`static mut ENGINE: Engine = Engine::new();`; `new` is een const initializer,
zodat de grote tabellen niet op een kernelstack worden opgebouwd.

## API en transactiegrens

`Engine` biedt `register`, `create_endpoint`, `bootstrap_delegate`,
`check_endpoint`, `close`, `derive`, `revoke`, `send`, `receive`, `exit`, `stats`
en `check_invariants`. PID's zijn `u64`; registratie is strikt stijgend en een
oude PID kan na exit niet terugkeren. Een procesrij mag opnieuw worden gebruikt,
maar slotgeneraties en gepensioneerde slots blijven behouden.

`Message` (alias `Message320`) bevat de velden uit het vaste 320-byte-protocol.
`decode(&[u8; 320])` valideert een **sendrequest**: `len/cap_count` geven
`MessageSize` bij overschrijding, niet-canonieke sender/reserved/unused/tailvelden
geven `Invalid`. Encode/decode gebruiken bytes met little-endian integers,
zonder structcast of alignmentaanname. `encode()` kan ook een receiveantwoord
met een ingevulde `sender_pid` schrijven; senddecode accepteert dat antwoord
bewust niet als nieuw sendrequest. `send` valideert ook rechtstreeks opgebouwde
`Message`-waarden.

De kernel controleert syscallnummer/argumenten, size en endpointbevoegdheid vóór
de usercopy. Hij maakt één eigen inputsnapshot en roept daarna decode/send aan.
Voor receive valideert hij **alle 320 outputbytes vóór `Engine::receive`**.
Receive reserveert alle lokale slots vóór queuewijziging; daarna moet de kernel
zonder nog mogelijke fout de geïnitialiseerde output kopiëren. Deze afspraken
zijn vereist: de core valideert geen usergeheugen en kan een foutieve kernelcopy
achteraf niet terugdraaien.

Lookupvolgorde: lokale slot/generation → node/generation/revoked →
object/generation/open → vereist recht → aangevraagde velden → resources.
Een cap zonder DERIVE/TRANSFER met óók onbekende nieuwe rechten geeft daarom
`Access`; een cap met het vereiste recht en onbekende rechten geeft `Invalid`.
Alle foutwaarden zijn exact -1 t/m -11 via `Error::code()`.

## Eigendom, intrekking en levensduur

- 16 processen, 255 bruikbare slots per proces, 1.024 nodes, diepte maximaal acht.
- 64 endpointrecords, maximaal vier toegewezen records per eigenaar, 16 FIFO-
  berichten per endpoint en maximaal twee overgedragen caps per bericht.
- TRANSFER maakt een nieuwe afgeleide node, zonder de bronhandle te verplaatsen.
  RECV wordt nooit overgedragen; een eigen lokale derive mag RECV behouden.
- `revoke` trekt alleen afstammelingen in. `close` trekt afstammelingen niet in
  en ruimt ook revoked/closed handles op. Exit trekt eigen slotnodes en door het
  proces uitgegeven nodes plus afstammelingen in.
- Revocation annuleert een volledig queued bericht zodra één cap daarin is
  ingetrokken. De resterende FIFO-volgorde blijft behouden. Al afgeleverde bytes
  blijven bij de ontvanger.
- De laatste actieve RECV-handle sluiten, of eigenaar-exit, sluit het endpoint
  en annuleert zijn queue. Overige niet-ingetrokken handles geven Closed;
  ingetrokken nodes blijven Revoked geven vóór de objectstatus wordt bekeken.
- Refcounts tellen lokale slots, kinderen en queueverwijzingen. Parentnodes en
  objecttombstones blijven bestaan zolang verwijzingen bestaan. Gesloten
  tombstones bezetten daarom nog objectcapaciteit, ook de eigenaarslimiet; na
  de laatste verwijzing keert die capaciteit terug. Geen hergebruik of wrap van
  generationtellers: een maximale generation wordt bij vrijgave gepensioneerd.
- Intrekking bekijkt maximaal 1.024 nodes met maximaal acht ouderstappen per
  node en maximaal 1.024 logische queueberichten. Garbage collection gebruikt
  één nodescan met begrensde vooroudercascade, zonder recursie of heap.

`check_invariants` telt referenties onafhankelijk opnieuw en controleert
eigenaarschap, parentrechten/diepte, live generaties, queuecaps en resourcegrenzen.
`Stats` is Copy/Eq en bevat een volledige diagnostische `state_digest`. Dit is
een snelle, niet-cryptografische vingerafdruk voor tests, geen autorisatiemiddel
of bewijs tegen hashcollisies. Gebruik de volledige scan niet op ieder normaal
syscallpad; negatieve kernelproeven kunnen voor/na vergelijken. Hosttests
vergelijken de volledige `Engine`-toestand, zonder hashcollisieaanname.

## Werkelijk uitgevoerd

Rust 1.90.0, Windows MSVC-host, afzonderlijke targetmap
`build/capability-tests`. De oorspronkelijke M4-uitvoering bevat 21 geslaagde unit-tests:

| Contract | Hostbewijs |
|---|---|
| KS-10 | Exacte wireoffsets, canonieke velden, grote integers, niet-uitgelijnde bytebron, 2.560 afzonderlijke bitmutaties |
| KS-11 | Gequeuede snapshot onafhankelijk van latere bronwijziging; FIFO en ringwrap |
| KS-12 | Lokale numerieke handlecollision, cross-proces lookup, stale slot/generation |
| KS-13 | Rechtenvermindering, nulrechten, DERIVE/TRANSFER/SEND ontbreken, RECV-transfer, gecombineerde foutvolgorde; beide transfercaps volledig gevalideerd vóór depthquota |
| KS-14 | A→B→C, kind vóór ouder in de nodetabel, revoke vóór/na aflevering, gehele berichtannulering |
| KS-15 | Queue16, slots255, twee slots nodig bij receive, 1.024 nodes en twee-cap-send met één node vrij, diepte8/9, owner/global objectquota, maximale slot/node/objectgeneraties |
| KS-16 | Laatste RECV, tombstones, revoked-vóór-closed, sender-exit met queuecaps, uitgegeven rechten en volledige reclaim |
| Extra | Strikt monotone PID/rijhergebruik, exacte foutnummers, 800 deterministische gemengde transacties met volledige snapshots en invarianten |

KS-09, echte syscall-/ring3-grenzen, scheduler-/mappinglifecycle, usercopy en de
onfeilbare outputcopy worden door de kernelintegratie getest. Deze hosttests
bewijzen die grens niet. `NoSys`, `Fault` en `Type` hebben de juiste numerieke
API-waarde; NoSys/Fault zijn kernelgrenzen. De M5-uitbreiding hieronder toetst
Type daadwerkelijk tussen Endpoint en BlockDevice.

Exacte controles vanuit de projectroot, met de projecttoolchain en
`CARGO_TARGET_DIR` op `build/capability-tests`:

```text
cargo +1.90.0 fmt -p capability-core -- --check
cargo +1.90.0 test -p capability-core --locked --offline -- --test-threads=3
cargo +1.90.0 clippy -p capability-core --all-targets --locked --offline -- -D warnings
cargo +1.90.0 check -p capability-core --target x86_64-unknown-none --locked --offline
git diff --check -- crates/capability-core
```

Uitvoering: exit0 voor de laatste versies van deze controles. Lokale logs en
hashrapport: `verification/`; de hoofdagent bewaart de compacte bewijsversie
onder `docs/verification/`. Geen productiegegevens, crypto, dependencies of
hostinstellingen toegevoegd. Herstel: draai uitsluitend deze crate en haar
door root beheerde workspace-/kerneldependency terug, gezamenlijk met callers.

## M5 — getypeerd readonly BlockDevice

Vier extra objectrecords komen naast de bestaande64 endpointrecords; de
endpointcapaciteit wordt niet verkleind. `BlockScope {device,first_sector,
sector_count}` is immutable per object en wordt vóór creatie op nul/overflow
gecontroleerd. Relatieve sectoren worden via `BlockAuthorization::sector`
gecontroleerd vertaald; de transportlaag toetst daarnaast devicecapaciteit.

Bootmanifest-API: `create_block_device(owner,scope) -> handle` met rootrechten29.
`check_block(pid,handle,BLOCK_READ) -> BlockAuthorization` valideert handle/
generation/revoked/object en vervolgens **type vóór recht**. De opaque
`object_key()` bevat typegescheiden objectindex en generation. `scope()` geeft
alleen een kopie van de vaste scope; er is geen userspace-creatiesyscall.

Voor BlockDevice zijn READ1, DERIVE4, TRANSFER8 en REVOKE16 geldig; bit2 is
ongeldig. Endpoint-SEND1 kan geen BlockDevice-READ1 worden. Een READ-cap zonder
DERIVE geeft Access vóór een fout nieuw rechtenmasker; een geldige admincap met
bit2 geeft Invalid. Lokale derive en IPC-transfer gebruiken dezelfde begrensde
boom/refcounts/revocation/issuer-exit als endpoints. BlockDevice heeft geen
RECV-handle of queue en valt niet onder last-RECV-closure. Na adminclose blijft
een afgeleide READ-handle bruikbaar. Het laatste noderef ruimt het object op;
owner-exit sluit nog gerefereerde tombstones.

De huidige M5-hostsuite bevat26 tests: alle21 M4-tests plus vijf BlockDevice-
transactieproeven. Gezamenlijke virtio-/capabilitychecks en actuele hashes staan
onder `docs/verification/m5/core/report.json`. De kernel moet poll opnieuw
autoriseren: een eerder teruggegeven `BlockAuthorization`-kopie is geen blijvende
vrijstelling van revocation. Tickets binden PID + objectkey/generation +
transportepoch, onafhankelijk van de gebruikte lokale handle; een andere nog
geldige READ-cap naar hetzelfde scopeobject binnen dezelfde PID mag pollen.
## M6 — expliciete WRITE/FLUSH-rechten

Block WRITE32 en FLUSH64 verruimen uitsluitend expliciet gemaakte grants.
Legacy `create_block_device` blijft root29; de nieuwe manifest-only
`create_block_device_with_rights(owner,scope,rights)` accepteert uitsluitend
Blockmasker125. Endpointmasker31, Block-bit2, scopes, quotas, refcounts en
generations behouden hun grens. Globale KNOWN_RIGHTS127 wordt vervolgens
typespecifiek begrensd. READ-only grants krijgen geen WRITE/FLUSH.

Zie [M6-API](../virtio-core/M6-API.md) voor precieze signatures, publicatie/
Unknown-beleid, testdekking, compatibiliteitsaanpassing van de oude unknown-bit32
probe en de grenzen tussen pure hosttests en echt kernel-/devicebewijs.
