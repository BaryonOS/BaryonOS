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
| G00 | hoofd | rootconfig, tools/, docs/ | bestaande toestand | actief | preflight exit0; bestaande M1/P02/Q01-rapporten aanwezig |
| K02 | kernel_m2 (Sol) | kernel/, crates/kernel-core/ | G00 | implementatie actief | frame-/mappingcore en eigen archlaag geschreven; nog geen build- of M2-poortbewijs |
| P02V | product_acceptance (Sol) | ui/prototype/ | bestaande P02 | afgerond binnen beschikbare controles | 23/23 Node-tests; skiplink/draft/focus/typezoek/288px-inspector echt gecontroleerd; 200%-zoom, schermlezer en mensenproef open; ui/prototype/verification/ |
| Q01V | quantum_contract (Sol) | experiments/quantum/ | bestaande Q01 | code/test afgerond; reviewer R02 | CDF-fix: oude code faalt nieuwe regressie, huidige 25tests/5fixtures groen; 17-caseaudit onder docs/verification/m2-m4/quantum/ |
| K03 | kernbuilder | later expliciet toegewezen | geslaagde K02 | geblokkeerd door poort | KS-04–08 vereist |
| K04 | later toegewezen | later expliciet toegewezen | geslaagde K03 | geblokkeerd door poort | KS-09–16 vereist |
| R04 | onafhankelijke reviewer | eerst alleen lezen | geïntegreerde M4 | wacht | concrete negatieve tegenvoorbeelden en herstel |
| V04 | hoofd | tools/, docs/verification/ | R04 en herstel | wacht | volledige eindmatrix en remote commitcontrole |

## Werkafspraken

- Maximaal drie actieve subagents, Sol; één schrijver per bestand. De hoofdchat behoudt zijn ingestelde modelinspanning/snelheid. Geen budget- of rechtenverhoging.
- Root beheert Cargo-workspace, lockfile en build-/QEMU-harnas. Builders melden interface-/featurebehoeften vóór integratie.
- Geen productiegegevens, hostdisks, firmwareflash, publieke release of betaalde diensten.
- Na iedere betekenisvolle poort en vóór lange controles: status/bewijs committen en privébranch pushen. Niet-uitgevoerde tests blijven open. Geen groene marker zonder waargenomen invariant.
- Hervat vanuit deze queue en PROJECT-STATE.md; verander het einddoel niet op basis van reeds afgerond deelwerk.

## Laatste checkpoint

Startcontrole: alleen de reeds bestaande ongetrackte `.codex/environments/` staat buiten Git; deze lokale instelling wordt behouden. De eerste batch is aanwezig op `144d952`; M2–M4 zijn nog niet geïmplementeerd. De tools zijn beschikbaar. Deze beurt is voortgang zodra queue/branch en concrete implementatie zijn vastgelegd; het vorige antwoord was een statuscontrole, geen verkleining van dit nieuwe doel.

Root-checkpoint: generiek QEMU-harnas met exacte marker-/exitverwachtingen en aparte M2-poort geschreven. Drie hosttests van de bewijsparser en Python-syntaxischecks slagen; dit is nadrukkelijk nog geen geslaagde M2-run.

K02 eerste echte VM-run: KS-01 slaagt, daarna kernelpanic vóór KS-02; rootpoort faalt expliciet. Diagnose/herstel door kernel_m2, geen M3-start. R02-review is parallel read-only actief.

Vervolg K02: OVMF-cacheattributen gericht hersteld; gewone M2-smoke nu groen. R02 vond onjuiste unmap-aliasklasse: code en negatieve regressie hersteld, bronreview akkoord, nieuwe tests nog uit te voeren. Extra echte double-fault/IST-proef wordt toegevoegd vóór de volledige poort. M3 blijft gesloten.
