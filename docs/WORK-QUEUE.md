# Werkqueue — ontwikkelversie M1–M4

Actieve fase gestart op 27 oktober 2026 vanaf `144d952`.
Doel: eigen boot, twee werkelijk geïsoleerde ring-3-processen, begrensde foutafhandeling waarbij het andere proces doorloopt, en geteste capabilities/IPC. M2, M3 en M4 zijn opeenvolgende poorten; een checkpoint sluit de fase niet af.

## Eindcriteria vóór implementatie

1. M1 blijft betrouwbaar booten met echte QEMU-logs, tijdslimiet, foutstatus en ongewijzigde host-/privacygrenzen.
2. M2: eigen framebeheer, vierlaags paging met 4 KiB-pages, eigen GDT/TSS/IDT en exceptions. KS-01–03 bewijzen unieke/eigenaarsgebonden frames, nulmaken, uitputting, gereserveerde gebieden, supervisorgrens en W^X inclusief fysieke aliases. Werkelijke page faults worden getest. M3 start pas na geslaagde M2-poort.
3. M3: eigen statische userprogramma's, ten minste twee gescheiden adresruimten, gecontroleerde int0x80/iretq-grens en PIT/PIC-preëmptie. KS-04–08 bewijzen disjuncte frames, A/B-patronen, verboden toegang/instructies, 16-worker-fairness, opruimen en B-voortgang na A-fault/exit. M4 start pas na aantoonbare procesisolatie.
4. M4: capabilities en 320-byte IPC volgens het volledige contract, vaste quota, generation zonder wrap, rechtenvermindering, delegatie, revocatie en lifecycle. KS-09–16 hebben hosttransactietests én relevante echte ring-3/QEMU-proeven; een fout verandert geen userbuffer/queue/captable gedeeltelijk.
5. Materiaal en qsim blijven afzonderlijke hostprototypes. Afgeronde implementatie niet herhalen; resterende contract-/toegankelijkheidscontroles gericht uitvoeren. Niet-uitvoerbare menselijke/hardwarechecks blijven zichtbaar en tellen niet als geslaagd.
6. Geïntegreerde versie bouwt met vaste lockfiles; onafhankelijke agentreview, herstel en actuele regressie-/eindtests. Broncode, compacte logs/hashes, besluiten, startcommando's en open punten staan op de privéwerkbranch.

## Werkverdeling en afhankelijkheden

| ID | Eigenaar | Exclusief schrijfgebied | Afhankelijk van | Status | Bewijs |
|---|---|---|---|---|---|
| G00 | hoofd | rootconfig, tools/, docs/ | bestaande toestand | afgerond | preflight exit0; bestaande M1/P02/Q01-rapporten aanwezig |
| K02 | kernel_m2 (Sol) | kernel/, crates/kernel-core/ | G00 | eindregressie geslaagd op ed09818 | acht groepen,27hosttests,zevenQEMU-runs; m2/suite.json |
| P02V | product_acceptance (Sol) | ui/prototype/ | bestaande P02 | afgerond binnen beschikbare controles | 23/23 Node-tests; skiplink/draft/focus/typezoek/288px-inspector echt gecontroleerd; 200%-zoom, schermlezer en mensenproef open; ui/prototype/verification/ |
| Q01V | quantum_contract (Sol) | experiments/quantum/ | bestaande Q01 | code/test afgerond; reviewer R02 | CDF-fix: oude code faalt nieuwe regressie, huidige 25tests/5fixtures groen; 17-caseaudit onder docs/verification/m2-m4/quantum/ |
| K03 | kernel_m2 + product_acceptance (Sol) | kernel/ + crates/kernel-core/ respectievelijk userspace/ | geslaagde K02 | eindregressie geslaagd op ed09818 |22groepen/19VM;14core/13probe/6largehosttests; m3/suite.json |
| K04K | kernel_m2 (Sol) | kernel/, crates/kernel-core/ | geslaagde K03 | geslaagd op ed09818 |12groepen/10VM,21cap+14core+13IPCartifacttests; m4/suite.json |
| K04C | quantum_contract (Sol) | crates/capability-core/ | geslaagde K03 | code/hostreview afgerond; nu R04K read-only | 21 hosttests, fmt/Clippy/no_std; a91e64e; docs/verification/m2-m4/capability-core/ |
| U04 | product_acceptance (Sol) | userspace/ | geslaagde K03 | afgerond; R04C engine-review gesloten | 13 IPC- en 6 large-artifacttests; a91e64e; userspace/verification/ |
| R04 | R04C/R04K (Sol) | alleen lezen | geïntegreerde M4 | afgerond | concrete errororder/observerfindings hersteld; m4-review.md |
| V04 | hoofd + product_acceptance (Sol) | tools/, docs/verification/ | R04 en herstel | eindaudit afgerond; overdracht vastgelegd in deze commit |52acceptatiegroepen/44VM,hashcontrole,eindmatrix enprivatecheckpoint |

## Werkafspraken

- Maximaal drie actieve subagents, Sol; één schrijver per bestand. De hoofdchat behoudt zijn ingestelde modelinspanning/snelheid. Geen budget- of rechtenverhoging.
- Root beheert Cargo-workspace, lockfile en build-/QEMU-harnas. Builders melden interface-/featurebehoeften vóór integratie.
- Geen productiegegevens, hostdisks, firmwareflash, publieke release of betaalde diensten.
- Na iedere betekenisvolle poort en vóór lange controles: status/bewijs committen en privébranch pushen. Niet-uitgevoerde tests blijven open. Geen groene marker zonder waargenomen invariant.
- Hervat vanuit deze queue en PROJECT-STATE.md; verander het einddoel niet op basis van reeds afgerond deelwerk.

## Laatste checkpoint

De gehele afgesproken M1–M4-ontwikkelfase is geïmplementeerd, gereviewd en getest. Kernelbron: ed09818. Samengestelde gate exit0: M2 achtgroepen/zevenVM, M3 22groepen/19VM, M4 twaalfgroepen/tienVM. Aanvullende volledige M1-regressie op622c2ee:exit0,tiengroepen/achtVM inclusief echte60s-timeout. Alle52groepen/44VM hebben afgebakend bewijs; geen hosttests alsVM meegeteld.

Capture van m1/m2/m3/m4 controleert serial/stderr/rawexpectations/artifacthashes; negatieve M1-records blijven false. V04 bevestigt de fasevolgorde en KS01–16-casedekking. ACCEPTANCE.md en FINAL-AUDIT.json geven de eindgrenzen. Materiaal/qsim-hashes blijven gelijk aan hun geteste bestanden; menselijke/hardware/OS-portcontroles blijven expliciet open.

De actuele commit bevat de laatste bewijs-/overdrachtsteksten en wordt privé gepusht op codex/m2-m4-isolation. Concept-PR#2 blijft open; main blijft de startspecificatie. Geen publieke release/licentiekeuze/hostconfig/hardwarewijziging.

Volgende implementatiefase na deze afgeronde opdracht: M5, eerste geïsoleerde systeemdienst/virtio-testimage met expliciet DMA-contract. Niet stilzwijgend gestart. De lokale gebruikersconfig .codex/environments/ blijft buiten Git behouden.
