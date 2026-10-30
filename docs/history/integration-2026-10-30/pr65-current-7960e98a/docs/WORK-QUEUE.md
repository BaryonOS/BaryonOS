# Werkqueue — volledige native testrelease, 30 oktober 2026

## Actuele integratie en exclusieve schrijfgebieden

Integrator `simon/native-test-version`: gepusht bewijscheckpoint `c532ba88`,
geteste code `eabc27ba879fd3b5960863ae8eede800c5045a57`, runtime v3
`a1998a126a3a1ccc18523925a422912d934c14bab865b3d7df346862a2c251a6`.
De onderstaande oudere toewijzingen zijn historie, geen actieve reserveringen.
IPC-wachten/consumenten, B02 en native TPM zijn geïntegreerd; normale GUI,
save/list/reopen en koude probeheropening zijn op hun eigen bron bewezen.
Alle zes volledige gewone zevenprocessenframes zijn nu op eabc27b native
bytegelijk aan de opnieuw uitgevoerde referentie, met diskorakel na raw0.
Bewijs: `verification/full-release-interactive-fixed/`.

| Taak | Eigenaar / exclusieve writes | Toestand en bewijsgrens |
|---|---|---|
| WINDOWS-INTERACTIVE-1 | peer `Keuze native os-functionaliteit`; alleen eigen `docs/verification/windows-qemu-interactive-1/`, eigen tijdelijke adapter/bundel/state | Vaste c532ba8-kopie in eigen worktree. Lokale ZIP en echte Windows CheckOnly/start/readiness gemeld; bediening loopt nog, geen geverifieerde eindclaim of publicatie |
| INTERACTIVE-COLD-1 | integrator; voorlopig alleen sessieartifacts en eigen gegenereerde cold-experiment-runs, daarna zo nodig `tools/run_materiaal_preview.py` en directe tests | Eerste exacte lijstvergelijking rood: verse Ready-status verschilt terecht van vorige Saved-status. Tweede verkennende run4019c620f8 bewijst volledig bytegelijk reopened-frame en onveranderde disk na QMP-quit/raw0, maar certificeert de initiële lijst niet (expliciete vaste wachttijd). Nog geen volledige koude acceptatie; renderer-afgeleid lijstorakel en strikt parent/artifact/trace-replay vereist. Geen kernel-/app-/SDK-writes |

De Windowspeer heeft de vaste build read-only gekopieerd en alle bron/copy-
hashes gecontroleerd; verder integratorwerk verandert haar vaste bron niet.
Geen parallelle schrijvers voor de gedeelde voortgangsmeter/status.
De integrator heeft de Materiaal-/toegankelijkheidssessie expliciet van deze
runnerreservering op de hoogte gebracht. Score blijft34%;50% open.

## Uitvoeringshistorie

Afgeronde fundamentstap `TPM-NATIVE-1`, eigenaar `simon/native-test-version`, vanaf
`cf02055`: alleen vaste supervisor-MMIO-adapter en read-only TPM2-property-
proef met echte PIT-polls, afzonderlijk absent-geval en bestaande volledige
CPL3-wachtacceptatie. Exclusieve writes: root/kernel Cargo-manifest/lock,
exacte pagingseam, M3-TPM-fixture/testhook, waitfixture-observer,
`tools/build.py`, `tools/run.py`, nieuwe `run_tpm.py`/hosttest en bewijsdocs.
Geen keyring/system-opslagwijzigingen; actieve PR #64 bezit dat gebied.
23 coretests, 4 harnastests en scalarbuild/ELF-inspectie groen. Beide echte
native VM's slagen op `4b09869`: propertyquery/release en nul-write-absent-
quarantaine, met volledige CPL3-wachtacceptatie. Eerste rode poging behouden;
`verification/native-test-version/tpm-evidence.json`. ADR-052; fundament
voor FR-B01/FR-O05, geen gesloten criterium. Integrator herhaalt nu de
gedeelde gates/proefversie; nieuwe CI-step nog niet uitgevoerd.
Voorafgaande sessiestap: alle zes echte K05-cases, inclusief twee Waiting-
kills en schoon hergebruik, groen op `586c33c`; brongebonden bewijs in
`verification/native-test-version/wait-lifecycle-evidence.json`.
De normale/koude native GUI en FPU/wacht zijn eveneens inmiddels groen op
hun eigen bron; oude rode queue-omschrijvingen hieronder zijn historisch.

## Peerhistorie — Windows-QEMU-testversie, nu geïntegreerd

