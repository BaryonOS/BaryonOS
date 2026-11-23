# ADR-NET-ISSUER-YYYYMMDD (date set on adoption) — Network grant issuer, helper subgrants, regrant and durable denial

**Status: PROPOSED — owner decision required.** This file is a draft for
adoption into `docs/DECISIONS.md`. Nothing in it is adopted, implemented or
tested. Every "Recommendation" below is a recommendation, not a decision.
Baseline: `origin/main` `2f4ac6ae814fac18bca52ff80d4b9f774d12bec2`; all
`file:line` citations refer to that commit.
Parent contract: [N04-ISSUER-DELEGATION.md](N04-ISSUER-DELEGATION.md) (PROPOSED).
Owner instruction (23 November 2026): issuer, subgrant and regrant code is held
until an ADR settles the issuer/storage split and who may authorise subgrants.
N04 stays open, and the roadmap score is unchanged.

## Context: current behaviour

Two independent mechanisms exist. Neither is a production issuer.

1. **`services/net-broker`** is a host-tested authority core. `Caller`
   (`services/net-broker/src/lib.rs:38`), `PolicyAuthority` (`:45`) and
   `ServiceInstance` (`:51`) are opaque, and safe external code cannot
   construct them (`compile_fail` doctests, `:9`–`:22`). Only private test
   fixtures construct them. `Broker::grant` (`:341`) picks a free slot whose
   generation is below `u64::MAX` (`:368`) and increments it (`:377`), so an
   exhausted slot is never wrapped. `Broker::revoke` (`:391`) removes the policy
   and, if a session uses the grant, calls `fence` (`:407`). `fence` sets a
   backend-wide quarantine (`:293`) that no code path clears, so every later
   operation, including `grant`, fails in `ready` (`:233`). Idle revocation
   permits slot reuse with a new generation. Revocation with a live session
   ends the broker instance. Deadlines are boot ticks bounded by
   `MAX_TIMEOUT_TICKS = 6000` (`:34`), and there are at most `MAX_GRANTS = 8`
   grants (`:33`).
2. **`services/network-service`** is the native path that the N04 revoke and
   helper evidence actually runs on. Identity is the kernel-authenticated
   sender PID matched against trusted startup `Configuration.pids`
   (`services/network-service/src/lib.rs:664`), and the wire `epoch` must
   equal `Configuration.epoch` (`:679`). `Service::revoke` (`:580`) sets
   `owner.revoked = true` (`:583`). That flag is cleared nowhere: its only
   other assignment is the initial `false` (`:240`). After revocation, only
   State, CloseTcp and Retire are answered, and everything else returns
   `Access`. `Owner.generation` is a per-connection counter that feeds the
   connection token (`:757`, `:779`), not a grant generation.
3. **The native issuer today is the kernel test fixture.** Under
   `test-net-service-revoke` and `test-net-service-helper`
   (`kernel/src/m2/m3/network_service.rs:12`–`:13`), the manifest creates
   endpoints through `channel` (`:73`). The revoke case delegates a private
   control endpoint to a fourth "policy" process (`:110`–`:131`). The helper
   case gives the helper a delegated SEND alias (`:83`). The service still
   rejects the helper because authority is the PID match, not possession of
   the endpoint (`services/network-service/src/lib.rs:664`). The fixture epoch
   is the constant `1` (`kernel/src/m2/m3/network_service.rs:6`). The control
   wire knows only `NETSTOP1` and `NETRVK01`
   (`sdk/examples/network-service/src/service.rs:259`, `:263`), which the
   service accepts only from the authenticated policy PID (`:251`).
4. **Revocation is in memory only.** A restarted service with the same
   configuration and epoch starts with every owner unrevoked. No grant,
   subgrant or regrant message exists.
5. **Kernel capability rules already provide attenuation and descendant
   revocation:** `bootstrap_delegate`
   (`crates/capability-core/src/lib.rs:1458`), the rights `subset` check
   (`:1712`), descendant marking in `revoke` (`:1488`), and slot retirement at
   maximum generation (`:1868`–`:1874`). The kernel contract states that
   revocation marks all descendants and that no lasting rights come from a dead
   delegator (`docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md:128`).
   Network destination grants are not kernel capability objects. They exist
   only in service state.
