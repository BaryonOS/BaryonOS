# capability-core — M4/M5 authority engine

Dependency-free `no_std` library with `forbid(unsafe_code)`. All process,
handle, node, endpoint and queue storage has fixed capacity. This crate has
no allocator, clock, I/O or user pointer. Kernel initialisation uses
`static mut ENGINE: Engine = Engine::new();`; `new` is a const initializer,
so the large tables are not constructed on a kernel stack.

## API and transaction boundary

`Engine` provides `register`, `create_endpoint`, `bootstrap_delegate`,
`check_endpoint`, `close`, `derive`, `revoke`, `send`, `receive`, `exit`, `stats`
and `check_invariants`. PIDs are `u64`; registration is strictly increasing
and an old PID cannot return after exit. A process row may be reused,
but slot generations and retired slots are retained.

`Message` (alias `Message320`) contains the fields of the fixed 320-byte protocol.
`decode(&[u8; 320])` validates a **send request**: excessive `len/cap_count`
returns `MessageSize`; non-canonical sender/reserved/unused/tail fields
return `Invalid`. Encode/decode use bytes with little-endian integers,
without struct casts or alignment assumptions. `encode()` can also write
a receive reply with a populated `sender_pid`; send decoding deliberately
does not accept that reply as a new send request. `send` also validates
`Message` values constructed directly.

The kernel checks syscall number/arguments, size and endpoint authority
before user copy. It takes one private input snapshot, then calls decode/send.
For receive, it validates **all 320 output bytes before `Engine::receive`**.
Receive reserves all local slots before changing the queue; the kernel must
then copy the initialised output without any remaining possibility of failure.
These rules are required: the core does not validate user memory and cannot
subsequently undo an incorrect kernel copy.

Lookup order: local slot/generation → node/generation/revoked →
object/generation/open → required right → requested fields → resources.
A capability lacking DERIVE/TRANSFER that also requests unknown new rights
therefore returns `Access`; a capability with the required right but unknown
rights returns `Invalid`. All error values are exactly -1 through -11 via
`Error::code()`.

## Ownership, revocation and lifetime

- 16 processes, 255 usable slots per process, 1,024 nodes, maximum depth eight.
- 64 endpoint records, at most four allocated records per owner, 16 FIFO
  messages per endpoint and at most two transferred capabilities per message.
- TRANSFER creates a new derived node without moving the source handle.
  RECV is never transferred; an owned local derivation may retain RECV.
- `revoke` revokes descendants only. `close` does not revoke descendants and
  also cleans up revoked/closed handles. Exit revokes owned slot nodes and
  nodes issued by the process, plus descendants.
- Revocation cancels an entire queued message when any capability in it is
  revoked. Remaining FIFO order is preserved. Delivered bytes remain with
  the recipient.
- Closing the last active RECV handle, or owner exit, closes the endpoint and
  cancels its queue. Other non-revoked handles return Closed; revoked nodes
  continue to return Revoked before object status is examined.
- Reference counts include local slots, children and queue references. Parent
  nodes and object tombstones remain while references exist. Closed tombstones
  therefore still occupy object capacity, including the owner limit; capacity
  returns after the last reference. Generation counters are never reused or
  wrapped: a maximum generation is retired on release.
- Revocation examines at most 1,024 nodes with at most eight parent steps per
  node and at most 1,024 logical queue messages. Garbage collection uses one
  node scan with a bounded ancestor cascade, without recursion or a heap.

`check_invariants` independently recounts references and checks ownership,
parent rights/depth, live generations, queue capabilities and resource bounds.
`Stats` is Copy/Eq and contains a complete diagnostic `state_digest`. This
is a fast non-cryptographic fingerprint for tests, not an authorisation
mechanism or proof against hash collisions. Do not run the full scan on every
ordinary syscall path; negative kernel experiments can compare before/after.
Host tests compare the complete `Engine` state without assuming no hash collisions.

`Stats` additionally reports global live counts and free, non-retired capacity
for the existing process, handle, node, endpoint, queue and typed-object
tables. In the current model, `threads` counts one scheduler context per
registered process; threads cannot be created independently. Availability is
derived per resource and reserves nothing: rights, owner quotas and combined
allocation conditions remain separate checks. The snapshot excludes process
memory, shared or pinned buffers, CPU time, storage and driver resources, and
is not a kernel or userspace reporting API.

## Actual execution

Rust 1.90.0, Windows MSVC host, separate target directory
`build/capability-tests`. The original M4 execution contains 21 passing unit tests:

