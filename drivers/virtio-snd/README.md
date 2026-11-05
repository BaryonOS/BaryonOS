# virtio-snd control codec (TASK-H03-VIRTIO-SND-CODEC)

Status: **hosttests + no_std crosscompile only**. No audio is played, no
device is driven, and nothing is integrated in kernel, loader, profile or
image. Rubric H03/A04; FR-H02/A06/A07 stay open.

`src/lib.rs` holds feature/config/PCM_INFO; `src/pcm.rs` adds the bounded
PCM lifecycle (SET_PARAMS/PREPARE/START/STOP/RELEASE), TX framing and event
codec with a per-stream state machine. Both are pure byte codecs.

Norm: OASIS virtio 1.3, section 5.14 (sound device). QEMU 10.2.1
`virtio-sound-pci` was only *detected* (PCI `1af4:1059`, streams 2, jacks 0,
chmaps 0); device-ID detection is not audio playback.

## Contract

| Item | Rule |
| --- | --- |
| Features | `negotiate_features` requires `VERSION_1`, refuses `ACCESS_PLATFORM` (no IOMMU/DMA claim, same as `virtio-core`) and never accepts `VIRTIO_SND_F_CTLS`. `Negotiated` is only constructible there. |
| Config | Exactly 12 bytes (`jacks`, `streams`, `chmaps`; no `controls`). Each field `<= MAX_ITEMS` (32). Caller does the config-generation-stable read. |
| Query | `PCM_INFO` (0x0100), `start_id`/`count`/`size=32`. `count` 1..=32, `start+count` checked and `<= streams`. |
| Response | Caller passes exactly the used-length bytes. Status must be `S_OK` (0x8000); every other value incl. `BAD_MSG`/`NOT_SUPP`/`IO_ERR`/unknown is `Status`. Success length must equal `4 + count*32` (checked). |
| Record | Direction 0/1 only; `1 <= channels_min <= channels_max`; padding zero; reserved feature/format/rate bits rejected; empty formats/rates unsupported. One bad record fails the whole response. |
| Selection | Output-only stereo S16/48 kHz. Capture (`INPUT`) is parsed for completeness but never selected. |
| Resources | No heap; fixed `[StreamInfo; 32]`; no `unsafe`; no panicking indexing on device data. |
| Lifecycle (`pcm::Stream`) | Only an output stream with stereo S16/48 kHz is driven. Conservative subset of the spec lifecycle (`TRANSPORT.md` §5): `Ready→Configured` (SET_PARAMS, also from `Configured`), `Configured→Prepared`, `Prepared`/`Stopped`→`Running`, `Running→Stopped`, `Stopped→Ready` (one RELEASE per configured lifetime, F19 refused before encoding). Every other request is `BadState`; one outstanding control request per stream (`Busy`). |
| SET_PARAMS (24 bytes) | `period_bytes` 4..=2048 in whole frames, 1..=4 periods, `buffer_bytes = periods × period_bytes` (≤ 8192; divisibility by construction). Features: only `EVT_XRUNS` and only when advertised; shared memory and polling are never selected. Channels 2, format S16 (5), rate 48 kHz (7), padding 0. |
| Control response | Exactly 4 bytes. `OK` applies the transition; `NOT_SUPP` leaves the state unchanged (`Unsupported`); `BAD_MSG`/`IO_ERR`/unknown and any other used length are `Status`/`Length` and require a fence (F5). |
| TX message | Header `stream_id` (4 bytes) only while `Prepared` or `Running` and not faulted; payload 1..=period_bytes in whole frames. Status exactly 8 bytes: `OK`/`IO_ERR` with `latency_bytes` passed through unvalidated; `IO_ERR` refuses further TX until a device-confirmed STOP (F9); any other status/length requires a fence; a completion after RELEASE is `LateCompletion` (fence). |
| Events | Exactly 8 bytes. Jack ids `< jacks`, PCM stream ids `< streams`, `CTL_NOTIFY` is `Unsupported` (CTLS never negotiated), unknown codes `InvalidEvent`. `EventBudget` ignores at most 8 invalid events, then `EventStorm` (fence, F17). PCM events count only for the own stream while `Running`; `EVT_XRUNS` only when SET_PARAMS selected it. After a device-confirmed STOP at most `periods + 1` own-stream PCM events are `Stale` (ignored, not counted in `EventBudget`, no fence); beyond that they are `InvalidEvent`. START resets that bound. |

## Not provided (separate contract and review with security/release)

Transport and queues, buffer posting and used-ring validation, DMA/MMIO,
reset/ACK/quarantine/epoch handling, slot ownership, broker or capability,
app grant, microphone/capture rights, control time-outs, and the kernel/root
wiring. `pcm::Error::requires_fence` only names the device violations the
transport must fence; the fence itself is the transport's. An advertised PCM
capability grants no app permission; output is not a mic right. No DMA- or
IOMMU-isolation claim follows from these bounded codecs.

## Gates

See `verification/README.md`.
