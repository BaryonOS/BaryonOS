# B02 — geauthenticeerde records en exacte sleutelstatus

Datum: 29 oktober 2026. Taak FR-B02, voortbouwend op B01/ADR-043 en
`services/volume`. Status: **core geïmplementeerd, hosttests en no_std-build
uitgevoerd; native opslagintegratie en volledige B02-acceptatie blijven open**.
Er is geen nieuwe kernelcryptografie of afhankelijkheid. De bestaande
KEY1-opcodes/ABI blijven behouden; FR-B02N voegt compatibele recordopcodes toe.

## Doel en beveiligingsgrens

De bestaande B01-SEAL bindt alleen domein en sleutelindex en is geen
opslagrecordformaat. Het bestaande herstelblob kent geen exacte
allocatieteller: legacy RESTORE maakt alle zestien indices gereserveerd.
De nieuwe bewerkingen staan daarom in dezelfde `keyring_core::Service`,
met dezelfde onderhouden XChaCha20-Poly1305/HKDF-SHA-256-implementaties.
De bestaande B01-bewerkingen en hun tests blijven bruikbaar.

De nieuwe core bevat geen schijf-I/O, geen broker die rechten toekent en geen
trusted cold-bootklok. De dienstadapter moet domein, objecttoegang en
verwachte revisie uit geautoriseerde systeemtoestand halen. Een app of
opslagserver mag die autoriteit niet verkrijgen door contextvelden in te
vullen. Gewone apps krijgen geen masterkey of persistente snapshotbevoegdheid.

Invarianten:

- Records authenticeren volume, aanvragerdomein, sleutelindex, object,
  revisie, chunkpositie, exacte lengte, recordsoort, encodingschema, suite en
  formaatversie. Gelijklange naam-/metadata-/inhoudrecords zijn niet uitwisselbaar.
- Een fout levert geen gedeeltelijke plaintext op. Uitvoerbuffers en de
  sleutelnamespace veranderen niet bij weigering. Entropiebewaking mag de
  dienst bij bronfalen bewust in `Degraded` zetten; diagnostische tellers
  blijven tellen volgens B01.
- Exacte allocaties en intrekkingen worden samen met de master versleuteld.
  Import valideert het gehele record vóór de namespacecommit. Onder dezelfde
  master worden allocaties met maximum en intrekkingen met bitgewijze unie
  samengevoegd; een eerder uitgegeven index wordt niet opnieuw uitgegeven.
- Een gebruikte of aan een volume gebonden instantie wisselt niet naar een
  andere master. Ook een leeg geldig persistent snapshot bindt de lineage;
  legacy RESTORE biedt geen omweg.
- Generatie-overflow geeft `Quota` vóór een gedeeltelijke namespacewijziging.
  Tijdelijke sleutel-/plaintextbuffers gebruiken `Zeroizing`; dit is
  geheugenhygiëne en geen claim over alle compiler-/registerkopieën.

## Core-interfaces en formaten

`records::RecordContext` bevat `volume: [u8; 16]`, `domain: u8`,
`object: u64`, `revision: u64`, `chunk: u32`, `length: u16`,
`kind: RecordKind` en `schema: u16` (nul ongeldig).
`Service::seal_record` en `unseal_record` ontvangen daarnaast het
kernelgebonden domein, de sleutelindex en de bestaande entropybron.
De domeinen zijn uitsluitend `0..DOMAINS`; context en aanvrager moeten
hetzelfde domein hebben. Object-ID nul en lege inhoud zijn geldige waarden.

Een record bevat `nonce[24] | ciphertext[length] | tag[16]`, maximaal
168 bytes voor de bestaande 128-bytechunk. De totale documentlengte wordt
hierdoor niet tot 128 bytes beperkt: de native adapter moet streamen.
De verwachte context staat niet als leesbare documentmetadata in het record.
Namen, tags, collecties, gevoelige indexen en manifesten moeten zelf in
geauthenticeerd versleutelde records worden opgeslagen.

De exact 64-byte AAD is canoniek little-endian:

| Offset | Inhoud |
|---|---|
| 0 | `NOSREC01` (8 bytes) |
| 8, 10 | Formaatversie 1, suite 1 (beide u16) |
| 12, 13 | Domein, sleutelindex (beide u8) |
| 14 | Recordsoort u8: 1 inhoud, 2 naam, 3 objectmetadata, 4 volumemanifest, 5 zoekindex |
| 15 | Gereserveerde nulbyte |
| 16 | Volume-ID (16 bytes) |
| 32, 40 | Object-ID en revisie (beide u64) |
| 48 | Chunkpositie (u32) |
| 52 | Plaintextlengte (u16) |
| 54 | Canonieke encodingversie binnen de recordsoort (u16, niet nul) |
| 56 | Acht gereserveerde nulbytes |

Suite 1 gebruikt de bestaande afgeleide B01-domein-/indexsleutel en
XChaCha20-Poly1305 met een verse DRBG-nonce. De afzonderlijke AAD-purpose
voorkomt uitwisseling met legacy SEAL. Lengte moet exact passen; truncatie,
extra bytes en gewijzigde verwachte lengte worden geweigerd.

