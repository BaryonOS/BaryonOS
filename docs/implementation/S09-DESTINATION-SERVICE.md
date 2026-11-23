# S09 — Backup destination service, trusted portal and per-operation rights (design specification)

**Status: design specification only. Nothing in this document is implemented,
tested or active, and it is not a production security boundary.** It specifies
D1 (b), D2 (a), D3, D4 (a) and D5 (a) of
[ADR-S09-DESTINATION-20261119](../DECISIONS.md#adr-s09-destination-20261119--backup-destination-service-trusted-dialog-and-os-backup-portal).
The ADR's implementation boundary applies unchanged:

> A separate destination service and any new production capability boundary
> require a documented interface, a security review, negative tests and
> verified enforcement before activation.

This document is the documented interface. The negative tests of section 6 and
verified enforcement remain open. The existing
test-fixture grant in [S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md) ("Native
slice") remains a fixture and is not changed by this text. S09 stays open.

**Revision 2 (23 November 2026).** Revision 1 answered the independent AI
review [SPEC-REVIEW-1.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-1.md)
(REQUEST_CHANGES, 13 required items). Revision 2 answers the independent AI
re-review [SPEC-REVIEW-2.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-2.md)
(REQUEST_CHANGES, required findings F1–F6, optional F7–F15). Section 9 records
the resolution of each item of both reviews. Rules that need a project-owner
decision are marked *[proposed; owner decision Oxx]* and are not decided
(O4 (b), O4 (c), O11, O12, O13). The independent AI re-review of revision 2,
[SPEC-REVIEW-3.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-3.md),
returned APPROVE with five optional findings, all applied (section 9). An AI
review is not a human audit; route step R1 (section 8.2) is complete as an AI
design review only, and the owner decisions above stay open.

**Evidence labels used below.** *[main]* means present on `main` at `035b44e5`.
Most such claims were checked by review 1 or by the verification of it for this
revision; the `sdk/task-grant-core` and `services/net-broker` claims of 3.3 and
the virtio-rng slot of `tools/run_keyring.py` were spot-checked only by
identifier, and the volume-service claims of 4.2 and the entropy flow after
PR #405 were not rechecked for this revision. PR #396
(the test-only second virtio-blk disk at PCI `00:06.0`, the portal-only
`BlockScope { device: 1, .. }` grant, the 22-stage volume gate, the `SimDisk`
host tests, [S09-DUAL-DISK-FIXTURE-REQUEST.md](S09-DUAL-DISK-FIXTURE-REQUEST.md)
and `docs/verification/s09-archive/dual-disk-native-20261119/`) is merged as
`8a7a3594`, an ancestor of `035b44e5`, and its content is labelled *[main]*.
Kernel PCI discovery at a slot other than `00:06.0` remains unchecked (section
7). *[design]* means specified here and not implemented. *[proposed]* means a
design rule that awaits a project-owner decision. *[unproven]* means an
assumption without a test.

## 1. Scope and non-goals

In scope: the destination service interface, the trusted backup portal and its
destination-selection flow, per-operation rights, source/destination isolation,
a negative test matrix, a disposable-QEMU verification plan and a staged route.

Not in scope and not changed:
- **Archive format v1** ([S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md), "Bytes")
  is the entry payload, byte for byte. No field is added.
- **Restore semantics** stay as specified: restore never overwrites, assigns new
  revisions and times, and requires `verify` on the same bytes first. Faithful
  native recovery of revision history and timestamps is a separate task that
  must first identify the required format and contract changes; this document
  neither assumes nor claims it.
- **ustar/pax export (D6 (a))** remains the lossy interchange export. It is not
  a destination entry type and nothing restores from it.
- **Encrypted NAS and provider destinations (D5).** Per the ADR they require
  the existing key-management and client-side encryption contracts
  ([ENCRYPTED-OBJECT-STORAGE.md](ENCRYPTED-OBJECT-STORAGE.md)). This design
  covers only local, user-owned block devices or, if the owner permits it,
  partitions; the service refuses any other destination kind by construction,
  because it only accepts a block capability. Whether a partition may serve as
  a destination, and under which condition, is open: section 2.1 proposes a
  separate device *[proposed; owner decision O11]*.

## 2. Destination service interface (D1 (b))

### 2.1 Ownership

The destination service is a userspace system service *[design]*. At boot the
manifest gives it exactly one block capability on the whole destination device
(or, if O11 permits it, a partition): `create_block_device_with_rights(service, scope, BLOCK_READ |
BLOCK_WRITE | BLOCK_FLUSH)`. The legacy `create_block_device` constructor must
not be used, because its default `BLOCK_READ_RIGHTS` includes DERIVE, TRANSFER
and REVOKE *[main]*; with them the service could pass raw device authority on.
No other process holds any capability on the destination device. The service
never writes outside the region rules of section 2.6, and it never holds a
capability on the source volume's block object (section 5).

**Manifest-only, non-transferable grant.** The grant is created only by the
manifest (K1) and carries no TRANSFER. On `main`, spawn (syscall 0x50) is
type-blind: it delegates any handle whose node holds TRANSFER, within a rights
subset (`check_delegation`, `bootstrap_delegate`) *[main]*. Every block grant in
a profile that has spawn is created without TRANSFER *[main]*, and that keeps a
process from passing block authority to a spawned or restarted process; the
kernel does not enforce it by object type. This document therefore requires the
destination grant to carry exactly READ|WRITE|FLUSH, and N18 probes those exact
rights, independent of the owner decisions below.

The absence of TRANSFER does not stop kernel profile code from creating a new
grant. On `main`, the M5 profile's `bind_service` creates a new block object
with `create_block_device` on every kernel-driven restart of its service
(`kernel/src/m2/m3/m5.rs:260-279`, called from `poll` at `:784-794`, at most
three restarts) *[main]*. **Once-per-boot grant rule** *[proposed; owner
decision O13]*: the destination profile creates the destination block object
exactly once per boot, at profile start, and after profile start no kernel
path (restart, rebind or poll) creates a block object whose scope intersects
the destination scope or delegates a handle on one. The rule is stated by
scope, not by object key, because a re-created object has a new key over the
same sectors. It is a profile code rule: the kernel does not enforce it, and
N32 is its test. Consequence, under this rule only: no instance in the same
boot can regain destination block authority after the service exits, and
recovery from the failed-mount state (2.6) and from the read-only safe path
(2.7) is a reboot. No in-boot restart route is specified (section 8, O8). The
alternative for the owner is a kernel write latch in the per-device broker
(O13).

**Device rule** *[proposed; owner decision O11]*. In v1 the destination scope is
on a block device of its own: `scope.device` differs from that of every other
block scope in the profile. `archive_grant` compares scopes with `check_block`
for the fixture (`kernel/src/m2/m3/volume.rs:749-753`) *[main]*; this is a
precedent for the comparison, not an existing different-device rule (those
lines check that the volume service's own block handle is a different object on
device 0). The alternative for the owner
is a partition whose scope is disjoint from every other block scope; such a
partition shares the device's reset and FLUSH with the source, so a source
fault can stop the destination and the reverse, and it is not an independent
copy against loss of that device. Until O11 is decided, every stage uses a
separate device (disk 1), and N18 asserts at least that the destination scope is
disjoint from every other block scope.

### 2.2 Message framing

Requests and replies use the volume service's framing *[main,
`services/volume/src/wire.rs`]* with a distinct version and operation set: a
224-byte body inside `Message320`, a 64-byte header and at most 160 data bytes.

| Offset | Field | Request rule | Reply rule |
|---|---|---|---|
| 0 | version u16 | = 1 (destination wire v1) | = 1 |
| 2 | operation u16 | 1..=11 (table 2.3), else `NoSys` | echoed |
| 4 | 4 × 0 | must be 0, else `Invalid` | 0 |
| 8 | request_id u64 | ≠ 0, strictly increasing per lane binding (control lane: per service start), else `Stale` | echoed |
| 16 | epoch u64 | = grant generation of the lane binding (control lane: 0), else `Stale` | echoed |
| 24 | entry u64 | entry ID (0 only where the table allows) | entry ID |
| 32 | offset u64 | byte offset (APPEND, READ) or total length (SEAL), else 0 | next offset or total length |
| 40 | length u32 | ≤ 160 and = data length, else `Invalid` | reply data length ≤ 160 |
| 44 | status i32 | must be 0 | service status (2.4) |
| 48 | name_length u32 | CREATE: label length 1..=32 and = `length`, else `Invalid`; BIND: operation 1..=3; else 0 | per operation |
| 52 | count u32 | LIST: cursor < 16; BIND, UNBIND: lane 1..=3; else 0 | per operation |
| 56 | aux u64 | READ: byte count 1..=160; BIND: grant generation; else 0 | per operation |
| 64 | data[length] | bytes after `length` must be 0, else `Invalid` | zero-padded |

**Field rules per operation (check 2).** "any" means not checked at check 2;
"≠ 0" means `Invalid` -1 if 0.

| Op | entry | offset | length | name_length | count | aux |
|---|---|---|---|---|---|---|
| INFO, FORMAT | 0 | 0 | 0 | 0 | 0 | 0 |
| LIST | 0 | 0 | 0 | 0 | < 16 | 0 |
| CREATE | 0 | 0 | 1..=32 | = `length` | 0 | 0 |
| APPEND | ≠ 0 | any | 1..=160 | 0 | 0 | 0 |
| SEAL | ≠ 0 | any | 0 | 0 | 0 | 0 |
| ABORT, VERIFY | ≠ 0 | 0 | 0 | 0 | 0 | 0 |
| READ | ≠ 0 | any | 0 | 0 | 0 | 1..=160 |
| BIND | 0 if `name_length` = 1, else ≠ 0 | 0 | 16 | 1..=3 | 1..=3 | any |
| UNBIND | 0 | 0 | 0 | 0 | 1..=3 | 0 |

**Check order.** Every request passes the following checks in this order; the
first that fails gives the reply status, and no later check, no device I/O and
no state change follows, except the high-water rule below:

1. **Transferred capabilities.** If the kernel message header's `cap_count` is
   not 0, the service closes (syscall 0x10) every handle that `recv` attached
   and replies `Invalid` -1. It never keeps a received handle, so it never
   holds a volume endpoint or lane root that boot-time counting cannot see.
2. **Header validity** (table above, as `validate_request` does for the
   volume). Check 2 validates exactly these fields, none of which depends on
   service state: version; operation (`NoSys`); bytes 4..8 zero; `status` 0;
   `length` ≤ 160 and data bytes after `length` zero; and `entry`, `offset`,
   `length`, `name_length`, `count` and `aux` per the field-rule table above
   (`Invalid` -1 for every failure other than the operation). Within check 2 the
   order is version, then operation, then the remaining fields. `epoch` and
   `request_id` are check 5. Whether `entry` is the binding's entry is check
   6. Values that depend on the catalog or the entry (whether a control-lane
   ABORT names the writing entry, whether a BIND entry is sealed, whether an
   APPEND `offset` equals the appended total, whether a READ range lies inside
   the entry) are checked by the operation after check 7.
3. **Lane bound** (operation lanes only): an unbound operation lane returns
   `Access` -5.
4. **Sender** (operation lanes only): a kernel-stamped `sender_pid` other than
   the bound portal (4.3) returns `Access` -5.
5. **Epoch and request ID**: a wrong `epoch` or a `request_id` that is 0 or not
   above the lane's high-water mark returns `Stale` -10.
6. **Operation permission**: an operation that the lane or its binding does
   not permit (2.3, "Permitted on"; 4.1) returns `Access` -5. All binding-level
   limits are part of this check. On a verify or restore lane, VERIFY and READ
   must name the bound entry; on an export lane, APPEND, SEAL and ABORT must
   name the binding's writing entry; otherwise `Access` -5. An export binding
   permits CREATE only if it has issued no CREATE; APPEND, SEAL and ABORT only while its own entry is
   `writing`; and, once its entry is no longer `writing` (sealed, aborted,
   discarded, or SEAL answered `Unknown`, whatever the later resolution of
   2.7), only INFO and LIST. An export binding whose outcome is *unknown* is
   therefore limited to INFO and LIST by this check, not by check 7.
7. **State gates.** The mount state of the instance (2.6, 2.7) decides the
   reply to a request that passed checks 1–6, by the table below. The
   operation itself runs only where the table says so.

| Mount state | Entered by | INFO | Other requests that passed checks 1–6 |
|---|---|---|---|
| mounted | mount or remount with a valid catalog; FORMAT `Ok` | answered, status `Ok` | the operation runs |
| mounted, safe path | the 2.7 trigger in a mounted instance | answered, status `Ok` | CREATE, APPEND, SEAL and FORMAT `Access` -5; all others run |
| remount pending | a device failure in SEAL steps 4–5 (2.7) | not received | no request is received; requests queued meanwhile are answered afterwards under the resulting state |
| failed mount | failed mount or remount FLUSH, or a failed mount or remount read of sectors 0–15 (not the FORMAT scan, which leaves a blank candidate) (2.6); a FORMAT write or flush failure (2.6) | answered, status `NoDevice` -14 if the broker returned -14, else `Io` -12 | LIST on an export lane whose outcome is *unknown*: `Unknown` -16 with zero records; all others `Io` -12 |
| `Corrupt` | the mount rule (2.6) | answered, status `Corrupt` -18 | `Corrupt` -18 |
| blank candidate | the mount rule (2.6) | answered, status `NotFound` -19 | FORMAT and UNBIND run; all others `NotFound` -19 |

In every state other than "mounted" and "mounted, safe path", INFO's reply
has `aux` 0, `count` 0 and 48 data bytes with destination ID and formatted
sectors zero and the grant-generation maxima as in 2.3. The safe path and the
blank candidate state never coexist, because a FORMAT failure leads to the
failed-mount state. The service receives no request before its boot mount has
ended in one of these states.

A request refused by checks 1–6 leaves the lane's `request_id` high-water mark
unchanged; a request that passes checks 1–6 advances it, including one refused
by check 7 or by the operation itself. The control lane has
no binding: checks 3 and 4 do not apply, its `epoch` is always 0 and its
`request_id` sequence starts afresh when the service starts. The kernel's
`receive` attaches transferred capabilities before the service sees the
message and fails before dequeuing when the receiver has no free handle slot
*[main, `capability-core` `receive`]*; check 1 does not prevent that stall.
Only the trusted dialog holds TRANSFER on a lane (4.3), and no portal handle
carries TRANSFER (4.1), so the stall is a residual risk of a dialog fault.

**Replies.** Replies to control-lane requests go to the dialog's control-reply
endpoint and replies to operation-lane requests go to the portal's reply
endpoint (4.3). An operation-lane request whose kernel-stamped `sender_pid` is
not the portal's process ID (fixed for the boot by the manifest, 4.3) gets no
reply at all, whatever check refused it: the portal never receives a reply
with a `request_id` it did not send, and refused requests from another sender
cannot fill its reply queue. The service still closes any transferred handle
(check 1) and counts the refusal in `DEST:DENIALS=` (section 7); a request
from the portal on an unbound lane is still answered `Access` -5. The service
never blocks on a reply send: if `send` returns the
kernel's queue-full result (`Again`), the reply is dropped and the request's
effect stands. The requester checks the echoed `operation`, `request_id` and
`epoch` of every reply and treats a mismatch as `Protocol`, ending the
operation.

### 2.3 Operations

Entry IDs are u64, assigned by the service from a monotonic counter stored in
the catalog (2.6); 0 is invalid. Only a *sealed* entry's ID is an entry
identity, and a sealed entry's ID is never reused. The ID that CREATE returns
is provisional: the catalog's next entry ID advances only in the SEAL commit
(2.7, step 4), so the ID of an entry that is aborted, discarded or lost on
reboot is issued again by a later CREATE. A provisional ID has no meaning
outside its export binding and none after a reboot; BIND, VERIFY, READ and
LIST accept only sealed IDs. Labels follow
`valid_name` (UTF-8, 1–32 bytes, no `.`/`..`); labels need not be unique,
because the entry ID identifies the entry.

| Op | Name | Request | Reply | Permitted on |
|---|---|---|---|---|
| 1 | INFO | entry 0 | aux = free sectors, count = sealed entries, length 48: data = 16-byte destination ID, u64 formatted sectors, then the highest accepted grant generation of lanes 1, 2 and 3 (u64 each, 0 if never bound); in the failed-mount, `Corrupt` and blank candidate states the reply status and fields follow the table of 2.2 check 7 | every lane, control |
| 2 | LIST | entry 0, count = cursor 0..16 | up to 2 entry records of 80 bytes (2.6 layout), length = 80 × n; count = next cursor or 0 | control (all entries); export lane sees only its own entry once sealed; verify/restore lanes only their bound entry |
| 3 | CREATE | entry 0, label in data | entry = provisional ID (2.3), state `writing` | export lane, once per binding |
| 4 | APPEND | entry = writing ID, offset = bytes appended so far, ≤ 160 bytes | offset = new total | export lane |
| 5 | SEAL | entry = writing ID, offset = total length | entry state `sealed`, aux = archive trailer FNV | export lane |
| 6 | ABORT | entry = writing ID | state discarded; on the control lane `NotFound` -19 if `entry` is not the writing entry (on an export lane that is check 6) | export lane, control |
| 7 | VERIFY | entry = bound ID | aux = source generation, count = objects; or `Corrupt` | verify lane |
| 8 | READ | entry = bound ID, offset, aux = byte count 1..=160 | data = bytes; `Range` -3 with no data if `offset + aux` overflows (checked arithmetic) or exceeds the entry length, including a read that straddles the end | restore lane |
| 9 | BIND | entry = entry ID (0 for export), aux = new grant generation, count = lane 1..=3, name_length = operation (1 export, 2 verify, 3 restore), length 16: data = destination ID the grant names | Ok; `Stale` -10 with no state change if aux ≤ the lane's highest accepted generation; `NoDevice` if the mounted ID differs; `NotFound` if the entry is not sealed; `Busy` if the lane is bound | control |
| 10 | UNBIND | count = lane 1..=3 | Ok; any writing entry on that lane is discarded; the lane is unbound, so later requests on it get `Access` until a new BIND; UNBIND of an unbound lane is `Ok` with no state change, so the dialog may repeat it | control |
| 11 | FORMAT | entry 0, length 0 | Ok, reply data = new destination ID; `Access` unless the scope is blank (2.6) | control |

The request offset in APPEND must equal the service's own count of appended
bytes; a mismatch returns `Stale` and appends nothing. This makes a duplicated
or replayed APPEND harmless. SEAL with an offset that differs from the appended
total returns `Invalid` and discards nothing; the portal may then ABORT. An
APPEND or SEAL whose `entry` is not the binding's writing entry returns
`Access` -5 at check 6 and changes nothing. A second CREATE on the same export binding
returns `Access` -5 at check 6, whatever the state of the first entry. A CREATE
while an entry is `writing` on another lane returns `Busy`: one writing entry
per destination.

**Grant generations.** The service keeps, for each operation lane, the highest
grant generation it has accepted in a BIND, for the life of the instance; INFO
reports these maxima. A BIND whose generation is not above that maximum returns
`Stale` and changes nothing. BIND applies its refusals in the order of its
row: `Stale`, then `NoDevice`, `NotFound` and `Busy`. Revocation does not remove plain data messages
already queued on a lane (`maintain` cancels only messages whose endpoint is
closed or whose carried capability is revoked) *[main]*, and `request_id`
restarts with each binding; without this rule a re-BIND with an earlier
generation would let a queued APPEND or SEAL of the old binding pass checks 5
and 6 of 2.2. CREATE when 16 entries are sealed, or when
fewer than one sector is free, returns `Full`. VERIFY runs `archive::verify` inside the service
over the stored extent; it is a media check that moves no content over IPC. The
user-facing acceptance check before a restore stays `archive::verify` in the
portal over READ, on the same bytes it then restores, as the restore rule
requires.

### 2.4 Status codes

Service statuses reuse `volume_core::Status` *[main]* values: `Ok` 0, `Invalid`
-1, `NoSys` -2, `Range` -3, `Access` -5, `Stale` -10, `Io` -12, `Timeout` -13,
`NoDevice` -14, `Protocol` -15, `Unknown` -16, `Full` -17, `Corrupt` -18,
`NotFound` -19, `Busy` -21. No new code is introduced.

**Layer ambiguity.** Kernel `capability_core::Error` codes overlap numerically
with service statuses: `Revoked` and `Stale` are both -10, `Access` is -5 in
both, `Invalid` is -1 in both *[main]*. A kernel error is the syscall return of
`send`/`recv`; a service status is the reply header's `status` field. Callers
and tests must record which layer produced a code; the matrix in section 6
always names the layer.

### 2.5 Entry state machine

```
(none) --CREATE--> writing --SEAL ok--> sealed
                     |  \--ABORT / UNBIND / revoke / reboot--> (discarded)
                     |  \--entry write or flush error (APPEND, SEAL last sector, barrier 1)--> (discarded; safe path, 2.7)
                     \--SEAL fails (readback Corrupt or Timeout, Full, pre-write check)--> (discarded)
                     \--SEAL catalog write or flush error (steps 4-5)--> (decided by the catalog after remount; safe path)
```

Reply codes on these edges: an entry write or barrier 1 that fails with a
device `Io` replies `Io` -12, and one that the broker reports as `Unknown` -16
or `Timeout` -13 replies that code; a failed step-3 readback replies `Timeout`
-13 (a READ past its deadline, which the broker reports as -13) or `Io` -12;
an extent that fails `archive::verify` or a catalog copy that fails the
pre-write check of step 4 replies `Corrupt` -18; a failure in steps 4–5 replies
`Unknown` -16 (2.7). Every device error on a destination write or flush starts
the read-only safe path of 2.7; a failed readback does not, because no write
is outstanding after barrier 1 has completed. A `Corrupt` reply to SEAL
concerns that entry only and does not put the service into the `Corrupt`
mount state of 2.6.

`writing` exists only in service memory. A `writing` entry is never returned by
LIST, never readable and never verifiable. After any reboot it does not exist;
its sectors are free space. `sealed` entries are immutable: no operation writes
their sectors, and v1 has no operation that deletes them (section 8, O3).

### 2.6 On-device layout

All integers little-endian. Sector size 512.

| Sectors | Content |
|---|---|
| 0–3 | catalog copy A (2048 bytes) |
| 4–7 | catalog copy B (2048 bytes) |
| 8–15 | reserved, zero |
| 16 .. `next_free` | sealed entry extents, ascending and contiguous (CREATE allocates at the committed `next_free` and SEAL advances it past the extent, so a discarded write leaves no gap) |
| `next_free` .. end of scope | free; may hold stale bytes of discarded writes |

**Catalog copy (2048 bytes).** Header 64 bytes: magic `NOSDST\0\x01` (0..8),
version u16 = 1 (8..10), sealed-entry count u8 ≤ 16 (10), 5 × 0 (11..16),
destination ID 16 bytes (16..32), sequence u64 ≠ 0 (32..40), next entry ID u64
(40..48), `next_free` sector u64 (48..56), formatted sector count u64 (56..64).
Then 16 entry records of 80 bytes (64..1344): entry ID u64, first sector u64,
length u64 (bytes, ≥ 80 and ≤ the largest v1 archive), archive trailer FNV-64
u64, sealing sequence u64, label length u8, 7 × 0, label[32] zero-padded. Unused
records and bytes 1344..2040 are zero. Bytes 2040..2048 hold FNV-64 over
0..2040. A copy is valid only if every rule holds: magic, version, checksum,
count, records strictly ascending by entry ID and by first sector, extents
contiguous (the first record starts at sector 16, each further record starts
where the previous extent of ⌈length/512⌉ sectors ends, and `next_free` equals
the end of the last extent, or 16 with zero entries), 16 ≤ `next_free` ≤ formatted
count (also for a catalog with zero entries),
formatted count = granted scope's `sector_count`, every entry ID < next entry
ID, next entry ID ≥ 1 (also for a catalog with zero entries), and all padding
zero.

The destination ID is 16 bytes drawn once at format time from the existing
entropy capability (an `Entropy` object and handle held by the destination
service; section 5.1). It is an identifier, not a secret and not authentication.
FNV-64 detects accidental damage only; a deliberately altered, swapped or
rolled-back image is not detected (section 5.4).

**Mount rule.** Every mount, including a remount (2.7), first issues a FLUSH
and reads sectors 0–15 only after it completes, so that the copies read are
not merely held in a volatile write cache. **A read error is not an invalid
copy:** if any read of sectors 0–15 at mount or remount fails (device `Io`,
`Timeout` or `Unknown`), the service enters the failed-mount state below and
writes nothing; a sector whose read failed also never counts as zero. Treating
a failed read of the newer copy as *invalid* would select the older copy, roll
`next_free` back to the first sector of the newest sealed extent and let the
next CREATE write into it (N8). The cost is a stated limit: a persistent read
error on a catalog sector leaves every entry on the destination unreachable
until the image is repaired outside the service; no read-only fallback is
specified.

With all reads successful: if exactly one copy is valid, use it. If both are
valid and byte-identical, use either. If both are valid and differ, the copy
with the higher sequence must *extend* the other: the same destination ID and
formatted count, the lower copy's records an exact prefix of the higher copy's
records, and `next_free` and next entry ID not lower; then the higher copy is
used. Every legitimate state satisfies this (a completed commit leaves
identical copies; a crash between barriers 2 and 3 leaves the newer copy one
record ahead). Equal sequences with different bytes, or a higher copy that does
not extend the lower, give `Corrupt`. If neither copy is valid and all sectors
0–15 read as zero, the device is a *blank candidate*; zero in sectors 0–15 is
only a heuristic and does not show that the rest of the scope holds no data.
In that state INFO reports `NotFound` -19, FORMAT and UNBIND run, and every
other request that passes checks 1–6 of 2.2 returns `NotFound` -19 (table of
2.2 check 7). Otherwise the destination is `Corrupt`: INFO is answered and
reports `Corrupt` -18 in its status field, every other request that passes
checks 1–6 of 2.2 is answered `Corrupt` -18, and the service never formats
automatically. `Corrupt` is terminal for the service instance: no request
clears it and the service never repairs or formats, and the instance of a
later boot that mounts the same bytes reaches `Corrupt` again. For a disposable test image the
only recovery is external: its owner (for the gates, the harness that created
it) re-zeroes the whole granted scope, not only sectors 0–15, because FORMAT
refuses a scope with data at sector ≥ 16; the next boot's instance then mounts a
blank candidate and FORMAT is permitted (N25). How a product distinguishes a FORMAT
interrupted before its first copy was durable from a damaged existing
destination, and whether any in-product recovery is offered, is open
(section 8, O10).

**Failed-mount state.** The service enters it when a mount or remount FLUSH
fails (any device error, a kernel -8 that persists past the bound of 2.7, or
kernel -14 for an absent or offline device, `volume.rs:967` *[main]*), when a
read of sectors 0–15 fails (mount rule above), or when a FORMAT write or flush
fails (Format below). It writes nothing and issues no further device request. INFO is
answered and reports `NoDevice` -14 in its status field if the broker returned
-14, otherwise `Io` -12; LIST on an export lane whose binding is still
*unknown* (2.7) returns status `Unknown` -16 with zero records; every other
request that passes checks 1–6 of 2.2 returns `Io` -12. This state is terminal
for the service instance: the service does not retry the FLUSH or the read and
no request clears the state, because a failed FLUSH gives no evidence about
earlier writes, and in the broker pattern of 2.7 a device reset that is not
acknowledged quarantines the kernel `Policy` until reboot
(`crates/virtio-core/src/lib.rs:560-569`, `kernel/src/m2/m3/volume.rs:1075-1079`
*[main]*). **Recovery is a reboot.** The destination block grant is
manifest-only and non-transferable, and under the once-per-boot grant rule
(2.1, O13) no other instance in the same boot can obtain it; normal operation
resumes only in the next boot's instance, after
its own mount FLUSH completes. Until then the replies above apply unchanged,
and the portal reports the destination as unavailable and any pending export as
not confirmed. An export binding whose outcome is *unknown* stays *unknown* for
the life of the instance; it never resolves as *sealed* without a completed
FLUSH. In the next boot, bindings and provisional IDs are void; the entry is
judged only from the catalog that that instance reads after its own
completed mount FLUSH, like any other sealed entry (control LIST, VERIFY), and
the earlier export is never reported as confirmed on the basis of its
provisional ID.

**Format.** Only the trusted control path may format, and only a *blank*
scope: before any write, FORMAT reads the whole granted scope and returns
`Access` unless every sector is zero; a read error during that scan returns
the read's device result (`Io` -12, or `Timeout` -13 for a read past its
deadline), writes nothing and leaves the instance a blank candidate. Format writes copy A with sequence 1,
zero entries, next entry ID 1 and `next_free` = 16, flushes, writes the identical copy B, and flushes.
After the last flush completes, FORMAT replies `Ok` and the instance is
mounted with that catalog. If any FORMAT write or flush gives a device result
other than success, FORMAT replies `Io` -12 (or `Unknown` -16 for a write or
flush the broker timed out), the instance enters the failed-mount state above
and does not remount; the next boot's instance mounts whatever reached the
device (blank candidate, `Corrupt`, or one valid copy; N25). The cost of the full-scope
read on large devices is not measured *[unproven]*.

