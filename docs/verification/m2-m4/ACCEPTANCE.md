# Acceptatieoverzicht M1–M4

Dit overzicht onderscheidt uitvoering van nog te leveren bewijs. Scope: het eigen x86_64/QEMU-ontwikkelprofiel, één CPU, 256 MiB, synthetische gegevens. Tweede-agentreview is geen menselijke audit.

| Requirement | Status | Werkelijk bewijs / vereiste aanvulling |
|---|---|---|
| M1 eigen loader/kernel | Behaald op basischeckpoint; M2-regressie groen | eerdere `docs/verification/boot-suite.json`; `m2/suite.json` bevat de latere defaultboot |
| KS-01 kaart/reserveringen/overflow/unieke frames | M2 behaald op a256951 | echte kernelasserts + bootprotocol-/framehosttests, `m2/guest-runs.json` |
| KS-02 owner/zero/quota/uitputting | M2 behaald op a256951 | fysieke patroon/free/reassign/zero-proef en quota; hosttests voor verkeerde owner/uitputting |
| KS-03 eigen tabellen/W^X/supervisor | M2 behaald voor kernelprofiel | echte tablewalk/aliasaudit; aparte RO/NX/null-faultboots. Userheap/-stack volgt KS-06 |
| Eigen GDT/TSS/IDT/CR3 en guardstacks | M2 behaald binnen genoemde proeven | eigen CR3-readback, zes guards ongemapt, echte #DF via IST1 bij onbruikbare MAIN-stack; NMI niet geïnjecteerd |
| KS-04 A/B eigen adressen/frames/TLB | M3 behaald op 6802b17 | echte CPL3-/CR3-wissels, private patronen, disjuncte frames; m3/suite.json |
| KS-05 A kan peer/kernel niet benaderen | M3 behaald op 6802b17 | drie afzonderlijke echte faultboots, survivor ná fault en cleanup; m3/guest-runs.json |
| KS-06 NX/guards/CLI/FP-instructies | M3 behaald op 6802b17 binnen qemu64 | aparte stack/heap/guards/CLI/x87/XMM/AVX/UD-boots; AVX #UD is geen AVX-contextbewijs |
| KS-07 preëmptieve 16-worker-fairness | M3 behaald op 6802b17 | twee echte 16-worker-rondes, iedere selectie gecontroleerd, private tellers |
| KS-08 exit/fault/opruimen | M3 behaald op 6802b17 | per-death resourcebalans, B/A-voortgang, nonzero/vroege B-exit, invalid-RSP en idle na beide exits |
| User-ELF en syscallprobe | Host- en CPL3-laag behaald op 6802b17 | 13 artifacttests plus daadwerkelijke int80-foutcodes/registersentinels/DF-entry in M3-boots |
| KS-09 usercopy/foutieve buffers | In M4-implementatie, niet behaald | hosttransacties én echte ring-3-geheugenfouten/atomiciteit vereist |
| KS-10 IPC-layout/lengtes/reserved/alignment | In M4-implementatie, niet behaald | volledige 320-byte-wirevalidatie en foutvolgorde vereist |
| KS-11 één snapshot/geen TOCTOU-claim | In M4-implementatie, niet behaald | testhook ná snapshot; geen onbewezen SMP-/shared-memorygarantie |
| KS-12 lokale handle/gen/stale | In M4-implementatie, niet behaald | eigen proc-tafel, collisions, close/reuse en generation-no-wrap vereist |
| KS-13 rechten/subset/transfer | In M4-implementatie, niet behaald | negatieve derive/send/transfer en geen mutatie bij fout vereist |
| KS-14 delegatie/revocatie | In M4-implementatie, niet behaald | A→B→C, vóór/na delivery, queued annulering en delivered EREVOKED vereist |
| KS-15 quota/atomiciteit/hergebruik | In M4-implementatie, niet behaald | queue16, slots255, nodes1024, depth8 en generationgrenzen vereist |
| KS-16 endpoint/proceslifecycle | In M4-implementatie, niet behaald | laatste RECV, senderdeath, receiverfault, geen orphans/ABA/underflow vereist |
| P02 Materiaal | Hostprototype met open toegankelijkheidschecks | 23 Node-tests, actuele gerichte browsermetingen onder `ui/prototype/verification/`; 200%-zoom/schermlezer/mensenproef nog open |
| Q01 lokale simulator | Hostprototype met afgebakend bewijs | 25 Rust-tests, vijf fixtures, no_std/Clippy; `quantum/`; OS-enforced networkdeny, OS-port en QPU niet bewezen |
| Lokale privacygrenzen | Huidige profielgrenzen behouden | QEMU zonder NIC/hostdisk; geen echte NAS/AI-dienst. Toekomstige LP-tests niet groen verklaard |
| Eindintegratie en remote checkpoint | Nog open | huidige M2/hostcheckpoints privé gepusht; eindmatrix, reviewherstel en gecombineerde regressie nog vereist |

`m2/suite.json` bewaart de oorspronkelijke gate-uitkomst. `capture_gate.py` heeft exit/markers, serial en artifacthashes opnieuw gecontroleerd. Bewijs wordt niet achteraf gewijzigd om nieuwere bronversies af te dekken; latere gates krijgen hun eigen bestanden en basisversie.
