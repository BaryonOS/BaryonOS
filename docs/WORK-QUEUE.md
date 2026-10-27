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
| K02 | kernel_m2 (Sol) | kernel/, crates/kernel-core/ | G00 | geslaagd op a256951 | acht groepen, twintig hosttests, zeven QEMU-runs; docs/verification/m2-m4/m2/ |
| P02V | product_acceptance (Sol) | ui/prototype/ | bestaande P02 | afgerond binnen beschikbare controles | 23/23 Node-tests; skiplink/draft/focus/typezoek/288px-inspector echt gecontroleerd; 200%-zoom, schermlezer en mensenproef open; ui/prototype/verification/ |
| Q01V | quantum_contract (Sol) | experiments/quantum/ | bestaande Q01 | code/test afgerond; reviewer R02 | CDF-fix: oude code faalt nieuwe regressie, huidige 25tests/5fixtures groen; 17-caseaudit onder docs/verification/m2-m4/quantum/ |
| K03 | kernel_m2 + product_acceptance (Sol) | kernel/ + crates/kernel-core/ respectievelijk userspace/ | geslaagde K02 | geslaagd op 6802b17 | 20 groepen/18 echte VM-runs; twaalf coretests en dertien user-artifacttests; m3/suite.json |
| K04K | kernel_m2 (Sol) | kernel/, crates/kernel-core/ | geslaagde K03 | implementatie actief | usercopy/syscalls/lifecycle en echte KS-09–16-observer |
| K04C | quantum_contract (Sol) | crates/capability-core/ | geslaagde K03 | code/hostreview afgerond; nu R04K read-only | 21 hosttests, fmt/Clippy/no_std; a91e64e; docs/verification/m2-m4/capability-core/ |
| U04 | product_acceptance (Sol) | userspace/ | geslaagde K03 | afgerond; R04C engine-review gesloten | 13 IPC- en 6 large-artifacttests; a91e64e; userspace/verification/ |
| R04 | onafhankelijke reviewer | eerst alleen lezen | geïntegreerde M4 | wacht | concrete negatieve tegenvoorbeelden en herstel |
| V04 | hoofd | tools/, docs/verification/ | R04 en herstel | wacht | volledige eindmatrix en remote commitcontrole |

## Werkafspraken

- Maximaal drie actieve subagents, Sol; één schrijver per bestand. De hoofdchat behoudt zijn ingestelde modelinspanning/snelheid. Geen budget- of rechtenverhoging.
- Root beheert Cargo-workspace, lockfile en build-/QEMU-harnas. Builders melden interface-/featurebehoeften vóór integratie.
- Geen productiegegevens, hostdisks, firmwareflash, publieke release of betaalde diensten.
- Na iedere betekenisvolle poort en vóór lange controles: status/bewijs committen en privébranch pushen. Niet-uitgevoerde tests blijven open. Geen groene marker zonder waargenomen invariant.
- Hervat vanuit deze queue en PROJECT-STATE.md; verander het einddoel niet op basis van reeds afgerond deelwerk.

## Laatste checkpoint

- Samengestelde rootpoort op `6802b17`: `python -u tools/test_milestones.py --through m3` exit0, M2 acht groepen/zeven VM-runs, M3 twintig groepen/achttien VM-runs. Alle exit/serial/artifacthashes daarna gecontroleerd door capture_gate. Bewijs onder `docs/verification/m2-m4/{m2,m3}/`.
- R03/R03V-bevindingen hersteld en gericht herbeoordeeld; zie `m3-review.md`. Idle na beide exits, acht IRQ0-wakeups, debuginit, u64-PID, user-return-context en exacte cleanup zijn meegenomen in de volledige gate.
- M4 nu expliciet vrijgegeven. Cap-engine, kernelintegratie en afzonderlijke IPC-userfixture hebben elk één schrijver. Root beheert workspace/lockfile/buildtools en testpoort. Geen gelijktijdige root-targetbuilds.
- M4-fixture: drie processen met eigen endpointroot/gereduceerde next-peer-cap, concrete scriptmodes copy/wire/snapshot/handles/rights/revoke/quotas/lifecycle. ABI/Message320 en foutvolgorde blijven bindend. Een hosttest bewijst geen CPL3-grens.
- P02V en Q01V zijn gericht hersteld en privé gecheckpoint. Native 200%-zoom is door de Computer Use URL-controle gestopt; zoom/schermlezer/mensenproef blijven open.
- Checkpoint a91e64e bewaart Engine, workspace/lockfile, drivers en IPC/large-fixtures. De M4-kernelintegratie en 2048-mappingarena zijn nog lokaal; hun volledige rootpoort is nog niet uitgevoerd. R04K toetst usercopy en foutvolgorde; de observer-read vóór auth is gericht hersteld.
- Volgende rootactie: na kernelwritepause complete bron pushen, `--through m4` en bootregressie draaien, bewijs vangen en de eindmatrix auditen. Gehele fase blijft actief tot M4 en eindregressie voldoen.