### 2.7 Durability and flush order

The `Disk` contract *[main]* applies: a completed write is not durable until a
later flush. The primary source for the barrier is *Virtual I/O Device
(VIRTIO) Version 1.2*, OASIS Committee Specification 01
(<https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html>):
section 5.2.3 defines feature bit 9, `VIRTIO_BLK_F_FLUSH`; section 5.2.6
defines request type `VIRTIO_BLK_T_FLUSH`; section 5.2.6.2 treats a completed
write as stable once a flush request sent after it has completed. The driver
negotiates bit 9 unless a test profile clears it (`no_flush`,
`drivers/virtio-blk/src/lib.rs:243-244` *[main]*). Three claims are kept
apart: the specification states the requirement; that QEMU and its host
storage stack honour it is assumed *[unproven]*; durability on physical media
is *[unproven]* and not tested here.

1. CREATE reserves `first = next_free` in memory only.
2. APPEND writes whole sectors in `[first, first + ⌈len/512⌉)`; a partial last
   sector is buffered and zero-padded at SEAL. A write at or beyond the scope end
   returns `Full`. There are exactly two write paths: the entry path accepts
   only `sector >= max(16, next_free)` (`next_free` the committed value) and
   below the scope end, so it never relies on catalog validation to keep
   sectors 0–15 out of reach; the
   catalog path accepts only catalog sectors 0–7 (order below). Any other
   write is refused before it reaches the device.
