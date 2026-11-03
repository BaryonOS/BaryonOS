#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
//! Exact QEMU 00:05.0 modern virtio-blk transport. No user address enters here.
//! Caller retains five exclusive, identity-mapped kernel DMA pages until reboot.
//! Protocol-conforming QEMU is trusted; this is not a malicious-device IOMMU.
use core::{
    arch::asm,
    ptr,
    sync::atomic::{fence, Ordering},
};
use virtio_core::{descriptor_plan, parse_capabilities, validate_descriptors, QUEUE_SIZE};
pub use virtio_core::{Bar, Completion, DmaPool, Error, Regions};
const BDF: u32 = 5 << 11;
const RAM_END: u64 = 256 * 1024 * 1024;
#[derive(Clone, Copy)]
pub struct Discovery {
    pub config: [u8; 256],
    pub bars: [Bar; 6],
    pub regions: Regions,
}
fn cfg_read(offset: u16) -> u32 {
    let value: u32;
    // SAFETY: CPL0 single-CPU initialization/transport, exact fixed PCI BDF.
    // No userspace-supplied config address and no access to the boot ESP's BDF.
    unsafe {
        asm!("out dx,eax",in("dx") 0xcf8u16,in("eax") 0x80000000|BDF|u32::from(offset&0xfc),options(nomem,nostack));
        asm!("in eax,dx",in("dx") 0xcfcu16,out("eax") value,options(nomem,nostack));
    }
    value
}
fn cfg_write(offset: u16, value: u32) {
    // SAFETY: Same fixed-device exclusive config-I/O contract as cfg_read.
    unsafe {
        asm!("out dx,eax",in("dx") 0xcf8u16,in("eax") 0x80000000|BDF|u32::from(offset&0xfc),options(nomem,nostack));
        asm!("out dx,eax",in("dx") 0xcfcu16,in("eax") value,options(nomem,nostack));
    }
}
fn command(value: u16) {
    // Use 16-bit config access: writing the status upper half could clear W1C.
    // SAFETY: CPL0, fixed BDF command register only; no arbitrary PCI authority.
    unsafe {
        asm!("out dx,eax",in("dx") 0xcf8u16,in("eax") 0x80000000|BDF|4,options(nomem,nostack));
        asm!("out dx,ax",in("dx") 0xcfcu16,in("ax") value,options(nomem,nostack));
    }
}
pub fn discover() -> Result<Discovery, Error> {
    if cfg_read(0) != 0x10421af4 {
        return Err(Error::Offline);
    }
    let original = cfg_read(4) as u16;
    command((original & !7) | (1 << 10)); // DMA and decode off while probing only target BARs.
    let mut bars = [Bar::NONE; 6];
    let mut i = 0;
    while i < 6 {
        let offset = 0x10 + i as u16 * 4;
        let low = cfg_read(offset);
        if low & 1 != 0 {
            i += 1;
            continue;
        }
        let wide = low & 6 == 4;
        if low & 6 != 0 && !wide || wide && i == 5 {
            return Err(Error::Unsupported);
        }
        let high = if wide { cfg_read(offset + 4) } else { 0 };
        cfg_write(offset, u32::MAX);
        if wide {
            cfg_write(offset + 4, u32::MAX);
        }
        let mask_low = cfg_read(offset);
        let mask_high = if wide { cfg_read(offset + 4) } else { u32::MAX };
        cfg_write(offset, low);
        if wide {
            cfg_write(offset + 4, high);
        }
        let base = u64::from(low & !15) | (u64::from(high) << 32);
        let mask = u64::from(mask_low & !15) | (u64::from(mask_high) << 32);
        let size = (!mask).wrapping_add(1);
        if base != 0 && size != 0 {
            if !size.is_power_of_two() || size > 65536 || base % 4096 != 0 {
                return Err(Error::Range);
            }
            bars[i] = Bar {
                base,
                size,
                is_memory: true,
            };
        }
        i += if wide { 2 } else { 1 };
    }
    command((original & !5) | 2 | (1 << 10)); // MMIO decode on; DMA remains off.
    let mut config = [0u8; 256];
    for offset in (0..256).step_by(4) {
        config[offset..offset + 4].copy_from_slice(&cfg_read(offset as u16).to_le_bytes());
    }
    let regions = parse_capabilities(&config, &bars)?;
    // Disable MSI/MSI-X as well as INTx; first driver polls without IRQ routing.
    let mut at = config[0x34] as usize;
    let mut count = 0;
    while at != 0 && count < 48 {
        if !(0x40..=0xfc).contains(&at) || !at.is_multiple_of(4) {
            return Err(Error::Protocol);
        }
        let word = cfg_read(at as u16);
        if config[at] == 5 {
            cfg_write(at as u16, word & !(1 << 16));
        }
        if config[at] == 0x11 {
            cfg_write(at as u16, (word & !(1 << 31)) | (1 << 30));
        }
        at = config[at + 1] as usize;
        count += 1;
    }
    if at != 0 {
        return Err(Error::Protocol);
    }
    Ok(Discovery {
        config,
        bars,
        regions,
    })
}
pub struct Device {
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
    pub unsafe fn initialize(
        info: Discovery,
        mapped: [u64; 6],
        pool: DmaPool,
    ) -> Result<Self, Error> {
        // SAFETY: Caller supplies the permanent validated MMIO and pinned pool.
        unsafe { Self::initialize_profile(info, mapped, pool, false, false) }
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
        pool.validate(RAM_END)?;
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
            command((cfg_read(4) as u16) | 6 | (1 << 10));
            write8(common + 20, 15);
            if read8(common + 20) != 15 {
                // Failure after BME-enable is not an idle/offline device: first
                // request reset and disable bus mastering. Caller quarantines
                // its permanent pool; no future reuse is authorized by this Err.
                write8(common + 20, 0);
                command((cfg_read(4) as u16 & !4) | (1 << 10));
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
    pub fn begin_reset(&mut self) {
        self.ready = false;
        // SAFETY: Fixed owned device; stop DMA authorization and request protocol
        // reset. Pages stay pinned even if the device never acknowledges reset.
        unsafe {
            write8(self.common + 20, 0);
        }
        command((cfg_read(4) as u16 & !4) | (1 << 10));
    }
    pub fn reset_acknowledged(&self) -> bool {
        // SAFETY: Own UC status register; BME read confirms fixed-device disable.
        unsafe { read8(self.common + 20) == 0 && cfg_read(4) & 4 == 0 }
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
