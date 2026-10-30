# B03 — ondertekende updates en rollback (profiel `native-update`, loader `signed-boot`)

Datum: 29 oktober 2026. Status: **ontwerpcontract, daarna geïmplementeerd in
dezelfde opdracht**; uitgevoerde controles en hun uitkomst staan uitsluitend
in [B03-ACCEPTANCE](../verification/B03-ACCEPTANCE.md). Criterium B03
(docs/progress.json): *authentieke updates, onderbreking, downgrade-aanvallen
en herstel naar een geldige vorige versie zijn end-to-end getest.* Vervult
P03 (systeemimage) uit [VERIFIABLE-SECURITY-AND-PQC](VERIFIABLE-SECURITY-AND-PQC.md)
§5 en EU-07 uit [EUROPE-AND-RELEASE](EUROPE-AND-RELEASE.md). Besluit: ADR-044.

## 1. Probleem

De loader laadt tot nu toe `kernel.elf` van de alleen-lezen ESP zonder enige
herkomstcontrole; er is geen tweede slot, geen versiebeleid en geen manier om
een nieuwe kernel te installeren. B03 voegt een **slotopslag** toe op een
blokapparaat, een **hybride ondertekend manifest** per slot, verificatie in de
loader met vastgepinde ankers, een firmwaregebonden **versievloer** tegen
downgrades, en een ring-3 **update-dienst** die een bundel in twee fasen
installeert zodat een onderbreking nooit het laatst geldige slot aantast.

## 2. Beveiligingsinvarianten (eerst)

1. **Beide handtekeningen of niets.** Een manifest is geldig als én de
   Ed25519-handtekening én de ML-DSA-65-handtekening over exact dezelfde
   canonieke bytes verifiëren tegen de vastgepinde ankers. Eén geldige
   handtekening is onvoldoende; een ontbrekende, verwisselde of over andere
   bytes gezette handtekening wordt geweigerd. Geen downgrade naar één
   algoritme.
2. **Digest bindt image aan manifest.** De loader hasht het volledige image
   dat het uit het slot leest en vergelijkt met `image_sha256` in het
   manifest vóór één byte wordt uitgevoerd. Lengte, doel-ID, formaatversie en
   slotpositie staan in de canonieke bytes.
3. **Versievloer.** De loader bewaart de hoogste gestarte versie in de
   UEFI-NV-variabele `NieuwOsMinVersion` (eigen vendor-GUID). Een slot met
   `version < vloer` wordt geweigerd als `DOWNGRADE`, ook als de pointer in de
   header ernaar wijst. De vloer stijgt alleen; zij daalt nooit automatisch.
4. **Terugval naar de vorige geldige versie.** De header-pointer is een hint,
   geen autoriteit. Faalt het aangewezen slot (handtekening, digest, vloer,
   lengte, lezen), dan probeert de loader het andere slot. Pas als beide
   falen eindigt de boot gecontroleerd met `BOOT:FAIL:NO_VALID_SLOT`.
5. **Onderbreking laat het actieve slot intact.** De update-dienst schrijft
   uitsluitend naar het niet-actieve slot: eerst image, FLUSH, dan manifest,
   FLUSH, dan de header met de nieuwe pointer, FLUSH. Een onderbreking vóór
   de laatste FLUSH laat de oude pointer en het oude slot ongewijzigd; de
   installatie is idempotent en wordt bij de volgende boot herhaald.
6. **De dienst verifieert zelf.** De update-dienst accepteert een bundel
   alleen na dezelfde hybride verificatie, digestcontrole en versiecontrole
   (`version > huidige` en `version ≥ vloer`); de kernel kent geen formaten
   en de loader vertrouwt de dienst niet: beide controleren onafhankelijk.
