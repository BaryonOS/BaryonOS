# TASK-N01-NATIVE-CONFIG

Sole NET writer per queue commit
`16151b9c28a766c47f900e4a3e9f99b642cc3fe5`.
Leaf base `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`; write area exclusively
`services/net-stack/**`, this new `services/net-config/**` and own evidence
below it. No protocol, root, kernel, SDK, driver or central status writes.
TLS #81 and broker #87 remain separate source-bound deliveries.

## Smallest first result

Address configuration must not stay only in a DHCP codec: the same
existing Stack must be able to start unconfigured, apply a checked
address/prefix/default route, and after withdrawal not keep sending under the old
configuration. The first red lifecycle case must show that
data traffic without applied configuration or after withdrawal is rejected.
That is the first implementation step, not the whole native acceptance.

It is followed by the DHCP lifecycle adapter reusing client source
`16adfe54f3e6539339aad60232f7145c4212e538` and the codec on mainca45, without
a legacy root merge or second client/transport stack. The pure client source is not yet
in this base; it must not silently be considered integrated.
Any move/copy of that own existing client leaf must remain exactly
source-bound and recognizable, not redesigned as a second client.

## Shared seams and boundaries

- The existing Stack has two TCP sockets, one IPv4 address, four neighbours,
  one route and a PacketDevice with one RX/TX slot. No dynamic
  address/default-route/DNS config API exists on this base.
- All DHCP, ARP and TCP egress must use the same PacketDevice and the same
  real matching completion path. No second producer publishing NIC frames
  alongside the Stack; no synthetic completion to throw frames away.
- The configuration adapter is trusted service code, not an app grant/issuer.
  A DHCP server/IP/route/DNS answer is not an authenticated TLS/NAS peer.
  Configuration grants apps no new process/destination rights.
- DHCP time is monotonic seconds; Stack/SDK time is 100 Hz. Convert checked;
  rounding must not extend expiry. UTC/RTC is not a network timer authority.
- CheckingAddress may only promote to usable Bound after real RFC5227 probes.
  No public trust boolean or a lone DHCPACK as probe evidence.
  Synthetic host probes remain host model evidence, not native collision detection.
- T1/T2/expiry, NAK, conflict and linkdown withdraw address/route/DNS coherently.
  Reconnect keeps no old configuration, queued RX, grants or sockets.
  DNS server config is route data; answer cache/name rights remain separate
  resolver/broker boundaries. No default DNS fallback when the option is missing.
- Hardware frames already published can still go out on the wire; promise no
  recall. Uncertain TX holds on to buffers/ownership. Withdrawal must
  stop new old-config publication without fake ACK/discard/replay.
  Missing real device reset/quiescence/epoch binding remains explicitly open;
  config code must not replace it with a new instance or `reset_ok`.

## Acceptance and evidence boundary

First targeted red and green lifecycle cases, then normal and negative
DHCP/route/DNS probes: malformed/stale/NAK/conflict/overflow/backpressure,
T1/T2/expiry/linkdown/reconnect and real model packet bytes with a single
completion path. New dependencies only if necessary and within the
existing checked pins/features. Rust1.91.1 like the existing Stack.

Locked host tests, Clippy/all-targets/-Dwarnings, fmt and
`x86_64-unknown-none` library build on the exact source; relevant existing
codec/stack tests retained. Logs, exit statuses, source/artifact hashes under
own `verification/`, checkpoints pushed before long gates.
Native issuer/caller/driver/device fence and the earlier TCP CR3 boot error remain
REL/SDK/HW dependencies; no own write claim for those.
No host tool executes the missing native DHCP/TCP/TLS client.

This document is the first task contract, not yet an implementation or gate.
Network 0/10 -> 0/10, total 34 -> 34 as a proposal to the central owner;
no central progress files changed.

## First implementation step: real Stack address lifecycle

The existing static `Config` constructor remains available; new are
`LinkConfig`, `AddressConfiguration`, `Stack::new_unconfigured`,
`configuration`, `apply_configuration` and `withdraw_configuration`.
These are trusted-service source interfaces, not evidence of a DHCP probe.
No DHCP/DNS/RFC5227 client adapter or new protocol source added yet.

