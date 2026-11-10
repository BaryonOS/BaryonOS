# Codex: korte ontwikkellus met behoud van bewijs

AGENTS.md is leidend. Dit is een uitvoeringshandleiding, geen tweede roadmap,
veiligheidsbewijs of garantie over snelheid. Actuele projectstatus en criteria
blijven in docs/PROJECT-STATE.md en docs/progress.json.

## 1. Hervatten zonder opnieuw te beginnen

Controleer eerst de daadwerkelijke checkout; de laatst besproken branch of
chatgeschiedenis hoeft niet de actuele code te bevatten:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Lees het nieuwste statusblok, de relevante contractspecificatie en geneste
AGENTS.md. Bekijk alleen de diff vanaf het geverifieerde vorige checkpoint.
Een ontbrekend checkpoint of onverwachte wijziging vraagt gericht onderzoek,
geen blind vertrouwen in een oude testclaim. Niet-gecommit lokaal werk van
Claude verschijnt niet automatisch in een andere cloudtaak.

Kies de eerstvolgende haalbare taak uit de bestaande projectstatus/roadmap.
Leg kort vast: **doel; afhankelijkheid; schrijfbestanden; acceptatiecommando;
bewijsgrens**. Controleer eerst of de implementatie of PR al bestaat.

## 2. Eerst gerichte feedback, daarna integratie

Voorbeelden uit de bestaande CI, niet een nieuwe definitie van alle vereiste
tests. Kies de suite die werkelijk door de wijziging wordt geraakt. Bij een
wijziging in een gedeelde crate moeten ook de relevante afnemers worden getest.

```sh
# Voorbeeld: een wijziging in volume-core.
cargo +1.90.0 test --locked -p volume-core
cargo +1.90.0 clippy --locked -p volume-core --all-targets -- -D warnings

# SDK: eigen workspace.
cargo +1.90.0 test --manifest-path sdk/Cargo.toml --locked

# Host-harnas voor DocumentSession.
python3 -m unittest discover -s tests -p 'test_document_session_*.py'

# Productprototype; NAS heeft een aparte suite.
node --test --test-isolation=none ui/prototype/tests/*.test.mjs
node --test --test-isolation=none services/nas/tests/*.test.mjs
```

Controleer formattering vroeg, vóór een kostbare integratierun. Gebruik bij
integratie de actuele commando's uit `.github/workflows/` en de relevante
native gate; kopieer geen verouderde testlijst uit een chat. Laat negatieve
capability-, herstel- en foutinjectietests intact. Een hosttest bewijst geen
native werking. Sla nooit een controle over omdat een cache werd hersteld.

## 3. Parallel werken zonder lock- of integratieverlies

Gebruik de agentlimieten uit AGENTS.md; meer agents is geen zelfstandig doel.
Deel alleen echt onafhankelijke taken op, met één eigenaar per interface en
één integrator. Deel het contract en de relevante code, niet telkens alle
historische documentatie. Een reviewer kan read-only meekijken.

Iedere schrijver krijgt een eigen worktree/branch. Native fixtures gebruiken
hun bestaande geïsoleerde `NOS_BUILD_DIR`, `NOS_TARGET_DIR` en
`CARGO_TARGET_DIR`; vermeng geen profielen of images. Laat twee Cargo-processen
niet dezelfde targetmap blokkeren. Voorkom dat agentparallelisatie én
onbegrensde buildparallelisatie tegelijk CPU en geheugen uitputten. Schaal
alleen op wanneer gemeten doorlooptijd daalt en integratiewerk beheersbaar blijft.

## 4. Wat de CI-cache wel en niet hergebruikt

De hoofdworkflow behoudt `host-quality`, `qemu-smoke`, hun volgorde, alle
bestaande commando's, time-outs en leesrechten. Andere acceptatieworkflows
zijn niet gewijzigd.

- Beide jobs mogen Cargo-registry-indexen en crate-archieven hergebruiken,
  gescheiden naar runner-OS, architectuur en de inhoud van alle Cargo.lock-files.
- Alleen `host-quality` hergebruikt de expliciet opgesomde **host-debug**
  targetmappen. De sleutel bevat daarnaast toolchain, manifests, Cargo-config
  en buildscripts. Er zijn geen brede restore-prefixes naar andere sleutels.
- Geen credentials, volledige home-directory, native release-targets,
  QEMU-images, sleutels, rapporten of acceptatiebewijs in deze caches.

