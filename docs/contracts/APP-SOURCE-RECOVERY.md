# APP-SOURCE-RECOVERY-1 — duurzame uitgeversautoriteit en herstel

30 oktober 2026; TASK-SEC-PUBLISHER-RECOVERY-CONTRACT-1.
**Adaptercontract, geen geïmplementeerde issuer, backend, opslagformaat,
capability of koude-herstelacceptatie.** Basis
`ca45a7dabe6a040921de03bdc2fad3e5045a75ba`.
Doel: de bestaande appverifier koppelen aan geautoriseerde, duurzaam gebonden
brontoestand zonder een tweede verifier/catalogus of nieuwe crypto te bouwen.
Rubric B04 en gekoppelde app-/releasecriteria blijven open.

## 1. Bestaande interfaces en afzonderlijke bronversies

| Bouwsteen | Bron | Bruikbaar, maar niet geleverd |
|---|---|---|
| `app_package_core::SourceRegistry`, `SourceVerifiedExecutable` | #73 `dce3b05aa358f4dbde6f2349b426217779b40052`, `crates/app-package-core/src/lib.rs` | Hybride verificatie en levende add/rotate/revoke; geen actorautorisatie of persistentie |
| `PackageIdentity`, `catalog::InstallCatalog` | #80 `bc27ed0f20097a68c9bfdfeecc8867c7e7721211`, `sdk/app-management/src/{lib,catalog}.rs` | Provenancemetadata en install/update/remove-model; geen issuer of duurzaam journaal |
| `ArtifactStore`, `ArtifactDescriptor`, `open_checked` | #78 `7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb`, `crates/artifact-store-core/src/lib.rs` | Immutable bytegeneratie/readcontract; geen publishertrust of backend |
| `records::RecordContext`, `Service::{export_persistent,restore_persistent}` | `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`, `services/keyring/src/{records,persistent}.rs` | AEAD-context en sleutelstatusbouwstenen; geen onafhankelijk koud rollbackanker |

Dit zijn afzonderlijke leaves, geen gezamenlijk geïntegreerd releaseimage.
Apps bezit verifier/catalogus/protocol; release bezit store/backend,
authorized-image-/kernelcopy-/dynamische-launchintegratie. SEC bezit dit
bronherstelcontract. Wijzigingen aan gedeelde/frozen code vereisen hun
eigenaar; dit document wijzigt die code niet.

`SourceRegistry::add(id, anchors)` maakt generatie 1.
`rotate(id, expected_generation, anchors)` en
`revoke(id, expected_generation)` vereisen de actuele generatie, verhogen
die checked en bewaren ingetrokken slots als tombstones.
`verify_executable(id, expected_generation, package, current_abi)` weigert
onbekende, ingetrokken of stale bronnen en retourneert alleen na de bestaande
pakket-/ELF-verificatie `SourceVerifiedExecutable`.
Deze core bezit geen publieke export/import van zijn private bronlijst:
een durable adapter bestaat dus niet door alleen de huidige crate aan te roepen.
Eventuele gecontroleerde reconstructie-/snapshotinterfaces moeten later met
Apps worden ontworpen; private layout kopiëren of revocations overslaan mag niet.

## 2. Beveiligingsinvariant en bronbeheeractor

Een succesvolle nieuwe installatie/update mag uitsluitend voortkomen uit:
een geautoriseerde beheerbeslissing, actuele duurzaam bevestigde brontoestand,
de bestaande pakketverifier en een afzonderlijk geautoriseerde installactie.
Een pakket, catalogusdescriptor, downloadserver of storelocator kan geen
vertrouwde bron, keyset, epoch of grant uitgeven.

Een toekomstige bronbeheerdienst krijgt een afzonderlijke capability voor
add/rotate/revoke. Zij bindt iedere operatie aan de werkelijke kernelafzender,
gebruikers-/beleidsautoriteit, het bedoelde brondomein en de actuele
dienstinstantie. Een payload-PID, SourceId, `verified`-boolean, UI-tekst of
publieke constructor is geen actorbewijs. De concrete issuer-/bootstrap-
capability is **nog niet vastgesteld of geïmplementeerd**.

