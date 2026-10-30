# virtio-snd control codec (TASK-H03-VIRTIO-SND-CODEC)

Status: **hosttests + no_std crosscompile only**. No audio is played, no
device is driven, and nothing is integrated in kernel, loader, profile or
image. Rubric H03/A04; FR-H02/A06/A07 stay open.

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

## Not provided (separate contract and review with security/release)

Transport and queues, buffer posting and used-ring validation, DMA/MMIO,
reset/ACK/quarantine/epoch handling, broker or capability, app grant,
microphone/capture rights, PCM set/prepare/start, and the kernel/root wiring.
An advertised PCM capability grants no app permission; output is not a mic
right. No DMA- or IOMMU-isolation claim follows from this bounded codec.

## Gates

See `verification/README.md`.
