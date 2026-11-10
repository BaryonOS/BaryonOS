# TASK-H03-TRANSPORT-CONTRACT-1 — virtio-snd transport, reset, and epoch boundary

Status: **proposed contract; not implemented or executed.** This step changes no
code, IRQ, MMIO, transport, capability, syscall, or shared ABI.
Rubric H03/A04; FR-H02/A06/A07 remain open. The host codec under `src/` is not
native audio. Document owner: TL-HW. Integration and any root, kernel,
capability, or ABI change require the release/kernel manager and a separate
security review; this document grants no write access to those areas.

## 1. Source pins

| Source | Version / hash | Used for |
| --- | --- | --- |
| OASIS, *Virtual I/O Device (VIRTIO) Version 1.3*, <https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.html> | sha256 `3423e9507f2c363fc8e7bbdd013afc0436ed2954b79aa34db55cbf9afa07d45d`, retrieved 30 October 2026 | §2.4 reset (status 0, then no queue interaction), §2.7.8 used ring (no epoch), §5.14.5 initialisation, §5.14.6.6.5 RELEASE, §5.14.6.8 PCM I/O |
| QEMU `v10.2.1` `hw/audio/virtio-snd.c`, <https://github.com/qemu/qemu/blob/v10.2.1/hw/audio/virtio-snd.c> | sha256 `ccb7e3b6ed7937602f86bf2a3badc70e77bd4e63d2dafb06664c717c74f81114` | stream direction, duplicate RELEASE, invalid TX, reset |
| QEMU `v10.2.1` `audio/wavaudio.c`, <https://github.com/qemu/qemu/blob/v10.2.1/audio/wavaudio.c> | sha256 `94ad3a5456465bac6aec549bcb0c5d24790a548c84a47c9caa8954094c3eae54` | observation oracle for the runtime gate (S16 supported) |
| `crates/virtio-core/src/lib.rs` @ `ca45a7d` | sha256 `24dc240458ff4558eac959c0512b11e73ca9b09da28ddd5179bf2b4361cbd293` | `Policy`: epoch, `reset_completed`, quarantine, ticket |
| `drivers/virtio-net/src/lib.rs` @ `ca45a7d` | sha256 `9d6689a2cc9c02e4ebd2f211b3d4760e95da052b168ec3c30691f99fadf8f7b5` | pinned-ring/bounce-buffer pattern, idle used-index rejection |
| `crates/capability-core/src/lib.rs` @ `ca45a7d` | sha256 `9c41ac47e783405901663423b0e0463d5fa665f445abfb8d62de1349debd9fb4` | manifest-only, non-delegable rights (`NETWORK`, `DISPLAY`, `INPUT`) |
| `docs/DECISIONS.md` ADR-026 (M5) and ADR-044 (network broker) | sha256 `651fc7586d42a08fef8c545b5dfc47a3a1ae2dc2b13bee5a72b39cdfa38882d1` @ `ca45a7d` | trusted kernel transport, no IOMMU, reset acknowledgement/quarantine |
| `docs/implementation/VIRTIO-NET.md` | sha256 `2114863edb97b2dbe2310ae0b8ccf78ce371839cb6dbe7527821cb97f1787de6` | bus mastering disabled + status 0 = confirmed reset |
| `docs/verification/m5/transport-review.md` | sha256 `e16d63180001056b4521206ae2a888a2c7abe3098f2e28aae8028312e8a2f06b` | error paths after BME enablement, device scope, orphaned error |
| QEMU detection probe (TL-HW session evidence) | QEMU 10.2.1, OVMF 4M | `1af4:1059` at `00:08.0`, streams 2, jacks 0, chmaps 0 |

Observed QEMU behaviour (source inspection, not execution): stream IDs
`< streams/2 + streams%2` are output streams and the remainder are input
streams; `streams=2` therefore makes stream 0 output and stream 1 **input**.
Stream information exposes `channels_min=1` and sets `channels_max` to the
configured audiodev channel count; it also exposes S16 and 48 kHz. A second
RELEASE of the same stream invokes `virtio_error` (device broken). A TX
request with an invalid or input stream ID completes with status `BAD_MSG`,
although §5.14.6.8 lists only `OK` or `IO_ERR` for I/O.

## 2. Trust model

- **TCB in this profile:** the project kernel (PCI/MMIO mapping, rings, pinned
  DMA pool, usercopy), the protocol-conforming QEMU device, and the audio
  backend. This is the same model as ADR-026/ADR-044; the transport remains a
  **trusted kernel transport**, not an isolated transport.
