# N01 — isolated native network service

## Task and evidence boundary

Baseline `0fe648b87d1f4959087d3f80a251a7d6fb64dd66`, branch
`codex/n01-native-service-20261102`. Goal: a reusable separate ring-3 network
service with two independent IPC apps. Only the service receives NIC and
entropy access. TCP uses the existing `net-stack` with smoltcp 0.14.0; UDP
uses the existing net-protocol encoder/parser and solicited ARP cache. No new
registry crate or existing kernel as a foundation. The kernel remains on
Rust 1.90; this app workspace uses the already permitted Rust 1.91.1,
without exchanging rlibs.

Excluded: TLS/PQC handshake (N03), DHCP/everyday internet configuration (N02),
a general dynamic policy issuer or network revocation (N04), physical Wi-Fi
or new syscalls. An IP/MAC address is not a cryptographic peer identity.
The normal proof uses only synthetic packets through the QEMU test peer.

The service/SDK/wire have 7/6/4 targeted host tests respectively, including
two actual smoltcp TCP connections with full 4096-B buffers, recovery and
closing. Strict Clippy passes for those three packages. The complete kernel
1.90 build with native Rust 1.91.1 service/client ELFs passes. The source-bound
native gate passed on `f152aec0615285065689f0a7e2855e5858fdb350`: three real
processes, ten TCP scenarios and both UDP error bursts, exit 33 after
8.413 seconds. The second native gate closes A's reply channel during a
request; B then completes its entire route (A exit tick 32, B tick 470),
exit 33 after 7.538 seconds. Source hash
`56ed10aade50703e08c8422251e323b4400bf37196320d24befe0b46883bbce8`.
Stored evidence is in `docs/verification/n01-native-service-20261102/`.
This document awards no points; the lead agent must rerun the gate on the
merged source.

## Trust boundary and interfaces

- Kernel bootstrap binds two actual app PIDs and, per app, a separate RECV
  and fixed SEND reply endpoint. Untrusted requests supply no PID/reply capability.
- Each app may use only its bootstrap-selected IPv4 address and five TCP
  ports, plus one UDP port. Apps receive no NetworkSource/Entropy.
- One TCP socket per app, 4096-B RX/TX, at most 208 B of data per IPC. UDP has
  one 208-B receive slot per app; backpressure does not overwrite the old
  datagram.
- All datagrams are unprotected transport bytes. No listen/server, IPv6,
  IP fragmentation, host socket or silent fallback by default.
- TCP connection tokens are generation+slot and are always checked against
  both the actual sender and service epoch. They are not independent
  capabilities.
- SDK and service do not repeat an IPC mutation already queued after a lost
  reply. A network TX slot is released only after actual NIC completion.
- Each app has 12,768 disjoint, non-reused local TCP ports per boot; exhaustion
  returns Status::Exhausted only for that app. No global service fault.
- A permanently lost app channel retires only that slot. An already published
  NIC transmission remains live until actual completion; the other app
  continues. Unrecoverable shared-device errors remain network-service errors.
- Service processing is bounded: one packet step and at most one request/
  reply per app per turn; A's full reply queue does not block B.

NET1 tag is `u64::from_le_bytes(*b"NETRPC01")`, payload exactly 256 B:
`NET1`@0, version u16=1@4, operation u16@6 (`|0x8000` for a reply),
request u64@8, epoch u64@16, connection u64@24, status u32@32,
argument u32@36, IPv4@40, port u16@44, reserved0@46, data208B@48.
All unused bytes must be 0. Ops 1 OpenTcp, 2 SendTcp, 3 ReceiveTcp,
4 CloseTcp, 5 State, 6 SendUdp, 7 ReceiveUdp, 8 Retire.
Open.argument is timeout 1..6000 ticks (100 Hz); Send.argument is data
length ≤208. State returns Closed0, Connecting1, Established2, PeerClosed3,
Closing4, TimedOut5. Retire closes only the app's own authority; no app can
stop the service globally.

