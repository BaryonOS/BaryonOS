#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
//! Registry-selected modern virtio-rng transport (B01, ADR-043). One queue,
//! one outstanding request, one device-writable bounce page. No user address
//! enters here; the kernel keeps all three DMA pages pinned until reboot. A
//! protocol-conforming QEMU is trusted; this is not a malicious-device IOMMU.
//! The driver hands raw device bytes to the kernel broker and never mixes,
//! caches or reuses them.
use core::{
    ptr,
    sync::atomic::{fence, Ordering},
};
use driver_core::PciAddress;
pub use virtio_core::{Bar, Error, Regions};
use virtio_core::{ACCESS_PLATFORM, DEVICE_ID_ENTROPY, VERSION_1, WRITE};
mod pci;

pub const DRIVER_REGISTRATION: driver_core::DriverRegistration = driver_core::DriverRegistration {
    id: driver_core::DriverId::new(0x4e4f_0003),
    name: "nieuw-os-virtio-rng",
    version: driver_core::DriverVersion::new(0, 1, 0),
    class: driver_core::DeviceClass::EntropyDevice,
    pci: driver_core::PciMatch {
        vendor_id: Some(0x1af4),
        device_id: Some(DEVICE_ID_ENTROPY),
        class: None,
        subclass: None,
        programming_interface: None,
        revision: None,
        subsystem_vendor_id: None,
        subsystem_device_id: None,
    },
};

const RAM_END: u64 = 256 * 1024 * 1024;
pub const QUEUE_SIZE: u16 = 8;
/// Largest request: one IPC payload worth of bytes.
pub const MAX_REQUEST: u32 = 256;

#[derive(Clone, Copy)]
pub struct Discovery {
    pub address: PciAddress,
    pub config: [u8; 256],
    pub bars: [Bar; 6],
    pub bar_grants: [Option<pci_core::PciBarGrant>; 6],
    pub regions: Regions,
    driver_factory: DriverFactory,
}

/// Matched RNG initializer selected by the PCI registry.
///
/// # Safety
/// The caller must uphold the permanent MMIO mapping and exclusive pinned-DMA
/// ownership contract documented by [`Device::initialize`].
pub type DriverFactory = unsafe fn(Discovery, [u64; 6], RngPool) -> Result<Device, Error>;
const DRIVER_FACTORY: DriverFactory = Device::initialize_matched;

/// Three exclusive, identity-mapped kernel RAM pages: descriptor table plus
/// available ring (page 0), used ring (page 1) and the bounce buffer (page 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RngPool {
    pub rings: u64,
    pub used: u64,
    pub bounce: u64,
}
impl RngPool {
    pub fn validate(self, ram_end: u64) -> Result<(), Error> {
        let pages = [self.rings, self.used, self.bounce];
        for (i, page) in pages.iter().enumerate() {
            if *page == 0
                || page % 4096 != 0
                || page.checked_add(4096).ok_or(Error::Range)? > ram_end
                || pages[..i].contains(page)
            {
                return Err(Error::Range);
            }
        }
        Ok(())
    }
    /// Available ring lives after the 8 × 16-byte descriptors on the same page.
    pub const fn available(self) -> u64 {
        self.rings + 256
    }
}

/// The only features this driver accepts: VERSION_1 without ACCESS_PLATFORM.
pub fn negotiate_rng_features(offered: u64) -> Result<u64, Error> {
    if offered & VERSION_1 == 0 || offered & ACCESS_PLATFORM != 0 {
        return Err(Error::Unsupported);
    }
    Ok(VERSION_1)
}

/// Validate a used-ring element for the single outstanding request.
pub fn validate_used(
    requested: u32,
    old_idx: u16,
    new_idx: u16,
    used_id: u32,
    used_len: u32,
) -> Result<u32, Error> {
    let delta = new_idx.wrapping_sub(old_idx);
    if delta == 0 {
        return Err(Error::Pending);
    }
    if delta != 1 || used_id != 0 || used_len > requested {
        return Err(Error::Protocol);
    }
    if used_len == 0 {
        return Err(Error::Io);
    }
    Ok(used_len)
}

pub fn discover() -> Result<Discovery, Error> {
    pci::discover()
}

/// One PCI function selected by generic discovery and the RNG registry.
pub fn discover_at(address: PciAddress) -> Result<Discovery, Error> {
    pci::discover_at(address)
}

