# Voortgangsmeting — van 0% naar een zelfstandig dagelijks OS

De actieve volledige releaseopdracht en productacceptatie staan in
[FULL-TEST-RELEASE.md](FULL-TEST-RELEASE.md). Dit vaste gewogen register blijft
de ontwikkelmeter; het vervangt de conjunctieve releaseacceptatie niet.

## Betekenis van het percentage

De README toont **behaalde acceptatiepunten / 100**. De gewichten zijn een
expliciete planningskeuze voor dit project, geen wetenschappelijk gemeten
voltooiingsgraad. Een punt staat voor een afgebakend resultaat met bewijs,
niet voor een aantal commits, tests, regels code of bestede uren. De laatste
20 punten kunnen veel meer werk vragen dan de eerste 20; de meter voorspelt
geen opleverdatum en drukt geen veiligheidsgarantie uit.

**0%** betekent dat nog geen criterium is gerealiseerd. Alleen plannen,
documentatie en een hostdemo leveren geen native productpunten op.
**100%** vereist dat **alle** onderstaande criteria zijn geaccepteerd: een
eigen, volwaardig dagelijks bruikbaar OS met native grafische bediening,
internet en nuttige apps, veilige persoonlijke opslag, herstel, updates,
ondersteunde fysieke hardware en een onderhoudbare uitgave. Het draait op de
eigen kernel, zonder Windows-, Linux-, BSD-, XNU-, seL4- of andere bestaande
OS-kernel als basis. Standaarden, ontwikkelgereedschappen en gecontroleerde
bibliotheken blijven toegestaan.

Het einddoel heeft dezelfde productcategorie als de grote besturingssystemen.
100% betekent geen bewezen gelijkheid met alle functies, alle apparaten of
alle decennia aan applicaties van elk bestaand platform. Ondersteuningsclaims
blijven gekoppeld aan concrete hardware en geteste gebruikstaken. Het profiel
moet een volwaardig dagelijks systeem opleveren; het mag niet worden versmald
tot alleen een VM-demo, één specialistische appliance of een testshell om
100% te kunnen tonen.

De bredere onderzoeksambitie blijft bestaan: fysieke QPU-besturing, CHERI,
verdere ISA's en uitbreiding van de hardware-/appmatrix staan in de roadmap.
De lokale quantumport telt hier mee; een fysieke QPU is geen voorwaarde om een
algemeen dagelijks desktop-OS te zijn en krijgt geen verzonnen voortgang.

## Hoe punten worden toegekend

1. Elk criterium heeft een vast gewicht en is `open` of `accepted`.
   Gedeeltelijk gebouwd of alleen gecompileerd blijft `open`; geen schijnprecisie
   met subjectieve deelpercentages binnen één criterium.
2. Acceptatie vereist relevante implementatie op het eigen OS en opgeslagen,
   versiegebonden bewijs volgens het criterium. Een smalle vroege proef krijgt
   alleen de punten van haar smalle criterium. Zo levert de vier-objectencatalogus
   één punt op; schaalbare volumes, grote documenten en algemene foutafhandeling
   houden hun eigen open punten.
3. Host-Materiaal, host-NAS/TLS en host-qsim zijn waardevolle voorbereiding en
   blijven zichtbaar in PROJECT-STATE.md. Zij voltooien geen native criterium.
   AI-review is geen menselijke audit; een pad naar een bewijsbestand is niet
   op zichzelf bewijs dat de inhoud klopt.
4. Eén resultaat wordt niet dubbel beloond. Bijvoorbeeld: H01 betreft de
   virtio-/DMA-/resetgrens; S01 de schrijvende dienst en flushsemantiek. Dezelfde
   testrun mag meerdere afzonderlijke eigenschappen ondersteunen, maar geen
   fictieve nieuwe runs of onafhankelijke bewijsklassen opleveren.