The base interface gate was rightly compile-red (exit101, no runtime).
Then a real runtime case found that `socket.abort()` alone was insufficient:
after reapplying the same address the old RST could still cause ARP/egress.
That red probe with the exact WIP patch has been kept separately.
Withdrawal now first removes address/routes and fences publication; all
sockets are aborted locally. The existing smoltcp SocketSet remove/add
resets the neighbour backoff metadata without releasing buffers and while keeping
the existing process-local handles;
the pinned upstream dispatch resets a tuple without a local source address
BEFORE it can transmit. Afterwards the old TCP/RX state is gone. No
clock is advanced, packet published or completion faked.

An existing PacketDevice TX slot is kept and invisible to new
publication. Reapply returns Busy until a real matching completion has
completed that slot; an unpublished/Unknown slot cannot be recovered
automatically by this. Other real reset/quiescence/epoch evidence remains open.
Clock rollback/overflow on withdrawal explicitly yields an error, but
address/routes/socket authority are nevertheless withdrawn first.

Validation rejects non-unicast/subnet/broadcast addresses and gateways outside
the subnet, equal to the own IP or non-unicast/subnet/broadcast. New
configuration during a live socket is rejected without mutation.
Unread ingress and the old neighbour cache are not passed on.
ARP rate limiting remains the real upstream backoff; a reconnect therefore
need not produce a new packet on the same tick.

Targeted host execution: seven new configuration cases and all fifteen
existing transport tests green. The default-route case observes a real
ARP for the gateway for an off-link TCP destination, not a host socket.
Full source-bound Clippy/fmt/none/codec gates on source2c0e37a are
stored with exit0 under `verification/address-lifecycle/`.
No native DHCP/IP config/app acceptance or points.

## Follow-up: reuse the existing client, no second NIC producer

This area now contains its own no_std crate with only existing local
Stack/codec dependencies and exactly the same registry locks/features as Stack.
The DHCP client and eleven existing packet tests were taken over from source16adfe5;
original client SHA256:
`1f9e64b272665d1ac43d63ae54fdc539f21d236ef91b636b3c2da39f695fd04c`.
On sourcef6d9f1f no algorithm change: only fmt/imports adjusted for this crate.
The former private `arp::unicast_ipv4` predicate has been included exactly as a private
compatibility helper, without widening protocol API/write.
The original branch is kept; no old root changes merged.
`dhcp_client` is still the same pure client, not the native service adapter.
Its `confirm_address_available` may be called by an eventual adapter only
itself after real own probe/completion observation, not via IPC.

New `time` helpers convert in a checked way between 100Hz and DHCP seconds,
with the same upper bound as Stack and rollback detection also within a second.
Flooring request seconds can make expiry earlier, never later. No UTC/RTC.

`Stack::queue_control_frame` is exclusively a trusted-service source interface,
not an app grant or raw socket IPC. The future configuration service validates
target/type/identity of DHCP/ARP before calling it. Frames use
the same PacketDevice TX slot, incrementing id and matching completion as TCP.
Busy/invalid length/exhausted id/clock error replaces no existing bytes.
Withdrawal also hides control frames but keeps storage until real completion.
Unknown/unpublished recovery has not been added yet.

This intermediate checkpoint alone did not yet deliver a coordinator; meanwhile the
actual source implementation follows below. Native driver/caller coupling
is still missing. N01 and network0/10 remain open.

## DHCP/probe/DNS coordinator: host core, not a native process

`service::Service` owns the existing Stack and DHCP client. `start`,
`send_dhcp`, `receive_frame`, `poll`, `complete_transmission`, `transmission`,
`configuration`, `phase`, `begin_renewal`, `link_down` and
`transport_uncertain` are trusted-service source interfaces, NOT new IPC,
app capability, kernel issuer, exclusive device lease or authentic peer binding.
No public probe-success boolean or constructor for a success attestation.

ACK still leaves address/routes/DNS absent. `poll` publishes three real RFC5227
ARP probe bytes via the same PacketDevice; only matching backend completions
advance the probe progress. Intervals come from a validated
`ProbeTiming`: init0..100ticks, then100..200ticks. The native caller must draw these
uniformly from authorized entropy; fixed host values are not an RNG.
After the third completion follow200ticks of waiting time and two announcements,
200ticks apart. Only after completion of the first announcement is the
address/default route applied, and the client Bound and DNS made visible together.

The existing pinned smoltcp wire views encode/decode ARP; no new
dependency or protocol crate write. RFC5227 probe responses with target IP0 are
indeed seen as a conflict. The older neighbour codec rightly rejects those in its
limited resolver profile and has not been used as a full conflict detector.
Own echo is not counted as a foreign probe. A foreign claim during the
full waiting period or during Bound withdraws the configuration; no
permanent address defence. After a conflict wait at least10s, after ten
conflicts at least60s (RFC5227rate-limit); state remains in this service.
A new object is no native device/crash/persistence limit evidence.