6. **Related decisions and contracts:**
   - ADR-044 (`docs/DECISIONS.md:3324`) requires one bounded broker.
   - ADR-035 (`:2716`) ("Security Floor remains active when privacy is voluntarily reduced") keeps the Security Floor in force when a privacy choice widens sharing; consent expands functionality, not trust.
   - ADR-B03-B04-TARGET (`:108`) leaves the persistent anchor backend
     unapproved.
   - The privacy contract requires durable blocking before cleanup and no false
     durability acknowledgement (`docs/LOCAL-PRIVACY-CONTRACT.md:72`;
     `docs/implementation/NAS-SHARING-AND-INTERNET.md:73`–`:77`).
   - B02 states that a cold instance cannot distinguish an old valid snapshot
     without an independently trusted lower bound
     (`docs/implementation/B02-SECURE-RECORDS.md:137`–`:144`).

## Decision 1 — production issuer and issuer/storage owner split

**Question.** Which component constructs `Caller`, `PolicyAuthority` and
`ServiceInstance` natively? Which component owns durable grant and denial
state? Which mechanism, broker or network service, becomes the production
enforcement path?

| Option | Identity issuer | Policy authority | Durable state owner | Assessment |
|---|---|---|---|---|
| 1A | Kernel IPC metadata (sender PID plus incarnation) read in the trusted service loop | Possession of a kernel-delegated private control endpoint, held by one dedicated policy process | Policy process | Keeps storage keys out of the process that parses untrusted packets. Requires a policy-to-service state handoff at start. |
| 1B | As 1A | As 1A | Network service | Persisting and fencing happen in one process, so ordering is simpler. The network-facing process gains a storage capability, which enlarges the blast radius of a parser compromise. |
| 1C | Kernel | Kernel (destination grants become kernel capability objects) | Kernel or storage service | Reuses kernel attenuation and revocation directly. Moves destination and network policy into the trusted kernel, contrary to ADR-044 and the minimal-kernel principle. |

**Recommendation (not a decision): 1A.**

- **`ServiceInstance`.** It is constructed once, in the trusted bootstrap of
  the network service. It binds the exclusive `NetworkSource` lease (ADR-044)
  and a kernel-supplied service incarnation. That incarnation replaces the
  constant fixture epoch.
- **`Caller`.** It is constructed only from kernel IPC metadata
  (`message.sender()` and a kernel incarnation value), never from a payload
  field.
- **`PolicyAuthority`.** It is constructed only for messages that arrive on
  the manifest-delegated control endpoint from the authenticated policy PID.
  This generalises the existing `NETRVK01` path.
- **Production path.** The broker semantics (opaque contexts, per-slot grant
  generations, exact destination, bounded deadline) become the production
  authority model inside the native network service. The static
  `Configuration.pids`/`destinations` bindings remain a test profile only.
- **Durable state.** The policy process is the single writer of durable grant
  and denial state, through the existing authenticated-record storage boundary.
  The network service holds only the live enforcement copy. It starts with
  every grant denied, and it opens no ingress or egress for app traffic until
  the policy process has delivered its current state.

**Rationale.** The network service handles untrusted frames and app requests.
Withholding storage and key capabilities from it follows least privilege. The
same split already exists natively for revocation (point 3 above).

**Open prerequisite (not verified here).** This proposal does not establish
whether the current kernel can reuse a PID within one boot. The issuer
integration must either show that PIDs are never reused or add an explicit
incarnation value to IPC metadata. The forged-PID and recycled-PID negative
tests are mandatory either way.

## Decision 2 — who may authorise a helper subgrant

| Option | Authoriser | Assessment |
|---|---|---|
| 2A | PolicyAuthority only. The parent holder may *request* a subgrant, but the request is not binding. | Consent stays at the policy boundary (ADR-035). An app cannot create network authority for another process. One extra policy round trip per helper. |
| 2B | Parent grant holder, directly, with attenuation (capability-style) | Fewer prompts. Broadens app capability semantics: an app becomes a network-authority issuer. Under the code review rules, this needs an explicit design. |
| 2C | Parent holder within a per-app delegation budget that PolicyAuthority sets in advance | A compromise, with more state and more negative tests. |

**Recommendation (not a decision): 2A for the first profile.** 2C is
deferred, and 2B is rejected for now. Endpoint delegation alone conveys no
network authority. The helper fixture already shows that a delegated SEND
alias is refused by the PID check. A subgrant therefore needs a service-side
child record.

These attenuation-only rules apply under every option:

- The helper has its own authentic `Caller`. A child grant is never usable by
  its parent, and a parent grant is never usable by the child.
- The child's destination is exactly the parent's IPv4 and port. There is no
  wildcard, range widening or hostname-derived expansion.
