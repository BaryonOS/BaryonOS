#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Small, allocation-free device contracts and PCI driver matching.
//!
//! Matching is deliberately separate from resource assignment and driver
//! startup. The kernel does not yet expose bounded, capability-backed device
//! resource grants, so this crate cannot safely start arbitrary drivers.

/// Stable identifier assigned by the device-discovery owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DeviceId(u64);

impl DeviceId {
    /// Creates an identifier from a kernel-assigned value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the underlying identifier.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Stable identifier assigned to a driver implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DriverId(u64);

impl DriverId {
    /// Creates an identifier for a driver.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the underlying identifier.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Product version of a driver implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DriverVersion {
    /// Major compatibility version.
    pub major: u16,
    /// Minor feature version.
    pub minor: u16,
    /// Patch version.
    pub patch: u16,
}

impl DriverVersion {
    /// Creates a version tuple.
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

/// Logical device category exposed to an operating-system subsystem.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DeviceClass {
    /// Block-addressed storage.
    BlockDevice,
    /// Packet-oriented network interface.
    NetworkDevice,
    /// Pixel-output device.
    DisplayDevice,
    /// Keyboard, pointer, or other input source.
    InputDevice,
    /// USB host controller.
    UsbHostController,
    /// Audio playback or capture device.
    AudioDevice,
    /// Storage controller below a block-device interface.
    StorageController,
    /// Serial port.
    SerialDevice,
    /// Hardware entropy source.
    EntropyDevice,
}

/// Lifecycle state reported by the trusted device manager.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DriverState {
    /// Discovered but not matched to a driver.
    Discovered,
    /// A registered driver matched the device.
    Matched,
    /// Initialization has begun.
    Starting,
    /// The device is available to its owning subsystem.
    Ready,
    /// Initialization or operation failed.
    Failed,
    /// No supported driver matched the device.
    Unsupported,
    /// Resources remain withheld pending a verified reset or shutdown.
    Quarantined,
}

/// Common identity and lifecycle metadata for an initialized device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceMetadata {
    /// Kernel-assigned device identity.
    pub id: DeviceId,
    /// Logical subsystem category.
    pub class: DeviceClass,
    /// Driver selected for this device.
    pub driver_id: DriverId,
    /// Version of the selected driver.
    pub driver_version: DriverVersion,
    /// Current trusted lifecycle state.
    pub state: DriverState,
}

/// PCI location of a function.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PciAddress {
    /// PCI segment number.
    pub segment: u16,
    /// Bus number.
    pub bus: u8,
    /// Device number.
    pub device: u8,
    /// Function number.
    pub function: u8,
}

/// Identity fields needed for driver selection.
///
/// Subsystem IDs are absent when the PCI header does not define them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciDeviceDescriptor {
    /// PCI location.
    pub address: PciAddress,
    /// PCI vendor identifier.
    pub vendor_id: u16,
    /// PCI device identifier.
    pub device_id: u16,
    /// PCI base class.
    pub class: u8,
    /// PCI subclass.
    pub subclass: u8,
    /// PCI programming interface.
    pub programming_interface: u8,
    /// PCI revision.
    pub revision: u8,
    /// Subsystem vendor, when present.
    pub subsystem_vendor_id: Option<u16>,
    /// Subsystem device, when present.
    pub subsystem_device_id: Option<u16>,
}

/// Errors while decoding a conventional PCI endpoint header.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PciHeaderError {
    /// The PCI device or function number is outside the conventional range.
    InvalidAddress,
    /// The vendor field indicates that no function is present.
    NotPresent,
    /// The snapshot is not a type-0 endpoint header.
    UnsupportedHeaderType,
}

