#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
//! Exact QEMU 00:05.0 modern virtio-blk transport. No user address enters here.
//! Caller retains five exclusive, identity-mapped kernel DMA pages until reboot.
//! Protocol-conforming QEMU is trusted; this is not a malicious-device IOMMU.
use core::{
    ptr,
    sync::atomic::{fence, Ordering},
};
use driver_core::PciAddress;
use pci_core::{MatchedPciFunction, Mechanism1, PciConfigAccess};
use virtio_core::{descriptor_plan, parse_capabilities, validate_descriptors, QUEUE_SIZE};
pub use virtio_core::{Bar, Completion, DmaPool, Error, Regions};
pub const DRIVER_REGISTRATION: driver_core::DriverRegistration = driver_core::DriverRegistration {
    id: driver_core::DriverId::new(0x4e4f_0001),
    name: "nieuw-os-virtio-blk-qemu",
    version: driver_core::DriverVersion::new(0, 1, 0),
    class: driver_core::DeviceClass::BlockDevice,
    pci: driver_core::PciMatch {
        vendor_id: Some(0x1af4),
        device_id: Some(0x1042),
        class: None,
        subclass: None,
        programming_interface: None,
        revision: None,
        subsystem_vendor_id: None,
        subsystem_device_id: None,
    },
};
/// Matched block-device initializer selected by the PCI registry.
///
/// # Safety
/// The caller must uphold the permanent MMIO mapping and exclusive pinned-DMA
/// ownership contract documented by [`Device::initialize_profile`].
pub type DriverFactory =
    unsafe fn(Discovery, [u64; 6], DmaPool, bool, bool) -> Result<Device, Error>;
const DRIVER_FACTORY: DriverFactory = Device::initialize_matched;
const RAM_END: u64 = 256 * 1024 * 1024;
fn config_error(_: pci_core::ConfigError) -> Error {
    Error::Protocol
}
fn bar_error(error: pci_core::PciBarError) -> Error {
    match error {
        pci_core::PciBarError::Config(_) | pci_core::PciBarError::Restore => Error::Protocol,
        pci_core::PciBarError::Unsupported => Error::Unsupported,
        pci_core::PciBarError::Range => Error::Range,
    }
}

fn select_device() -> Result<PciAddress, Error> {
    let mut registry = driver_core::DriverRegistry::<1, DriverFactory>::with_factories();
    registry
        .register_with_factory(DRIVER_REGISTRATION, DRIVER_FACTORY)
        .map_err(|_| Error::Protocol)?;
    let mut selected = None;
    let mut ambiguous = false;
    // SAFETY: Called synchronously during single-CPU kernel initialization
    // before device services or interrupts can access PCI configuration.
    let mut access = unsafe { Mechanism1::new() };
    pci_core::enumerate(&mut access, |descriptor| {
        match registry.best_match_with_factory(&descriptor) {
            Ok(Some(_)) => {
                if selected.replace(descriptor.address).is_some() {
                    ambiguous = true;
                }
            }
            Err(_) => ambiguous = true,
            _ => {}
        }
    })
    .map_err(|_| Error::Protocol)?;
    if ambiguous {
        return Err(Error::Protocol);
    }
    selected.ok_or(Error::Offline)
}

fn validate_registration(
    address: PciAddress,
    config: &[u8; 256],
) -> Result<driver_core::DriverFactoryMatch<DriverFactory>, Error> {
    let descriptor =
        driver_core::PciDeviceDescriptor::from_type0_config(address, config).map_err(|error| {
            match error {
                driver_core::PciHeaderError::NotPresent => Error::Offline,
                driver_core::PciHeaderError::InvalidAddress
                | driver_core::PciHeaderError::UnsupportedHeaderType => Error::Protocol,
            }
        })?;
    let mut registry = driver_core::DriverRegistry::<1, DriverFactory>::with_factories();
    registry
        .register_with_factory(DRIVER_REGISTRATION, DRIVER_FACTORY)
        .map_err(|_| Error::Protocol)?;
    match registry.best_match_with_factory(&descriptor) {
        Ok(Some(matched)) => Ok(matched),
        Ok(None) => Err(Error::Offline),
        Err(_) => Err(Error::Protocol),
    }
}
fn bind_error(error: pci_core::BindError) -> Error {
    match error {
        pci_core::BindError::Config(error) => config_error(error),
        pci_core::BindError::Header(_)
        | pci_core::BindError::DescriptorChanged
        | pci_core::BindError::DriverMismatch => Error::Protocol,
    }
}
#[derive(Clone, Copy)]
pub struct Discovery {
    /// PCI function selected by generic segment-zero enumeration and matching.
    pub address: PciAddress,
    pub config: [u8; 256],
    pub bars: [Bar; 6],
    /// Kernel-usable BAR grants from the matched PCI function.
    pub bar_grants: [Option<pci_core::PciBarGrant>; 6],
    pub regions: Regions,
    driver_factory: DriverFactory,
}
pub fn discover() -> Result<Discovery, Error> {
    discover_at(select_device()?)
}