7. **Geen productiesleutels in de repository.** De ankers in loader en
   dienst zijn expliciet gelabelde TEST-ankers (`tools/update-keys/TEST-*`),
   gegenereerd door `update-tool keygen`. Een releaseprofiel vervangt de
   ankers via een afzonderlijke, buiten CI gehouden ondertekenstap (ADR-044).
8. **Firmwaregrens benoemd.** OVMF start de loader zonder Secure Boot; de
   loader zelf wordt niet door de firmware geverifieerd (B04 open). B03
   beschermt de keten vanaf de loader: slotkeuze, manifest, image, vloer.

## 3. Formaten (`crates/update-core`, `no_std`, hosttests)

### Manifest (`NOSMAN01`, 3 500 bytes, canoniek little-endian)

| Offset | Veld |
|---|---|
| 0 | magic `NOSMAN01` |
| 8 | `format_version` u32 = 1 |
| 12 | `target` u32 (1 = kernel) |
| 16 | `image_length` u32 (≤ 2 MiB) |
| 20 | `version` u64 |
| 28 | `min_version` u64 (door de ondertekenaar beweerde ondergrens) |
| 36 | `image_sha256` [32] |
| 68 | `ed25519_key_id` [8] (eerste 8 bytes van SHA-256 over de publieke sleutel) |
| 76 | `mldsa_key_id` [8] |
| 84 | `ed25519_signature` [64] |
| 148 | `mldsa65_signature` [3309] |
| 3457 | nul tot 3500 |

De canonieke boodschap is `bytes[0..84]`; beide handtekeningen staan over
exact die 84 bytes. `key_id`'s moeten overeenkomen met de ankers; een
onbekende sleutel-ID wordt als `UNKNOWN_KEY` geweigerd zonder handtekening te
verifiëren.

### Slotopslag (`NOSSLOT1`, sectoren van 512 B)