5. Bij een aangetoonde regressie, ontbrekend bewijs of te brede eerdere claim
   gaat het betrokken criterium terug naar `open`. De score kan dalen.
   Review, opschonen en bugfixes kunnen nuttig zijn zonder een nieuw punt op
   te leveren. Een criterium sluiten kost soms meerdere opdrachten.
6. Nieuwe criteria of gewijzigde gewichten vereisen een expliciete wijziging
   van `rubric_version`, motivering in DECISIONS.md en vergelijking met de oude
   score. Verlaag de eindlat of verschuif gewichten nooit alleen om vooruitgang
   te suggereren. De vaste totaalnoemer blijft 100.

## Eerste meting en afbakening

Op bron `b58ae2a56be6978eab2b5ac4bb5d12e3d615865f` is de beginscore **13%**:
9 punten voor M1–M4 (boot, geheugen, processen, capability-IPC), 3 voor M6a–M6c
(WRITE/FLUSH, objectherstel, namen/catalogus), en 1 voor het M5-devicepad.
Deze eerste score is afgeleid van de bestaande bewijsarchieven. Er zijn voor
het toevoegen van de meter geen native proeven uitgevoerd.

De meter geldt voor de beoordeelde **ontwikkelstand**. De actuele broncommit,
branch, datum en opdracht staan in de README. Een behaalde mijlpaal op die
branch is niet automatisch opgenomen in `main` of vrijgegeven als product.
Het scoreblad verandert geen bestaande acceptatiegrenzen van M1–M6c.

M6d op uitvoeringsbron `88e6c04be218b8b09ea4f0a226f6b9719da706a6` sluit
**S07** met opgeslagen fout-/herstelbewijs en de vereiste regressie. De score
stijgt daardoor **13% → 14%**: opslag van 3 naar 4 punten; alle andere gewichten
en statussen blijven gelijk. De selecteerbare historische M6b-observer behoudt
zijn afzonderlijke beperking. Zie [M6d-acceptatie](verification/M6D-ACCEPTANCE.md).
K05 (algemeen dienstbeheer) en S08 (algemeen opslagherstel) blijven open.

## Bij iedere opdracht bijwerken

Managementwijziging op managerbasis `caa2c14` (30 oktober 2026):
**34% → 34%**. Directeur `ef7a7596...` en assistent `5d5f8513...` hebben hun
werkelijke app-ID en rol bevestigd. De assistent krijgt na donorbronvrijgave
uitsluitend queuewrites en zelfstandige routineplanning; teamleider houdt
technische samenhang/status/score, intake zijn oorspronkelijke PR-taak.
Geen ontbrekend bericht trekt een toewijzing in. De nieuwe UI-probe is
smal native deelbewijs, geen gewone DEV-/cold-/U-acceptatie; B05 blijft een
echte menselijke gate. Geen vaste criteria/evidence gewijzigd, geen punten
voor organisatie of AI-review. Donorbron/ontvangst volgens WORK-QUEUE.

Eerdere specialistentoetreding:
Specialistenuitbreiding op managerbasis `caa2c14` (30 oktober 2026):
**34% → 34%**. De bestaande14rollen zijn met acht werkelijk geïdentificeerde
Simon-sessies en zeven afdelingen uitgebreid;22rollen betekent niet22gelijktijdige
agents of VM-slots. De Windowsroute is formeel van12 naar21 overgedragen.
Ontwerp/assets, afgebakend TLS-onderzoek/model en routebewijs zijn geen nieuwe
native productpunten. Eén diagnostische VM faalde vóór meetcalls op een
manifestguard: geen prestatiebaseline. Gedeeld0818beeld, vaste gewone DEV-bundel,
alle62FR en fysieke/menselijke gates blijven open. Criteria/gewichten/evidence
zijn exact behouden; nieuwe resultaten/owners/overdrachten in WORK-QUEUE.

