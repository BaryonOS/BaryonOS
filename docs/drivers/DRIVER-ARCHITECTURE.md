# Driver Core and selection

The hardware goal and the choice of drivers for each device family are set out in
[HARDWARE-TARGETS](HARDWARE-TARGETS.md).

Status: generic segment-0 PCI discovery, registration and factory-linked
initialisation are implemented for VirtIO block, network and RNG. All three
select a function through `pci-core` and their own registry, with configuration
space bound to the match. Block, network and RNG receive kernel-owned BAR grants
before mapping; the existing transports retain their own I/O and DMA logic.
Separate, unintegrated NVMe and xHCI probe crates also exist. They match PCI
functions and inspect controller registers read-only. A shared heterogeneous
registry, interrupt broker and generic DMA/IOMMU resource broker are still
absent.

## Current implementation

`crates/driver-core` is a dependency-free `no_std` crate without `unsafe`.
It defines:

- common `DeviceId`, `DeviceClass`, `DriverId`, `DriverVersion`, `DriverState`
  and `DeviceMetadata` types;
- category identifiers and subsystem traits for block, network, display and
  input devices, plus identifiers for USB host controllers, audio, storage and
  serial devices;
- PCI identity, including BDF, vendor/device, class/subclass/programming
  interface/revision and optional subsystem IDs;
- a bounds-safe decoder for fixed 256-byte type-0 PCI configuration snapshots;
- a fixed-capacity `DriverRegistry` that selects a unique most-specific
  registration result and rejects equally ranked top matches.

`crates/pci-core` enumerates conventional PCI buses on segment 0 using
configuration mechanism 1. It visits functions 1–7 only when function 0
indicates the multifunction bit, ignores absent functions and reports only
type-0 endpoints to the caller. The scanner reads configuration space before
offering the descriptor to the registry. The current scan does not support
ECAM, segments other than 0, bridge topology or hotplug.

M5 kernel profiles perform an additional read-only scan before device
initialisation. Functions without a match among the drivers registered for that
profile are reported on COM1 as `UNSUPPORTED:PCI`, with segment, bus/BDF,
vendor/device, class/subclass/programming interface/revision and subsystem IDs
where available. Non-type-0 functions are reported only as a count; an
incomplete configuration scan is explicitly reported and produces no
unsupported records. This is a diagnostic inventory of matches, not evidence
that a matched driver has initialised successfully or that every profile
registry includes all PCI driver families. The RNG driver is now also started
through segment-0 enumeration and a registered factory; the current QEMU fixture
uses BDF `00:06.0`, but the driver does not require that location.

The VirtIO block, network and RNG drivers register their PCI IDs and select
their device using `DriverRegistry::best_match_with_factory`, rather than a
predefined BDF. BAR probing and subsequent access remain restricted to the
selected function. If more than one matching device is present while the kernel
path still supports only one instance, this is rejected as an error.
The activation paths also register a static initialiser function for each
match. `best_match_with_factory` returns metadata and that initialiser; the
value created by discovery stores it privately. After the kernel assigns the
existing BAR mappings and DMA pages, `Device::initialize` calls the selected
factory. Family-specific options and the existing reset/quarantine logic remain
with the concrete initialiser. No driver code is dynamically loaded, no heap
object is created and no additional resource authority is created; resources
remain kernel allocations.

`drivers/nvme` and `drivers/xhci` each use a local single-entry PCI registry and
a device-bound BAR grant for a limited probe with read-only controller MMIO.
PCI BAR sizing temporarily writes the configuration BARs and attempts to restore
their original values. A failed restoration write causes PCI-core to fail with
`PciBarError::Restore` (the xHCI adapter returns `Error::InvalidBar` for this).
PCI decoding and bus mastering remain disabled after a successful disable
write; if an earlier command read/write fails, the original command state may
still apply.
The NVMe crate reads `CAP`/`VS` according to NVM Express Base Specification 2.4,
§§3.1.1–3.1.2. The xHCI crate matches the USB host-controller class according to
PCI-SIG PCI Code and ID Assignment Specification 1.10, §1.2, and checks the
revision and register offsets according to the Intel xHCI Requirements
Specification 1.2c, §§5.2–5.3. Neither is part of the kernel activation chain:
they do not enable bus mastering or allocate DMA. Only the USB descriptor core
matches interface class/subclass/protocol according to USB 2.0 Specification
§9.6.5; this selection does not yet start a class driver either.

