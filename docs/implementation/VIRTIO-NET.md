# FR-NDRV — native virtio-net component

Task contract, 29 October 2026. Owner: `network_nas_ai`; integration: lead agent.
Write scope: `drivers/virtio-net/**` and this document. No kernel, syscall,
root-workspace or capability change in this commit. The lead agent later
connects the driver to a native broker and network service. N01 remains open.

## Profile and interfaces

- Dedicated modern PCI driver for QEMU q35, fixed BDF `00:07.0`, vendor `1af4`,
  device `1041`; one CPU, 256 MiB identity-mapped RAM, MMIO supervisor/UC.
  Reuse the existing `virtio-core` capability parser through a local path
  dependency.
- Requires `VERSION_1` and `MAC`; optionally negotiates `STATUS`. No checksum,
  segmentation, mergeable-buffer, multiqueue, indirect-descriptor or
  control-queue features. MTU 1500, Ethernet at most 1514 bytes, no FCS.
- The modern virtio-net header is **12 bytes**, even without `MRG_RXBUF` when
  using `VERSION_1`. Two split queues: RX 0, TX 1; eight descriptors each,
  initially one published buffer per queue. Larger buffers/more outstanding
  frames remain a later performance step, not a permanent product limit.
- `Discovery`, `NetPool`, `Device::initialize`; then `post_rx`, `poll_rx`,
  `send`, `poll_tx`, `link_up`, `begin_reset`, `reset_acknowledged`.
  `send` confirms only queue publication; `poll_tx` only device completion,
  never delivery to a peer. No built-in echo or fake ACK.

## Ownership and errors

- Four distinct aligned and permanently pinned RAM pages: RX ring, TX ring,
  RX bounce, TX bounce. Per ring page: descriptors at 0, available at 256,
  used at 512; all regions fit within their own page.
- No app pointer, protocol parser, key or route policy in the driver. The
  caller retains exclusive supervisor I/O ownership; buffer content is
  transferred through volatile byte copies, not shared Rust references to DMA.
- Descriptor ID, wrapping used-index delta, used length and network header
  are validated before copying. No received offload that the profile cannot
  process; insufficient output space produces no partial copy.
- TX has a monotonic deadline. A timeout or protocol error stops the driver,
  requests reset and disables PCI bus mastering. A TX may already have been
  sent at that point; no automatic replay. An empty RX queue is normal, not
  a fault: request timeouts are handled by the broker while the RX page
  remains pinned. A published RX buffer cannot be reclaimed without device
  completion/reset.
- Reset is confirmed only when device status is zero **and** bus mastering
  is off. No reuse until that confirmation; this profile additionally keeps
  all DMA pages pinned until reboot, including after failed initialisation.
  No IOMMU or protection against a malicious physical device is claimed.

## Acceptance and evidence boundary

Host register/DMA models exercise the same driver logic: features,
configuration generation, descriptor publication, actual byte copying,
ring wrap, wrong index/ID/length, offload rejection, smaller output,
TX deadline and reset ACK. Label these tests as **models**, not actual NIC
execution. Also actually compile for `x86_64-unknown-none`; Clippy and
formatting remain mandatory.

Native follow-up: the kernel connects pinned pages and authorised processes;
QEMU with a bounded test peer must demonstrate actual TX/RX, including a
missing device, rejection, timeout and recovery. No host network stack may
perform missing OS functionality. Only then follow ARP/DNS and the remaining
N01/N02/N03 gates.

## Sources

- OASIS, *Virtual I/O Device (VIRTIO) Version 1.2*, split virtqueues and
  network device §5.1; existing project baseline `virtio-core`/block driver.
  https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html
- QEMU Project, tag `v10.0.2`, `hw/net/virtio-net.c`,
  `virtio_net_set_mrg_rx_bufs`: `VERSION_1` selects the 12-byte
  `virtio_net_hdr_mrg_rxbuf`, separately from `MRG_RXBUF`.
  https://github.com/qemu/qemu/blob/v10.0.2/hw/net/virtio-net.c
  Primary emulator source read 29 October 2026; no QEMU code incorporated.

Execution, exact source hashes and unexecuted native checks are recorded
in `drivers/virtio-net/verification/` after the build step.

## Build and integration

