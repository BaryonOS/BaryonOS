# M5 — isolated block service and virtio transport

Build baseline recorded on 27 October 2026, started at `c630c59`. This makes roadmap milestone M5 concrete. Interface version 1; root coordinates changes and related tests. This document describes acceptance criteria, not test evidence already achieved.

## Result and evidence boundary

An actual Rust no_std service in ring 3, a client and a user-test supervisor use existing Message320 IPC. A small trusted kernel transport reads a per-run generated, read-only 64 KiB image through modern virtio-blk PCI. Four 128-byte replies form one 512-byte sector. Two request endpoints separately bind sectors 4 and 6. Only the service receives a typed READ device capability.

WRITE, FLUSH, DISCARD, file systems and networking are not M5 interfaces. A WRITE service request fails without device I/O. Writing and persistence follow in M6.

The trusted computing base (TCB) includes the kernel transport and protocol-conforming QEMU device/backend. The user service cannot select a DMA address, descriptor, BAR/MMIO, PCI configuration or bus-master setting. Under this driver protocol, DMA reaches only five permanently pinned supervisor RW/NX kernel pages. Without an IOMMU, containment of a malicious device has not been demonstrated. This is an isolated service with kernel transport, not a fully userspace hardware driver.

## Driver and memory

- Exact test device `00:05.0`, vendor `1af4`, device `1042`; the existing boot ESP is a different device. One split queue with eight positions, at most one outstanding READ, 512-byte sector, result slice 1–128 bytes.
- PCI capability chain, BAR type/size, capability offset/length and notify multiplier are bounded and processed with overflow checking. Only validated device regions receive supervisor RW/NX UC mappings. No general RAM-map bypass or user MMIO.
- Require VIRTIO_F_VERSION_1; negotiate only understood features. Read-only test backend; no arbitrary device commands. Capacity comes from a stable read of device configuration.
- Five fixed pages: descriptors, available ring, used ring, request/status and bounce buffer. Header: 16 bytes readable by device; bounce buffer: 512 bytes and status: 1 byte writable; chain 0→1→2. Validate the complete draft before publication. Memory barriers before available/notify and after used index before data read. Device interrupts remain masked; PIT/yield provide progress.
- Submit, poll and reset are finite steps; no I/O wait loop with interrupts disabled. Success requires head 0, correct index step, used length 513 and status OK. IOERR/UNSUPP yields no data. Invalid ID/index/length/status takes a local error path.
- Published descriptors and bounce buffer remain unchanged until completion or confirmed reset (status 0). Cancel/exit detaches the result and requests reset; never clear early. Without reset acknowledgement, the device remains offline and the pool quarantined until VM stop. Pool pages do not return to the frame allocator.
- Clearing PCI bus mastering does not itself demonstrate termination of old DMA. A used entry has no epoch/nonce; software tickets do not detect malicious replay with the same head. Protocol-conforming reset behaviour is an explicit assumption here.

## Authority and ABI 1.1

`capability-core` retains 64 endpoint objects and adds at most four BlockDevice objects. Scope contains device ID, first sector and sector count; M5 grants device 0, first sector 4 and four sectors. READ1 belongs exclusively to BlockDevice; SEND1 exclusively to Endpoint. Check object type before operational right. DERIVE4, TRANSFER8 and REVOKE16 require the type-specific mask and a subset. RECV2 is invalid for BlockDevice. The service receives READ1; a broader manifest root is attenuated and closed before user entry.

For M5, `abi_info` is `(1<<32)|1` (major 1, minor 1); M1–M4 remain 1.0. New errors: -12 EIO, -13 ETIMEDOUT, -14 ENODEV and -15 EPROTO. Register convention remains unchanged.

| Syscall | Arguments | Result |
|---|---|---|
| 0x40 block_submit | cap, relative sector, offset, length; R8/R9 zero | RDX opaque ticket; no user pointer |
| 0x41 block_poll | cap, ticket, output pointer, size 320; R8/R9 zero | RDX length; Message320 with sender 0, tag ticket, data and all other bytes zero |

A ticket binds PID, object key/generation, transport epoch and a monotonic ID without wraparound. Every poll reauthorises READ on the same object. Another valid READ handle of the same PID to the same object may poll. A stale/revoked handle fails; process exit cancels the result.

Check order: syscall/unused arguments → required poll size → capability/generation/node/object/type/READ → poll ticket or submit bounds → complete output validation → consumption and copy. Length 0 or greater than 128 returns EMSGSIZE; offset overflow/outside 512 returns EINVAL; sector outside scope returns EACCESS. Offline returns ENODEV, busy/not ready EAGAIN, incorrect/consumed ticket EINVAL. No user pointer is retained across scheduling or I/O. Invalid output does not consume a completion.

## Service protocol and restart

Payload exactly 192 bytes within Message320; app requests have cap_count 0:

| Offset | Field |
|---|---|
| 0 | version u16 = 1 |
| 2 | opcode u16: READ1, WRITE2 unsupported |
| 4 | flags u32 = 0 |
| 8 | request ID u64 |
| 16 | service epoch u64 |
| 24 | relative byte offset u64 |
| 32 | length u32 |
| 36 | status i32, request 0 |
| 40 | reserved u64 = 0 |
| 48 | data 128 bytes; request zero, unused reply bytes zero |
| 176 | reserved 16 bytes = 0 |