- The child's rights are a subset of the parent's rights. `Send`/`Receive`/
  `Both` intersection is checked before any slot is allocated.
- The child's deadline is no later than the parent's deadline at issue time.
- The child record stores the parent slot and parent generation. Any mismatch
  makes the child `StaleGrant`.
- Depth is limited to one: a helper cannot subgrant further. This limit is
  stricter than the kernel depth of 8 and is deliberately narrow.
- Revoking the parent revokes every child and fences its sessions in the same
  step. Revoking a child leaves the parent and its siblings unchanged.
- Child records count against `MAX_GRANTS`. Exhausting that limit returns
  `GrantLimit` and never evicts another grant.

**Known cost.** In the current broker, revoking any grant that has a live
session quarantines the whole shared backend (`services/net-broker/src/lib.rs`, `fence`, `:292`–`:293`).
Cascading revocation therefore also interrupts unrelated apps until a verified
device reset and an explicit recovery decision. Per-socket fencing is separate
future work and is not part of this ADR.

## Decision 3 — explicit regrant generations

**Current behaviour.** In the network service, a revoked owner stays revoked
for the life of the process, and no regrant message exists. In the broker, an
idle revoke allows a new generation, but a revoke with a live session ends the
instance.

| Option | Regrant mechanism | Assessment |
|---|---|---|
| 3A | Within the same service instance. An explicit PolicyAuthority decision issues a new grant record with a strictly higher owner generation. | Supports normal use without a service restart. Needs a new persisted per-owner generation. |
| 3B | Only through a new service incarnation | Simple. Every regrant disturbs every app, which is not acceptable for usability. |

**Recommendation (not a decision): 3A, under these rules.**

- **New state.** A per-(principal, destination) *owner generation*, kept by
  the policy process. It is distinct from the existing per-connection
  `Owner.generation`/token counter.
- **Monotonic and durable within the authenticated store.** The generation is
  persisted before the grant is delivered. Within one boot, and within one
  unbroken lineage of the authenticated store, it is never reset or reused.
- **Regrant.** A regrant issues generation g+1, greater than every generation
  previously issued or tombstoned for that owner *in the store as loaded*.
- **Cold boot and replay (limit under 5A).** After a cold boot the policy
  process reads the owner generation from the authenticated store. Option 5A
  gives no trusted lower bound, so a replayed older but authentic store can
  roll the counter back, and later re-confirmations can issue generation
  numbers that were already used before the power cycle; tombstones written
  after the replayed snapshot are absent. This does not revive old authority,
  because every token, connection, queued request and delivered grant is also
  bound to the service incarnation of the current boot, and nothing issued
  before the power cycle is valid after it (5A starts every grant denied).
  Generation numbers are therefore unique only per boot and store lineage,
  never as a global identifier, and no persistent reference may use a
  generation without the boot incarnation. A trusted cross-boot lower bound
  needs a freshness anchor, which is option 5B.
- **Old authority.** Tokens, connections, queued requests and late completions
  that carry an older generation or an older service incarnation are refused.
  - Generation or incarnation mismatch: `Stale` (wire) or `StaleGrant`/
    `StaleConnection` (broker).
  - A still-revoked owner without a new grant: `Access`.
  - Old RX data is never delivered to the new grant.
- **Exhaustion.** An owner at the maximum generation is permanently retired.
  The response is `GenerationExhausted`, with no wrap and no reset, mirroring
  `capability-core` `detach`.
- **No automatic re-enable.** None of the following re-enables a grant: grant
  expiry, an app or helper restart, a network-service restart, a policy-process
  restart, a reconnect, or a NAS/peer request. A regrant always needs a new
  explicit PolicyAuthority decision.
- **Quarantine.** While the backend is quarantined, a regrant is recorded but
  not activated until a verified reset (an existing ADR-044 requirement). A
  counter increment or a service restart is not a reset acknowledgement.

## Decision 4 — durable revocation acknowledgement across service restart

**Recommendation (not a decision).** A revocation result has explicit states,
reported separately:

1. **`LiveDenied`.** The live fence is closed on the current service
   incarnation. This happens first and never waits for storage.
2. **`DurablePending`.** The tombstone or generation write has been submitted
   to the authenticated-record boundary.
3. **`Durable`.** The write succeeded and exact readback matched. Only this
   state may be shown as "revocation survives restart".
