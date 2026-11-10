# B01 — native entropy en sleutelbeheer (profiel `native-keyring`)

Datum: 29 oktober 2026. Status: **ontwerpcontract, daarna geïmplementeerd
in dezelfde opdracht**; uitgevoerde controles en hun uitkomst staan
uitsluitend in [B01-ACCEPTANCE](../verification/B01-ACCEPTANCE.md).
Afwijkingen die tijdens het bouwen nodig bleken, staan in §7.
Criterium B01 (docs/progress.json): *native cryptografische entropy, key
lifecycle, bescherming en herstel gebruiken gecontroleerde standaarden en
geteste failure paths.* Vervult P02 (entropy) uit
[VERIFIABLE-SECURITY-AND-PQC](VERIFIABLE-SECURITY-AND-PQC.md) §5 en legt de
Key Service-grens uit [ENCRYPTED-OBJECT-STORAGE](ENCRYPTED-OBJECT-STORAGE.md)
§5 native vast. Besluit: ADR-043.

## 1. Probleem

Tot nu toe heeft het eigen OS geen entropybron en geen sleutels. Zonder een
getoetste bron met gereedheid en foutpad mag er geen sleutel worden
gegenereerd (SECURITY-AND-QUANTUM.md §7). B02 (geauthenticeerde opslag), B03
(ondertekende updates), N03 en P01 hebben alle een Key Service nodig die
sleutels nooit aan gewone apps afgeeft.

## 2. Beveiligingsinvarianten (eerst)

1. **Entropy is een capability.** Alleen de Key Service houdt een
   `Entropy`-capability (recht `ENTROPY` = 1024, objecttype zonder
   `DERIVE`/`TRANSFER`/`REVOKE`). `entropy_read` zonder die capability geeft
   `EBADHANDLE`/`ETYPE`/`EACCESS`. De kernel geeft ruwe apparaatbytes door en
   mengt, cachet of hergebruikt niets.
2. **Geen sleutel vóór gereedheid.** De dienst wordt pas `READY` na 1024
   bytes opstartentropy die de SP 800-90B-gezondheidstests doorstaan
   (Repetition Count en Adaptive Proportion, §3). Een afwezige, mislukkende of
   vastgelopen bron laat de dienst in `BLOCKED`: iedere sleuteloperatie geeft
   `NotReady`. Er is geen zwakkere terugvaloptie en geen testseed in het
   productpad.
3. **Sleutels verlaten de dienst niet via app-ingress.** Apps krijgen alleen
   sleutelhandles (domein, index). SEAL/UNSEAL gebeuren in de dienst; de
   domeinautoriteit komt uitsluitend van het kernelgebonden ingress-endpoint.
   Een handle van domein A is in domein B ongeldig. Alleen de afzonderlijke
   manifest-gebonden beheer-ingress mag herstelmateriaal afgeven.
4. **Geen eigen cryptografie.** Alleen onderhouden RustCrypto-crates met
   vastgepinde versies: `chacha20poly1305` 0.11.0 (XChaCha20-Poly1305 AEAD,
   RFC 8439/draft-irtf-cfrg-xchacha), `chacha20` 0.9.1 (keystream voor de
   DRBG), `hkdf` 0.12.4 + `sha2` 0.10.9 (HKDF-SHA-256, RFC 5869), `zeroize`
   1.8. Alle x86-SIMD-backends zijn uitgeschakeld
   (`chacha20_force_soft`, `chacha20_backend="soft"`,
   `poly1305_backend="soft"`, `sha2_backend="soft"`): het
   profiel heeft `CR0.EM=1` en geen SSE. De DRBG is de
   fast-key-erasure-constructie op ChaCha20 (Bernstein, 2017): sleutel K,
   keystream → nieuwe K en uitvoer, oude K gewist.
   Current dependency provenance and the portable-backend evidence boundary
   are recorded in [KEYRING-CRYPTO-DEPENDENCIES.md](KEYRING-CRYPTO-DEPENDENCIES.md).
5. **Binding.** Iedere AEAD-bewerking bindt formaatversie, domein,
   sleutelindex en doel (seal/herstel) in de AAD. Een ciphertext van sleutel
   A onder sleutel B, een gewijzigde byte, tag of nonce geeft `Auth` zonder
   gedeeltelijke plaintext.
6. **Herstel zonder escrow.** EXPORT_RECOVERY levert een willekeurig 256-bit
   herstelgeheim en een blob die de hoofdsleutel plus intrekkingsstatus onder
   dat geheim verzegelt uitsluitend via een afzonderlijke beheer-ingress aan
   de manifest-genoemde supervisor/herstelprincipal. Gewone app-ingress wijst
   EXPORT_RECOVERY en RESTORE af, ook als de app de geldige wire-opcodes kent.
   RESTORE in een andere dienstinstantie herleidt daarmee alle domeinsleutels.
   Zonder het geheim is de blob onbruikbaar; de dienst bewaart het geheim niet.