Eerdere claimherstelmeting:
Claimherstel op managerbron `caa2c14` (30 oktober 2026): **34% → 34%**.
Eén actueel rollen-/taakregister vervangt de historische schrijfclaims.
Operationele integratie is exclusief e8, PR-intake read-only f2, centrale
bestanden alleen manager; release behoudt runtime-/ABI- en integrale
acceptatieverantwoordelijkheid. Nieuwe R77/SEC72-native deelproeven en
catalogus-/firmwarebewijs zijn brongebonden eigenaarrapportages, geen volledige
productaftekening. Renderer- en NAS-runnerreviewgaten krijgen ieder één
producer en onafhankelijke hercontrole. Criteria/gewichten/evidence blijven
gelijk; alle62FR, honderd koude starts/24uur en fysieke/menselijke gates open.

Historische teamleidersmeting:
Teamleiding (30 oktober 2026), startbasis `0ade8014`, inhoudsbehoudend
geïntegreerd met `94c8908` na #66: **34% → 34%**.
De twaalf sessies zijn tegen de volledige roadmap en releaseacceptatie getoetst
en hebben concrete eigenaars-/voortzettingsafspraken ontvangen. Live PR-checks
en eigenaarrapportages zijn geen nieuwe native acceptatie. Criteria, gewichten
en bestaand brongebonden bewijs blijven ongewijzigd; de gewone geïntegreerde
QEMU-gebruikersketen en alle volledige productrijen blijven open. De centrale
taakverdeling en ontbrekende implementaties staan in WORK-QUEUE/PROJECT-STATE.

Historische Windows-packagingmijlpalen uit de geïntegreerde #66-bron:
WINDOWS-QEMU-TEST-VERSION (30 oktober 2026), implementatiebron `f777f7a`:
**34% → 34%**. De branch voegt een hashvaliderende Windows-QEMU-bundel,
launcher, native QMP-previewrunner en CI-artifactjob toe. Hosttests,
strikte preflight en de warning-vrije native Materiaal-build zijn geslaagd.
Na herstel van een onterechte vaste Source/Policy-readinessvolgorde in het
hostorakel slaagt de echte QEMU-preview: QMP-quit exit 0, 800×600, 310 kleuren.
31 hosttests slagen; beide native rapporten blijven brongebonden bewaard in
`verification/windows-qemu-preview/`. Windows-hostlaunch en ZIP-generatie zijn
niet uitgevoerd; distributie-infrastructuur sluit geen productcriterium.

Aanvullend op basis `19f2d33` met CheckOnly-wijziging: vijf echte Windows
PowerShell-launchertests slagen met synthetische bestanden; QEMU 11.1.0,
SDL en `rng-builtin` zijn aanwezig. Bundelgrootte/hash en profielweigering
zijn gecontroleerd zonder VM of state. Geen Windows-gastacceptatie of
extra punten: **34% → 34%**.

Op basis `8f4fb3a` met packager-hercontrole slagen 33 hosttests,
inclusief weigering van een ELF-wissel na initiële validatie. Geen OS-ZIP,
Windows-gast of extra acceptatie; **34% → 34%** blijft gehandhaafd.
Op bron `5b80865` slaagt ook de complete tools-hostsuite (111 tests);
dat is geen native productacceptatie en verandert de score niet.
Documentatie-integratie PR #62 (30 oktober 2026), PR-head `ebbfb470` met
`main` `0ade801`: **34% → 34%**. Het hardwarecheckpoint en eerdere
PR-integratiebewijzen blijven behouden. Hardware blijft 1/12; H02–H06 blijven
open. Geen runtime- of criteriumwijziging en geen nieuwe native acceptatie.

Conflictintegratie PR #62 (30 oktober 2026), PR-head `c87fd769` met
`main` `f52dab5`: **34% → 34%**. Hosttest-preflight, vendordoctestherstel,
RTC/UTC en B02-persistentie blijven samen behouden. De integratie accepteert
geen productcriterium; historisch bewijs blijft aan zijn oorspronkelijke bron
gebonden. Uitgevoerde integratiecontroles staan in PROJECT-STATE.md.