4. **`DurabilityUnknown`.** The write failed, timed out or read back
   differently. Live denial stays in place. The state is reported as unknown,
   never as success. Retry is allowed, and a later `Durable` requires a fresh
   readback. The grant stays denied for the rest of the boot, including
   across a restart of the policy process or the network service, even if
   storage still holds a valid allow record for it: a component that outlives
   both restarts within the boot keeps a boot-lifetime denial entry, and the
   network service refuses re-delivery of that owner until the revocation has
   reached `Durable` with a fresh readback. Which component holds that entry
   (for example the service manager or the kernel delegation table) is open
   question 7.

**Restart semantics** (warm restart within the same boot):

- A new network-service incarnation starts fully denied. Old tokens are
  `Stale` because the incarnation changed.
- The surviving policy process re-delivers only non-revoked grants of the
  current generation.
- Time-bounded grants do not survive any restart. They are expressed in boot
  ticks, and there is no trusted wall clock (ADR-050, ADR-057). Persisted
  grants are "until revoked" grants, or they are not persisted.
- If the policy process itself restarts, it reloads from storage, and every
  boot-lifetime denial entry from Decision 4 state 4 overrides any allow
  record it reloads. Missing,
  corrupt, unauthenticated or version-unknown state yields **no grants**
  (fail-closed) and a visible "network permissions could not be restored"
  status. It never yields reconstruction from app requests.
- **Proposed persistence model.** Persist an allow-list of grant records plus
  generation tombstones. Then loss of state denies access rather than
  re-enabling it. Within a boot, a failed tombstone write is covered by the
  boot-lifetime denial entry above, not by Decision 5. Across a power cycle,
  only *replay* of an older valid state can resurrect a revoked grant;
  Decision 5 addresses that cold-boot case.

## Decision 5 — cold freshness anchor

**Question.** After a power cycle, how is a replayed older but authentic grant
store, one that still contains a since-revoked grant, distinguished from the
current store? A checksum, a service incarnation or a sidecar counter on the
same disk cannot establish this (B02, `:137`–`:144`).

| Option | Description | Assessment |
|---|---|---|
| 5A | Keep cold reinstatement unavailable. After a cold boot, every network grant starts denied. Previously persisted grants are shown only as *non-authoritative suggestions* for explicit re-confirmation, which creates new generations. | No new trust root. Fail-closed. Costs one confirmation step per app after reboot. A replayed store can only *suggest*. Residual risk: a user may re-confirm a grant they had revoked. |
| 5B | Adopt a new anchor | **A new trust root.** Bind the store generation to the persistent anchor backend of ADR-B03-B04-TARGET D3 (for example a TPM NV monotonic counter), shared with update rollback protection rather than a network-specific root. That backend is not yet approved, and it adds lifecycle, crash-ordering and recovery obligations. |
| 5C | Freshness from a NAS or peer | Rejected. External providers are not trust anchors (ADR-036, `docs/DECISIONS.md:2744`), and core local tasks must work offline. |

**Recommendation (not a decision): 5A now. Revisit 5B only together with the
B03/B04 anchor backend decision.** The anchor would protect the policy
process's grant/tombstone store, not individual tokens. No network-specific
trust root is proposed.

The parent contract's requirement still holds: where denial freshness cannot
be established, cold restart remains unavailable. Under 5A that is the
designed behaviour, not a defect.

## Native evidence, negative tests, Security Floor, migration and rollback

**N04 acceptance clause** (`docs/progress.json`): "Process and destination
permissions, revocation, and helper routes are enforced natively and observed
in traffic; no unintended listening ports exist."

| Choice | Native evidence it unlocks (future, not executed) |
|---|---|
| 1A | A native case in which the production bootstrap, not a test feature, constructs the service instance, caller and policy contexts. A grant is issued by the policy process and observed on the wire. |
| 2A | A positive helper route: a policy-issued child grant is used by the helper on the wire. Amplification attempts are refused before backend effects. Parent revoke cuts the child's live connection (an RST observed on the wire). |
| 3A | Native regrant: after revoke, an explicit decision issues generation g+1. Old tokens and queued requests are refused, and the new connection is observed. |
| 4 | Warm service restart with intact policy state: revoked stays revoked, and non-revoked grants resume. Crash injection around the tombstone write. |
| 5A | Cold boot: no grant is active before re-confirmation. A replayed older store does not reactivate a revoked grant. |

This ADR does not address the "no unintended listening ports" part of the
clause, which remains separate evidence.

**Negative tests required before code** (host first, then native):

- **Issuer.** Forged payload PID. Recycled PID with an old incarnation.
  Policy message from a non-policy PID or on a wrong endpoint. Cross-service
  `PolicyAuthority`. Duplicate backend lease.
