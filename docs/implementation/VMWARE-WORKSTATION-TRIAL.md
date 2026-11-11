# VMware Workstation Pro — concrete trial assignment

2 November 2026. The installation of VMware Workstation Pro on the relevant
Windows PC has been reported. The assignment concerns research into a VMware route
alongside the existing development route.
There is as yet **no VMware execution, ISO or working VMware desktop proven**.

The designated executing session is **Ontwerp veilige OS-desktop** ("Design secure OS desktop"),
`01a19c0e-6a17-70e4-8da0-263949812063`, host `durable`. The coordinating session
has read this session. Execution was paused after the request for
alignment. This assignment is **prepared and not yet sent**: in the
coordinating session only `read_thread` was available, without
`send_message_to_thread` or `handoff_thread`. Storing this assignment therefore
does not mean that the executing session has been started.

## Assignment for the executing session

1. Keep the existing WIP and the new session slice in draft PR129. Do not
   take over overlapping desktop, auth, transport or score files again.
   The tested session source is `703cc0705471062ffb1483a16f42a2e580a6fdf7`;
   later documentation does not change that source-bound evidence.
2. First check whether access to the Windows PC is actually available. A
   cloud working directory `/workspace` is not access to the installed Workstation.
   Measure locally the Workstation version, available `vmrun`, host virtualisation
   mode and backend used. Do not change Windows, Hyper-V, VBS, TPM
   or firmware security to achieve a speed claim.
3. Investigate guest compatibility before starting a VM. The current
   `native-session-route` requires virtio-blk `1af4:1042` at `00:05.0` and
   virtio-rng `1af4:1044` at `00:06.0`. Workstation documents IDE/SATA/SCSI/
   NVMe; those are no replacement for these drivers. Converting a disk to VMDK
   therefore does not make the native Materiaal desktop work.
4. First make a separate, bounded platform trial with the existing
   `graphics-g0` profile: own UEFI loader, own kernel, GOP and polled
   PS/2 input, without a guest data disk or key service. Reuse `build.py`,
   the build manifest, the source fingerprint and the checked ESP files.
5. Build a reproducible UEFI El Torito ISO with a FAT boot image that contains
   **both** `EFI/BOOT/BOOTX64.EFI` and `kernel.elf`. The loader opens the
   kernel from the same firmware volume. The current runners supply only
   an ESP directory via QEMU `fat:esp`; there is no generic ISO exporter yet.
   Choose and check a maintained host tool with provenance/version/licence.
   Do not format or mount a physical disk as guest storage.
6. Create a new disposable VM, with its own NVRAM and exclusively the created ISO:
   UEFI, one vCPU, initially the tested 256 MiB budget, COM1 to a local
   file; no network, guest data disk, shared folder, clipboard integration,
   USB or GPU passthrough and no 3D. Verify which configuration the actually
   installed Workstation version supports. Do not change any existing VM.
7. Check the Secure Boot state and stop with a concrete diagnosis if
   the development loader is refused; do not silently turn it off. The
   G0 loader requests exactly 800×600 RGB/BGR GOP. A missing mode is a
   compatibility error, not a reason to simulate success.
8. Keep the COM1 log, actual image and key response, VMX/ISO/source hashes,
   version/backend, time to `G0:READY`, screenshot and normal stop. First use
   Workstation **VM → Capture Screen**. `vmrun captureScreen` requires
   a valid guest login according to the examined manual; Nieuw OS has
   no VMware Tools/login adapter. Do not claim working automation.
9. Only after that compare the same boot/render workload between VMware and QEMU.
   Name the QEMU accelerator actually used: TCG and Windows/WHPX are
   different profiles. Workstation is not faster without measurement.

Acceptance of this first trial means only UEFI/GOP/PS2 on VMware.
The isolated session desktop then requires suitable storage/entropy adapters,
bounded device/DMA contracts and its own negative trials. Keep QEMU as the
existing regression and acceptance route until VMware has independent evidence.

## Execution environment and boundaries

The active host is Linux x86_64. VMware, `vmrun`, `xorriso`, `mformat`, `mcopy`
and `mkfs.fat` were not found here, not even after activating the
project environment. No execution access to the Windows PC has been demonstrated.
No Windows setting, real disk or existing VM has been changed.

Primary sources, accessed on 2 November 2026:

- Broadcom, [Using VMware Workstation Pro 17](https://techdocs2-prod.adobecqms.net/content/dam/broadcom/techdocs/us/en/pdf/vmware/desktop-hypervisors/workstation/vmware-workstation-pro-17-0.pdf):
  controllers, firmware, PS/2, Host VBS Mode and screenshot conditions
  (`captureScreen`, p.314). This document does not prove which version is installed on the relevant PC.
- Broadcom, [Workstation with Hyper-V/VBS and WHP](https://knowledge.broadcom.com/external/article?legacyId=76918):
  support depends on the host configuration; determine locally.
- UEFI Forum, [UEFI 2.11, Media Access](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html):
  FAT and El Torito boot.

Source code for the current device boundaries:
`drivers/virtio-blk/src/lib.rs::discover`,
`drivers/virtio-rng/src/lib.rs::discover`,
`kernel/src/graphics.rs` and `boot/loader/src/main.rs`.