Installatie geeft geen bronbeheerrecht of algemene bestands-, NAS-, netwerk-,
sleutel- of launchbevoegdheid. Automatisch opnieuw controleren binnen een
bestaande grant is toegestaan; een nieuwe bron of andere trust authority
vereist een expliciete geautoriseerde beslissing. Geen regrant na Unknown.
Appuitgeversankers blijven gescheiden van B03-releaseankers, TLS-identiteit
en B02-gebruikerssleutels.

## 3. Duurzaam te binden toestand, zonder nieuwe byte-ABI

De toekomstige opslagtransactie bindt minimaal:

- het geautoriseerde bronbeheerdomein en de store-/volume-identiteit;
- SourceId en het exacte Ed25519/ML-DSA-65-paar;
- de checked brongeneratie en active/revoked-status, inclusief tombstones;
- de verwachte vorige toestand en de nieuwe geauthenticeerde commitidentiteit;
- voldoende geauthenticeerde herstelmetadata om een onzekere mutatie te
  onderscheiden van niet-gepubliceerd werk.

Dit is een inhoudelijk invariant, geen gekozen encoding, KDF, key-ID-schema,
journalingalgoritme of cryptografisch protocol. Gebruik bestaande
gecontroleerde cryptografie pas via een geautoriseerde adapter. Een checksum
of hash naast een bestand maakt het bestand niet authoritative.

| Teller/identiteit | Betekenis | Geen vervanger voor |
|---|---|---|
| SourceId | Bronidentiteit binnen geautoriseerd beheerdomein | Actorgrant of inhoudsdigest |
| Brongeneratie | Volgorde van keyrotatie/intrekking van die bron | Appversie of cold-bootanker |
| Appversie | Geauthenticeerde pakketversie | Bronvertrouwen of toegangsrecht |
| Immutable storegeneratie | Exacte opgeslagen bytes | Publisher-epoch of keylifecycle |
| Service-epoch | Scheidt levende instantie-/requesthandles | Duurzame revocationgeschiedenis |
| Duurzame commit/ankerbinding | Welke bronstaat aantoonbaar actueel is | Succesboolean of los rollbackbestand |

Geen wraparound, nulherstart van de geschiedenis of hergebruik van tombstones.
Uitputting weigert de operatie vóór publicatie; een nieuw domein is geen
stille omweg.

## 4. Mutatie, durable ACK en linearisatie

De volgende stappen zijn verplicht adaptergedrag, nog geen bestaande service:

1. Authenticeer actor en grant; valideer request, bron en verwachte generatie
   vóór opslag- of cryptowerk. Serialiseer conflicterende trustmutaties.
2. Bereken/valideer een kandidaat met dezelfde regels als `SourceRegistry`,
   zonder de actieve registry alvast zichtbaar te wijzigen. Een prepare op
   een private kandidaat is niet hetzelfde als een actieve commit.
3. Leg intent/kandidaat en herstelbinding vast volgens het nog te kiezen
   crashconsistente backend-/ankerprotocol. Geef geen duurzame ACK op basis
   van RAM, enqueue, een succesvolle write zonder vereiste flush of een
   caller-supplied commitbewijs.
4. Publiceer nieuwe bronstaat en pas daarna succes terug naar de actor als
   de concrete backend plus onafhankelijk rollbackanker hun vereiste
   commitgarantie hebben bevestigd. Het exacte linearisatiepunt moet door
   dat protocol worden bepaald; twee onafhankelijke writes zijn niet
   automatisch atomair.
5. Bij intrekking sluit de adapter nieuwe install-/updateacceptatie voor
   die bron vóór de mutatie-ingang als voltooid wordt gemeld. Bij
   post-publicatie-onzekerheid blijft die bron gesloten voor nieuwe acceptatie
   tot authoritative reconcile, ook als een oudere actieve snapshot bestaat.

