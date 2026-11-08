# Actuele projectstatus

## NEXT AGENT HANDOFF — English privacy contract — 9 November 2026

**Source:** `8331b69b9b06294ad8a8c5c571580271655ba8d1`; branch
`coderabbit/issue-274-privacy-contract-english`.

**Scope:** English translation of `LOCAL-PRIVACY-CONTRACT.md` for issue #274,
including all 31 LP scenarios and legacy section anchors. Contract revision,
requirements, evidence limits and runtime behavior are preserved. This is one
scoped documentation contribution; the remaining documents, language inventory,
UI localization, Wiki and Discussion categories remain outside this PR.

**Validation:** see [verification record](verification/issue-274-privacy-english.json).
No Rust, QEMU or hardware tests are claimed for this editorial change.
Product acceptance remains **41 → 41/100** with no new qualifying evidence.

**Next:** review the English contract against the source revision, then continue
the remaining English documentation coverage tracked in issue #274.
First local check: `python3 tools/progress.py --check`.

## Historical checkpoint — Herkomst na redactiereview — 9 november 2026

**Bron:** main `9922810a18f2272f534e1539e2d980d3d8922bb3`, branch
`docs/wetenschappelijke-herkomst-20261109`. De wetenschappelijke redactie uit
PR #265 en de CodeQL-wijziging uit PR #266 zijn behouden.

**Review en correctie:** CodeRabbit heeft dezelfde redactiedelta in PR #267
beoordeeld nadat de eerste review door de merge was afgebroken. Eén minor
bevinding betreft de herkomst van de desktoptoewijzing. De oorspronkelijke
bron `4c7147da` noemt de projecteigenaar als beslisser; die rol staat nu
expliciet in de onpersoonlijke formulering. Er wordt geen beslissende
AI-sessie verzonnen. Een ontbrekende lege regel vóór een Markdown-tabel is hersteld.

**Controle en grens:** voortgangscontrole, diffcontrole en gerichte
inhoudsvergelijking slagen (exitstatus 0). De afzonderlijke AI-bronbeoordeling
bevestigt de herkomst. Geen lokale Rust-, QEMU- of fysieke-hardwaretests voor
deze redactie. **41 → 41/100**; criteria, gewichten en bewijsstatus gelijk.
[Controleverslag](verification/wetenschappelijke-herkomst-20261109.json).

**Hervatten:** beoordeel GitHub-CI en de review op de nieuwe correctiehead;
CodeRabbit-resultaten op `bb3c1f6` betreffen de voorafgaande redactieversie.
Eerste lokale controle: `python3 tools/progress.py --check`.

## Historisch checkpoint — CodeQL-workflowcontract — 9 november 2026

**Bron en scope:** PR #266, branch `simonvanieperen-patch-3`, basis
`3402b30c3d67e769eea28abc07e1aaceeb3e2525`. De bestaande methode
`test_public_prs_keep_read_only_tokens` is vervangen door
`test_public_prs_keep_minimal_token_permissions`. Alleen de combinatie
`.github/workflows/codeql.yml` en job `analyze` krijgt de exacte mapping
`actions: read`, `contents: read`, `security-events: write`.
De al aanwezige CodeQL-workflow uit de PR is ongewijzigd.

**Controle en grens:** de oorspronkelijke suite faalde op deze uploadpermissie.
Na de wijziging slagen alle negen contracttests op 24 workflows en drie
composite actions. Eén toegestane en elf ongeldige permissievarianten geven
het verwachte resultaat. De geschreven diff is afzonderlijk door een tweede
AI gecontroleerd; dit is geen menselijke audit. `tools/progress.py --write`,
`--check` en `git diff --check` slagen met exitstatus 0.
[Versiegebonden controleverslag](verification/codeql-workflow-contract-20261109.json).
Geen lokale Rust-, QEMU-, CodeQL- of fysieke-hardwareproeven uitgevoerd voor
deze contracttestwijziging. De ontwikkelmeter blijft **41 → 41/100**.

**Hervatten:** beoordeel de nieuwe GitHub-CI en CodeQL-resultaten op de
gepubliceerde PR-head. Eerste lokale controle:
`python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py' -v`.

## Historisch checkpoint — Wetenschappelijke schrijfstijl — 9 november 2026

**Bron en wijziging:** main `1d5bc01e632fd5bab429f3befc9e9546c9f7661c`, branch
`docs/wetenschappelijke-schrijfstijl-20261109`. De eerste-persoonsredactie
uit PR #263 is gecorrigeerd naar onpersoonlijke, wetenschappelijke formuleringen.
`AGENTS.md` en `docs/AGENTS.md` leggen de gecorrigeerde schrijfafspraak vast.
De repositorybrede tekstzoeking omvat 6.338 UTF-8-bestanden binnen 6.599
versiebeheerde paden. In 55 Markdown-bestanden is de redactie bijgewerkt.

**Behoud:** besluiten, formele bevoegdheden, getallen, codeblokken, verwijzingen,
productcriteria en bewijsstatus blijven behouden. De naamverklaring van Baryon OS,
oorspronkelijke broncitaten, historische bronkopieën en hashgebonden bewijsbestanden
zijn ongewijzigd. Het eerdere redactieverslag blijft bewijs van de toenmalige
wijziging; de daarin beschreven ik-vorm geldt niet meer als schrijfafspraak.

**Controle en grens:** `python3 tools/progress.py --write`,
`python3 tools/progress.py --check` en `git diff --check` slagen (exitstatus 0).
De inhoudsvergelijking en afzonderlijke AI-redactiereview vonden geen inhoudelijke
blocker. Deze controles leveren geen nieuwe OS-testresultaten op; lokale Rust-,
QEMU- en fysieke-hardwaretests zijn voor deze redactie niet uitgevoerd.
**41 → 41/100**; criteria, gewichten en acceptatiestatus zijn ongewijzigd.
[Redactiecontrole en afbakening](verification/wetenschappelijke-schrijfstijl-20261109.json).

**Hervatten:** beoordeel de GitHub-reviews en vereiste CI op de gepubliceerde
PR-head afzonderlijk. De oorspronkelijke implementatietaken behouden hun eigen
bron en acceptatiecriteria. Eerste lokale controle:
`python3 tools/progress.py --check`.

## Historisch checkpoint — PR #264 conflictintegratie — 8 november 2026

**Bron en scope:** PR-head `b53a576a04f7cd5df3fc92cf32d2ceeb0c4939aa`,
branch `docs/baryon-name-20261108`, verenigd met main
`9ff3a691a535ebfd7d161d10408bf1a66b82d614`. Alleen conflictintegratie:
de naamverklaring, projectredactie en beide eerdere checkpoints blijven
behouden. Geen runtimecode, technische contracten of acceptatiecriteria gewijzigd.

**Controle en grens:** vóór afronding van de merge zijn
`python3 tools/progress.py --write`, `python3 tools/progress.py --check`
en `git diff --check` uitgevoerd en geslaagd (exitstatus 0).
De voortgang blijft **41 → 41/100**;
deze documentatie-integratie levert geen nieuw native bewijs.
Rust- en QEMU-tests zijn niet uitgevoerd voor deze documentatieconflicten.

**Hervatten:** beoordeel vereiste checks op de gepubliceerde integratie-head
afzonderlijk. Deze ronde doet geen CI- of reviewherstel en merget de PR niet.
Eerste lokale controle: `python3 tools/progress.py --check`.

## Historisch checkpoint — Naamverklaring Baryon OS — 8 november 2026

**Bron en scope:** working tree op main `4c7147dae955cbf1ba44d91258ef451b96169f72`,
branch `docs/baryon-name-20261108`. De README gebruikt Baryon OS als projectnaam
met een gecontroleerde verklaring. De bestaande repositorynaam en technische
identifiers blijven behouden. Dit is documentatie, geen nieuwe OS-functionaliteit.