SDK: `Session::new(Transport,epoch,service_pid,SendEndpoint,ReceiveEndpoint,budget)`;
`open/state/send/receive/close/send_udp/receive_udp/retire`. Buffer errors
are rejected before publication. Reply peer/tag/no-capabilities/request/
epoch/op/token are checked. As with the existing catalogue client, Transport
is a trusted adapter; self-constructed Rust values do not prove a kernel
sender.

## Native startup contract

All startup fields are u64 at offsets in the existing private 1024-B startup
page. The service validates separate handles and different service/app PIDs.

| Service offset | Meaning |
|---:|---|
|32|`u64::from_le_bytes(*b"NETSVC01")`|
|40|service epoch >0|
|48|NetworkSource|
|56/64/72|app A RECV / replySEND / actual PID|
|80/88/96|app B RECV / replySEND / actual PID|
|104/112|EntropySource / servicePID|
|120/128|first TCP port A/B; five permitted ports each|
|136/144|exact UDP port A/B|
|152/160|optional controllerRECV / actual controllerPID; both 0=absent|
|168|test-only device-reset terminal observation; 0 in other profiles|

The link profile is the existing NIC profile MAC02:00:00:00:00:07,
192.0.2.2/24, peer192.0.2.1. The library supports other validated on-link
configurations; the native proof records this explicit fixture configuration.
The service obtains 8 bytes through the actual EntropySource for the smoltcp
seed; no fixed trial seed as a production ISN claim. MAC/ARP provide no
authentication.

App offset 32=NETAPP01, 40 epoch, 48 SEND, 56 RECV, 64 servicePID,
72 slot0/1, 80 actual ownPID, 88 own TCP starting port, 96 own UDP port.
Offset104 selects normal (0), the app A client-failure trial (1), live
revocation (2), or the device-reset containment trial (3). In the reset trial,
app offset112 synchronizes injection and observed service exit; both clients
must still make an actual call on the old ingress handle. Both apps use the
same ELF; the actual bootstrap slot selects their own test data.

Optional controller tag NETCTL01, payload 16 B: `NETSTOP1`@0, epoch u64@8.
Only the bound controller may send it, without capabilities. The service
stops new requests, closes sockets and waits for actual TCP/NIC drain before
exit 0. Normal apps terminate themselves with Retire; when both apps are
retired, all pending replies have been sent and all sockets/TX are complete,
the service closes its own handles and ends with exit 0.

## Native peer and acceptance sequence

App A TCP base 8080, app B 8090; for each app, offsets mean:
0 normal HTTP, 1 SYN refusal (RST), 2 RST after request, 3 discard the first
SYN then HTTP, 4 ignore all SYNs until an actual 300-tick timeout. Native
apps exercise those five cases independently and sequentially; their
processes run concurrently. HTTP requests are exactly
`GET /app-a HTTP/1.1\r\nHost: fixture.invalid\r\nConnection: close\r\n\r\n`
or `/app-b`. Reply exactly
`HTTP/1.1 200 OK\r\nContent-Length: 19\r\nConnection: close\r\n\r\nNieuw OS service A\n`
or B. UDP9000/9001 echoes exactly `Nieuw OS UDP app A` or B to 45000/45001.
ARP must actually resolve the peer MAC; no fixed MAC in the service.
After each actually received guest UDP packet, the peer sends four bounded
frames: invalid checksum, wrong source port, valid echo and a second valid
datagram with different content. Each app waits 25 actual monotonic ticks
before receiving. Both first echoes must be preserved; the service counts
at least four malformed/scope rejections and two drops from full UDP slots.

Apps also check a wrong TCP port, another slot's connection token, old tokens
after reopen, malformed reserved bytes before SDK startup and a rejected
raw NIC call with an IPC handle. That final rejection grants no hardware
capability.

## Observation and building

Service words: 400 ready, 408 loop, 416 actual TX completions, 424 RX frames,
432 requests, 440 rejected, 448 UDP queued, 456 UDP received, 464 TCP opens,
472 TCP send bytes, 480 TCP receive bytes, 488 close calls, 496 retired mask,
504 fatal, 512 malformed frames, 520 UDP drops, 528 clean exit,
536 lost-app-channel mask. The native adapter publishes all values from
actual service/device operations. No plaintext payload logs.

