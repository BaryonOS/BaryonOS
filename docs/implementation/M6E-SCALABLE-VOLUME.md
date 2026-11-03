# M6e — schaalbare documentopslag met ruimtehergebruik (S04/S05)

Datum: 28 oktober 2026. Basis: `main` `6e12dcb` (20/100). Werkbranch
`ccr-d3fe1610-gl4r8s`. Bewijsstatus: [M6E-ACCEPTANCE](../verification/M6E-ACCEPTANCE.md).

> **Opgevolgd door volume v2 (S06, ADR-041).** Formaat `NOSVOL01` en IPC v5
> hieronder beschrijven de bron waarop S04/S05 zijn geaccepteerd. Het profiel
> `native-volume` gebruikt nu `NOSVOL02`/IPC v6 met collecties, metadata,
> historie en transacties: [S06-COLLECTIONS-HISTORY](S06-COLLECTIONS-HISTORY.md).
> De commitvolgorde, invarianten 1–7 en de crash-/recoveryregels gelden
> ongewijzigd; de S06-gate herhaalt de S04/S05-scenario's op v2.

## Doel en grenzen

M6c bewaarde vier objecten van hoogstens 128 bytes in een append-only journal
van 32 records; daarna was het volume blijvend vol. M6e vervangt die proefgrens
door een **copy-on-write volume** met grote documenten, ruimtehergebruik zonder
recordlimiet, quota per domein en crashconsistente vrijgave. M6c/M6d blijven
ongewijzigd en afzonderlijk bouwbaar; er is geen automatische migratie.

Proefbudget (projectkeuze, geen productlimiet): 16 objecten (8 per domein),
documenten tot 64 KiB, 64 chunks van 4 KiB (256 KiB inhoud), quota 40 chunks per
domein. De quota's overcommitteren het volume bewust (2 × 40 > 64), zodat zowel
`Quota` (eigen domein) als `Full` (gedeeld volume) reëel bereikbaar zijn.

Niet in deze stap: encryptie/authenticatie (B02), back-up/export (S09),
collecties/metadata/revisiehistorie (S06), fysieke schijven, groter geheugen-
of meervoudige volumes, en automatische migratie van M6c-volumes.

## Architectuur

| Laag | Onderdeel | Autoriteit |
|---|---|---|
| Ring 3 | `services/volume` (`volume-core`, `no_std`, `forbid(unsafe_code)`) | Al het formaat-, domein-, quota- en revisiebeleid |
| Ring 3 | `sdk/examples/volume-docs` `volume-service` | Eén gescopete blokcapability + twee kernelgebonden ingress-endpoints |
| Ring 3 | `volume-app` (twee instanties) | Alleen een send/reply-endpointpaar; geen blokrecht |
| Kernel | `kernel/src/m2/m3/volume.rs` (`native-volume`) | Generieke blokbroker: scope, rechten, tickets, begrensde kopieën |
| Host | `tools/volume_disk.py` | Onafhankelijk Python-schijforakel (eigen implementatie) |

De kernel kent geen objecten, namen, revisies, domeinen of het volumeformaat.
Hij telt alleen sectoren/publicaties en zet testbarrières (zonder autoriteit).
De SDK-runtime kreeg een kleine blok-API (`block_read/write/flush/wait`) plus een
pure, host-geteste validatie van completion-enveloppen.

## Schijfformaat v1

Gescopete sectoren `[4, 534)` van een 1 MiB-image. Sectoren daarbuiten bevatten
canaries; het orakel weigert iedere wijziging daarvan.

- Tabelkopie A: sectoren 0–8; kopie B: 9–17. Generatie `g` staat altijd in
  kopie `g % 2`. Sector 0 = header (magic `NOSVOL01`, versie, geometrie,
  generatie, request-ID van de commit, quota); sectoren 1–8 = twee entries van
  240 bytes.
