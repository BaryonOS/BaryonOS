# Hardware target: from QEMU through VMware to a physical PC

Status as of 5 November 2026; physical protocols, signed evidence and the
inventory correction updated 19 November 2026 (H06-T4). This document defines
the **end goal** towards which all virtual driver work is directed: Baryon OS
runs locally on a conventional x86-64 desktop or laptop with UEFI. Virtual
testing is the route to that goal, not the endpoint. This document does not
itself award roadmap points.

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
| Input/USB | xHCI + USB-HID boot protocol | `qemu-xhci` | yes | yes | native QEMU fixture only (`qemu-xhci` `1b36:000d`); physical controllers rejected (H02) |
| Storage | NVMe | `nvme` | yes | yes | absent (no driver or probe crate); next step |
| Storage | AHCI/SATA | `ich9-ahci` | yes | older PCs | later |
| Networking | Intel e1000e (82574/I219 family) | `e1000e` | `e1000e` | Intel Ethernet | next step (H04) |
| Audio | Intel HD Audio | `intel-hda` | `hdaudio` | yes | next step (H03) |
| Power | ACPI (FADT/DSDT, S5, later S3, `_BST`) | yes | yes | yes | S5 native on QEMU only; S3 host parsing only (H05) |
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

Physical operations are not authorised (see
[FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md)). Under its rule "if
authorization is missing, prepare the concrete package", the physical test
packages are prepared in [physical-protocols.json](physical-protocols.json)
and wait for the following open project-owner decisions. They are also
recorded, machine-checked, as `owner_decisions` in the
[hardware matrix](hardware-matrix.json):

| Decision | Question | Blocks |
|---|---|---|
| `OD-HW-TARGETS` | Number, make and model of the reference PC(s); sets the HW-FR target identity (HW-AI remains separate) | all protocols |
| `OD-HW-BOOT` | Authorization to boot a removable medium through the one-time boot menu, and the Secure Boot policy for the unsigned test image | all protocols |
| `OD-HW-DISKS` | Internal disks physically disconnected, or left untouched in place | all protocols |
| `OD-HW-CAPTURE` | Output capture route: COM1 null-modem cable to a second computer, or an on-screen route that is not yet demonstrated | all protocols |
| `OD-HW-SIGNERS` | Owner signing key(s) in [hw-allowed-signers](hw-allowed-signers) and the operators allowed to attest evidence | all protocols |
| `OD-HW-FAMILIES` | Required device families, including Wi-Fi, Bluetooth and GPU acceleration | H02-USB, H03-AV, H04-NET, R03-INSTALL |
| `OD-HW-PUBLISH` | Whether inventories and serial logs (PCI IDs, MAC addresses, firmware strings) may be published or must be redacted | all protocols |
| `OD-HW-INV-ROUTE` | Whether the native `HW-INV` inventory replaces the Linux live-USB `lspci -nn` step requested in earlier revisions of this document | HW-INV |
| `OD-HW-POWER` | Physical power profile: S3 wake source (power button, RTC alarm or keyboard/USB), battery and thermal scope | H05-S3 |

Output capture: markers are written to COM1 only. A physical COM1 port must be
connected to a second computer with a null-modem cable or a USB-to-serial
adapter attached to that second computer. A USB-to-serial adapter on the target
machine itself does not receive COM1 output, and no USB serial driver exists.
**On-screen route: unknown/not yet demonstrated.**

The COM1 `UNSUPPORTED:PCI` inventory described in earlier revisions of this
document and of [DRIVER-ARCHITECTURE](DRIVER-ARCHITECTURE.md) was never
implemented; no code emits that marker. Its replacement is the planned
read-only `native-hw-inventory` profile (slice H06-T3), specified as protocol
`HW-INV` with `HWINV:` markers.

## Physical test protocols

[physical-protocols.json](physical-protocols.json) defines one versioned
protocol per physical test. Each lists its criterion and FULL-TEST-RELEASE rows,
preconditions, exact operator, serial and host steps with time limits, required
and failure serial markers, forbidden actions, a recovery route, the evidence
fields and a status. No protocol has been executed; all seven are **blocked**:

| Protocol | Criterion / rows | Concrete blocking reasons |
|---|---|---|
| `HW-INV` v1 | H06 / FR-H03, FR-R03 | `native-hw-inventory` profile absent (H06-T3); no physical image; `0xF4` exit path; owner decisions |
| `H05-S5` v1 | H05 / FR-S04 | no physical image; rejection paths exit through `0xF4`; no `SMI_CMD`/`ACPI_ENABLE`, `_PTS`/`_TTS`; owner decisions |
| `H05-S3` v1 | H05 / FR-S05 | no native S3 entry/resume path (host parsing only); no wake source; must not rely on `0xF4`; owner decisions |
| `H02-USB` v1 | H02 / FR-H01 | xHCI accepts only QEMU `1b36:000d` at a fixed MMIO address; VT-d availability unknown; no IRQ/hub/hotplug; `0xF4` exit |
| `H03-AV` v1 | H03 / FR-H02 | no HD Audio driver; GOP evidence is QEMU-only; GPU scope undefined; `0xF4` exit |
| `H04-NET` v1 | H04 / FR-H03 | no e1000e driver; Wi-Fi/Bluetooth excluded pending `OD-HW-FAMILIES`; `0xF4` exit |
| `R03-INSTALL` v1 | R01 / FR-R03 | no installer image; no NVMe, AHCI or USB mass-storage driver; physical disk writes not authorised |