- **No IOMMU.** VT-d/AMD-Vi support is absent. Bounded descriptors and CPU
  paging do **not** prove containment of a malicious or defective device.
  An epoch or ticket does not make DMA safe; only a confirmed device fence
  (§5) terminates DMA to the pool. Physical hardware is outside this contract.
- **Audio broker (ring 3):** one configured process with a manifest-only,
  non-delegable output right (see open decision B1). It receives no PCI, MMIO,
  descriptor, physical-address, or queue-selection authority. Closing the
  capability or terminating the broker ends the lease.
- **Applications:** no device capability. An application communicates with the
  broker through IPC under a later contract. An advertised PCM format does not
  grant authority to an application; output authority is not microphone
  authority.
- **No capture.** This contract provides no input path; rxq never receives a
  buffer (§3).

## 3. PCI and queue ownership

| Element | Owner | Rule |
| --- | --- | --- |
| PCI function `00:08.0`, `1af4:1059`, modern, BARs | kernel | Fixed BDF for the profile; different ID/BDF or legacy ⇒ `Absent`. The kernel selects features with `negotiate_features` (VERSION_1, never CTLS; ACCESS_PLATFORM ⇒ reject). |
| Config (12 bytes) | kernel, read-only | Generation-stable read; `Config::parse`. |
| controlq (0) | kernel | One outstanding request; device-readable request plus device-writable response in a dedicated page. |
| eventq (1) | kernel | Fixed set of empty, 8-byte, device-writable buffers (§5.14.5 MUST). |
| txq (2) | kernel | Only a device-readable header plus PCM and an 8-byte device-writable status (§5.14.6.8.1.2: no device-writable PCM buffer). |
| rxq (3) | kernel | Ring configured but **no buffer is ever published** (open decision B5: disable instead of configure). |
| Stream selection | kernel through codec | `first_stereo_s16_48000`; never `INPUT`. Proposed VM-FR: `streams=1` (output only, open decision B6). |

## 4. DMA pool, output buffers, and lifetime (proposal)

- Kernel-owned, permanently pinned and aligned RAM pages
  (`Owner::Kernel`), never user frames and never released before the VM
  stops, including after failed initialisation: one ring page per queue (4),
  one control/event data page, and two TX bounce pages.
- Period: 10 ms at 48 kHz/stereo/S16 = 1920 bytes. Four TX slots
  (2 × 2048 bytes per bounce page); a 4-byte header and 8-byte status per slot
  in the control/event data page. At most 40 ms is buffered; this is not a
  latency claim (the budget is H03 work on physical hardware).
- PCM arrives from the broker through a complete, one-time kernel copy into a
  free slot. No Rust reference to DMA is shared, and no application or broker
  pointer enters a descriptor. An invalid or oversized broker buffer is
  rejected before copying and publishes nothing.
- Slot lifetime: `Free → Filled → Published → Completed → Free`. A slot is
  reused only after a validated completion (known published descriptor ID,
  used length exactly 8, status `OK` or `IO_ERR`;
  `BAD_MSG` or unknown ⇒ fence) **or** after a confirmed fence (§5).
  `Completed` means “the device consumed the buffer” (§5.14.6.8.1.1), not
  “audible playback completed”.

## 5. Lease, epoch, revocation, reset, and quiescence

- **Lease** = `(device epoch, broker PID, capability object key)`. Every
  syscall and every completion delivery reauthorises all three.
- **Epoch** is bookkeeping, not a fence. The used ring has no epoch (§2.7.8);
  therefore only the fence can exclude a late completion.
- **Fence (quiescence)** = write status 0, read status 0 within a monotonic
  deadline (§2.4.2), **and** disable PCI bus mastering and read it back.
  Only then may slots be zeroed, ring indices restored, and the epoch advanced
  with `checked_add(1)`. Disabling bus mastering alone is not a fence;
  status 0 without BME disabled is not a fence either.
- **Revocation** (broker exit, capability closed, management action): reject new
  submissions → fence → outstanding requests receive `Revoked` with
  unknown playback status; no replay. RELEASE is optional before the fence and
  is never required for safety.
- **Reset after an error:** the same fence. A fence timeout or epoch overflow
  yields `Quarantined` until reboot; the pool remains pinned and the
  device remains offline.
- **Reinitialisation** after a fence repeats the complete §5.14.5 sequence,
  including feature and configuration checks; prior stream state is not reused.

### States