The new crate is named `virtio-net-driver` and depends only on the existing
local `virtio-core`. At integration, the lead agent adds `drivers/virtio-net`
to the existing root workspace and lockfile. For the isolated agent build,
exactly that member was temporarily added only in the dedicated worktree;
both shared files are restored byte-for-byte before this partial commit.
No new external dependency or build script is added.

After workspace integration, the targeted checks are:

```sh
cargo +1.90.0 test -p virtio-net-driver --locked --offline
cargo +1.90.0 clippy -p virtio-net-driver --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build -p virtio-net-driver --lib --release --target x86_64-unknown-none --locked --offline
cargo +1.90.0 fmt -p virtio-net-driver --check
```

`post_rx` publishes one complete RX buffer; `poll_rx` returns a frame only
after a validated actual used completion. With an undersized output buffer,
the same completion remains available for a larger buffer; nothing is copied
or republished yet. `tx_pending` and `rx_posted` show only the local queue
phase, not connection status.

A changed used index while the queue is idle is rejected before new
publication: an old/unsolicited completion must not be attributed to the
next request. `DEVICE_NEEDS_RESET` or another invalid device status stops
mutating/polling paths; `link_up` returns `Io` while that state exists.
A link bit alone does not mean the device is ready.

The compact result is in
[`driver-acceptance.json`](../../drivers/virtio-net/verification/driver-acceptance.json).
The host tests are explicitly register/DMA models; physical PCI, native RX/TX,
QEMU peer connectivity and permission enforcement were not tested for this
slice.

## H04-T3: link loss and reconnect (test profile)

Task H04-T3, 19 November 2026. Scope: kernel feature `test-system-net-link`
(and `test-system-net-link-storm`) in the joint `native-system-net` profile.
Without these features the broker, the 0x70–0x72 ABI and every existing
profile are unchanged; see the evidence README for the section-level binary
comparison.

### Driver interfaces

- `Device::link_status() -> Result<(u8, bool), Error>`: one coherent sample of
  `(config_generation, LINK_UP)` with at most `LINK_READ_ATTEMPTS` (16)
  generation-bracketed reads, following the generation read-retry pattern of
  virtio 1.2 §2.5.1. Read-only: no status write, queue access, DMA or
  doorbell. It returns `Offline` before initialization, `Io` when the device
  status is not `DRIVER_OK`, `Unsupported` without `VIRTIO_NET_F_STATUS` and
  `Busy` when the generation never settles. `link_up` now delegates to it.
- `Device::negotiated_features()` exposes the fixed accepted feature bits.
- `Device::publications() -> (u16, u16)` exposes the wrapping RX/TX available
  indices. Each publication increments exactly one of them and rings that
  queue's doorbell exactly once; an unchanged pair proves no publication.
- `LinkMonitor` (pure, host-tested) wraps the H04-T2 `link::Tracker`. The
  kernel owns the strictly increasing observation serial and link epoch; the
  wrapping u8 generation is only a change hint, never a count or serial. A
  generation change without a bit change records exactly one conservative
  synthetic opposite/actual pair (epoch + 2, two budget units) regardless of
  the u8 delta. Exactly 256 changes between two samples are undetectable by
  design. Reset (`latch_reset`) or an exhausted budget latches: no state or
  event is exposed afterwards.

### Kernel broker invariants (test profile only)

- One link sample per PIT tick (100 Hz), at most two events per sample, a
  fixed lifetime budget of eight transitions after the first sample. `Busy`
  is counted and never produces an event.
- `0x70` (send) and `0x72` (receive) return **-23 (link down or not yet
  observed)** after the existing argument, capability and offline checks and
  before any descriptor publication, bounce-page (DMA) write, doorbell or
  copy-out. Precedence: argument errors, then capability errors, then
  Offline (-14), then -23, then Again (-8).
- At every down edge an undelivered RX copy is zeroed and dropped, and the
  same pinned RX page is handed back to the device; RX completions observed
  while the link is down or unknown are dropped the same way. No DMA page is
  freed or reused for another purpose.
