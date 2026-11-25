# Cold-freshness anchor — B-owned contract for N04

**Status:** contract, not implemented natively and not tested natively (task
B03-REL05-D1, 25 November 2026). The project owner approved option **5B**
(N04 may use this anchor) in ADR-NET-ISSUER-20261125, Decision 5B, under nine
binding conditions. Condition 7 applies: N04 keeps option **5A** (no cold
reinstatement) until the joint native tests of B and N pass (condition 9). The
N04 host prototype that consumes this contract is `services/net-broker/src/durable.rs`
on the N branch (`e3bef450`, `5abe7175`); it is a host model, not evidence.

## Domains and ownership

One shared TPM 2.0 infrastructure, owned by workstream B, with logically
separate **anchor domains** (conditions 1 and 2). A domain is a pair of NV
indices reachable only through a kernel capability for that domain:

| Domain | Id | Counter handle | Slot handle | Holder |
|---|---|---|---|---|
| `AnchorDomain::Network` | 0 | `0x0100B310` | `0x0100B311` | N04 policy process only |
| reserved (storage) | 1 | `0x0100B312` | `0x0100B313` | none until a separate decision |
| reserved | 2–7 | `0x0100B310 + 2d` | `0x0100B311 + 2d` | none |

The update domain does **not** use this interface: it keeps its own
PCR 13-bound records `TRIAL`, `CONFIRM`, `FLOOR` and `GEN` at
`0x0100B300`–`0x0100B303` (REL-05 specification section 6.1). No handle
overlaps; the whole range `0x0100B300`–`0x0100B31F` is a TEST candidate
against the TCG handle registry *[unverified, U9]*. The kernel exposes only
`AnchorDomain::Network` in this step; a capability for one domain reaches no
other domain's handles and no other TPM command (condition 3).

## Backend (TEST profile)

Disposable swtpm, as in [B03-REL05-TRIAL-BOOT-TPM.md](B03-REL05-TRIAL-BOOT-TPM.md).
Both indices: `nameAlg` SHA-256, `AUTHWRITE` and `AUTHREAD` with empty
`authValue`, `NO_DA`; not `ORDERLY`, `WRITEDEFINE`, `POLICY_DELETE` or any
`*_STCLEAR` attribute. The public areas are checked against constants on the
first use per boot (S2 freezes them).

- **Counter** (`TPM_NT_COUNTER`, 8 bytes): the domain generation `A`.
- **Slot** (ordinary, 64 bytes, `WRITEALL`): magic `NOSANC01` (8) · domain
  id (1) · reserved zero (7) · `gen` u64 LE (8) · `binding` (32) · reserved
  zero (8).

**Write authority is not phase-bound.** Anyone who can submit TPM commands
with the empty `authValue` (a compromised kernel, T3; a host administrator
with access to the swtpm state) can increment the counter or rewrite the
slot. The consequences are bounded as follows, which is why no PCR policy is
used:

- An increment or a slot write with a foreign `gen` makes the next read
  `Corrupt`, or makes the authentic store look rolled back: denial only.
- A forged `binding` matches no authentic store unless the attacker also
  holds two authentic stores at the same generation. Rule (d) below forbids
  the policy process to create such a fork, so a forged binding yields denial.
- Neither write can make an older authentic store (`S < A`) acceptable,
  because the counter never decreases and N04 authenticates `S` in the store.

Store authenticity (grants and tombstones, condition 4) is N04's
responsibility: the anchor stores a digest; it does not authenticate or
encrypt the store.

## Interface

```text
read()                       -> Ok(Current { generation: A, binding })
                              | Ok(Pending { generation: A, next_binding })
                              | Err(AnchorError)
advance(expected, binding)   -> Ok(()) | Err(AnchorError)
```

`read()` reads the counter `c` and the slot:

- slot `gen == c` → `Current { generation: c, binding: slot.binding }`;
- slot `gen == c + 1` → `Pending { generation: c, next_binding: slot.binding }`;
- anything else, bad magic, wrong domain id or non-zero reserved bytes →
  `Err(Corrupt)`.

`advance(expected, binding)` is a monotonic compare-and-swap from `expected`
to `expected + 1` that stores `binding` with the new generation:

1. Read `c` and the slot. `c != expected` → `Conflict`. `c == u64::MAX` →
   `Exhausted`. Budget spent → `RateLimited` (no TPM write).
2. If the slot is `Pending` with `next_binding == binding`, skip step 3
   (completion of an interrupted advance). If it is `Pending` with another
   binding → `Conflict`.
3. **W1** write slot `{gen: c + 1, binding}`; read back; mismatch → `Corrupt`.
4. **W2** `NV_Increment(counter)`; read back; value `!= c + 1` → `Corrupt`.

A lost or failed response is never retried inside the call; the caller's next
`read()` decides.

### Errors

None of them is ever an acceptance; N04 maps each to a denial (fail closed).