RTC-NATIVE-1 (30 oktober 2026), basis `5c334f9`: **34% -> 34%**.
De expliciet opt-in CMOS-bootstrap is een technische tijdproviderafhankelijkheid,
geen productieklok of vertrouwde TLS-tijd. Geen rubricpunt of productcriterium
wordt door deze beperkte proef gesloten. PROJECT-STATE en het RTC-contract
scheiden werkelijk native bewijs, hostcodec en firmwarebeperkingen.
De echte RTC-CI-run op `295b4db` is groen; ook dat sluit geen productcriterium.

Conflictintegratie PR #61 (30 oktober 2026): **34% → 34%**. B03 en de
foundationwijzigingen van main blijven behouden zonder nieuwe punten; de
integratiecontroles staan in PROJECT-STATE.md. Bestaand brongebonden bewijs
wordt niet als een nieuwe productacceptatie aangemerkt.

Doorbouwcheckpoint 28 oktober 2026: integratie van main `391f02e`, vijf gerichte
M6b-timeout/cancelproeven en opt-in G0-bron veranderen de score niet:
**14% → 14%**. G0 is geen compositor/sessie en voldoet niet aan U01 of de fysieke
hardware-/audiogates H02/H03. De nieuwe gecombineerde native eindcontroles zijn
op dit checkpoint nog open. Criteria, gewichten en eerder geaccepteerd bewijs
zijn ongewijzigd; geen historische capture is naar nieuwe bron omgelabeld.

De hoofdagent doet dit bij ieder afgerond werkpakket en bij een tussentijds
checkpoint vóór lange controles:

1. Lees `docs/progress.json` en bepaal welke criteria de opdracht raakt.
2. Beoordeel de feitelijke resultaten en hun bron-/testbinding. Zet uitsluitend
   volledig gedragen criteria op `accepted` en voeg hun bewijsbestanden toe.
   Laat niet-uitgevoerde controles en lopend werk open.
3. Werk `assessment` bij: datum in Europe/Amsterdam, vaste beoordeelde
   broncommit, branch, taak, vorige score en korte toelichting op de delta.
   Gebruik voor de bron de test-/implementatiecommit; een bewijscommit hoeft
   niet naar zichzelf te verwijzen. Bij geen functiewinst staat de vorige
   score gelijk aan de nieuwe, met reden. De eerste meting gebruikt `null`.
4. Genereer de weergaven en controleer de uitkomst:

   ```sh
   python3 tools/progress.py --write
   python3 tools/progress.py --check
   ```

5. Werk PROJECT-STATE.md bij met uitvoering en resterende grenzen. Neem JSON,
   README, de gegenereerde tabel en SVG mee in hetzelfde checkpoint. De
   eindreactie vermeldt de oude/nieuwe score en de reden van wijziging.

**Bronbestand:** [progress.json](progress.json). De leesbare tabel hieronder,
de README-sectie en [lokale voortgangsbalk](assets/progress.svg) worden daaruit
gemaakt door [tools/progress.py](../tools/progress.py). De SVG doet geen externe
netwerkverzoeken. `--check` schrijft niets en controleert de som van 100,
unieke criteria, geaccepteerde bewijsverwijzingen en verouderde weergaven.
Het script voert geen OS-tests uit en beoordeelt de bewijsinhoud niet.

## Vast criteriumregister

