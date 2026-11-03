# S06 — collecties, metadata, revisiegeschiedenis en transacties (volume v2)

Datum: 29 oktober 2026. Basis: M6e (`NOSVOL01`, IPC v5, ADR-040). Besluit:
ADR-041. Bewijsstatus: [S06-ACCEPTANCE](../verification/S06-ACCEPTANCE.md).

## Doel en grenzen

M6e bewaart grote documenten crashconsistent, maar kent alleen een naam en de
actuele inhoud. Materiaal (PRODUCT-SPEC §2, §6) vraagt meer van de opslaglaag:
documenten in collecties terugvinden, type en labels bewaren, een oudere versie
bekijken en als nieuwe revisie terugzetten, een verwijderd document terughalen
en meerdere documenten in één consistente stap verplaatsen. Volume v2 levert
die opslagfuncties native, in dezelfde ring-3-dienst en met hetzelfde
copy-on-write-commitprotocol.

| Materiaal-taak (PRODUCT-SPEC) | Opslagfunctie v2 |
|---|---|
| Terugvinden op titel/type/collectie | LIST met collectiefilter; STAT geeft naam, `kind` en `tags` |
| Collecties 'Vandaag', 'Huis', 'Archief' | COLLECTION: maken, hernoemen, verwijderen (alleen leeg); COLLECTION_STAT: naam, aanwezigheid en revisie lezen |
| Objectgeschiedenis openen | STAT met historie; READ met versieselector |
| Herstel versie als nieuwe revisie (P2-05) | RESTORE: oude inhoud wordt een nieuwe revisie, de rest van de historie blijft |
| Verwijderen ongedaan maken | DELETE bewaart de laatste versie als historie; RESTORE haalt die terug |
| Meerdere documenten tegelijk verplaatsen/opruimen | TRANSACT: alles-of-niets over maximaal 4 objecten |
| Ruimte vrijmaken | PURGE: historie definitief verwijderen |

Grens: dit is de opslaglaag. De Materiaal-interface draait nog niet native
(G-/U-criteria); zoeken op titel gebeurt in de app over LIST-resultaten.

Proefbudget (projectkeuze, geen productlimiet): 2 historische versies per
object, 4 collecties per domein, maximaal 4 objecten per transactie. Verder
gelden de M6e-grenzen: 16 objecten, 64 KiB per versie, 64 chunks, quota 40.

Niet in deze stap: zoekindex, vrije metadata (sleutel/waarde), geneste
collecties, gedeelde collecties tussen domeinen, onbeperkte historie, B02
(authenticatie/encryptie), S09 (back-up/export) en migratie van v1-volumes.

## Schijfformaat v2 (`NOSVOL02`)

Gescopete sectoren `[4, 584)`. Iedere tabelkopie telt 34 sectoren
(kopie A 0–33, kopie B 34–67); inhoudschunks beginnen bij sector 68.

- Sector 0: header (magic, versie 2, geometrie, generatie, request-ID,
  `MAX_CHUNKS`, quota, historiediepte, collecties per domein) en op offset 40
  een klok per domein (u64): het aantal commits van dat domein.
- Sector 1: 8 collectierecords van 56 bytes (revisie, request-ID, naamlengte,
  aanwezig, naam ≤ 32 bytes). Index = domein × 4 + collectie − 1.
- Per object twee sectoren: kop (revisie, request-ID, aangemaakt/gewijzigd als
  stand van de eigen domeinklok, `tags` u32, `kind` u16, aanwezig, collectie, historiediepte en de
  actuele versie) en historie (2 versies van 216 bytes).
- Een versie: versierevisie, lengte, chunklijst met per-chunk FNV,
  inhoudschecksum en naam. Ongebruikte velden zijn canoniek nul / `0xFFFF`.
- Alle sectoren dragen dezelfde trailer als v1: generatie, pogingdigest over
  de payload van alle 34 sectoren en een sectorchecksum.

## Toelating van lege en vreemde volumes

