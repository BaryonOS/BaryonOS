# Actuele projectstatus

## Hervatting M6a — 28 oktober 2026

Nieuwe werkbranch `codex/m6a-write-flush-core`, basis `5a58a74`. M6a is gestart
met een afgebakende eerste stap: zuivere WRITE/FLUSH-requestplannen in
`virtio-core`, nog zonder kernel-/driverintegratie of verruiming van rechten.
Contract en verdere acceptatie: [M6A-WRITE-FLUSH.md](implementation/M6A-WRITE-FLUSH.md).
De core-implementatie is klaar: aparte `Features`/`Operation`/`Plan`, exacte
header-/descriptorvalidatie en status-only-completions voor WRITE/FLUSH. Zeven
nieuwe tests plus alle twaalf bestaande virtio-coretests slagen (19 totaal).
Buildercontroles voor fmt, Clippy `-D warnings`, no_std en diffcontrole: exit 0;
exacte opdrachten en bronhashes onder `verification/m6a/first-step/`. Review en
geïntegreerde eindcontroles lopen; geen uitgevoerde WRITE/FLUSH-, persistentie-
of crashherstelproef. De eerdere M5-resultaten
hieronder blijven historisch bewijs op de daarbij genoemde bronversies.

Rust 1.90.0 en de cross-targets zijn aanwezig. De eerste inventaris miste
QEMU/OVMF; die zijn projectlokaal uit officiële Debian-pakketten ingericht.
Daarna slaagt `python3 tools/preflight.py --strict` (exit 0): QEMU 10.0.13,
OVMF 2025.02-8+deb13u1. Dit is een andere ontwikkelvariant; nog geen bootbewijs.
Installatie-/inventarisrapporten staan onder `build/m6a-environment/`, compact
bewaard in `verification/m6a/first-step/environment/`. De eerste M5-build en
smoke-start slagen (exit 0/raw 33), met het bestaande READ-schijforakel. Deze
start is een tussentijdse omgevingscontrole; de finale M5-gate volgt nog.

## Afgeronde M5-baseline (27 oktober 2026)

27 oktober 2026. M5 is geïmplementeerd en getest in het vaste QEMU-profiel.

Repository: https://github.com/simonvanieperen/nieuw-os. Branch `codex/m5-block-service`, concept-PR [#3](https://github.com/simonvanieperen/nieuw-os/pull/3). Basis `c630c59` (M1–M4); definitieve runtime-/harnasbron `7af544aa1b8d731ac27f384f0ba95aa44586a57d`. Latere commits bewaren documentatie en bewijs. Main bevat de startspecificatie; geen merge naar main of release uitgevoerd.

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

M6a: begrensde WRITE/FLUSH op wegwerpvolume met volle-schijf-, gedeeltelijke-write- en crashherstelproeven vóór bestandssysteem/updateketen. Nog niet gestart. Werkqueue: WORK-QUEUE.md. Contract: implementation/M5-BLOCK-SERVICE.md. M1–M4-baseline behouden onder verification/m2-m4/, inclusief oorspronkelijke PROJECT-STATE.md. Herstel via aparte consistente checkout c630c59; geen destructieve reset of hostconfigmigratie.
