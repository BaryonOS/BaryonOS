# N01/N02 — versleutelde NAS-opslag en hybride transportproef

**Uitvoerbaar Node-hostprototype:** N01 levert echte AES-256-GCM-encryptie, lokale ciphertextopslag en bevoegdheidscontroles binnen de service-API. De oorspronkelijke opslagdemo opent geen sockets. N02 voegt een afzonderlijke echte TLS 1.3/X25519MLKEM768-proef toe, uitsluitend op loopback: zie [TRANSPORT.md](TRANSPORT.md). Dit is geen native Nieuw OS-dienst of geconfigureerde thuis-NAS; de transportecho is nog niet aan de store gekoppeld.

Deze implementatie volgt de gewijzigde opdracht van 27 oktober 2026: een NAS kan thuis staan of bij vrienden/familie, met afzonderlijk toegestane directe internettoegang. De NAS-beheerder mag zijn opgeslagen kopieën zelfstandig verwijderen of de opslagruimte intrekken, zonder gebruikerssleutel of handtekening. Die beheerbevoegdheid geeft geen decryptiesleutel. Oudere teksten met uitsluitend eigen LAN zijn voor deze afgebakende opdracht herzien door de nieuwe gebruikersinstructie; de hoofdagent beheert de gedeelde contractdocumenten.

## Rust-bouwstenen voor native integratie

[NAS-CORE-1](native-core/README.md) voegt een allocatorvrije `no_std`-core toe
voor ontvangergebonden uitnodigingen, sessies, quotumreservering,
revisieconflicten, duurzame intrekking en logische verwijdering van ciphertext.
[NAS-VOLUME-1](native-volume/README.md) sluit die core aan op de bestaande
Rust-volumeclient en ruimt ook diens herstelbare versiegeschiedenis op vóór
succes. De tijdelijke Rust-capaciteitsgrenzen staan bij de crate; de
N01-grenzen hieronder horen uitsluitend bij het Node-hostprototype.

[NAS-IPC-PROBE-1](native-probe/CONTRACT.md) levert nu de lokale 256-byte
IPC-adapter en drie afzonderlijk te starten procesrollen als freestanding
ELF. Het contract legt startupwords, endpoints, geheugen en normale/koude
proefassertions vast. Dit pakket configureert zelf geen kernelmanifest en
heeft nog geen QEMU-uitvoerbewijs.

De Rust-hostproeven en bare-metalbuild zijn **geen draaiende native NAS**.
Transportidentiteit, kernel-IPC, clientencryptie, automatische sync en
multidevice-key-lifecycle blijven open. [Controlebewijs](native-volume/verification/README.md):
P01–P04 **0/5 → 0/5**, geen nieuwe rubricpunten.

## Uitvoeren

Vanaf de projectroot, met Node.js 24:

```sh
node services/nas/demo.mjs
node services/nas/transport-demo.mjs
node --test --test-isolation=none services/nas/tests/*.test.mjs
```

De eindige demo maakt een eigen `nieuw-os-n01-*`-wegwerpmap onder de systeemtempmap, schrijft en leest één synthetisch versleuteld document, controleert de beleidsgrens en trekt vervolgens de opslag in. Hij verwijdert alleen die zelfgemaakte map. De JSON-uitvoer bevat resultaten, geen documentinhoud, sleutel of capability. De tests gebruiken eveneens uitsluitend zelfgemaakte synthetische mappen. Ze bevatten drie echte proceskills na een bevestigde commit, revoke en eviction.

Gecontroleerd op Node.js **24.22.1**, Linux x86_64. `--test-isolation=none` laat in deze ontwikkelomgeving de individuele `node:test`-resultaten zien; zonder die optie presenteert de host de testbestanden als groepen. De killproeven starten afzonderlijke childprocessen, ook met deze testoptie.

## Modules en interfaces