The receiving endpoint determines fixed scope and reply endpoint; app fields select no physical LBA or reply capability. Unexpected received capabilities are closed. Other invalid fields, epoch or ranges produce a zero-filled error response; a subsequent valid request works. The kernel observer must correlate error requests and replies and demonstrate that no block call/publication/completion occurred for them.

One M5 user ELF with private roles: service 0, client 1, supervisor 2; CPU negative cases add attacker 3. Common virtual data at `0x40010000`, message buffers at `0x40011000`/`0x40012000`; frames remain private per process. Client digests at offsets 120/128. No seed or expected disk bytes in guest configuration.

A bounded kernel test coordinator may restart only the fixed service slot after an actual fault/exit and safe device quiescence. No general spawn API. Client PID, address space and progress remain alive. The new service receives a new PID, epoch and endpoint objects. It sends two actual SEND|TRANSFER request capabilities through IPC to the supervisor, which forwards SEND-only through the client control endpoint. No silent handle rewriting. The client observes old EREVOKED, closes those handles and resumes. Case 13 repeats three pending-read crashes and reads successfully in epoch 4. This does not demonstrate that DMA physically coincided with the fault.

## Acceptance cases

Every case requires kernel observations of calls, publication, completion, bytes, resources and progress. User status alone is insufficient. Only synthetic checksum metadata is logged. Expected termination: `M5:CASE:NAME:PASS`, `M5:SURVIVOR_PROGRESS:PASS`, `M5:RESOURCES:PASS`, `M5:PASS`, `M1:PASS`, raw exit 33, no timeout or unexpected FAIL.

| Case | Feature | Required evidence |
|---:|---|---|
| 0 | milestone-m5, READ | client→service→virtio→reply; sectors 4/6 through four slices; two different images with the same kernel binary |
| 1 | test-m5-service-errors | version/op/flags/reserved/len0/129/overflow/epoch/scope/capabilities rejected before I/O; then valid request |
| 2 | test-m5-device-rights | Incorrect type/rights/scope/ticket/capability; invalid poll output unchanged and not consumed; then valid poll |
| 3 | test-m5-async | Test-only notify delay; actual EAGAIN/PIT/peer progress and subsequent completion |
| 4 | test-m5-bad-descriptor | Draft injection rejected before publication/notify; no unsafe DMA; recovery |
| 5–9 | test-m5-used-id/index/short/long/status | Local snapshot injection after actual completion; no corrupt usercopy; local error/reset/recovery |
| 10 | test-m5-io-error | Injected IOERR status returns EIO, no data, recovery |
| 11 | test-m5-timeout | Test-only suppress-notify, tick deadline, reset acknowledgement, no early pool reuse, recovery |
| 12 | test-m5-reset-timeout | Test-only failure to observe reset acknowledgement; offline/quarantine/stable pool/peer progress; no data digests |
| 13 | test-m5-restart | Three actual service crashes, confirmed reset, new PID/epoch, actual IPC rebinding, revoked old authority, stable resources |
| 14–16 | test-m5-cpu-mmio/dma/pci | User read of MMIO/pool or PCI-out faults only attacker; service/client continue working |
| 17 | test-m5-no-device | Test device actually omitted; boot ESP not selected; local ENODEV, peer progress, no submissions |

Exact feature names appear in tools/build.py. All cases except 12/17 must recover and provide both sector checksums. The host generates a different image for every run; the guest knows no fixture answer. `M5:SECTOR4_DIGEST:<16hex>` and `M5:SECTOR6_DIGEST:<16hex>` derive from received bytes. The host compares FNV-1a64 and the unchanged SHA256 of the full image; internal guest assertions compare copies byte by byte. FNV is diagnostics, not cryptography.

Additionally: host tests for types/rights/transfer/revoke/quota/generation, PCI loops/BAR overflow, descriptors, completion fields, tickets/no wraparound, reset and quarantine. Fault injections remain explicitly labelled and compile-time test-only. Full M1–M4 regression, review by another agent, recovery and final hash/remote checks are required.

## Ownership and delivery

- kernel_m2: kernel/**, crates/kernel-core/**, drivers/virtio-blk/**; transport, scheduler, manifest, kernel observer.
- quantum_contract: crates/virtio-core/**, crates/capability-core/**; pure policy/parser, typed capabilities, host tests.
- product_acceptance: services/block/** and new M5 userspace files; service/client/supervisor, protocol, artefact tests. Retain existing probes.
- Root: workspace/lockfiles/tools/docs, images, build/run harness, branches/checkpoints. At most three active agents, one writer per file; initial reviews read-only.

After meaningful steps and before long tests, push privately to `codex/m5-block-service`. Retain source, pins, actual logs/exits and open points; no binaries/caches/secrets. Recover through a separate consistent checkout at `c630c59`, not a destructive reset.

## Primary sources

OASIS Open, *Virtual I/O Device (VIRTIO) Version 1.2*, Committee Specification 01, 1 July 2022: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html — §§2.4, 2.7, 3.1, 3.3, 4.1, 5.2. Version 1.2 is the selected protocol baseline, not a claim of being the latest version.

QEMU Project, system manual page and locally installed 11.1 help: https://www.qemu.org/docs/master/system/qemu-manpage.html — device/BDF/readonly options. Targeted check on 27 October 2026. Quotas, ABI and acceptance cases above are project decisions.