`Service::export_persistent(volume, secret, source, out)` ontvangt een
extern gehouden willekeurig 32-bytegeheim; het retourneert `SnapshotInfo`.
Het bewaart geen kopie van dit geheim. Dit is geen wachtwoord-KDF.
`restore_persistent(volume, secret, blob, minimum_generation, source)`
retourneert dezelfde informatie na authenticatie, schema- en lineagecontrole.
`snapshot_info()` en `key_generation()` bevatten geen sleutelbytes; hun
beheer-/metadata-uitvoer hoort eveneens achter passende capabilities.

Het persistente blob is exact 120 bytes:

| Onderdeel | Formaat |
|---|---|
| Header, 32 bytes | `NOSKEY01`, versie u16=1, suite u16=1, volume-ID[16], nul[4] |
| Nonce, 24 bytes | Verse DRBG-uitvoer |
| Ciphertext, 48 bytes | master[32], generatie u64, per domein created u16 + revoked u16 |
| Tag, 16 bytes | AEAD-tag over ciphertext en de gehele header |

De wrapsleutel komt uit HKDF-SHA-256 met salt
`nieuw-os persistent key-state v1`, het gebruikersgeheim als IKM en info
`wrap`. Geen zelfgemaakte cryptoprimitief. Alleen formaat, suite en
opaque volume-ID staan buiten de encryptie; geobserveerde afmetingen en
toegangspatronen zijn geen verborgen metadata.

GENERATE en REVOKE verhogen de generatie bij succes. Legacy RESTORE
verhoogt haar eveneens. Een persistent import behoudt een voldoende nieuwe
generatie of verhoogt het maximum als een samengevoegde namespace nog niet
door die generatie wordt beschreven. Herhaald importeren van dezelfde
actuele toestand verandert de generatie niet.

## Native integratiecontract en open acceptatie

1. Start de bestaande Key Service met de echte entropybroker. Een aparte
   gebruikers-/herstelagent levert het externe geheim onder een expliciete
   beheerbevoegdheid. Geen ingebouwd fixturegeheim in het productpad.
2. De bestaande native Key Service-adapter dispatcht de nieuwe bewerkingen
   8/9 via `wire::dispatch_record`. De eerdere KEY1-codes, lengtes en
   foutbetekenissen blijven intact. Hosttests voeren dezelfde dispatchfunctie
   uit; native uitvoering via de geïntegreerde app blijft nog te bewijzen.
3. Haal domein uit het ontvangende capability-endpoint en controleer bij iedere
   recordbewerking het object, de actie en de actuele grant. Haal context en
   revisie uit het geauthenticeerde volume, niet uit onbetrouwbare ciphertext.
   Bind de recordsoort en het encodingschema aan de toegestane bewerking:
   naamslezen verleent geen recht op een content-, manifest- of indexrecord.
4. Sla keyallocaties/intrekkingen met de crashconsistente volumetransactie op
   vóór een bevestigde handle of bevestigde revocation. Geef geen ciphertext-
   of documentcommit vrij onder een nog niet duurzaam vastgelegde allocatie.
   Bij onzekere I/O geen valse bevestiging of automatische herhaling; behoud
   de bestaande `Unknown`- en herstelgrens. Deze adapter is nog te bouwen.
5. Bewijs met echte VM-writes, uitschakelen en een nieuwe VM dat inhoud,
   namen, exacte allocaties en intrekkingen behouden blijven. Lees de echte
   images onafhankelijk uit en test bitflips, swaps, volle opslag, verkeerde
   sleutel/domein, onderbreking en sleutelverlies. Dit native bewijs is open.

`minimum_generation` kan een oud geldig blob weigeren, **alleen wanneer die
ondergrens onafhankelijk vertrouwd is**. Een tweede bestand naast het blob,
een UI-succesmelding of een herstelde VM-snapshot levert die garantie niet.
Een koude instantie met ondergrens nul kan een oude geldige snapshot niet
van de actuele onderscheiden. Evenzo accepteert recorddecryptie replay als
ook de oude verwachte manifestcontext wordt teruggezet. Volledige koude
rollbackankering, persistente nonce-/crashanalyse en multi-writer-keylifecycle
blijven zichtbaar open voor B02; hiervoor wordt geen criterium afgetekend.

De bestaande twee domeinen/zestien indices zijn B01-proefgrenzen. Dit werk
verklaart die grenzen niet geschikt voor de volledige productrelease;
schaalbare sleutel-/objectintegratie blijft een releasevereiste.

## Uitgevoerde controles

