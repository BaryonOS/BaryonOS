# Volume post-wake remount check (H05 Q2, S side) (design specification)

**Status: design specification, revision 0 (26 November 2026). Nothing in this
document is implemented, native or active.** It specifies the S side of the
mandatory remount after an S3 wake (owner decision Q2 of 25 November 2026,
recorded in the H05-T5c amendment of `ADR-H05-S3-SYSTEM-20261123` in
`docs/DECISIONS.md`): the generation and identity check that the volume service
runs before it serves any request after a wake. H05 slice order puts this
remount (slice 3, with S) before the Q1 ring-3 block re-grant (slice 4); the
UNLOCK order is tracked in #564 and B's keyring LOCK/UNLOCK contract (#563).

Markers: *[main]* is a fact checked on `main` `38237f7d`; *[design]* is
specified here and not built; *[unproven]* is an assumption that no test in
this project shows.

## 1. Facts this design builds on

- Before SUSPEND the kernel requires closed storage *[main]*: every published
  block request completed successfully and the last one was a FLUSH, no lane
  is dirty, resetting or in error, the broker is quiescent, and no block
  object exists (`power_gate` and `sleep_quiescent` in
  `kernel/src/m2/m3/volume.rs`, `storage_closed` in
  `kernel/src/m2/m3/system.rs`). The durable table therefore equals the table
  the volume service last committed.
- At SHUTDOWN the volume service flushes and sets itself read-only
  (`services/volume/src/service.rs`, shutdown path) *[main]*.
- Recovery only reads: `Service::recover(epoch, read_only = true, disk)` runs
  `table::recover_with_state`, which scans both table slots, chooses the valid
  copy with the highest generation and decodes it; there is no journal, replay
  or repair, and a read-only instance issues no FLUSH *[main]*. A corrupt table
  returns `Corrupt` -18 and the instance does not start.
- Every table sector carries the generation, an attempt digest (FNV-64 over
  the payload of all sectors of that copy) and an FNV-64 over slot, sector
  index and payload (`services/volume/src/table.rs`) *[main]*. These detect
  accidental damage and torn writes, not deliberate forgery.
- After the wake, H re-validates the device: PCI identity, negotiated state
  and capacity (Q5), with a quarantine on any refusal (Q4) *[main]*. The
  volume check below adds the *content* identity that the kernel cannot see.

## 2. The check

1. **Reference.** The pre-sleep reference is the table the read-only volume
   service instance holds in RAM after SHUTDOWN: its generation *g*, the slot
   `g % 2`, the attempt digest *D* of that copy and the decoded entries
   *[design]*. The instance holds no block object during sleep, only RAM.
   That the same instance survives the sleep with its RAM intact is
   *[unproven]*: no native S3 fixture runs the volume service across a wake.
2. **Grant.** After H's re-init and the latch lift, the kernel issues one
   read-only block grant to the volume service (T5f-1, settled with H on #564):
   READ over the whole volume scope, because the table sectors are shared by
   all domains; the domain-1 limit for init's key-state read is enforced by
   the service, not the grant *[design]*.
3. **Remount.** The service runs the read-only recovery of section 1 into a
   second table value and compares it with the reference: the chosen slot,
   generation *g*, attempt digest *D* and every decoded entry must be equal
   *[design]*. Equality is checked before the service answers any request.
4. **Outcome.**
   - Equal: the service serves read-only requests within the service-enforced
     scope (init's domain-1 endpoint) until T5f-2 *[design]*.
   - Not equal, `Corrupt`, or a read error: the service refuses every request,
     serves nothing, writes nothing and reports one distinct post-wake
     refusal *[design]*. Its mapping to the kernel's lane quarantine (-24,
     separate from -13/-14) is agreed with H before any code is written.
     A refusal is never repaired before authentication; the next cold boot
     recovers as usual.
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
  different chunk contents is not detected at mount. Chunk content sums are
  checked per READ, also with FNV. A keyed MAC is B's open item O7.
- **No reference after a restart.** If the volume service instance does not
  survive the sleep, there is no pre-sleep reference in ring 3, and a remount
  can only be as strong as a cold boot. Whether the kernel should keep a copy
  of (*g*, *D*) reported at SHUTDOWN is open question R1 below.
- No physical S3, no IOMMU and no DMA-isolation claim, as for H05-T5c.

## 4. Open questions

- **R1** (S with H): does the kernel keep the (*g*, *D*) pair reported by the
  volume service at SHUTDOWN, so that a restarted instance can still be
  checked? The value is only as trustworthy as the reporting service.
- **R2** (S with H): the post-wake refusal code and its mapping to -24.

## 5. Negative tests (planned)

| ID | Input | Expected |
|---|---|---|
| PW1 | wake, unchanged medium | remount equal; read-only requests served |
| PW2 | table copy of generation *g* changed during sleep (labelled injection) | refusal; no request served; no write |
| PW3 | both copies replaced by a valid table of generation *g* + 1 | refusal (generation differs) |
| PW4 | read error during the remount | refusal; no retry, no repair |
| PW5 | write request before T5f-2 | refused; no device write |
| PW6 | remount with keys absent | succeeds (no key dependency) |
