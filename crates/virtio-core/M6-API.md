# C06 — bounded WRITE/FLUSH core

Build baseline 448974b, contract docs/implementation/M6A-WRITE-RECOVERY.md v1.
Safe Rust/no_std, zero dependencies, no heap/unsafe/MMIO or user pointers.
This document describes the pure interface; hardware ordering, actual reset
acknowledgement, ownership/pinning and user copy remain the kernel's responsibility.

## Typed capabilities

Endpoint ALL_RIGHTS remains 31; Block READ1, WRITE32, FLUSH64, DERIVE4,
TRANSFER8, REVOKE16. Block bit2 remains invalid. KNOWN_RIGHTS contains 127,
but type-specific validation prevents Endpoint-WRITE/FLUSH or Block-RECV.

`create_block_device(owner,scope)` continues to create root29.
`create_block_device_with_rights(owner:u64,scope:BlockScope,rights:u64)` creates
only explicitly selected rights within 125; the manifest can create root125
and delegate 97 to the service. `check_block` checks type before right and
returns the same immutable scope/opaque object key. FLUSH affects the
device-wide cache; the kernel must check WRITE/READ against sector scope.
No new userspace creation syscall, object quota or lifetime infrastructure.

## Operations and fixed descriptor chains

```rust
enum Operation {
    Read { sector:u64, offset:u64, length:u64 },
    Write { sector:u64 },
    Flush,
}
struct DescriptorChain { pub entries:[Descriptor;3], pub count:u8 }
fn negotiate_writable_features(offered:u64) -> Result<u64,Error>;
fn descriptor_plan_for(op:Operation,pool:DmaPool,ram_end:u64)
    -> Result<DescriptorChain,Error>;
fn validate_descriptors_for(chain:&DescriptorChain,op:Operation,
    pool:DmaPool,ram_end:u64) -> Result<(),Error>;
fn header_for(op:Operation) -> [u8;16];
fn validate_used_for(op:Operation,head:u16,old:u16,new:u16,
    id:u32,len:u32,status:u8) -> Result<Completion,Error>;
```

WRITE is always 512B/offset0; FLUSH is header(type4,sector0)→status1 and has
no data descriptor. READ has header→512device-WRITE→status; WRITE has
header(type1)→512device-READ→status. Chain counts are 3/3/2; the unused FLUSH
descriptor is exactly zero and also validated. Successful used.len is
513/1/1; IO/UNSUPP returns no payload. The decoder never produces Unknown.

The writable profile requires VERSION1 and FLUSH9, rejects RO/ACCESS_PLATFORM
and negotiates only VERSION1|FLUSH. CONFIG_WCE is not negotiated.
Existing M5 feature/descriptor/header/used wrappers remain read-only.

## Publication and uncertain outcome

```rust
Policy::prepare(binding:Binding,op:Operation,capacity:u64,now:u64,timeout:u64)
    -> Result<Request,Error>;
Policy::publish(epoch:u64,ticket:u64) -> Result<(),Error>;
Policy::reject_prepared(binding:Binding,ticket:u64) -> Result<(),Error>;
Policy::exit(pid:u64) -> Option<Request>;
```

Request retains its old fields and adds `operation:Operation` and
`published:bool`. New path: Ready→Prepared→Published. Legacy submit(Read)
remains Phase::Pending and is conservatively already exposed. New transports
call publish immediately before publishing avail.idx, not only at notify.
Rejecting Prepared publishes nothing and removes the ticket. Do not reread
untrusted fields after snapshot/publication; this crate does not know the
actual user buffer.

PID/objectkey/epoch/ticket are monotonic bindings; Request.operation determines
the required polling right. The kernel reauthorises that right on a current
capability. Another valid capability to the same object within the same PID
remains usable.

Published WRITE/FLUSH with IO/UNSUPP becomes Completion::Unknown. Timeout or
start_reset without an observed completion also retains Unknown. A validated
successful WRITE remains Success, but does not mean durability; a later
successful actual FLUSH is required. No automatic replay. Completed results
do not expire while waiting for a user poll.

Reset acknowledgement increments the transport epoch but retains unconsumed
mutator results with the original ticket/epoch. Peek/consume accepts only
their exact binding; new prepare remains Busy until the result is processed.
Unknown can be retrieved during Resetting/Quarantined, but consume does not
clear that phase. Exit removes only the dying PID's result and preserves
necessary reset/quarantine. Prepared-exit has no device effect. Epoch/ticket
wrap is prohibited; terminal Unknown remains readable at epoch exhaustion.

## Host evidence and limits

Thirty capability and 21 virtio host tests preserve all 38 original tests
and add 13 transaction tests. One original unknown-bit probe was explicitly
updated because WRITE32 had been allocated in advance: legacyroot29+WRITE32
now returns EACCESS; bit128 still returns EINVAL. Both compare the complete
Engine before/after the error. The test and generation-retirement checks
are preserved.

New tests cover typed WRITE/FLUSH grants, all 256 small rights masks,
revoke/IPC/exit/generation/quota, all descriptor fields including the unused
FLUSH entry, operation-specific used lengths, Prepared/Published, Unknown
across reset, consume during reset/quarantine, owner exit and no-wrap.

Commands/exit statuses/logs/hashes are stored locally in verification/m6-report.json;
the lead agent retains the compact version in docs/verification. These host
tests do not demonstrate an actual input512 snapshot, PCI/DMA/FLUSH,
QEMU kill/recovery or power loss. VIRTIO used entries have no epoch nonce;
software binding does not detect malicious device replay. Kernel transport
and QEMU/backend remain part of the TCB.

Recovery requires a consistent return to 448974b for both crates and their
new kernel callers; do not revert one shared interface in isolation.
No external dependencies or lockfile changes were added.
