# M5-U — gerichte runtime/protocolreview

**Delta-review op basis 0d4bad72: beide concrete bevindingen hieronder zijn
gesloten door broninspectie en gerichte gastrecords. De oorspronkelijke
bevindingen blijven staan als reviewgeschiedenis. Zie het delta-addendum
onderaan voor de precieze bewijsgrens; de volledige gate blijft afzonderlijk.**

27 oktober2026. Scope: eerste echte Rust userdienst/cliënt/testsupervisor en hun M5-koppeling, één CPU/één thread. Read-only review na de eerste userbuild; geen kernel- of driverwrites door deze agent. Geen gast-/deviceclaim uit de onderstaande hosttests.

## Werkelijk uitgevoerd

- Root: `python tools/build_users.py --kind m5`, exit0, stderr leeg. ELF entry40000000, RX16199 bytes, RW12288 bytes, RO1118 bytes. Binary SHA256 `933291ef6f097f68648696f129b58c257a951c2a7fdf6920b5892694aa92914e`.
- Useragent: `cargo +1.90.0 test -p block-protocol --locked --offline`, exit0,9tests/0fail. Child-environment gebruikt de projecttoolchain en `CARGO_TARGET_DIR=<root>/build/m5-service-tests`; geen globale environmentwijziging. Werkelijke uitvoer: `wire-tests.txt`.
- Useragent: `python -B -X utf8 userspace/tests/test_m5_probe.py --elf build/users/m5_probe.elf`, exit0,11tests/0fail. Na toevoeging van de gevraagde vier-modulehashcontrole opnieuw uitgevoerd, eveneens11/11. Werkelijke laatste uitvoer: `artifact-tests.txt`.

De artifacttest vergelijkt main/linker/binary én de volledige set `module_sha256`: m5_probe.rs, m5_runtime.rs, wire.rs, user_service.rs. Hij controleert echte ELFsegmenten/symbolen, vaste buffers, geen TLS/dynamische relocaties en betekenisvolle negatieve ELFmutaties. Int80-opcodeaanwezigheid en nulstartup zijn artifacteigenschappen, geen uitgevoerde syscall of globale afwezigheidsproef voor fixturedata.

## Concrete bevindingen

1. **Blokkerende bewijsleemte SERVICE_ERRORS.** In de gelezen kernelversie registreerde `m5::sent` alleen announcements, forwarding en succesvolle READreplies. Case1 besliste alleen op cliëntwoord152>0. Daardoor kon extra block-I/O tijdens malformed requests onopgemerkt blijven zolang latere8reads/correlations goed waren. Usercounter/status alleen voldoet niet aan de M5spec. Root en kernelbuilder zijn rechtstreeks geïnformeerd. Minimaal herstel:13 echte clientrequests onafhankelijk classificeren, ID/foutvorm/status/epoch vastleggen, service0x40/41- en submission/completion-baseline per verzoek bewaren; bij echte errorreply exactzelfdeid/op/offset, serverepoch1, length0,caps0,flags/reserved/data/tailnul en nul I/O-delta vereisen. Alle13 bits/replies zijn verplicht. Het cap-carrying geval moet twee daadwerkelijke receiver-capcloses/nodebalans aantonen. Status: herstel door kernelbuilder nog afzonderlijk te verifiëren; geen groenclaim hier.

2. **Onnodige supervisorautoriteit voor cliënt.** De gelezen `manifest` delegeerde ook SEND op supervisor-announcements aan de cliënt en schreef dit op cliënt64. Onze runtime gebruikt deze handle niet. Een cliënt kan daardoor een ongeldig announcement in de supervisorqueue zetten; de supervisor weigert terecht de sender maar eindigt via fail43. Het afgesproken model heeft alleen service→SUP en SUP→client-control nodig. Voorkeur: verwijder die cliëntdelegatie en laat cliënt64 nul. Gemeld aan root/kernelbuilder; runtimewijziging is niet nodig. Dit is geen waargenomen misbruik door de goedwillende huidige fixture.

## Exacte SERVICE_ERRORS-volgorde

