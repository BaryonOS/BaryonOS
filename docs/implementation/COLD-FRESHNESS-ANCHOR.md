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
domain, agreed with N on 25 November 2026. Three points of this revision were
B proposals that **N confirmed on 25 November 2026**: (P1) `extend` takes `D(S)` and the
kernel derives `E` (the handoff had `extend(expected_chain, E)`); (P2) rule
(d) is no longer needed for safety; (P3) a stray extend is permanent denial
with all durable grants lost, and same-TPM recovery needs a fresh instance.

The undefine/redefine/replay revival found in this revision is addressed by
**M1-HAUTH** (project-owner decision of 25 November 2026, option 1, TEST
profile only; ADR-B03-REL05-TPM-20261125): the loader protects `ownerAuth` and
`lockoutAuth` with a secret readable only in the loader phase. Implemented so
far: codecs, constants, swtpm feasibility and squatting probes and the host
model. The loader TPM path does not exist yet; until it enforces M1-HAUTH
natively, the revival stays **not defended** and 5A stays in force.

| Artefact | Location | Kind |
|---|---|---|
| Pure functions `extend_event`, `chain_step`, `chain_after`, `instance`, `genesis_event`, `genesis_chain` | `crates/update-core/src/anchor.rs` | host code, `no_std` |
| NV constants (`ATTR_ANCHOR_CHAIN`, `anchor_domain`, `TPM_NT_EXTEND`) | `crates/update-core/src/trial_tpm.rs` | S2 constants |
| Host model of the chain against N's attack sequences, the M1-HAUTH loader provisioning, squatting, the accepted-pair store (U13c), clear and recovery | `crates/update-core/tests/anchor_chain_model.rs` | host model, not evidence of TPM behaviour |
| `HAUTH` constants (`NV_HAUTH`, `ATTR_HAUTH`, `POLICY_LOADER_READ`) | `crates/update-core/src/trial_tpm.rs` | S2 constants |
| `NV_Extend`, `NV_UndefineSpace`, `HierarchyChangeAuth`, policy `NV_Read` | `drivers/tpm-crb/src/codec.rs` | codecs with golden bytes |
| M1-HAUTH feasibility probe and transcript (secrets redacted) | `docs/verification/b03-rel05-hauth-probe-20261125/` | swtpm 0.10.1 without firmware; not native S5 |
| Squatting probe and transcript (secret redacted) | `docs/verification/b03-rel05-squat-probe-20261125/` | swtpm 0.10.1 without firmware; not native S5 |
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

### Hierarchy authorisation (M1-HAUTH)

The loader holds the only copy of the hierarchy secret outside the TPM:

| Item | Value |
|---|---|
| `HAUTH` index | `0x0100B304`, ordinary, 64 bytes = `ownerAuth ‖ lockoutAuth` (32 bytes each) |
| Attributes | `OWNERWRITE | POLICYREAD | WRITEALL | NO_DA` = `0x02081002`; no `AUTHREAD`, `OWNERREAD` or `POLICY_DELETE` |
| `authPolicy` | `PolicyPCR(13 = 0³²) + PolicyCommandCode(NV_Read)` = `339de543…ad82ae82` (`POLICY_LOADER_READ`) |
| Entropy | `EFI_RNG_PROTOCOL`, default algorithm (U25); absent → halt. Never derived from a record, nonce or chain value |
| Hygiene | the secret is never logged, never put in BootInfo or serial markers, and is wiped from loader memory before the PCR 13 extend; command and response buffers that carried it are wiped after use, and the policy `NV_Read` result is decoded into a caller buffer without `Copy` or `Debug` and wiped with volatile writes (loader slice; today `Response::Data` is `Copy + Debug` and must not carry the secret). Before the PCR 13 extend the loader overwrites the CRB data buffer up to the longest secret-bearing command or response length, because the kernel maps that buffer (native check planned) |
| Loader record | binds `SHA-256(HAUTH)` and names the anchored instance; owner index written only under `PolicyPCR(13 = 0³²) + CC_NV_Write`. **Open design item U26** (handle, size, per-domain layout, public-area check); the host model assumes it |
| Squatting | with ownerAuth empty (fresh or cleared TPM) anyone can define the `HAUTH` or record handle first (probe S1, S4). The owner cannot set `POLICY_DELETE` (S2, `TPM_RC_ATTRIBUTES`), so an owner squat is removed by an empty-ownerAuth undefine; a platform squat (U23 broken, S3) refuses owner undefine with `TPM_RC_NV_AUTHORIZATION` and survives `TPM2_Clear`. Before writing the secret the loader undefines both handles, tolerating only "not defined"; any other undefine or define failure halts (`NV_DEFINED` is never tolerated). On every boot it compares the Name of `HAUTH` with the frozen `NV_HAUTH` Names (both forms, `tests/trial_tpm_constants.rs`, observed in S1/N1) before using its contents, and it trusts the loader record only with its expected Name, because a squatted record is writable at run time (S4). Host model: `squatted_hauth_never_receives_the_secret`, `forged_record_after_a_platform_clear_is_never_adopted`; loader slice open |

