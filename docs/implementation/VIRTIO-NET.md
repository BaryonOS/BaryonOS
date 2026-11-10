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