pub struct Device {
    common: u64,
    notify: u64,
    hardware: pci::Hardware,
    pub pool: RngPool,
    available: u16,
    used: u16,
    pending: Option<u32>,
    ready: bool,
}
impl Device {
    /// # Safety
    /// `mapped` must hold the permanent supervisor-only UC mappings of the BAR
    /// regions validated in `info`, and every page of `pool` must be an
    /// exclusive pinned normal-RAM page owned by this device until reboot.
    pub unsafe fn initialize(
        info: Discovery,
        mapped: [u64; 6],
        pool: RngPool,
    ) -> Result<Self, Error> {
        // SAFETY: Forward the documented permanent-MMIO and pinned-pool
        // requirements to the factory selected by this PCI discovery.
        unsafe { (info.driver_factory)(info, mapped, pool) }
    }

    unsafe fn initialize_matched(
        info: Discovery,
        mapped: [u64; 6],
        pool: RngPool,
    ) -> Result<Self, Error> {
        pool.validate(RAM_END)?;
        // SAFETY: This factory is private to the registry-produced Discovery;
        // the binding revalidates the descriptor and registration.
        let hardware = unsafe { pci::hardware(info.address, &info.config)? };
        let address = |region: virtio_core::Region| -> Result<u64, Error> {
            let base = mapped[region.bar as usize];
            if base == 0 {
                return Err(Error::Range);
            }
            base.checked_add(u64::from(region.offset))
                .ok_or(Error::Range)
        };
        let common = address(info.regions.common)?;
        if common % 8 != 0 {
            return Err(Error::Range);
        }
        let mut result = Self {
            common,
            notify: 0,
            hardware,
            pool,
            available: 0,
            used: 0,
            pending: None,
            ready: false,
        };
        // SAFETY: All addresses come from validated supervisor UC BAR mappings
        // and the caller's pinned pool pages.
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
            let features = negotiate_rng_features(u64::from(low) | (u64::from(high) << 32))?;
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
            for page in [pool.rings, pool.used, pool.bounce] {
                ptr::write_bytes(page as *mut u8, 0, 4096);
            }
            // avail.flags = 1 (no interrupt); the driver polls.
            write16(pool.available(), 1);
            write64(common + 32, pool.rings);
            write64(common + 40, pool.available());
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
            fence(Ordering::SeqCst);
            result.hardware.bus_master(true)?;
            write8(common + 20, 15);
            if read8(common + 20) != 15 {
                write8(common + 20, 0);
                result.hardware.bus_master(false)?;
                return Err(Error::Protocol);
            }
        }
        result.ready = true;
        Ok(result)
    }
    /// Publish one device-writable request of `length` bytes (1..=256).
    pub fn request(&mut self, length: u32) -> Result<(), Error> {
        if !self.ready {
            return Err(Error::Offline);
        }
        if self.pending.is_some() {
            return Err(Error::Busy);
        }
        if length == 0 || length > MAX_REQUEST {
            return Err(Error::Range);
        }
        let descriptor = virtio_core::Descriptor {
            address: self.pool.bounce,
            length,
            flags: WRITE,
            next: 0,
        };
        // SAFETY: Exclusive pinned pages; no outstanding request; the chain is
        // published once and never altered until the used element is consumed.
        unsafe {
            ptr::write_bytes(self.pool.bounce as *mut u8, 0, 4096);
            for (j, byte) in descriptor.encode().into_iter().enumerate() {
                write8(self.pool.rings + j as u64, byte);
            }
            write16(
                self.pool.available() + 4 + u64::from(self.available % QUEUE_SIZE) * 2,
                0,
            );
            fence(Ordering::SeqCst);
            self.available = self.available.wrapping_add(1);
            write16(self.pool.available() + 2, self.available);
            fence(Ordering::SeqCst);
            self.pending = Some(length);
            write16(self.notify, 0);
        }
        Ok(())
    }
    /// Completed bytes, if the device has answered. `Err(Io)` for an empty
    /// answer: the device had nothing, which is never silently padded.
    pub fn poll(&mut self, out: &mut [u8; 256]) -> Result<Option<u32>, Error> {
        let Some(requested) = self.pending else {
            return Ok(None);
        };
        // SAFETY: The device writes used/bounce concurrently; read volatile
        // integers and bytes, then acquire ordering. No shared Rust references.
        unsafe {
            let next = read16(self.pool.used + 2);
            if next == self.used {
                return Ok(None);
            }
            fence(Ordering::SeqCst);
            let ring = self.pool.used + 4 + u64::from(self.used % QUEUE_SIZE) * 8;
            let id = read32(ring);
            let length = read32(ring + 4);
            let result = validate_used(requested, self.used, next, id, length);
            self.used = next;
            self.pending = None;
            let length = result?;
            for (i, byte) in out.iter_mut().enumerate().take(length as usize) {
                *byte = read8(self.pool.bounce + i as u64);
            }
            ptr::write_bytes(self.pool.bounce as *mut u8, 0, 4096);
            Ok(Some(length))
        }
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn begin_reset(&mut self) -> Result<(), Error> {
        self.ready = false;
        self.pending = None;
        // SAFETY: The MMIO mapping and config adapter remain bound to the
        // matched device; stop the queue before revoking bus mastering.
        unsafe {
            write8(self.common + 20, 0);
        }
        self.hardware.bus_master(false)
    }
    pub fn reset_acknowledged(&mut self) -> bool {
        // SAFETY: Own UC status register; config access remains function-bound.
        let reset = unsafe { read8(self.common + 20) == 0 };
        reset && matches!(self.hardware.bus_master_enabled(), Ok(false))
    }
}