Alle requests hebben normaal TAG, versie1,READ1,epoch1,offset0,length128,payload192,data0,status0,caps0. IDs1..13 wijzigen één geval:

| ID | Afwijking | Reply status |
|---:|---|---:|
| 1 | version2 | -1 |
| 2 | WRITE2 | -2 |
| 3 | flags@4=1 | -1 |
| 4 | reserved@40=1 | -1 |
| 5 | length0 | -3 |
| 6 | length129 | -3 |
| 7 | offsetMAX | -3 |
| 8 | offset511,length2 | -3 |
| 9 | requestepoch0 | -10 |
| 10 | requestdata[0]@48=1 | -1 |
| 11 | Message.len191 | -1 |
| 12 | Message.len193,extra bytenul | -1 |
| 13 | caps2,zelfde bronhandle derive(client.replyroot,9),elke descriptorrights1 | -1 |

Bij13 maakt IPC twee verschillende receiverhandles; de dienst sluit beide. De cliënt sluit zijn eigen bron9 pas na het ontvangen van de foutreply. Iedere foutreply heeft payloadversion1,sameid/op/offset,serverepoch1,length0,statuszoalsboven en verder alle data/reserved nul. De13 gevallen worden gevolgd door echte geldige reads.

## Gelezen paden die correct begrensd zijn

- De dienst verwerkt geen app-LBA/replyhandle. Ontvangstendpoint0/1 kiest de vaste service-scope4/6; checked relative byteoffset/length blijven binnen512. Alleen READ bereikt `device_read`; WRITE2 geeft NoSys. De bron doet bij een protocolfout geen block_submit/poll, maar de kernelobserver moet dit ook onafhankelijk meten.
- Ontvangen IPC-caps worden vóór verdere appvalidatie gesloten; unexpectedcaps geven Invalid. Er worden nooit descriptorwaarden uit een device-poll als ontvangen IPC-authoriteit behandeld.
- MSG/OUT zijn vaste private pagina's. Na IPC/poll wordt de volledige Message onmiddellijk naar eigen userstackarrays gekopieerd. Requestheader en ontvangen devicebytes blijven owned waarden; geen OUT-slice of userpointer wordt door de dienst als langdurige I/O-buffer bewaard.
- Devicebytes komen alleen uit succesvolle block_poll-Message320. Sender0/tagticket/length/caps/reserved/tail worden gecontroleerd. Geen seed of berekende diskrespons staat in de dienst. De cliënt assembleert twee volledige512-byte-sectoren en berekent uitsluitend diagnostische FNVmetadata.
- Rebind gebruikt echte service9-captransfer→supervisor9→client1. Supervisor sluit zijn ontvangen9-handles; cliënt controleert SUP-PID, exactvolgende epoch en nieuwe servicePID, test oudeEREVOKED en sluit zelf beideoudehandles. Een rebind onderbreekt de huidige RPC en herstart de volledige sectorassemblage, zodat digests geen epochs mengen.
- Restartcase13 gebruikt epochs1–3 voor echte submit→ud2 vóórpoll en epoch4 voor normale reads. De read-only retry heeft geen WRITE-replaysemantiek. Cliënt/supervisor worden door usercode niet herstart. Kernelreset/quiescence en behoud van hun PID/CR3 vereisen apart gastbewijs.
- CPU-negatieven zijn een afzonderlijke rol3. Alleen die rol voert de expliciete supervisorread/PCI-out uit; de dienst krijgt geen MMIO/DMA/PCI-opcode-interface.

## Hertestvoorwaarden en open grens

Na kernelobserverherstel: voer SERVICE_ERRORS uit en eis onafhankelijk13afwijzingen met nul devicecalls/submissions/completions; eis daarna8 echte slice-replies en beidehostdigests. Verifieer ook de twee ontvangen capcloses. Herhaal de RESTARTcase met exact3pendingcrashes,3resetacks,4verschillende servicePIDs/epochs,4echteannouncements/forwardings, oudeEREVOKED en ongewijzigde cliënt-/supervisorPID/CR3 plus counters die doorgaan. Twee verschillende per-run images moeten met dezelfde kernel/userbinary elk de overeenkomstigehostchecksums en ongewijzigdeimagehash opleveren.

