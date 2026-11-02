# Actuele projectstatus

## NEXT AGENT HANDOFF — open issues en integratie-PR 120 — 2 november 2026

**Bron:** branch `simonvanieperen-studious-waddle`, HEAD
`e08fb0e48c1bb0781e98357f588c2f73b6ae8500`; schone werkboom vóór deze
statusupdate. De vaste rubric blijft **34/100 → 34/100**.

**#109 en #110:** hun code is opgenomen in PR #120
(`562be4d13b9d039d7e8fece2f0fc26e767dccb15`), die de exacte PR-heads #115 en
#119 als ancestors bevat. Review van de volledige integratiekandidaat vond
geen concrete blocker; testuitvoering van die reviewagent was nul. PR #120 is
op 2 november gereed voor review gemaakt; exacte-head CI en CodeRabbit liepen
bij dit checkpoint nog. De twee issues zijn dus **niet gesloten of als
opgelost gemeld**. Merge alleen na groene vereiste checks en afhandeling van
concrete reviewpunten. #109 blijft host-/targetcontractbewijs, geen native
persistente adapter. #110 blijft uitsluitend native acceptatieharnasbewijs.

**Nog open:** #39 N01 mist de volledige native netwerk-/negatievenmatrix;
#40 U01 mist volledige geïsoleerde sessie-/compositorherstelacceptatie; #41
B03 mist productie-sleutelrotatie/-ankers en vraagt een afzonderlijke
ondertekenopdracht; #42 H02 vereist USB/HID/hotplug plus fysieke hardwaretests;
#43 is de uitvoeringsroute naar 50/100 en is niet bereikt. Geen hiervan mag
worden gesloten met plannen, hosttests, componentproeven of AI-review.

**Eerstvolgend:** controleer de actuele head en
`gh pr checks 120 --repo simonvanieperen/nieuw-os`; los concrete reviewpunten
op, merge alleen met passende groene gates en herbevestig daarna de exacte
main-commit. Sluit onderliggende PR's/issues pas nadat inhoudsbehoud is
geverifieerd. Werk vervolgens score en checkpoint opnieuw bij.

## Vorige handoff — integraties 1 en 2 — 30 oktober 2026

Integratie 1 (#96) is gesquash-merged als `c0b7147`; integratie 2 (#113) heeft
implementatiebron `ed63ed259746`. De bron- en reviewarchieven staan in
[pr-integration-20261030.json](verification/pr-integration-20261030.json) en
[pr-integration-2-20261030.json](verification/pr-integration-2-20261030.json).

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
