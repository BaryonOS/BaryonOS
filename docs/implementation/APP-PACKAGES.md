# Apppakketformaat — verificatiekern (A02-deelstap)

Status: geïmplementeerde `no_std`-verificatiekern en aansluiting op het
bestaande user-ELF-beleid; geen installer, catalogus, dynamische
appimage-laadketen of native acceptatie.

`app-package-core` verwerkt één pakket als een vaste manifestheader van 3.584
bytes gevolgd door precies één application image. Het manifest is versieerbaar,
little-endian en gebruikt `NOSAPP01` als domeinlabel. `verify_executable_package`
verifieert eerst het pakket en geeft de image daarna door aan
`kernel_core::user::parse`, hetzelfde bestaande ELF-beleid dat de kernel vóór
procesbouw gebruikt. De geretourneerde vaste laadbeschrijving bevat geen
gealloceerde frames of proces-/launchcapability en verleent op zichzelf geen
startrecht. Beide bestaande
standaardverificaties uit ADR-049 zijn vereist: Ed25519 en ML-DSA-65 over
dezelfde canonieke metadata. De caller kiest de uitgeversankers uit de
expliciet gekozen bron; de bundel kan zelf geen vertrouwde sleutel aanwijzen.
De verificatie bindt de exacte image-lengte en SHA-256 aan het manifest.

Primaire standaarden: Ed25519 is een EdDSA-variant uit NIST, *Digital
Signature Standard (DSS)*, FIPS 186-5, gepubliceerd op 2023-02-03,
https://csrc.nist.gov/pubs/fips/186-5/final. ML-DSA-65 is een parameterisatie
uit NIST, *Module-Lattice-Based Digital Signature Standard*, FIPS 204,
gepubliceerd op 2024-08-13,
https://csrc.nist.gov/pubs/fips/204/final. Beide handtekeningen zijn vereist
over dezelfde canonieke metadata; dit bewijst herkomst onder de gekozen
sleutel, niet dat de uitgever of app veilig is.

| Offset | Veld |
|---|---|
| 0 | `NOSAPP01` |
| 8 | formaatversie u32 = 1 |
| 12 | app-ID [64], ASCII, nul-opgevuld |
| 76 | minimale ABI u32 |
| 80 | maximale ABI u32 |
| 84 | appversie u64 |
| 92 | image-lengte u32 |
| 96 | image-SHA-256 [32] |
| 128 | Ed25519 key ID [8] |
| 136 | ML-DSA-65 key ID [8] |
| 144 | Ed25519-signature [64] |
| 208 | ML-DSA-65-signature [3309] |
| 3517 | nul-padding tot 3584 |

De signaturebytes dekken exact `manifest[0..144]`. ABI is gecodeerd als major
in de hoge 16 bits en minor in de lage 16 bits. De image begint op byte 3584
en het pakket bevat geen trailing padding. De decoder controleert framing en
ABI eerst, daarna de geselecteerde sleutel-ID's en imagehash, en voert pas
daarna de twee relatief dure handtekeningverificaties uit.

De metadata bevat alleen app-ID, ABI-interval, versienummer, image-lengte,
digest en sleutel-ID's. Er is bewust geen script, dependency-resolver,
bestandslocatie, capability-/rechtenverklaring of host-uitvoer. De package-ID
is een lowercase puntgescheiden identifier, geen autoriteit. Een geldige
handtekening bewijst alleen herkomst onder de geselecteerde sleutel en
integriteit; zij is geen verklaring dat een app veilig of betrouwbaar is en
verleent geen rechten.

## Meerdere appbronnen — policy-core

`SourceRegistry` houdt maximaal acht expliciet gekozen bronnen bij. Een
32-byte `SourceId` komt van de vertrouwde bronbeheerder en staat niet in het
pakket; de aanroeper kiest de bron voor iedere verificatie. Een bron kan
worden toegevoegd, op een verwachte generatie worden geroteerd of ingetrokken.
Rotatie en intrekking verhogen de generatie; stale mutaties falen zonder
toestandswijziging. Ingetrokken slots blijven tombstones en worden niet
stilzwijgend hergebruikt.

