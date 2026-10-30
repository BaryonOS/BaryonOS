# Gedeeld artifact-storecontract

**Status:** voorstel voor root-integratie; Apps- en AI-owneraftekening open.
**Eigenaar:** release-integratie. **Datum:** 31 oktober 2026.
**Bewijsgrens:** dit document beschrijft een contract, geen native store,
kernel-ABI, geserialiseerde Message320-operatie, dynamische launcher of
duurzaamheidsgarantie.

Dit contract scheidt grote immutable artifacts van het documentvolume. Het
documentvolume blijft voor documenten; zijn huidige object-/domeinlimieten
zijn geen artifactcapaciteit. Hetzelfde read-only pad kan later appimages en
AI-modellen bedienen, maar hun trust authorities blijven afzonderlijk.

## 1. Invarianten en niet-doelen

- Een read-handle is opaque, immutable en gebonden aan de geauthenticeerde
  clientcontext. De concrete service controleert die context, store-epoch en
  actuele toegang bij iedere operatie. Het handle-token zelf verleent geen
  rechten; revocation maakt vervolgreads stale of unauthorized.
- Descriptorvelden en SHA-256 zijn metadata en content-integriteitsinput,
  geen bewijs van herkomst, publishervertrouwen, modelveiligheid of
  uitvoerbaarheid. Geen `verified`-boolean.
- Reads zijn read-only. Dit contract verleent geen netwerk-, document-,
  sleutel-, launch- of beheercapability.
- Read-API-chunks zijn maximaal 4096 bytes. Dat is geen IPC-wireformaat en
  geen productlimiet voor artifactgrootte. De bestaande Message320-payload
  blijft maximaal 256 bytes; een toekomstige adapter moet grote reads
  opsplitsen zonder de IPC-limiet of capabilitygrens te omzeilen.
- Storagecapaciteit en quota zijn provider-/namespacegegevens in bytes.
  Een waarde van een kandidaatmodel, appfixture of documentvolume wordt geen
  globale OS-capaciteit.
- Installeren/publiceren is een aparte write-transactie. Een reservering
  vooraf is geen belofte van duurzame opslag. Alleen de concrete backend kan
  zijn commit-/flush-/recoverygaranties specificeren en bewijzen.

## 2. Logische artifactidentiteit en descriptor

De gedeelde store-identiteit is los van package-/modelprovenance:

| Veld | Type | Betekenis |
|---|---|---|
| `artifact_id` | `[u8; 32]` | Opaque ID toegewezen binnen de store |
| `immutable_generation` | `u64` | Exacte immutable bytegeneratie |
| `length` | `u64` | Exact aantal bytes |
| `sha256` | `[u8; 32]` | Verwachte volledige-inhoudsdigest |

`ArtifactRef` is het paar `(artifact_id, immutable_generation)`;
`ArtifactDescriptor` voegt `length` en `sha256` toe. Dit is een logische
in-memory vorm, geen `repr(C)`, canonieke bytecodering of IPC-wire-ABI.
Descriptorvelden zijn onbetrouwbare metadata totdat `open` ze tegen de
store-authoritatieve descriptor vergelijkt.

De huidige Apps-leaf gebruikt een ander 116-byte pakketdescriptor met
`artifact_id`, `source_id`, `source_generation`, `version`, `length` en
`sha256`; de AI-leaf gebruikt een lokale numerieke ID en `u64` generatie.
Deze zijn consumer-/leafmodellen, niet twee gedeelde store-ABI's. De
storemapping moet de packagebron-/intrekkingsgeneratie en packageversie
gescheiden houden van de immutable storegeneratie. De ownermapping en
aftekening zijn open; geen caller- of modelmetadata mag de store authority
fabriceren.

`open` vergelijkt de gevraagde descriptor met de store-authoritatieve
descriptor en opent uitsluitend die immutable generatie. Een digestveld bewijst
niet dat alle bytes al gelezen of gehasht zijn. Een consumer die bytes gaat
parseren of gebruiken, verifieert eerst de volledige stream tegen `length` en
`sha256`; gedeeltelijke plaintext/data wordt bij een mislukte integriteits-
controle niet als geldig artifact vrijgegeven. Een latere native adapter moet
exact aangeven waar die verificatie plaatsvindt.

