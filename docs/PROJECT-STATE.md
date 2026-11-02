# Actuele projectstatus

## NEXT AGENT HANDOFF — native sessiepreview en QEMU — 2 november 2026

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
Dit was de status vóór de actuele base-integratie.

Eerstvolgend: onderzoek de CI-startfout en bouw daarna een echte aparte
auth-/sessiedienst vóór beveiligd aanmelden.
Gebruik bestaande tools en behoud andere dirty checkouts/branches.

## Eerder handoff — PR #125-inhoudsbehoud — 2 november 2026

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