| Module | Interface en grens |
|---|---|
| `crypto.mjs` | `createDocumentCipher(namespace)` maakt een nieuwe client-only 256-bits sleutel. `encrypt({objectId, revision, name, content})` retourneert een `Buffer` met ciphertext-envelope. `decrypt({objectId, revision}, bytes)` verifieert de volledige binding en tag vóór plaintext wordt teruggegeven. |
| `crypto.mjs` | `exportRecoveryKey()` is een expliciete lokale export naar de aanroeper. `restoreDocumentReader(namespace, key)` herstelt uitsluitend lezen; het resultaat kan niet opnieuw versleutelen onder een geresette noncegeschiedenis. De NAS krijgt deze interfaces of exports niet. |
| `store.mjs` | `NasStore.createSynthetic()` retourneert `{store, adminCapability}`; `openSynthetic({directory})` opent alleen een gemarkeerde synthetische store. `close()` beëindigt alle tijdelijke handles. |
| `store.mjs` | Beheer: `createInvitation(admin, {quotaBytes, lifetimeMs})`, `inspectUsage(admin)`, `revokeNamespace(admin, namespace)`, `evictNamespace(admin, namespace)` en `deleteObject(admin, namespace, objectId)`. Geen beheermethode ontsleutelt data. |
| `store.mjs` | Gebruiker: `acceptInvitation({namespace, invitation, confirmedPeerIdentity})`, `openSession({namespace, accessCapability})`, `stageUpload(session, envelope)`, `commitUpload(session, upload)`, `cancelUpload`, `readObject`, `listObjects`, `closeSession`. Leesresultaten zijn ciphertext, geen documentnamen of inhoud. |
| `policy.mjs` | `createPolicyEngine()` retourneert `authority`, `check`, `planNasRoute` en `authorizeNasConnection`. De preflight controleert toestemming vóór socketaanmaak en verleent geen transportvertrouwen. Alleen de vertrouwde adapter houdt `authority`; apps krijgen hun eigen opaque handle/beslisinterface. Geen kernelafdwinging. |
| `transport.mjs` | Loopback TLS met hybride sleuteluitwisseling, mTLS/pins en levende policycontrole; de precieze interfaces en bewijsgrenzen staan in TRANSPORT.md. |
| `policy.mjs` | `planNasRoute({app, peerIdentity, transports})` kiest automatisch een beschikbare toegestane LAN-route, anders direct internet naar exact dezelfde peer onder bestaande toestemming. Resultaat: `{status: 'ready', route: 'lan' \| 'internet', transport, reason}`, of `{status: 'paused', route: null, transport: null, reason}`. |

De NAS-opslag kent geen gebruikersnaam of bestandsnaam. Namespace-ID's, object-ID's, versienummers, lengtes, status en quota blijven zichtbaar als operationele metadata. Een inhoudsnaam en de inhoud zelf zitten samen in het versleutelde JSON-payload. Lengtes en toegangsritme worden niet verborgen; traffic padding is niet geïmplementeerd.

## Cryptografie en sleutelgrens

AES-256-GCM komt uit Node's ingebouwde `node:crypto`/OpenSSL; er zijn geen externe packages of eigen cryptoprimitives toegevoegd. Iedere nieuwe clientcipher krijgt 32 bytes uit `randomBytes`. Iedere encryptie krijgt een 12-byte willekeurige nonce. Een ledger in de cipher voorkomt hergebruik binnen de volledige levensduur van die sleutel en begrenst die tot 8.192 encrypties. Er is geen API die een bestaande sleutel opnieuw als schrijfbare cipher importeert. De NAS weigert hergebruikte nonces eveneens, inclusief gereserveerde uploads en na verwijdering van één object.

