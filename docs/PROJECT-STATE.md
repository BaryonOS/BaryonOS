# Actuele projectstatus

## NEXT AGENT HANDOFF — teamleiding en integrale QEMU-route — 30 oktober 2026

Bronbasis `0ade8014c8179241dcfc68ea0ba8083664691240` (ook remote `main`
bij controle), schone start op branch
`simonvanieperen-symmetrical-barnacle`. Opdracht: toets en coördineer de
twaalf door Simon aangewezen sessies tegen de volledige roadmap en
FULL-TEST-RELEASE, richting een hoogwaardige daadwerkelijk testbare
QEMU-versie. De actuele taak-/fileverdeling en brongebonden rapportages staan
bovenaan [WORK-QUEUE.md](WORK-QUEUE.md), niet in een tweede roadmap.
Alle twaalf sessies zijn benaderd en hebben hun stand teruggeleverd; NAS is
uit een scopevraag vrijgemaakt voor afgestemde native integratie. De twee
PR-oplossers hebben verschillende verantwoordelijkheden. Nieuw gemelde
PR-intake van Simon wordt naar beide reviewers geleid; één mergecoördinator.
Manager bezit centrale status/queue/assessment; release finale productintegratie,
buildteam eerst zijn lopende technische #66-overdracht.

Actuele integratiebasis is `94c8908ee363d06a5be51417c5cae635549d566f`
(#66, na #62/main748). De managermerge had vijf uitsluitend centrale
documentatieconflicten;
de teamstand staat bovenaan, alle inkomende preflight-/reviewcheckpoints
blijven hieronder historisch behouden. Geen handmatige runtimebronwijziging.
Assessmentbron is die nieuwe mainbasis; eigen vorige checkpoint is
`be2145728afc5427092448953bb4ec2dbd39053d`. Geen historisch native bewijs
herlabeld. #66 is volgens live GitHubcontrole merged op 30 oktober
17:24:22 UTC; lifecycle rapporteert 13/13 checks groen en volledige
boomgelijkheid c096/main94. Postmerge-CI op main94 is nog een aparte gate.
Latere lifecyclecontrole meldt nu ook alle tien actuele postmergechecks
op main94 SUCCESS, inclusief IPC/TPM/native-system, Windows-packagecontrole
en qemu-smoke. Manager heeft deze specifieke checkruns niet zelf herhaald.
Volgende mergeprioriteit is #70 `68f4dd99`, CI nog lopend; geen
voortdurende nieuwe bronfreeze wegens alleen een centrale docsmerge.
Volledige `git diff --cached --check` tegen de vorige manager-HEAD gaf
exit 2 door zes al in main94 aanwezige afsluitende lege regels in
hashgebonden `full-release-native-wait-b02{,-net,-tcp}/host/*.stdout.txt`.
Die bewijsblobs blijven ongewijzigd; de managerdelta wordt daarom daarnaast
expliciet tegen main94 gecontroleerd, niet als volledig schone oude-base-diff
gepresenteerd.

**Score: 34% → 34%.** Geen criterium of gewicht gewijzigd, geen volledige
release- of fysieke/menselijke gate gesloten. Alle 62 productrijen blijven
open. De manager heeft roadmap, bindende matrix/context/privacy, rubric,
live sessiestatus en open issues/PR-heads/checks getoetst. Activiteit was
`unknown`; `is_running`, berichtacceptatie en eigenaarrapportage zijn niet
hetzelfde als een door de manager waargenomen draaiende build. Concrete
deelbewijzen en nog niet uitgevoerde native gates zijn in de queue gescheiden.

**Beschikbare deelbundel, door buildteam/Windowsverifier gerapporteerd:**
`C:\Users\simon\Downloads\NieuwOS-testversie-eabc27b-20261030\NieuwOS-Windows-Interactive-1.zip`,
2.296.965 bytes, SHA-256
`b42be826377afb58f96c8178aa972128abad18ef7f9b4872e102a9e4a7e9ea57`.
Start vanuit PowerShell:
`& "C:\Users\simon\Downloads\NieuwOS-testversie-eabc27b-20261030\NieuwOS\Start-NieuwOS.ps1"`.
Dit is de oude geteste `eabc27b`/`c532ba88`-deelversie, niet de nieuwe
reviewfixbron en niet de volledige testrelease. Eigenaren rapporteren echte
Windows-QEMU 11.1/SDL-invoer, zes Linux-bytegelijke frames en een diskoracle.
Bijbehorende bronbewijzen:
`docs/verification/full-release-interactive-fixed/` op #66 en
`docs/verification/windows-qemu-interactive-1/` op `3f3f99e`.
De manager heeft de brongebonden Linux- en Windowsbewijs-README's rechtstreeks
uit Git op `c532ba88` en `3f3f99e` gelezen. Bij de vervolgronde is de exacte
ZIP in Downloads vervolgens zelf read-only gecontroleerd: grootte/SHA-256
kloppen, ZIP-CRC slaagt, alle twaalf uitgepakte bestanden zijn bytegelijk,
manifestbron/kernel/loader passen bij bovenstaande oude bundel, `state/`
ontbreekt en de exacte QEMU-executable bestaat. Beide Python-inspecties
eindigden met exit 0. Compact resultaat:
`verification/teamleader-delivery-20261030.json`. Geen PowerShell-/VM-start,
geen gewijzigde Windowsbestanden of uitvoeringspolicy, geen nieuwe bronclaim.
Fixtureopslag bevat plaintexttestdata, geen persoonlijke data invoeren.

**Nieuw native deelresultaat:** beide normale gates na `d104fd9` zijn nu
groen op codebron `74d15ac09ac61c8d47a882c3c982711af1a9d891`, gepusht
bewijscheckpoint `3d2e1aba218feb39e02a1a5d257c5eb63f50e26b`:
referentie 139,078 s/raw 33, gewone zevenprocesvariant 94,889 s/raw 0,
zes bytegelijke frames en onafhankelijke post-exit-diskcontrole.
Manager las `docs/verification/full-release-interactive-review/README.md`
rechtstreeks uit Git. Geen nieuwe Windowsgast/ZIP, gewone cold-recovery-,
VM-FR- of volledige releaseacceptatie. De aparte verifier heeft twee eigen
builds (exit 0), maar stopte op verschillende kernel-/loader-/app-ELF-hashes
ondanks dezelfde bronvingerafdruk. Referentie-VM gestopt, gewone VM niet
gestart; onafhankelijke native acceptatie dus niet geslaagd. Manager heeft
een begrensde read-only toolchain/flags/ELF-sectie-/padtrace toegewezen;
oorzaak nog onbekend, geen tests versoepelen. Tracecheckpoint `79b956e`
meldt vier apps met gelijke geladen segmenten maar Materiaal met afwijkende
geladen instructies, key/volume met absolute bronpaden en geladen verschillen,
loader met timestamp/debug-ID-verschillen. Volledige historische rustc-env/
metadata ontbreekt; geen semantische gelijkheid of complete oorzaak bewezen.
Bouwreproduceerbaarheid blijft open. Afzonderlijke eigen fresh-source-runtime
is toegestaan met nieuwe eigen artifactbinding en ongewijzigde orakels,
zonder het mislukte byte-reproduceerbaarheidsverdict te overschrijven.

**Kritieke open stap:** buildteam bouwt de nieuwe cold-tools en heeft tevens
main `748e29b` met inhoudsbehoud opgenomen: gemelde codebron `d9f8c216`,
nieuwe v3 `c20a3300…`, referentie 135,971 s/raw 33 en gewone variant
97,227 s/raw 0. Lifecyclecoördinator heeft de summaries gehasht en koppelt
deze aan actuele #66-head `c0962e0`; manager heeft die head en lopende CI
zelf via GitHub gelezen, maar geen nieuwe VM uitgevoerd.
Release bezit eventuele runtime-/manifestfixes en neemt de bevroren bron
inhoudsbehoudend over. Geïntegreerde #66-head is `c0962e0`; `c532ba88` is uitsluitend de oudere
bron. De eerdere managercontrole met 12/13 groen is ingehaald door de
volledige premergecontrole en squashmerge; dit sluit geen postmergegate.
Lifecyclecoördinator meldt #62 regulier merged naar `748e29b`;
nieuwe main-opname en gezamenlijke gates zijn een herhaalplicht, geen
automatisch groen uit de vorige bron. UI herhaalt de keyboard/save/reopenketen;
apps, NAS en hardware wachten voor shared files op een concrete writerafspraak.
UI meldt inmiddels een keyboard-QMP-pass op `74d15ac`/v3 `24d5f4c1…`;
de semantiekcode `f6ec180` zit expliciet niet in dat image. Dit is uitsluitend
baselinebediening, geen native semantiek- of schermlezeracceptatie. Release
heeft opdracht de leafcode inhoudsbehoudend te integreren en daarna één
nieuwe bron/artifactbinding voor de UI-herhaling vast te leggen.
De read-only ABI-reviewer rapporteert op main `748e29b`: BootInfo-prefix
128 bytes, `reserved0` op headeroffset 120, GOP op 128..192. Bestaande
minor-1/2-validators weigeren minor 3; alleen de loader bumpen is fout.
Het snapshotbesluit bepaalt nog geen nieuwe offset. Release moet dispatch
voor 1.1/1.2/1.3, behoud van GOP, snapshotownership/retained mapping en
negatieve tests vastleggen vóór code. Rapport rechtstreeks aan release,
geen nieuwe tests door reviewer of manager uitgevoerd.
Nieuwste cold-toolsmijlpaal door buildteam gemeld op eigen bron
`420fac5783fdf86ed6d6464a8f1498190fdf78ef`, v3
`f6d4ca781ac058b7c6eba1d7b1bf49314e07e45828158e28a0afde07b74b2304`:
reference raw33/134,483s, gewone save/list/reopen raw0/93,21s en
koude child raw0/38,36s groen, volledige lijst/heropendbeelden en
zelfstandig onveranderd diskoracle. Archivering en tamper-negatieven volgen.
Dit vervangt uitsluitend de eerdere open gewone cold-toolsgate op die
bron; host-QMP-quit na save is geen guest clean shutdown, abrupte powerloss
tijdens schrijven, Windowscold of volledige FR-acceptatie. Release ontving
de exacte bron om na integratie opnieuw te bevriezen/testen.
Native DHCP/TLS, veilige persistente keys/rollbackanker, appinstallatie,
NAS/multidevice en lokale inference ontbreken nog als volledig geïntegreerde
productfuncties. AI moet na begrensd qsim-WIP naar ontbrekend L01; een al
geaccepteerd quantumcriterium verder harden vervangt dat niet. K06/SMP en
S09/import/export/back-up hebben nog geen afzonderlijke actuele builder.
Fysieke tests en menselijke/audit-/duur-/reproductiegates blijven open,
zonder de VM-implementatie te blokkeren.

**Uitvoering op deze managerbranch:** alleen documentatie/assessment;
geen kernel-, SDK-, driver-, app- of toolcodewijziging. `python3 -B
tools/progress.py --write`, `python3 -B tools/progress.py --check` en
`git diff --check` zijn uitgevoerd, alle exit 0; de SVG bleef bytegelijk
omdat score/datum niet veranderden. Volledig lokaal commandolog:
`/home/simon/.copilot/session-state/3c9dee7ff8e342deb2a0374357d86212/files/teamleader-validation.log`.
Ook de initiële rubriccheck en het lezen van beide oudere brongebonden
bewijsnotities via `git show` slaagden (exit 0). Daarna zijn ook
`git show 3d2e1aba218feb39e02a1a5d257c5eb63f50e26b:docs/verification/full-release-interactive-review/README.md`
en
`git show cad3b55cab20fd3c1f79985c70ac2f91055f89e9:docs/verification/B03-KEY-POLICY-1.md`
uitgevoerd (exit 0); dit leest rapporten, het herhaalt geen VM.
De definitieve 18-stage-updatebeleidsproef hoort bij `fecc4b9`, niet bij
een nieuwe main-/imageclaim. Live `gh pr list`, `gh issue list` en
`git ls-remote origin refs/heads/main` zijn uitgevoerd (exit 0), met de
head-/checkafbakening bovenaan WORK-QUEUE; deze lezen metadata, geen gastuitvoering.
Latere directe controle: `GH_PAGER='' gh pr view 62 --json
number,state,mergedAt,mergeCommit,headRefOid,url` en
`git ls-remote origin refs/heads/main refs/heads/simon/native-interactive-cold`
geven exit 0: #62 merged, main `748e29b7455ca4213a6bf3ba3808c3097458263d`,
buildcheckpoint `3d2e1ab` nog remote. Nieuwe kernelbuilds/native
gates/fysieke/menselijke tests: **niet uitgevoerd**
door de manager. Managercheckpoint `f404792a52072f33a1f0dac58af46a7eb20cd3c9`
is gecommit/gepusht; `git ls-remote` gaf exact dezelfde remote branchhash,
`git status --short` was leeg (alle commando's exit 0). Opvolgende
mijlpaalactualisaties blijven op dezelfde managerbranch, geen amendment.
Volgende managercontrole na de hernieuwde gebruikersopdracht: alle twaalf
oorspronkelijke sessies meldden `is_running=true`, geen input-/planblokkade;
activiteit blijft niet observeerbaar (`unknown`). Dit is geen bewijs dat
iedere sessie op dat moment een build uitvoerde. Geen continue of geplande
achtergrondbewaking ingesteld.
Latere live steekproef: 11 van 14 sessies running, geen input-/planblokkade;
buildteam en read-only reviewer niet-running. Buildteam is opnieuw gericht
op zijn aparte cold-tools, reviewer heeft de begrensde H05-layouttrace
opgeleverd. Geen claim dat alle sessies onafgebroken draaien. NAS kreeg
vrijgave zijn eigen nieuwe workflow uit te breiden van twee naar drie
werkelijk aanwezige crates, inclusief de native-probe; echte QEMU en
actuele CI op de nog te leveren nieuwe head blijven open.
Nieuwe NAS-eigenaarsmijlpaal: `ba557473f5d6cce0ee944e6f19345b4056b2f157`
gepusht, alle drie crates in afzonderlijke `nas-core.yml`-matrixjobs;
gerichte host-/target-/64KiB-stack-/faultgates groen gemeld, GitHub-CI nog
pending en geen NAS-QEMU. Security meldt NV-bewijscheckpoint
`0041a741a74abd74ce54519d55c501b82fbc305a`, getest `9ae314d`/c532:
twee increments, volledige QEMU+swtpm-stop, counter2 na coldread,
wrongauth/missing en normale/absente proeven geslaagd. Geen lineage-,
write-ahead-, powercut- of diskrollbackbewijs; geen main94-herlabeling.
Netwerk meldt listen-host/targetcheckpoint `91cda779`, maar actieve
native TCP-regressie stopt vóór de probe; dezelfde bootfout op baseline0ade.
Geen native listen/DHCP/TLS-acceptatie, release ontving de rootbootblocker.
De S09-bronzoeker vond alleen historische claim `06d46151`, niet de code
of capture in de geraadpleegde refs. Dat is geen bewijs van totale
afwezigheid. Herbruikbare S06-clienttests zijn gevonden; TL-INTAKE bouwt
nu uitsluitend `services/volume/tests/independent_copy.rs` met twee aparte
SimDisk-instanties, bronverlies/coldrecovery en foutgevallen. Hosttest is
geen productbackupportaal of native onafhankelijke herstelacceptatie.
Die builderstap is inmiddels gepusht als
`8a14770d6a719c3a122c54cb774db58ce9b1a728` op main94, uitsluitend de
nieuwe testleaf. Drie nieuwe tests en de 40 volume-core-hosttests,
historytest, Clippy en fmt groen gemeld; manager heeft de volledige
181-regelstest uit Git gelezen, geen test heruitgevoerd. De test bewaart
8365 bytes via bestaande Client-read/write in een andere SimDisk, laat
bron/clients/buffer vallen en herstelt uitsluitend de duurzame doeldisk;
stale/quota/full laten het bestaande doel intact. Geen source-/format-/ABI-
wijziging en geen punten. Lifecycle krijgt een aparte code-integratieroute;
deze centrale managermerge neemt die test niet mee.
De afzonderlijke concept-PR is #76, head `8a14770`; CI loopt.

**Nieuwste gedeelde contract-/CI-stap:** release heeft voorstel `43b5670`
gepusht, alleen `ARTIFACT-STORE-CONTRACT.md` plus ADR. Manager las het
volledige contract, geen implementatie-/nativegate. AI heeft directe
store-assigned ArtifactRef in plaats van u64-ID bevestigd en past positieve
shortreads aan; ModelPin blijft aparte provenance. Root Message320-payload
is volgens `sdk/runtime/src/message.rs` 256 bytes; 224 is geen algemene
ABI-limiet maar eventuele leafdata na framing. Capacity-accountingscope en
nul-lengthartifactbeleid zijn aan release als expliciete codekeuzes gegeven.
Pure contractvalidators bewijzen geen kernel-authenticated afzender.
Release bevestigt inmiddels de correcties: datum30okt, zelfde namespace/
moment voor capacity/quota, nul-length Invalid; Apps/AI tekenen de gedeelde
descriptor af. De concrete slice is exclusief `crates/artifact-store-core/**`
plus root-workspacemember, geen backend/authority. Eerste hosttest faalt
exit101 wegens ontbrekende cc; de bestaande UI-hosttoolroute is aangevraagd,
geen routineherinstallatie. Nieuwe gerichte
`.github/workflows/artifact-store-quality.yml` is aan release vrijgegeven
na melding aan build; bestaande root-CI blijft buiten deze writeclaim.

Apps heeft nieuwe #73-head `97c26c88b331217a3e123a3725e5be87c3e0e4ff`
gepusht. Manager las de workflow en beide manifests: bestaande
app-package-core als rootmember en echte standalone app-management worden
allebei expliciet getest/geclippyd/geformatteerd/freestanding gebouwd.
Daarmee is de bronmatige selectorgap aangepakt, niet de uitvoering bewezen:
`gh pr view 73` geeft lege checks en `gh run list --commit 97c26c...`
geen runs, beide commando's exit0. Owner onderzoekt de startoorzaak.
De commit bevat bovendien zeven bestanden/2280 regels met nieuwe protocol-/
storecode; oude c7-review is geen review van deze delta.

Security meldt op aparte #72-integratie `4de7e56` een werkelijk gefaalde
`--locked`-build van standalone ipc-wait-probe door de nieuwe SDK-zeroize-
dependency. Gerichte consumerlock-updates liggen bij security, geen brede
dependency-update of versoepeling van locked. Build is geïnformeerd om
dubbele lockwriters te voorkomen. Native successen op 4de blijven eigen
deelgates; nieuwe bron vereist de relevante system/B01/WAIT/RANDOM-gates.
Geen nieuwe manager-VM, criteriumacceptatie of scorewijziging.
Buildteam levert de cold-tools inmiddels als concept-PR #77,
head `bef94615244f639a4df3934f54565551a6f0dc37`, main94 opgenomen,
native uitvoeringsbron420fac onveranderd. Offline ketenreplay en
8 parenttamper-/2 scope-stopnegatieven groen gemeld, 66 originele files
hashgelijk; eigen volledige commands/logs onder native-cold-acceptance.
CI nog niet als groen vastgesteld; eerder genoemde cleanstop/powerloss/
Windows-/reproduceerbaarheidsgrenzen blijven ongewijzigd.
Manager heeft nu zelf exact #77-head `bef9461` in een aparte detached
worktree getest: `python3 -B -m unittest tools.tests.test_materiaal_cold`,
14 tests, exit0, 2,776s (wall2,87s). Tijdelijke schone checkout daarna
gericht verwijderd, geen VM of volledige originele artifactreplay.
Compact resultaat: `verification/teamleader-cold77-host.json`, lokaal log
`files/teamleader-cold77-tests.log`.
De afzonderlijke reviewer meldt twee MEDIUM-gaten: post-exit-PANIC/FAIL
wordt niet finaal gevalideerd en parent-replay hasht immutable firmwarebytes
niet opnieuw. De bestaande groene tests dekken dit niet af. Build ontving
minimale fix-/regressieopdracht; #77 blijft geblokkeerd tot gerichte
fix/review/gates. Geen aantoonbare fout in de opgeslagen native run geclaimd.
Directe prioritymelding naar lifecycle faalde expliciet als NOTSENT; een
bewust na huidig mergewerk gequeueëde melding is wel aangenomen, nog geen
claim dat lifecycle die verwerkt heeft.
Daarna zijn beide reviewgaten volgens de bouwer eerst rood gereproduceerd
(3 tests/6 failures), minimaal gerepareerd en door dezelfde reviewer
bronmatig gesloten op `7d66dc2744604d8119b8366a283dc10ea1a76000`.
Manager draaide zelf de bijgewerkte suite op die exacte bron in een
nieuwe aparte checkout: 16 tests, exit0, 3,772s/wall3,87s; schone checkout
gericht verwijderd, log `files/teamleader-cold77-fix-tests.log`.
Late PANIC/FAIL en vier immutable firmwaretampergevallen zijn nu gedekt,
mutable VARS blijft toegestaan. Geen nieuwe manager-VM. De nieuwe v3
`949ce3ab…` vereist eigen nativeherhaling; build kreeg opdracht
reference/parent/cold en replay op die bron te herhalen met geldige cache.
Oude420fac-rapporten blijven ongewijzigd, geen mergevrijgave op alleen tests.

Nieuwe #73-diagnostiek via eigen `gh api .../pulls/73`, exit0:
head `9d8a1196f3ee6eba2fefa14fe8cf22c568dfdbfe`, `mergeable=false`,
`mergeable_state=dirty`, `merge_commit_sha=null`. Daarmee is een concreet
mergeconflict vastgesteld, eerdere UNKNOWN-waarneming ingehaald. Afwezigheid
van de nieuwe workflow op main is niet als algemene CI-startvoorwaarde
aangenomen. Apps ontvangt na #74 één actuele basis voor inhoudsbehoudende
resolutie en echte CI; geen ready-toggle of gatebypass. Owner rapporteert
8 packagecore-/10 app-management-hosttests, Clippy/fmt/none groen op eigen
bron, nog geen GitHub-runs. Nieuwe code vereist eigen review.
Voorbereidende `git merge-tree --write-tree --name-only` op ca45/9d8a119
bevestigt exit1 met vijf conflictpaden: Cargo.toml, README, DECISIONS,
PROJECT-STATE en progress.json. Geen branch-/werkboommutatie; log
`files/teamleader-pr73-merge-tree.log`. Apps krijgt na lifecyclebasis
begrensde resolutie, behoud van beide ADR-/statushistoriën en echte CI.

Artifact-store-code is nu werkelijk opgeleverd: implementatie `0816a53`,
bewijs `7ab88e1`, ca45-integratie `53c2f14` op releasebranch gemeld.
Owner rapporteert zeven hosttests plus Clippy/fmt/none exit0 op0816,
hergates/remotecontrole53c nog open bij overdracht.
Manager las op53c de volledige 175-regelige crate, 214-regelige tests
en workflow, geen concrete bevinding in deze beperkte read-only delta;
geen eigen Rustuitvoering (`cargo`/`cc` niet op PATH, command-v exit1).
Nieuwe gerichte CI is bronmatig aanwezig, GitHubresultaat nog niet
vastgesteld. Geen backend/authority/nativeinstall of nieuwe punten.
Release gaat na dit checkpoint het concrete NAS-probeprofiel integreren.

UI heeft consumerwerk op aparte `codex/native-semantic-consumer-20261030`,
niet op bevroren #68 gepusht; exclusief controller.rs, interaction.rs en
directe NATIVE-MATERIAAL-consumersectie, met release afgestemd.
Owner meldt21 interactietests/fmt/check/Clippy groen; generation-bound
dispatch wordt nog aangescherpt. #68-head e382a989 is volgens owner een
legitieme main94-integratie van eerdere4a4, niet nieuwe consumeracceptatie.
Nieuwe code blijft host-only tot integratie en nativeherhaling.
Eerstvolgend bij hervatten: `git branch --show-current && git rev-parse HEAD
&& git status --short`, daarna verse `get_sessions_status` en actuele
buildteam/release-rapportages lezen; geen oude groene PR-head overnemen.

## Historisch branchcheckpoint PR #65 — main748-integratie op 0e441a9

Bron `0e441a9354fd519ebb5583eb52d2a396f452bbf6`, historische
assessmentbron `386ae6014e00e093346987859659914f86fce888`,
branch `simon/windows-qemu-test-version`. De eerdere Windows-handoff en
preview-/launcherproeven staan in de hieronder geïntegreerde historie.
Onderstaande aanvullende branchregistratie is behouden op verzoek van de
lifecyclecoördinator; geen nieuwe manageruitvoering of huidige main94-gate.
De te vroege sluiting van #65 is door lifecycle gecorrigeerd/heropend;
opnieuw sluiten vereist diens actuele inhoudsbehoudcontrole op main.

**Conflictintegratie PR #65:** main `0ade8014c8179241dcfc68ea0ba8083664691240`
is eerder opgenomen; deze ronde is bijgewerkt tot main
`748e29b7455ca4213a6bf3ba3808c3097458263d`. De interactieve proof-commit
`0887217360e924f94f3b86a52c3724cfdbe1c9ea` blijft behouden. De overlappende
README-/status-/rubricdelen behouden de Windows-QEMU-assessment; de PR #62-
checkpointen van main staan hieronder. Op de eerste geïntegreerde werkboom slagen
`python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` (111 tests),
`python3 -B -m unittest discover -s tests -p 'test_system_*.py'` (46 tests)
en `python3 -B tools/progress.py --check`; op de nieuwste proof-code slagen
`python3 -B -m unittest tools.tests.test_materiaal_preview` (22 tests).
Voor de nieuwe basewijzigingen slagen `python3 -B -m unittest discover -s tests
-p 'test_preflight.py'` (24 tests), `cargo +1.90.0 test --offline --locked
--workspace` (365 tests in 31 groepen, waaronder HKDF 3/3 en HMAC 2/2 doctests),
`cargo +1.90.0 fmt --all -- --check` en
`cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all -- --check`.
De eerste workspacepoging miste offline `ed25519-dalek`; `cargo +1.90.0 fetch
--locked` herstelde de gecachete lockfile-dependencies. De pinned rustfmt-
component ontbrak aanvankelijk; projecttoolchaincomponent is geïnstalleerd en
beide formatchecks slagen daarna. Twee rustfmt-regelafbrekingen in de al
gewijzigde `display.rs`/`ps2.rs` zijn exact conform de formatter bijgewerkt.
De interactieve native-reference-run, Windows-gaststart, ZIP en CI op de
uiteindelijke mergehead zijn niet uitgevoerd. Op mergecommit `386ae60` vond
CodeQL geen alerts in Actions of Python, maar de analyse stopte door timeout
voordat alle talen gereed waren; er is dus geen volledige CodeQL-goedkeuring.
De checker adviseerde geen herhaling. De vorige integratiecheck op
`0ade801` meldde een extra lege regel in
`verification/full-release-b02-keystate-system/host/1.stdout.txt`; dat
hashgebonden bewijs is ongewijzigd behouden. De huidige
`git diff --cached --check` slaagt.

Historische assessment, datum `2026-10-30`, vorige score 34, taak
`WINDOWS-QEMU-TEST-VERSION / INTERACTIVE-PROOF-1 / CONFLICT-MERGE-MAIN-748E29`:

> 34% blijft 34%. De Windows-QEMU/interactive-proof-branch en main-head 748e29b zijn geïntegreerd met behoud van eerdere checkpoints. De preflighttests (24), Rust-workspace-tests inclusief HKDF/HMAC-doctests (365), root-/SDK-rustfmt en progress-check slagen op de geïntegreerde bron. Eerdere 111 tools- en 46 systeemharnastests plus 22 previewtests zijn brongebonden aan de eerdere integratie-/proof-bronnen. Geen native gate, CI op de uiteindelijke mergehead, Windows-gaststart of interactieve referentierun uitgevoerd. Geen rubriccriterium gesloten.

## Historisch checkpoint uit #66 — Zes reviewfixes en Windowsbewijs geïntegreerd; native herhaling volgt

Vervolgbranch vanaf71c1d5c met d104fd9a7f33b478c3e1b43f8d99c6d65af73fd6
inhoudsbehoudend geïntegreerd. Alle zes concrete reviewfixes samen; volledige
Windowsbewijsmap blijft blob-identiek aan3f3f99e, bronoudeeabc/c532.
Code v3 moet24d5f4c112924cfe45681de3fcf3a92f62087e29524f15c74e14d83620891abd
blijven; geen experimentele coldhelper in tools opgenomen.
Gerichte55/volledige138 tools-tests exit0, diff/progresswrite/check0.
Logs `/tmp/nieuw-os-review-integrated-{targeted,tools}.log`.
Daarna checkpoint
push naar eigen vervolgbranch (PR66blijftc532) en echte normale referentie+
gewone bedieningsproef op deze exacte codebasis. Coördinator krijgt exact
broncommit/gates vóór mutatie van PR66. Score34;50% open.

Derde koude experiment8096f0533e slaagt: vooraf renderer-afgeleid volledig
lijstframe (Readyfooter, populatedlist), daarna exact reopenedframe, geen
maskers of vaste gereedheidsdelay; disk onveranderd+diskoracle naquit0.
Payload oudeeabc vastgehashd, hosttoolbron0e17… apart geregistreerd.
Dit blijft sessieonderzoek, geen complete same-source acceptatierunner.

## Uitvoeringshistorie — Windows-testversie afgeleverd; previewreviewfix apart

Vervolgbranch `simon/native-interactive-cold`: previewfix e6eecbc en volledige
Windowsbewijscommit 3f3f99e inhoudsbehoudend geïntegreerd (identieke Git-blobs)
als e280311. PR #66-head blijft voorlopig c532ba8 voor coördinatorintegratie.
Windowsbewijs: `verification/windows-qemu-interactive-1/README.md`.
Werkelijk Windows-QEMU11.1/SDL/pc-q35-8.2/rng-builtin, 102.445s/raw0,
zes volledige bytegelijke frames en zelfstandig diskorakel na QMP-quit.
918 QMP-commando's opnieuw gecorreleerd door peer; geen menselijke audit.
Bron blijft eabc27b/c532, v3a1998a12, NIET nieuwe previewfixbron.

Lokale schone ZIP:
`C:\Users\simon\Downloads\NieuwOS-testversie-eabc27b-20261030\NieuwOS-Windows-Interactive-1.zip`,
2296965 bytes, SHA256
b42be826377afb58f96c8178aa972128abad18ef7f9b4872e102a9e4a7e9ea57.
Start: uitgepakt `NieuwOS\Start-NieuwOS.ps1` in dezelfde map, eigen state
wordt pas door launcher gemaakt. Alleen synthetische ontwikkeldata; geen
persoonlijke gegevens in deze plaintextfixture bewaren. Geen publicatie.
De gast heeft expliciet read-only toegang tot de gebundelde ESP-directory
via QEMU fat:esp en eigen statebacking; "geen hostmap" in het historische
peerbewijs betekent geen algemene hostfileshare, niet afwezigheid van ESP.
Coördinator herstelt die formulering in actuele packagingdocs.

Preview stale-reportfix is lokaal52/135hostgroen, nog geen nieuwe native
acceptatie op die gewijzigde v3. Coördinator doet vijf andere concrete
packagingreviewpunten in eigen branch; geen gelijktijdige previewwriter.
Koude experimenten blijven apart, geen volledige acceptatie. Volgende stap:
reviewfixes op één vaste bron integreren en gerichte+native sourcegates;
daarna koude normale variant met volledig renderer-afgeleid lijstorakel.
**34% blijft34%; minstens50%, volledige product- en menselijke/hardwaregates open.**
## Uitvoeringshistorie — Windowsbundelreview gecombineerd; integratiegates open

Basis `e6eecbc28bb4e74d1b1c7c01b26f2ab249cf4291`, eigen branch
`copilot/windows-review-fixes-20261030`. De peer-previewfix is inhoudsbehoudend
fast-forward geïntegreerd; vijf overige concrete PR #65-reviewpunten zijn
hersteld in packager/directe tests, Windowslauncher/test/README en het
historische preview-README. Geen kernel-/capabilitywijziging. Uitvoerpad
normaliseert `..` vóór mkdir en weigert symlinktraversal vóór en na mkdir;
de launcher geeft een vaste fout voor een ontbrekend bundelbestand.
Historische JSON-bewijzen blijven bytegelijk; geen ongefundeerde v3-,
hostisolatie- of algemene Windowsacceptatieclaim.

Uitgevoerd: 55 gecombineerde gerichte Python-tests, exit 0; vijf
synthetische launchertests op echte Windows PowerShell 5.1/QEMU 11.1,
exit 0, zonder VM of policywijziging. Eerste Pythonrun faalde door een
inspringfout in de nieuwe test; na herstel slagen alle drie nieuwe
padregressies en de bestaande tests. Exacte commando's, logs en grenzen:
`verification/native-test-version/windows-bundle-review.md`.
Voortgang **34% → 34%**; nieuwe native/CI-gates op deze gecombineerde
bron zijn niet uitgevoerd. Linux-/Windows-interactiebewijs op eabc27b/c532
wordt niet naar deze bron omgelabeld.

GitHubtriage: issues #37/#38/#45 gesloten met bestaand bewijs of echte
negatieve rulesetproef. #39–#43 blijven open; B03 heeft al native
updatebewijs maar #41 vraagt ook nog sleutelrotatie/intrekking. Alle twaalf
CI-checks van PR #62-head `f6d3a108` slagen; de nieuwe review loopt nog en
de reviewbeslissing blijft `CHANGES_REQUESTED`. Geen reviewbypass.
PR #66-head `c532ba8` had dertien groene checks, maar deze reviewfixes en
Windowsbewijscommit `3f3f99e` moeten eerst inhoudsbehoudend worden opgenomen.
PR #65 niet sluiten voordat zijn volledige inhoud én reviewafhandeling in
de vervangende integratie zijn aangetoond. #68/#70–#73 hebben hun eigen
actieve bron-/revieweigenaren; geen gelijktijdige bronwijziging door triage.

Eerstvolgend: checkpoint pushen en remotehash controleren; eigenaar #66
integreert daarna deze bron en het afzonderlijke Windowsbewijs, voert de
vereiste nieuwe gates uit en houdt historische bewijsbinding intact.
Squashmerge alleen op actuele head/base met groene vereiste checks en
afgehandelde inhoudelijke review. Volledige release en 50%-doel blijven open.

## Uitvoeringshistorie — Opvolgbranch: oud preview-succes bij voorbereiding geweigerd

Vanaf c532ba88fbf0f53b960e9dbf77421cee29d97096 op
`simon/native-interactive-cold`; PR #66-branch blijft exact c532ba8 voor review.
Alleen `tools/run_materiaal_preview.py` en directe tests veranderen runtime-
tooling: eigen rapportpad via private_path valideren en oud rapport verwijderen
vóór build-/firmwarevalidatie. 52 gerichte/135 tools-tests exit0; bewijs
`verification/native-test-version/preview-stale-report.md`.
Geen nieuw native bewijs op deze v3; oude bewijzen niet omgelabeld.
Coördinator verbose-sniffle doet overige vijf packagingreviewpunten in eigen
opvolgcommit; beide previewbestanden expliciet bij deze integrator.

Koude experimenten blijven voorlopig sessieartifacts, geen acceptatierunner:
eerste lijstvergelijking rood door echte Ready/Saved-statusverschil.
Tweede experiment volledig reopened-frame bytegelijk en disk onveranderd,
maar initiële lijst niet gecertificeerd en vaste wachttijd gebruikt.
Derde experiment moet een vooraf renderer-afgeleid volledig lijstframe
zonder maskers toetsen; daarna strikt parent/artifact/trace-replay ontwerpen.
Status/resultaat onder sessiefiles/cold-interactive-result.json.
Windowspeer archiveert eigen echte zesfaseproef op vaste oude c532-kopie.
**34% blijft34%; nieuwe gecombineerde sourcegates/50% open.**

## Uitvoeringshistorie — Vaste interactieve bron groen; Windowsproef volgt

Vrij te geven testbron `eabc27ba879fd3b5960863ae8eede800c5045a57`, v3
`a1998a126a3a1ccc18523925a422912d934c14bab865b3d7df346862a2c251a6`.
Na rustfmt beide echte gates opnieuw groen: referentie73611f4dbf
133.320s/raw33; gewone zevenprocessenvariantc6f2a045aa
92.390s/QMP-quit/raw0. Zes volledige frames exact bytegelijk en onafhankelijke
diskcontrole na exit groen. Alle zeven appidentiteiten/platform gebonden.
Bewijs/commando's: `verification/full-release-interactive-fixed/`.
Gewone build: `build/native-test-version/interactive-current`;
kernelhash f2e9cc4aad87ad2b9fbc8b498ab146db2ab0df4a77b904eecf81c3a9beb198b3.
Geen bronwijziging nodig voor Windowspackaging; uitsluitend gewone variant,
geen controller/probe-image. Peer krijgt aparte Windows-QEMU-opdracht met
eigen artifact-/statepaden en expliciete Windowsbewijsgrens.

CI-formatfouten hersteld: host-quality/display/foundation/RTC op eabc27b
groen; meerdere andere jobs nog pending. Required-set nu host-quality
groen en qemu-smoke pending; geen complete CI/mergeclaim.
**34% blijft34%; Windows/ZIP, product-/menselijke-/hardwaregates en50% open.**
Blijf onafhankelijk OS-werk doen zonder de vrijgegeven runtimebron te muteren.

## Uitvoeringshistorie — CI-formatfix lokaal groen; nieuwe native bron vastzetten

Vanaf bewijscheckpoint `8a401b9`: `cargo +1.90.0 fmt --all` wijzigt alleen
de twee `cfg(any(...))`-attributen in `kernel/src/m2/m3/{display,ps2}.rs`.
De derde probe-hook in desktop.rs was al correct geformatteerd.
Geen gedrag/interface/rechten gewijzigd. Vier concrete CI-fouten waren
PR-veroorzaakte rustfmt-fouten, geen transient infra of genegeerde gates.
`LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
python3 -B tools/dev.py cargo +1.90.0 fmt --all -- --check`: exit0.
Zelfde omgeving met `CARGO_TARGET_DIR="$PWD/target/native-test-version/interactive-host"`
en `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace`:
exit0; log `/tmp/nieuw-os-interactive-format-workspace.log`.
`python3 -B -m unittest discover -s tools/tests`: 132 tests, exit0,
log `/tmp/nieuw-os-interactive-format-tools.log`.

Na push beide native gates opnieuw uitvoeren, met expliciete reguliere
OVMF CODE/VARS uit `.tools/nieuw-os/qemu/usr/share/OVMF`, de bestaande
buildroots en dezelfde machine/VGA-ROM. Formatteren wijzigt bronhash v3;
oud bewijs hieronder blijft geldig voor zijn oude bron, niet voor deze.
Geen complete CI-succesclaim. **34% blijft34%; Windows/ZIP/50% open.**

## Uitvoeringshistorie — Interactieve native keten groen; CI-formatfix volgt

Op vaste bron `fc40347b250c544ab9838cf89a62df16b32a797b`, runtime v3
`61f58e2e7c7a64e911883c64de5b2d416f7085af94123699fd206c7448ea2b5e`,
slagen referentie `224bfa2ade` (136.326s/raw33) en gewone zevenprocessenrun
`f07d8d122c` (92.774s/QMP-quit/raw0). Alle zes volledige PPM's bytegelijk,
zeven apprecords/loader/bron/platform exact gebonden. Onafhankelijke disk
na exit bevat object0/rev1/14B/generation1/clocks[1,0], canaries intact.
Bewijs en exacte commando's: `verification/full-release-interactive-native/`.
Eerste aanroep weigerde automatische firmware voor gaststart; expliciete
reguliere OVMF met dezelfde hashes slaagt, geen oracle gewijzigd.

CI is NIET groen: vier jobs op fc40347 falen door rustfmt van de drie
overgenomen cfg-hooks. Eerst formatteerbaar herstel, fmt/hostgates en
checkpoint/push, daarna dezelfde twee native proeven op nieuwe vaste bron.
Peer is geïnformeerd: nog geen bundelbron vrijgegeven. Windows/ZIP, gewone
koudrecovery, visuele/menselijke/hardwareacceptatie en50% blijven open.

## Uitvoeringshistorie — Interactieve runner geïntegreerd; native gate volgt

Integratie vanaf `8b199ad` met de volledige peerbranch
`0887217360e924f94f3b86a52c3724cfdbe1c9ea`. Packager, launcher,
eerdere raw bewijzen en opt-in strikte interactieve preview behouden.
Schrijfgebieden zijn niet meer parallel bezet: peer is na oplevering gestopt.
Huidige code vereist dezelfde nieuwe bron voor een volledige normale
achtprocessenreferentie en de gewone zevenprocessenvariant. Exacte zes
native frames en zeven appidentiteiten moeten passen, geen maskers,
F12/controller of kleurenaantal als bedieningsorakel. Na QMP-quit/raw0
volgt onafhankelijk diskorakel. Geen koudrecovery-/cleanupclaim voor de
interactieve run. Kernelwijzigingen zijn uitsluitend correcte cfg-gates
voor probe-only observatie; geen runtime-/rechtenversoepeling.

49 gerichte hosttests en de volledige toolsuite slagen op deze combinatie
(exit0; `/tmp/nieuw-os-interactive-integrated-{host,tools}.log`).
Na checkpoint/push referentie normaal bouwen
onder `build/native-test-version/interactive-reference` met targetcache
`target/native-test-version/gui-baseline`, uitvoeren met pc-q35-10.0 en
dezelfde expliciete VGA-ROM. Daarna `native-system-materiaal` bouwen onder
`build/native-test-version/interactive-current` en previewrunner starten met
`--interactive-reference-run` naar de echte geslaagde referentierun.
Nog geen native acceptatie/ZIP op deze combinatie; **34% → 34%; 50% open**.
Main-docscommit0ade801 is eveneens meegenomen; geen runtimecode gewijzigd.

## Uitvoeringshistorie — B02-netwerk normaal/koud groen; interactieve variant volgt

Gepushte basis `ec8275579fa07f70180452d325738304b2edb6ab`, runtime v3
`4c472a18940ea6cd42a0f0d53d26f8c491c5204274c4b8a6b8d6070b659a79ad`.
Op deze bron slagen nu ook vier echte netwerk-VM's: ARP/UDP/DNS normaal/koud
(37,085/37,138 s) en TCP normaal/koud (7,148/6,971 s), allemaal raw33/exit0.
Werkelijke peers/brokertellers, HTTP/EOF/refused/reset/checksumweigering/
retransmissie/guesttimeout en ongewijzigde koude disk zijn onafhankelijk
herverifieerd. Gates, exacte commando's, fouten en logs:
`verification/native-test-version/b02-network-evidence.json`; captures
`verification/full-release-native-wait-b02-{net,tcp}/`.

Rust1.91.1 ontbrak aantoonbaar; daarna ontbraken gepinde smoltcp-crates bij
offline retry. Alleen bestaande projectlokale compiler/none-target en
locked dependencies hersteld, geen manifest/lock/defaultcompiler gewijzigd.
**34% → 34%; N01/50% open.** Deze smalle clientproeven zijn geen volledige
productnetwerkdienst of dagelijks Internet. Geen extra onderhoudscode.
De peer bezit exclusief zijn previewrunner/directe tests voor de opt-in
interactieve zevenprocessenketen; geen gelijktijdige schrijver.
Volgende stap: zijn gepushte code integraal lezen/integreren, eigen native
referentierun op dezelfde nieuwe bron bouwen en vervolgens echte
interactieve edit/save/reopen met exacte app-/framebinding uitvoeren.
Daarna Windowsbundel afzonderlijk toetsen; geen probe-image omnoemen.
CI op ec82755 was bij laatste read nog gedeeltelijk pending; niet groen
geclaimd of geblokkeerd afgewacht. Eerstvolgend commando:
`git fetch origin && git log -3 --oneline origin/simon/windows-qemu-test-version`.

## Uitvoeringshistorie — B02/main native groen; TPM-CI-compatibiliteit hersteld

Bron `344f4972d6db72543dd6c04bf0f28668602a91fd`, v3
`c03070ddbbe8d233f91512a97927adc54c06da2c480966c68983cc936d5e7341`:
14 echte VM's groen: system normaal/koud, vijf B01, vier RANDOM, RTC en
GUI normaal/koud (134,657/62,177 s, volledige cleanup). Eigen B02-capture
herverifieert de werkelijke diskbytes en koude herstelrun onder
`verification/full-release-native-wait-b02/`. Exacte commando's, fouten,
bron-/rapporthashes: `verification/native-test-version/b02-integration-evidence.json`.

Native-system-CI op deze bron faalde vóór de TPM-gast: Ubuntu24.04 heeft
swtpm0.7.3 en weigert de optionele nieuwe `lock`-optie. PR-veroorzaakt
runnerprobleem, geen kernel- of transportfalen. Portable dir-only-optie
behoudt één verse private mode0700-state per eigen child en mode0600-socket.
Vijf gerichte harnastests groen, inclusief private state/childcleanup.
Beide echte TPM-VM's opnieuw raw33/exit0, runtime v3
`4c472a18940ea6cd42a0f0d53d26f8c491c5204274c4b8a6b8d6070b659a79ad`
(344f497 plus de twee runner/testbestanden). Normaal family0x322e3000,
11931 writes/release; absent nul writes/sticky quarantaine; beide volledige
CPL3-wachtacceptatie groen. Nieuwe CI met swtpm0.7.3 **nog niet uitgevoerd**.
Geen check of assertie overgeslagen. GitHub meldt geen verplichte checks;
de concrete eigen native-systemfout is desondanks hersteld.
De capture bewaart ruwe Cargo-uitvoer met een bestaande lege eindregel:
`git diff --cached --check` meldt die in `host/1.stdout.txt`. Die
hashgebonden bewijsbytes zijn niet bijgeknipt; staged bytes apart
vergeleken, overige bron/documentatiewitruimte wel gecontroleerd.

**34% → 34%; 50% blijft open.** Windows-PR #65 blijft afzonderlijk; eigenaar
bouwt nu alleen een strikte opt-in bedieningsproef voor de zevenprocessen-
variant in zijn bestaande previewrunner. Nog geen ZIP/interactieve claim.
Volgende stap: nieuwste CI-fout/status lezen zonder lopende runs af te
wachten; daarna runnercode inhoudsbehoudend integreren, normale interactieve
variant bouwen en bedienen op vastgelegde bron met exacte native
referentieframes/apphashes. De oude automatische testversie blijft apart.
Eerstvolgend commando: `GH_PAGER="" gh pr checks 66 --watch=false`.

## Uitvoeringshistorie — B02/main gecombineerd; hertoets toen open

Integratie vanaf `3f0845b83cdb5bf64386645a288b47ec09c424c5` met main
`f52dab5` (PR #64). De code is gecombineerd; onderstaande 16 VM's en
proefversie blijven uitsluitend bewijs van de oude bron. Main voegt apart
admin-ingress en persistent keystate toe. De Key Service bewaart beide
gedragingen: alleen legacy-ingress mag quiescent wachten; RANDOM en admin
behouden hun bestaande yield/poll-pad, zodat werk op die endpoints niet
achter een legacy-only wachtset blijft liggen. Geen nieuwe rechten buiten
het main-B02-manifest en geen publieke ABI-wijziging. ADR-051 blijft B02;
de branchlokale wacht-ADR is bij deze ID-botsing hernummerd tot ADR-053.

Eigen samenvoegresolutie: `sdk/examples/keyring/src/service.rs`, README,
status/assessment en DECISIONS; alle main-code, tests en brongebonden
B02-bewijs behouden. Hosthertoets op de gecombineerde werkboom groen:
`python3 -B -m unittest discover -s tests -p 'test_system_*.py'`
(46 tests), `python3 -B -m unittest discover -s tools/tests` (102 tests),
`LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" CARGO_TARGET_DIR="$PWD/target/native-test-version/wait-lifecycle-host" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --workspace`
(377 passed, 1 bestaande ignored), rootfmt en gerichte rustfmt van de
Key Service; alle exit0. Logs `/tmp/nieuw-os-b02-merge-{system-host,tools-host,workspace}.log`.
Native hertoets **nog niet uitgevoerd** op deze combinatie. Na checkpoint/push:
`tools/test_system.py`, B01, RANDOM/klok en normale/koude GUI waar passend.
Geen nieuwe punten: **34% → 34%; 50% blijft open**. Windowsbundel-PR #65
blijft apart; pas na eigen interactief bewijs bron/image vastleggen.
Eerstvolgend commando:
`python3 -B -m unittest discover -s tests -p 'test_system_*.py'`.

## Uitvoeringshistorie — 16 integratie-VM's groen; lokale proefversie gebonden

Branch `simon/native-test-version`; basis `ad2b981fe1f2bcccb03cb776b2281d32529d2f00`,
geteste niet-GUI-bron `37a74784f71889d6ea41a230d756be16c1728ae8`,
GUI-/proefversiebron `4bdc1a26171095f92ce03611b14a78b38706b062`, dezelfde v3
`a6c4c723fbcc6a5b50eac17b91b77f66161c8555c6d916fd10b7826815acc4b5`.
**34% → 34%; minstens 50% niet bereikt.** Beide echte TPM-VM's slagen
raw33/exit0: vaste supervisor-UC/NX-MMIO, echte TPM2-familyquery, bevestigde
release; zonder TPM exact nul writes en sticky quarantaine. Zeven
adaptergrensweigeringen en beide volledige CPL3-wachtproeven blijven groen.
Geen productgrant, NV-mutatie, sleutelpersistentie of rollbackanker geclaimd.
Bronnen, exacte commando's, payloads, logs en eerste rode mapperpoging:
`verification/native-test-version/tpm-evidence.json`.

Op de gecombineerde runtimebron slagen opnieuw 16 echte VM's: zes K05-cases
(inclusief twee Waiting-kills), beide FPU-profielen, system normaal/koud,
RTC, IPC-wacht, TPM normaal/absent en GUI normaal/koud. Alle rapporten
hebben dezelfde hierboven geverifieerde runtimebron. De volledige Rustworkspace slaagt nu:
372 tests, 0 failed, 1 bestaande ignored. Ook 415 Python- en 102 tooltests,
12 FPU-harnastests en root-rustfmt slagen. Geen CI-/hardwareclaim.
Native-system-CI krijgt beide echte cases; die CI **nog niet uitgevoerd**.
PR #64/keyring/system-opslag niet gewijzigd; PR #64 is inmiddels merged
maar nog niet opgenomen in deze branch. Geen achtergrondagent of
testproces actief. Eerdere GUI/koud/FPU/K05-bewijzen blijven brongebonden.
Nieuwste normale lokale automatische proefversie:
`build/native-test-version/test-version-current/esp`. Alle tien payloads
en het manifest zijn bitgelijk aan de echte geslaagde normale GUI-run.
Eerdere gedeelde hostharnassen op `f61ce3e`: 415 root-Pythontests en 102
tooltests slagen. De volledige root-Rustworkspace faalde daarna op de
reeds bekende ontbrekende HKDF-doctestdeps (367 unit/integratietests
ervoor groen). Exact de bestaande PR #62-fix voor beide vendormanifesten
is hergebruikt: alleen gepinde host-devdeps/lock, geen cryptobron,
runtime-afhankelijkheid of PR #64-wire/systeminterface gewijzigd.
Alle vijf gerichte HKDF/HMAC-doctests slagen nu. PR #62 is niet gesloten
of samengevoegd; zijn overige preflightinhoud is niet overgenomen.
De rode workspace-log blijft `/tmp/nieuw-os-native-tpm-regression-workspace.log`.
De volledige workspace en bovengenoemde native integratie zijn inmiddels
hertoetst; log `/tmp/nieuw-os-native-tpm-workspace-final.log` (27 s/exit0),
native logs `/tmp/nieuw-os-native-tpm-*-final.log`.
De nieuwe GUI-keten slaagt in 128,979 s: zes frames, edit/save/list/reopen,
14 echte diskbytes en volledige cleanup. Koude readonly-heropening in een
nieuwe VM slaagt in 61,100 s: drie frames, zichtbare writeweigering,
nul writes/flushes en onveranderde disk. Beide raw33/exit0; geen
deadlineverruiming of menselijke afwerkingsclaim. Bewijs en exacte commando's:
`verification/native-test-version/gui-tpm-integration-evidence.json`;
14 overige VM's/hostgates: `tpm-regression-evidence.json`.
Volgende stap: onafhankelijke Windowsbundel-eigenaar informeren over deze
exacte automatische variant (niet verwarren met blijvend interactieve
`native-system-materiaal`), daarna PR #64/main-integratie afzonderlijk
beoordelen en passende gewijzigde native gates herhalen. Eerstvolgend
commando: `git fetch origin && git log --oneline HEAD..origin/main`.
PR #65 bezit packaging/launcher/CI; niet tegelijk wijzigen.
Gerichte doctestlog:
`/tmp/nieuw-os-native-tpm-vendor-docs.log`.
Volledige product-/menselijke-/hardwareacceptatie en 50%-opdracht blijven open.

## Uitvoeringshistorie — TPM-CRB-MMIO gebouwd en eerste native herstel

Branch `simon/native-test-version`, basis `ad2b981fe1f2bcccb03cb776b2281d32529d2f00`;
laatste bewijscheckpoint `cf020559b7771a221713f2fd676c1dbed0081c24`.
**34% → 34%; minstens 50% niet bereikt.** Nu TPM-NATIVE-1: de bestaande
TPM-core verbinden met een vaste supervisor-RW/NX/UC-aperture, exact
begrensde volatile-I/O en echte PIT-polls. Alleen één read-only TPM2-
familyquery, bevestigde release en een aparte absent-case; geen sleutel-,
NV-, productgrant-, nieuwe ABI- of antirollbackclaim. ADR-052 en TPM-contract
leggen de grens vast. PR #64 bezit keyring/system-opslag; niet aangeraakt.

Eigen writes: root/kernel Cargo-manifest/lock, exact pagingseam,
M3-testhook/nieuwe tpm_fixture, waitfixture-observer, buildkeuze,
bestaande generieke runner plus nieuwe TPM-runner/hosttest en docs.
De adapter wordt slechts eenmaal geclaimd; quarantaine blijft sticky.
De bestaande twee CPL3-wachtapps en hun volledige orakels blijven behouden.
Scalarbuild/ELF-inspectie exit0 (`/tmp/nieuw-os-native-tpm-build.log`),
23 bestaande TPM-coretests exit0 (`/tmp/nieuw-os-native-tpm-core.log`),
4 gerichte TPM/IPC-harnastests exit0. Eerste metadata-aanroep weigerde
terecht een geneste workspace; tpm-crb nu zoals rtc expliciet uitgesloten,
alleen één bestaande lokale dependency toegevoegd aan root-lock.

De omgevingagent leverde een lokale officiële swtpm0.10.1/libtpms0.10.2-
prefix en bevestigde QMP-readiness met de juiste ctrl-Unixsocket; zijn
eerste data-socket/alive-only-proef was onvoldoende en is niet als native
bewijs gebruikt. Geen agent of testproces meer actief.
Eerste native poging op `8e4e29a` faalde raw35 vóór TPM-I/O: de nieuwe
caller herhaalde ten onrechte de bootstrap-only `map_tables`, wat bestaande
leafmappings terecht weigert (pagingregel334). De runtime-MMIO-seam gebruikt
al het bestaande scratchpad voor nieuwe tabellen, zoals keyring/m5.
Die dubbele bootstrapaanroep is verwijderd; mapperchecks zijn ongewijzigd.
Rode run `tpm/run/tpm-normal-ddf088b0dc`; hertoets nog open.
Eerstvolgend commando na checkpoint:
`LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" NOS_QEMU_MACHINE=pc-q35-8.2 NOS_BUILD_DIR="$PWD/build/native-test-version/tpm" NOS_TARGET_DIR="$PWD/target/native-test-version/tpm" python3 -B tools/dev.py python3 -B tools/run_tpm.py --case normal`
Daarna dezelfde opdracht met `--case absent`; bij falen echte serial lezen.
Eerdere GUI/koude proefversie, FPU en zes K05-cases blijven brongebonden
hieronder. Volledige product-/menselijke-/hardwareacceptatie en 50% open.

## Uitvoeringshistorie — Waiting-kill en schoon slothergebruik native groen

Branch `simon/native-test-version`, basis `ad2b981fe1f2bcccb03cb776b2281d32529d2f00`.
Geteste bron `586c33c2db2cd3c349612246f81e1470822aef3b`, v3
`5414d89744c29995ac028c7e112958404077b93e1561263469928dea027f2c29`.
**34% → 34%; minstens 50% nog niet bereikt.** Alle zes echte K05-cases
slagen raw33/exit0; inclusief twee kills van werkelijk Waiting kinderen,
vier exacte weigeringen, vers PID/hetzelfde slot, geen geërfde deadlinewake
en volledige cleanup. Vier blokkeringen, twee deadlinewakes, 84 idle-IRQ's.
De drie eerdere rode pogingen blijven bewaard. Geen gewijzigde productie-
rechten, kill-, cleanup- of schedulermechanismen. Bewijs en exacte commando:
`verification/native-test-version/wait-lifecycle-evidence.json`.

Eerder geslaagde normale/koude GUI, IPC, FPU, system en RTC houden hun
eigen bronbinding; de normale lokale QEMU-proefversie staat in
`build/native-test-version/test-version/esp`. Geen nieuwe punten, fysieke
hardware, menselijke acceptatie of volledige productrelease geclaimd.
Actuele overlapcontrole: PR #64 bezit persistente keyring/system-
opslagstappen; dat werk niet dupliceren. Geen achtergrondagent of schrijver.
Volgende stap: onafhankelijke bestaande TPM-CRB-blokkade afbakenen, zonder
PR #64 te overschrijven; eerst `git status --short` en de TPM-contracten lezen.
De brede opdracht blijft open; volledige matrix en 50%-doel niet aftekenen.

## Uitvoeringshistorie — GUI/koude proefversie en echte FPU-wacht

Branch `simon/native-test-version`; basis `ad2b981fe1f2bcccb03cb776b2281d32529d2f00`,
eerder gevalideerde code `deb7a3d5c1c72a2fc816a5900062f9ed0d554782`, integratiebewijs op
`36cbd215e03becbc4b495ae1b3a42b844603e0fa`, bron v3
`31c1f108e52336c222c1a9541dc91edd06c291c05eedbc88dec03d8df658bf9f`.
**Score 34% → 34%; gevraagde 50% niet bereikt.** Alle agents klaar; geen
achtergrondwerk of gedeelde schrijver. Werkbranch/codecheckpoints gepusht.

Native `test-fpu-wait` en de aparte legacy-FPU-baseline slagen beide raw33
op deze code. De combinatie meet 41 echte blokkeringen/deadlinewakes,
36 idle-IRQ's, 347 echte user-timerinterrupts en exact 456 saves/restores,
drie schone proceslevensduren, AVX-appfault/survivor/verse PID3, volledig
frame-/context-/Engineherstel. Vier eerdere failures blijven bewaard;
geen scheduler-, entryassembly-, rechten- of productdeadlineversoepeling.
Bewijs/commando's: `verification/native-test-version/fpu-wait-evidence.json`.

Definitieve hertoets op `36cbd21`: echte GUI-edit/save/list/reopen raw33 in
128,857 s; koude readonly-heropening raw33 in 60,679 s met bytegelijke disk,
zichtbare weigering en nul writes/flushes. Ook system normal/recover, RTC
en gewone IPC-wacht zijn op die bron opnieuw raw33 geslaagd. Alle bronnen,
commando's en hashes staan in `verification/native-test-version/integrated-evidence.json`.
Lokale normale proefversie: `build/native-test-version/test-version/esp`,
nu opnieuw gebouwd op die code; alle tien loader/kernel/userspace-payloads
zijn bitgelijk aan de geslaagde normale run. Core: 64 tests en 1 bestaande
ignored; SDK: 23 tests, beide offline/locked exit0.

Actieve kleine stap vanaf `686e847`: kill van een nog Waiting proces met de bestaande
K05-procesrechten, stale-handleweigering en vers slothergebruik. De bestaande
`supervisor::kill` loopt al via echte KILL-rechten en normale cleanup; lees
nu uitsluitend die fixture/apps/runner voor een afgebakende native proef.
Schrijfbestanden: kernel-feature/supervisor-fixture, SDK-supervisorprotocol/
worker/supervisor, buildkeuze, bestaande K05-runner/gate/hosttest,
`process-management.yml` en bewijsdocs. Geen wijziging aan kill-afdwinging,
scheduler, cleanup of publieke ABI. Eén eigenaar, geen achtergrondtaak.
De nieuwe case vereist twee echte Waiting-kills, Process-derive zonder DERIVE
en endpoint-killweigering,
stale status/kill na vers PID/slothergebruik, geen overgenomen oude wake,
lege Engine en volledig frameherstel. Acceptatie: protocol-/harnastests,
scalarbuild en alle zes echte K05-cases; score blijft zonder nieuw criterium34%.
De nieuwe scalarbuild/ELF-inspectie, 8 K05-harnastests, 1 protocolhosttest,
root-formatcheck en diffcheck slagen, exit0. De echte nieuwe lifecycle-VM
is bij dit codecheckpoint **nog niet uitgevoerd**. Buildlog:
`/tmp/nieuw-os-native-wait-lifecycle-build.log`.
De zes-casesrun op `dcb6b12` geeft vijf echte groene legacy-VM's, maar de
nieuwe case faalt raw35/step104: de proef nam ten onrechte aan dat een
Process-rootcap DERIVE bezat. Het bestaande contract geeft alleen
`OBSERVE|KILL`. De proef verwacht nu juist EINVAL op die derivepoging en
ETYPE op een endpoint-kill; beide stale-handleweigeringen blijven.
Geen rechten verruimd. Hertoets volgt; rode run:
`wait-lifecycle/run/supervisor-wait-208a0949b3`.
De bestaande ongebruikte `Incoming::Denied`-payloadwaarschuwing is niet
onderdrukt of als groene Clippy-controle voorgesteld.
Ook de hertoets op `d4b6ebc` hield de nieuwe case terecht rood op step104:
DERIVE is voor Process een ongeldige objectrechtklasse (`EINVAL`), niet
slechts een ontbrekend toegestaan recht (`EACCESS`). Dit volgt uit de
ongewijzigde `Engine::require`; de proef eist nu exact EINVAL. Alle vijf
legacy-VM's slaagden opnieuw. Rode run:
`wait-lifecycle/run/supervisor-wait-b0dfc7215f`; hertoets volgt.
Op `1414ca0` passeert de proef die exacte negatieve checks en de eerste
Waiting-kill, maar faalt vervolgens raw35/step47: de nieuwe caller sloot de
Process-handle zowel in `kill_and_confirm` als in `drop_worker_caps`.
Zij gebruikt nu de bestaande `kill_and_confirm_keep`; alleen de bestaande
eigenaar `drop_worker_caps` sluit de handle. Geen productie- of
orakelversoepeling; rode run `supervisor-wait-26d9a193e9`, hertoets volgt.
Open: overige native/productmatrix,
fysieke en menselijke acceptatie. Testfixtureplaintext is geen persoonlijke
opslag-/encryptie-/releaseclaim. Nieuwe concept-PR is niet geopend omdat de
app-sessie aan de oudere infra-branch is gebonden; geen CLI-omweg gebruikt.

## Uitvoeringshistorie — koude GUI en opbouw van de FPU-wachtproef

Branch `simon/native-test-version`, basis `ad2b981fe1f2bcccb03cb776b2281d32529d2f00`;
code `2066ae7ccb25ac9e5e23eb81ec79a7286699a8ec`, laatste bewijsbron
`de541fab03aba0ed850ba65a8cdaa549e265d521` (alleen docs gewijzigd).
Score **34% → 34%; gevraagde 50% niet bereikt**. Geen criterium versoepeld.
Alle deelagents zijn klaar; er is geen achtergrondwerk of gedeelde schrijver.

De echte normale QMP-keten slaagt in 131,561 s/raw33: zes frames,
muis/toetsen, save/list/reopen, 14 werkelijke diskbytes en volledige cleanup.
Een geheel nieuwe readonly-VM op dezelfde disk slaagt in 60,769 s/raw33:
document heropend, write zichtbaar geweigerd, nul writes/flushes/ACK's,
diskhash onveranderd, nul achtergebleven processen/handles.
Bewijs/commando's: `verification/native-test-version/{gui-normal,gui-recover}-evidence.json`
en de README aldaar. Synthetische plaintextfixture; geen release-/encryptieclaim.

Lokale normale proefversie: `build/native-test-version/test-version/esp`,
met manifest en acht ELFs onder dezelfde buildroot. Loader/kernel/acht ELFs
zijn bitgelijk aan de geslaagde normale run. De oorspronkelijke `gui-idle/esp`
bevat nu het readonly-herstelprofiel; de normale run houdt eigen bevroren inputs.
Recovery vereist parent én herstelrun in dezelfde beheerde build/run-root;
de eerdere poging met aparte root is vóór gaststart geweigerd.

De bestaande gezamenlijke normal/recover-poort en legacy-FPU-poort slagen
op `3064b1a`, alle drie VM's raw33. Logs:
`/tmp/nieuw-os-native-{system,fpu}-regression.log`.

Volgende kleine codeproef: `test-fpu-wait`, afzonderlijk van de bestaande
FPU-baseline. Schrijfgebied: kernel-feature/FPU-fixture en M3/M4-fixtureselectie, FPU-assemblyprobe,
buildselectie, bestaande FPU-runner/hosttests, native workflow en bewijsdocs.
Vervang uitsluitend in dit profiel de bestaande yield door een echte
deadlinewait; controleer de volledige x87/SSE-bank vóór/na blokkeren,
echte idle-IRQ's, timerpreëmptie, appfault, slothergebruik en cleanup.
Geen wijziging aan de productie-wacht, scheduler of publieke ABI.
Acceptatie: `tools/test_fpu.py`, normale én wait-FPU build/run en bestaande
GUI-keten na integratie; geen hosttest als native bewijs.
De implementatie en 12 gerichte hostverifier-tests zijn gereed, exit0;
ook `cargo +1.90.0 fmt --all -- --check` en `git diff --check` slagen.
Nieuwe native variant **nog niet uitgevoerd** vóór dit codecheckpoint.
De exacte legacy-regressies staan in `regression-baseline.json`.
De eerste native build op `dedeacd` is terecht geweigerd: `ipc-wait` activeert
M4, terwijl FPU nog uitsluitend M3 toeliet; daardoor werd ook de verkeerde
IPC-fixture-ELF vereist. De expliciete testcombinatie staat nu M4-mechanismen
toe maar sluit uitsluitend de generieke M4-acceptancefixture uit. Engine,
IPC-afdwinging en schedulerbody zijn niet gewijzigd. De nieuwe build en
format/diff-check slagen; runtime blijft nog open bij dit checkpoint.
Foutlog: `/tmp/nieuw-os-native-fpu-wait-attempt1.log`;
buildlog: `/tmp/nieuw-os-native-fpu-wait-build.log`.
Ook 100 tools- en 414 Python-hosttests zijn opnieuw exit0 geslaagd.
De eerste daadwerkelijke wait/FPU-run op `a263bef` faalde raw35 na de echte
AVX-fault: de nieuwe M4-compositie miste PID-registratie voor de algemene
cleanup. De fixture registreert nu PID1/2 en de verse PID3, zonder rechten
te geven, en eist na afloop een volledig lege, consistente Engine.
Geen wijziging aan process-exit of capability-afdwinging. Hertoets volgt;
de mislukte run blijft onder `fpu-wait/run/test-fpu-wait-e442556248`.
De gewone IPC-wachtpoort en native RTC-poort slagen wel op `a263bef`, raw33.
De hertoets op `61a507e` bereikte echte fault/survivor/slothergebruik, maar
faalde terecht op `TIMER_PREEMPTION`: deadline-idle alleen bewijst geen
timerinterrupt op levende user-FP-registers. De wachtvariant houdt daarom
per lus meer echte userspace-busyinstructies aan (1.000.000 versus de
ongewijzigde 50.000 in de yield-baseline). De eis van minstens twee echte
user-timerinterrupts per app blijft intact; nieuwe run volgt. Rode run:
`fpu-wait/run/test-fpu-wait-922d54455a`.
Op `a597db8` slaagt die echte userpreëmptie-eis, maar de oudere FPU-tellerrelatie
was nog onjuist voor idle: totale IRQ0's bevatten ook CPL0-idle, terwijl de
assembly daarvoor terecht geen user-FXSAVE/FXRSTOR uitvoert. Het orakel eist
nu exact totaal=user-A+user-B+idle en saves=user-A+user-B+syscalls+fault,
plus dezelfde restoregelijkheid. Hosttests weigeren ook de foutieve oude
relatie en ten onrechte toegevoegde idle-saves. Geen assembly gewijzigd.
Rode run: `fpu-wait/run/test-fpu-wait-1058905b5d`; nieuwe hertoets volgt.

Open: native kill/hergebruik en FPU met wachten, overige native regressies,
volledige product-/menselijke-/hardwareacceptatie. Eerstvolgende commando:
`NOS_BUILD_DIR="$PWD/build/native-test-version/system" NOS_TARGET_DIR="$PWD/target/native-test-version/system" python3 -B -u tools/test_system.py`
met de bestaande lokale `LD_LIBRARY_PATH`; daarnaast de workflow-FPU-poort.
Nieuwe concept-PR blijft geblokkeerd door de aan de oude infra-branch gebonden
app-sessie; de OS-werkbranch is apart gepusht. Geen voltooiingsclaim.

## Uitvoeringshistorie — integratie van de native testversie

Basis `ad2b981fe1f2bcccb03cb776b2281d32529d2f00` (main, inclusief RTC-PR #63),
branch `simon/native-test-version`. De eerdere infrastructuurtaak blijft
afzonderlijk in PR #62. Doel: OS-code en een echt gedraaide QEMU-testversie;
de gevraagde **minstens 50% is niet bereikt; score blijft 34%**.

Concrete blokkade: de bestaande Materiaal-proef overschreed op `9596abc`
de ongewijzigde 20.000-PIT-tickgrens voordat de echte documenthandeling
voltooid was. In plaats van budgetten te verruimen wordt de al aanwezige
componentbranch voor begrensde IPC-wacht geïntegreerd (`ae3f022`, `2b286d2`,
`57b6e93`). De integratie behoudt de nieuwere stackbereikvalidatie en RTC;
de wacht-ABI wordt 1.10 omdat 1.9 al de RTC-klok betekent (nu ADR-053).
Productprofielen gebruiken de nieuwe wacht nog niet.

Schrijfgebieden hoofdagent: capability-core-hostharnas, kernel-wacht/
scheduler/fixture/featureselectie, SDK-ABI, bestaande inputconsument,
`tools/build.py`, nieuwe native wachtgate en status/bewijs. Een builder
werkt uitsluitend aan `sdk/examples/ipc-wait-probe/**` voor de eigen
CPL3-proef; geen gedeelde schrijvers. Omgevingsherstel schrijft uitsluitend
genegeerde `.tools/nieuw-os/**`; strict preflight is inmiddels exit 0.

Gerichte eerste controle, alle exit 0 (Rust 1.90.0, offline/locked):
`CARGO_TARGET_DIR="$PWD/target/native-wait-checks"`, plus
`LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"`,
voor ieder volgend `python3 -B tools/dev.py cargo +1.90.0`-commando:

- `test --offline --locked -p capability-core`: 57 coretests en 7 tests
  van de echte kernel-wachtdecoder/evaluator; 1 bestaande benchmark ignored.
- `test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime`:
  23 tests; daarna is uitsluitend de aanvullende ABI-1.10-procesassertie
  toegevoegd en moet die nog opnieuw worden uitgevoerd.
- `test --offline --locked --manifest-path sdk/examples/input-service/Cargo.toml`:
  23 tests, inclusief per-app foutisolatie en echte queue-readiness.

Logs: `/tmp/nieuw-os-native-wait-{core,sdk,input}.log`.
Probecheckpoint op `76a8a1e16dad4a04841eeb745546a9bca91d1e4a`: de aparte
tweeprocessen-ELF is gereed en crosscompiled (door builder, nog geen boot).
De hoofdagent herhaalde de 3 probe-hosttests en 2 verifier-tests: exit 0.
414 bestaande Python-harnastests en 100 tools-tests slagen, evenals gerichte
Clippy voor capability-core, SDK-runtime en inputconsumenten (`-D warnings`).
De kernel verlangt exacte rolmaskers `0x3f`/`0x3c`, werkelijke wake-/idle-
counters en de nog volle 16-berichtenqueue na revoke; geen alleen-PASS-orakel.

Native wachtpoort **geslaagd**, exit 0/raw 33, 4,142 s:
`LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" NOS_BUILD_DIR="$PWD/build/native-test-version/wait" NOS_TARGET_DIR="$PWD/target/native-test-version/wait" python3 -B -u tools/run_ipc_wait.py`.
Echte queue-/revoke-/FIFO-/pointer-/deadlinegevallen; 14 blokkeringen,
7 ready-wakes, 6 deadline-wakes, 1 revoke-wake en 199 echte idle-IRQ0's.
De probe vond een bestaande lifecyclebug: audit_process las mappings van
een reeds vrijgegeven eerder processlot. Alleen levende slots worden nu
op overlap gecontroleerd; alle overige ownership/W^X-checks blijven staan.
Eerste poging raw 35 faalde daarop; tweede raw 33 werd terecht door de
hostpoort geweigerd wegens ontbrekende M2-bootstrapverwachting; de derde
eist ook die echte markering en slaagt. Logs/fingerprints staan in
`verification/native-test-version/wait-evidence.json`; geen foutpoging is
als geslaagd voorgesteld. QEMU meldde nog een ontbrekende optionele
kvmvapic-ROM-zoekpadinput; die stderr is bewaard.
De lokale ROM-zoekpadlinks en officiële Ubuntu VGA-ROM zijn daarna hersteld,
zonder tracked toolwijziging. Een gerichte AI-review vond bovendien dat
invalid-return-cleanup na de eerste waitrefresh een foutwake één PIT-tick
kon uitstellen. De selectielus herstart nu onmiddellijk bij refresh na
zulke cleanup; deze laatste correctie moet nog native worden herhaald.
Het specifieke owner-invalidreturn/blockedSEND-scenario blijft apart open.

Hertoets op gepushte `ab98d1a169f21eca6b5d64d25160462b5c36dc3f`:
de wachtpoort en bestaande native `test-m3-invalid-rsp` slagen beide raw33.
De legacyproef behoudt survivor-progress, cleanup en DF-entrybewijs.
De GUI-nulmeting op die bron faalt zoals verwacht na 202,396 s/raw35 op
de ongewijzigde 20.000-PIT-tickgrens: drie echte frames, 4 muis-events,
28 key-events, 32 input-ACK's, 14 getypte bytes, nog geen save.
Run: `build/native-test-version/gui-baseline/run/materiaal-normal-f5b3033777`.

Vervolgens alleen de bestaande Materiaal-profielen op `ipc-wait` gezet en
de drie GUI-daemon-entrychecks compatibel gemaakt met exact ABI1.8/1.10.
Inputplanners/tokens, SUR1, appgedrag, kernelbudget en runner blijven intact.
15 compositor- en 15 desktop-init-hosttests slagen; de nieuwe echte
Materiaal-ELFs en kernel zijn gebouwd/geïnspecteerd. De offline build miste
eerst de reeds vastgepinde fontdependency; `cargo +1.90.0 fetch --locked
--manifest-path sdk/examples/materiaal/Cargo.toml` via tools/dev.py herstelde
de lokale downloadcache, zonder manifest- of dependencywijziging.

De input-only-wachtproef op `acd413c836568d1a9c437ffd61edd0e4a8e56e22`
faalt eveneens: 202,363 s/raw35, opnieuw drie frames en geen save. Dat is
geen gemeten performanceherstel. Beide rode runs zijn met bron-/loghashes
bewaard in `verification/native-test-version/gui-attempts.json`.

Volgende afgebakende stap: ook de werkelijk quiescente keyring-, volume-
en desktop-init-diensten laten wachten in plaats van lege IPC rondpompen.
Keyring wacht alleen zonder RANDOM-ingress/pending-randomwerk; volume
alleen nadat alle synchrone disk-/replyhandelingen klaar zijn; desktop-init
alleen in Running zonder pending/actieve stop. Controller houdt zijn lokale
F12-triggerpolling. Nieuwe klok-/waitfouten falen expliciet via bestaande
servicefoutpaden; ABI1.8 behoudt het oude pad. Geen wijziging aan rechten,
SUR1, useractie, save-ACK, deadlines of renderer.
100 keyring/volume-hosttests en 21 desktop-init-tests slagen, inclusief
6 nieuwe echte Engine-/quiescentiegevallen. Service-native-Clippy,
desktop-Clippy en scalar-doelbuilds zijn groen. Het samengestelde
GUI-idle-image is gebouwd/geïnspecteerd.

**Native GUI-normaalpad nu geslaagd op `2066ae7ccb25ac9e5e23eb81ec79a7286699a8ec`**:
exit0/raw33 in 131,561 s binnen het ongewijzigde budget. De echte QMP-keten
doorloopt ready → editor → dirty → saved → list → reopened: 6 frames,
9.737 tile-ACK's, 40 key-events, 4 muis-events, 44 input-ACK's, één save-ACK.
De gast schrijft en herleest werkelijk `nieuw os werkt` (14 bytes), revisie1;
diskcanaries en systeemrand blijven intact. F12/stopketen eindigt met nul
processen/handles en herstelde frames. Geen renderer-, digest-, timeout-
of orakelversoepeling. Compact bewijs: `gui-normal-evidence.json` onder
`verification/native-test-version/`. Het betreft een synthetische plaintext-
fixture, geen versleutelde persoonlijke opslag of volledige GUI-release.
Het normale ESP is bewaard onder `build/native-test-version/test-version/esp`.

De daaropvolgende koude readonly-heropening is geslaagd: zie het actuele
handoff en `gui-recover-evidence.json`. De runner vereist bewust dezelfde
buildroot als de parent; het aparte `gui-recover`-pad is niet gebruikt voor
de geslaagde run. Native kill/hergebruik, FPU met wachten, brede native gates
en complete testreleaseacceptatie blijven open.
De werkbranch is gepusht. Een nieuwe concept-PR is nog niet geopend:
de app-sessie is aan de oudere infrastructuurbranch gebonden, terwijl
de OS-werkbranch bewust apart staat; de sessie-PR-tool kan geen andere head kiezen.
Geen nieuwe voortgangspunten, geen menselijke/hardware-/releaseacceptatie.
## Historisch checkpoint — PR #62 platformbeleidtest hersteld — 30 oktober 2026

Basis `f6d3a108c8b53debd5a8d5415a46ce0772794020`; branch
`simonvanieperen-studious-sniffle`. De nieuwe review vond terecht dat de
synthetische `cc`-test impliciet het hostbeleid gebruikte. Op Windows
controleert de preflight `link.exe`, waardoor die `cc`-asserties falen.
Gereproduceerd met het Windowsbeleid op de Linux-host: één test, twee
assertiefouten, exit 1. De test kiest nu expliciet het synthetische
POSIX-beleid, net als de afzonderlijke Windowstest expliciet haar beleid
kiest. Geen test overgeslagen of assertion verwijderd; beide platformpaden
blijven op iedere host controleerbaar. Productiepreflight niet gewijzigd.

De bronvermelding van het vorige checkpoint is als historische
integratiebron gelabeld. **34% → 34%**; geen nieuw rubriccriterium.
Gerichte controlecommando's en resultaten staan in
`verification/host-test-linker-preflight/README.md`.
CI op de oude head f6d3a108 was volledig groen; nieuwe-head-CI en de nieuwe
reviewbeslissing blijven afzonderlijk nodig vóór squashmerge.

## Historisch checkpoint — PR #62 reviewherstel — 30 oktober 2026

Bronnen: vorige PR-head (integratiebron) `f71393398ea5f28f2ce49eae5279673695f06f48` en toenmalige
`main` `0ade8014c8179241dcfc68ea0ba8083664691240`; branch
`simonvanieperen-studious-sniffle`. Gewone twee-oudermerge, geen rebase.
Main voegt sinds de vorige integratie uitsluitend README-/assessmenttekst en
het hardwarecheckpoint toe. Beide kanten blijven hieronder historisch behouden;
er is één actuele overdracht en één gegenereerde voortgangsmeter.

**34% → 34%**; hardware blijft 1/12, H02–H06 blijven open. Geen runtimebron,
dependency, criterium, native bewijs of bevoegdheidsgrens gewijzigd.
Alleen conflictintegratie is deze ronde geautoriseerd: open reviewfeedback
blijft ongemoeid en vereist een afzonderlijk geautoriseerde behandeling.

Uitgevoerd vanuit `/home/runner/work/nieuw-os/nieuw-os`:
`python3 tools/progress.py --write`, daarna
`python3 tools/progress.py --check` en `git diff --check HEAD`: elk exit 0.
Diffinspectie bevestigt dat alleen README, PROGRESS, PROJECT-STATE en
progress.json wijzigen; de SVG blijft bytegelijk en het criteriumregister
verandert buiten assessment niet. Geen tijdelijke logs of binaries toegevoegd.
Geen nieuwe build-/runtimeproeven nodig voor deze documentatie-only integratie;
eerdere resultaten blijven aan hun eigen bron gebonden. Nieuwe-headchecks en
reviewstatus worden na publicatie afzonderlijk beoordeeld. Eerstvolgend commando:
`python3 tools/progress.py --check`.

## Historisch checkpoint — conflictintegratie PR #62 — 30 oktober 2026

Bronnen: PR-head `c87fd769223e95fbf3a9423247468a0230ca9de1` met actuele
`main` `f52dab50af281dab2c94ab2c9e31aaabd299246a`; branch
`simonvanieperen-studious-sniffle`. Gewone twee-oudermerge, geen rebase.
Conflicten in README, PROJECT-STATE en progress.json zijn inhoudelijk
samengevoegd: beide historische checkpoints blijven behouden met één actuele
overdracht en assessment. De lockfile behoudt RTC-core én de PR-testdependencies.
Geen runtimebron handmatig aangepast; main's RTC/UTC en B02 blijven intact.

De eerdere lokale merge `6fe70fd` met basis `ad2b981` kon niet worden gepusht
door een serverfout en is niet beschikbaar in deze verse clone. Daarom is
opnieuw geïntegreerd tegen de inmiddels actuele basis, inclusief B02.
De schone werkboom bevatte geen IPC-WAIT-gebruikerswijzigingen; de vermelding
daarvan in het historische B02-checkpoint beschrijft een andere lokale sessie.
**34% → 34%**; geen criteria of gewichten gewijzigd.

Integratiecontroles op de samengevoegde werkboom slagen (exit 0): 22 preflight-,
12 RTC/UTC-verifier-, 46 systeemharnas- en 98 tooltests; rootworkspace
365 geslaagd en één bestaande ignored benchmark; SDK-runtime 20 en RTC-core
11 geslaagd; root-/SDK-formattering, keyring-/RTC-clippy en de
`native-system-clock`-crossbuild. De vijf extra workspacetests ten opzichte
van de oude PR-head komen uit B02. Eerste fmt/buildpogingen misten lokaal
Rust-componenten/targets; na herstel slagen ze. QEMU/OVMF ontbreken:
native runtimegates niet uitgevoerd, geen nieuw QEMU-bewijs geclaimd.
Exacte commando's, exits, logs en hashes:
[integratiebewijs](verification/host-test-linker-preflight/README.md#conflictintegratie-met-main-f52dab5).

Mergecommit `cc136a3d4a84dc045089ad86f20925ce39e11a72` is gepubliceerd;
`git ls-remote` bevestigt exact deze PR-head en ongewijzigde basis `f52dab5`.
Secretscan zonder bevindingen. CodeQL Actions/Python: nul alerts; de totale
analyse eindigde met een timeout voordat alle talen gereed waren, dus geen
volledige CodeQL-goedkeuring. Nieuwe-head-CI blijft afzonderlijk te beoordelen;
historische CI-/native resultaten bewijzen die niet. PR niet gemergd.
Eerstvolgend lokaal controlecommando:
`python3 tools/progress.py --check`.

## Historisch checkpoint — hosttest-preflight — 30 oktober 2026

Broncommit `4fb9a2320dd4d5383846507748ea8fdaf9e167cb`, branch
`simonvanieperen-studious-sniffle`, oorspronkelijke taakbasis `main`
`5c334f9ac3100f1c0e503e4f0ac4353a4a35154e`. Concept-PR
[#62](https://github.com/simonvanieperen/nieuw-os/pull/62). De
`--strict-host-tests`-controle meldt POSIX-hostlinkers los van bootvereisten;
de bestaande `--strict`-betekenis blijft ongewijzigd.

Na het lokaal beschikbaar maken van `cc` zonder systeembrede installatie
compileren en draaien een minimale C-proef (exit 0). De volledige
`cargo +1.90.0 test --offline --locked --workspace`-run onthulde ontbrekende
README-doctestdependencies in de runtime-only vendormanifesten van HKDF (3
voorbeelden) en HMAC (2 voorbeelden). Beide crates kregen uitsluitend de
in de meegeleverde originele vendormanifesten vastgelegde `sha2`- en
`hex-literal`-testdependencies; hun
runtime-dependencies zijn ongewijzigd. De gerichte doctests slagen HKDF 3/3
en HMAC 2/2. De volledige workspace-test slaagt daarna: 360 tests passed,
0 failed, 1 ignored, over 51 testresultaatgroepen; log en hash staan in het
bewijsrapport.

De eerdere 22 preflighttests en strikte ontbrekende-`cc`-detectie blijven
brongebonden aan `4fb9a23`; de CodeRabbit-status op die historische head is
onbekend. reviewfix: Windows selecteert nu `link.exe`, de probe gebruikt `/?`
en de negatieve foutstatus is synthetisch getest. De reviewdocumentatie noemt
de opgeslagen manifestpaden, de onbekende toolchainherstelopdracht en het
historische publicatiecheckpoint expliciet.

Uitgevoerd op PR-basis `f71393398ea5f28f2ce49eae5279673695f06f48` met de
reviewwijzigingen in de geïsoleerde worktree (exitstatus 0 tenzij vermeld):

- `python3 -B -m unittest discover -s tests -p 'test_preflight.py' -v` —
  24 tests; `/tmp/pr62-review-preflight-final.log`, SHA-256
  `e3092a33106096d778c6154365052c0b5df9a93c8d7cf06c9fbd7ed4b9af71b6`.
- `python3 -B -m py_compile tools/preflight.py tests/test_preflight.py` —
  geldig.
- `python3 -B tools/preflight.py --strict-host-tests` — exit 2, verwacht:
  deze Linux-host mist `cc`; JSON `/tmp/pr62-review-host-preflight-final.json`,
  SHA-256 `312f1a1c262a05ea3c175d0a64f648ff9308f8912a0c08d20137a11175c0dc9a`.
- `python3 -B tools/progress.py --write`,
  `python3 -B tools/progress.py --check` en `git diff --check` — geslaagd.

Windows-native uitvoering en nieuwe-head GitHub Actions zijn niet uitgevoerd
of nog niet gestart; de tests bewijzen geen Windows-host of OS-feature.
**34% → 34%**; geen productcriterium of native QEMU-bewijs toegevoegd. PR #62
blijft open tot actuele CI en reviewstatus zijn afgehandeld.
Volledige commando's, logs, hashes en grenzen:
[hosttest-preflight](verification/host-test-linker-preflight/README.md).

## Historisch checkpoint — hardware, drivers en energie — 30 oktober 2026

Basis `f52dab50af281dab2c94ab2c9e31aaabd299246a` (`main`), branch
`simonvanieperen-friendly-guacamole`. Opdracht: hardwarepunten H02–H06
verderbrengen richting 12/12; de vaste meter blijft **34% totaal, hardware
1/12**. H01 is het enige geaccepteerde hardwarepunt. H02 toetsenbord/muis/USB,
H03 graphics/audio, H04 netwerkhardware, H05 energie/slaap/wek en H06 de
fysieke ondersteuningsmatrix blijven open. De code bevat een afgebakende
native PS/2/QMP-proef, maar geen USB/xHCI-driver of ACPI-energiepad. Criteria
die fysieke metingen of meerdere fysieke configuraties vragen kunnen niet
worden afgetekend met QEMU-bewijs.

Uitgevoerd: `python3 -B -m unittest discover -s tools -p
'test_display_probe.py'` — exit 0, 11 hosttests voor de bewijspoort; dit
controleert het harnas, niet een guest-driver. De vastgezette Rust 1.90.0-
toolchain is lokaal onder `.tools/nieuw-os` hersteld met de officieel
gepubliceerde rustup-installer; SHA-256 gecontroleerd vóór uitvoering en geen
globale PATH-/systeemwijziging. Op `drivers/ps2` slagen Clippy, fmt en de
release-librarybuild voor `x86_64-unknown-none` (elk exit 0). De PS/2-hosttest
faalt met exit 101 omdat de systeem-C-linker `cc` en hostruntimebibliotheken
ontbreken.

Simon heeft de map `C:\Program Files\qemu` meegestuurd. In WSL is dit
`/mnt/c/Program Files/qemu`; QEMU 11.1.0 start via WSL-interoperabiliteit,
QMP via stdin/stdout is bevestigd, en de package bevat
`edk2-x86_64-code.fd`, `edk2-i386-vars.fd` en `vgabios-stdvga.bin`. De
projecttoolmap verwijst lokaal naar die binaries/firmware, zonder wijzigingen
aan de Windows-installatie of tracked source. `python3 -B tools/preflight.py
--strict` slaagt en rapporteert beide Rust-targets en x86_64-codefirmware
beschikbaar. Dit is alleen tool-/firmware-inventaris; native guestboot is nog
niet uitgevoerd. Geen fysiek apparaat gebruikt of gewijzigd.

Eerstvolgend: `python3 -B -u tools/build.py --kernel-feature
native-display-probe`, daarna de bestaande
`tools/run_display_probe.py --machine pc-q35-8.2 --qmp-transport pipe` met de
meegeleverde VGA-ROM. De 11.1-binary verschilt van de 10.0.13-CI-binary; de
runner moet die exacte versie en hashes vastleggen. Als native poort slaagt,
is dat afgebakend QEMU-bewijs voor de PS/2-display/input-slice, niet fysieke
acceptatie van H02. Daarna volgt een afzonderlijke USB/xHCI-stap. H02–H06
blijven open totdat hun volledige vereiste native/fysieke bewijsniveaus zijn
aangetoond; geen rubricpunt toegekend.

## Historisch checkpoint — B02-PERSIST-1: persistente sleutelstatus en verzegeld record native — 30 oktober 2026

Basis `ad2b981` (`main` na PR #63), branch
`codex/b02-persistent-keystate-20261030`, worktree
`/home/simon/projects/nieuw-os-b02`; doelbranch `main`. Geteste werkboom:
bronfingerprint v3 `32da9d6159bffc70c9ef60dd2c33c678e4222ba8c6a2cc72b5a54a89d4c74504`.
De andere zichtbare lokale sessie (worktree `simonvanieperen-studious-sniffle`,
PR #62) heeft niet-gepushte IPC-WAIT-wijzigingen in `kernel/src/m2/m3.rs`,
`sdk/runtime` en `tools/build.py`; dit werk raakt die bestanden niet.

Geïmplementeerd (ADR-051, FR-B02P in `implementation/B02-SECURE-RECORDS.md`):
KEY1-opcodes `EXPORT_PERSISTENT`/`RESTORE_PERSISTENT` (11/12) met
`wire::dispatch_persistent`; een optioneel derde administratief ingress in de
native Key Service (startwoorden 96/104/168) dat uitsluitend persistentie
bedient en alleen de manifest-afzender toelaat; in het FR-I01-profiel is init
de herstelagent met volumedomein 1. Normale run: app verzegelt in RAM →
`KEYS_READY` → init exporteert (created [2,0], revoked [1,0], generatie 3),
weigert verkeerd geheim/generatievloer/GENERATE-op-admin, commit het
120-byteblob als object 8, importeert idempotent → `PERSISTED` → app commit
het 168-byterecord als document 1. Koude herstart op dezelfde read-only
image: init leest object 8, weigert dezelfde negatieven, herstelt vóór `GO`;
app krijgt index 2 bij GENERATE, ontzegelt document 1 met sleutel 1, sleutel 0
geeft `Revoked`, een gewijzigde byte `Auth`. Het schijforakel behandelt
objecten 1 en 8 expliciet als opaque en controleert dat het record de
bekende plaintext niet bevat en het blob alleen `NOSKEY01`/v1/suite 1/
volume-ID buiten de AEAD-body draagt. **34% → 34%**: B02 blijft open
(geen vertrouwd rollbackanker, geen write-ahead per GENERATE in de dienst).

Uitgevoerd vanuit de worktree met de lokale `.tools`-omgeving
(Rust 1.90.0, QEMU 10.2.1, OVMF 2025.11, `pc-q35-8.2`, TCG, 1 CPU/256 MiB),
elk exit **0** tenzij vermeld:

| Commando | Resultaat / log |
|---|---|
| `cargo +1.90.0 test --offline --locked -p keyring-core` | 15 + 5 nieuwe (`persistent_wire`) + 8 + 23 + random; `target/b02-host` |
| `cargo +1.90.0 clippy --offline --locked -p keyring-core --all-targets -- -D warnings` | schoon |
| `cargo +1.90.0 fmt --all -- --check` en `… --manifest-path sdk/Cargo.toml --all -- --check` | schoon |
| `python3 -B -m unittest discover -s tests -p 'test_system_*.py'` | 46 tests (3 nieuw) |
| `python3 -B sdk/examples/keyring/build.py`, `sdk/examples/system/build.py` | native ELF's binnen linkerbudget |
| `python3 -B -u tools/test_system.py` (`NOS_BUILD_DIR=build/b02-system`) | FR-I01 normaal 3,2 s en koude herstart 3,3 s, raw 33; `build/b02-system/system-gate.json` |
| `python3 -B tools/capture_system.py --destination docs/verification/full-release-b02-keystate-system` | twee runs onafhankelijk hergevalideerd, 120 KiB compact bewijs |
| `python3 -B -u tools/test_keyring.py` (B01-regressie, `build/b02-keyring`) | 5/5 QEMU-gevallen, 52+15 host; adapter met admin-woorden op 0 ongewijzigd |
| `tools/run_random.py --case raw/sdk/unknown/wrong-caller` | alle vier native groen (`build/b02-random-raw`) |
| `tools/run_system_clock.py` | groen (`build/b02-clock`) |
| `tools/test_net_system.py` | normaal 31,9 s en herstel 32,0 s groen (`build/b02-net`) |
| `tools/build.py --kernel-feature native-system-images` + `tools/run_images.py --case normal` | groen (`build/b02-images`), na lokaal ophalen van de ontbrekende offline crates `simd-adler32`/`zune-*` |
| `tools/test_tcp_system.py` | normaal + koude herstart groen (`build/b02-tcp`), na lokale installatie van toolchain 1.91.1 en `cargo fetch` van `smoltcp`/`heapless` |

Baseline vóór de wijziging: `tools/test_system.py` op ongewijzigde `main`
`ad2b981` groen (generatie 1, 1 chunk, klokken [1, 0]), zodat de nieuwe
schijfverwachting (generatie 3, 3 chunks, [2, 1]) herleidbaar is tot de twee
extra commits. Evidence: `docs/verification/full-release-b02-keystate-system/`
(normal `DISK_WRITES` 112, `DISK_FLUSHES` 7; recover 720 reads, 0 writes;
image-SHA `742539dd…` vóór en na de koude herstart identiek).

**Niet uitgevoerd/open:** volledige kernel-clippy (bestaande M2-waarschuwingen,
niet door dit werk), GUI-/Materiaal-keten, CI op de PR-head, menselijke
review, fysieke hardware. Beperkingen: generatievloer is gedemonstreerd met
een lokaal gekozen waarde, geen vertrouwd anker; het geheim staat in de
SDK-ontvangstbuffer tot overschrijving (zoals legacy RESTORE); de
volume-identiteit is de profielconstante `NOS-SYSTEM-VOL01`; tussen GENERATE
en export is er geen write-ahead in de dienst; crash tussen `KEYS_READY` en
`PERSISTED` laat de app zonder commit achter (correct, niet getest).
Eerstvolgende stap: review van de concept-PR en CI op de head; daarna
write-ahead van allocaties in de Key Service via de volumetransactie en een
echt rollbackanker (TPM-CRB-spoor) als volgende B02-stap.
## Uitvoeringshistorie — Windows-QEMU-testversie — 30 oktober 2026

Basis `ad2b981fe1f2bcccb03cb776b2281d32529d2f00` (`origin/main`), branch
`simon/windows-qemu-test-version`; codecheckpoint `f777f7a`. De opgegeven
Windows-map `C:\Program Files\qemu` wordt automatisch herkend door
`tools/windows-qemu/Start-NieuwOS.ps1`; de instructies tonen ook expliciet
`-QemuPath "C:\Program Files\qemu\qemu-system-x86_64.exe"`.

**Packageraanvulling op basis `8f4fb3a`:** de packager hercontroleert de
werkelijk ingelezen kernel-/loaderbytes tegen het native buildmanifest en
de bronvingerafdruk vlak vóór archivering. Een wijziging tussen validatie
en kopiëren geeft een fout zonder ZIP. 33 gerichte/bestaande hosttests,
Python-syntax en diffcontrole slagen (exit 0), inclusief stale-source-,
ESP-symlink- en artifactwisselweigering. Alleen synthetische testarchieven
zijn aangemaakt en door TemporaryDirectory opgeruimd.

Op bron `5b808653c40409a661c5382173da314db1961c11` slaagt daarnaast
`python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` met
111 tests (exit 0, 3,843 s); log `/tmp/nieuw-os-windows-qemu-tools-tests.log`.
Verwachte negatieve testmeldingen in dat log zijn geen echte nieuwe VM-runs.
Read-only afstemming met de integratiesessie is afgerond: hergebruik de zes
normale QMP-bedieningsgroepen zonder F12, maar certificeer hun fasen alleen
met passend bevroren native pixels; niet met de probe-only snapshots.
Na QMP-quit volgt onafhankelijk `disk_oracle`. Nog geen runneruitbreiding,
OS-bronvervanging of bundel; integratiesessie bezit de bevroren runtimebron.

**Windows-launchercontrole op basis `19f2d33` met CheckOnly-wijziging:**
de gekoppelde QEMU-installatie is werkelijk bereikbaar via Windows PowerShell.
QEMU 11.1.0 biedt `rng-builtin` en SDL. Vijf echte PowerShell-launchertests
slagen met synthetische bestanden: expliciet pad, standaarddetectie, gewijzigde
kernel, ontbrekende kernel en verkeerd probeprofiel. Geen VM gestart of
VM-state gemaakt. Directe UNC-uitvoering werd door RemoteSigned geweigerd;
lokaal gestagede eigen scripts werken zonder policywijziging.
`verification/windows-qemu-preview/windows-launcher.md` bewaart de scope en
resultaten. De launcher heeft nu `-CheckOnly` en controleert alle verplichte
bundelbestanden tegen grootte/SHA-256; dat is geen handtekening. PR #65 blijft
concept. Volgende stap: afstemming van de afzonderlijk geteste interactieve
integratiebron, dan bundel bouwen en echte Windows-VM/bediening toetsen.

**Update native preview:** de eerste run faalde na 45 s door een onterechte
vaste Source/Policy-volgorde in het hostorakel, niet door ontbrekende
gastmarkers. De runner is hersteld: boot/manifest blijven geordend, alle
readinessmarkers moeten precies eenmaal verschijnen. Herbouw en herhaling
slagen met runner exit 0/QEMU exit 0 na QMP-quit, 3,874 s en een echt
800×600-frame met 310 kleuren. 31 hosttests slagen. Beide oorspronkelijke
rapporten en seriële logs staan in `verification/windows-qemu-preview/`;
uitvoeringsbasis `6ec0bac` met readinessfix, exacte v3-bronbinding in rapport.
De optionele `kvmvapic.bin`-ROM ontbreekt lokaal; boot slaagt, geen claim van
foutloze firmwareconfiguratie. Geen ZIP gemaakt; de nieuwere GUI-/koudbewijsbron
`simon/native-test-version` wordt eerst met de andere sessie afgestemd.

Geïmplementeerd: beperkte packager valideert het echte normale
`native-system-materiaal`-buildmanifest, alle zeven embedded app-ELF's,
firmware/licentiebestanden en maakt een hashgebonden ZIP. De launcher start
een netwerkloze QEMU-VM met een lege synthetische schrijfbare documentimage,
read-only ESP, lokale OVMF-state, zonder hostmappen/-schijven. De CI-workflow
bouwt en voert de eigen kernel uit, controleert native readiness en echte
framebufferbytes en bewaart de testbundel als tijdelijk artifact. QEMU zelf
wordt niet meegeleverd.

Geslaagd vóór deze overdracht:

- `python3 -B -m unittest tools.tests.test_materiaal_preview tools.tests.test_package_windows_qemu` — exit 0, 9 tests.
- `python3 -B -m unittest tools.tests.test_materiaal_preview tools.tests.test_package_windows_qemu tools.tests.test_materiaal_probe` — exit 0, 29 tests.
- `python3 -B -m py_compile tools/run_materiaal_preview.py tools/package_windows_qemu.py tools/tests/test_materiaal_preview.py tools/tests/test_package_windows_qemu.py && git diff --check` — exit 0.
- `. .tools/nieuw-os/rtc-env.sh && python3 -B tools/preflight.py --strict` — exit 0.
- Met geïsoleerde `build/windows-qemu` en `target/windows-qemu`: `python3 -B -u tools/build.py --kernel-feature native-system-materiaal` — exit 0, zeven apps en ELF's geïnspecteerd, geen waarschuwingen; volledig log `/tmp/nieuw-os-windows-qemu-build.log`.
- Lokale QEMU 10.2.1 bevestigt `rng-builtin`; de host biedt alleen de `none`-displaybackend. De previewrunner gebruikt headless QMP; dit bewijst geen SDL/Windows-launch.

**CI:** historische run [41089090576](https://github.com/simonvanieperen/nieuw-os/actions/runs/41089090576) op `f777f7a` faalde op dezelfde inmiddels herstelde readinessvolgorde (exit 1). Log: `/tmp/nieuw-os-windows-qemu-ci-failure.log`. Dit is geen herhaling op huidige head; geen actuele groene CI-claim.

**Nog niet uitgevoerd:** OS-ZIP-generatie, artifactdownload en Windows-gaststart/bediening. Windows-launcherpreflight is wel uitgevoerd; het script vereist QEMU 10.2.1+, SDL en `rng-builtin`. Geen nieuw native productcriterium gesloten; score **34% → 34%**. Deze tak is alleen de VM-test/distributiestap en sluit het gevraagde 50%-doel niet. Geen volledige release-, hardware- of Windows-compatibiliteitsclaim.

Herhaalcommando (volgende stap: afstemming van de bundelbron):
`. .tools/nieuw-os/rtc-env.sh && PATH="$PWD/.tools/nieuw-os:$PATH" NOS_BUILD_DIR="$PWD/build/windows-qemu" NOS_TARGET_DIR="$PWD/target/windows-qemu" CARGO_TARGET_DIR="$PWD/target/windows-qemu" python3 -B -u tools/run_materiaal_preview.py --bios-rom .tools/nieuw-os/emulator/usr/share/seabios/bios-256k.bin --vga-rom .tools/nieuw-os/emulator/usr/share/seabios/vgabios-stdvga.bin`.

## Historisch checkpoint — NATIVE-CLOCK-1 afgerond; PR blijft draft voor review — 30 oktober 2026

Bronbasis `c057d864e413840644da39b6c8338e883cbe2ff4`, implementatiecheckpoint
`652cd5d4c9c68afb0b143c0147197aa7fc6f8220`, bewijscheckpoint
`ff51dc01b36d3200fc6a887b03429748e93eb457`, CI-geteste codehead
`cd17baed11b3efbab3935697d1e47c0feea517dc`, branch
`simonvanieperen-expert-disco`; doelbranch `main`. Alle genoemde commits zijn
gepusht en de remote-head is gecontroleerd.
De aparte workspace-doctestblokkade blijft buiten deze taak. De bestaande
RTC-NATIVE-1-proef en concept-PR #63 zijn behouden.

Nieuw werk: opt-in ABI 1.9 `native-system-clock`; syscall `0x73` voert
alleen-lezen CMOS UTC uit bij IF=0, zonder user pointer of nieuwe capability.
SDK retourneert `WallClockReading` met vaste kwaliteit `UnauthenticatedRtc`.
De bestaande ring-3-systeemapp voert twee reads, controleert monotone ticks en
weigert ongeldige syscallargumenten; de nieuwe runner vergelijkt timestamps
met expliciete VM-RTC-tijd. Contract: `implementation/RTC-WALLTIME.md`,
ADR-050. Native QEMU-gate geslaagd; **geen nieuw rubriccriterium gesloten;
score blijft 34%.**

Geslaagd op broncommit `baeb09aece05a6fb65bfa7178ae1c146d5506b75`:

- `python3 -B -m unittest tools.tests.test_system_clock` — exit 0, 3 hosttests.
- `PATH="$PWD/.tools/nieuw-os:$PATH" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` — exit 0, 20 tests.
- `PATH="$PWD/.tools/nieuw-os:$PATH" python3 -B tools/dev.py cargo +1.90.0 fmt --all -- --check` en `PATH="$PWD/.tools/nieuw-os:$PATH" python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all -- --check` — beide exit 0.
- `python3 -B -m py_compile tools/run_system_clock.py tools/tests/test_system_clock.py` en `git diff --check` — beide exit 0.
- `PATH="$PWD/.tools/nieuw-os:$PATH" python3 -B tools/build.py --kernel-feature native-system-clock` — exit 0; kernel en native systeemimages gebouwd en geïnspecteerd, uiteindelijke build zonder waarschuwingen.
- Eerste lokale buildpogingen faalden doordat de geïsoleerde `cc`-wrapper niet in `PATH` stond; de expliciete `PATH` hierboven loste dat op. Een waarschuwing over native-only imports is eveneens verholpen en de uiteindelijke build was schoon.
- Uitvoerlogs voor formattering en gerichte host-/SDK-tests staan tijdelijk in `/tmp/nieuw-os-clock-{root-fmt,sdk-fmt,unit,sdk-tests}.log`; ze bevatten geen native QEMU-resultaat.

Native gate geslaagd:
`. .tools/nieuw-os/rtc-env.sh && PATH="$PWD/.tools/nieuw-os:$PATH" python3 -B -u tools/run_system_clock.py`
— exit 0; QEMU raw exit 33, geen timeout. De echte ring-3-app las
`1793361601` tweemaal bij vaste RTC-basis `2026-10-30T12:00:00` (basis +1 s),
zag monotone PIT-ticks `1 -> 1` en passeerde de ongeldige-argumentproef
`-22`. QEMU 10.2.1, q35-8.2/TCG, qemu64, 1 CPU/256 MiB; synthetisch
wegwerpvolume en virtio-RNG. Rapport en seriële uitvoer staan lokaal onder
`build/rtc-native/`; hun hashes en de native ELF-/firmware-hashes zijn bewaard
in `docs/verification/system-clock/evidence.json`.

De brongebonden bewijsnotitie is opgeslagen onder
`docs/verification/system-clock/README.md`. De native runtimegate liep op
assessmentcheckpoint `baeb09aece05a6fb65bfa7178ae1c146d5506b75`. De PR is
bijgewerkt volgens de bestaande template. Op codehead `cd17bae` zijn alle
12 checks geslaagd, inclusief host-quality, native-system (met de nieuwe
klokpoort), native-foundation, native-RTC en de bestaande systeemgates. PR #63
blijft draft: CodeRabbit sloeg hem over en er is geen menselijke review of
reviewbesluit; die reviewstatus wordt niet als goedkeuring voorgesteld.

Open: RTC blijft niet-geauthenticeerd; onbekende UTC-/eeuwconfiguratie en
firmwaregedrag zijn niet opgelost door de syscall. Geen persistente
antirollback, trusted certificate time, NTP, suspend, physical hardware,
SMP-serialisatie, TLS-integratie of volledige FR-S03/FR-N03-releaseacceptatie.

## Historisch checkpoint — RTC-NATIVE-1 bewezen deelstap — 30 oktober 2026

Basis `5c334f9ac3100f1c0e503e4f0ac4353a4a35154e`, branch
`simonvanieperen-expert-disco`; doelbranch `main`. Schone basis en geen open
PR bij inventarisatie. De andere zichtbare sessie beheert de onafhankelijke
workspace-doctestblokkade. RTC-scope en exclusieve bestanden staan in WORK-QUEUE.

Geimplementeerd: opt-in IF0 CMOS-adapter na eigen M2-paging/IDT, echte reads,
floor-/readbudgetweigering, indexherstel en eigen vijfkalender-QEMU-runner/
CI-workflow. Contract: `implementation/RTC-WALLTIME.md`. Geen nieuwe apprechten,
productiekloksyscall, vertrouwde tijd of TLS-integratie. **34% -> 34%**.

Uitvoeringsbron `c02675e21cbbaf32294298f46ce1cb394b360895`; native artifact-,
firmware- en v3-bronbinding: [bewijs en exacte commando's](verification/rtc-native/README.md).
Elf RTC-Rusttests, RTC-clippy, rootfmt, 95 tools-hosttests (negen RTC),
414 bestaande Python-hosttests en loader/kernelbuild slagen met exit 0.
Vijf echte kalenderboots slagen met raw33: millennium/schrikkeldag 2000,
2026, post-i32-2038 en schrikkeldag 2096, inclusief echte budget-/floor-/
restoregevallen. Onafhankelijke verifier: M2 8/8 groepen, zeven VM's
(vijf raw33, twee verwachte raw35); hoofdagent: M6c CI-smoke raw33.
Concept-PR geopend. De echte `native-rtc`-GitHub-job slaagt op `295b4db`
in 47 s, inclusief alle vijf kalenderboots en artifactupload;
`verification/rtc-native/ci-native-rtc.json` bewaart de runbinding.
Overige PR-checks liepen bij deze overdracht nog; geen merge of volledige
CI-goedkeuring geclaimd. Dit laatste checkpoint verandert geen runtimebron.
Geen nieuwe externe dependencyversie; rootlockfile voegt alleen de eigen
dependencyvrije RTC-core toe. Lokale Rust/QEMU/GCC/OVMF-omgeving onder `.tools/`
hersteld zonder sudo/globale configuratiewijziging. Ontwikkelvariant:
QEMU 10.2.1, OVMF 2025.11-3ubuntu7.3; geen historische binarygelijkheid.

Een echte firmwarebeperking is gereproduceerd: de epoch-/eeuwproeven lezen
2070 in plaats van 1970 en 2000 in plaats van 2100. Het onverzwakte
kalenderorakel weigert beide (exit 1, raw33). EDK2's RTC-eeuwcallback
verklaart dit; de gast schrijft geen tijddatacorrectie. Deze gevallen blijven
expliciete fail-closed diagnostiek, geen native range-acceptatie.
Onbeperkte kernelclippy met `-D warnings` faalt op vier bestaande
M2/main-waarschuwingen (geen RTC-locatie); niet als geslaagd gemeld en niet
verstopt met nieuwe lintallows. De nieuwe workflow gebruikt de bestaande
strikte RTC-crateclippy en echte kernelbuild.

RTC-NATIVE-1 is als beperkte afhankelijkheid gereed voor concept-PRreview,
niet als volledige release. Bronreview en bewijscontrole afgerond; geen
open RTC-codeblocker. Alle artefacts blijven aan de uitvoeringsbron gebonden.
Eerstvolgend lokaal herhaalcommando:
`. .tools/nieuw-os/rtc-env.sh && python3 -B tools/run_rtc.py --case all`.
Productvervolg: minimaal geautoriseerde userspace-TimeProvider/trusted-timeketen;
geen ongecontroleerde uitbreiding naar IRQ/SMP of TLS.
Niet afgerond/geaccepteerd: fysieke hardware, menselijke audit, volledige CI
en volledige releaseacceptatie. De 62 productcriteria blijven open.

## Historisch checkpoint — PR #58, integratie met main — 30 oktober 2026

Bronnen: PR-head `8365f14e1c3fae90a781b836405836906ee785ab` en
`main` `970c7713e0c40adc50b4ae5ac665602b2d44d6c2`, op
`codex/native-materiaal-integration`. Gewone twee-oudermerge, zonder rebase.
De integratie behoudt de oorspronkelijke productiedigest, manifestguards,
benchmarkbronbinding, calculator en retire-wisdeadline van main, naast
de PR-reparaties voor gescheiden surfacehandles, lokale replyuitval,
herhaalde pointerannulering en gebalanceerde meetvolgorde. Beide
hostscenarioverifiers en beide compositorregressiesets blijven behouden.

De geteste samengevoegde bron heeft fingerprint-v3
`a27c14d4cd49970efbd7ecfc3998bccda6eae724d1b220d6671d198343a0e761`.
De hieronder opgeslagen runtime-rapporten noemen de toenmalige HEAD als
historische ouder; de fingerprint bindt de daadwerkelijk geteste werkboom.
**34% → 34%**: geen nieuw geaccepteerd productcriterium of versoepelde gate.

Uitgevoerd vanuit `/home/runner/work/nieuw-os/nieuw-os`, elk commando exit **0**:

| Commando | Resultaat en lokaal log |
|---|---|
| `NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B tools/preflight.py --strict` | omgeving compleet; `/tmp/pr58-v3-preflight.log` |
| `python3 -B -m unittest discover -s tools/tests` | 84 tests, 12,005 s; `/tmp/pr58-v3-python.log` |
| `python3 -B -m unittest discover -s tests -p 'test_system_*.py'` | 44 tests, 3,333 s; `/tmp/pr58-v3-system-host.log` |
| `python3 -B -m unittest discover -s tests -p 'test_update_*.py'` | 10 tests, 0,416 s; `/tmp/pr58-v3-update-host.log` |
| `python3 -B tools/build.py --kernel-feature native-cap-digest-bench` | scalar kernel/ELF-build; `/tmp/pr58-v3-bench-build.log` |
| `python3 -B tools/run_cap_bench.py` | echte QEMU-run, pariteit, staat en 9×16 calls per variant; `/tmp/pr58-v3-bench-run.log` |
| `python3 -B -u tools/test_system.py` | FR-I01: normaal en koude readonly recovery, raw33, elk circa 6,2 s; `/tmp/pr58-v3-system-native.log` |
| `python3 -B sdk/examples/materiaal/verify.py` | beide hosttellingen 20, scalar ELF en echte loaderpolicytest; `/tmp/pr58-v3-materiaal-verify.log` |
| `python3 -B tools/build.py --kernel-feature native-system-materiaal` | volledig desktop/compositor/Materiaal-profiel gebouwd; `/tmp/pr58-v3-materiaal-build.log` |
| `python3 -B tools/capture_system.py --destination /home/runner/work/nieuw-os/nieuw-os/docs/verification/full-release-pr58-v3-system` | twee runs opnieuw gecontroleerd en gearchiveerd; `/tmp/pr58-v3-system-capture.log` |
| `cargo +1.90.0 test --offline --locked -p capability-core -p ui-core -p kernel-core -p keyring-core` | 166 geslaagd, één bestaande ignored benchmark; `/tmp/pr58-v3-core-targeted.log` |
| `cargo +1.90.0 test --offline --locked --workspace --manifest-path sdk/Cargo.toml` | 73 tests; `/tmp/pr58-v3-sdk-confirm.log` |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/compositor/Cargo.toml` | 15 tests, inclusief beide reply-/wisregressies; `/tmp/pr58-v3-host-logs/12-test-compositor.log` |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml` | 20 tests; `/tmp/pr58-v3-host-logs/13-test-materiaal.log` |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml` | 19 tests; `/tmp/pr58-v3-host-logs/14-test-ui-native.log` |
| `cargo +1.90.0 fmt --all -- --check` | root; `/tmp/pr58-v3-host-logs/20-fmt-root.log` |
| `cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all -- --check` | SDK; `/tmp/pr58-v3-sdk-fmt-confirm.log` |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/compositor/Cargo.toml -- --check` | `/tmp/pr58-v3-host-logs/22-fmt-compositor.log` |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/materiaal/Cargo.toml -- --check` | `/tmp/pr58-v3-host-logs/23-fmt-materiaal.log` |
| `cargo +1.90.0 fmt --manifest-path ui/native/Cargo.toml -- --check` | `/tmp/pr58-v3-host-logs/24-fmt-ui-native.log` |
| `cargo +1.90.0 clippy --offline --locked -p capability-core --all-targets -- -D warnings` | `/tmp/pr58-v3-host-logs/30-clippy-capability-core.log` |
| `cargo +1.90.0 clippy --offline --locked -p ui-core --all-targets -- -D warnings` | `/tmp/pr58-v3-host-logs/31-clippy-ui-core.log` |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/compositor/Cargo.toml --all-targets -- -D warnings` | `/tmp/pr58-v3-host-logs/32-clippy-compositor.log` |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --all-targets -- -D warnings` | `/tmp/pr58-v3-host-logs/33-clippy-materiaal.log` |
| `cargo +1.90.0 clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | `/tmp/pr58-v3-host-logs/34-clippy-ui-native.log` |
| `python3 tools/progress.py --write` en `python3 tools/progress.py --check` | 34/100 consistent |
| `sha256sum --check SHA256SUMS` | beide nieuwe bewijsarchieven intact |

Alle bovenstaande host-Cargocommando's gebruiken
`CARGO_TARGET_DIR=/tmp/pr58-v3-host-target`. De volledige
`cargo +1.90.0 test --offline --locked --workspace` is óók uitgevoerd:
**exit 101**, drie hkdf-vendordoctests kunnen `sha2`/`hex_literal` niet
importeren (`/tmp/pr58-v3-host-logs/10-test-root.log`). De betrokken
vendorbestanden, rootmanifest en lockfile zijn ongewijzigd tussen beide
ouders. Geen dependency of test is aangepast om dit te verbergen; de
afzonderlijke relevante corecommando's hierboven slagen volledig.
`git diff --check` slaagt voor de eigen wijzigingen. De volledige staged
mergecontrole meldt exit 2 voor twee van main overgenomen calculatorlogs
met een lege eindregel. De finale controle tegen main meldt dezelfde soort
waarschuwing in vier nieuwe ruwe host-stdoutbestanden. Alle bewijsbytes en
bijbehorende hashes blijven ongewijzigd; dit zijn geen broncodefouten.

De benchmark gebruikt `NOS_BUILD_DIR=.../build/pr58-v3-bench` en
`NOS_TARGET_DIR=.../target/pr58-v3-bench`; de systemgate overeenkomstig
`pr58-v3-system` en daarnaast `CARGO_TARGET_DIR=.../target/pr58-v3-system-host`.
De Materiaal-ELF-verifier gebruikt `NOS_BUILD_DIR=.../build/pr58-v3-materiaal`
en `CARGO_TARGET_DIR=.../build/pr58-v3-materiaal-native`; de desktopbuild
gebruikt build-/targetmappen `pr58-v3-materiaal-system`. Alle native runs
gebruiken `NOS_QEMU_MACHINE=pc-q35-8.2` en eigen wegwerpimages.
Compact exact-bytebewijs staat onder
`docs/verification/full-release-pr58-v3-system/` en
`docs/verification/full-release-pr58-v3-components/`. Historische Materiaal-
verificatielogs zijn niet overschreven.

Een tweede AI heeft de samengevoegde reparaties alleen-lezen beoordeeld
zonder significante bevinding; dit is geen menselijke audit.
De secretscan vindt geen secrets in de gewijzigde/nieuwe bestanden.
CodeQL is uitgevoerd na de lokale mergecommit: Python voltooid met nul
alerts, analyse van overige talen afgebroken door timeout. Die overige
analyse is niet groen; de tool verbiedt opnieuw starten. Dit beperkt de
automatische securitycontrole, naast de geslaagde gerichte tests en bronreview.
**Niet uitgevoerd:** volledige GUI-keten, calculator-VM, alle 13 native
updategevallen, fysieke hardware en menselijke productacceptatie.
Het eerdere rode GUI-bewijs blijft dus rood en brongebonden. De hosttests
voor surface-isolatie bewijzen geen native multiapp-bediening.

**Open/blocker:** deze ronde mag uitsluitend conflicten oplossen. De door de
app gemelde review/change-request vereist een afzonderlijk geautoriseerde
reviewronde; er zijn nu geen reviewtargets meegeleverd. Geen PR-merge.
Na publicatie moeten de nieuwe HEAD en bijbehorende CI worden beoordeeld.
Eerstvolgende actie: de app laat actuele reviews/mergevoorwaarden bepalen;
bij een volgende productopdracht blijft de volledige acceptatiematrix
`docs/FULL-TEST-RELEASE.md` leidend, niet alleen deze gerichte regressies.

## Historisch checkpoint — merge van Materiaal en B03 — 30 oktober 2026

De werkbranch `codex/native-materiaal-integration` op `2992ba76` is
samengevoegd met `main` op `9f017354`. De Materiaal- en updateprofielen,
workspaceleden en historische bewijsstukken blijven behouden. De B03-acceptatie
komt uit main; deze conflictresolutie levert geen nieuwe acceptatie op.
De eerdere controles hieronder gelden alleen voor hun oorspronkelijke bron.
Voor deze samengevoegde werkboom zijn geen projectcontroles uitgevoerd.

## Vorig checkpoint — 30 oktober 2026 — PR #58 conflictintegratie

Werkbranch `codex/native-materiaal-integration`; bron-HEAD
`48b51878c03b5b73449f563735c7854b24926fc4`, actuele PR-basis `main`
`6d3bbf0aab07bbcf9b36a9bd74d668adc22f9922`. Alleen conflictresolutie
is in deze Agent Merge-tick toegestaan; geen PR-merge of verdere productbouw.
De eerdere integratie tegen `faf2c134` is afgebroken na de app-update.
`main` en die oude basis hebben dezelfde tree
`fd8c8269a915f0c8cc6fdddeb98d3d1769f00a40`, maar verschillende historie
door squash-PR #56. De huidige integratie is opnieuw berekend met
`git merge-tree --write-tree --merge-base=3f20f087a7f0ddf4757af8d3ea110dd0c044a72c HEAD origin/main`;
die inhoudsvoorouder voorkomt dat al opgenomen werk als nieuwe add/add-wijziging
wordt behandeld. De uiteindelijke merge behoudt `main` als tweede ouder.

De echte conflicten zijn verenigd in `Cargo.toml`, `sdk/Cargo.toml`,
`kernel/build.rs`, `kernel/src/m2/m3.rs` en `tools/build.py`: alle bestaande
workspace-uitsluitingen, ELF-invoer en afzonderlijke Materiaal-, benchmark-,
RNG- en imageprofielen blijven behouden, inclusief hun procesaantallen.
`docs/WORK-QUEUE.md` behoudt het nieuwere foundationcheckpoint;
`docs/progress.json`, README en gegenereerde voortgang blijven **32% → 32%**.
Geen criterium, gewicht, recht, timeout of testassertie is versoepeld.

Op de opnieuw samengevoegde werkboom uitgevoerd, elk commando exit **0**:

| Commando vanuit de repositoryroot | Resultaat / lokaal volledig log |
|---|---|
| `python3 -m unittest discover -s tools/tests -v` | 63 tests, 16,810 s; `/tmp/pr58-probe-tests.log` |
| `python3 -m unittest discover -s tests -p 'test_system_*.py' -v` | 44 tests, 8,632 s; `/tmp/pr58-system-tests.log` |
| `python3 -m unittest discover -s tests -p 'test_build_paths.py' -v` | 3 tests, 0,004 s; `/tmp/pr58-build-path-tests.log` |
| `CARGO_TARGET_DIR=/tmp/pr58-root-target cargo +1.90.0 test --locked -p capability-core -p ui-core -p keyring-core` | 146 geslaagd, één bestaande genegeerde test; `/tmp/pr58-root-tests.log` |
| `CARGO_TARGET_DIR=/tmp/pr58-root-target cargo +1.90.0 clippy --locked -p capability-core -p ui-core -p keyring-core --all-targets -- -D warnings` | `/tmp/pr58-root-clippy.log` |
| `CARGO_TARGET_DIR=/tmp/pr58-sdk-target cargo +1.90.0 test --locked --manifest-path sdk/Cargo.toml` | 83 geslaagd; `/tmp/pr58-sdk-tests.log` |
| `CARGO_TARGET_DIR=/tmp/pr58-sdk-target cargo +1.90.0 clippy --locked --manifest-path sdk/Cargo.toml --all-targets -- -D warnings` | `/tmp/pr58-sdk-clippy.log` |
| `cargo +1.90.0 fmt --all -- --check` | root/kernel formattering groen |
| `cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all -- --check` | SDK-formattering groen |
| `python3 -B tools/build.py --kernel-feature native-system-materiaal-probe` | cross-build, geen gastuitvoering; `/tmp/pr58-materiaal-build.log` |
| `python3 -B tools/build.py --kernel-feature native-system-random` | cross-build; `/tmp/pr58-random-build.log` |
| `python3 -B tools/build.py --kernel-feature native-system-images` | cross-build; `/tmp/pr58-images-build.log` |
| `python3 -B tools/build.py --kernel-feature native-cap-digest-bench` | cross-build; `/tmp/pr58-benchmark-build.log` |
| `python3 tools/progress.py --write` en daarna `python3 tools/progress.py --check` | 32/100 consistent; geen nieuwe acceptatie |
| `git diff --check` | geen witruimtefouten |

Rust 1.90.0 met de vastgelegde targets en bestaande gelockte dependencies is
hersteld; geen dependencyversie of lockfile hiervoor gewijzigd. Een eerste
Materiaal-build miste cache-inhoud; na `cargo fetch --locked` zijn de
bovenstaande volledige builds opnieuw geslaagd. Een leeshelper wijzigde
onterecht de werkboom; die helper is gestopt, de integratie is opnieuw
opgebouwd en alle bovenstaande controles zijn daarna herhaald. De gerichte
read-only code-review van het eindresultaat vond geen significante fout;
dit is geen menselijke audit.

**Bewijsgrens/open werk:** QEMU/OVMF ontbreken in deze omgeving; native
gastproeven en volledige CI zijn hier **niet uitgevoerd**. Historische groene
foundationresultaten en rode GUI-resultaten hieronder blijven aan hun eigen
bron gebonden. Geen volledige GUI-keten, productopslag of release geclaimd.
Na publicatie volgen evaluatie van de nieuwe HEAD en de bestaande native gates
in een ingerichte omgeving. Eerstvolgend inventariscommando:
`python3 -B tools/preflight.py --strict`; in deze omgeving nog geen groene
native inventaris. PROJECT-STATE en de bestaande queue blijven de overdracht.
De samenvoeging behoudt de native productintegratie en de afzonderlijk geaccepteerde B03-updateproef. De rubric bevat daarmee 34 punten; het B03-bewijs en de productmetingen hieronder blijven aan hun oorspronkelijke bronnen gebonden. Deze samenvoeging is nog niet als gecombineerde bron gevalideerd.

## Historisch checkpoint — gezamenlijke native productintegratie, 30 oktober 2026

Op de beschreven productbron bleef de score **32% → 32%**; alle 62 volledige productcriteria blijven open. [Opdracht/matrix](FULL-TEST-RELEASE.md) → [productcontext](FULL-TEST-RELEASE-CONTEXT.md) → [werkqueue](WORK-QUEUE.md).
Integratiebranch `codex/full-test-release` staat op `faf2c134510ce6c2f1d556efe4e64061558e6474`, concept-PR #56. Gezamenlijke bron `9596abc576fceefe2b3ff2f3bd120bcd914eb04c` staat lokaal én op remote `codex/native-product-integration`, concept-PR [#60](https://github.com/simonvanieperen/nieuw-os/pull/60), worktree `/workspace/nieuw-os-worktrees/root-product`. De huidige vervolgreparatie wordt op deze branch gepubliceerd; geen volledige releaseclaim.

**Werkelijk op dezelfde gezamenlijke bron getest:** alle negen CI-workflows op exact `9596abc` slagen, inclusief vier echte RNG-gevallen en PNG/JPEG, FR-I01, display/input, FPU-/geheugen-/opslag-/procesregressies. [CI-metadata en vijf native foundationstappen](verification/full-release-ci-9596abc/README.md). De afzonderlijke echte GUI-keten faalt raw 35 na de vaste 20.000 PIT-ticks (202,859 seconden): wel Materiaal en editor, 3219 echte tiles en vier mouse/ACK-events; nog geen keyverwerking, Save/Reopen of afsluiten. [Origineel rood bewijs](verification/full-release-gui-9596abc/README.md). Het echte Materiaalframe is bekeken; geen visuele eindacceptatie. Het herstel naar de bitexacte oorspronkelijke digest verbetert de eerste-frame-indicatie van 170 naar circa 100 seconden, maar lost de keten niet op.

**Gerichte vervolgreparaties in deze kandidaat:** onverenigbare diagnostische manifests worden expliciet bij compilatie geweigerd; voorheen kon GUI een tegelijk gevraagde probe stil verdringen. Onafhankelijke matrix: 21 geldige gevallen behouden, 22 verboden combinaties weigeren, inclusief TCP-aliassen. Dit sluit geen productfuncties uit. Het afzonderlijke benchmarkharnas trekt oude groene latest-rapporten vóór preflight in, bindt bron/firmware/ELF/manifestkopieën aan vooraf bevroren bytes, en controleert de initiële writable VARS-copy vóór starten; legitieme firmwarewijzigingen tijdens de run blijven toegestaan. Gerichte hosttegenproeven zijn rood vóór en groen na herstel; onafhankelijke laatste hertoets is groen (18 tests, 16 extra gevallen; [bewijs](verification/full-release-product-repairs/README.md)). CI voert voortaan alle tools-tests verplicht uit. De nieuwe echte native benchmark moet nu op de bevroren reparatiebron worden uitgevoerd.

**Bestaande componenten en bewijsgrenzen:** foundation `844ec40`, GUI `48b5187`, digestherstel `5337500` en calculator `0254206` zijn samengebracht. Nieuwe calculator heeft algemene decimale engine, 17 bestaande hosttests, vier extra echte IPC-tegenproeven en 26.424 onafhankelijke rekengevallen; scalar W^X-target is geïnspecteerd. [Review](verification/full-release-calculator-0254206/README.md). Geen calculator-VM, launcher of multiappacceptatie. De eerdere read-only kernelbenchmark heeft juiste bestaande artifacts, 128 echte calls per variant en bitpariteit; meet alleen relatieve TCG-TSC-eenheden vóór userspace/timer, geen fysieke cycles of GUI-prestaties.

**Actieve uitvoering / één eigenaar per bestand:**

- Root: gezamenlijke bron, ABI/build/manifest/status/Git; bevroren native uitvoering en samengestelde regressies.
- kernel_storage: `ipc-wait`, basis `9596abc`; eigen core/SDK/m3/waitmodule en feature. ABI 1.9/sys0x22, maximaal 16 gekopieerde endpointcondities, absolute deadline en horizon 1000 PIT-ticks. Nieuwe Waiting-toestand en terugkeerbare idle/PIT-telling moeten nog worden getest en onafhankelijk beoordeeld; geen userpointerretentie of verminderde checks. Bestaand WIP bleef na een tijdelijke modelcapaciteitsfout behouden en de worker is hervat.
- security_release: `key-volume-wait`, basis `9596abc`; uitsluitend idlehoofdlussen van Key/Volume en nieuw contract. Dezelfde nonblockingoperatie na wake, behoud van protocoldeadlines/unknown/no-replay; oude ABI-profielen en devicepolls behouden. SDK-afhankelijkheid nog in uitvoering.
- native_ui_apps: `desktop-control`, basis `9596abc`; uitsluitend nieuwe gedeelde no_std APPCTRL2/Switch-codec en multiappcontract. Echte Pause/Resume, focus, appwisseling en frames blijven consumerintegratie, geen codecclaim.
- git_build_audit: onafhankelijke benchmarkharnas- en mergehertoets; volgende prioriteit is IPC-wait-review zodra de bron bevroren is. Geen menselijke audit.

**Eerstvolgende uitvoerbare stap:** bewaar/publiceer de gerichte reparaties plus de exacte nieuwe CI-/rode GUI-rapporten; controleer remote/tree. Na onafhankelijke harnasgoedkeuring: bouw/run de read-only kernelbenchmark op die bevroren bron, met eigen `build/product-cap-bench` en `target/product-cap-bench`. Integreer daarna de beoordeelde IPC-wachtkern en idleconsumers, sluit de overige Source/Policy/Comp/Init/App-wachtplannen aan en voer eerst echte native wake/deadline/intrekking/all-waiting-idle-tegenproeven uit. Herhaal dan de GUI-keten op één nieuwe bevroren bron met hetzelfde kernelbudget; cold recovery alleen na een werkelijk geslaagde normale parent. Geen hostdraft, fake succes of verruimde deadline.

**Artifacts / omgeving / resterend verplicht werk:** laatste echte GUI-run `root-product/build/product-gui/run/materiaal-normal-c6820ba1f1`, bron `9596abc`; laatste GUI-image `root-product/build/product-gui/esp/` hoort bij die rode keten. Laatste groene decoderimage `root-foundation/build/foundation-stack-images/esp/`, bron `844ec40`. Alle disks zijn eigen wegwerpimages. Deze probes gebruiken qemu64/TCG/één CPU/256 MiB, fixtures, plaintext disk en tijdelijke RAM-sleutels; VM-FR, veilige persistente opslag/TPM, alle apps/sessie/toegankelijkheid, native TLS/NAS/providers/AI, boot/PQC, updates/herstel, 100 koude starts, 24 uur productgebruik en fysieke/menselijke gates blijven open. TLS `34191bb` en browseronderzoek `a2085b9` blijven intact in eigen worktrees. Een extra TLS-worker kon door de werkelijke threadlimiet niet starten. Tijdelijke exec-uitval is hersteld; onderbroken controles tellen alleen met geverifieerde logs/nieuwe exitstatus. Geen persoonlijke disks/NAS of fysieke hardware aangeraakt.

## Historisch B03-integratiecheckpoint

## PR #61 — conflictintegratie met main — 30 oktober 2026

Bronnen: PR-head `2061863c9226b74d64158f36a3fd1bbbf0bb60df` en
`main` `6d3bbf0aab07bbcf9b36a9bd74d668adc22f9922`. Een gewone merge
behoudt beide geschiedenissen: B03-updateprofielen en de native
systeem-/display-/netwerkprofielen van main. Workspaceleden en lockfiles
worden verenigd; B03 heet ADR-049 omdat main ADR-044 al voor netwerk gebruikt.
Dit verandert geen beveiligingscontract of capabilityrecht.

**Gevalideerd:** beide native gates slagen op dezelfde gecombineerde
bronvingerafdruk v3
`958ff398e14ae01763c1d01fb7d4f555e66f47e039ed502246996be109158cd5`.
De ontbrekende QEMU/OVMF-tools zijn eenmalig geïnstalleerd
(`sudo -n apt-get update` en
`sudo -n apt-get install -y --no-install-recommends qemu-system-x86 ovmf`,
beide exit 0). Rust 1.90.0 en beide x86_64-targets waren aanwezig.
De Cargo-cache is aangevuld met `cargo fetch --manifest-path Cargo.toml --locked`
(exit 0); de lockfiles behouden de versies van beide ouders. QEMU 8.2.2,
`pc-q35-8.2`, TCG, één CPU, 256 MiB en OVMF 4M; uitsluitend wegwerpimages.

**Gewijzigde integratiebestanden:** `.gitattributes`, root- en SDK-manifests/
lockfiles, `boot/loader/Cargo.toml`, `kernel/src/m2/m3.rs`, `tools/build.py`;
ADR-verwijzingen in loader/update-core/SDK/update-tool en B03-documentatie;
`DECISIONS.md`, dit statusbestand en de vier voortgangsbestanden.
De overige bronwijzigingen komen ongewijzigd uit main. De twee nieuwe
bewijsarchieven bevatten tekst/JSON/hashes, geen binaries of geheime sleutels.

**Exact uitgevoerde controles (ieder exit 0):**

```sh
python3 -B -m unittest discover -s tests -p 'test_update_*.py'
python3 -B -m unittest discover -s tests -p 'test_system_*.py'
cargo +1.90.0 test --locked -p kernel-core -p boot-protocol -p elf-loader-core
cargo +1.90.0 test --manifest-path sdk/Cargo.toml --locked
cargo +1.90.0 clippy --locked -p update-core -p update-tool -p kernel-core -p boot-protocol -p elf-loader-core --all-targets -- -D warnings
cargo +1.90.0 clippy --manifest-path sdk/Cargo.toml --locked --all-targets -- -D warnings
cargo +1.90.0 fmt --all -- --check
cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all -- --check
NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B tools/preflight.py --strict
NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B -u tools/test_update.py
NOS_QEMU_MACHINE=pc-q35-8.2 NOS_BUILD_DIR=/home/runner/work/nieuw-os/nieuw-os/build/pr61-system NOS_TARGET_DIR=/home/runner/work/nieuw-os/nieuw-os/target/pr61-system python3 -B -u tools/test_system.py
NOS_BUILD_DIR=/home/runner/work/nieuw-os/nieuw-os/build/pr61-display NOS_TARGET_DIR=/home/runner/work/nieuw-os/nieuw-os/target/pr61-display python3 -B tools/build.py --kernel-feature native-display-probe
NOS_BUILD_DIR=/home/runner/work/nieuw-os/nieuw-os/build/pr61-g0 NOS_TARGET_DIR=/home/runner/work/nieuw-os/nieuw-os/target/pr61-display python3 -B tools/build.py --kernel-feature graphics-g0
NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B tools/capture_update.py --destination /home/runner/work/nieuw-os/nieuw-os/docs/verification/full-release-pr61-conflict-update
NOS_QEMU_MACHINE=pc-q35-8.2 NOS_BUILD_DIR=/home/runner/work/nieuw-os/nieuw-os/build/pr61-system NOS_TARGET_DIR=/home/runner/work/nieuw-os/nieuw-os/target/pr61-system python3 -B tools/capture_system.py --destination /home/runner/work/nieuw-os/nieuw-os/docs/verification/full-release-pr61-conflict-system
python3 tools/progress.py --write
python3 tools/progress.py --check
git diff --check
```

Resultaten: 10 Python-updatetests, 44 Python-systeemtests, 38 coretests,
83 SDK-tests; de updategate herhaalt ook 7 update-core- en 1 update-tooltest.
Alle 13 B03-stages slagen, inclusief echte kill/hervatting en de verwachte
weigeringen; systeem normaal en koud read-only herstel slagen beide met raw
exit 33. [Updatebewijs](verification/full-release-pr61-conflict-update/update-gate.json)
en [systeembewijs](verification/full-release-pr61-conflict-system/system-gate.json)
zijn apart bewaard, inclusief seriële logs en SHA256SUMS.
Lokale volledige commandologs: `/tmp/nieuw-os-pr61-{update,system,display,g0,preflight}.log`;
builddetail onder `build/logs/`, `build/pr61-system/logs/` en de twee
displaybuildmappen. G0 compileert met de bestaande `validate_pixels`
dead-codewaarschuwing; geen waarschuwing onderdrukt.

De eerste systeemcapture met naam `pr61-conflict-system` weigerde terecht
(exit 1): het bestaande harnas vereist `full-release-<unique>`. De capture
met de vereiste naam slaagt (exit 0), zonder wijziging van het harnas.
Een afzonderlijke read-only AI-integratiecontrole vond geen concrete
integratiefout; dat is geen menselijke audit.

De aanvullende `git diff --cached --check` over de volledige merge meldt
exit 2 voor ruwe bewijsbestanden: bestaande CRLF-/lege-eindregelbytes uit
main en de lege eindregel in de nieuw vastgelegde Cargo-stdout. Deze
hashgebonden logs blijven bytegetrouw; er is geen broncodeformattering
aangepast of controle verzwakt om ze te verbergen. De gerichte broncontrole
`git diff --cached --check -- . ':!docs/verification/**'` slaagt afzonderlijk
(exit 0); de archiefbytes zijn met beide `sha256sum -c SHA256SUMS`
gecontroleerd (exit 0).

**Roadmap: 34% → 34%.** Geen nieuw criterium geaccepteerd. Historisch bewijs
hieronder blijft aan zijn oorspronkelijke bron gebonden; de eerdere
foundationstatus van 32% is geen nieuwe beoordeling van deze integratie.
Hosted checks moeten na publicatie op de nieuwe head worden beoordeeld.
Niet uitgevoerd: native display-/G0-bediening, volledige regressiematrix,
fysieke hardwaretests, menselijke audit of volledige productrelease.
Geen PR-merge uitgevoerd. De bestaande B03-grenzen (TEST-ankers, geen
Secure Boot of bescherming tegen wijzigen van OVMF_VARS) blijven gelden.

## Historische bronstatussen vóór deze integratie

## B03 ondertekende updates en rollback geaccepteerd, 34% — 30 oktober 2026

Branch `ccr-43832ef7-gi2qlz`, basis `main` `f63a0c1` (squash van
[PR #52](https://github.com/simonvanieperen/nieuw-os/pull/52), B01; GitHub
Actions daarop groen inclusief `b01-native-supervisor`). Omgeving:
Linux-container, Rust 1.90.0, QEMU 8.2.2 (Ubuntu), `pc-q35-8.2`, OVMF 4M,
slotopslag als `virtio-blk-pci` 00:05.0.

**Gerealiseerd (loaderfeature `signed-boot`, kernelprofiel `native-update`, ADR-049):**
- `crates/update-core`: manifest `NOSMAN01` (84 canonieke bytes + Ed25519 +
  ML-DSA-65), slotopslag `NOSSLOT1` (header-hint, twee slots van 2 MiB, inbox),
  hybride verificatie tegen vastgepinde ankers, selectie met versievloer en
  terugval, installatiebeslissing.
- Loader `boot/loader/src/signed.rs`: zoekt de slotopslag via Block I/O,
  verifieert beide handtekeningen, digest én ELF-laadbaarheid vóór één byte
  wordt uitgevoerd, leest en verhoogt de UEFI-vloer `NieuwOsMinVersion`, valt
  terug op het andere slot, stopt met `BOOT:FAIL:NO_VALID_SLOT` of
  `BOOT:FAIL:NO_STORE` (nooit terugval naar een onondertekende ESP-kernel);
  selectie op een eigen 1 MiB-stack.
- Kernel `kernel/src/m2/m3/update.rs`: dezelfde generieke blokbroker als het
  volumeprofiel over de hele slotopslag, zonder formaatkennis of
  cryptografie; gelabelde crashfixture `test-update-cut`;
  `KERNEL:IMAGE_VERSION`; 512 KiB gebruikersstack in dit profiel
  (`kernel-core::user::valid_return_layout` kreeg de stackgrens als parameter).
- Ring-3 update-dienst `sdk/examples/update`: verifieert de inboxbundel met
  beide TEST-ankers en de digest vóór de eerste write, bepaalt het draaiende
  slot via geverifieerde versie = kernelversie (nooit via de header),
  installeert image → FLUSH → manifest → FLUSH → header → FLUSH, hervat een
  onderbroken installatie.
- `tools/update-tool` (keygen/sign/verify/pack), TEST-ankers en -seeds in
  `tools/update-keys/`, Python-orakel `tools/update_disk.py`, runner
  `tools/run_update.py` (13 stages in 9 reeksen), gate `tools/test_update.py`,
  capture, 10 harnastests, workflow `B03 signed updates acceptance`.

**Uitgevoerd en geslaagd:** zie [B03-ACCEPTANCE](verification/B03-ACCEPTANCE.md)
en `docs/verification/b03/`. Loader- en kernelbewijs: installatie v1→v2 (3 048
sectoren, 3 flushes, 1 headerwrite), herstart in slot B met vloer 1→2,
downgrade van een geldig ouder slot geweigerd, geflipte ML-DSA-byte, geflipte
imagebyte en ondertekend niet-ELF-image elk als slot geweigerd met terugval en
herstel van slot B, echte hostkill midden in de kopie zonder header- of slot-
A-wijziging gevolgd door hervatting en start van v2, beide slots ongeldig →
gecontroleerde stop zonder kernel, inboxbundel met kapotte handtekening,
lagere versie of vreemde sleutel geweigerd zonder één write. Hosttests: 7
`update-core`, 1 `update-tool`, 16 `kernel-core`, 10 Python.

**Onafhankelijke review (AI, geen menselijke audit):** een adversariële
reviewronde over loader, kernel en dienst leverde vier bevindingen die vóór de
gate zijn verwerkt: terugval naar de ESP zonder slotopslag (nu harde stop),
vloer verhoogd vóór ELF-validatie (nu pas na een laadbaar image, stage
`bad-elf`), hardgepinde 64 KiB-stackgrens in `kernel-core` (nu parameter) en
een cut-controle die een hervatte installatie zou afkeuren.

**Roadmap rubric v1: 32% → 34%** (B03 +2).

**Open en niet bewezen:** GitHub Actions op de PR-head moet de gate herhalen;
geen firmwareverificatie van de loader (Secure Boot, B04), geen
sleutelrotatie/intrekking of releaseankers (TEST-ankers met seeds in de
repository), geen netwerkdistributie, geen pakket-/app-updates (A02), geen
fysieke schijf of stroomuitval, geen bescherming tegen een schrijver van
OVMF_VARS. De kernelbroker pollt het apparaat nu één keer direct bij
`block_wait`; het volumeprofiel is daarin niet gewijzigd.
## Eerder foundationcheckpoint — 30 oktober 2026 — volledige testrelease in uitvoering

Integratiebranch `codex/full-test-release`, bron `844ec40d0717a51d76f2da3a03fa415633725f6a`, tree `6710e599e6984f69e227367981f9a07b525b1255`.
Deze exacte bron staat lokaal en op remote; concept-PR [#56](https://github.com/simonvanieperen/nieuw-os/pull/56) blijft de volledige releaseopdracht volgen. Foundation-PR #59 is na gecontroleerde fast-forward opgenomen en gesloten.
[Volledige opdracht en 62 criteria](FULL-TEST-RELEASE.md) → [aanvullende productcontext](FULL-TEST-RELEASE-CONTEXT.md) → [werkqueue](WORK-QUEUE.md) → [vaste rubric](PROGRESS.md).
**32% → 32%; alle 62 volledige productcriteria blijven open. Er is geen volwaardige testrelease.**

**Nieuw aantoonbaar resultaat:** alle negen CI-workflows op exact `844ec40` slagen. De foundationjob voert de vier RNG-gevallen en de PNG/JPEG-proef daadwerkelijk native uit, zonder overgeslagen stap. [CI-metadata en bewijsgrens](verification/full-release-ci-844ec40/README.md).
De lokale decoderproef op dezelfde bron slaagt met raw exit 33: acht PNG's, drie JPEG's, 149.647 pixels, 623.808 samplebytes en tien negatieve gevallen. De oude stackguardfout is gericht hersteld met fallible heap-snapshots; stack 64 KiB, heap 4 MiB en pixeloracles zijn behouden. Een afzonderlijke hercontrole bevestigt bron, zes ELF's, firmware, ESP, counters, 4096 schijfbytes, canaries en cleanup. [Decoderbewijs en onafhankelijke controle](verification/full-release-images-844ec40/README.md).
De eerdere bootstrap-, ABI- en stackguardmislukkingen blijven in hun oorspronkelijke archieven staan. Dit is geen beeldviewer of bewijs van volledige formaatdekking, native OOM/herstel of veilige persistente productopslag.

**Kritieke GUI-keten blijft rood:** `codex/native-materiaal-integration`, worktree `/workspace/nieuw-os-worktrees/root-materiaal`, concept-PR [#58](https://github.com/simonvanieperen/nieuw-os/pull/58), bron `48b51878c03b5b73449f563735c7854b24926fc4`. De echte acht-processenketen toont Materiaal en de editor op bron `5e126b7`, maar bereikt binnen de vaste 200 seconden nog geen typen → opslaan → heropenen → afsluiten. De digestoptimalisatie op `1fd879d` maakt de eerste volledige frame native trager (170 seconden). Rood bewijs blijft bewaard; de timeout of controles zijn niet verruimd.
Een echte, read-only kernelmeting op `48b5187` slaagt met 128 aanroepen per variant en bitexacte pariteit: reference 36.998.312, direct 43.931.758, zero-run 77.735.944 opgetelde TCG-TSC-eenheden. Dit zijn relatieve ontwikkelmetingen vóór userspace en timer, geen fysieke cycles of GUI-acceptatie. Root herstelt de snelste gemeten referencevariant en herhaalt daarna de werkelijke GUI-proef.

**Lopende onafhankelijke taken en eigenaarschap:**

- `CAP-DIGEST-TCG-2`: kernel_storage, worktree `cap-digest-opt`, basis `952e9ac`; uitsluitend capability-core en digestcontract. Root bezit kernelhooks, build, ABI en integratie.
- `MULTIAPP-CONTRACT-1`: native_ui_apps, worktree `multiapp-contract`, basis `48b5187`; afzonderlijk contract voor echte Materiaal-/rekenmachineprocessen, focus, appwisseling en levensduur.
- `CALCULATOR-REVIEW-1`: git_build_audit, alleen lezen op `0254206f9b44c75499cf55a9c3716fdb363372ce`. Component: 17 Rust-hostgroepen, 1.262 Decimal-vergelijkingen en targetbuild groen; nog geen native appbediening of FR-A08-aftekening.
- `GUI-BENCH-REVIEW-1`: security_release, alleen lezen op `48b5187`; onafhankelijke controle van echte benchmark en harnas. Geen menselijke veiligheidsaudit.

Beschikbare agentthreads worden gebruikt; een extra TLS-worker kon door de werkelijke threadlimiet niet starten en blijft wachtrij. TLS `34191bb` en browseronderzoek `a2085b9` staan intact in hun eigen worktrees, nog niet geïntegreerd. TPM `6bed190` is geïntegreerde transport-/commandocore met host-/targetbewijs; native MMIO-adapter, TPM-anker en koude rollbackdetectie blijven open.

**Eerstvolgende uitvoerbare stap:** neem de beoordeelde reference-herstelcommit in `root-materiaal` over, wijzig de benchmark-ZERO-pointer expliciet naar `diagnostic_zero_digest`, controleer bitpariteit en bouw/herhaal `tools/run_materiaal_probe.py` op bevroren gepubliceerde bron met eigen buildmap. Gebruik de bestaande LOCALAPPDATA/PATH/OVMF/QEMU-omgeving en dezelfde 200-secondenacceptatie. Integreer daarna de foundation- en GUI-bronnen zonder gebruikerswerk of bewijs te verliezen en test de samengestelde versie. Het rekenmachinecontract wordt parallel voorbereid; geen simulaties of verborgen bediening.

**Artifacts en grenzen:** laatste geslaagde lokale decoderimage: `/workspace/nieuw-os-worktrees/root-foundation/build/foundation-stack-images/esp/`; manifest en runpad staan in het gekoppelde bewijs. Laatste GUI-benchmark: `root-materiaal/build/cap-bench/cap-bench-report.json`; laatste rode GUI-run: `root-materiaal/build/materiaal-digest/run/materiaal-normal-230683c431`. Alle testdisks zijn eigen wegwerpimages. Deze ontwikkelproeven gebruiken qemu64/TCG/één CPU/256 MiB, vaste fixtures, plaintext testdisk en tijdelijke RAM-sleutels. Zij vervangen VM-FR (Nehalem-v1, 8 GiB, 32 GiB disk, resoluties/schalen), fysieke tests of menselijk onderzoek niet. Alle verplichte apps, toegankelijkheid, sessie/instellingen, beveiligde opslag, netwerk/TLS, NAS/providers, lokale AI, boot/PQC, updates en herstel blijven zichtbaar open. Honderd koude starts en 24 uur productgebruik vereisen eerst een stabiel geïntegreerd productimage.

## Historisch checkpoint vóór integratie van 844ec40

Onderstaande tekst bewaart de toenmalige uitvoering. De actuele bron, taken en hervatstap staan uitsluitend hierboven.

## Volledige native testrelease — in uitvoering, 32% — 2026-10-30

**Actueel herstelcheckpoint:** nativeRNG is opnieuw4/4raw33 op9aa74870 en alleachtbestaandeCI-workflows op9aa groen. Image-op9aa faalde naABIcheck met echteJPEGstackguardfault; [origineleuitvoer](verification/full-release-image-stack-failure-9aa/README.md) behouden. Snapshotfixade6+onafhankelijkeOOM/rollbackreview isnugeïntegreerd metongewijzigde64KiB/4MiB. Nieuwe negendeCI-workflow93e991 voertalle4RNGgevallen+imageechtnativeuit; ookTPM6bedpurecorechecks toegevoegd. StandaloneRNG-fmt isgerichthersteld, geencheckuitgeschakeld. Nieuwedecode-VM/9CIenaktuëleRNGherhalingnogvereist.

GUI1fd/db04blijftFAIL200s, eersteReady170s: host2,48×vertaaldenietnaarnativewinst. Bitexactehelpersblijvenveilig, maarvariantkeuzemoetopdoelmetingengebeuren. Er wordt een korte eigenkernelbenchmarkreference/directtyped/zerorunsvoorbereid; géén200s-verruiming, geenrechten-/IPCcheckverwijdering. Calculatorappisinonafhankelijkeuitvoering; code/hostzijngeenappacceptatie. Nativebeeldviewer/NAS/AI/opslag/update/herstel enalle62criteriaopen.

**Vier nativeRNGgevallen op2b5 slagen raw33**, onafhankelijkhercontroleerd: [bewijs](verification/full-release-random-2b5d69c/README.md). Image-opdezelfdebronFAILABIstep1: [rood](verification/full-release-image-abi-failure-2b5/README.md). Fix9d8gebruiktdewerkelijkekernelABIcall; standalonekeyring-clientlockwordtgerichtbijgewerkt voorCI --locked. Huidigeviercasesblijvenexactaan2b5gebonden; volgendeherstelbronvereistnieuweVMs.

**NativeRNG-eersteuitvoering454bf1b is rood:** raw35 vóórSYSTEMMANIFEST, echtekernelweigering bij bootstrapdelegatie uitKey-parent zonderTRANSFER. [Oorspronkelijke rawrapporten](verification/full-release-random-bootstrap-454bf1b/README.md) behouden. Rootcorrectie geeft uitsluitend deinterneKey-parent het bedoeldeTRANSFER, geenprobe-/APP-rechtenverruiming engeencontrolewijziging. Nieuweviercases enimage-uitvoering volgen opbevrorenherstelbron. Harnasreparaties zijn onafhankelijk opnieuwgroen21+20+10tegenvoorbeelden, zie bijgewerktepre-nativearchief.

**Nieuw foundationcheckpoint:** branch `codex/native-foundation-integration`, eigenworktree `root-foundation`, basis `3f20f087a7f0ddf4757af8d3ea110dd0c044a72c` (achtCIgroen inclusief echte display/input). VolledigeRNG- en PNG/JPEG-doelbuilds slagen; [componentreviews/hostchecks/gerichteharnasreparaties](verification/full-release-foundation-pre-native/README.md) zijn vastgelegd. RootsharedABI/bootstrap/locks/buildadmission zijn aangesloten. De vier RNG-gevallen en imageprobe moeten nu op deze bevroren bron werkelijk worden uitgevoerd; tot die tijd geen native acceptatie.

ParallelleGUIbron `7d230a6e8e7d7975f8e3297ff4f2b166c38b1803`, concept-PR[#58](https://github.com/simonvanieperen/nieuw-os/pull/58), heeft een echte8-processenrun met Source/Policy/Comp/Focus en één volledig native geschilderd Materiaalframe. De volledigeproef is FAILna200s: eerste1608tegelframe rond106s en tweede pas1553/1608tegels. Geen Saved/Reopen/afsluitacceptatie. Root onderzoekt uitsluitend observeroverhead (batchedPITsample); functies,veiligheidschecks en de200s-proefgrens blijven staan. Drie GUIreplaybevindingen worden door de eigenaar gerepareerd vóór heracceptatie. Huidigfragment is geen productimage; veilige persistente opslag/NAS/AI/apps/boot/update/herstel en alle62criteria blijven verplicht open.

**Eerstvolgende concrete hervatstap:** controleer HEAD/index/remote en activeworkers; bouw en voer achtereenvolgens `native-system-random`→run_random raw, `test-system-random-sdk`→sdk, `test-system-random-unknown`→unknown, `test-system-random-wrong-caller`→wrong-caller uit, ieder eigen `NOS_BUILD_DIR=build/foundation-<case>` en hergebruik `NOS_TARGET_DIR=target/foundation` uitsluitend sequentieel. Daarna `native-system-images`→run_images normal. Gebruik bestaande LOCALAPPDATA/PATH/QEMU/OVMFenv. Vier RNG-probes en imageprobe zijn deelacceptatie, niet volledige release; toets na laatste bronwijziging opnieuw. Behoud `/workspace/nieuw-os-worktrees/root-materiaal` met eigen GUIWIP en originele nativeFAILartifacts.

Hieronder volgt de behouden voorgeschiedenis van de actuele native regressiebasis; eerdere hervatcommando’s worden door dit foundationcheckpoint vervangen.

Integratiebranch `codex/full-test-release`, concept-PR[#56](https://github.com/simonvanieperen/nieuw-os/pull/56).
Desktopintegratie `codex/root-desktop-integration`, concept-PR[#57](https://github.com/simonvanieperen/nieuw-os/pull/57).
[Volledige opdracht/matrix](FULL-TEST-RELEASE.md) → [productcontext](FULL-TEST-RELEASE-CONTEXT.md) → [werkqueue](WORK-QUEUE.md).
De volledige oorspronkelijke opdracht, `/goal`, rubric en alle62 productcriteria blijven behouden en open. **Geen volledige release;32%→32%.**

**TweedeGUI5e iseveneensFAIL raw35/202,866s:** [bewaarbewijs](verification/full-release-gui-second-5e126b7/README.md). CompleteMateriaal énEditorframes zijn échtbereikt/bekeken, maargeenSaved/Reopen/cleanSTOP. Deobserververbeteringalleenisevidentnietgenoeg. GerichtebitexacteIPCdiagnosticoptimalisatiedb04wordtnugeïntegreerd:52coretests/Clippy/scalartargetgroen, host2,48×geenNativeclaim. Alleveiligheids-/IPCchecks/deadlineblijven, onafhankelijkehashreviewloopt; nieuweVMvereist.

**GUI7d eerste native uitvoering is FAIL, niet afgetekend.** [Originelelogs](verification/full-release-gui-first-7d230a6/README.md): Source/Policy/Comp/Focus en echtvolledig Materiaalframe1608tegels; eerste frame rond106s, totale3161tegels bij200s. Geen Saved/Reopen/STOP. De huidige gerichte observercorrectie leest één metadataframe per echtePITtick via éénbounded1024B-scratchkopie; brokers blijveniederentry, alleIPC-/cap-/FPU-/pagingchecks en200sdeadline blijven. Eindcleanup staatbuitencadence (max1tickfalse-negative hersteld). Onafhankelijkehelper/Gate7cases groen, geen nieuweprestatiemeting of nativeclaim.

GUIharnasfix3e2dd322 voorkomt stale/latecapturen, bindtactievedisk aan afterimage en herberekent de ene concrete koudeNormalouderketen.19repo+21onafhankelijkechecks groen; oude rode tegenvoorbeeldenbewaard. De nieuwe bron wordt nu gepubliceerd en daarna nativeherhaald; oudeGUI7d-run blijft rood. Foundationbron2b5d69c repareert internRNGTRANSFER na een terechtekernelweigering; eigen4RNG/imageVMs lopen onafhankelijk in root-foundation, geenresultaat geclaimd.

**Actuele integratie:** mainbranch staat op `3f20f087a7f0ddf4757af8d3ea110dd0c044a72c`, alleachtCI-workflows groen inclusief de echte display/input-CI. [Metadata, jobstappen en ruwe uitvoerselectie](verification/full-release-ci-3f20f08/README.md). De lokale overdracht van het CI-ZIP is proxy403-geblokkeerd; lokale eigenimages zijn beschikbaar. Bron `907d6ce` had eerder allezevenbestaandeCI groen.

**Nieuwe GUI-integratie in eigenworktree `root-materiaal`:** SUR1deadlinefix6066645, typedINP1client4c0ddef, compositor0390f70, appReady/entryfix18155b0+F12uitbreiding8037a84 en echte init/controllerba5b83e zijn opgenomen met oorspronkelijke workerbewijzen. Onafhankelijke [app-/initreviews](verification/full-release-gui-component-reviews/README.md) vinden na de Ready-reparatie geen concrete componentblocker; zij zijn geen menselijke audit. De werkelijk gekoppelde8ELFs en kernel hebben een eerste volledige targetbuild doorstaan; root herhaalt20 app- en15 init-hostcases op de samengestelde bron. Noggeen nativeGUI-run.

Root bezit de nieuwe desktopmanifesten, buildhook en een pureproefgate die Saved→latereList→latereReopen→nieuwe echteF12Press vereist. De8eCPL3controller heeft slechts tweeIPC-leaves en een eigen testbarrier; hij voert de echte initSTOP-RPC uit. APPgeen rawapparaatrechten; compositorDisplay; SourceInput; policyvierontvangstkanalen. Eerste targetedreview vond en herstelde een builderimportfout, ontbrekendeRECV voor een ongebruiktKey-replyendpoint en logsymlinkwrites. Alleoutputpaden worden nu vóór het eerste commando gevalideerd. De200s-timeout geldt uitsluitend voor de opt-inproef, niet de7-processeninteractievevariant. Alle gegevens blijven plaintextfixtures met RAMkeys; dienstcrashnotificaties/herstart, productopslag en overigeGUI/releasescope zijn open.

Parallelle rootworktree `root-foundation` bevat nativeRANDOMfc2/3299/651+c01/e7 plus PNG/JPEG9ae en imageobserver474. Canonieke key/systembuilders hebben beide portableChaCha-configuraties; echte volledigeRNG- en imageprofielen bouwen. Hostkeyring/clientchecks slagen. VierRNG-VMs en decoder-VM moeten nog op hun finale bevroren bron worden uitgevoerd; nieuwe onafhankelijke adapters-/decodereviews enRNGharnas lopen. Geen uitvoering geclaimd vanuit deze builds. TPM-CRB-transportcore is nu een nieuw concreet onafhankelijk spoor; ankerclient/native/persistente sleutels blijven open.

**Geteste bronnen en werkelijke grenzen:**

- `985ed4da0ead6865c8d1fd0d4deb34ef7f97f3eb`: dertien lokale VM-runs en zeven CI-workflows geslaagd. [Basis/cold](verification/full-release-base-985ed4d/capture-validation.json), [ARP/DNS/cold](verification/full-release-net-985ed4d/capture-validation.json), [TCP/cold](verification/full-release-tcp-985ed4d/capture-validation.json), [geheugen](verification/full-release-memory-985ed4d/README.md), [FPU](verification/full-release-fpu-985ed4d/fpu-report.json), [G0](verification/full-release-g0-985ed4d/graphics-g0-result.json), [CI](verification/full-release-ci-985ed4d/workflow-runs.json).
- `15e8cfced28ad46f15f5f573e5f00e8e87b74215`: C-entry springt na echte CPL3-uitlijningscontrole direct naar de C-functie; vijf nieuwe geheugencases werkelijk geslaagd, inclusief NX/guard/peerfault en fysieke OOM. [Nieuw bewijs](verification/full-release-memory-15e8cfc/README.md), [zeven CI groen](verification/full-release-ci-15e8cfc/workflow-runs.json). V3-bron `a82648f0673ef6c0b37406724fc5512ffadc5b53b4987bf21e96521faa112eaf`. De historische985-runs bewezen de gecorrigeerde entry niet.
- `d52882196da3f38a32bf01edea379ce5300e04e2`: **echte display-/PS2-proef geslaagd**, zes CPL3-levens/twee devicegeneraties/drie slothergebruiken. QMP-toetsen en muis, volledige input-/outputpointercontrole, geweigerde rechten,512 unieke native pixels/768 vergelijkingen, cleanup. [Native bewijs](verification/full-release-display-d528821/README.md), [zeven CI groen](verification/full-release-ci-d528821/workflow-runs.json). Raw33/3,387s; bron `eb46a28e25e3c532f0efb5b864f449eae6d9ee4dc59c9eee23c03ffe31fe2406`.

De displayfixture selecteerde eerst ten onrechte1280×720 terwijl de bestaande loader expliciet800×600 selecteert. De [oorspronkelijke mislukking](verification/display-first-geometry-failure/result.json) blijft ongewijzigd bewaard; daarna is uitsluitend de ontwikkelproef strikt aan de bestaande800×600-loadercontractselectie gebonden. VM-FR blijft1280×720/1920×1080/200%. Bij eerste Git-archivering normaliseerde Git twee CRLF-regelafbrekingen in de mislukte serial; deze evidence-only correctie herstelt de originele430bytes/SHAed6e4ce7... en controleert de staged blobs tegen SHA256SUMS. Het werkelijk geteste d528-image verandert daardoor niet.

Dit zijn QEMU-ontwikkelprofielen pc-q35-8.2/qemu64/TCG/1CPU/256MiB (OOM128MiB), geen VM-FR/Nehalem/8GiB/32GiB of fysieke acceptatie. Volumeproeven gebruiken plaintext fixtures en tijdelijke RAM-sleutels. Native transport is geen verbonden Materiaaldesktop of veilige persistente productopslag.

**Volgende integratie:** Materiaal source7f8d220/bewijs1f36e0f, SUR1fix9115cb4, INP1SDK73c8e41, inputdaemons a337167 en compositor aa7df72 zijn afzonderlijke componenten. Appreview heeft stale-ready en C-entry gevonden; reparatie volgt vóór nativeGUI. Root bouwt de7-processenbootstrap; kernel_storage maakt echte init; native_ui_apps maakt QMP-bediening; acceptance_contract reviewt appadapters; git_build_audit voegt echte display-CI toe. Gedeelde ABI/build/grants/status blijven exclusief root.

RANDOM fc2e2a2+KDF-wispatch3299f28 en TLS34191bb hebben componentbewijs, nog geen native uitvoeringsclaim. security_release bouwt vier echte RNG-/revoke/cancel/replycases inclusief nieuwe legacy-STATUS ná terminale RNG-fout. PNG/JPEG9ae1c73 heeft gerichte oracle-/stackcorrecties; netwerkworker maakt de native observer, root bootstrap volgt. Browsera2085b9 documenteert concrete Chromium/Gecko-portblokkades zonder enginekeuze. Alle apps, NAS/providers, inference, veilige opslag, boot/PQC/updates/herstel, toegankelijkheid en duurproeven blijven verplicht.

### NEXT AGENT HANDOFF

- Controleer actuele HEAD/status/worktrees en actieve agents; historische blokken hieronder zijn geen actuele werkstand. Behoud alle eigen worktrees en wijzigingen. Integratie mag alleen als inhoud, passende checks, reviewbevindingen en remote-SHA aantoonbaar kloppen.
- Laatste bruikbare native displayimage: `/workspace/nieuw-os-worktrees/root-desktop/build/display-geometry-fixed/run/display-input-e6dc2a21c2/esp/`, originele PPMs/inputs/result/serial in dezelfde runmap. Zie exact geteste d528-bron en hashes hierboven. Geheugen15e: `/workspace/nieuw-os/build/full-release-15e8cfc-memory/{normal,nx,guard,peer,oom}/esp/`. Geen volledig releaseimage.
- Eerstvolgende uitvoering: voeg de concrete GUI-QMPrunner toe aan root-materiaal, leg de gedeelde bron/index plus checkpoint vast en publiceer de eigenbranch zonder lokale workercommits te verliezen; bouw op die exact bevroren bron `tools/build.py --kernel-feature native-system-materiaal-probe` met eigen `NOS_BUILD_DIR=build/materiaal-integration` en `NOS_TARGET_DIR=target/materiaal-integration`. Voer daarna echte QMPedit/save/heropen/F12/cleanSTOP uit, inspecteer echte screenshots en herhaal koude readonly-heropening. InitialSource/SYNC/Policy/CompFocusgates zijn vereist; APPStartupReady is geen paintReady. Native proeven nog niet uitgevoerd.
- Parallel: nativeRANDOM4cases en PNG/JPEGprobe krijgen uitsluitend eigen nieuwe bestanden. Root beheert feature-/SDK-/workspace-/buildwiring. App heeft geen rawDisplay/Input, compositor uitsluitend DISPLAY, bron uitsluitend INPUT, policy vasteIPCkanalen. Geen hostsimulatie als functionaliteit.
- CLI-push heeft HTTPS-auth401; de beschikbare GitHub-koppeling publiceert met lease/exactetree en fetch/ff-verificatie. Geen credentialinspectie/forcepush. `/tmp/nieuw-os-make-candidate.py` controleert nu ook gewijzigde staged bewijsarchieven tegen hun eigen SHA256SUMS; gebruik fail-fast shells.
-100 koude starts,24uur gecombineerde sessie, menselijk/hardwarebewijs en compleet testpakket blijven open. Geen uitvoering ná een harde sessiegrens beloven.

## Historische checkpoints

Alle onderstaande checkpointteksten beschrijven hun eigen bron en bewijs.

## K05 startcapabilityposities native hersteld, 32% — 29 oktober 2026

Branch `codex/supervisor-startup-slots`, basis `main`
`383636193faedc65b80c748c5bc4df28676dc2d3` (#54 gemerged na zes groene
workflows). Exact geteste bron `56b576b16da1e4503bc5f9167a476474a58c33cf`,
bronvingerafdruk `cccf0e3e7fcec439428208c4458a0194ec23c6120e1f03d96abe65b0711631b7`.
Latere wijzigingen zijn uitsluitend docs/bewijs.

**Hersteld:** P2 uit #51: een geldige raw `proc_spawn` met lege cap0 en
SEND in cap1 gaf cap1 foutief op startup-offset 56 in plaats van 64.
Verzamelen/installeren behouden nu beide indices; count blijft voor quota-
preflight. Geen rechten-, ABI-, SDK-interface- of afhankelijkheidswijziging.

**Uitgevoerd en geslaagd (exit 0):** `tools/test_supervisor.py`,5/5 echte
QEMU-runs: normal, crash, denied, parent-exit en quota. Normal bewijst één sparse
spawn, drie slot-/rechtencontroles, 24 echte IPC-antwoorden en herstart;
denied 21/21 gematchte controles, 11 geweigerde spawns. Engine-/framecleanup
in alle gevallen groen. Host: 40 capability-, 17 SDK-, 1 voorbeeldlib- en 7 harnastests;
voorbeeldclippy, root-/SDKfmt en diffcontrole groen. Oude kernel met nieuwe
native test faalde aantoonbaar na 15 seconden; gerichte herstelde normalcase groen.
Afzonderlijke agentreview: geen blockers; 7 harnastests onafhankelijk herhaald.
Bewijs en exacte commando's: [k05-startup-slots](verification/k05-startup-slots/README.md).

Omgeving: Rust 1.90.0, QEMU 10.0.13, expliciet pc-q35-8.2, qemu64, TCG, 1 CPU,
256 MiB, OVMF 2025.02-8+deb13u1 (4M). Geen fysieke hardware- of SMPclaim.
**Score 32% → 32%:** bestaand K05-ABI-randgeval hersteld; geen nieuw criterium.
Eind-CI/review moet groen zijn vóór de geschikte squash-merge.

Vorige geïntegreerde batch: #53 en #54 gemerged; #48 aantoonbaar vervangen en
gesloten. Alle acht B01/S08-reviewthreads opgelost. B01/S08/S06 hebben 27 echte
VM-runs op hun eigen bevroren bron 53e0d57 (historisch bewijs hieronder).

### NEXT AGENT HANDOFF

- Bron: `codex/supervisor-startup-slots`, geteste commit volledig hierboven.
  Controleer actuele HEAD/PR-merge; `git fetch origin main` vóór vervolgwerk.
- Afgerond: K05-slotfix en lokale vijfgevallenacceptatie; productiecode en
  bewijs gecommit/gepusht, geen losse rootwijzigingen. Integratie uitsluitend
  na actuele groene PR-checks; los daarna thread PRRT_kwDOVZC91s6xvSPj op.
- Volgende concrete productstap: B02, persistente geauthenticeerde sleutel-/
  volumeopslag boven services/keyring en services/volume; begin met record-
  formaat en negatieve manipulatie-/rollbacktests in een eigen branch.
- Open grenzen blijven: koude antirollback, requestercrash/reset-zonder-ACK-
  nativebewijs, TPM/fysieke RNG/hardware. Geen nieuw scorepunt toegekend.
- S09 historische lokale bron niet remote gevonden; controleer beschikbare
  lokale bronnen vóór een eventuele dubbele implementatie.

## Historische checkpoints

Onderstaande blokken zijn versiegebonden geschiedenis; actuele uitvoering
staat hierboven.

## B01-herstel en S08 geïntegreerd geverifieerd, 32% — 29 oktober 2026

Branch `codex/native-recovery-hardening`, PR [#54](https://github.com/simonvanieperen/nieuw-os/pull/54),
basis `main` `00e6af5` (de gecontroleerde squash van workflow-PR #53).
Exact geteste gezamenlijke bron: `53e0d57af99c9673c97bc900fae15b98e08c8ca3`;
bronvingerafdruk v3 `60edda7943e979a8e2c16caf6aa873a8b7d7535cd633e2a10f094d4b0bb6c00f`.
Latere checkpointwijzigingen betreffen uitsluitend documentatie/bewijs.

**Gerealiseerd:** B01 bewaart RCT/APT-toestand vanaf opstart over alle reseeds.
Herstel van dezelfde master verenigt intrekkingen; een gebruikte dienst
weigert een andere master. De native app bewijst oude-blobintrekking. De
broker begrenst reset-ACK en sluit outstanding I/O bij requester-exit af
(offline, gepinde quarantaine). Nieuwe afzonderlijke fixtures: werkelijk
onbeantwoorde kernel-RNG-aanvraag en reset/herinitialisatie van een antwoordende
driver op dezelfde pool met een nieuwe 32-byteaanvraag. S08-timing uit #48
is overgenomen met actuele main intact: deadline 500 PIT-ticks, diagnostiek
ook bij falen en latentie van `Device::poll`-foutcompletions.

**Uitgevoerd en geslaagd (exit 0):**
- `tools/test_keyring.py`: 5 echte VM-runs; 58 core/driver/capability-, 17 SDK-
  en 11 harnastests. Timeout: firmware 16 bytes, kernel 255+1 onbeantwoord,
  geen geleverde kernelbyte/sleutel, 1 timeout en 1 reset-ACK.
- `tools/test_storage_resilience.py`: 13 echte VM-runs, inclusief zes koude
  read-only recoveries; orakels, geen replay, overige voortgang en cleanup groen.
- `tools/test_volume.py`: 9 echte VM-runs met drie echte kill/recovery-paren,
  corruptieweigering; 28 volume-, 17 SDK- en 21 harnastests.
- Onafhankelijke hostverificatie: 325 Python-, 6 skill-, 238 root-Rust- en
  46 SDK-tests; root-/SDK-Clippy, fmt, syntax en voortgangscontrole groen.
- Afzonderlijke AI-review: geen blockers; vijf nieuwe keyringregressies bewezen
  rood vóór reparatie en groen erna. Twee extra hersteltegenvoorbeelden groen.

Omgeving: Rust 1.90.0, Debian QEMU 10.0.13, expliciet `pc-q35-8.2`,
qemu64/TCG/1 CPU/256 MiB, OVMF 2025.02-8+deb13u1 (4M). Alle exacte
commando's, rapporten, bron-/image-/loghashes en de ontwikkelfouten staan in
[verification/native-recovery-hardening/](verification/native-recovery-hardening/README.md).
GitHub eindhead-/reviewstatus: [PR #54](https://github.com/simonvanieperen/nieuw-os/pull/54);
samenvoegen pas na groene toepasselijke checks. #48 mag pas sluiten nadat de
vervangende integratie daadwerkelijk is opgenomen.

**Roadmap:** oorspronkelijk 32%, tijdelijk 30% wegens bevestigde P1-fouten;
na herstel en dezelfde native acceptatie weer **32%**. Geen nieuw criterium
of gewijzigde gewichten. B01-bewijs is aangevuld, oude bewijsbestanden blijven
historisch. Requestercrash en reset zonder ACK zijn niet native getest;
herstartbewijs geldt voor een antwoordend apparaat. Koude antirollback,
persistente sleutels/geauthenticeerde opslag (B02), TPM, fysieke RNG-kwaliteit
of hardware worden niet geclaimd. De eerste stille fixture blokkeerde ook
firmwareboot en is als mislukt vastgelegd, daarna gericht gecorrigeerd.

### NEXT AGENT HANDOFF

- Bron: `codex/native-recovery-hardening`; getest commit volledig hierboven;
  PR #54 bevat alle drie agentimplementaties en bewijs. Controleer actuele
  branch/HEAD/PR-merge; `git fetch origin main` vóór vervolgwerk.
- Afgerond: B01-R1/R2, S08-R, ENV en host/native verificatie; geen onafgemaakte
  productiecode in de hoofdwerkboom. De entropy-agentworktree bevat alleen
  lokale kopieën van inmiddels geïntegreerde R1-/featurewijzigingen.
- Open: B02, requestercrash/reset-zonder-ACK-nativebewijs. S09 staat alleen in
  een historisch lokaal checkpoint; geen remote branch/commit teruggevonden.
- Volgende concrete implementatiestap: B02, persistente geauthenticeerde
  sleutel-/volumeopslag boven `services/keyring` en `services/volume`; begin met
  versiegebonden recordformaat en negatieve manipulatie-/rollbacktests in een
  eigen branch. Houd secrets buiten kernel en gewone apps. Geen S09 dubbel
  implementeren zonder eerst eventuele lokale bron terug te vinden.

### Oudere historische checkpoints

Alle onderstaande gedateerde blokken zijn historische registraties van hun
expliciete bron. Oude woorden als “open”, acceptatie in afwachting en oude
percentages beschrijven die bron, niet de actuele toestand bovenaan.

## B01 native entropy en sleutelbeheer geaccepteerd, 32% — 29 oktober 2026

Branch `ccr-43832ef7-gi2qlz`, basis `main` `c92238b` (squash van
[PR #51](https://github.com/simonvanieperen/nieuw-os/pull/51), K05; GitHub
Actions daarop 6/6 groen inclusief de nieuwe job `k05-native-supervisor`).
Omgeving: Linux-container, Rust 1.90.0, QEMU 8.2.2 (Ubuntu), `pc-q35-8.2`,
OVMF 4M, virtio-rng-pci met host-`/dev/urandom`.

**Gerealiseerd (native, profiel `native-keyring`, ADR-043):**
- `drivers/virtio-rng`: eigen modern-virtio-pci-driver (00:06.0), één wachtrij,
  één apparaat-beschrijfbare buffer, gepinde DMA-pagina's; `virtio-core`
  parseert capabilities nu per apparaat-ID.
- `capability-core`: objecttype `Entropy` (recht `ENTROPY`=1024, zonder
  `DERIVE`/`TRANSFER`/`REVOKE`, quotum 4).
- Kernelbroker `kernel/src/m2/m3/keyring.rs`: syscalls `entropy_request`
  (0x60) en `entropy_poll` (0x61) met één aanvraag tegelijk, deadline 64 ticks,
  geen cryptografie, geen caching; gelabelde fixtures voor afwezig en
  vastgelopen apparaat. ABI-minor 1.4.
- `services/keyring` (`keyring-core`): SP 800-90B Repetition Count (6) en
  Adaptive Proportion (512/62), fast-key-erasure-DRBG op ChaCha20,
  HKDF-SHA-256-afleiding, XChaCha20-Poly1305 met AAD-binding, 16 sleutels per
  domein, permanente intrekking, herstelblob onder een gebruikersgehouden
  geheim. Alleen vastgepinde RustCrypto-crates met portable backends.
- SDK-runtime `native::entropy_request/entropy_wait`; `sdk/examples/keyring`
  (dienst en acceptatie-app, eigen linkerscript met 128 KiB code).
- Gate `tools/test_keyring.py`: 3 hostcontroles en 3 echte QEMU-runs; runner,
  capture, 7 harnastests; workflow `B01 entropy and key management acceptance`.

**Uitgevoerd en geslaagd:** zie [B01-ACCEPTANCE](verification/B01-ACCEPTANCE.md)
en `docs/verification/b01/`. Bronvingerafdruk `202a3e68…`. Kernelbewijs:
normal 2048 bytes geleverd, beide diensten READY, 17 sleutels, 4 manipulaties
en 3 verkeerde-sleutel/domeinpogingen geweigerd, herstel A→B met behoud van
intrekking; no-device Offline en beide diensten BLOCKED zonder sleutels; stuck
Repetition Count blokkeert beide diensten zonder sleutels.

**Roadmap rubric v1: 30% → 32%** (B01 +2).

**Open en niet bewezen:** GitHub Actions op de PR-head moet de gate herhalen;
geen fysieke RNG-kwaliteit, TPM, persistente sleutelopslag (B02), wachtwoord-
herstel of zijkanaalclaim. Nieuwe dependencies zijn in ADR-043 en
B01-ACCEPTANCE geïnventariseerd (licenties, buildscripts, SIMD-controle).

## K05 algemeen proces- en dienstbeheer native geaccepteerd, 30% — 29 oktober 2026

Branch `ccr-43832ef7-gi2qlz`, basis `main` `8b60354`. Omgeving: Linux-container,
Rust 1.90.0, QEMU 8.2.2 (Ubuntu), `pc-q35-8.2`, OVMF 4M.

**Gerealiseerd (native, profiel `native-supervisor`, ADR-042):**
- `capability-core`: objecttypen `Launch` (rechten `LAUNCH`=128) en `Process`
  (`OBSERVE`=256, `KILL`=512), zonder `DERIVE`/`TRANSFER`/`REVOKE`; `orphans()`
  voor kinderen van een gestorven starter; 9 nieuwe hosttests.
- Kernel `kernel/src/m2/m3/supervisor.rs`: syscalls `endpoint_create` (0x30),
  `proc_spawn` (0x50), `proc_kill` (0x51), `proc_status` (0x52); alle controles
  vóór toewijzing; dood via het bestaande `cleanup`-pad; wezen worden iteratief
  in dezelfde gebeurtenis beëindigd. ABI-minor 1.3.
- SDK-runtime: `native::endpoint_create/spawn/kill/status`, `LaunchImage`,
  `ChildProcess`, `ProcessStatus`; drie gewone programma's in
  `sdk/examples/supervisor` (supervisor, herstartbare worker, client).
- Gate `tools/test_supervisor.py`: 4 hostcontroles en 5 echte QEMU-runs
  (normal, crash, denied, parent-exit, quota); runner, capture, 7 harnastests;
  workflow `K05 process management acceptance`.

**Uitgevoerd en geslaagd:** zie [K05-ACCEPTANCE](verification/K05-ACCEPTANCE.md)
en `docs/verification/k05/`. Kernelbewijs per run: normal 3 spawns/2 kills/1
herstart/24 antwoorden; crash 1 echte `#UD`, herstart, 24 antwoorden; denied
17/17 foutcodes, 2 processen; parent-exit 2 wezen beëindigd zonder kills;
quota 15 spawns, 16e `EQUOTA` zonder frameverandering, 16 gelijktijdig, 15
kills. Elke run eindigt met lege engine en framebaseline.

**Roadmap rubric v1: 28% → 30%** (K05 +2).

**Open en niet bewezen:** GitHub Actions op de PR-head moet de gate herhalen;
geen SMP, fysieke hardware, gebruikersgeleverde images (A02) of beleid voor wie
supervisor mag zijn. Tijdens de eerste gate-run is de bronvingerafdruk
veranderd door een gelijktijdige README-wijziging onder `sdk/`; die run is
terecht als mislukt afgewezen en volledig herhaald.

## S06 geaccepteerd na merge en onafhankelijke herhaling, 28% — 29 oktober 2026

Branch `ccr-43832ef7-gi2qlz`, basis `main` `8b60354` (squash van
[PR #49](https://github.com/simonvanieperen/nieuw-os/pull/49)). Omgeving: een
tweede Linux-container met Rust 1.90.0, QEMU 8.2.2 (Ubuntu-pakket), `pc-q35-8.2`,
OVMF 4M. Deze container is niet de omgeving waarin PR #49 is gebouwd.

**Uitgevoerd en geslaagd:**
- `NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B -u tools/test_volume.py` op een
  worktree van PR-head `eadba7e`: exit 0. Hostcontroles 28 volume-core-, 14 SDK-
  en 18 Python-tests; daarna 9 echte QEMU-runs. Bronvingerafdruk `528dfe4c…`,
  gelijk aan `docs/verification/s06/volume-gate.json`; alle 9 `after_sha256`-
  imagehashes zijn byte-identiek aan het vastgelegde PR-bewijs (normal/recover
  `19e2fdb3…`, cut-data `48dcfcdd…`, cut-table `2d37c7c1…`, cut-txn `ede3a54c…`,
  corrupt `a19398f1…`).
- GitHub Actions op `eadba7e`: host-quality, qemu-smoke, k07-native,
  m6e-native-volume, s08-native-resilience en CodeRabbit alle geslaagd; de drie
  CodeRabbit-threads zijn opgelost. Codex-review kon niet draaien (gebruikslimiet).
- Baseline in deze container: `tools/preflight.py --strict`, build
  `milestone-m6c` en `run_m6c.py --case normal --smoke` exit 0. Zonder
  `NOS_QEMU_MACHINE=pc-q35-8.2` weigert QEMU 8.2.2 het standaardprofiel
  `pc-q35-10.0`; dat is de bestaande, vastgelegde ontwikkelkeuze, geen fout.

**Roadmap rubric v1: 26% → 28%** (S06 +2). Dit is de bewijsgrens van
[S06-ACCEPTANCE](verification/S06-ACCEPTANCE.md): opt-in profiel, emulator-kill
is geen stroomuitval, geen authenticatie/encryptie (B02), geen back-up/export
(S09), geen Materiaal-UI.

**Open:** PR [#48](https://github.com/simonvanieperen/nieuw-os/pull/48)
(S08-timing, 0 punten) heeft vijf open CodeRabbit-bevindingen en conflicteert na
deze merge met `main`; de eigenaarsessie is inactief. Volgende implementatiestap
richting 50%: K05 (algemeen proces- en dienstbeheer) als nieuw native profiel.

## S06 volume v2: native gate geslaagd, acceptatie wacht op review en CI, 26% — 29 oktober 2026

Branch `ccr-d3fe1610-gl4r8s`, PR [#49](https://github.com/simonvanieperen/nieuw-os/pull/49).
S06 staat als `71a0f9f` direct op `main` `092ee52` (de squash van #47, M6e).
Het huidige bewijs is van `ce181bf`, met de test- en doccorrecties uit de
tweede review. Uitgevoerd in een Linux-container met Rust 1.90.0, QEMU 8.2.2,
`pc-q35-8.2` en OVMF 4M uit het Ubuntu-pakket.

**Gerealiseerd (native):** volume v2 (`NOSVOL02`, IPC v6) in `services/volume`,
de native dienst en app, het kernelgeval `test-volume-cut-txn`, het v2-orakel,
de runner en de gate. Onderdelen:
- collecties (4 per domein, ook leesbaar met COLLECTION_STAT);
- metadata (`tags`, `kind`, collectie) met stempels uit een klok per domein;
- twee historische versies per object, met RESTORE en verwijderen ongedaan maken;
- PURGE;
- TRANSACT over maximaal 4 objecten in één commit.

**Uitgevoerd en geslaagd (op `ce181bf`, schone kloon):**
- `python3 -B -u tools/test_volume.py`, exit 0. Hostcontroles: 28 volume-core-,
  14 SDK- en 18 Python-tests. Daarna 9 echte QEMU-runs op bronvingerafdruk
  `528dfe4c…`. Guest-uitvoer en image-hashes zijn byte-gelijk aan de vorige run
  op `cb20c27`:
  - normal en koude read-only recovery (domeinklokken 57/27, generatie 84,
    62 chunks);
  - cut-data, cut-table en cut-txn, elk met echte hostkill en koude recovery.
    Bij cut-table zijn 4 en bij cut-txn 5 gescheurde sectoren van g+1
    aangetoond; bij cut-data is de andere kopie onaangeroerd;
  - corrupt geweigerd.
- `python3 -B -u tools/mutate_volume.py`, exit 0. Alle 24 mutanten zijn
  gevangen. Elke mutant schakelt één regel uit die de reviews noemden; het
  rapport staat in `verification/s06/volume-mutants.json`.
- `python3 -B tools/capture_volume.py`, exit 0, vastgelegd als
  `docs/verification/s06/`. Uitslagen:
  [S06-ACCEPTANCE](verification/S06-ACCEPTANCE.md); besluit: ADR-041.
- Een onafhankelijke review van de eerste S06-versie (drie AI-reviewers)
  vond vijf inhoudelijke punten:
  - STAT liet een domein de commits van het andere tellen;
  - collecties waren via IPC alleen schrijfbaar;
  - RESTORE verloor historie;
  - er was geen flush na schrijfbare recovery;
  - de gate kon geen gescheurde tabelwrite aantonen.

  Alle vijf zijn opgelost met regressietests.
- Een tweede review (vijf AI-reviewers; drie stopten door een gebruikslimiet)
  vond geen fout in de productiecode. Wel vond ze tests die zonder hun regel
  slaagden, ontbrekende negatieve tests voor COLLECTION_STAT en twee te ruime
  claims:
  - een read-only antwoord op een `Unknown` commit is voorlopig;
  - `Full` en de instantiebrede `Unknown` zijn zichtbaar over domeinen heen.

  Alles is gecorrigeerd in `ce181bf`. CodeRabbit vond drie kleine punten
  (formulering vingerafdruk, niet vastgelegde mutatiecontrole, U+202E in een
  testfixture); ook die zijn verwerkt.
- Bij de integratie bleek dat een eerdere lokale merge van de M6e-branch in de
  S06-branch M6e-inhoud van `main` had teruggezet. Dat ging om het bewijs, de
  voortgangsmetadata en de subpackage-binding uit `d9e4b6b`. Die inhoud is
  hersteld; alle andere M6e-commits zijn per commit gecontroleerd.

**Roadmap rubric v1: blijft 26%.** S06 (+2) wordt pas geaccepteerd als drie
dingen afgerond zijn:
- de derde reviewronde (RESTORE/flush, gate-bewijs, merge/docs en een
  controle van `ce181bf`), die nu loopt;
- GitHub Actions op de PR-head;
- de herbeoordeling door CodeRabbit.

**Open en niet bewezen:**
- Er zijn geen fysieke schijven, encryptie of authenticatie (B02).
- De Materiaal-interface draait niet native; S06 levert alleen de opslaglaag.
- Native I/O-foutinjectie ontbreekt. De flushfout na recovery en het
  `Unknown`-pad zijn alleen hostgetest.
- S08: de CI-job `s08-native-resilience` faalde op `c87f980` één keer in het
  bestaande geval `flush-error` (`USER_OR_PROTOCOL`) en slaagde bij de herhaling.
  Die commit veranderde alleen documentatie. De oorzaak is nog onbekend: de
  onderzoeksagent is door een onderbreking gestopt zonder rapport.
- S09 (back-up/herstel) staat in een aparte lokale branch op S06. De native
  S09-gate (8 QEMU-runs) is daar geslaagd, en de S06-gate als regressie ook
  (9/9, bron `06d46151…`). S09 krijgt een eigen PR na de merge van #49.

## M6e schaalbare documentopslag native geaccepteerd, 26% — 28 oktober 2026

Branch `ccr-d3fe1610-gl4r8s`, broncommit `d9e4b6b` (bevat `main` `78a96ae`), concept-PR
[#47](https://github.com/simonvanieperen/nieuw-os/pull/47). Uitgevoerd in een
Linux-container met Rust 1.90.0, QEMU 8.2.2, `pc-q35-8.2` en OVMF 4M uit het
Ubuntu-pakket.

**Gerealiseerd (native):**
- `services/volume` (`volume-core`, `no_std`, `forbid(unsafe_code)`): een
  copy-on-write-volume met twee per sector verzegelde tabelkopieën en
  4 KiB-chunks.
- SDK-blok-API en een ring-3 `volume-service` met twee geïsoleerde app-instanties
  (`sdk/examples/volume-docs`).
- Generieke kernelbroker `native-volume` zonder formaatkennis.
- Onafhankelijk Python-orakel (`tools/volume_disk.py`), runner met echte VM-kills,
  en de gate `tools/test_volume.py` met capture.

**Uitgevoerd en geslaagd:**
- `python3 -B -u tools/test_volume.py`, exit 0, met 3 hostcontroles (17 volume-core-,
  14 SDK- en 8 orakeltests) en 7 QEMU-runs op bron `14fa4d37…`.
- Reviewcorrectie (CodeRabbit, PR #47): iedere tabelsector draagt nu een
  pogingdigest, zodat sectoren van twee schrijfpogingen voor dezelfde generatie
  nooit samen een geldige kopie vormen. De nieuwe hosttest faalt zonder de
  correctie; het oude bewijs van bron `535d672e…` is vervangen.
- Tweede reviewronde: de gate schrijft zijn verdict pas na de afsluitende build
  (een mislukte build liet eerder `passed: true` achter) en ADR-040 noemt zijn
  eigenaar. Derde ronde: het verdict is nu ook gebonden aan de SHA-256 van de
  hosttestinputs onder `tests/`; vierde ronde: ook in subpackages, zoals
  `unittest discover` ze laadt. Gate herhaald en bewijs opnieuw vastgelegd.
- `python3 -B tools/capture_volume.py`, exit 0, naar `docs/verification/m6e/`.
- Regressie: `milestone-m6c` normal smoke.
- Uitslagen en grenzen: [M6E-ACCEPTANCE](verification/M6E-ACCEPTANCE.md).
  Contract: [M6E-SCALABLE-VOLUME](implementation/M6E-SCALABLE-VOLUME.md).
  Besluit: ADR-040.

**Roadmap rubric v1: 20% → 26%** (S04 +3, S05 +3).

**Open en niet bewezen:**
- GitHub Actions slaagden op `1037ee3`, `f9e22c1` en `bf0bf61` (5/5, inclusief
  `m6e-native-volume`); de herhaling op de eindhead moet nog slagen.
- Geen native I/O-foutinjectie voor dit profiel; I/O-fouten zijn alleen hostgetest.
- Geen fysieke schijf, encryptie of authenticatie (B02), back-up (S09),
  collecties/historie (S06) of migratie van M6c-volumes.

**Ook in deze opdracht:**
- PR #46 (N01-encoder) is na groene CI en een CodeRabbit-review zonder bevindingen
  squash-gemerged (`78a96ae`), zonder punt. De Codex-review kon niet draaien door
  het gebruikslimiet.
- Drie Claude-subagents zijn bevestigd gestart in eigen worktrees, met lokale
  QEMU: G1 (ring-3 display/invoer), K05 (procesbeheer) en N01 (virtio-net/UDP).
  G1 en K05-a (ring-3-supervisor met niet-overdraagbare procescapabilities,
  3 native gevallen volgens het agentrapport) zijn opgeleverd maar nog niet
  geïntegreerd, onafhankelijk herhaald of geaccepteerd; N01 loopt nog.
- S06 (collecties, metadata, revisiegeschiedenis, transacties) is in een aparte
  lokale worktree in ontwikkeling: hosttests groen, native gate nog niet gedraaid.

## N01 voorbereidend netwerkwerk — UDP-uitvoer zonder native NIC (28 oktober 2026)

Ontwikkelbranch `feature/n01-bounded-udp-ipv4-tx-20261028`, begonnen op main `6e12dcbbd4a37d404dd5a60bc2c8a8e32d2d187e`.
De bestaande `no_std`-protocolmodule is uitgebreid met een begrensde
Ethernet II/IPv4/UDP-transmitter. Deze code genereert uitsluitend bytes in een
caller-buffer: expliciete effectieve MTU, TTL-controle, IPv4/UDP-checksums,
zero-checksumcodering volgens RFC 768, zero padding en geen gedeeltelijke
bufferschrijfactie bij geweigerde input. De ingestelde source/destination
blijven data, geen gevalideerde app- of peerrechten. De bestaande parser wordt
als roundtrip-validator gebruikt; vaste onafhankelijk afgeleide UDP-vectoren
blijven het orakel, ook voor foutgevallen.

**Bewijsstatus:** broncode en aanvullende Rust-hosttests staan in deze branch.
De testcommando's `cargo +1.90.0 test/clippy/fmt` en cross-build naar
`x86_64-unknown-none` zijn in deze chatsessie **niet lokaal uitgevoerd**:
Rust en QEMU ontbreken in de beschikbare lokale omgeving. PR-CI kan dit
op een gecontroleerde runner uitvoeren; de uitkomst daarvan moet nog op de
exacte PR-head worden vastgelegd. Geen native NIC-/packettransmissie,
TCP-stack, service-, hardware- of QEMU-acceptatie geclaimd.

De zes aparte Codex-taken voor K05/S04/N01/U01/B03/H02 zijn op 28 oktober
wel aangemaakt, maar direct zonder code afgerond met
`No usable execution environment is available`. Dit is geen lopende
parallelle agentuitvoering. De hoofdchat bouwt onafhankelijk verder via
GitHub-bestanden. **Rubric v1 blijft 20/100 (20% → 20%); N01 is open.**

## Native A01 en S08 geïntegreerd, 20% — 28 oktober 2026

Op `main` commit `b9c7cf8f4e1804f0333e8fba8862941190224155` zijn twee
afzonderlijke functies nieuw opgenomen. [PR #31](https://github.com/simonvanieperen/nieuw-os/pull/31)
voegt de begrensde gewone native SDK-app en gebruikersdienst toe; een echte
QEMU-acceptatiecontrole toetst normale en geweigerde rechten, foutafhandeling,
proceslevenscyclus en resourcecleanup. [PR #34](https://github.com/simonvanieperen/nieuw-os/pull/34)
voegt de S08-resiliencegate toe met dertien aparte QEMU-runs, waaronder zes
koud gestarte recoveries van exact dezelfde bewaarde image. Read-only is
hardware-emulator-, driver-, capability- en userspace-afgedwongen, niet slechts
een gelijke hash voor en na de proef.

De acceptatiebewijzen staan in
[A01-ACCEPTANCE](verification/A01-ACCEPTANCE.md) en
[S08-ACCEPTANCE](verification/S08-ACCEPTANCE.md). Voor A01 en S08
zijn de gespecialiseerde gates groen; [main-CI 40761772907](https://github.com/simonvanieperen/nieuw-os/actions/runs/40761772907)
is eveneens volledig groen. De oorspronkelijke CodeRabbit-bevindingen zijn
gecorrigeerd; oude formele reviews zijn pas na oplossing en nieuwe groene
tests met onderbouwing gesloten. Codex-cloudagents konden niet starten door
het huidige gebruikslimiet, dus er is geen parallelle agentuitvoering geclaimd.

Roadmapacceptatie volgens bestaande rubric v1: **17% → 20%** (A01 +2, S08 +1).
Alle overige criteria blijven open. Zonder fysiek hardwarebewijs, native GUI,
breed app-ecosysteem, netwerkintegratie en updates is geen 50% geclaimd.

## Volledige repository-audit en integratie — 28 oktober 2026

De volledige huidige repository is opnieuw gecontroleerd op architectuurgrenzen,
duplicatie, build-/testconfiguratie, dependencies, gegenereerde rommel, secrets,
documentatie en open pull requests. De relevante inhoud van PR #16 is via
[PR #25](https://github.com/simonvanieperen/nieuw-os/pull/25) geïntegreerd en
PR #16 is daarna als vervangen gesloten. Een later verschenen, verouderde
documentatie-closeout PR #27 is eveneens inhoudelijk vervangen in plaats van
blind gemergd.

De integratie behoudt de eigen Rust-kernel en de bestaande microkernelachtige
scheiding: de Ethernet/IPv4/UDP-parser is een dependencyvrije
`no_std`-userspace/servicebibliotheek zonder driver-, socket- of capabilityrecht.
De DocumentSession-proef gebruikt bestaande catalogus-/IPC-capabilities en
verruimt kernel-, IPC- of opslagformaten niet. Bewuste duplicatie voor afzonderlijke
binaries/security-isolatie is niet samengevoegd.

GitHub Actions bevat nu een read-only CI met gepinde officiële actions. Na de
auditmerge werd op exact dezelfde Git-tree één niet-deterministische normale
M6c-QEMU-fout waargenomen: de client ontving een geldige reply, maar de mutatie
bereikte de bestaande `Unknown`-route doordat de completion niet binnen de
20-tick brokerdeadline werd waargenomen. De onmiddellijke rerun op dezelfde tree
slaagde. [PR #26](https://github.com/simonvanieperen/nieuw-os/pull/26) verhoogt
uitsluitend de M6c-acceptatiebrokerdeadline naar 64 ticks; faultfixtures blijven
completionwaarneming tot de echte deadline onderdrukken, zodat timeout/Unknown
niet wordt weggefilterd of geretryd.

De definitieve push-run op `main`,
[run 40741321766](https://github.com/simonvanieperen/nieuw-os/actions/runs/40741321766),
is volledig groen. `host-quality` slaagt voor Python/DocumentSession, root Rust,
Clippy/rustfmt, SDK, netwerkparser inclusief native cross-build, quantum,
productprototype, NAS-tests, native releasebuild en voortgangs-/clean-treecontrole.
`qemu-smoke` slaagt voor strict preflight, baseline M6c en DocumentSession
normal plus koude recovery in afzonderlijke VM-invocaties.

CodeRabbit leverde op PR #16 vijf concrete reviewbevindingen; de geldige punten
zijn in de integratie hersteld. Op PR #26 was de CodeRabbit-review bij merge nog
asynchroon in uitvoering en waren geen inhoudelijke bevindingen of reviewthreads
gepubliceerd; de volledige CI was wel groen. Codex code/security review op de
verouderde PR #27 kon niet starten doordat het gekoppelde reviewquotum bereikt
was; PR #27 is daarom niet als goedgekeurd beschouwd en juist vervangen.

Repositoryonderhoud is aangescherpt met `CONTRIBUTING.md`, PR-/bugtemplates,
beperkte maandelijkse Dependabot-voorstellen, binary-attributen voor PDF/GZIP,
en gitignore-regels voor gegenereerde SDK-/netwerk-targets. De bestaande
main-ruleset beschermt verwijdering, non-fast-forward en lineaire geschiedenis
en vereist PR's plus opgeloste reviewthreads. Verplichte CI-statuschecks en
GitHub-securityinstellingen konden via de beschikbare integratie niet worden
gewijzigd of volledig uitgelezen; dit blijft een repository-instellingspunt.

**Roadmap: 14% → 14%.** De audit maakt bestaande functionaliteit aantoonbaar
schoner en beter bewaakt, maar DocumentSession is nog een begrensde headless
proef, de netwerkparser is nog geen native netwerkdienst en de Security Floor/
provider-/premium-UX-eisen zijn nog geen volledige native acceptatie. Daarom is
geen nieuw vast criterium toegekend.

## Architectuurbesluit: Security Floor, opaque provideropslag en premium UX — 28 oktober 2026

De nieuwe gebruikerskeuze is als documentatie-/architectuurwijziging vastgelegd.
Privacy is configureerbaar, maar de **Security Floor** niet: internet,
vrijwillige telemetrie, microfoon, externe opslag of andere expliciete
gegevensdeling verruimt alleen de betreffende capability/gegevensstroom en
schakelt kernel-/procesisolatie, sleutelisolatie, geheugenbescherming,
pakket-/update-authenticiteit of bescherming van niet-gedeelde data niet uit.

Het absolute leverancierscloudverbod uit privacycontract revisie 3 is vervangen
door revisie 4. Persoonlijke NAS blijft een eersteklas route. Daarnaast mag een
gebruiker een algemene externe opslagprovider expliciet kiezen als
**onbetrouwbare opaque ciphertextbackend**. De provider krijgt geen
Nieuw-OS-inhoudssleutel, documentplaintext of gevoelige manifestplaintext.
Geen stille providerfallback of key escrow. Metadata, beschikbaarheid,
providerdelete en endpointcompromittering blijven expliciete rest-risico's.

[ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md)
legt het toekomstige objectformaat, key hierarchy, crypto-agility,
rollback-/replay-eisen, multi-providergrens en EO-acceptatietests vast. Er is
geen zelfbedachte cryptografische primitief gekozen en geen claim dat een
provider “absoluut niet kan kraken”. P05/EO-tests zijn nog open.

Ook de productambitie is aangescherpt: P3/P4/M7 moet een eigen premium
productiviteitsfamilie opleveren met Materiaal als objectmodel, één designsysteem,
contextuele gereedschappen, hoogwaardige typografie/motion, toekomstige
gedeelde native renderlaag en geteste interoperabiliteit. Geen Office-ribbon-
of bestaande-OS-kopie als einddoel. Het huidige browserprototype blijft alleen
ontwerpgereedschap.

Privacyuitleg en capabilities zijn gescheiden. Educatieve privacyuitleg mag
globaal of per app uit; dat wijzigt geen grants. In focus/gaming worden
niet-urgente educatieve meldingen onderdrukt, terwijl een nieuwe gevoelige
functie nog steeds een noodzakelijke grant kan vereisen.

Vastgelegd in ADR-035 t/m ADR-038. Er is in deze stap **geen native runtime-,
crypto-, provider- of UI-implementatie uitgevoerd en geen nieuw
acceptatiecriterium behaald**. De vaste roadmap blijft **14% → 14%**.


## Markt- en consumentenonderzoek — 28 oktober 2026

Twee aanvullende onderzoeken zijn als strategische onderbouwing vertaald naar
`docs/research/`: een markt-/concurrentieanalyse en een wetenschappelijke
consumentenadoptieverkenning. Nieuw is vooral de externe onderbouwing van
overstapfrictie, compatibiliteit, herstel, ondersteuning, Europese alternatieven
en 90-/180-dagenbehoud als latere marktvalidatie. Materiaal, de eigen kernel,
lokale AI, beperkte apprechten en de privacyrichting waren al projectbesluiten.

De synthese maakt expliciet dat doelgroep, kleurpaletten, prestatiedoelen,
steekproefgroottes, retentiedrempels, verkoopkanalen en verdienmodellen nog
onderzoekshypothesen zijn. Zij wijzigen `PRODUCT-SPEC.md`, capabilities,
kernel-ABI of privacycontract niet zonder afzonderlijk besluit.

De inmiddels op `main` opgenomen privacybrowser-aanvulling (#18) blijft
volledig behouden en is een aangrenzend productonderzoek, geen conflict met
deze markt-/adoptiesynthese.

Aanvullend is [CONSUMER-VALUE-2026-10-28.md](research/CONSUMER-VALUE-2026-10-28.md)
opgenomen als diepere productvertaling. Die legt de te onderzoeken kernbelofte
“materiaal terugvinden, ermee verdergaan en controle houden over wijzigingen en
toegang” vast en definieert DV-01 als latere geïntegreerde Materiaal-toets met
twee gereedschappen, bevestigd opslaan, herstel en revocation. P2 blijft
bevroren; DV-01 is geen reeds bewezen product- of native eigenschap.

De consumentenwaarde-verkenning bevat nu ook de ontbrekende bewijsgrenzen rond
ervaren competentie, desktopgewenning, tijdelijke opslag, local-first,
regelbare meldingen en ervaren werklast. PRX1–PRX4 vertalen dit naar vroege
taakobservatie, een DV-01-prototypeproef, boodschaptest en vierweekse veldproef.
Er zijn nog geen deelnemers onderzocht; de 90-/180-dagenmarktvalidatie blijft
een afzonderlijke latere stap.

Geen broncode, native runtime of acceptatiebewijs is in deze documentatiestap
gewijzigd of uitgevoerd. De technische momentopname in de oorspronkelijke
rapporten kan daarom ouder zijn dan deze actuele projectstatus. Roadmap:
**14% → 14%**.

## Actuele vervolgacceptatie — 28 oktober 2026

**Zes extra echte VM-runs slagen op `dbd35b04d742b18d66b40fdd65d8a9b900656cac`**,
bronfingerprint v3 `c5f3d9316433ba09ba4b886985ffa8b28643d67c72a9ebb0e2b0136362dc7a9f`.
G0 draait met vijf echte invoerbeelden en nieuwe atomische capture; SDK/M6c
en M6b bewaren en lezen terug in elk twee afzonderlijke VM's; de qsim-proef
draait in twee geïsoleerde gebruikersprocessen. Alle zes ruwe exits zijn 33.
Eigen builds, targets, volumes, QMP-pipes en logs voorkomen gedeelde output.
De [onafhankelijke gecombineerde controle](verification/parallel-followup-integration/README.md)
herhasht 216 unieke bestanden en bevestigt alle zes bron- en artifactketens.

- [G0: bron, opdrachten, vijf echte beelden en capture](verification/parallel-followup-g0/README.md).
- [SDK/catalogus: normale opslag en koude herstart](verification/parallel-followup-sdk-m6c/README.md).
- [Quantum: actuele strict run en vergelijking van werkelijke binaries](verification/parallel-followup-qsim/README.md).
- [M6b en gerichte hostgrenzen](verification/parallel-followup-root/README.md).

Op `f52ef0a6` zijn de G07-documentsessie, I08-jobcodec en B07-capture al
gepubliceerd. De documentsessie bewaart geladen revisie en lokaal concept bij
CAS-conflicten of onzekere opslag. Deze appbibliotheek staat buiten de kernel.
De 39 SDK-hosttests en native librarybuilds slagen; I10 vond geen blocker
binnen haar reviewscope. De canonieke 256-byte jobcodec heeft 13 tests en een
gerichte onafhankelijke review; dispatch en gebruikersjobs blijven open.

G09 vond een dynamische outputlijst in de capture. B09 gebruikt een vaste
lijst vóór indexering én vóór publicatie. De negatieve test faalde op de oude
bron en slaagt na reparatie; onafhankelijke herhaling en alle 34 grafische
hosttestmethoden slagen. De verse B08-VM gebruikt de gerepareerde capture;
alle originele inputs en lossless PNG's zijn opnieuw gecontroleerd.
[Documentreview](verification/parallel-i10/README.md),
[capturereparatie](verification/parallel-b09/README.md),
[hercontrole](verification/parallel-g09/README.md).

De [gerichte PR14-behoudcontrole](verification/parallel-b10/README.md)
bevestigt dat de zeven eerdere reviewvragen in de integratiebron zijn opgelost
en dat alle bestaande bestandspaden behouden zijn. De volledige bomen zijn
vergeleken omdat de GitHub-compare slechts een deel van de paden retourneerde.

De acht concrete CodeRabbit-bevindingen zijn afgehandeld met bron-/bewijslinks;
de actuele vervolgbeoordeling en PR-status staan op GitHub. De eerdere
volledige acceptatie van 80 VM's hieronder behoudt haar oorspronkelijke bron.
Bestaande kernel-, dienst- en probebron bleef tijdens deze vervolgstap gelijk.
De nieuwe qsim-kernel en app zijn byte-identiek; twaalf gewijzigde loaderbytes
zijn verklaard als PE-tijdstempels/buildidentifier, met originele artifacts.

G11 bouwt nu de documentsessie in de bestaande headless gebruikersproef om
ook openen, bewerken, bewaren en heropenen via deze bibliotheek native te
controleren. Die uitvoering is nog niet geaccepteerd. G1/Materiaal blijft bij
de bestaande eigenaar; publicatie van diens broncheckpoint is de kleinste
integratieafhankelijkheid. Geen grafische documentflow of algemene appstarter
wordt uit de huidige proeven afgeleid. Roadmap: **14% → 14%**.

## Gecombineerde native acceptatie — 28 oktober 2026

Uitvoeringsbron `3e41bbf93822f7044c6020259f4a752925e1966d`, branch
`codex/parallel-native-foundation-20261028`, PR#15 boven de behouden PR#14.
Lokale/GitHub-commit identiek vóór gates; bronfingerprint v3
`307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60` bleef gelijk.
Alle vier werkelijke agentplaatsen zijn benut en hergebruikt met eigen
worktrees/builds/targets/VMs/logs. G1 en de andere M6a-opdracht blijven bij hun
bestaande eigenaars.

**80 geaccepteerde native VM-runs:** M6b32 groepen/28 VM, SDK/M6c35/31,
M3-regressie22/19, native quantum1 en bevestigde G0-pipe1. Volledige M6b- en
M6c-captures slagen. Beide normale SDK-catalogusruns bewaren sequence10 en
lezen na een nieuwe koude VM dezelfde bytes terug zonder writes. Twee quantum-
userprocessen voeren bestaande referentieberekeningen uit, met64 KiB-arena's,
4 timerIRQs/505 syscalls en96 werkelijk vrijgemaakte frames. G0 verwerkt echte
invoer met per-toetsbeelden en exact backspaceherstel. Geen nieuwe syscall,
capabilityverruiming, on-diskformaat of externecrate.

De gecombineerde artifactcontrole bevestigt80 unieke VMs en616 directe
hashvergelijkingen. De oorspronkelijke G0-run is uitgesloten wegens een
later afwijkend bewaard serialartifact; oorzaak onbekend, niets hersteld.
De verse private pipe-run en alle 15artifacts zijn herhaaldelijk gecontroleerd.
Een aparte UNIX-socketprobe faalde op sandbox-EPERM; de bruikbare pipe-route
is de native geverifieerde variant. Oude fouten en ruwe resultaten blijven
behouden. Alle bron-/bewijs-/reproduceerdetails staan in
[PARALLEL-NATIVE-ACCEPTANCE.md](verification/PARALLEL-NATIVE-ACCEPTANCE.md).

Concrete CodeRabbit- en gerichte AI-bevindingen zijn verwerkt voor QMP-
deadlines/ID's, screenshots, ROMpin, WRITE-timeout-oracle, standalone preflight
en prefixed FPU-instructies. De nieuwe PR#15-review op3e41 is aangevraagd;
definitieve review/mergestatus wordt op GitHub bijgehouden. Geen veiligheids-
of fysieke hardwareclaim door de AI-review. QEMU 7.2.22/pc-q35-7.2 is expliciet
vastgelegd en verschilt van het historische10.0-referentieprofiel.

Voortgang **14% → 14%**; vaste criteria/gewichten behouden. A01 algemene appstart,
U03 native Materiaal, S04/S05 bruikbare documentgrootte/ruimtehergebruik en L02
externe native quantumjobs blijven open. De catalogusclient+SDK bestaan nu;
G07 levert een documentsessie, I08 een jobcodec en B07 een strikte G0-capture.
Deze geïntegreerde bouwblokken zijn nog geen volledige productfunctie.
Zie `WORK-QUEUE.md` voor de kleine actuele taakverdeling en blokkades.

### Directe nieuwe native bewijslinks

- G0: [commando, raw33 en alle controles](verification/parallel-final-g0-confirmed/pipe/result.json), [ruwe serial](verification/parallel-final-g0-confirmed/pipe/serial.txt).
- SDK/catalogus: [opdrachten en exitstatussen](verification/parallel-final-sdk-m6c/driver.json), [alle31 ruwe VM-records](verification/parallel-final-sdk-m6c/guest-runs.json).
- Quantum: [opdrachten en bronbinding](verification/parallel-final-qsim/i05-run-checks.json), [native raw33-resultaat](verification/parallel-final-qsim/result.json), [ruwe serial](verification/parallel-final-qsim/serial.txt).

Alle drie nieuwe bundels registreren werkelijk QEMU7.2.22/pc-q35-7.2.
Historische M6c-consolelogs van andere QEMU-profielen zijn hiervoor niet gebruikt.

## Historische faseverslagen

De verslagen hieronder bewaren hun toenmalige bron- en bewijsgrenzen.

## Doorbouw — opslagtimeouts en native grafische basis

Integratiecheckpoint: `main` op `391f02e` (M6C/D, PR #13) is via normale merge
opgenomen. Alle catalogus-/M6d-features, `archive_destination` met direct-child-
grens en Windows `wait(timeout=10)` blijven behouden. De andere chat beheert
haar afzonderlijke M6a-observerbranch; die bron is hier niet overschreven.
Oude captures houden hun oorspronkelijke bronbinding. Gecombineerde eindgates
zijn op dit checkpoint nog niet uitgevoerd. Voortgangsmeter: **14% → 14%**;
de lopende G0- en timeoutstappen sluiten geen nieuw volledig criterium.

28 oktober 2026, vanaf `7b4945f` op `codex/m6b-storage-errors`. De OS-bouw loopt
door; het vorige checkpoint is geen voltooid OS. K08 voegt kerneldeadlines,
resetquarantaine en lokaal afgebroken polls toe. De dienst sluit bij eigen
polluitputting zijn enige blokcapability via de bestaande ABI; de kernel kan
zo echte autoriteitsbeëindiging waarnemen en pending DMA veilig annuleren.
Vijf gerichte echte VM-proeven op bron `9c0c434` slagen met raw-exit 33:
WRITE-/FLUSH-deadline, gemaskeerde resetack/quarantaine en lokaal uitgeputte
WRITE-/FLUSH-polls. De cancelproeven sluiten de blokcap vóór de kerneldeadline;
de resetproef houdt de pool vast. Het harnas heeft 33 geslaagde hosttests.
Afzonderlijk bewijs: `verification/m6b-timeouts-directed/`. De uitgebreide
volledige 32-groepen-/28-VM-poort volgt na de native grafische bronfreeze.

G09 bouwt parallel een opt-in native grafische basis volgens ADR-033 en
`implementation/GRAPHICS-G0.md`: eigen framebufferuitvoer en PS/2-invoer,
geen browser, volledige desktop of opslagadapter. De pure `ui-core` is aanwezig
met tien geslaagde hosttests; de native GOP-/kernelintegratie staat klaar voor
de eerste hardwareproef. Elf bootprotocoltests, drie grafische harnastests,
fmt/Clippy/no_std slagen. Standaard BootInfo 1.1 blijft behouden. Deze
De eerste native G0-run op `f2c4055` slaagt: raw 33, eigen post-EBS/M2-uitvoer,
800×600-framebuffer, echte QMP-toetsen a/b/backspace/c en aantoonbaar alleen
gewijzigde pixels in de invoerregel. Backspace herstelt exact het eerdere beeld.
Dit bewijst RAM-invoer en grafische bootstrap, geen compositor of opslagUI.
Het harnas wordt nog aangescherpt met exacte marker-/geometriecontrole;
nieuwe gecombineerde eindgates volgen. Bewijs: `verification/graphics-g0-first/`.
De eerste G0-start faalde vóór kernelboot door een
ontbrekende VGA-ROM-lookup. De reeds geïnstalleerde officiële SeaBIOS-ROM is nu
uitsluitend in de projectlokale QEMU-datamap gelinkt; de mislukte run blijft
failed bewaard.

## Hervatting — native M6b-opslagfouten, 28 oktober 2026

Basis `1f24f994bff1b2c7cb0a026b600c5c4948140cfe`, branch
`codex/m6b-storage-errors`. De lokale checkout liep achter op GitHub; na fetch
is deze actuele M6b-basis gekozen. De voorlopige, dubbele M6a-planstap op
`codex/m6a-write-flush-core` is niet geïntegreerd en is geen vervolg op M6b.

Afgeronde stap op uitvoeringsbron `592dccfb329c05107866b18f7187e8b340ccab98`:
de observerpanic bij een terminale WRITE-/FLUSH-fout is gerepareerd. De observer
volgt de echte brokerfout naar de ring-3-dienst, verwacht een canoniek Unknown-
antwoord en bevestigt/commit de mutatie niet. Latere GET/PUT/DELETE en dezelfde
PUT opnieuw geven Unknown zonder nieuwe I/O; verkeerde domeintoegang blijft
Access. Geen nieuwe rechten, syscall, dependency, unsafe of schijfformaat.
De foutinjectie
vervangt uitsluitend een lokale statussnapshot ná echte devicecompletion;
geen claim van een fysieke schijffout of stroomuitval. Eisen en markers staan
in `implementation/M6B-OBJECT-STORAGE.md`.

`python3 -B -u tools/test_m6b.py` en
`python3 -B tools/capture_m6b.py --destination docs/verification/m6b-storage-errors`
slagen (exit 0): **22 groepen / 18 echte VM-runs**. Daarin zitten beide nieuwe
foutcases met negen werkelijk ontvangen Unknown-replies, nul mutatie-ACKs,
blijvend sequence 0 in de levende dienst en precies één werkelijk geschreven
record op schijf. Beide koude vervolgruns herstellen sequence 1 zonder writes.
De overige CRUD-/domein-/full-/corrupt-/kill-/recoverycases en M1-regressiestart
slagen eveneens. Hostlaag: 22 Rust-objecttests en 31 harnastests. De gate telt
acht koude recoveries en twee echte externe QEMU-kills.

Vooraf slagen twee gerichte foutVMs; zij worden niet bij de gate opgeteld.
Targeted rustfmt/diffcontrole slagen. Onafhankelijke AI-bronreview heeft geen
blockers; extra negatieve markerproeven slagen. Checkpoint:
`verification/m6b-storage-errors-checkpoint/`. Volledige ruwe gate/capture:
`verification/m6b-storage-errors/`. Exacte aanvullende commando's, review,
hashaudit en normale eindstart: `verification/m6b-errors-review/`.
De verifier controleerde alle 18 gastrecords, 157 artifacthashes, 36 ruwe
serial-/stderrbestanden en acht koude herstelkoppelingen; geen blockers.
De normale `milestone-m6b`-build en een extra smoke-start slagen (exit 0/raw 33);
kernel en loader zijn bytegelijk aan de normale eindgate. Dit is AI-verificatie,
geen menselijke audit. Concept-PR: [#14](https://github.com/simonvanieperen/nieuw-os/pull/14).

Ontwikkelomgeving: projectlokale Rust 1.90.0, QEMU 10.0.13 en OVMF
2025.02-8+deb13u1 uit officiële Debian-pakketten; geen systeembrede installatie.
Dit is een andere ontwikkelvariant dan het historische Linuxbewijs hieronder.

Grenzen: geen nieuwe volledige M1–M6a-regressiesuite, browser-/NAS-/quantumproef
of fysieke-hardwaretest. Echte backend-IOERR, gedeeltelijke write, timeout,
reset-timeout en uitputting van de eigen poll-LIMIT van de dienst zijn niet
nieuw native geïnjecteerd. Die laatste lokale timeout kan nog van de verwachte
observerstatus afwijken als geen terminale brokerfout is teruggegeven. De
snapshotproeven sluiten dat afzonderlijke randgeval niet. Volgende opslagstap:
deze fout-/timeoutgrenzen, daarna ruimtehergebruik en de verdere productadapter;
native netwerk/desktop en updateketen blijven afzonderlijke bouwtaken.
## Reviewgrenzen en historische auditbron — 28 oktober 2026

Basis `322c553f58f11843aab7cf895bcba05564a9c7ac`, branch
`codex/integrate-m6d-main`. De parallel gepubliceerde recorderreparatie en vier
regressietests zijn behouden; dubbel lokaal herstelwerk is apart bewaard.
De documentatie begrenst receiptbehoud nu expliciet tot beëindigde childprocessen:
de bestaande timeout werpt vóór registratie `TimeoutExpired`.

De oorspronkelijke A04-auditor is toegevoegd als `a04_m6b_audit.original.py`,
exact gebonden aan commit `7484d87`, zijn 22802 bytes en de hash van het oude
auditrecord. De zes historische methoderegels blijven onderscheiden van het
latere script met zeven regels; geen volledige rawaudit is herhaald.
M6D-CLOSEOUT beschrijft de huidige R09-bestandsnaam en de vereiste canonieke
oorspronkelijke uitvoeringsroot. De M6c-queue heeft een expliciet historische
subkop. Ongewijzigde uitvoeringsrecords behouden hun eigen hashes.

Exacte uitgevoerde host-/tekstcontroles, exitstatussen en bytevergelijkingen:
[review-provenance.json](verification/m6d-closeout/review-provenance.json).
Geen nieuwe build of VM-run. Native bronbinding en de vaste score blijven
**14% → 14%**. Het aanvullende native scenario dat een normale pending-poll
en het latere clientresultaat samen observeert, blijft open; de bestaande
terminale foutproeven vervangen die gerichte observatie niet.
De uiteindelijke review- en squashmergestatus staat in PR #13.

## Volledige PR-review: recorder en documentatie — 28 oktober 2026

Broncheckpoint `7113f4ddcf3af34918f89f3869aa3889694d47b2`, branch
`codex/integrate-m6d-main`. Drie aanvullende CodeRabbit-bevindingen uit de
volledige review op `4c8ae3c` zijn gericht verwerkt. De ontwikkelhandleiding
kiest voor iedere M6c/M6d- en M6b-capture een nieuwe expliciete UUID-bestemming;
de roadmap vermeldt de bestaande M6d-acceptatie van 35 groepen / 31 VM,
M6b-regressie 18 / 14 en S07, met behoud van de bewijsgrenzen.

De historische capture-recorder schreef fouten en gewijzigde inputs wel in
zijn receipt, maar eindigde daarna ten onrechte met status 0. De recorder
bewaart nu eerst receipt en uitvoer en retourneert daarna status 1 bij
een beëindigd subproces met niet-nul exitstatus of gewijzigde inputs. Succes met
ongewijzigde inputs houdt status 0. De grens van 60 seconden blijft gelden:
bij een timeout werpt `subprocess.run` eerst `TimeoutExpired`, voordat deze
recorder een receipt of uitvoerbestanden schrijft. De gerichte proeven hieronder
testen beëindigde subprocessen, geen timeoutregistratie.

Werkelijk uitgevoerd: twee gerichte hosttestinvocaties met
`python3 -B -m unittest discover -s tests -p test_capture_check_recorder.py -v`.
De identieke vier scenario's controleren succes, een falend subproces,
gewijzigde input en beide fouten samen. Vóór herstel slaagt één scenario en
falen drie op de onterechte recorderstatus 0 (suite-exit 1); na herstel slagen
**4/4** (suite-exit 0). Receiptvelden, inputhashes en ruwe output blijven ook
bij afwijzing controleerbaar. Exacte executable, opdrachten, cwd, bronhashes,
uitvoer en exitstatussen staan in
[capture-check-regressions.json](verification/m6c-review/integration-followup/capture-check-regressions.json).

De oorspronkelijke 29-testsuite, provenance en drie commandorecords blijven
bytegelijk en aan de historische recorder gebonden. De actuele recorder en
nieuwe regressiecontrole hebben afzonderlijke hashes in het bijbehorende
manifest. Deze vier tests worden niet bij de historische 29 of Windows-173
opgeteld. De twee documentwijzigingen zijn tegen de actuele captureargumenten,
links en opgeslagen aantallen gecontroleerd; Bash-syntax en UUID-fragmenten
zijn lokaal gecontroleerd, PowerShell alleen statisch.

De 144 native/build/runnerinputs en het gepubliceerde Windowsbewijs blijven
ongewijzigd. Geen nieuwe native build, VM-uitvoering of extra productpunt.
Voortgang **14% → 14%**; criteria en gewichten blijven gelijk.
`python3 tools/progress.py --write` en daarna `--check` zijn uitgevoerd met
exit 0. De definitieve externe review- en squashmergestatus staat in
[PR #13](https://github.com/simonvanieperen/nieuw-os/pull/13).

## PR-integratie en qsim-herstel — 28 oktober 2026

Broncheckpoint `8d756ed4b85585e9878f440ea537c7875b02da45`, branch
`codex/integrate-m6d-main`. De eerdere squash van PR #12 naar main (`1f24f994`)
was inhoudelijk aanwezig, maar ontbrak als ouder van de integratiebranch.
Commit `e089e929` verbindt beide geschiedenissen zonder bestandswijzigingen.
Het daarna gepubliceerde checkpoint `c8945c6` bewaart die verbinding en voegt
het hieronder beschreven Windowsbewijs toe. Ook het latere Copilot-checkpoint
`f835a061` en zijn integratieverslag blijven behouden. De twee oude M5-logs blijven weg.

Een nog ontbrekende CodeRabbit-reparatie uit PR #2 is gericht overgenomen:
de qsim-JSON-parser accepteert alleen objecten voor requests en gates.
De oude parser faalt op twee nieuwe regressietests; na herstel slagen
**27 hosttests**, `cargo fmt --all -- --check` en
`cargo clippy --offline --locked --workspace --all-targets -- -D warnings`.
Lockfile, dependencies en quantumcore blijven gelijk. Exacte commando's,
bronbinding en ruwe uitvoer: [qsim-request-validation](verification/qsim-request-validation/README.md).

De 144 native/build/runnerinputs zijn bytegelijk aan Windows-uitvoeringsbron
`326d2f6`. Deze integratiestap controleert gepubliceerd bewijs en voert geen
nieuwe native build of VM-test uit. De historische M6b-observerbeperking en
de twee ontbrekende individuele M1-QEMU-hashes blijven expliciet. Voortgang
**14% → 14%**: de hostparserfix en integratie voegen geen native criterium toe.
`python3 tools/progress.py --write` en daarna `--check` zijn uitgevoerd met
exit 0; zij controleren de berekening, bewijsverwijzingen en weergave.
Reviewaanvulling `82c94221f4e50fbcc5ce3a6200372f94f1d63efc` behandelt vijf
CodeRabbit-punten: bytebehoud van de historische archieven, brongebonden
M6c-opdrachten, de actuele R08-receiptnaam, de latere A04-scriptversie en de
actuele FINAL-REVIEW-hash met vorige/tussenliggende commits. Alleen de twee
gecorrigeerde documenthashes veranderen in de actuele M6d-index; historische
test- en auditreceipts blijven bytegelijk. De CodeRabbit-deelclaim dat het
A04-sectorbestand als ontbrekend stond vermeld, bleek niet reproduceerbaar;
het bestand stond al in Git en in de overdrachtstabel.

Een afzonderlijke lokale Git-controle met `core.autocrlf=true` en
`core.eol=crlf` bevestigt `text: unset` voor alle 26 betrokken archiefpaden.
Acht CRLF-proefbestanden (beide archieven, direct/genest, JSON/TXT) blijven
bytegelijk in de index en na `git checkout-index --all --force`.
De documenthashes, historische hashherkomst en receiptverwijzingen zijn
eveneens gecontroleerd; geen native uitvoering of nieuwe acceptatiepunten.

Dit verslag legt de bron en uitvoering van de integratie vast. De definitieve
review- en squashmergestatus staat in
[PR #13](https://github.com/simonvanieperen/nieuw-os/pull/13).

## Main- en remote-PR-integratie — conflictresolutie 28 oktober 2026

De actuele `main` (`1f24f994`) is samengevoegd in mergecommit
`6603a6bc`, vanaf gemeenschappelijke basis `5a58a743` op integratiebron
`1068e89c`. De M6c/M6d-
implementatie bleef behouden. De gedeelde kernel-, capability-, virtio- en
dienstbronnen zijn bytegelijk aan zowel de M6d-uitvoeringsbron `88e6c04` als de
overlappende bron op `main`; de M6c-selectie is apart behouden. De twee
eerder uitgesloten, onbewezen M5-corelogs uit de inkomende `main` zijn niet
opnieuw toegevoegd; overig main-bewijs is behouden.

Na conflictresolutie zijn uitgevoerd (allemaal exit 0):

```text
python3 -B tests/test_guest_harness.py                         # 7 tests
python3 -B tests/test_m6b_capture.py                            # 13 tests
python3 -B -m py_compile tools/build.py tools/build_users.py tools/capture_m6b.py tools/run.py tools/run_m6b.py
cargo +1.90.0 test --locked -p capability-core -p virtio-core -p block-protocol -p object-core -p catalog-core -p kernel-core # 139 tests
rustup +1.90.0 target add x86_64-unknown-none x86_64-unknown-uefi
python3 -B tools/build.py --kernel-feature milestone-m6c
```

De eerste buildpoging eindigde met exit 1 omdat `x86_64-unknown-none` niet was
geïnstalleerd (`E0463`). Na installatie van de twee officiële Rust-targets
slaagde de build; twee bestaande dead-codewaarschuwingen bleven staan. Rustup
installeerde automatisch de ontbrekende 1.90.0-toolchain en Cargo ververste de
index zonder lockfilewijziging.

De M1–M6d-QEMU-gates zijn **niet opnieuw uitgevoerd** op de mergecommit.
De succesvolle M6c-crossbuild bewijst alleen compilatie, geen boot of VM-gedrag.
De eerder opgeslagen M6d-gates blijven aan hun exacte uitvoeringsbron gebonden;
de native bronvergelijking bevestigt dat de merge geen runtimecode van die bron
veranderde. Voortgang blijft **14% → 14%**: geen nieuw native criterium of
bewijsclaim.

De bijgewerkte remote PR-branch `c8945c6` brengt het M6d-closeoutbewijs mee:
opgeslagen M6c/M6d- en M6b-captures, plus een Windows-hostsuite van 173 tests.
Dit zijn ingekomen, brongebonden rapporten op checkpoint `326d2f6`, geen nieuwe
lokale VM-runs. De volledige hostregressie is ook lokaal op de gecombineerde
werkboom uitgevoerd: `python3 -B -m unittest discover -s tests -v`, **173/173,
exit 0**. De uitvoer is in deze sessie getoond, maar niet als nieuw
repository-logbestand opgeslagen.

## M6d-reviewherstel — Windowsacceptatie 28 oktober 2026

Aanvullende uitvoering op `326d2f6`, in een afzonderlijke worktree buiten
synchronisatiemappen. Fingerprint v3:
`5c69802a6de7c591207950f311d2e9e4f7eb07628d3b91dc9457f21c0bfa8c75`.
M6c/M6d: **35 groepen / 31 VM**, M6b: **18 groepen / 14 VM**, beide captures
exit 0. Samen 45 VM-starts, 20 koude recoveries en vier externe Windows-kills.
De twee gewone M1-runs missen een eigen per-run-QEMU-executablehash; die
meetgrens blijft open. Native kernel/service/probe zijn bytegelijk aan 88e6c04.

Host: M6c 30 Rust- en 48 Python-tests; M6b 22 Rust- en 30 Python-tests.
De bredere definitieve Python-hostsuite slaagt met **173/173**, inclusief
22 auditor- en 19 capturechecks. Deze overlappende aantallen niet optellen.
De eerdere brede suite faalde door LF→CRLF-conversie in synthetische fixtures;
alleen fixturebytes zijn gecorrigeerd. Exacte productiebytevergelijking en
single-linkguard blijven intact. Vier bronmutanten toetsen de auditbypass;
geen algemene formele Pythoncontrole.

De mislukte gesynchroniseerde run blijft failed bewaard. Een onmiddellijke
`fsutil hardlink list` toonde een echte tweede naam onder `.tmp.driveupload`;
het veroorzakende proces is niet vastgesteld. Identieke bron buiten die map
slaagt. Oorspronkelijk Linuxbewijs blijft aan 88e6c04 gebonden; de volledige
historische rawaudit is niet opnieuw uitgevoerd. Een tweede agent controleerde
alle 45 rawrecords, 395 artifacthashes en de werkelijke imagebytes; geen
menselijke audit of fysieke stroomuitvalclaim.

Exacte opdrachten en grenzen: [M6D-CLOSEOUT.md](verification/M6D-CLOSEOUT.md).
Voortgang **14% → 14%**; volgende native stap blijft catalogusherstart.
PR #11 is tussentijds extern gemerged op checkpoint 326d2f6 naar de M6c-branch
(`baeaa7bf`); #12 is extern naar main gemerged (`1f24f994`). Beide nieuwe trees
zijn vergeleken: zij voegen geen main-only broncorrectie toe. PR #13 behoudt
deze actuele main als ouder en voegt M6c/M6d plus het definitieve reviewbewijs
toe. Twee eerder verwijderde oude M5-logs blijven uitgesloten. Bij het schrijven
van dit closeoutverslag was de merge van #13 nog niet geclaimd; de actuele
integratiestatus staat in het bovenstaande checkpoint.

### Vooraf bewaard checkpoint

Hervat vanaf GitHub-head `7484d87`, afzonderlijke checkout `nieuw-os-review`.
De oude M6a-werkmap en ongetrackte bewijsbestanden zijn behouden. De vijf
CodeRabbit-punten op PR #11 worden gericht afgehandeld: exact QEMU-contract
ook voor M1, verplichte 164-bestandsinventaris, reproduceerpaden, historische
bestandsnaam en een nieuwe expliciete bronreview van de definitieve native bytes.
De historische R08-review en receipts blijven bytegelijk; hashwijzigingen van
de bijgewerkte audit/docs staan bewust in `verification/m6d-summary.json`.

De eerste Windows-hostcontrole faalde bij de natuurlijke exit van een child
met WinError 5, zowel in M6b als M6c. Na verlenging van uitsluitend de begrensde
wait bij een geweigerde TerminateProcess-call van 0,25 naar de bestaande
cleanupgrens van 10 seconden slagen 48 M6c- en 30 M6b-Python-tests. Een natuurlijke
exit blijft `kill_requested:false`; werkelijke live kills en timeoutafwijzing
blijven getest. De kernel/service/probe zijn ongewijzigd.

Dit checkpoint gaat vóór de nieuwe volledige Windows-gates naar GitHub.
Die gates, hun captures, externe herreview en merges waren op dit checkpoint
nog **niet** afgerond. De volledige historische Linux-rawaudit is in deze Windowscheckout
niet uitvoerbaar zonder oorspronkelijke ruwe bestanden en QEMU-binary; de
nieuwe audit heeft afzonderlijke negatieve hosttests. Voortgang **14% → 14%**.
Volgende native functie blijft de beperkte catalogusherstart uit WORK-QUEUE.

## M6d — native onzekere I/O gebouwd en geaccepteerd

28 oktober 2026. Uitvoeringsbron `88e6c04be218b8b09ea4f0a226f6b9719da706a6`,
fingerprint v3 `8ca6d5d5291fde62f91fb47dca7df4d058424604449fb32b9e5899b1d4fbef93`.
Werkbranch `codex/m6d-io-recovery-roadmap`, [PR #11](https://github.com/simonvanieperen/nieuw-os/pull/11).
De broncheckpoint is vóór de eindgates gepusht en bewaart zowel M6d-checkpoint
b739183 als upstream M6c-review 5ba238cc. De afsluiting wijzigt documentatie en
bewijs; alle 45 eindruns zijn aan deze ene native uitvoeringsbron gebonden.

De kernelobserver koppelt de eerste Unknown aan een werkelijk teruggegeven
terminale muterende syscallfout. Geen onzekere commit, succes-ACK of levende
revisie; beide appdomeinen blijven actief. Na Unknown leveren geldige
GET/LIST/PUT/DELETE/RENAME-aanvragen geen nieuwe I/O op. Ongeldige of vreemde
verzoeken houden hun eigen validatie-/autorisatiefout. Acht native testfeatures
dekken WRITE/FLUSH-status, time-outs, resetquarantaine, descriptorweigering en
ongeldige used-length. De eigen kernel, capabilities, IPC v4 en journal v2
blijven het fundament; geen nieuwe externe dependency.

| Werkelijk uitgevoerd op deze bron | Resultaat |
|---|---|
| M6d-gate en capture | exit 0; 35 groepen / 31 VM-invocaties, 14 koude recoveries, 8 foutcases, 2 externe cuts |
| Afzonderlijke M6b-regressie en capture | exit 0; 18 groepen / 14 VM-invocaties, 6 koude recoveries, 2 externe cuts |
| Hostcontroles | 30 Rust + 48 Python voor M6c; 22 Rust + 30 Python voor M6b; exit 0 |
| Gerichte rustfmt en normale eindbuild | exit 0; kernel/loader bytegelijk aan beide normale M6d-runs |
| Twee afzonderlijke bewijsaudits | exit 0; 274 + 121 artifacthashes, 16 + 7 werkelijke volumes; bronnen en herstelkoppelingen gecontroleerd |

R09 vergeleek alle 144 bronbestanden en vier M6c-hosttestbestanden met de
uitvoeringscommit; alle 368 M6d-runfiles zijn gehasht. De 16 M6d-images zijn
ook vanuit alleen seeds en sectorhex opnieuw opgebouwd met dezelfde hashes.
A04 controleerde afzonderlijk de 14 M6b-runs en hun werkelijke schijfbytes.
AI-review is geen menselijke audit of formeel bewijs. Exacte commando's,
archieven en grenzen: [M6D-ACCEPTANCE.md](verification/M6D-ACCEPTANCE.md).

De eerste gate stopte vóór een VM door een ontbrekend firmwarezoekpad;
de failed suite is bewaard en het aanwezige passende OVMF-paar is naar de
ondersteunde lokale testmap gekopieerd. Daarna is de volledige gate opnieuw
uitgevoerd zonder bron-/criteriumwijziging. De twee concrete reviewbevindingen
(archiefbestemming en precieze request-ID's) zijn vóór de bronfreeze hersteld.

**Voortgang: 13% → 14% (+1 punt voor S07).** README bevat de zichtbare balk;
`progress.json`, `PROGRESS.md` en `tools/progress.py` houden de vaste 100-punten-
roadmap consistent. AGENTS.md verplicht herbeoordeling na iedere opdracht,
ook als de score gelijk blijft. Geen tijdsraming of veiligheidspercentage.

Grenzen: koude recovery start een nieuwe VM op dezelfde bytes; automatische
herstart in dezelfde sessie is nog open. De historische M6b-observer houdt zijn
foutpadbeperking. Vier objecten, 128 inhoudsbytes en 32 records blijven
proefbudgetten; geen desktop, grotere documenten, encryptie of native NAS.
Status-/completioninjecties zijn geen fysieke schijffout of stroomuitval.
Volgende beperkte stap: geautoriseerde catalogusherstart met nieuwe generatie,
veilige reset en journalherstel zonder replay. K05 en S08 blijven open.

## M6c — native documentnamen en catalogus gebouwd en getest

28 oktober 2026. Basis `bb22fdbe15919edc23baa158a46236195ce314b2`, werkbranch
`codex/m6c-named-catalog`. De afgeronde M6b-basis hieronder blijft behouden.

Gebouwd: dependencyvrije `catalog-core`, IPC v4, journal v2, blijvende UTF-8-namen,
GET/PUT/DELETE/RENAME en één domeingebonden LIST-snapshot. Hernoemen bewaart de
inhoud onder een nieuwe objectrevisie; alleen na WRITE én FLUSH volgt succes.
De vier objecten, twee domeinen, 128 inhoudsbytes en 32 journalrecords blijven
proefbudgetten. Nieuwe formaten zijn expliciet; bezet M6b-v1 wordt vóór READY
geweigerd, zonder migratie of formattering. Bestaande M6b blijft afzonderlijk
bouwbaar. Contract: `implementation/M6C-NAMED-CATALOG.md`; besluit ADR-030.

Definitieve uitvoeringsbron: `bc4cd104e86e2c047b727ae96e07a03fbc810c3c`;
fingerprint v3 `4468b3c18bd7fa57bf3631d634d52ee4c5fe9a76da9f125714d6db95e4407345`.
Deze bron is vóór de lange controles op GitHub bewaard. De oorspronkelijke afsluitende wijzigingen
betreffen documentatie en bewijs. De latere integratie van het M6b-reviewherstel
wijzigt hosttools en tests; de native runtimebron blijft gelijk. Het afzonderlijke
hostbewijs staat in `verification/m6c-review/merge-followup/`. [PR #10](https://github.com/simonvanieperen/nieuw-os/pull/10)
staat boven de M6b-branch; geen merge naar main of release uitgevoerd.

| Scope op deze bron | Resultaat | Bewijs |
|---|---|---|
| Volledige M6c-gate en capture | exit 0; 19 groepen / 15 VM-invocaties | `verification/m6c/` |
| Volledige M6b-regressie en capture | exit 0; 18 groepen / 14 VM-invocaties | `verification/regression-m6c/` |
| Catalog-core en harnas | 30 Rust- en 33 Python-tests; fmt/Clippy/no_std exit 0 | `verification/m6c/`, `verification/m6c-review/core-recheck/` |
| Echte adapter via host-ABI-shim | 15 scenario's, exit 0; geen native devicefoutinjectie | `verification/m6c-review/adapter-result.json` |
| Afsluitende M6c-build | exit 0; kernel en loader bytegelijk aan beide normale eindruns | `verification/m6c-ready/` |

De 15 M6c-gate-invocaties zijn 14 M6c-VM's plus één gewone M1-boot. Ze omvatten
twee normale images, domeingrenzen na populatie, volle journal, corruptie, geldig
legacy-v1, zes koude recoveries en twee echte externe QEMU-kills tijdens RENAME.
Vóór WRITE blijft `Café A` op revisie 1; na FLUSH vóór ACK verschijnt
`Café hernoemd A` op revisie 2. De 128 inhoudsbytes blijven exact gelijk.
Alle zes koude herstelruns veranderen geen schijfbytes. De M6b-regressie omvat
eveneens een eigen gewone M1-boot; dit zijn geen nieuwe volledige M1–M6a-gates.

Een afzonderlijke AI-audit controleerde 130 geregistreerde artifacthashes,
176 runbestanden, acht werkelijke volumes en alle 141 bronbestanden tegen
`bc4cd104`. De 67 niet-lege journal-sectoren en seeds zijn als tekst bewaard;
daaruit zijn alle imagehashes opnieuw opgebouwd. Zes vroege runrecords noemen
nog de toenmalige lokale HEAD `bb22fdb`; hun volledige broninhoud en fingerprint
zijn bytegelijk aan `bc4cd104`. Dit metadataverschil blijft zichtbaar.
De aparte statische integratiecontrole bindt het native gebruikers-ELF aan de
geteste kernel en controleert budgetten en instructies. Reviews zijn AI-reviews,
geen menselijke audit of formeel bewijs. Exacte commando's, afbakening en
provenance: [M6C-ACCEPTANCE.md](verification/M6C-ACCEPTANCE.md).

De vijf GitHub-reviewopmerkingen over bewijsstatus en reproduceerbaarheid zijn
afgehandeld in `verification/m6c-review/REVIEW-FOLLOWUP.md`: actuele native
resultaten gelinkt, portable adapter opnieuw getest en de vier corecontroles
read-only herhaald met HEAD/status en bronhashes vóór/na. Historische evidence
blijft behouden. Dit verandert de native bron of het aantal VM-runs niet.

Het geïntegreerde hosttoolherstel heeft 123 unieke hosttests over twee runs:
121 bestaande tests, gevolgd door vijf preflighttests waarvan drie herhaald
en twee nieuw. Een latere reviewaanscherping bindt de drie M6c-hostlogtrio's
met SHA-256 aan de gate en controleert de binding vóór én na het kopiëren.
Deze wijziging heeft afzonderlijk hosttestbewijs in
`verification/m6c-review/host-log-binding/`; de eerdere 123 tests worden niet
als uitvoering op deze latere toolbron opgevoerd. De oorspronkelijke M6c-suite
mist gate-time-hostloghashes en blijft als historisch bewijs ongewijzigd.

Na opname van het M6b-vervolgherstel zijn de gedeelde capturepaden samen
gecontroleerd: 29 tests geslaagd (vier M6a, tien M6b, vijftien M6c), met
ongewijzigde inputhashes vóór/na. Dit is één afzonderlijke hostinvocatie;
het aantal wordt niet bij eerdere overlappende runs opgeteld. Broncombinatie,
commando en ruwe uitvoer staan in `verification/m6c-review/integration-followup/`.
Geen native build of VM uitgevoerd voor deze integratiecontrole.

Bekende observerfout op de hierboven vastgelegde historische M6c-bron: native
WRITE-/FLUSH-fouten of time-outs kunnen ook in de gewone `milestone-m6c` een
kernel panic veroorzaken voordat de client `Unknown` ontvangt. De host-ABI-
proeven toetsen alleen de serviceadapter. Contract en M6C-ACCEPTANCE vermelden
expliciet dat dit native pad op die bron niet gerepareerd of end-to-end bewezen
is. De afzonderlijke M6d-reparatie en haar geslaagde foutinjectiegate staan bovenaan.
De onderzoeksacceptatie blijft begrensd tot de vastgelegde QEMU/device-fixture.

Vervolg: deze opslaginterface aan native productfuncties koppelen; grotere
documenten, ruimtehergebruik, encryptie, sleutels en native NAS/netwerk vereisen
eigen ontwerp en acceptatie. M6c heeft nog geen interactieve desktop. FNV blijft
uitsluitend corruptiediagnose; QEMU-kills bewijzen geen fysieke stroomuitval.

## M6b — native documentopslag gebouwd en getest

Basis `10b32e31671827f2bd63444113b981ff15bfbb73`, werkbranch `codex/m6b-object-storage`.
De nieuwe `services/objects`-crate bevat een allocatorvrije Rust `no_std`-objectdienst:
vier objecten van maximaal 128 bytes, twee via IPC-endpoints gescheiden domeinen,
GET/PUT/DELETE, revisiecontrole en blijvende verwijderrecords. Alleen de dienst
krijgt blokrechten op de 32 journal-sectoren; apps krijgen hun eigen endpoint.
Succes volgt pas na echte WRITE- en FLUSH-completion. Corruptie en onzekere
mutaties sluiten toegang af; er is geen automatische herhaling van writes.

Definitieve uitvoeringsbron: `1e61a20beac67d685cf4bff5f0f9f588bf2cb56e`;
fingerprint v3 `43e6c06e43e3ba602b49c304328f0c1700bc26dae0a1256f0d5d420421f55ae2`.
Op 28 oktober 2026 (UTC-logdatum 27 oktober) zijn beide volledige eindgates en hun captures met exit 0
afgerond. De resultaten hebben afzonderlijke scopes en worden niet met eerdere
deelruns opgeteld:

| Scope | Bron | Groepen / echte VM-runs | Bewijs |
|---|---|---|---|
| M6b volledige objectopslag | `1e61a20` | 18 / 14 | `verification/m6b/` |
| M1 volledige boot- en foutgate | `1e61a20` | 10 / 8 | `verification/regression-m6b/m1/` |
| M2–M5 volledige regressies | `44892e7` | 67 / 56 | `verification/regression-m6b/m2/` t/m `m5/` |
| M6a volledige ongewijzigde baseline | `10b32e3` | 54 / 49 | `verification/m6a-linux/` |
| M6a normal/herstel op M6b-runtime | `44892e7` | afzonderlijk paar / 2 | `verification/regression-m6b/m6a-pair/` |

M6b omvat twee normale images met dezelfde binaries, CRUD/domeingrenzen, een
volle journal, corruptie, twee echte externe QEMU-kills en zes koude recoveries.
Een aparte AI-audit decodeerde zeven werkelijke volumes en controleerde alle
121 artifacthashes. De M1-audit controleerde acht eigen gastlogs, 31 artifacthashes,
vier buildprofielen en de echte 60-seconden-time-out. De 56 eerdere M2–M5-runs
zijn onder de laatste parser opnieuw gecontroleerd; alle 68 native bron- en
bouwinputs zijn sinds `44892e7` bytegelijk. Dat zijn geen 56 nieuwe runs.

Hostbewijs: 22 Rust-objecttests, 26 M6b-harnastests, 8 snapshotbindingtests,
7 guestparsertests en 9 bestaande capturetests. Zeven aanvullende proeven van
de echte serviceadapter met een host-ABI-shim controleren onzekere WRITE/FLUSH-
uitkomsten; dit is geen native M6b-devicefoutinjectie. Reviews staan in
`verification/m6b-review/`; het zijn AI-reviews, geen menselijke audit of formeel
bewijs. Rust 1.90.0, QEMU 10.0.2 en het bijbehorende OVMF-profiel zijn vastgelegd
in `verification/native-linux-toolchain/`.

Drie eerdere M1-pogingen zijn expliciet afgewezen en behouden. De oude parser
accepteerde eerst een hogere testfase; later blokkeerde de hashguard teruggekeerde
oude artifacts vóór QEMU-start. De vervangingsoorzaak is niet vastgesteld.
M1 gebruikt nu per build een eigen gecontroleerde UUID-snapshot, dezelfde normale
kopie voor alle coldboots/negatieve fixtures, strikte fasecontrole en hashes vóór
iedere start. De afsluitende volledige M1-gate slaagt. De wrapperconsole is niet
overal compleet; de bewijsbasis zijn de afzonderlijke gecontroleerde gastrecords.
Details en de geslaagde capture-hercontrole zonder nieuwe VM-runs:
`verification/regression-m6b/FINAL-ACCEPTANCE.md` en
`verification/m6b-review/capture-recheck/`.

De laatste lokale build is opnieuw `milestone-m6b`; kernel en loader zijn
bytegelijk aan beide normale eindgate-runs. Dit is een buildcontrole zonder
extra VM-start (`verification/m6b-ready/`).

Reviewcorrecties van 28 oktober 2026, basis `bb22fdbe15919edc23baa158a46236195ce314b2`:
historische audits/guards en M1-afwijzingen verduidelijkt, bewijsbeschrijvingen
vertaald en M5-vergelijkingen aan geverifieerde gastrecords gebonden. Preflight
weigert nu ook een niet-nul `probe_exit` en een `probe_error`; de oude `rustup`-probe met `-6` was dus
geen geldige inventariscontrole. De M6b-gate probeert na fouten de normale M1-build
te herstellen en bewaart de oorspronkelijke fout. De 40 onafhankelijk uitgevoerde gerichte hosttests slagen; exacte commando's,
bronhashes en ruwe uitvoer staan in `verification/m6b-review/merge-followup/`.
Deze follow-up betreft hosttools; de oorspronkelijke native uitvoeringsbronnen
in de tabel hierboven blijven behouden.
De vervolgreview scherpt de M5-archiefchecker aan met expliciete controles,
ook onder `python -O`; een gewijzigd seedrecord wordt nu afgewezen. Vier
nieuwe hosttests verifiëren de M6a-stderrverwijzing en volledige
stdout/metadata/stderr-capture. Historische meetbestanden blijven intact;
de M6a-suite heeft een afzonderlijke gecorrigeerde leesindex. Dit nieuwe
hostbewijs staat in `verification/m6b-review/review-followup-2/` en is geen
nieuwe native gate of herhaling van de eerdere 40 tests.
Geen nieuwe native gate uitgevoerd; de M3-hosttestcheckout en de redirectinstelling
van de oude buitenste capturewrapper blijven expliciet onbekend.
Historische native observerbeperking op deze eerdere bron: WRITE-/FLUSH-fouten konden, ook in
`milestone-m6b`, een kernel panic veroorzaken voordat de client `Unknown` krijgt.
Deze oude gates leveren daarvoor geen native foutbewijs. De reparatie en nieuwe
afgebakende injectieproeven op `592dccf` staan in de bovenste hervattingssectie.

Deze begrensde lokale opslagproef heeft geen namen, directoryboom, garbage
collection of encryptie. Verwijderrecords voorkomen het terugkomen van oude
appverzoeken; zij wissen historische schijfbytes niet fysiek. FNV is uitsluitend
een corruptiediagnose. Native NAS/netwerk, sleutels en desktopkoppeling blijven
afzonderlijke stappen. Contract: `implementation/M6B-OBJECT-STORAGE.md`.

## N02 — hybride PQC-transport: echte hostproef getest

27 oktober 2026, basis `ece94a5cfebecaec763eb2855aa9f84e55dcdcbc` (PR #7), werkbranch `codex/pqc-transport-foundation`; eerste checkpoint `3d05f9f8dccb4f515d8522578d20bbf00d8b8ed7`. De aangeleverde beveiligingsblauwdruk is getoetst aan de actuele bronnen en repository. ADR-028 en `implementation/VERIFIABLE-SECURITY-AND-PQC.md` leggen het standaard-TLS-profiel, formele bewijsgrenzen en concrete native vervolgopdrachten vast.

`services/nas/transport.mjs` heeft echte loopback TLS 1.3 met gemeten X25519MLKEM768, AES-256-GCM-SHA384, vereist ALPN, wederzijdse CA-/certificaatpincontrole en serverhostnamecontrole. De client versleutelt vóór transport; de server echoot alleen begrensde ciphertext. Policy controleert toestemming vóór de socketpoging, na authenticatie en bij iedere verzending/response. Een transportfact is gebonden aan app en toestemmingsgeneratie; intrekken en opnieuw toestaan herstelt geen oude geauthenticeerde verbinding. Alleen internet intrekken laat de toegestane LAN-route bestaan. Geen klassieke fallback of sessiehergebruik.

Werkelijk uitgevoerd op Node 24.22.1/OpenSSL 3.5.9/Linux x64: `node --test --test-isolation=none services/nas/tests/*.test.mjs ui/prototype/tests/*.test.mjs` → **124/124**, exit 0, geen skips (85 service: 13 crypto, 19 policy, 23 store, 30 transport; 39 UI). Beide demo's exit 0; 20 JS/mjs-syntaxchecks en diffcontrole geslaagd. Ruwe uitvoer, bronhashes, runtimeversies en review: `verification/pqc-transport/`. De oorspronkelijke sandbox weigerde lokaal luisteren; de expliciete loopback-escalatie is automatisch toegestaan en de tests zijn vervolgens werkelijk uitgevoerd. Geen externe verbinding of echte NAS/router gewijzigd.

Twee afzonderlijke AI-reviews; een concreet randgeval rond herleving van oude idletransportfacts is hersteld en onafhankelijk hertoetst. Dit is geen formeel bewijs of menselijke audit. De testcertificaten zijn klassiek Ed25519; netwerklabels zijn harnasaannames bij loopback. Geen postkwantumauthenticatie, WAN-/LAN-detectie of koppeling van transportecho aan de N01-store. Tijdelijke frame-/tijdsgrenzen zijn geen productbeperkingen. De browserrendering is in deze stap niet opnieuw getest.

Geen native PQC-crate, entropy-syscall, boot-/blokcryptografie of Grover-uitbreiding toegevoegd. De native build heeft concrete ABI-/geheugen-/instructiegrenzen; Rust/Cargo en QEMU waren tijdens die N02-hostsessie niet beschikbaar. Het huidige M6b-traject hierboven heeft inmiddels een werkende native toolchain. Dit verandert geen bestaande M1–M6a-bewijsstatus. N01 hieronder blijft als historische basis bewaard.

## N01 — internet en persoonlijke NAS-ruimte: hostproef getest

27 oktober 2026, gebouwd vanaf ontwikkelbasis `a96ad45adb044177e18670cd38624ea2a671db3d` op `codex/nas-private-sharing`. De nieuwe gebruikersopdracht verruimt de eigen-LAN-beperking naar eigen of vrienden-/familie-NAS, met expliciet peergebonden internet, client-only inhoudssleutels en eenzijdig adminverwijderrecht. Privacycontract revisie 3 en ADR-027 zijn leidend.

Werkende code in `services/nas/`: AES-256-GCM voor inhoud en namen; echte lokale ciphertextstore; tenantgrenzen, quota, eenmalige uitnodigingen, aparte admincapability, revoke/evict, afwijzen van oude sessies en uploads, duurzame tombstones en heropening. Materiaal bevat Mijn NAS, Ruimte ontvangen en Ruimte aanbieden als browsermock. Eenmalig toestaan volstaat voor automatische routekeuze LAN→internet→LAN naar dezelfde geverifieerde peer; planner en UI verlenen daarbij geen nieuwe rechten. Lokale beheerrol is apart van NAS-datatoegang; internet geeft geen beheerrecht.

Werkelijk uitgevoerd op Node 24.22.1/OpenSSL 3.5.9/Linux x64: `node --test --test-isolation=none services/nas/tests/*.test.mjs ui/prototype/tests/*.test.mjs` → **89/89**, exit0 (50 service, 39 UI). `node services/nas/demo.mjs` → passed/exit0. Ruwe uitvoer, bronhashes, review en grenzen: `verification/nas/`. De procescrashproeven gebruiken eigen synthetische tempmappen. Geen echte gebruikersbestanden, NAS of router gewijzigd.

**Niet geleverd/bewezen:** native netwerkstack, TLS-/echte peer-authenticatie, socketafdwinging, NAS-installatiemodus op eigen OS, veilige persistente schrijfkeys/multi-device recovery, fysieke stroomuitval, vijandige-admin snapshotrollback of fysieke gegevensvernietiging. Key recovery is alleen lezen; N01 bevat geen achtergrondsync. Algemene internettoestemming is nog appgebonden, NAS-internet wel peergebonden. Browserrendering van de nieuwe schermen is niet gecontroleerd: de Browser-plugin kon zonder oorspronkelijke browsercontext niet starten; state-/syntaxcontroles wel uitgevoerd.

N01 wijzigt de M6a-runtimecode en bestaande bewijsclaims niet. De M6a-eindpoort hieronder blijft een eigen open stap. Vervolg: M6a afronden, dan native opslag-/netwerkadapter en clientkeybeheer volgens implementation/NAS-SHARING-AND-INTERNET.md.

## Actief: M6a

Nieuwe opdracht 27 oktober 2026 vanaf `448974b`, branch `codex/m6a-write-recovery`. Bouwcontract vóór code: implementation/M6A-WRITE-RECOVERY.md. Gekozen: WRITE512/FLUSH met expliciete rechten, een vierrecords-journal, echte dienstcrashes en QEMU-proceskills plus koude recovery op uitsluitend harnasgegenereerde images. Geen fysieke stroomuitvalclaim. M5-basis hieronder behouden; oorspronkelijke status ook in verification/m5/PROJECT-STATE.md.

Mergecheckpoint: de basisbranch `codex/m2-m4-isolation` (`e816dfd`) is geïntegreerd. De M5-kernel- en archiefvalidatie uit die branch blijft behouden; M6a-rechten, transport, toolroutes en herstelcode blijven actief. Post-merge geslaagd: Python-tool `py_compile`; evidence-/capture-/M6-harnascontroles (9, 1, 4, 4 en 20 tests); M6-ELF-artifacttests (12); Rust-hosttests (`cargo +1.90.0 test --locked -p capability-core -p virtio-core -p block-protocol -p kernel-core`: 30/21/20/16); en `python -B -X utf8 tools/build.py --kernel-feature milestone-m5` plus `--kernel-feature milestone-m6a`. De M1–M6a-QEMU-poorten en gastregressies zijn na deze merge niet opnieuw uitgevoerd.

Eerste samenhangende bronversie: typed WRITE32/FLUSH64, Prepared/Published/Unknown, kernel-inputsnapshot512, modern transport, echte nieuwe M6-userdienst/client/supervisor en Python-volume-/crashharnas. Daadwerkelijk geslaagd: 30 capability- en 21 virtio-tests, 20 protocol/journaltests, 16 kernel-core-tests en 12 M6-ELF-artifacttests; core fmt/Clippy/no_std exit0. Eerste echte M6-crossbuild exit0; latere twee observercorrecties worden opnieuw gebouwd. Het root-harnas heeft 8 schijf-oracle- en 5 childprocescontroles geslaagd, geen guestbewijs. Logs/hashes onder verification/m6a/core/ en service/.

Gerichte echte M6-QEMU-proeven zijn nu geslaagd: normaal gen2, 512B-rechten/snapshot, dienstcrashes vóór WRITE en na ACK, FLUSH-fout en resetquarantaine. Ook een echte externe QEMU-kill na FLUSH (raw3765043201, geen PASS) plus nieuwe VM op exact dezelfde image is geslaagd: gen2, orakel groen, recovery veranderde geen imagebytes. Negen diagnostische records inclusief de eerdere afgewezen getalnotatie blijven onder verification/m6a/diagnostics/. Laatste broncorrecties zijn nog niet door de volledige gate afgedekt.

Transport-/harnasreviews sloten concrete foutvolgorde-/Rust-borrow-/recovery-/coveragebevindingen; rapporten onder verification/m6a/. Root-hostharnas heeft nu 19 geslaagde controles. De volledige poort is geïmplementeerd (verwacht 54 groepen/49 VM, nog niet als geslaagd geclaimd), met 6 dienstcuts, 6 echte VMcut/recoveryparen, Unknown/full/corruptie en twee normale images. Nu: bronfreeze/checkpoint, die hele poort werkelijk uitvoeren, daarna M1–M5-regressie en onafhankelijke raw/hash-audit.

GitHub-e-mailprivacy blokkeerde de eerste push; alleen de ongepushte commitmetadata is naar het GitHub-noreplyadres hersteld. De branchpush en remote-ID-controle slaagden daarna, zonder privacy-/globale configwijziging. M6a staat als concept-PR https://github.com/simonvanieperen/nieuw-os/pull/5 boven op de M5-branch.

Finale kandidaatbron `8c53ea4` bevat ook de twee tussentijdse remote-orakelcorrecties via een gewone merge, zonder force-push. Fingerprint v3: `6fce5c12e87cf6b1a1353743357593de3d8b6843012b12be6deaf7fe17ce3b66`. Volledige M6a-poort gestart; tussentijdse snapshot bij 43 geslaagde groepen bewaart nog nadrukkelijk running/false in verification/m6a/checkpoints/. Normaal schrijven, volle journal en koude recovery zijn daarin werkelijk groen; volledige einduitslag/regressie/audit volgen afzonderlijk. De root-harnascontroles zijn inmiddels 20/20 groen, inclusief behoud van beide remote-correcties.

27 oktober 2026. M5 is geïmplementeerd en getest in het vaste QEMU-profiel.

Repository: https://github.com/simonvanieperen/nieuw-os. Branch `codex/m5-block-service`, concept-PR [#3](https://github.com/simonvanieperen/nieuw-os/pull/3). Basis `c630c59` (M1–M4); definitieve runtime-/harnasbron `7af544aa1b8d731ac27f384f0ba95aa44586a57d`. Latere commits bewaren documentatie en bewijs. Main bevat de startspecificatie; geen merge of release uitgevoerd.
## Reviewfollow-up

De bewijsarchiefcapture valideert een volledige nieuwe opname in een tijdelijke map en laat het bestaande archief bij validatie- of installatiefouten ongemoeid. Alleen stdoutlogs onder `build/logs/` kunnen als hostbewijs worden opgenomen; daarnaast is alleen het gevalideerde `m5-two-image-check.json` toegestaan als gestructureerd M5-hostbewijs. M2–M4-gastresultaten voldoen aan de gedeelde fase-/caseverwachtingen voor markers en ruwe exitstatus; alle gastresultaten doorlopen hun eigen controles. `python -B -X utf8 tests/test_capture_gate.py` (8 tests) slaagt; dit is hosttoolbewijs, geen nieuwe browser- of OS-test. De registratie van vijf `.png`-bestanden als JPEG gebruikt nu hun signatures: `python3 tools/update_evidence_files.py` en `python -B -X utf8 tests/test_evidence_files.py` (4 tests) slagen. Alle genoemde controles exit 0. De grote-probehostcontrole is lokaal geblokkeerd: Rust 1.90.0 en `x86_64-unknown-none` ontbreken; `tools/preflight.py` meldt ook geen QEMU/OVMF.

De capturegate herleidt M2–M4-markers, verboden markers en ruwe exitstatus nu uit één fase-/casecontract dat ook de milestone-runner gebruikt; verwachtingen in een opgeslagen runrecord worden daarvoor niet vertrouwd. `python -B -X utf8 tests/test_capture_gate.py` (8 tests), `python -B -X utf8 tests/test_guest_harness.py` (3 tests), de gedeelde-contractimportcontrole en vergelijking met alle 36 gearchiveerde M2–M4-gastruns slagen (exit 0). Dit is hostvalidatie van de verifier, geen nieuwe QEMU-run.

Na de nieuwste samenvoeging van `codex/m2-m4-isolation` zijn `python -B -X utf8 tests/test_capture_gate.py` (8 tests), `python -B -X utf8 tests/test_evidence_files.py` (4 tests), `python -B -X utf8 tests/test_guest_harness.py` (4 tests) en `python -B -X utf8 -m py_compile tools/build.py tools/capture_gate.py tools/run.py tools/gate_contract.py tools/test_milestones.py tools/test.py` geslaagd (exit 0). Dit zijn gerichte hostcontroles; de QEMU-gates zijn voor deze merge niet opnieuw uitgevoerd.

De reviewfollow-up blokkeert nu ook een gevalideerde hostlog als bewijs voor een M2–M4-gastcase; de fase-/caseverwachting uit `gate_contract.py` bepaalt dat hiervoor een gecontroleerde runmap nodig is. `python -B -X utf8 tests/test_capture_gate.py` (9 tests), `python -B -X utf8 tests/test_common.py` (1 test), `node --test ui/prototype/tests/state.test.mjs` (23 tests) en `python -B -X utf8 -m py_compile tools/capture_gate.py tools/common.py tools/build.py experiments/quantum/verify.py` slagen (exit 0). Dit zijn host-/mockcontroles; geen QEMU-run.

Een alleen-lezen-Webvenstergrant vervangt nu geen nog geldige Notities-schrijftoestemming van de editor; beide grants blijven afzonderlijk begrensd. `node --test ui/prototype/tests/state.test.mjs` slaagt met 24 tests en `node --check ui/prototype/state.js && node --check ui/prototype/app.js` slaagt (exit 0). Dit is mockbroker-/hostbewijs, geen OS-enforcementbewijs.


## Wat werkt

- Eigen UEFI-loader en Rust no_std-kernel, BootInfo 1.1, eigen framebeheer/paging, W^X, bewaakte stacks, ring-3-processen, PIT-preëmptie, capabilities en Message320-IPC. Geen bestaande kernel als basis.
- Echte Rust-blokdienst in ring 3 met client en supervisor. Alleen de dienst krijgt READ op een getypeerd BlockDevice; requestendpoints binden sector 4 en 6. Appvelden kiezen geen fysieke sector of replycap.
- Modern virtio-blk op PCI `00:05.0`: splitqueue van acht plaatsen, maximaal één uitstaande read, vijf gepinde kernel-DMA-pagina's, begrensde submit/poll/reset en supervisor-RW/NX/UC-MMIO. User-MMIO/DMA/PCI-toegang wordt geweigerd.
- Drie echte dienstcrashes met pending software-read, bevestigde reset, nieuwe dienst-PID/epoch en echte IPC-capoverdracht. Client en supervisor behouden PID/adresruimte; epoch 4 levert nieuwe devicebytes. Geen claim dat DMA fysiek tegelijk met de fault liep.
- Alle 13 ongeldige dienstverzoeken veroorzaken aantoonbaar geen blockcall/publicatie/completion. Foutreplies zijn leeg/nulgevuld; onverwachte caps worden gesloten. Time-out/reset/quarantaine en lokale foutafhandeling zijn getest.

M5 is alleen lezen op synthetische wegwerpimages. Nog geen schrijvende opslag, bestandssysteem, interactieve native desktop of algemene spawn-/driverbeheer-API.

## Werkelijk uitgevoerd

| Opdracht / fase | Resultaat | Bewijs onder verification/m5/ |
|---|---|---|
| preflight.py --strict | exit0, geen ontbrekende tools/targets | preflight.json |
| python -u tools/test_m5.py | exit0; 25 groepen/20 VM | suite.json,guest-runs.json |
| python -u tools/test_milestones.py --through m4 | exit0; M2 8/7, M3 22/19, M4 12/10 groepen/VM | regression/{m2,m3,m4}/ |
| python -u tools/test.py | exit0; 10 groepen/8 VM | regression/m1/ |
| Vijf capture_gate.py-opdrachten | exit0; raw/serial/stderr/artifacthashes en M5-schijforacles hercontroleerd | genoemde rapportmappen |
| Normale M5-build + run.py --smoke --milestone m5 | exit0/raw33; extra eindstart | manual-m5.json,final-build-manifest.json |

Definitieve suites: **77 controlegroepen en 64 echte VM-runs**, plus één normale M5-eindstart. Eerdere diagnostische runs zijn niet opgeteld. M1-negatieve guestrecords blijven terecht passed:false; de suite controleert ontbrekende/corrupte kernel, panic en echte 60s-time-out.

M5-hostlaag: 62 Rust-tests (26 capability, 15 kernel-core, 12 virtio, 9 protocol), 11 ELF-artifacttests, 4 harnasparsertests en 5 negatieve schijf-oracletests. Alle 21 eerdere M4-capabilitytests zijn behouden. Extra corecontroles fmt/Clippy-Dwarnings/no_std exit0; exacte scope/argv in core/report.json. Hosttests bewijzen op zichzelf geen device-I/O.

Alle definitieve guestrecords hebben fingerprint v3 c7455ebe2dfcd0e4337d058f440712d60b3ae600d2953c385ee81ba6d69a0307. Twee normale M5-runs gebruiken dezelfde kernel/loader en verschillende echte images; ontvangen sectorchecksums kloppen en de volledige image-SHA256 blijft gelijk. FNV is diagnostiek, geen cryptografie; de guest kent geen seed/fixtureantwoord. Exacte opdrachten/gewijzigde bronbestanden: validation.json. Testmatrix: ACCEPTANCE.md.

## Starten

Vanuit de projectroot in PowerShell:

```powershell
python tools/build.py --kernel-feature milestone-m5
python tools/run.py --smoke --milestone m5
```

De runner maakt zelf een nieuwe alleen-lezen 64KiB-image onder build/run/. De eindige proef leest via de dienst en sluit QEMU na de assertions af. De laatste lokale build staat op het normale M5-profiel. Testsuites herstellen na foutvarianten M1; bouw bij hervatten expliciet het gewenste profiel. Zie DEVELOPMENT.md, drivers/virtio-blk/README.md, services/block/README.md en userspace/M5-PROBE.md.

## Review en grenzen

Herstelreviews: transport-review.md en service/user-review.md. De verifier controleerde alle 65 definitieve VM-runs,279 artifacthashes en werkelijke M5-schijfbytes: m5-verifier-guest.json en m5-verifier-final.json. Finale kernel/loader zijn gelijk aan beide normale gate-runs en de laatste CLI-start. Dit zijn AI-reviews, geen menselijke audit.

Echte tests vonden en herstelden opnieuw resetten tijdens quarantainecleanup en een verkeerde sectorscope-foutcode. De eerste gecombineerde gate bleef rood; failed-records blijven als failed bewaard onder diagnostics/. Andere herstelpunten: BME-clear/reset na initfout, device-id-binding, terminale PID-resultaten, onnodige clientautoriteit, CPU-adresbreedte en harde observatie van alle 13 protocolfouten. DRIVER_OK-readbackfout, device1-scope en exit na resetack vóór foutpoll zijn bronfixes zonder aparte gastinjectie; aanvullende testmogelijkheden.

Kerneltransport en protocolconform QEMU-device/backend zijn vertrouwd. Geen IOMMU, containment van kwaadwillige DMA, SMP- of fysieke-hardwareclaim. Pool blijft gepind; zonder resetack in quarantaine tot VM-stop. Injecties zijn test-only: lokale snapshots na echte completion of suppress-notify/mask-ack. Geen Secure Boot/PQC/productiegeschiktheidsclaim.

Materiaal is browserprototype; qsim is hostsimulator. 9 UI- en 15 quantum-bron-/binaryhashes opnieuw gelijk; eerdere resultaten blijven geldig voor die bestanden, tests niet herhaald. 200%-zoom, schermlezer/menselijke proeven, native UI, OS-quantumport, QPU en OS-afgedwongen lokale AI/NAS-grenzen blijven open.

Geen hostdisk, firmware, NAS, productiekeys, globale Codexconfig of betaalde infrastructuur gewijzigd. Bestaande .codex/environments/ behouden buiten Git. Licentiekeuze vóór publieke publicatie blijft volgens LICENSE-PLAN.md open.

## Gerichte M5-reviewreparaties — 27 oktober 2026

Op basis `448974b1352493fba41158da08daf1ad1937d973`: MMIO-/DMA-testadressen worden uitsluitend aan attackerrol 3 verstrekt in respectievelijk case 14/15; dienst, client, supervisor en PCI-case 16 krijgen ze niet. De succesvolle responsecontrole begrenst lengte, sector en slice vóór bytevergelijking en meldt afwijkingen via `s.unexpected` aan de bestaande `M5:FAIL:USER_OR_PROTOCOL`-afhandeling. ADR-026 heeft primaire bronnen en ontbrekende templatevelden; de capability-README verwijst naar de ingecheckte corekopie. Verouderde `core/tests.txt` en `core/clippy.txt` zijn verwijderd; actuele bewijsbestanden zijn behouden.

Op deze reparaties opnieuw uitgevoerd in Ubuntu 26.04.1 met Rust 1.90.0/QEMU 10.2.1/OVMF 2025.11: `python3 -u tools/test_milestones.py --through m5` en `python3 -u tools/test.py`, beide exit0; samen 77 controlegroepen/64 VM-runs. De 11 hostharnesstests slagen; beide nieuwe regressiegroepen falen terecht tegen de oude bron. Twee aanvullende echte gastproeven met responslengte 0 en `u32::MAX` eindigen gecontroleerd met `M5:FAIL:USER_OR_PROTOCOL`/raw35, zonder panic; de synthetische images blijven bytegelijk. Inventaris, kernel-rustfmt, documentatie-/diffcontroles en de onafhankelijke gerichte codecontrole zijn afgerond. Exacte opdrachten, bron-/artifacthashes en bewijsgrenzen: [review-repairs.json](verification/m5/review-repairs.json). Het eerdere Windowsbewijs hierboven is behouden. Geen contract- of architectuurwijziging.

## Vervolg en herstel

M6a: begrensde WRITE/FLUSH en crashherstel op een synthetische wegwerpimage; de actieve status en nog uit te voeren volledige poort staan hierboven. Werkqueue: WORK-QUEUE.md. Contract: implementation/M6A-WRITE-RECOVERY.md. M1–M5-baseline en eerder bewijs blijven behouden; de volledige QEMU-poort is na deze merge niet opnieuw uitgevoerd. Herstel via aparte consistente checkout c630c59; geen destructieve reset of hostconfigmigratie.
