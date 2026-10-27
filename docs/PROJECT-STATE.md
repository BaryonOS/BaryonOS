# Actuele projectstatus

Bijgewerkt: 27 oktober 2026. Eigen kernel, ontwikkelfase M1–M4.

Repository: https://github.com/simonvanieperen/nieuw-os (privé). Werkbranch: `codex/m2-m4-isolation`. Kernel-/integratiebron: `ed09818e0f80b3a755237b813783b662f9153faf`. Concept-PR: https://github.com/simonvanieperen/nieuw-os/pull/2, basis `codex/first-build`. Main bevat nog de startspecificatie. De laatste bewijs-/documentatiecommit volgt bovenop de genoemde bronversie.

## Reviewfollow-up

De bewijsarchiefcapture valideert een volledige nieuwe opname in een tijdelijke map en laat het bestaande archief bij validatie- of installatiefouten ongemoeid. Alleen stdoutlogs onder `build/logs/` kunnen als hostbewijs worden opgenomen; gastresultaten doorlopen hun eigen controles. `python -B -X utf8 tests/test_capture_gate.py` (4 tests) slaagt. De registratie van vijf `.png`-bestanden als JPEG gebruikt nu hun signatures: `python3 tools/update_evidence_files.py` en `python -B -X utf8 tests/test_evidence_files.py` (4 tests) slagen. Alle controles exit 0; dit is hosttool-/bewijsregistratie, geen nieuwe browser- of OS-test. De grote-probehostcontrole is lokaal geblokkeerd: Rust 1.90.0 en `x86_64-unknown-none` ontbreken; `tools/preflight.py` meldt ook geen QEMU/OVMF.

## Hoofdresultaat

De ontwikkelfase M1–M4 is geïmplementeerd en getest. De samengestelde M2–M4-poort slaagt op deze bron: 42 controlegroepen en 36 echte QEMU-runs. De aanvullende volledige M1-bootfoutregressie op `622c2ee` slaagt met tien groepen en acht extra VM-runs, inclusief de bewuste60s-timeout. De eindmatrix en het onafhankelijke V04-bewijsoverzicht staan onder verification/m2-m4/. De laatste bewijs-/documentatiecommit is de commit die dit bestand bevat.

## Geïmplementeerd en getest

- Eigen UEFI-loader en Rust no_std-kernel; BootInfo1.1, geen bestaande kernel als basis.
- M2: eigen framebeheer, CR3/GDT/TSS/IDT, W^X inclusief fysieke aliases, bewaakte stacks en gecontroleerde exceptions. Echte RO/NX/null/UD/#DF-IST-proeven.
- M3: private ring-3-adresruimten, int80/iretq, PIC/PIT-preëmptie, twee16-worker-rondes en lokale procesbeëindiging. A-fault/B-voortgang, B exit7, invalid-RSP en veilige idle met acht IRQ0-wakeups. Mappingmetadata biedt2048 plaatsen per proces; de grotere image heeft35 imagepages +32 heap/stackpages =67 usermappings en alle BSS-tailbytes zijn nul.
- M4: capability-rechten, lokale generatiehandles, delegatie/revocatie en atomaire Message320-IPC. Drie echte userprocessen, normale exchange en KS09–16-cases. Revoked queuecaps annuleren het hele bericht; exit trekt autoriteit in vóór frames worden vrijgegeven.
- Materiaal blijft een lokaal browserprototype; qsim blijft een lokale hostsimulator. Geen native UI-/NAS-/AI-/QPU-integratie geclaimd.

## Werkelijk testbewijs