**Inhoud:** een baryon is een samengesteld deeltje; gewone baryonen worden
beschreven met drie valentiequarks. De drie functionele hoofdlagen zijn een
naammetafoor, geen drie processorringen. Isolatie en beperkte bevoegdheden
zijn ontwerpkeuzes; prestaties en herstel vragen afgebakend bewijs. De
README verwijst naar CERN/CMS en ESA. Het besluit staat in
[ADR-BARYON-NAME-20261108](DECISIONS.md#adr-baryon-name-20261108--naamverklaring-in-de-readme).

**Controle en grens:** `python3 tools/progress.py --write`,
`python3 tools/progress.py --check` en `git diff --check` slagen.
De criteria en gewichten zijn behouden: **41 → 41/100**. Geen nieuwe
Rust-, QEMU- of fysieke-hardwaretests voor deze documentatiewijziging.

**Hervatten:** controleer de actuele PR-head, vereiste CI en reviewbevindingen.
Het technische vervolg uit het checkpoint hieronder blijft open: beoordeel
de aparte native Materiaal-proef op haar eigen bron. Naamgeving levert geen
nieuw bewijs voor die gate. Eerste lokale documentatiecontrole:
`python3 tools/progress.py --check`.

## Historisch checkpoint — Professionele projectstem — 8 november 2026

**Bron:** main `4c7147dae955cbf1ba44d91258ef451b96169f72`, branch
`docs/professionele-projectstem-20261108`. De 6.598 versiebeheerde paden zijn
geïnventariseerd; 6.337 UTF-8-bestanden zijn doorzocht op verwijzingen naar de
auteur. De redactie van 40 bestaande bestanden formuleert projectdoelen,
besluiten en instructies rechtstreeks en gebruikte de ik-vorm voor persoonlijke
keuzes en ervaringen. Die schrijfafspraak is na de gebruikerscorrectie vervangen
door de onpersoonlijke, wetenschappelijke stijl in `docs/AGENTS.md`. Het
oorspronkelijke redactieverslag hieronder blijft bewijs van de eerdere wijziging.

**Behoud en controle:** technische contracten, bevoegdheden, meetwaarden,
codeblokken, bronverwijzingen en productcriteria blijven behouden. Exacte
historische bronkopieën, ruwe logs, namen in bronmetadata en testinvoer zijn
ongewijzigd. De voortgangscontrole, diffcontrole en inhoudsvergelijkingen slagen;
de afzonderlijke AI-redactiereview vond geen inhoudelijke blocker.
Bewijs en afbakening: [redactiecontrole](verification/professionele-projectstem-20261108.json).
**41 → 41/100**; geen nieuwe functionaliteit of productacceptatie.

**Reviewcorrectie:** de Graphify-review op `12441cac` bevat één niet-blokkerend
advies: maak expliciet naar wie de ik-vorm bij mandaten verwijst. De
projectinstructies en het besluitenregister noemen daarom de projecteigenaar
in vaste metadata; de schrijfregel behoudt de oorspronkelijke spreker van
broncitaten. De bestaande bevoegdheden blijven ongewijzigd. CodeRabbit
meldde één minor issue: de gemelde vertraging moest expliciet ‘per scherm’
blijven. Die eenheid is hersteld; de instructie om fasetijden te meten blijft
behouden.

**Review en hervatten:** de lokale CodeRabbit-CLI is niet aangemeld;
`auth login --agent` meldt `environment_unsupported`. Er is geen lokale
CodeRabbit-review uitgevoerd. Beoordeel de GitHub-review en verplichte CI op
de nieuwe PR-head afzonderlijk. Eerste lokale controle:
`python3 tools/progress.py --check`. De eerdere Materiaal-proef en andere
open implementatietaken blijven bij hun eigen bron en acceptatiecriteria.

## Historisch checkpoint — Materiaal-bootparser na runnerherstel — 8 november 2026

**Bron:** working tree op main `8c8b528306f9d0e474c6b5d338cead904564c2df`,
branch `ci/materiaal-boot-diagnostics-20261108`. PR #249 en #261 zijn
opgenomen. De verplichte [main-CI](https://github.com/simonvanieperen/nieuw-os/actions/runs/42526645196)
is geslaagd. De ingehaalde #251 is na inhoudscontrole gesloten; vier oude
main-runs en de wachtende #251-run zijn geannuleerd. Nieuwe storage- en
volumeproeven op `1f3e7b6a` slagen; ook SMP op die bron slaagt.

**Concrete fout:** [Materiaal-run 42526590198](https://github.com/simonvanieperen/nieuw-os/actions/runs/42526590198)
startte op een GitHub-hosted `ubuntu-24.04`-runner, maar stopte na 3,21 s met
`Wrong boot/manifest sequence`. De kernel meldde na `KS-02:PASS` de voor dit
q35-profiel verwachte `ACPI:UNAVAILABLE:NO_DMAR` en
`DEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT`. De parser verwachtte meteen
`KS-03:PASS` en beëindigde QEMU zelf. Geen gast- of hosttimeout, geen
artifactquotafout en geen bewezen kernelregressie.

**Reparatie:** `tools/run_materiaal_probe.py` gebruikt het bestaande
`with_acpi_markers(BOOT)` in de incrementele openingsgrammatica. Het paar is
verplicht en exact geordend; er wordt niets blind uit het bewijs gefilterd.
Ook zulke meldingen vóór de bootoverdracht worden geweigerd. De bestaande
hostfixtures volgen het actuele profiel en regressies controleren elke
bytegrens en ongeldige/misplaatste diagnostiek. Kernel, hardwareprofiel,
watchdogs, pixels, schijfcontrole en overige acceptatie blijven gelijk.

**Bewijs en grens:** bronhashes, hosttestuitvoer en oorspronkelijke fout staan
in [materiaal-boot-parser-20261108](verification/materiaal-boot-parser-20261108/README.md).
Geen lokale Rust-/QEMU-toolchain beschikbaar; native uitvoering gebeurt via
het bestaande handmatige `materiaal-probe.yml` op de gepubliceerde branch.
Die gate moet de rest van de Materiaal-keten nog aantonen. **41 → 41/100**;
geen productcriterium afgetekend of gewicht aangepast.

**Hervatten:** controleer de actuele PR-head, verplichte CI en de afzonderlijke
Materiaal-run. Eerste lokale controle:
`python3 -B tools/tests/test_materiaal_probe.py`; vervolgens
`python3 tools/progress.py --check`. Oude PR-branches met self-hosted workflows
moeten main inhoudelijk opnemen; een rerun wijzigt hun oude configuratie niet.

## Historisch checkpoint — CodeRabbit-reviewbeleid en verifier — 8 november 2026

**Bron en scope:** working tree op main
`1f3e7b6acdfd4b5ca84a69422a2d3ed6e424609a`, waarin PR #249 is opgenomen.
Vervolgbranch `ci/coderabbit-review-policy-20261108`. De nieuwe main-boom is
exact gelijk aan de eerder geteste PR249-head `7b82315a`; alle gelijktijdige
reviewfixes en hun bewijsbestanden zijn behouden. **41 → 41/100**.

**Diagnose:** de main-regel vereist `host-quality`, `qemu-smoke`, actuele
branches en opgeloste reviewgesprekken, maar nul approvals. CodeRabbit is
geen vereiste statuscheck. In de oorspronkelijke momentopname van 27 open PRs
hadden 22 een actieve CodeRabbit-afkeuring en 17 bovendien mergeconflicten.
Bij onder meer #207 wacht CI nog op verwijderde self-hosted runners.
Herlees actuele heads en reviews vóór verdere afhandeling.

**Gerealiseerd:** `.coderabbit.yaml` kiest `quiet`, behoudt automatische
incrementele reviews en zet automatische botafkeuringen/goedkeuringen uit.
Overige instellingen worden geërfd; effectief gebruik en globale overrides
moeten op de volgende review worden beoordeeld. De GitHub-regels blijven gelden.
De verifier behoudt de controles uit `7b82315a`, maar ontbrekende workflows
of onleesbare baselines geven nu expliciete fouten en een bruikbaar rapport.
Acht regressietests draaien via de bestaande CI-discovery.

**Uitgevoerd:** 23 collector-/verifier-tests en 9 workflowtests geslaagd,
exit 0. De gecombineerde verifier slaagt voor 228 controles en 16
bewijsworkflows; de historische artifactbron slaagt voor 189 controles.
CodeRabbit-YAML voldoet aan het officiële schema. Bronhashes, commando's,
logs en bewijsgrenzen staan in
[coderabbit-unblock-20261108.json](verification/coderabbit-unblock-20261108.json).
Geen lokale Rust-/QEMU-uitvoering of nieuwe productacceptatie.

**Integratie en open werk:** #249 is tijdens deze sessie door parallel werk
samengevoegd; alle drie eerdere reviewpunten zijn daarin afgehandeld.
Deze kleine vervolgwijziging wordt als aparte concept-PR gepubliceerd omdat
#249 al gesloten is. Behoud squash-auto-merge en controleer de nieuwe
verplichte CI en CodeRabbit-configuratie. De oude hosted momentopname op
`338bec8c` blijft historisch: de toen lopende venstersessietest eindigde later
als `cancelled`. Oudere PRs moeten actuele workflows/configuratie opnemen;
bestaande inhoudelijke bevindingen en conflicten blijven afzonderlijk werk.
Volgend commando: `python3 tools/progress.py --check`; daarna de actuele
remote head, review en vereiste checks controleren.

## Historisch checkpoint — hosted CI uitgevoerd, review afgerond in bron — 8 november 2026

**Bron:** PR #249, `338bec8ca34ba8eb2451eb660a0f1790c194efd2`, op
integratiebasis `39586927`; huidige vervolgwijzigingen betreffen uitsluitend
documentatie, het historische verificatiescript en nieuw opgeslagen bewijs.
Werkbranch `ci/github-hosted-runners-20261108`; score **41 → 41/100**.

**Werkelijk uitgevoerd op GitHub:** bij de waarneming van 21:17:57 UTC slagen
39 van 40 Actions-jobs op bron `338bec8c`. De verplichte `host-quality` en
`qemu-smoke` starten parallel en slagen op echte `ubuntu-24.04`-runners;
ook de volledige SMP-suite en hosted Copilot-setup slagen. De compacte
QEMU-upload is beschikbaar (97.119 bytes, drie dagen retentie). De lange
venstersessietest loopt nog; geen mislukte job vastgesteld. Zie het
[brongebonden GitHub-checkpoint](verification/ci-public-runners-20261108/hosted-checkpoint.json).
De historische 8-vCPU-timeout is in deze run niet opgetreden; geen geval of
tijdslimiet is versoepeld.

**Reviewvervolg:** de uploadbeschrijving noemt alle drie best-effort uploads;
het gegenereerde README onderscheidt integratiebasis, geteste working tree
en gepubliceerde bron. De 224 tests zijn expliciet historisch; de 248 lokale
tests en 179 shellsyntaxcontroles blijven aan hun oorspronkelijke hashes
gebonden. Het verificatiescript gebruikt de gepubliceerde configuratie als
vaste baseline en registreert nieuwe workflows zonder verzonnen vergelijking.
Alle **202 actuele configuratie-/bewijscontroles** slagen, exit 0; bronhashes
en afzonderlijke vergelijkingen staan in
[review-workflow-checks.json](verification/ci-public-runners-20261108/review-workflow-checks.json).
De workflowbestanden, productcode en testcommando's zijn sinds `338bec8c`
ongewijzigd.

**Eerstvolgende stap:** publiceer dit beperkte reviewvervolg op dezelfde PR,
handel de drie concrete reviewthreads af en behoud squash-auto-merge volgens
de bestaande branchregels. Nieuwe-head CI en review moeten nog volgen;
geen merge of volledige native einduitslag geclaimd. Sluit PR #251 pas na
aantoonbare opname via #249 in main. De oude Copilot-WSL-keuze uit #245 mag
niet opnieuw worden ingevoerd.

## Historisch checkpoint — publieke hosted CI geïntegreerd — 8 november 2026

**Bron:** main `0cf49b584f9264e676d62168caeb2230669ca538`, runner-PR #249
`0de9d805` en artifact-PR #251 `dc6c6bca`; lokale integratiebasis `3195e659`.
Dezelfde integratieboom staat op GitHub als `39586927`; push verloopt via
de GitHub-koppeling omdat lokaal geen Git-pushcredential beschikbaar is.
Werkbranch blijft `ci/github-hosted-runners-20261108`; dezelfde PR #249.
GitHub bevestigt dat de bestaande repository `public` is. **41 → 41/100**.

**Gerealiseerd:** alle 23 workflows kiezen `ubuntu-24.04` (26 jobdefinities,
42 mogelijke matrixjobs). Ook de historische private audit heeft geen eigen
runnerselector meer; zijn private-only voorwaarde blijft behouden. De vaste
Python-/QEMU-/firmwarepins en compact-bewijsaction zijn gecombineerd.
`host-quality` en `qemu-smoke` draaien parallel en blijven verplichte checks.
Specialistische main-pushes volgen de bestaande documentfilters; geen dubbele
`ready_for_review`-runs. Twee volledig overlappende matrixentries zijn
samengevoegd met hun specialistische gates, inclusief scalar build; 14
matrixjobs, maximaal vier tegelijk. De hostcache krijgt een bron-SHA/herstel-
prefix. Copilot heeft een hosted setup met dezelfde gereedschappen.

**Werkelijk lokaal gecontroleerd:** de working tree op integratiebasis
`39586927` (dezelfde basisboom als lokaal `3195e659`), met de aanvullende
workflowwijzigingen en regressietests. De basiscommit alleen bevat die
aanvullingen niet. Op deze working tree: 239 bestaande host-/gate-/collectortests
plus 9 configuratieregressietests = **248 geslaagd**, exit 0. 23 workflow- en
3 Action-YAMLs met unieke sleutels; **179 bashblokken** syntactisch geldig.
Alle **170 bestaande run-stappen** sinds de integratiebasis en **16
bewijsselecties** behouden. De enige extra run-opdracht in bestaande jobs is
de nieuwe workflowregressietest. Rapport en bronhashes:
[ci-public-runners-20261108.json](verification/ci-public-runners-20261108.json).
Geen nieuwe lokale Rust-/QEMU-run: die tools zijn hier niet aanwezig.

**Open bij checkpoint:** nieuwe hosted CI op de te pushen gecombineerde head;
echte toolinstallatie, native suites en uploadstatus apart beoordelen. De
historische budgetfout was op de toen private repository en voorspelt de
nieuwe run niet. Branchregels blijven ongewijzigd; reviewthreads moeten
inhoudelijk zijn afgehandeld. PR #251 pas sluiten nadat zijn inhoud via #249
in main is opgenomen. De oude Copilot-WSL-keuze uit #245 mag niet terugkeren.

**Eerstvolgende stap:** push dezelfde branch, werk PR #249 bij en activeer
squash-auto-merge volgens bestaande regels; verwerk concrete CI-/reviewfouten.
De oude 8-vCPU SMP-timeout blijft een te meten runnercapaciteitsrisico;
geen markers, cases of tijdslimieten zijn hiervoor versoepeld.

## Historisch checkpoint — PR #251 artifactopslag hervat — 8 november 2026

**Bron:** PR #251, head `5c291cff3b9c28e4948aeb36adda3acc6ee6a8ab`;
oorspronkelijke basis `363b486b06023d3d578f8eca2b23ff49045a1757`; geïntegreerde
main `5b2189824c32145ad5c110429517dfcea3f2720f` (openbare projectrichting); werkbranch
`ci/artifact-storage-20261108`. **41 → 41/100**; geen criterium gewijzigd.
Scope: bestaande artifactreparatie afronden. De runner-/toolchainmigratie
blijft een afzonderlijke wijziging. Besluit: ADR-CI-EVIDENCE-20261108.

**Gerealiseerd:** vijftien workflows gebruiken al de gedeelde collector/action.
Dit vervolg vult de concrete ADR-reviewvelden en ontbrekende docstrings aan,
en voegt twaalf configuratiecontroles toe voor opname van het Windows-pakket
in het volledige archief. De twee huidige package-workflows voldeden al;
ook het verwijderen van hun betreffende padregel wordt nu gedetecteerd.
Collector en collectortests hebben na verwijderen van docstrings dezelfde AST
als de bron. Geen workflow, uploadactie, testgate of runnerkeuze gewijzigd.
De advisory over veertien callees in `collect()` is beoordeeld: orkestratie
van expliciete selectie-, lees-, budget- en rapportstappen; geen aangetoond
defect en geen cosmetische herstructurering van de veiligheidsgrenzen.

**Werkelijk runtimebewijs:** native-display-input (run `42511057121`, job
`128824002965`) en B03 signed updates (run `42511057222`, job `128824003122`)
zijn op PR-mergecheckout `89722c5fd132beeecd95ae6777af601c83fd0f6f` geslaagd.
De compacte uploads falen in beide logs met `Artifact storage quota has been
hit`; de vooraf geschreven bronbinding, rapporten en waarschuwing blijven
zichtbaar. Dit bewijst de echte quota-terugval voor deze bron, geen beschikbare
artifactdownload of volledige raw-hercontrole. De nieuwe documentatiehead
krijgt eigen CI; de oude run wordt niet naar die head omgelabeld.

**Lokale validatie:** 85 bron-/bewijsbestanden en 22 oorspronkelijke workflows
zijn tegen Git-blobhashes gecontroleerd. Dit is een deelsnapshot, geen volledige
clone. Alle 15 collector-hosttests en 163 YAML-/shell/configuratiecontroles
slagen; voortgangscontrole en diff-check slagen. Bewijs en bronhashes:
`docs/verification/ci-artifact-storage-20261108/review-followup.json`.
Geen lokale Rust-/QEMU-uitvoering; Cargo en QEMU ontbreken.

**Open en hervatstap:** controleer nieuwe PR-head, reviewthreads en verplichte
`host-quality`/`qemu-smoke`; squash-auto-merge volgt de bestaande regels.
De reparatie is pas actief op main na merge. Geen artifacts verwijderd en
geen vrijgemaakte opslag geclaimd. Een ontbrekend raw-archief blijft
onvoldoende voor acceptatie die die bytes vereist. Lokale hercontrole in
een volledige clone: `python3 -B docs/verification/ci-artifact-storage-20261108/verify_workflows.py --repo . --output /tmp/ci-evidence-workflow-checks.json`.

## Historisch checkpoint — PR #249 hervat; Actions-budget blokkeert — 8 november 2026

**Bron en taak:** hervat PR #249 op `575a50409ad7a86786bbfc7da2c409a545bed33f`
en neem main `5b2189824c32145ad5c110429517dfcea3f2720f` op met behoud van de
openbare ontwikkelrichting en het MPL-2.0-controlekader uit PR #250.
Werkbranch: `ci/github-hosted-runners-20261108`. **41 → 41/100**.

**Gerealiseerd:** vier documentconflicten opgelost. Runnerkeuze, gedeelde
toolinstallatie, oorspronkelijke testcommando's, private publicatie-audit en
opgeslagen testlogs blijven behouden. De aanvullende QEMU-upload selecteert
alleen bekende rapport- en logpaden; brede `build/**`-patronen zijn verwijderd.
De publicatierichting, licentievoorwaarden en beide checkpointgeschiedenissen
zijn behouden. Geen productcode gewijzigd.

**Reviewopvolging:** de vier opmerkingen van CodeRabbit op bron `575a5040`
zijn getoetst: uploadselectie begrensd, geïmplementeerde configuratie duidelijk
gescheiden van hosted uitvoeringsbewijs, het oude apt-besluit verduidelijkt,
en de correcte telscopes toegelicht: 21 reguliere workflows + 2 Action-YAMLs
= 23 YAMLs; 167 workflowstappen + 3 Action-stappen = 170. `requirements.txt`
is geen YAML. De afzonderlijke private audit valt buiten die telling.

**Bevestigde uitvoeringsblokkade:** de GitHub-runpagina van
[CI 42511934820](https://github.com/simonvanieperen/nieuw-os/actions/runs/42511934820)
op bron `575a5040` meldt letterlijk:
“The job was not started because an Actions budget is preventing further use.”
`host-quality` faalde vóór uitvoering; `qemu-smoke` is overgeslagen. Dit
bevestigt de budgetblokkade voor deze run; het is geen testuitslag of
artifact-uploadfout. De exacte budgetlimiet en het verbruik zijn niet ingezien.

**Controle:** bronbomen en gebruikte commits zijn tegen de Git-object-ID's
gecontroleerd. Inhoudsbehoud, YAML/JSON, uploadselectie, conflictmarkers en de
voortgangsgenerator staan in `verification/ci-resume-249-20261108.json`. De 224 eerdere
hosttests en de gepauzeerde QEMU-toolprobe blijven uitsluitend bij hun
historische bronbewijs horen. In deze integratie zijn geen Rust-/Node-
hostsuite, native OS-starts of fysieke tests uitgevoerd.

**Open en hervatten:** rond de afzonderlijke publicatiecontrole af volgens
`LICENSE-PLAN.md` en de bestaande openbare projectrichting. Zolang de private
repo onder dezelfde budgetblokkade valt, levert opnieuw starten geen nieuw
testbewijs. Controleer na gewijzigde uitvoeringsvoorwaarden de actuele PR-head
en start de vereiste hosted gates; beoordeel ook het historische
8-vCPU/TCG-timingrisico met een echte meting. Behoud checks en branchregels.
Deze hervatting wijzigt geen zichtbaarheid, facturering, quota of opslag.
Eerste lokale controle: `python3 -B tools/progress.py --check`.

## Historisch checkpoint — Gepinde reguliere CI — 8 november 2026

**Bron:** PR #249 `d3d8e3e0a3b0b455b1832bdc53220352c2b20960`, met main
`363b486b06023d3d578f8eca2b23ff49045a1757` (private audit behouden).
Werkbranch: `ci/github-hosted-runners-20261108`. **41 → 41/100**;
criteria, gewichten en productacceptatie blijven gelijk.

**Taakcontract en resultaat:** maak reguliere CI uitvoerbaar op standaard
GitHub-runners met vaste tools. Alle 24 jobdefinities in 21 reguliere
workflows gebruiken Ubuntu 24.04 en Python 3.12.12. De 17 QEMU-jobs delen
één snapshotgebonden installatie met exacte pakket- en firmwarecontrole.
Rust-/Node-pins blijven behouden. De hoofdgate krijgt 99 bestaande
runner-/preflight-/M6c-hosttests. De zes native starts, time-outs,
rechten, triggers en testcommando's blijven behouden. Details: `CI.md`;
besluit: ADR-CI-PINS-20261108.

**Uitgevoerd:** 224 geselecteerde bestaande hosttests geslaagd op lokale
Python 3.12.14; YAML-/shellcontrole, behoud van bestaande testcommando's en
APT-resolutie van de exacte pins geslaagd. Pakketindexen, handtekeningen,
payloads en firmwarehashes zijn gecontroleerd. De uitgepakte QEMU 8.2.2
start een gepauzeerd pc-q35-8.2/TCG-proces en sluit via QMP af (exit 0);
dit is geen eigen-OS-boot. Een AI-review vond een
GitHub-specifieke diagnosekopie in de lokale gate; die is gescheiden van de
onvoorwaardelijke preflight en opnieuw gecontroleerd. Bewijs en exacte
grenzen: `verification/ci-standard-runners-20261108.json` en
`verification/ci-packages-20261108.json`.

**Open:** volledige Rust-/Node-hostkwaliteit en zes eigen-OS-QEMU-starts op
de nieuwe PR-head. Het lokale hostbewijs is niet op de beoogde Python-pin
3.12.12 uitgevoerd. De sudo-installatiestap kan hier door een UID-mapping-
beperking niet letterlijk worden uitgevoerd; APT is afzonderlijk met
tijdelijke bronnen/indexen gecontroleerd. De vorige hosted CI-run op
`d3d8e3e0` stopte vóór runner-toewijzing; precieze reden niet geverifieerd.
Nieuwe CI-status hoort bij de actuele PR-head, niet bij dat oude resultaat.

**Hervatten:** controleer PR #249 en zijn nieuwe `host-quality` en
`qemu-smoke`-jobs, inclusief eventuele annotatie wanneer GitHub geen stappen
start. Behoud branchregels; geen zichtbaarheid, facturering of quota aanpassen
als bijwerking. Bewijs van een toolprobe is geen native OS-acceptatie.

## Historisch checkpoint — GitHub-hosted runners — 8 november 2026

**Bron:** main `6e429de98b5c692ccceaa12973818eecb1622062`, werkbranch
`ci/github-hosted-runners-20261108`. **41 → 41/100**; geen productcriterium,
gewicht of bestaand acceptatiebewijs wijzigt.

**Gerealiseerd:** alle 24 jobdefinities in 21 workflows gebruiken
`ubuntu-24.04` (42 jobs na matrixuitbreiding). De host-Rust-buildcache heeft
een nieuwe Ubuntu-namespace. Teststappen, toolchainpins, firmwarepaden,
time-outs, rechten, triggers, caches voor downloads en artifactretentie blijven
behouden. Besluit: ADR-CI-HOSTED-20261108 in `DECISIONS.md`.

**Taakcontract:** uitsluitend runnerselectie, de bijbehorende hostcache en
projectregistratie aanpassen. Acceptatie: geen self-hosted selectie meer in
de workflows, verder gelijke uitvoeringsconfiguratie en consistente
voortgangsweergave. Bestaande kernel-/runtimefouten worden hiermee niet als
hersteld beschouwd.

**Controle:** de lokale snapshot is via de GitHub-API op de vaste broncommit
gelezen en tegen Git-blob-ID's gecontroleerd. YAML-structuur en shellsyntaxis,
behoud van de overige workflowvelden, de zeven bestaande agent-gate-hosttests
en `tools/progress.py --write`/`--check` slagen en zijn vastgelegd in
`verification/github-hosted-runners-20261108.json`. Geen Rust- of QEMU-run
lokaal: die gereedschappen ontbreken in deze uitvoeromgeving.

**Open:** GitHub-CI op de nieuwe PR-head en de bestaande branchregels blijven
van toepassing. De repository was bij controle privé; standaard hosted
runners zijn pas onbeperkt gratis na een afzonderlijke openbaarmaking.
Open PR #245 introduceert nog een self-hosted Copilot-workflow en moet vóór
integratie aan de nieuwe runnerkeuze worden aangepast. De huidige
SMP-run op main `6e429de` faalde alleen bij artifact-upload, niet in de
teststappen: run `42389248097`, job `128453130104`.

**Hervatten:** controleer de actuele head, reviewbevindingen en CI van de
runner-PR; houd oude branchwijzigingen met self-hosted labels buiten de
integratie. Geen runnerregistraties, publicatie- of betaalinstellingen gewijzigd.

## Historisch checkpoint — Discussies en templates — 8 november 2026

**Onderzoeksbron:** main `363b486b06023d3d578f8eca2b23ff49045a1757`;
**integratiebasis:** main `5b2189824c32145ad5c110429517dfcea3f2720f`
(PR #250, openbare projectrichting). Werkbranch
`community/discussions-templates-20261108`. **41 → 41/100**; criteria,
gewichten en bestaand acceptatiebewijs blijven ongewijzigd. Deze opdracht
betreft uitsluitend community-inrichting; CI-, artifact- en publicatiewerk
in andere PR's blijft bij de betreffende eigenaren.

**Gerealiseerd:** de volledige Git-boom (6.524 bestanden) geïnventariseerd en
de bronbestanden tegen Git-blobs gecontroleerd. De Graphify-index kwam met
deze main overeen. Actieve contracten, codegebieden, tests, bestaande
templates en open issues/PR's onderzocht; geen regel-voor-regel securityaudit.
Zes bestaande lege discussiecategorieën ingericht en vijf startdiscussies
gepubliceerd: [wegwijzer #252](https://github.com/simonvanieperen/nieuw-os/discussions/252)
(vastgezet), [release #253](https://github.com/simonvanieperen/nieuw-os/discussions/253),
[desktop #254](https://github.com/simonvanieperen/nieuw-os/discussions/254),
[NAS #255](https://github.com/simonvanieperen/nieuw-os/discussions/255) en
[TLS #256](https://github.com/simonvanieperen/nieuw-os/discussions/256).

**Repositorywijziging:** drie issueformulieren, één Markdown-taaktemplate,
issuekeuzescherm, vier categorieformulieren, PR-template en
[communitywegwijzer](COMMUNITY.md), met links in README en CONTRIBUTING.
Formulieren kennen geen pipeline-labels toe. Alleen beoordeelde pipeline-
taken gebruiken de bestaande letterlijke `Criterium:`-/`Gate:`-regels.
Beveiligingsmeldingen volgen SECURITY.md; geen nieuw meldadres of licentie.

**Controles:** YAML-/veldstructuur en repositorylinks gecontroleerd; een
ingevuld taaktemplate door de bestaande pipelineparser gelezen, inclusief
de weigering zonder uitvoeringslabel. `python3 tools/progress.py --write`
en `python3 tools/progress.py --check`: exit 0, 41/100 consistent.
Categorieformaten en slugs zijn in GitHub bevestigd. Brongebonden overzicht:
[communitycontrole](verification/community-20261108.json).

**Open bij dit checkpoint:** [PR #258](https://github.com/simonvanieperen/nieuw-os/pull/258)
is na de conceptfase gereed voor review; squash-auto-merge is ingesteld.
Opname volgt de vereiste checks en branchregels; daarna de live formulierweergave controleren.
Een branchbestand maakt een template nog niet actief. Geen nieuwe Rust-,
QEMU-, fysieke of menselijke acceptatie uitgevoerd of geclaimd.
**Eerstvolgende stap:** de checks op de geïntegreerde PR-head volgen;
na opname de issuekeuze en vier discussieformulieren in GitHub openen. Wijzig geen branchregels om dit af te dwingen.

## Historisch checkpoint — openbare projectrichting — 8 november 2026

**Bron:** `main` op `363b486b06023d3d578f8eca2b23ff49045a1757`; documentatiebranch
`docs/public-development-direction-20261108`. **41 → 41/100**; alle criteria,
gewichten, statussen en bestaande bewijsverwijzingen blijven gelijk.

**Bevestigd besluit:** Nieuw OS wordt openbaar ontwikkeld. Het bestaande
MPL-2.0-plan voor eigen OS-code is het uitgangspunt voor de licentiecontrole.
Dit staat in `LICENSE-PLAN.md`, ADR-PUBLIC-DEVELOPMENT-20261108, de README en
`implementation/EUROPE-AND-RELEASE.md`. De vijf bestaande licentievoorwaarden
zijn niet afgetekend. Het meegebundelde OSDev-/boekmateriaal heeft afzonderlijke
rechten en is expliciet onderdeel van de nog open publicatiecontrole.

**Controle en bewijsgrens:** 56 actuele bron-/bewijsbestanden zijn via GitHub
opgehaald en byte voor byte tegen hun Git-blob-ID gecontroleerd. De bestaande
voortgangsgenerator en documentatievergelijking zijn op deze bron uitgevoerd;
uitkomsten staan in [het controlerapport](verification/public-development-20261108.json).
Dit is documentatiecontrole, geen volledige licentieaudit, secretscan,
nieuwe Rust-/QEMU-test, menselijke juridische beoordeling of productacceptatie.

**Open:** rechten-/componentinventaris, licentietoepassing, passende notices,
bijdragenvoorwaarden en de overige publicatiecontroles. De repository was
bij controle privé. GitHub-hosted runners worden behandeld in
[PR #249](https://github.com/simonvanieperen/nieuw-os/pull/249); deze opdracht
verandert geen workflows, zichtbaarheid of branchregels. R03 blijft open.

**Eerstvolgende inhoudelijke stap:** voer de vijf bestaande controles in
`LICENSE-PLAN.md` uit op de werkelijk te publiceren bron en geschiedenis,
inclusief alle workspaces en `.agents/skills/os-dev/`.
Concrete publicatie volgt pas nadat de controles zijn afgehandeld.
Eerstvolgend documentatiecommando: `python3 -B tools/progress.py --check`.

## Historisch checkpoint — PR #211 parserintegratie — 7 november 2026

**Bron:** `verify/parser-hardening-20261105`, integratiecommit
`69265f4b1d1220ea3eee6a9358ca09dc5a8b5e7d`; ouders PR `328ac91d` en main
`9bc5f527`. **41 → 41/100**; geen criterium gewijzigd. Deze opdracht behandelt
uitsluitend PR #211; PR #235 wordt in een andere sessie afgehandeld.

**Gerealiseerd:** main zonder wijziging van de negen PR-bestanden opgenomen.
De bestaande Unraid-workflows, gates en branchregels blijven behouden.
Drie reparaties: STOR-1, NET-2 en LOAD-1. De NET-1-smoltcp-canary bewaakt een
nog open defect. Bewijsformulering over wrap zonder overflowcontroles
gecorrigeerd naar codeafleiding; die route is niet opnieuw gemeten.

**Werkelijk gecontroleerd op integratiebron `69265f4b`, vóór dit documentatiecheckpoint:**
de verschillen met de toenmalige main zijn exact de negen PR-paden; 55 lokaal geraadpleegde bron-/bewijsbestanden zijn tegen hun Git
blob-ID gecontroleerd. `python3 -B -m unittest discover -s tests
-p 'test_volume_*.py'`: exit 0, 34 tests. Dit is hostbewijs, geen VM-run.
Receipt: `docs/verification/parser-fuzz-20261105/integration-20261107.json`.

**Open bij dit checkpoint:** nieuwe-head CI, directe netwerk-/volume-/update-
gates en actuele reviews. Lokaal zijn Rust en QEMU niet beschikbaar. De oude
rode CI geeft geen geldige acceptatie van deze nieuwe bron. Er zijn geen
workflowchecks versoepeld of quota-artefacten verwijderd. De PR-beschrijving
registreert de uiteindelijke CI-resultaten met SHA en run-URL.

**Eerstvolgende controle:** status, jobs en logs van PR #211 op de actuele
head; bij een gewijzigde main de integratie opnieuw controleren zonder het
werk van PR #235 te overschrijven. Alleen squashen als branchregels, inhoud,
passende tests en concrete reviews dit toelaten.

**Main-verversing binnen dezelfde opdracht:** de inmiddels gemergde PR #235
op main `5cf3d3fa17eed4005e5f97ce9ba6da84cbb3280d` is opgenomen bovenop
`5120d9ef`. De acht product-/testbestanden van PR #211 blijven byte voor byte
behouden. De milestone-runnerfix, PR235-receipt en onderstaande historische
overdracht blijven aanwezig. De 34 hosttests en het oorspronkelijke receipt
blijven aan integratiebron `69265f4b` gebonden; controles op de nieuwe
gezamenlijke head moeten afzonderlijk slagen.

## Historisch checkpoint — PR #235: buildpaden en CI-basis — 7 november 2026

**Bron:** PR #235, branch `fix/tm-ipc-probe-build-dir`, oorspronkelijke head
`bff194b51d51a9702ee9fb9ad4cf3de9580327fb`, geïntegreerd met main
`9bc5f5270ffe2a8f4a214dd4d58c4290a4de6bec`. **41 → 41/100**; geen criterium wijzigt.

**Gerealiseerd:** de vier `BUILD`-padcorrecties blijven behouden; `test_m2`
en `test_m4` hebben nu een docstring. De bestaande CI-reparatie uit #241
komt via main mee, inclusief de afgebakende Unraid-runnerlabels. Er zijn
geen nieuwe workflow- of branchregelwijzigingen.

**Controle:** Python-syntax en AST-vergelijking slagen; buiten de twee nieuwe
docstrings is de uitvoerbare runner gelijk aan de oorspronkelijke PR-head.
Rubric, criteria, gewichten en bewijsverwijzingen blijven gelijk.
Bronhashes en bewijsgrenzen: [PR235-review](verification/pr235-review-20261107.json).
**Niet uitgevoerd in deze sessie:** Rust-/QEMU-gates; Cargo en QEMU ontbreken.
De oorspronkelijke PR meldt een geslaagde M2–M4-run; dit is hier niet opnieuw bewezen.

**Open:** nieuwe `host-quality` en `qemu-smoke` op de bijgewerkte PR-head,
reviewcontrole en squash-auto-merge binnen de bestaande main-regels. De
oude jobs zonder stappen gebruiken nog de hosted-runnerconfiguratie;
herstart die niet. Optionele artifact-uploadfouten blijven afzonderlijk zichtbaar.
**Eerstvolgend commando:** `gh pr checks 235 --repo simonvanieperen/nieuw-os`.

## Historisch checkpoint — CI-herstel: Unraid-runners en artifactquota — 6 november 2026

**Bron:** branch `ci/unraid-runner-labels` op main `d18aac4e`, PR #241. **41 → 41/100**;
geen criterium wijzigt. Besluit: ADR-CI-RUNNERS-20261106.

**Gerealiseerd:**
- Alle 21 workflows draaien op `[self-hosted, Linux, X64, unraid, build]`. Het oude label
  `[self-hosted, Linux]` matchte ook `MSIMON-wsl`. Dat is WSL op de persoonlijke ontwikkel-pc, waar
  PR-code als gebruiker `simon` draaide.
- `upload-artifact` bewaart nu 3 dagen.
- `apt-get` draait alleen als `dpkg` de pakketten mist.
- PR's die alleen `docs/**` of `*.md` in de root wijzigen, slaan de niet-verplichte gates over.

**Diagnose (6 nov, GitHub-API en joblogs):**
- ~75% van de rode jobs faalde alleen op de uploadstap met "Artifact storage quota has been
  hit" (~6 GB live). De teststappen zelf waren groen. Dit is een infrafout.
- Jobs met 0 stappen van vóór ~19:00 UTC: budgetstop van de hosted runners.
- Echte codefout: `test_gs_discipline` in #238.
- Onbeslist (code of capaciteit): 8-vCPU-timeout in native-smp op de K06-keten.

**Ingrepen met toestemming van de projecteigenaar:**
- Runnerservice `MSIMON-wsl` gestopt (`systemctl` is inactive, GitHub meldt hem offline).
- 38 wachtende PR-runs geannuleerd.
- Artifacts ouder dan 24 uur verwijderd.

**Open:**
- CI-resultaat van #241 op de Unraid-runners.
- PR's #213, #210 en #233 brengen eigen workflows met `ubuntu-24.04` mee. Die moeten op
  de unraid-labelset voor ze mergen.
- PR-integratievolgorde: zie de PR-beschrijving van #241.

**Eerstvolgend commando:** `gh pr checks 241 --repo simonvanieperen/nieuw-os`.

## Historisch checkpoint — K06 stap 5a, ticketlock, irqsave en lockdep — 6 november 2026

**Bron:** branch `feat/k06-5a-ticketlock-lockdep` op main `2c7edaf5`; spec
`docs/implementation/K06-SMP.md` (§4, stap 5a). **41 → 41/100**; K06 blijft open.
Ontwerp: ADR-074.

**Gerealiseerd:** `crates/smp-lock` (ticketlock en lockdep),
`kernel/src/m2/lock.rs` (`IrqLock`), een GS-anker voor de BSP, een BSP-zelftest
(`SMP:LOCKDEP:PASS`) en het negatieve profiel `test-smp-lockdep-order`. De
AP-stresstest draait nu op de ticketlock.

**Bewijs (lokaal):** `tools/gate.sh` exit 0, met 483 hosttests waarvan 8 nieuw.
`test_smp_serial.py` 6/6. Kernelbuilds `native-smp`, `test-smp-lockdep-order`
en `milestone-m2` slagen. `cargo fmt --check` en clippy op `smp-lock` zijn schoon.
**Niet uitgevoerd:** native QEMU (geen QEMU in deze WSL-omgeving). Die staat
open voor `native-smp.yml` op de PR.

**Open:** stap 5b (KERNEL-lock om M3-syscalls, scratchslot per CPU, ADR-075)
t/m 5f; zie de spec §8.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-test-smp-lockdep-order"; python3 -B tools/build.py --kernel-feature test-smp-lockdep-order && python3 -B -u tools/run_smp.py --profile test-smp-lockdep-order`.

## Historisch checkpoint — K06 stap 4, gepreëmpteerde kernelthreads op AP's — 5 november 2026

**Bron:** `c308b1ee9d8deedcfacd923c407392222ff7a12f` op `k06/smp-step4-preempt`. De keten is #191
(`0cfcdd4a`) → #201 (`64f8abc6`) → #202 (`58e11fd0`) → #203 (`e49e7819`) →
stap 4. **41 → 41/100**; K06 blijft open. Ontwerp: ADR-073.

**Gerealiseerd:** op elke AP preëmpt de LAPIC-timer twee kernelthreads, met
een registercanary, voortgang tussen twee metingen en bevriezing na STOP.
Een AP met een dode timer leidt tot quarantaine. **Bewijs:** 14/14 native
gevallen; regressie K05 6/6, `milestone-m2` en `native-rtc` 5/5. Zie
[k06-smp-preempt](verification/k06-smp-preempt/README.md).

**Open:** merge van de keten (beslissingsbevoegdheid: projecteigenaar). Daarna volgt de
eigenlijke SMP-scheduler voor processen: per-CPU-runqueues, lockvolgorde,
interrupts op de BSP, een CR3-wissel per thread met shootdown voor de
hele procesadresruimte, en de herziening van ADR-024.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-native-smp"; python3 -B tools/build.py --kernel-feature native-smp && python3 -B -u tools/run_smp.py --profile native-smp`.

## Historisch checkpoint — K06 stap 3, TLB-shootdown vóór frame-hergebruik — 5 november 2026

**Bron:** `9c64c1dbe85290501934331a28a9ec7f61f39399` op
`k06/smp-step3-shootdown`. De keten is #191 (`5263a5ef`) → #201
(`cd78ea74`) → #202 (`8c694ef0`) → stap 3; main is `c7a61d11`.
**41 → 41/100**; K06 blijft open. Ontwerp: ADR-072.

**Gerealiseerd:** een vervangen kernelframe wordt pas vrijgegeven nadat
elke AP beide vertalingen (het venster en de identity-alias) met `invlpg`
heeft ongeldig gemaakt en de generatie van de ronde heeft bevestigd.
Daarna volgen `free`, een volledige audit en een zichtbaarheidsprobe. Een
ontbrekende ack zet het frame in quarantaine en reset alle AP's.

**Bewijs:** 13/13 native QEMU/TCG-gevallen, waaronder een dove AP en een
controle zonder shootdown. Bij die controle lezen alle AP's
deterministisch het vrijgegeven frame. Regressie: K05 6/6, `milestone-m2`
en `native-rtc` 5/5. Zie
[k06-smp-shootdown](verification/k06-smp-shootdown/README.md).

**Open:** besluitvorming over #191 door de projecteigenaar. Volgens de integrator
loopt #191 achter op main, draaide de volledige CI daar niet, en staan er 6
CodeRabbit-threads open plus de `run_cap_bench`-regel; de integrator kan
niet op die branch schrijven. Daarna komen CI en integratie van
#201/#202/stap 3. Volgende K06-stap: interrupts op de BSP en een
SMP-veilige scheduler, met per-CPU-runqueues, lockvolgorde, procesbrede
shootdown per CR3 en de herziening van ADR-024.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-native-smp"; python3 -B tools/build.py --kernel-feature native-smp && python3 -B -u tools/run_smp.py --profile native-smp`.

## Historisch checkpoint — K06 stap 2, eigen TSS/IST en IPI's per AP — 5 november 2026

**Bron:** `486371e5a3790c7c1df817820ff4cc373c5fc42c` (stap-2-code `1993ade8` + #201-stackfix `36703f50` + main `8bb15309`) op
`k06/smp-step2-percpu-ipi`. Dat is gestapeld op #201
(`k06/smp-ap-bringup`, `047acf39`), dat op #191 staat. Main is `8bb15309`.
**41 → 41/100**; K06 blijft open. Ontwerp: ADR-071.

**Gerealiseerd:** per AP een eigen GDT/TSS (RSP0, IST1 #DF, IST2 NMI) en
een GS-anker. Een IPI-vector 0xf0 met EOI, unicast-rondes met een exacte
telling, parkeren via IPI, en quarantaine van alle AP's als er één doof is.
Een AP met een afwijkende IA32_APIC_BASE wordt afgewezen.

**Bewijs:** 11/11 native QEMU/TCG-gevallen over zes profielen, waaronder
NMI en #DF op de eigen AP-IST-stack en een dove AP met quarantaine.
Daarnaast host-tests, Clippy en fmt, plus regressie van `milestone-m2` en
`native-rtc` (5/5). Zie [k06-smp-ipi](verification/k06-smp-ipi/README.md).
Review door AI; alle bevindingen zijn verwerkt.

**Open:** CI-beoordeling van #201 en de nieuwe stap-2-PR, plus integratie in
de volgorde #191 → #201 → stap 2. Volgende K06-stap: een
TLB-shootdownprotocol op deze IPI's, daarna BSP-interrupts, de
SMP-scheduler en de herziening van ADR-024.
**Eerstvolgend commando:**
`export NOS_BUILD_DIR="$PWD/build/k06-native-smp"; python3 -B tools/build.py --kernel-feature native-smp && python3 -B -u tools/run_smp.py --profile native-smp`.

## Historisch checkpoint — K06 stap 1 en conflictintegratie — 5 november 2026

**Historische bron van deze conflictintegratie:** PR-head
`cd78ea74f14d8b8921aad51cc46a41c41274ab45` op `k06/smp-ap-bringup`,
geïntegreerd met base `main` op
`7dd91fad6d341d0c7b66b885bbe8713d2020e1df`. De PR-head bevat ook de
device-exec/ACPI-handoff-integratie. Drie statusconflicten zijn opgelost met
behoud van de K06-projectstatus. **41 → 41/100**: geen criteria of gewichten
gewijzigd; K06 blijft open. De actuele PR-head en -base staan in PR #201
en in `docs/verification/k06-smp-bringup/native-report.json`; deze sectie
wordt niet bij iedere update-merge herschreven.

**Gerealiseerd:** een opt-in M2-profiel `native-smp` (ADR-070) met
MADT-inventaris, INIT-SIPI-SIPI, een nooit gemapte SIPI-pagina, AP-stacks met
guards in het kernelimage, contextclaims via CAS met fail-closed time-out en
INIT-quarantaine, registergelijkheid met de BSP en een begrensde
atomic- en spinlockproef. Een native gevonden BSP-stack-overloop door een
by-value MADT is hersteld; die zou alle profielen hebben geraakt.

**Bewijs:** 8 native QEMU/TCG-gevallen geslaagd (1/2/4/8/9 vCPU's, single-
en multi-TCG, capaciteit, dubbele MADT via `-acpitable`, AP-stall,
registermismatch). Daarnaast 25 boot-protocol-tests (exit 0; 28 na de #191-tests), 4+2+10+8
Python-harnastests, Clippy/fmt en regressie van `milestone-m2` en
`native-rtc` (5/5). Zie
[k06-smp-bringup](verification/k06-smp-bringup/README.md). Twee
AI-reviews, geen menselijke audit: alle concrete bevindingen zijn verwerkt.

**Controles na integratie:** `cargo +1.90.0 test --locked -p kernel-core`
en de relevante USB-, PS/2- en Python-tests; formatting, Clippy, freestanding
driverbuilds en native-smp kernelbuild geslaagd. `tools/progress.py --check`
geslaagd. CodeQL op de eerste mergecommit `c014b17d` vond geen
Actions/Python-alerts maar liep af vóór alle talen klaar waren; de
geresynchroniseerde head is dus niet volledig gescand. Nieuwe-head hosted CI
moet nog worden beoordeeld.
**Open:** volledige CodeQL-dekking en native K06-QEMU-bewijs op de geïntegreerde
head.
Voor K06 nog nodig: per-CPU-staat, TSS/IST, IPI/TLB-shootdown, SMP-scheduler,
lockvolgorde en herziening van ADR-024. Er is geen fysiek hardwarebewijs.
**Eerstvolgend commando:**
`NOS_BUILD_DIR="$PWD/build/k06-native-smp" python3 -B tools/build.py --kernel-feature native-smp`
en daarna, met dezelfde `NOS_BUILD_DIR`,
`NOS_BUILD_DIR="$PWD/build/k06-native-smp" python3 -B -u tools/run_smp.py --profile native-smp`
(`run_smp.py` bouwt niet zelf).
## Historisch checkpoint — sessie-/bibliotheekintegratie (#206) — 5 november 2026

**Basis:** main `bce6cabc71184d90fc228fe2ae8215339ff7bf28`, branch `integrate/20261105-session-library`
(PR #206). **41 → 41/100.**

**Opgenomen uit #135/#147:** `ui/native`-bibliotheek, `window-session`-shell,
`document-session`-volumeuitbreiding, `ui/graphics`-reflow/damage, kernelfeature
`native-session-route` (alleen diagnostische tellers) en native bibliotheekgates in
`native-window-session.yml`. **Niet opgenomen:** het sealed-generatie-0-volumeontwerp
uit #133, dat is ingehaald door #148/ADR-064 en #205, samen met de dubbele ADR-064
en het bijbehorende bewijs. **Integratorfixes:** Escape annuleert een hangende
ontgrendeling, `Font::advance` schaalt mee met magnify, en de `DropRead`-test raakt nu
werkelijk een late read.

**U03:** het branchbewijs `docs/verification/materiaal-library-20261102/` claimt
41→44, maar is hashgebonden aan `64044318`. Op main blijft U03 open, totdat de native
bibliotheekgates op de main-merge groen zijn en de eigenaar dit bevestigt.
**Open vervolg (CodeRabbit op #206):** hoverdamage alleen bij een hoverwissel,
`hit()`-focus op uitgeschakelde editorknoppen, een positieve checkpointtest in
`test_window_library.py`, de provenance-controles in `verify-ci-artifact.py` en
documentatiebronnen.

## Historisch checkpoint — integratie PR-wachtrij — 5 november 2026

**Basis:** main `c7a61d1126caa346a128fba3004fe4add4ccff61` (na #198), branch `integrate/20261105-platform-batch`
(PR #199). **41 → 41/100**; geen criterium volledig geaccepteerd.

**Gemerged:** #198 = #179 #183 #187 #190 (USB-framing, HID boot reports en
-interfacebinding, PS/2-health; hostmodellen). **In #199:** #169 #173 #176 #182
#188 #189 #193 #194 (acht zelfstandige platformcrates, nu in de
`component-leaves`-matrix; voorheen draaide CI er geen enkele test van),
#165 (EXIF in tile-viewer), #150 (agentprofielen) en #195 (instructieopschoning).
Lokaal (Rust 1.90.0): test, clippy `-D warnings`, fmt en freestanding build per
crate exit 0; onafhankelijke AI-reviews MERGE (geen menselijke audit).

**Open:** #155 vraagt een besluit van de projecteigenaar (score 41→39, B01-criterium); #181, #191
en #177 FIX (reviewcomments op de PR's); calculator #184/#186/#192 onderling
CONFLICT (26+ hunks, ook in `tools/run_window_session.py`); #133, #135, #146,
#147, #154, #157 nog niet beoordeeld; concepten #167 #185 #197 niet overgedragen;
#196 overbodig na het besluit om alleen de Graphify-MCP te gebruiken.
**Bewijsgrens:** geen native app-launch, fysieke USB/HID of menselijke test.
Eerstvolgend commando: `python3 tools/progress.py --check`.

## Historisch checkpoint — instructieonderhoud — 4 november 2026

**Basis:** `8bb153092e967aae314ecde35bcf963565a87ac7`, branch
`copilot/dream-curate-memories-20261104132442`. Vijf instructiebestanden
verliezen verouderde beperkingen op opslaglocaties, native UI-werk en
voortgangsrapportage. De actuele privacycontracten, lokale AI, apprechten,
eigen kernel en bewijsgrenzen blijven ongewijzigd. **41 → 41/100**:
geen runtimecode, criterium, gewicht of nieuwe productacceptatie.

**Controles (alle exit 0):** `python3 tools/progress.py --write`,
`python3 tools/progress.py --check`,
`python3 -B .agents/skills/os-dev/scripts/validate_skill.py`,
`python3 -B -m unittest discover -s .agents/skills/os-dev/tests -v`
(zes tests) en `git diff --check`. De onafhankelijke AI-tegenvoorbeeldreview
vond geen concrete weerleggingen; dit is geen menselijke audit.
**Bewijsgrens:** documentatieonderhoud; geen OS-build, QEMU-, hardware- of
menselijke acceptatie uitgevoerd. Deze controles bewijzen geen OS-functionaliteit.
De bestaande productblokkades en historische receipts hieronder blijven gelden.

**Vervolg:** beoordeel de concept-PR en haar actuele controles vóór integratie.
Eerstvolgend lokaal commando: `python3 tools/progress.py --check`.

## Historisch checkpoint — PR158 met main 95d79451 — 3 november 2026

**Bronnen:** PR-head `04d727c306d13e03a192d9d002a7d44348841c9f` en
main `95d7945110ce853a1b5827ec59f6115475691805`, branch
`fix/b03-test-anchors-guard`. De drie documentconflicten behouden de
B03- en PR138-checkpointgeschiedenissen en hun brongebonden bewijs.
De automatisch geïntegreerde Rust-wijzigingen uit main zijn uitsluitend
commentaar; de B03-ondertekenpolicy blijft ongewijzigd.
**41 → 41/100**; geen nieuw criterium, gewicht of native acceptatie.
De historische PR148-integratiereviewvelden blijven expliciet historisch.

**Validatie op de opgeloste boom:** 78 Rust-hosttests en 14 Python-harnastests
geslaagd; gerichte Clippy en beide formatcontroles exit 0. De eerste
parallelle Rust-commando's raakten een toolchain-downloadrace; na voltooide
installatie en één herstel van ontbrekende rustfmt/Clippy zijn alle controles
opnieuw geslaagd. Geen dependency of lockfile gewijzigd.
Vergelijkingen bevestigen exact behoud van de B03-code uit de PR en van de
drie gedocumenteerde Rust-bestanden uit main. Exacte commando's, logpaden,
exitstatussen en hashes staan in
[integratiebewijs](verification/pr158-main-integration-20261103.json).
`tools/progress.py --write` en `--check` slagen; de SVG blijft bytegelijk.
Native builds en QEMU niet herhaald voor deze documentintegratie.
Oud testbewijs geldt niet automatisch voor deze integratie.

**Vervolg:** na publicatie nieuwe-head CI laten beoordelen; geen oude
CI-run herstarten. Productiesleutelbeleid en issue #41 blijven open.
Eerstvolgende lokale controle: `python3 tools/progress.py --check`.

## Historisch checkpoint — B03 TEST-ankergrens — 3 november 2026

**Basis:** `main` `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e`; codecommits
`07e7f9a0471798c49e1844bf885f94491bb481d8` en
`ceb957557af821bb95786aa505b5df41077f7403` op
`fix/b03-test-anchors-guard`. `test_policy` vereist nu de expliciete
`test-signing-anchors`-feature; UEFI-loader- en update-servicebuilds zonder
gekozen ondertekenbeleid falen tijdens compilatie. TEST-sleutels blijven
uitsluitend fixtures. Productieankers, sleutelrotatie/-intrekking en
releaseceremonie zijn niet ontworpen of bewezen; #41 blijft open.

**Gerichte controles:** update-core met TEST-feature 13 tests, update-tool
2 tests, update-core zonder TEST-feature 7 tests en B03 Python-harnas 14 tests
geslaagd. De testloader en update-service zijn crossgebouwd. Negatieve
compilatiechecks voor de ongemarkeerde loader en service falen verwacht met
alleen de bedoelde policyfout. Update-core/update-tool- en service-Clippy
slagen; loader-Clippy slaagt met alleen de bestaande
`clippy::not_unsafe_ptr_arg_deref`-lint toegestaan; de ongewijzigde pointer
entrypoint is geen onderdeel van deze wijziging. Zie de exacte commando's en
bewijshashes in
[B03-verificatierapport](verification/b03-test-anchor-gating/README.md).
Een directe release-`cargo build` buiten de projectwrapper raakte daarnaast
een LLVM SIGILL tijdens codegeneratie voor `curve25519-dalek`; de
voorgeschreven update-service-buildwrapper slaagde wel en is het gerapporteerde
buildresultaat.

**Native vervolgbewijs:** op productbron `26108a25` is
`python3 -B -u tools/test_update.py` uitgevoerd: exit 0, 106,91 s,
alle 18 QEMU-stappen en de afsluitende normale build geslaagd. Zie
[native-sprint.json](verification/b03-test-anchor-gating/native-sprint.json)
voor de exacte omgeving, hashes en bewijsgrens. Evidencehead `568741f3`
had 31 geslaagde checks en een overgeslagen check. Geen hardwaretest,
productiesleutelbeleid of nieuwe productcriteria geaccepteerd.

**Voortgang:** branchscore **41 → 41/100**; `tools/progress.py --write`
en `--check` bevestigen deze berekening. Er zijn geen criteria of gewichten
gewijzigd en deze wijziging levert geen productacceptatiepunten op.

**Eerstvolgend:** de gerichte sprintcorrectie na actuele-head-CI integreren;
#41 blijft open voor productiesleutelbeleid en bijbehorende acceptatie.

## Historisch checkpoint — PR138 met main d686bd54 — 3 november 2026

**Bronnen:** PR-head `fc3b5e2522e04f636bdb443299da6e8f99161afe` en
main `d686bd54e766aed542e9918d1a37dcce434f18e2`, branch
`simonvanieperen-laughing-fortnight`. Alleen PROJECT-STATE.md conflicteerde;
beide checkpointgeschiedenissen blijven behouden. Code is automatisch
samengevoegd zonder handmatige runtimewijzigingen. **41 → 41/100**;
geen criterium, gewicht of historisch testbewijs gewijzigd.

**Uitgevoerd op de opgeloste boom:** 87 Python-harnastests en 76
Rust-hosttests geslaagd; gerichte Clippy en workspace-formatcontrole exit 0.
De `native-volume`-cross-build van UEFI-loader, kernel en volume-apps slaagt
na één herstel van ontbrekende, reeds vergrendelde SDK-downloads. De eerste
offline build mislukte wegens ontbrekende `zeroize v1.9.1`; geen dependency
of lockfile gewijzigd. Codevergelijkingen bevestigen exact behoud van main
voor kernel/volume/harnassen en van de PR voor crates/window-session.
Exacte commando's, exitstatussen en loghashes:
[integratiebewijs](verification/pr138-main-integration-20261103.json).
Historische receipts bewijzen niet automatisch deze opgeloste boom.
QEMU niet uitgevoerd: `qemu-system-x86_64` ontbreekt. Geen nieuwe
native-runtime-, release-, fysieke-hardware- of menselijke acceptatie.

**Vervolg:** na publicatie de nieuwe-head-controles laten beoordelen.
De oude native-foundation-run wordt na deze integratie niet herstart.
Eerstvolgende lokale controle: `python3 tools/progress.py --check`.

## Historisch checkpoint — PR138 documentatieconflicten — 3 november 2026

**Bronnen:** PR-head `d263acdeb9f94905c52457cccfeafabdc3011571` en
main `5c66ec57aaf8eb411a83b8d6e42320ddc379383c`. De vier conflicterende
statusbestanden behouden beide geschiedenissen; de laatst vastgelegde
beoordeling blijft **41/100**. Historische controles blijven aan hun eigen
bron gebonden en gelden niet automatisch voor de samengevoegde boom.

**Vervolg:** beoordeel de controles op de nieuwe mergecommit. Volledige
release-, fysieke-hardware- en menselijke acceptatie blijven open.
## Historisch checkpoint — PR148 met main 8e0c15d — 3 november 2026

**Basis:** PR148 `47ec535f15bb10912dbc6cbf1af3de9df0c6c226` met main
`8e0c15d8c71ec4bd4e946e14dbf58bcfe748baa2`, branch
`simonvanieperen-upgraded-disco`; schone werkboom bij aanvang.
Drie centrale documentconflicten behouden beide checkpointreeksen en de
brongebonden receipts. Gedeelde harnashulpfuncties komen exact uit main;
volume-admissie blijft uit PR148. Geen handmatige codewijziging.
**41 → 41/100**, geen criterium of gewicht gewijzigd.

**Uitgevoerd op de opgeloste boom (alle eindcommando's exit 0):**
- `python3 -B -m unittest discover -s tests -p 'test_common.py'`: 8 tests.
- `python3 -B -m unittest discover -s tools/tests -p 'test_random.py'`: 21 tests.
- `python3 -B -m unittest discover -s tools/tests -p 'test_images.py'`: 20 tests.
- `python3 -B tools/test_fpu.py`: 12 tests.
- `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'`: 26 tests.
- `python3 tools/progress.py --write` en `python3 tools/progress.py --check`.
- `git diff --check`; vergelijking met beide parents bevestigt exact behoud
  van de gewijzigde harnassen uit main en de volume-implementatie uit PR148.

De eerste discoverypogingen voor random/images/fpu onder `tests` vonden
geen tests (elk exit 5); daarna zijn de werkelijke bovenstaande entrypoints
uitgevoerd. Verwachte foutmeldingen in negatieve harnastests zijn geen
native uitvoering. Uitvoer staat in het sessietranscript, zonder afzonderlijke
logbestanden. Rust-builds en QEMU niet herhaald; geen native acceptatie.
<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:69f0ce3ef4e7240e6d8b1e44a29d90b2c36c7dcd2008b6a678a3f075e30514e8 -->

**PR148-integratiereview: open.** De assessmentbasis is PR148 `47ec535f`;
de tweede mergeouder is main `8e0c15d8` en de resulterende merge is `7412d0bb`.
De hierboven gemelde gerichte controles hebben alleen uitvoer in het
sessietranscript; een afzonderlijk brongebonden integratiereviewbewijs ontbreekt
in de checkout. De PR148-hostreceipts hieronder betreffen eerdere werkbomen en
bewijzen deze integratiereview niet. De `reviewed_commit`-verwijzing naar
`cc1dea7` in `docs/progress.json` is een historische review van de eerdere
verduidelijking van documentatiestatus en hostbewijs.

**Eerstvolgend:** nieuwe-head CI en review beoordelen en passend reviewbewijs
vastleggen. Native volume-acceptatie blijft afzonderlijk open.

<!-- /coderabbit-ai-generated:v1 -->

## Historisch checkpoint — PR142 main-integratie — 3 november 2026

**Basis vóór integratie:** PR-head `37ad1e98328e59b752c230310a4abeaca749b669`;
main `5c66ec57aaf8eb411a83b8d6e42320ddc379383c`. De enige conflicthunk was
`docs/verification/retained-document-heads-20261103.json`. De opgeloste
snapshotlijst behoudt alle unieke PR131/132-, PR141- en PR142-ingangen;
PR149's code/tests en receipts zijn eveneens behouden uit main. Geen receipt
wordt als bewijs voor deze nieuwe mergeboom aangeduid.

**Uitgevoerd op de opgeloste boom:** `node --test ui/prototype/tests/*.test.mjs`
— 41 geslaagd, 0 mislukt; `node --check ui/prototype/app.js` en
`node --check ui/prototype/state.js` — exit 0. De README-voorgestelde
`--test-isolation=none`-optie wordt niet herkend door Node v22.23.4; de suite
is daarna met de ondersteunde standaardopties geslaagd. Progressmeter is
bijgewerkt: **41 → 41/100**, zonder nieuw criteriumbewijs. Native/QEMU,
fysieke en menselijke acceptatie zijn niet uitgevoerd.
**Open:** finaliseer de merge na JSON/progress- en diffcontroles en secret scan;
na publicatie moeten de nieuwe-head-checks worden geëvalueerd.

## Historisch checkpoint — PR149 derde main-integratie — 3 november 2026

**Basis:** Watch-merge `bb8877e3e1aaec304d7478510b65c8116aeb634e`, met
ouders `416886354dbb0c6135a0740c429a3fbd4530a16d` en main
`99d132768fd72276caa4a3b3e9a09f8f1909f0ed`, branch
`simonvanieperen-reimagined-train`. Vier centrale documentatieconflicten;
de compacte UI-fixes en hun bewijs blijven uit de PR behouden, de
PR139-receipts komen ongewijzigd uit main. Beide historische statuslijnen
blijven behouden en krijgen geen nieuwe bewijsbron toegewezen.
Rubric en score blijven **41 → 41/100**.

**Werkelijk opnieuw uitgevoerd op schone `bb8877e3`:**
`cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml`,
59 hosttests, exit0, 10,816s. Cwd, HEAD, schone status en Cargo.lock-/Rust-
bronhashes vóór én na, exact commando en volledig gehasht log:
[primaire receipt](verification/pr149-bb8877-host-receipt.json).
Dit is een nieuwe run, geen retrospectieve bronbinding van de door Watch
gemelde tooluitvoer/tijden. De integrator controleerde apart rubricbehoud,
ongewijzigde Rust-code ten opzichte van532, één handoff en geen markers.
Een historische PR140-verwijzing naar PR139-bewijs is bij deze integratie
hersteld naar haar eigen oorspronkelijke commando's en bron.
Dit is geen menselijke audit. Geen nieuwe formatter-/Clippy-run,
cross-build, volledige CI of native/QEMU-uitvoering voor deze
uitsluitend documentatie betreffende integratie.
**Open/eerstvolgend:** na publicatie nieuwe-head-controles laten evalueren.
Alleen bewijs-/statuscorrecties; geen oude CI-retry of nieuwe runtimecode.
Native/QEMU-, fysieke en menselijke acceptatie blijven afzonderlijk open.

## Historisch checkpoint — PR148 slot1-admissie opvolging — 3 november 2026

**Basis:** PR148-head `c6b25873c915a98f8c0b2bb7400395e01ba44741`, branch
`simonvanieperen-upgraded-disco`, met slot1-admissiefix, marker-crashgrens
en host-suitebewijs al aanwezig. Score en rubric blijven **41 → 41/100**.

**Status en classificatie (actueel):**
- Vereiste checks (`host-quality`, `qemu-smoke`) zijn groen.
<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:d8b787176e01e7132a22480ebc41537196905df76f55d8e674355a4700a5bc60 -->

- `native-foundation` faalde volgens de handoff in run `41640409938` met
  `system marker sequence mismatch`/`RANDOM guest report ... not bound`.
  De oorzaak is **onbekend**: de primaire workflowbron, de exacte gefaalde
  stap en het resultaat van de gestarte rerun (`gh run rerun --failed 41640409938`)
  ontbreken. De gate blijft open.

<!-- /coderabbit-ai-generated:v1 -->
- Open CodeRabbit-threads betreffen vooral documentatiestatus en bronlabels;
  productiegedrag voor slot1-admissie is al gecorrigeerd en hostmatig bewezen.

**Primaire bronnen (code + bewijs):**
- Recovery/admissie: `services/volume/src/table.rs` op commit `c6b25873`.
- Onafhankelijk orakel: `tools/volume_disk.py` op commit `c6b25873`.
- Bewijs: `docs/verification/pr148-slot1-review.json`,
  `docs/verification/pr148-marker-crash-review.json`,
  `docs/verification/pr148-volume-core-host-suite.json`.
- Repositorybron: https://github.com/simonvanieperen/nieuw-os/tree/c6b25873c915a98f8c0b2bb7400395e01ba44741

**Open/eerstvolgend:** rond PR148 de resterende documentatie-threads sluiten,
uitkomst van de `native-foundation` rerun verwerken en daarna opnieuw
merge-preflight op exact head/base/checks/reviewthreads draaien.
Native metadata-only fixture en native QEMU-volume-admissie blijven apart open.

## Historisch checkpoint — PR149 compacte kaartregressies hersteld — 3 november2026

**Bron:** `53256f82baf097ebaab2497018ef2c37196c8e5d`, boven
layoutfix `f67b1e85` en actuele Watch-parent `5492158d` met main `d115cc96`.
Smalle/hoge viewports gebruiken nu dezelfde compacte geometrie voor
hit-testing en tekenen. De Start-breedtetest volgt de werkelijke
werkbalkbreedte; nieuwe320/400/639/640-pixelgevallen en echte geschaalde
framebufferpixels controleren de grens. Compacte toestandstekst past
binnen de kaart en onderscheidt opslagfouten van volle objectruimte en
andere fouten. Geen autorisatie-/opslagcontract gewijzigd.

**Echt rood/groen:** oude kaart was200 in plaats van96 pixels hoog;
`Ready` veranderde pixel109,163 buiten de kaart en opslag-/niet-opslagfouten
gaven identieke pixels. Na correctie **59 hosttests**, Clippy, formattering
en scalar `x86_64-unknown-none`-librarybuild: exit0.
Volledige relevante logs, exacte commando's, SHA-256 en bronbinding:
[reviewbewijs](verification/pr149-compact-review.json).
De eerste extra pixeltest gebruikte onjuist logische coördinaten bij200%;
die testfout is hersteld, de mislukte poging blijft vermeld.

**Open/eerstvolgend:** `GH_PAGER="" gh pr checks 149`; verse volledige
CI/native-QEMU-/screenshotacceptatie en review op exact de nieuwe head.
Geen gast-run, fysieke/human-/releaseacceptatie of gebruiksmeting geclaimd.
Oude55/57-tests en Watch-meldingen hieronder zijn historisch, niet verse
bronacceptatie. **41 → 41**, geen criteriumwijziging of mandaatvrijgave.

## Historisch checkpoint — PR149 tweede main-integratie

**Bron:** PR-head `b75cfc3c7c5cc16efd618b2d6d179938abdabbdf` met main
`d115cc96c19ac4371baed77409a23d1adddffaf0`, branch
`simonvanieperen-reimagined-train`. Alleen vier centrale
documentatieconflicten; beide checkpointgeschiedenissen behouden.
De artifact-store-tests en bewijsrapporten komen ongewijzigd uit main;
de UI blijft ongewijzigd uit de PR-parent. Score **41 → 41/100**.

**Validatie op de opgeloste boom (exit 0):**
`cargo +1.90.0 test --locked -p artifact-store-core` (9 tests),
`cargo +1.90.0 test --manifest-path ui/native/Cargo.toml --locked`
(57 hosttests), `cargo +1.90.0 clippy --locked -p artifact-store-core --all-targets -- -D warnings`,
`cargo +1.90.0 fmt --manifest-path crates/artifact-store-core/Cargo.toml -- --check`,
`python3 tools/progress.py --write`, `python3 tools/progress.py --check`
en `git diff --check`. De eerste UI-poging stopte met exit 1 doordat de
gelijktijdige Rust-installatie een downloadbestand verplaatste; de herhaling
na installatie slaagde. Bestaande rustfmt/Clippy-componenten hersteld met
`rustup component add --toolchain 1.90.0 rustfmt clippy` (exit 0).
Uitvoer: toolsessies `artifact-tests`, `ui-retry`, `clippy`, `fmt`, `progress`.
Een directe vergelijking met Gitblobs bevestigt de ongewijzigde rubric
buiten assessment voor beide parents, behouden UI uit de PR en exacte
tests/bewijsrapporten uit main; geen conflictmarkeringen (exit 0).
Geen nieuwe cross-build of volledige CI-uitvoering.
**Open/eerstvolgend:** nieuwe-head-controles en open reviewpunten door de
bevoegde vervolgstap laten beoordelen. Deze tick is uitsluitend voor
conflictoplossing; geen reviewafhandeling of PR-merge.
Native/QEMU-acceptatie en menselijke audit zijn niet uitgevoerd.

## Historisch checkpoint — PR149-conflictoplossing met main

**Bron:** PR-head `92fdf2ff941284dfaeacffc9ad49295b2e9da21f` met main
`69809c2cb53dfb3bd77abe5352892810fba6e82d`, branch
`simonvanieperen-reimagined-train`. De vier conflicten betreffen alleen
README, PROJECT-STATE, PROGRESS en het assessment in progress.json.
Beide checkpointgeschiedenissen blijven behouden; de gegenereerde meter
wordt opnieuw opgebouwd. Geen handmatige runtime-, rechten- of testcodewijziging.

**Uitgevoerd op de opgeloste boom (alle onderstaande eindcontroles exit 0):**
`python3 tools/progress.py --write`, `python3 tools/progress.py --check`,
`cargo +1.90.0 test --manifest-path ui/native/Cargo.toml --locked`
(57 hosttests), `cargo +1.90.0 fmt --manifest-path ui/native/Cargo.toml --all -- --check`,
`cargo +1.90.0 clippy --manifest-path ui/native/Cargo.toml --locked --all-targets -- -D warnings`
en `git diff --check`.
`umask 077 && python3 -m unittest discover -s tests -p 'test_storage_backend_replay.py'`
slaagt met 16 tests. De eerste opslagrun met de omgevings-umask faalde
(1 failure, 2 errors: schrijfbare fixturebestanden); de private umask
herstelt de fixturevoorwaarde zonder test-/productiecode te wijzigen.
Rustfmt startte eerst niet door een gelijktijdige toolchaininstallatie;
daarna ontbraken rustfmt/Clippy. Eén `rustup component add --toolchain 1.90.0 rustfmt clippy`
herstelde de bestaande gereedschappen, waarna beide controles slaagden.
De uitvoer staat in de toolsessies `ui-tests`, `storage-tests`,
`storage-private`, `fmt-final` en `clippy-final`; geen raw logs toegevoegd.
Secretscan op de opgeloste bestanden en geïmporteerde Python-code: geen
secrets gevonden. Een tweede AI controleerde alleen-lezen het inhoudsbehoud;
dat is geen menselijke audit.

**Bewijsgrens:** `git diff --quiet HEAD -- ui/native` bevestigt bytegelijkheid
met de PR-parent (exit 0); de vaste rubric buiten assessment is gelijk aan
beide parents. Historische netwerk-, opslag- en native resultaten hieronder
blijven aan hun eigen bron gebonden. QEMU/native gates, cross-build en de
volledige CI zijn deze tick niet uitgevoerd. Score **41 → 41/100**;
geen nieuw criterium, native runtimebewijs of volledige releaseacceptatie.

**Open/eerstvolgend:** na gevalideerde publicatie de nieuwe-head-CI door
GitHub laten beoordelen. Deze tick merge't de PR niet en herstart geen
oude-head-checks. De native preview- en productacceptatie blijven open.

## Historisch checkpoint — native startoppervlak verfijnen

**Bron:** `bd4134c1d565f58736a2c829587f337a415dd47b`, bovenop
`cbbc9c9941b08ea5edfbd3637d494635b4fb1df0` (op dat moment `origin/main`),
branch `simonvanieperen-reimagined-train`.
Het opt-in VM-startscherm heeft nu een duidelijke hiërarchie, een zichtbare
Start-actie en een Materiaalkaart die de echte bibliotheekstatus weergeeft.
De kaart onderscheidt niet-geladen, leeg, fout en bestaand document; de
compacte indeling voorkomt overlap met de navigatiebalk. Er zijn geen
capabilities, authenticatie, opslagrechten of sessieovergangen toegevoegd.

**Gewijzigd:** `ui/native/src/desktop.rs`, `ui/native/src/render.rs`,
`ui/native/tests/render_regressions.rs`, `ui/native/README.md` en
`docs/implementation/SESSION-AND-DESKTOP.md`.

**Uitgevoerd:** Rust 1.90.0 `fmt --check`, `cargo check --tests`,
Clippy (`--all-targets -- -D warnings`) en `git diff --check`: exit 0.
De bestaande scalar `x86_64-unknown-none`-releasebuild van de native UI slaagt
ook (exit 0); het `.rlib` heeft SHA-256
`42eda9397dc739d098b666d9f5a013911fc0b3d867469fde8a558b34b3888b5d`.
Dit is compilebewijs, geen OS-runtimebewijs. De volledige `ui/native`
hosttestsuite slaagt met 55 tests (37 algemene, 5 renderer-, 10 sessie- en
3 fixturetests). Omdat de host `cc` mist, is GCC 15.2/libc6-dev uit Ubuntu
26.04-pakketten alleen naar `/tmp` uitgepakt; er is niets op het systeem
geïnstalleerd. De eerste directe testpoging stopte daardoor op ontbrekende
`cc` (exit 101); de geïsoleerde rerun slaagde. Zonder tijdelijke linker
eindigde preflight met exit 2; met die linker op `PATH` was exit 2 alleen
wegens ontbrekende QEMU en OVMF.
Er is geen QEMU-runtime of native screenshot gemaakt. Commando's, logs en
bewijsgrenzen: `docs/verification/native-start-surface-20261103/README.md`.

**Voortgang:** 41 → 41/100. Geen rubriccriterium gesloten; UI-compilatie en
een toegevoegde test zijn geen native visuele acceptatie. U03/U04 en de
volledige release blijven open.

De vaste meter is opnieuw gegenereerd en gecontroleerd:
`python3 tools/progress.py --write` en `python3 tools/progress.py --check`,
beide exit 0. De assessmentbron is de codecommit hierboven; de huidige
status-/meterupdate is een documentatiecheckpoint daarop.

**Eerstvolgend:** voer bij beschikbaarheid van QEMU en BIOS/VGA-ROM's de
brongebonden sessiepreview uit via `python3 -B tools/run_session_preview.py`
met expliciete `--bios-rom` en `--vga-rom`. De hostrenderer is daarin alleen
oracle; alleen QMP-invoer en pixels uit de echte gast kunnen native bewijs
opleveren. De huidige omgeving heeft QEMU/firmware niet.

## Historisch checkpoint — PR140 bewijs en descriptorclaims — 3 november 2026

**Geteste bron:** `2bf9578290bef88d53c17f9913434da6f27441f2`, inclusief
main `69809c2cb53dfb3bd77abe5352892810fba6e82d`.
Op deze exacte bron zijn met Rust 1.90.0 de 9 artifact-store-contracttests,
`cargo check --tests`, Clippy en formattering opnieuw uitgevoerd: elk exit 0.
Exacte commando's, omgeving, source-tree, codehashes en volledige logs met
SHA-256 staan duurzaam in
[pr140-review-evidence.json](verification/pr140-review-evidence.json).
De tijdelijke logpaden zijn alleen oorspronkelijke uitvoerpaden.

De tests bewijzen descriptorvalidatie, binding en foutdoorgifte, niet
identiteit, herkomst of cryptografische authenticatie. Het historische
rapport behoudt bron `08710d55`, commando's, exits en hashes; alleen de
scopeclaim is gecorrigeerd en het verse bewijs toegevoegd.
Score **41 → 41**, geen nieuwe productacceptatie of productiecodewijziging.

**Open:** verse volledige CI en definitieve review vóór merge. Native/QEMU,
fysieke en menselijke acceptatie zijn niet opnieuw uitgevoerd. PR135's
volume-fixture- en RANDOM-gates blijven afzonderlijk open; bron-PRs niet
blind sluiten. **Eerstvolgend:** `GH_PAGER="" gh pr checks 140`; bij
Watch-headwijziging opnieuw head/base en inhoud controleren.

## Historisch checkpoint — PR139 exacte contracttestreceipt — 3 november 2026

**Werkelijk opnieuw geteste bron:** `0a49a3822e5d872f75573f12cd072e76e1249e1b`,
de geïntegreerde PR139-boom met main `d115cc96`.
Negen artifact-store-contracttests, Rust1.90.0 fmt en Clippy: elk exit0.
Volledige primaire logs, exacte commando's, tree en codehashes:
[contracttestreceipt](verification/pr139-artifact-contract-receipt.json).
Dit vervangt geen native acceptatie; eerdere Watch-testclaims zonder
primaire receipt zijn niet retrospectief als geverifieerd bestempeld.
Replay-/umaskbewijs blijft exact aan732 gebonden en OS-workflowhistorie
uitsluitend aan380. **41 → 41**, geen criteriumwijziging.

**Open/eerstvolgend:** `GH_PAGER="" gh pr checks 139`, daarna live
head/base/threads en inhoudsbehoud hercontroleren vóór guarded squash.
Geen testbewijs stilzwijgend naar een latere Watch-head promoveren.

## Historisch checkpoint — PR139 actuele main geïntegreerd — 3 november 2026

Basis vóór integratie: PR-head `341d1b428d436ac0e33c5bf3d7eb684f8543109c`
en actuele `main` `d115cc96c19ac4371baed77409a23d1adddffaf0`.
Conflicten in README, progress.json, PROGRESS.md en PROJECT-STATE.md zijn
opgelost met behoud van PR139-bronrollen en PR140-statushistorie. Main voegt
de `artifact-store-core`-contracttests toe. Eerder gerapporteerd, destijds
zonder primaire PR139-receipt (dus niet onafhankelijk geverifieerd):
`cargo test --locked
--manifest-path crates/artifact-store-core/Cargo.toml` slaagt (9 tests);
`cargo +1.90.0 fmt --manifest-path crates/artifact-store-core/Cargo.toml
-- --check` en `cargo +1.90.0 clippy --locked --manifest-path
crates/artifact-store-core/Cargo.toml --all-targets -- -D warnings` slagen.
Dit is contracttestbewijs, geen native acceptatie. Voortgang blijft **41/100**.
Er waren geen reviewtargets meegeleverd, dus geen reviewverzoek is afgehandeld.

De gerapporteerde `b03-native-update`-failure hoort bij oude head `341d1b42`.
Na publicatie van deze conflictintegratie is dat bewijs achterhaald; de nieuwe
head moet haar eigen checks krijgen. Reviews zijn volgens de app nog open.
Eerstvolgend: publiceer de gevalideerde merge en laat nieuwe checks evalueren.

## Historisch checkpoint — PR139 primaire receipts en bronrollen — 3 november 2026

**Geteste bron:** `732fa9cca04ced66b06bee4f0e4f57249c09e96a`, inclusief
main `69809c2cb53dfb3bd77abe5352892810fba6e82d`.
De 16 replay-hosttests slaagden met `umask 0077`; dezelfde suite met `umask
0002` faalde echt (1 failure, 2 errors) door groepsschrijfbare fixtures.
Exacte commando's, tree/codehashes en logs: `verification/pr139-review-receipts.json`.
Het historische prioriteitencheckpoint koppelt zijn 15 workflowruns uitsluitend
aan `380111b`; dat bewijs is niet opnieuw uitgevoerd.

## Historisch checkpoint — PR140 main-integratie — 3 november 2026

PR-head `75cda7f4d49c5afe895277ada1102d630fdb75ad` is inhoudsbehoudend
geïntegreerd met main `69809c2cb53dfb3bd77abe5352892810fba6e82d`.
De Nederlandse beschrijvingen in het PR140-hostrapport blijven behouden;
main's documentstapel, opslagherstelmateriaal, audit en replaytests zijn
eveneens behouden. De rubric is ongewijzigd: **41 → 41/100**.

**Destijds gerapporteerd:** `umask 077 && python3 -B -m unittest discover -s tests -p
'test_storage_backend_replay.py'` slaagt (16 tests). De eerste poging met de
omgeving-umask `0002` faalde op drie permission-gebonden fixtures; geen
productiecode gewijzigd. `python3 tools/progress.py --write` en `--check`,
JSON-syntaxiscontrole en `git diff --check` slagen.
Dit historische PR140-resultaat is niet het PR139-bewijs op `732fa9cc`
en is niet opnieuw uitgevoerd voor deze PR149-integratie.
**Toen open:** de nieuwe integratiehead moest opnieuw door hosted controles
worden beoordeeld. Geen nieuwe native/QEMU-acceptatie uitgevoerd.

## Historische context — eerdere main/statusintegratie op cbbc9c99

Main staat op `cbbc9c9941b08ea5edfbd3637d494635b4fb1df0`. De handmatige
squash-merge van #134 is als persoonlijke handeling door de projecteigenaar bevestigd
(3 november, 10:05:36Z).
Geen concurrerende agent of integratieoverdracht; geen rollback gevraagd.
Die mainbron is nu behoudend opgenomen. Drie assessment/statusconflicten
behouden de recentere documentcombinatie; de TCP-productiefix en haar
volledige regressie-/bewijsbestanden zijn ongewijzigd uit main overgenomen.
De mainrubric is buiten assessment exact gelijk aan de behouden41-puntenrubric.

Nieuwste119-head `5a8317f130cc388e661b876b5258c9978e391460` is nu
volledig behouden. Alleen vier centrale statusconflicten, geen nieuwe code.
Alle rubricvelden buiten assessment zijn identiek; twee latere NAS-
checkpointuitvoeringen blijven historisch, niet verse tests van deze boom.
Ook cloudmain-mergeheads136/143/144/145 zijn geconstateerd: hun exacte
centrale checkpoint-/assessmentbytes zijn met Gitblob en SHA-256 opgeslagen
in `verification/retained-document-heads-20261103.json`. Dit is archivering,
geen stil akkoord op nieuwe heads of sluiting van de bijbehorende PRs.

Bron `1552e5ae1a0e053a6df5ef802f6c0e4e78544cf7` bewaart volledige heads
143/d169796a,144/a974fb9f,145/da5918fb en136/f6921e49 als mergeparents
boven PR114-fix720cd431. Unieke README-codeleesintro, vaste14-commit-
changelogrange, beperkt AI-reviewrapport en volledigheidsaudit zijn behouden.
Vier centrale metadata-conflictklassen zijn inhoudelijk opgelost:
rubrics buiten assessment waren identiek; actuele score41 blijft leidend,
oudere unieke status is historisch opgenomen, niet als actieve opdracht.
De securitytekst noemt overal AI zonder bestandsdekking, geen menselijke audit.
Geen runtime-/tool-/testbron verschilt van720cd431 door deze vier merges.

PR114's vier vervolgthreads zijn beantwoord/resolved in720cd431; volledige
originele82+17 Btrfs-manifestbinding en16 hostregressies blijven behouden.
De audit blijft39 groepen/62open FR-rijen; geen nieuwe productpunten.
Nieuwe gecombineerde CI/review blijft vereist vóór guarded squashmerge.
136/143/144/145 nog niet sluiten: controleer opnieuw actuele heads en bewijs
na daadwerkelijke opname in main. Laatste119-head5a8317f1 is nu behouden.
121-b83 blijft apart
met eerder bewaarde historische metadata; geen blinde ancestryclaim.

Andere stack:135/eea1cdcc bevat echteN01-acceptatie op6f15/source1ca en
onafhankelijk gecontroleerde19VM-volumeproeven opce02/source2c410.
Hosted foreignfixtureartifact/run-ID blijft ontbreken; geen gate overgeslagen.
Main is cbbc9c99;39/41/42/43 blijven open. Integratormandaat is niet
overgedragen. Score **41 -> 41**. Eerstvolgend: actuele publicatie-CI/review
afhandelen, bronbehoud controleren en pas daarna geschikte PRs squashen.

## Historisch checkpoint PR119 — tweede en derde statusresolutie

De tweede119-resolutie op052812b4 combineerde ee5003e6 met main380:
overname van bestaande41 punten, geen nieuwe toekenning (branch34 ->41,
main41 ->41). Vier statusbestanden conflicteerden; buiten die bestanden
werd bytegelijkheid met main vastgelegd. De eigen run meldde39 NAS-
hosttests, exit0,0,806s; `/tmp/nieuw-os-pr119-second-nas.log`.
Progress write/check en diff-check exit0; geen nieuwe Rust/QEMU-uitvoering.

De derde119-resolutie5a8317f1 combineerde052812b4 met maincbbc:
drie statusconflicten, behoud van beide geschiedenissen en de TCP-fix.
Progress write/check en diff-check exit0; bytegelijkheid met main buiten
README/PROJECT-STATE/PROGRESS/progress/SVG exit0. De eerdere vergelijking
zonder SVG-uitzondering gaf1 door datum2 ->3nov, geen codeverschil.
Geen nieuwe Rust/native/QEMU-proef of productpunt. Historische volledige
opdrachten, bronversies en beperkingen staan exact in
`verification/retained-document-heads-20261103.json`; score41 ->41.

## Historisch checkpoint PR134 — terminale TCP-ontvangst

Maincbbc9c99 behoudt de oorspronkelijke134-head27cb3948 via squash.
ReceiveTcp geeft na connect-timeout Closed, terwijl State TimedOut behoudt;
buffer-/tokencontroles blijven intact en opnieuw openen vernieuwt het token.
De oorspronkelijke test was host-rood (Again versus Closed), daarna10
servicetests groen plus35 broker-,26 stack-,4 protocol-,6 client- en11
peerharnastests, format/Clippy/native-librarybuild.
Historische normal/client-exit op5925c915/source9a8b521f: raw33,
8,27/7,79s, peer compleet en cleanup. Dat was nog geen native bewijs
van de nieuwe timeout-ontvangstcase; die beperking blijft expliciet.
`verification/issue-43-receive-timeout.{json,md}` bewaart exacte opdrachten,
hashes en oorspronkelijke open hosted-CI/reviewgrenzen. AI-read-onlycheck
is geen menselijke audit. Geen nieuwe rechten/dependencies of punten.
De latere135-gastasserties en run41614590388 staan afzonderlijk beschreven.

## Historisch checkpoint — PR114-vervolgreview — 3 november 2026

Nieuwe reviewbasis `783a48aac610b42e4e1715ed9551f5012376d12b`:
vier vervolgpunten hersteld. Primaire bronmetadata staat gedeeld in
`verification/native-storage-backend-gate/PRIMARY-SOURCES.md`, met expliciet
verschil tussen handleidingsemantiek en lokaal gemeten metadataverlies.
Alle historische README-/archiefbytes blijven intact.
Lokale Btrfs-manifesten zijn nu aan twee onafhankelijk gepinde oorspronkelijke
receipts gebonden (82+17 entries). Beide volledige lijsten worden vóór enige
payloadhash gecontroleerd; gedeeltelijk/uitgebreid manifest faalt expliciet.
Een gepinde `../metadata_probe.py`-alias blijft na lexicale normalisatie binnen
de expliciete bewijsroot; geen absolute historische bronpaden gevolgd.
De echte nieuwe regressies waren rood (2 failures/2 errors); daarna16
hosttests groen en207 oorspronkelijke archiefhashes plus beide receipts
gecontroleerd. Geen full-local-payloads-, engine- of native replay op114.
Score **41 -> 41**. Eerst publiceren/reply+resolve, daarna verse CI/review;
technisch groene checks op783 vervangen geen review van deze followup.
Integrator blijft deze sessie;136-reconcile41034178 wacht lokale opname.

## Historisch checkpoint PR136 — volledigheidsaudit — 3 november 2026

Volledige bron `f6921e49d08a7c547ffda356f71bc17f1de15f31` behoudt d777,
reconcile `d7b4c378` en bewijs `41034178`. De historische native L02-bronnen
433bab9f/7fb8c722 zijn niet als ontbrekende port voorgesteld; geïntegreerde
FR-L04-regressie en fysiekeQPU blijven apart. A01 accepted2/R03open
verduidelijkt beperkte SDKacceptatie, geen releasegovernance of extra punten.
Parent valideerde de zeven geleverde bestandshashes, Draft2020-12,
39 unieke groepen,62 releasekoppelingen, VM-telling21 en0 gesloten
volledige releasecriteria. D777's exact-head-CI-entry is behouden.
Zie `verification/pr136-review-fixes.json` en
`verification/pr136-integrator-check.json` voor exacte commando's en logs.

De eerdere auditcontrole op4a52 rapporteerde29 groene checks, één overgeslagen
CodeRabbit-job en nul mislukt; vier toenmalige threads waren later opgelost.
Die branch-/reviewstatus blijft historisch, niet een verklaring dat de
nieuwe documentcombinatie is geaccepteerd. Geen nieuwe lokale productbuild,
native/QEMU-, hardware-, gebruikerstest, menselijke securityaudit of
releasevalidatie. Alle62 FR-rijen blijven open; score41 ->41.
De integrator is niet overgedragen. Eerstvolgend was nieuwe-head-CI/review.

## Historisch checkpoint PR145 — statische AI-review — 3 november 2026

Overgenomen van `da5918fbe6d8173c4cf44810d403f965c204d36e`, rapportbasis
`380111b68688c981eec6abb8fab29174a29f5601`, toenmalige branch
`simonvanieperen-effective-tribble`. De AI-review meldde geen bevestigde
kwetsbaarheid maar leverde geen lijst van gelezen bestanden/regels of
subsystemendekking. Dit is geen onafhankelijke menselijke audit, volledige
codebase-audit of bewezen branchdiff-dekking.

De toenmalige Dependabot-alert-API gaf geen open meldingen en de gerapporteerde
manifestdiff tegenover `origin/main` was leeg. Dat is geen volledige
dependency-advisoryscan. Secret Scanning stond uit (404); gitleaks,
trufflehog, detect-secrets, cargo-audit, osv-scanner en dependabot ontbraken.
Geen volledige scan, codefix, tests of builds uitgevoerd. Score41 ->41.
De toenmalige vervolgstap was volledige scans uitvoeren zodra tooling of
Secret Scanning beschikbaar is; geen afwezigheid van secrets/risico's
afleiden uit deze beperkte uitkomst. Volledige herkomst en beperkingen:
`verification/security-audit-20261103.md`.

## Historisch checkpoint PR144 — changelog — 3 november 2026

Overgenomen van `a974fb9f731847a8b4dc55f1e080ced945a58e63`, basis
`380111b68688c981eec6abb8fab29174a29f5601`, branch
`simonvanieperen-shiny-waddle`. `CHANGELOG.md` vat de14 commits in het vaste
bereik `ad2b981fe1f2bcccb03cb776b2281d32529d2f00..380111b68688c981eec6abb8fab29174a29f5601`
samen, niet een veranderlijk bereik "sinds main". Documentatie-only,
geen OS-tests of nieuwe productacceptatie; score41 ->41.

Het oorspronkelijke checkpoint signaleerde dat de HEAD-tekst30 CI-controles
meldde terwijl `pr127-review-integration.json` exacte-head/native bewijs nog
open liet. Dat is een historische provenancewaarschuwing, geen actuele
afwijzing van later brongebonden postmergebewijs. De changelogtaak heeft
die tests niet herhaald. Fysieke hardware, menselijke audit en volledige
testrelease bleven open; eerstvolgend was brongebonden CI-resultaten vastleggen.

## Historisch checkpoint — eerste PR114-reviewfixintegratie

Bron `d7d1fdc0356be47c225d7bd216075c4a62030ad4`: researchfix
`fcbf64e3`, veilige bewijsroute `305f1133` en historische metadatafix
`6d0852fa` zijn inhoudsbehoudend opgenomen. Vier researchbevindingen
zijn met echte Markdownrendering gecontroleerd;242 bronlinkgrenzen en
Ubuntu-desktop/Initial-Setup-volgorde hersteld.
Alle207 oorspronkelijke opslagblobhashes blijven geldig. Onveilige oude
scripts zijn niet-uitvoerbare historische snapshots, niet veilige huidige
entrypoints. Nieuwe private offlinevoorbereiding/bewijscontrole heeft7
groene negatieve hosttests. Volledige foreign-engine/native replay blijft
open; OpenZFS-create/crash is expliciet geblokkeerd, niet fictief gerepareerd.
Bewijs: `verification/pr114-doc4-fixes.json` en
`verification/pr114-storage-review-fixes.json`. Score **41 -> 41**.
Publicatiehead vereist verse CI/review en inhoudelijke threadafhandeling.
De eerdere guarded merge is door GitHub geweigerd; geen merge of bypass.
Main blijft380111b6.135 bevat de latere desktop-/volumestapel en134 wordt
afzonderlijk inhoudsbehoudend geïntegreerd; rode fixturelevering en B03-
flushcompletion moeten vóór merge opgelost/geverifieerd worden.
Eerstvolgend: publicatiehead controleren, threads beantwoorden en actuele
CI uitvoeren; geen fysieke writes, productiekeys of volledige releaseclaim.

## Historisch checkpoint — ontwerp, onderzoek en opslagbewijs combineren

Actuele metadatareviewbron: `52e1909cb2667a5862732c9d2dc74a6f81b4e016`.
De verwijzing naar het gebruikerswisselcontract noemt nu de echte
fixcommit68d427b in plaats van diens basis c1d62d83; score41 blijft41.

Basis `517117fbd8359084eca10e2e2d5f4cdf1015b52c` (PR114 en main74a34d05)
met PR131 `a03f7f0c355cedca72611893f07c4a766a7b7d4f`. Integrator `f090b913`.
Geen runtimecode uit onderzoek; ADR-058 blijft leidend voor de shell,
P2-bevriezing betreft het historische prototype. De onafhankelijke
read-only broncontrole bevestigt de Eurobarometer58%-dienstoverstapclaim
tegen de originele data-annex/PDF-hash; dit is geen OS-adoptiepercentage.
De onderzoeks- en ontwerpcheckpointteksten hieronder blijven historisch
behouden; hun toenmalige branchstatus is geen actuele integratie-instructie.
Ook PR132 `6afdf306f19072c19fd572701a6126220a4d56d2` wordt behouden:
de read-only AI-controle bevestigt207 echte Git-blobs en acht gekoppelde
bootmanifesten. Dit is archiefintegriteit, geen onafhankelijke native replay
of menselijke audit. Btrfs/OpenZFS-adoptie en S09 blijven open.
Nieuwste main is `380111b68688c981eec6abb8fab29174a29f5601` via PR127:
exact dezelfde tree als geteste head5343b0a,31 actuele checks geslaagd,
review zonder bevindingen en nul open threads. Native22fasen/raw33,
clipboard/herstel en onafhankelijke14-byte/rev1-schijfcontrole zijn groen;
beide verse N01-netwerkcases raw33. Main gaat34 -> 41 met U01/N01;
de documentatie zelf voegt0punten toe. Postmerge-CI loopt nog afzonderlijk.
PR126 en122 zijn na actuele head/ancestry/treecontrole als overgenomen gesloten.
Details: [mainacceptatie](verification/pr127-main-acceptance.json).
Bij documentintegratie zijn vijf botsende UI-ADR-nummers uniek gemaakt
(059-063), met oude titelankers en historische bewijsnummers behouden.
De laatste PR119-head `ee5003e6` is eveneens volledig behouden; unieke
PR121-statusinformatie `b83e4e05` is onderaan als historisch checkpoint
opgenomen. Controles/bewijsgrenzen: verification/pr114-doc-integration.json.
Vervolg: verse exacte-head CI/review en afstemming met de desktopintegratie.

## Onderzoekscheckpoint PR131 — volledige OS-journey — 2 november 2026

**Scope:** uitsluitend onderzoek en documentatiestatus, geen productcode.
Bronbasis main `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`; branch
`codex/os-research-phases-2-4-20261102`. De apart gelezen ontwikkelbranch
`codex/mature-native-session-20261102` is vastgepind op `1f0dd9c93959e3b1335656da1e548525a288133e`
en niet in deze documentatiebranch geïntegreerd.

**Parallelle integratie:** laatste remotecontrole ziet main
`74a34d05ef5591b782a0da9da34e4669910d0722` (#125, nog 34/100).
Deze onderzoeksbranch behoudt de bovenstaande bronbasis; er is geen
productcode uit main ingemengd. Concept-PR #131 meldt integratieconflicten.
De aangewezen integrator moet bij opname de nieuwste main-status en
bestaande evidence behouden; onderzoeksvoorstellen leveren geen punten op.

**Geleverd:** [fase 2–4](research/2026-11-02/README.md) en de
[volledige installatie-/day-one-journey](research/2026-11-02/journey/README.md).
De uitbreiding bevat 31 installatieaspecten, 28 dagelijkse taken,
herkomstprofielen, mentale modellen, familiarity budget en afzonderlijk
gelabelde installer-/onboarding-/interactievoorstellen. Het bestaande
PRODUCT-SPEC/ROADMAP is pas na de vergelijking begrensd aangevuld;
P2 blijft bevroren. Bronregisters geven versie, datum, passage en beperkingen.
Eurobarometer 572/QB6.4 is gecontroleerd in originele vragenlijst, rapport en
data-annex: 21+37=58% instemming met dienstoverstap, geen OS-migratie.
Bronafwijkingen bij technische rapportkop en Cyprus-landlabel zijn zichtbaar bewaard.

**Bewijs en score:** mainbasis 34/100 → 34/100; geen nieuwe native, fysieke,
menselijke of releaseacceptatie. Ontwikkelbron `1f0dd9c` rapporteert afzonderlijk
41/100; deze onderzoekstaak kopieert die score niet naar main.
Geen installaties, fysieke schijven, productcode of echte geheimen gebruikt.
Document-/bronverwijzingen, matrixdekking, scope en progressgeneratie
worden vóór commit gecontroleerd; geen kernelbuild/QEMU-herhaling voor deze
documentatietaak. Definitieve controles:
[fase 2–4](research/2026-11-02/verification.json) en
[journey](research/2026-11-02/journey/verification.json).

**Open:** geen eigen gebruikers-/installatieproef, representatieve
EU-studie, praktijkmigratie of retentiemeting. Exacte macOS-paneelposities en
enkele KDE-staptellingen blijven onbekend; actuele KDE-ingangen zijn op
v6.8.0-code bevestigd. Native installer, persoonlijke authenticatie,
encryptie-/recoveryketen en OS-brede toegankelijkheid blijven afzonderlijke gates.
**Eerstvolgend:** lees journey-index, installer IN-/onboarding ON-/UX-criteria
naast de actuele ontwikkelbron. Hercontroleer code/services vóór een begrensde
niet-schrijvende proef. Productcode blijft buiten deze opdracht. Bestaande
integrator- en mergeafspraken blijven behouden; concept-PR wordt niet gemerged.

## Ontwerpcheckpoint PR114 — gebruikerswisselcontract — 2 november 2026

Fixbron `68d427b76fd4c5fc0a071433d567097038e71a65`, alleen documentatie.
De exclusieve integrator is `f090b913`; hieronder staat historisch mandaat,
geen nieuwe toewijzing. Main is inmiddels `74a34d05` via PR125; #109/#110
zijn aantoonbaar gesloten. Deze ontwerpbranch is nu inhoudsbehoudend met
main74a34d05 gecombineerd; latere desktopdocumentatie blijft apart.

PR114-review4428521664 is uitgewerkt: A's sessiehandles intrekken of via
generatie blokkeren voordat B actief wordt; A's sleutels blijven uitsluitend
in diens geisoleerde domein; B krijgt geen handles, objecten of mappings van A.
Terugkeer vereist authenticatie en nieuwe handles, geen stille herleving.
Afzonderlijk toegestane achtergrondsync blijft aan A en bestaande scope
gebonden. Bij onvoldoende scheiding wisselen weigeren tot expliciet
afmelden, geen stil verlies van werk. Native A/B/key-state/foutinjectie-
acceptatie is benoemd, **niet uitgevoerd**; geen implementatieclaim.

Controles: `python3 tools/progress.py --write`,
`python3 tools/progress.py --check`, `git diff --check`.
Score **34 -> 34**, nul nieuwe punten. Eerstvolgend: actuele main en
overige ontwerpvoorstellen inhoudsbehoudend combineren en verse CI/review;
PR127 heeft apart de gecombineerde desktop/native-netwerkgates lopen.

## Opslagcheckpoint PR132 — native backendtoets — 2 november 2026

**Bronnen:** eerste proef `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`;
bevestigingsproef `74a34d05ef5591b782a0da9da34e4669910d0722`; bewijsbranch
`codex/native-storage-backend-gate-20261102`. Eigen kernel en drivers ongewijzigd.
Alleen wegwerpbestanden; vreemde datadisks uitsluitend read-only. Vier nieuwe
VM-runs op iedere bron (acht totaal), met dezelfde uitkomsten; de eerdere
bewijzen zijn behouden en de upstreamintegratie is volledig overgenomen.

**Uitgevoerd:** opnieuw gebouwde native NOSVOL02-normal en cold-recover slagen
(QEMU raw exit 33). Btrfs en één OpenZFS-mirrorlid leveren geen native import
of bestandsherstel (exit 35). Btrfs-nulheaders worden als lege proefstaat
toegelaten, waarna de documentcontrole faalt; dit is geen veilige generieke
formaatafwijzing. OpenZFS wordt als Corrupt geweigerd. Alle vreemde bron- en
proefimagehashes zijn gelijk voor/na de boots.

**Hostbewijs:** Btrfs-reservesuperblockherstel slaagt; databitcorruptie wordt
gedetecteerd maar `restore` geeft corrupte inhoud met exit 0 terug. Aparte
metadata-import/salvage verliest onder meer hardlinktopologie en lege-mapdata.
OpenZFS-libzpool/zdb kan de hostpool controleren/importeren; `ztest -E` heeft
een `/dev/zfs`-discoverygrens, de crashproef meldt een onopgelost allocatieverschil
en gewone bestands-/backuprestore is niet aangetoond. Dit is geen native port.

**Besluit en score:** native adoptie van beide backends blijft niet geaccepteerd;
geen eigen driver of nieuw diskformaat gestart. **34/100 → 34/100**; S09 blijft
open. Bestaand positief NOSVOL02-bewijs wordt niet uitgebreid tot vreemde FS'en.
Zie [toets en reproduceerroute](verification/native-storage-backend-gate/README.md).
De oorspronkelijke hoofdcheckout en ongerelateerde `.github/github-app.yml`
zijn behouden; deze branch wijzigt uitsluitend bewijs/documentatie/assessment.

**Open:** positieve formaatidentiteit vóór writable toelating; gecontroleerde
bestaande-engineport naar de eigen ABI; onafhankelijk inhouds- én metadata-orakel;
verklaring van de OpenZFS-harnasbevindingen. Geen fysieke powercut-, volledige
snapshot-/encryptie-/backup- of prestatietoelating.

**Eerstvolgende controle:**
`python3 tools/progress.py --check`, daarna de beschreven native-enginepoort
beoordelen zonder een Linux-kernel of nieuw bestandssysteem als omweg in te voeren.

## NAS-checkpoint PR119 — conflictoplossing — 2 november 2026

**Bron:** PR-head `fa97c69221fd9b5635684c48b60e72bf67b2541a`, branch
`claude/issue-110-nas-replay`; geïntegreerde main:
`74a34d05ef5591b782a0da9da34e4669910d0722`. Alleen conflictoplossing is
geautoriseerd; de PR wordt niet gemerged en oude CI wordt niet herstart.

**Inhoudsbehoud:** main bevat de replay-implementatie en het bewijs uit deze
PR al via #125. De vijf conflicten zijn opgelost met behoud van mains latere
lekcontrole, negatieve tests, contractcorrecties en ruwe-bewijsattributen:
`.gitattributes`, `docs/verification/nas-replay-110/README.md`,
`services/nas/native-probe/CONTRACT.md`, `tests/test_nas_runner.py` en
`tools/test_nas.py`. Vóór dit status-/metercheckpoint was de volledige
opgeloste werkboom bytegelijk aan main (`git diff --exit-code origin/main -- .`,
exit 0). Er is geen nieuwe kernel-, interface- of rechtenwijziging ontworpen.

**Lokale verificatie op de opgeloste boom:**
- `python3 -B -m unittest discover -s tests -p 'test_nas_*.py' -v`:
  exit 0, 39 tests; `/tmp/nieuw-os-pr119-nas-host.log`.
- `python3 -B -m unittest discover -s services/nas/native-probe/tests -p 'test_*.py' -v`:
  exit 0, 17 tests; `/tmp/nieuw-os-pr119-replay-oracle.log`.
- `python3 -B -m unittest discover -s tests -p 'test_volume_runner.py' -v`:
  exit 0, 3 tests; `/tmp/nieuw-os-pr119-volume-host.log`.
- `sha256sum --check SHA256SUMS` vanuit `docs/verification/nas-replay-110`:
  exit 0; `/tmp/nieuw-os-pr119-evidence.log`.
- `python3 -B tools/preflight.py --strict`: exit 2;
  `/tmp/nieuw-os-pr119-preflight.json`. Rust 1.90.0, beide native targets,
  QEMU en OVMF ontbreken. Native build/QEMU en Rust-format/lint zijn daarom
  niet uitgevoerd; dit is geen nieuwe native acceptatie. De gerichte
  hosttests en exacte vergelijking met main verifiëren de conflictoplossing.

**Score en vervolg:** 34/100 → 34/100; geen criterium of bewijsniveau wijzigt.
Na publicatie moeten de checks op de nieuwe PR-head worden beoordeeld.
Historische bewijsbundels blijven aan hun eigen bron gebonden; alle open
product-, fysieke en menselijke acceptatiegrenzen blijven bestaan.
Eerstvolgende lokale controle bij hervatten: `git status --short`.

## Historisch checkpoint PR121 — conflictoplossing — 2 november 2026

Overgenomen van `b83e4e05b923dc2c0d8b4339fc9509b5752cb384`;
onderstaande resultaten zijn die historische uitvoering, niet hier herhaald.
De runtime-inhoud van ouder `0fe648b87d1f4959087d3f80a251a7d6fb64dd66`
zit in de PR127-ancestry; de nieuwe statuscommit zelf nog niet in main.

**Bron:** mergecommit `8686f977a7f1fbfba1eb1214d301935c1859e8ed`, met ouders
PR-head `0fe648b87d1f4959087d3f80a251a7d6fb64dd66` en actuele `main`
`74a34d05ef5591b782a0da9da34e4669910d0722`. De conflictintegratie behoudt de
native sessiepreview en de wijzigingen en bewijsbestanden van `main`. In
`ui/native/tests/interaction.rs` staan zowel de PR-desktopproeven als de
drie main-regressies voor focus en verouderde pointergebaren.

**Historische validatie:** `cargo test --manifest-path ui/native/Cargo.toml
--test interaction`: 34 tests geslaagd; `cargo clippy --manifest-path
ui/native/Cargo.toml --test interaction -- -D warnings` en
`rustfmt --edition 2021 --check ui/native/tests/interaction.rs` slagen.
De PR-delta `git diff --check origin/main` is schoon. Full-release-,
NAS-replay- en sparse-frame-`SHA256SUMS`-controles slagen. Een volledige
diff-check waarschuwde voor hashgebonden CRLF-/blank-at-EOF-bewijsbytes;
die bytes zijn niet aangepast.

**Historische scan/bewijsgrens:** CodeQL vond geen Python-alerts maar liep vast
voordat alle talen klaar waren; geen volledige CodeQL-scan. De app meldde vóór
deze integratie geen vereiste checks; nieuwe-headchecks bleven vereist.
Score34, geen productacceptatiebesluit. De toenmalige vraag
om publicatie via `report_progress` is historisch, geen nieuw mandaat.

## Historisch checkpoint — mainstand en documentatie — 3 november 2026

**Basis:** `380111b68688c981eec6abb8fab29174a29f5601` op
`simonvanieperen-laughing-fortnight`, destijds gelijk aan `origin/main`; parent
`74a34d05ef5591b782a0da9da34e4669910d0722`. PR126-inhoud en de zeven
PR127-reviewfixes zijn nu in deze main-commit geïntegreerd.

**Exact-head controles:** GitHub rapporteert 28/28 check-runs als `success`.
De [native venstersessie-run](https://github.com/simonvanieperen/nieuw-os/actions/runs/41535553113)
sluit de actuele QEMU-route met echte invoer, pixel-/isolatieasserties,
compositorherstel en schijfcontrole af; de
[netwerk-run](https://github.com/simonvanieperen/nieuw-os/actions/runs/41535553158)
sluit hostkwaliteit en de native netwerkdienst met gecontroleerde peer af.
Dit is geen Windows-runtime-, fysieke-hardware-, menselijke-audit- of
volledige-releasebewijs.

**Documentatie:** README, status, voortgangsassessment en verificatierecord
zijn bijgewerkt naar deze bron. Alle publieke functies in
`kernel-core`, `artifact-store-core` en de window-session-scene hebben nu
Rustdoc; comments
verduidelijken beschadigingstegels en toetsrelease-eigenaarschap. Deze
documentatiewijziging verandert geen OS-gedrag of acceptatiecriterium.

**Voortgang:** **41/100 → 41/100**, rubric v1; er zijn geen punten toegekend.
U02, authenticatie/vergrendeling, de resterende releasecriteria, fysieke
hardware en de volledige testrelease blijven open.

**Uitgevoerd:** `python3 tools/progress.py --write` exit 0;
`python3 tools/progress.py --check` exit 0; `git diff --check` en beide
`python3 -m json.tool`-controles exit 0. Publieke-functiedekking van de drie
gewijzigde Rust-bronbestanden is statisch gecontroleerd. Exact-head status is
gecontroleerd met `gh api repos/simonvanieperen/nieuw-os/commits/380111b68688c981eec6abb8fab29174a29f5601/check-runs --jq '.total_count'`
(28, alle success), plus `gh run view 41535553113 --json conclusion,headSha,jobs,workflowName,url`
en `gh run view 41535553158 --json conclusion,headSha,jobs,workflowName,url` (beide exit 0). Rustdocgeneratie
`RUSTDOCFLAGS='-D missing-docs' cargo doc --locked --no-deps -p capability-core -p kernel-core -p virtio-core -p ui-core -p update-core -p app-package-core -p artifact-store-core`
is **niet uitgevoerd**: exit 127, `cargo: command not found`.
`python3 -B tools/dev.py cargo +1.90.0 --version` exit 1 doordat de Cargo-
binary ontbreekt. CodeRabbit CLI-review is niet gestart; het uitvoerbare
bestand was niet vindbaar en beschikbaarheid/authenticatie zijn onbekend.

**Eerstvolgend commando:** `git status --short && git rev-parse HEAD &&
python3 -B tools/progress.py --check`; raadpleeg daarna `docs/WORK-QUEUE.md`
voor het volgende productwerk.

## Native checkpoint PR127 — zeven reviewfixes — 2 november 2026

**Basis:** PR127 `798706c2a53d0062ce0474e5f84f294b2c37b825`, drie lokale
fixleaves en PR126 `8f08db4c12573f91b57e55c30fa417e7abd358e6`, met main
`74a34d05ef5591b782a0da9da34e4669910d0722`. Integrator blijft `f090b913`.
Deze PR127-kandidaat behoudt de volledige main/sparse/retained/expiry/
clipboardinhoud en herstelt paletdispatch, proportionele pointerposities,
TCP-deadline/Closed, probegebruikersconfig, volledige dockhoverdamage en
write-only semantisch wachtwoord. Geen capabilities of dependencies verruimd.

**Nieuwe echte integratiebevindingen:** dubbele caretfunctie na automerge
opgelost met behoud van de proportionele implementatie. Bestaande
damage-regressie vond pixels buiten de oude 16px-/monospacevlakken;
renderer en model delen nu resultaat-/statusbounds met echte fonthoogten
en het formuleveld wordt correct begrensd hertekend. Alle bestaande
pixel- en >=90%-SUR-sendbudgetasserties blijven behouden. De nieuwe
wachtwoordtest gebruikte een ongeldige 1600x1200-viewport; beide schalen
worden nu op ondersteunde 800x600 fysieke pixels getest.

**Uitgevoerd:** 189 Rust-app/UI-tests en 31 Python-harnastests, exit0 in22s;
9 TCP-tests met echte gecontroleerde smoltcp-peer op Rust1.91.1, exit0.
Beide TCP-bugs zijn eerst op oude productiecode echt rood gereproduceerd
(7pass/2fail, exit101), niet alleen statisch aannemelijk gemaakt.
Clippy/fmt en echte native-targetchecks/releasebuild slagen.
Exacte commands/loghashes: verification/pr127-review-integration.json.

**Parentbewijs:** alle actuele PR126-jobs slagen. Native run41530207152,
artifact13182057101, testmergea1cb32b met exact main74a34d05/head8f08:
gelijke tree92c3200, source8b2238b2,22fasen/raw33/native_gate/clipboard_gate,
328,969s, herstel en schijf14bytes/rev1/canaries groen. Lokale volledige
PR126-probebuild eveneens exit0 in8s na hergebruik van Rust1.90 met private
shims. Dit bewijst NIET de nieuwe PR127-bron.

**Open:** verse gecombineerde CI/review en volledige native-/herstel-/
pixel-/klembord-/schijfgates. PR126 heeft een nieuwe TCP-Closed-review
die juist in deze kandidaat is hersteld: dus niet los ongefixte PR126 mergen.
De netwerk-CI herhaalt nu ook de bestaande normal/client-exit-QEMU-gates
op schone eigen-kernelimages: na TCP-productiewijzigingen is ouder N01-bewijs
alleen onvoldoende. Deze nieuwe job is nog niet als geslaagd aangemerkt.
Pas na main-inhoudsbehoud ingehaalde bron-PRs sluiten. #109/#110 zijn al
aantoonbaar via #125/main opgelost; #39-#43 blijven inhoudelijk open.
Branchscore **41 → 41**, main **34 → 34**; nul nieuwe punten.
Eerstvolgend: nieuwe `native-window-session` en netwerk-CI op deze head
controleren, reviewthreads inhoudelijk beantwoorden en nieuwe bevindingen
oplossen. Geen Windows/WHPX-, fysieke, menselijke of volledige releaseclaim.

## Vorig checkpoint — main geïntegreerd; gezamenlijke desktopgate volgt — 2 november 2026

**Basis:** PR126 `51580271eada508d9a1edcac5a1e3599f6d726f5` met main
`74a34d05ef5591b782a0da9da34e4669910d0722`; exclusieve integrator blijft
`f090b913`. PR125 is met 29 actuele groene jobs, actuele code-review zonder
bevindingen en head/base/treeguard gesquasht. Main-tree is bytegelijk aan
geteste kandidaat `b2761f1`. #109/#110 en aantoonbaar opgenomen bron-PRs
#115/#116/#117/#118/#120 zijn gesloten. #119 kreeg gelijktijdig een cloudmerge
`ee5003e`; alleen vier status-/meterdocumenten verschillen nog van main.
Die nieuwe bron is niet ten onrechte als oude ancestor gesloten.

**Deze integratie:** alle main-/PR120-followups en alle desktop/retained/
expiry/clipboardfuncties behouden. SUR1-sparsecontrole zit nu ook in de
retained-positionering; hersynchronisatie blijft volledig en doorlopend.
Materiaal behoudt beide APIs maar mengt geen impliciete digestgaten met
expliciete retain-coverage. Een gecombineerde cache+retained-regressie
controleert iedere tegel, exacte pixels en framecompletion. Bestaande
capability-/UI-tests van beide kanten zijn bytebehoudend samengevoegd.
App-hosttests voor Materiaal en Rekenmachine zijn aan de bestaande native
workflow toegevoegd; geen dependency of recht verruimd.

**Uitgevoerd:** volledige Rust-hostsuites voor window-session, Materiaal,
SUR1/client, ui/native en capability-core: exit 0. De vroegere ontbrekende
`cc` is hersteld met alleen in de sessiemap uitgepakte, gepinde en gehashte
Ubuntu GCC15/libgcc/libc-dev-pakketten; geen systeeminstallatie. Rust1.90,
host- en echte native-target type/lintchecks slagen. Exacte commando's,
exits, aantallen en loghashes: verification/pr126-main-integration.json.
De groene hostsuite is geen QEMU-, fysiek of Windows-bewijs.

**Nog open:** nieuwe exacte-head hosted CI met volledige native venstersessie,
herstel, pixels, klembord, schijf en package. #127-fixleaves zijn apart lokaal
gecombineerd; neem die later hunkgewijs op en behoud deze expiry/retained-route.
Main-meter **34 blijft 34**; deze branch behoudt zijn eerdere **41 → 41**
zonder nieuwe punten. U02, alle FR-rijen en volledige release blijven open.
Eerstvolgend: nieuwe `native-window-session`-run en alle concrete nieuwe
reviewbevindingen op de gezamenlijke head controleren, niet oude jobs hergebruiken.

## Vorig checkpoint — PR126: tweede fixturefout hersteld — 2 november 2026

Basis `bd981f79a234dc45fbf38ea1cae25c08ecd1e9c1`, branch
`codex/native-retained-frames-20261102`; main-integratie blijft uitsluitend
bij integrator `f090b913`. Die fixcommit activeert retained rendering alleen
voor de venstersessie en verwerkt verlaten SUR1-transacties via de bestaande
timeoutreply/backpressureroute. Geen ruimere capabilities of dependencies.

De klokcorrectie is uitgevoerd in CI-run `41526505812`, job `125555496660`.
Die run toont een tweede fixturefout op tests/idle.rs:689: de gedeeltelijke
Chunk gebruikte 1 byte, terwijl SUR1 voor iedere chunk exact
`min(resterend, CHUNK)` bytes vereist. De fixture stuurt nu de eerste geldige
volledige chunk van een nog onvolledige tegel. Een assert controleert dat
deze chunk kleiner is dan de tegel. Geen productie- of protocolwijziging;
de exacte 209/210-grens, nul bevestigde pixels, eenmaal timeout, tokenweigering,
monotone ticks en herstelasserties blijven behouden. De CI-hostsuite was
dus nog rood (exit 101), niet groen; de native fase werd niet bereikt.

`cargo fmt --manifest-path sdk/examples/window-session/Cargo.toml -- --check`
en `cargo clippy --offline --locked --manifest-path
sdk/examples/window-session/Cargo.toml --all-targets --target-dir
target/pr126-clock -- -D warnings`: beide exit 0 met bestaande Rust 1.90.0.
Log `/tmp/nieuw-os-pr126-chunk-clippy.log`. Lokale Rust-testuitvoering blijft
geblokkeerd door ontbrekende `cc`; geen QEMU/Windows uitgevoerd. Nieuwe
exacte-head CI moet het fixtureherstel en de native route nog bevestigen.
Rapport: `docs/verification/pr126-review-fixes-20261102.json`.

Geerfde ontwikkelbranchscore **41/100 → 41/100**; nul nieuwe punten voor
deze fixes. Dit is geen automatische overname van 41 punten op main
(integratieassessment #125 blijft afzonderlijk 34/100).
Eerstvolgend: na push `native-window-session` en alle relevante nieuwe
checks op de exacte head controleren; niet mergen zolang een gate rood is.

## Vorig checkpoint — PR126 CI-interfaceherstel — 2 november 2026

Basis `fb5810e45bd8d2ab4a73d886d6c23cf526b10cec`, branch
`codex/native-retained-frames-20261102`. Alleen de drie aangewezen falende
CI-controles onderzocht; geen merge of wijziging van capabilities/dependencies.
Ontbrekende damage-/presentatie-interfaces en scrolbewuste cursorplaatsing
hersteld; de native window-compositor behandelt nu ook gevalideerde Retain-replies.
Damage blijft staan over gedeeltelijk vervangen frames en wordt alleen na
actuele volledige presentatie gewist. Backspace op een automatische
regelovergang is afzonderlijk rood/groen gereproduceerd en afgedekt.

**Bewijs:** 169 gerichte hosttests groen (rekenmachine36, Materiaal40,
native UI52, window-session41), Clippy/fmt voor deze vier crates groen.
Alle acht native app-ELFs gebouwd/geïnspecteerd; ook de normale compositor
zonder fault-injection gebouwd. Dit is geen native runtimeacceptatie.
De integrale builder stopt daarna op ontbrekende offline dependency
`hex-literal`; één gerichte SDK dependency-fetch slaagde eerder.
Strict preflight meldt ontbrekende QEMU/OVMF. Volledige QEMU-route,
Windows-pakket en hosted CI op de herstelcommit zijn **niet uitgevoerd**.
Commandostatussen, bron-/ELF-hashes en lokale logpaden staan in
`verification/gui-idle-20261102/agent-merge-ci-repair.json`.
**41/100 → 41/100:** U02 en de volledige releaseacceptatie blijven open.
Eerstvolgende controle: de nieuwe CI-resultaten laten beoordelen; lokale
runtimeacceptatie pas met volledige dependencies, QEMU en expliciete firmware.

## Vorig checkpoint — native klembord geïntegreerd; U02 blijft open — 2 november 2026

Branch `codex/mature-native-session-20261102`, concept-PR122 op PR121.
PR124 is na18 groene CI-workflows en code-/artifactreview opgenomen als
`9863a52651504c4838d6826b3e1b2240ae0ed6b2`. Geen merge naar main;
de bestaande integrator behoudt die verantwoordelijkheid. **41/100 blijft41**.
U02, bredere clipboardcapaciteit, volledige interactiebudgetten, echte
accountauthenticatie en de volledige release blijven open.

**Nieuw:** echte CtrlC/CtrlV tussen twee geïsoleerde native processen. De
compositor controleert huidige focus, echte PID/inputepoch en het eigen nog
staande CtrlC/V-Press-aanbod; één aanbod geeft één gebruik. InputSession
blijft de enige RX-eigenaar. Pendingbackpressure bewaart alleen ticketmetadata,
geen plaintext; iedere flush valideert opnieuw. De app past ontvangen Text pas
na echte ACKAcknowledged toe; Superseded en transportfalen wissen de tijdelijke
kopie. Atomische rekenmachinepaste valideert de hele tekst vóór mutatie en
start geen berekening. Copy wist vooraf oude inhoud; CtrlShiftDelete wist de
vluchtige clipboard; compositorherstart begint leeg en behoudt de appmodellen.
Geen nieuwe kernel-ABI, proces, apparaat-/opslag-/netwerkrecht of dependencyversie.
Eerste capaciteit192 UTF-8-bytes met zichtbare weigering, zonder afkappen,
historie, hele-document- of oude-antwoordfallback. Geen credential-/authclaim.

**Bron en bewijs:** functionele d0ae570bd4666626096f92e57acef981c0f0d7b6,
source-v3 05307fef9684d97480a77ea39050d80f6affcf522a6c09c7eb53340c789b35aa.
Root-TCG/q35-8.2/qemu64/1CPU/256MiB/1280×720 slaagt in1196,887s,
alle22fasen/raw33/clipboard_gate=true. Oorspronkelijke13fasen behouden,
9clipboardfasen met werkelijk4Copy/5Paste/1Clear, roundtrip3→3+4,
atomisch letters weigeren, clear en post-faultEmpty. Source/frame/ACK/fault/
PID/epoch/werkbehoud en onafhankelijke diskoracle14bytes/rev1/canaries/cleanup0
zijn gecontroleerd. Rawserialhashb5bd0b0e…72961. Run:
build/clipboard-native/run/window-2efc331ed02d4f89bea34d7287840cde.
Screenshot clipboard-calculator-copied.png is verliesloos uit de echtePPM.

De finale e3182f0bba72f300596dd81417fbbd371a8d5d79 wijzigt daarna uitsluitend
beide app-chain-hosttests. Volledige source-v3:
038615b1b425d3f24ea2d9c52b36184ea666f81237c813303dc0fac93fb756ca.
Herbouw op dezelfde omgeving geeft alle10binaireSHA256s identiek. Oude rootrun
blijft op d0ae/source0530 gebonden. Alle18CI-workflows op e318 slagen. Echte
CI-merge5754763ffb257d6420f74126cae7252e36afab1e/source0386 doorloopt dezelfde
22fasen in1407,011s, native_gate/clipboard_gate/raw33,34hostorakels en17volledige
appbody-pixelvergelijkingen. Opgeslagen beelden/serial/disk/8ingebeddeELFs en
19normalepayloads/font/firmware zijn onafhankelijk teruggelezen. Geen masks.
Bewijs: verification/native-clipboard-20261102/README.md met raw.gz en hashes.

28client+40brokerhosttests/scalarbuilds; volledige UI/apphosttests/Clippy/fmt;
46gerichte appketens voor opt-in/default-uit, echte focuscancel en transportverlies
vóór én na serverACK zonder edit/eval/valseappACK.187Python-tooltests:184pass,
3expliciete skips.16gerichte runner/latencychecks groen, zes overlappen die187.
Geen menselijke audit. Fase-READY/GEN kan achterlopen op het juiste beeld:
claim appbodypixels+echte inputACK, geen causale laatste-SUR-generatiegarantie.
Tijdvelden omvatten host/QMP/observatie/capture, geen p95/whole-screen/scanout.
De latencysample-recorder is gebouwd, zijn volledige workload niet uitgevoerd;
TCG-schermopbouw blijft te traag. Geen performance- of veiligheidsvergelijking.

**Windowspakket:** normale CI-bundel zonder crashcontroller:
`/workspace/nieuw-os-windows-package/build/clipboard-e318/NieuwOS-klembord-e318-Windows-QEMU.zip`,
SHA256 e29a6ca5933a0958b10238ae2f660237bbf5295c170d5c92146eb9d4ddf2fc3d.
Alle19payloads/8ELFs/7fontinputs/firmware en fakeCheckOnly zonderstate/VM groen;
20gerichtePython/22launcher/33helperchecks eenmaal groen. Een apart lokaal
gebouwde ZIP heeft eigen hashes en wordt niet met CI vereenzelvigd. Nieuwe
Windows/WHPX/SDL-runtime nog niet uitgevoerd in deze chat; lokale testchat heeft
de exacte e318-bron en --clipboard opdracht gekregen. Geen hostinstellingen of
fysieke schijven/doorgifte gewijzigd. Beide Linux-QEMU-proeven eindigden netjes.

**Volgende stap:** U02-snelheid: meet invoer/focus/appwissel/plakken volgens het
vooraf vastgelegde plan, onderzoek volledige1792tile-Materiaalhertekeningen en
verminder onnodig werk/IPC zonder caller-/epoch-/bounds-/COMMIT-/deadlinechecks
te schrappen. Complete-frame-100/250ms-doelen op de vaste WHPX-referentie blijven
open; ruimere clipboardstreaming volgt apart. Eerstvolgende broninspectie:
`rg -n 'paint_step|paint_tile|invalidate' sdk/examples/materiaal/src/lib.rs sdk/examples/calculator/src/app.rs ui/native/src/controller.rs`.
Contracten: implementation/NATIVE-CLIPBOARD.md en GUI-IDLE-WAIT.md;
meethelper tools/window_latency.py. Auth/sessiebroker/keyring-unlock/FIDO2,
contextmenu/IME/toegankelijkheid en volledige releasecriteria blijven behouden.

## Vorig geïntegreerd checkpoint — desktop en GUI-idle

Branch `codex/mature-native-session-20261102`, concept-PR122 op PR121. PR123 is
na centrale review en **18/18 groene CI-workflows** geïntegreerd als
`8289d60cd5caa8a26b7823a694858a15371dc2af`. Geen merge naar main;
de bestaande integrator behoudt die verantwoordelijkheid. De bewijsupdate bevat alleen documentatie en rapporten; geen
productcodewijziging na de geteste freeze.

**Roadmap: 41/100**, tegenover 34 vóór deze opdracht: N01 +3 en U01 +4.
De CPU-optimalisatie krijgt geen extra punten. U02, echte authenticatie,
klembord, volledige interactiebudgetten en de volledige testrelease blijven open.

**Eigen native desktop:** 1280×720, proportionele antialiased letters, eigen
pictogrammen, twee geïsoleerde appvensters, focus, slepen, dock en aanwijzer.
Materiaal en Rekenmachine behouden hun proces en inhoud bij een echte
compositorcrash; nieuwe presentatie-/invoerprocessen krijgen verse epochs.
Alle dertien fasen inclusief herstel/save/list/reopen slagen op 337 in root-TCG
(753,648s), CI (853,340s) en lokale Windows/WHPX (36,946s).
Raw exit 33, exacte appbeelden,14 bytes/revisie 1, onafhankelijke diskoracle, canaries en
volledige cleanup. U01-bewijs: verification/mature-session-20261102/U01-ACCEPTANCE.md.
Rode 2bb/b1-resultaten blijven rood; geen release- of authenticatieclaim.
Screenshot: build/window-final/run/window-b42797a76f084c62a04223c8dbd9de1b/reopened.png.

**Idle:** vier GUI-lussen parkeren via bestaande IPC_WAIT uitsluitend als er
geen onvoltooid werk is; kernel, ABI, rechten en transactiedeadlines blijven
ongewijzigd. Zes formele Windows/WHPX-routes, drie per arm, hetzelfde volledige beeld
vóór/na 15 s zonder QMP/invoer/pause tijdens ieder interval. ProcesCPU mediaan
99,3724%→11,6663% van één logische CPU: 88,2600% lager. Gemapte vCPU-thread
98,9558%→11,3538%. Root heeft ruwe 100ns-tellers en monotone meetintervallen
onafhankelijk herberekend. Primaire en twee gerichte AI-bronreviews vonden
geen blocker. Geen energie-, geen-redraw-, harde 20ms- of algemene latencyclaim.

**Bronbinding:** formele A 337/source39194035…f1995 tegenover
B 1f2ac0ae7cf8944d77ca0656affb01fe9b086f1b/source-v3
8faf3f7f47e3f26409e446b2311673416ddd70d5735ef106d94c9f30d2c6847a.
Head 2a7b74426027093a4793d78d5ddca94e15ae018a voegt alleen bewijs,
CI-fetchvolgorde en een Python-F12-testfix toe. Volledige huidige source-v3:
084478cf4d2e137a3ba37bc6029716be3e38fecfdf76ca82fe55cd409b8536d0.
Loader/kernel/acht normale app-ELFs en alle19 pakketbytes zijn bytegelijk aan
de productbron 8faf; alleen pakketmanifest verschilt. Oude metingen worden
niet naar 2a omgelabeld. Alle 18 CI-workflows op2a slagen, inclusief de echte
TCG-venstergate 41495583952 en gezamenlijke native systeemgate 41495583552.
Bron-/artifact-/hashbewijs: verification/gui-idle-20261102/INTEGRATION.md.
181 Python-tooltests, gerichte Rust-hosttests/Clippy/fmt,22 launcher- en 33 helperchecks
slagen. De CI-fetchfix behoudt offline/locked; cachehit is geen cold-cachebewijs.

**Windowspakket:** normale native-window-session zonder crashcontroller:
`/workspace/nieuw-os-windows-package/build/window-package-2a7b744/NieuwOS-venstersessie-2a7b744-Windows-QEMU.zip`,
SHA256 b685eb421e279da680da2c84bd280cc1edbc8684471f2019b2879c129c3ba1e0.
19 payloads/8 ELFs/alle fontnotices teruggelezen, synthetische CheckOnly zonder state.
Het oude 337-CI-pakket startte lokaal werkelijk via PS5.1/Whpx/SDL en sloot met 0;
visuele/inputacceptatie van die normale bundel bleef open door capturefout
0x800b0101. De volledige headless probes hebben hun eigen geslaagde beeld-
en invoerbewijs. Geen hostinstellingen, fysieke disks of passthrough gewijzigd.

**Eerstvolgende implementatiestap:** U02 aanvullen met expliciet begrensd,
gebruikersgestuurd klembord tussen de afzonderlijke apps, intrekbare rechten,
negatieve cross-app-proeven en gemeten invoer-/focus-/appwisselbudgetten.
Bestaande documenten: implementation/NATIVE-WINDOW-SESSION.md,
implementation/GUI-IDLE-WAIT.md en implementation/SESSION-AND-DESKTOP.md.
Echte account-/sessiebroker, sleutelvrijgave, vergrendeling en FIDO2 blijven
afzonderlijk noodzakelijk; gastaanmelding is geen beveiligde gebruikersgrens.
Geen nieuwe roadmap of scopeverlaging; alle volledige releasecriteria behouden.

## Vorige afgeronde checkpoint — native sessiepreview


**Functionele implementatie / Windows-pakket:** `344dc0afde893784503217ac676e154b13662507`,
branch `codex/session-desktop-vm-20261102`, concept-PR #121. Bronbasis
`e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. De latere opmaakcommit `cfed2db9a39396e48a7bc758eb2c533f3af3f22c`
wikkelt uitsluitend één kernelarray-element om volgens rustfmt. De volledig
lokaal geteste344-bron heeft source-v3
`05b047819f1c1cddce6868034ab2208d8a35a95d9f974860afb8bbc53eeb8658`;
de opgemaakte bron heeft
`0bfd8385bbe08d3ace384977848fb42f6a65f6261b28be19805eccd90fce14f0`
en een eigen geslaagde achtstappen-cloud-QEMU-herhaling. Geen verdere
functionele wijzigingen. Beide rootfmt- en diffchecks slagen.
De bestaande andere integrator blijft eigenaar van merges; deze PR is niet gemerged.

**Geleverd:** opt-in `native-system-session-preview` met gebruikerskeuze,
herkenbaar aanmeldformulier, expliciete openbare VM-gast, bureaublad/taakbalk/
programmasearch en echte Materiaal. Eigen no_std-toestanden binden authantwoorden
maar verlenen zelf geen rechten. `UnavailableBroker` weigert alle persoonlijke
credentials; geen gedeeld demowachtwoord of nepbiometrie. Owned secrets worden
met de reeds gepinde zeroize gewist. Geen nieuwe externe crate of capability.

Windows Auto/Whpx/Tcg gebruikt een echte sequentiële QMP-initprobe met één
10 s-deadline en binaire LF-invoer. De lokale chat reproduceerde en valideerde
de Windows-HMP-racefix. Source-v3 sorteert expliciet case-sensitive padcomponenten,
waardoor gelijke Windows/Linuxbronnen dezelfde hash leveren; controles blijven intact.
De compositor verwerkt max8 berichten per beurt; de interactieve app max4
tegeltransacties per input/savepass. Geen raw framebuffer of gedeelde userpointer.

**Eindbewijs cloud:** alle 8 native stappen op `344dc0a` slagen: accounts →
persoonlijke synthetische testinvoer → geweigerd/gewist → accountkeuze →
gastlogin → desktop → zoek 'mat' → Materiaal. Iedere app-pixel wordt vergeleken,
zonder interne masks; QEMU raw exit0. TCG/QEMU 10.0.13/q35-10.0/1CPU/256 MiB:
118,79s totaal; Materiaal16,08s versus21,63s op `51d883e` en33,94s op `7648a8f`.
Dit zijn losse metingen in deze cloudomgeving, geen productlatentiegarantie.
Ook de bestaande native Materiaal normal/save/list/reopen-regressie slaagt
met onafhankelijke diskoracle en cleanup (rawexit33, runnerexit0;163,82s).
46 frontendhosttests,27 SDK-apphosttests,15 compositorhosttests,72 Pythonchecks,
39 synthetische PowerShellchecks, Clippy/fmt en scalar-none/objectscan slagen.
De oude verifier telde9 archives terwijl de bestaande SDKzeroize10 oplevert;
hersteld naar de exacte tien namen, met behoud van alle objects-/broncontroles.

**Lokale hardware:** geautoriseerde chat `01a19c13-c83d-7781-8f61-de0541ac6d95`
rapporteert Windows 11 25H2, Ryzen 9 9950X3D,64 GiB. De bytegelijke QMPhelper
slaagt 6/6 op PS 5.1/PS 7.6. De 51d-nativebaseline slaagt 6/6 (3WHPX/3TCG).
De eindmeting op `344dc0a` slaagt 6/6 (3 WHPX/3 TCG), 48 appbeelden exact.
WHPX gebruikerskeuze 6,625 s gemiddeld (6,481–6,823), gast→desktop 4,070 s
(3,932–4,231). De echte PS 5.1-starter kiest Auto→WHPX, toont SDL en sluit
met launcherexit 0. Er blijft geen QEMU-proces achter. Ruwe Windowsrapporten
staan in de lokale testmap; aangeleverde hashes en grenzen zijn vastgelegd
in local-pc-reported.json. Geen hostinstellingen gewijzigd.
Geen netwerk, gedeelde hostmappen, fysieke disks of USB/PCI/GPU-doorvoer.

**Grenzen en hervatten:** de gast gebruikt hetzelfde synthetische appdomein met
vooraf verleende VM-rechten. Het accountformulier is geen beveiligde sessiegrens.
Echte profiel-/verifierpersistentie, auth-/sessiebroker, keyring-unlock, limitering,
compositor/input-lock en revocation blijven nodig; FIDO2/vingerafdruk zijn niet
gekoppeld. Ook dirty-regiontransport, vensterbeheer, algemene appstart en
OS-brede toegankelijkheid blijven open. Score **34 → 34**, geen FR-/fysieke/
menselijke aftekening. [Contract](implementation/SESSION-AND-DESKTOP.md),
[hardware](implementation/VM-HARDWARE-AND-PERFORMANCE.md),
[eindbewijs/reproductie](verification/session-desktop-20261102/README.md).

GitHub Actions op cfed2db meldt startup_failure zonder gestarte jobs; de
oorzaak is niet vastgesteld. Dit is geen groene remote-CI en de PR blijft concept.

Eerstvolgend: onderzoek de CI-startfout en bouw daarna een echte aparte
auth-/sessiedienst vóór beveiligd aanmelden.
Gebruik bestaande tools en behoud andere dirty checkouts/branches.

## Vorig checkpoint — PR #125-inhoudsbehoud — 2 november 2026

**Integratormandaat:** op 2 november is door de projecteigenaar expliciet het
exclusieve integratormandaat van sessie `7f895683` aan deze sessie (`f090b913`)
overgedragen en is de #85/#76-mergepauze opgeheven, met behoud van review-, test-
en branchregels. De oude integrator bevestigde geen lopende/verzonden
transacties, actief WIP of conflictclaim en stopt met operationele integratie.
De wijziging staat ook in [WORK-QUEUE.md](WORK-QUEUE.md). Dit is de actuele
autorisatie; het bewijst niet achteraf dat de desktop→cloud-overdracht vóór
integratie 3 expliciet was.

**Bron:** PR #125, branch `simonvanieperen-studious-waddle`, basis
`c66b593dda8d651e3cc73f0fe65bbe72ff7c7de7`; actuele `main`:
`e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. De inhoudscontrole toonde dat
PR #120 slechts tot `562be4d` was opgenomen. De latere followups tot
`fb1e1a5c25429a0495e861f01602d4c0691f4f03` worden nu met behoud van de
#125-reviewfixes samengevoegd. Opname en bestanden staan in
[pr-integration-3-20261031.json](verification/pr-integration-3-20261031.json).

**Reviewfixes gecommit en gepusht in `4b044f5`:**
- `tools/test_nas.py` bewaart alleen vaste NAS-foutlabels en exact 20-cijferige
  diagnostics, naast bestaande allowlist-markers.
- `tests/test_nas_runner.py` test veilige foutdiagnostiek, te lange waarden en
  tokenbytes; `services/nas/native-probe/CONTRACT.md` beschrijft de precieze
  kernel- en runnercontroles.
- `docs/WORK-QUEUE.md` maakt expliciet dat historische afwezigheid van
  desktopactiviteit geen overdrachtsbewijs is en noteert alleen de nieuwe,
  rechtstreeks door de projecteigenaar bevestigde overdracht van 2 november.

**Vervolgpunten uit #120, nu mede opgenomen:**
- #116: `APP_HANDLE_COUNT` dekt nu de volledige app-handletabel op het
  manifestmoment. Historisch native bewijs en een negatieve proef staan
  in het integratierapport; de huidige combinatie vereist nieuwe CI.
- #119: de lekcontrole draait op de ruwe QEMU-bytes vóórdat er iets wordt
  weggeschreven: elk 16-bytevenster per token, ruw en als hex. Een misvormde
  capture wordt in zijn geheel doorzocht; bij een lek komen er alleen
  placeholders. Lost ook de Codex-P1 en de CodeRabbit-securitybevinding op
  #120 op. De huidige #125-allowlist voor serialdiagnostiek blijft behouden.
- `.github/workflows/materiaal-probe.yml`: voorlopig alleen
  `workflow_dispatch`. Een tweede lokale run met de CI-commandoreeks liep na
  208 s vast op de gastwatchdog (~200 s), met dezelfde tegeltellingen als de
  geslaagde run. De timingmarge blijft open; geen watchdog/grens verruimd.

**Lokale verificatie:** `python3 -B -m unittest discover -s tests -p
'test_nas_*.py' -v` — exit 0, 39 tests (35 runner); volume-runner: 3 tests;
private replay/oracle: 17 tests; Materiaal-probeharnas: 23 tests, alle exit 0.
Logs: `/tmp/nieuw-os-pr125-preservation-{nas-all,volume,replay,materiaal}.log`.
Rustfmt root/Materiaal en capability-core-Clippy slagen (exit 0).
De gerichte capability-tests starten niet: exit 101, hostlinker `cc`
ontbreekt; log `/tmp/nieuw-os-pr125-preservation-capability.log`.
Er zijn geen systeempackages geinstalleerd. Dit is geen native acceptatie.

**CI vóór deze samenvoeging:** op `c66b593` slaagde onder meer de native
NAS normal/cold-gate, run `41524587617`. De artifact noemt checkout
`ef0796b5263958107d17e1a52b9fea83768e1797` (GitHub-testmerge). Ouders zijn
exact de hierboven genoemde main/head; boom `1174b01d9fbc0c245f751c5507acd7502696f25d`
is gelijk aan `c66b593`. De bronbinding is via de API bevestigd.
Deze run bewijst de nieuwe samenvoeging niet.

**Open issues:** #109/#110 blijven open tot aantoonbare main-integratie.
#39–#43 missen volledige native, fysieke, sleutel- of routeacceptatie.
Kernelbuilds moeten buiten een geneste checkout draaien om dubbele
`.cargo/config.toml`-linkerargumenten te voorkomen.

**Parallel:** de twee #126-reviewfixes van sessie `c44e6fed` zijn ontvangen
als `5dead467d491107f7244e00450435c41072d6dc7`, inhoudelijk bekeken en door
de integrator fast-forward naar de bestaande #126-branch gepusht; remote-SHA
bevestigd. Rust-hosttests en native CI blijven vereist; nog geen merge.
Die sessie werkt nu uitsluitend aan vier #127-UI-reviewfixes op een aparte
branch (palet, proportionele klik, hoverdamage, semantisch wachtwoord).
Agent `87ef37fe` herstelt alleen
de twee TCP-reviewfouten uit #127 in een afzonderlijke worktree. Geen claims
op centrale status/score of elkaars bestanden.

**Voortgang:** 34/100 → 34/100; geen rubriccriterium is nieuw geaccepteerd.

**Eerstvolgend:** valideer en push deze inhoudsbehoudende samenvoeging, volg
de nieuwe exacte-head CI en review. Integreer pas
wanneer alle vereiste gates groen zijn en de delta/overlap met actuele `main`
is gecontroleerd; sluit vervangen PR's/issues pas na aantoonbaar inhoudsbehoud.

## Vorige integratiehandoffs

Integratie 3 (#115–#119) is vastgelegd in
[pr-integration-3-20261031.json](verification/pr-integration-3-20261031.json).
Integraties 1 en 2 staan in
[pr-integration-20261030.json](verification/pr-integration-20261030.json) en
[pr-integration-2-20261030.json](verification/pr-integration-2-20261030.json).
Oudere handoffs staan in het [archief van 30 oktober](history/integration-2026-10-30/README.md).