<!-- OS-RUBRIC:BEGIN -->
### K — Eigen kern en procesisolatie (13/15)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **K01 — Zelfstandige UEFI-boot** | 2 | Behaald | Eigen loader draagt over aan eigen kernel; normale, ontbrekende/corrupte kernel- en timeoutgevallen zijn native gecontroleerd. [bewijs 1](../docs/verification/regression-m6b/FINAL-ACCEPTANCE.md) |
| **K02 — Eigen geheugenbeheer en foutgrenzen** | 2 | Behaald | Eigen pagetables/framebeheer, W^X, guardstacks en ongeldige mappings zijn in het vaste QEMU-profiel gecontroleerd. [bewijs 1](../docs/verification/m2-m4/ACCEPTANCE.md) · [bewijs 2](../docs/verification/regression-m6b/FINAL-ACCEPTANCE.md) |
| **K03 — Geïsoleerde, gepreëmpteerde processen** | 2 | Behaald | Ring-3-processen hebben afzonderlijke adresruimten; een appfault stopt de andere testprocessen niet. [bewijs 1](../docs/verification/m2-m4/ACCEPTANCE.md) · [bewijs 2](../docs/verification/regression-m6b/FINAL-ACCEPTANCE.md) |
| **K04 — Capabilities en begrensde IPC** | 3 | Behaald | Rechtenvermindering, generaties, intrekking, usercopy en begrensde Message320-IPC hebben native negatieve/lifecycleproeven. [bewijs 1](../docs/verification/m2-m4/ACCEPTANCE.md) · [bewijs 2](../docs/verification/regression-m6b/FINAL-ACCEPTANCE.md) |
| **K05 — Algemeen proces- en dienstbeheer** | 2 | Behaald | Niet-fixture-apps kunnen via begrensde API’s starten, afsluiten en diensten herstarten; afhankelijkheden en resources worden correct vrijgegeven. [bewijs 1](../docs/verification/K05-ACCEPTANCE.md) · [bewijs 2](../docs/verification/k05-startup-slots/gate/supervisor-gate.json) · [bewijs 3](../docs/verification/k05-startup-slots/integration.json) |
| **K06 — Meerdere processorkernen** | 2 | Open | SMP-scheduling, synchronisatie, interrupts en adresruimtewisseling werken onder concurrente belasting op het ondersteunde profiel. |
| **K07 — Resourcebeheer onder druk** | 2 | Behaald | Geheugen-, CPU-, IPC- en handle-uitputting blijven per app begrensd; het bruikbare systeem herstelt zonder kerneluitval. [bewijs 1](../docs/verification/K07-ACCEPTANCE.md) |

### S — Opslag en gegevensherstel (13/15)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **S01 — Native lezen, schrijven en flush** | 1 | Behaald | Dienstgebonden READ/WRITE/FLUSH lopen via echte virtio-I/O; succes volgt pas na gecontroleerde completion. [bewijs 1](../docs/verification/m6a-linux/README.md) · [bewijs 2](../docs/verification/m6b/README.md) |
| **S02 — Journal en koud herstel** | 1 | Behaald | Een begrensde objectjournal doorstaat native onderbrekingen, revisiecontrole, volle opslag en koude recovery met onafhankelijk schijforakel. [bewijs 1](../docs/verification/m6b/README.md) |
| **S03 — Namen en domeingebonden catalogus** | 1 | Behaald | Native GET/PUT/DELETE/RENAME/LIST bewaren namen, inhoud en domeingrenzen, ook na onderbroken hernoemen. [bewijs 1](../docs/verification/M6C-ACCEPTANCE.md) |
| **S04 — Ruimtehergebruik en schaalbare volumes** | 3 | Behaald | Opslag kan verwijderde ruimte veilig hergebruiken en langer dan de huidige 32 records doorwerken; crashconsistente compactie en quota zijn getest. [bewijs 1](../docs/verification/M6E-ACCEPTANCE.md) · [bewijs 2](../docs/verification/m6e/volume-gate.json) |
| **S05 — Documenten van bruikbare omvang** | 3 | Behaald | Grote documenten en streams overschrijden de huidige 128-byteproefgrens; begrensde buffers, volle volumes en deelwrites zijn getest. [bewijs 1](../docs/verification/M6E-ACCEPTANCE.md) · [bewijs 2](../docs/verification/m6e/volume-gate.json) |
| **S06 — Collecties, metadata en bewerkingsgeschiedenis** | 2 | Behaald | Native objectcollecties, persistente metadata en herstelbare revisies ondersteunen dagelijkse Materiaal-taken met consistente transacties. [bewijs 1](../docs/verification/S06-ACCEPTANCE.md) · [bewijs 2](../docs/verification/s06/volume-gate.json) |
| **S07 — Onzekere I/O zonder kernelpanic** | 1 | Behaald | De actuele M6c-catalogusdienst levert bij native WRITE/FLUSH-fouten gecontroleerd Unknown, sluit verdere toegang en herhaalt geen write; geen foutieve ACK of kernelpanic, overige processen blijven leven. Foutinjectie, koude recovery, schijforakel en normale M6c-/M6b-regressie zijn opgeslagen. De historische M6b-observer krijgt hiermee geen nieuwe foutafhandelingsclaim. [bewijs 1](../docs/verification/M6D-ACCEPTANCE.md) · [bewijs 2](../docs/verification/m6d/capture.json) · [bewijs 3](../docs/verification/m6d-m6b-regression/capture.json) · [bewijs 4](../docs/verification/m6d-review/m6d-final-native-audit.json) |
| **S08 — Algemeen opslagherstel bij fouten** | 1 | Behaald | Een bruikbaar systeem blijft reageren bij diskuitval, timeouts, corruptie en resource-uitputting; expliciete herstelkeuze voorkomt stille gegevensoverschrijving. [bewijs 1](../docs/verification/S08-ACCEPTANCE.md) |
| **S09 — Back-up, herstel en data-export** | 2 | Open | Een gebruiker kan onafhankelijke herstelkopieën maken, controleren, terugzetten en data in gedocumenteerde formaten exporteren; verlies-/conflictscenario’s zijn getest. |

