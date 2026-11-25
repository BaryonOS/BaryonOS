# Cold-freshness anchor — B-owned contract for N04

**Status:** contract proposal only (task B03-REL05-D1, 25 November 2026). Not
implemented, not tested. Using this anchor for the N04 grant store is option
**5B** of [N04-ISSUER-ADR-PROPOSAL.md](N04-ISSUER-ADR-PROPOSAL.md) Decision 5,
which that document classifies as a new trust root; adoption requires an N04
decision by the project owner. Until then N04 option 5A (no cold
reinstatement) stays in force.

## Backend

The anchor shares the persistent backend selected for update rollback
resistance (ADR-B03-B04-TARGET D3) in the TEST profile of
[B03-REL05-TRIAL-BOOT-TPM.md](B03-REL05-TRIAL-BOOT-TPM.md): TPM 2.0 NV storage
on disposable swtpm. One store has exactly one NV counter (`TPM_NT_COUNTER`,
8 bytes) at a fixed handle from `0x0100B310` upward (handle range unverified,
U9 there). Attributes: `AUTHWRITE` and `AUTHREAD` with empty `authValue`,
`NO_DA`, not `ORDERLY`. An increment can only cause denial, never acceptance of
older state, so the write authority is deliberately not phase-bound. The
public area is checked against constants on first use per boot.

## Interface (requirements N1–N4)

| N04 need | Contract |
|---|---|
| N1 read at boot via an attenuated capability held only by the N04 policy process | The kernel grants `AnchorRead` and `AnchorAdvance` on one store handle to the N04 policy process only. `read() -> Ok(g)` or `Err(Unavailable)`. No other process, and no other store, is reachable through it. |
| N2 advance only after the store commit is durable, with defined crash behaviour | `advance(expected = g) -> Ok(g+1)`: the kernel reads the counter, refuses if it is not `g`, increments once, reads back. A lost response is never retried; the next `read()` decides. Ordering and crash table below. |
| N3 fail closed on a missing or corrupt anchor | Missing index, wrong public area, TPM absent/faulty, or a read error → `Unavailable`; the policy process then treats every persisted grant as non-authoritative (5A behaviour). |
| N4 one generation per store | One counter per store; values never repeat (counter initial value semantics unverified, see U6 of the REL-05 specification). The store records the counter value it was committed under. |

## Commit protocol and crash outcomes

The store carries its own authenticated generation field `s` (authentication
is N04's responsibility). Acceptance rule at boot with anchor value `c`:
`s == c` → current; `s == c + 1` → committed-not-advanced (below);
anything else → stale or forged → fail closed.

**Variant A — commit then advance (as requested by N):**

| Step | Action | Crash after this step |
|---|---|---|
| A1 | Write store with `s = c + 1`; flush; read back | Store `c+1`, anchor `c`: policy process calls `advance(c)` and then accepts. |
| A2 | `advance(c)` → `c + 1` | Consistent. |

Residual window: if a crash occurs between A1 and A2 **and** a disk attacker
then replaces the store with the previous authentic store (`s = c`), that store
is accepted. A revocation committed in A1 is lost. The policy process must also
never write a second, different store with `s = c + 1` before reconciling.

**Variant B — reserve then commit (recommended for revocations):**

| Step | Action | Crash after this step |
|---|---|---|
| B1 | `advance(c)` → `c + 1` | Store `s = c < c + 1` → fail closed (all grants denied, re-confirmation needed). No replay accepted. |
| B2 | Write store with `s = c + 1`; flush; read back | Consistent. |

Variant B has no replay window at the cost of a denial after a crash. The
choice between A and B (or B for revocation-bearing commits only) belongs to
N04.

## Limits

The anchor inherits the REL-05 threat model: a cleared or replaced TPM yields
`Unavailable` (fail closed); a host administrator who snapshots and restores
swtpm state, or a boot-path attacker before B04, defeats it. A counter
establishes freshness of one store only; it does not authenticate the store,
bind keys or replace N04's own grant-generation checks within a boot.
