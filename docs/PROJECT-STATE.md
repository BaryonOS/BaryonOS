# Actuele projectstatus

## NEXT AGENT HANDOFF — volume-admissie native gerepareerd — 2026-11-02

**Bron:** `94f12da0b9740a1a0ced1f08b4a30787f82ae79f`, basis `74a34d05ef5591b782a0da9da34e4669910d0722`;
branch `codex/volume-format-admission-20261102`, [concept-PR #133](https://github.com/simonvanieperen/nieuw-os/pull/133).
Fingerprint v3 `2cffba892608c907b717158a83b8a1c66c7d79dc5843d2d03eabc4ff1831c028`.

**Resultaat:** recovery vereist een volledig geldige eigen tabelkopie;
expliciet vooraf geformatteerde canonieke generatie 0 werkt. Onbekende en
nulgevulde volumes worden tijdens servicestart geweigerd vóór beschikbaar-
stelling van opslag. Bestaande complete generaties en eerste-commit-crash-
recovery blijven ondersteund. Eigen kernel behouden; geen nieuw driver-,
runtimeformat-, IPC- of capabilitypad.

**Bewijs:** 10 native matrixruns slagen: eigen normal/cold en Btrfs,
OpenZFS-mirrorlid, rawzero, nulheaders+payload elk read-only én schrijfbaar.
Alle acht negatieven: startup-Corrupt/-18, raw35, nul WRITE-traces, volledige
bron/kopiehashgelijkheid. Normal registreert 4963 echte writes; cold behoudt
exacte bytes/generatie84/digests. De bestaande 9-VM-volume/crashgate slaagt
met drie werkelijke kills en gekoppelde koude herstelruns. Geen timeouts.
Host: 45 volume-core + Clippy, 28 document-session, 24 Materiaal, 141 Python-
harnastests slagen; rode regressie en omgevingsfouten zijn behouden.

**Bewijsgrens/herstel:** uitsluitend wegwerpbestanden/QEMU; geen fysieke
powercut of native Btrfs/ZFS-engine. AI-teamreview is geen menselijke audit.
Oudere code weigert sealed-gen0-lege images: rollback neemt code/tooling
samen terug en formatteert geen data. FNV is geen authenticatie.
ADR-053 en [bewijsbundel](verification/volume-format-admission/README.md).
Geen nieuw criterium; **34/100 blijft 34/100**.

**Volgende stap:** review/integratie van PR #133 via het actuele exclusieve
integratormandaat; deze sessie merge niet naar main. Native backendport en
S09-herstel/export blijven afzonderlijk open. Het eerdere native gatebewijs
op e08/74a blijft historisch geldig voor die bronversies.

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