A real red runtime case also found that upstream smoltcp does not answer zero-source
peer probes during Bound. The regression and exact WIP patch
against4c335d1 have been kept. The configuration adapter now answers exclusively
a validated foreign probe for the actually applied own address,
without caching 0 as a neighbour, via the same TX slot/completion path.
Busy holds the current frame; the receiving loop keeps its ingress
until it can offer it again. No fake completion or invisible discard.

T1 reports RenewalDue; the loop supplies a fresh XID to `begin_renewal`.
Renewal uses the server MAC from the accepted direct DHCP exchange,
only as an unauthenticated routing hint. T2 changes to broadcast
Rebinding; expiry/NAK/linkdown remove address/route/DNS and hide old
TX. DNS is the server configuration, not a resolver/cache or app name right.
With unchanged address/prefix/gateway the lease/DNS renews without a TCP reset.
A changed route/prefix retires the old sockets; only after old TX completion
does the new epoch become active. That can interrupt connections and is no
claim of seamless route change. Clock fault or Unknown is terminal:
no automatic retry/release/recovery or resuming an old lease.

Twelve targeted host cases are green, including probe/announcement bytes,
stale completion, TX backpressure, late conflicts, ordinary peer probe answers,
T1/T2/NAK, changed gateway/DNS,10conflicts,expiry,linkdown and clock overflow.
Not yet: full source-bound final gates on this coordinator, native process,
real NIC completion/DMA fence/driver reset/CR3, coupled broker policy,
native RNG/retry scheduler/auto-reconnect or full DHCP release acceptance.
On source03277a7 the codec had no DHCPDECLINE builder: a concrete extra claim
was requested then but not taken over. That boundary is explicitly
released below. RFC2131section3.1 requires DECLINE;10s waiting is a SHOULD there and
is applied conservatively here.

The targeted follow-up case on source678081c found a real wire difference: selecting
REQUEST had `secs=5`, DISCOVER `secs=1`. The own moved client now keeps
the `secs` of the last encoded DISCOVER for selecting REQUEST
(RFC2131section3.1step3), without shifting the conservative lease clock forward/back.
The new regression also checks real expiry at the
original REQUEST time+80s. This is a separately recorded
algorithm correction on top of the exact16ad import, NOT a change to the donor branch
or the shared protocol crate. The red test patch/log against678081c remain
kept source-bound.

A separate read-only AI code check on sourcef16062b found a concrete
delayed-renewal case: after acceptance of a new lease T1 can already have passed
and the pure client immediately returns RenewalDue instead of Bound.
Handling only Bound left the old gateway/DNS visible while the client
internally already used the new lease. Targeted runtime reproduction was red;
the coordinator now applies accepted renewal ACKs coherently in both phases.
The case keeps the new T1 and expiry; no lease extension or success fallback.
This is a bounded second AI review, not a human audit/native acceptance.

## TASK-N01-DHCP-DECLINE-SEAM

Explicit extra assignment:
queue `dea97f9b57c6fda9549a48072119663fe155a563`,
blob `53ac59d53fa7daece42b07925d6ecffe852012f6`.
Exclusively `services/net-protocol/src/dhcp.rs` and `tests/dhcp.rs` on top of
the existing Stack/config claim; no other protocol/root/SDK writes.

`dhcp::build_decline(out,xid,mac,address,server)` reuses the existing
private request encoder/validators. Message4, BOOTREQUEST, zero secs/flags/ciaddr,
requestedIP/server identifier and the same client identifier. Only static
message56 added; NO parameter list55, maxsize57, leasetime51 or other
REQUEST-only options. All checks in advance, no output change on error.
300 UDP data bytes with zero padding and untouched tail beyond the return length.

On a really observed conflict the coordinator first withdraws old address/route/DNS
and TX publication. It keeps only the old lease/XID for DECLINE.
`poll` queues the new broadcast frame only once the existing PacketDevice is
actually free; no completion/discard of the retired frame.
DECLINE also uses the same id/storage/completion. New acquisition waits
at least10s (after10conflicts60s) AND for real DECLINE completion. Unpublished
or EAGAIN remains the same pending frame; Unknown remains terminal.
Linkdown/clock fault keeps no authorized old lease or recovery shortcut.

