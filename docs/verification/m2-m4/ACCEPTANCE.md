# Acceptatieoverzicht M1–M4

Dit overzicht onderscheidt uitvoering van nog te leveren bewijs. Scope: het eigen x86_64/QEMU-ontwikkelprofiel, één CPU, 256 MiB, synthetische gegevens. Tweede-agentreview is geen menselijke audit.

| Requirement | Status | Werkelijk bewijs / vereiste aanvulling |
|---|---|---|
| M1 eigen loader/kernel | Behaald op basischeckpoint; M2-regressie groen | eerdere `docs/verification/boot-suite.json`; `m2/suite.json` bevat de latere defaultboot |
| KS-01 kaart/reserveringen/overflow/unieke frames | M2 behaald op a256951 | echte kernelasserts + bootprotocol-/framehosttests, `m2/guest-runs.json` |
| KS-02 owner/zero/quota/uitputting | M2 behaald op a256951 | fysieke patroon/free/reassign/zero-proef en quota; hosttests voor verkeerde owner/uitputting |
| KS-03 eigen tabellen/W^X/supervisor | M2 behaald voor kernelprofiel | echte tablewalk/aliasaudit; aparte RO/NX/null-faultboots. Userheap/-stack volgt KS-06 |
| Eigen GDT/TSS/IDT/CR3 en guardstacks | M2 behaald binnen genoemde proeven | eigen CR3-readback, zes guards ongemapt, echte #DF via IST1 bij onbruikbare MAIN-stack; NMI niet geïnjecteerd |
| KS-04 A/B eigen adressen/frames/TLB | In implementatie, niet behaald | meerdere echte CPL3-/CR3-wissels, private patronen en disjuncte frames vereist |
| KS-05 A kan peer/kernel niet benaderen | Niet uitgevoerd | afzonderlijke peer-read/write en kernelwrite, exacte faultmetadata en B-voortgang vereist |
| KS-06 NX/guards/CLI/FP-instructies | Niet uitgevoerd | afzonderlijke stack/heap/guards/CLI/x87/XMM/AVX-boots vereist; precieze virtuele CPU-grens benoemen |
| KS-07 preëmptieve 16-worker-fairness | Niet uitgevoerd | geen yield, meerdere volledige selectierondes en private tellers vereist |
| KS-08 exit/fault/opruimen | Niet uitgevoerd | veilige CR3/stack vóór zero/free, resourcebalans en voortgang overlevende vereist |
| User-ELF en syscallprobe | Hostlaag behaald, CPL3 open | `userspace/verification/`: 13 artifacttests, echte fout-/syscallopcodes, geen isolatieclaim |
| KS-09 usercopy/foutieve buffers | Niet gestart vóór M3-poort | hosttransacties én echte ring-3-geheugenfouten/atomiciteit vereist |
| KS-10 IPC-layout/lengtes/reserved/alignment | Niet gestart vóór M3-poort | volledige 320-byte-wirevalidatie en foutvolgorde vereist |
| KS-11 één snapshot/geen TOCTOU-claim | Niet gestart vóór M3-poort | testhook ná snapshot; geen onbewezen SMP-/shared-memorygarantie |
| KS-12 lokale handle/gen/stale | Niet gestart vóór M3-poort | eigen proc-tafel, collisions, close/reuse en generation-no-wrap vereist |
| KS-13 rechten/subset/transfer | Niet gestart vóór M3-poort | negatieve derive/send/transfer en geen mutatie bij fout vereist |
| KS-14 delegatie/revocatie | Niet gestart vóór M3-poort | A→B→C, vóór/na delivery, queued annulering en delivered EREVOKED vereist |
| KS-15 quota/atomiciteit/hergebruik | Niet gestart vóór M3-poort | queue16, slots255, nodes1024, depth8 en generationgrenzen vereist |
| KS-16 endpoint/proceslifecycle | Niet gestart vóór M3-poort | laatste RECV, senderdeath, receiverfault, geen orphans/ABA/underflow vereist |
| P02 Materiaal | Hostprototype met open toegankelijkheidschecks | 23 Node-tests, actuele gerichte browsermetingen onder `ui/prototype/verification/`; 200%-zoom/schermlezer/mensenproef nog open |
| Q01 lokale simulator | Hostprototype met afgebakend bewijs | 25 Rust-tests, vijf fixtures, no_std/Clippy; `quantum/`; OS-enforced networkdeny, OS-port en QPU niet bewezen |
| Lokale privacygrenzen | Huidige profielgrenzen behouden | QEMU zonder NIC/hostdisk; geen echte NAS/AI-dienst. Toekomstige LP-tests niet groen verklaard |
| Eindintegratie en remote checkpoint | Nog open | huidige M2/hostcheckpoints privé gepusht; eindmatrix, reviewherstel en gecombineerde regressie nog vereist |

`m2/suite.json` bewaart de oorspronkelijke gate-uitkomst. `capture_gate.py` heeft exit/markers, serial en artifacthashes opnieuw gecontroleerd. Bewijs wordt niet achteraf gewijzigd om nieuwere bronversies af te dekken; latere gates krijgen hun eigen bestanden en basisversie.
