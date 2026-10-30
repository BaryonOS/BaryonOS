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

## Test en bewijsgrens

```sh
cargo +1.90.0 test --offline --locked -p app-package-core
cargo +1.90.0 clippy --offline --locked -p app-package-core --all-targets -- -D warnings
CARGO_ENCODED_RUSTFLAGS=$'--cfg\x1fcurve25519_dalek_backend="serial"' \
  cargo +1.90.0 build --offline --locked -p app-package-core --target x86_64-unknown-none
```

Deze tests bewijzen alleen host-/bare-metal-libraryverificatie. Zij tonen niet
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
