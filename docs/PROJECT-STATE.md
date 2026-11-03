# Actuele projectstatus

## NEXT AGENT HANDOFF — PR139 actuele main geïntegreerd — 3 november 2026

Basis vóór integratie: PR-head `341d1b428d436ac0e33c5bf3d7eb684f8543109c`
en actuele `main` `d115cc96c19ac4371baed77409a23d1adddffaf0`.
Conflicten in README, progress.json, PROGRESS.md en PROJECT-STATE.md zijn
opgelost met behoud van PR139-bronrollen en PR140-statushistorie. Main voegt
de `artifact-store-core`-contracttests toe. `cargo test --locked
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

PR-head `75cda7f4d49c5afe895277ada1102d630fdb75ad` is geïntegreerd met main
`69809c2cb53dfb3bd77abe5352892810fba6e82d`. De Nederlandse beschrijvingen
in PR140's hostrapport en main-documentatie/replaytests zijn behouden.
De toenmalige umask-beperking en 16 replaytests zijn hierboven geregistreerd;
dat resultaat is historisch en geen huidige test van deze nieuwe merge.
De rubric bleef **41 → 41/100**. Geen nieuwe native/QEMU-acceptatie.

## Historische context — eerdere main/statusintegratie op cbbc9c99

Main is inmiddels `cbbc9c9941b08ea5edfbd3637d494635b4fb1df0`: Simon
bevestigde zelf dat hij134 handmatig squash-mergede (3nov10:05:36Z).
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
voordat alle talen klaar waren; geen volledige CodeQL-scan. Nieuwe-headchecks
bleven vereist. Score34, geen productacceptatiebesluit. De toenmalige vraag
om publicatie via `report_progress` is historisch, geen nieuw mandaat.

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