Loader provisioning order (TEST profile, re-run from the start after any
cut): while ownerAuth is still empty, remove leftovers and owner squats of
`HAUTH` and the loader record, define and write `HAUTH`, define and write the
loader record binding it, `HierarchyChangeAuth(owner)`,
`HierarchyChangeAuth(lockout)`, then the anchor genesis (undefine, define,
genesis extend), the loader record naming the accepted pair (TPM copy, U13c),
the UEFI copy of that pair (ten counted writes on a fresh TPM, where the
undefines find nothing), then the PCR 13 extend. The loader never adopts a `HAUTH` that no trusted record binds (for
example one provisioned by an attacker after a clear): without a newer record
it halts; with one it clears with that `HAUTH`'s lockout secret, or with an
empty lockoutAuth when `lockoutAuthSet` is 0, and provisions new secrets; if
that clear fails it halts until a firmware-mediated clear (host model
`every_recovery_cut_converges`). The provisioning state is read from
`TPM_PT_PERMANENT` (`ownerAuthSet`, `lockoutAuthSet`), never by trying an
authorisation: a wrong `lockoutAuth` counts toward the lockout hierarchy's
dictionary-attack protection. `HAUTH` is never written once `ownerAuthSet` is
1. The model loader is generic over the TPM command set, so it sees only
permanent flags, public areas and policy reads, never an authValue, and no
honest path tries and fails a lockoutAuth.
`endorsementAuth` stays empty and is not changed by B (outside the approved
scope); a runtime attacker can set it and so block the EK check of
section 10.5, which is denial (`BOOT:FAIL:TPM_IDENTITY`), not revival.

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
holder cannot extend on behalf of another domain (P1, agreed with N).

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
| nothing, or a torn A1 | old `A` (requirement on N04: A1 replaces the store atomically or keeps the previous authenticated copy recoverable; rejecting the torn copy alone does not restore `A`) | after `A` | accept `A`; if no authentic store survives, denial, never acceptance |
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
  is no longer needed for safety (P2, agreed with N; N still writes no second store at one position, as hygiene).

## Provisioning and recovery (TEST only)

Under M1-HAUTH the loader, not the host tool of REL-05 section 10.3,
defines the index and performs the genesis extend, because only the loader
phase can authorise owner commands. Genesis is allowed only for a record
whose serial is above the accepted pair (U13c rules of REL-05 section 10.5;
mechanism open); the current record on a restart authorises none. The
accepted pair advances only after the genesis of every domain the record
names: first the TPM copy, in the same policy write of the loader record
(PCR 13 = 0³²) that names the anchored instance, so a runtime attacker cannot
forge "already anchored"; then the UEFI copy. The effective pair is the
surviving copy with the higher serial, and two copies with one serial but
different records halt. REL-05 section 10.5 orders a record replacement as
TPM copy, UEFI copy, new ESP record; the anchor genesis is part of accepting
the record and precedes that sequence, because the TPM copy is the same write
that marks the genesis done. A cut before it re-runs the genesis under the
still-fresh record; a cut after it leaves the UEFI copy and the ESP record to
follow as in section 10.5. For a first enrolment the record is already on
the ESP, so every cut converges on the next boot (host model
`every_provisioning_cut_converges`, which also fixes the number of writes).
For a record replacement, section 10.5 writes the ESP record last and does
not say where the new record resides before that write. The host model
presents the new record again after every cut and does not model the ESP
write: under that assumption every cut converges
(`every_recovery_cut_converges`); when only the old ESP record is presented
after the TPM copy has advanced, the boot halts (`Halt::NeedsRecovery`,
section 10.5: denial, never acceptance of the older record). **Open U13c
item:** the location and durability of a new record between its arrival and
the ESP write.
Losing one copy keeps older records refused, and the loader rewrites a lost
UEFI copy. Losing both (a TPM clear with U23 broken or by firmware, plus a UEFI
NVRAM reset) lets the old `ENROLL` authorise a second genesis and revives that
instance's first store: residual `accepted_pair_survives_one_lost_copy`, not
accepted and reported to the project owner. A TPM clear keeps the endorsement
primary seed, so the old `ENROLL` still proves the identity (the anchor model
does not model the EK). This residual is not the "rolling back both copies
together with the TPM state and the disk" item of section 10.5 (T7), which
needs a restored snapshot: this one needs only deletion, and section 10.5
does not define a boot with both copies absent, which looks like a first
enrolment. **Open U13c item** against C665 and C28.

