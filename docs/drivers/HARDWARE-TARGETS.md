# Hardware target: from QEMU through VMware to a physical PC

Status as of 5 November 2026; physical protocols, signed evidence and the
inventory correction updated 19 November 2026 (H06-T4); validator hardening
(owner-decision gating, prerequisite records, native report binding) updated
20 November 2026. This document defines
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
| Power | ACPI (FADT/DSDT, S5, later S3, `_BST`) | yes | yes | yes | S5 and S3 native on QEMU only (S3 woken by QMP `system_wakeup` under the QEMU/OVMF extended-vector policy); no physical evidence (H05) |
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
| `OD-HW-FAMILIES` | Required device families, including Wi-Fi, Bluetooth and GPU acceleration, and the install-target transport `R03-INSTALL` must exercise (USB mass storage, removable SATA/eSATA for AHCI, an external NVMe enclosure, or none) | H02-USB, H03-AV, H04-NET, R03-INSTALL |
| `OD-HW-PUBLISH` | Whether inventories and serial logs (PCI IDs, MAC addresses, firmware strings) may be published or must be redacted | all protocols |
| `OD-HW-INV-ROUTE` | Whether the native `HW-INV` inventory replaces the Linux live-USB `lspci -nn` step requested in earlier revisions of this document | HW-INV |
| `OD-HW-POWER` | Physical power profile: S3 wake source (power button, RTC alarm or keyboard/USB), battery and thermal scope | H05-S3 |

The "Blocks" column is also the minimum that `tools/hw_evidence.py` enforces:
the matrix may list more protocols for a decision, never fewer. A protocol can
only be `ready` while every decision that blocks it is recorded as `decided`
with a reference; `tools/hw_evidence.py --check` and
`tools/hw_matrix.py --check` both reject a `ready` protocol with an open
blocking decision, even when no record references it.

Output capture: markers are written to COM1 only. A physical COM1 port must be
connected to a second computer with a null-modem cable or a USB-to-serial
adapter attached to that second computer. A USB-to-serial adapter on the target
machine itself does not receive COM1 output, and no USB serial driver exists.
**On-screen route: unknown/not yet demonstrated.**

