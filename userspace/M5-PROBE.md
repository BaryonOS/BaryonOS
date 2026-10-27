# M5 userprogramma en startupmanifest

`m5_probe.rs` is één echte Rust `no_std` ELF met aparte private mappings per
proces. De startuprol kiest dienst, cliënt, testsupervisor of de afzonderlijke
CPU-negatieve attacker. De dienstbron is `services/block/src/user_service.rs`;
het gedeelde wirecontract staat in `services/block/README.md`. Bestaande
M3/M4-probes en hun linker/scripts worden niet vervangen.

## Build en artifactcontrole

Vanuit de root: `python tools/build_users.py --kind m5` bouwt
`build/users/m5_probe.elf` en `manifest-m5.json`. De kernelbuild ontvangt
`NOS_M5_ELF`; het buildtool bepaalt de exacte rustc 1.90-flags. Deze directe
build gebruikt `x86_64-unknown-none`, statische relocaties, no red zone,
panic=abort, opt-level=2 en `m5_linker.ld`, zonder dependencies of hostcalls.
Het manifest bevat binary/main/linkerhashes en hashes van de vier opgenomen
modules. Er zijn geen normale I/O-, debugport- of successmarkerrechten.

Artifactcontrole:
`python -B -X utf8 userspace/tests/test_m5_probe.py --elf build/users/m5_probe.elf`.
De elf tests controleren segmenten, buffers, symbolen, modulehashes en
negatieve ELF-mutaties. Opcodeaanwezigheid is geen uitgevoerde syscall.

## Vast geheugen en velden

RX begint op `0x40000000`; private RW op `0x40010000` met drie pagina's.
Message-input ligt op `0x40011000`, output op `0x40012000`, afzonderlijke
RO/NX-data op `0x40013000`. `m5_runtime.rs` gebruikt scalar int80 en kopieert
output direct naar eigen waarden voordat een volgende syscall output vervangt.
Onderstaande offsets zijn u64-velden vanaf RW; geen frame wordt tussen
processen gedeeld.

| Offset | Betekenis |
|---:|---|
| 0 | rol: dienst 0, cliënt 1, testsupervisor 2, attacker 3 |
| 8 | private syscallcounter |
| 16 / 24 | nietnul privaat patroon / reserved nul |
| 32 | service-epoch bij start |
| 40 | status: 0 actief, 1 klaar, >=100 fixturefout |
| 48 | READ-only Blockcap, uitsluitend dienst |
| 56 | dienst requestroot0; cliënt replyroot; supervisor announcementroot |
| 64 | supervisor SEND naar cliëntcontrol; overige rollen hebben dit niet nodig |
| 72 | dienst SEND naar supervisor-announcements |
| 80 | dienst requestroot1; cliënt control-RECVroot |
| 88 | dienst SEND naar vast cliëntreplyendpoint |
| 96 / 104 | dienstscopes: absolute sector 4 / 6 |
| 112 | testcase 0..17 |
| 120 / 128 | cliënt FNV-1a64 van de twee werkelijk ontvangen sectoren |
| 136 | laatst geaccepteerde/doorgestuurde service-epoch |
| 144 / 152 | cliënt RPC-/negatiefcounter, alleen aanvullende observatie |
| 160 / 168 | gebonden cliënt-PID / supervisor-PID |
| 176 | initiële service-PID; geen stille herschrijving in peers |
| 184 | test-only servicefaultgate: 2 submit→ud2 vóór poll, 1 idle-ud2, 0 normaal |
| 192 | cliëntfase: 0 wachten, 1 eerste reads, 2 negatieven, 4 latere reads, 5 klaar |
| 200 | eerste sector van Blockscope (4; count is 4) |
| 208 / 216 | uitsluitend attacker: supervisor-MMIO-/DMA-testadres |
| 224 | numerieke serviceBlock-handle voor proceslokale authority-negatief |
| 232 | echte cliënt Blockcap met nul READ-rechten, alleen case2 |
| 240 | lokale failurediagnose |

Announcement TAG `0x4d35414e4e000001`: payload vier u64
`(1, epoch, 0, 2)` plus twee echte caps met rechten 9. De supervisor gebruikt
de kernel-sender-PID en forwardt TAG `0x4d3542494e000001`, payload
`(1, epoch, service_pid, 2)` met twee SEND-only caps. De cliënt valideert de
supervisor-PID, volgende epoch en oplopende service-PID. Op herbinding worden
oude handles via echte send op EREVOKED getest en door de cliënt gesloten.

## Cases en acceptatie

| Case | Proef |
|---:|---|
| 0 | twee sectoren via acht echte 128-byte-reads |
| 1 | dertien protocolnegatieven, daarna normale reads |
| 2 | getypeerde devicecap/rechten en niet-consumerende foutieve polloutput |
| 3 | asynchrone EAGAIN met peerprogress |
| 4..9 | bad descriptor; used-id/index/short/long/status |
| 10..11 | I/O-fout / timeout, herstel en normale reads |
| 12 | reset-timeout, lokale foutreply en peerprogress, geen sectordigests |
| 13 | epochs1..3 pending submit→ud2; epoch4 normale reads |
| 14..16 | vierde attacker: MMIO-read / DMA-read / PCI-out fault |
| 17 | geen device, lokale foutreply en peerprogress, geen sectordigests |

Case1 request-IDs1..13: versie2→Invalid, WRITE2→NoSys, flags1→Invalid,
reserved1→Invalid, lengte0→Range, lengte129→Range, offsetMAX→Range,
offset511/lengte2→Range, oude epoch→Stale, requestdata1→Invalid,
outerlen191→Invalid, outerlen193→Invalid, caps2→Invalid. Het laatste geval
draagt dezelfde lokale sourcecap tweemaal over als SEND; de dienst moet beide
daadwerkelijke receiverhandles sluiten. De broncap wordt pas na foutreply
door de cliënt gesloten. De kernelobserver moet per echt verzoek de foutvorm,
zerodata-foutreply en nul devicecalls/publications/completions vastleggen;
het userwoord152 alleen is geen bewijs.

De cliënt assembleert beide volledige 512-byte-sectoren en berekent FNV
offset `14695981039346656037`, prime `1099511628211`. Er is geen verwachte
diskinhoud of seed in ELF/startup. Root vergelijkt deze metadata met twee
verschillende wegwerpimages en controleert dat hun hashes niet veranderen.
Bij rebind wordt de volledige assemblage herstart om epochs niet te mengen.

De geïntegreerde QEMU-gate, driverobserver en ongewijzigde M1–M4-regressies
zijn afzonderlijk vereist. Hosttests of een userstatus leveren geen gast- of
devicebewijs. Cliënt en supervisor blijven na klaar via counter/yield actief
zodat survivorprogress observeerbaar blijft. Writes, filesystem en hardware-
of IOMMU-containment zijn buiten deze eerste M5-slice.