impl PciDeviceDescriptor {
    /// Decodes identity fields from a fixed-size type-0 PCI configuration
    /// snapshot. This does not enumerate hardware or grant device resources.
    pub fn from_type0_config(
        address: PciAddress,
        config: &[u8; 256],
    ) -> Result<Self, PciHeaderError> {
        if address.device > 31 || address.function > 7 {
            return Err(PciHeaderError::InvalidAddress);
        }

        let vendor_id = u16::from_le_bytes([config[0x00], config[0x01]]);
        if vendor_id == u16::MAX {
            return Err(PciHeaderError::NotPresent);
        }
        if config[0x0e] & 0x7f != 0 {
            return Err(PciHeaderError::UnsupportedHeaderType);
        }

        Ok(Self {
            address,
            vendor_id,
            device_id: u16::from_le_bytes([config[0x02], config[0x03]]),
            revision: config[0x08],
            programming_interface: config[0x09],
            subclass: config[0x0a],
            class: config[0x0b],
            subsystem_vendor_id: Some(u16::from_le_bytes([config[0x2c], config[0x2d]])),
            subsystem_device_id: Some(u16::from_le_bytes([config[0x2e], config[0x2f]])),
        })
    }
}

/// Exact-value PCI selectors for a driver registration.
///
/// More specific registrations are ranked by subsystem identity, vendor/device
/// identity, class hierarchy, then revision. A match with equal specificity to
/// another registration is rejected as ambiguous; registration order is never
/// used as a tie-breaker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciMatch {
    /// Required vendor identifier, if constrained.
    pub vendor_id: Option<u16>,
    /// Required device identifier, if constrained.
    pub device_id: Option<u16>,
    /// Required base class, if constrained.
    pub class: Option<u8>,
    /// Required subclass, if constrained.
    pub subclass: Option<u8>,
    /// Required programming interface, if constrained.
    pub programming_interface: Option<u8>,
    /// Required revision, if constrained.
    pub revision: Option<u8>,
    /// Required subsystem vendor, if constrained.
    pub subsystem_vendor_id: Option<u16>,
    /// Required subsystem device, if constrained.
    pub subsystem_device_id: Option<u16>,
}

impl PciMatch {
    /// Creates a selector with no constraints.
    ///
    /// An unconstrained selector is invalid for registration.
    pub const fn any() -> Self {
        Self {
            vendor_id: None,
            device_id: None,
            class: None,
            subclass: None,
            programming_interface: None,
            revision: None,
            subsystem_vendor_id: None,
            subsystem_device_id: None,
        }
    }

    /// Checks selector structure before registration.
    pub const fn is_valid(self) -> bool {
        let has_identity = self.vendor_id.is_some() || self.device_id.is_some();
        let has_class = self.class.is_some();
        let has_subsystem =
            self.subsystem_vendor_id.is_some() || self.subsystem_device_id.is_some();

        (has_identity || has_class)
            && (self.vendor_id.is_some() || self.device_id.is_none())
            && (self.subclass.is_none() || has_class)
            && (self.subclass.is_some() || self.programming_interface.is_none())
            && (!has_subsystem
                || (self.vendor_id.is_some()
                    && self.device_id.is_some()
                    && self.subsystem_vendor_id.is_some()
                    && self.subsystem_device_id.is_some()))
    }

    /// Returns whether all selector fields match the device.
    pub fn matches(self, device: &PciDeviceDescriptor) -> bool {
        self.is_valid()
            && self.vendor_id.is_none_or(|value| value == device.vendor_id)
            && self.device_id.is_none_or(|value| value == device.device_id)
            && self.class.is_none_or(|value| value == device.class)
            && self.subclass.is_none_or(|value| value == device.subclass)
            && self
                .programming_interface
                .is_none_or(|value| value == device.programming_interface)
            && self.revision.is_none_or(|value| value == device.revision)
            && self
                .subsystem_vendor_id
                .is_none_or(|value| Some(value) == device.subsystem_vendor_id)
            && self
                .subsystem_device_id
                .is_none_or(|value| Some(value) == device.subsystem_device_id)
    }

    const fn specificity(self) -> PciSpecificity {
        PciSpecificity {
            subsystem: self.subsystem_vendor_id.is_some() as u8
                + self.subsystem_device_id.is_some() as u8,
            identity: self.vendor_id.is_some() as u8 + self.device_id.is_some() as u8,
            class: self.class.is_some() as u8
                + self.subclass.is_some() as u8
                + self.programming_interface.is_some() as u8,
            revision: self.revision.is_some(),
        }
    }
}

