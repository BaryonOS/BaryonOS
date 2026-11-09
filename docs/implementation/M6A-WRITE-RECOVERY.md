# M6a — WRITE, FLUSH en herstel op wegwerpvolume

27 oktober 2026. Gekozen bouwbasis v1 op `448974b` (M5), branch `codex/m6a-write-recovery`. De vastgelegde opdracht omvat zelfstandige implementatie, daadwerkelijke QEMU-tests en het privé pushen van broncode en resultaten na afgebakende stappen. Dit document bevat criteria, geen geclaimde testresultaten.

## Invariant en bewijsgrens

Alleen harnasgegenereerde synthetische 64KiB-rawimages onder build/run/. Geen hostdisk, persoonlijke gegevens, NAS, firmware, productiekeys, nieuwe betaalde dienst of publieke release. Bestaande M1–M5-code/evidence en `.codex/environments/` behouden. Geen filesystem, GC of updateketen in deze stap.

Ring-3-dienst blijft geïsoleerd; kerneltransport en protocolconform QEMU-device/backend zijn vertrouwd. Geen IOMMU of kwaadwillige-DMA-containment. Vijf bestaande gepinde kernelpagina's blijven de enige DMA-pool. Geen user-PA/MMIO/PCI-/descriptorargumenten. Volledige inputkopie vóór publicatie, geen userpointer over scheduling. Gepubliceerde buffers blijven onaangeroerd tot completion of waargenomen resetstatus0; bij ontbrekende ack offline/quarantaine tot VM-stop.

WRITE-completion is geen duurzaamheidsbewijs. Alleen WRITE-completion gevolgd door succesvolle echte FLUSH-completion staat een app-ACK toe. Reset bevestigt quiescence, geen rollback of duurzaamheid. Geen automatische herhaling van WRITE/FLUSH bij onzekere afloop. Echte QEMU-procesabort laat hostcaches bestaan; dit bewijst geen hoststroomuitval of atomische sectorwrites. Gedeeltelijke/corrupte records worden apart als synthetische injectie gelabeld.

## Autoriteit, transport en ABI

Behoud Endpoint ALL_RIGHTS31 en alle M4/M5-bits. Block READ1, WRITE32, FLUSH64; bit2 blijft ongeldig. DERIVE4/TRANSFER8/REVOKE16 behouden. Globale known-mask omvat nieuwe bits, objecttype beperkt ze vóór operatierecht. Legacy create_block_device blijft root29. Nieuwe expliciete manifestfunctie create_block_device_with_rights maakt root125; dienst ontvangt uitsluitend READ|WRITE|FLUSH97 en adminroot sluit vóór userentry. READ/WRITE sectorscope device0, first4,count4. FLUSH-recht omvat expliciet het devicebrede cache-effect.

M6 abi_info major1/minor2; M5 blijft1.1 en eerdere profielen1.0. M5 READ0x40/poll0x41 blijven compatibel. Nieuwe fouten: -16 OUTCOME_UNKNOWN, -17 ENOSPC, -18 CORRUPT (laatste twee dienstprotocol).

| Call | Argumenten | Resultaat |
|---|---|---|
| 0x42 WRITE | cap,relative_sector,inputptr,512,0,0 | ticket in RDX; exacte512B-snapshot vóór publicatie |
| 0x43 FLUSH | cap,0,0,0,0,0 | ticket; geen data/sectorargument |
| 0x41 poll | cap,ticket,outptr,320,0,0 | READ zoals M5; WRITE/FLUSH leeg Message320 met tag=ticket, RDX0 |

Controle: syscall/unusedargs/exacte wiregrootte → typed cap/gen/node/object/device/operatierecht → ticket of sectorscope/scalargrenzen → transportbeschikbaarheid → hele input/outputplan → snapshot/publicatie of consumptie/kopie. Buiten scope EACCESS; input512 ongeldig EFAULT; readonly/misaligned en geldige paginaovergang zijn leesbare input. IPC-copylimiet320 blijft; aparte sector-inputhelper512. Poll herautoriseert het recht bij de opgeslagen operatie op dezelfde PID/objectkey/epoch/ticket. Ongeldige output consumeert niet.