**Same-TPM recovery** (after a stray extend, P3, agreed with N) is
`NV_UndefineSpace` authorised by the loader-phase ownerAuth, then provisioning with a **fresh instance**: every
genesis uses a new provisioning nonce from a new signed record, and the tool
refuses an instance that was used before. Re-genesis with the old instance
would let a disk attacker walk the old store history forward one boot at a
time and choose any branch, including an unacknowledged sibling (host model
`same_tpm_regenesis_after_a_stray_extend_needs_a_fresh_instance`). Recovery
keeps the hierarchy secret and refuses an older record or one that reuses
the accepted serial (host model
`authorised_recovery_needs_a_fresh_record`), so it cannot revive revoked
grants or lower anti-rollback state. N04's first store has `prev` = genesis chain.
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
previous store (review C143) is a binding requirement of this contract on
the N04 store (agreed with N, 25 November 2026; implemented by N in N04-T3,
#529, with a host test that cuts the write at every byte boundary): A1 is an
atomic replacement or retains a recoverable authenticated copy of the
previous store, so a torn A1 yields the crash outcome above; a torn A1 with
no recoverable copy is denial, never acceptance.

## Threat model

Agreed with N on 25 November 2026. Residual risks are stated literally; no
"denial only" claim is made beyond what the host model shows.

| Actor | Can | Result |
|---|---|---|
| Disk attacker (store replay) | present any authentic store it has seen | only the store the chain points to is accepted, plus the unacknowledged roll-forward of rule (c) |
| T3: compromised, correctly measured kernel | extend arbitrary data on any domain | **denial only, but permanent** (P3, agreed): the chain leaves every authentic store and the domain stays denied until authorised re-genesis with a fresh instance, with all durable grants lost. No authorisation policy stops this, and a PCR 13 phase policy would not either, because the kernel legitimately extends at run time |
| T3, TEST profile without M1-HAUTH (empty ownerAuth; the state until the loader enforces it) | `NV_UndefineSpace` + `NV_DefineSpace` + replay of the public genesis and extend events | **revival of an older store: NOT defended** (retained regression `undefine_redefine_replay_revives_with_empty_owner_auth`) |
| T3 at run time, M1-HAUTH provisioned | undefine, redefine, clear, read `HAUTH` | **refused**: `TPM_RC_BAD_AUTH` / `AUTH_FAIL` / `POLICY_FAIL` on swtpm (probe H1–H3), next honest boot rejects X as `Stale` (`undefine_redefine_replay_refused_with_hauth_provisioning`). Writing the loader record is refused in the host model only (U26). Not yet native |
| T3 using ACPI S3: suspend without `TPM2_Shutdown(STATE)` | firmware may fall back to `Startup(CLEAR)` on resume, resetting PCR 13 to 0³² inside the same kernel, then read `HAUTH` | **NOT defended, unverified (U27)**: until the TPM-bearing TEST configuration disables S3 or the firmware is shown to halt or cap PCR 13 on resume, the runtime claim excludes S3 |
| T3 with an OS-usable platformAuth (U23 broken) | platform `TPM2_Clear`, then rebuild the old chain and its own `HAUTH`/hierarchy auth | denial *(host model only; rests on the open loader record U26 and on the U13c store: the UEFI copy survives the clear, is written after the TPM copy and is not writable at run time, U28)*: the record is missing or squatted (a forged record in a writable squat is ignored, `forged_record_after_a_platform_clear_is_never_adopted`), the next boot halts until a newer recovery record, recovery clears the foreign hierarchy and provisions new secrets, old stores are `Foreign` (`tpm_clear_is_denial_until_authorised_recovery`). If the UEFI copy is lost as well: revival (U13c residual above) |
| T3 while ownerAuth is empty (a cleared TPM that reached a kernel, REL-05 section 7 item 9 open) | squat the `HAUTH` or record handle (probe S1, S4) | **refused in the host model**: owner squats are removed before the secret is written, a platform squat halts (`squatted_hauth_never_receives_the_secret`); loader slice open |
| T3 | set `endorsementAuth`; trip the lockout hierarchy with a wrong `lockoutAuth` | denial: EK check fails (section 10.5 halt); recovery uses ownerAuth, not lockoutAuth |
| Boot-path attacker before B04: loader replaced on the ESP, `BootNext` to other code, or T5 (foreign OS) | run code at PCR 13 = 0³², read `HAUTH`, then undefine, redefine and replay | **revival of an older store one reboot later: NOT defended until B04** (residual `reboot_path_attacker_at_pcr13_zero_still_revives`). PCR 13 phase separation does not authenticate the boot path, the TPM identity or the kernel |
| swtpm host administrator | snapshot and restore the whole TPM state with a matching store | not defended (REL-05 threat model) |
| Physical attacker | — | not defended |

### Decided: M1-HAUTH (project owner, 25 November 2026)

Option 1 of the earlier OWNER question was approved: B protects `ownerAuth`
and `lockoutAuth` in the TEST/QEMU/swtpm profile (ADR-B03-REL05-TPM-20261125,
amendment M1-HAUTH). The earlier proposal of a sealed object was replaced by
the policy-read `HAUTH` index: a sealed object needs a primary key under the
owner hierarchy, which conflicts with a protected ownerAuth; the consumer
interface is unchanged. The counter alternative was not pursued.

**What is claimed:** within a boot, code without loader-phase access (the
kernel phase, T3 at run time, excluding S3 resume until U27) cannot undefine, redefine or clear the anchor,
so the targeted M1 revival is refused at run time and on the next honest
boot (swtpm probe and host model; native proof open).
**What is not claimed:** protection against code that runs at PCR 13 = 0³²
(boot path before B04), against an OS-usable platformAuth beyond denial
(U23), against the swtpm host administrator, or any production TPM
procedure. The revival through the boot path is a stated limitation that
needs B04 (authenticated boot path; for example binding `HAUTH` to the
measured loader through `PolicyAuthorize` under the update trust anchors);
it is not accepted as a residual risk by B and is reported to the project
owner.

## Unverified TPM behaviour

Listed in REL-05 section 14; the host model assumes them, S5 must prove them.

- U21 The first `NV_Extend` of a new extend index starts from zeros; reading
  it before any extend fails. *Observed on swtpm 0.10.1 (probe), not native.*
- U22 One `NV_Extend` is atomic across power loss.
- U23 The OS can never use platformAuth after OVMF randomises it.
- U24 Platform authorisation can undefine an owner-created index and clear
  the TPM (*observed on swtpm 0.10.1 with empty platformAuth*); with
  lockoutAuth set, `TPM2_Clear(lockout)` with empty auth fails. U23 is
  therefore load-bearing.
- U25 OVMF provides `EFI_RNG_PROTOCOL` in the loader phase (entropy for
  `HAUTH`).
- U27 ACPI S3 resume in the TPM-bearing TEST QEMU configuration: whether
  OVMF falls back to `Startup(CLEAR)` after a missing `TPM2_Shutdown(STATE)`
  and so resets PCR 13. Required: disable S3 there, or prove the firmware
  halts or caps PCR 13, plus a native negative test.
- U28 The UEFI copy of the accepted pair (U13c) survives TPM clear and
  replacement and cannot be written after `ExitBootServices`
  (boot-services-only variable). The host model assumes it.
- REL-05 section 7 item 9 still routes a cleared TPM to a `DEGRADED`
  known-good boot, which runs the kernel with an empty ownerAuth; M1-HAUTH
  and C665 require a halt until authorised recovery. Open conflict, resolved
  in the loader slice.
- U26 Loader record design (handle, size, per-domain layout and public-area
  check of the record that binds `HAUTH` and names the anchored instances).
  The clear-is-denial result rests on it.
- Firmware-mediated clear (physical-presence interface) under the QEMU
  configuration is not examined; it yields a cleared TPM, which halts until
  recovery like any other clear.

## Limits

The anchor inherits the REL-05 threat model: a host administrator who
snapshots and restores the whole swtpm state together with the store, or a
boot-path attacker before B04, defeats it. So does an attacker who deletes
both copies of the accepted pair (U13c; no snapshot needed): the old `ENROLL`
then authorises a second genesis (`accepted_pair_survives_one_lost_copy`,
open U13c item, reported to the project owner, not accepted). A chain establishes freshness of
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
undefine/redefine/replay sequence (revival without M1-HAUTH, refusal with
it); M1-HAUTH provisioning cut after every write; `HAUTH` read refused after
the PCR 13 extend; owner and platform squats of `HAUTH` and the loader
record (probe S1-S4) before provisioning; loss of one accepted-pair copy
refused, loss of both shown as the residual; TPM clear, replaced TPM and foreign boot environment each
halting until authorised recovery; authorised same-TPM recovery with a fresh
instance; the boot-path residual demonstrated, not hidden.
