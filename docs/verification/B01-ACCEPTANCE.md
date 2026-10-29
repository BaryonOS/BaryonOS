# B01 — acceptatie native entropy en sleutelbeheer

Datum: 29 oktober 2026 (Europe/Amsterdam). Contract:
[B01-ENTROPY-AND-KEYS](../implementation/B01-ENTROPY-AND-KEYS.md); besluit
ADR-043. Bewijsmap: [`b01/`](b01/).

## Geaccepteerde eigenschap

Criterium B01: *native cryptografische entropy, key lifecycle, bescherming en
herstel gebruiken gecontroleerde standaarden en geteste failure paths.* Op het
eigen OS in het QEMU-referentieprofiel levert de kernel ruwe virtio-rng-bytes
uitsluitend aan houders van een `Entropy`-capability; twee ring-3 Key
Service-instanties worden pas READY na 1024 bytes die de SP 800-90B-tests
doorstaan, genereren/verzegelen/openen/trekken sleutels in per domein en
herstellen een hoofdsleutel in een andere instantie onder een
gebruikersgehouden herstelgeheim. Zonder apparaat of met een vastgelopen
apparaat blijven beide diensten geblokkeerd en ontstaat geen sleutel.

## Bronbinding

- Branch `ccr-43832ef7-gi2qlz`, basis `main` `c92238b` (na de squash van K05).
  De gate draaide op de werkboom die als PR-head is gecommit; bronvingerafdruk
  v3 `202a3e684c57c8b0…` (SHA-256 over de broninputs). De hosttestinputs
  `tests/test_keyring_*.py` zijn apart in `b01/keyring-gate.json` vastgelegd en
  vóór het verdict opnieuw gecontroleerd.
- Omgeving: Linux-container, Rust 1.90.0, QEMU 8.2.2 (Ubuntu-pakket
  `1:8.2.2+ds-0ubuntu1.18`), `pc-q35-8.2`, TCG, 1 CPU, 256 MiB, OVMF 4M,
  `NOS_QEMU_MACHINE=pc-q35-8.2`. Entropyapparaat: `virtio-rng-pci` op 00:06.0
  met `rng-random,filename=/dev/urandom` (host-urandom; geen claim over
  fysieke RNG-kwaliteit).
- SHA-256 van kernel-ELF en de twee gebruikersbinaries staan per stap in
  `b01/<n>-<case>/result.json`.
- Dependencies (nieuw, vastgepind in `Cargo.lock` en `sdk/Cargo.lock`):
  `chacha20poly1305` 0.10.1, `chacha20` 0.9.1, `poly1305` 0.8.0, `hkdf`
  0.12.4, `hmac` 0.12.1, `sha2` 0.10.9, `digest` 0.10.7, `zeroize` 1.9.1 en
  hun RustCrypto-basis (`aead`, `cipher`, `universal-hash`, `crypto-common`,
  `generic-array`, `typenum`, `subtle`, `cpufeatures`, `inout`, `block-buffer`,
  `opaque-debug`, `cfg-if`). Licenties MIT OR Apache-2.0 (`subtle`
  BSD-3-Clause). Buildscript alleen in `generic-array` (`version_check`).
  Backends geforceerd portable; `objdump` van de proefbinary toonde 0 SIMD-
  instructies.

## Uitgevoerde commando's

```sh
export NOS_QEMU_MACHINE=pc-q35-8.2
python3 -B -u tools/test_keyring.py                          # exit 0
python3 -B tools/capture_keyring.py                           # exit 0 -> docs/verification/b01/
```

## Hostcontroles (exit 0)