Intrekking blokkeert nieuwe packageverificatie en updates via die bron. Zoals
door Simon gekozen op 30 oktober 2026 blijven reeds geïnstalleerde apps
startbaar; data of apps worden niet stil verwijderd. Beheer van bronkeuze en
rotatie vereist afzonderlijke gebruikersautorisatie. Deze vaste array is
alleen een in-memory policy-core: geen consent-UI, capabilitycontrole,
duurzame bronregistratie of transacties. Acht bronnen is geen productlimiet.

ABI-versies zijn twee u16-waarden in één u32: major in de hoge 16 bits, minor
in de lage. De verifier ontvangt de daadwerkelijk onderhandelde ABI als
argument; hij raadt geen host- of SDK-versie. De image-limiet van 2 MiB is
uitsluitend deze eerste fixturegrens en geen vastgesteld productbudget.
De manifestdecoder weigert onbekende versies, nul-/te grote images,
oncanonieke padding, ongeldige IDs en ongeldige ABI-intervallen.

## Streaming artifact-store en app-managementcontract

De zelfstandige `sdk/app-management`-crate scheidt het immutable
read-handlecontract van de schrijftransactie. Beide zijn host-/targetcontracten;
er is nog geen persistent store-adapter of native installer.

`ArtifactRef` bevat `artifact_id[32]` en `immutable_generation:u64`.
`ArtifactDescriptor` combineert die ref met `length:u64` en `sha256[32]`.
Publishermetadata (`source_id`, bron-generatie, packageversie en
app-identiteit) blijft apart in package-provenance; storegeneratie is geen
publisher-epoch. Dit zijn Rust-records, geen vastgelegde serialized opslag-ABI.
De `u64`-capaciteitsmelding rapporteert totale capaciteit, gebruikte bytes,
gereserveerde bytes en quota; er is geen globale productcap in dit contract.
Een provider moet bij `begin` de volledige objectlengte atomair reserveren of
falen vóór hij chunks accepteert.

De leeszijde is sinds #109 het gedeelde `artifact-store-core`-contract
(ADR-054, een contractvoorstel; native uitvoering staat open);
app-management herdefinieert geen read-types meer. **Contracteis, nog niet
geïmplementeerd:** er is nog geen persistente storeadapter. Een toekomstige
adapter implementeert `ArtifactStore`, construeert het opaque handle-type
(niet serialiseerbaar of overdraagbaar), bindt de handle aan de
kernel-geauthenticeerde clientcontext en de actuele store-epoch, en
controleert beide bij iedere read; alleen de testadapter doet dat nu. `open_checked` weigert een
store-autoritatieve descriptor die afwijkt van de verwachte (`Corrupt`);
`read_bounded` gebruikt de descriptor uit de geopende handle, zodat een
aanroeper geen andere descriptor kan meegeven. Iedere opslagoperatie is
begrensd tot 4 KiB en fouten zijn expliciet: stale epoch, corruptie,
truncatie vóór de gedeclareerde lengte, overflow, EOF (`EndOfArtifact`) en een
offset buiten het object (`OutOfRange`). Dit is een storagelaaggrens; de
bestaande Message320-IPC blijft maximaal 256 payloadbytes. Het app-management-chunkrequest gebruikt maximaal
224 databytes per bericht.

De schrijftransactie bindt `caller`, `request_id` en `service_epoch` aan de
storedescriptor: immutable `ArtifactRef`, lengte en digest. `source_id`,
brongeneratie en packageversie staan apart in `PackageIdentity`; zij zijn niet
aan `InstallTransaction` gekoppeld en vormen geen store- of publisherautoriteit.
De pakketpolicy moet de herkomst en toelaatbaarheid afzonderlijk beoordelen;
een persistente issuer-/sleutellevenscyclus ontbreekt nog. Chunks zijn
niet-leeg, maximaal 4 KiB en strikt opeenvolgend op de verwachte offset.
Commit is alleen toegestaan na exact de gedeclareerde lengte en publiceert
atomair na een volledige lengte-/digestcontrole. Cancel is alleen geldig vóór
commit. `Unknown` blokkeert replay en automatisch opnieuw committen; een
uitkomst mag alleen binnen dezelfde service-epoch worden opgevraagd. Na een
herstart moeten oude requests stale worden en moet herstel de duurzame
catalogus raadplegen. De huidige implementatie is alleen een in-memory
transactiewachter met fake-storetests: geen duurzaam journaal of bewezen
cold recovery.