## 3. Read-only API en begrenzing

De logische API is:

```text
open(descriptor) -> opaque ReadHandle
read_at(handle, offset: u64, output: &mut [u8]) -> byte_count
capacity() -> Capacity
```

`ReadHandle` bevat intern minimaal de descriptorbinding en store-epoch. Velden
zijn niet door callers te construeren of te muteren. De concrete service bindt
het handle aan de kernel-authenticated afzender; een caller-ID in payload,
numerieke locator of publieke epoch is geen authority. Iedere `read_at`
controleert opnieuw afzender, rechten, revocation en actuele epoch.

Voor `read_at` geldt:

1. `output.len()` moet tussen 1 en 4096 liggen; anders `Invalid`.
2. `offset == length` geeft expliciet `EndOfArtifact`, zonder backend-I/O.
   `offset > length` geeft `OutOfRange`.
3. Bereken met checked arithmetic `remaining = length - offset` en
   `requested = min(output.len(), remaining)`. Overflow of inconsistente
   descriptor-/store-state faalt gesloten.
4. De backend mag tussen 1 en `requested` bytes teruggeven. Een positieve
   short read vóór `length` is geldig; de caller vervolgt op
   `offset + byte_count`. Nul bytes vóór de gedeclareerde EOF is `Truncated`;
   meer bytes dan `requested` is `Protocol`.
5. De caller berekent de nieuwe offset met checked addition. De API geeft
   nooit lege succesdata als EOF-fallback.
6. Een stale epoch, ingetrokken handle of gewijzigde generatie geeft
   `Stale` of `Unauthorized`; er wordt niet opnieuw geopend onder een
   impliciet nieuwe generatie.
7. Iedere fout laat het resultaat expliciet ongeldig. Een backendfout wordt
   niet omgezet in succes, EOF, nulbytes of een andere generatie.

De logische foutset is `Invalid`, `Unauthorized`, `Stale`, `Corrupt`,
`Truncated`, `Overflow`, `EndOfArtifact`, `OutOfRange`, `NotFound`,
`Protocol` en provider-I/O-fout. De service-/wireadapter mag deze alleen naar
een bestaande, expliciete status mappen; zij mag oorzaak of onzekerheid niet
stil laten verdwijnen.

## 4. Capaciteit en quota

Een store rapporteert `capacity_bytes`, `used_bytes`, `reserved_bytes` en
`quota_bytes`, ieder als `u64`. Dit zijn momentopnamen, geen reservering of
commitgarantie. Beschikbare ruimte is:

```text
limit = min(capacity_bytes, quota_bytes)
committed = checked_add(used_bytes, reserved_bytes)
available = checked_sub(limit, committed)
```

Overflow bij optellen geeft `Overflow`; `committed > limit` geeft
`Corrupt`. Geen saturating arithmetic, negatieve/clamped waarde of
success-shaped fallback. De operationele `begin` blijft autoritatief en kan
na een eerdere capaciteitssnapshot nog `Full` of `Quota` geven.

## 5. Afzonderlijke schrijftransactie

Write-operaties zijn uitsluitend voor een expliciet geautoriseerde
install-/publishprincipal; read-only consumers krijgen deze interface niet.
De service haalt calleridentiteit uit kernel-authenticated berichtmetadata.
De transactie bindt minstens `(caller, request_id, service_epoch)` aan één
descriptor.

```text
begin(key, descriptor) -> reserve full length
write_chunk(key, exact_next_offset, nonempty_bytes <= 4096) -> next_offset
commit(key, descriptor) -> published immutable handle
cancel(key) -> cancelled
outcome(key) -> Pending | Committed | Cancelled | Unknown
```

- `begin` reserveert de volledige gedeclareerde lengte atomair vóór enig
  chunk wordt geaccepteerd. Een definitieve fout (`Full`, `Quota`, invalid,
  unauthorized, busy) betekent geen mutatie. `Unknown` betekent dat de
  mutatie-uitkomst onzeker is en de transactie wordt gepoisoned.