The COM1 `UNSUPPORTED:PCI` inventory described in earlier revisions of this
document and of [DRIVER-ARCHITECTURE](DRIVER-ARCHITECTURE.md) was never
implemented; no code emits that marker. Its replacement is the read-only
`native-hw-inventory` profile (slice H06-T3), specified as protocol `HW-INV`
with `HWINV:` markers. The profile exists and is tested in QEMU only; see
[HW-INV measurement package](#hw-inv-measurement-package).

## HW-INV measurement package

`python3 -B tools/build.py --kernel-feature native-hw-inventory` builds the
paired kernel and the `hw-inventory` loader; one without the other is refused.
The loader records the UEFI `SecureBoot` variable in a 16-byte firmware-facts
extension at BootInfo offset 256 (`crates/boot-protocol/src/firmware.rs`);
`EFI_NOT_FOUND` is reported as `ABSENT`, never as disabled. The kernel runs the
normal IF0 M2 discovery, prints `M2:PASS`/`M1:PASS`, then the `HWINV:` block,
and stops in a `CLI`/`HLT` loop. PCI identity is read only from configuration
offsets `0x00`, `0x08` and `0x0C` of every present segment-0 function, through
legacy mechanism 1; no BAR, command register or bus master is touched. Since
H06-T5 every terminal path of the pair (panic, M1/M2 failure, M2 exception,
`HWINV:FAIL`, serial timeout, loader `M1:FAIL` and the loader handoff return)
halts without writing port `0xF4`; QEMU fixtures in
[h06-halt-terminals-20261124](../verification/h06-halt-terminals-20261124/README.md).
H06-T7 adds the serial-timeout fixture (a labelled injection makes the UART
report busy, so the bounded wait expires and the kernel halts silently; the
truncated block is rejected) and checks `HLT=1` for every terminal fixture:
[h06-serial-timeout-20261124](../verification/h06-serial-timeout-20261124/README.md).

`python3 -B tools/hw_inventory.py --log <serial> --out <inventory.json>` turns
one complete, passing `HWINV:` block into the schema-2 inventory JSON consumed
by `tools/hw_evidence.py`; it rejects failure markers, reordered or duplicated
lines, count mismatches and an absent DMAR table with units.

QEMU reference configurations (`python3 -B tools/run_hw_inventory.py --case all`,
`pc-q35-8.2`, TCG, one CPU, 256 MiB, OVMF without enrolled keys):

| Case | CPU | Devices | Expected |
|---|---|---|---|
| `plain` | `qemu64,vendor=AuthenticAMD,-hypervisor` | default q35 | `VENDOR=AuthenticAMD:HYPERVISOR=0`, `DMAR=ABSENT:DRHD=0`, kernel `ACPI:UNAVAILABLE:NO_DMAR` |
| `iommu` | `qemu64,vendor=GenuineIntel` | `intel-iommu`, `pcie-root-port` at `00:06.0`, `qemu-xhci` behind it | `VENDOR=GenuineIntel:HYPERVISOR=1`, `DMAR=PRESENT:DRHD=1`, kernel `ACPI:DISCOVERY:MCFG_DMAR` |
| `image` | as `plain` | as `plain`; boots the removable-media image as a raw virtio disk instead of the virtual FAT directory | as `plain`, and the image SHA-256 equals its record before and after the run |

Each case passes only when the `HWINV:` PCI set equals QMP `query-pci` (address,
vendor, device, class, subclass, including functions behind bridges), QEMU is
still running one second after `HWINV:PASS` (no exit-port write), no failure
marker appears, `SECUREBOOT` is not `1`, the inventory binds to its own log, and
`hw_evidence` rejects it as a virtual capture. Evidence limits: QEMU/TCG only;
no physical PC or signed record, and the removable-media image has booted only
in QEMU; `SECUREBOOT=0`/`1` is
host-tested only; QEMU's `query-pci` is an oracle for QEMU, not for firmware on
real hardware. A physical run still requires the owner decisions listed in
[hardware-matrix.json](hardware-matrix.json) and the protocol in
[physical-protocols.json](physical-protocols.json).

## Physical test protocols

[physical-protocols.json](physical-protocols.json) defines one versioned
protocol per physical test. Each lists its criterion and FULL-TEST-RELEASE rows,
preconditions, exact operator, serial and host steps with time limits, required
and failure serial markers, forbidden actions, a recovery route, the evidence
fields, the accepted records it requires (`prerequisites`), the transport
markers that identify each listed device (`device_markers`) and a status. No
protocol has been executed; all seven are **blocked**:

| Protocol | Criterion / rows | Concrete blocking reasons |
|---|---|---|
| `HW-INV` v3 | H06 / FR-H03, FR-R03 | `native-hw-inventory` profile present and QEMU-tested (H06-T3); every terminal path halts without `0xF4` (H06-T5, QEMU fixtures); removable-media image built and QEMU-booted (H06-T6) but never booted physically, and its GPT/FAT32 layout awaits `OD-HW-BOOT`; owner decisions |
| `H05-S5` v1 | H05 / FR-S04 | no physical image; rejection paths exit through `0xF4`; no `SMI_CMD`/`ACPI_ENABLE`, `_PTS`/`_TTS`; owner decisions |
| `H05-S3` v1 | H05 / FR-S05 | QEMU-native S3 entry/resume exists (`native-acpi-s3`, [h05-native-s3-20261119](../verification/h05-native-s3-20261119/README.md)) but is woken only by QMP `system_wakeup` under the QEMU/OVMF `Edk2Version0Extended` vector policy; physical firmware path (FACS version, real-mode or extended vector, `_PTS`/`_WAK`) unverified; no physical wake source (`OD-HW-POWER`); protocol markers not yet emitted in the specified form; must not rely on `0xF4`; owner decisions |
| `H02-USB` v2 | H02 / FR-H01 | xHCI accepts only QEMU `1b36:000d` and maps its BAR0 at a fixed kernel virtual address; VT-d availability unknown until `HW-INV` records the DMAR table; no IRQ/hub/hotplug; `0xF4` exit |
| `H03-AV` v2 | H03 / FR-H02 | no HD Audio driver; GOP evidence is QEMU-only; GPU scope undefined; `0xF4` exit |
| `H04-NET` v1 | H04 / FR-H03 | no e1000e driver; Wi-Fi/Bluetooth excluded pending `OD-HW-FAMILIES`; `0xF4` exit |
| `R03-INSTALL` v2 | R01 / FR-R03 | no installer image; no NVMe, AHCI or USB mass-storage driver; install-target transport undecided (`OD-HW-DISKS`, `OD-HW-FAMILIES`); physical disk writes not authorised |

Version 3 of `HW-INV` adds the `M2:FAIL*` failure marker (H06-T5: the M2 kernel
exception path of the profile). Version 2 of `HW-INV` adds the read-only `HWINV:ACPI:DMAR=<PRESENT|ABSENT>:DRHD=<n>`
observation and inventory schema 2 (`acpi.dmar`, `acpi.drhd_units`); version 2
of `H02-USB` requires that observation (DMAR present, at least one DRHD unit)
and an xHCI function (`0c/03/30`) in the bound inventory. A DMAR table is never
evidence of VT-d containment; only a run's own `*:BUS_MASTER:CONTAINED` marker
is. Version 2 of `H03-AV` starts listening before playback and requires an
`HDA:PLAYBACK:START:*` marker that reports the tone duration. Version 2 of
`R03-INSTALL` lists `nvme` or `ahci` only with that controller's own
`INSTALL:TARGET:NVME:*` or `INSTALL:TARGET:AHCI:*` marker; until the owner
decides the transport, an install to a USB or other removable target lists no
storage device. No marker pattern of one device may match a line that another
device's pattern of the same protocol also matches (exact or wildcard overlap).
A protocol with prerequisites is accepted only when an accepted, passing record
of each prerequisite exists with the same target, `source_sha256`,
`source_commit` and inventory SHA-256 as the dependent record.

Common to every protocol:

- The image is a **removable-media** image whose SHA-256 is recorded and
  re-read from the medium. `tools/hw_inventory_image.py` builds one only for
  `HW-INV`: a fixed 64 MiB raw GPT disk with one FAT32 EFI system partition
  (8.3 upper-case names, timestamps fixed at 1980-01-01, GUIDs and volume
  serial derived from the payload), so the same ESP payload yields the same
  bytes. It writes a regular file in the build directory and records
  `image_sha256` and `image_bytes`; on a larger medium step S1 compares the
  first `image_bytes` bytes. The GPT backup header then sits inside the medium
  rather than at its end. The layout is an assumption until `OD-HW-BOOT` is
  decided, and the loader is not yet bit-reproducible (B04), so a rebuild from
  a clean checkout can produce a different image hash. The other protocols'
  kernel features still have no image.
- Outside the `native-hw-inventory` profile, every milestone-m2 terminal path
  calls `exit()` in `kernel/src/serial.rs`, which writes QEMU's `isa-debug-exit`
  port `0xF4`. That port is undefined on physical PCs; each such profile first
  needs a halt-based terminal path, as `native-hw-inventory` has since H06-T5.
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
inventory or `HWINV:` lines); whose inventory is not schema 2 or lists a PCI
address twice; whose inventory PCI functions differ from the serial log's
`HWINV:PCI:` lines (each function exactly once, lowercase hex) or from its
single `HWINV:PCI:COUNT` (required for every `HW-INV` record and for any log
that names `HWINV:PCI`); whose `HWINV:ACPI:DMAR` line, or kernel ACPI marker
(`ACPI:UNAVAILABLE:NO_DMAR` needs no DMAR table, `ACPI:UNAVAILABLE:NO_DMAR_UNITS`
a DMAR table with no DRHD unit, `ACPI:DISCOVERY:MCFG_DMAR` at least one DRHD
unit), contradicts the inventory; that claims `VT-d contained` without a
protocol `*:BUS_MASTER:CONTAINED` marker, or runs such a protocol on an
inventory without a DMAR table and DRHD unit; that lists a device whose
transport markers are absent from the serial log; that claims `passed` while a
step failed, a required marker is missing or out of order, a failure marker
appears, or `BOOT:EBS_OK` occurs more or fewer times anywhere in the log,
including after firmware output on the same line, than the protocol requires
(an unexpected restart); that is reached through a symlink or another
aliased path; or for which no accepted, passing record of each prerequisite
protocol exists for the same target, source hash, source commit and inventory
digest (`precondition unmet`). Allowed-signers entries must carry a public-key
blob with exactly the layout of their key type.

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
- `native-verified` applies only to QEMU rows. It requires `native-qemu`
  evidence under `docs/verification/` that includes a JSON harness report with
  a passing run set: the report (or one entry of its `profiles` object) and
  every run and guest passed, no guest timed out, each raw QEMU exit equals the
  expected exit, every guest ran with `-machine <profile machine>` (for
  example `pc-q35-8.2`) and the report's lowercase source SHA-256, and the run
  set is the device family's positive case. Bindings exist only for `xhci`
  (the no-op command case: every guest requires and every run observed
  `XHCI:BUS_MASTER:CONTAINED`, `XHCI:NOOP_COMPLETION:PASS` and `XHCI:PASS`)
  and `acpi` (kernel feature `native-acpi-s5`, every run observed
  `POWER:S5:WRITE`); other families stay at most `implemented` or `pending`.
  Passing negative profiles, such as the xHCI DMA fault-injection and
  missing-DMAR runs that also end with `XHCI:PASS`, never bind. A related run
  set that failed rejects the row, and so does a positive xHCI run set that
  also carries an `XHCI:NO_DMAR:`, `XHCI:DMA_FAULT:` or `XHCI:FAIL` marker.
  Prose files (README, notes) are supporting context only and never bind.
  VMware rows cannot reach `native-verified` until a VMware report format is
  defined in code. It is neither criterion acceptance nor physical support, and
  a regression reopens it. Current rows: VM-FR `acpi`
  ([h05-acpi-s5](../verification/h05-acpi-s5/README.md); the QEMU S3 report
  [h05-native-s3-20261119](../verification/h05-native-s3-20261119/README.md) is
  cited as context) and VM-FR `xhci`
  ([issue42-native-input-20261114](../verification/issue42-native-input-20261114/README.md)).
