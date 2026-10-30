# Werkqueue — volledige native testrelease, 2026-10-30

Integratie-eigenaar root, branch `codex/full-test-release`, main-basis `73ebc1e`.
[Volledige opdracht/matrix](FULL-TEST-RELEASE.md) → [actuele uitvoering](PROJECT-STATE.md).
Werkpakketten heten `TASK-*`; de vaste `FR-*`-ID's hieronder zijn productcriteria.
Een afgeronde deelimplementatie tekent geen volledig productcriterium af.

| Werkpakket | Toestand | Eigenaar / exclusieve writes | Acceptatie-ID's | Interface / bewijsgrens |
|---|---|---|---|---|
| TASK-DOC | geïntegreerd | root; FULL/status/queue/rubric | alle | opdracht intact, 62 open rijen; 32% ongewijzigd |
| TASK-I01 | root985 dertien VMs/zeven CI;15e entryfix vijf nieuwe cases+zeven CI groen | root | FR-K02,FR-K03,FR-O04,FR-B01 | bron/hash/ESP/disk gebonden, geen release |
| TASK-A03/R1 | native v7documentsessie geïntegreerd | root | FR-A02,FR-A13,FR-O01–FR-O03 | lost-BEGIN/ownerdeath-cleanup blijft open |
| TASK-TCP | root61ea nativepair + strictcapture groen | root; netwerkbuilder leverdecore/app/peer | FR-N01,FR-N02,FR-H03 | 21TX/13RX,HTTP/FIN/refused/reset/retry/300ticktimeout,strictcapture; TLS/config/appgrants open |
| TASK-KEYSDK-NATIVE | root61ea nativepair + strictcapture groen | root; keyclient/app/Cargo | FR-B01,FR-O05 | crypto2/2/9+record1/1/10/2; veilige disk/keypersistentie blijftopen |
| TASK-INPUT | core/review+echte PS2transportprobe groen; verbonden daemonketen open | root | FR-U01–FR-U05,FR-H01 |20hosttests/target; geen werkelijk inputservice-/compositorbewijs |
| TASK-FPU/CLOCK | root61ea native groen | root | FR-K02,FR-K03 |3levens/157save=restore/#UD/reuse/100Hz, geen fullCPU/hardwareclaim |
| TASK-MEMORY | 15e C-entryfix vijf nieuwe native cases+CI groen | kernel_storage; m3.rs,nieuwheap.rs,paging.rs,SDKmemory/native/tests | FR-K02,FR-K03,FR-L02,FR-N02 | ABI1.7sys4voorstel, atomicfallibleownRW/NXzeroheap8MiBquota; Geenanderewritersruntime-native.rs |
| TASK-USER-ALLOC | 42d4bc9 geïntegreerd;15 hosttests/collecties en targetbuild groen | native_ui_apps; sdk/heap-allocator/**,USER-ALLOCATOR.md | FR-K02,FR-L02,FR-A05,FR-N02 | gelockte no_stdallocator/eigenarena/align/OOM/stats; geenhostfallback |
| TASK-TLS-PORT | actief; tls-portworktree | network_nas_ai; services/tls-engine/**,NATIVE-TLS.md | FR-N02,FR-P01–FR-P06 | onderhoudenrustls/no_stdproviderport, exactePQC/CA/name/pin/mTLS; random/signer/heap/trustedtime gates |
| TASK-KEYRANDOM | actief; keyring-randomworktree | security_release; keyringcore/serviceadapter,nieuwrandomclient/contract | FR-B01,FR-N02 | random-onlyingress/op10, HKDF-opruimreview3299 groen, native ingress nogopen, onderhoudenCSPRNG/health/quota/clockfailclosed; rootcapbinding/nativebewijsvolgt |
| TASK-RTC | core68dd9f5 geïntegreerd; native adapter open | git_build_audit; drivers/rtc/**,RTC-WALLTIME.md | FR-N02,FR-S03 | 11 hosttests + onafhankelijke14 met17.892datums, target; kwaliteitUnauthenticatedRtc, geen certtijdclaim |
| TASK-RTC-V | afgerond, alleen lezen/eigentarget | acceptance_contract | FR-N02,FR-B04 | gerichtegrensreview/tegenvoorbeelden; geenmenselijkeaudit |
| TASK-CONTEXT | aanvullende20secties verwerkt | root; FULL/context/queue/progress | alle | stabiele62IDs, UC/compatmatrix/formaat/100cold/24uur; oorspronkelijkeopdracht intact |
| TASK-MATERIAAL-NATIVE | component1f36+review; stale-ready/entryreparatie vóórGUI | native_ui_apps; eigen nieuwe UI-worktree/bestanden na contract | FR-U01–FR-U05,FR-A02,FR-O01 | echte userspace renderer/editor; geen hostprototype als nativebewijs |
| TASK-DISPLAY-INPUT | d528 echte native transportprobe+zeven CI groen | git_build_audit; eigen nieuwe transportworktree/bestanden na contract | FR-U01,FR-K03,FR-H01 | cap04d geïntegreerd/52-reviewtests; GOP hardware los; adapters in uitvoering, root bezit ABI/hooks/build |
| TASK-INT/GIT | 985/15e/d528 exact gepubliceerd met eigen nativebewijs+zeven CI | root; sharedABI/build/locks/status/Git | allegeraakt | GitData lease/treecontrole; 985 alle13VMs+7CI groen; entryfix, desktop en RANDOM-native vervolg |

| TASK-INPUT-DAEMON | in uitvoering; input-daemon | kernel_storage; nieuwe sdk/examples/input-service/** en contract | FR-U01–FR-U05,FR-H01 | twee echte ELF-targetbuilds, hostbackpressure; echte kernel-PID/Ready/PEEK-ACK; native gate open |
| TASK-SURFACE/APP | in uitvoering; materiaal-native-app | native_ui_apps; nieuwe SUR1/input-client/Materiaalapp en mousechecked-constructor | FR-U01–FR-U05,FR-A02 | gescheiden compositor, geen raw display/input voor app, native verbonden keten open |
| TASK-DISPLAY-PROBE | d528 echte QMP/PS2/pixelprobe groen; CI-uitbreiding volgt | git_build_audit; nieuwe drie-CPL3-fixture en nieuwe run/testtools | FR-K03,FR-U01,FR-H01 | echte QMP-keys/mouse/pixels, PID/epoch/vollecopycontrole; root integreert en voert uit |
| TASK-RANDOM-NATIVE | in uitvoering; random-native-probe | security_release; nieuwe fixture en gericht RNG-replypad | FR-B01,FR-N02 | vier native gevallen, echte Virtio-RNG/CSPRNG, revoke/cancel/backpressure; root bootstrap/locks/build |
| TASK-BROWSER-PORT | component a2085b9 gereed, rootintegratie volgt | network_nas_ai | FR-A05 | echte upstreamconfig/header/linkproeven exit2; port geblokkeerd, geen enginekeuze |
| TASK-IMAGES | 9ae1c73 component/oracle/stackfix; nativeobserver in uitvoering | network_nas_ai; nieuwe services/image-codec/** +nativeprobe | FR-A03 | echte PNG/JPEGdecode, native heap en pixeloracle; engine/profielen en viewer open |

| TASK-GUI-INIT | in uitvoering; desktop-init | kernel_storage; nieuwe sdk/examples/desktop-init/** | FR-U01–FR-U05,FR-K03 |7 echte processen, Source/SYNC/Policy/Comp/focusgates vóór GO; nativebootstrap root |
| TASK-GUI-QMP | in uitvoering; materiaal-qmp | native_ui_apps; nieuw runner/tests/contract | FR-U01–FR-U05,FR-A02,FR-O01 | echte key/mouse/edit/save/cold-heropen; geen draftinject; adapterreparatie eerst |
| TASK-REV-APP | actief; alleenlezen/eigentarget | acceptance_contract | FR-U01,FR-A02,FR-K03 | onafhankelijke tegenvoorbeelden adapters; stale-ready gevonden, hertest nafix |
| TASK-DISPLAY-CI | in uitvoering; display-ci | git_build_audit; één nieuweworkflow | FR-K03,FR-U01,FR-H01 | echte nativeprobe en standalonehostchecks, behoud overigeCI/cleantree |

Volgende verplichte werk blijft zichtbaar: schaalbaar geheugen, echte
compositor/sessie/toegankelijkheid, audio en alle apps/engineports, persistente
beveiligde opslag en TPM-ankering, TLS/netwerkconfiguratie, NAS/providersync,
lokale inference, boot/PQC/updates/herstel en complete productvalidatie.
Fysieke/humane gates blijven releaseblokkerend op hun concrete bewijsniveau;
onafhankelijk bron-/VM-werk gaat door. Geen actieve capaciteit uit historische
queues hieronder afleiden.

## Historische registratie

# Werkqueue — 29 oktober 2026

Actuele geteste bron `53e0d57af99c9673c97bc900fae15b98e08c8ca3`, PR #54.
Root integreert; onafhankelijke worktrees en buildmappen zijn gebruikt.
Historische registraties hieronder zijn geen actieve capaciteitsclaims.

| ID | Toestand | Eigenaar / exclusief gebied | Resultaat / afhankelijkheid |
|---|---|---|---|
| B01-R1 | afgerond, geïntegreerd getest | keyring_repairs: services/keyring/** | Blijvende Health, monotone intrekking; 15 tests en extra tegenvoorbeelden groen |
| B01-R2 | afgerond, geïntegreerd getest | entropy_timeout: keyringbroker/app/harnas | Echte timeout/reset-ACK, aparte herstart, native oude-blobintrekking; 5/5 gate |
| B01-V | afgerond | security_verifier: alleen lezen | Geen blocker; 615 hosttests plus lint/fmt/voortgang groen |
| S08-R | afgerond, geïntegreerd getest | storage_resume: m6c-/volumebroker | Err-completionmeting; 13/13 gezamenlijke gate; aparte targeted cases groen |
| ENV | afgerond | emulator_setup: uitsluitend .tools/** | Officiële lokale QEMU10.0.13/OVMF, preflight groen |
| INT | lokale controles afgerond | root: docs/voortgang/bewijs/GitHub | 27 VM-runs groen; geschikte merge uitsluitend na finale PR-checks |
| K05-SLOTS | lokaal afgerond, eindintegratie volgt | root + storage_resume, security_verifier | 5/5 native, 21 negatieve controles; exact56b576b, geen nieuw punt |
| B02 | gereed om te bouwen | volgende eigen branch | Persistente geauthenticeerde sleutel-/volumeopslag; live herstel is geen koude antirollback |
| S09 | wacht op bron of nieuwe implementatie | niet toegewezen | Historische lokale branch niet remote gevonden; geen geïntegreerde implementatie claimen |

## Vroege productvalidatie — consumentenadoptie

De literatuurverkenning in
[CONSUMER-VALUE-2026-10-28.md](research/CONSUMER-VALUE-2026-10-28.md)
levert vier onderzoeksstappen naast de native bouw. Deze taken gebruiken geen
agentcapaciteit totdat zij werkelijk worden uitgevoerd en tellen niet als
roadmappunt zonder het bewijs van het betreffende acceptatiecriterium.

| ID | Toestand | Onderzoeksdoel | Afhankelijkheid / grens |
|---|---|---|---|
| PRX1 | gereed voor onderzoek | 12–18 taakinterviews/-observaties om twee of drie kerntaken en overstapvoorwaarden vast te leggen | Praktische pilotomvang; geen statistische representativiteitsclaim |
| PRX2 | wacht op PRX1 en bruikbaar DV-01-prototype | circa 24–36 deelnemers; terugvinden, twee gereedschappen, herstel en rechten vergelijken | Taakprestatie en aantrekkelijkheid apart meten; definitieve omvang na pilot/poweranalyse |
| PRX3 | wacht op stabiele kernflow | eerlijke productbeloftes vergelijken op zelfstandig uitgevoerde taak, niet alleen klikgedrag | Zelfde prototypekwaliteit, prijs-/beschikbaarheidsinformatie en vooraf gekozen primaire uitkomst |
| PRX4 | wacht op bruikbare afgebakende workflow | vierweekse vroege veldproef met vrijwillig hergebruik, fouten, herstel, hulp en terugvalredenen | Geen claim van langdurige retentie; 90-/180-dagenvalidatie blijft latere marktproef |

DV-01 in [PRODUCT-SPEC.md](implementation/PRODUCT-SPEC.md) blijft het
geïntegreerde Materiaal-scenario. U06 blijft open totdat representatieve
gebruikers de afgesproken kerntaken daadwerkelijk uitvoeren en relevante
frictie binnen productbudgetten is opgelost.

## Historische registratie

# Werkqueue — native M6b-opslagfouten

28 oktober 2026, basis `1f24f99`, branch `codex/m6b-storage-errors`.

| ID | Eigenaar | Exclusieve writes | Status |
|---|---|---|---|
| K07 | m6a_architect_review | kernel/Cargo.toml, kernel/src/m2/m3/m6b.rs, userspace/m6b_probe.rs | generieke Unknown-observerreparatie en twee statussnapshotinjecties klaar |
| G07 | root | tools/tests/docs/Git | 31 hosttests, 22 groepen/18 VM en capture exit 0; nieuw bewijs apart gearchiveerd |
| R07/V07 | m6a_environment | alleen lezen | bronreview geen blockers; onafhankelijke hash-/schijf-/recoveryaudit |

Vereist: geen mutatie-ACK of live-stateclaim na onzekere I/O, geen replay,
GET/PUT/DELETE fail-closed, canoniek foutantwoord en overlevende processen.
Twee gerichte native foutcases en de volledige M6b-gate met beide koude
foutrecoveries slagen. Historisch bewijs behouden. Geen nieuwe rechten,
dependency, schijfformaat of architectuur. Bron `592dccf`; bewijs en nog open
backend-/timeout-/LIMIT-grenzen staan in PROJECT-STATE.md.

## Oudere werkqueue — historisch
# Werkqueue — M6d native foutafhandeling en productvoortgang

## Afgeronde uitvoering M6d — 28 oktober 2026

Uitvoeringsbron `88e6c04be218b8b09ea4f0a226f6b9719da706a6`, branch
`codex/m6d-io-recovery-roadmap`, PR #11. Contract:
[M6D-IO-UNCERTAINTY.md](implementation/M6D-IO-UNCERTAINTY.md).

| ID | Werkverdeling | Afgerond resultaat |
|---|---|---|
| A01/A02 | architecture_next_step / kernelobserver, native probe, integratie twee hosttools | Unknown zonder valse ACK; acht native foutfeatures; upstream hostlogbinding behouden |
| A03 | architecture_next_step / M6b-capture en bijbehorende tests | begrensde alternatieve bestemming; drie negatieve/positieve bestemmingschecks |
| D01/D02/D03 | progress_roadmap / scoremodel en historische documentintegratie; later alleen advies | vaste 100 punten, bron-/bewijsgrenzen en concreet herstelvervolg |
| R08/R09 | native_review / aanvankelijk alleen lezen, daarna zelfstandige audit buiten de repo | twee vroege harnasbevindingen hersteld; 31 M6d-runs, 274 artifacthashes en 16 images gecontroleerd |
| A04 | architecture_next_step / afzonderlijke read-only M6b-bewijsaudit | 14 M6b/M1-runs, 121 artifacthashes, zeven volumes en zes cold-links gecontroleerd |
| G08/V08 | root / gedeelde interfaces, integratie, uitvoering, score en Git | 35/31 plus 18/14, beide captures en normale eindbuild geslaagd; S07 geaccepteerd, 13% → 14% |

Eén schrijver per bestand en één build/QEMU-slot. Geen nieuwe OS-kernelbasis,
capabilityverruiming, schijfformaat of externe dependency. Exacte uitslagen en
bewijs: [M6D-ACCEPTANCE.md](verification/M6D-ACCEPTANCE.md). Iedere volgende
opdracht herbeoordeelt de vaste meter volgens AGENTS.md.

### Eerstvolgende native stap — begrensde catalogusrecovery

Voorgesteld, nog niet gebouwd of als punt geteld:

1. Eén geautoriseerde supervisor mag alleen deze catalogusdienst vervangen;
   gewone apps krijgen geen reset-, processtart- of blokrechten.
2. Oude servicegeneratie, endpoints en blokhandles intrekken vóór vervanging.
   DMA-geheugen pas hergebruiken na bevestigde reset; anders quarantaine.
3. Nieuwe instantie herstelt de echte journal zonder write-replay. Apps blijven
   actief, krijgen binnen hun bestaande rechten nieuwe endpoints en lezen de
   actuele revisie terug; Unknown wordt nooit alsnog een verzonnen succes-ACK.
4. Herstartbudget begrenzen; stale caps, onbevoegde supervisor, corrupte journal,
   herhaalde uitval en ontbrekende reset-ACK als negatieve gevallen opnemen.
5. In dezelfde VM testen en de relevante M6d-regressie herhalen. Deze beperkte
   proef voltooit niet automatisch algemeen dienstbeheer K05 of opslagherstel S08.

Native Materiaal, grotere documenten en ruimtehergebruik houden daarnaast hun
eigen productcriteria. Geen sleutelgeneratie zonder native entropybron.

## Historische M6c-queue

### Historische M6c-uitvoering — 28 oktober 2026

Basis `bb22fdb`, branch `codex/m6c-named-catalog`. M6a en M6b zijn al afgerond
volgens de versiegebonden bewijsarchieven in PROJECT-STATE.md. De oudere queue
hieronder bewaart uitsluitend de toenmalige werkverdeling.

| ID | Eigenaar / schrijfgebied | Afgerond resultaat |
|---|---|---|
| C07-core | catalog_core / services/catalog core en tests | namen, v4-wire, v2-journal, catalogus; 30 hosttests en no_std groen |
| C07-native | catalog_native / nieuwe serviceadapter, probe en kernelobserver | finale native gate: 19 groepen / 15 VM-invocaties inclusief M1 |
| G07 | root / gedeelde config, tools, harnas en docs | onafhankelijk schijforakel; 33 hosttests; checkpoint bc4cd104 vóór lange controles |
| R07 | context_review / alleen lezen | 15 scenario's van echte adapter via host-ABI-shim; geen native devicefoutinjectie |
| V07 | root + reviewers | M6c-capture, M6b 18 groepen / 14 VM, acht volumes en 130 artifacthashes geaudit; brongebonden integratiecontrole |

Alle eindgates en captures hebben exit 0. Bron, exacte commando's, host-/native
grenzen en bewijslinks: [M6C-ACCEPTANCE.md](verification/M6C-ACCEPTANCE.md).
De M6c-eindbuild is bytegelijk aan de twee normale eindgate-runs. De reviews zijn
AI-reviews, geen menselijke audit of formeel bewijs.

Vervolg: native Materiaal-koppeling en beveiligingsafhankelijkheden
uit VERIFIABLE-SECURITY-AND-PQC blijven afzonderlijke stappen; geen automatische
sleutelgeneratie zonder entropy of grafische/native NAS-claim door deze catalogus.

## Historische queue vóór afgeronde M6b

## Parallel N01 — internet en persoonlijke NAS

Gebruikersuitbreiding 27 oktober 2026; privacycontract revisie 3. Bron vanaf `a96ad45`, branch `codex/nas-private-sharing`.

| ID | Gebied | Status/bewijs |
|---|---|---|
| N01-C | services/nas/ | echte hostencryptie, ciphertextstore, quota, revoke/evict en netwerkbeleidsmodel; 50 tests groen |
| N01-P | ui/prototype/ | drie NAS-flows en afzonderlijke lokale beheerrol; 39 tests groen, nieuwe visuele browsercontrole open |
| N01-R | alleen-lezen review | vier concrete bevindingen hersteld; gerichte hercontroles groen |
| N01-G | root/docs/Git | herziene productgrens, ADR-027, integratiebewijs 89 tests + demo |
| N02 | services/nas/transport.mjs | begrensde echte hybride TLS-hostproef; actuele resultaten onder verification/pqc-transport/ en PROJECT-STATE.md |
| N03 | toekomstige native integratie | netwerkstack/TLS-port/identiteit, clientkeys/multi-device, NAS-rol en socketcapabilities nog open |

Resultaten tellen niet als M6a-/M8-QEMU- of echte-internetbewijs. Eerste kernelvervolg blijft de hieronder beschreven M6a-poort.

De aangeleverde formele-/PQC-blauwdruk is verwerkt in `implementation/VERIFIABLE-SECURITY-AND-PQC.md`: V01 voor machinegecontroleerde capability-invarianten, P01–P04 voor native build/entropy/image-/opslagintegratie en Q02 voor een afzonderlijke Grover-onderwijsproef. Deze taken zijn gespecificeerd, niet reeds uitgevoerd. N02 voegt geen eigen cryptografische tunnel of bestaande OS-kernel toe.

27 oktober 2026, basis `448974b`, branch `codex/m6a-write-recovery`. Gekozen invariant/ABI/cases vóór implementatie: implementation/M6A-WRITE-RECOVERY.md.

| ID | Eigenaar | Exclusieve writes | Status |
|---|---|---|---|
| K06 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | bron bevroren; 16 hosttests en gerichte QEMU/cut/recoveryproeven groen |
| C06 | quantum_contract | capability-core/,virtio-core/ | 30 + 21 tests/fmt/Clippy/no_std exit 0; bron gepauzeerd |
| U06 | product_acceptance | services/block/,nieuwe M6-userspace | 20 hosttests + 12 artifacttests exit 0; echte dienstbron gepauzeerd |
| G06 | root | config/tools/tests/docs/Git | guarded writable image/crashharnas/capture,19 hostcontroles groen; volledige poort gereed |
| R06 | C06/U06 | alleen lezen | gerichte transport- en zes harnas/capturebevindingen hersteld/gesloten |
| V06 | later verifier | alleen lezen | finale M1–M6-poorten, echte cuts/recovery en hashbinding |

Eén schrijver per bestand, maximaal drie agents en één gedeeld build/QEMU-slot. Geen nieuwe externe dependency. Na elke afgebakende stap bron/bewijs/open punten privépushen; noreply-afzender alleen per Git-commando, geen globale configwijziging. Gerichte M6-proeven groen; finale 54 groepen / 49 VM-poort en regressie nog uit te voeren. QEMU-procescrash is geen fysieke stroomuitvalclaim.

## M5 — behaalde basis (historisch)

27 oktober 2026. Bron `7af544a`, basis `c630c59`, branch `codex/m5-block-service`, concept-PR #3. Definitieve fingerprint v3: c7455ebe2dfcd0e4337d058f440712d60b3ae600d2953c385ee81ba6d69a0307.

| ID | Eigenaar | Gebied | Status / bewijs |
|---|---|---|---|
| G05 | root | config/tools/docs/Git | geïntegreerd; preflight exit 0, private checkpoints, einddocs |
| K05 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | gereed; M5 25 groepen/20 VM exit 0 |
| C05 | quantum_contract | virtio-core/,capability-core/ | gereed; 12/26 tests, fmt/Clippy/no_std exit 0 |
| U05 | product_acceptance | services/block/,M5-userspace | gereed; 9 protocol- en 11 artifacttests exit 0, echte gastdienst |
| R05 | twee reviewers | alleen lezen | herstelpunten gesloten; ongeïnjecteerde randen expliciet |
| V05 | root/verifier | ruwe evidence/hashbinding | 77 groepen/64 VM plus M5-eindstart; alle 65 VM en 279 artifacthashes geaudit |

Opdrachten, resultaten, grenzen en hervatroute: PROJECT-STATE.md en verification/m5/ACCEPTANCE.md. Eerste mislukte gate blijft failed bewaard; de finale bron is opnieuw getest. Bestaande UI/quantum en M1–M4-baseline behouden.

Volgende stap: de M6a-bronpoort en regressie hervatten vanaf deze checkpoint. De M6a-specificatie, huidige teststatus en open punten staan in PROJECT-STATE.md. Geen echte hardware/disks/NAS/keys, betaalde infrastructuur, publieke release of merge zonder passende opdracht. Licentie vóór publicatie blijft open.

