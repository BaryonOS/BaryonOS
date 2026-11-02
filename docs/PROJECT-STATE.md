# Actuele projectstatus

## NEXT AGENT HANDOFF — expliciete volume-admissie — 2 november 2026

**Basis:** `74a34d05ef5591b782a0da9da34e4669910d0722`; werkbranch
`codex/volume-format-admission-20261102`. Eigen kernel behouden; uitsluitend
wegwerpimages; geen nieuwe bestandssysteemdriver.

**Hostcheckpoint:** recovery vereist minstens één volledig geldige eigen
metadata-kopie. Canonieke sealed generatie 0 in slot 0 wordt expliciet door de
offline fixturemaker geschreven; nul-/vreemde/onvolledige metadata wordt niet
meer als lege opslag toegelaten. Bestaande complete generaties en herstel na
een onderbroken eerste commit blijven ondersteund. Generatie-adjacentie is
overflowveilig. Geen kernel-, capability-, IPC- of runtimeformatverruiming.

**Bewijs tot dit checkpoint:** nieuwe rawzero-regressie eerst rood; 45 Rust
volume-coretests en Clippy groen, 28 document-session- en 24 Materiaal-apptests
groen. Het onafhankelijke Python-orakel en afhankelijke harnassen: 141 tests
groen. Windows-platformfouten zijn behouden en betreffende suites ongewijzigd
onder WSL groen herhaald. Alleen bestaande locked afhankelijkheden naar de
lokale cache opgehaald; lockfiles ongewijzigd. Native proeven zijn hier nog
**niet uitgevoerd** en worden op de hierna bevroren broncommit geregistreerd.

**Contract:** ADR-053 en `docs/verification/volume-format-admission/TASK.md`.
Native plan: eigen normal/cold, vier onbekende beelden elk read-only én
schrijfbaar, trace reads >0/writes=0 en volledige imagehashgelijkheid; daarna
de bestaande negen-VM-volume/crashregressie. Geen nieuw rubricpunt; **34/100
blijft 34/100**. Oudere code weigert nieuwe sealed-gen0-lege beelden: rollback
neemt code en fixturetooling samen terug, formatteert geen bestaande data.

**Volgende stap:** native uitvoering op exact checkpoint volgens de externe
runner `C:/Users/simon/Documents/Codex/2026-11-02/volume-format-admission/native/native_admission.py`.

## NEXT AGENT HANDOFF — PR #125-inhoudsbehoud — 2 november 2026

**Integratormandaat:** Simon heeft op 2 november expliciet het exclusieve
integratormandaat van sessie `7f895683` aan deze sessie (`f090b913`)
overgedragen en de #85/#76-mergepauze opgeheven, met behoud van review-, test-
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
  rechtstreeks door Simon bevestigde overdracht van 2 november.

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
