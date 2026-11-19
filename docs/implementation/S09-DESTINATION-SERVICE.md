# S09 — Backup destination service, trusted portal and per-operation rights (design specification)

**Status: design specification only. Nothing in this document is implemented,
tested or active, and it is not a production security boundary.** It specifies
D1 (b), D2 (a), D3, D4 (a) and D5 (a) of
[ADR-S09-DESTINATION-20261119](../DECISIONS.md#adr-s09-destination-20261119--backup-destination-service-trusted-dialog-and-os-backup-portal).
The ADR's implementation boundary applies unchanged:

> A separate destination service and any new production capability boundary
> require a documented interface, a security review, negative tests and
> verified enforcement before activation.

This document is the documented interface. The security review, the negative
tests of section 6 and verified enforcement remain open. The existing
test-fixture grant in [S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md) ("Native
slice") remains a fixture and is not changed by this text. S09 stays open.

**Evidence labels used below.** *[main]* means present on `main` at `0b0ed91a`.
*[PR #396]* means present only on the unmerged branch
`claude/s09-independent-recovery` (head `d0d00fde`): the test-only second
virtio-blk disk at PCI `00:06.0`, the portal-only `BlockScope { device: 1, .. }`
grant, the 22-stage volume gate, the `SimDisk` host tests and the documents
`docs/implementation/S09-DUAL-DISK-FIXTURE-REQUEST.md` and
`docs/verification/s09-archive/dual-disk-native-20261119/`; those files do not
exist on this branch and are named in plain text. *[design]* means specified
here and not implemented. *[unproven]* means an assumption without a test.

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
  covers only local, user-owned block devices or partitions; the service refuses
  any other destination kind by construction, because it only accepts a block
  capability.

## 2. Destination service interface (D1 (b))

### 2.1 Ownership

The destination service is a userspace system service *[design]*. At boot the
manifest gives it exactly one block capability on the whole destination device
or partition: `create_block_device_with_rights(service, scope, BLOCK_READ |
BLOCK_WRITE | BLOCK_FLUSH)`. The legacy `create_block_device` constructor must
not be used, because its default `BLOCK_READ_RIGHTS` includes DERIVE, TRANSFER
and REVOKE *[main]*; with them the service could pass raw device authority on.
No other process holds any capability on the destination device. The service
never writes outside the region rules of section 2.6, and it never holds a
capability on the source volume's block object (section 5).

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
| 32 | offset u64 | byte offset (APPEND, READ), else 0 | next offset or total length |
| 40 | length u32 | ≤ 160 and = data length, else `Invalid` | reply data length ≤ 160 |
| 44 | status i32 | must be 0 | service status (2.4) |
| 48 | name_length u32 | CREATE: label length 1..=32 and = `length`, else `Invalid`; BIND: operation 1..=3; else 0 | per operation |
| 52 | count u32 | 0 except LIST cursor | per operation |
| 56 | aux u64 | per operation, else 0 | per operation |
| 64 | data[length] | bytes after `length` must be 0, else `Invalid` | zero-padded |

Every field is validated before any state is consulted, as `validate_request`
does for the volume. A request on an operation that the lane's binding does not
permit returns `Access` before any device I/O. Status order on an operation
lane: an unbound lane returns `Access` for every request; a bound lane whose
kernel-stamped sender is not the bound portal (4.3) returns `Access`; a bound
lane with a wrong `epoch` or a non-increasing `request_id` returns `Stale`. The
control lane has no binding: its `epoch` is always 0 and its `request_id`
sequence starts afresh when the service starts.

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
| 1 | INFO | entry 0 | aux = free sectors, count = sealed entries, length 24: data = 16-byte destination ID, u64 formatted sectors; `Corrupt` or blank state in status/aux | every lane, control |
| 2 | LIST | entry 0, count = cursor 0..16 | up to 2 entry records of 80 bytes (2.6 layout), length = 80 × n; count = next cursor or 0 | control (all entries); export lane sees only its own entry once sealed; verify/restore lanes only their bound entry |
| 3 | CREATE | entry 0, label in data | entry = provisional ID (2.3), state `writing` | export lane, once per binding |
| 4 | APPEND | entry = writing ID, offset = bytes appended so far, ≤ 160 bytes | offset = new total | export lane |
| 5 | SEAL | entry = writing ID, offset = total length | entry state `sealed`, aux = archive trailer FNV | export lane |
| 6 | ABORT | entry = writing ID | state discarded | export lane, control |
| 7 | VERIFY | entry = bound ID | aux = source generation, count = objects; or `Corrupt` | verify lane |
| 8 | READ | entry = bound ID, offset, aux = byte count 1..=160 | data = bytes; `Range` past the length | restore lane |
| 9 | BIND | entry = entry ID (0 for export), aux = new grant generation (> every earlier one for that lane), count = lane 1..=3, name_length = operation (1 export, 2 verify, 3 restore), length 16: data = destination ID the grant names | Ok; `NoDevice` if the mounted ID differs; `NotFound` if the entry is not sealed; `Busy` if the lane is bound | control |
| 10 | UNBIND | count = lane 1..=3 | Ok; any writing entry on that lane is discarded; the lane is unbound, so later requests on it get `Access` until a new BIND | control |
| 11 | FORMAT | entry 0, length 0 | Ok, reply data = new destination ID; `Access` unless the scope is blank (2.6) | control |

The request offset in APPEND must equal the service's own count of appended
bytes; a mismatch returns `Stale` and appends nothing. This makes a duplicated
or replayed APPEND harmless. SEAL with an offset that differs from the appended
total returns `Invalid` and discards nothing; the portal may then ABORT. A
second CREATE while an entry is `writing` on any lane returns `Busy`: one
writing entry per destination. CREATE when 16 entries are sealed, or when
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
                     |  \--ABORT / UNBIND / revoke / any write error / reboot--> (discarded)
                     \--SEAL fails (Corrupt, Io, Full)--> (discarded)
                     \--SEAL Unknown--> (decided by the catalog after remount)
```

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
| 16 .. `next_free` | sealed entry extents, ascending, contiguous or with gaps left by discarded writes |
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
disjoint, each extent inside [16, `next_free`), 16 ≤ `next_free` ≤ formatted
count (also for a catalog with zero entries),
formatted count = granted scope's `sector_count`, every entry ID < next entry
ID, next entry ID ≥ 1 (also for a catalog with zero entries), and all padding
zero.

The destination ID is 16 bytes drawn once at format time from the existing
entropy capability (an `Entropy` object and handle held by the destination
service; section 5.1). It is an identifier, not a secret and not authentication.
FNV-64 detects accidental damage only; a deliberately altered, swapped or
rolled-back image is not detected (section 5.4).

**Mount rule.** Read both copies. If exactly one is valid, use it. If both are
valid, use the higher sequence (the read is preceded by a FLUSH, below);
equal sequences with different bytes give `Corrupt`. If neither is valid and
all sectors 0–15 are zero, the device is a *blank candidate*; zero in sectors
0–15 is only a heuristic and does not show that the rest of the scope holds no
data. Otherwise the destination is
`Corrupt`: the service refuses every operation except INFO and never formats
automatically. `Corrupt` is terminal for the service instance: no request
clears it and the service never repairs or formats, and a new instance that
mounts the same bytes reaches `Corrupt` again. For a disposable test image the
only recovery is external: its owner (for the gates, the harness that created
it) re-zeroes the whole granted scope, not only sectors 0–15, because FORMAT
refuses a scope with data at sector ≥ 16; a new instance then mounts a blank
candidate and FORMAT is permitted (N25). How a product distinguishes a FORMAT
interrupted before its first copy was durable from a damaged existing
destination, and whether any in-product recovery is offered, is open
(section 8, O10). Every mount, including a remount (2.7), first issues a FLUSH
and reads the catalog only after it completes, so that the copies read are
not merely held in a volatile write cache. **Failed mount FLUSH.** If a mount
or remount FLUSH fails, the service writes nothing; INFO is answered and
reports `Io` -12 in its status field; LIST on an export lane whose binding is
still *unknown* (2.7) returns status `Unknown` -16 with zero records; every
other request on any lane returns `Io` -12. This state is terminal for the
service instance: the service does not retry the FLUSH and no request clears
the state, because a failed FLUSH gives no evidence about earlier writes, and
in the broker pattern of 2.7 a device reset that is not acknowledged
quarantines the kernel `Policy` until reboot
(`crates/virtio-core/src/lib.rs:560-569`, `kernel/src/m2/m3/volume.rs:1075-1079`
*[main 8a7a3594]*). Normal operation resumes only in a new service instance
whose own mount FLUSH completes: a service restart, or a reboot when the block
lane is quarantined; after an `Unknown` on a destination write or flush, only
a cold boot ends the read-only state of the 2.7 safe path. Until then the
replies above apply unchanged, and the portal reports the destination as
unavailable and any pending export as not confirmed. Who initiates the
restart, and through which mechanism, is not specified (section 8, O8). An
export binding whose outcome is *unknown* stays
*unknown* for the life of the instance; it never resolves as *sealed* without a
completed FLUSH. After the restart, bindings and provisional IDs are void; the
entry is judged only from the catalog that the new instance reads after its
own completed mount FLUSH, like any other sealed entry (control LIST, VERIFY),
and the earlier export is never reported as confirmed on the basis of its
provisional ID.

**Format.** Only the trusted control path may format, and only a *blank*
scope: before any write, FORMAT reads the whole granted scope and returns
`Access` unless every sector is zero. Format writes copy A with sequence 1,
zero entries, next entry ID 1 and `next_free` = 16, flushes, writes the identical copy B, and flushes. The cost of the full-scope
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
`drivers/virtio-blk/src/lib.rs:243-244` *[main 8a7a3594]*). Three claims are kept
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
4. Write the new catalog copy (sequence + 1, the entry appended, `next_free`
   advanced past the extent, next entry ID + 1) into the *target* slot: the
   slot holding an invalid copy, otherwise the slot with the lower sequence,
   otherwise (both copies valid and byte-identical) copy A; **flush
   (barrier 2)**.
5. Mirror the identical copy into the other slot; **flush (barrier 3)**. Reply
   `Ok` only after barrier 3 completes.

If step 4, barrier 2, step 5 or barrier 3 fails with `Io`, `Timeout` or
`Unknown` (the broker pattern below reports a timed-out write or flush as
`Unknown` -16), the
service replies `Unknown`, discards its in-memory catalog and writing-entry
state and remounts from the device before it accepts another request. Lane
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
The lane then accepts only INFO and LIST; its LIST returns the entry record if
the outcome is *sealed* and zero records otherwise. If the remount's FLUSH
fails, the outcome stays *unknown* and the failed-mount-FLUSH rule of 2.6
applies (INFO answered with `Io`; the export lane's LIST returns `Unknown` with
zero records; every other request `Io`). Bindings do not survive a service restart;
a later instance mounts only after its own mount FLUSH completes, and the
export is then not confirmed to the portal.

**Completion fence.** The remount FLUSH is issued and the export binding is
resolved only after the request that failed is quiescent: completed and
consumed, or ended by an acknowledged device reset. Until that fence the
binding stays *unknown* and every CREATE is refused; after it, CREATE stays
refused for the rest of the boot under the safe path below. On
`main` the M3 volume broker implements this fence for its device *[main
8a7a3594]*: one global `Policy` admits at most one request
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
disk 1; this is a requirement, not existing behaviour. A kernel -8 during the
reset is not a FLUSH result: the service resubmits after the broker leaves the
reset, which ends within `RESET_TIMEOUT` (`volume.rs:25`) either ready or
offline, and offline counts as a failed FLUSH (2.6). Because the `Policy`
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
fails. **Safe path while O9 is unproven:** after any `Unknown` on a destination
write or flush, catalog or entry (the latter widens the trigger to a timed-out
entry write during APPEND or SEAL step 3), the destination accepts no further
catalog or entry write for the rest of the boot (read-only: INFO, LIST,
VERIFY, READ); CREATE, APPEND, SEAL and FORMAT on any lane return `Access` -5
with no device write. These `Access` replies apply only while the instance's
mount or remount FLUSH has completed and the catalog is not `Corrupt`: the
failed-mount-FLUSH state (`Io` -12, INFO and the *unknown*-binding LIST
excepted) and the `Corrupt` state (2.6) take precedence, and neither writes to
the device. A late write therefore cannot overwrite a newer
catalog copy or a newly allocated extent: no extent is reused in that
boot. In particular, if the remount resolves E2 as *not sealed*, E2's sectors
lie at or beyond the committed `next_free` and would be the next CREATE's
extent, but no CREATE, and so no APPEND, is admitted in that boot, so a late
E2 catalog write cannot make E2 listed over another entry's data. Across a
cold boot the late write cannot occur only under the power-cycle assumption
below; reuse of E2's sectors by an export in the next boot is safe only
under that assumption, and O9 remains a blocking open assumption. The
read-only state must also bind a restarted service instance in the same boot
and survive any reboot that is not a cold boot; the mechanism that carries it
is not specified (O8, O9), and until it is, recovery after such an `Unknown`
is a cold boot. The in-boot resolution is reported to the portal only as
"provisional"; the export counts as confirmed only after a cold boot, whose
mount FLUSH and catalog read follow a device power cycle that no request of
the previous boot survives *[design; QEMU process restart assumed
equivalent, unproven]*, and after VERIFY of the entry. The portal treats
`Unknown` as "not sealed" until the export lane's LIST returns the entry ID;
it must not repeat the export blindly and must not report success. If the
remount yields `Corrupt`, the service fails closed as in 2.6.

**Append-only meaning.** No operation writes a sealed extent, and catalog
copies are written only in the order above. An interrupted new export therefore cannot damage
any previously sealed entry. This resolves the [PR #396] host finding that on a
single raw range an interrupted re-export destroys the previous archive as soon
as one differing sector lands.

**Why step 5 mirrors the catalog.** Without step 5, decay of the newer copy
while the older one stays valid would make the mount rule fall back to the
older copy; the newest sealed entry would then no longer be listed, and its
extent would lie at or beyond the older `next_free`, so a later export could
overwrite it. Silent media decay is outside the crash model; the mirror is a
mitigation subject to the R1 review. After a completed commit both copies are
byte-identical, so decay of one copy loses no entry; decay of both gives
`Corrupt` and fails closed, unless sectors 0–15 all read as zero, which the
mount rule treats as a blank candidate that FORMAT still refuses while entry
data remains (N8). A crash between barrier 2 and barrier 3 leaves
one newer and one older valid copy, which the mount rule handles, and the next
commit completes the mirror. The catalog write path accepts only the eight
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
   every live lane bound to that destination. Restore is not remembered by
   default; whether it may be is O1. Persisting this record needs a new
   `Duration` and storage in `task-grant-core` or a new store, which is an SDK
   change, not a kernel change. It extends, rather than follows, the existing
   permission model named in D2 (a) (section 8, O1).
2. **Per-operation grant**: a kernel capability node with SEND only on one
   operation lane, plus the service's lane binding `{operation, entry ID, grant
   generation, portal process ID}` (section 4.3). It lives until the operation ends or is revoked.

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
lane on *[main mechanism: `capability-core` `subset`/`require`]*.

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
  the manifest that starts the portal *[design]*. A request on a bound
  operation lane whose `sender_pid` differs returns `Access` before any state
  is consulted (N22). The service refuses any request on an unbound lane with
  `Access`. That process IDs are not reused within a boot is assumed
  *[unproven]*. Placing the issuing roots in the dialog
  rather than in the portal is this document's interpretation of D4 (a)
  ("a trusted OS backup portal holds destination capabilities"): the portal
  holds every capability that performs an operation, the dialog only the
  authority to issue and revoke them. The project owner may require the roots
  in the portal instead (section 8, O7).
- Replies: as for the volume, the portal owns its own reply endpoint, keeps
  RECV on it and the manifest delegates SEND on it to the destination service.
  The dialog reaches the portal through one further portal-owned endpoint, on
  which it transfers lane handles. Both handles are included in the manifest
  counting of section 5.1. With an IPC transfer the new node is a child of the
  sender's node *[main, `Engine::send`]*, which is what makes the dialog's
  `revoke` reach the portal's lane handle.
- To grant: the dialog sends BIND (lane, operation, entry, new grant generation)
  on the control lane, waits for `Ok`, then transfers SEND on that lane to the
  portal, and tells the portal the grant generation, which the portal places in
  every request's `epoch` field.
- To revoke: the dialog first sends UNBIND on the control lane and waits for
  `Ok`, so that any request already queued on the lane (including a SEAL) is
  answered with `Access` and cannot commit; it then calls `revoke`
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
  exists only under a test feature *[PR #396]*.
- **K5** One virtio `Policy` per block device, so that a destination fault
  cannot fail the source volume closed (section 5.2).

## 5. Isolation between source and backup devices

### 5.1 Ownership rule

The volume service holds a block capability only on the source volume scope; the
destination service holds one only on the destination scope; the portal holds
neither. The volume service never holds a destination lane or block handle, and
the destination service never holds a volume endpoint or block handle. At
manifest time this is checked by counting, as `archive_grant` does today
*[main]*: one new block object, node and handle per grant, in the intended
process, with every other process's handle count unchanged and distinct object
keys. The same counting covers the endpoint handles of section 4.3: the
destination service holds exactly four lane RECV handles, one SEND handle on
the portal's reply endpoint and one handle on its own `Entropy` object (one of
`MAX_ENTROPY_OBJECTS` = 4) for FORMAT *[main limit]*; the dialog holds one control SEND, three lane roots
and one SEND on the portal's intake endpoint; the portal holds its reply and
intake RECV handles and, at most, three lane SEND handles at run time.

### 5.2 DMA and lanes

The kernel virtio-blk driver performs DMA; userspace never does. [PR #396]
gives the second device its own DMA pages, disjoint from device 0, and an
unmapped lane-1 MMIO window after review. It also records a fixture limit: both
lanes share one virtio `Policy`, so a failed device-1 reset fails both lanes
closed. That is safe but couples availability: a faulty backup device stops the
source volume. The product requires one `Policy` per device, so that a
destination fault yields `Io` or `NoDevice` on the destination only. This is a
kernel change (K5) inside the virtio-blk path, not an ABI change.

### 5.3 Device identity

A destination is identified by its catalog destination ID. A grant names the
destination ID; if the mounted device holds another ID, or no device is present,
the service returns `NoDevice` and performs no write. The current fixture's
"wrong device" check (portal granted the device-0 range, verify gives
`Corrupt`) [PR #396] becomes a `NoDevice` refusal before any I/O.

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
volatile write cache (as the [PR #396] `SimDisk`); **QEMU** = native stage on
disposable images.

| # | Case | Precondition | Action | Expected (layer: code) | Enforced by | Level |
|---|---|---|---|---|---|---|
| N1 | Unauthorized device | app or portal without a destination block handle | block read/write with own or forged handle | K: `BadHandle` -4 or `Access` -5; counted in `app_device_denials` | K | QEMU |
| N2 | Scope escape, device | destination service | block write at scope end | K: `Access` -5; host sees no byte past the scope | K | QEMU |
| N3 | Scope escape, entry | restore lane bound to entry E | READ entry ≠ E; READ past E's length; APPEND on a verify lane | S: `Access` -5; `Range` -3; `Access` -5 | S | host + QEMU |
| N4 | Scope escape, sealed extent | export writing after sealed entries; separately, a crafted catalog with zero entries and `next_free` < 16; a crafted catalog with zero entries and next entry ID 0 | crafted write path below `next_free` and below sector 16 (mutation test); mount of each crafted catalog | write refused; sealed extents and sectors 0–15 byte-identical; each crafted copy is invalid | S | host |
| N5 | Revoked before operation | dialog revoked the lane | portal `send` | K: `Revoked` -10 (syscall return) | K | host + QEMU |
| N6 | Revoked during export | 50 % appended | dialog sends UNBIND, then revokes | K: `Revoked` -10 on next send; S discards writing entry; LIST unchanged; earlier entries verify | K + S | host + QEMU |
| N7 | Corrupted entry | one byte flipped in a sealed extent on the host image | VERIFY; restore flow | S: `Corrupt` -18; P: `archive::verify` `Corrupt`, nothing restored | S + P | host + QEMU |
| N8 | Corrupted catalog | one copy damaged; then both damaged; then both copies zeroed with sectors 8–15 zero and sealed entries at sector ≥ 16 | mount; control FORMAT | one copy: the other, identical copy used, LIST unchanged; both damaged (a non-zero sector in 0–15): S `Corrupt`, only INFO answered, no write, no format; both zeroed: blank candidate per the mount rule (INFO reports blank), no write, FORMAT `Access` -5 after the full-scope read finds the entry data (as N23) | S | host + QEMU |
| N9 | Incomplete write | appended ≠ declared | SEAL | S: `Invalid` -1; nothing listed | S | host |
| N10 | Non-archive bytes | appended bytes fail `verify` | SEAL | S: `Corrupt` -18; nothing listed; `next_free` unchanged after reboot | S | host + QEMU |
| N11 | Power cut during APPEND | one sealed entry E1; export of E2 running | QEMU SIGKILL after k destination writes | after cold boot: LIST = {E1}; E1 verifies byte-identical | S (2.7) + H | host + QEMU |
| N12 | Power cut during SEAL | cut after barrier 1, during the catalog write, after barrier 2 | SIGKILL at each point (host: every write prefix with cache loss) | LIST = {E1} or {E1, E2}; a listed E2 verifies; never a listed E2 that fails | S + H | host + QEMU |
| N13 | `Unknown` at SEAL | catalog flush fails with `Timeout` or `Unknown` (broker timeout, 2.7) at barrier 2 or 3, with and without the new copy durable; then, after the 2.7 completion fence (N24), the remount FLUSH succeeds, or fails | SEAL, then LIST on the export lane; CREATE on another lane before and after resolution | S: `Unknown` -16, remount, binding kept; remount FLUSH succeeds: export-lane LIST returns E2's record if the remounted catalog lists E2 (P reports provisionally sealed, confirmed only after cold boot and VERIFY, 2.7), else zero records (P reports not sealed); remount FLUSH fails (2.6 failed-mount-FLUSH rule): LIST status `Unknown` -16 with zero records, INFO reports `Io`, every other request `Io` -12, no device write, no FLUSH retry, state unchanged until restart, binding never resolved as sealed (P reports not confirmed); after a restart the provisional ID is void and only the new instance's catalog decides; remount FLUSH succeeds: CREATE on any lane before and after resolution refused for the rest of the boot (`Access` -5, 2.7 safe path), further CREATE/APPEND/SEAL on that lane `Access` -5; remount FLUSH fails: CREATE, APPEND and SEAL on any lane `Io` -12 (failed-mount-FLUSH state takes precedence, 2.7); no repeated export | S + P | host |
| N14 | Wrong or foreign device | grant names ID X; device holds Y or a non-Baryon, non-blank layout | any lane op; format | S: `NoDevice` -14, no I/O; format not offered, control format `Access` -5 | S + D | host + QEMU |
| N15 | Destination full | free space < archive | export | S: `Full` -17 at APPEND; nothing sealed; earlier entries intact; catalog sequence unchanged | S | host + QEMU |
| N16 | Third-party raw access | app process in the profile | forged lane, control and block handles; IPC to service without a handle | K: `BadHandle` -4; counted; service receives nothing | K | QEMU |
| N17 | Stale grant replay and header rules | lane rebound with generation g+1; lane unbound; control lane | request with epoch g; duplicated APPEND offset; second SEAL; any request on an unbound lane; control request with epoch ≠ 0 or non-increasing `request_id`; CREATE with `length` ≠ `name_length` | S: `Stale` -10 (reply status); no extra bytes; no second entry; unbound lane `Access` -5; control `Stale` -10; CREATE `Invalid` -1 | S | host + QEMU |
| N18 | Authority escape at manifest | profile start | count objects, nodes and handles; probe object keys | destination service holds exactly the 5.1 handles plus one `Entropy` object and handle; any extra handle in volume service, destination service or app: manifest fails (`ARCHIVE_ESCAPE`-style) | K manifest | QEMU |
| N19 | Volume read-only binding | export lane plus RO volume endpoint | portal sends BEGIN/META/DELETE to the volume | V: `Access` -5; volume unchanged | V | host (after R4) |
| N20 | Volume create-only binding | restore into a domain with occupied slots | portal sends a commit into an occupied slot, DELETE, RENAME | V: `Access` -5; existing objects keep their revision | V | host (after R4) |
| N21 | Second writer | export lane writing | CREATE on another lane | S: `Busy` -21 | S | host |
| N22 | Dialog uses a bound lane | lane bound to the portal | dialog sends INFO, APPEND or READ on that lane with its own root | S: `Access` -5 (`sender_pid` ≠ bound portal); no I/O | S | host + QEMU |
| N23 | Data beyond the catalog area | catalog sectors 0–15 zero, non-zero data at sector ≥ 16 | control FORMAT | S: `Access` -5; no sector written | S | host |
| N24 | Late-completing write (2.7 completion fence) | simulated disk that completes a timed-out catalog-sector write late: (a) before the fence (reset acknowledgement), (b) after the remount FLUSH, (c) after a remount that resolved E2 as *not sealed*; (d) an entry-sector write during APPEND that times out (`Unknown`) and completes late | SEAL with the step-4 write timing out (`Unknown`); CREATE on another lane and LIST on the export lane before the fence; (c) CREATE and APPEND on another lane after the resolution, before the late write lands; (d) CREATE and APPEND on another lane after the acknowledged reset; then remount and cold boot | (a) before the fence no remount FLUSH, no resolution, CREATE refused, export-lane LIST `Unknown` with zero records; after it the resolution matches the cold-boot catalog; (b) outside the assumed device model (O9), recorded, not a pass: no sealed extent written, never a listed entry that fails VERIFY, LIST after cold boot {E1} or {E1, E2} (E2 listed although P reported not sealed: safe direction) or S `Corrupt` fail closed with E1 byte-identical; mutation that issues the remount FLUSH or admits CREATE before the fence must fail (a); (c) CREATE `Access` -5 and APPEND `Access` -5 in that boot, no device write, E2's sectors not allocated again, so the late E2 write lands on no other entry's data; mutation that admits CREATE after the resolution in the same boot must fail (c); (d) writing entry discarded, CREATE and APPEND `Access` -5 for the rest of the boot, the late entry write lands only in unallocated sectors; mutation that admits CREATE after the reset in the same boot must fail (d) | S + K fence | host |
| N25 | Power cut during FORMAT | blank scope | FORMAT with a cut at every write prefix of copy A, with cache loss (host), before copy B | no non-zero sector of copy A durable (a zero-entry copy A is non-zero only in sectors 0 and 3, so a durable subset of sectors 1–2 alone counts here): blank candidate, FORMAT `Ok` again; some but not all non-zero sectors of copy A durable, or copy A otherwise invalid: S `Corrupt`, only INFO answered, no write, no format; after the owner re-zeroes the whole scope (2.6), a new instance mounts a blank candidate and FORMAT returns `Ok`; copy A valid (sectors 0 and 3 durable): mounted with one valid copy (sequence 1, zero entries) | S | host |

Every row except N19, N20 and N24 needs only the destination service and
existing primitives; N24 also needs a simulated disk with late completion and,
natively, the K4/K5 fence of 2.7. Each host test is paired with a mutation that disables its check and
must make the test fail, as for the archive and name-check tests *[main
practice]*.

## 7. Isolated disposable-QEMU verification plan

**Images.** All images are created fresh under `$NOS_BUILD_DIR/run/` by the
harness: the source volume image as today, and a destination image attached as
the second virtio-blk device at `00:06.0` [PR #396]. No host disk, no physical
device, and no image survives between gate runs except as recorded evidence.

**PCI slot conflict.** Every runner that provides virtio-rng, the entropy
source FORMAT needs, places it at `00:06.0` *[main: `tools/run.py`,
`tools/run_keyring.py`; B01-ENTROPY-AND-KEYS.md]*, and the [PR #396] volume
runner places the second virtio-blk disk at the same slot without virtio-rng.
A profile with both devices therefore needs one of them at another PCI slot.
Whether the kernel's device discovery accepts another slot has not been
checked; if it does not, this is a kernel or profile change under K4 (O4).
Until it is resolved, `dest-format` cannot draw a destination ID and the
stages that depend on it cannot run.

**Reuse.** `tools/run_volume.py` and the `PLAN` of `tools/test_volume.py`
*[main]*, the disk-1 attachment and stages `archive-disk-*` [PR #396],
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
| `dest-cut-append` | `dest-export` | `VOLUME:CUT:` marker after k writes, host SIGKILL |
| `dest-cut-seal` | `dest-export` | cut between barrier 1 and barrier 2 |
| `dest-cut-mirror` | `dest-export` | cut after barrier 2 completes, before barrier 3 completes (N12, third point) |
| `dest-after-cut` | each cut stage | cold boot; LIST and VERIFY per N11/N12, for `dest-cut-mirror` LIST = {E1, E2} and E2 verifies; oracle agrees |
| `dest-restore` | `dest-export-2` + blank disk 0 | verify then restore entry 2; byte-exact per the existing restore oracle |
| `dest-revoke` | `dest-export` | revoke mid-export (N6) |
| `dest-foreign`, `dest-full`, `dest-replay`, `dest-corrupt` | as N14, N15, N17, N7/N8 | as in the matrix |

**Evidence lines.** `DEST:CATALOG:SEQ=`, `DEST:ENTRIES=`, `DEST:NEXT_FREE=`,
`DEST:IO:READS|WRITES|FLUSHES`, `DEST:DENIALS=` and markers
`DEST:CASE:<stage>:PASS`. Only these markers plus the raw QEMU exit code 33
count as a pass, as for the existing gates.

**Stage-count pins.** Adding stages to `PLAN` changes `len(PLAN)`, which two
protected workflow values must equal (`.github/workflows/kvm-pilot.yml` and
`.github/workflows/volume-acceptance.yml`, checked by
`tests/test_ci_workflows.py`). That is the same lead-owned correction that
blocks PR #396. Destination stages must not be added before PR #396 and its pin
update land, and their own pin update is a lead-owned change (section 8, O6).

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
  under D2.
- **O2** Formatting: whether a non-blank device may ever be erased into a
  destination (proposed: no in v1; blank devices only).
- **O3** Retention: v1 has no deletion, so `Full` is terminal for a destination.
  Deleting or rotating old entries needs a decision, a catalog operation and its
  own crash tests.
- **O4** Approval of kernel changes K1–K5 as needed (K4 and K5 are required for
  any product profile; K1–K3 can wait).
- **O5** Time display: whether any wall-clock time is shown for an entry. The
  catalog stores none, and v1 object times are a per-domain logical clock,
  not wall-clock time ([S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md)).
- **O6** (lead-owned) workflow stage-count pins for new destination stages, or a
  separate gate.
- **O7** Whether the dialog may hold the lane issuing roots (section 4.3) or
  D4 (a) requires them in the portal. With the sender check of 4.3 the dialog's
  residual authority on an operation lane is to bind, transfer and revoke, not
  to perform an operation.
- **O8** Restart after a failed mount or remount FLUSH (2.6): who initiates
  the destination service restart (portal, dialog or a system service
  manager), through which mechanism, and how a quarantined block lane is
  reported as requiring a reboot. No restart mechanism is specified.
- **O9** Completion fence (2.7): whether a write that the device accepted
  before an acknowledged reset can still reach the backing medium after the
  reset and the remount FLUSH, for QEMU and its host storage stack, and
  whether the K4/K5 per-device broker implements the reset fence of the M3
  volume broker for disk 1. Both are *[unproven]*; N24 (b) records the
  outcome without counting it as a pass.
  Until both are shown, the 2.7 safe path applies (read-only for the rest of
  the boot after an `Unknown`; confirmation only after a cold boot and
  VERIFY), including for a restarted service instance in the same boot and
  across a reboot that is not a cold boot, by a mechanism not yet
  specified; its trigger covers entry writes as well as catalog writes. N24 is
  a planned test, not evidence; the independent security review (R1 of 8.2)
  also remains open before implementation.
- **O10** `Corrupt` after an interrupted FORMAT (2.6, N25): the policy by
  which a product distinguishes a FORMAT interrupted before its first copy
  was durable from a damaged existing destination, and whether any recovery
  other than external re-zeroing of a disposable image is offered. Not
  specified; until decided the service never formats a `Corrupt` scope.

### 8.2 Staged route

Each step is one reviewable draft PR; none activates a production boundary.

| Step | Content | Acceptance check |
|---|---|---|
| R1 | This specification; independent security review of it | review findings recorded and resolved in this file |
| R2 | Host crate for the service core: catalog codec, mount rule, state machine, wire codec, write path; simulated disk with cache loss | host tests for N3, N4, N8–N13, N15, N17, N21–N25 (simulated disk with late completion for N24), each with a killing mutation; Clippy, fmt, `cargo test --locked` per the CI workflow commands |
| R3 | `tools/destination_disk.py` independent oracle | Python unit tests: parses valid copies, refuses each invalid rule |
| R4 | Volume service read-only and create-only endpoint bindings (separate spec) | N19, N20 host tests; existing volume and archive host tests unchanged |
| R5 | After PR #396 merges and O4 approves K4/K5: per-device `Policy`, destination service binary in a test-only profile on disk 1, and a scripted test-only dialog driver for the control lane that sends FORMAT and BIND and transfers lane SEND to the portal (4.3) | native stages `dest-format`, `dest-export`, `dest-export-2`, `dest-restore`, each lane bound by BIND with the destination ID that INFO reports after `dest-format`; N1, N2, N16, N18 counted (the driver's handles as the dialog's in 5.1); the per-device `Policy` admits no request between a timeout and the acknowledged reset (2.7 completion fence), shown by a native timeout fixture on disk 1 |
| R6 | UNBIND, revocation and rebinding through the scripted test-only dialog driver of R5 | N5, N6, N14, N17 native; crash stages N11, N12 |
| R7 | Trusted dialog surface and remembered permission (after O1) | UI checks per `ui/AGENTS.md`; revocation of a remembered permission revokes live lanes |
| R8 | Independent security review of the implementation, then an owner activation decision | all matrix rows pass natively where marked QEMU; review closed |

Until R8, every grant remains test-only and S09 remains open; this document
changes no score.