/// Discovers and validates one registry-selected type-0 VirtIO block function.
pub fn discover_at(address: PciAddress) -> Result<Discovery, Error> {
    // SAFETY: Called synchronously during single-CPU kernel initialization
    // before device services or interrupts can access PCI configuration.
    let mut pci = unsafe { Mechanism1::new() };
    if pci.read_u32(address, 0).map_err(config_error)? != 0x10421af4 {
        return Err(Error::Offline);
    }
    let mut config = [0u8; 256];
    for offset in (0..256).step_by(4) {
        config[offset..offset + 4].copy_from_slice(
            &pci.read_u32(address, offset as u8)
                .map_err(config_error)?
                .to_le_bytes(),
        );
    }
    let descriptor = driver_core::PciDeviceDescriptor::from_type0_config(address, &config)
        .map_err(|_| Error::Protocol)?;
    let matched = validate_registration(address, &config)?;
    let mut pci =
        MatchedPciFunction::bind(pci, descriptor, matched.driver_match).map_err(bind_error)?;
    let original = pci.read_u32(address, 4).map_err(config_error)? as u16;
    let bar_grants = pci.probe_memory_bars().map_err(bar_error)?;
    let mut bars = [Bar::NONE; 6];
    for (index, grant) in bar_grants.iter().enumerate() {
        if let Some(grant) = grant {
            if grant.size() > 65536 || grant.base() % 4096 != 0 {
                return Err(Error::Range);
            }
            bars[index] = Bar {
                base: grant.base(),
                size: grant.size(),
                is_memory: true,
            };
        }
    }
    pci.write_u16(address, 4, (original & !5) | 2 | (1 << 10))
        .map_err(config_error)?; // MMIO decode on; DMA remains off.
    for offset in (0..256).step_by(4) {
        config[offset..offset + 4].copy_from_slice(
            &pci.read_u32(address, offset as u8)
                .map_err(config_error)?
                .to_le_bytes(),
        );
    }
    let matched_after_probe = validate_registration(address, &config)?;
    if matched_after_probe.driver_match.registration.id != matched.driver_match.registration.id {
        return Err(Error::Protocol);
    }
    let regions = parse_capabilities(&config, &bars)?;
    // Disable MSI/MSI-X as well as INTx; first driver polls without IRQ routing.
    let mut at = config[0x34] as usize;
    let mut count = 0;
    while at != 0 && count < 48 {
        if !(0x40..=0xfc).contains(&at) || !at.is_multiple_of(4) {
            return Err(Error::Protocol);
        }
        let word = pci.read_u32(address, at as u8).map_err(config_error)?;
        if config[at] == 5 {
            pci.write_u32(address, at as u8, word & !(1 << 16))
                .map_err(config_error)?;
        }
        if config[at] == 0x11 {
            pci.write_u32(address, at as u8, (word & !(1 << 31)) | (1 << 30))
                .map_err(config_error)?;
        }
        at = config[at + 1] as usize;
        count += 1;
    }
    if at != 0 {
        return Err(Error::Protocol);
    }
    Ok(Discovery {
        address,
        config,
        bars,
        bar_grants,
        regions,
        driver_factory: matched.factory,
    })
}
pub struct Device {
    address: PciAddress,
    pci: MatchedPciFunction<Mechanism1>,
    common: u64,
    notify: u64,
    pub pool: DmaPool,
    pub capacity: u64,
    available: u16,
    used: u16,
    pending: bool,
    ready: bool,
    operation: virtio_core::Operation,
    prepared: bool,
}
impl Device {
    /// Kernel passes only supervisor UC mappings corresponding to the validated
    /// BARs, and exclusive normal-RAM DMA pages. These lifetimes last to reboot.
    ///
    /// # Safety
    /// `mapped` must contain the permanent supervisor-only UC mappings for the
    /// BAR regions validated in `info`. Every page in `pool` must be an exclusive,
    /// pinned normal-RAM DMA page owned by this device for the full device lifetime.
    /// `info.address` must be the exact function selected by trusted PCI matching,
    /// and `mapped`/`pool` must belong only to that function. Initialization
    /// requires single-core execution with interrupts disabled and exclusive
    /// serialized access to PCI configuration ports.
    pub unsafe fn initialize(
        info: Discovery,
        mapped: [u64; 6],
        pool: DmaPool,
    ) -> Result<Self, Error> {
        // SAFETY: Caller upholds the mapping and DMA ownership contract above;
        // the factory was selected by this discovery's validated registry match.
        unsafe { (info.driver_factory)(info, mapped, pool, false, false) }
    }
    /// Same mapping/pool contract as initialize. Writable mode is explicit;
    /// no_flush is a test-only advertisement snapshot selected by the kernel.
    ///
    /// # Safety
    /// The caller must uphold the same permanent MMIO mapping and exclusive
    /// pinned-DMA ownership requirements as [`Self::initialize`]. `writable` and
    /// `no_flush` may select protocol behavior but do not relax those memory rules.
    pub unsafe fn initialize_profile(
        info: Discovery,
        mapped: [u64; 6],
        pool: DmaPool,
        writable: bool,
        no_flush: bool,
    ) -> Result<Self, Error> {
        // SAFETY: Caller upholds the mapping and DMA ownership contract above;
        // the factory was selected by this discovery's validated registry match.
        unsafe { (info.driver_factory)(info, mapped, pool, writable, no_flush) }
    }

