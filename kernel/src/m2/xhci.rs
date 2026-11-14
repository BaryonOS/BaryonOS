//! H02 isolated QEMU xHCI DMA fixture; optional bounded USB enumeration, no IRQs.
//!
//! Register contracts: Intel xHCI 1.2c §§4.2, 4.6.2, 5.4, 5.5 and 6.4.3.1.
//! The kernel owns all PCI/MMIO access. Only an established native VT-d domain
//! can enable bus mastering; the generic device-execution model stays blocked.
//! All DMA and translation pages remain retained through the terminal boot.
use super::{iommu, paging, FRAME_COUNT};
use crate::serial;
use core::{
    arch::asm,
    ptr::NonNull,
    sync::atomic::{fence, Ordering},
};
use driver_core::PciAddress;
use kernel_core::{device_execution as device, Frames, Owner, PAGE};
use nieuw_os_xhci::{ControllerCapabilities, RegisterAccess};
use pci_core::{ConfigError, PciConfigAccess};

#[cfg(feature = "native-usb-hid-probe")]
mod usb;

const MMIO: u64 = 0x3f210000;
const POLLS: usize = 1_000_000;
const QEMU_ID: u32 = 0x000d_1b36;
const COMMAND: u8 = 4;
const MEMORY: u16 = 2;
const BUS_MASTER: u16 = 4;
const INT_DISABLE: u16 = 1 << 10;

#[derive(Debug)]
enum Error {
    Pci,
    Profile,
    Mapping,
    Timeout,
    Dma,
    Event,
    Guard,
    Isolation,
    #[cfg(feature = "native-usb-hid-probe")]
    Usb,
}

/// IF=0 bootstrap-only PCI mechanism 1; no untrusted caller or concurrent CPU.
struct Pci;
impl Pci {
    fn select(address: PciAddress, offset: u8) -> Result<(), ConfigError> {
        if address.segment != 0 {
            return Err(ConfigError::UnsupportedSegment);
        }
        if address.device > 31 || address.function > 7 {
            return Err(ConfigError::WrongFunction);
        }
        let selector = 0x8000_0000u32
            | u32::from(address.bus) << 16
            | u32::from(address.device) << 11
            | u32::from(address.function) << 8
            | u32::from(offset & !3);
        // SAFETY: CPL0, one BSP with interrupts disabled owns PCI config ports.
        unsafe {
            asm!("out dx, eax", in("dx") 0xcf8u16, in("eax") selector, options(nostack));
        }
        Ok(())
    }
}
impl PciConfigAccess for Pci {
    fn read_u32(&mut self, address: PciAddress, offset: u8) -> Result<u32, ConfigError> {
        if offset & 3 != 0 {
            return Err(ConfigError::InvalidOffset);
        }
        Self::select(address, offset)?;
        let value: u32;
        // SAFETY: Selected aligned conventional config dword, serialized above.
        unsafe {
            asm!("in eax, dx", in("dx") 0xcfcu16, out("eax") value, options(nostack));
        }
        Ok(value)
    }
    fn write_u32(
        &mut self,
        address: PciAddress,
        offset: u8,
        value: u32,
    ) -> Result<(), ConfigError> {
        if offset & 3 != 0 {
            return Err(ConfigError::InvalidOffset);
        }
        Self::select(address, offset)?;
        // SAFETY: Same exclusive selected config access. Used by BAR probing.
        unsafe {
            asm!("out dx, eax", in("dx") 0xcfcu16, in("eax") value, options(nostack));
        }
        Ok(())
    }
    fn write_u16(
        &mut self,
        address: PciAddress,
        offset: u8,
        value: u16,
    ) -> Result<(), ConfigError> {
        if offset & 1 != 0 {
            return Err(ConfigError::InvalidOffset);
        }
        Self::select(address, offset)?;
        // SAFETY: Word access avoids writing adjacent W1C PCI status bits.
        unsafe {
            asm!("out dx, ax", in("dx") (0xcfcu16 + u16::from(offset & 2)), in("ax") value, options(nostack));
        }
        Ok(())
    }
}

