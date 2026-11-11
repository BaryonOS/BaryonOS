# M6b — native bounded object service

Own dependency-free Rust core with `no_std` and `forbid(unsafe_code)`. `wire.rs`
and `journal.rs` are tested on the host and included in the native ELF.
`user_service.rs` is the actual ring-3 adapter and is compiled directly
by the native user fixture with `crate::{rt,wire,journal}`.
The library alone performs no I/O and proves no native isolation/durability.

## Authorities and boundaries

Four object slots, IDs0–3. The kernel creates two service endpoints: the receiving
endpoint determines domain0 (object0/1) or domain1 (object2/3). The real
`Message.sender_pid` must moreover match the client PID bound to that domain.
A request contains no domain, reply capability, device capability,
absolute sector address or pointers to the storage. Unexpected received
capabilities are closed before further processing.

The service owns one Block capability with READ|WRITE|FLUSH97, sectors4–35 of
the synthetic disk. Clients receive exclusively SEND to their own
service endpoint and RECV on their own reply endpoint. Rights on the same
endpoint cover, in this first probe, GET/PUT/DELETE within that domain.

At most128bytes per object,32records of512bytes, fixed tables, no heap,
no GC, no network and no encryption. This is a native storage foundation;
not a filesystem, NAS, general app API or product-suitable storage limit.

## Mutations and recovery

- GET returns content and object revision. A never-existing object yields
  `NotFound=-19`, revision0; a deleted object yields NotFound with its
  retained tombstone revision. A present empty object yields success, length0.
- PUT contains the expected revision. CAS0 creates a new object; the current
  tombstone revision enables explicit re-creation. Success increments the
  object revision by one.
- DELETE requires the current revision and an existing object. An old revision
  yields Stale; an already absent object with current revision yields NotFound without
  I/O. The tombstone keeps the incremented revision and contains no old payload.
- `State::prepare` changes nothing. The adapter writes exactly one new sector,
  waits for WRITE completion and then for real FLUSH completion, only
  then calls `State::commit` and sends the app reply. A prepared record
  may be committed exclusively on its original state.
- Service contract: every uncertain
  mutation outcome sets the service to `Unknown`. Otherwise valid requests get
  `Unknown` until a new process recovers the entire journal. WRITE/FLUSH
  are never repeated automatically. On source `592dccf` the observer panic
  has been repaired and two local WRITE/FLUSH status snapshot errors have been natively
  checked. New kernel deadline/reset and poll LIMIT probes are
  separately under construction. On locally abandoned polls the adapter closes its
  only block capability, so that the kernel can safely cancel and reset the pending
  request. This is no automatic write repetition.
  See the [evidence boundaries](../../docs/implementation/M6B-OBJECT-STORAGE.md#bewijsgrenzen).
- `request_id` correlates request and reply; it is not a separate idempotency system.
  An old request with an old CAS revision is rejected.
- Recovery scans all32sectors in order. Only complete valid prefixes
  followed exclusively by zero sectors are accepted. A damaged
  occupied record, gap, wrong order, revision jump or delete without an existing
  object yields Corrupt. Recovery guesses no earlier value and does not format.
- An empty, fully zero journal is a valid empty state. A full journal
  yields Full before WRITE. This step claims no recovery on physical power loss.

## Wire protocol version3

Payload is exactly224bytes in Message320; TAG=`0x4d36424c4b000003`.
All integers are little-endian; unused bytes and payload tail are zero.

| Offset | Meaning |
|---:|---|
|0/2|u16 version3 / operation GET1, PUT2 or DELETE3|
|4|u32 reserved0|
|8/16|u64 request ID / service epoch|
|24/32|u64 object ID / expected or returned revision|
|40/44|u32 length / i32 status|
|48–64|reserved0|
|64–192|payload, at most128bytes with zero tail|
|192–224|reserved0|

`request(id,epoch,object,revision,op,value)` and
`reply(header,epoch,revision,status,value)` construct bounded arrays.
`validate_request(bytes,epoch,domain)` checks format, identity scope and
semantics; `validate_reply(bytes,request_header,epoch)` checks correlation,
revision and canonical replies. All error replies contain length0 and no
payload; only NotFound may return the already authorized object revision.

## Journal format

Sector512: magic `NOSM6B01`@0; version u16=1@8; header u16=64@10;
flags u32=0@12; sequence u64@16; request-ID u64@24; object u64@32;
revision u64@40; PUT2/DELETE3 u16@48; length u16@50; reserved0@52–64;
payload@64–192 with zero tail; padding0@192–504; checksum u64@504.

Sequence is exactly slot index+1. Object revisions increase per object by exactly one,
including deletes. FNV-1a64 over all512bytes with the checksum field zero is
only diagnostic corruption detection, not authentication or a collision guarantee.

Public core API: `State::{new,sequence,get,prepare,commit}`,
`Prepared::record`, `decode_record`, `Recovery::{new,push,finish}`.
`Recovery::push` has a persistent error state: a record once rejected
cannot be polished away by later giving the same scanner other bytes.

## Native startup and observation

Startup offsets: epoch32, Block-cap48, ingress56/80, reply88/72 for domain0/1,
client-PIDs160/168, service-PID176, case112. The runtime supplies fixed
MSG/OUT/SECTOR buffers and the existing scalar int80 ABI.

Service writes sequence128 and status232. Stage192:1 ready (only after full
recovery and FLUSH),2 before WRITE,3 after WRITE completion before FLUSH,4 after
FLUSH completion/commit before app reply,5 Unknown. Startup error sets status232
and completion40=1 without READY. These fields are diagnostics; the kernel observer
must independently follow real device publications/completions and IPC replies.

## Checks

Targeted host commands from the root:

```text
cargo +1.90.0 test -p object-core --locked --offline
cargo +1.90.0 check -p object-core --target x86_64-unknown-none --locked --offline
```

Executed commands, native QEMU results, source hashes and limitations are
recorded by root under `docs/verification/`; this guide is no
substitute for that evidence. No existing M6a volumes or protocols are migrated:
M6b has its own profile, magic, version and disposable image.
