//! Opt-in, single-BSP QEMU VT-d fixture. No userspace authority is issued.
//!
//! One validated DRHD, one direct bus-zero requester, legacy three-level
//! translation, 4-KiB pages and register invalidation only. IRQ sources remain
//! masked. Tables and failed initialization resources are permanently retained.
//! The caller must stop the controller and clear PCI bus mastering before
//! `deny_all`; this module never turns translation off or releases DMA frames.
//!
//! Register/table encodings: Intel VT-d Architecture Specification, revision
//! 4.0, sections 3.4.2, 6.5, 6.7, 9.1 and 11.4. Native evidence is separate
//! from the ACPI inventory and from the host device-execution model.

use super::{paging, FRAME_COUNT};
use boot_protocol::acpi::Discovery;
use core::{
    arch::{asm, x86_64::__cpuid},
    ptr,
    sync::atomic::{fence, AtomicBool, Ordering},
};
use kernel_core::{
    device_execution::{DeviceLease, DmaDirection, DmaGrant, PciAddress},
    Frames, Owner, PAGE,
};

const WINDOW: u64 = 0x3f20_0000;
pub(super) const IOVA_BASE: u64 = 0x10_0000;
const MAX_PAGES: usize = 16;
const LIMIT: usize = 1_000_000;
const TE: u32 = 1 << 31;
const SRTP: u32 = 1 << 30;
const INVALIDATE: u64 = 1 << 63;
static TAKEN: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Error {
    Inventory,
    Grant,
    Ownership,
    Address,
    Unsupported,
    Busy,
    Frame,
    Mapping,
    Timeout,
    Invalidation,
    Status,
    FaultOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Fault {
    pub requester: u16,
    pub address: u64,
    pub reason: u8,
    pub write: bool,
}

struct Registers {
    bytes: u64,
    iotlb: u64,
    fault_start: u64,
    fault_count: u16,
}

/// A hardware-established domain, bound to the live lease by its Rust borrow.
/// The private constructor requires actual table installation, invalidation
/// completion and translation-enable readback. Dropping it releases nothing.
pub(super) struct Domain<'a> {
    _lease: &'a DeviceLease,
    requester: PciAddress,
    registers: Registers,
    context: u64,
    cache_line: usize,
    starts: [u8; MAX_PAGES],
    counts: [u8; MAX_PAGES],
    grants: usize,
    active: bool,
}

