# TASK-NATIVE-NET — real packets in the combined system

30 October 2026, work package for FR-N01/FR-N02/FR-H03 and FR-K03.
This is a targeted native integration step; the full network/TLS/NAS
acceptance in FULL-TEST-RELEASE.md remains mandatory.

## Boundary and ownership

The opt-in profile `native-system-net` adds a real modern virtio-net on
PCI `00:07.0` to the existing init, key, storage and app processes.
A separate ring-3 network trial is the only one to receive `NetworkSource`.
App/process supervision keeps working while that trial processes packets.
The existing `native-system` does not change into a network profile.

Kernel worker owns kernel, capability-core and SDK runtime. Network worker owns
the ring-3 trial and the bounded external protocol counterpart. Root owns
central build/run tools, shared manifests/locks, integration and final evidence.

The new kernel object has `NETWORK=2048`, no derive/transfer/revoke;
closing and owner exit end the authority. This raw device object is only
for the network service, not the eventual app/destination grant. Ordinary apps
get no raw packets, DMA or general network authority.

## Native ABI and device boundary

ABI minor 1.5 adds typed SDK operations, without changing Message320:

- `0x70`: send(cap, pointer, length, timeout 1..500 ticks) → ticket.
- `0x71`: poll_tx(cap, ticket) → sent frame bytes or EAGAIN.
- `0x72`: receive(cap, pointer, capacity) → complete frame or EAGAIN.

Length at most 1514 bytes. All user mappings and rights are checked before
publication or buffer modification, also across page boundaries. Data
is copied to/from four permanently pinned driver pages. The
privileged code processes no ARP, IP, UDP or DNS policy. The current small
broker plus virtio driver does remain part of the examined TCB; native
userspace driver/IOMMU isolation is not claimed by this.

Only the new profile gets its bounded UC/NX MMIO window from
`0x3f100000`. MAC `02:00:00:00:00:07` and VERSION_1/MAC/STATUS choices are
actually checked. An RX waiting period is not a safe DMA reset.
An uncertain TX is not repeated; memory stays pinned until reset ACK or
quarantine. Completion means device completion, not receipt by the peer.

## Counterpart and acceptance

QEMU connects exclusively via its own loopback socket to the test server started by the
runner; no TAP, bridge, router change or default guest network.
The server speaks the Ethernet frame protocol, delivers test replies and keeps
its actual transcript. It performs no OS client, name resolver or
rights check for the guest.

The guest performs real ARP, UDP echo and DNS A resolution with synthetic
192.0.2.0/24 addresses. Wrong sender/target, wrong DNS ID,
compression loop and an unanswered query produce bounded errors. The cache
result must not be overwritten by wrong ARP replies.
The trial also checks cap type, pointers, oversized frames, ticket zero and
use after close. The independent peer check compares real packets;
the system gate additionally keeps its disk, crypto and cleanup conditions.

Expected additional figures: ARP_REPLIES=1, UDP_ECHOES=1, DNS_ANSWERS=1,
DENIED=7, TX_COMPLETIONS=8, RX_FRAMES=7, ARP_REJECTS=2, DNS_REJECTS=2,
RX_TIMEOUTS=1, CACHE_INTACT=1, STALE_CAP=1 (prefix `NET_`). Those values are
requirements recorded in advance, not results already obtained.

## Open boundaries

No full N01/N02 sign-off: TCP, user configuration, routes,
general resolver, TLS/PQC/time/certificates, app destination policy,
network recovery, NAS and internet use still have to be integrated natively.
The fixed VM profile proves no Wi-Fi, Bluetooth, physical DMA/IOMMU or
general hardware support. Source/image/packet/log hashes and negative
cases are mandatory before this sub-step counts as natively passed.