Three new codec cases check exact packet/header/options/tail,
invalid addresses/MAC and each of the300too-short buffer lengths all-or-nothing.
Fifteen coordinator cases green, including conflict->retired TX->DECLINE->
matching completion->fresh acquisition and existing network lifecycle.
The first API gate was compile101/E0425 (missing function), NOT runtime.
Full source-bound final gates on this source follow after the checkpoint.
No new manifest/lock/dependency/entropy source or network grant, no
native completion/fence/app/network acceptance and no new points.

## Transport under the active lease

The coordinator owns the Stack; without bounded transport pass-through a
native integrator would still need a second Stack or unchecked mutable access.
`add_transport_client`, `connect`, `send`, `receive`, `close`,
`cancel` and `state` therefore share the existing buffers/handles/Stack.
NO mutable Stack/device is lent out. Every operation first processes
the actual monotonic tick/lease expiry; data operations also require the
actually applied active configuration. After withdrawal status/cancel only return
the existing closed state and do not restore a lease.
This remains a trusted-service source interface, not an app grant or broker policy.

Two new coupled host cases show real ARP plus TCP SYN from the same
Stack after DHCP/probes; control backpressure keeps that SYN slot, expiry
closes the socket/hides TX and blocks send/receive/close without output write.
An unchanged renewal keeps a live socket; a new gateway
retires it and shows the new DNS/route only after old completion.
This proves no native connection, application answer or rights enforcement.

## Automatic service steps with explicit entropy source

`automatic_link_up(ticks)` first withdraws the old lease and activates new
acquisition; `link_down` switches those steps off again. They are real
link notifications from the trusted caller, not a rights/device reset attestation.
`drive(ticks, &mut RandomSource)` always yields an explicit result:
LinkDown, WaitingUntil, Backpressure, Published or Advanced. No busy spin.
The integrator first feeds ingress received on that tick and real completions,
then at most one bounded drive step. Do not let another owner
call start/send manually at the same time.

Newly activated acquisition waits uniformly1..10s, supplies a fresh XID and
new probe intervals. DISCOVER/REQUEST retry uses randomized
exponential backoff4,8,16,32,64s with at most1s jitter and an absolute64s cap.
The same XID/REQUEST lease clock is kept on retransmission.
Renewal/rebinding waits the recommended half of the remaining T2/expiry duration,
with a minimum of60s and at most1s positive jitter, but NEVER beyond T2/expiry.
The lease transition takes priority over an old retry deadline.

Correction after review of #113: the retry schedule (`due`/`backoff`) depended only
on the phase. A new renewal transaction that after an ACK started directly from
RenewalDue in the same phase (Renewing->Renewing or Rebinding->Rebinding)
thus inherited the deadline of the previous lease. That could lie after the new expiry,
so that an accepted lease expired without a single REQUEST. The schedule is now
bound to the XID of the transaction and is reset on every new XID.
Host evidence (no native evidence) is provided by
`renewal_retry_schedule_is_bound_to_the_new_transaction_not_the_old_lease`.
Before the fix `drive(7300)` returned WaitingUntil(8100), beyond the new
expiry of 8000 (the test failed). After the fix drive(7300) directly publishes
XID+2 with waiting time until 8000. The Renewing variant in the same test publishes
directly and then waits until the new T2 (10100); the review reproduction saw there
the old deadline 7100 before the fix.
Timeout/NAK/expiry automatically leads back to acquire, only after cooldown and
any DECLINE completion. Reconnect keeps no old configuration; an
old TX remains real backpressure, not faked as completed.

`RandomSource` is an explicit source port with its own error type. There is
NO native implementation here or implicit RNG/RTC/fixed seed/cloud fallback.
Uniform sampling uses rejection sampling (no modulo bias), at most
eight draws; a stuck source returns RandomRejected.
Unavailable entropy returns its real error and publishes no
substitute packet; an existing valid lease is not artificially
extended by it. A new XID must not equal the previous transaction.

Six additional host cases check exact1s/10s startup boundaries,
4/8/16/32/64 backoff and real120s exchange timeout, the same XID, no new
entropy draw/frames on backpressure, bounded bad-source rejection,
real model probes, automatic T1/T2 and reacquire after expiry. No native
event loop, entropy grant, sleep/resume or device fence evidence.
Primarily read: RFC2131sections3.7,4.1,4.4.1and4.4.5.