/// Static identity and PCI matching rule registered by a driver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DriverRegistration {
    /// Stable driver identifier.
    pub id: DriverId,
    /// Human-readable diagnostic name.
    pub name: &'static str,
    /// Driver implementation version.
    pub version: DriverVersion,
    /// Device category produced by this driver.
    pub class: DeviceClass,
    /// Supported PCI identity rule.
    pub pci: PciMatch,
}

/// A selected driver registration and its match specificity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DriverMatch {
    /// Selected registered driver.
    pub registration: DriverRegistration,
    /// Specificity used for deterministic selection.
    pub specificity: PciSpecificity,
}

/// A selected driver registration paired with its concrete initializer.
///
/// The factory type is supplied by the kernel-facing registry, keeping this
/// crate independent of transport resources and concrete driver crates.
/// Statically linked function pointers are used; registration allocates no
/// memory and does not load executable code.
#[derive(Clone, Copy)]
pub struct DriverFactoryMatch<F: Copy> {
    /// Selected registered driver and match specificity.
    pub driver_match: DriverMatch,
    /// Initializer registered for the selected driver.
    pub factory: F,
}

/// Specificity of a PCI match; fields are ordered from strongest to weakest.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PciSpecificity {
    /// Number of exact subsystem identifiers.
    pub subsystem: u8,
    /// Number of exact vendor/device identifiers.
    pub identity: u8,
    /// Number of exact class, subclass, and programming-interface fields.
    pub class: u8,
    /// Whether the PCI revision is constrained.
    pub revision: bool,
}

/// Errors while registering or selecting drivers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryError {
    /// No free registration slot remains.
    Full,
    /// A driver identifier is already registered.
    DuplicateDriverId,
    /// The PCI selector is empty or structurally inconsistent.
    InvalidPciMatch,
    /// Multiple registrations have the same highest specificity.
    AmbiguousMatch,
}

#[derive(Clone, Copy)]
struct RegisteredDriver<F> {
    registration: DriverRegistration,
    factory: F,
}

/// Fixed-capacity PCI driver registry; it allocates no memory.
pub struct DriverRegistry<const N: usize, F = ()> {
    entries: [Option<RegisteredDriver<F>>; N],
    len: usize,
}

impl<const N: usize> DriverRegistry<N, ()> {
    /// Creates an empty registry.
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            len: 0,
        }
    }

    /// Registers one PCI driver without replacing existing entries.
    pub fn register(&mut self, registration: DriverRegistration) -> Result<(), RegistryError> {
        self.register_with_factory(registration, ())
    }
}

impl<const N: usize, F: Copy> DriverRegistry<N, F> {
    /// Creates an empty registry whose entries each carry an initializer.
    pub const fn with_factories() -> Self {
        Self {
            entries: [None; N],
            len: 0,
        }
    }

    /// Registers one PCI driver and the initializer associated with it.
    pub fn register_with_factory(
        &mut self,
        registration: DriverRegistration,
        factory: F,
    ) -> Result<(), RegistryError> {
        if !registration.pci.is_valid() {
            return Err(RegistryError::InvalidPciMatch);
        }
        if self
            .entries
            .iter()
            .flatten()
            .any(|entry| entry.registration.id == registration.id)
        {
            return Err(RegistryError::DuplicateDriverId);
        }
        let Some(slot) = self.entries.iter_mut().find(|entry| entry.is_none()) else {
            return Err(RegistryError::Full);
        };
        *slot = Some(RegisteredDriver {
            registration,
            factory,
        });
        self.len += 1;
        Ok(())
    }