`Absent`, `Offline`, `Init`, `Ready` (no stream),
`Configured` (SET_PARAMS OK), `Prepared`, `Running`,
`Stopped`, `Fencing`, and `Quarantined`.
Transitions: `Offline→Init→Ready` (§5.14.5),
`Ready→Configured→Prepared→Running⇄Stopped`,
`Stopped→Ready` through one RELEASE (stream marked as released, never
twice), every state `→Fencing` on error/revocation, and
`Fencing→Offline` (acknowledged) or `→Quarantined` (timeout).

## 6. Error table

| # | Trigger | Detection | Response | Negative test (runtime) |
| --- | --- | --- | --- | --- |
| F1 | No device / different ID / legacy | PCI enumeration | `Absent`; broker receives `NoDevice`; no DMA pages exposed | QEMU without `virtio-sound-pci` |
| F2 | VERSION_1 missing, ACCESS_PLATFORM offered | `negotiate_features` | `Offline`, no DRIVER_OK | host codec (existing) |
| F3 | Invalid config, streams 0 or > 32 | `Config::parse` | `Offline` | `streams=0` profile |
| F4 | No output stream with S16/48 kHz/stereo | codec | `Ready` without stream; `Unsupported` | host codec; `streams=0` |
| F5 | Control status ≠ OK, used length ≠ expected | codec + transport | stream not started; BAD_MSG/IO_ERR ⇒ fence | — |
| F6 | Control timeout | monotonic deadline | fence | blocked backend (open) |
| F7 | TX used ID unknown/not published, used length ≠ 8 | transport | fence; no slot released | model injection |
| F8 | Used index changes while nothing is outstanding (stale) | idle check before publication | fence | model injection |
| F9 | TX status `IO_ERR`, `BAD_MSG`, or unknown | status | `IO_ERR`: release slot, return `IoError` to broker, stop stream. `BAD_MSG`/unknown: fence | model injection (the driver never submits an invalid stream ID) |
| F10 | `DEVICE_NEEDS_RESET` / configuration change | status read | fence | duplicate RELEASE (QEMU `virtio_error`) |
| F11 | Completion after status-0 acknowledgement | ring snapshot after fence | §2.4.1 violation ⇒ `Quarantined` | model injection |
| F12 | Fence timeout | deadline | `Quarantined`, pool pinned | model injection |
| F13 | Broker exit / capability closed during `Running` | kernel cleanup | revocation fence; no replay | terminate broker during playback |
| F14 | Invalid/oversized broker buffer | usercopy validation | `Invalid`, nothing published | syscall with invalid pointer/length |
| F15 | All slots occupied | slot count | `Again`, do not block | rapid writes |
| F16 | Underrun (no data) | broker | `Stopped`, no error | pause |
| F17 | Invalid event (length ≠ 8, stream ≥ streams, unknown type) | event parser | ignore + counter; repeated ⇒ fence | model injection |
| F18 | Epoch overflow | `checked_add` | `Quarantined` | host test |
| F19 | Second RELEASE | stream flag | reject before submission | host test |

## 7. Next minimum runtime gate (VM-SND-1, not executed)

Precondition: B1–B4 decided and root integration performed by release.

```text
qemu-system-x86_64 -machine pc-q35-8.2 -cpu Nehalem-v1 -accel tcg -smp 1 -m 8G \
  <dedicated OVMF CODE/VARS copies, project loader/kernel images> \
  -audiodev wav,id=snd0,path=<disposable-directory>/out.wav,out.frequency=48000,out.channels=2,out.format=s16 \
  -device virtio-sound-pci,addr=08.0,audiodev=snd0,streams=1
```

Normal case: the broker plays a fixed pattern generated in the guest. The guest
log reports feature bits, configuration, selected stream, completion count, and
fence acknowledgement.

**Oracle (release B8, fixed before the gate and not broadened afterwards):**

- Pattern: exactly 48,000 frames (1 s), 2 channels, S16 little-endian,
  interleaved L,R. Frame `n` (0 ≤ n < 48,000): `L = n as u16`,
  `R = (n as u16) ^ 0x8000`, both stored as i16. Frame 0 is therefore
  not silent (`R = 0x8000`).
- The WAV is parsed as a container (RIFF/WAVE; `fmt␠`, where `␠` denotes the
  trailing ASCII space: PCM, 2 channels,
  48,000 Hz, 16 bits; `data` chunk). Header bytes are not compared in
  their entirety; a different `fmt␠` is a failure.
- The `data` payload must consist of 0 through 4,800 completely silent
  leading frames (≤ 100 ms), followed by all 48,000 pattern frames
  **contiguously and byte-identically**, followed by 0 through 4,800 silent
  frames. Any other byte, gap, repetition, missing frame, or exceeded window
  is a failure.
