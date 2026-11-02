# Actuele projectstatus

## NEXT AGENT HANDOFF — volledige OS-journey vastgelegd — 2 november 2026

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

## Vorige handoff — centrale documentatie en bronbehoud (#96) — 30 oktober 2026

**Integratieopdracht:** Simon heeft in deze sessie het oplossen, squashen en mergen
van issues en pull requests opgedragen. Dit actuele mandaat geldt voor deze
integratie; de historische queue en bronclaims blijven bewaard.

**Bronbasis:** main `d791a342cc3bdb107245692f2f542d88d5fda7f1`;
management en laatste overgenomen queue: PR #75 op
`c26f57b60d4eaffbb57c0eb5114fa00bd7254eb6`.
[Bronindex en volledige historie](history/integration-2026-10-30/README.md)
scheiden deze basis van oudere branchhandoffs. De gecombineerde implementatiebron is
[`3fedd03e`](https://github.com/simonvanieperen/nieuw-os/commit/3fedd03eecff62d6bffae038ec959c56734e2137).
Gerichte SDK- en netwerkregressies zijn groen; volledige kandidaat-CI
en de native NAS normal/cold-gate blijven vereist vóór merge.

**Eigenaars en claims:** de nieuwste vastgelegde [werkqueue](WORK-QUEUE.md)
blijft de primaire bron voor taken, fileclaims en acceptatie.
Assistent `5d5f8513-2d31-41af-96ce-81c5ef56ef80` is de enige queuewriter;
teamleider 89f houdt technische samenhang, PROJECT-STATE en scorebestanden.
De queueovername en e8-vrijgave/7f-ontvangst zijn vastgelegd; bestaande
inhoudseigenaren en opdrachten blijven behouden. Stilte of een timeout
geeft geen schrijfgebied vrij. De queuebron bewaart ook de afzonderlijke
mergepauze rond #85; de managerhandoff beschrijft 7f als operationele
integrator. Dit bronverschil is [expliciet geregistreerd](history/integration-2026-10-30/README.md#overdracht-en-bronverschil);
dit checkpoint verzint geen later stop- of hervattingsbesluit.

**Product en bewijs:** 34/100 blijft 34/100; criteria, gewichten, statussen
en geaccepteerde evidence blijven gelijk aan de main-bron.
Alle 62 FR-gates, 100 cold-starts, 24 uur, fysieke hardware en menselijke
acceptatie blijven open. Het [DEV1-resultaat](https://github.com/simonvanieperen/nieuw-os/blob/557b85e26372cf3809c0760d40b430ec5ef7cc6b/docs/verification/ordinary-dev1/reference-ac4f038e86/result.json)
op bron `ca45a7dabe6a040921de03bdc2fad3e5045a75ba` is mislukt
(`passed=false`, raw exit 0), vóór het Materiaalmanifest.
De [gecorrigeerde buildfreeze](https://github.com/simonvanieperen/nieuw-os/blob/557b85e26372cf3809c0760d40b430ec5ef7cc6b/docs/verification/ordinary-dev1/isolated-build-1/build-freeze.json)
heeft `vm_gates_started=false`: geen nieuwe guestherproef uitgevoerd.
Bron: Nieuw OS / S.W.J.B.M. van Ieperen, 30 oktober 2026;
[volledige metadata en bewijsgrens](history/integration-2026-10-30/README.md#dev1-bronrecord).
UI0818 is beperkt native deelbewijs. Store/READ, issuer,
launch, TLS en NAS-lifecycle blijven productafhankelijkheden.
B05 heeft in de managementbron geen bevestigde auditor; deze integratie
is geen menselijke audit, externe opdracht of releaseaftekening.
Het [cold77-hostverslag](verification/teamleader-cold77-host.json)
houdt sourcefix, native herhaling en mergegoedkeuring apart.

**App-management:** pakketvalidatie, SDK-identiteit, catalogus, protocol en
transactiebewaking zijn samengevoegd. De gecombineerde SDK-hostsuite heeft
26 tests; een testbackend bewijst geen duurzame journal- of native launch-keten.
De volledige integratiechecks en de native NAS normal/cold-gate moeten op de
feitelijke integratiecommit slagen. PR #93 blijft buiten deze integratie:
de export en startupbinding van de vorige NAS-boot ontbreken nog.

**Eerstvolgend:** controleer live head/base, concrete reviewpunten en
vereiste checks voor de feitelijke integratiebron. Laat het progressscript
README en het voortgangsoverzicht uit die bron genereren, met de vaste
rubric en zonder oude assessments als nieuwe testuitvoering te presenteren.
Behoud lopende domeintaken; voer een DEV1-herproef alleen op de daarvoor
vastgelegde bron en bestaande slotafspraak uit. De historische volgorde,
queued retries en oude groene runs zijn geen actuele integratiegate.

**Historie:** de volledige main-DHCP-handoff, PR #75-managementgeschiedenis,
beide PR #65-teksten plus hun 0e441a9-voorganger, en de oudere PR #73/#79-
handoffs staan bytegetrouw in het [gelinkte archief](history/integration-2026-10-30/README.md).
Daar blijven oorspronkelijke testbronnen, onzekerheden en assessments
behouden. Alleen dit bovencheckpoint is het actuele hervatpunt.