    /// Selects the unique most-specific matching driver without its factory.
    ///
    /// Metadata-only consumers can continue to use this method. Activation
    /// paths should use [`Self::best_match_with_factory`].
    pub fn best_match(
        &self,
        device: &PciDeviceDescriptor,
    ) -> Result<Option<DriverMatch>, RegistryError> {
        Ok(self
            .best_entry(device)?
            .map(|entry| Self::driver_match(entry.registration)))
    }

    /// Selects the unique most-specific matching driver and its initializer.
    ///
    /// Returns `Ok(None)` when no driver matches and `AmbiguousMatch` when
    /// equally specific registrations match. Callers must not initialize a
    /// driver when selection is ambiguous.
    pub fn best_match_with_factory(
        &self,
        device: &PciDeviceDescriptor,
    ) -> Result<Option<DriverFactoryMatch<F>>, RegistryError> {
        Ok(self.best_entry(device)?.map(|entry| DriverFactoryMatch {
            driver_match: Self::driver_match(entry.registration),
            factory: entry.factory,
        }))
    }

    fn driver_match(registration: DriverRegistration) -> DriverMatch {
        DriverMatch {
            registration,
            specificity: registration.pci.specificity(),
        }
    }

    fn best_entry(
        &self,
        device: &PciDeviceDescriptor,
    ) -> Result<Option<RegisteredDriver<F>>, RegistryError> {
        let mut best: Option<RegisteredDriver<F>> = None;
        let mut ambiguous = false;

        for entry in self.entries.iter().flatten() {
            if !entry.registration.pci.matches(device) {
                continue;
            }
            match best {
                None => {
                    best = Some(*entry);
                    ambiguous = false;
                }
                Some(current)
                    if entry.registration.pci.specificity()
                        > current.registration.pci.specificity() =>
                {
                    best = Some(*entry);
                    ambiguous = false;
                }
                Some(current)
                    if entry.registration.pci.specificity()
                        == current.registration.pci.specificity() =>
                {
                    ambiguous = true;
                }
                Some(_) => {}
            }
        }

        if ambiguous {
            Err(RegistryError::AmbiguousMatch)
        } else {
            Ok(best)
        }
    }
}

impl<const N: usize, F> DriverRegistry<N, F> {
    /// Returns the number of registered drivers.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the registry has no registrations.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const N: usize> Default for DriverRegistry<N, ()> {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a device operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceError {
    /// Device is absent or has been disconnected.
    NoDevice,
    /// Device or operation is not supported.
    Unsupported,
    /// The operation cannot complete before its deadline.
    Timeout,
    /// A nonblocking operation has no completed data.
    WouldBlock,
    /// The device reported malformed or inconsistent data.
    Protocol,
    /// The device is busy with an operation that cannot be queued.
    Busy,
    /// A device I/O operation failed.
    Io,
}

/// Common metadata interface implemented by initialized devices.
pub trait Device {
    /// Returns trusted identity and lifecycle metadata.
    fn metadata(&self) -> DeviceMetadata;
}

/// Block storage operations over bounded caller-owned buffers.
pub trait BlockDevice: Device {
    /// Number of addressable blocks.
    fn block_count(&self) -> u64;
    /// Bytes in one block.
    fn block_size(&self) -> u32;
    /// Reads whole contiguous blocks into the supplied buffer.
    fn read_blocks(&mut self, first_block: u64, output: &mut [u8]) -> Result<(), DeviceError>;
    /// Writes whole contiguous blocks from the supplied buffer.
    fn write_blocks(&mut self, first_block: u64, input: &[u8]) -> Result<(), DeviceError>;
    /// Makes previously accepted writes durable according to the device contract.
    fn flush(&mut self) -> Result<(), DeviceError>;
}

/// Packet-oriented network operations; no network policy is implied.
pub trait NetworkDevice: Device {
    /// Returns the device's link-layer address.
    fn hardware_address(&self) -> [u8; 6];
    /// Queues one complete frame for transmission.
    fn transmit(&mut self, frame: &[u8]) -> Result<(), DeviceError>;
    /// Copies one received frame into caller-owned storage.
    fn receive(&mut self, output: &mut [u8]) -> Result<usize, DeviceError>;
    /// Returns the current physical/link status when available.
    fn link_up(&self) -> Result<bool, DeviceError>;
}