Een lege tabel is niet genoeg om een volume als nieuw te behandelen: een
vreemd bestandssysteem kan de NOSVOL02-tabelregio niet gebruiken en elders
wel gegevens bevatten. Als beide tabelkopieën leeg zijn en er geen geldige
formaatmarker staat, leest herstel daarom ook alle sectoren van de
experimentele volumegrens `[0, 580)`. Alleen een volledig nulgevuld volume
wordt dan als generatie 0 geaccepteerd. Niet-nuldata zonder herkenbare
NOSVOL02-toestand geeft `Corrupt`; de dienst formatteert of schrijft niets.

Dit omvat ook beide tabelkopieën: willekeurige niet-nulbytes in alleen slot 1
zijn geen bewijs van een onderbroken eigen tabelwrite. Zonder geldige marker
of herstelde tabel moet de volledige tabelregio eveneens nul zijn. Met een
geldige, geflushte marker blijft een onderbroken eerste tabel herstelbaar.

Vóór de eerste upload accepteert `BEGIN` eerst een geldig lege of reeds
gemarkeerde volume-invoer en schrijft sector 0 als `NOSFMT02`-formaatmarker,
gevolgd door `FLUSH`, vóór de eerste inhoudssector. Andere eerste mutaties
zetten dezelfde marker vóór hun tabelcommit. Bij een onderbroken eerste
upload kan herstel zo de marker herkennen en de logische lege generatie
teruggeven; ongecommitte inhoud blijft weesdata en wordt niet als bestand
gepresenteerd. Zodra generatie 2 naar tabelslot 0 wordt geschreven, vervangt
die tabel de marker. Een ongeldige of beschadigde marker is geen bewijs van
leegte.

Een direct geconstrueerde service vertrouwt ook een meegegeven generatie niet:
vóór de eerste mutatie vergelijkt zij de herstelde schijftoestand met de
meegegeven tabel. Alleen `Service::recover` mag de toelatingscontrole vooraf
als voltooid markeren.

De marker bevat de formaatversie en geometrie en heeft een checksum tegen
toevallige beschadiging. Dit is geen authenticatie of bescherming tegen een
opzettelijk nagemaakte marker. De native buitenlandse-mediafixture bevat
alleen een Btrfs-achtige magicwaarde op de bekende 64-KiB-offset; zij is geen
geldig Btrfs-volume en bewijst geen volledige detectie van andere
bestandssystemen.

## Invarianten (aanvulling op M6e)

1. **Historie is strikt ouder.** Versierevisies dalen: actueel > historie[0] >
   historie[1]. Een verwijderd object heeft geen actuele versie, collectie,
   `tags` of `kind`, maar mag historie houden.
2. **Gedeelde chunks alleen ongewijzigd.** Versies van één object mogen een
   chunk alleen delen op dezelfde positie met dezelfde checksum (RESTORE
   verwijst opnieuw, herschrijft niet). Objecten delen nooit chunks.
3. **Quota telt historie mee.** Een chunk telt eenmaal per object; behouden
   historie houdt ruimte vast tot PURGE. `Quota`/`Full` vóór iedere I/O.
4. **Collecties zijn domeingebonden.** Een object kan alleen in een bestaande
   collectie van het eigen domein staan; een niet-lege collectie verwijderen
   geeft `Busy`. Domein komt uitsluitend van het kernelgebonden endpoint.
5. **Transacties zijn alles-of-niets.** TRANSACT controleert iedere
   voorwaarde (bestaan, verwachte revisie, collectie, onderscheiden objecten)
   vóór één tabelwrite. Eén transactie = één generatie. Een crash tijdens de
   tabelwrite laat de vorige generatie volledig staan.
6. **Iedere commit is herleidbaar.** De request-ID van de header staat bij ten
   minste één gewijzigd object of collectie; COLLECTION_STAT en STAT geven die
   request-ID terug, zodat een `Unknown` uitkomst door lezen op te lossen is.
   Definitief is dat antwoord pas van een schrijfbare instantie, na haar flush
   uit invariant 9. Een read-only instantie flusht niets: zij kan een poging
   zien die nog alleen in de schijfcache staat en na stroomverlies verdwijnt.
   Haar antwoord is dus voorlopig.