App words: 400 ready, 408 HTTPcomplete(2), 416 UDPcomplete(1),
424 scopeDenied(1), 432 staleDenied(≥5), 440 malformedDenied(1),
448 sendchunks, 456 last HTTP response bytes, 464 gracefulclose(2),
472 reopens(4), 480 rawNICDenied(1), 488 reset(1), 496 timeout(1),
504 retransmit(1), 512 refused(1), 528 done, 536 closed-reply trial.
The kernel also emits the actual A/B exit ticks. Fatal code is at 240. Root
must record these words before process cleanup, as with existing probes;
a dead address space must not later be read back for evidence.

`python3 sdk/examples/network-service/build.py` builds both actual ELFs with
Rust 1.91.1/matching rust-lld/locked-offline/W^X linker script. Output:
NOS_NETWORK_SERVICE_ELF and NOS_NETWORK_CLIENT_ELF. Targets are only
x86_64-unknown-none; compilation/ELF inspection is not native execution.

## Reproducible gates

Activate the existing development environment. The three packages have separate
workspaces/lockfiles; the kernel remains on Rust 1.90, smoltcp on Rust 1.91.1.
The test-only smoltcp dependency uses exactly the same pinned 0.14.0 version
and features as the existing net-stack, without a new registry dependency.

```
cargo +1.90.0 test --offline --locked --manifest-path services/network-service/protocol/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/net-client/Cargo.toml
cargo +1.91.1 test --offline --locked --manifest-path services/network-service/Cargo.toml
python3 -B -m unittest discover -s tools -p test_network_service_peer.py
python3 tools/run_network_service.py --case normal
python3 tools/run_network_service.py --case client-exit
python3 tools/run_network_service.py --case revoke
python3 tools/run_network_service.py --case device-reset
```

`native-net-service` inherits only existing kernel mechanisms; a separate
manifest boots exactly three actual processes. No storage service starts
and no block device is discovered or mapped. For existing compile-time
fixture references, the build path also creates the unchanged system ELFs,
but those are not started in this manifest. The only devices are the read-only
disposable ESP, virtio-rng and virtio-net with only the local synthetic socket
peer. No TAP, bridge, host share, ordinary internet connection or real disk.
The kernel retains observation words before generic process cleanup and
checks capability/frame cleanup and actual NIC reset acknowledgement. The
Python gate retains the original bounded stdout/stderr bytes, rereads them
byte-for-byte and decodes only valid UTF-8. It compares serial output, three
process observations, NIC completions and causally replayed actual packets.
It checks source, build, ELF, firmware, QEMU and stored-output binding before
and after execution.

### Unexpected device-reset containment

`test-net-service-device-reset` enables a driver fault hook only for this
native profile. After a real TCP connection and held HTTP request, the hook
writes the actual virtio device-status register from 15 to 0 immediately
after a further TX descriptor has been published. It leaves the software
pending ticket and bus-master state intact. The ordinary driver health and
broker paths must detect the fault, disable bus mastering, acknowledge reset and return
`Io` for that same ticket exactly once. Permanent DMA pinning is unchanged.

The peer permits the uncertain final frame zero or one time and rejects a
replay. The oracle requires no further TX publication or RX delivery after
the fault. A receive error may precede the next TX poll: only the reset
fixture then records the actual RX `Io` and performs one real poll of its
original pending TX ticket before retaining the existing fatal service exit.
Service words 520/536/544/552 record the RX error, terminal-observation tick,
TX error and ticket in this profile. Both clients must observe actual ingress
closure after service exit, and the unchanged generic cleanup checks must
pass. A normal or revoked service retains its existing exit contract.

This is a deterministic fault injection into a real QEMU device, not a
physical-device reset trial or an automatic reconnection/restart guarantee.
The source-bound execution receipt and the retained first failed run are
described in [network-reset.md](../verification/issue43-20261114/network-reset.md).

Source orientation: OSDev Network Stack, W002538, revision23577
(16 April 2019). Existing project drivers/ABI and the checked smoltcp/
net-protocol code remain authoritative; no corpus examples were incorporated
or executed.
