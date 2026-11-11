# M6a — WRITE, FLUSH and recovery on a disposable volume

27 October 2026. Selected build baseline v1 at `448974b` (M5), branch `codex/m6a-write-recovery`. The recorded assignment includes autonomous implementation, actual QEMU tests and private pushes of source and results after scoped steps. This document contains criteria, not claimed test results.

## Invariant and evidence boundary

Only harness-generated synthetic 64 KiB raw images under build/run/. No host disk, personal data, NAS, firmware, production keys, new paid service or public release. Retain existing M1–M5 code/evidence and `.codex/environments/`. No file system, GC or update chain in this step.

The ring-3 service remains isolated; kernel transport and protocol-conforming QEMU device/backend are trusted. No IOMMU or containment of malicious DMA. Five existing pinned kernel pages remain the only DMA pool. No user PA/MMIO/PCI/descriptor arguments. Complete input copy before publication, no user pointer across scheduling. Published buffers remain untouched until completion or observed reset status 0; missing acknowledgement means offline/quarantine until VM stop.

WRITE completion is not durability evidence. Only WRITE completion followed by successful actual FLUSH completion permits an app ACK. Reset confirms quiescence, not rollback or durability. No automatic retry of WRITE/FLUSH with an uncertain outcome. Actual QEMU process abort leaves host caches intact; this does not demonstrate host power loss or atomic sector writes. Partial/corrupt records are separately labelled as synthetic injection.

## Authority, transport and ABI

Retain Endpoint ALL_RIGHTS31 and all M4/M5 bits. Block READ1, WRITE32, FLUSH64; bit2 remains invalid. Retain DERIVE4/TRANSFER8/REVOKE16. Global known mask includes new bits; object type restricts them before operational right. Legacy create_block_device remains root29. New explicit manifest function create_block_device_with_rights creates root125; service receives only READ|WRITE|FLUSH97 and the admin root closes before user entry. READ/WRITE sector scope device0, first4,count4. FLUSH right explicitly includes the device-wide cache effect.

M6 abi_info major1/minor2; M5 remains 1.1 and earlier profiles 1.0. M5 READ0x40/poll0x41 remain compatible. New errors: -16 OUTCOME_UNKNOWN, -17 ENOSPC, -18 CORRUPT (last two in service protocol).

| Call | Arguments | Result |
|---|---|---|
| 0x42 WRITE | cap,relative_sector,inputptr,512,0,0 | Ticket in RDX; exact 512 B snapshot before publication |
| 0x43 FLUSH | cap,0,0,0,0,0 | Ticket; no data/sector argument |
| 0x41 poll | cap,ticket,outptr,320,0,0 | READ as in M5; WRITE/FLUSH empty Message320 with tag=ticket, RDX0 |

Check order: syscall/unused args/exact wire size → typed capability/generation/node/object/device/operational right → ticket or sector scope/scalar bounds → transport availability → complete input/output plan → snapshot/publication or consumption/copy. Outside scope returns EACCESS; invalid 512-byte input returns EFAULT; read-only/misaligned input and a valid page crossing are readable input. IPC copy limit remains 320; separate 512-byte sector-input helper. Poll reauthorises the right for the stored operation on the same PID/object key/epoch/ticket. Invalid output does not consume.

Pure core owns Operation{Read,Write,Flush}, Request.operation and explicit Prepared/Published semantics, fixed descriptor chain/count and compatible M5 wrappers. READ header16→512W→status1, used513; WRITE header16→512R→status1, used1; FLUSH header16→status1, used1 and unused descriptor exactly zero. FLUSH type4/sector0, WRITE type1. Conservatively mark exposed immediately before avail.idx, not only at notify. M6 requires VERSION1 and FLUSH9, rejects RO; do not negotiate/touch CONFIG_WCE. Harness uses cache=writeback, a backend that honours flush; never cache=unsafe/no-flush.

Published WRITE/FLUSH with uncertain completion/timeout/exit/reset returns Unknown, not “not written” or success. Retain terminal outcome until valid poll or owner exit; reset acknowledgement does not silently remove it. Prepared rejection must cause no descriptor publication or device effect. A PUT with an uncertain device result returns -16 and blocks further mutations until recovery; no WRITE replay.

## Journal and service wire