Een nieuwe rotatie of intrekking moet reeds geverifieerde maar nog niet
gecommitte install-/updatewerkzaamheden opnieuw laten autoriseren. Alleen
een oude `SourceVerifiedExecutable` bewaren om de actuele brongeneratiecheck
te omzeilen is onvoldoende. Geen globaliserende stop van andere onafhankelijke
bronnen tenzij de backend aantoonbaar geen kleinere veilige grens kent.

## 5. Crashvensters en Unknown

| Venster / uitkomst | Toegestane conclusie | Vereiste vervolggrens |
|---|---|---|
| Vóór publicatie, aantoonbaar niets uitgegeven/opgeslagen | Geen trustmutatie bevestigd | Nieuwe expliciete aanvraag mogelijk; geen ACK fabriceren |
| Intent geschreven, kandidaat niet gecommitte | Niet voltooid | Herstel valideert volledige intent/commitketen; geen gedeeltelijke registry publiceren |
| Kandidaatdata duurzaam, onafhankelijk anker niet bevestigd | Niet aantoonbaar actueel | Unknown, bronacceptatie gesloten; niet vertrouwen op datahash alleen |
| Anker mogelijk gevorderd, data-/commituitkomst onzeker | Unknown | Niet terugvallen op oudere actieve staat; reconcile of herstelbare fout |
| Commit voltooid, antwoord verloren | Mogelijk al uitgevoerd | Uitkomst op authoritative commitidentiteit onderzoeken; geen automatische mutatiereplay |
| Revoke duurzaam, oude cache/installer nog actief | Oude acceptatie is ongeldig | Cache/installer herautoriseert; geen nieuwe installatie op oude generatie |
| Cold boot zonder onafhankelijk actualiteitsbewijs | Geen cold antirollback | Oude authentieke bronlijst is niet bewezen actueel; geen productieacceptatie |
| Corrupt, conflicterend, ontbrekend of verkeerd domein | Geen geldige staat | Gesloten fout met diagnose, geen lege Default-registry als succesfallback |

Unknown is geen geslaagde intrekking, installatie of flush en ook geen
toestemming om opnieuw uit te voeren. Een authenticated snapshot mag geen
lagere brongeneratie, verloren tombstone of teruggezet keyset autoriseren.
Een service-epoch na reboot is geen bewijs dat de opgeslagen lijst actueel is.

## 6. Cold recovery en ankerafhankelijkheid

De herstelagent authenticeert volledige bronstaat, beheerdomein en storebinding,
controleert ze tegen onafhankelijk vertrouwde actualiteit en publiceert pas
daarna een complete registry. Reconcile mag alleen uit die autoriteit komen,
niet uit een appmetadatarecord, een opgeslagen `PackageIdentity`, een vorige
succesmelding of een snapshot die dezelfde aanvaller kan terugzetten.

Een TPM-NV-counterfixture op een andere bron bewijst niet de benodigde
bronstaat-/lineagebinding. B02 `minimum_generation=0` accepteert authenticiteit
zonder koude actualiteitsgarantie. B02-restore, firmware-releasefloor en
artifactgeneratie mogen niet als nieuwe apppublisherankers worden hergebruikt.
Echte ankerprovider, duurzame backend, write-ahead-/commitcoördinatie en
reconstructie-API zijn **OPEN**. Zonder die dependencies bestaat geen
positieve productie-cold-recoveryclaim.

## 7. Installerconsume en reeds geïnstalleerde apps

De installer bindt authorisatie aan exact SourceId, actuele brongeneratie,
app-ID, geauthenticeerde pakketversie, ABI, image-lengte en digest. Hij gebruikt
de bestaande `SourceRegistry::verify_executable`, niet een tweede verifier of
een publiek zelf te construeren verified-token.

