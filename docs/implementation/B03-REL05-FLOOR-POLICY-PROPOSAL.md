# B03 REL-05: proposal for a health-confirmed anti-rollback floor

**Status:** proposal for a project-owner decision. Nothing here is implemented,
and no code, ADR or acceptance changes with this document. It amends
ADR-049 point 2 only if the project owner accepts it in a new ADR.

## Problem (characterized, not hypothetical)

ADR-049 point 2 stores "the highest started version" in the UEFI NV variable
`NieuwOsMinVersion`. `boot/loader/src/signed.rs` `select()` calls
`raise_floor` as soon as the selected slot's hybrid signature, digest and ELF
load succeed: before `BOOT:SIGNED:OK`, before ExitBootServices and before the
kernel has run. A correctly signed release that then crashes leaves the floor at
its version, and the previous known-good slot is refused as `DOWNGRADE` on
every later boot.

Evidence: [B03-F1 characterization](../verification/b03-floor-crash-20261123/README.md)
(QEMU 10.2.1 at `a91d5ade`; repeated on 25 November 2026 with QEMU 8.2.2 on
main-merged source; that re-run's raw files are not committed, so it is a
supporting observation, not verifiable evidence). Audit finding REL-05 in
`docs/audits/subagents/08-release-supply-chain.md` asks (translated from Dutch) for "an explicit policy
for trial boot / health confirmation or authorised recovery above the floor,
with a negative native test and data preservation; do not silently disable
anti-rollback". The audit also bounds the goal: fallback to a *permitted*
previous version, not unlimited downgrade.

## Invariants any option must keep

1. Only a slot that verifies with both pinned anchors and its digest is started
   (ADR-049 point 1). Unchanged.
2. The floor never decreases.
3. After a version has been confirmed healthy, no lower version starts again.
4. A disk-only attacker (who can rewrite the unsigned slot header and store but
   not OVMF_VARS) gains no older version than today's design allows.
5. No unsigned ESP fallback; no new trust root or key.

## Options

| | A. Trial boot with deferred floor | B. Signed recovery permit above the floor | C. Keep ADR-049, accept the risk |
| --- | --- | --- | --- |
| Idea | The loader records a *pending* version instead of raising the floor. The floor rises on a later boot, after the kernel has confirmed health for that version. Bounded boot attempts; then fall back to the other slot. | A separately signed manifest type allows one boot of a version below the floor. | Document REL-05 as an accepted risk. |
| Recovers a crashing release | Yes, automatically, to the previous slot. | Only after an operator obtains a signed permit. | No. |
| New key or ceremony | No. | Yes (permit signing, revocation, replay limits). | No. |
| Rollback window | Between "new version selected" and "health confirmed": the previous slot (≥ the old floor) may still start. That equals today's A/B guarantee before an update. | None without a permit. | None. |
| Main risk | Health confirmation must be tied to the exact version and must not be forgeable into a *lowering*. | Key management and permit replay. | Bricking by a signed but broken release. |

### Option A sketch

- New NV+BS variable `NieuwOsPendingVersion` (version, attempt counter). The
  loader, before ExitBootServices, sets it when it selects a version above the
  floor and increments the counter on each start of that version.
- The kernel cannot write BS-only variables after ExitBootServices. The health
  signal therefore travels through the slot store: after a defined health point
  (for example the supervisor reaching its steady state), the update service
  writes a "healthy V" record to the store header area.
- On the next boot the loader raises the floor to V only if the pending
  version is V *and* the store carries "healthy V". It then clears the pending
  variable.
- If the counter exceeds a bound (proposal: 3) without "healthy V", the loader
  marks that slot failed for this pending cycle and selects the other slot.
  The floor is untouched, so the previous version is still permitted.
- Design analysis (unverified): a forged "healthy V" (disk attacker) can only
  raise the floor early, which is today's behaviour. Withholding confirmation
  keeps the old floor, so only versions ≥ the old floor remain startable
  (invariant 4).
- Known window: after the kernel writes "healthy V" to the unsigned store but
  before the next boot raises the floor, a disk attacker can delete the record.
  The attempt bound then returns to the previous slot, which violates
  invariant 3 for that window (invariant 4 still holds). An implementation must
  close or explicitly accept this window, for example by raising the floor in
  the same boot through an authenticated health call.

### Tests the implementation would need (native, QEMU)

Crashing v2 → automatic return to v1 with data preserved; healthy v2 → floor
raised on the following boot and v1 then refused; forged confirmation for a
different version is ignored; attempt bound reached → fallback; interruption
between each write (pending set, health record, floor raise, pending clear);
repetition of B03-F1 turned into a passing regression test.

## Decision requested from the project owner

1. Choose A, B or C. The B workstream's preference is A, because it is the only
   option with automatic recovery; B fits only if automatic recovery is not
   wanted.
2. If A: confirm the attempt bound and the health point that may confirm a
   version.

Until a decision exists, the B workstream makes no change to the loader's floor semantics.
