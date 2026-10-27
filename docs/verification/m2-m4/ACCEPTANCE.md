# Acceptatieoverzicht M1–M4

Eindaudit27 oktober2026. Kernel-/runtimebron: `ed09818`; aanvullende volledige M1-regressie: `622c2ee`. Scope: qemu64/TCG, één CPU,256MiB, synthetische gegevens. Een AI-review is geen menselijke audit. De matrix sluit de afgesproken ontwikkelfase af; de expliciete toekomstige/human checks hieronder worden niet groen verklaard.

| Requirement | Status | Daadwerkelijk bewijs en grens |
|---|---|---|
| M1 eigen loader/kernel en bootfouten | Behaald | m1/suite.json:10groepen/8VM;3cold boots,missing/corrupt,EBSstale-key,panic,echte60s-hang. Negatieve guestrecords blijven false; de test op correcte detectie slaagt |
| KS-01 kaart/reserveringen/overflow/unieke frames | Behaald | m2/suite.json en hostlog; echte kernelasserts plus ongeldige kaart-/rangegevallen |
| KS-02 eigenaar/nulmaken/quota/uitputting | Behaald | fysieke patroon/free/reassign/zero-proef; hostuitputting/verkeerdeowner/2048-framequota |
| KS-03 eigen tabellen/W^X/supervisor | Behaald binnen profiel | tablewalk/fysiekealiasaudit, RO/NX/null en kernel-UD; userheap/stack via KS06 |
| Eigen GDT/TSS/IDT/CR3 en guardstacks | Behaald voor geteste paden | CR3-readback,zesguards,echte#DF/IST1 bijonbruikbare MAIN-stack. NMI-overname niet geïnjecteerd |
| KS-04 A/B private adressen/frames/TLB | Behaald | m3/guest-runs.json: echte CPL3/CR3-wissels, disjuncte frames en private patronen; groteimage67 userpages/BSSnul als extra |
| KS-05 peer/kerneltoegang geweigerd | Behaald | drie afzonderlijke peer-read/write/kernelwrite-boots, exacte faultmetadata en survivor ná fault |
| KS-06 NX/guards/CLI/FP-opcodes | Behaald binnen qemu64 | afzonderlijke stack/heap/guards/CLI/x87/XMM/AVX/UD; AVX#UD is geen bewijs van AVX-contextgebruik |
| KS-07 preëmptieve16-worker-fairness | Behaald | twee echte16-worker-rondes, iedere selectie gecontroleerd, private counters, geen worker-yield |
| KS-08 exit/fault/opruimen/idle | Behaald | exacte per-processfreebalans, latere survivorprogress, Bexit7, invalidRSP, beideexit en achtidleIRQwakeups |
| User-ELF/syscallgrens | Behaald binnen vaste manifesten |14coretests,13probe/6large-artifacttests; echte int80foutcodes/registersentinels/DF-entry; geen dynamische applicatieladerclaim |
| KS-09 usercopy/buffers | Behaald | test-m4-copy:13EFAULTgevallen, misalignment, ROinput, cross-page/ROoutput, fout-outputongewijzigd en volledige320bytes readback na0xa5poison |
| KS-10 wire/len/reserved/size/alignment | Behaald | test-m4-wire plus hostcodec/fuzz:len257,caps3,size0/319/321/MAX,reserved/tail,misaligned geldige buffers |
| KS-11 één snapshot | Behaald binnen één-threadprofiel | test-m4-snapshot herschrijft origineleuserbytes nácopy;ontvanger krijgt origineleABCD/len4; onbeschikbareAPI's ENOSYS. Geen SMP/TOCTOU-concurrentieclaim |
| KS-12 lokale/stale handles/generation | Behaald | test-m4-handles:foreignnumeriek/owncollision,slot/gen,close/reuse/stale; generationretirement aanvullendhostmodel |
| KS-13 rechten/subset/transfer | Behaald | test-m4-rights plus hosttests:geenDERIVE/SEND/TRANSFER,extra rights,RECV-transfer; gecombineerde errororder en geenmutatie |
| KS-14 delegatie/revocatie | Behaald | test-m4-revoke:queued cancellation en echte A→B→C-delivery/revoke;deliveredEREVOKED,oudebytes blijven |
| KS-15 quota/atomiciteit/hergebruik | Behaald met gescheiden lagen | echte queue16/slots255/depth8–9/close-reuse; globale1024nodes/maxgeneration in21capability-hosttests; geen extraVMclaim daarvoor |
| KS-16 endpoint/proceslifecycle | Behaald | drie echte lifecycle-rondes:lastRECVclose,senderexit,receiverfault;ECLOSED/annulering/EREVOKED,refs/resources/invariants ensurvivor |
| P02 Materiaal | Hostprototype; externe controles open |23Node-tests +gerichtebrowsermetingen;9huidigehashes opnieuwgelijk. Echte200%zoom/schermlezer/mensenproef open |
| Q01 lokale simulator | Hostprototype met afgebakend bewijs |25Rusttests/5fixtures/Clippy/no_std;15huidigehashes opnieuwgelijk. Geen OS-port/QPU/networksandbox/crypto-claim |
| Privacy-/hostgrenzen | Huidige ontwikkelgrenzen behouden | geenNIC/hostdisk inQEMU;geen echteNAS/AI/telemetrie. ToekomstigeLP-tests niet uitgevoerd |
| Open-sourcepublicatie/licentie | Afzonderlijk open besluit | privécheckpoint/draftPR;LICENSE-PLAN.md. Geen publieke release of definitieve licentie toegepast |
| Integratie/review/eindbewijs | Behaald voor M1–M4-code |42M2–M4groepen/36VM +10M1groepen/8VM;alle serial/stderr/artifacthashes gecontroleerd;R03/R04/V04;bron/bewijs via privébranch |

De actuele suites bewaren hun eigen basisversie, exacte commando's, ruwe uitkomsten en artifacts. Eerdere bewijssnapshots blijven in Gitgeschiedenis behouden. V04 controleerde aanvullend fasevolgorde en volledige KS01–16-koppeling; zie V04-AUDIT.md. Geen ontbrekend bewijs is als impliciet succes behandeld.