### H — Hardware, drivers en energie (1/12)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **H01 — Eerste eigen devicepad** | 1 | Behaald | Eigen virtio-bloktransport, begrensde DMA/MMIO-toegang, reset en driver-/dienstcrashes zijn native in QEMU getest. [bewijs 1](../docs/verification/m5/ACCEPTANCE.md) · [bewijs 2](../docs/verification/m5/review-repairs.json) |
| **H02 — Toetsenbord, muis en USB** | 3 | Open | Gebruikelijke invoer en USB-apparaten werken op het gedocumenteerde referentieprofiel; hotplug en ongeldige device-antwoorden zijn begrensd. |
| **H03 — Grafische uitvoer en audio** | 3 | Open | Native grafische uitvoer, benodigde GPU-functionaliteit en audio voldoen aan vastgelegde bruikbaarheids-/latencybudgetten op fysieke referentiehardware. |
| **H04 — Netwerkhardware en verbindingen** | 2 | Open | Ethernet, wifi en de afgesproken Bluetooth-functies werken op ondersteunde hardware, inclusief verbreken, hervatten en driverfouten. |
| **H05 — Energiebeheer en slaap/wek** | 2 | Open | ACPI/energiebeheer, slaap/wek, shutdown en thermische/batterijfuncties zijn op ondersteunde apparatuur gemeten en herstelbaar. |
| **H06 — Fysieke ondersteuningsmatrix** | 1 | Open | Meerdere vooraf benoemde representatieve pc-configuraties doorstaan de gedocumenteerde hardwarematrix; DMA/IOMMU-grenzen en beperkingen staan expliciet vast. |

### U — Native Materiaal en toegankelijkheid (0/15)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **U01 — Native compositor en sessie** | 4 | Open | De eigen kernel draait een interactieve grafische sessie met render- en inputketen, venster/surface-isolatie en gecontroleerd compositorherstel. |
| **U02 — Vloeiende dagelijkse bediening** | 2 | Open | Toetsenbord, aanwijzer, focus, klembord en appwisseling zijn native bruikbaar en voldoen aan gemeten interactiebudgetten. |
| **U03 — Materiaal van openen tot bewaren** | 3 | Open | Gebruikers openen, bewerken, hernoemen en organiseren echte native documenten; rechten, historie en locatie blijven begrijpelijk en correct. |
| **U04 — Native toegankelijkheid** | 3 | Open | Toetsenbord-only, schermlezer, schaal/zoom, contrast en verminderde beweging werken in de native keten; taakproeven met gebruikers zijn vastgelegd. |
| **U05 — Instellingen, zoeken en herstel** | 2 | Open | Native systeeminstellingen, lokale zoekfunctie, account/sessiebeheer en begrijpelijke fout-/herstelroutes werken consistent. |
| **U06 — Afwerking getoetst met gebruikers** | 1 | Open | Representatieve gebruikers voltooien afgesproken kerntaken; frictie, fouten en visuele inconsistenties zijn gemeten en opgelost binnen productbudgetten. |