- Assumption tested by the experiment itself: QEMU 10.2.1 passes
  S16/48 kHz/stereo without resampling or mixer modification. A loss of exact
  identity blocks the pass; the windows or tolerance are not broadened
  retrospectively.
- The result counts only as N/emulator-output partial evidence, not audible or
  physical H03/FR-H02/A06 acceptance.

Negative cases: F1 (no device), F13 (broker terminated during playback →
fence acknowledgement; the WAV subsequently grows only with silent frames
inside the final window), F14 (invalid buffer), and F10 (duplicate RELEASE
rejected before submission). Record the source commit,
`tools/common.py` fingerprint, kernel/loader/image hashes, QEMU/OVMF
hashes, exact command, wall-clock duration, and completion count. H03 and
FR-H02/A06 remain open.

## 8. Open decisions (grouped for 14/2/4)

Release response (2ab17599), 30 October 2026, read once at `5223214`;
**no approval for root integration.** Security response (081d6f31),
30 October 2026, read once at `5223214`: contract consistency only;
**no** physical fence evidence, vulnerability audit, or implementation
approval. Response from 14 remains open.

| # | Decision | Owner | Status |
| --- | --- | --- | --- |
| B1 | New manifest-only, non-delegable output right in capability-core (value/name) | release/kernel + security | Agreed in principle as a design proposal (release). Name/bit, issuer, check, revocation, and cleanup still require kernel + security; **no capability has been allocated**. Security: acceptable as a design, without IPC, microphone, or management implications; the actual capability/API change remains a separate REL/kernel claim. |
| B2 | Syscall numbers and ABI minor (0x73 is occupied by native-clock) | release | **Not approved.** 0x73 is occupied; allocate the ABI only on a confirmed integration source. |
| B3 | Confirm kernel transport (as for M5/network); no user DMA | kernel + security | Security: acceptable only as a labelled **trusted** QEMU kernel transport with a kernel copy and pinned RAM; **not isolation**. Kernel: open. |
| B4 | Record feature profile and BDF `00:08.0` in the VM-FR manifest | release | `00:08.0` is provisional only for VM-SND-1; it is not reserved in the VM-FR device inventory. |
| B5 | Configure rxq without buffers or disable it; determine QEMU behaviour in a runtime experiment | TL-HW | Open (TL-HW, runtime experiment). |
| B6 | VM-FR `streams=1` (no input stream) instead of the QEMU default of 2 | release + security | Agreed for bounded output-only VM-SND-1 (release): no RX buffers and no microphone right. Final VM-FR remains subject to integration. Security: acceptable for this fixed QEMU profile, provided native configuration/direction checks reject any profile that is not output-only; no general claim that stream 1 is always output. |
| B7 | Period/slot size and latency budget (measurable only on hardware) | TL-HW + Materiaal/applications | Open. |
| B8 | Exact WAV oracle or tolerance, and whether it counts as VM evidence | release/verifier | Decided (release): exact PCM, no tolerance; windows are defined in §7. |
| B9 | IOMMU remains absent: document that no DMA containment is claimed | security | Security: agreed; no IOMMU or DMA containment is claimed. |

### Security correction to §5 (binding before transport implementation)

- For a **conforming** device, the fence is a conjunction: status 0 read back
  **and** PCI BME disabled and read back, on the same complete device function,
  with a deadline and ordering. The epoch itself is not a fence.
- In `Fencing`, all queue publication, control, timer, and IRQ work is
  frozen **before** reset and cleanup. Slot ownership remains uncertain until
  both acknowledgements; slots are not released selectively. Overflow or an
  error remains quarantined.
- After both acknowledgements, the used ring is snapshotted through an
  explicitly ordered DMA read. Any completion difference after acknowledgement
  causes quarantine; there is no selective slot release and no masking through
  an epoch update.
- Initialisation and reinitialisation inherit no old completion or lease, even
  when a PID is reused. Capability-object generation and device epoch remain
  checked.
- For VM-SND-1, the oracle “no further WAV growth” is not equivalent to a
  cancelled host audio queue, because that queue may already have consumed the
  data. A completion does not mean “audible” (§4).

Host model for these rules (model evidence only):
`src/transport_model.rs`, PR #91.

## 9. Evidence boundary

This document contains design work only, based on primary sources and existing
project contracts. Nothing was executed in a guest. It makes no claim of audio
output, latency, DMA isolation, physical-hardware operation, or human audit.
Rollback consists of removing this file; nothing depends on it.
