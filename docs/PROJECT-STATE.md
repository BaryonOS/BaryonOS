# Actuele projectstatus

## NEXT AGENT HANDOFF — integratie 3 (open PR's na #113) — 31 oktober 2026

**Mandaat:** Simon vroeg op 31 oktober in een Claude Code-cloudsessie om
verder te gaan waar de vorige sessies stopten. Deze sessie zette daarom het
integratormandaat van 30 oktober voort (zie [WORK-QUEUE.md](WORK-QUEUE.md));
er was geen activiteit van de desktopsessie na 30 oktober 21:48 UTC zichtbaar.

**Bron:** main `e08fb0e`; integratiebranch `ccr-4bb5b18c-t9u3cv`.
Alle bronnen, reviews, fixes en uitgevoerde commando's staan in
[pr-integration-3-20261031.json](verification/pr-integration-3-20261031.json).

**Opgenomen:**
- #118: herstel van vier fouten uit de post-merge review van #113
  (Materiaal-focus, pointerstatus, DHCP-retryschema per XID).
- #115: app-management-leeszijde via `artifact-store-core` (sluit #109).
- #116: B02-reviewherstel; de vacuüme `APP_ADMIN_HANDLE`-kernelcontrole
  vergelijkt nu endpointobjecten.
- #119: exacte boot-1 NAS-tokenreplay native actief (sluit #110).
- #117: SUR1 sparse frames; na het eerste volledige frame worden alleen
  gewijzigde tegels plus de laatste tegel verstuurd.
- Reviewfixes: doc-overclaim over FakeStore, Nederlandse R113-notitie,
  sterkere #117-tests (mutatie maakt ze rood), loze assert in de
  NAS-runnertest, herkomstnotities.
- #116 en #119 zijn onafhankelijk door AI gereviewd (MERGEN, geen
  blockers). Dat is geen menselijke audit.

**Native lokaal bewijs (QEMU 8.2.2, `pc-q35-8.2`, TCG):** NAS normal/cold
en FR-I01 normal/recover `passed=True raw=33`. De Materiaal-probe
[slaagt met #117](verification/sparse-frames-117/README.md) in 178,8 s;
het frame na 28 getypte toetsen kost 9 in plaats van 1619 tegels. Main
zelf faalt die probe in deze omgeving (raw 35, gasttime-outs). Dat is
omgevingsafhankelijk, één run per bron, en geen CI-gate.

**Niet opgenomen:** #114 (ADR-058, herkenbare desktop/inloggen). Het
overschrijft de productambitie in de root-`AGENTS.md`. De PR schrijft het
besluit aan Simon toe; dat is hier niet te verifiëren. Er staan 15 open
CodeRabbit-threads, waarvan 5 major (secure-attention-oppervlak,
sleutelovergang bij gebruikerswissel, te ruime app-claim, reproduceerbare
prestatie-acceptatie, positieve hersteltest). Wacht op Simons bevestiging.

**Open vervolgpunten:**
- #116: de manifestcontrole dekt alleen de zes bekende app-handles. Voeg
  een `available_slots`-telling toe (patroon `random_fixture.rs`); daarna
  is nieuw native bewijs nodig.
- #119: de lekcontrole in `tools/test_nas.py` zoekt alleen het volledige
  80-bytepaar en schrijft `serial.txt` vóór de controle.
- `native-foundation` faalde op #117 in de RANDOM-rawgate (raw 35 na
  `SYSTEM:MANIFEST:PASS`); die image bevat geen #117-code. Als dit opnieuw
  optreedt, onderzoek dan of het flaky is.
- De native Materiaal-probe draait in geen enkele CI-workflow.
- #39–#43: productmilestones; hun criteria zijn niet aangetoond.

**Score:** 34/100 → 34/100. Geen criterium is volledig geaccepteerd:
host-, component- en lokaal native deelbewijs vervangen geen FR-gate,
honderd koude starts, fysieke of menselijke acceptatie.

