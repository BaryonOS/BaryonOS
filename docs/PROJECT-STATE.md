# Actuele projectstatus

## Actief: M6a observercontrole — 28 oktober 2026

Branch codex/m6a-write-recovery; remote-basis 803bf43 is met fast-forward geïntegreerd, inclusief bestaande reviewherstellen. Andere M6b/M6c/M6d-worktrees blijven behouden. Deze opdracht scherpt uitsluitend de drie M6a-testcontroles aan en herhaalt de QEMU-poort; er is geen merge naar main of release uitgevoerd.

Historisch werkelijk groen: M6a op bron 8c53ea4, fingerprint v3 6fce5c12e87cf6b1a1353743357593de3d8b6843012b12be6deaf7fe17ce3b66: 54 groepen / 49 VM-runs, capture exit0. De oorspronkelijke lokale capture is bewaard onder verification/m6a/history/6fce5c12/. De aansluitende M1–M5-regressie stopte bij M3 met Windows-bestandsvergrendeling (os error32, build exit101); M1 werd daarna niet gestart.

Geïmplementeerd, nog niet in QEMU herbevestigd: (1) echte vierde FULL/ENOSPC-request, send en clientreceive eisen; (2) alle unieke foutverzoek-id's 100–114 en daadwerkelijke clientontvangst controleren; (3) terminale Unknown-reply plus exacte WRITE/FLUSH-publicaties eisen en extra publicaties na Unknown afwijzen. De eerdere groene gate bewijst deze aangescherpte controles nog niet.

Huidige uitvoering: python -X utf8 tools/preflight.py --strict, exit0; compiler, beide Rust-targets, QEMU en firmware aanwezig. De kernfix telt unieke request/send/receive-maskers, bewaart negatieve probes tot clientreceive en vergrendelt de WRITE/FLUSH-tellers vanaf Unknown. Crossbuild exit0 en onafhankelijke begrensde bronreview zonder blokkerende bevindingen. De volledige M6a-poort is opnieuw groen: 54 groepen / 49 VM-runs, bronfingerprint e721bec4c283c90fe6d05a66d092372cd59f7727815de79f0da18760155c365c; capture exit0. Acht QEMU-tegenproeven zijn exit0: twee oude false-greens gereproduceerd en zes nieuwe verwachte afwijzingen op exacte observerregels; drie daarvan zijn expliciete observerinstrumentatie. Definitieve M1–M5-regressie loopt nog. De 43 Python-hostcontroles slagen; twee Windows-regelafbrekingsfouten in synthetische capturedata zijn hersteld zonder de bytecontrole te verzwakken. Geen verzonnen testresultaten.

De terugkerende Codex-routingfout is afzonderlijk onderzocht. Een gecontroleerde privéback-up en diagnose staan buiten deze repository. De huidige appverbinding werkt; een permanente oplossing is niet aangetoond. Geen chats/databases/configuratie gewist, geen firewallwijziging of onbewezen herinstallatie uitgevoerd. Deze appdiagnose is geen OS-testbewijs.

Contract: implementation/M6A-WRITE-RECOVERY.md. Alleen synthetische 64KiB-wegwerpimages; WRITE-completion gevolgd door echte FLUSH-completion vóór ACK. Geen fysieke stroomuitval-, IOMMU- of productieclaim. Bestaande .codex/environments/ blijft buiten Git behouden.

## Historische M5-basis

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