- Chunks zijn niet leeg, hoogstens 4096 bytes, volgen exact de verwachte
  offset en overschrijden nooit `length`. Replay, gat, overlap, overflow of
  bevestiging met een andere volgende offset faalt; een mismatch maakt de
  uitkomst `Unknown` wanneer een write mogelijk al heeft plaatsgevonden.
- `commit` is alleen toegestaan na exact `length` bytes. De backend
  controleert lengte en volledige SHA-256 opnieuw vóór atomische publicatie.
  Een digest-/lengtefout publiceert niets. Een `Ok` is geen algemene
  power-loss-, crashrecovery- of duurzame-media-garantie.
- `Unknown` blokkeert nieuwe chunks, commit, cancel en automatische replay.
  `outcome` mag alleen binnen dezelfde service-epoch worden geraadpleegd.
  Na serviceherstart is de oude epoch stale; herstel van de transactie vereist
  een afzonderlijk ontworpen en getest persistent journal. Er is hier geen
  success-shaped fallback of aanname dat restart gelijkstaat aan cancel.
- `cancel` geldt vóór de commitgrens. Bij onzekere uitkomst blijft de status
  `Unknown`; cancel mag die niet in `Cancelled` veranderen.

Daarmee blijft volledige reservering los van duurzaamheid. Een native store
moet later storage-I/O-fouten, flushvolgorde, crash-/koude recovery, orphan
cleanup, rollback en de precieze betekenis van `Committed` als aparte
acceptatie bewijzen.

## 6. Trust authorities en consumergrenzen

- **Apps:** `source_id`, `source_generation` en package-version zijn
  package-/source-identiteit; zij zijn niet de immutable storegeneratie.
  SHA-256 bindt bytes; het maakt de huidige
  in-memory SourceRegistry niet tot een persistente publisher trust authority.
  Appinstallatie verleent geen dynamic launch authority. Een kernel- of
  broker-geverifieerde launchcapability is een afzonderlijk toekomstig
  contract.
- **AI:** model-pin, formaatvalidatie, volledige streamdigest, publishertrust
  en geschiktheid voor inferentie zijn afzonderlijke controles. De numerieke
  leaf-ID moet door een store-assigned `artifact_id`-mapping worden vervangen
  of verpakt; die mapping is nog niet door de owner bevestigd. Deze reader
  kent geen netwerk- of persoonlijke-documentrechten toe. AI-inferentie blijft
  zonder netwerkcapability.
- **Shared store:** beheert immutable bytes, versie en lees-/transactiestatus.
  Het documentvolume, publisher keyring, B02/B03-root of caller-gevulde
  boolean wordt niet stil als store authority hergebruikt.

## 7. Implementatie- en acceptatiegrens

De eerste gedeelde code-stap is een dependencyvrije `no_std` contractcrate met
zuivere descriptor-, range-, capacity-, handle- en transaction-state
validators plus host-negatieventests. Zij mag geen kernel ABI, syscall,
Message320-layout, backend, package launch, modeldownload of succesvolle
install-/inferentieclaim toevoegen.

Minimaal te testen: descriptorlengte/canonicaliteit; nul-/verkeerde ID,
generatie en digest; stale/revoked handle; readbuffer 0/4097; exacte EOF en
offset voorbij EOF; positieve short read, nul vóór EOF, onmogelijke
backend-bytecount en checked-add-overflow; capacity add/subtract overflow en
`used + reserved > limit`; quota/full vóór writes; lege/grote chunk,
out-of-order/replay/overrun; mismatch-ack; incomplete/digest-foute commit;
Unknown op begin/chunk/commit/cancel; outcome met verkeerde transactie of
epoch; en restart-invalidatie zonder automatische retry.

Hosttests controleren alleen de contractlogica. Targetbuild, native store,
dynamic launch, publishertrust, modelbytes, inferentie, flushduurzaamheid,
herstel, performance en volledige FR-/rubricacceptatie blijven open.
