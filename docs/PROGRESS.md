# Voortgangsmeting — van 0% naar een zelfstandig dagelijks OS

Naamverklaring Baryon OS (8 november 2026), working tree op main
`4c7147da`: **41 → 41/100**. De README beschrijft baryonen als
samengestelde deeltjes en gebruikt modulariteit en onderzoek als naammetafoor.
Stabiliteit, veiligheid en prestaties blijven toetsbare ontwikkeldoelen.
Alleen documentatie en assessment gewijzigd; geen nieuwe productacceptatie.
[Naamverklaring](../README.md#waarom-de-naam-baryon-os) ·
[Naamgevingsbesluit](DECISIONS.md#adr-baryon-name-20261108--naamverklaring-in-de-readme).

Materiaal-bootparser en runneropruiming (8 november 2026), working tree op
main `8c8b5283`: **41 → 41/100**. De hoofd-CI op die main-bron is groen.
De aparte Materiaal-proef wees verwachte ACPI-diagnostiek ten onrechte af;
de exacte incrementele grammatica en negatieve hosttests herstellen dit.
Nieuwe native acceptatie blijft afzonderlijk te controleren. Geen criterium
of gewicht gewijzigd. [Bron en controles](verification/materiaal-boot-parser-20261108/README.md).

CodeRabbit-reviewopvolging (8 november 2026), working tree op main `1f3e7b6a`:
**41 → 41/100**. De drie reviewfixes uit de inmiddels gemergede PR #249 zijn
behouden. 32 gerichte Python-tests, 228 verifiercontroles en officiële
CodeRabbit-schemavalidatie slagen lokaal. Geen productcriterium of gewicht
gewijzigd; nieuwe-head CI en effectieve botconfiguratie worden apart getoetst.
[Bronbinding en controle](verification/coderabbit-unblock-20261108.json).

Publieke hosted CI (8 november 2026), gepubliceerde bron `338bec8c` op
integratiebasis `39586927` (dezelfde boom als lokaal `3195e659`): **41 → 41/100**.
23 workflows met standaardrunners; 248 Python-tests en 179 shellsyntaxcontroles
zijn lokaal uitgevoerd op de in het rapport gehashte working tree, later
gepubliceerd als `338bec8c`. De 224 tests verderop horen bij een eerder checkpoint.
Op GitHub slagen bij het afzonderlijke checkpoint 39 van 40 Actions-jobs;
de venstersessietest loopt nog. Native gates, productcriteria en gewichten behouden.
[Lokaal brongebonden bewijs](verification/ci-public-runners-20261108.json) ·
[GitHub-checkpoint](verification/ci-public-runners-20261108/hosted-checkpoint.json).

PR #249 hervat (8 november 2026), bron `575a5040` met main `5b218982`:
**41 → 41/100**. Vier documentconflicten opgelost met behoud van de
runnerwijziging en openbare ontwikkelrichting. GitHub bevestigt een
Actions-budgetblokkade vóór runnerstart; review verduidelijkt de bewijsgrens
en begrenst de diagnose-upload. Geen nieuwe host-/native
acceptatie. [Integratie- en statusbewijs](verification/ci-resume-249-20261108.json).

Gepinde reguliere CI (8 november 2026), bron PR #249 `d3d8e3e0`:
**41 → 41/100**. Vaste Python-/QEMU-/firmwareversies en 99 extra bestaande
hosttests; 224 gerichte hosttests en de configuratiecontrole slagen.
Geen nieuw native productcriterium; nieuwe-head OS-gates blijven nodig.
Zie [CI](CI.md) en [het controleverslag](verification/ci-standard-runners-20261108.json).

Runnerwissel op basis main `6e429de9` (8 november 2026): **41 → 41/100**.
Alle 21 workflows gebruiken standaard GitHub-hosted Ubuntu 24.04-runners;
24 jobdefinities en 42 uitgewerkte matrixjobs. Alleen CI-infrastructuur en
de host-buildcache veranderen; geen productcriterium of gewicht wijzigt.
Configuratie-/hostcontroles zijn geen nieuw native acceptatiebewijs.
Zie [de migratiecontrole](verification/github-hosted-runners-20261108.json).

PR #235-herstel (7 november 2026), PR-bron `bff194b5` met main `9bc5f52`:
**41 → 41/100**. Alleen buildpadcorrecties, twee functiedocstrings en een
actuele CI-basis; geen criterium of gewicht gewijzigd. Syntax-/AST-controle
slagen; geen nieuwe native uitvoering in deze sessie. Nieuwe-head CI blijft
vereist. Zie [het reviewbewijs](verification/pr235-review-20261107.json).

Sessie-/bibliotheekintegratie #206 op main `bce6cabc` (5 november 2026): **41 → 41/100**.
De U03-claim (41→44) uit het hashgebonden branchbewijs wordt op main niet
overgenomen; U03 blijft open tot de native bibliotheekgates op main slagen
en de eigenaar acceptatie bevestigt.

Integratie PR-wachtrij (#198 gemerged; #199 via branch `integrate/20261105-platform-batch`) op basis main `c7a61d11` (5 november 2026): **41 → 41/100**.
Driverhostmodellen (#198), acht platformhostapps met CI-dekking, EXIF-viewer,
agentprofielen en instructieopschoning geïntegreerd; geen criterium volledig
geaccepteerd en geen native, fysieke of menselijke acceptatie toegevoegd.

Instructieonderhoud op basis `8bb15309` (4 november 2026): **41 → 41/100**.
Verouderde beperkingen op opslaglocaties, prototype-eerst bouwen en
voortgangsrapportage zijn verwijderd. De actuele privacy-, native product-
en bewijscontracten blijven leidend. Geen runtimecode, criteria, gewichten
of bewijsstatus gewijzigd; geen nieuwe native, fysieke of menselijke acceptatie.

PR158-integratie met main `95d79451` op basis `04d727c3`: **41 → 41/100**.
De drie documentconflicten behouden beide checkpointgeschiedenissen en hun
brongebonden bewijs. De Rust-wijzigingen uit main zijn uitsluitend commentaar;
B03 blijft behouden. Geen criterium, gewicht of native acceptatie gewijzigd.
De historische PR148-integratiereview geldt niet voor deze integratie.
Nieuwe-head CI blijft afzonderlijk te beoordelen.

PR138-integratie met main `d686bd54` op basis `fc3b5e25`: **41 → 41/100**.
Alleen het statusconflict is handmatig opgelost; beide geschiedenissen en
brongebonden receipts blijven behouden. Geen nieuw acceptatiecriterium,
gewijzigd gewicht of nieuwe native acceptatie. Gerichte controles van de
opgeloste boom staan in PROJECT-STATE.md; nieuwe-head CI blijft afzonderlijk.

De actieve volledige releaseopdracht en productacceptatie staan in
[FULL-TEST-RELEASE.md](FULL-TEST-RELEASE.md). Dit vaste gewogen register blijft
de ontwikkelmeter; het vervangt de conjunctieve releaseacceptatie niet.

PR149-review op bron `53256f82`: **41 → 41**. Drie daadwerkelijk rood
gereproduceerde kaartfouten hersteld: smal/hoog, overlopende tekst en
onjuiste opslagoorzaak.59 volledige hosttests, Clippy, formattering en
scalarno_std-librarybuild slagen. Geen nieuwe native gast-/screenshot- of
menselijke acceptatie. [Volledig bewijs](verification/pr149-compact-review.json).

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

Community-inrichting (8 november 2026), basis `363b486b`: **41 → 41**.
Discussiecategorieën, startonderwerpen en bijdrageformulieren zijn toegevoegd;
deze opdracht wijzigt geen criterium, gewicht of runtimecode. De controles
betreffen formulierstructuur, links, pipeline-metadata en voortgangsweergaven,
geen nieuwe native productacceptatie. Zie de actuele handoff in
[PROJECT-STATE.md](PROJECT-STATE.md).

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:ab73c9e98b6e8e4f98c9394ee7fc9b7f34e8e2a2808b0067c5965ffb55c55446 -->

PR148-integratie (3 november 2026), PR-bron `47ec535f` met main `8e0c15d8`,
merge `7412d0bb`: **41 → 41**. De drie documentconflicten zijn opgelost met
behoud van beide bewijsreeksen; gedeelde harnashulpfuncties komen uit main en
volume-admissie uit PR148. Geen nieuw criterium of native acceptatie.

De integratiereview staat **open**: de handoff meldt gerichte controles met
uitvoer alleen in het sessietranscript; afzonderlijk brongebonden bewijs van
de review ontbreekt in de checkout. De eerdere PR148-hostreceipts zijn geen
bewijs voor deze integratiereview. De `reviewed_commit` `cc1dea7` in
`progress.json` hoort bij de historische documentatiereview; nieuwe-head CI en
review moeten nog worden beoordeeld. Zie [de handoff](PROJECT-STATE.md).

<!-- /coderabbit-ai-generated:v1 -->

PR149 bronbewijsvervolg (3 november 2026), schone bron `bb8877e3`:
59 hosttests echt opnieuw exit0 met cwd/HEAD/status/lock-/Rust-hashes vóór/na
en volledig gehasht log in
[de nieuwe receipt](verification/pr149-bb8877-host-receipt.json).
De eerdere Watch-tijden zijn geen primaire receipt. Scalar/Clippy/fmt blijven
hun oorspronkelijke bron532 houden; geen nieuw native/fysiek/menselijk bewijs.
**41 →41**, geen code- of criteriumwijziging.

PR149 derde main-integratie (3 november 2026), basis `41688635` met main
`99d13276`: **41 → 41**. Vier documentatieconflicten verzoend met behoud
van de compacte UI-fixes, hun brongebonden bewijs en de PR139-receipts.
Geen runtimecode gewijzigd of nieuw acceptatiecriterium gesloten.

PR149 tweede main-integratie (3 november 2026), basis `b75cfc3c` met main
`d115cc96`: **41 → 41**. Vier documentatieconflicten inhoudsbehoudend
verzoend. UI-code en artifact-store-tests blijven uit hun betreffende
parent behouden; geen nieuwe criteria of native acceptatie.

PR149-conflictoplossing (3 november 2026), PR-bron `92fdf2ff` met main
`69809c2c`: **41 → 41**. Vier centrale documentatieconflicten verzoend;
beide checkpointgeschiedenissen en de bestaande rubric blijven behouden.
Geen runtimecode handmatig gewijzigd, geen nieuw acceptatiecriterium.
Nieuwe-head-CI en native productacceptatie blijven afzonderlijke gates.

Native startoppervlak (3 november 2026), implementatiebron `bd4134c`:
**41% → 41%**. De opt-in VM-preview toont niet-geladen, lege, fout- en
gevulde Materiaaltoestanden. Alle 55 `ui/native`-hosttests slagen, inclusief
de pixelregressie; rustfmt, check, Clippy en de scalar
`x86_64-unknown-none`-releasebuild slagen. De testlinker is tijdelijk uit
Ubuntu-pakketten in `/tmp` opgebouwd, zonder systeeminstallatie. QEMU/OVMF en
native screenshot ontbreken. Geen nieuw criterium of runtimebewijs.
Zie `verification/native-start-surface-20261103/README.md`.

PR139-review (3 november2026), exact geteste geïntegreerde bron `0a49a382`:
**41 → 41**. Negen artifact-store-contracttests, Rust1.90.0 fmt en Clippy
daadwerkelijk opnieuw groen; primaire logs/boom/commando's/codehashes staan
in [de contracttestreceipt](verification/pr139-artifact-contract-receipt.json).
Geen retrospectieve validatie van eerdere Watch-claims zonder receipt,
geen nieuwe native of menselijke acceptatie.

PR139-mainintegratie (3 november 2026), basis `341d1b42` met main
`d115cc96`: **41 → 41**. De negen `artifact-store-core`-contracttests,
formatcontrole en Clippy waren destijds als geslaagd gemeld zonder primaire
PR139-receipt. Niet retrospectief geverifieerd; de nieuwe uitvoering op
`0a49a382` hierboven heeft wel volledige primaire receipts.
Dit toetst descriptorvalidatie en bounded-readgedrag, niet identiteit of
authenticatie; geen native/productacceptatie of nieuwe punten. De op oude
head `341d1b42` gerapporteerde `b03-native-update`-failure is door de
gepubliceerde merge superseded en wordt niet als actuele testuitslag hergebruikt.

PR139-reviewvervolg (3 november 2026), geteste bron `732fa9cc`: **41 → 41**.
De 16 replay-hosttests met `umask 0077` zijn opnieuw groen; met `umask 0002`
blijft exit 1 (1 failure, 2 errors) zichtbaar, geen groen resultaat.
[Primaire receipts met volledige logs](verification/pr139-review-receipts.json)
staan los van het
[historische prioriteitencheckpoint](verification/priority-plan-checkpoint-20261103.json).
Alleen dat historische checkpoint bevat de 15 workflowruns op `380111b`.
De primaire PR139-receipts bevatten uitsluitend de twee replaytestuitvoeringen.
Geen nieuwe native of productacceptatie.

PR139-mainintegratie (3 november 2026), basis `76743f5d` met main
`69809c2c`: **41 -> 41**. `76743f5d` is de assessmentbasis; de 15
historische OS-workflowruns blijven uitsluitend bewijs op `380111b` en zijn
niet opnieuw uitgevoerd. De unieke main-statushistorie is behouden; de 16
storage-backend-replaytests slagen met
umask `0077`. De ongewijzigde aanroep faalde in deze sandbox met umask
`0002` doordat de private-inputtests groepsschrijfbare fixtures produceerden;
de private-inputcontrole weigerde die correct. Progresscontrole slaagt; geen
nieuwe productcriteria of punten. Reviewverzoeken blijven open en zijn niet
in deze conflictintegratie afgehandeld.

PR139-reviewcorrectie (3 november2026), basis`716f3ec1`: **41 ->41**.
De gegenereerde `Bron` identificeert de assessmentbasis, niet automatisch
de commit van ieder onderliggend bewijs. De samenvatting noemt nu expliciet
de oorspronkelijke assessmentbasis`d3e5602c`, actuele basis`716f3ec1` en
uitsluitende historische OS-bewijsbron`380111b`. Bestaand verplicht veld
`assessment.source_commit` en generator blijven intact; afzonderlijke
bewijsbron staat in het checkpoint. Geen nieuwe native uitvoering of punten.

PR140-reviewvervolg (3 november 2026), geteste bron `2bf95782`: **41 → 41**.
De 9 contracttests, typecontrole, Clippy en formattering zijn opnieuw groen.
Volledige primaire logs met SHA-256 staan duurzaam in
[het reviewbewijs](verification/pr140-review-evidence.json).
Descriptorvalidatie/binding is geen bewijs van identiteit of authenticatie.
Geen nieuwe native of productacceptatie.

Historische PR140-reviewcorrectie (3 november 2026), basis `c07d15b8`: **41 → 41**.
Alleen Nederlandse beschrijvingen in het historische hostrapport; commands,
exitstatussen, bron-/loghashes en criteria blijven gelijk. Geen nieuwe tests
of native productacceptatie door deze tekstcorrectie.
PR119-inhoudsbehoud (3 november 2026), head `5a8317f1`: **41 -> 41**.
Volledige nieuwste NAS-history opgenomen; alleen centrale statusconflicten,
rubric buiten assessment identiek. Tweede/derde119-resolutie blijven
historische uitvoeringen; hun exacte bronstatus en vier cloudmain-mergeheads
zijn afzonderlijk gehasht bewaard, niet nieuwe native acceptatie.

Main-reconciliatie (3 november 2026), bron `cbe28f39`: **41 -> 41**.
De handmatige squash-merge van #134 (`cbbc9c99`) door de projecteigenaar is met behoud van inhoud
opgenomen; 10 netwerkservice- en 16 opslaghosttests geslaagd, 207
archiefhashes en 99 manifestvermeldingen geldig.
Rubric buiten assessment identiek; historische134-nativegrenzen niet als
verse uitvoering gepresenteerd. Volledige nieuwe CI/review blijft vereist.

Documentstapelintegratie (3 november 2026), bron `1552e5ae`: **41 -> 41**.
Volledige136/143/144/145-heads zijn boven de PR114-fixes behouden; geen
runtime-, tool- of testcode gewijzigd door die vier merges. Alle rubricvelden
buiten assessment waren identiek. Unieke intro/changelog/AI-rapport/audit
blijven behouden, zonder AI-review als menselijke audit te presenteren.
Gecombineerde huidige-head CI/review blijft vereist.

PR136-documentintegratie (3 november 2026), bron `f6921e49`: **41 -> 41**.
De L02-nativebron en A01accepted/R03open zijn correct geclassificeerd,
zonder nieuwe productpunten. Alle zeven geleverde bestandshashes en het
schema met39 groepen/62open releasekoppelingen zijn door parent gecontroleerd;
de d777-CI-entry blijft behouden. Geen verse native of menselijke
acceptatie afgeleid uit deze documentatiecorrectie.

PR145-rapportintegratie (3 november 2026), rapportbasis `380111b6`:
**41 -> 41**. Statische AI-review zonder opgeleverde bestands-/regeldekking;
geen onafhankelijke menselijke of volledige codebase-audit. Toenmalige
Dependabot-alertstatus was leeg, geen volledige advisoryscan. Secret
Scanning stond uit en lokale scanners ontbraken; geen complete secretscan,
codefix of producttest uitgevoerd. Zie
`verification/security-audit-20261103.md`; geen nieuw acceptatiepunt.

PR114-vervolgreview (3 november 2026), basis `783a48aa`: **41 -> 41**.
Primaire bronmetadata verduidelijkt historische proofscope. Twee originele
Btrfs-receipts binden82+17 entries onafhankelijk; onvolledige lijsten kunnen
geen lokale succesrapportage meer krijgen. Regressies eerst rood,16
hosttests daarna groen;207 oude archivehashes behouden. Geen nieuwe native
criteria voor een documentatie-/harnascorrectie.

PR114-research-/replayreview (3 november 2026), bron `d7d1fdc0`:
**41 -> 41**. Bronlinks/Ubuntu-volgorde hersteld;207 historische hashes
behouden, onveilige historische entrypoints afgeschermd en7 negatieve
hosttests groen. Volledige engine-/native replay blijft open; geen
backendadoptie, nieuw productcriterium of veiligheidspercentage.

Geïntegreerde mainstand (3 november 2026), bron
`380111b68688c981eec6abb8fab29174a29f5601`: **41% → 41%** voor deze
documentatietaak; geen criterium of gewicht gewijzigd. De exacte commit heeft
28/28 geslaagde GitHub-checkruns, waaronder de actuele native venstersessie en
de gecontroleerde native netwerkdienst. De wijzigingen documenteren bestaand
gedrag en voegen geen productbewijs of rubricpunten toe. Fysieke hardware,
Windows-runtime, menselijke audit en volledige testrelease blijven open.

PR114-metadataverwijzing (2 november 2026), bron `52e1909c`: **41 -> 41**.
De contractverwijzing noemt nu de echte fixbron `68d427b`, niet haar ouder
`c1d62d83`; het actuele assessment verwijst naar de volledige gecombineerde
bron. Geen runtimewijziging of nieuw acceptatiepunt.

Main-desktopintegratie (2 november 2026), `380111b6` / geteste head `5343b0a`:
**34 -> 41** door overname van U01 (+4) en N01 (+3), geen nieuwe gewichten.
31 actuele checks en brongebonden review zonder bevindingen;22 native
window/clipboard/herstelfasen en beide native netwerkcases slagen.
De schijfcontrole bevestigt14bytes/rev1/canaries; source-v3b7ccd11f.
De samengevoegde documentatie voegt zelf0punten toe; fysieke, menselijke,
productiesleutel- en volledige releasegrenzen blijven open.
Postmerge-CI loopt apart, geen hergebruik van oude bronclaims.

PR114-review (2 november 2026), fixbron `68d427b`: **34 -> 34**.
Het gebruikerswisselcontract beschrijft sleutelisolatie en handlelevensloop
in beide richtingen; native negatieve acceptatie is nog niet uitgevoerd.
Alleen ontwerp, geen profielenimplementatie of nieuwe productpunten.

PR #119-conflictoplossing (2 november 2026), bron `fa97c692` met main
`74a34d05`: **34% → 34%**. Main bevat de replay-implementatie en het bewijs
al; de vijf conflicten behouden de latere lekcontrole en regressietests.
Vóór het status-/metercheckpoint was de opgeloste boom bytegelijk aan main.
39 NAS-, 17 replay/oracle- en 3 volume-hosttests en de NAS-archiefhashcontrole
slagen. Strikte preflight faalt wegens ontbrekende Rust-/native-/QEMU-/OVMF-tools;
native uitvoering en Rust-format/lint zijn niet uitgevoerd. Nieuwe-head CI
blijft vereist. Geen nieuw geaccepteerd criterium of productpunt.

PR127-reviewintegratie (2 november 2026), basis `798706c` met `8f08db4`:
branch **41 → 41**, main **34 → 34**. Zeven reviewpunten en gekoppelde
font/damage-integratie hersteld; 189 Rust-app/UI-,9 netwerk- en31 Python-tests
slagen, inclusief echte TCP-rood/groenreproductie. De native22fasenacceptatie
van PR126 is aan die parentbron gebonden; deze nieuwe kandidaat vereist
eigen CI/runtime/review. Geen nieuwe punten, U02 of volledige releaseclaim.
Zie `verification/pr127-review-integration.json`.

PR126/main-integratie (2 november 2026), basis `5158027` en main `74a34d05`:
geerfde branchscore **41 → 41**, main-assessment **34 → 34**. Geen nieuwe
criteria of gewichten. Alle 260 gecombineerde Rust-hosttests slagen; een
bestaande expliciete ignore blijft vermeld. Sparse/retained-coverage en
beide testcaseverzamelingen zijn behouden. Nieuwe gezamenlijke native
QEMU-/herstel-/pixel-/klembord-/schijfgates blijven nodig; dit checkpoint
promoveert de geerfde U01/N01-bewijzen niet automatisch naar main.
Zie `verification/pr126-main-integration.json`.

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

### U — Native Materiaal en toegankelijkheid (4/15)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **U01 — Native compositor en sessie** | 4 | Behaald | De eigen kernel draait een interactieve grafische sessie met render- en inputketen, venster/surface-isolatie en gecontroleerd compositorherstel. [bewijs 1](../docs/verification/mature-session-20261102/U01-ACCEPTANCE.md) · [bewijs 2](../docs/verification/mature-session-20261102/U01-ACCEPTANCE.json) · [bewijs 3](../docs/verification/mature-session-20261102/window-337-ci-verification.json) |
| **U02 — Vloeiende dagelijkse bediening** | 2 | Open | Toetsenbord, aanwijzer, focus, klembord en appwisseling zijn native bruikbaar en voldoen aan gemeten interactiebudgetten. [bewijs 1](../docs/implementation/GUI-IDLE-WAIT.md) · [bewijs 2](../docs/verification/gui-idle-20261102/windows-ab/README.md) · [bewijs 3](../docs/verification/gui-idle-20261102/central-review.json) · [bewijs 4](../docs/verification/gui-idle-20261102/INTEGRATION.md) · [bewijs 5](../docs/implementation/NATIVE-CLIPBOARD.md) · [bewijs 6](../docs/verification/native-clipboard-20261102/README.md) |
| **U03 — Materiaal van openen tot bewaren** | 3 | Open | Gebruikers openen, bewerken, hernoemen en organiseren echte native documenten; rechten, historie en locatie blijven begrijpelijk en correct. |
| **U04 — Native toegankelijkheid** | 3 | Open | Toetsenbord-only, schermlezer, schaal/zoom, contrast en verminderde beweging werken in de native keten; taakproeven met gebruikers zijn vastgelegd. |
| **U05 — Instellingen, zoeken en herstel** | 2 | Open | Native systeeminstellingen, lokale zoekfunctie, account/sessiebeheer en begrijpelijke fout-/herstelroutes werken consistent. |
| **U06 — Afwerking getoetst met gebruikers** | 1 | Open | Representatieve gebruikers voltooien afgesproken kerntaken; frictie, fouten en visuele inconsistenties zijn gemeten en opgelost binnen productbudgetten. |

### N — Internet en netwerkdiensten (3/10)

| Criterium | Punten | Status | Acceptatie en bewijs |
|---|---:|---|---|
| **N01 — Native pakket- en transportstack** | 3 | Behaald | De eigen OS-netwerkdienst verwerkt echte packets en begrensde IP/TCP/UDP-verbindingen; malformed input, backpressure en afsluiten zijn getest. [bewijs 1](../docs/verification/mature-session-20261102/README.md) · [bewijs 2](../docs/verification/mature-session-20261102/n01-integrated.json) · [bewijs 3](../docs/verification/n01-native-service-20261102/host-verification.json) |
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
