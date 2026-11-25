# Cold-freshness anchor — B-owned contract for N04

**Status:** contract with a host model, not implemented natively and not
tested natively (task B03-REL05-D1, 25 November 2026; M1 revision of the same
day). The project owner approved option **5B** (N04 may use this anchor) in
ADR-NET-ISSUER-20261125, Decision 5B, under nine binding conditions, and
decision **M1** (protected anchor slot via B: B owns the slot, its policy,
its NV names and its lifecycle; no other trust-root change; no direct TPM
access for N; native verification required). Condition 7 applies: N04 keeps
option **5A** (no cold reinstatement) until the joint native tests of B and N
pass (condition 9).

This revision replaces the earlier counter-plus-slot design (counter
`TPM_NT_COUNTER`, 64-byte slot `NOSANC01`, `Pending` read): a hostile slot
rewrite could revive an older authentic store at the same generation (review
M1, confirmed by N's red host tests). The slot is now one hash chain per
domain, agreed with N on 25 November 2026. Three points of this revision are
**B proposals awaiting N's confirmation**: (P1) `extend` takes `D(S)` and the
kernel derives `E` (the handoff had `extend(expected_chain, E)`); (P2) rule
(d) is no longer needed for safety; (P3) a stray extend is permanent denial
with all durable grants lost, and same-TPM recovery needs a fresh instance.

| Artefact | Location | Kind |
|---|---|---|
| Pure functions `extend_event`, `chain_step`, `chain_after`, `instance`, `genesis_event`, `genesis_chain` | `crates/update-core/src/anchor.rs` | host code, `no_std` |
| NV constants (`ATTR_ANCHOR_CHAIN`, `anchor_domain`, `TPM_NT_EXTEND`) | `crates/update-core/src/trial_tpm.rs` | S2 constants |
| Host model of the chain against N's attack sequences | `crates/update-core/tests/anchor_chain_model.rs` | host model, not evidence of TPM behaviour |
| N04 consumer | `services/net-broker/src/durable.rs` (N branch) | N's host prototype; N moves it to this contract |

## Domains and ownership

One shared TPM 2.0 infrastructure, owned by workstream B, with logically
separate **anchor domains** (conditions 1 and 2). A domain is one NV index
reachable only through a kernel capability for that domain:

| Domain | Id | Handle | Holder |
|---|---|---|---|
| `AnchorDomain::Network` | 0 | `0x0100B310` | N04 policy process only |
| reserved (storage) | 1 | `0x0100B311` | none until a separate decision |
| reserved | 2–7 | `0x0100B310 + d` | none |

`0x0100B318`–`0x0100B31F` are unused. The update domain does **not** use this
interface: it keeps its own PCR 13-bound records `TRIAL`, `CONFIRM`, `FLOOR`
and `GEN` at `0x0100B300`–`0x0100B303` (REL-05 specification section 6.1).
No handle overlaps; the whole range `0x0100B300`–`0x0100B31F` is a TEST
candidate against the TCG handle registry *[unverified, U9]*. The kernel
exposes only `AnchorDomain::Network` in this step; a capability for one
domain reaches no other domain's index and no other TPM command (condition 3).

## Backend (TEST profile)

Disposable swtpm, as in [B03-REL05-TRIAL-BOOT-TPM.md](B03-REL05-TRIAL-BOOT-TPM.md).
Per domain one **`TPM_NT_EXTEND`** index: `nameAlg` SHA-256, size 32,
attributes `TPM_NT_EXTEND | AUTHWRITE | AUTHREAD | NO_DA` = `0x02040044`,
empty `authValue` and `authPolicy`; not `ORDERLY` (an orderly index may lose
updates on a non-orderly shutdown), `WRITEDEFINE`, `POLICY_DELETE`,
`PLATFORMCREATE` or any `*_STCLEAR` attribute. The public area is checked
against `anchor_domain(d)` on the first use per boot; a counter, ordinary or
platform-created index squatting the handle fails the check.

The NV Name hashes the current attributes, so it changes once the TPM sets
`WRITTEN` at the genesis extend. Both forms are frozen in
`tests/trial_tpm_constants.rs` for domain 0:

| Form | Name |
|---|---|
| before genesis | `000b4626434643c485b98e9e38ab863cccd989faa890fe6067f7fd820ca3a2c84db9` |
| after genesis (`WRITTEN`) | `000b442b22e4b54d200b29adf9ce099a389754078300340bdcb3764d0b51608d1f6d` |

`NV_Extend(data)` sets the index to `SHA-256(old ‖ data)` (TPM 2.0 Part 3,
31.7). The value cannot be set, only extended. Write authority is still not
phase-bound: anyone who can submit TPM commands (a compromised kernel, T3; a
host administrator with access to the swtpm state) can extend. Protection
against revival comes from the chain, not from write authority (see Threat
model).

### Chain encoding

All preimage fields have fixed lengths; `dom` is the domain id as one byte.

| Value | Definition | Preimage length |
|---|---|---|
| `instance` | `SHA-256("BARYON-ANCHOR-INSTANCE-V1" ‖ ENROLL digest ‖ dom ‖ nonce)` | 25 + 32 + 1 + 32 |
| genesis event | `SHA-256("BARYON-ANCHOR-GENESIS-V1" ‖ dom ‖ instance)` | 24 + 1 + 32 |
| genesis chain | `SHA-256(0³² ‖ genesis event)` (first extend of a new index, *[unverified, U21]*) | 64 |
| `E(S)` | `SHA-256("BARYON-ANCHOR-EXT-V1" ‖ dom ‖ D(S))` | 20 + 1 + 32 |
| chain step | `SHA-256(prev ‖ E(S))` | 64 |

The ENROLL digest and the 32-byte provisioning nonce come from the signed
enrolment or recovery record (REL-05 section 10.5, C665) and reach the
kernel through BootInfo. **The kernel derives `E` from the capability's
domain and the caller's `D(S)`**; the caller never supplies `E`, so a domain
holder cannot extend on behalf of another domain (proposal P1).

Golden vectors (Rust test `golden_vectors`, recomputed independently with
Python `hashlib`; N's fake and the S5 swtpm run must produce the same bytes):
ENROLL digest `11³²`, nonce `22³²`, domain 0, `D(S) = 33³²`.

| Value | Hex |
|---|---|
| `instance` | `bfe1bb14b6e07bf6c176d752ebaa1425c915d3b9e54b567fadbe2d3bd6b15a71` |
| genesis event | `695716ce7f9f7ca92945c51f3425cfbd4d88ed4976dad3a09a1717c015247cf9` |
| genesis chain | `f5f47a2563aae97e4a3b116dbdf449bb78b89889475fa9a2c6995f9183f2261f` |
| `E(S)` | `a0e8380d73d61df74dc648499898091350f03a83e59de072a234188679bb011c` |
| chain after `S` on genesis | `ba929ee819c99d8f0fb01bcec7e0ff999f4e8b3d651079ad127add6b75cd7d96` |

Store authenticity (grants and tombstones, condition 4) is N04's
responsibility: the anchor binds a digest; it does not authenticate or
encrypt the store.

## Interface

```text
read()                         -> Ok(chain) | Err(AnchorError)
extend(expected_chain, D(S))   -> Ok(())    | Err(AnchorError)
instance()                     -> Ok(instance) | Err(AnchorError)
```

There is no `Pending` state and no generation number: one `NV_Extend` is the
only anchor write of a commit.

`extend(expected, D)` is a compare-and-extend. **The kernel serialises it per
domain (review C87)**, so no second call on the same domain runs between its
steps:

1. Read the chain `c`. `c != expected` → `Conflict` (no TPM write).
2. Budget spent → `RateLimited` (no TPM write).
3. `NV_Extend(E(D))` *[atomic across power loss, unverified, U22]*.
4. Read back; value `!= SHA-256(expected ‖ E(D))` → `Corrupt`.

A lost or failed response is **never retried** inside the call or blindly by
the caller: an extend is irreversible, so a second one would move the chain
past every store (permanent denial, see below). The caller's next `read()`
decides; a retry through `extend(expected, D)` turns into `Conflict` because
step 1 sees the new value.

### Errors

None of them is ever an acceptance; N04 maps each to a denial (fail closed).

| Error | Produced by the kernel anchor service when |
|---|---|
| `Unavailable` | the boot is `DEGRADED` (the kernel does not claim the CRB, REL-05 section 10.2); the TPM is absent, faulty or times out; the index is missing, never extended, or its public area differs from the constants (cleared, replaced or squatted TPM) |
| `Corrupt` | read-back mismatch after the extend |
| `Conflict` | the chain differs from `expected` |
| `RateLimited` | the per-domain extend budget of this boot is spent, or the TPM reports `TPM_RC_NV_RATE` |

`Exhausted` no longer exists: a hash chain has no upper bound.

**Write budget (condition 6).** Every commit costs exactly one NV write. The
kernel enforces a fixed per-domain budget of extends per boot (TEST value
fixed in S7 and recorded as a constant); N04 batches grant changes into one
store commit per acknowledgement.

## Commit protocol — commit then extend

The store carries `instance`, its digest `D(S)` and `prev`, the chain value
it was committed on top of.

| Step | Writer | Action |
|---|---|---|
| A1 | N04 | write store `S` with `prev = read()`; flush; read back |
| X | anchor | `extend(prev, D(S))` |

### Startup rule (N04, agreed 25 November 2026)

With `chain = read()`:

- `instance() != S.instance` → `Foreign` (replaced or re-provisioned TPM).
- `chain == SHA-256(S.prev ‖ E(S))` → accept.
- `chain == S.prev` → roll forward: `extend(S.prev, D(S))`, accept on
  success. Such a store was never acknowledged (rule a).
- anything else → one denial (`Stale`; it covers what were `Rollback`,
  `Fork` and `Ahead`, which the chain value cannot tell apart).
- any `Err(_)` → denial, 5A behaviour.

### Crash outcomes (every write prefix)

| Crash after | Store on disk | Chain | Next startup |
|---|---|---|---|
| nothing, or a torn A1 | old `A` (A1 must be atomic or rejected by N04's authenticator) | after `A` | accept `A` |
| A1 | new `S` | `S.prev` (= after `A`) | roll forward once, then accept |
| X | new `S` | after `S` | accept |

No ordinary crash leads to a denial, and a roll-forward completes only once:
a second `extend(S.prev, D(S))` returns `Conflict`.

### Rules

- **(a) Acknowledge after extend.** A grant, subgrant, revocation or durable
  denial is effective, and is reported to its caller, only after X succeeds.
  A1 alone is never an acknowledgement.
- **(c) Replay window.** After A1 and before X, a disk attacker who replays
  `A` gets it accepted. This is harmless: the lost entry was never
  acknowledged. The policy process may then commit a sibling `Y` on the same
  `prev`; once `Y` is acknowledged, the chain is past `S.prev` and the
  earlier sibling can never be accepted again (host model
  `hostile_extend_cannot_revive_a_stale_store_at_the_same_position`,
  `crashed_sibling_cannot_displace_a_rolled_forward_store`). The old rule (d)
  is no longer needed for safety (proposal P2).

## Provisioning and recovery (TEST only)

The TEST provisioning tool of REL-05 section 10.3 defines the index and
performs one extend with the genesis event. It is re-runnable after a cut
between the two steps: it extends only while the index reads as never
written (U21, U16), so it never revives a written index.

**Same-TPM recovery** (after a stray extend, proposal P3) is owner-authorised
`NV_UndefineSpace`, then provisioning with a **fresh instance**: every
genesis uses a new provisioning nonce from a new signed record, and the tool
refuses an instance that was used before. Re-genesis with the old instance
would let a disk attacker walk the old store history forward one boot at a
time and choose any branch, including an unacknowledged sibling (host model
`same_tpm_regenesis_after_a_stray_extend_needs_a_fresh_instance`). This uses
the same primitive as the undefine/redefine/replay residual below; only the
authorisation differs. N04's first store has `prev` = genesis chain.
**A missing store is never accepted (review L2, agreed with N):** startup
with no store is denial. Genesis is entered only through authorised
provisioning or recovery, never by the policy process on its own. On a
provisioned index no extend reproduces the genesis chain, because the chain
never returns to zero.

A cleared, replaced or rolled-back TPM yields `Unavailable` (index missing,
never extended, public area changed, or the REL-05 TPM identity check of
section 10.5 fails) or a store that no longer matches: fail closed, 5A
behaviour. Explicit recovery is the authenticated REL-05 recovery (owner
decision C665: signed recovery record naming the new TPM's EK and a new
provisioning nonce, hence a new `instance`) plus N04 rejecting its old store
(`Foreign`) and reinitialising at the new genesis. B carries no tombstone or
chain value across; old network grants are not restored and need explicit
re-confirmation (agreed with N, 25 November 2026). Torn-A1 recovery of the
previous store (review C143) is an N04 store requirement (N04-T3), not part
of this contract.

## Threat model

Agreed with N on 25 November 2026. Residual risks are stated literally; no
"denial only" claim is made beyond what the host model shows.

| Actor | Can | Result |
|---|---|---|
| Disk attacker (store replay) | present any authentic store it has seen | only the store the chain points to is accepted, plus the unacknowledged roll-forward of rule (c) |
| T3: compromised, correctly measured kernel | extend arbitrary data on any domain | **denial only, but permanent** (P3): the chain leaves every authentic store and the domain stays denied until authorised re-genesis with a fresh instance, with all durable grants lost. No authorisation policy stops this, and a PCR 13 phase policy would not either, because the kernel legitimately extends at run time |
| T3, today's TEST profile (empty ownerAuth) | `NV_UndefineSpace` + `NV_DefineSpace` + replay of the public genesis and extend events | **revival of an older store: NOT defended** (host model `undefine_redefine_replay_revives_with_empty_owner_auth`) |
| T3 with empty lockoutAuth | `TPM2_Clear` | owner indices disappear; the loader's TRIAL/FLOOR check halts the next boot (ANCHOR_LOST, REL-05) *[reachability unverified, U24]* |
| T5: foreign OS booted on the machine | the same as T3 | out of scope until B04 (measured boot); a PCR 13 phase policy would not stop it |
| swtpm host administrator | snapshot and restore the whole TPM state with a matching store | not defended (REL-05 threat model) |
| Physical attacker | — | not defended |

### Open: undefine/redefine/replay (OWNER decision required)

**Proposed mitigation, not decided:** the loader sets `ownerAuth` and
`lockoutAuth` to a secret sealed to PCR 13 = 0³² (loader phase only), so the
kernel phase cannot undefine or redefine an anchor index or clear the TPM.
The host model shows the sequence refused with protected `ownerAuth`
(`undefine_redefine_replay_refused_with_protected_owner_auth`). This changes
hierarchy authorisation, which M1 does not cover ("bounded slot-policy
changes … no other trust-root change"). It also needs new codecs (`Create`
and `Unseal` of a sealed object, `HierarchyChangeAuth`) and native proof, and
it rests on U23 and U24: an OS-usable platformAuth could delete the index
(denial) but not redefine it as an owner index.

**Alternative for N and the owner:** a per-domain NV counter alongside the
chain (a new counter starts at the TPM's highest-ever value, U6) would detect
a redefinition within slot scope, at the cost of a two-write commit again.

`OWNER:` may B change `ownerAuth`/`lockoutAuth` in the TEST profile for this
mitigation (loader-phase authorised recovery must then still be able to
undefine the index for re-genesis), or should the undefine/redefine/replay
revival be recorded as an accepted residual risk, or the counter alternative
be pursued?

Until then the claim "no revival by T3" is **false** in the TEST profile.

## Unverified TPM behaviour

Listed in REL-05 section 14; the host model assumes them, S5 must prove them.

- U21 The first `NV_Extend` of a new extend index starts from zeros; reading
  it before any extend fails.
- U22 One `NV_Extend` is atomic across power loss.
- U23 The OS can never use platformAuth after OVMF randomises it.
- U24 Whether platform authorisation can undefine an owner-created index, and
  whether `TPM2_Clear` with empty lockoutAuth is reachable by T3.

`NV_Extend` is not yet in the `tpm-crb` codec (S3 covers `NV_DefineSpace` of
an extend index only).

## Limits

The anchor inherits the REL-05 threat model: a host administrator who
snapshots and restores the whole swtpm state together with the store, or a
boot-path attacker before B04, defeats it. A chain establishes freshness of
one store per domain; it does not authenticate the store, bind keys or
replace N04's own grant-generation checks within a boot. Domain isolation is
enforced by the kernel capability, not by TPM authorisation: a compromised
kernel (T3) reaches every domain. There is no cloud, NAS or software fallback
(condition 8).

## Joint native tests (condition 9; planned, none executed)

Cold restart accept; roll forward once after a cut at A1; accept after a cut
at X; replayed older store refused; sibling store refused after an
acknowledgement; stray extend → denial; lost response → no double extend;
TPM absent, cleared, squatted or never provisioned → `Unavailable`; replaced
TPM → `Foreign` until authorised genesis; same-TPM re-genesis with a fresh
instance after a stray extend; `Conflict` on a concurrent extend;
spent budget → `RateLimited`; a network capability cannot reach the update
records or another domain; the golden vectors above on swtpm; the
undefine/redefine/replay sequence (expected revival in today's TEST profile,
refusal once the owner decision is implemented).
