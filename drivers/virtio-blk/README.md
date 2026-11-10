# Initial virtio-blk transport

This driver belongs to the M5 development experiment: QEMU q35/TCG, one CPU, exactly PCI `00:05.0`, vendor/device `1af4:1042`, modern virtio 1.2 and a separate read-only 64 KiB test image. The virtio boot ESP is not selected. Only reads are supported, with one split queue of eight descriptors and at most one outstanding sector read.

The driver and protocol-conforming QEMU device/backend belong to the trusted computing base. An actual ring-3 service uses only a typed READ capability and bounded submit/poll syscalls. Userspace receives no authority over PCI configuration, BAR/MMIO, bus mastering, descriptors or DMA addresses. There is no IOMMU initialisation or protection against a malicious device; CPU isolation and checked requests are separate properties.

The kernel retains five separate pinned supervisor-RW/NX pages until VM stop: descriptors, available ring, used ring, request/status and bounce. Header16B is device-readable; bounce512B and status1B are device-writable. The validated chain is `0→1→2`. Only checked BAR subregions receive supervisor-NX/UC mappings, after BAR-bound, PAT-UC and CPU-MAXPHYADDR checks. The page pool is not user memory and never returns to the frame allocator.

Publication follows full descriptor validation and memory barriers. Completion uses volatile snapshots and barriers; head/index/length/status are validated before data is copied. Invalid metadata produces a local protocol error without payload. Fault injection changes only a rejected draft or a local completion snapshot; no unsafe DMA address is published.

Reset first requests virtio status0 and clears PCI bus-master enable. Clearing bus mastering alone does not prove that old DMA has ended. Queue/bounce are reinitialised only after reset status0 has actually been observed. Without confirmation, the pool remains quarantined and the device offline until VM stop. A software epoch/ticket is absent from the used virtqueue and does not demonstrate protection against malicious hardware replay.

From the repository root:

```powershell
python tools/build.py --kernel-feature milestone-m5
python tools/run.py --smoke --milestone m5
```

The complete M5 suite, including variant-specific markers and the external image oracle:

```powershell
python -u tools/test_m5.py
```

The normal smoke CLI expects the READ case. Negative features must use the variant-aware gate; raw33 alone is not a complete test result. All runs use separately generated images under `build/run/`, without networking or host devices. FNV sector checksums are diagnostic test metadata; SHA256 records the unchanged host image. See `docs/implementation/M5-BLOCK-SERVICE.md` for ABI1.1, the 17 error cases, scope and evidence limits.

## M6a WRITE/FLUSH profile

The separate M6a profile selects the same fixed device, eight-entry queue and
five pinned pages. It requires VERSION_1 and FLUSH and rejects a read-only
device. CONFIG_WCE is neither negotiated nor changed; the flush-respecting
QEMU backend is treated as writeback. Existing M5 initialisation and the
read-only fixture remain separate.

WRITE uses a device-readable header16 and bounce512 plus device-writable
status1; FLUSH uses only header16 and status1, with sector0. The unused third
FLUSH descriptor is exactly zero. Successful WRITE/FLUSH has used length1.
The kernel first copies all 512 user bytes into private RAM, then validates
the fixed descriptor chain and marks Published immediately before writing
avail.idx. Notify is not the publication boundary. At most one operation
remains outstanding; do not wait for device I/O with interrupts disabled.

Only WRITE completion followed by actual FLUSH completion permits an app ACK.
Uncertain published WRITE/FLUSH outcomes are not automatically replayed.
Reset establishes quiescence, not rollback or durability. Without reset
acknowledgement, the pool remains quarantined; user frames are never DMA targets.
The M5 trust model without IOMMU remains in force. Aborting a QEMU process
does not demonstrate host power loss or atomic physical-sector writes.

```powershell
python tools/build.py --kernel-feature milestone-m6a
python tools/run_m6.py --smoke
```

Variants must be started with their corresponding feature and `--case`.
Cold recovery uses only the existing guarded Volume through run_m6;
a newly created volume is not recovery evidence. The exact protocol, six
service cuts and six external VM cuts are in docs/implementation/M6A-WRITE-RECOVERY.md.
