# M6c — native documentnamen en afgeschermde catalogus

Datum: 28 oktober 2026. Status: huidig M6c-contract met aanwezige implementatie.
Oorspronkelijke basis: `bb22fdbe15919edc23baa158a46236195ce314b2` (afgeronde M6b).
Werkbranch: `codex/m6c-named-catalog`. Bewijsstatus en acceptatiegrenzen:
[M6C-ACCEPTANCE.md](../verification/M6C-ACCEPTANCE.md).
Bekende beperking: de native observerpanic is nog ongerepareerd, ook met
`milestone-m6c`. Het `Unknown`-gedrag is via host-ABI getest; gecontroleerde
end-to-end-afhandeling bij native WRITE-/FLUSH-fouten of time-outs is niet
gerealiseerd. Zie [Bekende native observerbeperking](#bekende-native-observerbeperking).

## Doel en grenzen

Een echte ring-3-dienst laat twee geïsoleerde native apps documenten met namen
aanmaken, opvragen, bijwerken, hernoemen, verwijderen en inventariseren. Dit
levert de eerste native catalogusinterface voor Materiaal; geen grafische shell.
Object-ID blijft identiteit en autorisatiebasis. Namen zijn labels, geen paden;
gelijke namen zijn toegestaan. Er is geen globale zoek- of directorydienst.

Behoud de geteste grenzen: vier objecten (0–3), twee per endpointdomein, maximaal
128 inhoudsbytes, 32 append-only records en uitsluitend sectoren 4–35. De naam
heeft maximaal 32 UTF-8-bytes. Dit zijn proefbudgetten, geen productlimieten.
De M6b-crate en het M6b-formaat blijven ongewijzigd en afzonderlijk bouwbaar.

## Invarianten

- Endpoint én door de kernel ingevulde sender-PID bepalen het domein. Appdata
  verleent geen autoriteit. LIST ziet uitsluitend aanwezige eigen objecten.
- Namen zijn niet leeg, geldige UTF-8 en bevatten geen NUL/besturingstekens,
  slash, backslash of bidi-override/isolate-tekens (U+202A–202E, U+2066–2069).
  `.` en `..` worden afgewezen. Geen normalisatie of hoofdletterconversie;
  toegestane labels worden bytegetrouw bewaard en vergeleken.
- PUT bewaart naam en inhoud samen. RENAME vereist een bestaand object en de
  actuele revisie en behoudt alle inhoudsbytes. DELETE bewaart de verhoogde
  revisie, maar wist naam/inhoud uit de levende toestand en catalogus.
- Voor mutaties: valideer/prepareer → WRITE-completion → FLUSH-completion →
  commit levende toestand → ACK. Bij onzekerheid sluit ook LIST met Unknown;
  geen automatische replay of gegevens uit mogelijk verouderde toestand.
- Vol is Full vóór I/O, zonder wrap/overschrijven. Recovery vereist een gehele
  geldige prefix met uitsluitend lege records erachter. Corruptie sluit af.
- LIST is één atomisch overzicht van de maximaal twee aanwezige objecten in
  één domein, in oplopende object-ID-volgorde. Geen inhoud, tombstonenamen,
  globale telling of globale journalsequence. De catalogusrevisie is de som
  van uitsluitend de twee eigen objectrevisies (inclusief tombstones).
- Catalogusantwoorden, ook op een leeg domein, verruimen geen rechten. Een
  later GET of mutatie wordt opnieuw geautoriseerd; revisies zijn geen grants.

## IPC v4 (224 payloadbytes in Message320)

De bestaande M6b-v3-padding wordt niet onder dezelfde versie hergebruikt.
TAG = `0x4d36434c4b000004`, versie = 4. GET=1, PUT=2, DELETE=3,
RENAME=4 en LIST=5. Statuscodes blijven gelijk aan M6b.

| Offset | Inhoud |
|---|---|
| 0 / 2 | u16 versie / operatie |
| 4 | vier nulbytes |
| 8 / 16 / 24 / 32 | u64 request-ID / service-epoch / object-ID / revisie |
| 40 / 44 | u32 inhoudslengte / i32 status |
| 48 / 52 | u32 naamlengte / u32 catalogusaantal |
| 56–63 | nul |
| 64–191 | inhoud, of bij LIST maximaal twee catalogusentries |
| 192–223 | naam bij PUT/RENAME-request en succesvolle GET-reply |

Alle ongebruikte bytes zijn nul. Constructoroverschrijding wordt een expliciet
ongeldig begrensd bericht, geen panic of stille truncatie. GET/DELETE bevatten
geen naam of inhoud. PUT vereist een naam en accepteert lege inhoud. RENAME
vereist een naam en geen meegegeven inhoud. Mutatie-ACKs bevatten alleen de
nieuwe revisie, geen naam/inhoud. NotFound mag uitsluitend de geautoriseerde
tombstonerevisie teruggeven; overige fouten hebben revisie nul en lege data.

LIST-request: object-ID exact het eigen domeinbegin (0 of 2), revisie/lengtes/
aantal nul. Cross-domain selectors geven Access; een eigen oneven selector
Invalid. LIST-reply echoot selector/request-ID/epoch, geeft de domeinrevisie,
aantal 0–2 en lengte `56 * aantal`. Een entry van 56 bytes: object-ID u64 op 0,
objectrevisie u64 op 8, inhoudslengte u32 op 16, naamlengte u32 op 20, naam op
24–55 met nulpadding. Rest van de payload is nul. De client valideert domein,
volgorde, unieke IDs, lengtes, naam, revisies en padding. Een lege catalogus
geeft Ok, aantal/lengte nul en de huidige eigen domeinrevisie.

## Journal v2

512 bytes, magic `NOSM6C02`, versie 2, headersize 64. Sequence/request-ID/
object-ID/objectrevisie blijven op 16/24/32/40; operatie u16 op 48,
inhoudslengte u16 op 50 en naamlengte u16 op 52. Overige headerbytes nul.
Inhoud op 64–191, naam op 192–223, rest tot 504 nul; diagnostische FNV64 op
504 zoals M6b. PUT bevat de volledige nieuwe naam/inhoud. RENAME-record bevat
nieuwe naam plus exact de bestaande inhoud; recovery controleert predecessor,
revisie én inhoudsbehoud. DELETE heeft lege naam/inhoud en vereist een bestaand
object. Alle overgangen verhogen objectrevisie en globale sequence met één.

Legacybeleid: een bezet M6b-v1-record wordt expliciet als niet-ondersteund/
Corrupt geweigerd vóór READY; er is geen automatische migratie of formattering.
De M6b-reader blijft beschikbaar voor de eerdere synthetische volumes. Deze
stap wijzigt geen echte gebruikersvolumes. Een expliciete migrator is later werk.

## Implementatiegrenzen

`services/catalog` is een nieuwe dependencyvrije `no_std`-crate `catalog-core`
met `forbid(unsafe_code)`. De native adapter gebruikt de bestaande scalar
runtime; geen nieuwe syscall, capabilitytype, blokrechten of hardwaredriver.
De kernel bevat uitsluitend de bestaande blokbroker en een expliciete
acceptatieobserver voor deze fixture, geen naam-/catalogusbeleid voor apps.
Native probe: `userspace/m6c_probe.rs`; selector: `milestone-m6c`.

## Bekende native observerbeperking

De `Unknown`-afhandeling is met de echte serviceadapter en een host-ABI-shim
gecontroleerd. In `kernel/src/m2/m3/m6c.rs` bewaart de acceptatieobserver bij
een voorbereide mutatie echter de verwachte `Ok`-reply. Na een native WRITE-
of FLUSH-fout of time-out kan de adapter `Unknown` aanbieden, waarna `sent()`
op die verwachting panikeert voordat de client het antwoord ontvangt. Dit
geldt ook voor `milestone-m6c`, niet alleen voor fouttestfeatures. Dit is een
geldige, nog ongerepareerde observerfout.

De huidige native integratie voldoet onder die devicefouten nog niet aan de
bedoelde gecontroleerde `Unknown`-respons en voortzetting. De ontwerpvereisten
blijven staan: geen commit zonder WRITE+FLUSH, geen automatische replay en
geen verdere geldige GET/LIST/mutaties bij onzekerheid. Het geteste
serviceadaptergedrag is geen bewijs dat het defecte native observerpad werkt.

De begrensde native acceptatie gebruikt de vastgelegde QEMU/device-fixture
zonder native devicefoutinjectie; `capture_m6c.py` weigert `M6C:OUTCOME`.
De 15 host-ABI-scenario's vervangen geen toets van de kernelobserver.
Herstel vraagt een afzonderlijke native wijziging met WRITE-/FLUSH-fout- en
time-outinjectie, controle van het werkelijk ontvangen clientantwoord en
nieuwe brongebonden native verificatie.

## Acceptatie

1. Hosttests voor wire/UTF-8/canonicalisatie, catalogusfiltering, CAS, tombstones,
   hernoemen, vol, corruptie, legacyweigering en herstel. no_std-build/fmt/Clippy.
2. Native normale flow per domein: lege catalogus/GET; maak twee documenten;
   LIST; GET; RENAME eerste; weiger stale rename; GET/LIST; DELETE tweede;
   LIST/GET tombstone; PUT eerste; GET/LIST. Eindsequence 10: even objecten
   live/revisie 3, oneven objecten tombstone/revisie 2. Iedere app doet 16 RPCs.
3. Koude tweede VM: exacte namen/inhoud/revisies/catalogus, geen diskmutatie.
4. Negatieve native verzoeken: domeingrens voor GET/PUT/DELETE/RENAME/LIST,
   vreemde versie/epoch, ongeldige UTF-8, naamlimiet, padding, ongeldige LIST,
   stale revisie en onjuiste/gesloten capabilities. Geen I/O of metadata-lek.
5. Vol: 32 writes, extra PUT/RENAME geweigerd vóór I/O; laatste naam blijft staan.
6. Twee echte externe kills tijdens een RENAME: vóór WRITE en na FLUSH vóór ACK.
   Alleen app A schrijft in deze fixture: eerst create (sequence 1), dan rename.
   Nieuwe VM ziet respectievelijk oude naam/revisie 1 of volledige nieuwe naam/
   revisie 2 met exact dezelfde inhoud. Onafhankelijk Python-schijforakel.
7. Corruptie en geldig legacy-v1-volume weigeren vóór READY zonder diskmutatie.
8. M6b-regressie en relevante build-/bootcontroles. Bronnen, binaries, exacte
   commando's, ruwe uitvoer, exitcodes, imagescope en hashes blijven gebonden.
   Nieuwe hostgroepen leggen de hashes van stdout, commandometadata en stderr
   direct in het gate-record vast. Capture vergelijkt de bronbestanden én de
   gekopieerde archiefbytes met deze vaste map; ontbrekende binding wordt
   afgewezen. De historische suite vóór deze aanscherping heeft die binding
   niet; afbakening en afzonderlijke hosttests staan in
   `../verification/m6c-review/host-log-binding/README.md`.

Een emulatorproceskill is geen fysieke stroomuitval. FNV is geen authenticatie;
DELETE wist historische diskbytes niet. Geen encryptie, native NAS, grafische
desktop, formeel veiligheidsbewijs of productiegeschiktheid wordt geclaimd.

## Bronnen en versiebeleid

- Nieuw OS. (2026, 28 oktober). *M6B-OBJECT-STORAGE.md; PROJECT-STATE.md*
  (commit bb22fdbe15919edc23baa158a46236195ce314b2). Bestaande native basis.
- The Rust Project. (z.d.). *x86_64-unknown-none*. The rustc book.
  https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
  (geraadpleegd 28 oktober 2026). Geen std/default allocator; hosttests en
  echte gastuitvoering zijn afzonderlijke controles. Toolchain blijft 1.90.0.

De nieuwe formaten en proefbudgetten hierboven zijn expliciete projectkeuzes.