3. SEAL: write the padded last sector; **flush (barrier 1)**; read the extent
   back and run `archive::verify` on it; on failure reply `Corrupt` and discard.
4. Encode the new catalog copy (sequence + 1, the entry appended, `next_free`
   advanced past the extent, next entry ID + 1). **Before any write**, run the
   mount validator of 2.6 on the encoded copy and check that it extends the
   committed copy: the same records plus exactly one, sequence + 1, `next_free`
   not lower. If either check fails, reply `Corrupt` -18, discard the writing
   entry and write nothing; this guards against an encoder defect and does not
   enter the `Corrupt` mount state. Then write the copy into the *target*
   slot: the slot holding an invalid copy (one that read successfully but
   failed validation; a read error never reaches this step, 2.6), otherwise
   the slot with the lower sequence,
   otherwise (both copies valid and byte-identical) copy A; **flush
   (barrier 2)**.
5. Mirror the identical copy into the other slot; **flush (barrier 3)**. Reply
   `Ok` only after barrier 3 completes.

If the write of step 4, barrier 2, the write of step 5 or barrier 3 gives any
device result other than success (`Io`, `Timeout` or `Unknown`; the broker
pattern below reports a timed-out write or flush as `Unknown` -16), the
service replies `Unknown`, discards its in-memory catalog and writing-entry
state and remounts from the device before it receives another request (state
"remount pending" of 2.2 check 7; this interval includes the completion fence
below and is limited by the -8 and poll bounds of "Time source" below). Lane
bindings, including the export binding's provisional entry ID, are kept across
this remount; no CREATE is accepted on any lane before the outcome below is
resolved, nor, under the safe path below, for the rest of the boot, so that
ID cannot be issued again in the meantime.

A catalog read after a failed write or barrier can return bytes still held in
the device's volatile write cache, which a power loss may discard. The
remount therefore begins with a FLUSH (2.6, mount rule), and only a remount
whose FLUSH completed resolves the export binding, once: *sealed* if the
remounted catalog lists the provisional entry ID, otherwise *not sealed*; the
outcome is fixed in the binding and does not depend on later catalog changes.
The lane accepts only INFO and LIST (check 6 of 2.2, from the `Unknown`
reply on); its LIST returns the entry record if the outcome is *sealed* and
zero records otherwise. If the remount's FLUSH or
catalog read fails, the outcome stays *unknown* and the failed-mount state of
2.6 applies (INFO answered with `Io` or `NoDevice`; the export lane's LIST
returns `Unknown` with zero records; every other request that passes checks
1–6 of 2.2 `Io`). Bindings do not survive the reboot that ends that state; the
next boot's instance mounts only after its own mount FLUSH completes, and the
export is then not confirmed to the portal.

**Completion fence.** The remount FLUSH is issued and the export binding is
resolved only after the request that failed is quiescent: completed and
consumed, or ended by an acknowledged device reset. Until that fence the
binding stays *unknown* and the service receives no request (state "remount
pending", 2.2 check 7); after it, CREATE stays refused for the rest of the
boot under the safe path below. On
`main` the M3 volume broker implements this fence for its device *[main]*:
one global `Policy` admits at most one request
(`kernel/src/m2/m3/volume.rs:124-126`); a request past its deadline is answered
`Unknown` -16 if mutating (WRITE and FLUSH are mutating,
`crates/virtio-core/src/lib.rs:56-58`) or `Timeout` -13 otherwise, and the
device is reset (`volume.rs:1087-1099`, `:1006-1016`); while the reset runs the
broker admits no request (`:964`, kernel return -8); the device is reused only
after it reads back status 0 with bus mastering disabled
(`drivers/virtio-blk/src/lib.rs:511-524`, `volume.rs:1043-1074`), otherwise the
lane is quarantined and offline (-13 on reset timeout, `volume.rs:1075-1079`;
-14 if re-initialisation after an acknowledged reset fails, `:1068-1072`)
until reboot.
The per-device destination broker (K4/K5) must provide the same fence for
disk 1; this is a requirement, not existing behaviour.

*Meaning of kernel -8.* On `main` a submission (syscalls 0x40, 0x42, 0x43)
returns -8 when `r.resetting || r.error.is_some() || r.policy.phase() !=
Phase::Ready` (`volume.rs:964`), and these are global broker fields
(`volume.rs:122-131`) *[main]*: while a source-lane request is outstanding or
its result unconsumed, the destination would legitimately see -8 for up to
`TIMEOUT` = 500 ticks (`volume.rs:24`) plus a reset. The completion poll
(0x41) returns -8 while the caller's own request is still pending
(`volume.rs:911-912`) *[main]*. This document therefore defines K5 (4.3) as
**per-device broker state**: the `Policy`, `resetting`, `reset_deadline`,
`error`, `active` and the data buffer are kept per block device. Under K5 the
destination service is disk 1's only client, and it consumes the result of
each request (polls until the poll returns something other than -8) before it
submits the next one. A -8 on a disk-1 submission can then mean only disk 1's
own reset, which ends within `RESET_TIMEOUT` = 10 ticks (`volume.rs:25`)
either ready or offline; offline counts as a failed FLUSH (2.6). A -8 on a
submission is not a device result: the service resubmits. The service bounds
the wait itself *[design]*: a -8 on a submission that persists for longer
than 2 × `RESET_TIMEOUT` (twice the longest legitimate reset under K5, the
factor being a margin for tick granularity) counts as a device `Io` for that
request, and a completion poll that still returns -8 after `TIMEOUT` + 2 ×
`RESET_TIMEOUT` from the submission counts as `Unknown` -16 for a write or
flush and `Timeout` -13 for a read. For a mount or remount FLUSH this is a
failed FLUSH and gives the failed-mount state (2.6); for any other destination
write or flush it starts the safe path below; a read fails with `Io` or
`Timeout`. The bound is not valid against `main`'s shared broker state, where
it would turn ordinary source latency into a destination `Io`, and where a
quarantine leaves the shared phase `Quarantined`
(`crates/virtio-core/src/lib.rs:562-568`) so that the other lane receives -8
indefinitely (`volume.rs:961-966`) *[main]*; the destination service is
therefore never run on shared broker state, and full per-device state is an R5
precondition.

*Time source.* The bound needs a boot-monotonic tick count in the service. On
`main`, generic syscall 3 returns the boot-monotonic PIT tick count (nominally
100 Hz, the unit of `TIMEOUT` and `RESET_TIMEOUT`) only under the feature
`monotonic-clock` (`kernel/src/m2/m3.rs:2366-2370`) *[main]*, which
`native-volume` does not enable (`kernel/Cargo.toml`) *[main]*. Whether
`monotonic-clock` combines with the volume-fixture and
`test-volume-archive-device` features without a `compile_error!` is unchecked;
the R5 test profile must enable it, and this is an R5 precondition. The bound
keeps the single loop from stalling indefinitely: UNBIND waits only for the
device requests already in progress, each limited by these bounds; while a
SEAL catalog failure is being remounted that is at most two submission bounds
(fence, then remount FLUSH) plus the remount's own polls (4.3, N26).
Because the `Policy`
admits one request and barrier 1 has completed, the only request that can be
outstanding at a failure in steps 4–5 is one catalog-sector write or one
flush. The same fence follows a timed-out entry write during APPEND or SEAL
step 3: the writing entry is discarded (2.5) and no request, and so no later
CREATE that reuses its sectors, is admitted before the reset is
acknowledged; under the safe path below, no CREATE is admitted for the rest
of the boot. The reset fences guest DMA and the used ring; whether a write the
device accepted before the reset can still reach the backing medium after the
reset, and so after the remount FLUSH, is not settled by the VIRTIO sections
cited above (5.2.6.2 covers completed writes only) and is *[unproven]* for
QEMU and its host storage stack (section 8, O9). The resolution above holds
only under that assumption; N24 (section 6) states the expected outcome if it
fails. **Safe path while O9 is unproven:** the trigger is any device result
other than success (`Io`, `Timeout` or `Unknown`, or a kernel -8 past the
bound above) on any destination write or flush, catalog or entry: an APPEND
write, the SEAL last-sector write, barrier 1, the step-4 or step-5 write,
barrier 2 or 3 (a FORMAT write or flush failure leads to the stricter
failed-mount state instead, 2.6). A device `Io` is included because a
failed write or flush, like a timed-out one, gives no evidence about which
earlier or in-flight data reached the medium, and a later FLUSH that succeeds
does not show it either. After the trigger the destination accepts no further
catalog or entry write for the rest of the boot (read-only: INFO, LIST,
VERIFY, READ); CREATE, APPEND, SEAL and FORMAT on any lane return `Access` -5
with no device write. These `Access` replies are the state "mounted, safe
path" of the table of 2.2 check 7: they come after checks 1–6, and they apply
only in a mounted instance; while a remount is pending no request is received,
and if the remount ends in the failed-mount or `Corrupt` state, that state's
row applies. None of these states writes to the device. A late write therefore cannot overwrite a newer
catalog copy or a newly allocated extent: no extent is reused in that
boot. In particular, if the remount resolves E2 as *not sealed*, E2's sectors
lie at or beyond the committed `next_free` and would be the next CREATE's
extent, but no CREATE, and so no APPEND, is admitted in that boot, so a late
E2 catalog write cannot make E2 listed over another entry's data. Across a
cold boot the late write cannot occur only under the power-cycle assumption
below; reuse of E2's sectors by an export in the next boot is safe only
under that assumption, and O9 remains a blocking open assumption.

**Carrier of the read-only state** *[proposed; owner decision O13]*. The
read-only state is a latch in the memory of the single service instance. That
latch is sufficient only if no other instance in the same boot can write. The
missing TRANSFER right (2.1, N18) prevents a user process from passing the
grant on, but kernel profile code can create a new grant on a restart, as the
M5 profile does on `main` (2.1). The latch is therefore a sufficient carrier
only under the once-per-boot grant rule of 2.1, which N32 tests. The
alternative for the owner is a kernel write latch in the per-device broker
(an extension of K5): after the trigger, the broker refuses every write to the
destination scope for the rest of the boot, whatever process submits it. Under
the proposed rule the latch ends only with the boot. Every destination runner
must pass `-no-reboot` to QEMU, as `tools/run_volume.py:309` already does
*[main]*, so that each guest reboot ends the QEMU process and every boot in the
gates is a new QEMU process; warm reboot of a product, which a guest cannot
distinguish from a cold boot, stays open under O9.

The in-boot resolution, and any in-boot outcome after the safe-path trigger,
is reported to the portal only as "provisional"; the export counts as
confirmed only after a cold boot, whose mount FLUSH and catalog read follow a
device power cycle that no request of the previous boot survives *[design; a
new QEMU process assumed equivalent, unproven]*, and after VERIFY of the
entry. The portal treats `Unknown` as "not sealed" until the export lane's LIST
returns the entry ID; it must not repeat the export blindly and must not report
success. If the remount yields `Corrupt`, the service fails closed as in 2.6.

**Append-only meaning.** No operation writes a sealed extent, and catalog
copies are written only in the order above. An interrupted new export therefore cannot damage
any previously sealed entry. This resolves the host finding of PR #396 (merged
as `8a7a3594`) that on a
single raw range an interrupted re-export destroys the previous archive as soon
as one differing sector lands.

**Why step 5 mirrors the catalog.** Without step 5, decay of the newer copy
(bytes that read successfully but fail validation; a read error is a failed
mount, 2.6, and never selects the older copy) while the older one stays valid
would make the mount rule fall back to the older copy; the newest sealed entry would then no longer be listed, and its
extent would lie at or beyond the older `next_free`, so a later export could
overwrite it. Silent media decay is outside the crash model; the mirror is a
mitigation subject to the R1 review. After a completed commit both copies are
byte-identical, so decay of one copy loses no entry; decay of both gives
`Corrupt` and fails closed, unless sectors 0–15 all read as zero, which the
mount rule treats as a blank candidate that FORMAT still refuses while entry
data remains (N8). A crash between barrier 2 and barrier 3 leaves
one newer and one older valid copy, which the mount rule handles (the newer
copy extends the older), and the next commit targets the lower-sequence slot
and then mirrors. A read error on the newer copy is a failed mount, not a
fallback to the older one. Decay of the newer copy, however, is not covered
in that window: until the next commit, which under the safe path may not come
in that boot, decay of the newer copy alone would select the older copy and
leave E2 unlisted and overwritable. This window is an **accepted residual** of
the decay mitigation; a mount-time mirror is not specified, because it would
add a third catalog write path at mount, which the safe path and the
failed-mount state forbid in exactly the boots where the window is longest. The catalog write path accepts only the eight
catalog sectors, in the order target slot, barrier, other slot, barrier.

### 2.8 Bounds