| Sector | Inhoud |
|---|---|
| 0 | header: magic, `format_version` u32, `active` u32 (0/1), `generation` u64, voor slot A en B elk `manifest_sector` u32, `image_sector` u32, `image_capacity_sectors` u32; `inbox_manifest_sector` u32, `inbox_image_sector` u32, `inbox_capacity_sectors` u32; rest nul |
| 1–7 | slot A manifest (3 500 B in 7 sectoren) |
| 8–4103 | slot A image (2 MiB; een kernelimage met ingesloten ring-3-programma's is ca. 1,5 MiB) |
| 4104–4110 | slot B manifest |
| 4111–8206 | slot B image |
| 8207–8213 | inbox manifest |
| 8214–12309 | inbox image |

Totaal 12 310 sectoren (ca. 6 MiB).

De header draagt geen handtekening; zij is een hint. Vaste posities worden
door loader en dienst gecontroleerd tegen deze tabel (geen vertrouwen in
header-offsets buiten het bereik van het apparaat).

## 4. Loader (`boot/loader`, feature `signed-boot`)

1. Zoek via `LocateHandleBuffer(BLOCK_IO)` het eerste blokapparaat waarvan
   sector 0 de slotmagic draagt (≤ 8 handles). Geen apparaat → bestaand
   M1-pad (`kernel.elf` van de ESP), zodat alle bestaande profielen ongewijzigd
   blijven.
2. Lees header, lees de vloer uit `NieuwOsMinVersion` (ontbreekt → 0).
3. Voor het aangewezen slot en daarna het andere: lees manifest, controleer
   magic/versie/doel/lengte, `key_id`'s, beide handtekeningen, lees het image
   (`image_length` bytes), SHA-256, vergelijk; weiger `version < vloer`.
   Serieel: `BOOT:SLOT:<A|B>:VERSION:<n>` of
   `BOOT:SLOT:<A|B>:REJECTED:<SIGNATURE|DIGEST|DOWNGRADE|FORMAT|READ>`.
4. Verhoog de vloer naar `version` van het gekozen slot (`SetVariable`,
   NV+BS). Laad het image zoals `kernel.elf` (zelfde ELF-validatie), druk
   `BOOT:SIGNED:OK` af en ga door met de bestaande overdracht.

Cryptografie in de loader: `ed25519-dalek` 2.2.0 (serial backend) en
`ml-dsa` 0.1.1 (RustCrypto, pure Rust), `sha2` 0.10.9. Geen allocaties
buiten de bestaande pagina-allocaties; het image wordt in het bestaande
`MAX_FILE_SIZE`-buffer gelezen.

## 5. Kernelprofiel `native-update` en update-dienst

- Kernel: generieke blokbroker (zoals `native-volume`, zelfde syscalls
  0x40–0x43) met een scope over de hele slotopslag en een ring-3
  update-dienst (slot 0) plus een observer-app (slot 1) die de uitkomst
  rapporteert. De kernel drukt zijn eigen imageversie af
  (`KERNEL:IMAGE_VERSION:<n>`, build-env `NOS_IMAGE_VERSION`) en kent geen
  manifest- of slotformaat.
- Dienst: leest header, bepaalt het actieve slot en de inbox; verifieert de
  inboxbundel (invariant 6); als `inbox.version > actieve versie`:
  installeert in het niet-actieve slot in de volgorde van invariant 5 en
  meldt `UPDATE:INSTALLED:<n>`; anders `UPDATE:UP_TO_DATE`. Geweigerde bundel:
  `UPDATE:REJECTED:<reden>` zonder één schrijfactie.
- Fixture `test-update-cut`: de kernel beëindigt de VM (echte hostkill) na
  een vast aantal image-writes in het niet-actieve slot, vóór de header-
  write. De volgende boot moet het oude slot starten en de installatie
  afmaken.

## 6. Acceptatiegevallen (echte QEMU-runs, één slotopslagimage, één OVMF_VARS)

| # | Run | Vereiste observatie |
|---|---|---|
| 1 | install | slot A = v1, inbox = v2; loader `SLOT:A:VERSION:1`, `SIGNED:OK`; dienst `UPDATE:INSTALLED:2`; hostorakel: slot B = v2-bytes + manifest, header `active=B`, slot A byte-identiek |
| 2 | boot-v2 | zelfde image/vars: loader `SLOT:B:VERSION:2`; `KERNEL:IMAGE_VERSION:2`; dienst `UP_TO_DATE`; vloer = 2 |
| 3 | downgrade | host zet header `active=A`; loader `SLOT:A:REJECTED:DOWNGRADE`, `SLOT:B:VERSION:2`; kernel v2 |
| 4 | bad-signature | verse vars; host flipt één byte in de ML-DSA-handtekening van slot B en zet `active=B`; loader `SLOT:B:REJECTED:SIGNATURE`, `SLOT:A:VERSION:1`; dienst installeert opnieuw (`INSTALLED:2`); orakel: slot B hersteld |
| 5 | bad-digest | verse image; host flipt één imagebyte in slot B (manifest intact); `SLOT:B:REJECTED:DIGEST`, terugval naar A |
| 6 | cut | verse image, feature `test-update-cut`: echte kill tijdens image-writes; herboot: `SLOT:A:VERSION:1`, header ongewijzigd, dienst voltooit `INSTALLED:2`; derde boot: `SLOT:B:VERSION:2` |
| 7 | no-valid-slot | beide manifesten beschadigd: `BOOT:FAIL:NO_VALID_SLOT`, exit 0x11; geen kernelmarker |
| 8 | tampered-inbox | inbox met één handtekening of met `version` verlaagd onder de vloer: dienst `UPDATE:REJECTED:*`, geen schrijfactie (orakel: slot B leeg, header ongewijzigd) |

Hostorakel `tools/update_disk.py` leest de slotopslag onafhankelijk (Python,
zonder cryptografie: byte-vergelijking met de bekende v1/v2-images en
manifesten). Gate `tools/test_update.py`: hosttests (`update-core` met
vastgelegde testvectoren voor Ed25519 (RFC 8032) en ML-DSA-65 (vast seed-
vector uit de crate), manifestcanoniek, selectiebeleid, planner; `update-tool`
keygen/sign/pack roundtrip) en daarna de runs hierboven op één bevroren bron.

## 7. Wat B03 niet bewijst

Geen firmware-/Secure-Boot-verificatie van de loader zelf (B04), geen
sleutelrotatie-/intrekkingsbeleid in productie (ankers zijn TEST-ankers),
geen bescherming tegen een aanvaller met schrijftoegang tot OVMF_VARS, geen
updates van gebruikersprogramma's of pakketten (A02), geen netwerkdistributie
(N-spoor), geen fysieke schijf of stroomuitval (een emulator-kill is iets
anders).

## 8. Afwijkingen tijdens implementatie

- **Slotgrootte 2 MiB.** Een kernelimage met ingesloten ring-3-programma's is
  ca. 1,5 MiB (grotendeels geïnitialiseerde kerneldata); de slots zijn
  daarom 4096 sectoren en `image_length` ≤ 2 MiB. De tabel in §3 is
  bijgewerkt.
- **Geen aparte observer-app.** Het profiel `native-update` start alleen de
  update-dienst (één ring-3-proces). De drie onafhankelijke partijen van de
  acceptatie zijn de loader (verificatie bij de volgende boot), de dienst
  (verificatie vóór installatie) en het hostorakel `tools/update_disk.py`
  (bytes op schijf). Een app zonder blokrechten is al in M6e bewezen.
- **Draaiend slot door versie, niet door header.** De dienst bepaalt het
  slot waarnaar zij niet mag schrijven als het slot waarvan de geverifieerde
  manifestversie gelijk is aan de door de kernel gemelde eigen imageversie
  (startwoord 120); de header is alleen de volgorde-hint. Zonder zo'n slot
  weigert de dienst te installeren (foutcode -19). Daarmee kan een
  gemanipuleerde header de dienst nooit het actieve slot laten overschrijven
  (acceptatiegeval bad-signature).
- **Hervatting zichtbaar.** Vóór de digestcontrole kijkt de dienst of het
  doelslot al de eerste imagesector van deze bundel maar nog geen manifest
  draagt en meldt dat als `RESUMED` (startwoord 368). De gelabelde
  crashfixture `test-update-cut` snijdt alleen een eerste poging af (64
  voltooide niet-headerwrites), zodat de herboot de installatie kan afmaken.
- **Loaderstack.** ML-DSA-65-verificatie houdt ca. 200 KiB stackframes vast;
  de loader voert de slotselectie uit op een eigen 1 MiB-paginaallocatie
  (asm-stackwissel, efiapi) en de kernel geeft de dienst in dit profiel 512 KiB
  gebruikersstack.
- **Loadermarkers.** Naast `BOOT:SLOT:<X>:VERSION:<n>` en
  `BOOT:SLOT:<X>:REJECTED:<reden>` print de loader `BOOT:FLOOR:<n>` (gelezen
  vloer) en meldt een leeg slot als `REJECTED:FORMAT`. `BOOT:STORE:NONE`
  markeert het terugvalpad naar de ESP zonder slotopslag.
- **Twaalf stages in vier reeksen** in plaats van acht runs: de reeks
  install → boot-v2 → downgrade deelt slotopslag en OVMF_VARS; bad-signature,
  bad-digest en no-valid-slot zijn verse reeksen; cut → cut-resume →
  cut-final deelt de slotopslag; de inboxgevallen (handtekening, downgrade,
  vreemde sleutel) zijn elk een verse reeks. Het gate-script bouwt drie
  kernels (v1, v2, v1 met cutfixture), ondertekent met de TEST-ankers en
  één ad-hoc vreemd sleutelpaar, en controleert na iedere stage de schijf
  met het Python-orakel.
