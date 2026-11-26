# Volume post-wake remount check (H05 Q2, S side) (design specification)

**Status: design specification, revision 1 (26 November 2026). Apart from the
`Quarantined` -24 pass-through in `services/volume` (section 2, step 4),
nothing in this document is implemented, native or active.** It specifies the
S side of the mandatory remount after an S3 wake (owner decision Q2 of
25 November 2026, recorded in the H05-T5c amendment of
`ADR-H05-S3-SYSTEM-20261123` in `docs/DECISIONS.md`): the generation and
identity check that the volume service runs before it serves any request after
a wake. H05 slice order puts this remount (slice 3, with S) before the Q1
ring-3 block re-grant (slice 4); the UNLOCK order is tracked in #564 and B's
keyring LOCK/UNLOCK contract (#563).

Revision 1 records the answers to R1 and R2 proposed by H on #564 and fixes
the four review points of revision 0 (#580): the closed-storage condition, an
explicit empty-volume reference, the chunk sum check and the post-wake chunk
cache. The R1 and R2 answers are not binding until the ADR amendment.

Markers: *[main]* is a fact checked on `main` `8033e2dd`; *[design]* is
specified here and not built; *[unproven]* is an assumption that no test in
this project shows.

## 1. Facts this design builds on

- Before SUSPEND the kernel requires closed storage *[main]*: the completion
  counts of every lane match its published requests, no lane is dirty,
  resetting or in error, the broker is quiescent, and no block object exists
  (`power_gate` and `sleep_quiescent` in `kernel/src/m2/m3/volume.rs`,
  `storage_closed` in `kernel/src/m2/m3/system.rs`). The kernel does not
  require the last request to be a FLUSH.
- At SHUTDOWN the volume service flushes and sets itself read-only
  (`services/volume/src/service.rs`, shutdown path) *[main]*. With closed
  storage, the durable table therefore equals the table the volume service
  last committed.
- Recovery only reads: `Service::recover(epoch, read_only = true, disk)` runs
  `table::recover_with_state`, which scans both table slots, chooses the valid
  copy with the highest generation and decodes it; there is no journal, replay
  or repair, and a read-only instance issues no FLUSH *[main]*. A corrupt table
  returns `Corrupt` -18 and the instance does not start. An all-zero volume
  recovers as the empty generation-0 state with no selected slot and no
  attempt digest; nothing is formatted *[main]*.
- Every table sector carries the generation, an attempt digest (FNV-64 over
  the payload of all sectors of that copy) and an FNV-64 over slot, sector
  index and payload (`services/volume/src/table.rs`) *[main]*. These detect
  accidental damage and torn writes, not deliberate forgery.
- Chunk content sums (FNV-64) are checked when a chunk is loaded from the
  medium into the service's single-chunk cache; reads served from a valid
  cache entry are not re-read or re-checked (`services/volume/src/service.rs`,
  chunk cache) *[main]*. SHUTDOWN leaves the cache in place.
- After the wake, H re-validates the device: PCI identity, negotiated state
  and capacity (Q5), with a quarantine on any refusal (Q4) *[main]*. The
  volume check below adds the *content* identity that the kernel cannot see.

## 2. The check

1. **Reference.** The pre-sleep reference is the table the read-only volume
   service instance holds in RAM after SHUTDOWN *[design]*. For a committed
   volume it is the generation *g*, the slot `g % 2`, the attempt digest *D*
   of that copy and the decoded entries. For an empty volume it is the
   explicit empty reference: generation 0, no selected slot, no attempt digest
   and no entries. The instance holds no block object during sleep, only RAM.
   That the same instance survives the sleep with its RAM intact is
   *[unproven]*: no native S3 fixture runs the volume service across a wake.
2. **Grant (R1).** After H's re-init and the latch lift, the kernel issues one
   read-only block grant (T5f-1, settled with H on #564): READ over the whole
   volume scope, because the table sectors are shared by all domains; the
   domain-1 limit for init's key-state read is enforced by the service, not
   the grant *[design]*. The kernel issues this grant only to the volume
   service instance that ran SHUTDOWN before SUSPEND: the same slot and pid,
   with no exit or restart since. A restarted instance gets no grant and no
   remount until the next cold boot. The kernel keeps no (*g*, *D*) pair
   reported by the service: it could not verify such a value, which would add
   a ring-3 value to kernel state, and FNV is not a MAC (B's O7). The grant
   check is H's T5f-1 kernel path, reviewed by S *[design]*.
3. **Remount.** The service runs the read-only recovery of section 1 into a
   second table value and compares it with the reference *[design]*. A
   committed reference is equal only to a remount that chose the same slot,
   generation *g*, attempt digest *D* and every decoded entry. The empty
   reference is equal only to a remount that again recovered the empty
   generation-0 state; a committed table on a volume that was empty before
   sleep, or the reverse, is not equal. Equality is checked before the service
   answers any request. Before the first READ after the wake the service
   invalidates its chunk cache, so every post-wake read loads its chunk from
   the medium and checks the chunk sum.
4. **Outcome (R2).**
   - Equal: the service serves read-only requests within the service-enforced
     scope (init's domain-1 endpoint) until T5f-2 *[design]*.
   - Not equal, `Corrupt`, or a read error during the remount: the service
     refuses every request with `RemountRefused`, serves nothing and writes or
     repairs nothing *[design]*. `RemountRefused` is a service status of its
     own, latched in the service and terminal until reboot; it needs no
     kernel change. Its value is fixed in the ADR amendment; -26 is proposed
     (free on `main` at this revision; -25 is H's KL2 SUSPEND code).
   - A kernel lane quarantine after H's device re-validation (Q4) reaches the
     service as -24 and is passed through one-to-one as `Status::Quarantined`
     -24, terminal until reboot and never retried. `Status::from_code` had no
     -24 arm, so the quarantine reached clients as `Protocol` -15; the arm
     and the pass-through on the certain-abort device paths are added with
     this revision.
   - Clients and the portal handle both `Quarantined` and `RemountRefused`
     fail-closed but report them separately: the two codes state different
     facts (the device was refused by the kernel; the content differs from
     the reference). Neither is repaired before authentication; the next cold
     boot recovers as usual.
5. **Writable again (T5f-2).** Writes come back only through a new writable
   instance: recovery runs again, then the barrier FLUSH, as on every boot
   *[design]*. No step writes before UNLOCK.

The check needs no Key Service keys (B's O4), so the order is: H re-init and
latch lift → read-only grant and this remount → init reads the key-state blob
through the volume service → RESTORE_PERSISTENT and UNLOCK → writable
re-grant.

## 3. Limits

- **Integrity, not authenticity.** An attacker with write access to the medium
  during sleep can build a different table with valid FNV values; the check
  detects such a change only because the generation, digest or entries differ
  from the RAM reference, and an exact replay of the reference table with
  different chunk contents is not detected at mount. Chunk sums are checked
  when a chunk is loaded from the medium, also with FNV, so a chunk changed
  together with a recomputed sum is not detected either. A keyed MAC is B's
  open item O7.
- **No remount after a restart.** If the volume service instance does not
  survive the sleep, there is no pre-sleep reference in ring 3. By R1 such an
  instance gets no grant, so the volume stays unavailable until the next cold
  boot; the design never treats a restarted instance as checked.
- No physical S3, no IOMMU and no DMA-isolation claim, as for H05-T5c.

## 4. Answered and open questions

- **R1 answered (proposal by H, #564):** no kernel copy of (*g*, *D*); the
  T5f-1 grant goes only to the same instance (section 2, step 2). Owed: one
  native case in which the same instance survives the wake and one negative
  case in which a restarted instance is refused the grant (PW7), both after
  T5d-T, T5e and UNLOCK.
- **R2 answered (proposal by H, #564):** `Quarantined` -24 is the one-to-one
  pass-through of the kernel lane quarantine; a content mismatch is the
  separate `RemountRefused` (section 2, step 4).
- **Open:** the value of `RemountRefused` and the binding text of R1 and R2,
  both in the ADR amendment. `RemountRefused` is not built before then.

## 5. Negative tests (planned)

| ID | Input | Expected |
|---|---|---|
| PW1 | wake, unchanged medium | remount equal; read-only requests served |
| PW2 | table copy of generation *g* changed during sleep (labelled injection) | `RemountRefused`; no request served; no write |
| PW3 | both copies replaced by a valid table of generation *g* + 1 | `RemountRefused` (generation differs) |
| PW4 | read error during the remount | `RemountRefused`; no retry, no repair |
| PW5 | write request before T5f-2 | refused; no device write |
| PW6 | remount with keys absent | succeeds (no key dependency) |
| PW7 | volume service restarted between SHUTDOWN and wake | no grant; no remount; no request served |
| PW8 | empty volume, unchanged medium | remount equal to the empty reference |
| PW9 | empty volume before sleep, valid committed table written during sleep | `RemountRefused` |
| PW10 | cached chunk changed on the medium during sleep without a matching sum (labelled injection) | first post-wake READ of that chunk fails `Corrupt`; no stale bytes served |
| PW11 | kernel lane quarantine after re-validation (Q4) | clients see `Quarantined` -24, distinct from `RemountRefused` |