pub(super) fn establish<'a>(
    discovery: &Discovery,
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
    lease: &'a DeviceLease,
    grants: &[&DmaGrant],
) -> Result<Domain<'a>, Error> {
    let requester = lease.function_address();
    if requester.segment() != 0 || requester.bus() != 0 {
        return Err(Error::Inventory);
    }
    let unit = discovery
        .native_dma_unit(requester.device(), requester.function())
        .ok_or(Error::Inventory)?;
    if grants.is_empty() || grants.len() > MAX_PAGES {
        return Err(Error::Grant);
    }
    let mut pages = [0; MAX_PAGES];
    let mut permissions = [0; MAX_PAGES];
    let mut starts = [0; MAX_PAGES];
    let mut counts = [0; MAX_PAGES];
    let mut count = 0;
    for (index, grant) in grants.iter().enumerate() {
        let addresses = lease.dma_pages(grant).map_err(|_| Error::Grant)?;
        let permission = match lease.dma_direction(grant).map_err(|_| Error::Grant)? {
            DmaDirection::DeviceReads => 1,
            DmaDirection::DeviceWrites => 2,
            DmaDirection::DeviceReadsWrites => 3,
        };
        if addresses.is_empty() || addresses.len() > MAX_PAGES - count {
            return Err(Error::Grant);
        }
        starts[index] = count as u8;
        counts[index] = addresses.len() as u8;
        for &address in addresses {
            if address == 0
                || address >= PAGE * FRAME_COUNT as u64
                || !kernel_core::physical_page_fits(address, discovery.dmar_host_address_width)
                || pages[..count].contains(&address)
            {
                return Err(Error::Address);
            }
            if !matches!(frames.owner(address), Ok(Owner::Dma(_))) {
                return Err(Error::Ownership);
            }
            pages[count] = address;
            permissions[count] = permission;
            count += 1;
        }
    }
    // CLFLUSH also supports non-coherent VT-d page walks (QEMU reports C=0).
    // SAFETY: CPUID leaf 1 is present on the supported x86-64 bootstrap CPU;
    // it reads CPU feature registers and does not access memory.
    let cpu = unsafe { __cpuid(1) };
    let cache_line = ((cpu.ebx >> 8) & 0xff) as usize * 8;
    if cpu.edx & (1 << 19) == 0
        || !(16..=256).contains(&cache_line)
        || !cache_line.is_power_of_two()
    {
        return Err(Error::Unsupported);
    }
    if TAKEN
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err(Error::Busy);
    }
    // Failure from this point deliberately consumes the fixture. Retrying or
    // recycling resources after an uncertain hardware operation is forbidden.
    for offset in (0..unit.register_bytes).step_by(PAGE as usize) {
        paging
            .map_native_device(frames, WINDOW + offset, unit.register_base + offset)
            .map_err(|_| Error::Mapping)?;
    }
    let mut registers = Registers {
        bytes: unit.register_bytes,
        iotlb: 0,
        fault_start: 0,
        fault_count: 0,
    };
    let version = registers.read32(0);
    let cap = registers.read64(8);
    let ecap = registers.read64(16);
    let major = (version >> 4) & 0xf;
    if !(1..=5).contains(&major)
        || cap & (1 << 9) == 0 // 39-bit, three-level second-stage tables.
        || ((cap >> 16) & 0x3f) + 1 < 39
        || cap & ((1 << 54) | (1 << 55)) != (1 << 54) | (1 << 55)
    {
        return Err(Error::Unsupported);
    }
    registers.iotlb = ((ecap >> 8) & 0x3ff) * 16 + 8;
    registers.fault_start = ((cap >> 24) & 0x3ff) * 16;
    registers.fault_count = (((cap >> 40) & 0xff) + 1) as u16;
    let fault_end = registers.fault_start + u64::from(registers.fault_count) * 16;
    if registers.iotlb < 0x88
        || registers.iotlb + 8 > registers.bytes
        || registers.fault_start < 0x80
        || fault_end > registers.bytes
        || registers.fault_count > 16
        || (registers.fault_start < registers.iotlb + 8 && registers.iotlb - 8 < fault_end)
    {
        return Err(Error::Address);
    }
    // No takeover of pre-existing translation, interrupt remapping, queued
    // invalidation or advanced fault logging; all are outside this fixture.
    if registers.read32(0x1c) != 0
        || registers.read64(0x28) & INVALIDATE != 0
        || registers.read64(registers.iotlb) & INVALIDATE != 0
        || registers.read32(0x34) & 0x7f != 0
    {
        return Err(Error::Busy);
    }
    registers.write32(0x38, 1 << 31); // Fault events are polled; MSI stays masked.
    if registers.read32(0x38) & (1 << 31) == 0 {
        return Err(Error::Status);
    }
    for index in 0..registers.fault_count {
        if registers.read64(registers.fault_start + u64::from(index) * 16 + 8) & INVALIDATE != 0 {
            return Err(Error::Busy);
        }
    }
    let mut tables = [0; 5]; // Root, context, level 3, level 2, level 1.
    frames
        .allocate_many(Owner::Kernel, &mut tables, paging::zero_dynamic)
        .map_err(|_| Error::Frame)?;
    table_write(tables[0], 0, tables[1] | 1);
    let devfn = usize::from(requester.device()) * 8 + usize::from(requester.function());
    // Domain 1, AGAW=39; TT=00 selects translation, never pass-through.
    table_write(tables[1], devfn * 2 + 1, (1 << 8) | 1);
    table_write(tables[1], devfn * 2, tables[2] | 1);
    table_write(tables[2], 0, tables[3] | 3);
    table_write(tables[3], 0, tables[4] | 3);
    for index in 0..count {
        table_write(
            tables[4],
            IOVA_BASE as usize / PAGE as usize + index,
            pages[index] | permissions[index],
        );
    }
    for table in tables {
        flush_table(table, cache_line);
    }
    registers.write64(0x20, tables[0]); // Legacy RTADDR, all other requesters absent.
    if registers.read64(0x20) != tables[0] {
        return Err(Error::Status);
    }
    registers.write32(0x18, SRTP);
    registers.wait32(0x1c, SRTP, SRTP)?;
    registers.invalidate()?;
    registers.write32(0x18, TE);
    registers.wait32(0x1c, TE, TE)?;
    if registers.read64(0x20) != tables[0] || registers.read32(0x34) & 0x7f != 0 {
        return Err(Error::Status);
    }
    Ok(Domain {
        _lease: lease,
        requester,
        registers,
        context: tables[1],
        cache_line,
        starts,
        counts,
        grants: grants.len(),
        active: true,
    })
}

impl Domain<'_> {
    pub(super) fn requester(&self) -> PciAddress {
        self.requester
    }

    pub(super) fn translation_enabled(&self) -> bool {
        self.active && self.registers.read32(0x1c) & TE != 0
    }

    pub(super) fn iova(&self, grant: usize, page: usize) -> Result<u64, Error> {
        if !self.active || grant >= self.grants || page >= usize::from(self.counts[grant]) {
            return Err(Error::Grant);
        }
        Ok(IOVA_BASE + (u64::from(self.starts[grant]) + page as u64) * PAGE)
    }

    pub(super) fn fault(&self) -> Result<Option<Fault>, Error> {
        let status = self.registers.read32(0x34);
        if status & 1 != 0 {
            return Err(Error::FaultOverflow);
        }
        for index in 0..self.registers.fault_count {
            let offset = self.registers.fault_start + u64::from(index) * 16;
            let high = self.registers.read64(offset + 8);
            if high & INVALIDATE == 0 {
                continue;
            }
            let low = self.registers.read64(offset);
            if high != self.registers.read64(offset + 8) || high & (1 << 28) != 0 {
                return Err(Error::Status);
            }
            return Ok(Some(Fault {
                requester: high as u16,
                address: low & !0xfff,
                reason: (high >> 32) as u8,
                write: high & (1 << 62) == 0,
            }));
        }
        if status & (1 << 1) != 0 {
            Err(Error::Status)
        } else {
            Ok(None)
        }
    }

    /// Called only after controller halt and PCI bus-master clear/readback.
    /// Failure permanently retains all tables/pins; translation stays enabled.
    pub(super) fn deny_all(&mut self) -> Result<(), Error> {
        self.active = false;
        let devfn =
            usize::from(self.requester.device()) * 8 + usize::from(self.requester.function());
        table_write(self.context, devfn * 2, 0);
        flush_table(self.context, self.cache_line);
        self.registers.invalidate()?;
        if self.registers.read32(0x1c) & TE == 0 {
            return Err(Error::Status);
        }
        Ok(())
    }
}

