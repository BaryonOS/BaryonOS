# Native example: task-grant broker (A03-T2a)

A ring-3 grant broker and one client ELF that runs as two processes: the
owner app and a hostile helper. The broker is the trusted adapter of
[`nieuw-os-task-grant-core`](../../task-grant-core/README.md); its policy
semantics are unchanged.

## Invariant

Every request is authorized by the **kernel-supplied sender PID** of the IPC
message (`Received::sender()`, set by capability-core to the authenticated
sending process). No app-supplied value authenticates an operation: the
request's claimed actor field is decoded and then ignored, object numbers only
select an authoritative object, and model handles never leave the broker.
CLOSE succeeds only for an actor that holds a live grant on the object, so a
helper cannot revoke another task's grant. Replies are routed by the sender
PID to a per-client reply endpoint fixed at bootstrap.

## Wire protocol

Tag `0x41335447`. Request: 24 bytes, three little-endian u64 values
`(operation, object, claimed_actor)`, operations READ=1, WRITE=2, CLOSE=3.
Reply: 32 bytes, `(status, length, 16 data bytes)`; status OK=1, DENIED=2,
BAD_REQUEST=3, plus the ordering signals GO=4 and RESUME=5. A wrong length,
tag, attached capability or unknown operation from a known client is answered
with BAD_REQUEST; a message from any other sender is counted and gets no reply.
Object 1 (revision 1) contains the 12 bytes `A03 grant ok`; object 2 exists
but is never granted.

## Fixture standing in for the trusted dialog

In the product the grant comes from a U-owned trusted permission dialog, which
is not built here. The native profile `native-task-grant` instead writes a
bootstrap grant into the broker's private startup page: owner PID, object 1,
revision 1, READ, until close. This is a **test fixture**, not a product
bootstrap ABI. The negative profile `test-task-grant-denied` omits it.

## Native acceptance

`kernel/src/m2/m3/task_grant.rs` starts the broker, the owner and the helper
(endpoint rights only), and runs this sequence, ordered by broker signals:
owner READ 1 (ok, exact bytes), WRITE 1 (denied), READ 2 (denied); then the
helper READ 1 claiming the owner's PID (denied) and CLOSE 1 (denied); then the
owner READ 1 (still ok), CLOSE 1 (ok), READ 1 (denied). The kernel compares the
broker's eight-entry decision log, its counters and the clients' recorded
statuses and bytes exactly, then audits cleanup, the frame baseline and the
capability engine. Without the fixture every request is denied.

```sh
python3 -B tools/build.py --kernel-feature native-task-grant
python3 -B tools/run_task_grant.py --case normal
python3 -B tools/build.py --kernel-feature test-task-grant-denied
python3 -B tools/run_task_grant.py --case denied
```

## Trust assumptions

- PID uniqueness: `kernel-core` `issue_pid` hands out strictly increasing PIDs
  with checked overflow, and capability-core `register` rejects any PID that
  is not greater than the last registered one. A PID is therefore never
  reused within one boot. The model's epoch is a constant boot-scoped value
  (`EPOCH = 1`); it does not distinguish boots, so grants must not outlive the
  broker process. The app principal equals the PID until an app identity
  registry exists.
- The kernel bootstrap and the broker are trusted; the clients are not.

## Evidence limits

QEMU evidence of one fixed sequence on x86_64. Not demonstrated: the trusted
dialog, Materiaal volume per-object enforcement (no block or volume rights are
granted and object content is a constant), lifecycle-driven `retire_process`
on client exit, revision advance, session end, WRITE/EXPORT paths, and any
physical hardware. Revocation cannot erase bytes already read. A03 remains
open.
