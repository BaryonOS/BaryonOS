# I10 — gerichte review van de documentbewerklaag

## Oordeel en afbakening

**Geen concrete blokkerende bevinding gevonden binnen de toegewezen scope.**
Beoordeeld is G07-broncommit
`ccb2112072574d45b9c94c6f7181de23828bf79a`, uitsluitend de nieuwe
`sdk/document-session`-crate, haar tests/contract en de daarvoor relevante
bestaande transportgrens. Er zijn geen implementatiebestanden gewijzigd.

Voor de uitvoering zijn 21 relevante bron-, manifest-, lock- en testbestanden
byte voor byte met die commit vergeleken. Hun SHA256 staat in `checks.json`;
de bestanden bleven tijdens de controle gelijk. De tests draaiden vanuit
de integratiecheckout met een afzonderlijke targetmap. Deze bewijsbestanden
zijn uitsluitend in de worktree van de reviewer geschreven.

## Beoordeelde grenzen

| Grens | Concrete vaststelling |
|---|---|
| Geladen revisie en CAS | `open` bewaart de gelezen revisie. `save` gebruikt exact die revisie in PUT en accepteert alleen hetzelfde object met ACK-revisie `loaded + 1`. Zonder geldige ACK worden basisbytes en revisie niet bijgewerkt. |
| Stale/conflict | Een Stale-antwoord zet de conflicttoestand, behoudt het concept en voorkomt een volgende PUT. Er is geen automatische rebase of overschrijving. |
| Access | Lokaal ReadWrite kent geen rechten toe. Een canonieke Access-fout behoudt concept en geladen revisie; het hostmodel bevestigt dat geen journalmutatie plaatsvindt. |
| Unknown/verloren antwoord | Unknown en een onderbroken geënqueuede mutatie houden het concept intact en maken de sessie blijvend onzeker. Een volgende save verstuurt niets; lokaal discard wist onzekerheid niet. |
| Vóór enqueue | De expliciete `Delivery::NotEnqueued`-uitkomst bewaart het concept, sluit de transportverbinding en claimt geen mogelijke commit. Er is geen automatische reconnect of herhaling. |
| Expliciet discard/reopen | Een vuil concept blokkeert open/reopen. Discard herstelt uitsluitend lokale basisbytes; pas een geslaagde expliciete GET levert een nieuwe revisie/basis en wist conflict. Een onzekere sessie kan daarmee niet worden heropend. |
| Capintrekking | De nieuwe laag verwerft of vergroot geen endpoints. Intrekking in de echte capability-engine laat de PUT vóór enqueue mislukken; ReadWrite verandert dat niet en het concept blijft behouden. |
| Begrensde lokale bewerking | Alle range-/lengtecontroles vinden vóór wijziging plaats. De vaste 128-bytebuffers blijven opaque bytes; UTF-8-/GUI-beleid wordt niet naar deze laag of de kernel verplaatst. |

`is_dirty` vergelijkt lengte en de begrensde buffer met de laatst geladen of
bevestigde toestand. Alle bestaande vervangingspaden houden de ongebruikte
bufferstaart nul. `replace_range` begrenst de nieuwe lengte voordat zij
verschuift of kopieert; ook foutieve ranges en te grote invoer behouden het
vorige concept. Een ongewijzigd document veroorzaakt geen PUT.

Bij de review zijn `available`, `open`, `discard_changes`, `save` en `close`
langs de bovengenoemde foutpaden gevolgd. De bestaande catalogussessie
levert het onderscheid tussen niet-geënqueuede, geënqueuede en onzekere
aanvragen en sluit terminale transport-/servicefouten. De documentlaag
voegt daaraan geen verborgen retries of herverleende capability toe.

## Uitgevoerde controles

**Alle 12 bestaande gerichte hosttests slagen.** Daaronder vallen de twee
concurrerende schrijvers, antwoordverlies na corecommit, Unknown na lokaal
discard, EAGAIN-timeout vóór enqueue, Access, daadwerkelijke endpoint-
intrekking, behoud bij Full, ontbrekende/vreemde documenten, lege/maximale/
niet-UTF-8-inhoud en lokale revert zonder transport.

De test voor antwoordverlies herstelt de service uit de bewaarde
journalrecordbytes, niet uit de oude in-memory State. Een beschadigd record
wordt door de bestaande Recovery afgewezen. De vastgelegde WRITE/FLUSH/
commit/ACK-volgorde blijft een hostmodel, geen nieuwe native schijfproef.

```bash
CARGO_TARGET_DIR=/tmp/nieuw-os-par-i10-target \
  cargo test --manifest-path sdk/Cargo.toml \
  -p nieuw-os-document-session --locked --offline -- --test-threads=2
CARGO_TARGET_DIR=/tmp/nieuw-os-par-i10-target \
  cargo build --manifest-path sdk/Cargo.toml \
  -p nieuw-os-document-session --lib --release \
  --target x86_64-unknown-none --locked --offline
```

Beide opdrachten hebben exit 0. `document-session-tests.json` en
`native-library-build.json` bevatten de oorspronkelijke stdout/stderr,
opdrachten en hashes. `checks.json` bindt de bron en library-artifact.

## Resterende integratiegrens

Dit is een gerichte agentreview plus hosttest- en **native library-
compilatiebewijs**. Er is geen nieuwe VM, GUI-actie, G1-/Materiaal-integratie,
appstart of native WRITE/FLUSH-/crashproef uitgevoerd. De laag behandelt één
reeds bestaand document van maximaal 128 bytes; zij maakt geen groter
document- of opslagformaat aannemelijk.

De werkelijke native gebruikershandeling blijft afhankelijk van de daarvoor
afgesproken G1/appintegratie en haar eigen acceptatie. Deze review geeft
geen extra roadmapscore en verandert de eerdere native bewijsbundels niet.