Common to every protocol:

- The image is a **removable-media** image whose SHA-256 is recorded and
  re-read from the medium. No such image can be built yet: `tools/build.py`
  exports an ESP directory for QEMU's virtual FAT backend, and
  `tools/repro_check.py` compares only that ESP payload manifest.
- Every milestone-m2 terminal path calls `exit()` in `kernel/src/serial.rs`,
  which writes QEMU's `isa-debug-exit` port `0xF4`. That port is undefined on
  physical PCs; a physical image first needs a halt-based terminal path.
- Forbidden: firmware flashing, saving firmware settings, `BootOrder` or other
  persistent UEFI variable changes, Secure Boot key or state changes, internal
  disk access, and personal data. Boot only through the firmware one-time boot
  menu from the removable medium; internal disks are disconnected or untouched.
- Recovery: hold the power button for at least 10 seconds, remove the medium,
  power on normally; the previous system starts unchanged because nothing
  persistent was modified. If it does not, stop and escalate; no firmware
  repair is attempted.

## Signed physical evidence

A physical run is recorded as
`docs/verification/<new-directory>/hw-evidence*.json`, with the inventory and
serial log in the same directory and a detached OpenSSH signature
`<record>.sig`:

```bash
ssh-keygen -Y sign -f <owner-private-key> -n baryonos-hw-evidence <record>
python3 -B tools/hw_evidence.py --check
```

`tools/hw_evidence.py` verifies the signature only through the standard
`ssh-keygen -Y verify -f docs/drivers/hw-allowed-signers -n baryonos-hw-evidence`
tool; it implements no cryptography. It rejects a record that is unsigned, is
signed by an unknown signer, with another key or in another namespace, or was
modified after signing; that binds another protocol id or version, a blocked
protocol, another matrix target or devices outside the protocol; whose
inventory or serial-log SHA-256 does not match; whose image, source or commit
identity is malformed; whose Secure Boot state contradicts the inventory; that
lacks a confirmation for any forbidden action; that is a virtual capture (CPUID
hypervisor bit, or PCI vendor `1af4`, `1b36`, `1234`, `15ad` or `80ee`, in the
inventory or `HWINV:` lines); or that claims `passed` while a step failed, a
required marker is missing or out of order, or a failure marker appears.

[hw-allowed-signers](hw-allowed-signers) deliberately lists **no key**
(`OD-HW-SIGNERS`). Until the project owner adds one, every record is rejected
(fail closed) and the check output says so. A signature binds the record to a
key holder; it does not prove that the described physical run took place.

## Machine-checked inventory

The [hardware matrix](hardware-matrix.json) (schema 2) records the three
profiles without selecting a physical machine; the HW-FR target stays `null`
(`OD-HW-TARGETS`). Run `python3 -B tools/hw_matrix.py --check`.

Each device row carries an explicit `evidence_level`: `none`, `host`,
`native-qemu`, `vmware` or `physical`. A QEMU row may use only `none`, `host` or
`native-qemu`; a VMware row only `none`, `host` or `vmware`; an HW-FR row only
`none`, `host` or `physical`. Any level other than `none` needs references, and
`none` needs none.

- `implemented` means source is present for a virtual-first transport, not that
  every device/profile combination has passed. `pending` and `unsupported`
  retain missing acceptance or absent drivers; a null driver path explicitly
  means absent.
- `native-verified` applies only to QEMU and VMware rows. It requires
  `native-qemu` (or `vmware`) evidence under `docs/verification/` that names the
  profile's machine, for example `pc-q35-8.2`. It is bound to the source of that
  evidence; it is neither criterion acceptance nor physical support, and a
  regression reopens it. Current rows: VM-FR `acpi`
  ([h05-acpi-s5](../verification/h05-acpi-s5/README.md)) and VM-FR `xhci`
  ([issue42-native-input-20261114](../verification/issue42-native-input-20261114/README.md)).
- QEMU and VMware rows can never be `supported`. An HW-FR row can be
  `supported` only with a selected target, level `physical` and a passing
  record that `tools/hw_evidence.py` validates for that target and device.
  Every referenced `hw-evidence` record must validate, otherwise the matrix
  check fails. Unsigned legacy JSON records no longer qualify, and a schema-1
  matrix cannot express `supported`.
- xHCI stays `pending` outside the QEMU profile because
  `kernel/src/m2/xhci.rs` accepts only QEMU's `qemu-xhci` (`1b36:000d`) at a
  fixed fixture MMIO address.

`trusted kernel transport` does not establish isolated userspace DMA.
`unsupported` makes no DMA-containment claim. `VT-d contained` requires an
archived passing target/device-bound record with that explicit posture: a
validated signed physical record for HW-FR, or the structural legacy native
record for a virtual profile. No current row claims it.

These checks do not establish owner authorization or hardware acceptance.
Physical selection and execution remain with the project owner; H06 remains
open.