`simon/windows-qemu-test-version`, basis `ad2b981`; codecheckpoint `f777f7a`.
`TASK-WINDOWS-QEMU-TEST-VERSION` voegt een gecontroleerde ZIP-packager,
PowerShell-launcher en bron-/hashgebonden QMP-previewrunner toe voor het echte
zeven-app `native-system-materiaal`-profiel. De opgegeven map
`C:\Program Files\qemu` is het ingebouwde standaarddetectiepad. De launcher
vereist QEMU 10.2.1+, SDL en `rng-builtin`; QEMU zelf wordt niet meegeleverd.

Hosttests (31: packager, transcript/frame-orakel en bestaande Materiaal-probe),
strikte preflight en normale Materiaal-build slagen; de laatste build is
waarschuwingvrij. De eerste QEMU-preview faalde door een onterechte vaste
Source/Policy-volgorde in het hostorakel. Na correctie slaagt de echte run
in 3,874 s: QEMU exit 0 na QMP-quit, 800×600 en 310 kleuren.
Bewijs: `verification/windows-qemu-preview/`. Bundelgeneratie, GitHub-artifactjob en
Windows-uitvoering staan nog open; de uiteindelijke bron wordt eerst met
`simon/native-test-version` afgestemd. De lokale QEMU-build heeft QMP en
`rng-builtin`, maar uitsluitend de `none`-displaybackend, zodat dit geen
Windows-UI- of SDL-bewijs is. Geen volledige-release- of roadmapcriteria
afgetekend; rubric blijft 34%.

Windows-preflightaanvulling: echte PowerShell bereikt de opgegeven
QEMU-installatie (11.1.0), met SDL en `rng-builtin`. Vijf launchertests met
synthetische bestanden slagen zonder VM of state; `-CheckOnly` valideert
bundelgrootte/SHA-256. Bewijs: `verification/windows-qemu-preview/windows-launcher.md`.
Dit is geen Windows-gaststart of handmatige bediening. PR #65 is concept.

Packager verifieert ook de ingelezen kernel-/loaderbytes en bron opnieuw
vlak vóór archivering. 33 hosttests groen, inclusief artifactwissel na
initiële validatie: fout zonder ZIP. Geen OS-bundel gemaakt.
Historische CI41089090576 op f777 is rood (herstelde readinessvolgorde);
huidige-head-CI blijft open.

Eerstvolgende gate:
`. .tools/nieuw-os/rtc-env.sh && PATH="$PWD/.tools/nieuw-os:$PATH" NOS_BUILD_DIR="$PWD/build/windows-qemu" NOS_TARGET_DIR="$PWD/target/windows-qemu" CARGO_TARGET_DIR="$PWD/target/windows-qemu" python3 -B -u tools/run_materiaal_preview.py --bios-rom .tools/nieuw-os/emulator/usr/share/seabios/bios-256k.bin --vga-rom .tools/nieuw-os/emulator/usr/share/seabios/vgabios-stdvga.bin`.

Afgeronde deelstap `RTC-NATIVE-1`: `simonvanieperen-expert-disco`, basis
`5c334f9ac3100f1c0e503e4f0ac4353a4a35154e`. De opt-in `native-rtc`-bootstrap,
`kernel/src/m2/rtc.rs`, RTC-buildkeuze, RTC-runner/hosttests/workflow en
`RTC-WALLTIME.md` zijn op die bron bewezen. Uitvoeringsbron `c02675e`: vijf
RTC-kalenderboots, zeven M2-regressie-VM's, M6c-smoke en hostgates slagen;
firmware-epoch/2100 blijft expliciet geweigerd.

Afgeronde deelstap `TASK-NATIVE-CLOCK-1`: opt-in ABI 1.9 syscall `0x73`,
SDK-type met expliciet `UnauthenticatedRtc`, en echte ring-3-positief- en
negatiefproef in de bestaande systeemapp. De native QEMU-gate slaagde met
gecontroleerde UTC-basis, twee echte reads, monotone tickcontrole en
`-22`-argumentweigering; brongebonden bewijs staat onder
`verification/system-clock/`. Het is een directe FR-N03/FR-S03-afhankelijkheid,
geen vertrouwde tijd, TLS-klok of gesloten roadmapcriterium.
[Brongebonden bewijs](verification/rtc-native/README.md); concept-PRreview,
geen samenvoeging of volledige productrelease geclaimd. De andere
zichtbare sessie beheert uitsluitend een root-workspace-doctestblokkade.
De oudere agentnamen hieronder zijn historische toewijzingen, geen actuele
claim dat hun agents draaien. `main` op deze basis bevat PR #58/#60/#61;
de actuele gewogen score is 34%, niet de historische 32% hieronder.

Integratie-eigenaar root, branch `codex/full-test-release`; laatste native foundationbron `844ec40`, gezamenlijke bron `9596abc` (negen CI groen, GUI rood) in `codex/native-product-integration`, PR #60.
[Volledige opdracht/matrix](FULL-TEST-RELEASE.md) → [actuele uitvoering en exacte hervatstap](PROJECT-STATE.md).
Werkpakketten zijn implementatiestappen; de 62 vaste `FR-*`-criteria blijven open. De gewogen ontwikkelscore blijft 32%.