De geauthenticeerde context is exact de UTF-8-JSON-array `['nieuw-os/nas-object', 1, 'aes-256-gcm', namespace, objectId, revision]`. Een gewijzigde namespace, object-ID, revisie, nonce, ciphertext of 16-byte tag wordt bij decryptie geweigerd. De client moet de verwachte object-ID en revisie zelf aanleveren. De NAS kan uitsluitend het envelopeformaat controleren; zonder gebruikerssleutel kan hij niet vaststellen of een door een kwaadaardige client aangeleverde tag authentiek is.

Alleen de clientmodule houdt de decryptiesleutel. De store bewaart SHA-256-digests van willekeurige 256-bits beheer-/toegangscapabilities; ruwe capabilities staan niet in het disk-snapshot. Beheerder en gebruikerssessie zijn verschillende bevoegdheden. Het demo-/testproces bevat beide rollen om de interfaces te testen; dit bewijst geen procesisolatie tegen een debugger, roottoegang of een overgenomen client. De productie-inrichting moet de clientcipher op het gebruikersapparaat houden, gescheiden van de NAS-host.

**Nog open:** beveiligde lokale sleutelbewaring, offline herstel-UX, multi-device sleuteloverdracht, sleutelrotatie en een blijvende clientledger voor verwachte revisies. De export is ruwe geheime sleuteldata: de aanroeper moet die zelf op het eigen apparaat veilig bewaren, nooit bij de NAS, in een log of in Git. Zonder export verdwijnt de in-memory sleutel na procesverlies. Read-only recovery kan bestaande data ontsleutelen; nieuwe schrijftoegang na clientverlies vereist in N01 een expliciete nieuwe namespace met nieuwe sleutel. Dat is geen afgeronde sync-/back-upervaring.

## Intrekken, wissen en herstarten

`revokeNamespace` maakt de namespace permanent ontoegankelijk voor de gebruiker; aanwezige ciphertext blijft bewaard totdat de beheerder die verwijdert. `evictNamespace` verwijdert alle door deze store beheerde objectkopieën in die namespace en bewaart uitsluitend een tombstone met status/epoch/quota. Beide operaties vereisen alleen de beheercapability. Een nieuwe uitnodiging krijgt een nieuwe willekeurige namespace. Er bestaat geen automatische retry, herstel van oude rechten of hergebruik van een verwijderde namespace.

Iedere revoke, eviction en afzonderlijke admin-delete verhoogt de epoch. Bestaande sessies en in-flight uploads worden ingetrokken. Een oude upload kan dus na intrekking niet alsnog committen. Een afzonderlijke object-delete behoudt revisie- en noncegeschiedenis; een gebruiker met nog actieve namespace kan alleen door expliciet een nieuwe sessie te openen een volgende revisie uploaden. Voor permanent beëindigen van verdere uploads gebruikt de beheerder revoke/evict.

De store schrijft één begrensd snapshot: exclusieve tempcreatie, schrijven, bestand-`fsync`, rename en directory-`fsync`, vóór hij succes meldt. Bij een opslagfout wordt de handle onbruikbaar totdat hij opnieuw wordt geopend. Heropening valideert de bestaande toestand en maakt nooit stil een lege autoriteit aan. Sessies en gestagede uploads staan alleen in RAM; na restart zijn ze ongeldig. Gecommitteerde ciphertext, consumptie van uitnodigingen, digests, quota, revisies, nonces en tombstones blijven op disk.

Een exclusieve writerlock verhindert twee gelijktijdige servicehandles. Een aangetroffen dode lokale PID wordt alleen onder een tweede exclusieve `recovery.lock` hersteld, na opnieuw lezen van de writerlock. Die recoverylock wordt zelf nooit automatisch gestolen. Een crash precies tijdens lockherstel kan daarom handmatige offline controle nodig maken: stop/controleer alle betrokken lokale serviceprocessen en verwijder pas daarna een achtergebleven `recovery.lock` in de bevestigde eigen synthetische testmap. PID-hergebruik geeft veilig een blokkade. Gedeelde netwerkfilesystems, meerdere hosts en beheerders die lockbestanden handmatig wijzigen zijn buiten dit model.