Custom bounded append-only journal of four sectors at LBA4–7. One 128 B value. Host initialises slot0/gen1/request1 with a random synthetic value; other three slots zero. Other image bytes are canaries. No seed/expected value in guest configuration. Every subsequent record has generation=slot+1, PUT request ID and the complete new value. No overwrite of a valid old record or GC. After generation4, the next PUT returns ENOSPC before I/O.

Record512: magic `NOSM6A01`@0; version u16=1@8; header-size u16=64@10; flags u32=0@12; generation u64@16; request-id u64@24; payload-length u32=128@32; slot u32@36; reserved24zero@40; payload128@64; zero312@192; checksum u64@504. FNV-1a64 over all 512 bytes with checksum field zero. Placing the checksum at the end makes prefix256 injection meaningful. Diagnostic corruption detection only, not authentication or a collision-free proof. Validate all fields/slot/generation/lengths/reserved/checksum before use.

Recovery reads all four records through actual READs, selects the highest valid generation; none valid returns CORRUPT, never formats. An invalid tail is not presented as a value. Complete recovery FLUSH must succeed before READY. PUT expects the current generation (CAS), writes only the next slot and proceeds WRITE→poll success→FLUSH→poll success→ACK. No ACK before FLUSH. After recovery, an explicit new PUT may replace the first invalid tail; the last valid value remains intact.

New Message320 payload192, wire version2; GET1/PUT2; flags0@4, request-id@8, serviceepoch@16, generation@24, length@32, status@36, reserved0@40, data128@48, reserved16zero@176. GET request length0/data0; PUT request length128 and expected current generation; GET success128, PUT ACK0 and new generation. Fully validate epoch/fields/capabilities/sender/ingress before I/O. Actually close unexpected capabilities. Two prebound ingress endpoints: GET-only and GET|PUT; reply prebound, app selects no LBA/reply handle. Restart announces new capabilities through supervisor/client control as in M5; old authority revoked. Client never repeats an uncertain PUT, but requests GET after rebinding.

Fixture transformation: new value byte i = previous byte i + (0x31 + new generation), wrapping u8. Synthetic test data only, no cryptography. Client computes this from an actually received GET; host computes independently from the initial image. Normally one PUT→gen2, full: three PUTs→gen4. Request ID equals new generation. Guest reports only the actual final value as M6A:GENERATION:N, M6A:REQUEST_ID:N, M6A:VALUE_DIGEST:16hex; no content logs.

## Crash points

| Cut | Actual observation | Permitted recovery |
|---:|---|---|
| 1 | Validated WRITE before publication | Old |
| 2 | WRITE published, completion not yet observed | Old or new |
| 3 | WRITE completed successfully, before FLUSH publication | Old or new |
| 4 | FLUSH published, completion not yet observed | Old or new |
| 5 | Actual FLUSH completed successfully, before app ACK | New |
| 6 | App ACK actually delivered to client | New |

Every cut receives an actual user UD2, reset/quiescence, new service PID/epoch and IPC rebinding in the same VM, plus a separate actual external QEMU process kill and cold new VM on exactly the same harness image. Software pending is not a claim of concurrent physical DMA. A test-only kernel gate lets the user execute ud2 after syscall return; no fabricated device completion. VM cut logs exactly M6A:CUT:N and waits exclusively for host kill; do not fabricate PASS/raw33. Harness requires a running process at kill, actual termination result, exact cut/boot markers, no FAIL/PASS, deadline and a new recovery VM. For cut6, actual client receive is observed.

The recovery feature performs only GET/recovery FLUSH; no PUT. Before/after every stage, snapshot/hash the complete image. Outside permitted journal bytes, everything is exactly unchanged. Recovery value must independently equal valid old/new record data; after cut5/6, new is mandatory. No hybrid or checksum-invalid record. After recovery, the complete image must remain byte-identical (FLUSH writes no raw-volume data).

## Cases and numbers (frozen interface)

