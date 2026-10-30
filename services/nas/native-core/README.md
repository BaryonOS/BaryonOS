# NAS-CORE-1 — begrensde Rust-beheercore

**Implementatiestap voor P04; nog geen native NAS-dienst of rubricpunt.**
Deze zelfstandige `no_std`-crate heeft geen externe afhankelijkheden, allocator,
host-API of cryptografische primitief. De bestaande Node-proeven blijven
ongewijzigd. De core bewaart uitsluitend opaque clientciphertext en noodzakelijke
beheermetadata; zij ontvangt geen documentnamen of inhoudssleutels.

## Contract vóór native aansluiting

De vertrouwde ingress levert `Principal`, een reeds cryptografisch
geverifieerde apparaat-/beheerdersidentiteit. Deze publieke fingerprint is
**geen** authenticatiebewijs op zichzelf; JSON of IPC-invoer mag haar niet
zelf tot afzender promoveren. De native adapter moet dit uit geauthenticeerd
transport en kernelafgedwongen endpoints halen. De constructor en publieke
Rust-methoden vormen geen kernelbeveiligingsgrens.

Beheer en gast zijn afzonderlijke principals. Een uitnodiging bindt een
verse namespace en eenmalige nonce aan de bedoelde ontvanger en vervaltijd.
De beheerder stelt het quotum afzonderlijk in en mag dat aanpassen.
Aanmaken en accepteren zijn duurzame transacties. De adapter
controleert bovendien de NAS-identiteit, gekozen LAN/internetrechten en
betrouwbare tijd; die zaken worden niet door deze core gesimuleerd.
`Invitation` is alleen de lokale store-tuple, geen vastgesteld interoperabel
NAS-token, ondertekend netwerkbericht of multideviceherstelcontract.
De transport-envelop moet de gekozen NAS-identiteit afzonderlijk binden.
Het B02-keyringfixture wordt hier niet als NAS-contract gebruikt.

De aanroeper reserveert vooraf een `Workspace` buiten de native stack,
bijvoorbeeld in exclusief statisch of toegewezen procesgeheugen. Zowel
`initialize` als `recover` lenen dat werkgeheugen exclusief. State,
transactiekopie, snapshot en tijdelijke uploads leven daar; de core voegt
geen allocator of `unsafe` toe. `validate_snapshot` controleert uitsluitend
schema, niet authenticiteit of actualiteit. De geoptimaliseerde hostproef
met 64 KiB stack controleert het bestaande native stackbudget als
vooronderzoek; daadwerkelijke kerneluitvoering blijft afzonderlijk vereist.

Sessies en uploads zijn aan caller, namespace, epoch en een verse boot-ID
gebonden. Ze worden nooit opgeslagen of na herstart hersteld. Staging kopieert
de invoer, telt tegen het quotum en publiceert nog niets. Commit controleert
de oorspronkelijke revisie opnieuw; een concurrerende schrijver krijgt een
conflict, geen stille overschrijving. Annuleren en sluiten geven uitsluitend
eigen reserveringen vrij. Lezen levert bij iedere weigering een ongewijzigde
uitvoerbuffer op.

`Storage::replace` moet een hele snapshot atomisch en duurzaam vervangen;
een oude of een volledige nieuwe snapshot zijn de enige toegestane
crashuitkomsten. De adapter moet exclusief één schrijver garanderen. Succes
vóór flush of alleen een RAM-kopie voldoen niet. Iedere fout bij vervangen
blokkeert de instantie; zij herhaalt geen mutatie. `Unknown` betekent dat
commit mogelijk wel plaatsvond. De aanroeper moet opnieuw openen en de
duurzame toestand inspecteren, niet opnieuw succes veronderstellen.

Intrekken legt eerst `Revoked` met een hogere epoch vast en verwijdert daarna
RAM-handles. Eviction doet een **tweede** transactie die de beheerde
ciphertextkopieën verwijdert en `Evicted` bewaart. Mislukt die tweede stap,
dan blijft de reeds duurzame intrekking gelden. De beheerder heeft voor beide
stappen geen verbinding, toestemming of decryptiesleutel van de gast nodig.
Een volle gastquota blokkeert de beheertransacties niet. Tombstones worden
niet hergebruikt: een nieuw aanbod krijgt een nieuwe namespace.

Dit is logische verwijdering van deze servicekopie, geen fysieke secure
erase, verwijdering van onafhankelijke back-ups of lokale clientbestanden.
De storageadapter moet ook eigen tijdelijke snapshots en eventuele
opslaggeschiedenis volgens dat beheercontract opruimen; zonder dat bewijs
mag de productinterface geen volledige verwijdering claimen.

## Snapshot en proefgrenzen

`NOSNAS01` v1 heeft een canonieke little-endian encoding met vaste lengte
`SNAPSHOT_BYTES` (17.216 bytes). Header, versie, nulvelden, namespace-status,
ID-uniciteit, quota, revisies, lengtes en ongebruikte payloadruimte worden
vóór herstel gevalideerd. Een ontbrekend of beschadigd bestaand record wordt
nooit stil een lege store. Initialisatie is een expliciete andere operatie.

De snapshot biedt **geen eigen checksum, MAC of antirollbackanker**.
De storageadapter moet integriteit en crashconsistentie leveren; een
kwaadwillige NAS-beheerder kan de eigen autoriteitsdatabase vervangen,
oude ciphertext aanbieden of beschikbaarheid beëindigen. De client moet
AEAD-binding en vertrouwde revisiegeschiedenis zelfstandig verifiëren.
Een schema-validator detecteert niet iedere semantisch geldige bitwijziging.

Voor deze eerste bouwstap: vier namespaces inclusief tombstones, vier
objectslots per namespace, 1.024 ciphertextbytes per object, zestien sessies
en acht uploads. Copy-on-write-reservering telt de oude én nieuwe bytes.
Dit zijn zichtbare proefgrenzen, geen geaccepteerde dagelijkse capaciteit.
Streamen, productiecapaciteit, netwerkservice, clientencryptie, naam-/manifest-
encoding, achtergrondsync, meerdere apparaten, sleutelrotatie/herstel,
native UI en onafhankelijke menselijke beoordeling blijven open.

## Gerichte controles

Met de bestaande Rust 1.90.0-toolchain en een eigen targetmap:

```sh
cargo test --offline --locked --manifest-path services/nas/native-core/Cargo.toml
cargo test --offline --locked --release --manifest-path services/nas/native-core/Cargo.toml
cargo clippy --offline --locked --manifest-path services/nas/native-core/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path services/nas/native-core/Cargo.toml --all -- --check
cargo build --offline --locked --manifest-path services/nas/native-core/Cargo.toml --release --target x86_64-unknown-none
```

De hosttests injecteren opslaguitkomsten vóór en na commit. Zij bewijzen
geen stroomuitval, QEMU-uitvoering, native transportauthenticatie of complete
P04-acceptatie. Actuele commando's, exits en bronbinding horen in de bestaande
projectstatus en verificatiebundle, niet als onuitgevoerde groene claim hier.