| Werkpakket | Toestand | Eigenaar / exclusieve writes | Acceptatie | Interface en bewijsgrens |
|---|---|---|---|---|
| B02-PERSIST-1 | geïmplementeerd; native FR-I01-gate (normaal + koude herstart) zie PROJECT-STATE | root; keyring wire/adapter, system init/app/protocol, system.rs manifest, run_system/volume_disk/capture | FR-O05, FR-B02 (deel), FR-O01 | KEY1-opcodes 11/12 alleen via apart admin-ingress (ADR-051); keystate-commit vóór ciphertextcommit; vloer is gedemonstreerd mechanisme, geen vertrouwd anker; B02 blijft open |
| TASK-NATIVE-CLOCK-1 | implementatie gebouwd; native QEMU-gate volgt | root; RTC-bridge, M3 syscall, SDK, systeem-app en `run_system_clock.py` | FR-S03, FR-N03 | ABI 1.9 syscall 0x73; echte ring-3-reads en foutargument; uitsluitend `UnauthenticatedRtc`, geen TLS-/productklokclaim |
| TASK-DOC / TASK-CONTEXT | opdracht en aanvullende context vastgelegd | root; FULL/status/queue/rubric | alle | oorspronkelijke scope, stabiele ID's, formats/compatibiliteit en 100 cold/24 uur behouden |
| TASK-I01 / TASK-INT/GIT | foundation geïntegreerd op 844ec40 | root; gedeelde ABI/build/locks/Git | alle geraakt | negen CI groen op exact die bron; geen volledige productacceptatie |
| TASK-A03/R1 / TASK-KEYSDK-NATIVE | document-v7 en keyclient geïntegreerd | root | FR-A02, FR-A13, FR-O01–FR-O05, FR-B01 | native deelproeven; lost-BEGIN, owner-death en veilige disk-/sleutelpersistentie open |
| TASK-TCP | eigen native TCP-keten en cold pair groen | root | FR-N01, FR-N03, FR-H03 | HTTP/FIN/refused/reset/retry/timeout en strict capture; configuratie/TLS/appgrants open |
| DHCP-CODEC-1 | volledige hostgate op `30748d6`: fmt, 56/56 tests, Clippy en `x86_64-unknown-none`-releasebuild groen | `simonvanieperen-musical-happiness`; alleen `services/net-protocol/**` en statusdocs | FR-N01 | no_std DHCPv4 DISCOVER/selecting REQUEST/renewal REQUEST en begrensde OFFER/ACK/NAK-parser; tijdelijke host-GCC in `/tmp`; geen clienttoestanden, configuratie of native bewijs; 34% blijft staan |
| TASK-FPU/CLOCK / TASK-MEMORY / TASK-USER-ALLOC | geïntegreerd en native deelproeven groen | root | FR-K02, FR-K03, FR-L02 | eigen heap/guard/NX/OOM/FPU/context/clock; geen SMP- of algemeen hardwarebewijs |
| TASK-DISPLAY-INPUT / TASK-DISPLAY-PROBE / TASK-DISPLAY-CI | geïntegreerd en native CI groen | root | FR-U01, FR-K03, FR-H01 | echte PS/2/QMP, pixels en negatieve cap/lifetimegevallen; volledige desktop open |
| TASK-INPUT / TASK-INPUT-DAEMON / TASK-SURFACE/APP / TASK-GUI-INIT | componenten in GUI-branch | root; integratie shared files | FR-U01–FR-U05, FR-A02, FR-H01 | echte Source/Policy/Comp/Init/App-processen, begrensde IPC; brede sessie en crashherstart open |
| TASK-GUI-QMP / TASK-MATERIAAL-NATIVE | 5e echte Materiaal-/editorframes; 1fd regressie; keten rood | root; root-materiaal | FR-U01–FR-U05, FR-A02, FR-O01 | 200 seconden vast; Save/Reopen/afsluiten nog niet bewezen |
| TASK-REV-APP | gevonden stale-ready/entryfouten hersteld | gerichte onafhankelijke review afgerond | FR-U01, FR-A02, FR-K03 | host-/bronreview; volledige native GUI-keten blijft vereist |
| TASK-RANDOM-NATIVE / TASK-KEYRANDOM | geïntegreerd, vier echte gevallen opnieuw in CI op 844 | root | FR-B01, FR-N03 | onderhouden CSPRNG, echte virtio-RNG, quota/clock/cancel/revoke; persistente productkeys/TLS open |
| TASK-IMAGES | decoder native groen op 844, onafhankelijk hergevalideerd | root; image-codec component ade6 | FR-A03 | PNG/JPEG, heap/stack/pixels/negatieven; viewer, volledige formaten, native OOM en mid-decode cancel open |
| TASK-FOUNDATION-CI | alle vijf native stappen echt groen, 844 | root | FR-B01, FR-A03 | afzonderlijke builds, geen skips of genegeerde fouten; raw artifactmetadata bewaard |
| TASK-TPM-CRB | geïntegreerde pure transport-/commandocore | root; component 6bed190 | FR-B01, FR-B03, FR-O05 | host-/targetbewijs; native MMIO, virtuele TPM, ankertransacties en sleutelherstel open |
| TASK-RTC / TASK-RTC-V | core geïntegreerd, review afgerond | root | FR-N03, FR-S03 | 11 hosttests en onafhankelijke datumvectoren; UnauthenticatedRtc is geen vertrouwde certificaattijd |
| TASK-TLS-PORT | component 34191bb in eigen tls-port worktree | wachtrij voor verdere integratie | FR-N03, FR-P01–FR-P06 | no_std rustls/ML-KEM/mTLS host en target; native random/signer/trusted-time/TCP-keten open |
| TASK-BROWSER-PORT | concreet Chromium/Gecko-onderzoek a2085b9 gereed | wachtrij voor rootintegratie | FR-A05 | echte portproeven geblokkeerd, engine_selected null; verplichte browser blijft open |
| TASK-CAP-PERF / CAP-DIGEST-TCG-2 | productieherstel geïntegreerd; gezamenlijke GUI-keten blijft rood | kernel_storage; capability-core en digestcontract, basis 952e | FR-K03, FR-U01 | 5337500 geïntegreerd: reference sneller onder TCG; alle controles behouden; echte gezamenlijke GUI-herhaling volgt |
| GUI-BENCH-REVIEW-1 | P2-harnasfix geïntegreerd en onafhankelijk groen | security_release; run_cap_bench.py en test_cap_bench.py, basis 342387e | FR-K03, FR-U01 | exacte native artifacts correct; stale-index/inputdrift/initial-VARS/disk-readorder hersteld; native herhaling volgt |
| TASK-CALCULATOR / CALCULATOR-REVIEW-1 | component 0254206 geïntegreerd, onafhankelijke review groen | root; shared multiappgrants nog open | FR-A08 | 17+4 IPC-hostgroepen en 26424 exacte oraclecases; target groen; echte multiapp-/QMP-bediening open |
| MULTIAPP-CONTRACT-1 | actief | native_ui_apps; nieuw NATIVE-MULTIAPP.md, basis 48b | FR-U01–FR-U05, FR-A08 | twee echte CPL3-apps, minimale grants, focus/dirty/retire/lifetime; root bezit gedeelde code |
| PRODUCT-MERGE-REVIEW-1 | afgerond; configuratie-P2 gericht hersteld | git_build_audit; c91714b | alle gedeelde interfaces | gerichte controle foundation/GUI/build/manifest, geen root-WIP-writes |
| IPC-WAIT-1 | in uitvoering; WIP nog niet native getest | kernel_storage; core/SDK/m3/waitmodule/feature, basis 9596 | FR-K03, FR-U01 | 16 endpoints, absolute deadline, veilige wake/revoke/lifetime en terugkeerbare IRQ-idle; rootnativegate vereist |
| WAIT-CONSUMERS-IDLE-1 | in uitvoering; SDK-afhankelijkheid open | security_release; Key/Volume service en nieuw contract | FR-K03, FR-B01, FR-O01 | alleen echte idlepass; oude ABI/devicepolling en no-replay/deadlines behouden |
| MULTIAPP-CODEC-1 | hostchecks groen, target en integratie volgen | native_ui_apps; nieuw sdk/desktop-control en contract | FR-U01, FR-A08 | APPCTRL2 en beperkte Switch, geen native Pause/Resume-blitclaim |
| NATIVE-TLS-PROBE-1 | wachtrij, threadlimiet verhinderde start | nog niet actief | FR-N03, FR-P01 | volgende vrije worker: echte native TLS-/signerketen, geen voorspelbare RNG of hostsucces |

Volgend verplicht werk blijft zichtbaar: volledige compositor/sessie/toegankelijkheid,
alle apps en appbeheer, veilige persistente opslag en TPM-ankering, netwerkconfiguratie
met TLS, NAS/providersynchronisatie, lokale inference, boot/PQC, updates/herstel en
integrale productvalidatie. Dit zijn geen naar later verplaatste releasevereisten.
Fysieke en menselijke gates blijven open; onafhankelijk bron-/VM-werk gaat door.
Uit historische queues hieronder mag geen actieve agentcapaciteit worden afgeleid.

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
