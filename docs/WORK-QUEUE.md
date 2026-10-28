# Werkqueue — 28 oktober 2026

Werkelijke capaciteit: vier agents tegelijk, inclusief hoofdagent. Eén schrijver
per gebied; aparte builds, targets, volumes, VM-processen en logbestemmingen.
De volledige native acceptatie blijft gebonden aan `3e41bbf9`. Het vervolg op
`f52ef0a6` bevat de documentsessie, jobcodec, capture en reviewreparaties.

| ID | Toestand | Eigenaar / exclusief gebied | Resultaat / kleinste afhankelijkheid |
|---|---|---|---|
| ROOT | in uitvoering | root: integratie, docs, GitHub en gedeelde buildhook | Zes gerichte native resultaten publiceren; PR15/14 afronden; G11-interface bewaken |
| G11 | in uitvoering | native_apps: userspace-probe en eigen SDK-builddeel | DocumentSession echt in ring3 voor openen, bewerken, bewaren en heropenen; bestaande catalogus en oracle |
| I13 | in uitvoering | runtime_platform: nieuwe document-session-poort en tests | Herbruikbare strikte normal/recover-acceptatie voor de G11-appbibliotheek |
| B10 | in uitvoering | native_storage: parallel-b10 | Zeven concrete PR14-bevindingen toetsen aan actuele bron en bewijs |
| B08/G10/I11/I12/ROOT | gereed voor integratie/controle | root, gescheiden bewijsbundels | Zes echte actuele VM-runs slagen; koude images en vijf grafische frames gecontroleerd |
| G07/I08/B09/G09/I10 | gereed voor integratie/controle | geïntegreerd op f52ef0a6 en dbd35b04 | Documentsessie, jobcodec en gerepareerde capture; gerichte reviews afgerond |
| G1 | wacht op een benoemde afhankelijkheid | bestaande andere sessie: Materiaal en display/input | Eigenaar publiceert broncheckpoint van toegewezen G1-branch voor integratie |
| S08 | gereed om te bouwen | volgende bouwer; root bewaakt herstelcontract | Begrensde vervanging van catalogusdienst zonder mutatiereplay |
| S04/S05 | wacht op een benoemde afhankelijkheid | root: opslagcontract | Versieerbaar formaat en crashacceptatie voor grotere documenten en ruimtehergebruik |
| N03/P02/H | wacht op een benoemde afhankelijkheid | nog toe te wijzen | Native netwerk, entropy/crypto en peerrechten voor native NAS |

Vier toestanden: gereed om te bouwen; in uitvoering; wacht op een benoemde
afhankelijkheid; gereed voor integratie/controle. De oude capaciteitsvoorkeuren
in de historische registratie gelden niet voor deze opdracht.

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
| K06 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | bron bevroren; 16hosttests en gerichte QEMU/cut/recoveryproeven groen |
| C06 | quantum_contract | capability-core/,virtio-core/ | 30+21tests/fmt/Clippy/no_std exit0; bron gepauzeerd |
| U06 | product_acceptance | services/block/,nieuweM6-userspace | 20host+12artifacttests exit0; echte dienstbron gepauzeerd |
| G06 | root | config/tools/tests/docs/Git | guarded writableimage/crashharnas/capture,19hostcontroles groen; volledige poort gereed |
| R06 | C06/U06 | alleen lezen | gerichte transport- en zes harnas/capturebevindingen hersteld/gesloten |
| V06 | later verifier | alleen lezen | finale M1–M6-poorten, echte cuts/recovery en hashbinding |

Eén schrijver per bestand, maximaal drie agents en één gedeeld build/QEMUslot. Geen nieuwe externe dependency. Na elke afgebakende stap bron/bewijs/open punten privépushen; noreplyafzender alleen per Git-commando, geen globale configwijziging. Gerichte M6-proeven groen; finale 54groepen/49VM-poort en regressie nog uit te voeren. QEMU-procescrash is geen fysieke stroomuitvalclaim.

## M5 — behaalde basis (historisch)

27 oktober 2026. Bron `7af544a`, basis `c630c59`, branch `codex/m5-block-service`, concept-PR#3. Definitieve fingerprint v3: c7455ebe2dfcd0e4337d058f440712d60b3ae600d2953c385ee81ba6d69a0307.

| ID | Eigenaar | Gebied | Status / bewijs |
|---|---|---|---|
| G05 | root | config/tools/docs/Git | geïntegreerd; preflight exit0, private checkpoints, einddocs |
| K05 | kernel_m2 | kernel/,kernel-core/,drivers/virtio-blk/ | gereed; M5 25 groepen/20 VM exit0 |
| C05 | quantum_contract | virtio-core/,capability-core/ | gereed; 12/26 tests, fmt/Clippy/no_std exit0 |
| U05 | product_acceptance | services/block/,M5-userspace | gereed; 9 protocol- en 11 artifacttests exit0, echte gastdienst |
| R05 | twee reviewers | alleen lezen | herstelpunten gesloten; ongeïnjecteerde randen expliciet |
| V05 | root/verifier | ruwe evidence/hashbinding | 77 groepen/64 VM plus M5-eindstart; alle 65 VM en 279 artifacthashes geaudit |

Opdrachten, resultaten, grenzen en hervatroute: PROJECT-STATE.md en verification/m5/ACCEPTANCE.md. Eerste mislukte gate blijft failed bewaard; de finale bron is opnieuw getest. Bestaande UI/quantum en M1–M4-baseline behouden.

Volgende stap: de M6a-bronpoort en regressie hervatten vanaf deze checkpoint. De M6a-specificatie, huidige teststatus en open punten staan in PROJECT-STATE.md. Geen echte hardware/disks/NAS/keys, betaalde infrastructuur, publieke release of merge zonder passende opdracht. Licentie vóór publicatie blijft open.

