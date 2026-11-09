# G09 — gerichte review van de G0-capture

## Bron en uitkomst

Gereviewde B07-commit: `764cc9f84291ed3ad7b0ea3de65debcbf1cc7da6`, gebaseerd op
`3e41bbf93822f7044c6020259f4a752925e1966d`. Alleen de nieuwe capture, de gerichte
capturetests en de direct hergebruikte G0-frame-/bronhelpers zijn gelezen.
`source.json` en `checks.json` bevatten de hashes van de onveranderlijke
Git-snapshot. De implementatiewerkbomen en bestaande native artifacts zijn niet
gewijzigd. Alle nieuwe uitvoering gebruikt tijdelijke synthetische hostfixtures;
er is geen VM gestart.

**De ene concrete bevinding, G09-01, is inmiddels hersteld en onafhankelijk
hercontroleerd op B09-commit `030790820688f366abe5ad10c608f83655b0d81f`.**
Oorspronkelijk kon de outputindex een onverwacht regulier bestand opnemen dat
vóór de indexopbouw in de tijdelijke stagingmap verscheen. Hieronder staat het
oorspronkelijke bewijs; de slotsectie en `b09-fix-check.json` leggen de reparatie
vast. De overige genoemde grenzen vertoonden bij deze gerichte review geen
aanvullende blokkade.

## G09-01: verwachte outputset afgeleid van de map zelf

In `capture()` maakt `captured["files"]` zijn naamset met `staged.iterdir()`.
De laatste controle vergelijkt vervolgens met diezelfde afgeleide set. Daardoor
wordt een onverwacht regulier bestand dat al vóór deze indexopbouw aanwezig is,
als verwachte output behandeld.

De afzonderlijke proef `extra_staged_file()` in `review_boundaries.py` gebruikt
uitsluitend de bestaande synthetische B07-fixture. Zij schrijft bij de eerste
PNG-conversie `unexpected.bin` in de eigen tijdelijke stagingmap en laat de
oorspronkelijke encoder verder werken. De huidige capture publiceert vervolgens
een archief met **13** bestanden en `passed: true`, inclusief de extra bytes en
hun hash in `capture.json`. De beloofde vaste inhoud is **12** bestanden: vijf
tekst-/JSON-inputs, vijf PNG's, `image-roundtrip.json` en `capture.json`.

De kleinste oplossing is een expliciete vaste outputnaamset vóór de indexopbouw
en opnieuw vlak vóór publicatie, met een negatieve fixture voor een vroeg
toegevoegd regulier bestand. Dit is een afgebakende archiefgrens; de proef geeft
geen bewijs dat een andere gebruiker toegang tot de private stagingmap heeft,
en schrijft de onopgeloste eerdere G0-serialwijziging niet aan een oorzaak toe.

## Overige beoordeelde grenzen

| Grens | Resultaat van bronreview en gerichte controles |
| --- | --- |
| Input-allowlist en paden | Exacte artifactnamen en toegestane submappen; ontbrekende en extra artifacts worden afgewezen. Canonieke roots, `O_NOFOLLOW` per geopende component, reguliere bestanden met één link en begrensde omvang. |
| Bewijsbinding | Actuele source-v3, graphics-g0-manifest, werkelijk gelezen kernel-/loaderbytes, complete artifacthashes en QEMU-entry worden gekoppeld. Het commit-ID is bewust een formaatcontrole; inhoudsbinding volgt uit bronfingerprint en manifest. |
| Serial en QMP | Opnieuw gelezen serial moet exact gelijk zijn aan de vastgelegde stdout, met volledige native volgorde en eindmarkers. De elf QMP-request/responseparen moeten bij dezelfde toetsen, IDs en eigen screenshotpaden horen. Een afgebroken serial wordt niet uit het rapport gerepareerd. |
| Pixels en ROM | Bestaande framechecks worden opnieuw uitgevoerd, inclusief begrensde wijzigingen en exacte backspace-terugkeer. Een geclaimde ROM-pin moet bij de echte ROM-hash horen. |
| Laatste inputcontrole | Bytes én bestandsidentiteiten worden opnieuw gelezen; bron en QEMU-entry worden nogmaals gecontroleerd. De aanvullende proef vervangt serial door identieke bytes met een andere inode: correct afgewezen, geen archief gepubliceerd. |
| PNG | De begrensde RGB8/filter-0-reader controleert chunkvolgorde, CRC, dimensies, volledige zlib-eindecondities en exacte gedecomprimeerde lengte. De originele PPM-header plus teruggelezen RGB moet bytegelijk zijn aan het origineel. |
| Publicatie | Zelfde-parent staging en Linux `renameat2(RENAME_NOREPLACE)`; geen overschrijvende fallback. De bestaande destination-racefixture behoudt daadwerkelijk een vlak vóór de rename gemaakte lege map. Atomiciteit betekent hier geen power-loss-durability of onveranderlijkheid na publicatie. |

De nieuwe review is geen attestatie van een QEMU-proces en biedt geen bescherming
tegen een onbeperkt bevoegde schrijver die broncode en alle bewijsbestanden kan
vervangen. Er is evenmin een menselijke audit uitgevoerd.

## Uitgevoerde controles en vervolg

De **22 bestaande B07-capturetests** zijn op de immutable snapshot opnieuw
uitgevoerd en slagen. De twee aanvullende publicatieproeven bevestigen zowel de
werkende inodecontrole als G09-01. Commando's, exitstatussen, ongewijzigde logs en
hashes staan in `checks.json`. De B07-bundel met bestaande PNG-decodercontrole is
gelezen; er is geen nieuwe brede suite toegevoegd.

## B09-hercontrole

De kleine diff op `030790820688f366abe5ad10c608f83655b0d81f` definieert precies
elf vooraf bekende outputnamen, controleert die set vóór indexeren, bouwt de
index uitsluitend uit die constante en vereist opnieuw exact die elf namen plus
`capture.json` vlak vóór publicatie. De bestaande andere controles blijven staan.

De capturebron heeft SHA-256
`6585a357d58f6957616669c4923bd8db377de230d5ccafd08db42506f71f5c43`.
Dezelfde twee onafhankelijke probes zijn tegen een tweede immutable Git-snapshot
uitgevoerd. Het vroege extra bestand wordt nu afgewezen met
`Unexpected or missing staged archive file`; er wordt geen archief gepubliceerd.
De controle op vervangen inputidentiteit blijft slagen. G09-01 is daarmee binnen
het gereviewde bereik opgelost. `b09-fix-check.json` bewaart deze bronbinding,
commando en ruwe loghashes; de oorspronkelijke baselinebevinding in `checks.json`
blijft als historisch bewijs intact.

De eigenaar heeft daarnaast aantoonbaar de nieuwe negatieve test eerst op B07
rood en daarna op B09 groen uitgevoerd, met 34 geslaagde graphics-testmethoden;
deze review heeft die bestaande bundel gelezen en niet de gehele suite nogmaals
herhaald. Een nieuwe echte native G0-run en capture blijven werk van de centraal
toegewezen uitvoerder, op diens eigen bevroren bron en aparte artifacts.