Onder writerlock verwijdert de store eigen achtergebleven `.state-<32 hex>.tmp`-snapshots vóór nieuwe mutaties en vóór het serveren na heropening. Een cleanupfout blokkeert de operatie; eviction meldt dan geen succes. Andere bestanden worden niet opgeschoond. Dit is logische verwijdering van actieve servicekopieën en eigen tijdelijke snapshots. Forensische secure erase, SSD-remanentie, externe snapshots/back-ups en kopieën die een NAS-eigenaar zelf buiten de service maakte zijn niet aantoonbaar gewist.

De uitgevoerde proceskills testen heropening na een bevestigde operatie. **Geen fysieke stroomuitvaltest**, geen fault injection op iedere schrijf-/rename-instructie en geen garantie tegen een kwaadwillige NAS-eigenaar die een oud volledig snapshot terugzet. Zo'n eigenaar kan opslag weigeren, verwijderen of oude ciphertext aanbieden; de inhoud blijft zonder clientkey onleesbaar. Bescherming tegen rollback vereist later een vertrouwde clientcheckpoint-/revisiestrategie. De lees-API verwerpt een oud object wanneer de aanroeper een nieuwere verwachte revisie heeft.

## Internet- en appbeleid

- Installeren geeft geen internet-, LAN-, document-, NAS- of beheerrechten.
- Systeem-AI kan geen netwerkrecht krijgen; internet/LAN/NAS worden altijd geweigerd.
- Een browser kan expliciet internet krijgen zonder algemene document- of NAS-rechten. Een afzonderlijke documenttoestemming geldt voor precies één object en lees-/schrijfmodus.
- Een NAS-dienst vereist een expliciet gekoppelde peer-identiteit en een gekozen lokaal netwerk. LAN is de standaard toegestane route na koppeling. Direct internet wordt pas toegestaan na een afzonderlijke toestemming voor exact die peer.
- Na één keer koppelen en toestemming voor direct internet kiest `planNasRoute` bij iedere netwerkovergang automatisch de beste toegestane beschikbare route: LAN, anders direct internet naar dezelfde NAS. Terug op het toegestane LAN krijgt die route opnieuw voorrang. De planner vraagt geen herhaalde toestemming en verleent zelf geen nieuwe rechten.
- Vendorrelay en redirects worden geweigerd, ook met internettoestemming. Ontbrekende of ingetrokken peerfeiten leveren geen fallback op.
- Model-/updatedownloaders krijgen geen persoonlijke document- of NAS-capability via deze beleidsinterface. Begrenzing tot specifieke downloadservers en pakket-/modelverificatie zijn nog niet geïmplementeerd.

De vertrouwde adapter levert maximaal 64 actuele beschikbare transportfacts aan. De planner maakt geen verbinding en test zelf geen bereikbaarheid; een verdwenen verbinding moet uit de kandidaten worden verwijderd of met `forgetTransport` ongeldig worden gemaakt. Zonder bruikbare route retourneert hij expliciet `status: 'paused'`, zodat de interface bijvoorbeeld “Synchronisatie gepauzeerd” kan tonen terwijl lokale taken doorgaan. Er is geen uitwijking naar een andere NAS, relay of ongeverifieerde verbinding. Intrekking van internet- of peerrechten werkt meteen bij de volgende beslissing, ook wanneer oude transportfacts nog bestaan. Een eerdere `ready`-uitkomst is geen blijvende capability: de adapter moet bij gebruik actuele rechten controleren en later ook echte actieve sessies bij intrekking beëindigen.

De beleidsrechten leven in dit hostprototype in geheugen. Een productieversie moet eenmalige keuzes veilig op het apparaat bewaren, na restart herstellen en nieuwe transportidentiteit opnieuw controleren; daarvoor mag geen reeks nieuwe toestemmingsvragen nodig zijn. Ook de beperkte noncebudgetten en read-only sleutelrecovery hierboven zijn tijdelijke proefgrenzen. Betrouwbare achtergrondwerking, bruikbaar sleutelherstel en normale dagelijkse toegang zijn producteisen, geen aan de gebruiker op te leggen permanente handmatige omwegen.