    unsafe fn initialize_matched(
        info: Discovery,
        mapped: [u64; 6],
        pool: DmaPool,
        writable: bool,
        no_flush: bool,
    ) -> Result<Self, Error> {
        pool.validate(RAM_END)?;
        // SAFETY: Initialization happens synchronously before interrupts or a
        // second caller can access the serialized PCI config mechanism.
        let access = unsafe { Mechanism1::new() };
        let descriptor =
            driver_core::PciDeviceDescriptor::from_type0_config(info.address, &info.config)
                .map_err(|_| Error::Protocol)?;
        let matched = validate_registration(info.address, &info.config)?;
        let pci = MatchedPciFunction::bind(access, descriptor, matched.driver_match)
            .map_err(bind_error)?;
        let address = |region: virtio_core::Region| -> Result<u64, Error> {
            let base = mapped[region.bar as usize];
            if base == 0 {
                return Err(Error::Range);
            }
            base.checked_add(u64::from(region.offset))
                .ok_or(Error::Range)
        };
        let common = address(info.regions.common)?;
        let device = address(info.regions.device)?;
        if common % 8 != 0 || device % 8 != 0 {
            return Err(Error::Range);
        }
        let mut result = Self {
            address: info.address,
            pci,
            common,
            notify: 0,
            pool,
            capacity: 0,
            available: 0,
            used: 0,
            pending: false,
            ready: false,
            operation: virtio_core::Operation::Read {
                sector: 0,
                offset: 0,
                length: 512,
            },
            prepared: false,
        };
        // SAFETY: All addresses come from validated supervisor UC BAR mappings.
        unsafe {
            write8(common + 20, 0);
            if read8(common + 20) != 0 {
                return Err(Error::Timeout);
            }
            write8(common + 20, 1);
            write8(common + 20, 3);
            write32(common, 0);
            let low = read32(common + 4);
            write32(common, 1);
            let high = read32(common + 4);
            let mut offered = u64::from(low) | (u64::from(high) << 32);
            if no_flush {
                offered &= !(1 << 9);
            }
            let features = if writable {
                virtio_core::negotiate_writable_features(offered)?
            } else {
                virtio_core::negotiate_features(offered)?
            };
            write32(common + 8, 0);
            write32(common + 12, features as u32);
            write32(common + 8, 1);
            write32(common + 12, (features >> 32) as u32);
            write8(common + 20, 11);
            if read8(common + 20) & 8 == 0 {
                return Err(Error::Unsupported);
            }
            if read16(common + 18) == 0 {
                return Err(Error::Protocol);
            }
            write16(common + 22, 0);
            if read16(common + 24) < QUEUE_SIZE {
                return Err(Error::Unsupported);
            }
            write16(common + 24, QUEUE_SIZE);
            write16(common + 16, 0xffff);
            write16(common + 26, 0xffff);
            for page in [
                pool.descriptors,
                pool.available,
                pool.used,
                pool.request,
                pool.bounce,
            ] {
                ptr::write_bytes(page as *mut u8, 0, 4096);
            }
            write16(pool.available, 1);
            write64(common + 32, pool.descriptors);
            write64(common + 40, pool.available);
            write64(common + 48, pool.used);
            let queue_offset = read16(common + 30);
            let relative = info.regions.notify_address(&info.bars, queue_offset)?
                - info.bars[info.regions.notify.bar as usize].base;
            result.notify = mapped[info.regions.notify.bar as usize]
                .checked_add(relative)
                .ok_or(Error::Range)?;
            if !result.notify.is_multiple_of(2) {
                return Err(Error::Range);
            }
            write16(common + 28, 1);
            if read16(common + 28) != 1 {
                return Err(Error::Protocol);
            }
            let mut capacity = None;
            for _ in 0..4 {
                let generation = read8(common + 21);
                let value = read64(device);
                if generation == read8(common + 21) {
                    capacity = Some(value);
                    break;
                }
            }
            result.capacity = capacity.ok_or(Error::Protocol)?;
            if result.capacity < 8 {
                return Err(Error::Range);
            }
            fence(Ordering::SeqCst);
            let command = result
                .pci
                .read_u32(result.address, 4)
                .map_err(config_error)? as u16;
            result
                .pci
                .write_u16(result.address, 4, command | 6 | (1 << 10))
                .map_err(config_error)?;
            write8(common + 20, 15);
            if read8(common + 20) != 15 {
                // Failure after BME-enable is not an idle/offline device: first
                // request reset and disable bus mastering. Caller quarantines
                // its permanent pool; no future reuse is authorized by this Err.
                write8(common + 20, 0);
                let command = result
                    .pci
                    .read_u32(result.address, 4)
                    .map_err(config_error)? as u16;
                result
                    .pci
                    .write_u16(result.address, 4, (command & !4) | (1 << 10))
                    .map_err(config_error)?;
                return Err(Error::Protocol);
            }
        }
        result.ready = true;
        Ok(result)
    }
    pub fn submit(
        &mut self,
        sector: u64,
        suppress_notify: bool,
        bad_descriptor: bool,
    ) -> Result<(), Error> {
        if !self.ready {
            return Err(Error::Offline);
        }
        if self.pending {
            return Err(Error::Busy);
        }
        if sector >= self.capacity {
            return Err(Error::Range);
        }
        self.operation = virtio_core::Operation::Read {
            sector,
            offset: 0,
            length: 512,
        };
        let mut descriptors = descriptor_plan(self.pool, RAM_END)?;
        if bad_descriptor {
            descriptors[1].address = 0x02000000;
        }
        validate_descriptors(&descriptors, self.pool, RAM_END)?;
        // SAFETY: Exclusive pinned pages, queue has no outstanding request.
        // Published descriptors are never altered until completion/reset ack.
        unsafe {
            ptr::write_bytes(self.pool.request as *mut u8, 0, 4096);
            ptr::write_bytes(self.pool.bounce as *mut u8, 0, 4096);
            write32(self.pool.request, 0);
            write64(self.pool.request + 8, sector);
            write8(self.pool.request + 16, 0xff);
            for (i, descriptor) in descriptors.into_iter().enumerate() {
                let bytes = descriptor.encode();
                for (j, byte) in bytes.into_iter().enumerate() {
                    write8(self.pool.descriptors + i as u64 * 16 + j as u64, byte);
                }
            }
            write16(
                self.pool.available + 4 + u64::from(self.available % QUEUE_SIZE) * 2,
                0,
            );
            fence(Ordering::SeqCst);
            self.available = self.available.wrapping_add(1);
            write16(self.pool.available + 2, self.available);
            fence(Ordering::SeqCst);
            self.pending = true;
            if !suppress_notify {
                write16(self.notify, 0);
            }
        }
        Ok(())
    }
    /// Prepare a validated fixed chain without exposing it to the device.
    /// Input is a kernel-owned exact sector snapshot, never a user address.
    pub fn prepare_operation(
        &mut self,
        operation: virtio_core::Operation,
        input: Option<&[u8; 512]>,
        bad_descriptor: bool,
    ) -> Result<(), Error> {
        if !self.ready {
            return Err(Error::Offline);
        }
        if self.pending || self.prepared {
            return Err(Error::Busy);
        }
        let sector = match operation {
            virtio_core::Operation::Read { sector, .. }
            | virtio_core::Operation::Write { sector } => Some(sector),
            _ => None,
        };
        if sector.is_some_and(|n| n >= self.capacity) {
            return Err(Error::Range);
        }
        if matches!(operation, virtio_core::Operation::Write { .. }) != input.is_some() {
            return Err(Error::Invalid);
        }
        let mut chain = virtio_core::descriptor_plan_for(operation, self.pool, RAM_END)?;
        if bad_descriptor {
            chain.entries[1].address = 0x02000000;
        }
        virtio_core::validate_descriptors_for(&chain, operation, self.pool, RAM_END)?;
        let header = virtio_core::header_for(operation);
        // SAFETY: No outstanding/exposed chain. Distinct exclusive pinned pages
        // are supervisor-only RAM. All writes finish before any avail publication.
        unsafe {
            ptr::write_bytes(self.pool.request as *mut u8, 0, 4096);
            ptr::write_bytes(self.pool.bounce as *mut u8, 0, 4096);
            for (i, b) in header.into_iter().enumerate() {
                write8(self.pool.request + i as u64, b);
            }
            if let Some(bytes) = input {
                for (i, b) in bytes.iter().copied().enumerate() {
                    write8(self.pool.bounce + i as u64, b);
                }
            }
            write8(self.pool.request + 16, 0xff);
            for (i, d) in chain.entries.into_iter().enumerate() {
                for (j, b) in d.encode().into_iter().enumerate() {
                    write8(self.pool.descriptors + i as u64 * 16 + j as u64, b);
                }
            }
            write16(
                self.pool.available + 4 + u64::from(self.available % QUEUE_SIZE) * 2,
                0,
            );
        }
        self.operation = operation;
        self.prepared = true;
        Ok(())
    }
    /// Kernel must first mark its matching policy ticket Published. Even a
    /// suppressed notify can be observed by a polling device after avail.idx.
    pub fn publish_prepared(&mut self, suppress_notify: bool) -> Result<(), Error> {
        if !self.ready || !self.prepared || self.pending {
            return Err(Error::Invalid);
        }
        self.prepared = false;
        self.pending = true;
        fence(Ordering::SeqCst);
        self.available = self.available.wrapping_add(1);
        // SAFETY: Fully validated immutable chain/snapshot, one producer. The
        // raw release publication is the only transition authorizing device DMA.
        unsafe {
            write16(self.pool.available + 2, self.available);
        }
        fence(Ordering::SeqCst);
        if !suppress_notify {
            self.kick();
        }
        Ok(())
    }
    /// Injection changes only a local snapshot after an actual device used-index
    /// advance. It never gives QEMU an unsafe descriptor or DMA target.
    pub fn poll(&mut self, injection: u8) -> Result<Option<([u8; 512], Completion)>, Error> {
        if !self.pending {
            return Ok(None);
        }
        // SAFETY: Device may write used/status/bounce concurrently. Read raw
        // volatile integers, then acquire ordering; no shared Rust references.
        unsafe {
            let mut next = read16(self.pool.used + 2);
            if next == self.used {
                return Ok(None);
            }
            fence(Ordering::SeqCst);
            let ring = self.pool.used + 4 + u64::from(self.used % QUEUE_SIZE) * 8;
            let mut id = read32(ring);
            let mut length = read32(ring + 4);
            let mut status = read8(self.pool.request + 16);
            match injection {
                5 => id = 8,
                6 => next = self.used.wrapping_add(2),
                7 => length = 512,
                8 => length = 514,
                9 => status = 0xff,
                20 => length = 2,
                10 => {
                    status = 1;
                    length = 1;
                }
                _ => {}
            }
            let outcome = virtio_core::validate_used_for(
                self.operation,
                0,
                self.used,
                next,
                id,
                length,
                status,
            )?;
            let mut data = [0u8; 512];
            if outcome == Completion::Success
                && matches!(self.operation, virtio_core::Operation::Read { .. })
            {
                for (i, byte) in data.iter_mut().enumerate() {
                    *byte = read8(self.pool.bounce + i as u64);
                }
            }
            self.used = next;
            self.pending = false;
            Ok(Some((data, outcome)))
        }
    }
    pub fn begin_reset(&mut self) -> Result<(), Error> {
        self.ready = false;
        // SAFETY: Fixed owned device; stop DMA authorization and request protocol
        // reset. Pages stay pinned even if the device never acknowledges reset.
        unsafe {
            write8(self.common + 20, 0);
        }
        let command = self.pci.read_u32(self.address, 4).map_err(config_error)? as u16;
        self.pci
            .write_u16(self.address, 4, (command & !4) | (1 << 10))
            .map_err(config_error)
    }
    pub fn reset_acknowledged(&mut self) -> bool {
        // SAFETY: Own UC status register; BME read confirms fixed-device disable.
        (unsafe { read8(self.common + 20) == 0 })
            && self
                .pci
                .read_u32(self.address, 4)
                .is_ok_and(|command| command & 4 == 0)
    }
    pub fn pending(&self) -> bool {
        self.pending
    }
    pub fn kick(&mut self) {
        if self.pending && self.ready {
            // SAFETY: Validated own queue notify register, no pointer argument.
            unsafe {
                write16(self.notify, 0);
            }
        }
    }
    pub fn status(&self) -> u8 {
        // SAFETY: Bounded read of own mapped status register.
        unsafe { read8(self.common + 20) }
    }
}
// Callers establish mappings, ownership, field bounds and alignment. These raw
// scalar accesses never construct a reference to concurrently DMA-updated RAM.
unsafe fn read8(a: u64) -> u8 {
    unsafe { (a as *const u8).read_volatile() }
}
unsafe fn read16(a: u64) -> u16 {
    unsafe { (a as *const u16).read_volatile() }
}
unsafe fn read32(a: u64) -> u32 {
    unsafe { (a as *const u32).read_volatile() }
}
unsafe fn read64(a: u64) -> u64 {
    unsafe { (a as *const u64).read_volatile() }
}
unsafe fn write8(a: u64, v: u8) {
    unsafe {
        (a as *mut u8).write_volatile(v);
    }
}
unsafe fn write16(a: u64, v: u16) {
    unsafe {
        (a as *mut u16).write_volatile(v);
    }
}
unsafe fn write32(a: u64, v: u32) {
    unsafe {
        (a as *mut u32).write_volatile(v);
    }
}
unsafe fn write64(a: u64, v: u64) {
    unsafe {
        (a as *mut u64).write_volatile(v);
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    #[test]
    fn fixed_profile_registration_accepts_only_supported_endpoint_headers() {
        let mut config = [0; 256];
        config[0x00..0x04].copy_from_slice(&[0xf4, 0x1a, 0x42, 0x10]);
        config[0x08..0x0c].copy_from_slice(&[1, 0, 0, 1]);
        let address = PciAddress {
            segment: 0,
            bus: 0,
            device: 5,
            function: 0,
        };
        assert_eq!(
            validate_registration(address, &config)
                .unwrap()
                .driver_match
                .registration
                .id,
            DRIVER_REGISTRATION.id
        );

        config[0x02..0x04].copy_from_slice(&[0x43, 0x10]);
        assert!(matches!(
            validate_registration(address, &config),
            Err(Error::Offline)
        ));

        config[0x02..0x04].copy_from_slice(&[0x42, 0x10]);
        config[0x0e] = 1;
        assert!(matches!(
            validate_registration(address, &config),
            Err(Error::Protocol)
        ));
    }
}