/// Pixel layout reported by a display device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisplayMode {
    /// Visible horizontal pixels.
    pub width: u32,
    /// Visible vertical pixels.
    pub height: u32,
    /// Bytes between the starts of adjacent rows.
    pub stride_bytes: u32,
    /// Pixel encoding.
    pub format: PixelFormat,
}

impl DisplayMode {
    /// Returns the minimum byte length for a complete frame, checking overflow.
    pub fn frame_bytes(self) -> Option<usize> {
        let row_bytes = (self.width as usize).checked_mul(self.format.bytes_per_pixel())?;
        if (self.stride_bytes as usize) < row_bytes {
            return None;
        }
        (self.height as usize)
            .checked_sub(1)?
            .checked_mul(self.stride_bytes as usize)?
            .checked_add(row_bytes)
    }
}

/// Supported unaccelerated pixel encodings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PixelFormat {
    /// Red, green, blue, and unused byte in memory order.
    Rgbx8888,
    /// Blue, green, red, and unused byte in memory order.
    Bgrx8888,
    /// Packed 5:6:5 RGB.
    Rgb565,
}

impl PixelFormat {
    /// Number of bytes in one encoded pixel.
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgbx8888 | Self::Bgrx8888 => 4,
            Self::Rgb565 => 2,
        }
    }
}

/// Rectangle submitted for display.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisplayRect {
    /// Left coordinate in pixels.
    pub x: u32,
    /// Top coordinate in pixels.
    pub y: u32,
    /// Rectangle width in pixels.
    pub width: u32,
    /// Rectangle height in pixels.
    pub height: u32,
}

/// Unaccelerated display operations. Implementations copy pixels and never
/// expose a physical framebuffer address through this interface.
pub trait DisplayDevice: Device {
    /// Returns the active display mode.
    fn mode(&self) -> DisplayMode;
    /// Presents tightly packed pixels for a rectangle within the active mode.
    fn present(&mut self, rect: DisplayRect, pixels: &[u8]) -> Result<(), DeviceError>;
}

/// Input event from an initialized input device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputEvent {
    /// A key identified by its hardware-independent usage code.
    Key {
        /// Usage code.
        usage: u16,
        /// Whether the key is pressed.
        pressed: bool,
    },
    /// Relative pointer movement and button state.
    Pointer {
        /// Horizontal movement since the preceding event.
        dx: i32,
        /// Vertical movement since the preceding event.
        dy: i32,
        /// Current button bitset.
        buttons: u8,
    },
}

/// Nonblocking input event source.
pub trait InputDevice: Device {
    /// Returns the next event, or `WouldBlock` when no event is available.
    fn next_event(&mut self) -> Result<InputEvent, DeviceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK_ID: DriverId = DriverId::new(1);
    const NET_ID: DriverId = DriverId::new(2);

    fn descriptor() -> PciDeviceDescriptor {
        PciDeviceDescriptor {
            address: PciAddress {
                segment: 0,
                bus: 0,
                device: 5,
                function: 0,
            },
            vendor_id: 0x1af4,
            device_id: 0x1042,
            class: 0x01,
            subclass: 0x00,
            programming_interface: 0x00,
            revision: 1,
            subsystem_vendor_id: Some(0x1af4),
            subsystem_device_id: Some(0x1100),
        }
    }

    #[test]
    fn type0_header_decodes_identity_and_subsystem_fields() {
        let mut config = [0; 256];
        config[0x00..0x04].copy_from_slice(&[0xf4, 0x1a, 0x42, 0x10]);
        config[0x08..0x0c].copy_from_slice(&[1, 0, 0, 1]);
        config[0x0e] = 0x80;
        config[0x2c..0x30].copy_from_slice(&[0xf4, 0x1a, 0x00, 0x11]);
        let decoded = PciDeviceDescriptor::from_type0_config(
            PciAddress {
                segment: 0,
                bus: 0,
                device: 5,
                function: 0,
            },
            &config,
        )
        .unwrap();
        assert_eq!(decoded, descriptor());
    }

