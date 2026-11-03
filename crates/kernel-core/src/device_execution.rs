//! Kernel-owned device lease and resource-lifecycle policy.
//!
//! This module contains no PCI, MMIO, interrupt-controller, IOMMU, or reset
//! hardware implementation. Its `DeviceControl` implementation must remain
//! kernel-trusted and report observed readbacks, not driver assertions.
use crate::{Alias, Error as FrameError, Frames, Owner, PAGE};

pub const MAX_DMA_GRANTS: usize = 16;
pub const MAX_DMA_PAGES: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciAddress {
    segment: u16,
    bus: u8,
    device: u8,
    function: u8,
}

impl PciAddress {
    pub fn new(segment: u16, bus: u8, device: u8, function: u8) -> Result<Self, Error> {
        if device > 31 || function > 7 {
            return Err(Error::InvalidAddress);
        }
        Ok(Self {
            segment,
            bus,
            device,
            function,
        })
    }

    pub const fn segment(self) -> u16 {
        self.segment
    }

    pub const fn bus(self) -> u8 {
        self.bus
    }

    pub const fn device(self) -> u8 {
        self.device
    }

    pub const fn function(self) -> u8 {
        self.function
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverId(u64);

impl DriverId {
    pub const fn new(value: u64) -> Result<Self, Error> {
        if value == 0 {
            Err(Error::InvalidDriver)
        } else {
            Ok(Self(value))
        }
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciBar {
    base: u64,
    size: u64,
    memory: bool,
}

impl PciBar {
    pub const NONE: Self = Self {
        base: 0,
        size: 0,
        memory: false,
    };

    pub const fn new(base: u64, size: u64, memory: bool) -> Result<Self, Error> {
        if base == 0 || size == 0 || !size.is_power_of_two() || !base.is_multiple_of(size) {
            return Err(Error::InvalidBar);
        }
        if base.checked_add(size).is_none() {
            return Err(Error::InvalidBar);
        }
        Ok(Self { base, size, memory })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciFunction {
    address: PciAddress,
    vendor_id: u16,
    device_id: u16,
    class: u8,
    subclass: u8,
    programming_interface: u8,
    generation: u64,
    bars: [PciBar; 6],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelPciSnapshot {
    pub address: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub programming_interface: u8,
    pub generation: u64,
    pub bars: [PciBar; 6],
}

impl PciFunction {
    pub const fn from_kernel_probe(snapshot: KernelPciSnapshot) -> Result<Self, Error> {
        if snapshot.vendor_id == 0
            || snapshot.vendor_id == u16::MAX
            || snapshot.device_id == u16::MAX
            || snapshot.generation == 0
        {
            return Err(Error::InvalidFunction);
        }
        Ok(Self {
            address: snapshot.address,
            vendor_id: snapshot.vendor_id,
            device_id: snapshot.device_id,
            class: snapshot.class,
            subclass: snapshot.subclass,
            programming_interface: snapshot.programming_interface,
            generation: snapshot.generation,
            bars: snapshot.bars,
        })
    }

    pub const fn address(self) -> PciAddress {
        self.address
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverMatch {
    id: DriverId,
    vendor_id: u16,
    device_id: u16,
    class: u8,
    subclass: u8,
    programming_interface: u8,
}

impl DriverMatch {
    pub const fn new(
        id: DriverId,
        vendor_id: u16,
        device_id: u16,
        class: u8,
        subclass: u8,
        programming_interface: u8,
    ) -> Self {
        Self {
            id,
            vendor_id,
            device_id,
            class,
            subclass,
            programming_interface,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FunctionKey {
    address: PciAddress,
    vendor_id: u16,
    device_id: u16,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Matched,
    Authorized,
    Initializing,
    Ready,
    Revoking,
    Quiesced,
    Quarantined,
    Released,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaDirection {
    DeviceReads,
    DeviceWrites,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptMode {
    Msi,
    Msix,
    Intx,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarRange {
    function: FunctionKey,
    driver: DriverId,
    generation: u64,
    bar: u8,
    base: u64,
    size: u64,
    read: bool,
    write: bool,
}

impl BarRange {
    fn check_access(&self, offset: u64, length: u64, write: bool) -> Result<(), Error> {
        if length == 0 || write && !self.write || !write && !self.read {
            return Err(Error::Permission);
        }
        let end = offset.checked_add(length).ok_or(Error::Range)?;
        if end > self.size {
            return Err(Error::Range);
        }
        self.base.checked_add(end).ok_or(Error::Range)?;
        Ok(())
    }

    pub const fn bar_index(self) -> u8 {
        self.bar
    }

    pub const fn size(self) -> u64 {
        self.size
    }

    pub const fn physical_base(self) -> u64 {
        self.base
    }

    pub const fn function_address(self) -> PciAddress {
        self.function.address
    }

    pub const fn vendor_device_ids(self) -> (u16, u16) {
        (self.function.vendor_id, self.function.device_id)
    }

    pub const fn driver_id(self) -> u64 {
        self.driver.value()
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DmaRecord {
    id: u64,
    pages: [u64; MAX_DMA_PAGES],
    page_count: u8,
    length: u32,
    direction: DmaDirection,
    live: bool,
}

const EMPTY_DMA: DmaRecord = DmaRecord {
    id: 0,
    pages: [0; MAX_DMA_PAGES],
    page_count: 0,
    length: 0,
    direction: DmaDirection::DeviceReads,
    live: false,
};

#[derive(Debug, PartialEq, Eq)]
pub struct DmaGrant {
    function: FunctionKey,
    driver: DriverId,
    generation: u64,
    slot: u8,
    id: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfigRange {
    function: FunctionKey,
    driver: DriverId,
    generation: u64,
    offset: u16,
    length: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterruptGrant {
    function: FunctionKey,
    driver: DriverId,
    generation: u64,
    vector: u8,
    mode: InterruptMode,
}

impl InterruptGrant {
    pub const fn vector(self) -> u8 {
        self.vector
    }

    pub const fn mode(self) -> InterruptMode {
        self.mode
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidAddress,
    InvalidDriver,
    InvalidBar,
    InvalidFunction,
    NoMatch,
    StaleGeneration,
    InvalidState,
    Permission,
    Range,
    Exhausted,
    Duplicate,
    Hardware,
    Quarantined,
    Frame(FrameError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlError {
    Unsupported,
    Failed,
}

/// Trusted kernel hardware adapter. Implementations must address only the
/// supplied function/driver/generation, bound every operation, and keep
/// reset/readback waits bounded. `bind_interrupt` returns only after binding a
/// masked route; partial binding is rolled back on error. A successful
/// `synchronize_unbind_interrupt` means the route is released and no handler is
/// still running.
pub trait DeviceControl {
    fn read_config(
        &mut self,
        function: PciAddress,
        driver: DriverId,
        generation: u64,
        offset: u16,
        width: u8,
    ) -> Result<u32, ControlError>;
    fn read_bar(&mut self, grant: &BarRange, offset: u64, width: u8) -> Result<u64, ControlError>;
    fn write_bar(
        &mut self,
        grant: &BarRange,
        offset: u64,
        width: u8,
        value: u64,
    ) -> Result<(), ControlError>;
    fn set_bus_master(
        &mut self,
        function: PciAddress,
        driver: DriverId,
        generation: u64,
        enabled: bool,
    ) -> Result<(), ControlError>;
    fn bus_master_enabled(
        &mut self,
        function: PciAddress,
        driver: DriverId,
        generation: u64,
    ) -> Result<bool, ControlError>;
    fn reset_and_wait(
        &mut self,
        function: PciAddress,
        driver: DriverId,
        generation: u64,
    ) -> Result<bool, ControlError>;
    fn bind_interrupt(
        &mut self,
        function: PciAddress,
        driver: DriverId,
        generation: u64,
    ) -> Result<(u8, InterruptMode), ControlError>;
    fn mask_interrupt(
        &mut self,
        function: PciAddress,
        driver: DriverId,
        generation: u64,
        vector: u8,
        mode: InterruptMode,
    ) -> Result<(), ControlError>;
    fn unmask_interrupt(
        &mut self,
        function: PciAddress,
        driver: DriverId,
        generation: u64,
        vector: u8,
        mode: InterruptMode,
    ) -> Result<(), ControlError>;
    fn acknowledge_interrupt(
        &mut self,
        function: PciAddress,
        driver: DriverId,
        generation: u64,
        vector: u8,
        mode: InterruptMode,
    ) -> Result<(), ControlError>;
    fn synchronize_unbind_interrupt(
        &mut self,
        function: PciAddress,
        driver: DriverId,
        generation: u64,
        vector: u8,
        mode: InterruptMode,
    ) -> Result<bool, ControlError>;
}

pub struct DeviceLease {
    function: FunctionKey,
    driver: DriverId,
    phase: Phase,
    bars: [PciBar; 6],
    dma: [DmaRecord; MAX_DMA_GRANTS],
    next_dma_id: u64,
    bus_master: bool,
    interrupt: Option<InterruptGrant>,
    interrupt_masked: bool,
}

impl DeviceLease {
    pub fn matched(function: PciFunction, driver: DriverMatch) -> Result<Self, Error> {
        if function.vendor_id != driver.vendor_id
            || function.device_id != driver.device_id
            || function.class != driver.class
            || function.subclass != driver.subclass
            || function.programming_interface != driver.programming_interface
        {
            return Err(Error::NoMatch);
        }
        Ok(Self {
            function: FunctionKey {
                address: function.address,
                vendor_id: function.vendor_id,
                device_id: function.device_id,
                generation: function.generation,
            },
            driver: driver.id,
            phase: Phase::Matched,
            bars: function.bars,
            dma: [EMPTY_DMA; MAX_DMA_GRANTS],
            next_dma_id: 1,
            bus_master: false,
            interrupt: None,
            interrupt_masked: true,
        })
    }

    pub const fn phase(&self) -> Phase {
        self.phase
    }

    pub const fn generation(&self) -> u64 {
        self.function.generation
    }

    pub fn authorize(&mut self, driver: DriverId, generation: u64) -> Result<(), Error> {
        if self.phase != Phase::Matched {
            return Err(Error::InvalidState);
        }
        self.check_binding(driver, generation)?;
        self.phase = Phase::Authorized;
        Ok(())
    }

    pub fn begin_initialization(&mut self) -> Result<(), Error> {
        if self.phase != Phase::Authorized {
            return Err(Error::InvalidState);
        }
        self.phase = Phase::Initializing;
        Ok(())
    }

    pub fn bar_range(&self, bar: u8, read: bool, write: bool) -> Result<BarRange, Error> {
        if !matches!(self.phase, Phase::Initializing | Phase::Ready)
            || bar >= self.bars.len() as u8
            || !read && !write
        {
            return Err(Error::InvalidState);
        }
        let selected = self.bars[bar as usize];
        if !selected.memory {
            return Err(Error::Permission);
        }
        Ok(BarRange {
            function: self.function,
            driver: self.driver,
            generation: self.function.generation,
            bar,
            base: selected.base,
            size: selected.size,
            read,
            write,
        })
    }

    fn check_bar(
        &self,
        grant: &BarRange,
        offset: u64,
        width: u8,
        write: bool,
    ) -> Result<(), Error> {
        if !matches!(self.phase, Phase::Initializing | Phase::Ready) {
            return Err(Error::InvalidState);
        }
        if grant.function != self.function
            || grant.driver != self.driver
            || grant.generation != self.function.generation
        {
            return Err(Error::StaleGeneration);
        }
        if !matches!(width, 1 | 2 | 4 | 8) || !offset.is_multiple_of(u64::from(width)) {
            return Err(Error::Range);
        }
        grant.check_access(offset, u64::from(width), write)
    }

    pub fn read_bar<C: DeviceControl>(
        &self,
        control: &mut C,
        grant: &BarRange,
        offset: u64,
        width: u8,
    ) -> Result<u64, Error> {
        self.check_bar(grant, offset, width, false)?;
        control
            .read_bar(grant, offset, width)
            .map_err(|_| Error::Hardware)
    }

    pub fn write_bar<C: DeviceControl>(
        &self,
        control: &mut C,
        grant: &BarRange,
        offset: u64,
        width: u8,
        value: u64,
    ) -> Result<(), Error> {
        self.check_bar(grant, offset, width, true)?;
        control
            .write_bar(grant, offset, width, value)
            .map_err(|_| Error::Hardware)
    }

    pub fn config_range(
        &self,
        offset: u16,
        length: u16,
        write: bool,
    ) -> Result<ConfigRange, Error> {
        if !matches!(self.phase, Phase::Initializing | Phase::Ready)
            || length == 0
            || write
            || offset.checked_add(length).is_none_or(|end| end > 256)
        {
            return Err(Error::Permission);
        }
        Ok(ConfigRange {
            function: self.function,
            driver: self.driver,
            generation: self.function.generation,
            offset,
            length,
        })
    }

    pub fn config_offset(
        &self,
        grant: &ConfigRange,
        offset: u16,
        length: u16,
    ) -> Result<u16, Error> {
        if !matches!(self.phase, Phase::Initializing | Phase::Ready) {
            return Err(Error::InvalidState);
        }
        if grant.function != self.function
            || grant.driver != self.driver
            || grant.generation != self.function.generation
        {
            return Err(Error::StaleGeneration);
        }
        if length == 0
            || offset
                .checked_add(length)
                .is_none_or(|end| end > grant.length)
        {
            return Err(Error::Range);
        }
        grant.offset.checked_add(offset).ok_or(Error::Range)
    }

    pub fn read_config<C: DeviceControl>(
        &self,
        control: &mut C,
        grant: &ConfigRange,
        offset: u16,
        width: u8,
    ) -> Result<u32, Error> {
        if !matches!(width, 1 | 2 | 4) || !offset.is_multiple_of(u16::from(width)) {
            return Err(Error::Range);
        }
        let absolute = self.config_offset(grant, offset, u16::from(width))?;
        control
            .read_config(
                self.function.address,
                self.driver,
                self.function.generation,
                absolute,
                width,
            )
            .map_err(|_| Error::Hardware)
    }

    pub fn allocate_dma<const N: usize>(
        &mut self,
        frames: &mut Frames<N>,
        length: usize,
        direction: DmaDirection,
        mut zero: impl FnMut(u64),
    ) -> Result<DmaGrant, Error> {
        if self.phase != Phase::Initializing || length == 0 || length > u32::MAX as usize {
            return Err(Error::InvalidState);
        }
        let pages = length.checked_add(PAGE as usize - 1).ok_or(Error::Range)? / PAGE as usize;
        if pages > MAX_DMA_PAGES {
            return Err(Error::Exhausted);
        }
        let slot = self
            .dma
            .iter()
            .position(|record| !record.live)
            .ok_or(Error::Exhausted)?;
        let id = self.next_dma_id;
        self.next_dma_id = self.next_dma_id.checked_add(1).ok_or(Error::Exhausted)?;
        let mut addresses = [0; MAX_DMA_PAGES];
        frames
            .allocate_many(Owner::Kernel, &mut addresses[..pages], |address| {
                zero(address)
            })
            .map_err(Error::Frame)?;
        self.dma[slot] = DmaRecord {
            id,
            pages: addresses,
            page_count: pages as u8,
            length: length as u32,
            direction,
            live: true,
        };
        Ok(DmaGrant {
            function: self.function,
            driver: self.driver,
            generation: self.function.generation,
            slot: slot as u8,
            id,
        })
    }

    pub fn dma_pages<'a>(&'a self, grant: &DmaGrant) -> Result<&'a [u64], Error> {
        if !matches!(self.phase, Phase::Initializing | Phase::Ready) {
            return Err(Error::InvalidState);
        }
        let record = self.record(grant)?;
        Ok(&record.pages[..record.page_count as usize])
    }

    pub fn dma_length(&self, grant: &DmaGrant) -> Result<u32, Error> {
        if !matches!(self.phase, Phase::Initializing | Phase::Ready) {
            return Err(Error::InvalidState);
        }
        Ok(self.record(grant)?.length)
    }

    pub fn dma_direction(&self, grant: &DmaGrant) -> Result<DmaDirection, Error> {
        if !matches!(self.phase, Phase::Initializing | Phase::Ready) {
            return Err(Error::InvalidState);
        }
        Ok(self.record(grant)?.direction)
    }

    pub fn enable_bus_master<C: DeviceControl>(&mut self, control: &mut C) -> Result<(), Error> {
        if self.phase != Phase::Initializing
            || self.bus_master
            || !self.dma.iter().any(|record| record.live)
        {
            return Err(Error::InvalidState);
        }
        let address = self.function.address;
        let driver = self.driver;
        let generation = self.function.generation;
        if control
            .set_bus_master(address, driver, generation, true)
            .is_ok()
            && control
                .bus_master_enabled(address, driver, generation)
                .is_ok_and(|enabled| enabled)
        {
            self.bus_master = true;
            return Ok(());
        }
        let disabled = control
            .set_bus_master(address, driver, generation, false)
            .is_ok()
            && control
                .bus_master_enabled(address, driver, generation)
                .is_ok_and(|enabled| !enabled);
        if !disabled {
            self.phase = Phase::Quarantined;
        }
        Err(if disabled {
            Error::Hardware
        } else {
            Error::Quarantined
        })
    }

    pub fn bind_interrupt<C: DeviceControl>(&mut self, control: &mut C) -> Result<u8, Error> {
        if self.phase != Phase::Initializing || self.interrupt.is_some() {
            return Err(Error::InvalidState);
        }
        let (vector, mode) = control
            .bind_interrupt(self.function.address, self.driver, self.function.generation)
            .map_err(|_| Error::Hardware)?;
        if !(32..=254).contains(&vector) || vector == 128 {
            let masked = control
                .mask_interrupt(
                    self.function.address,
                    self.driver,
                    self.function.generation,
                    vector,
                    mode,
                )
                .is_ok();
            let drained = control
                .synchronize_unbind_interrupt(
                    self.function.address,
                    self.driver,
                    self.function.generation,
                    vector,
                    mode,
                )
                .unwrap_or(false);
            if !masked || !drained {
                self.phase = Phase::Quarantined;
                return Err(Error::Quarantined);
            }
            return Err(Error::Hardware);
        }
        self.interrupt = Some(InterruptGrant {
            function: self.function,
            driver: self.driver,
            generation: self.function.generation,
            vector,
            mode,
        });
        self.interrupt_masked = true;
        Ok(vector)
    }

    pub fn unmask_interrupt<C: DeviceControl>(&mut self, control: &mut C) -> Result<(), Error> {
        let grant = self.interrupt.ok_or(Error::InvalidState)?;
        self.check_interrupt(&grant)?;
        if !matches!(self.phase, Phase::Initializing | Phase::Ready) || !self.interrupt_masked {
            return Err(Error::InvalidState);
        }
        control
            .unmask_interrupt(
                self.function.address,
                self.driver,
                self.function.generation,
                grant.vector,
                grant.mode,
            )
            .map_err(|_| Error::Hardware)?;
        self.interrupt_masked = false;
        Ok(())
    }

    pub fn acknowledge_interrupt<C: DeviceControl>(
        &self,
        control: &mut C,
        grant: &InterruptGrant,
    ) -> Result<(), Error> {
        self.check_interrupt(grant)?;
        if self.phase != Phase::Ready || self.interrupt_masked {
            return Err(Error::InvalidState);
        }
        control
            .acknowledge_interrupt(
                self.function.address,
                self.driver,
                self.function.generation,
                grant.vector,
                grant.mode,
            )
            .map_err(|_| Error::Hardware)
    }

    pub fn interrupt_grant(&self) -> Result<InterruptGrant, Error> {
        self.interrupt.ok_or(Error::InvalidState)
    }

    pub fn ready(&mut self) -> Result<(), Error> {
        if self.phase != Phase::Initializing || !self.bus_master {
            return Err(Error::InvalidState);
        }
        self.phase = Phase::Ready;
        Ok(())
    }

    pub fn revoke(&mut self) -> Result<(), Error> {
        if !matches!(
            self.phase,
            Phase::Authorized | Phase::Initializing | Phase::Ready
        ) {
            return Err(Error::InvalidState);
        }
        self.phase = Phase::Revoking;
        Ok(())
    }

    pub fn quiesce<C: DeviceControl>(&mut self, control: &mut C) -> Result<(), Error> {
        if !matches!(self.phase, Phase::Revoking | Phase::Quarantined) {
            return Err(Error::InvalidState);
        }
        self.phase = Phase::Revoking;
        let mut safe = true;
        if let Some(interrupt) = self.interrupt {
            let masked = control
                .mask_interrupt(
                    self.function.address,
                    self.driver,
                    self.function.generation,
                    interrupt.vector,
                    interrupt.mode,
                )
                .is_ok();
            let drained = control
                .synchronize_unbind_interrupt(
                    self.function.address,
                    self.driver,
                    self.function.generation,
                    interrupt.vector,
                    interrupt.mode,
                )
                .unwrap_or(false);
            safe &= masked && drained;
            if masked && drained {
                self.interrupt = None;
                self.interrupt_masked = true;
            }
        }
        let reset = control
            .reset_and_wait(self.function.address, self.driver, self.function.generation)
            .unwrap_or(false);
        let disabled = control
            .set_bus_master(
                self.function.address,
                self.driver,
                self.function.generation,
                false,
            )
            .is_ok()
            && control
                .bus_master_enabled(self.function.address, self.driver, self.function.generation)
                .is_ok_and(|enabled| !enabled);
        safe &= reset && disabled;
        if safe {
            self.bus_master = false;
            self.phase = Phase::Quiesced;
            Ok(())
        } else {
            self.phase = Phase::Quarantined;
            Err(Error::Quarantined)
        }
    }

    pub fn release_dma<const N: usize>(
        &mut self,
        grant: &DmaGrant,
        frames: &mut Frames<N>,
        mut zero: impl FnMut(u64),
    ) -> Result<(), Error> {
        if self.phase != Phase::Quiesced {
            return Err(Error::InvalidState);
        }
        let record = self.record(grant)?;
        for address in &record.pages[..record.page_count as usize] {
            if frames.owner(*address).map_err(Error::Frame)? != Owner::Kernel
                || frames.alias(*address).map_err(Error::Frame)?
                    != (Alias {
                        writable: 0,
                        executable: 0,
                        total: 0,
                    })
            {
                return Err(Error::Frame(FrameError::Owner));
            }
        }
        for address in &record.pages[..record.page_count as usize] {
            frames
                .free(*address, Owner::Kernel, &mut zero)
                .map_err(Error::Frame)?;
        }
        self.dma[grant.slot as usize] = EMPTY_DMA;
        if self.dma.iter().all(|entry| !entry.live) {
            self.phase = Phase::Released;
        }
        Ok(())
    }

    fn check_binding(&self, driver: DriverId, generation: u64) -> Result<(), Error> {
        if generation != self.function.generation {
            return Err(Error::StaleGeneration);
        }
        if driver != self.driver {
            return Err(Error::Permission);
        }
        Ok(())
    }

    fn check_interrupt(&self, grant: &InterruptGrant) -> Result<(), Error> {
        if grant.generation != self.function.generation {
            return Err(Error::StaleGeneration);
        }
        if grant.function != self.function || grant.driver != self.driver {
            return Err(Error::Permission);
        }
        if self.interrupt != Some(*grant) {
            return Err(Error::InvalidState);
        }
        Ok(())
    }

    fn record<'a>(&'a self, grant: &DmaGrant) -> Result<&'a DmaRecord, Error> {
        if grant.generation != self.function.generation {
            return Err(Error::StaleGeneration);
        }
        if grant.function != self.function || grant.driver != self.driver {
            return Err(Error::Permission);
        }
        let record = self
            .dma
            .get(grant.slot as usize)
            .filter(|record| record.live && record.id == grant.id)
            .ok_or(Error::InvalidState)?;
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Range;

    const DRIVER: DriverId = DriverId(0x4e56_4d45);
    const OTHER_DRIVER: DriverId = DriverId(0x5848_4349);
    const GENERATION: u64 = 9;
    const ADDRESS: PciAddress = PciAddress {
        segment: 0,
        bus: 0,
        device: 4,
        function: 0,
    };

    fn function() -> PciFunction {
        function_with_generation(GENERATION)
    }

    fn function_with_generation(generation: u64) -> PciFunction {
        let mut bars = [PciBar::NONE; 6];
        bars[0] = PciBar::new(0x8000_0000, 0x4000, true).unwrap();
        PciFunction::from_kernel_probe(KernelPciSnapshot {
            address: ADDRESS,
            vendor_id: 0x1234,
            device_id: 0x5678,
            class: 1,
            subclass: 8,
            programming_interface: 2,
            generation,
            bars,
        })
        .unwrap()
    }

    fn matched() -> DeviceLease {
        DeviceLease::matched(
            function(),
            DriverMatch::new(DRIVER, 0x1234, 0x5678, 1, 8, 2),
        )
        .unwrap()
    }

    struct Control {
        bus_master: bool,
        fail_enable_readback: bool,
        fail_disable_readback: bool,
        reset_ack: bool,
        irq_vector: u8,
        irq_mode: InterruptMode,
        fail_irq_sync: bool,
        fail_irq_mask: bool,
        calls: [u8; 11],
    }

    impl Default for Control {
        fn default() -> Self {
            Self {
                bus_master: false,
                fail_enable_readback: false,
                fail_disable_readback: false,
                reset_ack: false,
                irq_vector: 0,
                irq_mode: InterruptMode::Msi,
                fail_irq_sync: false,
                fail_irq_mask: false,
                calls: [0; 11],
            }
        }
    }

    impl DeviceControl for Control {
        fn read_config(
            &mut self,
            function: PciAddress,
            driver: DriverId,
            generation: u64,
            offset: u16,
            width: u8,
        ) -> Result<u32, ControlError> {
            assert_eq!(function, ADDRESS);
            assert_eq!(driver, DRIVER);
            assert_eq!(generation, GENERATION);
            assert!(matches!(width, 1 | 2 | 4));
            self.calls[10] += 1;
            Ok(u32::from(offset))
        }

        fn read_bar(
            &mut self,
            grant: &BarRange,
            offset: u64,
            width: u8,
        ) -> Result<u64, ControlError> {
            assert_eq!(grant.function.address, ADDRESS);
            assert_eq!(grant.driver, DRIVER);
            assert_eq!(grant.generation, GENERATION);
            assert_eq!(grant.bar_index(), 0);
            assert_eq!(grant.physical_base(), 0x8000_0000);
            assert!(matches!(width, 1 | 2 | 4 | 8));
            self.calls[8] += 1;
            Ok(offset + u64::from(width))
        }

        fn write_bar(
            &mut self,
            grant: &BarRange,
            offset: u64,
            width: u8,
            value: u64,
        ) -> Result<(), ControlError> {
            assert_eq!(grant.function.address, ADDRESS);
            assert_eq!(grant.driver, DRIVER);
            assert_eq!(grant.generation, GENERATION);
            assert_eq!(grant.bar_index(), 0);
            assert!(matches!(width, 1 | 2 | 4 | 8));
            assert_eq!(offset + u64::from(width), 0x14);
            assert_eq!(value, 0x1234);
            self.calls[9] += 1;
            Ok(())
        }

        fn set_bus_master(
            &mut self,
            function: PciAddress,
            driver: DriverId,
            generation: u64,
            enabled: bool,
        ) -> Result<(), ControlError> {
            assert_eq!(function, ADDRESS);
            assert_eq!(driver, DRIVER);
            assert_eq!(generation, GENERATION);
            self.calls[0] += 1;
            self.bus_master = enabled;
            Ok(())
        }

        fn bus_master_enabled(
            &mut self,
            function: PciAddress,
            driver: DriverId,
            generation: u64,
        ) -> Result<bool, ControlError> {
            assert_eq!(function, ADDRESS);
            assert_eq!(driver, DRIVER);
            assert_eq!(generation, GENERATION);
            self.calls[1] += 1;
            if self.bus_master && self.fail_enable_readback
                || !self.bus_master && self.fail_disable_readback
            {
                return Err(ControlError::Failed);
            }
            Ok(self.bus_master)
        }

        fn reset_and_wait(
            &mut self,
            function: PciAddress,
            driver: DriverId,
            generation: u64,
        ) -> Result<bool, ControlError> {
            assert_eq!(function, ADDRESS);
            assert_eq!(driver, DRIVER);
            assert_eq!(generation, GENERATION);
            self.calls[2] += 1;
            Ok(self.reset_ack)
        }

        fn bind_interrupt(
            &mut self,
            function: PciAddress,
            driver: DriverId,
            generation: u64,
        ) -> Result<(u8, InterruptMode), ControlError> {
            assert_eq!(function, ADDRESS);
            assert_eq!(driver, DRIVER);
            assert_eq!(generation, GENERATION);
            self.calls[3] += 1;
            Ok((self.irq_vector, self.irq_mode))
        }

        fn mask_interrupt(
            &mut self,
            function: PciAddress,
            driver: DriverId,
            generation: u64,
            vector: u8,
            mode: InterruptMode,
        ) -> Result<(), ControlError> {
            assert_eq!(function, ADDRESS);
            assert_eq!(driver, DRIVER);
            assert_eq!(generation, GENERATION);
            assert_eq!(vector, self.irq_vector);
            assert_eq!(mode, self.irq_mode);
            self.calls[4] += 1;
            if self.fail_irq_mask {
                Err(ControlError::Failed)
            } else {
                Ok(())
            }
        }

        fn unmask_interrupt(
            &mut self,
            function: PciAddress,
            driver: DriverId,
            generation: u64,
            vector: u8,
            mode: InterruptMode,
        ) -> Result<(), ControlError> {
            assert_eq!(function, ADDRESS);
            assert_eq!(driver, DRIVER);
            assert_eq!(generation, GENERATION);
            assert_eq!(vector, self.irq_vector);
            assert_eq!(mode, self.irq_mode);
            self.calls[5] += 1;
            Ok(())
        }

        fn acknowledge_interrupt(
            &mut self,
            function: PciAddress,
            driver: DriverId,
            generation: u64,
            vector: u8,
            mode: InterruptMode,
        ) -> Result<(), ControlError> {
            assert_eq!(function, ADDRESS);
            assert_eq!(driver, DRIVER);
            assert_eq!(generation, GENERATION);
            assert_eq!(vector, self.irq_vector);
            assert_eq!(mode, self.irq_mode);
            self.calls[6] += 1;
            Ok(())
        }

        fn synchronize_unbind_interrupt(
            &mut self,
            function: PciAddress,
            driver: DriverId,
            generation: u64,
            vector: u8,
            mode: InterruptMode,
        ) -> Result<bool, ControlError> {
            assert_eq!(function, ADDRESS);
            assert_eq!(driver, DRIVER);
            assert_eq!(generation, GENERATION);
            assert_eq!(vector, self.irq_vector);
            assert_eq!(mode, self.irq_mode);
            self.calls[7] += 1;
            Ok(!self.fail_irq_sync)
        }
    }

    fn frames<const N: usize>() -> Frames<N> {
        let mut frames = Frames::new();
        frames.admit(Range::pages(0, N as u64).unwrap(), 7, 8, &[]);
        frames
    }

    fn free_count<const N: usize>(frames: &Frames<N>) -> usize {
        (1..N)
            .filter(|index| frames.owner(*index as u64 * PAGE) == Ok(Owner::Free))
            .count()
    }

    fn initialize<const N: usize>(
        lease: &mut DeviceLease,
        control: &mut Control,
        frames: &mut Frames<N>,
    ) -> DmaGrant {
        lease.authorize(DRIVER, GENERATION).unwrap();
        lease.begin_initialization().unwrap();
        let grant = lease
            .allocate_dma(frames, 4097, DmaDirection::DeviceWrites, |_| {})
            .unwrap();
        assert_eq!(lease.dma_pages(&grant).unwrap().len(), 2);
        if control.irq_vector != 0 {
            lease.bind_interrupt(control).unwrap();
            lease.unmask_interrupt(control).unwrap();
        }
        lease.enable_bus_master(control).unwrap();
        lease.ready().unwrap();
        grant
    }

    #[test]
    fn exact_match_generation_and_function_bounds() {
        assert_eq!(PciAddress::new(0, 0, 32, 0), Err(Error::InvalidAddress));
        assert_eq!(PciAddress::new(0, 0, 1, 8), Err(Error::InvalidAddress));
        assert_eq!(
            DeviceLease::matched(
                function(),
                DriverMatch::new(OTHER_DRIVER, 0x1235, 0x5678, 1, 8, 2)
            )
            .err(),
            Some(Error::NoMatch)
        );
        let mut lease = matched();
        assert_eq!(
            lease.authorize(OTHER_DRIVER, GENERATION),
            Err(Error::Permission)
        );
        assert_eq!(
            lease.authorize(DRIVER, GENERATION + 1),
            Err(Error::StaleGeneration)
        );
        lease.authorize(DRIVER, GENERATION).unwrap();
    }

    #[test]
    fn stale_dma_and_mmio_grants_do_not_cross_device_generations() {
        let mut old = matched();
        old.authorize(DRIVER, GENERATION).unwrap();
        old.begin_initialization().unwrap();
        let mut frames = frames::<8>();
        let dma = old
            .allocate_dma(&mut frames, 64, DmaDirection::DeviceReads, |_| {})
            .unwrap();
        let bar = old.bar_range(0, true, false).unwrap();

        let next_generation = DeviceLease::matched(
            function_with_generation(GENERATION + 1),
            DriverMatch::new(DRIVER, 0x1234, 0x5678, 1, 8, 2),
        )
        .unwrap();
        let mut next_generation = next_generation;
        next_generation.authorize(DRIVER, GENERATION + 1).unwrap();
        next_generation.begin_initialization().unwrap();
        assert_eq!(next_generation.dma_pages(&dma), Err(Error::StaleGeneration));
        let mut control = Control::default();
        assert_eq!(
            next_generation.read_bar(&mut control, &bar, 0, 4),
            Err(Error::StaleGeneration)
        );
        assert_eq!(control.calls[8], 0);
    }

    #[test]
    fn bar_and_config_grants_are_bounded_and_read_only() {
        let mut lease = matched();
        lease.authorize(DRIVER, GENERATION).unwrap();
        lease.begin_initialization().unwrap();
        let bar = lease.bar_range(0, true, false).unwrap();
        assert_eq!(bar.function_address(), ADDRESS);
        assert_eq!(bar.vendor_device_ids(), (0x1234, 0x5678));
        assert_eq!(bar.driver_id(), DRIVER.value());
        assert_eq!(bar.generation(), GENERATION);
        assert_eq!(
            lease.read_bar(&mut Control::default(), &bar, 0x3ffc, 8),
            Err(Error::Range)
        );
        let mut control = Control::default();
        assert_eq!(lease.read_bar(&mut control, &bar, 0x10, 4), Ok(0x14));
        assert_eq!(control.calls[8], 1);
        assert_eq!(
            lease.write_bar(&mut control, &bar, 0x10, 4, 0x1234),
            Err(Error::Permission)
        );
        assert_eq!(control.calls[9], 0);
        assert_eq!(
            lease.read_bar(&mut control, &bar, u64::MAX - 1, 4),
            Err(Error::Range)
        );
        let writable = lease.bar_range(0, true, true).unwrap();
        assert_eq!(
            lease.write_bar(&mut control, &writable, 0x10, 4, 0x1234),
            Ok(())
        );
        assert_eq!(control.calls[9], 1);
        assert_eq!(lease.bar_range(1, true, false), Err(Error::Permission));
        let config = lease.config_range(0xfc, 4, false).unwrap();
        assert_eq!(lease.config_offset(&config, 0, 4), Ok(0xfc));
        assert_eq!(lease.config_offset(&config, 0, 8), Err(Error::Range));
        assert_eq!(lease.config_range(0xfc, 8, false), Err(Error::Permission));
        assert_eq!(lease.config_range(0x10, 4, true), Err(Error::Permission));
        let config = lease.config_range(0x10, 4, false).unwrap();
        assert_eq!(lease.read_config(&mut control, &config, 0, 4), Ok(0x10));
        assert_eq!(control.calls[10], 1);
    }

    #[test]
    fn dma_is_allocator_issued_directional_and_pinned_until_quiescence() {
        let mut lease = matched();
        lease.authorize(DRIVER, GENERATION).unwrap();
        lease.begin_initialization().unwrap();
        let mut frames = frames::<16>();
        let initial_free = free_count(&frames);
        let zeroed = core::cell::Cell::new(0);
        let grant = lease
            .allocate_dma(&mut frames, 4097, DmaDirection::DeviceWrites, |_| {
                zeroed.set(zeroed.get() + 1)
            })
            .unwrap();
        assert_eq!(zeroed.get(), 2);
        assert_eq!(free_count(&frames), initial_free - 2);
        assert_eq!(lease.dma_direction(&grant), Ok(DmaDirection::DeviceWrites));
        assert_eq!(lease.dma_length(&grant), Ok(4097));
        assert!(lease.dma_pages(&grant).unwrap().iter().all(|pa| *pa != 0));
        let mut control = Control::default();
        assert_eq!(
            lease.release_dma(&grant, &mut frames, |_| {}),
            Err(Error::InvalidState)
        );
        assert_eq!(free_count(&frames), initial_free - 2);
        assert_eq!(lease.phase(), Phase::Initializing);
        assert_eq!(
            lease.allocate_dma(
                &mut frames,
                PAGE as usize * (MAX_DMA_PAGES + 1),
                DmaDirection::DeviceReads,
                |_| {}
            ),
            Err(Error::Exhausted)
        );
        control.reset_ack = true;
    }

    #[test]
    fn bus_master_requires_match_authority_and_dma_and_rolls_back() {
        let mut lease = matched();
        let mut control = Control::default();
        assert_eq!(
            lease.enable_bus_master(&mut control),
            Err(Error::InvalidState)
        );
        lease.authorize(DRIVER, GENERATION).unwrap();
        lease.begin_initialization().unwrap();
        assert_eq!(
            lease.enable_bus_master(&mut control),
            Err(Error::InvalidState)
        );
        let mut frames = frames::<8>();
        let grant = lease
            .allocate_dma(&mut frames, 32, DmaDirection::DeviceReads, |_| {})
            .unwrap();
        let second_grant = lease
            .allocate_dma(&mut frames, 32, DmaDirection::DeviceWrites, |_| {})
            .unwrap();
        assert_eq!(
            lease.dma_direction(&second_grant),
            Ok(DmaDirection::DeviceWrites)
        );
        control.fail_enable_readback = true;
        assert_eq!(lease.enable_bus_master(&mut control), Err(Error::Hardware));
        assert!(!control.bus_master);
        assert_eq!(lease.phase(), Phase::Initializing);
        control.fail_disable_readback = true;
        assert_eq!(
            lease.enable_bus_master(&mut control),
            Err(Error::Quarantined)
        );
        assert_eq!(lease.phase(), Phase::Quarantined);
        assert_eq!(lease.dma_pages(&grant), Err(Error::InvalidState));
    }

    #[test]
    fn interrupt_binding_is_device_bound_and_teardown_must_drain() {
        let mut lease = matched();
        lease.authorize(DRIVER, GENERATION).unwrap();
        lease.begin_initialization().unwrap();
        let mut control = Control {
            reset_ack: true,
            irq_vector: 64,
            irq_mode: InterruptMode::Msi,
            ..Control::default()
        };
        let vector = lease.bind_interrupt(&mut control).unwrap();
        assert_eq!(vector, 64);
        let grant = lease.interrupt_grant().unwrap();
        assert_eq!(
            lease.acknowledge_interrupt(&mut control, &grant),
            Err(Error::InvalidState)
        );
        let mut frames = frames::<8>();
        lease
            .allocate_dma(&mut frames, 32, DmaDirection::DeviceReads, |_| {})
            .unwrap();
        lease.enable_bus_master(&mut control).unwrap();
        lease.unmask_interrupt(&mut control).unwrap();
        lease.ready().unwrap();
        lease.acknowledge_interrupt(&mut control, &grant).unwrap();
        lease.revoke().unwrap();
        control.fail_irq_sync = true;
        assert_eq!(lease.quiesce(&mut control), Err(Error::Quarantined));
        assert_eq!(lease.phase(), Phase::Quarantined);
        assert_eq!(control.calls[7], 1);
        assert_eq!(lease.interrupt, Some(grant));
    }

    #[test]
    fn only_observed_reset_busmaster_off_and_irq_drain_allow_reuse() {
        let mut lease = matched();
        let mut control = Control {
            reset_ack: true,
            irq_vector: 65,
            irq_mode: InterruptMode::Msix,
            ..Control::default()
        };
        let mut frames = frames::<16>();
        let before = free_count(&frames);
        let grant = initialize(&mut lease, &mut control, &mut frames);
        assert_eq!(lease.dma_pages(&grant).unwrap().len(), 2);
        lease.revoke().unwrap();
        lease.quiesce(&mut control).unwrap();
        assert_eq!(lease.phase(), Phase::Quiesced);
        assert_eq!(lease.dma_pages(&grant), Err(Error::InvalidState));
        assert!(lease.interrupt.is_none());
        lease.release_dma(&grant, &mut frames, |_| {}).unwrap();
        assert_eq!(free_count(&frames), before);
        assert_eq!(lease.phase(), Phase::Released);
    }

    #[test]
    fn failed_quiescence_keeps_dma_pages_quarantined() {
        let mut lease = matched();
        lease.authorize(DRIVER, GENERATION).unwrap();
        lease.begin_initialization().unwrap();
        let mut frames = frames::<8>();
        let before = free_count(&frames);
        let grant = lease
            .allocate_dma(&mut frames, 4096, DmaDirection::DeviceReads, |_| {})
            .unwrap();
        let mut control = Control {
            reset_ack: false,
            ..Control::default()
        };
        lease.enable_bus_master(&mut control).unwrap();
        lease.ready().unwrap();
        lease.revoke().unwrap();
        assert_eq!(lease.quiesce(&mut control), Err(Error::Quarantined));
        assert_eq!(
            lease.release_dma(&grant, &mut frames, |_| {}),
            Err(Error::InvalidState)
        );
        assert_eq!(free_count(&frames), before - 1);
        control.reset_ack = true;
        control.fail_disable_readback = true;
        assert_eq!(lease.quiesce(&mut control), Err(Error::Quarantined));
        assert_eq!(free_count(&frames), before - 1);
        control.fail_disable_readback = false;
        lease.quiesce(&mut control).unwrap();
        lease.release_dma(&grant, &mut frames, |_| {}).unwrap();
        assert_eq!(free_count(&frames), before);
    }

    #[test]
    fn completed_quiescence_releases_only_the_original_grant_once() {
        let mut lease = matched();
        lease.authorize(DRIVER, GENERATION).unwrap();
        lease.begin_initialization().unwrap();
        let mut frames = frames::<8>();
        let before = free_count(&frames);
        let grant = lease
            .allocate_dma(&mut frames, 4096, DmaDirection::DeviceReads, |_| {})
            .unwrap();
        let mut control = Control {
            reset_ack: true,
            ..Control::default()
        };
        lease.enable_bus_master(&mut control).unwrap();
        lease.ready().unwrap();
        lease.revoke().unwrap();
        lease.quiesce(&mut control).unwrap();
        lease.release_dma(&grant, &mut frames, |_| {}).unwrap();
        assert_eq!(free_count(&frames), before);
        assert_eq!(lease.phase(), Phase::Released);
        assert_eq!(
            lease.release_dma(&grant, &mut frames, |_| {}),
            Err(Error::InvalidState)
        );
    }
}
