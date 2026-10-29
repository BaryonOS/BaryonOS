# B01-voorbeeld: Key Service en acceptatie-app

Twee gewone `no_std`-programma's voor de Nieuw OS-ABI 1.4 (profiel
`native-keyring`, ADR-043). Het beleid en alle sleutelmateriaal zitten in de
bibliotheek [`keyring-core`](../../../services/keyring/); dit voorbeeld
verplaatst alleen bytes tussen de kernelbroker, de IPC-enveloppen en die core.

| Programma | Autoriteit bij start | Gedrag |
|---|---|---|
| `keyring-service` | één `Entropy`-capability, twee ingress-endpoints (domein 0 en 1), één `SEND` naar de app | haalt 1024 bytes opstartentropy via de kernelbroker, laat ze door de SP 800-90B-gezondheidstests lopen, wordt READY of BLOCKED; bedient STATUS, GENERATE, SEAL, UNSEAL, REVOKE, EXPORT_RECOVERY en RESTORE per domein; sleutelbytes verlaten het proces nooit |
| `keyring-app` | `SEND` naar beide domeinen van twee dienstinstanties, één eigen antwoordendpoint | drijft de levenscyclus (genereren, verzegelen, openen, intrekken, quotum), de negatieve paden (manipulatie, verkeerd domein, ingetrokken sleutel, fout herstelgeheim, ontbrekende entropy) en het herstel van instantie A naar instantie B; alle waarnemingen zijn bewijswoorden die de kernel controleert |

De cryptografie komt uit vastgepinde RustCrypto-crates; de build (`build.py`)
schakelt alle x86-SIMD-backends uit omdat het ring-3-profiel geen SSE/AVX
heeft (`chacha20_force_soft`, `poly1305_force_soft`, `sha2_backend="soft"`).
Het eigen linkerscript geeft de code 128 KiB vanaf `0x40020000`; de
startpagina blijft op `0x40010000`.

```sh
python3 -B sdk/examples/keyring/build.py
```

Standaarduitvoer: `build/sdk-keyring/x86_64-unknown-none/release/keyring-*`.
De kernelgate `tools/test_keyring.py` bouwt deze binaries zelf via
`tools/build.py --kernel-feature native-keyring` en de twee testvarianten.

## Grenzen

Geen persistente sleutelopslag, geen hardwarebescherming van de hoofdsleutel,
geen wachtwoordherstel. De app is een acceptatiefixture, geen productinterface
voor sleutelbeheer; de dienst is de eerste native Key Service-grens voor B02,
B03, N03 en P01.
