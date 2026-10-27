# Actuele projectstatus

Bijgewerkt: 27 oktober 2026. Eigen kernel, ontwikkelfase M1–M4.

Repository: https://github.com/simonvanieperen/nieuw-os (privé). Werkbranch: `codex/m2-m4-isolation`. Kernel-/integratiebron: `ed09818e0f80b3a755237b813783b662f9153faf`. Concept-PR: https://github.com/simonvanieperen/nieuw-os/pull/2, basis `codex/first-build`. Main bevat nog de startspecificatie. De laatste bewijs-/documentatiecommit volgt bovenop de genoemde bronversie.

## Hoofdresultaat

De samengestelde M2–M4-poort is daadwerkelijk geslaagd op deze bron: 42 controlegroepen en 36 echte QEMU-runs. De aanvullende volledige M1-bootfoutregressie en afsluitende overdrachtsaudit staan op dit tussencheckpoint nog open. Het actieve doel wordt pas daarna afgesloten.

## Geïmplementeerd en getest

- Eigen UEFI-loader en Rust no_std-kernel; BootInfo1.1, geen bestaande kernel als basis.
- M2: eigen framebeheer, CR3/GDT/TSS/IDT, W^X inclusief fysieke aliases, bewaakte stacks en gecontroleerde exceptions. Echte RO/NX/null/UD/#DF-IST-proeven.
- M3: private ring-3-adresruimten, int80/iretq, PIC/PIT-preëmptie, twee16-worker-rondes en lokale procesbeëindiging. A-fault/B-voortgang, B exit7, invalid-RSP en veilige idle met acht IRQ0-wakeups. Mappingmetadata biedt2048 plaatsen per proces; echte67-page-image heeft alle BSS-tailbytes nul.
- M4: capability-rechten, lokale generatiehandles, delegatie/revocatie en atomaire Message320-IPC. Drie echte userprocessen, normale exchange en KS09–16-cases. Revoked queuecaps annuleren het hele bericht; exit trekt autoriteit in vóór frames worden vrijgegeven.
- Materiaal blijft een lokaal browserprototype; qsim blijft een lokale hostsimulator. Geen native UI-/NAS-/AI-/QPU-integratie geclaimd.

## Werkelijk testbewijs

| Opdracht / laag | Resultaat | Bewijs |
|---|---|---|
| `python -u tools/test_milestones.py --through m4` | exit0; M2 8 groepen/7 VM, M3 22 groepen/19 VM, M4 12 groepen/10 VM | `verification/m2-m4/{m2,m3,m4}/suite.json` en guest-runs.json |
| M2-hosttests | 6 bootprotocol +7 ELF +14 kernel-core geslaagd | m2/m2-host-tests.* |
| M3-host/artifacttests | 14 kernel-core +13 gewone user +6 large-image geslaagd | m3/*host/artifact-tests.* |
| M4-host/artifacttests | 21 capability-core +14 kernel-core +13 IPC-artifacttests geslaagd | m4/*host/artifact-tests.* |
| `python tools/capture_gate.py m2/m3/m4` (drie opdrachten) | exit0; serial, raw-exitverwachtingen en alle artifacthashes opnieuw gecontroleerd | genoemde suite-/guestrapporten |
| Capability-core extra controles | fmt, Clippy-Dwarnings, no_std: exit0; 2.560 wire-bitmutaties/800 transacties | verification/m2-m4/capability-core/report.json |
| Materiaal | 23 Node-tests en gerichte browsermetingen geslaagd; bron ongewijzigd sinds P02V | ../ui/prototype/verification/ |
| Quantum | 25 Rust-tests, vijf fixtures, Clippy/no_std; bron ongewijzigd sinds Q01V | verification/m2-m4/quantum/ |

Hosttests zijn per fase opnieuw uitgevoerd; de aantallen zijn geen som van unieke tests. Global1024-node-uitputting en maximale generations zijn hostmodelbewijs. QEMU-KS15 toetst queue16,255 capslots,depth8/9 en capaciteit na close. De testvingerafdruk is diagnostisch; hosttests vergelijken de volledige Engine-toestand.

## Starten

Vanuit de projectroot in PowerShell:

```powershell
python tools/build.py --kernel-feature milestone-m4
python tools/run.py --smoke --milestone m4
```

Dit is de normale eindige ontwikkelproef: de kernel laat synthetische processen berichten uitwisselen en sluit QEMU na controle af. Er is nog geen interactieve OS-shell/native desktop. De volledige testreeks herstelt na foutvarianten de gewone M1-image; bouw daarom het gewenste profiel expliciet. Zie DEVELOPMENT.md voor Materiaal op loopback en de quantum-CLI.

## Reviews en concrete grenzen

Gerichte tweede-agentreviews en herstel: `verification/m2-m4/m3-review.md` en `m4-review.md`; geen menselijke audit. Privilege-/geheugenbewijs geldt voor qemu64/TCG, één CPU,256MiB. AVX veroorzaakt #UD; geen FP-contextswitchingclaim. Gastcycli zijn geen realtimegarantie. NMI is niet geïnjecteerd; SMP, DMA/IOMMU, sidechannels, fysieke hardware, Secure Boot, crypto/PQC, drivers en opslag zijn niet bewezen.

Echte200%-browserzoom, schermlezer- en menselijke taakproeven blijven open. Computer Use stopte de native zoomcontrole doordat de actuele Chrome-URL op Windows niet betrouwbaar kon worden vastgesteld; geen geverifieerde zoommutatie. Quantum heeft geen eigen-OS-port, QPU of OS-afgedwongen netwerkdeny-bewijs. Geen echte NAS/AI-diensten of privacy-LP-tests geclaimd.

Geen publicatie/release of licentiekeuze uitgevoerd. LICENSE-PLAN.md bewaart dat open besluit vóór publieke open-sourcepublicatie. Geen hostdisk, firmware, NAS, productiekeys of globale Codexconfig gewijzigd. De bestaande lokale `.codex/environments/` blijft buiten Git behouden.

## Vervolg en hervatten

De werklijst en gates staan in WORK-QUEUE.md; de eis-voor-eis-matrix staat in verification/m2-m4/ACCEPTANCE.md. Eerst de genoemde afsluitende M1-regressie/audit afronden. Na volledige afsluiting volgt M5: een eerste user-space-systeemdienst met geïsoleerde virtio-testimage en expliciet DMA-/drivercontract. Geen echte hardware- of opslaginstallatie zonder nieuwe opdracht.

Eerdere basis: eerste batch op `144d952` en concept-PR#1. De historische bewijzen in verification/boot-suite.json, quantum-report.json en browser-report.json blijven behouden; de nieuwe m2-m4-rapporten hebben eigen bronversies en gemeten uitkomsten.
