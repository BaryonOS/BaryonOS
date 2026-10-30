# TEST-ankers voor B03

Deze sleutels zijn **testankers** voor de B03-gate (`tools/test_update.py`),
gegenereerd met `update-tool keygen --dir tools/update-keys --label TEST`.
De `.seed`-bestanden zijn de private seeds en staan bewust in de repository:
de loader met feature `signed-boot` en de ring-3 update-dienst pinnen de
bijbehorende `.pub`-bytes, zodat CI reproduceerbaar kan ondertekenen en
verifiëren. Zij zijn **geen** releasesleutels en beschermen niets buiten de
QEMU-acceptatie; een releaseprofiel vervangt de ankers in een afzonderlijke,
buiten CI gehouden ondertekenstap (ADR-044, invariant 7 in
docs/implementation/B03-SIGNED-UPDATES.md).

| Bestand | Inhoud |
|---|---|
| `TEST-ed25519.seed` | 32-byte Ed25519-seed (RFC 8032) |
| `TEST-mldsa65.seed` | 32-byte ML-DSA-65-seed (FIPS 204, `KeyGen_internal`) |
| `TEST-ed25519.pub` | 32-byte publieke sleutel (anker in loader en dienst) |
| `TEST-mldsa65.pub` | 1952-byte publieke sleutel (anker in loader en dienst) |
| `TEST-anchors.json` | sleutel-ID's (eerste 8 bytes van SHA-256 over de publieke sleutel) |

Dezelfde vijf bestanden bestaan met labels `TEST-next`, `TEST-revoked` en
`TEST-expired`. Dit zijn eveneens opzettelijk openbare QEMU-fixtures, geen
geheimen of productieankers. `update-core::test_policy` is de enige gedeelde
beleidstabel voor loader en dienst:

| Paar | Toegestane releaseversies | Toestand |
|---|---|---|
| `TEST` | 1 tot en met u64::MAX | bestaand paar, behouden voor regressies |
| `TEST-next` | 2 tot en met u64::MAX | vooraf vastgepinde opvolger |
| `TEST-revoked` | geen | onvoorwaardelijk ingetrokken |
| `TEST-expired` | alleen 1 | buiten dit releasevenster geweigerd |

`KEY_EXPIRED` betekent hier **buiten de toegestane releaseversies**, niet
kalenderverval. De eigen RTC is geen vertrouwde tijdbron. Online sleutel-
discovery, dynamische truststore-updates en productie-intrekkingsdistributie
zijn niet geïmplementeerd. Het manifest kan geen anker of beleid toevoegen.
`update-tool verify` controleert een expliciet gegeven sleutelpaar alleen
cryptografisch; `verify-pinned` controleert het gedeelde QEMU-beleid.