- QEMU and VMware rows can never be `supported`. An HW-FR row can be
  `supported` only with a selected target, level `physical` and a passing
  record that `tools/hw_evidence.py` validates for that target and device.
  Every referenced `hw-evidence` record, on any row and at any level, must
  validate and bind that row's profile, target and device, otherwise the
  matrix check fails. References must name the file itself; a symlink or a
  path through a symlinked directory is rejected. Unsigned legacy JSON records
  no longer qualify, and a schema-1 matrix cannot express `supported`.
- xHCI stays `pending` outside the QEMU profile because
  `kernel/src/m2/xhci.rs` accepts only QEMU's `qemu-xhci` (`1b36:000d`). It
  maps the discovered BAR0 (at most 64 KiB, at or above `0x80000000`, not
  overlapping the register window of the DMAR unit that native DMA would use,
  when one qualifies) at the fixed kernel virtual address `0x3f210000`.

`trusted kernel transport` does not establish isolated userspace DMA.
`unsupported` makes no DMA-containment claim. `VT-d contained` requires
target/device-bound evidence for that explicit posture: on HW-FR a validated,
passing signed physical record with that posture; on a QEMU row `native-qemu`
evidence with a passing, machine- and device-bound native report (the same
binding as `native-verified`) for a family whose binding requires and observes
a `*:BUS_MASTER:CONTAINED` marker, which today is only `xhci`. Hand-written
JSON records no longer qualify, and VMware rows cannot claim it. No current row
claims it.

These checks do not establish owner authorization or hardware acceptance.
Physical selection and execution remain with the project owner; H06 remains
open.