Pure core bezit Operation{Read,Write,Flush}, Request.operation en expliciete Prepared/Published-semantiek, vaste descriptorchain/count en compatible M5-wrappers. READ header16→512W→status1, used513; WRITE header16→512R→status1, used1; FLUSH header16→status1, used1 en ongebruikte descriptor exactnul. FLUSH type4/sector0, WRITE type1. Markeer exposed conservatief direct vóór avail.idx, niet pas notify. M6 vereist VERSION1 en FLUSH9, weigert RO; CONFIG_WCE niet onderhandelen/aanraken. Harnas gebruikt cache=writeback, flush-respecterende backend; nooit cache=unsafe/no-flush.

Gepubliceerde WRITE/FLUSH met onzekere completion/timeout/exit/reset geeft Unknown, geen 'niet geschreven' of succes. Bewaar terminale uitkomst tot geldige poll of eigenaar-exit; resetack verwijdert die niet stil. Prepared-reject mag geen descriptorpublicatie of device-effect hebben. Een PUT met onzeker device-resultaat geeft -16 en blokkeert verdere mutaties tot recovery; geen WRITE-replay.

## Journal en dienstwire

Eigen bounded append-only journal van vier sectoren op LBA4–7. Eén waarde128B. Host initialiseert slot0/gen1/request1 met willekeurige synthetische waarde; overige drie slots nul. Andere imagebytes zijn canaries. Geen seed/verwachte waarde in guestconfig. Elk volgend record heeft generation=slot+1, request-id van PUT en de volledige nieuwe waarde. Geen overschrijven van een geldig oud record of GC. Na generatie4 geeft volgende PUT ENOSPC vóór I/O.

Record512: magic `NOSM6A01`@0; version u16=1@8; header-size u16=64@10; flags u32=0@12; generation u64@16; request-id u64@24; payload-length u32=128@32; slot u32@36; reserved24nul@40; payload128@64; zero312@192; checksum u64@504. FNV-1a64 over alle512 met checksumveld nul. Checksum aan het einde maakt prefix256-injectie betekenisvol. Alleen diagnostische corruptiedetectie, geen authenticatie/collisionvrij bewijs. Alle velden/slot/gen/lengtes/reserved/checksum worden gevalideerd vóór gebruik.

Recovery leest alle vier records via echte READs, kiest hoogste geldige generatie; geen geldige geeft CORRUPT, nooit formatteren. Een ongeldige tail wordt niet als waarde gepresenteerd. Volledige recovery-FLUSH moet slagen vóór READY. PUT verwacht huidige generatie (CAS), schrijft alleen volgende slot en verloopt WRITE→pollsuccess→FLUSH→pollsuccess→ACK. Geen ACK vóór FLUSH. Na recovery kan een expliciete nieuwe PUT de eerstvolgende ongeldige tail vervangen; de laatste geldige waarde blijft intact.

Nieuwe Message320-payload192, wireversie2; GET1/PUT2; flags0@4, request-id@8, serviceepoch@16, generation@24, length@32, status@36, reserved0@40, data128@48, reserved16nul@176. GETreq length0/data0; PUTreq length128 en verwacht huidige gen; GETsuccess128, PUTack0 en nieuwe gen. Epoch/velden/caps/sender/ingress volledig valideren vóór I/O. Onverwachte caps daadwerkelijk sluiten. Twee prebound ingressendpoints: GET-only en GET|PUT; reply prebound, app kiest geen LBA/replyhandle. Herstart kondigt nieuwe caps werkelijk via supervisor/client-control aan zoals M5; oude autoriteit ingetrokken. Client herhaalt onzekere PUT nooit, maar vraagt GET na rebind.

Fixturetransformatie: nieuwe waarde byte i = vorige byte i + (0x31 + nieuwe generatie), wrapping u8. Alleen synthetische testdata, geen cryptografie. Client berekent dit uit werkelijk ontvangen GET; host berekent onafhankelijk uit de aanvankelijke image. Normaal één PUT→gen2, full drie PUTs→gen4. Request-id gelijk aan nieuwe generatie. Guest meldt uitsluitend werkelijke eindwaarde als M6A:GENERATION:N, M6A:REQUEST_ID:N, M6A:VALUE_DIGEST:16hex; geen inhoudslogs.

