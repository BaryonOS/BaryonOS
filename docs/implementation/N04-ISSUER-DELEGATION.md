# N04 issuer, helper delegation and durable denial

**PROPOSED — no production issuer, trust root, grant or wire interface is adopted.**
Baseline: main `69d82075`. This contract identifies the next integration boundary;
it does not reopen the existing native helper investigation or enable a runner.

## Existing source and evidence

[net-broker](../../services/net-broker/src/lib.rs) defines opaque `Caller`,
`PolicyAuthority` and non-cloneable `ServiceInstance` contexts. Their production
issuer is absent. `Broker::grant` increases slot generations and bounds destination,
rights and deadline; `Broker::revoke` removes policy before fencing cleanup.
An exclusive stack borrow does not prove exclusive device/DMA ownership.
The [network service](../../services/network-service/src/lib.rs) instead uses
trusted startup `Configuration` bindings for two clients and bounded destinations.
These are distinct existing mechanisms, not an already integrated production ABI.

The [native helper record](../verification/n04-helper-20261114/README.md) reports
rejection of six requests from an unregistered helper before legitimate clients
are released, on its recorded source. It demonstrates a negative caller boundary,
not positive delegation or durable revocation. The
[kernel security contract](KERNEL-AND-SECURITY-CONTRACT.md) retains authentic envelope
identity, attenuation, generation checks and descendant revocation. An issuer's
own capability surviving descendant revocation does not authorize automatic regrant.

## Proposed interfaces and invariants

| Boundary | Required input and result | Refusal before any backend effect |
|---|---|---|
| Trusted bootstrap | Kernel-bound service instance, process incarnation and exclusive backend lease | Request PID, client token, address or boolean cannot construct identity |
| Policy administration | Separate instance-bound policy authority and explicit destination/right/deadline decision | App/helper grant cannot become policy authority |
| Helper delegation | Authenticated principal and helper; parent identity/generation; exact destination; rights intersection; deadline no later than parent | Wrong incarnation, new destination, extra right, later expiry, missing parent or implicit inheritance |
| Explicit regrant | New generation after an explicit authorization decision | Old grant, queued request, old completion and generation exhaustion |
| Restart | Authenticated denial state and a new service incarnation, before ingress/egress opens | Missing, corrupt, stale or uncertain denial state; no reconstruction from an app request |

Each helper has its own authentic caller identity and independently revocable
child grant. Revoking a parent cuts every existing descendant and pending operation;
revoking a child leaves siblings' authority unchanged, subject to the existing
shared-backend quarantine limit. A child cannot extend its parent or delegate when
its parent lacks that right. Destination authorization remains separate from peer
TLS/NAS identity. No wildcard listening permission or hostname-derived expansion.

The egress fence precedes cleanup. Queued RX is withheld; unpublished TX is dropped.
Already published frames cannot be recalled. DMA buffers/completions remain owned
until actual quiescence/reset is established. Restarting a service or incrementing
a counter is not a reset acknowledgement. Device uncertainty keeps the backend
quarantined, including unrelated sockets where the current shared transport requires it.

## Proposed durable ordering

Close the live authorization fence first. Persist the denial generation/tombstone
through the existing authenticated storage boundary; require successful durability
and exact readback before acknowledging durable revocation. A failed or uncertain
write keeps live access denied and reports an unknown durability result. Recovery
loads denial state before accepting grants; explicit regrant requires a strictly
new generation and an explicit policy decision. No counter wraps or resets to zero.

Process restart with intact trusted state and cold rollback of that state are
separate claims. A replayed older valid disk cannot be excluded by a checksum or
service incarnation. A trusted cold generation anchor, its lifecycle and crash
ordering are **OWNER/SECURITY DECISION REQUIRED**. Until resolved, cold restart must
remain unavailable where denial freshness cannot be established; no new trust root,
escrow or automatic grant recovery is proposed as an implemented solution.

## Negative acceptance and release boundary

| Test owner | Required negative experiment |
|---|---|
| Kernel/issuer integration | Forged PID, recycled PID with old incarnation, cross-service policy context, duplicate backend lease |
| Broker host tests | Helper rights/destination/expiry amplification, parent/child revoke, stale regrant and generation exhaustion |
| Native service/peer harness | Retained child endpoint and queued request after revoke; late RX/TX completion; real reset failure with fence still closed |
| Storage/restart integration | Crash before/after denial persistence and acknowledgement, torn/corrupt/missing state, stale authenticated state, explicit regrant after restart |

These are required future tests, not executed results. Current source-bound helper
and revoke evidence is retained; none proves these additional interfaces. The
production issuer/storage owner split, durable acknowledgement semantics and cold
freshness anchor require explicit alignment before implementation. N04 and the
41/100 roadmap score remain unchanged. Hardware and external human audit remain open.