Op branch `codex/full-release-storage`, basis `73ebc1e`, met de bestaande
Rust 1.90.0-toolomgeving. De controles gebruiken uitsluitend synthetische
testdata, geen echte schijf, NAS, gebruikersgeheimen of productiekeys.

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-fr-storage-target python3 -B tools/dev.py cargo +1.90.0 test --locked -p keyring-core
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-fr-storage-target python3 -B tools/dev.py cargo +1.90.0 clippy --locked -p keyring-core --all-targets -- -D warnings
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py cargo +1.90.0 fmt -p keyring-core -- --check
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-fr-storage-target RUSTFLAGS='--cfg chacha20_force_soft --cfg poly1305_force_soft --cfg sha2_backend="soft"' python3 -B tools/dev.py cargo +1.90.0 build --locked -p keyring-core --release --target x86_64-unknown-none
```

De hosttests voeren de echte onderhouden AEAD/HKDF-code uit, inclusief
alle ondersteunde recordlengtes, alle bitflips van volledige records en
snapshots, truncatie, contextwissels, generatiegrens, exacte allocatie,
intrekkingsunie en een expliciet aangetoond onbeschermd koud replaygeval.
De bestaande B01-tests blijven de standaardtestvectoren en eerdere
herstel-/entropiegevallen controleren. Host-/no_std-bewijs vervangt geen
native dienst-/schijfuitvoering of onafhankelijke beveiligingsaudit.

Uitkomst op de uiteindelijke bron: **15 bestaande + 23 nieuwe hosttests
geslaagd**, Clippy met `-D warnings`, formatteringscontrole, diffcontrole en
de portable `x86_64-unknown-none`-releasebuild alle exit 0. De nieuwe
recordsoort-/schemabinding kwam uit afzonderlijke agentreview: vóór integratie
is de concrete uitwisselbaarheid van gelijklange naam-/inhoudrecords opgelost.
Geen VM-run, werkelijk opgeslagen versleuteld volume of volledige koude
antirollback is door deze deelbatch geclaimd.

## FR-B02N — aangesloten KEY1-recordbewerkingen

`SEAL_RECORD = 8` en `UNSEAL_RECORD = 9` gebruiken dezelfde KEY1-tag,
256-bytepayload, 16-byteheader en normale foutantwoorden. De body is een
canonieke context van exact 64 bytes gevolgd door plaintext respectievelijk
`nonce | ciphertext | tag`. SEAL-body: 64..192 bytes; UNSEAL-body:
104..232 bytes. Beide passen binnen de bestaande 240-bytebody.

De contextprefix heeft deze vorm, onderscheiden van de interne record-AAD:

| Offset | Inhoud |
|---|---|
| 0 | `NOSCTX01` (8 bytes) |
| 8, 10 | Recordformaatversie 1 en suite 1 (u16) |
| 12 | Recordsoort u8, uitsluitend de vijf genoemde waarden |
| 13 | Drie gereserveerde nulbytes |
| 16 | Volume-ID[16] |
| 32, 40 | Object-ID en revisie (u64) |
| 48 | Chunkpositie (u32) |
| 52, 54 | Plaintextlengte en niet-nul encodingschema (u16) |
| 56 | Acht gereserveerde nulbytes |

Er staat **geen domein in de wirecontext**. De native adapter bepaalt dat
uitsluitend uit het ontvangende endpoint. `decode_record_context` vult
`RecordContext.domain` daarmee in. Zo wordt ciphertext onder domein A bij
ontvangst op domein B met een andere AAD/sleutel beoordeeld en geweigerd.
De wirecontext is geen objectgrant; de eerder beschreven object-/actiecontrole
blijft de verantwoordelijkheid van de native toegangsadapter.

Gebruik voor een native aanroeper:

```rust
let payload = keyring_core::wire::record_request(
    keyring_core::wire::SEAL_RECORD, key_index, request_id, context, plaintext,
)?;
// Verzend payload onder KEY1 via het reeds verleende endpoint.
// Valideer afzender, tag, nul ontvangen capabilities, headerop|REPLY,
// key_index, request_id, status, exacte lengte en nul padding van het antwoord.
```

UNSEAL gebruikt dezelfde builder met opcode 9 en het ontvangen sealed record.
Een SEAL-antwoord bevat maximaal 168 bodybytes; een UNSEAL-antwoord maximaal
128 bytes. Iedere fout heeft lengte nul en nulgevulde body. De dispatcher
controleert exacte context-/datalengte, versie, suite, soorten, schema,
reserveringen, requeststatus en ingressdomein vóór cryptografische verwerking.
Er zijn geen persistente snapshotopcodes of nieuwe herstelbevoegdheden voor
gewone apps toegevoegd.

Uitgevoerd op de uiteindelijke FR-B02N-bron: **46 hosttests (15 legacy,
23 secure-record en 8 wire/dispatch)**, Clippy met `-D warnings`, formattering
en de echte native service-/appbuild voor `x86_64-unknown-none`, alle exit 0.
De bestaande `sdk/examples/keyring/build.py` gebruikt expliciet portable
crypto-backends en controleert via het linkerscript de native ELF-budgetten.
Command voor die build:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-fr-storage-target/native-service python3 -B sdk/examples/keyring/build.py
```

Dit bouwbewijs voert geen VM uit. De hoofdintegrator moet de nieuwe opcodes
native aanroepen en cross-domain-/type-/schema-/manipulatiefouten observeren;
de persistente opslag-/rollbackgates blijven open.
