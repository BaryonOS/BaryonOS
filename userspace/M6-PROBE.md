# M6a user service and recovery trial

Contract: `docs/implementation/M6A-WRITE-RECOVERY.md` v1, base `448974b`.
This describes the implementation and required tests; a present artifact or
host test is not an executed guest/durability trial. M5 fixture sources remain
unchanged. Only harness-generated synthetic disposable images are permitted.

## Build and modules

Root builds `python tools/build_users.py --kind m6` to
`build/users/m6_probe.elf`, manifest `manifest-m6.json`, kernel input `NOS_M6_ELF`.
The same direct rustc1.90/no_std/scalar flags as M5 use their own
`m6_linker.ld`. The six included Rust modules are m6_probe.rs, m6_runtime.rs,
services/block/src/wire.rs (existing codecs/FNV only), m6_wire.rs,
journal.rs and m6_user_service.rs. Every module belongs in the manifest hashes.

Artifact check:
`python -B -X utf8 userspace/tests/test_m6_probe.py --elf build/users/m6_probe.elf`.
Host protocol check:
`cargo +1.90.0 test -p block-protocol --locked --offline`, with the project tools
and own target `build/m6-service-tests`. No external package added.

## Private startup manifest

One ELF contains real roles service0, client1 and test supervisor2. Every role
has its own physical mappings. Virtual RX starts at40000000, control RW at
40010000, Message input11000, output12000 and sector buffer13000. RO starts at
40014000; the first512 bytes are0xa7 for the explicit input copy trial.
A successful poll is copied directly into own values. No user pointer
is passed on by the service as an asynchronous kernel/DMA buffer.

All header offsets are u64 from40010000:

| Offset | Meaning |
|---:|---|
| 0/8/16/24 | role / syscall counter / private pattern / reserved0 |
| 32/40 | service epoch / status0active,1done,>=0x100fixture error |
| 48 | service BlockREAD1|WRITE32|FLUSH64=97 |
| 56 | service GET-onlyroot; client replyroot; supervisor announcementroot |
| 64/72 | supervisor→client control SEND / service→supervisor SEND |
| 80/88 | service GET|PUTroot (client controlroot) / service→client reply SEND |
| 96/104/112 | fixed scopefirst4 / count4 / case0..30 |
| 120/128 | diagnostic final-value FNV / final generation |
| 136/144/152 | accepted epoch / RPCcounter / negative counter |
| 160/168/176 | bound clientPID / supervisorPID / initial servicePID |
| 184 | kernel-only testgate1: service executes a real ud2 after syscall return |
| 192 | phase:0start,1recovered,2PUT,3flushed,5clientdone |
| 200/208/216/224 | case2 scratchobject97 / READ1 / WRITE32 / FLUSH64 |
| 232 | service terminal signed error status or scratchsuccess1; client final generation/Unknown |
| 240 | local fixture error |

Case2 caps have the same scratch object, scope sector12. Other roles
get no device I/O rights. Gate184 is not an app request field. The kernel determines
the cut only after actual transport observation; VMcuts are logged by the kernel
and actually aborted by the host. User code cannot print PASS.

## Journal and state machine

Four complete512-byte records at sector4..7, value128. RecordmagicNOSM6A01,
version1/header64; generation@16,requestID@24,len128@32,slot@36,reserved40..63zero,
value64..191,zero192..503,checksum@504. FNV is computed over512 with the checksum field
zero. Slot binding generation=slot+1 and all structural fields are checked alongside the
checksum. The checksum is diagnostic, not authentication or a general
guarantee against tears/collisions.

Startup reads16 real READ slices and selects the highest valid record.
Without a valid record: CORRUPT and no formatting/WRITE/READY. A real
recovery FLUSH must succeed before announcement and handling of app requests.
PUT checks the expected generation, writes only the next slot and
does exactly WRITE→completion→FLUSH→completion→ACK. After an uncertain device result
it returns Unknown and blocks all further value claims/mutations until recovery.
No automatic WRITE/FLUSH replay. After generation4 the journal is logically full.

Appwire192 is version2 with TAG4d36424c4b000002, GET1/PUT2. ID@8,epoch@16,
generation@24,length@32,status@36,reserved@40,data48..175,tail176..191zero.
GETreq gen0/len0/data0; PUTreq current gen/len128. GETsuccess contains128 bytes,
PUTack len0/newgen. Error reply gen0/len0/data0. Unknown=-16,ENOSPC=-17,
CORRUPT=-18. Endpoint choice determines GET-only versus GET|PUT, never app flags.
Caps are actually closed; origin/outer wire/fields/epoch/op/right/
length/generation/space are checked before device calls.

## Client, recovery and concrete negatives

The client starts with real GET data and computes new bytes exclusively as
`oud + (0x31 + nieuwegeneratie)`, wrapping u8. Normally PUTid2/gen2, full still
PUTid3/gen3 and PUTid4/gen4; nextPUTid5 becomes ENOSPC. Case16 first tries
PUTid99/expectedgen2 while current1 and requires Stale, without I/O.

Case1 first does GETid1, then the errors below (current epoch1):

| ID | Deviation | Status |
|---:|---|---:|
| 100 | version1 | -1 |
| 101 | op3 | -2 |
| 102/103 | flags1 / reserved40=1 | -1 |
| 104 | GETlen1 | -1 |
| 105/106 | PUTlen0 /129 | -3 |
| 107/108 | PUTgen0 /MAX | -3 |
| 109 | epoch0 | -10 |
| 110 | GETdata[0]=1 | -1 |
| 111/112 | outerlen191 /193 | -1 |
| 113 | two SEND transfers of the same source cap | -1 |
| 114 | valid PUT on GET-only ingress | -5 |

All other requests use GETgen0/len0/data0, except105..108/114 which
use PUTgen1/len128. ID113 makes the service close both received handles;
the client closes its source9 only after the error reply. Kernel observation must
independently check every error/reply/zero I/O and resource balance. After that
a real good PUT/FLUSH/GET follows.

Case2 performs wrong types/rights/scopes/lengths, unreadable input and
non-consuming erroneous poll output. It first writes a valid read-only
512-byte source0xa7 to scratch12 and reads it back. Then it writes the misaligned
cross-page source40013f01 (255bytes0x3c +257bytes0xa7), mutates the RW255 to0xe1
after submit and requires exactly the original snapshot after FLUSH/READ. Only
scratch12 may change in addition to journal4..7.

Service cuts18..23 use one PUT without replay. Real new caps go via
service→supervisor→client; old handles are tested for EREVOKED and closed by the client itself.
After rebind the client does GET and requires exactly old or new; cuts5/6 require new.
VMcuts24..29 wait for a real host kill. Recovercase30 does only GET and no
PUT. Errorcases4..8/11..13 require Unknown and produce no final-value digest.
Startup errors9/10/15/17 remain local with live peers, without READY/value.

The full cut/device/cold boot matrix and both images for the same binary belong
to the integrated gate. QEMU process kill is not host power loss. No filesystem,
GC, network, physical hardware, IOMMU or production claim. Local work logs under
services/block/verification/ are ignored; root keeps compact evidence.