`PackageIdentity` is metadata; de immutable artifactdescriptor bewijst alleen
bytebinding na daadwerkelijke volledige verificatie. Geen van beide verleent
een install-, bronbeheer- of launchcapability. Voor commit opnieuw actuele
bronstaat en installgrant controleren; revoke/rotate na verificatie geeft geen
stille uitzondering. De concrete transactie moet deze check met publicatie
serialiseren om een check-then-commitrace te voorkomen.

Een cataloguscommit volgt pas op werkelijk gecommitte exacte artifactbytes en
de toegestane installactie. Store-Unknown maakt de catalogus onbepaald volgens
de bestaande catalogusregels; authoritative herstel is geen hoger
generatienummer uit callerinput.

**Bestaand beleid blijft:** intrekking van een bron blokkeert nieuwe
pakketverificatie en updates, maar reeds geïnstalleerde apps blijven startbaar;
geen stille verwijdering van apps of gebruikersdata. Een geldig geïnstalleerd
artifact houdt zijn bestaande afzonderlijke launchpolicy. Deze uitzondering
mag niet worden gebruikt om een nieuw pakket als al geïnstalleerd te labelen.
Intrekking van een individuele launch-/netwerk-/objectgrant is een andere
handeling en blijft gewoon afdwingbaar.

## 8. Minimale implementatie- en acceptatiegrens

Vóór productieclaim zijn afzonderlijk nodig: geautoriseerde bronbeheeractor,
durable backend/ankerprovider, gecontroleerde registryreconstructie en
installer/launcherwiring door hun eigenaren. Geen daarvan is met dit document
geleverd. De bestaande vaste coregrenzen zijn proefgrenzen, geen productcaps.

Vereiste gerichte tegenproeven op de uiteindelijke implementatie:

- fout actor/domein/caller en directe package-/storepayload als autoriteit;
- stale generatie, verkeerde keyset, dubbele bron/keys, overflow en tombstonehergebruik;
- rotate/revoke tussen verificatie en installcommit;
- ieder hierboven benoemd crashvenster, verloren antwoord en Unknown zonder replay;
- authentieke oude snapshot, anker-/datamismatch en cold boot zonder anker;
- correcte bronrevoke: nieuwe install/update weigeren, bestaande app startbaar;
- artifactswap of catalogusreconstructie uit ongeautoriseerde metadata weigeren.

Hostmodel, crosscompile, AI-review en documentatie zijn geen native, duurzaam
herstel- of menselijke-auditbewijs. Bron-/artifactbinding en vereiste
QEMU-/menselijke gates blijven apart; dit contract kent geen punten toe.
Apps en release hebben ieder één gerichte read-only interfacelezing gedaan:
geen concrete interface-/beleidsfout gevonden binnen hun bestaande
SourceRegistry/catalogus respectievelijk store/launchgrens. Dit is geen
brede audit, nieuwe testuitvoering of productieacceptatie.
Rollback van alleen dit document vergt geen datamigratie en verandert geen code.

## 9. Projectreferenties

- [B02-records en sleutelstatus](../implementation/B02-SECURE-RECORDS.md)
- [Privacycontract](../LOCAL-PRIVACY-CONTRACT.md)
- [`APP-PACKAGES.md` op Apps-reviewbron](https://github.com/simonvanieperen/nieuw-os/blob/dce3b05aa358f4dbde6f2349b426217779b40052/docs/implementation/APP-PACKAGES.md)
- [`ARTIFACT-STORE-CONTRACT.md` op rootstorebron](https://github.com/simonvanieperen/nieuw-os/blob/7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb/docs/implementation/ARTIFACT-STORE-CONTRACT.md)
- [Gedeelde cryptoautoriteit op afzonderlijke SEC-leaf](https://github.com/simonvanieperen/nieuw-os/blob/c5fbb21a0e46e010d5e9b4748ae9b9307a67cef4/docs/contracts/NATIVE-CRYPTO-AUTHORITY.md)
