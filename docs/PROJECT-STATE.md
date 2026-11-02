# Actuele projectstatus

## NEXT AGENT HANDOFF — PR #125-review en issue-integratie — 2 november 2026

**Integratormandaat:** Simon heeft op 2 november expliciet het exclusieve
integratormandaat van sessie `7f895683` aan deze sessie (`f090b913`)
overgedragen en de #85/#76-mergepauze opgeheven, met behoud van review-, test-
en branchregels. De oude integrator bevestigde geen lopende/verzonden
transacties, actief WIP of conflictclaim en stopt met operationele integratie.
De wijziging staat ook in [WORK-QUEUE.md](WORK-QUEUE.md). Dit is de actuele
autorisatie; het bewijst niet achteraf dat de desktop→cloud-overdracht vóór
integratie 3 expliciet was.

**Bron:** PR #125, branch `simonvanieperen-studious-waddle`, reviewfixcommit
`4b044f5e769053c851863693963657e934725133`; basis `main` is
`e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. PR #120-head
`fb1e1a5c25429a0495e861f01602d4c0691f4f03` bevat de bron voor #109/#110 en
componentdelta's; opname en bestanden staan in
[pr-integration-3-20261031.json](verification/pr-integration-3-20261031.json).

**Reviewfixes gecommit en gepusht in `4b044f5`:**
- `tools/test_nas.py` bewaart alleen vaste NAS-foutlabels en exact 20-cijferige
  diagnostics, naast bestaande allowlist-markers.
- `tests/test_nas_runner.py` test veilige foutdiagnostiek, te lange waarden en
  tokenbytes; `services/nas/native-probe/CONTRACT.md` beschrijft de precieze
  kernel- en runnercontroles.
- `docs/WORK-QUEUE.md` maakt expliciet dat historische afwezigheid van
  desktopactiviteit geen overdrachtsbewijs is en noteert alleen de nieuwe,
  rechtstreeks door Simon bevestigde overdracht van 2 november.

**Lokale verificatie:** `python3 -B -m unittest discover -s tests -p
'test_nas_runner.py' -v` — exit 0, 20 tests; log
`/tmp/nieuw-os-nas-runner-review-fixes.log`. `git diff --check` — exit 0.
Dit is hosttestbewijs, geen nieuwe native productacceptatie.

**Exacte-head CI vóór deze delta:** op `2b479c1` slaagden alle relevante
checks, waaronder `host-quality`, `qemu-smoke`, `native-system`,
`nas-native-normal-cold`, `b03-native-update`, de componentchecks en CodeRabbit.
De laatste review meldde drie opmerkingen; twee code/documentatiepunten zijn
lokaal hersteld en de overdrachtshistorie is transparant bijgewerkt. De nieuwe
delta staat nu op de remote branch; exacte-head CI voor `4b044f5` moet nog
voltooien.

**Open issues:** #109/#110 blijven open totdat de vervangende code aantoonbaar
in `main` is opgenomen. #39 vereist nog de volledige N01-pakket-/transport- en
negatievenmatrix; #40 volledige native compositor/sessieacceptatie; #41
productiesleutelrotatie/-ankers en bevoegde ondertekening; #42 USB/HID/hotplug
en fysieke acceptatie; #43 de daadwerkelijk uitgevoerde route naar 50/100.
Geen van die open bewijsvereisten wordt gesloten met alleen deze tests.

**Voortgang:** 34/100 → 34/100; geen rubriccriterium is nieuw geaccepteerd.

**Eerstvolgend:** voer exacte-head CI en de bijgewerkte review voor `4b044f5`
uit. Integreer pas
wanneer alle vereiste gates groen zijn en de delta/overlap met actuele `main`
is gecontroleerd; sluit vervangen PR's/issues pas na aantoonbaar inhoudsbehoud.

## Vorige integratiehandoffs

Integratie 3 (#115–#119) is vastgelegd in
[pr-integration-3-20261031.json](verification/pr-integration-3-20261031.json).
Integraties 1 en 2 staan in
[pr-integration-20261030.json](verification/pr-integration-20261030.json) en
[pr-integration-2-20261030.json](verification/pr-integration-2-20261030.json).
Oudere handoffs staan in het [archief van 30 oktober](history/integration-2026-10-30/README.md).