- A completion that is already in the RX used ring at a link edge, down or
  up, is taken into the empty broker buffer at that edge and dropped with
  it. The broker polls RX only after the link sample, so without this step:
  - a generation-only sample (synthetic down, then up, applied back to back)
    would let a frame that completed before the down edge be delivered in
    the new epoch;
  - a real up edge seen from a down epoch would do the same. QEMU accepts
    RX while its own link is up, so `set_link` on, a peer frame, `set_link`
    off and on again between two samples collapse into one real up edge.
    The bit changed, so there is no synthetic pair, and the frame from the
    hidden session is still in the used ring.

  Every completion present at an up edge arrived while the kernel epoch was
  down or unknown. The cost is at most one legitimate frame per up edge,
  because there is a single RX buffer. A failed RX poll there stops the
  broker. At a down edge the take, the drop and the repost happen before
  the down window opens; at an up edge they happen after the window has
  closed. Neither repost is counted as a down-window publication.
- An in-flight TX is not terminalized at a down edge. QEMU drops queued TX
  data while the link is down and completes the descriptor with `used.len`
  0 (`virtio_net_drop_tx_queue_data`), which `validate_used` already accepts
  for every TX: this is ordinary TX completion, not link-specific behaviour.
  The native cases do not exercise an in-flight TX at a down edge.
  Decided, not implemented: amendment H04-TXDOWN of ADR-H04-LINK-20261119
  (27 November 2026) reports such a completion as -27 when a kernel link
  edge lies between publication and collection; TXDOWN-1/2 are NOT RUN.
- Exhaustion or any broker reset latches the monitor; the kernel runs the
  existing `stop()` (reset request, bus mastering off, pinned pages kept,
  reset acknowledgement awaited, no reinitialization). Later link changes
  produce no event. 0x74 returns Offline (-14) after any stop, quarantine or
  monitor latch, whatever the cause of the stop. 0x70/0x72 keep the
  broker's existing offline code: -14 after exhaustion or owner exit, and
  the mapped stop cause otherwise (for example -12 after a device-status
  error).
- Without `VIRTIO_NET_F_STATUS` the link is assumed active (virtio 1.2
  §5.1.4.2, "Driver Requirements: Device configuration layout"), no sample
  is taken and 0x74 returns -15 (the broker's existing mapping of the driver's
  `Unsupported`). For 0x74, -15 has no other meaning: a stopped or
  quarantined broker returns -14 even when its stop cause maps to -15.

### Test-only ABI

`0x74(handle, 0, 0, 0, 0, 0)`: the same NetworkSource capability and object
generation key as 0x70–0x72 (`authorize`); every reserved argument must be 0
(-1). The result encodes only `epoch << 1 | up`; unknown state returns -8,
a device without `VIRTIO_NET_F_STATUS` -15, and a stopped, quarantined or
latched broker -14.
No registers, MAC, generation value or rights are returned. The syscall
exists only with `test-system-net-link`; other kernels return NoSys. It is
not part of a published ABI version (`abi_info` stays 1.5); a product link
API needs a separate decision.

### Observed QEMU 8.2.2 behaviour

`set_link net0 off|on` on the socket netdev also sets the peer NIC's
`link_down` and calls `virtio_net_set_link_status`, which flips
`VIRTIO_NET_S_LINK_UP` and, only when the bit changes, notifies a config
change (config generation increment). Repeated identical `set_link` calls do
not change the generation. In the handshaked native runs every toggle was
observed as exactly one real event; no synthetic pair and no `Busy` sample
occurred, so those paths are host-tested only. QEMU 8.2.2 provides no 802.11
or Bluetooth device model; Wi-Fi and Bluetooth are outside this evidence.

Native evidence: [`h04-native-link-20261119`](../verification/h04-native-link-20261119/README.md).
QEMU/TCG only; no physical NIC, cable or link partner was tested. H04 remains
open.

Additional sources, read 19 November 2026:

- OASIS, *Virtual I/O Device (VIRTIO) Version 1.2*, Committee Specification
  Draft 01, §2.5.1 and §5.1.4.2.
  https://docs.oasis-open.org/virtio/virtio/v1.2/csd01/virtio-v1.2-csd01.html
- QEMU Project, tag `v8.2.2`, `hw/net/virtio-net.c`
  (`virtio_net_set_link_status`, `virtio_net_handle_tx_bh`,
  `virtio_net_drop_tx_queue_data`).
  https://github.com/qemu/qemu/blob/v8.2.2/hw/net/virtio-net.c