**Blocker (31 oktober, 14:22 UTC):** GitHub Actions start op #120 geen enkele
workflow. Push en pull_request eindigen allebei in `startup_failure` onder een
placeholder `BuildFailed`, met 0 jobs. Er zijn geen workflowwijzigingen
t.o.v. main. De laatste normale runs waren op 30 oktober rond 21:30 UTC. De
oorzaak ligt waarschijnlijk op account- of repo-niveau (Actions-tegoed of
-limiet, of een gewijzigde instelling); dit is niet geverifieerd. Simon moet
dit in de GitHub-instellingen controleren.

**Eerstvolgend:** controleer de CI van de integratie-PR (#120). Bij groen:
squash-merge, de vijf component-PR's sluiten als overgenomen, en #109 en
#110 sluiten. Daarna #114 na Simons antwoord.

## Vorige handoff — PR- en issueafhandeling na integratie 1 en 2 — 30 oktober 2026

**Mandaat:** Simon heeft de Claude Code-sessie in de desktopapp op 30 oktober
(~21:10 CET) aangewezen als enige operationele integrator voor alle open PR's
en issues, inclusief mergen en sluiten. Merges naar main lopen via die
integrator; tussentijdse losse merges (#71 en #74 door Simon zelf) dwingen een
nieuwe up-to-date-CI-run. Vastgelegd in [WORK-QUEUE.md](WORK-QUEUE.md).

**Broncommits:** #96 (integratie 1) gesquash-merged als `c0b7147`.
Integratie 2 (#113) heeft implementatiebron `ed63ed259746`. De bronnen en
oplossingen staan in [pr-integration-20261030.json](verification/pr-integration-20261030.json)
en [pr-integration-2-20261030.json](verification/pr-integration-2-20261030.json).

**Afgehandeld:**
- #68, #72–#81, #83, #86, #88, #89, #92, #94 en #97–#107 zijn als overgenomen
  gesloten. Per PR is gecontroleerd dat elke commit na de vastgelegde bron,
  exclusief main, aantoonbaar in main zit.
- #111 is gesloten. Het was een parallelle, oudere subset van #96 en #113; het
  bewijs en archief ervan zijn behouden. #112 is gesloten als automatisch
  duplicaat van #113.
- Reviewfixes in #96:
  - net-stack heeft een expliciete discard-route voor een nooit aangeboden,
    ingehouden TX-frame, met regressie (rood en daarna groen);
  - het dependencyregister is aangevuld;
  - in het werkregister zijn de spaties hersteld.
- Integratiefix in #113: de Materiaal-chaintest volgt het nieuwe
  semantische buffercontract. Dat was de oorzaak van de rode CI op #107.

**Open punten:**
- #109: app-management-reader naar `artifact-store-core`.
- #110: exacte boot-1 NAS-replay native activeren. De replay-gastcode uit #93
  is bewust niet actief, omdat de native normal/cold-gate anders vast faalt.
- #39–#43: productmilestones; hun criteria zijn niet aangetoond.

**Bewijs en score:** 34/100 → 34/100. Er zijn geen nieuwe native-, FR-,
fysieke of menselijke acceptaties. Hosttests, freestanding-builds en AI-review
zijn geen productpunten. Niet lokaal uitgevoerd: de freestanding-build van
`app-package-core` (lokale rustc gaf SIGILL); CI is de gate.

**Eerstvolgend commando:** `gh pr list --state open`. Beoordeel nieuwe PR's
tegen main met `docs/verification/pr-integration-2-20261030.json` als
bronbasis. Een volgende integratie start bij #109 of #110, of bij een
DEV1-herproef op vastgelegde bron en slot.

## Eerdere handoffs

De handoff over centrale documentatie en bronbehoud (#96) staat bytegetrouw in
[`e08fb0e:docs/PROJECT-STATE.md`](https://github.com/simonvanieperen/nieuw-os/blob/e08fb0e48c1bb0781e98357f588c2f73b6ae8500/docs/PROJECT-STATE.md);
oudere handoffs staan in het [archief van 30 oktober](history/integration-2026-10-30/README.md).
