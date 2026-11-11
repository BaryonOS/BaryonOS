# TASK-N04-NET-BROKER - bounded socket authority

Contract proposal for read-only alignment with Security, not yet the fixed
SDK/IPC interface. Base `4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8`.
Rubric N04/A03/B06; release FR-N01/FR-N04/FR-K03. Native TCP stack,
codec/client/listen branches and their source-bound evidence are retained.

## Goal, owner and evidence boundary

NET owns only this new standalone `services/net-broker/**` leaf and
`.github/workflows/net-broker-quality.yml`, per manager `89f84098`.
Kernel, driver/device adapter, SDK and existing stack files are not
modified. Security owns the identity/capability/crypto contracts; native
manifest/issuer/SDK integration is aligned separately with the relevant owners.
No second transport stack, crypto layer or wire ABI.

First complete step: allocator-free authority core with fixed grant and
connection tables on top of the existing `nieuw-os-net-stack::Stack`.
Normal/negative host tests, Clippy, formatting and bare-metal library build
are the gates for this step, not native network enforcement or points.
The current real TCP run fails for ring 3 at the CR3 switch; REL owns that
blocker. An authentic caller/issuer seam does not yet exist in this leaf.

## Security invariants

- Caller identity arises exclusively from authentic kernel/
  capability metadata plus trusted bootstrap binding to service instance and
  generation. No request PID, IP, route, hostname, client or
  `verified=true` field can supply that identity.
- The intended `Caller`, `PolicyAuthority` and `ServiceInstance` are opaque
  internal context types without a public constructor/parser/From/Default.
  A production issuer is explicitly missing: no safe native caller can
  fabricate them itself now. Only private host test fixtures model this context;
  that is not successful authentication or kernel binding.
- Installing/revoking policy requires a separate `PolicyAuthority` for
  the same service instance; an app grant never becomes policy authority.
  A helper has its own caller context and does not receive the grant of its
  principal. Delegation is not implicitly supported in this first core.
- At most eight grants, each with exactly one unicast IPv4 address and non-zero
  TCP port; at most two sockets, equal to the existing stack. No wildcard,
  implicit listen, name-to-IP authorization or automatic destination expansion.
  An endpoint is a permitted coordinate binding, not a TLS/NAS identity.
- Every grant/connection binds caller, service instance and its own monotonic
  slot generation. Reuse keeps the generation counter; overflow fails without
  publication. Old handles cannot serve a new slot or another app.
- Connect/send/receive/close/cancel check the current grant and caller; send and
  receive have separate rights. Rejection comes before backend action or
  output change. Receive errors deliver no new bytes. Connect itself chooses
  a bounded local port; an app opens no implicit listening port.
- The service event loop supplies real monotonic SDK ticks, never request time.
  Absolute grant/connection deadlines, rollback and overflow are checked before a
  backend step. The existing stack remains owner of TCP timers,
  packet buffers, retransmission and backpressure; equal ticks do not add up.
- Send reports only locally queued bytes, no NIC, peer or
  application ACK. An uncertain outcome is never silently repeated.

## Revocation and transport ownership - to be assessed explicitly

The current stack has a single shared TX slot, but no external
per-flow ownership/discard API. NET will not fake a local release via
`complete_transmission`: that remains exclusively a real, matching
device completion from the trusted adapter.

Safe first lifecycle after SEC alignment: the linearization point for
revoke/expiry/cancel first closes the egress fence: no new TX publication,
including timer retransmission or control packets from abort/cleanup. Then
all connections of the relevant grant are retired before any
backend step takes place; queued RX is no longer delivered to a caller.
Only then may sockets be aborted locally. Software-queued frames are
not published. Frames already published to hardware can still go out on the wire:
revocation cannot recall them and promises no immediate network silence.
This requires transport reset/quarantine before slots are reused.
An already queued or possibly published frame is not re-offered,
reused or booked as completed. An uncertain NIC result
likewise quarantines the entire shared backend/device. No other broker,
producer or new service instance may bypass the fence. No public
`reset_ok=true` detour or
fictitious reset ACK; recovery requires a future real driver/issuer boundary
with proven device quiescence/reset plus trusted epoch binding or demonstrably
drained old completions. Only a new issuer/service instance is
expressly not sufficient. The first core offers no recovery function.

This first conservative quarantine can also pause other sockets on the same
device instance. That is a visible provisional core limit, not
accepted daily multi-app behaviour or a justification for N04 points.
Fine-grained per-flow drain and native reset/cleanup are follow-up integration, not
silently bypassed by a second stack or an unproven completion.

An exclusive Stack borrow does not prove exclusive NIC ownership. The
production issuer would have to supply an exclusive backend lease, without
public constructor or possibility of duplication; that issuer and device-wide
fence do not exist yet. Private fixtures model only a single producer.
Retaining DMA/buffer ownership across service crash/Drop also requires the
real driver boundary: Rust lifetimes or this core are no evidence of that.