    #[test]
    fn type0_header_rejects_absent_functions_invalid_bdf_and_other_headers() {
        let address = descriptor().address;
        assert_eq!(
            PciDeviceDescriptor::from_type0_config(address, &[0xff; 256]),
            Err(PciHeaderError::NotPresent)
        );
        assert_eq!(
            PciDeviceDescriptor::from_type0_config(
                PciAddress {
                    device: 32,
                    ..address
                },
                &[0; 256]
            ),
            Err(PciHeaderError::InvalidAddress)
        );
        assert_eq!(
            PciDeviceDescriptor::from_type0_config(
                PciAddress {
                    function: 8,
                    ..address
                },
                &[0; 256]
            ),
            Err(PciHeaderError::InvalidAddress)
        );
        let mut bridge = [0; 256];
        bridge[0x00] = 0xf4;
        bridge[0x0e] = 1;
        assert_eq!(
            PciDeviceDescriptor::from_type0_config(address, &bridge),
            Err(PciHeaderError::UnsupportedHeaderType)
        );
    }

    const fn registration(id: DriverId, pci: PciMatch) -> DriverRegistration {
        DriverRegistration {
            id,
            name: "test-driver",
            version: DriverVersion::new(1, 0, 0),
            class: DeviceClass::BlockDevice,
            pci,
        }
    }

    #[test]
    fn pci_match_checks_all_identity_and_class_fields() {
        let mut matcher = PciMatch::any();
        matcher.vendor_id = Some(0x1af4);
        matcher.device_id = Some(0x1042);
        matcher.class = Some(0x01);
        matcher.subclass = Some(0x00);
        matcher.programming_interface = Some(0x00);
        matcher.revision = Some(1);
        matcher.subsystem_vendor_id = Some(0x1af4);
        matcher.subsystem_device_id = Some(0x1100);
        assert!(matcher.matches(&descriptor()));
        assert!(!matcher.matches(&PciDeviceDescriptor {
            device_id: 0x1043,
            ..descriptor()
        }));
        assert!(!matcher.matches(&PciDeviceDescriptor {
            programming_interface: 1,
            ..descriptor()
        }));
        assert!(!matcher.matches(&PciDeviceDescriptor {
            revision: 2,
            ..descriptor()
        }));
        assert!(!matcher.matches(&PciDeviceDescriptor {
            subsystem_device_id: Some(0x1101),
            ..descriptor()
        }));
    }

    #[test]
    fn malformed_or_wildcard_selectors_are_rejected() {
        assert!(!PciMatch::any().is_valid());
        assert!(!PciMatch {
            device_id: Some(1),
            ..PciMatch::any()
        }
        .is_valid());
        assert!(!PciMatch {
            vendor_id: Some(1),
            subclass: Some(2),
            ..PciMatch::any()
        }
        .is_valid());
        assert!(!PciMatch {
            vendor_id: Some(1),
            device_id: Some(2),
            subsystem_vendor_id: Some(3),
            ..PciMatch::any()
        }
        .is_valid());
    }

    #[test]
    fn registry_selects_most_specific_match_not_registration_order() {
        let broad = PciMatch {
            vendor_id: Some(0x1af4),
            ..PciMatch::any()
        };
        let exact = PciMatch {
            vendor_id: Some(0x1af4),
            device_id: Some(0x1042),
            ..PciMatch::any()
        };
        let mut registry = DriverRegistry::<2>::new();
        registry.register(registration(BLOCK_ID, broad)).unwrap();
        registry.register(registration(NET_ID, exact)).unwrap();

        let selected = registry.best_match(&descriptor()).unwrap().unwrap();
        assert_eq!(selected.registration.id, NET_ID);
        assert_eq!(selected.specificity.identity, 2);
    }