| Opdracht / laag | Resultaat | Bewijs |
|---|---|---|
| `python -u tools/test_milestones.py --through m4` | exit0; M2 8 groepen/7 VM, M3 22 groepen/19 VM, M4 12 groepen/10 VM | `verification/m2-m4/{m2,m3,m4}/suite.json` en guest-runs.json |
| `python -u tools/test.py` | exit0; tien groepen, acht VM-runs en dertien hosttests | verification/m2-m4/m1/ |
| Normale M4-build + `run.py --smoke --milestone m4` | exit0/raw33; extra CLI-start, loader/kernel bytegelijk aan volledige M4-gate | verification/m2-m4/manual-m4.json |
| M2-hosttests | 6 bootprotocol +7 ELF +14 kernel-core geslaagd | m2/m2-host-tests.* |
| M3-host/artifacttests | 14 kernel-core +13 gewone user +6 large-image geslaagd | m3/*host/artifact-tests.* |
| M4-host/artifacttests | 21 capability-core +14 kernel-core +13 IPC-artifacttests geslaagd | m4/*host/artifact-tests.* |
| `python tools/capture_gate.py m1/m2/m3/m4` (vier opdrachten) | exit0; serial/stderr, raw-exitverwachtingen en alle artifacthashes opnieuw gecontroleerd | genoemde suite-/guestrapporten; negatieve M1-guestrecords blijven false |
| Capability-core extra controles | fmt, Clippy-Dwarnings, no_std: exit0; 2.560 wire-bitmutaties/800 transacties | verification/m2-m4/capability-core/report.json |
| Materiaal | 23 Node-tests en gerichte browsermetingen geslaagd; bron ongewijzigd sinds P02V | ../ui/prototype/verification/ |
| Quantum | 25 Rust-tests, vijf fixtures, Clippy/no_std; bron ongewijzigd sinds Q01V | verification/m2-m4/quantum/ |

Hosttests zijn per fase opnieuw uitgevoerd; de aantallen zijn geen som van unieke tests. Global1024-node-uitputting en maximale generations zijn hostmodelbewijs. QEMU-KS15 toetst queue16,255 capslots,depth8/9 en capaciteit na close. De testvingerafdruk is diagnostisch; hosttests vergelijken de volledige Engine-toestand.

Voor Materiaal en quantum zijn de negen respectievelijk vijftien geregistreerde bron-/binaryhashes opnieuw vergeleken: alle gelijk. De ongewijzigde tests zijn daarom niet opnieuw uitgevoerd; eerdere resultaten blijven aan exact die bestanden gebonden.

## Starten

Vanuit de projectroot in PowerShell:

```powershell
python tools/build.py --kernel-feature milestone-m4
python tools/run.py --smoke --milestone m4
```

Dit is de normale eindige ontwikkelproef: de kernel laat synthetische processen berichten uitwisselen en sluit QEMU na controle af. Er is nog geen interactieve OS-shell/native desktop. De volledige testreeks herstelt na foutvarianten de gewone M1-image; de laatste afzonderlijke CLI-start heeft daarna het M4-profiel gebouwd/gecontroleerd. Bouw bij hervatten het gewenste profiel expliciet. Zie DEVELOPMENT.md voor Materiaal op loopback en de quantum-CLI.

## Reviews en concrete grenzen

Gerichte tweede-agentreviews en herstel: `verification/m2-m4/m3-review.md` en `m4-review.md`; geen menselijke audit. Privilege-/geheugenbewijs geldt voor qemu64/TCG, één CPU,256MiB. AVX veroorzaakt #UD; geen FP-contextswitchingclaim. Gastcycli zijn geen realtimegarantie. NMI is niet geïnjecteerd; SMP, DMA/IOMMU, sidechannels, fysieke hardware, Secure Boot, crypto/PQC, drivers en opslag zijn niet bewezen.

Echte200%-browserzoom, schermlezer- en menselijke taakproeven blijven open. Computer Use stopte de native zoomcontrole doordat de actuele Chrome-URL op Windows niet betrouwbaar kon worden vastgesteld; geen geverifieerde zoommutatie. Quantum heeft geen eigen-OS-port, QPU of OS-afgedwongen netwerkdeny-bewijs. Geen echte NAS/AI-diensten of privacy-LP-tests geclaimd.

Geen publicatie/release of licentiekeuze uitgevoerd. LICENSE-PLAN.md bewaart dat open besluit vóór publieke open-sourcepublicatie. Geen hostdisk, firmware, NAS, productiekeys of globale Codexconfig gewijzigd. De bestaande lokale `.codex/environments/` blijft buiten Git behouden.

## Vervolg en hervatten

De werklijst en gates staan in WORK-QUEUE.md; de eis-voor-eis-matrix staat in verification/m2-m4/ACCEPTANCE.md. De volgende implementatiefase is M5: een eerste user-space-systeemdienst met geïsoleerde virtio-testimage en expliciet DMA-/drivercontract. Geen echte hardware- of opslaginstallatie zonder nieuwe opdracht.

Eerdere basis: eerste batch op `144d952` en concept-PR#1. De historische bewijzen in verification/boot-suite.json, quantum-report.json en browser-report.json blijven behouden; de nieuwe m2-m4-rapporten hebben eigen bronversies en gemeten uitkomsten.
