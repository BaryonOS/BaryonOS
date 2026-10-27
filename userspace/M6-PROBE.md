# M6a userdienst en herstelproef

Contract: `docs/implementation/M6A-WRITE-RECOVERY.md` v1, basis `448974b`.
Dit beschrijft de implementatie en vereiste tests; een aanwezig artifact of
hosttest is geen uitgevoerde gast-/duurzaamheidsproef. M5-fixturebronnen blijven
ongewijzigd. Alleen harnasgegenereerde synthetische wegwerpimages zijn toegestaan.

## Build en modules

Root bouwt `python tools/build_users.py --kind m6` naar
`build/users/m6_probe.elf`, manifest `manifest-m6.json`, kernelinput `NOS_M6_ELF`.
Dezelfde directe rustc1.90/no_std/scalar-flags als M5 gebruiken de eigen
`m6_linker.ld`. De zes opgenomen Rustmodules zijn m6_probe.rs, m6_runtime.rs,
services/block/src/wire.rs (alleen bestaande codecs/FNV), m6_wire.rs,
journal.rs en m6_user_service.rs. Iedere module hoort in de manifesthashes.

Artifactcontrole:
`python -B -X utf8 userspace/tests/test_m6_probe.py --elf build/users/m6_probe.elf`.
Hostprotocolcontrole:
`cargo +1.90.0 test -p block-protocol --locked --offline`, met de projecttools
en eigen target `build/m6-service-tests`. Geen extern pakket toegevoegd.

## Privaat startupmanifest

Eén ELF bevat echte rollen dienst0, cliënt1 en testsupervisor2. Iedere rol
heeft eigen fysieke mappings. Virtuele RX begint op40000000, control-RW op
40010000, Message-input11000, output12000 en sectorbuffer13000. RO begint op
40014000; de eerste512 bytes zijn0xa7 voor de expliciete inputcopyproef.
Een succesvolle poll wordt direct in eigen waarden gekopieerd. Geen userpointer
wordt door de dienst als asynchrone kernel-/DMA-buffer doorgegeven.

Alle headeroffsets zijn u64 vanaf40010000:

| Offset | Betekenis |
|---:|---|
| 0/8/16/24 | rol / syscallcounter / privaat patroon / reserved0 |
| 32/40 | service-epoch / status0actief,1klaar,>=0x100fixturefout |
| 48 | dienst BlockREAD1|WRITE32|FLUSH64=97 |
| 56 | dienst GET-onlyroot; cliënt replyroot; supervisor announcementroot |
| 64/72 | supervisor→cliëntcontrol SEND / dienst→supervisor SEND |
| 80/88 | dienst GET|PUTroot (cliëntcontrolroot) / dienst→cliëntreply SEND |
| 96/104/112 | vaste scopefirst4 / count4 / case0..30 |
| 120/128 | diagnostische eindwaarde-FNV / eindgeneratie |
| 136/144/152 | geaccepteerde epoch / RPCcounter / negatiefcounter |
| 160/168/176 | gebonden cliëntPID / supervisorPID / initiële dienstPID |
| 184 | kernel-only testgate1: dienst doet na syscallretour echte ud2 |
| 192 | fase:0start,1recovered,2PUT,3flushed,5clientdone |
| 200/208/216/224 | case2 scratchobject97 / READ1 / WRITE32 / FLUSH64 |
| 232 | dienst terminale signed foutstatus of scratchsuccess1; cliënt eindgeneratie/Unknown |
| 240 | lokale fixturefout |

Case2-caps hebben hetzelfde scratchobject, scope sector12. Andere rollen
krijgen geen device-I/O-rechten. Gate184 is geen apprequestveld. Kernel bepaalt
de cut pas na daadwerkelijke transportwaarneming; VMcuts worden door de kernel
gelogd en door de host daadwerkelijk afgebroken. Usercode kan geen PASS printen.

## Journal en state machine

Vier volledige512-byte-records op sector4..7, waarde128. RecordmagicNOSM6A01,
versie1/header64; generatie@16,requestID@24,len128@32,slot@36,reserved40..63nul,
waarde64..191,zero192..503,checksum@504. FNV berekent over512 met checksumveld
nul. Slotbinding generatie=slot+1 en alle structurele velden worden naast de
checksum gecontroleerd. Checksum is diagnostisch, geen authenticatie of algemene
garantie tegen tears/collisions.