    #[test]
    fn factory_registry_returns_initializer_for_selected_registration() {
        fn block_factory() -> u8 {
            1
        }
        fn network_factory() -> u8 {
            2
        }

        let broad = PciMatch {
            vendor_id: Some(0x1af4),
            ..PciMatch::any()
        };
        let exact = PciMatch {
            vendor_id: Some(0x1af4),
            device_id: Some(0x1042),
            ..PciMatch::any()
        };
        let mut registry = DriverRegistry::<2, fn() -> u8>::with_factories();
        registry
            .register_with_factory(registration(BLOCK_ID, broad), block_factory)
            .unwrap();
        registry
            .register_with_factory(registration(NET_ID, exact), network_factory)
            .unwrap();

        let selected = registry
            .best_match_with_factory(&descriptor())
            .unwrap()
            .unwrap();
        assert_eq!(selected.driver_match.registration.id, NET_ID);
        assert_eq!((selected.factory)(), 2);
    }

    #[test]
    fn subsystem_match_outranks_device_and_class_matches() {
        let device = PciMatch {
            vendor_id: Some(0x1af4),
            device_id: Some(0x1042),
            ..PciMatch::any()
        };
        let subsystem = PciMatch {
            vendor_id: Some(0x1af4),
            device_id: Some(0x1042),
            subsystem_vendor_id: Some(0x1af4),
            subsystem_device_id: Some(0x1100),
            ..PciMatch::any()
        };
        let mut registry = DriverRegistry::<2>::new();
        registry.register(registration(BLOCK_ID, device)).unwrap();
        registry.register(registration(NET_ID, subsystem)).unwrap();
        assert_eq!(
            registry
                .best_match(&descriptor())
                .unwrap()
                .unwrap()
                .registration
                .id,
            NET_ID
        );
    }

    #[test]
    fn equally_specific_matches_fail_closed() {
        let matcher = PciMatch {
            vendor_id: Some(0x1af4),
            device_id: Some(0x1042),
            ..PciMatch::any()
        };
        let mut registry = DriverRegistry::<2>::new();
        registry.register(registration(BLOCK_ID, matcher)).unwrap();
        registry.register(registration(NET_ID, matcher)).unwrap();
        assert_eq!(
            registry.best_match(&descriptor()),
            Err(RegistryError::AmbiguousMatch)
        );
    }

    #[test]
    fn registry_rejects_duplicate_ids_and_capacity_overflow() {
        let matcher = PciMatch {
            vendor_id: Some(0x1af4),
            ..PciMatch::any()
        };
        let mut registry = DriverRegistry::<1>::new();
        registry.register(registration(BLOCK_ID, matcher)).unwrap();
        assert_eq!(
            registry.register(registration(BLOCK_ID, matcher)),
            Err(RegistryError::DuplicateDriverId)
        );
        assert_eq!(
            registry.register(registration(NET_ID, matcher)),
            Err(RegistryError::Full)
        );
    }

    #[test]
    fn unknown_device_has_no_match() {
        let matcher = PciMatch {
            vendor_id: Some(0x1af4),
            device_id: Some(0x1042),
            ..PciMatch::any()
        };
        let mut registry = DriverRegistry::<1>::new();
        registry.register(registration(BLOCK_ID, matcher)).unwrap();
        let unknown = PciDeviceDescriptor {
            vendor_id: 0x1234,
            device_id: 0xabcd,
            ..descriptor()
        };
        assert_eq!(registry.best_match(&unknown), Ok(None));
    }

    #[test]
    fn display_frame_length_checks_stride_and_overflow() {
        let mode = DisplayMode {
            width: 10,
            height: 2,
            stride_bytes: 48,
            format: PixelFormat::Rgbx8888,
        };
        assert_eq!(mode.frame_bytes(), Some(88));
        assert_eq!(
            DisplayMode {
                stride_bytes: 39,
                ..mode
            }
            .frame_bytes(),
            None
        );
        assert_eq!(
            DisplayMode {
                width: u32::MAX,
                height: u32::MAX,
                stride_bytes: u32::MAX,
                ..mode
            }
            .frame_bytes(),
            None
        );
    }
}