| Contract | Host evidence |
|---|---|
| KS-10 | Exact wire offsets, canonical fields, large integers, unaligned byte source, 2,560 individual bit mutations |
| KS-11 | Queued snapshot independent of later source changes; FIFO and ring wrap |
| KS-12 | Local numeric handle collision, cross-process lookup, stale slot/generation |
| KS-13 | Rights attenuation, zero rights, missing DERIVE/TRANSFER/SEND, RECV transfer, combined error order; both transferred capabilities fully validated before depth quota |
| KS-14 | A→B→C, child before parent in the node table, revoke before/after delivery, whole-message cancellation |
| KS-15 | Queue16, slots255, two slots required for receive, 1,024 nodes and two-capability send with one node free, depth8/9, owner/global object quotas, maximum slot/node/object generations |
| KS-16 | Last RECV, tombstones, revoked before closed, sender exit with queued capabilities, issued rights and complete reclamation |
| Extra | Strictly monotonic PID/row reuse, exact error numbers, 800 deterministic mixed transactions with full snapshots and invariants |

KS-09, actual syscall/ring3 boundaries, scheduler/mapping lifecycle, user copy
and infallible output copy are tested by kernel integration. These host tests
do not demonstrate that boundary. `NoSys`, `Fault` and `Type` have the correct
numeric API values; NoSys/Fault are kernel boundaries. The M5 extension below
actually tests Type between Endpoint and BlockDevice.

Exact checks from the project root, with the project toolchain and
`CARGO_TARGET_DIR` set to `build/capability-tests`:

```text
cargo +1.90.0 fmt -p capability-core -- --check
cargo +1.90.0 test -p capability-core --locked --offline -- --test-threads=3
cargo +1.90.0 clippy -p capability-core --all-targets --locked --offline -- -D warnings
cargo +1.90.0 check -p capability-core --target x86_64-unknown-none --locked --offline
git diff --check -- crates/capability-core
```

Execution: exit 0 for the latest versions of these checks. Local logs and hash
report: `verification/`; the lead agent retains the compact evidence version
under `docs/verification/`. No production data, cryptography, dependencies or
host settings were added. Recovery: revert only this crate and its workspace/
kernel dependency managed by the lead agent, together with callers.

## M5 — typed read-only BlockDevice

Four additional object records sit alongside the existing 64 endpoint records;
endpoint capacity is not reduced. `BlockScope {device,first_sector,
sector_count}` is immutable per object and checked for zero/overflow before
creation. Relative sectors are checked and translated through
`BlockAuthorization::sector`; the transport layer additionally checks device capacity.

Boot-manifest API: `create_block_device(owner,scope) -> handle` with root rights29.
`check_block(pid,handle,BLOCK_READ) -> BlockAuthorization` validates handle/
generation/revoked/object and then **type before right**. The opaque
`object_key()` contains a type-separated object index and generation.
`scope()` returns only a copy of the fixed scope; there is no userspace
creation syscall.

For BlockDevice, READ1, DERIVE4, TRANSFER8 and REVOKE16 are valid; bit2 is
invalid. Endpoint-SEND1 cannot become BlockDevice-READ1. A READ capability
without DERIVE returns Access before an invalid new rights mask; a valid
admin capability with bit2 returns Invalid. Local derivation and IPC transfer
use the same bounded tree/reference counts/revocation/issuer-exit as endpoints.
BlockDevice has no RECV handle or queue and is not subject to last-RECV closure.
After adminclose, a derived READ handle remains usable. The last node reference
reclaims the object; owner exit closes tombstones that remain referenced.

The current M5 host suite contains 26 tests: all 21 M4 tests plus five
BlockDevice transaction experiments. Combined virtio/capability checks and
current hashes are under `docs/verification/m5/core/report.json`. The kernel
must reauthorise polling: a previously returned `BlockAuthorization` copy is
not a permanent exemption from revocation. Tickets bind PID + object key/
generation + transport epoch, independently of the local handle used; another
still-valid READ capability to the same scope object within the same PID may poll.

## M6 — explicit WRITE/FLUSH rights

Block WRITE32 and FLUSH64 broaden only explicitly created grants.
Legacy `create_block_device` remains root29; the new manifest-only
`create_block_device_with_rights(owner,scope,rights)` accepts only Block mask125.
Endpoint mask31, Block bit2, scopes, quotas, reference counts and generations
retain their boundaries. The combined Endpoint/Block mask is `127`; the current
global [`KNOWN_RIGHTS`](src/lib.rs) mask is `16383` and also covers launch,
process, entropy, network, display and input rights. Type-specific checks
still restrict each object to its valid rights. READ-only grants receive no
WRITE/FLUSH.

See [M6-API](../virtio-core/M6-API.md) for exact signatures, publication/Unknown
policy, test coverage, the compatibility change to the old unknown-bit32 probe,
and boundaries between pure host tests and actual kernel/device evidence.