`actions/cache` is pinned to the reviewed v6.1.0 commit
`55cc8345863c7cc4c66a329aec7e433d2d1c52a9`. The upstream `v6.1.0` tag
points directly to this complete commit, whose signature GitHub reports as
valid. `action.yml` uses Node 24; `package.json` declares version 6.1.0,
Node `>=24`, and the MIT license. The minimum runner version therefore remains
2.327.1. This is a CI dependency, not an OS runtime library. Maintain this pin
through the existing review and dependency process; this metadata and tag
verification is not an audit of all bundled action code.

According to the upstream changelog, version 6.1.0 handles read-only cache
access and demotes a redundant cache-save warning to debug level. This does
not expand token permissions: the cache remains best-effort, and an
unavailable save does not constitute a successful build or test step.

De cache is best-effort. Een miss voert de normale build uit. Cachekeys zijn
onveranderlijk: een hit betekent niet dat de nieuwste projectcode al is
gecompileerd. Cargo en alle testcommando's worden opnieuw aangeroepen.
De workflow-summary toont cache-hitdiagnostiek, geen geslaagd testresultaat.
De verplichte schone native testmappen worden niet om de cache heen aangepast.

## 5. Meten vóór een snelheidsclaim

Gebruik bestaande job- en steptijden in GitHub Actions. Vergelijk bij voorkeur
meerdere runs met dezelfde bron, suite, runnerklasse en toolversies: eerst
zonder passende cache, daarna met cache. Noteer cache-hit/miss, wachttijd,
build-/testduur, exitstatus en opnieuw te doen werk. Vergelijk niet een kleinere
testsuite met een volledige native gate alsof alleen de uitvoering sneller werd.

Een lokale Linux-meting kan bijvoorbeeld zo, zonder een pipeline die een fout
verbergt:

```sh
/usr/bin/time -p cargo +1.90.0 test --locked -p volume-core
```

Gebruik dezelfde sessie en geldige omgeving voor samenhangend vervolgwerk.
Een GitHub-wijziging stelt **niet** automatisch de gepubliceerde
Codex-cloudomgeving, modelkeuze, budget of persoonlijke config.toml in.
Vermijd herinstallatie, routine-`cargo clean` en onnodige volledige audits.
Een cache die duurder blijkt dan opnieuw bouwen kan weer worden verwijderd.

## 6. Overdracht en herstel

Werk de bestaande PROJECT-STATE.md en voortgangsassessment bij volgens
AGENTS.md. Gebruik daar één compacte `NEXT AGENT HANDOFF`, bijvoorbeeld:

```text
Bron: <branch + volledige HEAD, base en relevante PR>
Doel/resultaat: <één controleerbare implementatiestap>
Open werk: <bestanden, eigenaar, nog niet geïntegreerde wijzigingen>
Bewijs: <exact commando, exitstatus, logpad; of NIET UITGEVOERD>
Blokkade: <oorzaak en noodzakelijke bevoegdheid/afhankelijkheid>
Volgende stap: <één direct uitvoerbaar commando of implementatietaak>
```

Geen extra roadmap, gefingeerd agentwerk of kunstmatige scoreverhoging.
Deze workflowoptimalisatie accepteert zelf geen nieuw OS-criterium.
Bij een cachingprobleem: herhaal eerst de volledige betrokken controle zonder
cache; verhoog zo nodig alleen de cacheversieprefix in een beoordeelde wijziging.
Behoud testgrenzen en beveiliging. Terugdraaien kan met een normale revert van
de workflowwijziging, zonder bestaande branches te resetten.

## Bronnen

GitHub. (z.d.). *Dependency caching reference*.
https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching

GitHub. (2026). *actions/cache*, version 6.1.0.
https://github.com/actions/cache/tree/55cc8345863c7cc4c66a329aec7e433d2d1c52a9

GitHub. (June 26, 2026). *actions/cache v6.1.0 release notes*.
https://github.com/actions/cache/releases/tag/v6.1.0

OpenAI. (z.d.). *Using Codex Cloud*.
https://help.openai.com/en/articles/20001545-using-codex-cloud

The cache source, tag, and release metadata were verified on November 10,
2026; the remaining sources were accessed on October 29, 2026. Repository
commands are aligned with base commit
`f63a0c1b56fdcd28e43579aa6e9c5a56e1e2cbdb`; consult the current source files
afterward.
