# S09 — Herstelkopie- en exportarchief (formaat v1)

**Status:** hostgeïmplementeerd in `services/volume/src/archive.rs`, getest in
`services/volume/tests/archive.rs`. Nog geen native portaal, geen gebruikerstaak,
geen fysieke schijf. S09 blijft open.

## Doel en invariant

Een gebruiker kan van één documentdomein een **onafhankelijke kopie** maken die
zonder bronvolume, bronservice of brondisk controleerbaar en terug te zetten is.

- **Export** is een consistente momentopname: er wordt geen archief afgesloten als het domein
  tijdens de export wijzigt (`Stale`), als het doel vol raakt of een I/O-fout
  geeft (fout van de sink), of als gelezen inhoud afwijkt van de eigen
  inhoudssom van de dienst (`Corrupt`). Na een fout is de uitvoer ongeldig, en
  `verify` weigert die.
- **Verify** heeft geen bijwerkingen en accepteert alleen een volledig, canoniek archief.
- **Restore** overschrijft nooit iets. Een bezette doelslot of een al gebruikte naam
  wordt als conflict gemeld en blijft onaangeroerd. Een doelcollectie met
  een andere naam wordt nooit hernoemd; het object komt dan buiten collectie te staan
  (`uncollected`). Elk object wordt pas gecommit nadat zijn record en inhoud
  zijn gecontroleerd. Omdat de trailer pas na het laatste object komt, moet een
  portaal **eerst `verify` op dezelfde bytes** uitvoeren. Zonder verify
  kunnen bij afkapping alleen de eerdere, volledig gecontroleerde objecten zijn
  teruggezet; de `Report` noemt die exact.
- Een write met uitkomst `Unknown` telt alleen als teruggezet als `stat` precies
  de verwachte revisie, naam, lengte en inhoudssom toont; anders stopt restore met
  `Unknown`. Faalt het zetten van tags, soort of collectie na een geslaagde write,
  dan staat het object in `restored` én in `bare`, en stopt restore met die fout.
- Restore veronderstelt dat er tijdens de restore één schrijver op het doeldomein is. Een gelijktijdige
  slotwijziging breekt af met `Stale`; een gelijktijdig aangemaakte gelijke naam
  wordt niet gedetecteerd (de volume dwingt geen unieke namen af).
- Een verwijderd doelslot (tombstone) wordt gevuld. De bewaarde historie van dat
  slot blijft staan en kan later via lokale historie worden teruggehaald; er gaat
  geen data verloren, maar lokaal undo van dat slot toont dan oudere, andere inhoud.

## Bytes (little-endian)

| Deel | Grootte | Inhoud |
|---|---|---|
| Header | 64 | `NOSBAK\0\x01`, versie u16=1, 2×0, aantal objecten u8 (≤8), aantal collecties u8 (≤4), 2×0, brongeneratie u64, totale inhoud u64, 24×0, FNV-64 over bytes 0..56 |
| Collectierecord | 48 | index u8 (1..4, strikt oplopend), naamlengte u8, 6×0, naam[32] (opvulling 0), FNV-64 over 0..40 |
| Objectrecord | 96 | slot u8 (0..7, strikt oplopend), naamlengte u8, collectie u8 (0 of aanwezig), 0, kind u16, 2×0, tags u32, lengte u32 (≤64 KiB), bronrevisie u64 (≠0), created u64, modified u64, inhouds-FNV-64, naam[32] (opvulling 0), 8×0, FNV-64 over 0..88 |
| Inhoud | lengte | ruwe documentbytes direct na het eigen record |
| Trailer | 16 | `NOSBAKZ\0`, FNV-64 over alle voorgaande bytes |

Na de trailer moet de stroom eindigen. Namen volgen `valid_name` (UTF-8,
1–32 bytes, geen `.`/`..`). Dubbele namen zijn toegestaan, net als in de volume
zelf. Een bron die meer
bytes meldt dan er gevraagd zijn, levert `Protocol` op; elke andere afwijking levert `Corrupt` op.

## Wat bewust niet in v1 zit

- **Geen authenticiteit of versleuteling.** FNV-64 vangt alleen toevallige
  corruptie, afkapping en torn transfers op, geen opzettelijke manipulatie. Het archief
  is plaintext persoonsgegevens. Het mag alleen naar een door de gebruiker gekozen
  eigen doel (LOCAL-PRIVACY-CONTRACT: eigen offline herstelkopie). Voor NAS of een
  provider is client-side versleuteling nodig (B02, P01, ENCRYPTED-OBJECT-STORAGE);
  dat staat hier nog open.
- **Alleen huidige versies.** Lokale historie is geen back-up en wordt niet
  meegenomen. `created`/`modified` worden bewaard en zijn exporteerbaar,
  maar bij restore kent de dienst nieuwe tijden toe.
- **Eén domein per archief**, binnen de huidige volumelimieten (8 objecten,
  64 KiB per object, 4 collecties).

## Bewijs en open punten

Hostbewijs (musl-target, Rust 1.90.0, toegang via de echte volume-client en -service):
round-trip na verlies van de bron en koude herstart van het doel; elke prefix, elke
bitflip (bits 0x01 en 0x80 op iedere byte) en een extra byte worden geweigerd;
een vol doel op 40 grenzen; gelijktijdige wijziging van een gelist object en
van een nieuw object; inhoud die de dienstsom tegenspreekt; conflicten zonder
overschrijven; quota tijdens restore; ongeldig domein. Mutatieproeven op de
naamconflict-, EOF-, generatie-, inhoudssom-, collectie- en protocolcontrole
laten elk een test falen. Opnieuw verzegelde, niet-canonieke records (20 gevallen)
worden geweigerd; `Unknown` (wel/niet geland), mislukte META, een tombstone-slot en
dubbele bronnamen zijn getest. Niet door een test gedekt: de inhoudssom-
vergelijking bij `Unknown` en de lengtecontrole list/stat bij export (alleen
relevant als een andere schrijver of de dienst zelf afwijkt).

Open voor S09/FR-O06: native back-upportaal met een door de gebruiker gekozen
doelvolume en capability, native QEMU-proef met een tweede virtuele schijf,
gebruikersexport naar gangbare formaten (bijv. platte tekst/afbeelding per
document), versleutelde variant voor NAS/provider, en een gebruikerstaak.