Verder: gerichte device-rights/polloutputnietconsume, alle descriptor/completion/resetfoutcases en CPUattackerfaults met survivorprogress, gevolgd door de geïntegreerde M1–M4-regressie. Deze review start geen tests opnieuw en kent nog geen volledige M5-gastgate toe. Kerneltransport/device zijn TCB; zonder IOMMU geen kwaadaardige-hardware-DMA-containmentclaim. De userdienst is echt ring3, geen volledig userspace-hardwaredriver. Geen WRITE/filesysteem/M6- of fysiekehardwareclaim.

## Delta-addendum: gericht herstel gecontroleerd

Basis `0d4bad72e6570354d779ed49bdf475ba242d9b6e`. Alleen de afgesproken
observer-/manifestdelta en de twee bestaande diagnostische gastrecords zijn
gelezen; geen tests, builds of bronwijzigingen uitgevoerd.

1. **SERVICE_ERRORS gesloten.** `kernel/src/m2/m3/m5.rs:522` classificeert
   na een daadwerkelijk geslaagde client-send de IDs 1..13, exacte requestdata,
   lengtes en capcount. De verwachte status wordt in de observer bepaald.
   Per request worden servicecalls naar 0x40/0x41, submissions en completions
   vastgelegd. De echte service-send moet een exact leeg foutantwoord met
   dezelfde ID/opcode/offset en huidige epoch bevatten; alle drie counters
   moeten gelijk blijven. Voor ID13 moeten twee succesvolle service-capcloses
   zijn uitgevoerd, handles terug op baseline en nodes met twee verminderd.
   `m4.rs:205` telt uitsluitend geslaagde cap_close-resultaten; de sendhook
   volgt op geslaagde Engine.send. De eindobserver vereist beide masks
   `0x1fff` en geen onafgehandeld request (`m5.rs:825`). Daarmee vervalt de
   eerdere afhankelijkheid van alleen de usercounter als bewijs.
2. **Onnodige cliëntautoriteit gesloten.** De manifestfunctie maakt alleen de
   supervisor→client-control-delegatie; de eerdere client→SUP-delegatie is
   weg. Cliëntveld64 wordt expliciet nul gezet en gecontroleerd
   (`m5.rs:250`). Er is geen wijziging aan de userruntime nodig.
3. **Gericht herstartbewijs gelezen.** De faultobserver vereist voor case13
   een echte service-#UD met een open request en maximaal drie herstarts.
   Herinitialisatie volgt op daadwerkelijke resetack; vóór iedere vervanging
   blijven cliënt-/supervisor-PID en CR3 gelijk. Het eindpad vereist drie
   pending crashes, drie herstarts, minimaal drie resets, vier announcements
   en forwards, cliënt-epoch4 en acht met devicebytes gecorreleerde slices.
   De cliënt valideert iedere epoch en oude EREVOKED-handles in de ongewijzigde
   ELF. De usercounter blijft aanvullend, geen zelfstandig restartbewijs.

Gelezen bestaande records:

| Runmap onder build/run | Raw exit | Timeout | Record | Diskoracle |
|---|---:|---|---|---|
| m5-service-errors-diagnostic-9f848293fa | 33 | nee | passed=true | beide sectoren correct; image ongewijzigd |
| m5-restart-diagnostic-f53e180603 | 33 | nee | passed=true | beide sectoren correct; image ongewijzigd |

De serials bevatten alle vereiste markers, geen verboden FAIL-marker en lege
stderr. Parent rapporteert voor beide harnas-exit0; de resultaatrecords zelf
leggen raw exit33/expected33 en passed=true vast. De exacte resultaat-, serial-,
kernel- en bronhashes staan in `user-evidence.json`.

Deze twee diagnostische runs hebben verschillende kernel-/bronhashes. Ze
vormen dus geen bewijs voor de nog vereiste twee images met exact dezelfde
bevroren binary, en vervangen de samengestelde M5-gate plus M1–M4-regressies
niet. Binnen deze gerichte delta-review blijven geen concrete bevindingen
open. Userbron en handleidingen blijven bevroren.