### N — Internet en netwerkdiensten (0/10)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **N01 — Native pakket- en transportstack** | 3 | Open | De eigen OS-netwerkdienst verwerkt echte packets en begrensde IP/TCP/UDP-verbindingen; malformed input, backpressure en afsluiten zijn getest. |
| **N02 — Configuratie en dagelijks internet** | 3 | Open | Adresconfiguratie, naamresolutie, routes, netwerkovergangen en benodigde internetprotocollen werken op echte toegestane verbindingen. |
| **N03 — Native geauthenticeerd transport** | 2 | Open | Onderhouden cryptografie/TLS met de vastgelegde hybride PQC-keuze werkt native, met peercontrole, negatieve handshakes en geen stille onveilige fallback. |
| **N04 — Afdwingbare netwerkrechten** | 2 | Open | Proces-/bestemmingsrechten, intrekking en helperroutes worden native afgedwongen en via verkeer waargenomen; geen onbedoelde luisterpoorten. |

### B — Beveiliging, privacy en updates (4/12)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **B01 — Entropy en sleutelbeheer** | 2 | Behaald | Native cryptografische entropy, key lifecycle, bescherming en herstel gebruiken gecontroleerde standaarden en geteste failure paths. [bewijs 1](../docs/verification/B01-ACCEPTANCE.md) · [bewijs 2](../docs/verification/native-recovery-hardening/b01/keyring-gate.json) · [bewijs 3](../docs/verification/native-recovery-hardening/integration.json) |
| **B02 — Geauthenticeerde persoonlijke opslag** | 2 | Open | Inhoud en gevoelige metadata worden native beschermd; manipulatie, replay/rollback binnen het dreigingsmodel en sleutelverlies zijn getest. |
| **B03 — Ondertekende updates en rollback** | 2 | Behaald | Authentieke updates, onderbreking, downgrade-aanvallen en herstel naar een geldige vorige versie zijn end-to-end getest. [bewijs 1](../docs/verification/B03-ACCEPTANCE.md) · [bewijs 2](../docs/verification/b03/update-gate.json) |
| **B04 — Vertrouwde boot- en distributieketen** | 2 | Open | Image-/bootverificatie, sleutelrotatie en dependency-/buildherkomst zijn voor het releaseprofiel afgebakend en onafhankelijk gecontroleerd. |
| **B05 — Gerichte onafhankelijke beveiligingscontrole** | 2 | Open | Externe deskundigen onderzoeken kernel-, dienst-, DMA-, parser- en capabilitygrenzen; blokkerende bevindingen zijn opgelost en relevante invarianten machinegecontroleerd waar gekozen. |
| **B06 — Native privacycontract** | 2 | Open | De toepasselijke LP-tests slagen op het eigen OS, inclusief geen telemetrie/cloudfallback, lokale AI en gescheiden app-/downloadrechten; resterende risico’s zijn gepubliceerd. |