16 sealed entries per destination, one writing entry, 160 bytes per APPEND or
READ, at most 4 lanes (one control and three operation lanes; section 4.3).
The largest v1 archive is 525 328 bytes, so APPEND needs at most 3 284
messages; this throughput is acceptable for the bounded v1 volume and is not
measured *[unproven]*.

## 3. Trusted backup portal and destination selection (D2 (a), D4 (a))

### 3.1 Components

- **Backup portal** *[design]*: a system component shipped and signed with the
  OS image. It is the only component whose requests the service accepts on
  an operation lane. The trusted dialog holds usable issuing roots (SEND,
  TRANSFER and REVOKE on each operation lane, section 4.3); the service refuses
  the dialog's own requests on those lanes by the kernel-stamped sender check
  of section 4.3, not by a behavioural promise. It runs `archive::export`, `archive::verify` and `archive::restore` against
  the volume and the destination service.
- **Trusted dialog** *[design]*: a system surface rendered by the shell on a
  trusted surface, not drawable by applications. It holds the control lane of
  the destination service and the lane root handles (section 4.3).
- **Third-party applications** hold no lane, no control handle and no block
  capability on any destination. A later export of a copy to an app goes through
  the export portal as a different permission, not through this service.

### 3.2 What the user sees and confirms (functional)

For export: the source (the user's own document domain, with document and
collection counts), the destination (device label, capacity, free space, the
number of existing backups, and whether it is already a Baryon OS destination),
the statement that the backup is not encrypted and is suitable only for the
user's own local device, and an action button "Back up". An option "Remember
this destination for future backups" is shown unchecked. Errors are reported in
plain words (destination full, destination not found, destination damaged).

For verify: the selected backup (label, entry ID, size) and the result.

For restore: the list of backups on the destination (label, entry ID, object
count, size), the target domain, and the statement that existing documents are
never overwritten and that conflicts are reported by name. Times are shown only
if a trusted source exists; the catalog stores no wall-clock time and the
dialog never fabricates one (section 8, O5).

For format: only a blank device is offered, with its label and capacity and the
statement that it becomes a dedicated backup destination. Non-blank devices are
not offered (section 8, O2).

### 3.3 Grant representation

The existing permission models on `main` are per-session host models:
`sdk/task-grant-core` binds read, write and export rights to an app, a process
incarnation and an object revision, with durations `UntilClose` and `Session`
only; `services/net-broker` binds grants with an instance, slot and generation
*[main]*. Neither has a persisted grant. This design therefore uses two levels:

1. **Remembered permission** *[design; needs O1]*: a persisted record
   `{principal = backup portal, destination ID, operations ⊆ {export, verify},
   revocation generation}` held by the trusted dialog's store. It authorizes the
   dialog to issue a new per-operation grant for normal repetition without
   asking again. Revoking it increments the revocation generation and revokes
   every live lane bound to that destination, together with the portal's
   volume-side binding (4.1, 4.2) for each operation on those lanes. The record
   keys on the destination ID, which is not authenticated (5.4): an attacker
   with physical access who presents a clone of the catalog would receive
   automatic plaintext exports under a remembered export permission. This is an
   input to O1, not a design change. Restore is not remembered by
   default; whether it may be is O1. Persisting this record needs a new
   `Duration` and storage in `task-grant-core` or a new store, which is an SDK
   change, not a kernel change. It extends, rather than follows, the existing
   permission model named in D2 (a) (section 8, O1).