Een digest is uitsluitend content-integriteitsmetadata; hij bewijst geen
publisher of modelgeschiktheid. `SourceRegistry` blijft in-memory. De
beveiligings-/B02-/B03-eigenaren hebben geen bestaande issuer-/trust-root-
levenscyclus-API vrijgegeven, en B02- of B03-ankers zijn niet herbruikbaar als
appuitgeversbeleid. Er wordt dus geen `verified`-boolean, neppe publisher-epoch
of claim van persistente signertrust toegevoegd. Een geldige pakketverificatie
onder een tijdelijke, gekozen sleutel maakt op zichzelf nog geen persistent
installerrecht. De documentvolumeclient is geen autoritatieve appstore zolang
zijn huidige authenticatie-/rollbackgrens onvoldoende is; deze crate schrijft
er niet naar.

### Install-/update-/remove-catalogus (contractmodel)

`catalog::InstallCatalog<N>` is een begrensd in-memory toestandsmodel boven de
artifacttransactie. Iedere mutatie is tweefasig (`prepare` → `apply`/`abort`)
en gebonden aan service-epoch en verwachte cataloggeneratie; een stale,
herhaalde of gelijktijdige intent faalt zonder mutatie. Een private, monotoon
oplopende intentserial voorkomt dat een geannuleerde opdracht weer geldig
wordt wanneer dezelfde handeling opnieuw wordt voorbereid. Uitputting faalt
met `Overflow` voordat een pending opdracht ontstaat; abort, restart en
reconcile hergebruiken geen serials binnen dit catalogusobject. Een echte
dienst moet bij reconstructie een verse service-epoch gebruiken; het model
levert die epochautoriteit niet. Het opgeslagen
artifact moet in lengte en SHA-256 gelijk zijn aan de image uit de
package-identity, en `apply` vereist exact de door de store gecommitte
`ArtifactRef`. Update vereist dezelfde bron, geen lagere brongeneratie en een
strikt hogere versie; bronwissel of downgrade is geen update. Remove vereist
de exact geïnstalleerde entry. `released` meldt alleen een oud artifact dat
geen andere entry in deze catalogus meer gebruikt. Het is een opruimkandidaat,
geen wisrecht: duurzame catalogwijziging en de store-lifetimecontrole
(waaronder andere consumers en open read-handles) blijven vereist.
De catalogus verwijdert geen bytes en heeft geen gebruikersdata- of
capabilityhandle; een pakket verwijderen is geen verzoek gebruikersdocumenten
te wissen. Behoud van echte gebruikersdata en intrekking van oude taakrechten
na update/verwijderen moeten nog native worden aangesloten en getest.
Entries bevatten geen capabilities: installatie verleent niets.

Een `Unknown` storeuitkomst maakt de catalogus onbepaald: mutatie én lookup
falen gesloten, ook na herstart, tot `reconcile` een snapshot met hogere
generatie installeert. Die autoritatieve, duurzame bron bestaat nog niet; dit
model bewijst dus geen persistentie of koude recovery. `N` is een
compile-time modelgrens, geen productlimiet.

#### Gerichte uitvoering TASK-A02-CATALOG — 30 oktober 2026

Codebron `05e658e58eb69d29a2a800660616308c4cecf2e3`, branch
`codex/app-management-catalog-leaf`, basis `9d8a119` met eerste cataloguscommit
`c8c68ee`. SHA-256 van `sdk/app-management/src/catalog.rs`:
`1ecd0e45bdb7f8690c1784e75428acdcf3f767e4b669f5d709382c334f1e7582`.
De twee regressietests voor cancel/herprepare en gedeelde image-opruiming
faalden eerst op de ongewijzigde catalogus van `c8c68ee` (exit 101, vijf
bestaande catalogustests groen, twee nieuwe rood). Op de codebron hierboven
slagen alle 18 app-managementtests, waaronder acht catalogustests.

Lokale omgeving voor onderstaande vier commando's:

```sh
export CC="$PWD/.tools/nieuw-os/dev/cc-wrapper"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$CC"
export CARGO_TARGET_DIR=/tmp/nieuw-os-app-management-target
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/app-management/Cargo.toml
python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path sdk/app-management/Cargo.toml --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/app-management/Cargo.toml --all -- --check
CARGO_ENCODED_RUSTFLAGS=$'--cfg\x1fcurve25519_dalek_backend="serial"' \
  python3 -B tools/dev.py cargo +1.90.0 build --offline --locked --release --manifest-path sdk/app-management/Cargo.toml --target x86_64-unknown-none
```

