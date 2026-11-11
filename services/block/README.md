# First block service (M5)

This crate contains the pure wire protocol for the first isolated Rust
block service. `src/user_service.rs` is included directly in the real
`no_std` ring-3 ELF from `userspace/m5_probe.rs`; it is not a host simulation.
The host crate tests only `wire.rs`. The kernel manages the virtio transport,
the five fixed DMA pages, tickets and reset/quiescence. This is not a complete
userspace hardware driver and not evidence against malicious DMA without an IOMMU.

The service has one typed READ-only Block capability and two dedicated
request endpoints. Each endpoint is bound to one 512-byte sector,
in the disposable probe sector 4 and 6 respectively. The client holds only
SEND on these endpoints. The service selects the sector from the receiving endpoint
and replies exclusively via the pre-bound reply endpoint. An app cannot
supply an absolute LBA, device capability or reply handle.

## Wire contract version 1

A request or reply is exactly 192 payload bytes in the existing Message320.
All integers are little-endian; there are no Rust struct-layout assumptions.

| Offset | Type/bytes | Meaning |
|---:|---|---|
| 0 | u16 | version 1 |
| 2 | u16 | READ=1; WRITE=2 yields NoSys |
| 4 | u32 | flags, zero |
| 8 | u64 | request ID, copied into the reply |
| 16 | u64 | service epoch |
| 24 | u64 | relative byte offset within the bound sector |
| 32 | u32 | length, READ request 1..128 |
| 36 | i32 | status, zero in request |
| 40 | 8 | reserved, zero |
| 48 | 128 | data, zero in request; only bytes actually read in reply |
| 176 | 16 | reserved, zero |

TAG is `0x4d35424c4b000001`. Requests carry no transferred caps.
Unexpectedly received caps are closed immediately. Then the real
kernel sender PID, the outer message, reserved/data, epoch, opcode and the
range are checked. `offset + lengte` uses checked arithmetic and must not
exceed 512. An error reply uses the current server epoch,
the same ID/opcode/offset, length 0 and all-zero data. Invalid app requests
and WRITE do not reach the backend path. The kernel observer must check this absence
of I/O independently; a client counter is insufficient for that.

App statuses: Invalid=-1, NoSys=-2, Range=-3, Access=-5, Stale=-10,
Io=-12, Timeout=-13, NoDevice=-14, DeviceProtocol=-15. These app statuses are
in the payload and are separate from the syscall results in RAX/RDX.

## Device interface and recovery

ABI info is 1.1. `block_submit` (0x40) takes
`(cap, relative_sector, offset, length, 0, 0)` and on success returns a ticket
in RDX. `block_poll` (0x41) takes `(cap, ticket, outptr, 320, 0, 0)`;
EAGAIN means the service yields. Only a successfully validated
poll Message provides bytes for an app reply. The service contains no disk seed or
expected response. After a device error only READ is retried,
at most once; reset-timeout/no-device make no data claim.

New request caps go via real IPC: the service derives SEND|TRANSFER (9),
the test supervisor receives 9 and transfers SEND (1) to the client. Intermediate caps
are closed. On restart the supervisor learns the new service PID from
Message.sender_pid and requires exactly the next epoch. The client keeps
its own address space, checks old EREVOKED handles, closes them itself and
restarts the sector assembly. Only the kernel test coordinator may, after
fault and device quiescence, replace the fixed service slot; there is no general
spawn or management API.

## Verification boundary

Host command: `cargo +1.90.0 test -p block-protocol --locked --offline`.
Use the project toolchain with a dedicated `CARGO_TARGET_DIR` under
`build/m5-service-tests`. The nine pure tests check the wire contract;
they do not prove CPL3, device I/O or restart. The dedicated ELF guide and
artifact test are in `userspace/M5-PROBE.md`.

The integrated guest gate must demonstrate malformed requests without device I/O, real
sector bytes, device error paths and three pending-read crashes with rebinding.
Two different host-generated images are compared with the same
kernel/user binary; FNV-1a64 is merely diagnostic test metadata.
Local working input under `verification/` is ignored; root keeps selected
text evidence under `docs/verification/m5/service/`. No WRITE, filesystem,
M6, network or physical-hardware claim.