`authority.attestVerifiedTransport(...)` is het vertrouwde adapterkoppelvlak. Het geeft een opaque handle terug die niet uit JSON is na te maken; een clientobject met `verified: true` wordt geweigerd. **Deze functie voert zelf geen TLS, certificaatcontrole of identiteitsbewijs uit.** N01-policytests leveren hier een expliciete aanname aan; de N02-loopbackadapter doet dit pas na echte TLS-controle. De `peerIdentity` in de synthetische store blijft een willekeurig fixture-ID en is nog niet aan deze transportpin gekoppeld. Native interface-, route-, DNS- en socketvalidatie en integratie tussen store en transport blijven open.

## Begrensde invoer

| Grens | N01-waarde |
|---|---|
| Documentnaam / inhoud | maximaal 512 / 16.384 UTF-8-bytes |
| Envelope | maximaal 140.000 bytes, strikt schema |
| Namespace / object-ID | willekeurige opaque 256-bits namespace / 128-bits hex object-ID |
| Namespacequota | 256 t/m 2.000.000 bytes; standaard 1.000.000 |
| Namespaces | 16 inclusief blijvende tombstones |
| Object-ID's | 32 per namespace, ook na afzonderlijke deletes |
| In-flight uploads | 8 per namespace, meegeteld in quota en noncebudget |
| Sessies | 16 per namespace / 128 totaal |
| Uitnodiging | eenmalig; standaard 5 minuten, maximaal 24 uur |
| Nonces per cipher/namespace | 8.192, daarna expliciete rotatie nodig |
| Volledig snapshot | maximaal 48.000.000 bytes |

## Bewijs en resterende integratie

De testset controleert encryptie/decryptie, tag-/metadatawijzigingen, verkeerde sleutels, read-only recovery, geheimen buiten NAS-diskdata, tenantgrenzen, beheerdersrechten, uitnodigingen, quota, conflictsituaties, noncegrenzen, stale uploads, restart, corrupted/symlinked snapshots, orphan-cleanup en het internetbeleid. De routeproeven omvatten LAN→internet→LAN met één toestemming, pauze/hervatten, intrekking terwijl oude facts bestaan en afwijzing van verkeerde peers, relays, redirects en vervalste facts. Testcommando's, actuele telling en versies worden door de hoofdagent in het projectbewijs vastgelegd. N01-unit-/policytests bewijzen geen TLS. De afzonderlijke N02-handshakeproeven leveren echt loopback-TLS-bewijs; zij zijn geen QEMU-, echte-NAS- of productiebeveiligingsbewijs.

Open: native OS-proces-/capabilityintegratie, verbinding met een echte thuis-NAS, geauthenticeerde pairing tussen apparaten, veilige key lifecycle, crash-/powerlossdekking, rollbackbescherming, background sync/conflictafhandeling, productie-transportquota en onafhankelijke menselijke securityaudit. N02 levert alleen de begrensde echte loopbackverbinding. M1–M6/kernelbestanden worden door deze modules niet aangepast.

## Primaire implementatiereferenties

Node.js. (2026). *Crypto: Node.js v24.22.1 documentation*. https://nodejs.org/docs/latest-v24.x/api/crypto.html

Node.js. (2026). *File system: Node.js v24.22.1 documentation*. https://nodejs.org/docs/latest-v24.x/api/fs.html

Geraadpleegd 27 oktober 2026. Deze referenties beschrijven de standaard-API's; het envelopeformaat, bevoegdhedenmodel en bovenstaande grenzen zijn eigen prototypekeuzes, geen door Node.js gecertificeerd beveiligingsprotocol.