## Crashpunten

| Cut | Werkelijke waarneming | Toegestane recovery |
|---:|---|---|
| 1 | gevalideerde WRITE vóór publicatie | oud |
| 2 | WRITE gepubliceerd, completion nog niet waargenomen | oud of nieuw |
| 3 | WRITE succesvol voltooid, vóór FLUSH-publicatie | oud of nieuw |
| 4 | FLUSH gepubliceerd, completion nog niet waargenomen | oud of nieuw |
| 5 | echte FLUSH succesvol voltooid, vóór app-ACK | nieuw |
| 6 | app-ACK daadwerkelijk aan client geleverd | nieuw |

Iedere cut krijgt een echte user-UD2, reset/quiescence, nieuwe dienst-PID/epoch en IPC-rebind in dezelfde VM, plus afzonderlijk echte externe QEMU-proceskill en koude nieuwe VM op exact dezelfde harnasimage. Softwarepending is geen claim over gelijktijdige fysieke DMA. Test-only kernelgate laat user na syscallretour ud2 uitvoeren; geen vervalste devicecompletion. VMcut logt exact M6A:CUT:N en wacht uitsluitend op hostkill; geen PASS/raw33 fabriceren. Harnas vereist draaiend proces bij kill, werkelijk terminationresultaat, exact cut/bootmarkers, geen FAIL/PASS, deadline en nieuwe recovery-VM. Voor cut6 wordt daadwerkelijke clientreceive geobserveerd.

Recoveryfeature voert alleen GET/recovery-FLUSH uit; geen PUT. Vóór/na elk stadium snapshot/hash van volledige image. Buiten toegestane journalbytes exact onveranderd. Recoverywaarde moet onafhankelijk gelijk zijn aan geldige old/new-recorddata; na cut5/6 verplicht nieuw. Geen hybride of checksumongeldig record. Na recovery moet de hele image bytegelijk blijven (FLUSH schrijft geen rawvolumedata).

## Gevallen en nummers (bevroren interface)

| ID | Featureachtervoegsel test-m6a- | Vereiste kern |
|---:|---|---|
| 0 | normaal: milestone-m6a | GET1→PUT2→FLUSH→ACK→GET2; twee images bij dezelfde binary, koude recovery |
| 1 | service-errors | concrete malformed/GET-only-PUT/caps vóór I/O geweigerd, daarna normale PUT |
| 2 | device-rights | typed rechten/pointers/grenzen/outputnonconsume; input snapshot positief en negatief |
| 3 | full | gen4, daarna ENOSPC zonder I/O; koude recoverygen4 |
| 4–8 | write-error,flush-error,write-timeout,flush-timeout,reset-timeout | Unknown, geen ACK/retry; lokale peers/poolgrenzen; koude recovery |
| 9 | no-flush | test-only gemaskeerde feature-advertisement, init fail-closed, geen write/ACK |
| 10 | readonly | echt RO-device, init geweigerd, geen write/ACK |
| 11 | bad-descriptor | WRITE-draft vóór publicatie geweigerd; geen write/retry/ACK; recovery |
| 12–13 | used-write,used-flush | lokale used-len-snapshot2 na echte completion; Unknown/geen ACK/replay, recovery |
| 14 | torn-tail | hostinjectie prefix256 nieuw record/ontbrekende checksum vóór boot; hersteloud, geen write |
| 15 | corrupt-all | hostcorruptie van enige geldige record, CORRUPT, geen format/write/READY |
| 16 | stale-generation | verkeerde CAS vóór I/O geweigerd, daarna normale PUT |
| 17 | no-device | target echt niet gekoppeld; boot-ESP niet gekozen; lokaal ENODEV |
| 18–23 | service-cut-1 t/m6 | zes echte dienstcrashes en rebind/recovery zonder PUT-replay |
| 24–29 | vm-cut-1 t/m6 | zes echte hostkills plus koude recovery op dezelfde image |
| 30 | recover | nieuwe VM: alleen READ/recovery-FLUSH/GET, geen WRITE |