Elk commando eindigde met exit 0. De volledige hosttests zijn uitgevoerd,
niet afgeleid uit een buildcache. Doorlooptijden in deze omgeving:
test <1 s, Clippy <1 s, fmt 1 s, targetbuild <1 s; dit zijn geen
runtimeprestatiemetingen. Lokale logs:
`/tmp/nieuw-os-app-catalog-05e658e-{test,clippy,fmt,none}.log`;
rode regressie: `/tmp/nieuw-os-app-catalog-regression-before.log`.

Niet uitgevoerd: native installatie/update/remove, echte I/O-/powercutproef,
gebruikersdatabehoud op disk, rechtenintrekking na update, cold recovery,
CI op deze bron, fysieke of menselijke acceptatie. De crate voegt geen
engine, dependency, persistentiebackend, publisherverifier of bevoegdheid toe.
A01 behoudt zijn eigen historische SDK-bewijs; A02/A03/A04 blijven open.
Dit catalogusresultaat is geen afronding van FR-A11/A12/A13 of van de overige
dagelijkse apps en compatibiliteitstaken uit FULL-TEST-RELEASE-CONTEXT.

De gedeelde read-only interface is ook de beoogde ingang voor consumers zoals
een lokale AI-modelreader. De consumer moet zelf formaat/parsergrenzen en
toepasselijke modelintegriteit beoordelen; de artifactdigest is geen
publishertrust of veiligheidsverklaring. Een dynamische launcher, one-use
`AuthorizedImage`-handle, kernel-copy/preflightpad en app-/catalogus-
persistency zijn niet geïmplementeerd. Het huidige launchbericht draagt een
package identity, maximaal twee door de policybroker gereduceerde capabilities
(standaard nul), en epochs; manifestrechten creëren geen capabilities.

## Test en bewijsgrens

```sh
cargo +1.90.0 test --offline --locked -p app-package-core
cargo +1.90.0 clippy --offline --locked -p app-package-core --all-targets -- -D warnings
CARGO_ENCODED_RUSTFLAGS=$'--cfg\x1fcurve25519_dalek_backend="serial"' \
  cargo +1.90.0 build --offline --locked -p app-package-core --target x86_64-unknown-none
cargo +1.90.0 test --offline --locked --manifest-path sdk/app-management/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/app-management/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/app-management/Cargo.toml --all -- --check
CARGO_ENCODED_RUSTFLAGS=$'--cfg\x1fcurve25519_dalek_backend="serial"' \
  cargo +1.90.0 build --offline --locked --release --manifest-path sdk/app-management/Cargo.toml --target x86_64-unknown-none
```

De eerste drie commando's testen alleen de bestaande package-core.
App-managementtests controleren bounded wire decode, zero-grant launch,
capability-/epochbinding, readgrenzen en transaction-state met fake backends;
ze bewijzen geen echte provider, install/update/remove, trustpersistency,
native uitvoering of koude recovery. De cross-build bewijst alleen dat de
library voor de freestanding target compileert.

Deze package-core-tests bewijzen alleen host-/bare-metal-libraryverificatie. Zij tonen niet
dat een app kan worden geïnstalleerd, op schijf atomair beheerd, na herstart
gevonden of via de kernel gestart. De user-ELF-parser gebruikt dezelfde
adres-/segment-/quota-policy als de huidige kernel loader. Tests tonen dat een
correct ondertekend maar W+X ELF-image niet launch-klaar wordt verklaard.
SourceRegistry-tests controleren onafhankelijke bronkeuze, generatiegebonden
keyrotatie, intrekking, dubbele sleutels en de vaste capaciteit; de registry
is niet duurzaam en kent geen rechten toe. De helper bouwt geen mappings,
proces of `LaunchImage`; A02 vereist daarnaast bronbeheer op disk,
installertransacties, update/verwijderbeleid, een dynamische image-laadketen
en echte native positieve/negatieve taken. De loaderservice moet bij gebruik
rekening houden met de circa 200 KiB ML-DSA-verificatiestack en een passende
begrensde stack hebben.