struct Controller {
    address: PciAddress,
    bytes: u64,
    caps: ControllerCapabilities,
}
impl Controller {
    fn read(&self, offset: u64) -> u32 {
        assert!(
            offset.is_multiple_of(4) && offset.checked_add(4).is_some_and(|end| end <= self.bytes)
        );
        // SAFETY: Validated BAR0, supervisor UC/NX, aligned bounded u32 access.
        unsafe { ((MMIO + offset) as *const u32).read_volatile() }
    }
    fn write(&self, offset: u64, value: u32) {
        assert!(
            offset.is_multiple_of(4) && offset.checked_add(4).is_some_and(|end| end <= self.bytes)
        );
        // SAFETY: Same exclusive kernel-owned BAR mapping; no userspace alias.
        unsafe {
            ((MMIO + offset) as *mut u32).write_volatile(value);
        }
    }
    fn write64(&self, offset: u64, value: u64) {
        assert!(offset.is_multiple_of(8));
        // The fixed QEMU profile supports the xHCI §5.1 split-access fallback:
        // low DWORD before high DWORD. This is not a generic portability claim.
        self.write(offset, value as u32);
        self.write(offset + 4, (value >> 32) as u32);
    }
    fn wait(&self, offset: u64, mask: u32, wanted: u32) -> Result<(), Error> {
        for _ in 0..POLLS {
            if self.read(offset) & mask == wanted {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(Error::Timeout)
    }
    fn reset(&self) -> Result<(), Error> {
        let op = self.caps.operational_offset;
        self.wait(op + 4, 1 << 11, 0)?; // CNR
        self.write(op, 0); // halt, interrupts disabled
        self.wait(op + 4, 1, 1)?;
        self.write(op, 2); // HCRST only while halted
        self.wait(op, 2, 0)?;
        self.wait(op + 4, 1 << 11, 0)?;
        if self.read(op + 8) & 1 == 0 {
            return Err(Error::Profile);
        }
        Ok(())
    }
    fn stop(&self) -> Result<(), Error> {
        // Clear PCI BME even if controller halt times out. Retain every buffer.
        let op = self.caps.operational_offset;
        self.write(op, 0);
        set_command(self.address, MEMORY | INT_DISABLE)?;
        self.wait(op + 4, 1, 1)
    }
    fn enable_dma(&self, domain: &iommu::Domain<'_>) -> Result<(), Error> {
        let requester = domain.requester();
        if requester.segment() != self.address.segment
            || requester.bus() != self.address.bus
            || requester.device() != self.address.device
            || requester.function() != self.address.function
            || !domain.translation_enabled()
        {
            return Err(Error::Isolation);
        }
        set_command(self.address, MEMORY | INT_DISABLE | BUS_MASTER)
    }
}

fn set_command(address: PciAddress, command: u16) -> Result<(), Error> {
    Pci.write_u16(address, COMMAND, command)
        .map_err(|_| Error::Pci)?;
    let readback = Pci.read_u32(address, COMMAND).map_err(|_| Error::Pci)? as u16;
    if readback & (MEMORY | BUS_MASTER | INT_DISABLE | 1) != command {
        return Err(Error::Pci);
    }
    Ok(())
}

fn mask_pci_interrupts(address: PciAddress) -> Result<(), Error> {
    let status = Pci.read_u32(address, COMMAND).map_err(|_| Error::Pci)?;
    if status & (1 << 20) == 0 {
        return Ok(());
    }
    let mut next = (Pci.read_u32(address, 0x34).map_err(|_| Error::Pci)? & 255) as u8;
    let mut visited = [false; 64];
    for _ in 0..48 {
        if next == 0 {
            return Ok(());
        }
        if next < 0x40 || next & 3 != 0 || visited[usize::from(next / 4)] {
            return Err(Error::Profile);
        }
        visited[usize::from(next / 4)] = true;
        let cap = Pci.read_u32(address, next).map_err(|_| Error::Pci)?;
        let control = (cap >> 16) as u16;
        let changed = match cap as u8 {
            5 => Some(control & !1),                          // MSI Enable=0
            0x11 => Some((control & !(1 << 15)) | (1 << 14)), // MSI-X disable/mask
            _ => None,
        };
        if let Some(value) = changed {
            Pci.write_u16(address, next + 2, value)
                .map_err(|_| Error::Pci)?;
            if (Pci.read_u32(address, next).map_err(|_| Error::Pci)? >> 16) as u16 != value {
                return Err(Error::Pci);
            }
        }
        next = (cap >> 8) as u8;
    }
    Err(Error::Profile)
}

fn prepare(
    discovery: &boot_protocol::acpi::Discovery,
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
) -> Result<(Controller, device::PciFunction), Error> {
    // Validate the fixture identity in a read-only scan before the generic
    // probe can size BARs or disable decoding on a matched controller.
    let mut matches = 0;
    let mut supported = true;
    pci_core::enumerate(&mut Pci, |d| {
        if d.class == 0x0c && d.subclass == 3 && d.programming_interface == 0x30 {
            matches += 1;
            supported &= d.vendor_id == 0x1b36
                && d.device_id == 0x000d
                && d.address.segment == 0
                && d.address.bus == 0
                && d.address.function == 0;
        }
    })
    .map_err(|_| Error::Pci)?;
    if matches != 1 || !supported {
        return Err(Error::Profile);
    }
    let found = nieuw_os_xhci::discover(Pci).map_err(|_| Error::Pci)?;
    let address = found.address();
    let id = Pci.read_u32(address, 0).map_err(|_| Error::Pci)?;
    serial::hex(b"XHCI-DIAG:PCI_ID:", u64::from(id));
    // The fixture never takes ownership of physical hardware or bridged devices.
    if id != QEMU_ID || address.segment != 0 || address.bus != 0 || address.function != 0 {
        return Err(Error::Profile);
    }
    set_command(address, INT_DISABLE)?;
    mask_pci_interrupts(address)?;
    let bar = found.bar0();
    serial::hex(b"XHCI-DIAG:BAR_BASE:", bar.base());
    serial::hex(b"XHCI-DIAG:BAR_BYTES:", bar.size());
    if bar.size() > 0x10000 || bar.base() < 0x80000000 {
        return Err(Error::Profile);
    }
    if let Some(unit) = discovery.native_dma_unit(address.device, address.function) {
        if bar.base() < unit.register_base + unit.register_bytes
            && unit.register_base < bar.base() + bar.size()
        {
            return Err(Error::Profile);
        }
    }
    for offset in (0..bar.size()).step_by(PAGE as usize) {
        paging
            .map_native_device(frames, MMIO + offset, bar.base() + offset)
            .map_err(|_| Error::Mapping)?;
    }
    set_command(address, MEMORY | INT_DISABLE)?;
    // SAFETY: Exact complete BAR0 mapping installed above; BME remains disabled.
    let mut registers = unsafe { found.map_bar0(NonNull::new(MMIO as *mut u8).unwrap()) }
        .map_err(|_| Error::Mapping)?;
    for offset in [0, 4, 8, 0x10, 0x14, 0x18] {
        serial::hex(b"XHCI-DIAG:CAP_OFFSET:", offset);
        serial::hex(
            b"XHCI-DIAG:CAP_VALUE:",
            u64::from(registers.read_u32(offset).map_err(|_| Error::Profile)?),
        );
    }
    let caps =
        ControllerCapabilities::inspect(&mut registers, bar.size()).map_err(|_| Error::Profile)?;
    // This initial QEMU profile needs no scratchpad buffers or legacy handoff.
    if registers.read_u32(8).map_err(|_| Error::Profile)? & ((31 << 21) | (31 << 27)) != 0 {
        return Err(Error::Profile);
    }
    let controller = Controller {
        address,
        bytes: bar.size(),
        caps,
    };
    let mut next = caps.extended_capability_offset.unwrap_or(0);
    for _ in 0..64 {
        if next == 0 {
            break;
        }
        if next + 4 > bar.size() || !next.is_multiple_of(4) {
            return Err(Error::Profile);
        }
        let cap = controller.read(next);
        serial::hex(b"XHCI-DIAG:EXT_CAP:", u64::from(cap));
        if cap as u8 == 1 {
            return Err(Error::Profile);
        } // no unproved BIOS ownership handoff
        let delta = u64::from((cap >> 8) as u8) * 4;
        if delta == 0 {
            next = 0;
            break;
        }
        next = next.checked_add(delta).ok_or(Error::Profile)?;
    }
    if next != 0 {
        return Err(Error::Profile);
    }
    controller.reset()?;
    let mut bars = [device::PciBar::NONE; 6];
    bars[0] = device::PciBar::new(bar.base(), bar.size(), true).map_err(|_| Error::Profile)?;
    let function = device::PciFunction::from_kernel_probe(device::KernelPciSnapshot {
        address: device::PciAddress::new(0, address.bus, address.device, address.function)
            .map_err(|_| Error::Profile)?,
        vendor_id: id as u16,
        device_id: (id >> 16) as u16,
        class: 0x0c,
        subclass: 3,
        programming_interface: 0x30,
        generation: 1,
        bars,
    })
    .map_err(|_| Error::Profile)?;
    Ok((controller, function))
}

fn page_write(pa: u64, words: &[(usize, u64)]) {
    paging::physical(pa, |p| {
        for &(index, value) in words {
            assert!(index < 512);
            // SAFETY: Exclusive pinned page, bounded aligned u64; device stopped.
            unsafe {
                p.cast::<u64>().add(index).write_volatile(value);
            }
        }
    });
}
fn guard_unchanged(pa: u64) -> bool {
    paging::physical_read(pa, |p| {
        // SAFETY: Retained private page, exactly one page of bounded volatile reads.
        (0..PAGE as usize).all(|i| unsafe { p.add(i).read_volatile() } == 0xa5)
    })
}

fn execute(
    controller: &Controller,
    discovery: &boot_protocol::acpi::Discovery,
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
    function: device::PciFunction,
) -> Result<(), Error> {
    let driver = device::DriverId::new(0x4e4f_0004).map_err(|_| Error::Dma)?;
    let mut registry = device::LeaseRegistry::<1>::new();
    let mut lease = registry
        .lease(
            function,
            device::DriverMatch::new(driver, 0x1b36, 0x000d, 0x0c, 3, 0x30),
        )
        .map_err(|_| Error::Dma)?;
    lease.authorize(driver, 1).map_err(|_| Error::Dma)?;
    lease.begin_initialization().map_err(|_| Error::Dma)?;
    let command = lease
        .allocate_dma(
            frames,
            PAGE as usize,
            device::DmaDirection::DeviceReads,
            paging::zero_dynamic,
        )
        .map_err(|_| Error::Dma)?;
    let event = lease
        .allocate_dma(
            frames,
            PAGE as usize,
            device::DmaDirection::DeviceWrites,
            paging::zero_dynamic,
        )
        .map_err(|_| Error::Dma)?;
    let erst = lease
        .allocate_dma(
            frames,
            PAGE as usize,
            device::DmaDirection::DeviceReads,
            paging::zero_dynamic,
        )
        .map_err(|_| Error::Dma)?;
    let dcbaa = lease
        .allocate_dma(
            frames,
            PAGE as usize,
            device::DmaDirection::DeviceReads,
            paging::zero_dynamic,
        )
        .map_err(|_| Error::Dma)?;
    let command_pa = lease.dma_pages(&command).map_err(|_| Error::Dma)?[0];
    let event_pa = lease.dma_pages(&event).map_err(|_| Error::Dma)?[0];
    let erst_pa = lease.dma_pages(&erst).map_err(|_| Error::Dma)?[0];
    let guard = frames
        .allocate(Owner::Kernel, |pa| {
            paging::physical(pa, |p| {
                // SAFETY: Allocator's exclusive unpublished page, exactly PAGE bytes.
                unsafe {
                    core::ptr::write_bytes(p, 0xa5, PAGE as usize);
                }
            })
        })
        .map_err(|_| Error::Dma)?;
    let mut domain = iommu::establish(
        discovery,
        frames,
        paging,
        &lease,
        &[&command, &event, &erst, &dcbaa],
    )
    .map_err(|_| Error::Isolation)?;
    let command_io = domain.iova(0, 0).map_err(|_| Error::Dma)?;
    let event_io = if cfg!(feature = "test-xhci-dma-fault") {
        if (0x100000..0x110000).contains(&guard) {
            return Err(Error::Profile);
        }
        guard
    } else {
        domain.iova(1, 0).map_err(|_| Error::Dma)?
    };
    page_write(command_pa, &[(0, 0), (1, ((23u64 << 10) | 1) << 32)]);
    // One 256-TRB event segment. Remaining command entries have cycle=0.
    page_write(erst_pa, &[(0, event_io), (1, 256)]);
    fence(Ordering::SeqCst);
    paging.audit(frames);
    serial::write(b"XHCI:PAGING:PASS\nXHCI:DOMAIN:PASS\n");
    controller.enable_dma(&domain)?;
    serial::write(b"XHCI:BUS_MASTER:CONTAINED\n");
    let op = controller.caps.operational_offset;
    let intr = controller.caps.runtime_offset + 0x20;
    controller.write(op + 0x38, 1); // CONFIG MaxSlotsEn
    controller.write64(op + 0x30, domain.iova(3, 0).map_err(|_| Error::Dma)?);
    controller.write64(op + 0x18, command_io | 1);
    controller.write(intr, 0); // IMAN.IE=0
    controller.write(intr + 8, 1); // ERSTSZ
    controller.write64(intr + 0x18, event_io);
    controller.write64(intr + 0x10, domain.iova(2, 0).map_err(|_| Error::Dma)?);
    controller.write(op, 1); // run, INTE=0
    controller.wait(op + 4, 1, 0)?;
    controller.write(controller.caps.doorbell_offset, 0);
    let mut completed = false;
    for _ in 0..POLLS {
        if cfg!(feature = "test-xhci-dma-fault") {
            if let Some(fault) = domain.fault().map_err(|_| Error::Isolation)? {
                let expected = u16::from(controller.address.bus) << 8
                    | u16::from(controller.address.device) << 3
                    | u16::from(controller.address.function);
                if fault.requester != expected
                    || fault.address & !(PAGE - 1) != guard
                    || !fault.write
                    || fault.reason != 5
                {
                    return Err(Error::Isolation);
                }
                serial::write(b"XHCI:DMA_FAULT:WRITE_DENIED\n");
                completed = true;
                break;
            }
        } else {
            let words = paging::physical_read(event_pa, |p| {
                // SAFETY: Observe the device's published cycle bit before its
                // payload. All accesses are bounded volatile reads of the pinned
                // event page; no Rust reference aliases device-written memory.
                unsafe {
                    let header = p.cast::<u64>().add(1).read_volatile();
                    if header >> 32 & 1 == 0 {
                        return [0, header];
                    }
                    fence(Ordering::Acquire);
                    [p.cast::<u64>().read_volatile(), header]
                }
            });
            if words[1] >> 32 & 1 != 0 {
                fence(Ordering::Acquire);
                let expected = (1u64 << 24) | (((33u64 << 10) | 1) << 32);
                if words[0] != command_io || words[1] != expected {
                    return Err(Error::Event);
                }
                if domain.fault().map_err(|_| Error::Isolation)?.is_some() {
                    return Err(Error::Isolation);
                }
                serial::write(b"XHCI:NOOP_COMPLETION:PASS\n");
                completed = true;
                break;
            }
        }
        core::hint::spin_loop();
    }
    if !completed {
        return Err(Error::Timeout);
    }
    controller.stop()?;
    domain.deny_all().map_err(|_| Error::Isolation)?;
    if !guard_unchanged(guard) {
        return Err(Error::Guard);
    }
    serial::write(b"XHCI:GUARD:UNCHANGED\nXHCI:QUIESCE:PASS\nXHCI:PAGES:RETAINED\n");
    // No release API is invoked: the fixture ends with table and DMA ownership
    // retained, including any failure. This is not a reusable native driver yet.
    Ok(())
}

pub(super) fn run(
    discovery: &boot_protocol::acpi::Discovery,
    frames: &mut Frames<FRAME_COUNT>,
    paging: &mut paging::Paging,
) {
    serial::write(b"XHCI:BEGIN\n");
    let (controller, function) = prepare(discovery, frames, paging).unwrap_or_else(|error| {
        serial::hex(b"XHCI:FAIL:PREPARE:", error as u64);
        serial::fail(b"XHCI_PREPARE")
    });
    if cfg!(feature = "test-xhci-no-dmar") {
        assert!(discovery
            .native_dma_unit(controller.address.device, controller.address.function)
            .is_none());
        // Exercise the actual establishment boundary: inventory absence must
        // reject the same path used by the positive controller execution case.
        let rejected = matches!(
            execute(&controller, discovery, frames, paging, function),
            Err(Error::Isolation)
        );
        let bus_master_was_clear = Pci
            .read_u32(controller.address, COMMAND)
            .is_ok_and(|command| command & u32::from(BUS_MASTER) == 0);
        controller.stop().expect("xHCI absent-DMAR halt");
        assert!(
            rejected,
            "missing DMAR did not reject native DMA establishment"
        );
        assert!(
            bus_master_was_clear,
            "DMA rejection enabled PCI bus mastering"
        );
        paging.audit(frames);
        serial::write(b"XHCI:NO_DMAR:BUS_MASTER_BLOCKED\nXHCI:PASS\n");
        serial::exit(0x10);
    }
    #[cfg(feature = "native-usb-hid-probe")]
    let result = usb::execute(&controller, discovery, frames, paging, function);
    #[cfg(not(feature = "native-usb-hid-probe"))]
    let result = execute(&controller, discovery, frames, paging, function);
    // Both success and error clear BME before a terminal result; failed teardown
    // never turns retained pages back into allocator capacity.
    let stopped = controller.stop();
    result.expect("xHCI contained DMA gate");
    stopped.expect("xHCI final halt");
    serial::write(b"XHCI:PASS\n");
    serial::exit(0x10)
}