Alle normale/negatieve eindige guests: BOOT:EBS_OK,KERNEL:ENTRY,BOOTINFO:OK,M6A:CASE:UPPER_SNAKE:PASS,M6A:SURVIVOR_PROGRESS:PASS,M6A:RESOURCES:PASS,M6A:PASS,M1:PASS in volgorde; raw33, geen timeout/FAIL. VMcut verwacht bewust hosttermination, apart gelabeld. Errorcases4–8/11–13: expliciet OUTCOME_UNKNOWN, geen eindwaardedigest; cases9/10/15/17 geen waarde/WRITE. Andere cases leveren geverifieerde eindwaarde.

Case2 heeft uitsluitend voor negatieve acceptatie een extra getypeerd scratchobject op sector12 met97, nooit appautoriteit. Test valide readonly512bron en misaligned crosspagebron0x40013f01 (255 bytes0x3c +257 bytes0xa7 uit RO op0x40014000), wijzig na submit RWbron naar0xe1; device moet snapshot0x3c/0xa7 krijgen. FLUSH en READ terug, geen copy uit gewijzigd userbuffer. Host staat alleen in deze case sector12-wijziging toe en vergelijkt exact deze512bytes; alle overige canaries intact. Layout M6: data40010000, message40011000, out40012000, sectorbuffer40013000, RO40014000 (eerste512bytes0xa7). Bestaande M5-ELF blijft bytegelijk.

Hosttests: caps/type/subset/revoke/quota/nonwrap; READ/WRITE/FLUSH ketens/directions/len/header; Prepared/Published/Unknown/ticket/exit/reset; volledige512-input/snapshot; journalformaat/checksum/torn/full/CAS/noautoinit; wire/error/rebind; harnasrejects voor willekeurige/externe diskpaden, markers zonder echte crash/flush, buiten-journalmutaties en hybride recovery. Volledige M1–M5-regressie en onafhankelijke reviews/raw/hash-audit op de finale bron.

## Ownership en oplevering

- K06 kernel_m2: kernel/**, crates/kernel-core/**, drivers/virtio-blk/**. Transport/syscalls/observer/coördinator. Geen root/tools/docswrites.
- C06 quantum_contract: crates/virtio-core/** en crates/capability-core/**. Enige eigenaar gedeelde pure Rust-interfaces; deel exacte API tijdig met K06.
- U06 product_acceptance: services/block/** en nieuwe M6-userspacebestanden/docs/tests. Dienst/journal/wire/echte roles. Geen bestaande M5-fixturewijzigingen.
- G06 root: rootconfig/lock/tools/tests/docs/build/run/images/Git/checkpoints. Maximaal3 agents, één schrijver per bestand, één gedeeld build/QEMUslot. Eigen host-targetdirs toegestaan.

Root maakt build_users kindm6, m6_probe.elf/manifest-m6.json en NOS_M6_ELF; K06 bezit kernel-Cargo-featureverklaring/build.rs. Root bezit M6A_CASES in build.py en afzonderlijk run_m6.py/test_m6.py/m6_disk.py. Nieuwe dependencies alleen eigen lokale code; geen extern pakket gepland. Bron/resultaten/open punten na iedere stap en vóór lange tests privépushen, remoteHEAD controleren; geen binaries/images/caches/secrets in Git. Herstel via aparte consistente checkout448974b.

## Primaire bronnen

OASIS Open, VIRTIO1.2 CS01,1juli2022, §§5.2.3,5.2.5.1,5.2.6 en5.2.6.2: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html — WRITE/FLUSH, feature/cache- en stable-write-semantiek. Gericht gecontroleerd27oktober2026. Bovenstaande quota/ABI/journal/cuts zijn projectbesluiten. Kerneltransport/QEMU/backend blijven expliciete aannames, niet door de specificatie bewezen implementatie-eigenschappen.
