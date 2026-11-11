# virtio-core — pure M5/M6 policy

Safe Rust `no_std`, zero dependencies and `forbid(unsafe_code)`. This crate
receives only integers and fixed arrays: no raw pointers, MMIO, PCI I/O,
clock or heap. The profile is modern virtio-blk, split queue8, one outstanding
request, 512-byte READ and a user slice of 1–128 bytes in the M5 profile.
The separate M6 profile adds full-sector WRITE and FLUSH.
IOMMU integration is not provided.

## API

- `parse_capabilities(&[u8;256], &[Bar;6]) -> Regions`: PCI snapshot with
  vendor/device `1af4:1042`, header0, capability-chain cycle/length checks,
  explicit memory BARs and non-overlapping common/notify/ISR/device regions.
  The strict QEMU slice requires 8-byte common/device-offset alignment.
  `Bar` contains `base/size/is_memory`; the kernel-probed size must match the
  configuration BAR. `Regions::notify_address` checks multiplier and range.
  Addresses above the x86-64 PTE limit of 52 bits are rejected; the kernel
  additionally checks the CPU width actually supported and overlap with
  RAM/reservations.
- `negotiate_features`: requires VERSION_1 and readonly; accepts only those
  two bits. ACCESS_PLATFORM is rejected: this profile configures no IOMMU.
  Other acceleration/queue features are not negotiated.
- `descriptor_plan(DmaPool, ram_end)` and `validate_descriptors`: five unique,
  page-aligned kernel pages; exactly header16/RO → bounce512/device-WRITE →
  status1/device-WRITE. No indirect descriptor, variable chain or user address.
  `Descriptor::encode` and `read_header` produce little-endian bytes.
- `validate_used(head, old_idx, new_idx, id, len, status)`: no new entry
  returns Pending; one entry with the correct head is required. Successful
  READ requires len513. IO/unsupported with len1–513 yields a terminal error
  outcome, never permission to return bounce data as success. Unknown status,
  index jump, wrong id and invalid length return Protocol. Normal 16-bit
  ring-index wrapping is supported.
- `Policy`: const initializer, `submit(Binding, Read, capacity, now, timeout)`,
  `complete`, `peek`, `consume`, `start_reset`, `reset_completed`, `timeout`.
  Binding contains PID and a typed object key/generation. Every actual poll
  must first reauthorise a current READ capability to the same object.

## Lifetime and evidence limits

`Policy::new` begins Offline. Only a transport-confirmed reset opens Ready
with a new epoch. Submit validates bounds and checked deadlines before
publishing a ticket. Peek does not consume; the kernel validates the entire
user output before consume and the subsequent infallible copy.

Timeout/cancel moves to Resetting and retains pending metadata. Without
acknowledgement, the policy becomes permanently Quarantined; a late `true`
does not undo this. Epoch and ticket never wrap. The kernel keeps all DMA
pages pinned for the entire boot and may change exposed buffers only after
completion or verified reset.

**A virtio used entry contains no epoch or nonce.** Parameters of
`Policy::complete(epoch,ticket,...)` come from kernel metadata. The old-epoch
host test demonstrates software binding only; it does not detect malicious
device replay with the same reused head id. Actual status0 reset, barriers
and queue quiescence must be demonstrated in the transport driver.

The pure pool check does not establish actual frame ownership, pinning,
cache type or DMA containment. The kernel must bind those properties to
actual Frames/page tables/PCI registers. Without VT-d, the QEMU device
remains trusted; there is no hardware-IOMMU claim or real-device support.

## Checks

Twelve host tests cover PCI capabilities/bounds/cycles/overflow and 1,536 byte
variants, descriptor addresses/flags/chain/lengths, completion index/id/length/
status, slice bounds, PID/object binding, timeout/quarantine and epoch/ticket
no-wrap. Maximum counters are set only in the `cfg(test)` module.

Additional capability-core tests preserve all 21 M4 tests and add five
targeted BlockDevice tests: scope/type/invalidmask, adminclose,
transfer/revocation/exit, separate quotas, generation retirement and mixed
queue cancellation.

Exact commands from the project root with Rust 1.90.0 and a separate
`CARGO_TARGET_DIR=build/m5-core-tests`:

```text
cargo +1.90.0 fmt -p capability-core -p virtio-core -- --check
cargo +1.90.0 test -p capability-core -p virtio-core --locked --offline -- --test-threads=3
cargo +1.90.0 clippy -p capability-core -p virtio-core --all-targets --locked --offline -- -D warnings
cargo +1.90.0 check -p capability-core -p virtio-core --target x86_64-unknown-none --locked --offline
git diff --check -- crates/capability-core crates/virtio-core
```

Executed results, exact argv and source hashes are retained locally in
`verification/`; the lead agent copies the compact evidence version to
docs/verification. A host test is not actual PCI/DMA/reset or ring3 execution.
The kernel agent owns that separate boundary. Rollback requires removing
this crate together with the workspace/driver dependency managed by the lead agent.

Source for the interpreted fields/protocol boundary:
[OASIS VIRTIO1.2](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html),
particularly §§2.4,2.7,3.3,4.1 and5.2. No external implementation code was copied.