impl Registers {
    fn read32(&self, offset: u64) -> u32 {
        assert!(offset.is_multiple_of(4) && offset + 4 <= self.bytes);
        // SAFETY: The exclusive BSP mapped this validated DRHD aperture UC/NX,
        // supervisor-only. The aligned scalar stays inside its retained mapping;
        // volatile access creates no references and IRQs remain disabled.
        unsafe { ptr::read_volatile((WINDOW + offset) as *const u32) }
    }

    fn read64(&self, offset: u64) -> u64 {
        assert!(offset.is_multiple_of(8) && offset + 8 <= self.bytes);
        // SAFETY: Same exclusive, aligned and permanently mapped MMIO aperture
        // as read32; this is a native 64-bit register access with no references.
        unsafe { ptr::read_volatile((WINDOW + offset) as *const u64) }
    }

    fn write32(&self, offset: u64, value: u32) {
        assert!(offset.is_multiple_of(4) && offset + 4 <= self.bytes);
        // SAFETY: The validated UC/NX MMIO range remains owned by this BSP.
        // Only named protocol registers receive aligned volatile scalar writes;
        // no reference or pointer outlives this synchronous access.
        unsafe { ptr::write_volatile((WINDOW + offset) as *mut u32, value) }
        fence(Ordering::SeqCst);
    }

    fn write64(&self, offset: u64, value: u64) {
        assert!(offset.is_multiple_of(8) && offset + 8 <= self.bytes);
        // SAFETY: Same exclusive mapped MMIO contract as write32, with a native
        // aligned 64-bit transaction; the high byte triggers register commands.
        unsafe { ptr::write_volatile((WINDOW + offset) as *mut u64, value) }
        fence(Ordering::SeqCst);
    }

    fn wait32(&self, offset: u64, mask: u32, expected: u32) -> Result<(), Error> {
        for _ in 0..LIMIT {
            if self.read32(offset) & mask == expected {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(Error::Timeout)
    }

    fn wait_invalidation(&self, offset: u64, actual: u64) -> Result<(), Error> {
        for _ in 0..LIMIT {
            let value = self.read64(offset);
            if value & INVALIDATE == 0 {
                return if value & (actual * 3) == actual {
                    Ok(())
                } else {
                    Err(Error::Invalidation)
                };
            }
            core::hint::spin_loop();
        }
        Err(Error::Timeout)
    }

    fn invalidate(&self) -> Result<(), Error> {
        if self.read64(0x28) & INVALIDATE != 0 || self.read64(self.iotlb) & INVALIDATE != 0 {
            return Err(Error::Busy);
        }
        self.write64(0x28, INVALIDATE | (1 << 61));
        self.wait_invalidation(0x28, 1 << 59)?;
        // CAP.DRD/DWD were required; complete global IOTLB invalidation with
        // both translated reads and writes drained before returning success.
        self.write64(self.iotlb, INVALIDATE | (1 << 60) | (1 << 49) | (1 << 48));
        self.wait_invalidation(self.iotlb, 1 << 57)
    }
}

fn table_write(page: u64, index: usize, value: u64) {
    assert!(index < 512);
    paging::physical(page, |base| {
        // SAFETY: A new, exclusively kernel-owned page is held permanently.
        // The scratch callback exposes one aligned 4-KiB RW/NX mapping only;
        // index was bounded and this volatile scalar creates no aliases.
        unsafe { base.cast::<u64>().add(index).write_volatile(value) }
    });
}

fn flush_table(page: u64, cache_line: usize) {
    fence(Ordering::SeqCst);
    paging::physical(page, |base| {
        for offset in (0..PAGE as usize).step_by(cache_line) {
            // SAFETY: CPUID established CLFLUSH support and cache-line size.
            // Every operand lies within the owned, aligned scratch page; no
            // pointer escapes the callback. The trailing fence orders writeback.
            unsafe {
                asm!("clflush [{}]", in(reg) base.add(offset), options(nostack, preserves_flags))
            }
        }
        fence(Ordering::SeqCst);
    });
}