7. **Intrekking.** REVOKE wist de afgeleide sleutel en markeert de index; de
   index wordt niet hergebruikt. Herstel onder dezelfde hoofdsleutel verenigt
   de bestaande en geïmporteerde intrekkingsbitmaps. Een andere hoofdsleutel
   mag alleen in een ongebruikte instantie zonder uitgegeven/geïmporteerde
   indices worden geïmporteerd; anders `Invalid`. Een oude blob mag geen
   bestaande intrekking terugdraaien of een index opnieuw uitgeefbaar maken.
   Dit geldt binnen de levende instantie. Een koude nieuwe dienst kan zonder
   vertrouwde persistente geschiedenis geen nieuwere intrekking uit een oude
   herstelblob afleiden; die rollbackgrens blijft open voor B02.
8. **Kernel blijft klein.** Geen cryptografie in de kernel; de kernelbroker
   kent alleen de virtio-rng-wachtrij, DMA-eigendom, timeouts en de
   `Entropy`-capability (kernel/AGENTS.md).

## 3. Entropybron en gezondheidstests

- Apparaat: virtio-rng-pci (modern, PCI 00:06.0, `-object rng-random,
  filename=/dev/urandom`), eigen driver `drivers/virtio-rng` op `virtio-core`
  (capability-parsing, één wachtrij van 8, één apparaat-beschrijfbare buffer
  van 256 bytes per aanvraag, gepinde DMA-pagina's). Geen interrupts; polling
  met deadline in ticks zoals de blokbroker.
- Syscalls `0x60 entropy_request(cap, len)` → ticket en `0x61
  entropy_poll(cap, ticket, ptr, len)` → bytes: `len` 1..=256; de broker
  houdt één aanvraag tegelijk (`EAGAIN` voor een tweede) en levert exact de
  door het apparaat geschreven bytes (kortere levering is `EIO`, nooit
  opgevuld); `-14 Offline` zonder apparaat, `-13 Timeout` na 64 ticks.
  Fouten laten de uitvoerbuffer ongewijzigd; na een geslaagde poll wist de
  kernel zijn kopie. Net als de blokbroker wacht de kernel nooit in een
  syscall op het apparaat.
- Opstart: 4 × 256 bytes. Repetition Count Test met cutoff C = 1 +
  ⌈20 / H⌉ bij een conservatief aangenomen H = 4 bits per byte → C = 6: zes
  gelijke opeenvolgende bytes falen. Adaptive Proportion Test met venster
  W = 512 en cutoff 62: de kleinste c met P[Bin(512, 2⁻⁴) ≥ c] ≤ 2⁻²⁰
  (exact binomiaal herberekend in `tests/test_keyring_harness.py`). Falen →
  `BLOCKED` met reden; geen herkansing zonder nieuwe start.
- Continu: iedere latere bijvulling (256 bytes per reseed, na elke 64
  sleuteloperaties) doorloopt dezelfde tests met blijvende toestand vanaf de eerste opstartbyte.
  RCT loopt over alle buffergrenzen; APT-vensters van 512 bytes gaan door over
  twee reseeds van 256 bytes. Falen blokkeert verdere sleuteloperaties met
  `Degraded`; geen nieuwe `Health` per reseed.
- Testfixture `test-keyring-stuck`: de kernel vervangt de apparaatbytes door
  een constante vóór de kopie naar de dienst. Dit is een gelabelde
  foutinjectie om de gezondheidstest te bewijzen, geen productpad.

## 4. Sleutelhiërarchie en bewerkingen (IPC v1, tag `KEY1`)

```text
master (32 B, uit DRBG)          ──HKDF-SHA-256(info = "nieuw-os key v1" ‖ domein ‖ index)──> DEK[domein][index]
recovery_secret (32 B, uit DRBG) ──XChaCha20-Poly1305(aad = "nieuw-os recovery v1")──> blob = nonce ‖ enc(master ‖ revoked-bitmap) ‖ tag
```

| Bereik | Op | Verzoek | Antwoord | Fouten |
|---|---|---|---|---|
| App-domein | STATUS | – | state (0 starting, 1 ready, 2 blocked, 3 degraded), reden, aantal sleutels | – |
| App-domein | GENERATE | – | handle (index 0..15 binnen het domein) | NotReady, Quota (16 per domein) |
| App-domein | SEAL | handle, plaintext ≤ 128 B | nonce ‖ ciphertext ‖ tag (≤ 168 B) | NotReady, BadHandle, Revoked |
| App-domein | UNSEAL | handle, nonce ‖ ciphertext ‖ tag | plaintext | NotReady, BadHandle, Revoked, Auth |
| App-domein | REVOKE | handle | – | BadHandle |
| Beheer | EXPORT_RECOVERY | – | secret (32 B), blob (76 B) | NotReady |
| Beheer | RESTORE | secret, blob | aantal herstelde domeinen | NotReady, Auth, Invalid (andere master in gebruikte instantie) |

De dienst houdt per domein een teller en de intrekkingsbitmap bij; het
domeinnummer volgt uit het endpoint waarop het app-verzoek binnenkomt. De
beheer-ingress accepteert alleen de manifest-genoemde caller, zonder
gedelegeerde capabilities. Verzoeken met een capability, verkeerde tag of
lengte worden geweigerd zonder effect.

## 5. Acceptatiegevallen (echte QEMU-runs)

| Geval | Feature | Vereiste observatie |
|---|---|---|
| normal | `native-keyring` | dienst A en B worden READY (opstartentropy geslaagd); app in domein 0 en 1: GENERATE, SEAL/UNSEAL rondgang, cross-domain handle geweigerd, gemanipuleerde byte/tag/nonce geweigerd zonder plaintext, REVOKE → Revoked, 17e sleutel → Quota; app-ingress weigert EXPORT_RECOVERY en RESTORE zonder herstelbytes af te geven of toestand te wijzigen. De afzonderlijke beheer-wiretest exporteert en herstelt herstelmateriaal via de management-dispatcher; dit is hostbewijs, geen native caller-authenticatie. Kernel: alleen de twee diensten lezen entropy, appverzoeken om entropy → geweigerd |
| no-device | `test-keyring-no-device` (QEMU zonder virtio-rng) | `entropy_read` → Offline; beide diensten BLOCKED; app: elke GENERATE → NotReady; kernel: 0 geslaagde entropyleveringen, 0 sleutels |
| stuck | `test-keyring-stuck` | gelabelde constante-bytefixture na echte completion; Repetition Count faalt bij de opstart; BLOCKED met reden; geen sleutels |
| timeout | `test-keyring-timeout` | lokale stille rng-egd-backend ontvangt de echte apparaataanvraag maar levert niets; deadline, reset-ACK, 0 geleverde bytes, 0 sleutels en behoud van de gepinde DMA-pool |
| restart | `test-keyring-restart` | normale bron; na de sleuteltaken gecontroleerde reset-ACK, herinitialisatie van dezelfde DMA-pool en een nieuwe aanvraag van 32 bytes met echte completion |

Markers `KEYRING:MANIFEST:PASS`, `KEYRING:READY`, `KEYRING:CASE:<X>:PASS`,
`KEYRING:RESOURCES:PASS`, `KEYRING:PASS`, `M1:PASS`; getalsbewijs
`KEYRING:<NAAM>:<20 cijfers>`. Gate `tools/test_keyring.py`: hosttests
(`keyring-core` met RFC 8439/draft-xchacha- en RFC 5869-testvectoren, de
90B-tests met synthetische reeksen, wire-codec), daarna de vijf runs op één
bevroren bron.

## 6. Wat B01 niet bewijst

Geen hardwarebescherming van de hoofdsleutel (TPM), geen wachtwoordherstel
(Argon2id volgt later met gemeten parameters), geen persistente sleutelopslag
op schijf (B02), geen SMP, geen fysieke RNG-kwaliteit (QEMU levert
`/dev/urandom` van de host: de gezondheidstests toetsen gedrag, niet de
werkelijke minimumentropie van echte hardware), geen zijkanaalweerstand van
de softwarebackends buiten constante-tijdimplementatie van de crates zelf.

## 7. Afwijkingen tijdens implementatie

- **Asynchrone broker.** Eén syscall die op het apparaat wacht zou de kernel
  blokkeren; daarom twee syscalls (aanvraag/ticket en poll), zoals de
  blokbroker. Twee diensten delen één brokerslot en herhalen bij `EAGAIN`.
- **Telling in de normalcase.** De app genereert 2 + 1 + 14 = 17 sleutels
  (waarna het quotum van 16 per domein `Quota` geeft), verzegelt en opent één
  keer. Herstel via de beheer-wiretest is een afzonderlijke hosttest; het
  oude bewijs voor app-ingress wordt niet als bewijs voor deze grens gebruikt.
- **Codebudget.** De dienst met de portable crypto-backends is 50 KiB code;
  het voorbeeld gebruikt een eigen linkerscript met 128 KiB code vanaf
  `0x40020000` en een losse entry-pagina op `0x40000000`, binnen de
  bestaande gebruikersgrenzen van de kernel (`0x40000000–0x60000000`).

## 8. Herstelreview 29 oktober 2026

De post-merge P1-bevindingen op #52 heropenden B01. Naast de native gevallen
moeten regressies oude blobs na intrekking, verschillende domeinen, geweigerde
masterwissel met behoud van bestaande sleutels en RCT/APT over reseeds testen.
Eigenaarsverlies met een open RNG-aanvraag stopt de driver en sluit de broker
af; de DMA-pool blijft in quarantaine tot reboot. Timeout controleert reset-ACK
met een deadline; zonder bevestigde reset blijft de pool gepind en de broker
offline. Requestercrash en ontbrekende reset-ACK zijn niet native getest. Een timeout is een ander geval dan de constante-
bytefixture; de afzonderlijke herstartproef bewijst geen herstel van een
vastgelopen of fysiek apparaat. Uitvoering uitsluitend in het nieuwe bewijs.