2. **Per-operation grant**: a kernel capability node with SEND only on one
   operation lane, plus the service's lane binding `{operation, entry ID, grant
   generation, portal process ID}` (section 4.3). It lives until the operation
   ends or is revoked; ending or revoking it also ends the portal's volume-side
   binding for that operation (4.1).

**Owner principal** *[proposed; owner decision O12]*. Neither the catalog record
(2.6) nor the archive header ([S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md),
"Bytes") names an owner, yet [PROFILES-AND-USERS.md](PROFILES-AND-USERS.md)
requires that profile A cannot open profile B's catalogue. With the dialog's
LIST of all entries and a restore into "the target domain" (3.2), user B could
restore user A's plaintext backup. Proposed for v1: the destination service is
single-profile, and the dialog refuses every destination operation while more
than one profile exists, until an owner principal is bound per destination or
per entry. The alternative for the owner is to bind that principal now, which
needs a catalog field and a format decision. R7 and the R8 activation are
blocked until O12 is decided; the test-only stages of R5 and R6 run in a
single-profile fixture and do not depend on it.

## 4. Operation-specific capabilities (D3)

### 4.1 Rights per operation

| Operation | Destination service (service-enforced lane binding) | Kernel node held by the portal | Source volume |
|---|---|---|---|
| Export | INFO, CREATE once, APPEND, SEAL, ABORT on one new entry; LIST restricted to its own entry (zero records until sealed, 2.3; after an `Unknown` reply, the resolution of 2.7) | SEND on one lane; no RECV, DERIVE, TRANSFER, REVOKE | domain-bound volume endpoint, read-only binding *[needs volume change, 4.2]* |
| Verify | INFO, LIST (bound entry), VERIFY on one sealed entry | SEND on one lane | none |
| Restore | INFO, LIST (bound entry), READ on one sealed entry | SEND on one lane | domain-bound volume endpoint, create-only binding *[needs volume change, 4.2]* |
| Selection (dialog) | INFO, LIST (all), ABORT, BIND, UNBIND, format of a blank scope | control lane SEND | none |

The portal never holds a block capability on the destination device or on the
source volume. No portal handle carries DERIVE or TRANSFER, so it cannot pass a
lane on *[main mechanism: `capability-core` `subset`/`require`]*; on `main` the
portal's existing volume handles are SEND on the domain ingress and RECV on its
reply endpoint, and its archive block handle carries no TRANSFER
(`volume.rs:594-603`, `:713-724`) *[main]*. The read-only and create-only
volume bindings exist only for the duration of one operation: when its lane is
unbound or revoked, or the remembered permission behind it is revoked (3.3),
the volume-side binding for that operation ends with it (mechanism specified
with R4).

### 4.2 Source volume side

Export reads the domain through the normal volume client. Today the whole
volume service is made read-only during the fixture export
(`service_read_only()` in `kernel/src/m2/m3/volume.rs`) *[main, fixture only]*.
A product needs a per-endpoint **read-only binding** in the volume service that
refuses every mutating operation with `Access` before state is consulted.

Restore writes through the normal volume client. The non-overwrite rule is
enforced today by `archive::restore` in the caller, which is library code linked
into the portal; the volume service itself accepts any valid BEGIN/COMMIT, META,
DELETE, RENAME or PURGE from the domain's endpoint *[main]*. Least privilege
therefore needs a **create-only binding** in the volume service: commits only
into an empty or tombstoned slot, META and COLLECTION only on objects and
collections created through that binding, and DELETE, RENAME, PURGE, RESTORE
and TRANSACT refused with `Access`. Both bindings are volume-service changes
behind the existing kernel-bound endpoint; they change no kernel ABI, but they
change the volume wire's authorization behaviour and need their own
specification and host tests (route step R4). Until then, restore's
non-overwrite guarantee rests on trusted portal code, and this document does not
claim service enforcement.

### 4.3 Mapping onto capability-core and the kernel

Facts on `main`: user syscalls exist for close (0x10), derive (0x11), revoke
(0x12) and send/recv with up to two transferred capabilities (0x20/0x21) in
`kernel/src/m2/m3/m4.rs`; endpoint and block-object creation exist only in
kernel manifest code; a process owns at most `MAX_OWNED_ENDPOINTS` = 4
endpoints; `revoke` marks every strict descendant of the caller's node revoked
and keeps the caller's own node; a revoked node resolves to `Error::Revoked`;
revoking a node also removes queued messages that carry capabilities derived
from it, but nothing shows that it removes plain data messages already queued
by the revoked holder *[main, `capability-core` tests]*.

Design with existing primitives only:
- The manifest creates four endpoints owned by the destination service: one
  control lane and three operation lanes. The service keeps RECV on each. The
  dialog receives SEND on the control lane and SEND|TRANSFER|REVOKE on each
  operation lane. Because `send` requires the parent node to hold TRANSFER and
  the transferred rights (`subset`), these roots are usable: the dialog can
  itself send on an operation lane *[main, `Engine::send`]*. The service
  therefore binds the caller by lane, as the volume service binds a domain by
  its ingress endpoint *[main pattern]*, **and** by sender: the kernel stamps
  `sender_pid` into every queued message (message offset 0) *[main]*, and BIND
  records in the lane binding the portal's process ID, fixed for the boot by
  the manifest that starts the portal *[design]*. The same manifest writes that
  process ID into the destination service's start context before the service
  runs, so the service knows it before any BIND, including for reply
  suppression on unbound lanes; N18 checks the value against the portal's
  actual process ID *[design]*. A request on a bound
  operation lane whose `sender_pid` differs is refused with `Access` with no
  state change and no device I/O, and gets no reply (check 4 and Replies of
  2.2; N22). The service refuses any
  well-formed request on an unbound lane with `Access` (check 3). That process
  IDs are not reused within a boot is assumed
  *[unproven]*. Placing the issuing roots in the dialog
  rather than in the portal is this document's interpretation of D4 (a)
  ("a trusted OS backup portal holds destination capabilities"): the portal
  holds every capability that performs an operation, the dialog only the
  authority to issue and revoke them. The project owner may require the roots
  in the portal instead (section 8, O7).
- Replies: as for the volume, the portal owns its own reply endpoint, keeps
  RECV on it and the manifest delegates SEND on it to the destination service;
  operation-lane replies go there. The dialog owns a control-reply endpoint,
  keeps RECV on it and the manifest delegates SEND on it to the destination
  service; control-lane replies go there, so the dialog receives the BIND and
  UNBIND replies it waits for. The dialog owns no other endpoint, so this uses
  one of its `MAX_OWNED_ENDPOINTS` = 4 *[main limit]*. The dialog reaches the
  portal through one further portal-owned endpoint, on which it transfers lane
  handles. All these handles are included in the manifest counting of section
  5.1. With an IPC transfer the new node is a child of the
  sender's node *[main, `Engine::send`]*, which is what makes the dialog's
  `revoke` reach the portal's lane handle.
- To grant: the dialog sends BIND (lane, operation, entry, new grant generation,
  above the maximum that INFO reports for that lane, 2.3) on the control lane,
  waits for `Ok` on its control-reply endpoint, then transfers SEND on that lane to the
  portal, and tells the portal the grant generation, which the portal places in
  every request's `epoch` field.
- To revoke: the dialog first sends UNBIND on the control lane and waits for
  `Ok` on its control-reply endpoint, so that any request already queued on the
  lane (including a SEAL) is answered with `Access` and cannot commit; if the
  reply does not arrive (for example because it was dropped, 2.2), the dialog
  repeats UNBIND, which is idempotent (2.3), and does not revoke before a
  reply. A reply of `Io` (failed-mount state) or `Corrupt` (2.6) also permits
  the revoke, because in those states the service writes nothing and no queued
  request can commit. A kernel `Revoked` -10 or `Closed` -9 returned by the UNBIND
  `send` itself also permits it. When the service exits, `Engine::exit`
  revokes every node that descends from the service's nodes, including the
  dialog's delegated control and lane nodes, so `resolve` returns `Revoked`
  before it reaches the endpoint; `Closed` covers an endpoint that `maintain`
  has already reclaimed. In both cases no queued request can commit and no
  reply will come *[main, `capability-core` `Engine::exit` lib.rs:1588-1628,
  `resolve` lib.rs:1690-1701]*. N6 includes this case. It then calls `revoke`
  on its lane node, which revokes the portal's child node and keeps the
  dialog's node for reuse. The service serves the control lane and the
  operation lanes from one loop, one request at a time, so UNBIND is ordered
  against every request it has not yet served. A SEAL already served before
  UNBIND has completed and stays sealed; revocation never undoes a completed
  operation.

Needs explicit approval, because each is a kernel or shared-ABI change, and is
not assumed here:
- **K1** Runtime device ownership (hot-plugged or user-attached destination).
  Block objects are manifest-only, so v1 supports only a destination present at
  boot.
- **K2** More than three concurrent operation lanes, which would need endpoint
  badges or a higher owned-endpoint quota.
- **K3** A kernel-enforced rule that the portal can never hold the volume's or
  the destination's block object. Today this is shown by manifest counting
  (`ARCHIVE_ESCAPE`) only.
- **K4** A destination block device in a production profile at all: today
  `device_call` refuses `scope.device != 0` with `Access` *[main]*, and device 1
  exists only under the test feature `test-volume-archive-device`
  (`volume.rs:877-881`) *[main]*. K4 covers production profiles only; the
  test-only R5 profile additionally needs the owner to amend
  ADR-S09-DUALDISK-20261119 B3 and B5 (O4 (b)).
- **K5** Per-device broker state: the virtio `Policy`, `resetting`,
  `reset_deadline`, `error`, `active` and the data buffer kept per block
  device (today global, `volume.rs:122-131` *[main]*), so that a destination
  fault cannot fail the source volume closed (section 5.2) and a kernel -8 on
  disk 1 means only disk 1's own reset (2.7). A per-device `Policy` alone does
  not give this, because `resetting`, `error` and `active` gate every
  submission (`volume.rs:964`). If the owner chooses it under O13, K5 also
  carries a kernel write latch for the destination scope.

## 5. Isolation between source and backup devices

### 5.1 Ownership rule

The volume service holds a block capability only on the source volume scope; the
destination service holds one only on the destination scope; the portal holds
neither. The volume service never holds a destination lane or block handle, and
the destination service never holds a volume endpoint or block handle. At
manifest time this is checked by counting, as `archive_grant` does today
*[main]*: one new block object, node and handle per grant, in the intended
process, with every other process's handle count unchanged and distinct object
keys. The same counting covers the endpoint handles of section 4.3. The
destination service holds exactly: one block handle on the destination scope
(READ|WRITE|FLUSH, 2.1), four lane RECV handles, one SEND handle on the
portal's reply endpoint, one SEND handle on the dialog's control-reply endpoint
and one handle on its own `Entropy` object (one of `MAX_ENTROPY_OBJECTS` = 4)
for FORMAT *[main limit]*; in the R5 test profile with the deterministic test
destination ID (section 7, O4 (c)) it holds no `Entropy` handle, and N18
asserts that. The dialog holds exactly: one control-lane SEND,
three lane roots (SEND|TRANSFER|REVOKE), one SEND on the portal's intake
endpoint and RECV on its own control-reply endpoint. The portal holds its reply
and intake RECV handles, its volume handles (4.1) and, at most, three lane SEND
handles at run time. Counting alone does not detect excess rights, so N18 also
probes the exact rights of each handle.

### 5.2 DMA and lanes

The kernel virtio-blk driver performs DMA; userspace never does. PR #396
(merged as `8a7a3594`) gives the second device its own DMA pages *[main]*, disjoint from device 0, and an
unmapped lane-1 MMIO window after review. It also records a fixture limit: both
lanes share one virtio `Policy`, so a failed device-1 reset fails both lanes
closed (the faulting lane goes offline, the other receives kernel -8
indefinitely, 2.7). That is safe but couples availability: a faulty backup device stops the
source volume. The product requires per-device broker state (K5: the
`Policy` and the `resetting`, `reset_deadline`, `error`, `active` and data
fields), so that a destination fault yields `Io` or `NoDevice` on the
destination only. This is a kernel change (K5) inside the virtio-blk path, not
an ABI change.

### 5.3 Device identity

A destination is identified by its catalog destination ID. A grant names the
destination ID; if the mounted catalog holds another ID, BIND returns
`NoDevice` -14 and performs no I/O. If no device is present, the mount FLUSH
receives kernel -14 and the service is in the failed-mount state (2.6): INFO
reports `NoDevice` -14, every other request that passes checks 1–6 of 2.2
returns `Io` -12. A kernel -14 or offline -13 on any destination write or
flush is a device failure: the request gets `NoDevice` -14 or `Io` -12
respectively under the 2.5 edges, and the 2.7 safe-path trigger applies. A non-blank layout that is not a valid catalog is `Corrupt`
(2.6), and every request except INFO returns `Corrupt` -18. A blank candidate
has no destination ID; BIND there returns `NotFound` -19 at check 7 (2.2).
`NoDevice` is thus a reply status only of BIND in the mounted state and a
status reported by INFO. The current
fixture's "wrong device" check (portal granted the device-0 range, verify gives
`Corrupt`) *[main]* becomes a `NoDevice` refusal at BIND, before any lane is
bound.

### 5.4 Limits

No IOMMU model: QEMU's virtio-blk device is trusted to follow the protocol and
DMA only into granted pages. A malicious device is out of scope. The destination
ID and FNV-64 are not authentication: an attacker with physical access can swap
or roll back the image undetected. v1 archives are plaintext personal data and
belong only on the user's own local device (D5 (a)).

## 6. Negative test matrix

Layers: **K** kernel capability or block broker, **S** destination service,
**V** volume service, **P** portal code, **D** trusted dialog, **H** host
oracle. Levels: **host** = host Rust test with a simulated disk that models a
volatile write cache (as the `SimDisk` of PR #396, *[main]*); **QEMU** = native
stage on disposable images. A native "cut" kills the QEMU process on the host;
with `cache=writeback` (`tools/run_volume.py:314`, `:321`) *[main]* data that
QEMU has handed to the host page cache survives the kill, so a QEMU cut is a
**process kill without cache loss** and cannot detect a missing barrier.
Flush-order and cache-loss acceptance is therefore host-only (`SimDisk`, R2);
the native cut stages check recovery and the recorded flush order (section 7).
Rows added or changed in revisions 1 and 2 name, after "Kills:", the mutation
that must make the test fail and how it is observed. In host tests the status
of a request is the value returned by the service's request handler, also
where 2.2 sends no reply.

| # | Case | Precondition | Action | Expected (layer: code) | Enforced by | Level |
|---|---|---|---|---|---|---|
| N1 | Unauthorized device | app or portal without a destination block handle | block read/write with own or forged handle | K: `BadHandle` -4 or `Access` -5; counted in `app_device_denials` | K | QEMU |
| N2 | Scope escape, device | destination service | block write at scope end | K: `Access` -5; host sees no byte past the scope | K | QEMU |
| N3 | Scope escape, entry | restore lane bound to entry E | READ entry ≠ E; READ past E's length; READ straddling the end; READ with `offset + aux` overflowing u64; APPEND on a verify lane | S: `Access` -5; `Range` -3; `Range` -3 with no data; `Range` -3 with no data; `Access` -5. Kills: a READ bound check that compares only `offset` with the length, or uses wrapping addition | S | host + QEMU |
| N4 | Scope escape, sealed extent | export writing after sealed entries; separately, a crafted catalog with zero entries and `next_free` < 16; a crafted catalog with zero entries and next entry ID 0 | crafted write path below `next_free` and below sector 16 (mutation test); mount of each crafted catalog | write refused; sealed extents and sectors 0–15 byte-identical; each crafted copy is invalid | S | host |
| N5 | Revoked before operation | dialog revoked the lane | portal `send` | K: `Revoked` -10 (syscall return) | K | host + QEMU |
| N6 | Revoked during export | 50 % appended; (b) the service exits instead | dialog sends UNBIND, then revokes; (b) the dialog's UNBIND `send` returns `Revoked` -10 (4.3) | K: `Revoked` -10 on next send; S discards writing entry; LIST unchanged; earlier entries verify; (b) the dialog takes its revoke step and the revoke succeeds or is a no-op (host `Engine` case). Kills (b): a dialog that permits the revoke only on `Closed` -9 and repeats UNBIND forever | K + S | host + QEMU |
| N7 | Corrupted entry | one byte flipped in a sealed extent on the host image | VERIFY; restore flow | S: `Corrupt` -18; P: `archive::verify` `Corrupt`, nothing restored | S + P | host + QEMU |
| N8 | Corrupted catalog; catalog read error | one copy damaged; then both damaged; then both copies zeroed with sectors 8–15 zero and sealed entries at sector ≥ 16; separately, A = sequence n+1 {E1, E2} and B = sequence n {E1} (crash between barriers 2 and 3) with the simulated disk failing the read of one sector of A (`Io`, then `Timeout`, then `Unknown`); and the same with a read error on one of sectors 8–15 while 0–7 read as zero | mount; control FORMAT; for the read-error cases also INFO, control LIST and control BIND | one copy: the other, identical copy used, LIST unchanged; both damaged (a non-zero sector in 0–15): S `Corrupt`, INFO answered with `Corrupt`, every other request `Corrupt` -18, no write, no format; both zeroed: blank candidate per the mount rule (INFO status `NotFound` -19, destination ID zero), control LIST and BIND `NotFound` -19, no write, FORMAT `Access` -5 after the full-scope read finds the entry data (as N23); read error on A: failed-mount state (2.6), INFO reports `Io`, LIST, BIND and FORMAT `Io` -12, no device write, B never selected, E2's sectors never written; read error in 8–15: failed-mount state, never a blank candidate. Kills: a mount that treats a copy with a read error as invalid (selects B), and one that treats an unreadable sector as zero; (revision 2) a blank state that answers INFO with `Ok` or BIND with anything but `NotFound` -19 | S | host (read-error cases) + QEMU |
| N9 | Incomplete write | appended ≠ declared | SEAL | S: `Invalid` -1; nothing listed | S | host |
| N10 | Non-archive bytes | appended bytes fail `verify` | SEAL | S: `Corrupt` -18; nothing listed; `next_free` unchanged after reboot | S | host + QEMU |
| N11 | Power cut during APPEND | one sealed entry E1; export of E2 running | host: cut after k destination writes with cache loss; QEMU: host process kill after k destination writes | after cold boot: LIST = {E1}; E1 verifies byte-identical | S (2.7) + H | host (cache loss) + QEMU (process kill, no cache loss) |
| N12 | Power cut during SEAL | cut after barrier 1, during the catalog write, after barrier 2 | host: every write prefix with cache loss; QEMU: host process kill at each point | LIST = {E1} or {E1, E2}; a listed E2 verifies; never a listed E2 that fails; QEMU additionally: the `DEST:IO:FLUSHES` count and flush order recorded at the cut marker match 2.7; both are counted by the kernel broker per destination object key, and the cut marker is emitted by the kernel after k destination writes, independent of the flush count (section 7). Kills (revision 1, QEMU): a SEAL without barrier 1 or barrier 2 (detected by the flush count and order, not by recovery); (revision 2) a service that skips the flush call, which a service-side counter could hide but the kernel count cannot | S + H | host (cache loss) + QEMU (process kill, no cache loss) |
| N13 | `Unknown` at SEAL | the step-4 or step-5 write, or the flush at barrier 2 or 3, gives a device `Io`, `Timeout` or `Unknown` (broker timeout, 2.7), with and without the new copy durable; bound lanes: export lane X (the SEAL), a second export lane Y bound before the SEAL with no CREATE yet, a verify lane and a restore lane; then, after the completion fence (N24), the remount FLUSH succeeds, or fails | SEAL on X; then on X: LIST, CREATE, APPEND, SEAL, ABORT; on Y: CREATE; on the verify and restore lanes: CREATE, and LIST, VERIFY or READ; on the control lane: INFO, LIST, BIND | S: SEAL `Unknown` -16 for every device result, remount, bindings kept. Remount FLUSH succeeds: X's LIST returns E2's record if the remounted catalog lists E2 (P reports provisionally sealed, confirmed only after cold boot and VERIFY, 2.7), else zero records (P reports not sealed); X's CREATE, APPEND, SEAL and ABORT `Access` -5 (check 6: binding limited to INFO and LIST); Y's CREATE `Access` -5 (check 7, safe path) for the rest of the boot; the verify and restore lanes' CREATE `Access` -5 (check 6), their LIST, VERIFY and READ run; control INFO, LIST and BIND run. Remount FLUSH fails (failed-mount state): X's LIST `Unknown` -16 with zero records; X's CREATE, APPEND, SEAL and ABORT `Access` -5 (check 6); Y's CREATE `Io` -12; the verify and restore lanes' CREATE `Access` -5 (check 6), their LIST, VERIFY and READ `Io` -12; control INFO status `Io` -12, control LIST and BIND `Io` -12; no device write, no FLUSH retry, state unchanged for the rest of the boot, binding never resolved as sealed (P reports not confirmed); in the next boot the provisional ID is void and only that boot's catalog decides. No repeated export in either case. Kills (revision 1): a safe-path trigger that fires only on `Unknown`, so that a device `Io` at barrier 2 or 3 readmits Y's CREATE after a successful remount (Y's CREATE runs). Kills (revision 2): a validator that applies check 7 before check 6 (X's CREATE and the verify lane's CREATE return `Io` -12 in the failed-mount state); a binding limit applied only after resolution (X's APPEND reaches check 7) | S + P | host |
| N14 | Wrong, foreign or absent device | grant names ID X; device holds valid catalog Y (natively an image whose catalog the harness writes with an ID other than the deterministic test ID, section 7); separately, a non-Baryon, non-blank layout; separately, no device (broker -14 to the mount FLUSH) | BIND naming X; INFO; a well-formed request on an unbound lane; control FORMAT | catalog Y: BIND `NoDevice` -14, no I/O, lane stays unbound, its requests `Access` -5; foreign layout: `Corrupt` mount, INFO reports `Corrupt`, BIND and FORMAT `Corrupt` -18, format not offered by D, no write; no device: failed-mount state, INFO reports `NoDevice` -14, BIND and FORMAT `Io` -12. Kills (revision 1): a check order that returns `NoDevice` from a request other than BIND, or answers FORMAT in the `Corrupt` state with anything but `Corrupt` | S + D | host + QEMU |
| N15 | Destination full | free space < archive | export | S: `Full` -17 at APPEND; nothing sealed; earlier entries intact; catalog sequence unchanged | S | host + QEMU |
| N16 | Third-party raw access | app process in the profile | forged lane, control and block handles; IPC to service without a handle | K: `BadHandle` -4; counted; service receives nothing | K | QEMU |
| N17 | Stale grant replay and header rules | lane rebound with generation g+1; lane unbound; control lane | request with epoch g; duplicated APPEND offset; second SEAL; any well-formed request on an unbound lane; control request with epoch ≠ 0 or non-increasing `request_id`; CREATE with `length` ≠ `name_length`; re-BIND of a lane with a generation equal to (and separately lower than) its highest accepted one while an APPEND of the old binding is still queued; a request rejected by checks 1–6 followed by one with the next `request_id`; in the safe path a CREATE followed by a request with the same `request_id` | S: `Stale` -10 (reply status); no extra bytes; second SEAL `Access` -5 (check 6), no second entry; unbound lane `Access` -5; control `Stale` -10; CREATE `Invalid` -1; re-BIND `Stale` -10 with no state change, lane stays unbound, the queued APPEND gets `Access` -5 and writes nothing, INFO reports the unchanged maximum; the request after one rejected by checks 1–6 is accepted (such a rejection does not advance the high-water mark); in the safe path, a CREATE refused by check 7 (`Access` -5) advances it, so a request that reuses its `request_id` returns `Stale` -10. Kills (revision 1): a BIND that accepts a generation equal to the lane maximum, and a validator that advances the high-water mark on a request rejected by checks 1–6. Kills (revision 2): a validator that does not advance the mark on a check-7 refusal (the reused `request_id` is accepted) | S | host + QEMU |
| N18 | Authority escape at manifest | profile start | count objects, nodes and handles; probe object keys; probe exact rights with `check_block` and `check_endpoint`, as `ARCHIVE_READONLY_RIGHTS` does (`volume.rs:743-748`); compare the destination scope with every other block scope by `check_block` (precedent: `volume.rs:749-753`, which compares the volume service's own handle, 2.1) | the destination service, dialog and portal hold exactly the 5.1 handles (in the R5 profile with the deterministic test destination ID, no `Entropy` handle); the destination block handle passes READ, WRITE and FLUSH and refuses DERIVE, TRANSFER and REVOKE; the dialog's control-lane handle is SEND only (the portal holds no lane handle at profile start; R5 checks at run time that each lane handle transferred to it is SEND only); the destination scope is disjoint from every other block scope, and under the proposed O11 rule on another device; any extra handle or right in volume service, destination service, dialog, portal or app: manifest fails (`ARCHIVE_ESCAPE`-style). Kills (revision 1): a manifest that grants `BLOCK_RIGHTS` (with DERIVE, TRANSFER, REVOKE) at an unchanged count; a destination scope overlapping the device-0 source scope; under O11, a disjoint scope on device 0; (revision 2) an `Entropy` handle granted to the destination service in the deterministic-ID R5 profile (count grows) | K manifest | QEMU |
| N19 | Volume read-only binding | export lane plus RO volume endpoint | portal sends BEGIN/META/DELETE to the volume | V: `Access` -5; volume unchanged | V | host (after R4) |
| N20 | Volume create-only binding | restore into a domain with occupied slots | portal sends a commit into an occupied slot, DELETE, RENAME | V: `Access` -5; existing objects keep their revision | V | host (after R4) |
| N21 | Second writer | export lane writing | CREATE on another lane | S: `Busy` -21 | S | host |
| N22 | Dialog uses a bound lane | lane bound to the portal | dialog sends INFO, APPEND or READ on that lane with its own root | S: refused at check 4 (handler status `Access` -5, `sender_pid` ≠ bound portal); no reply reaches the portal's reply endpoint (2.2 Replies; natively the kernel-reported `DEST:PORTAL:REPLYQ=` is unchanged); `DEST:DENIALS=` grows by one (service report, supporting only); no I/O. Kills: a service that skips the sender check (the request runs or is answered `Ok`); (revision 2) a service that replies to the portal for a non-portal sender (the portal's reply queue grows, observed by the portal or the host `Engine`) | S | host + QEMU |
| N23 | Data beyond the catalog area | catalog sectors 0–15 zero, non-zero data at sector ≥ 16 | control FORMAT | S: `Access` -5; no sector written | S | host |
| N24 | Late-completing write (2.7 completion fence) | simulated disk that completes a timed-out catalog-sector write late: (a) before the fence (reset acknowledgement), (b) after the remount FLUSH, (c) after a remount that resolved E2 as *not sealed*; (d) an entry-sector write during APPEND that times out (`Unknown`) and completes late, and separately one that fails with a device `Io` while its data still reaches the medium | SEAL with the step-4 write timing out (`Unknown`); CREATE on another lane and LIST on the export lane sent before the fence; (c) CREATE and APPEND on another lane after the resolution, before the late write lands; (d) CREATE and APPEND on another lane after the acknowledged reset; then remount and cold boot | (a) before the fence no remount FLUSH, no resolution, and neither request is received (remount pending, 2.2 check 7); after the remount CREATE `Access` -5 (safe path) and the export-lane LIST follows the resolution, which matches the cold-boot catalog; (b) outside the assumed device model (O9), recorded, not a pass: no sealed extent written, never a listed entry that fails VERIFY, LIST after cold boot {E1} or {E1, E2} (E2 listed although P reported not sealed: safe direction) or S `Corrupt` fail closed with E1 byte-identical; mutation that issues the remount FLUSH or receives a request before the fence must fail (a) (observed in the simulated disk's event order: remount FLUSH or a reply before the late completion); (c) CREATE `Access` -5 and APPEND `Access` -5 in that boot, no device write, E2's sectors not allocated again, so the late E2 write lands on no other entry's data; mutation that admits CREATE after the resolution in the same boot must fail (c); (d) writing entry discarded (APPEND replies `Unknown` -16, respectively `Io` -12), CREATE and APPEND `Access` -5 for the rest of the boot, the late entry write lands only in unallocated sectors; mutation that admits CREATE after the reset in the same boot must fail (d); Kills (revision 1): a safe-path trigger that ignores a device `Io` on an entry write | S + K fence | host |
| N25 | Power cut during FORMAT | blank scope | FORMAT with a cut at every write prefix of copy A, with cache loss (host), before copy B | no non-zero sector of copy A durable (a zero-entry copy A is non-zero only in sectors 0 and 3, so a durable subset of sectors 1–2 alone counts here): blank candidate, FORMAT `Ok` again; some but not all non-zero sectors of copy A durable, or copy A otherwise invalid: S `Corrupt`, INFO reports `Corrupt`, every other request `Corrupt` -18, no write, no format; after the owner re-zeroes the whole scope (2.6), the next boot's instance mounts a blank candidate and FORMAT returns `Ok`; copy A valid (sectors 0 and 3 durable): mounted with one valid copy (sequence 1, zero entries); (revision 2) separately, the simulated disk fails a FORMAT write or flush with `Io` and, in a second run, `Unknown`: FORMAT replies that code, the instance enters the failed-mount state (INFO reports `Io`, a repeated FORMAT `Io` -12), and no further device request follows. Kills (revision 2): a FORMAT failure that leaves the instance a blank candidate, so that a repeated FORMAT writes again (observed by the simulated disk's write counter) | S | host |
| N26 | Kernel -8 (2.7) | broker stub that returns kernel -8 on submission forever: (a) to the mount FLUSH; (b) to the barrier-2 flush and every later request; (c) to an APPEND write; separately (d) -8 on submission for `RESET_TIMEOUT` ticks (disk 1's own reset under K5) and then success, and a completion poll that returns -8 (pending) for `TIMEOUT` − 1 ticks and then success; (e) a completion poll that returns -8 for longer than `TIMEOUT` + 2 × `RESET_TIMEOUT` on a write | mount; SEAL; APPEND; UNBIND on the control lane during the wait | (a) after one bound (2 × `RESET_TIMEOUT`): failed-mount state, INFO reports `Io`; (b) after one bound SEAL `Unknown` -16, then the remount FLUSH also exceeds the bound, failed-mount state; (c) after one bound APPEND `Io` -12, writing entry discarded, safe path (CREATE `Access` -5 for the boot); (d) both requests complete normally, no `Io`, no safe path; (e) the write counts as `Unknown` -16 and starts the safe path. UNBIND is answered after at most one bound in (a) and (c) and at most two bounds in (b) (the stub admits no request, so no poll occurs; with real polls the limit of 2.7 "Time source" applies), `Io` -12 in (a) and (b) (failed-mount state) and `Ok` in (c), and no device write follows. Kills: a resubmission loop without the bound (the test times out and UNBIND is never served); (revision 2) a bound shorter than one legitimate reset, or the submission bound applied to the pending poll ((d) reports `Io`); a poll loop without a bound ((e) never ends) | S | host |
| N27 | Transferred capability on a request (2.2 check 1) | dialog double sends one request with one transferred handle on the control lane, and one on a bound operation lane | any request with `cap_count` 1 | S: handler status `Invalid` -1 (control lane: replied to the dialog; operation lane: no reply, because the sender is not the portal, 2.2 Replies); the service's handle count after the request equals the count before; no state change; `request_id` high-water mark unchanged. Kills: a service that keeps the received handle (count grows) or parses the request before closing it | S + K | host (capability-core `Engine`) + QEMU (`DEST:HANDLES=`) |
| N28 | Check order (2.2) | requests that violate two checks of 2.2 at once, for every pair of checks 1–7 (table-driven) | send each request | S: the handler status of the earlier check; no state change; no device I/O; high-water mark unchanged. Kills (narrowed in revision 2): a swap of two checks whose outcomes differ in status, reply, handle count or high-water mark: check 1 with any later check (handle count, and the status for checks 3–7), check 2 with checks 3–7 (`Invalid` against `Access`, `Stale` or a state code), check 5 with checks 3, 4, 6 and 7 (`Stale` against `Access` or a state code), and check 6 with check 7 in the failed-mount, `Corrupt` and blank states (as in N13). Equivalent mutants, which no test can detect and which are listed rather than tested: swaps among checks 3, 4 and 6 and the safe-path row of check 7, which all return `Access` -5 with no state change (checks 3 and 4 cannot fail together, because an unbound lane has no bound sender) | S | host |
| N29 | Pre-write catalog check (2.7 step 4) | test hook that alters the encoded copy: wrong checksum; `next_free` lower; sequence + 2; one record dropped | SEAL | S: `Corrupt` -18 for that SEAL; no catalog sector written (write counter unchanged after barrier 1); writing entry discarded; service not in the `Corrupt` mount state (INFO without `Corrupt`, LIST unchanged, a later export seals). Kills: a step 4 that writes without running the validator and the extension check | S | host |
| N30 | Mixed catalog copies (2.6) | two valid copies with different sequences where the higher does not extend the lower: a record missing; a record changed; `next_free` lower; another destination ID; separately the legitimate case (higher one record ahead) | mount; INFO | non-extending pair: S `Corrupt`, INFO reports `Corrupt`, no write; legitimate pair: the higher copy is used. Kills: a mount that takes the higher sequence without the extension check | S | host |
| N31 | Operation × lane table (2.3) | control lane; unbound lane; export, verify and restore lanes bound | every operation 1..=11 on every lane kind, table-driven from the "Permitted on" column; a second CREATE on the same export binding after SEAL and after ABORT; LIST on each lane before and after a seal; VERIFY, READ, APPEND, SEAL and ABORT naming an entry other than the binding's; control ABORT naming an entry that is not writing | permitted operations pass check 6, all others `Access` -5; second CREATE `Access` -5; LIST scoping as in 2.3 (export lane: zero records until its entry is sealed, then that entry only; verify and restore lanes: the bound entry only; control: all); a wrong entry on an operation lane `Access` -5 (check 6, also in the failed-mount state); control ABORT of a non-writing entry `NotFound` -19. Kills: a permission table with any entry flipped; CREATE permitted twice per binding; a LIST that returns an entry outside the lane's scope; (revision 2) an entry check placed after check 7 (a wrong-entry READ in the failed-mount state returns `Io` -12) | S | host |
| N32 | Once-per-boot destination grant (2.1, O13) | R5 profile; a test feature makes the destination service exit (syscall 2) after its mount, and the profile then runs for at least as many kernel poll rounds as the M5 restart path needs (`m5.rs:784-794`) | kernel counts from the `Engine`'s object, node and handle tables before and after the exit: block objects whose scope intersects the destination scope, their nodes and their handles in every process, emitted by the kernel as `DEST:GRANT:OBJECTS=`, `DEST:GRANT:NODES=`, `DEST:GRANT:HANDLES=` | K: after the exit all three counts are 0 (`Engine::exit` reclaims the exited service's node and then the object, lib.rs:2003-2012) and stay 0 at every poll round until the profile ends; no process holds a handle on such an object; no process is started with one. Kills: a profile that re-creates the destination grant on a kernel-driven restart in the M5 pattern (`m5.rs:260-279`), observed as `DEST:GRANT:OBJECTS=` or `DEST:GRANT:HANDLES=` above 0 after the exit | K profile | QEMU (R5); no host model, because the rule is about kernel profile code |

Every row except N19, N20, N24, N26 and N32 needs only the destination service and
existing primitives; N24 also needs a simulated disk with late completion and,
natively, the K4/K5 fence of 2.7; N26 needs a broker stub; N32 needs the R5
profile with a test feature that ends the service and kernel-emitted counts. Each host test is
paired with a mutation that disables its check and must make the test fail, as for the archive and name-check tests *[main
practice]*.

## 7. Isolated disposable-QEMU verification plan

**Images.** All images are created fresh under `$NOS_BUILD_DIR/run/` by the
harness: the source volume image as today, and a destination image attached as
the second virtio-blk device at `00:06.0` *[main]*. No host disk, no physical
device, and no image survives between gate runs except as recorded evidence.
Every destination runner passes `-no-reboot` to QEMU (2.7), as
`tools/run_volume.py:309` does *[main]*, so a guest reboot ends the QEMU
process and each boot is a new process.

**PCI slot and entropy conflict (R5 precondition).** Every runner that
provides virtio-rng, the entropy source FORMAT needs, places it at `00:06.0`
(`tools/run.py:156`, `tools/run_keyring.py:32`) *[main]*, and the volume
runner places the second virtio-blk disk at the same slot without virtio-rng
(`tools/run_volume.py:322`) *[main]*. In addition, no volume-fixture profile
creates an `Entropy` object (`create_entropy` occurs only in the desktop,
keyring, nas, network_service, system and window_session profiles), and
`test-volume-archive-device` cannot be combined with `native-system`
(`volume.rs:15-18`) *[main]*. As `main` stands, `dest-format` cannot draw a
destination ID. Two ways out, both test-profile matters and not K4:
(a) a test-profile kernel change: virtio-rng at another PCI slot, accepted by
the kernel's device discovery (unchecked), plus an `Entropy` object and handle
for the destination service in the test-only destination profile; or (b)
*[proposed; owner decision O4 (c)]* a **deterministic test destination ID**:
under a test-only service feature, FORMAT writes a fixed 16-byte ID instead of
drawing one, the service holds no `Entropy` handle, and every FORMAT evidence
line carries the label `DEST:ID:TEST-ONLY`. Option (b) is non-product: it
never builds without the test feature, and it leaves FORMAT's entropy draw
without native evidence, which then comes only with the first product profile
under K4 (an R8 precondition) or earlier with option (a). Stages that need a
second destination ID (N14 `dest-foreign`) use an image whose catalog the
harness writes with another ID. R5 does not start before one option is in
place.

**Reuse.** `tools/run_volume.py` and the `PLAN` of `tools/test_volume.py`
*[main]*, the disk-1 attachment and the seven stages `archive-disk-*` *[main]*,
`tools/volume_archive.py` as the independent v1 reader, `tools/volume_disk.py`
for the source image, the `ARCHIVE_ESCAPE` counting and the
`VOLUME:ARCHIVE:READS|WRITES|FLUSHES` accounting. New host oracle:
`tools/destination_disk.py`, an independent Python parser of the catalog layout
(2.6) that shares no code with the Rust service and refuses every invalid copy.

**New stages (proposed; names indicative).**

| Stage | Parent | Checks |
|---|---|---|
| `dest-format` | blank disk 1 | two byte-identical catalog copies with sequence 1, no other sector written |
| `dest-export` | `dest-format` + recovered volume | one sealed entry; extent parses with `volume_archive.py`; disk 0 unchanged |
| `dest-export-2` | `dest-export` | second entry appended; first extent and its catalog record byte-identical |
| `dest-cut-append` | `dest-export` | process kill, no cache loss: `VOLUME:CUT:` marker after k writes, host kill of the QEMU process; `DEST:IO:FLUSHES` and the flush order at the marker match 2.7 |
| `dest-cut-seal` | `dest-export` | process kill, no cache loss: cut between barrier 1 and barrier 2; flush count and order at the marker as above |
| `dest-cut-mirror` | `dest-export` | process kill, no cache loss: cut after barrier 2 completes, before barrier 3 completes (N12, third point); flush count and order at the marker as above |
| `dest-after-cut` | each cut stage | cold boot; LIST and VERIFY per N11/N12, for `dest-cut-mirror` LIST = {E1, E2} and E2 verifies; oracle agrees |
| `dest-restore` | `dest-export-2` + blank disk 0 | verify then restore entry 2; byte-exact per the existing restore oracle |
| `dest-revoke` | `dest-export` | revoke mid-export (N6) |
| `dest-foreign`, `dest-full`, `dest-replay`, `dest-corrupt` | as N14, N15, N17, N7/N8 | as in the matrix |
| `dest-scope`, `dest-sender`, `dest-nonarchive` | `dest-export` | N3, N22 and N10 at QEMU level |
| `dest-grant-once` | `dest-format` | N32 |

**Evidence lines.** `DEST:CATALOG:SEQ=`, `DEST:ENTRIES=`, `DEST:NEXT_FREE=`,
`DEST:IO:READS|WRITES|FLUSHES` (with the sequence of writes and flushes up to
each cut marker), `DEST:HANDLES=` per process and stage, `DEST:GRANT:*` (N32),
`DEST:DENIALS=` and markers `DEST:CASE:<stage>:PASS`. `DEST:IO:*` and the
order of writes and flushes are counted by the kernel broker, keyed by the
destination object key, as `archive_ops` is for `VOLUME:ARCHIVE` today
*[main]*; `DEST:HANDLES=` and `DEST:GRANT:*` come from the kernel `Engine`;
the cut marker is emitted by the kernel after k destination writes,
independent of the flush count. None of these is a service self-report, so a
service that skips a flush cannot keep a counter consistent with the order of
2.7. `DEST:CATALOG:SEQ=`, `DEST:ENTRIES=` and `DEST:NEXT_FREE=` are service
reports and are cross-checked by the oracle's parse of the final image.
`DEST:DENIALS=` is a service report that no image parser can check; it
supports N22 but does not decide it. N22's native evidence is
`DEST:PORTAL:REPLYQ=`, the length of the portal's reply-endpoint queue,
emitted by the kernel from the `Engine`. Only these markers plus the raw QEMU exit
code 33 count as a pass, as for the existing gates.

**Cut stages are not power-cut evidence.** The `dest-cut-*` stages kill the
QEMU process with the host page cache intact (section 6), so they show
recovery after a process kill and the recorded flush order, not durability
under cache loss. Their recovery checks alone cannot detect a missing barrier
1 or 2; the flush count and order asserted at each cut marker can. Behaviour
under cache loss is accepted host-only (`SimDisk`, R2). A replay
oracle over QEMU's `blklogwrites` log, which would rebuild every image the
cache model allows and mount each, is an optional later research item, not a
requirement of this plan.

**Stage-count pins.** Adding stages to `PLAN` changes `len(PLAN)`, which two
protected workflow values must equal (`.github/workflows/kvm-pilot.yml` and
`.github/workflows/volume-acceptance.yml`, checked by
`tests/test_ci_workflows.py`). On `main` both pins and `len(PLAN)` are 22
(`volume-acceptance.yml:62`, `kvm-pilot.yml:124`, `tools/test_volume.py`)
*[main]*. Adding destination stages to `PLAN` therefore needs a lead-owned pin
update, or the stages go into a separate gate (section 8, O6).

**Remains unproven after this plan.** No IOMMU and no malicious-device model;
QEMU's virtio-blk is trusted. No physical disk, partition table or USB device.
No authenticity of the destination or archives. No hosted acceptance until the
hosted gate runs. No human audit.

## 8. Open items and staged implementation route

### 8.1 Items needing a project-owner decision

- **O1** Remembered permission: which operations may be remembered (proposed:
  export and verify, not restore), where the record is persisted, and whether
  the "remember" option is unchecked by default. A persisted remembered
  permission is a new extension of the existing per-session permission model,
  not the model D2 (a) refers to as existing, and needs owner confirmation
  under D2. Input for the decision: the record keys on the unauthenticated
  destination ID (3.3, 5.4), so a cloned catalog could receive automatic
  plaintext exports under a remembered export permission.
- **O2** Formatting: whether a non-blank device may ever be erased into a
  destination (proposed: no in v1; blank devices only).
- **O3** Retention: v1 has no deletion, so `Full` is terminal for a destination.
  Deleting or rotating old entries needs a decision, a catalog operation and its
  own crash tests.
- **O4** (a) Approval of kernel changes K1–K5 as needed (K4 and K5 are required
  for any product profile; K1–K3 can wait). Revision 2 widens K5 from one
  `Policy` per device to per-device broker state (4.3), because only that
  confines kernel -8 on disk 1 to disk 1's own reset (2.7); the approval asked
  for is the wider one. (b) **Owner question:** does the
  project owner amend ADR-S09-DUALDISK-20261119 before R5, so that B3 permits device 1
  to be held by the destination service with a whole-device scope (today B3
  grants device 1 only to the slot-1 portal, with a scope that ends below the
  disk end, `volume.rs:705-711` *[main]*), and B5 permits a scripted test-only
  dialog driver (today B5 states that there is no dialog and no persistent
  grant)? See [S09-DUAL-DISK-FIXTURE-REQUEST.md](S09-DUAL-DISK-FIXTURE-REQUEST.md).
  R5 does not start before this amendment.
  (c) (new in revision 2) **Owner question:** may R5 use the deterministic,
  test-only destination ID of section 7 (proposed), or must the test-profile
  kernel change (virtio-rng at another PCI slot plus an `Entropy` object in the
  test-only destination profile) be made before R5? R5 does not start before
  this is decided; either way FORMAT's entropy draw needs native evidence
  before R8.
- **O5** Time display: whether any wall-clock time is shown for an entry. The
  catalog stores none, and v1 object times are a per-domain logical clock,
  not wall-clock time ([S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md)).
- **O6** (lead-owned) workflow stage-count pins for new destination stages, or a
  separate gate.
- **O7** Whether the dialog may hold the lane issuing roots (section 4.3) or
  D4 (a) requires them in the portal. With the sender check of 4.3 the dialog's
  residual authority on an operation lane is to bind, transfer and revoke, not
  to perform an operation.
- **O8** (withdrawn in revision 1) In-boot restart after the failed-mount
  state. Revision 0 named a service restart as a recovery route. Under the
  proposed once-per-boot grant rule (2.1, O13), no instance in the same boot
  can regain the manifest-only, non-transferable block grant, so recovery from
  the failed-mount state and from the safe path is a reboot, and the restart
  route is withdrawn. If the owner later wants an
  in-boot restart, a re-grant mechanism and a kernel write latch in the
  per-device broker become owner decisions together; neither is assumed here.
  How the portal reports "reboot required" to the user remains a UI detail of
  R7.
- **O9** Completion fence (2.7): whether a write that the device accepted
  before an acknowledged reset can still reach the backing medium after the
  reset and the remount FLUSH, for QEMU and its host storage stack; whether the
  K4/K5 per-device broker implements the reset fence of the M3 volume broker
  for disk 1; whether a new QEMU process is equivalent to a device power cycle
  that no request of the previous boot survives; and how a product handles a
  warm reboot, which a guest cannot distinguish from a cold boot. All are
  *[unproven]*; N24 (b) records the outcome without counting it as a pass.
  Until they are shown, the 2.7 safe path applies: read-only for the rest of
  the boot after any device error on a destination write or flush (catalog or
  entry, `Io` included), carried by the single instance's in-memory latch
  under the once-per-boot grant rule (2.1, O13), with confirmation only after a
  cold boot and VERIFY. The gates avoid warm
  reboots by `-no-reboot` (2.7, section 7).
  - *Source-level evidence only.* A reading of the QEMU 10.2.0 source made for
    this revision indicates that `virtio_blk_reset` drains in-flight requests
    synchronously (`blk_drain`) and that a guest write of device status 0 resets
    the device synchronously after stopping ioeventfd. This is not empirical,
    it covers neither the host kernel's page cache nor asynchronous I/O
    teardown, and the review's own "not evidence" list includes reading QEMU
    source; it does not count towards O9.
  - *Planned empirical route* (the review's proposal; not to be run without
    owner scheduling, and only after the destination service, its
    `DEST:O9:*` markers and `tools/destination_disk.py` exist). Common setup:
    the runner's exact QEMU version and machine (`pc-q35-8.2`), a raw image on a
    local ext4 or xfs file system (not tmpfs), `aio=threads` and `io_uring`,
    `cache.direct=off` and `on`, `werror=report,rerror=report`, `-no-reboot`,
    and host-side QEMU trace events for virtio status writes and virtio-blk
    submit and completion. **E1 (in boot):** a test feature lowers the request
    timeout and emits markers for arming, reset begin, reset acknowledgement
    and remount FLUSH completion; at arming, the harness delays the targeted
    host `pwrite`/`fdatasync` below QEMU's block layer (for example with
    `strace` fault injection, if the installed version supports it) for longer
    than the timeout. Cells: step-4 sector 0, step-4 sector 3, step-5, the
    barrier-2 FLUSH and an APPEND entry write, each in both aio modes and both
    `cache.direct` settings, at least 20 runs per cell, each ended by QMP
    `quit` and parsed by `tools/destination_disk.py`. E1 supports O9 for that
    exact configuration only if every run shows the -16 reply and the reset
    path for the targeted request, the delayed host call returning before the
    status-0 write completes (or never issued), no write of pre-reset content
    to the targeted range after the remount `fdatasync` returns, and the
    guest's resolution equal to the oracle's parse of the final image; any
    pre-reset write returning later, or a resolution mismatch, refutes it.
    **E2 (across boots):** hold a host write as in E1, then kill the QEMU
    process, and separately kill it during a write flood with `io_uring` and
    `cache.direct=on`; after `waitpid` the image must stay byte-stable (hash
    and file-change watch) until the next QEMU process's first write, and a
    lock failure on restart is a failed start, not a pass. E2 shows only that
    no request survives a process restart; it shows nothing about cache loss.
  - *Not evidence for O9:* a delay shorter than the timeout; no host-side
    trace; guest readback or markers alone; `SimDisk`; reading QEMU source; a
    single aio mode; tmpfs, `cache=unsafe` or `no-flush`; a VM stop; a QEMU
    version, backend or options other than the runner's; any claim about
    physical media. The K4/K5 broker fence still needs its own native timeout
    fixture (R5). N24 and E1/E2 are planned tests, not evidence.
- **O10** `Corrupt` after an interrupted FORMAT (2.6, N25): the policy by
  which a product distinguishes a FORMAT interrupted before its first copy
  was durable from a damaged existing destination, and whether any recovery
  other than external re-zeroing of a disposable image is offered. Not
  specified; until decided the service never formats a `Corrupt` scope.
- **O11** (new in revision 1) **Owner question:** must a v1 destination be a
  block device of its own (proposed, 2.1), or may it be a partition whose
  scope is disjoint from every other block scope, accepting that it shares
  reset and FLUSH with the source device and is no independent copy against
  loss of that device? Requiring a separate device narrows the product scope
  that section 1 stated in revision 0 ("device or partition"). The exact-rights
  part of N18 does not depend on this decision. R8 activation is blocked until
  O11 is decided; the test stages use disk 1 and satisfy both options.
- **O12** (new in revision 1) **Owner question:** is the v1 destination
  service single-profile, with the dialog refusing every destination operation
  while more than one profile exists (proposed, 3.3), or is an owner principal
  bound per destination or per entry now, with the catalog and format change
  that needs? Without either, profile B could list and restore profile A's
  plaintext backup, contrary to
  [PROFILES-AND-USERS.md](PROFILES-AND-USERS.md). R7 and the R8 activation are
  blocked until O12 is decided; the test-only R5 and R6 stages run in a
  single-profile fixture.
- **O13** (new in revision 2) **Owner question:** is the in-memory latch of the
  single service instance accepted as the carrier of the read-only safe path,
  on the condition of the proposed once-per-boot grant rule (2.1: the
  destination block object is created exactly once per boot, and no kernel
  restart, rebind or poll path creates a block object intersecting the
  destination scope or delegates a handle on one; tested by N32 only, not
  enforced by the kernel), or does the project owner require a kernel write
  latch in the per-device broker (an extension of K5) that refuses every write
  to the destination scope for the rest of the boot after the trigger? On
  `main`, kernel profile code can re-create a block grant within one boot
  (`m5.rs:260-279`, `:784-794`), so the absence of TRANSFER alone does not
  carry the latch. R5 implements the proposed rule and N32, which are
  compatible with either answer; R8 activation is blocked until O13 is decided,
  and if the owner chooses the kernel latch, it and its own native test are
  added before R8.

### 8.2 Staged route

Each step is one reviewable draft PR; none activates a production boundary.

| Step | Content | Acceptance check |
|---|---|---|
| R1 | This specification; independent security review of it | review findings recorded and resolved in this file (review 1 and review 2: section 9; owner items O4 (b), O4 (c), O11, O12 and O13 open; a re-review of revision 2 is not yet done) |
| R2 | Host crate for the service core: catalog codec, mount rule, state machine, wire codec, write path; simulated disk with cache loss | host tests for N3–N15, N17 and N21–N31 (simulated disk with late completion for N24, broker stub for N26; N5 and N6 with the capability-core `Engine`), each with a killing mutation; flush-order and cache-loss acceptance for N11, N12 and N25 is established here, not natively; Clippy, fmt, `cargo test --locked` per the CI workflow commands |
| R3 | `tools/destination_disk.py` independent oracle | Python unit tests: parses valid copies, refuses each invalid rule; a test-only encoder for crafted images (N14's second destination ID, N4, N30), whose output the oracle's own parser accepts or refuses as intended |
| R4 | Volume service read-only and create-only endpoint bindings (separate spec) | N19, N20 host tests; existing volume and archive host tests unchanged |
| R5 | Preconditions: O4 (a) approves K5 as per-device broker state (K4 is not needed for R5); the owner amends ADR-S09-DUALDISK-20261119 B3 and B5 (O4 (b)); O4 (c) is decided and its option is in place (section 7); the test profile enables `monotonic-clock` for the -8 bound (2.7; feature combination unchecked). Content: per-device broker state, destination service binary in a test-only profile on disk 1 that follows the once-per-boot grant rule (2.1), and a scripted test-only dialog driver for the control lane that sends FORMAT and BIND and transfers lane SEND to the portal (4.3) | native stages `dest-format`, `dest-export`, `dest-export-2`, `dest-restore` and `dest-grant-once`, each lane bound by BIND with the destination ID that INFO reports after `dest-format`; N1, N2, N16, N18 counted and rights-probed (the driver's handles, including its control-reply RECV, as the dialog's in 5.1); N27 natively with `DEST:HANDLES=`; N32 natively with `DEST:GRANT:*`; at run time each lane handle transferred to the portal is SEND only (`check_endpoint`); every runner with `-no-reboot`; `dest-restore` is test-only and does not wait for R4; the per-device broker admits no disk-1 request between a disk-1 timeout and the acknowledged reset (2.7 completion fence), and a disk-1 submission is admitted while a disk-0 request is in flight, both shown by native timeout fixtures |
| R6 | UNBIND, revocation and rebinding through the scripted test-only dialog driver of R5 | QEMU levels of N3, N5, N6, N7, N8, N10, N14, N15, N17 and N22 with stages `dest-revoke`, `dest-foreign`, `dest-full`, `dest-replay`, `dest-corrupt`, `dest-scope`, `dest-sender` and `dest-nonarchive`; crash stages `dest-cut-append`, `dest-cut-seal`, `dest-cut-mirror` and `dest-after-cut` (N11, N12) at QEMU level as process kills without cache loss, with the kernel-counted flush count and order asserted at each cut marker (section 7) |
| R7 | Trusted dialog surface and remembered permission (after O1 and O12) | UI checks per `ui/AGENTS.md`; revocation of a remembered permission revokes live lanes and their volume-side bindings |
| R8 | After R4, O11, O12 and O13: independent security review of the implementation, then an owner activation decision | all matrix rows pass natively where marked QEMU; N19 and N20 pass natively, not only on the host; FORMAT's entropy draw has native evidence (section 7); under O13, either the once-per-boot rule (N32) is accepted or the kernel latch passes its own native test; review closed |

Until R8, every grant remains test-only and S09 remains open; this document
changes no score.

## 9. Review record

Review 1: [SPEC-REVIEW-1.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-1.md)
(independent AI review of revision 0 at `bc3f63a1`, checked against `main`
`035b44e5`; REQUEST_CHANGES, 13 required items). Each item was checked against
the primary sources before it was resolved; where that check narrowed an item,
the narrowed fix was applied and the remainder is recorded here as refuted or
not required. An AI review and its verification are not a human audit.

| # | Review item | Resolution | Where |
|---|---|---|---|
| 1 | Failed catalog read at mount | Resolved. A read error on sectors 0–15 is a failed mount, never an invalid or zero sector. Not required: the review's mirror before the next CREATE; with a read error terminal, the next commit already targets the lower-sequence slot and then mirrors | 2.6 mount rule and failed-mount state; 2.7 step 4 and "Why step 5"; N8 |
| 2 | Safe path covers only `Unknown` | Resolved. Trigger is any device `Io`, `Timeout` or `Unknown` (or persistent -8) on any destination write or flush; same provisional-until-cold-boot rule; reply codes per edge; readback `Timeout` edge added | 2.5; 2.7 failure paragraph and safe path; N13; N24 (d); O9 |
| 3 | Safe path not implementable as written | Resolved in revision 2 (review 2, F1). The grant is manifest-only and non-transferable; the single instance's latch carries read-only for the boot only under the proposed once-per-boot grant rule (2.1, owner decision O13, test N32); recovery is a reboot; `-no-reboot` in every destination runner; warm reboot stays under O9. Corrected: revision 1 called the kernel write latch (3a) refuted because "on `main` no profile can re-grant block authority in the same boot". That claim is false: `m5.rs:260-279` re-creates a block grant on every kernel-driven restart (`:784-794`). The absence of TRANSFER blocks only user-space delegation. The kernel latch is therefore not refuted; it is not required only on the condition of the proposed rule, and it remains the owner's alternative under O13. O8 restart route withdrawn on the same condition | 2.1; 2.6; 2.7 "Carrier of the read-only state"; section 7; O8; O9; O13; N32 |
| 4 | Kernel -8 wedge | Resolved; corrected in revision 2 (review 2, F5): K5 is per-device broker state, not only a per-device `Policy`; the service consumes each result before the next submission, so -8 on a disk-1 submission means only disk 1's own reset; the service treats -8 beyond 2 × `RESET_TIMEOUT` as a device `Io` and a pending poll beyond `TIMEOUT` + 2 × `RESET_TIMEOUT` as `Unknown` or `Timeout` | 2.7 completion fence; 4.3 K5; 5.2; N26 |
| 5 | Transferred capabilities on requests | Resolved. Check 1 of 2.2 closes every received handle and replies `Invalid`; the slot-exhaustion stall is recorded as a residual dialog-fault risk; `DEST:HANDLES=` added. Refuted: the portal-TRANSFER half; on `main` the portal's volume and archive handles carry no TRANSFER, and 4.1 already excludes it | 2.2; 4.1; section 7 evidence lines; N27 |
| 6 | No endpoint for control-lane replies | Resolved. Dialog-owned control-reply endpoint (dialog RECV, service SEND); handle lists corrected (block handle added, `Entropy` no longer counted twice) | 2.2 replies; 4.3; 5.1; N18 |
| 7 | N18 checks counts only | Resolved: N18 probes exact rights and scope disjointness, with mutations. Proposed, owner decision O11: v1 requires a separate device | 1; 2.1 device rule; N18; O11; R8 |
| 8 | Rebind replay | Resolved. Per-lane maximum generation for the instance; BIND with a generation not above it returns `Stale`; INFO reports the maxima | 2.3 BIND and INFO rows, "Grant generations"; 4.3; N17 |
| 9 | Conflicting status codes | Resolved; completed in revision 2 (review 2, F2–F4, F14): binding limits all in check 6, one state table in check 7, explicit check-2 fields. One total check order; requests rejected by checks 1–6 do not advance the high-water mark; `Corrupt` refusal code `Corrupt` -18; absent device is the failed-mount state with INFO reporting `NoDevice`; `NoDevice` only from BIND or INFO | 2.2 check order; 2.6; 5.3; N14; N28 |
| 10 | Catalog not validated before write | Resolved. Step 4 runs the mount validator and the extension check before any write; failure replies `Corrupt` for that SEAL only. Refuted: the review's three examples are unreachable under the spec's own rules (archive verify bounds the length; the entry-path scope rule bounds `next_free`; CREATE bounds the label length); the guard remains against encoder defects. The optional APPEND cap at 525 328 bytes was not added | 2.5; 2.7 step 4; N29 |
| 11 | Native cuts cannot detect a missing barrier | Resolved. QEMU cuts relabelled "process kill, no cache loss"; flush count and order asserted at each cut marker; flush-order and cache-loss acceptance host-only at R2; `blklogwrites` replay oracle optional later research | section 6 levels; N11; N12; section 7; R2; R6 |
| 12 | D3 gaps | Resolved: R8 requires R4 and native N19/N20; ending or revoking a grant or remembered permission ends the portal's volume-side binding. Proposed, owner decision O12: single-profile v1 or an owner principal; R7 and R8 blocked until decided. Refuted: gating the test-only R5 restore stage on R4; the trusted-portal dependency is already disclosed in 4.2 | 3.3; 4.1; O12; R5; R7; R8 |
| 13 | Fixture boundary and stale labels | Resolved: labels rebased on `main` `035b44e5` (PR #396 merged as `8a7a3594`); pins and R5 precondition rewritten. Proposed, owner decision O4 (b): amend ADR-S09-DUALDISK-20261119 B3 and B5 before R5 | header; 2.7; 4.3 K4; 5.2; 5.3; section 7; O4; R5 |

Review 1, optional items taken: READ bound and straddling read, reply echo checks (2.2,
2.3, N3); no blocking reply send (2.2) with idempotent UNBIND (2.3, 4.3); mount
extension rule (2.6, N30); contiguous extents (2.6); op × lane table, CREATE
once per binding and LIST scoping (2.3, N31); the cloned-catalog input to O1
(3.3, O1). Not taken: the dialog revoking after an UNBIND timeout together with
a kernel write latch, because under the proposed once-per-boot grant rule
(2.1, O13) no in-boot re-grant path exists; the kernel latch remains the
owner's alternative under O13; a sequence increase of 2 on a boot's
first commit, because it conflicts with the sequence + 1 extension check and
targets only a cross-boot late write that the O9 assumption already covers.

Review 2: [SPEC-REVIEW-2.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-2.md)
(independent AI re-review of revision 1, checked against `main` `035b44e5`;
REQUEST_CHANGES, required findings F1–F6, optional F7–F15). Each finding was
checked against the spec and `main` before it was resolved: F1 against
`kernel/src/m2/m3/m5.rs:260-279` and `:784-794`; F5 against `volume.rs:24-25`,
`:122-131`, `:900-912` and `:964` and the virtio-core `Phase` enum; F6
against `create_entropy` in `kernel/src/m2/m3/`, `tools/run.py:156`,
`tools/run_keyring.py:32` and `tools/run_volume.py:322`; F10 against
`capability-core` `send`, `resolve` and `maintain`; F13 against
`volume.rs:745-753`; F15 against `kernel/src/m2/m3.rs:2366-2370` and
`kernel/Cargo.toml`. All fifteen findings hold. Revision 2 adds owner questions
O4 (c) and O13 and does not decide them. An AI review and its resolution are
not a human audit.

| # | Finding | Resolution | Where |
|---|---|---|---|
| F1 | Latch argument rests on a false claim about `main` | Resolved. Once-per-boot grant rule stated by scope as *[proposed; owner decision O13]*, with the kernel write latch as the owner's alternative; new N32 (native, kernel-counted) kills the M5 restart pattern; section 9 item 3 corrected to the conditional basis | 2.1; 2.6 failed-mount state; 2.7 "Carrier of the read-only state"; 4.3 K5; O8; O9; O13; N32; R5; R8; section 9 item 3 |
| F2 | N13 conflicts with the check order | Resolved. All binding limits are in check 6, including the *unknown* export binding (INFO and LIST only, by check 6) and a wrong entry on an operation lane (N31); N13 rewritten per lane and binding for both remount outcomes, with a check-6/7 swap mutation | 2.2 check 6; 2.7 failure paragraph; N13 |
| F3 | Header table contradicts BIND/UNBIND `count` and SEAL `offset` | Resolved. `offset`, `count` and `aux` rows corrected; a field-rule table assigns every header field of every operation to check 2, and check 2 names the entry check (check 6) and the catalog-dependent checks left to the operation | 2.2 header table, field-rule table and check 2 |
| F4 | Reachable states without a gate | Resolved. One state table in check 7 with INFO encoding (blank: `NotFound` -19, destination ID zero); the pre-fence interval is "remount pending", in which no request is received; a FORMAT write or flush failure enters the failed-mount state; blank-state replies defined; N8, N24 (a), 5.3 aligned | 2.2 check 7; 2.3 INFO row; 2.6 blank, failed-mount state, Format; 2.7; 5.3; N8; N13; N24 |
| F5 | -8 bound false-triggers on shared broker state | Resolved by option (a): K5 is per-device broker state (`Policy`, `resetting`, `reset_deadline`, `error`, `active`, data buffer); the service consumes each result before the next submission; poll -8 (pending) is bounded separately; N26 (d) and (e) added; R5 also shows that disk 0 in flight does not block disk 1 | 2.7 "Meaning of kernel -8"; 4.3 K5; 5.2; O4 (a); N26; R5 |
| F6 | PCI-slot/entropy conflict without a home | Resolved as an R5 precondition, classified as a test-profile matter, not K4: option (a) test-profile kernel change, or option (b) a deterministic test-only destination ID *[proposed; owner decision O4 (c)]*; 5.1 and N18 adjusted; N14 uses a harness-written second ID; native entropy evidence becomes an R8 precondition | section 7; 5.1; N14; N18; O4 (c); R5; R8 |
| F7 | N28 overclaims | Taken. Kill claim narrowed to distinguishable swaps; equivalent mutants listed | N28 |
| F8 | Emitting layer of flush evidence | Taken. `DEST:IO:*`, flush order and the cut marker from the kernel broker keyed by object key; `DEST:HANDLES=` and `DEST:GRANT:*` from the kernel `Engine` | section 7 evidence lines; N12 |
| F9 | Route coverage gaps | Taken. Every host level assigned to R2 (or R4), every QEMU level and stage to R5 or R6; three QEMU stages added for N3, N22 and N10 | section 7 stages; R2; R5; R6 |
| F10 | Dialog cannot revoke after the service exits | Taken. A kernel `Revoked` -10 or `Closed` -9 on the UNBIND send permits the revoke (corrected after review 3: `Engine::exit` revokes the dialog's delegated nodes, so -10 is the normal case); N6 covers it | 4.3 "To revoke", N6 |
| F11 | Spurious replies to the portal | Taken. An operation-lane request from a sender other than the portal gets no reply; N22 and N27 observe that, natively through the kernel-reported length of the portal's reply queue (`DEST:PORTAL:REPLYQ=`), not through the service's `DEST:DENIALS=` | 2.2 Replies; N22; N27 |
| F12 | Decay window after a crash between barriers 2 and 3 | Taken as an accepted residual; a mount-time mirror is declined, because it adds a third catalog write path exactly where the safe path forbids writes | 2.7 "Why step 5" |
| F13 | Inaccurate `archive_grant` citation | Taken. Cited as a precedent for comparing scopes, in 2.1 and N18 | 2.1 device rule; N18 |
| F14 | Self-contradictory high-water rule | Taken. A request refused by checks 1–6 leaves the mark unchanged; one that passes checks 1–6 advances it, also when check 7 refuses it; N17 tests both | 2.2; N17 |
| F15 | Hidden time-source dependency; UNBIND latency | Taken. Generic syscall 3 under `monotonic-clock` named, its combination with the volume-fixture features unchecked, listed as an R5 precondition; UNBIND latency stated as at most two bounds in N26 (b) | 2.7 "Time source"; N26; R5 |

Review 2, optional items declined: none in full. Partly declined: in F12,
the mount-time mirror (reason above); in F5, option (b), widening the bound to
`TIMEOUT` + `RESET_TIMEOUT`, because it would only mask shared broker state
that the product must not have and would delay every genuine reset failure by
five seconds.

Review 3 ([SPEC-REVIEW-3.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-3.md),
APPROVE, optional findings only):

| Finding | Subject | Resolution | Where |
|---|---|---|---|
| R3-1 | F10 named the wrong kernel error | Taken: `Revoked` -10 or `Closed` -9 permits the revoke; N6 case (b) added | 4.3 "To revoke", N6, F10 row |
| R3-2 | N32 kill observation did not hold | Taken: counts are 0 after the exit and stay 0; the kill is a count above 0 | N32 |
| R3-3 | Leftover unconditional "no in-boot re-grant" claim | Taken: made conditional on O13 | this section, review 1 optional items |
| R3-4 | Source of the portal's process ID unspecified | Taken: the manifest writes it into the service's start context; N18 checks it | 4.3 |
| R3-5 | Consistency nits | Taken: (a) table reference and the order within check 2; (b) failed-mount row excludes the FORMAT scan; (c) kernel -14 and -13 on writes mapped in 5.3; (d) K4 removed from the R5 preconditions. Not taken: (e) native N19/N20 are already named in R8; (f) the bound wording is left as revision 2 states it | 2.2, 5.3, 8.2 |
