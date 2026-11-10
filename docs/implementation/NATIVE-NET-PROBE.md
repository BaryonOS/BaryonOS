# Native network trial within the joint system profile

Task contract, 29 October 2026. App/test peer/runner: `network_nas_ai`;
kernel/capabilities/SDK network ABI: `kernel_storage`; build/manifest
integration and joint acceptance: lead agent. Write scopes are separate.

## Actual path and profile

A fifth ring-3 process uses ABI 1.5 and a manifest-only `NetworkSource` to
send and receive real Ethernet frames through the project's kernel broker
and virtio-net driver. ARP, UDP and DNS are handled in the app by the existing
`no_std` codec. Normal init, key, volume, app and worker scenarios remain
active simultaneously. This is not a complete network stack/N01: TCP, TLS,
DHCP, general configuration and the permissions portal remain open.

QEMU q35/TCG/one CPU/256 MiB, NIC `00:07.0`, MAC `02:00:00:00:00:07`.
Only a QEMU socketnetdev to a temporary `127.0.0.1` test peer; no TAP, bridge,
WAN, real NAS, personal content or external accounts. Client IP `192.0.2.2`;
explicit fixture next hop `192.0.2.1` with MAC `02:00:00:00:00:01`;
DNS fixture `192.0.2.53` through that next hop. No MAC/IP pair is presented
as an authenticated NAS/TLS identity.

## Interfaces and scenarios

Startup: network capability at offset 48, separate endpoint handle at 56 for
wrong-type rejection, own PID at 176. No init IPC or simulated DONE: the
kernel waits for actual exit and checks its own broker counters.

- `net_send(source, frame, timeout_ticks)` publishes and returns one ticket;
  `net_poll_tx(source, ticket)` confirms only device completion. After
  publication, the same frame is not automatically repeated.
- `net_receive(source, output)` returns one fully validated frame or `EAGAIN`.
  No space or pointer error may consume a frame.
- Three real ARP requests: the peer replies with a source-header mismatch,
  a wrong target and then a valid reply. The native app must prove two
  rejections and an unchanged cache before using the correct next hop.
- One UDP echo to port 9000. The peer echoes the actual received payload,
  including the app's own PID added by the app; the app compares all bytes.
- Three DNS requests for `native.nieuw.test`: first a wrong transaction ID,
  then a compression cycle, then a valid A record `198.51.100.42`. Only the
  last may produce a resolution result.
- One query for `timeout.nieuw.test` deliberately receives no reply. The app
  cancels after a fixed receive-poll budget; this proves bounded waiting,
  not the still-absent general system-time/resolver-timeout API.
- Seven exactly matched raw ABI errors: wrong capability type `-6`, bad TX
  pointer `-3`, TX length 1515 `-11`, bad RX pointer `-3`, RX capacity 1515
  `-11`, zero ticket `-1`, reuse after closing `-4`. SDK, kernel and driver
  permissions are not relaxed for these cases.

## Evidence and acceptance

The kernel observes app-private evidence words 400 through 480 and compares
its own TX/RX/denial/cleanup observations. Required expectations:

| Counter under `SYSTEM:` | Value |
|---|---:|
| `NET_ARP_REPLIES`, `NET_UDP_ECHOES`, `NET_DNS_ANSWERS` | 1 each |
| `NET_DENIED` | 7 |
| `NET_TX_COMPLETIONS` | 8 |
| `NET_RX_FRAMES` | 7 |
| `NET_ARP_REJECTS`, `NET_DNS_REJECTS` | 2 each |
| `NET_RX_TIMEOUTS`, `NET_CACHE_INTACT`, `NET_STALE_CAP` | 1 each |
| `NET_APP_DENIED` | 7 |
| `NET_TX_SUBMITTED`, `NET_TX_COMPLETED` | 8 each |
| `NET_RX_COMPLETED`, `NET_RX_DELIVERED` | 7 each |
| `NET_TX_TIMEOUTS` | 0 |
| `NET_RESETS`, `NET_RESET_ACKS` | 1 each |

The host peer independently reports the eight actually received requests and
seven sent replies, with packet hashes and parsed questions. Its protocol
roles are solely counterparty and fault injector: it performs no client
ARP/DNS and never writes kernel/app success markers.

`run_net_system.py` reuses the guarded `run_system` image, source, ELF, exit,
storage-oracle and cold-recovery checks. It additionally requires exact
network evidence and a peer transcript. A failed network check also sets the
storage-stage handoff to `passed=false`, so recovery cannot use a partially
successful normal run as a fully successful predecessor.

Host tests check the peer as a parser/framer/test harness, with independent
wire fixtures, fragmented framing, checksums, length bounds and hostile
input. A target build proves only compilation. Approval of this slice
requires real native execution of both normal and cold read-only recovery
on the same disposable image, with exact commit/configuration/image hashes.

## First actually executed native pair

The kernel agent executed both actual QEMU runs on source fingerprint
`355c111fdaeed5dd7e7f487cbd702fba362d7fcec1c1376db6b60b5f3bcd17d8`:
normal 54.47 s and cold read-only recovery 60.02 s, both raw exit 33 and
`network_gate=true`. The disk hash after normal and before/after recovery
is in each case
`ad40c83d6b9d0a409391113f7ed4deb5aafd7c27f6629c22d7a6ddc060c639ea`.
Recovery performs zero writes and zero flushes. In both runs, the actual
eight transmissions, seven received frames, error paths and independent
server observations were checked alongside the complete existing system trial.

The actually tested probe ELF has hash
`36a57a9163777e2b0db9c7fda3426f6c50dc80153fe23290e5b46bf48f470273`.
A separate build in another worktree has the same app source and loaded
code/data but different rodata because of absolute build paths. This is an
explicit reproducibility boundary; the other ELF receives no native evidence
from this run. Path remapping remains a separate build improvement.

This pair used the joint development source frozen at that time. The final
merged root source and strengthened harness must be executed natively again;
old logs or host reassessment do not close that integration gate.

## Firmware isolation after integrated failure reproduction

On root 309ff53, the strict peer closed before OS ARP because of actual OVMF
IPv6 traffic (EtherType 86dd, MLDv1 membership report). The failed reproduction
and bounded frame bytes/hashes remain
[stored](../verification/full-release-net-firmware-diagnosis/README.md).
The test profile disables only the preboot firmware drivers through
`-fw_cfg name=opt/org.tianocore/VirtioNetSupport,string=no`, `IPv4Support,string=no`
and `IPv6Support,string=no`, plus `romfile=`. These are documented
[EDK2 runtime options](https://github.com/tianocore/edk2/blob/edk2-stable202502/OvmfPkg/RUNTIME_CONFIG.md);
[the stable202502 driver configuration](https://github.com/tianocore/edk2/blob/edk2-stable202502/OvmfPkg/Include/Dsc/NetworkComponents.dsc.inc)
binds them to the entry-point wrapper. The PCI NIC remains present and
Baryon OS's own driver performs all native TX/RX. The peer continues to reject
every unknown EtherType; no whitelist, simulated network or changed 8/7
acceptance. This is only a bounded VM profile, not a claim that physical
firmware on other devices cannot send preboot traffic.
