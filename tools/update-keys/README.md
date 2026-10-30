# TEST-ankers voor B03

Deze sleutels zijn **testankers** voor de B03-gate (`tools/test_update.py`),
gegenereerd met `update-tool keygen --dir tools/update-keys --label TEST`.
De `.seed`-bestanden zijn de private seeds en staan bewust in de repository:
de loader met features `signed-boot,test-anchors` en de ring-3 update-dienst pinnen de
bijbehorende `.pub`-bytes, zodat CI reproduceerbaar kan ondertekenen en
verifiëren. Zij zijn **geen** releasesleutels en beschermen niets buiten de
QEMU-acceptatie; een releaseprofiel vervangt de ankers in een afzonderlijke,
buiten CI gehouden ondertekenstap. De loader accepteert zonder `test-anchors`
alleen publieke releaseankers via absolute bestandspaden in
`NIEUWOS_RELEASE_ED25519_ANCHOR` en `NIEUWOS_RELEASE_MLDSA65_ANCHOR`;
de bekende TEST-sleutels worden daarbij bij compilatie geweigerd. De testloader
meldt `BOOT:ANCHORS:TEST`. De huidige update-dienst gebruikt nog TEST-ankers
en is geen releaseprofiel (ADR-044, invariant 7 in
docs/implementation/B03-SIGNED-UPDATES.md).

| Bestand | Inhoud |
|---|---|
| `TEST-ed25519.seed` | 32-byte Ed25519-seed (RFC 8032) |
| `TEST-mldsa65.seed` | 32-byte ML-DSA-65-seed (FIPS 204, `KeyGen_internal`) |
| `TEST-ed25519.pub` | 32-byte publieke sleutel (anker in loader en dienst) |
| `TEST-mldsa65.pub` | 1952-byte publieke sleutel (anker in loader en dienst) |
| `TEST-anchors.json` | sleutel-ID's (eerste 8 bytes van SHA-256 over de publieke sleutel) |