### A — Apps, SDK en installatie (2/8)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **A01 — Publiek gedocumenteerde app-API en SDK** | 2 | Behaald | Een reproduceerbare SDK bouwt gewone apps voor het eigen OS; API/ABI-versies, sandbox en foutgedrag zijn gedocumenteerd en getest. [bewijs 1](../docs/verification/A01-ACCEPTANCE.md) · [bewijs 2](../sdk/README.md) |
| **A02 — Apps installeren en bijwerken** | 2 | Open | Pakketten uit meerdere bronnen kunnen met herkomstcontrole worden geïnstalleerd, verwijderd en bijgewerkt; installatie verleent geen algemene capabilities. |
| **A03 — Apprechten en gereedschapintegratie** | 2 | Open | Native apps gebruiken Materiaal-objecten via zichtbare, intrekbare taakrechten; extensies en hulpprocessen kunnen die grenzen niet omzeilen. |
| **A04 — Nuttige dagelijkse applicaties** | 2 | Open | Ten minste internetgebruik, documentbewerking, media en bestandsuitwisseling werken met geschikte native apps of gecontroleerde ports; realistische gebruikstaken zijn getest. |

### P — Persoonlijke NAS en meerdere apparaten (0/5)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **P01 — Native end-to-end versleutelde NAS-opslag** | 2 | Open | Client en NAS-rol op het eigen OS koppelen met geverifieerde identiteit; namen/inhoud zijn voor de NAS-beheerder niet leesbaar, ook bij fout-/aanvalspaden. |
| **P02 — Automatisch synchroniseren en conflicten** | 1 | Open | Eerder toegestane peers hervatten automatisch; offline edits, conflicten en LAN/internetwisseling behouden rechten en geven geen stille dataverliezen. |
| **P03 — Meerdere apparaten en key lifecycle** | 1 | Open | Apparaten kunnen veilig worden toegevoegd/ingetrokken; sleutelrotatie, gelijktijdige schrijvers en herstel werken zonder leverancierssleutelbewaring. |
| **P04 — Ruimte aanbieden, intrekken en verwijderen** | 1 | Open | De native NAS-beheerder beheert quota en trekt gastautoriteit in/verwijdert eigen kopieën zonder gastconsent of decryptiesleutel; oude sessies herleven niet. |

### L — Lokale AI en quantum-runtime (1/3)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **L01 — Bruikbare lokale systeem-AI** | 2 | Open | Minstens de afgesproken AI-kerntaken draaien op het eigen apparaat met begrensde context, afzonderlijke modeldownloads en geteste netwerkweigering; beperkte resources leiden niet tot cloudfallback. |
| **L02 — Lokale quantumuitvoering op eigen OS** | 1 | Behaald | De lokale simulator/jobinterface draait native met quota, annulering en referentiecircuits; hostsimulatie en fysieke QPU-ondersteuning blijven afzonderlijk gelabeld. [bewijs 1](../docs/verification/L02-ACCEPTANCE.md) |

### R — Dagelijks gebruik en onderhoudbare release (0/5)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **R01 — Installatie en toegankelijk herstel** | 1 | Open | Een gebruiker kan het OS op ondersteunde hardware installeren, upgraden en zelfstandig herstellen met gecontroleerde behoud-/wisopties; praktijkproef geslaagd. |
| **R02 — Langdurige stabiliteit en prestatiebudgetten** | 2 | Open | Vooraf vastgelegde duur-, workload-, energie-, geheugen-, latency- en dataintegriteitsbudgetten zijn op fysieke hardware gehaald; geen bekende blokkerende gebruiksfouten. |
| **R03 — Reproduceerbare uitgave en onderhoud** | 2 | Open | Onafhankelijke reproductie, licenties/SBOM, release-/patchbeleid, verantwoord meldpunt, ondersteuning en geldende toegankelijkheids/privacyverplichtingen zijn concreet geregeld. |

<!-- OS-RUBRIC:END -->

## Herkomst

Deze rubric is een projectbesluit op de opdracht van 28 oktober 2026. De
inhoud bouwt voort op [ROADMAP.md](ROADMAP.md), [DESIGN-AMBITION.md](DESIGN-AMBITION.md),
[LOCAL-PRIVACY-CONTRACT.md](LOCAL-PRIVACY-CONTRACT.md) en de bestaande native
acceptatiearchieven die bij ieder behaald criterium zijn gekoppeld.
De gewichten zijn geen externe benchmark of certificering.
