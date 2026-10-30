# Gerichte RustCrypto-geheugenhygiënepatch

Dit zijn lokale runtimevarianten van **HKDF0.13.0** en **HMAC0.13.0**, met
uitsluitend gewijzigde bufferlevensduur/wissen, traitcontroles en dependency-
featurekoppelingen. De KDF-/MAC-algoritmen, labels, bytevolgorde en formaten
blijven upstream. Dit is geen upstreamrelease of onafhankelijke crypto-audit.

| Upstreamarchief | SHA256 |
|---|---|
| hkdf-0.13.0.crate | 4aaa26c720c68b866f2c96ef5c1264b3e6f473fe5d4ce61cd44bbe913e553018 |
| hmac-0.13.0.crate | 6303bc9732ae41b04cb554b844a762b4115a61bfaa81e3e83050991eeb56863f |

De originele officiële bronarchieven staan in `upstream/`; hun oorspronkelijke
bron kan rechtstreeks uit het archief worden gelezen. Elke uitgepakte variant
behoudt `Cargo.toml.upstream`, `Cargo.toml.orig`, README en de volledige
MIT-/Apache-2.0-licenties met oorspronkelijke auteursvermeldingen.
De kleine brondiff staat in
[`hygiene.diff`](../verification/hkdf-hygiene/hygiene.diff).
De lokale Cargo-manifests selecteren alleen de bibliotheken; upstreamtests en
benches blijven in de originele archieven beschikbaar. Lokale regressies en
oudeversie-orakels staan onder `../tests/`.

De wrappers wissen in normale return-/drop-paden:

- De verworpen PRK in `Hkdf::new` en de optionele KDF-traitadapter.
- Elk HKDF-expandblok en zijn vorige blok, ook bij vervanging/einde.
- Het genormaliseerde HMAC-keyblok, ipad/opad en de lange-key-hash.
- De interne HMAC-hash, ook in beide resetvarianten.
- Bewaarde SimpleHmac-ipad/opad via `Zeroizing`.

Voor bewaarde HMAC-hashstates is **ook** `sha2=0.11.1/zeroize` vereist.
De upstream SHA2-Drop wist state/block_len; digest/block-buffer wist de
pending buffers. `ZeroizeOnDrop`-markers zijn alleen beschikbaar wanneer
de werkelijk gekozen hash/core dezelfde marker heeft. De productielibrary
controleert deze concrete SHA256→HMAC→HKDF-keten tijdens compilatie.
Een verwijderde SHA2-zeroize-feature moet die compilatie laten mislukken.

`extract` retourneert volgens upstream bewust een PRK aan zijn caller;
die caller moet zijn eigen geretourneerde bytes beschermen. `new` wist de
PRK die deze API zelf weggooit. De productiecode gebruikt `new`/`from_prk`.
HMAC/HKDF-uitvoer naar een caller wordt niet onverwacht gewist. De tests
controleren bytepariteit met de oorspronkelijke implementaties.

Deze ingreep claimt geen volledige erasure: diepe SHA-compressietemporaries,
Rust-moves, compiler-/registerkopieën en abort/crashpaden zijn niet volledig
onderzocht of gewist door deze patch. Geen lezen van vrijgegeven geheugen
of onveilige objectoverschrijving wordt als wisbewijs gebruikt.

Bij een upstreamupdate moeten archiefherkomst/licenties, patch, actieve
graaf/features, markerketen, RFC/KAT-/bytepariteit en scalar1.90-nonebuilds
opnieuw worden gecontroleerd. SHA2 wordt rechtstreeks uit de registry
gebruikt; zijn algoritmebron is niet lokaal gewijzigd.
