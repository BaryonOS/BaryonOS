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
| K03 | kernel_m2 + product_acceptance (Sol) | kernel/ + crates/kernel-core/ respectievelijk userspace/ | geslaagde K02 | implementatie actief | KS-04–08 vereist; root owns tools/build_users.py en image-invoer |
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

- M2 op `a256951`: acht groepen, twintig hosttests, zeven QEMU-runs. Serial/exit/artifacthashes opnieuw gecontroleerd; bewijs `docs/verification/m2-m4/m2/`. De eerste mislukte run en daaropvolgende cacheattribuut-/aliasfix staan in de Git- en lokale runhistorie.
- M3 code aanwezig, volledige rootpoort nog niet uitgevoerd. Gerichte echte proeven voor normal/peer-write/XMM, B exit7 en ongeldige return-RSP slagen volgens builderlogs. Reviews hebben exit/faultobserver van generieke procesafhandeling gescheiden, monotone u64-PID's, gecontroleerde returnvalidatie en volledige resourcebalans aangescherpt.
- Nog vóór de M3-rootpoort: expliciete debugregisterinitialisatie en veilige idle/IRQ0-route na exit van alle processen. Kernelbuilder bezit het buildslot; root wijzigt alleen eigen tools/docs. Geen M4-start.
- U03: probe modes0–16, zes int80-sites, dertien artifacttests groen; actuele bron/hashes onder `userspace/verification/`. Hostbewijs is geen CPL3-bewijs. Root ondersteunt vijftien afzonderlijke M3-procescases plus normale fairness en idleproef.
- P02V en Q01V zijn gericht hersteld en privé gecheckpoint. Native 200%-zoom is door de Computer Use URL-controle gestopt; geen geverifieerde zoommutatie. Zoom, schermlezer en mensenproef blijven open.
- Volgende rootactie: na kernelwritepause broncheckpoint pushen, `python -u tools/test_milestones.py --through m3` draaien, reviewbevindingen sluiten en bewijs bewaren. Alleen bij geslaagde M3-poort wordt M4 expliciet vrijgegeven.