The core must revoke the authentic grant before cleanup is attempted.
A cleanup error therefore cannot restore old authority. A success result
may describe only local retirement, not packets deleted at
a peer. All open device buffer lifetimes remain at the hardware/driver boundary.

## Intended source interface, not a wire schema

- `Broker`: exclusive owner of the existing `Stack` and two already assigned
  caller-owned sockets/buffers; no public mutable stack bypass.
- `grant`/`revoke`: only with opaque policy context, bounded endpoint and
  rights, real monotonic time and generation binding.
- `connect`/`send`/`receive`/`close`/`cancel`: opaque caller, grant and socket token;
  explicit results for deny/stale/limit/clock/timeout/quarantine/backend.
- Poll/ingress/egress are exclusively the trusted service/device seam,
  not an app message. A future adapter may only pass on real device completion;
  `Unknown` yields no ACK, republication or buffer release.
- There is no native RPC dispatch, issuer, SDK client or manifest grant yet.
  Message320/sender PID alone is not promoted to strong identity.

## Acceptance for this core

Normal authorized connect and rights; deny without backend side effect,
cross-caller/service instance, stale/reused generation, revoke on existing and
new sockets, helper without delegation, grant/socket exhaustion, wrong
destination, timeout/rollback/overflow, backpressure and Unknown/no-retry.
Where real TCP bytes are needed the existing stack remains the implementation;
test counterparties may only be synthetic peers/model media.

After interface alignment: Rust 1.91.1 like the existing stack, dependency only
on that existing stack and, if needed, the same already checked smoltcp for
host counterparty tests. No new registry version or feature expansion.
Exact commands/results/source hashes are stored under `verification/`.
Production gates must show that fixture constructors are not exported,
feature-enableable or reachable via safe public wrappers.
A separate quality workflow must run all new core tests, Clippy, fmt and
`x86_64-unknown-none`; no proxy for native acceptance.

No gate executed and no criteria signed off in this contract proposal.
Score proposal to central manager: network 0/10 -> 0/10, total 34 -> 34.
The manager remains owner of PROJECT-STATE, progress.json and roadmap.

## Implementation checkpoint after contract alignment

SEC read proposal `9bef711` read-only and asked for five concrete clarifications;
the fence, backend, linearization, recovery and fixture boundaries above
have been tightened accordingly. This is not a human audit or native approval.
The core now implements the local authority checks and fence with the existing
Stack. Public context types have no production constructor; there is no
usable native issuer yet. All methods are source API for the trusted
service loop, not request fields or extra wire ABI.

Grant revocation without a used socket frees a generation-safe grant slot.
Revocation with an active socket conservatively retires ALL sessions
on this backend, closes all local egress paths and only then aborts sockets.
No recovery/reset function. A NIC completion after the fence is not processed as
a core release; real device quiescence and crash/Drop ownership remain
requirements for the future adapter.

The two sockets are set up in advance. `finish` is explicitly relinquishing
a closed session, including unread RX, and requires no offered or
queued TX. The next real Stack connect resets RX/TX before a new
caller gets the socket. There is no mutable Stack bypass in the public API.
The model also tests this non-inheritance and generation binding.

First host probe: 21 rights tests plus four compile-fail production boundaries
green. Extended TCP model probe afterwards: 23 green / two red, exit101;
the fixture read only 64 of 96 received bytes because after Closed it no longer
read. Fixture fixed to drain until empty/EOF, including Closed;
afterwards 25 plus four compile-fail green. This is no repaired native
data-loss claim. The first incomplete WIP fingerprint was not used as
acceptance evidence. With extra boundary/slot probes 28 unit and four
compile-fail tests are now green. Final source-bound gates follow on the
source checkpoint; Clippy/none/CI/native have not yet been signed off.

## Follow-up: control packets also have destination rights

A separate read-only code review of `0962d02` found that unsolicited
ingress can make the Stack generate a RST or ARP reply to an unauthorized destination.
The original `offer` checked only the local
fence, not that derived destination. A targeted ARP regression confirmed this
on unmodified production source `0962d02` with an extra test: exit101.

`offer` now checks every frame, including transport control traffic, with
the same already used smoltcp checked wire views. IPv4/TCP must match the active
socket, exact local port and current grant destination/port.
ARP is only permitted for the own local address to an active
grant peer. The current Stack has no gateway configuration; a future
next-hop/DHCP infrastructure exception requires an explicit separate
configuration authority, not a wildcard app grant.

Unauthorized or invalid egress is NOT offered: first local
backend fence/retirement, no synthetic completion/discard, slot bytes are
retained for the real future device recovery boundary. On
unsolicited ingress this can require quarantining the entire backend, a visible
availability limit; no product claim or fine-grained discard function.

smoltcp `=0.14.0` is now a direct production dependency for this policy view,
with the same selected features/registry graph as the existing Stack;
no new parser or transport implementation. Pre-fix evidence and new gates
are stored separately: earlier v3/CI source bindings remain historically valid,
but are no green evidence for this follow-up source.
Targeted host re-run: 31 unit plus four compile-fail green.