Startup leest16 echte READ-slices en selecteert het hoogste geldige record.
Zonder geldig record: CORRUPT en geen formattering/WRITE/READY. Een echte
recovery-FLUSH moet slagen vóór announcement en behandeling van apprequests.
PUT controleert verwachte generatie, schrijft alleen het volgende slot en
doet precies WRITE→completion→FLUSH→completion→ACK. Na onzeker device-resultaat
geeft hij Unknown en blokkeert alle verdere waardeclaims/mutaties tot recovery.
Geen automatische WRITE/FLUSH-replay. Na generatie4 is het journal logisch vol.

Appwire192 is versie2 met TAG4d36424c4b000002, GET1/PUT2. ID@8,epoch@16,
generation@24,length@32,status@36,reserved@40,data48..175,tail176..191nul.
GETreq gen0/len0/data0; PUTreq huidige gen/len128. GETsuccess bevat128 bytes,
PUTack len0/nieuwegen. Foutreply gen0/len0/data0. Unknown=-16,ENOSPC=-17,
CORRUPT=-18. Endpointkeuze bepaalt GET-only tegenover GET|PUT, nooit appflags.
Caps worden daadwerkelijk gesloten; herkomst/outerwire/velden/epoch/op/recht/
lengte/generatie/ruimte worden vóór devicecalls gecontroleerd.

## Cliënt, herstel en concrete negatieven

De cliënt begint met echte GET-data en berekent nieuwe bytes uitsluitend als
`oud + (0x31 + nieuwegeneratie)`, wrapping u8. Normaal PUTid2/gen2, full nog
PUTid3/gen3 en PUTid4/gen4; volgendePUTid5 wordt ENOSPC. Case16 probeert eerst
PUTid99/verwachtegen2 terwijl huidig1 en eist Stale, zonder I/O.

Case1 doet eerst GETid1, vervolgens onderstaande fouten (actuele epoch1):

| ID | Afwijking | Status |
|---:|---|---:|
| 100 | versie1 | -1 |
| 101 | op3 | -2 |
| 102/103 | flags1 / reserved40=1 | -1 |
| 104 | GETlen1 | -1 |
| 105/106 | PUTlen0 /129 | -3 |
| 107/108 | PUTgen0 /MAX | -3 |
| 109 | epoch0 | -10 |
| 110 | GETdata[0]=1 | -1 |
| 111/112 | outerlen191 /193 | -1 |
| 113 | twee SEND-transfers van dezelfde broncap | -1 |
| 114 | geldige PUT op GET-only ingress | -5 |

Alle overige requests gebruiken GETgen0/len0/data0, behalve105..108/114 die
PUTgen1/len128 gebruiken. ID113 laat de dienst beide ontvangen handles sluiten;
de cliënt sluit zijn bron9 pas na foutreply. Kernelobservatie moet iedere
fout/reply/nul-I/O en resourcebalans onafhankelijk controleren. Daarna volgt
een echte goede PUT/FLUSH/GET.

Case2 doet verkeerde typen/rechten/scopes/lengtes, onleesbare input en
niet-consumerende foute polloutput. Hij schrijft eerst een geldige readonly
512-byte-bron0xa7 naar scratch12 en leest terug. Daarna schrijft hij de misaligned
crosspagebron40013f01 (255bytes0x3c +257bytes0xa7), muteert de RW255 naar0xe1
na submit en eist exact de oorspronkelijke snapshot na FLUSH/READ. Alleen
scratch12 mag naast journal4..7 wijzigen.

Servicecuts18..23 gebruiken één PUT zonder replay. Echte nieuwe caps gaan via
dienst→supervisor→cliënt; oude handles worden op EREVOKED getest en zelf gesloten.
Na rebind doet de cliënt GET en eist exact oud of nieuw; cuts5/6 eisen nieuw.
VMcuts24..29 wachten op echte hostkill. Recovercase30 doet alleen GET en geen
PUT. Errorcases4..8/11..13 eisen Unknown en produceren geen eindwaardedigest.
Startupfouten9/10/15/17 blijven lokaal met levende peers, zonder READY/waarde.

Volledige cut-/device-/coldbootmatrix en beide images bij dezelfde binary horen
bij de geïntegreerde gate. QEMU-proceskill is geen hostpowerloss. Geen filesystem,
GC, netwerk, fysiekehardware-, IOMMU- of productieclaim. Lokale werklogs onder
services/block/verification/ worden genegeerd; root bewaart compact bewijs.