| Error | Produced by the kernel anchor service when |
|---|---|
| `Unavailable` | the boot is `DEGRADED` (the kernel does not claim the CRB, REL-05 section 10.2); the TPM is absent, faulty or times out; an index is missing or its public area differs from the constants (cleared, replaced or squatted TPM) |
| `Corrupt` | slot magic, domain id, reserved bytes or `gen` inconsistent with the counter; read-back mismatch after W1 or W2 |
| `Conflict` | the counter differs from `expected`, or a `Pending` slot carries a different binding |
| `RateLimited` | the per-domain advance budget of this boot is spent, or the TPM reports `TPM_RC_NV_RATE` |
| `Exhausted` | the counter is at `u64::MAX`, so `expected + 1` does not exist |

**Write budget (condition 6).** Every advance costs at most two NV writes. The
kernel enforces a fixed per-domain budget of advances per boot (TEST value
fixed in S7 and recorded as a constant); N04 batches grant changes into one
store commit per acknowledgement. The TPM's own NV rate limiting maps to
`RateLimited`. A 64-bit counter is not exhausted by any realistic number of
increments; `Exhausted` exists so that the boundary fails closed
(condition 5).

## Commit protocol — commit then advance

Agreed in ADR-NET-ISSUER-20261125 Decision 5B. The store carries its own
authenticated generation `S`, digest `D(S)` and parent binding.

| Step | Writer | Action |
|---|---|---|
| A1 | N04 | write the store at `S = A + 1` with parent `binding(A)`; flush; read back |
| W1 | anchor | slot `{gen: A + 1, binding: D(A + 1)}` |
| W2 | anchor | counter `A → A + 1` |

### Startup rule (b)

With store generation `S` and the anchor read result:

- `Current(A, b)`, `S == A`: accept only if `D(S) == b` (rejects a
  same-generation fork); otherwise reject.
- `Current(A, b)`, `S == A + 1`: roll forward. The store at `S` must
  authenticate and its parent must equal `b`; N04 calls `advance(A, D(S))`
  and accepts on success. The entry was never acknowledged, so nothing the
  user saw is exposed.
- `Pending(A, n)`: only `S == A + 1` with `D(S) == n` may complete, through
  `advance(A, n)`. A store at `S == A` is **refused**: its binding was
  overwritten by W1.
- `S < A` (rollback) or `S > A + 1`: reject.
- Any `Err(_)`: reject and behave as 5A.

### Crash outcomes (every write prefix)

| Crash after | Store | Anchor read | Next startup |
|---|---|---|---|
| nothing, or a torn A1 | `A` (A1 must be atomic or rejected by N04's authenticator) | `Current(A, D(A))` | accept the old store |
| A1 | `A + 1` | `Current(A, D(A))` | roll forward (W1, W2) |
| W1 | `A + 1` | `Pending(A, D(A + 1))` | complete (W2) |
| W2 | `A + 1` | `Current(A + 1, D(A + 1))` | accept |

No ordinary crash leads to a denial. Reserve-first was rejected for that
reason: it would leave the store looking rolled back after every ordinary
crash.

### Rules

- **(a) Acknowledge after advance.** A grant, subgrant, revocation or durable
  denial is effective, and is reported to its caller, only after W2 succeeds.
  A1 alone is never an acknowledgement.
- **(c) Replay windows.** After A1 and before W1, a disk attacker who replays
  the previous authentic store (`S = A`) gets it accepted. This is harmless:
  the lost `A + 1` entry was never acknowledged (rule a). After W1 the same
  replay is refused (`Pending`), which is denial only.
- **(d) No second store at `A + 1`.** The policy process never writes a
  different store at `S = A + 1` before the pending advance is completed or
  the store is reinitialised under recovery.

## Provisioning and recovery (TEST only)

The TEST provisioning tool of REL-05 section 10.3 defines both indices of a
domain, increments the counter once (a never-incremented counter cannot be
read, U16; its first value is the TPM's highest-ever counter value, U6) and
writes the slot `{gen: c, binding: 0³²}`. The all-zero binding is the genesis
binding: N04's first store is written at `S = c + 1` with parent `0³²`.

A cleared, replaced or rolled-back TPM yields `Unavailable` (index missing or
public area changed) or a store that no longer matches (stale `S`, fork):
fail closed, 5A behaviour. Explicit recovery is re-provisioning plus N04
reinitialising its store at the new genesis; persisted grants are then only
suggestions for explicit re-confirmation. A production procedure (owner auth
custody, physical presence, EK binding U13) is not specified and needs an
owner decision.

## Limits

The anchor inherits the REL-05 threat model: a host administrator who
snapshots and restores the whole swtpm state together with the store, or a
boot-path attacker before B04, defeats it. A counter establishes freshness of
one store per domain; it does not authenticate the store, bind keys or
replace N04's own grant-generation checks within a boot. Domain isolation is
enforced by the kernel capability, not by TPM authorisation: a compromised
kernel (T3) reaches every domain. There is no cloud, NAS or software fallback
(condition 8).

## Joint native tests (condition 9; planned, none executed)

Cold restart accept; roll forward after a cut at A1; completion after a cut
at W1; replayed store after W1 refused; `S < A` refused; fork at `S == A`
refused; TPM absent, cleared or squatted → `Unavailable`; forged slot →
`Corrupt` or denial; `Conflict` on concurrent advance; spent budget →
`RateLimited`; a network capability cannot reach the update records or
another domain.