unsafe fn read8(a: u64) -> u8 {
    unsafe { (a as *const u8).read_volatile() }
}
unsafe fn read16(a: u64) -> u16 {
    unsafe { (a as *const u16).read_volatile() }
}
unsafe fn read32(a: u64) -> u32 {
    unsafe { (a as *const u32).read_volatile() }
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
mod tests {
    use super::*;
    #[test]
    fn pci_registration_matches_only_the_virtio_entropy_identity() {
        let descriptor = driver_core::PciDeviceDescriptor {
            address: driver_core::PciAddress {
                segment: 0,
                bus: 3,
                device: 12,
                function: 2,
            },
            vendor_id: 0x1af4,
            device_id: DEVICE_ID_ENTROPY,
            class: 0xff,
            subclass: 0,
            programming_interface: 0,
            revision: 0,
            subsystem_vendor_id: None,
            subsystem_device_id: None,
        };
        assert!(DRIVER_REGISTRATION.pci.matches(&descriptor));
        let unrelated = driver_core::PciDeviceDescriptor {
            device_id: 0x1042,
            ..descriptor
        };
        assert!(!DRIVER_REGISTRATION.pci.matches(&unrelated));
    }

    #[test]
    fn features_accept_only_version_1_without_platform_access() {
        assert_eq!(negotiate_rng_features(VERSION_1), Ok(VERSION_1));
        assert_eq!(negotiate_rng_features(VERSION_1 | (1 << 9)), Ok(VERSION_1));
        assert_eq!(negotiate_rng_features(0), Err(Error::Unsupported));
        assert_eq!(
            negotiate_rng_features(VERSION_1 | ACCESS_PLATFORM),
            Err(Error::Unsupported)
        );
    }
    #[test]
    fn used_elements_are_validated_and_empty_answers_are_io_errors() {
        assert_eq!(validate_used(256, 3, 3, 0, 256), Err(Error::Pending));
        assert_eq!(validate_used(256, 3, 4, 0, 256), Ok(256));
        assert_eq!(validate_used(256, 3, 4, 0, 17), Ok(17));
        assert_eq!(validate_used(256, 3, 4, 0, 0), Err(Error::Io));
        assert_eq!(validate_used(256, 3, 4, 0, 257), Err(Error::Protocol));
        assert_eq!(validate_used(256, 3, 4, 1, 256), Err(Error::Protocol));
        assert_eq!(validate_used(256, 3, 5, 0, 256), Err(Error::Protocol));
        assert_eq!(validate_used(16, u16::MAX, 0, 0, 16), Ok(16));
    }
    #[test]
    fn pool_pages_must_be_distinct_aligned_ram_pages() {
        let pool = RngPool {
            rings: 0x1000,
            used: 0x2000,
            bounce: 0x3000,
        };
        assert_eq!(pool.validate(RAM_END), Ok(()));
        assert_eq!(pool.available(), 0x1100);
        for bad in [
            RngPool { rings: 0, ..pool },
            RngPool {
                used: 0x1000,
                ..pool
            },
            RngPool {
                bounce: 0x1001,
                ..pool
            },
            RngPool {
                bounce: RAM_END,
                ..pool
            },
        ] {
            assert_eq!(bad.validate(RAM_END), Err(Error::Range));
        }
    }
}