| Controle | Resultaat |
|---|---|
| `cargo test -p capability-core -p virtio-rng-driver -p keyring-core` | 40 + 10 + 3 = 53 tests: `Entropy`-type (typering, geen derive/transfer/revoke, quota 4, eigenaarsexit), virtio-rng-kenmerkonderhandeling en used-ring-validatie, en de keyring-core-suite: HKDF-SHA-256 RFC 5869 testgeval 1, XChaCha20-Poly1305 draft-irtf-cfrg-xchacha-03 §A.3.1, DRBG-determinisme en sleutelwissing, 90B-tests (gevarieerd, vastgelopen, bevooroordeeld), blokkeren zonder/vastgelopen bron, seal/unseal met binding en manipulatie, intrekking en quotum, herstel in een andere instantie, reseed en degradatie, canonieke wire-headers |
| `cargo test --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | 17 tests: ABI 1.4, rechtenmasker 2047, statuswoorden |
| `python3 -m unittest discover -s tests -p 'test_keyring_*.py'` | 7 harnastests (markervolgorde, failure-markers, exitcode, elk getal dragend) |

## QEMU-runs (alle raw exit 33, `KEYRING:PASS`, `M1:PASS`)

| # | Geval | Feature | Waargenomen (kernelbewijs) |
|---|---|---|---|
| 0 | normal | `native-keyring` | beide diensten READY (reden 0) na elk 1024 bytes; 2048 bytes geleverd; app: 17 sleutels, 1 verzegeling, 2 openingen, 4 manipulaties geweigerd (nonce, inhoud, tag, afkapping), 3 verkeerde-sleutel/domeinpogingen geweigerd, ingetrokken sleutel geweigerd, 17e sleutel `Quota`; herstel: 2 pogingen vóór RESTORE geweigerd, fout geheim geweigerd, na RESTORE opent B de ciphertext van A en blijft de ingetrokken sleutel ingetrokken (B: 32 sleutelindices); app-aanvraag om entropy met endpoint-handle `ETYPE` |
| 1 | no-device | `test-keyring-no-device` (zonder virtio-rng) | broker Offline (-14) bij ≥2 aanvragen, 0 bytes geleverd; beide diensten BLOCKED reden Source (1); GENERATE/SEAL → NotReady; 0 sleutels |
| 2 | stuck | `test-keyring-stuck` (gelabelde fixture: constante bytes) | ≥256 bytes geleverd; Repetition Count faalt; beide diensten BLOCKED reden Repetition (2); 0 sleutels |

Elke run eindigt met een lege capability-engine (ook 0 `Entropy`-objecten),
de framebaseline en een kernel-eigen DMA-pool; de hostrunner controleert
markervolgorde, raw exit, `FAIL`/`PANIC` en de getallen onafhankelijk
(`tools/run_keyring.py`, `EXPECT`/`AT_LEAST`).

## Negatieve tests

- Zonder `Entropy`: `entropy_request` met endpoint-handle → `ETYPE`
  (native, app); hosttests: `derive`/`revoke`/overdracht van het
  `Entropy`-type → `EINVAL`, vreemd proces → `EBADHANDLE`, 5e object → `EQUOTA`.
- Ontbrekend apparaat → Offline; vastgelopen apparaat → gezondheidstest
  blokkeert; kort/leeg antwoord → `EIO` (hosttest driver); tweede aanvraag
  tegelijk → `EAGAIN`.
- Manipulatie van nonce, ciphertext, tag of lengte → `Auth` zonder
  plaintext; verkeerde sleutel of ander domein → `Auth`; onbestaande index →
  `BadHandle`; ingetrokken sleutel → `Revoked`; 17e sleutel → `Quota`;
  fout herstelgeheim of gewijzigde blob → `Auth`; UNSEAL vóór herstel →
  `BadHandle`.

## Bewijsgrens

Niet bewezen: fysieke RNG-kwaliteit of min-entropie (QEMU levert
host-urandom; de 90B-tests toetsen gedrag), hardwarebescherming van de
hoofdsleutel, persistente sleutelopslag (B02), wachtwoordherstel (Argon2id
volgt met gemeten parameters), zijkanaalweerstand buiten de constante-tijd-
implementaties van de crates, SMP. De GitHub Actions-workflow
`B01 entropy and key management acceptance` moet op iedere PR-head groen
zijn; zie PROJECT-STATE.md voor de run op de eindhead.