- Iedere tabelsector eindigt met `u64 generatie`, `u64 pogingdigest` (FNV-64
  over de payload, bytes 0–487, van alle negen sectoren van die kopie) en een
  FNV-64 over slot, index en de eerste 504 bytes. Zo is een gescheurde of
  verplaatste sector per sector zichtbaar, en kunnen sectoren van twee
  verschillende schrijfpogingen voor dezelfde generatie nooit samen een
  geldige kopie vormen (bevinding uit de CodeRabbit-review van PR #47).
- Entry: revisie, request-ID van de laatste mutatie, lengte, aanwezig, naam
  (max. 32 UTF-8-bytes, M6c-naamregels), chunklijst (≤ 16) met per-chunk FNV en
  een inhoudschecksum. Tombstones houden revisie + request-ID. Alle ongebruikte
  velden zijn canoniek nul / `0xFFFF`.
- Inhoud: chunk `c` = sectoren `18 + 8c … 18 + 8c + 7`. De laatste sector is
  met nullen opgevuld; het orakel en de dienst controleren die padding.

Een volledig nulvolume is de lege generatie 0; er wordt niets geformatteerd.

Dit is de historische v1-regel. Voor de huidige `NOSVOL02`-dienst is impliciete
nulvolume-admissie verwijderd: alleen expliciet vooraf geformatteerde,
volledig geldige lege metadata wordt toegelaten. Zie <!-- coderabbit-ai-generated:v1 --><!-- coderabbit-ai-generated-section:v1:2941fcfe7d64441664b4df6848d0ea457481b12ec613622a2b7e228bcd269c2b -->ADR-064<!-- /coderabbit-ai-generated:v1 --> en
[het v2-toelatingscontract](S06-COLLECTIONS-HISTORY.md#invarianten-aanvulling-op-m6e).

## Duurzaamheidsvolgorde en invarianten

1. Nieuwe inhoud gaat **uitsluitend** naar chunks die de duurzame tabel niet
   refereert en die geen andere sessie gereserveerd heeft.
2. Commit: laatste inhoudssector → FLUSH → volgende tabelkopie in het **oudere**
   slot → FLUSH → live staat → ACK. De slot met de actuele generatie wordt nooit
   overschreven.
3. Een fout vóór de tabelwrite is een **zeker** afbreken (`Io`): de duurzame
   staat is onveranderd en de dienst blijft bruikbaar. Een fout tijdens of na de
   tabelwrite is `Unknown`: de instantie weigert daarna iedere geldige aanvraag
   (eerst validatie, dan `Unknown`) en herhaalt de write nooit.
4. Herstel kiest de nieuwste volledig geldige kopie, maar alleen als de andere
   kopie de vorige generatie is of een onderbroken poging tot de volgende
   (per sector: generatie `g−1`, `g+1`, nul alleen bij `g = 1`, of gescheurd).
   Een kopie waarvan alle sectoren geldig verzegeld zijn maar waarvan de inhoud
   niet canoniek is, is corruptie en stopt herstel (geen stille terugval).
5. Ruimte van overschreven of verwijderde objecten komt vrij zodra de volgende
   tabel duurzaam is; een volgende commit hergebruikt die (laagste vrije chunk
   eerst, deterministisch). Er is geen journal, dus geen recordlimiet en geen
   aparte compactiestap.
6. Quota tellen **vastgelegde plus gestagede** chunks per domein; een
   overschrijving vraagt dus tijdelijk ruimte voor oud + nieuw (copy-on-write).
7. Inhoud wordt bij lezen per chunk tegen de checksum geverifieerd. Corruptie
   geeft `Corrupt`, nooit stille afwijkende bytes.

**Bekend restrisico** (vastgepind in een test): één onleesbare sector in de
nieuwste tabelkopie is niet te onderscheiden van een onderbroken, niet-geflushte
herschrijving van die slot. Herstel valt dan terug naar de vorige generatie
(verlies van de laatste commit); inhoud blijft per chunk gecontroleerd. FNV is
geen authenticatie; een aanvaller met schijftoegang kan consistente wijzigingen
maken. Dat valt onder B02.

## IPC v5 (224 payloadbytes in Message320)

TAG `0x4d36454c4b000005`, versie 5. Header: versie/operatie, request-ID,
epoch, object, revisie, lengte, status, naamlengte, count, aux; data 64–223
(160 bytes). Operaties: STAT, BEGIN, APPEND, COMMIT, ABORT, READ, DELETE,
RENAME, LIST (gepagineerd, 3 entries per pagina met domeingeneratie).

- Streaming upload: `BEGIN(object, verwachte revisie, totale lengte, naam)` →
  `APPEND(offset == ontvangen, ≤ 160 bytes)` … → `COMMIT`. Eén sessie per domein;
  een nieuwe BEGIN uit hetzelfde kernelgebonden domein vervangt een verlaten
  sessie (bijv. na herstart van de app).
- `READ(object, actuele revisie, offset, max ≤ 160)`; nooit over een
  chunkgrens; een oudere revisie geeft `Stale`.
- Domein komt uitsluitend van het endpoint. Een object van het andere domein
  geeft `Access`, een object ≥ 16 `Range`. Alle ongebruikte bytes moeten nul
  zijn; afwijkingen geven `Invalid` zonder I/O.
- De client valideert ieder antwoord strikt (echo, padding, per-operatie
  vorm) en behandelt afwijkingen als `Protocol`.

## Acceptatie

1. Hosttests `volume-core` (17): grote documenten, hergebruik (60 commits,
   > 4 × capaciteit gealloceerd), Quota/Full vóór I/O, domeinisolatie,
   malformed requests, streamingregels, tombstones, read-only, corruptie,
   zekere I/O-fouten, `Unknown` bij iedere tabelwritepositie, en een
   **uitputtende crashsimulatie** over 10 gemengde stappen: iedere muterende
   diskoperatie onderbroken, met prefix-, gescheurde, herordende en
   suffixpersistentie van de schrijfcache. Mutatietests bevestigden dat het
   weglaten van de flush vóór de tabelwrite en schrijven naar de actieve slot
   worden gevangen.
2. Orakeltests (`tests/test_volume_disk.py`, 8) en SDK-completiontests.
3. Native gate `tools/test_volume.py`, zeven echte QEMU-invocaties op één
   bevroren bron: normaal → koude read-only recovery van exact dezelfde image;
   kill midden in inhoudswrites → koude recovery; kill midden in de tabelwrite
   → koude recovery; corrupte image geweigerd vóór READY.
4. Iedere geslaagde run vereist: exacte markervolgorde, raw exit 33 (of een
   echte hostkill na de cutmarker), image-hashbinding, onveranderde image bij
   read-only, onaangeroerde canaries buiten de scope, en gelijke
   domeindigests tussen guestrapport en onafhankelijk orakel.

Een emulator-kill is geen fysieke stroomuitval; FNV is geen authenticatie.
Er wordt geen productie-, encryptie- of fysieke-schijfclaim gedaan.