Primary sources: NVM Express, *NVM Express Base Specification*, revision 2.4,
31 July 2026, <https://nvmexpress.org/specification/nvm-express-base-specification/>;
PCI-SIG, *PCI Code and ID Assignment Specification*, revision 1.10,
28 November 2017, <https://pcisig.com/PCIExpress/Specs/Base/CodeandIDAssignment_1.10>;
Intel, *eXtensible Host Controller Interface for Universal Serial Bus (xHCI)
Requirements Specification*, revision 1.2c, October 2025,
<https://cdrdv2-public.intel.com/868295/xHCI__Rev1.2c.pdf>; USB Implementers
Forum, *USB 2.0 Specification*, descriptor description §9.6.5,
<https://www.usb.org/document-library/usb-20-specification>.

`MatchedPciFunction` binds configuration-space access to a revalidated descriptor
and `DriverMatch`. Reads and writes to another BDF are rejected; stale
descriptors and registrations that do not match the device are not bound. This
is a concrete device boundary for PCI configuration.
`MatchedPciFunction` can probe memory BARs after matching. I/O decoding, memory
decoding and bus mastering remain disabled after the successful disable write;
the probe attempts to restore BAR registers even on errors and reports a failed
restoration write. A `PciBarGrant` binds the selected BDF, driver ID, BAR index
and validated base/length. Block, network and RNG receive such a grant before
mapping. `Paging::map_pci_bar` rejects a different device, driver or BAR index
and permits only complete 4-KiB pages within that BAR to be mapped. Existing
PAT-UC, CPU-MAXPHYADDR and supervisor-NX checks remain active. This is a bounded
internal kernel grant, not a userspace capability or driver isolation. IRQ
allocation, bus-master permission as a separate grant and DMA buffer management
remain outside this broker. Block and RNG kernel profiles allocate their DMA
pool only after successful discovery and a registry match; the network path
already followed this order.

```text
PCI segment-0 scan
    -> type-0 DeviceDescriptor
    -> DriverRegistry::best_match_with_factory
    -> device-bound configuration adapter and BAR grants
    -> kernel validates grant and creates restricted UC/NX mapping
    -> selected VirtIO factory
    -> kernel validates device-bound BAR grants and DMA ownership
    -> existing block, network or RNG I/O
```

ArceOS's category-oriented traits and `DriverProbe`/`init_drivers` structure
served as architectural references; no ArceOS code was incorporated. The Baryon
OS kernel separates matching, resource authorisation and activation: a PCI
match alone grants no access to BARs, interrupts or DMA.

## Architecture still required

1. Extend the kernel-owned resource boundary to explicit BAR subregions for
   the remaining PCI drivers. The block/network paths now have a matched BAR
   grant, but the mapping code and device initialisation do not yet form one
   general resource broker. Integrate interrupt lifecycle and DMA with the
   maturity-branch semantics; do not create competing capability or IOMMU
   models.
2. Consolidate the per-family factories into a shared registry with a typed
   resource context, and connect activation state/cleanup to the lifecycle.
   The current static factory functions activate only the existing block,
   network and RNG initialisers; they do not yet form a general heterogeneous
   plug-and-play registry.
3. Preserve the existing VirtIO policy, DMA, timeout, reset and quarantine
   logic when moving to that resource boundary.
4. Add only the subsystem interfaces required by concrete drivers. USB requires
   a separate enumeration, class-matching and resource boundary; the current
   PCI registry does not claim USB support.
5. Extend discovery in a controlled manner to bridge topology, ACPI/ECAM,
   additional segments and subsequently hotplug, with tests for every new
   access path.

PCI BAR grants and statically linked initialisers for block, network and RNG
paths are integrated; no general BAR/MMIO, interrupt or DMA resource broker yet
exists. The current PCI code is not yet a complete plug-and-play chain. This
status makes no claim of physical hardware support, DMA isolation, hotplug,
USB, GPU acceleration or recovery from a crashed driver.
