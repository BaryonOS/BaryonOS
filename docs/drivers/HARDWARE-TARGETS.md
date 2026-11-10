# Hardware target: from QEMU through VMware to a physical PC

Status as of 5 November 2026. This document defines the **end goal** towards which
all virtual driver work is directed: Baryon OS runs locally on a conventional
x86-64 desktop or laptop with UEFI. Virtual testing is the route to that goal,
not the endpoint. This document does not itself award roadmap points.

## Three profiles, in this order

| Profile | Description | Role | Operator |
|---|---|---|---|
| `VM-FR` | QEMU `pc-q35-8.2`, OVMF, see [FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md) | Automated regression testing in CI; negative tests and fault injection | CI and agents |
| `VM-VMW` | VMware Workstation Pro 17 on the project owner's Windows PC, UEFI, a disposable VM containing only the built ISO | A second, independent emulation of **real** device models (xHCI, NVMe, e1000e, HD Audio, SATA). Detects QEMU-specific behaviour | The project owner, with a script and expected COM1 output |
| `HW-FR` | One specifically identified physical desktop or laptop (not yet selected) | Acceptance on real hardware; IOMMU, power and battery | The project owner, with the same image and recovery route |

A criterion that requires "physical" hardware remains open until `HW-FR`
provides the evidence. `VM-VMW` does not replace this, but does demonstrate that
a driver does not depend solely on QEMU. The VMware startup trial is documented in
[VMWARE-WORKSTATION-TRIAL](../implementation/VMWARE-WORKSTATION-TRIAL.md).

## Fewest drivers, broadest coverage

For each device family, first select the standard interface present in **all three**
profiles. One driver then covers QEMU, VMware and most physical PCs.

| Family | Driver (standard) | VM-FR | VM-VMW | HW-FR | Status |
|---|---|---|---|---|---|
| Display | UEFI GOP framebuffer | yes | yes | yes | present (G0/U01) |
| Input | PS/2 (i8042) | yes | yes | often (laptop keyboard) | present, polled |
| Input/USB | xHCI + USB-HID boot protocol | `qemu-xhci` | yes | yes | in progress (H02) |
| Storage | NVMe | `nvme` | yes | yes | probe crate present; next step |
| Storage | AHCI/SATA | `ich9-ahci` | yes | older PCs | later |
| Networking | Intel e1000e (82574/I219 family) | `e1000e` | `e1000e` | Intel Ethernet | next step (H04) |
| Audio | Intel HD Audio | `intel-hda` | `hdaudio` | yes | next step (H03) |
| Power | ACPI (FADT/DSDT, S5, later S3, `_BST`) | yes | yes | yes | S5 in PR #213 (H05) |
| Entropy | RDRAND/RDSEED | yes | yes | yes | virtio-rng present |
| DMA boundary | Intel VT-d (DMAR) | QEMU `intel-iommu` optional; guest isolation not demonstrated | vIOMMU depends on the VM configuration; not demonstrated | only with suitable VT-d hardware | open (H06) |

VirtIO (blk/net/rng/snd) remains the fast CI route, but does not count as coverage
for real hardware.

Deliberately **excluded** from the first round: Wi-Fi and Bluetooth (require chip-specific
firmware blobs and a licensing decision), GPU acceleration (GOP is the baseline;
"required GPU functionality" in H03 still needs to be defined), Realtek
Ethernet, USB hubs and isochronous USB. These choices are for the project owner
to decide once `HW-FR` has been selected.

## How drivers can be added without kernel changes

The kernel will receive one generic driver interface bounded by capabilities
(BAR mapping, DMA pages, release on a crash; see branch `hw/driver-grant`).
Each driver above will then run as a userspace service. Without a configured
guest IOMMU and explicit DMA allocation, guest DMA isolation is not proven;
the emulator boundary does not automatically restrict DMA to a driver's buffers.
On `HW-FR`, this isolation requires suitable VT-d hardware and implementation (H06).

Reuse before writing new code: register/type crates from rust-osdev (`xhci`,
`pci_types`, `acpi`) after the review in [DEPENDENCIES](../DEPENDENCIES.md);
see the [driver research](../research/2026-11-03/os-repos-and-hardware-drivers.md).

## What the project owner must provide for HW-FR

1. The target machine's make/model and the output of a PCI listing (for example,
   from a Linux live USB: `lspci -nn`), so that each device ID in the table
   above is assigned a driver or an explicit "not supported" designation.
2. Permission to use a USB stick as the boot medium; no internal disk.
3. COM1-only markers, such as `UNSUPPORTED:PCI`, require a serial connection.
   Connect the target machine's serial port to a second computer using a
   suitable serial or USB-to-serial adapter connected to that second computer.
   A USB-to-serial adapter on the target machine itself does not automatically
   receive COM1 output; without a serial port, USB output requires a supported
   USB serial driver. **On-screen route: unknown/not yet demonstrated.** Whether
   the same diagnostic markers appear on screen must be tested separately
   before a physical trial without a serial connection can succeed.