- **Subgrant.** Wrong destination or port. Extra right. Later deadline.
  Missing parent, stale parent generation or revoked parent. Grandchild
  attempt. A parent using the child's grant and the reverse.
  `GrantLimit` without eviction.
- **Regrant.** Stale token after regrant. A queued request and a late
  completion from the old generation. Generation exhaustion without wrap. No
  re-enable after expiry, any restart, or a peer request.
- **Durability.** Crash before and after the tombstone write and before and
  after the acknowledgement. Torn, corrupt or missing store. Unknown version.
  Readback mismatch reported as `DurabilityUnknown`.
- **Cold boot.** Replay of an older authentic store under 5A.
- **Mutation controls.** A mutant that skips each check must fail the
  corresponding test.

**Security Floor.** 1A, 2A, 3A, 4 and 5A only narrow or preserve authority.
None disables isolation, capability checks, key isolation or protection of
unshared data. 1B adds a storage capability to the network-facing process.
2B broadens app authority and needs an explicit design. 5B adds a trust root.
These three are the options with Security Floor impact.

**Migration.**

- The existing test features, the static `Configuration` bindings,
  `NETRVK01` and the helper alias stay test-only. Their recorded evidence
  (`docs/verification/n04-native-revoke-20261105/`,
  `docs/verification/n04-helper-20261114/`) remains valid for its own source
  and is not reinterpreted.
- New control messages are versioned additions next to `NETRVK01`.
- Broker contexts gain a constructor only inside the trusted bootstrap path,
  and the `compile_fail` guarantees are kept.

**Rollback.**

- Revert to the static test-profile bindings and keep N04 open.
- A downgraded build treats an unknown store version as "no grants". A
  rollback never reinstates a revoked grant.
- The denial state format is versioned from its first release.

## Questions for the project owner

1. Is 1A adopted, with the policy process as the single durable owner and the
   network service holding no storage capability?
2. Is helper subgrant authority PolicyAuthority-only (2A), with depth one and
   the attenuation rules above, or is a parent-holder budget (2C) wanted now?
3. Is regrant within a running service (3A), with a strictly new persisted
   owner generation, accepted? Is whole-backend quarantine on cascading revoke
   acceptable until per-socket fencing exists?
4. Are the four acknowledgement states of Decision 4 and the "no time-bounded
   grant survives restart" rule accepted?
5. Cold freshness: is cold reinstatement kept unavailable (5A)? Or should the
   B03/B04 anchor backend be adopted as a **new trust root** for network grant
   state (5B), and if so, when?
6. Does N04 acceptance require durable revocation across restart, or only
   native enforcement, revocation and helper routes within one boot?
7. Which component, outliving both a policy-process and a network-service
   restart within one boot, holds the boot-lifetime denial entries of
   Decision 4 (`DurabilityUnknown`)?

## Sources

All repository sources are from the BaryonOS project (organisation BaryonOS),
repository <https://github.com/BaryonOS/BaryonOS>, at baseline commit
`2f4ac6ae814fac18bca52ff80d4b9f774d12bec2` (2026-11-23). A citation `path:line`
resolves to
`https://github.com/BaryonOS/BaryonOS/blob/2f4ac6ae814fac18bca52ff80d4b9f774d12bec2/<path>#L<line>`.

- `docs/DECISIONS.md`: ADR-035 "Security Floor remains active when privacy is
  voluntarily reduced"; ADR-036 "An external provider may only provide opaque
  ciphertext storage"; ADR-044 "Native networking through one bounded
  capability broker"; ADR-050 "Opt-in wall-clock syscall remains visibly
  untrusted"; ADR-057 "Exclusive native CMOS bootstrap, no trusted time";
  ADR-B03-B04-TARGET-20261119 "Target architecture for update authenticity and
  recovery".
- Contracts: `docs/LOCAL-PRIVACY-CONTRACT.md`,
  `docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md`,
  `docs/implementation/NAS-SHARING-AND-INTERNET.md`,
  `docs/implementation/B02-SECURE-RECORDS.md`,
  `docs/implementation/N04-ISSUER-DELEGATION.md`.
- Implementation: `services/net-broker/src/lib.rs`,
  `services/network-service/src/lib.rs`,
  `sdk/examples/network-service/src/service.rs`,
  `kernel/src/m2/m3/network_service.rs`, `crates/capability-core/src/lib.rs`.
- Evidence: `docs/verification/n04-native-revoke-20261105/`,
  `docs/verification/n04-helper-20261114/`.
