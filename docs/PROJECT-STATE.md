# Actuele projectstatus

## NEXT AGENT HANDOFF — ontwerp en onderzoek inhoudsbehoudend combineren — 2 november 2026

Basis `517117fbd8359084eca10e2e2d5f4cdf1015b52c` (PR114 en main74a34d05)
met PR131 `a03f7f0c355cedca72611893f07c4a766a7b7d4f`. Integrator `f090b913`.
Geen runtimecode uit onderzoek; ADR-058 blijft leidend voor de shell,
P2-bevriezing betreft het historische prototype. De onafhankelijke
read-only broncontrole bevestigt de Eurobarometer58%-dienstoverstapclaim
tegen de originele data-annex/PDF-hash; dit is geen OS-adoptiepercentage.
De onderzoeks- en ontwerpcheckpointteksten hieronder blijven historisch
behouden; hun toenmalige branchstatus is geen actuele integratie-instructie.
Score34 blijft34; PR127 is nog een afzonderlijke native ontwikkelkandidaat.
Vervolg: opslagbewijs en overige unieke documentatie behouden, gegenereerde
voortgang controleren en verse exacte-head CI/review afwachten.

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

Bron `c1d62d83acc8357c698c0ef576367834e81d4a21`, alleen documentatie.
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

## Historisch checkpoint — PR125-inhoudsbehoud — 2 november 2026

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
