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
`s == c` → current; `s == c + 1` → committed-not-advanced (rule b below);
anything else → stale or forged → fail closed.

**Chosen protocol — commit then advance** (N04, 25 November 2026;
ADR-NET-ISSUER-20261125). Reserve-first was rejected: every ordinary crash would
leave the store looking rolled back and force a fail-closed denial, without a
security gain over rules (a)+(b).

| Step | Action | Crash after this step |
|---|---|---|
| A1 | Write store with `s = c + 1`; flush; read back | Store `c+1`, anchor `c`: roll forward at the next boot (rule b). |
| A2 | `advance(expected = c)` → `c + 1` | Consistent. |

Rules:

- **(a) Acknowledge after advance.** A grant, subgrant, revocation or durable
  denial is effective, and is reported to its caller, only after A2 succeeds.
  A store commit (A1) alone is never an acknowledgement.
- **(b) Startup check** with store generation `S` (authenticated by N04) and
  anchor generation `A`:
  - `S == A` → accept;
  - `S == A + 1` → roll forward: the entry at `S` must carry a valid
    authenticator; the policy process calls `advance(expected = A)` and then
    accepts (the entry was never acknowledged, so nothing is exposed);
  - `S < A` → rollback: reject, fail closed;
  - `S > A + 1` → impossible under this protocol: reject, fail closed.
- **(c) Crash window between A1 and A2.** A disk attacker who replays the
  previous authentic store (`S = A`) gets it accepted. This is harmless: the
  lost `A + 1` entry was never acknowledged (rule a), so no state the user saw
  is rolled back.
- **(d) Monotonic compare-and-swap.** `advance` succeeds only if the anchor
  still holds the expected old value (`A → A + 1`); of two concurrent advances
  at most one succeeds. The policy process never writes a second, different
  store at `S = A + 1` before reconciling.

## Limits

The anchor inherits the REL-05 threat model: a cleared or replaced TPM yields
`Unavailable` (fail closed); a host administrator who snapshots and restores
swtpm state, or a boot-path attacker before B04, defeats it. A counter
establishes freshness of one store only; it does not authenticate the store,
bind keys or replace N04's own grant-generation checks within a boot.