| ID | Feature suffix test-m6a- | Required core behaviour |
|---:|---|---|
| 0 | normal: milestone-m6a | GET1→PUT2→FLUSH→ACK→GET2; two images with the same binary, cold recovery |
| 1 | service-errors | Concrete malformed/GET-only-PUT/capability requests rejected before I/O, then normal PUT |
| 2 | device-rights | Typed rights/pointers/bounds/output non-consumption; positive and negative input snapshot cases |
| 3 | full | gen4, then ENOSPC without I/O; cold recovery gen4 |
| 4–8 | write-error,flush-error,write-timeout,flush-timeout,reset-timeout | Unknown, no ACK/retry; local peer/pool boundaries; cold recovery |
| 9 | no-flush | Test-only masked feature advertisement, initialisation fails closed, no write/ACK |
| 10 | readonly | Actual RO device, initialisation rejected, no write/ACK |
| 11 | bad-descriptor | WRITE draft rejected before publication; no write/retry/ACK; recovery |
| 12–13 | used-write,used-flush | Local used-length snapshot2 after actual completion; Unknown/no ACK/replay, recovery |
| 14 | torn-tail | Host injection of prefix256 of new record/missing checksum before boot; recover old, no write |
| 15 | corrupt-all | Host corruption of the only valid record, CORRUPT, no format/write/READY |
| 16 | stale-generation | Wrong CAS rejected before I/O, then normal PUT |
| 17 | no-device | Target actually unattached; boot ESP not selected; local ENODEV |
| 18–23 | service-cut-1 through 6 | Six actual service crashes and rebinding/recovery without PUT replay |
| 24–29 | vm-cut-1 through 6 | Six actual host kills plus cold recovery on the same image |
| 30 | recover | New VM: only READ/recovery FLUSH/GET, no WRITE |

All normal/negative finite guests: BOOT:EBS_OK,KERNEL:ENTRY,BOOTINFO:OK,M6A:CASE:UPPER_SNAKE:PASS,M6A:SURVIVOR_PROGRESS:PASS,M6A:RESOURCES:PASS,M6A:PASS,M1:PASS in order; raw33, no timeout/FAIL. VM cut deliberately expects host termination, separately labelled. Error cases4–8/11–13: explicit OUTCOME_UNKNOWN, no final-value digest; cases9/10/15/17 no value/WRITE. Other cases provide a verified final value.

Case2 has an additional typed scratch object on sector12 with97 solely for negative acceptance, never app authority. Test a valid read-only 512-byte source and misaligned cross-page source0x40013f01 (255 bytes0x3c +257 bytes0xa7 from RO at0x40014000), change RW source to0xe1 after submit; device must receive snapshot0x3c/0xa7. FLUSH and READ back, no copy from the changed user buffer. Only in this case does the host permit sector12 mutation and compare exactly these512bytes; all other canaries remain intact. M6 layout: data40010000, message40011000, out40012000, sectorbuffer40013000, RO40014000 (first512bytes0xa7). Existing M5 ELF remains byte-identical.

Host tests: capabilities/type/subset/revoke/quota/no wraparound; READ/WRITE/FLUSH chains/directions/length/header; Prepared/Published/Unknown/ticket/exit/reset; complete 512-byte input/snapshot; journal format/checksum/torn/full/CAS/no automatic initialisation; wire/error/rebinding; harness rejections for arbitrary/external disk paths, markers without actual crash/flush, mutations outside the journal and hybrid recovery. Full M1–M5 regression and independent reviews/raw/hash audit on final source.

## Ownership and delivery

- K06 kernel_m2: kernel/**, crates/kernel-core/**, drivers/virtio-blk/**. Transport/syscalls/observer/coordinator. No root/tools/docs writes.
- C06 quantum_contract: crates/virtio-core/** and crates/capability-core/**. Sole owner of shared pure Rust interfaces; share exact API with K06 promptly.
- U06 product_acceptance: services/block/** and new M6 userspace files/docs/tests. Service/journal/wire/actual roles. No existing M5 fixture changes.
- G06 root: root configuration/lock/tools/tests/docs/build/run/images/Git/checkpoints. At most 3 agents, one writer per file, one shared build/QEMU slot. Private host target directories allowed.

Root creates build_users kindm6, m6_probe.elf/manifest-m6.json and NOS_M6_ELF; K06 owns kernel Cargo feature declarations/build.rs. Root owns M6A_CASES in build.py and separate run_m6.py/test_m6.py/m6_disk.py. New dependencies: project-local code only; no external package planned. After every step and before long tests, privately push source/results/open points and verify remote HEAD; no binaries/images/caches/secrets in Git. Recover through a separate consistent checkout at448974b.

## Primary sources

OASIS Open, VIRTIO1.2 CS01, 1 July 2022, §§5.2.3,5.2.5.1,5.2.6 and5.2.6.2: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html — WRITE/FLUSH, feature/cache and stable-write semantics. Targeted check on 27 October 2026. Quotas/ABI/journal/cuts above are project decisions. Kernel transport/QEMU/backend remain explicit assumptions, not implementation properties demonstrated by the specification.
