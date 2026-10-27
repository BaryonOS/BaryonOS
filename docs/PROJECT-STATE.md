# Actuele projectstatus

De bovenste fase geeft de actuele ontwikkeling weer. De oudere faseverslagen hieronder bewaren hun toenmalige implementatie- en bewijsgrenzen.

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
Deze bron is vóór de lange controles op GitHub bewaard. De afsluitende wijzigingen
betreffen documentatie en bewijs. [PR #10](https://github.com/simonvanieperen/nieuw-os/pull/10)
staat boven de M6b-branch; geen merge naar main of release uitgevoerd.

| Scope op deze bron | Resultaat | Bewijs |
|---|---|---|
| Volledige M6c-gate en capture | exit 0; 19 groepen / 15 VM-invocaties | `verification/m6c/` |
| Volledige M6b-regressie en capture | exit 0; 18 groepen / 14 VM-invocaties | `verification/regression-m6c/` |
| Catalog-core en harnas | 30 Rust- en 33 Python-tests; fmt/Clippy/no_std exit 0 | `verification/m6c/`, `verification/m6c-review/core-final/` |
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