7. **Domeinen lezen elkaars activiteit niet af.** Iedere commit verhoogt
   precies één domeinklok; de klokken tellen op tot de generatie. Revisies en
   tijdstempels van een domein overschrijden nooit de eigen klok. De
   leesantwoorden (STAT, LIST, COLLECTION_STAT, READ) en de revisies,
   stempels en tellers in mutatieantwoorden bevatten alleen toestand van het
   eigen domein. Twee kanalen tussen domeinen blijven bewust bestaan
   (restrisico):
   - `Full` hangt af van de gedeelde volumeruimte. De quota's samen
     (2 × 40 chunks) zijn groter dan het volume (64), dus of BEGIN `Full`
     geeft, hangt ook af van wat het andere domein vasthoudt.
   - Mislukt de tabelwrite van één domein, dan sluit de instantie en krijgt
     ook het andere domein `Unknown` tot een nieuwe instantie start.

   Wie volledige scheiding wil, heeft per domein niet-overboekte ruimte en
   foutisolatie per domein nodig; dat valt buiten S06.
8. **RESTORE verliest geen historie.** De teruggezette versie verlaat de
   historie en de vervangen actuele versie neemt haar plaats in.
9. **Herstel is duurzaam vóór nieuwe writes.** Een schrijfbare instantie
   flusht na recovery vóór het eerste verzoek. Anders kan een hergebruikte
   chunk de terugvalkopie beschadigen als de gekozen kopie nog alleen in de
   schijfcache stond. Read-only herstel schrijft en flusht niets.

Restrisico's uit M6e blijven gelden (één onleesbare sector in de nieuwste kopie
geeft terugval; FNV is geen authenticatie).

## IPC v6 (TAG `0x4d36454c4b000006`)

Nieuwe operaties: RESTORE (selector 1–2), PURGE, META (`tags`/`kind`/collectie
met revisie-CAS), COLLECTION (maken/hernoemen/verwijderen met CAS), TRANSACT
(1–4 sub-operaties META/DELETE/PURGE van 24 bytes op objectindex binnen het
eigen domein) en COLLECTION_STAT (op 15, alleen lezen: revisie, aanwezigheid,
naam en request-ID van één collectie van het eigen domein). READ krijgt een versieselector; LIST een filter (0 = alles,
1 = zonder collectie, c + 1 = collectie c); STAT geeft metadata en per
historische versie revisie, lengte en inhoudschecksum. Alle ongebruikte bytes
moeten nul zijn; de client valideert ieder antwoord strikt.

## Acceptatie

1. Hosttests `volume-core`: v2-formaat, historie, collecties, transacties,
   negatieve gevallen en de uitputtende crashsimulatie (nu ook over
   transacties, collecties, RESTORE en PURGE), plus de test tegen gemengde
   schrijfpogingen.
2. Orakeltests (`tests/test_volume_disk.py`): v2-decodering, historie- en
   chunkregels, collectielidmaatschap, digest en de recoveryregels.
3. Native gate `tools/test_volume.py`, negen echte QEMU-invocaties op één
   bevroren bron: normal → koude read-only recovery; cut-data, cut-table en
   cut-txn (kill midden in de tabelwrite van een transactie over twee
   objecten), elk gevolgd door koude recovery; corrupte image geweigerd. Voor
   cut-table en cut-txn eist de runner een echt gescheurde poging in de andere
   kopie (een niet-lege, onvolledige prefix van g+1-sectoren boven de vorige
   kopie); voor cut-data moet die kopie onaangeroerd zijn.
4. Normal bewijst native: collecties maken en dubbel maken (`Stale`), META
   met ongeldige collectie (`Range`), een transactie met één verouderde
   voorwaarde verandert niets (`Stale`), een geldige transactie verplaatst twee
   documenten in één commit, LIST-filters, `Busy` bij een niet-lege collectie,
   historie lezen en terugzetten als nieuwe revisie, verwijderen ongedaan
   maken, `Quota` door behouden historie en PURGE om ruimte vrij te maken,
   en collecties lezen met COLLECTION_STAT. De koude recovery verifieert de
   volledige eindtoestand inclusief historie, collectienamen en de
   domeinklokken (57/27), en weigert iedere mutatie, ook RESTORE en TRANSACT
   (`Access`).

Een emulator-kill is geen fysieke stroomuitval. Er is geen productie-,
encryptie- of fysieke-schijfclaim.
