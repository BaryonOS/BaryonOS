#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
//! Exact QEMU modern virtio-blk transport on one of two closed, fixed PCI
//! functions: 00:05.0 (every profile) and the test-only 00:06.0 archive disk.
//! No caller supplies a bus/device/function number. No user address enters here.
//! Caller retains five exclusive, identity-mapped kernel DMA pages until reboot.
//! Protocol-conforming QEMU is trusted; this is not a malicious-device IOMMU.
use core::{
    arch::asm,
    ptr,
    sync::atomic::{fence, Ordering},
};
use virtio_core::{descriptor_plan, parse_capabilities, validate_descriptors, QUEUE_SIZE};
pub use virtio_core::{Bar, Completion, DmaPool, Error, Regions};
const RAM_END: u64 = 256 * 1024 * 1024;
/// Closed set of fixed PCI functions this driver may touch.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Function {
    /// QEMU 00:05.0, the only function of every existing profile.
    Primary,
    /// QEMU 00:06.0, the test-only second (archive) disk of the volume fixture.
    Archive,
}
impl Function {
    const fn bdf(self) -> u32 {
        match self {
            Function::Primary => 5 << 11,
            Function::Archive => 6 << 11,
        }
    }
}
#[derive(Clone, Copy)]
pub struct Discovery {
    pub function: Function,
    pub config: [u8; 256],
    pub bars: [Bar; 6],
    pub regions: Regions,
}
fn cfg_read(function: Function, offset: u16) -> u32 {
    let value: u32;
    let bdf = function.bdf();
    // SAFETY: CPL0 single-CPU initialization/transport, one of two closed fixed
    // PCI functions. No userspace-supplied config address and no access to the
    // boot ESP's BDF (the harness proves the ESP is on neither fixed slot).
    unsafe {
        asm!("out dx,eax",in("dx") 0xcf8u16,in("eax") 0x80000000|bdf|u32::from(offset&0xfc),options(nomem,nostack));
        asm!("in eax,dx",in("dx") 0xcfcu16,out("eax") value,options(nomem,nostack));
    }
    value
}
fn cfg_write(function: Function, offset: u16, value: u32) {
    let bdf = function.bdf();
    // SAFETY: Same closed fixed-function exclusive config-I/O contract as cfg_read.
    unsafe {
        asm!("out dx,eax",in("dx") 0xcf8u16,in("eax") 0x80000000|bdf|u32::from(offset&0xfc),options(nomem,nostack));
        asm!("out dx,eax",in("dx") 0xcfcu16,in("eax") value,options(nomem,nostack));
    }
}
fn command(function: Function, value: u16) {
    let bdf = function.bdf();
    // Use 16-bit config access: writing the status upper half could clear W1C.
    // SAFETY: CPL0, command register of one closed fixed function only; no
    // arbitrary PCI authority.
    unsafe {
        asm!("out dx,eax",in("dx") 0xcf8u16,in("eax") 0x80000000|bdf|4,options(nomem,nostack));
        asm!("out dx,ax",in("dx") 0xcfcu16,in("ax") value,options(nomem,nostack));
    }
}
/// Discover the primary function 00:05.0; unchanged for every existing caller.
pub fn discover() -> Result<Discovery, Error> {
    discover_function(Function::Primary)
}
/// Discover one function of the closed set. An absent or non-virtio-blk
/// function fails closed with `Offline` before any BAR is probed.
pub fn discover_function(function: Function) -> Result<Discovery, Error> {
    let cfg_read = |offset| cfg_read(function, offset);
    let cfg_write = |offset, value| cfg_write(function, offset, value);
    let command = |value| command(function, value);
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
    quiesce_interrupts(function, &config)?;
    Ok(Discovery {
        function,
        config,
        bars,
        regions,
    })
}

/// Disable MSI/MSI-X as well as INTx; the driver polls without IRQ routing.
/// Walks the bounded capability list of the snapshot `config`.
fn quiesce_interrupts(function: Function, config: &[u8; 256]) -> Result<(), Error> {
    let mut at = config[0x34] as usize;
    let mut count = 0;
    while at != 0 && count < 48 {
        if !(0x40..=0xfc).contains(&at) || !at.is_multiple_of(4) {
            return Err(Error::Protocol);
        }
        let word = cfg_read(function, at as u16);
        if config[at] == 5 {
            cfg_write(function, at as u16, word & !(1 << 16));
        }
        if config[at] == 0x11 {
            cfg_write(function, at as u16, (word & !(1 << 31)) | (1 << 30));
        }
        at = config[at + 1] as usize;
        count += 1;
    }
    if at != 0 {
        return Err(Error::Protocol);
    }
    Ok(())
}

/// How the six BAR dwords of the fixed function compare with discovery after
/// an S3 wake (H05-T5c; a copy of the virtio-rng H05-T5b rule). No BAR is
/// ever sized again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarState {
    /// Every BAR dword still equals the discovered value.
    Kept,
    /// Every BAR lost its address (only read-only type bits remain); the
    /// discovered values were written back and read back equal.
    Restored,
}

/// Classify the current BAR dwords against the discovered ones. Accepted are
/// exactly two states: all unchanged, or all addresses cleared with the
/// read-only type bits intact. Anything else (a mixture, a different non-zero
/// base, changed type bits) is refused: the device then stays offline.
pub fn classify_bars(saved: [u32; 6], now: [u32; 6]) -> Result<BarState, Error> {
    if now == saved {
        return Ok(BarState::Kept);
    }
    let mut i = 0;
    while i < 6 {
        let low = saved[i];
        let (type_bits, wide) = if low & 1 != 0 {
            (3, false)
        } else {
            (15, low & 6 == 4)
        };
        if now[i] != low & type_bits {
            return Err(Error::Range);
        }
        if wide {
            if i == 5 || now[i + 1] != 0 {
                return Err(Error::Range);
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    Ok(BarState::Restored)
}

fn saved_dword(config: &[u8; 256], offset: usize) -> u32 {
    u32::from_le_bytes([
        config[offset],
        config[offset + 1],
        config[offset + 2],
        config[offset + 3],
    ])
}

/// The identity dwords compared after a wake, in this order: 0x00
/// vendor/device, 0x08 revision/class, 0x2c subsystem, 0x34 capability
/// pointer (its low byte; the rest is reserved).
const IDENTITY: [u16; 4] = [0x00, 0x08, 0x2c, 0x34];

/// Pure identity comparison (H05-T5c I4): every dword of `IDENTITY` must equal
/// the discovered snapshot, and the snapshot must be the modern virtio-blk.
pub fn identity_matches(saved: &[u8; 256], now: [u32; 4]) -> Result<(), Error> {
    if saved_dword(saved, 0) != 0x10421af4 || now[0] != 0x10421af4 {
        return Err(Error::Offline);
    }
    for (i, offset) in IDENTITY.into_iter().enumerate() {
        let mask = if offset == 0x34 { 0xff } else { u32::MAX };
        if saved_dword(saved, offset as usize) & mask != now[i] & mask {
            return Err(Error::Protocol);
        }
    }
    Ok(())
}

/// Re-validate the primary function after an S3 wake without sizing any BAR
/// (H05-T5c I2-I4, I13): identity first with config reads only, then the six
/// BAR dwords (restored only from the all-cleared state and read back),
/// memory decode with INTx disabled and no bus mastering, the capability
/// regions (equal to `saved`) and the MSI/MSI-X quiesce of discovery. No MMIO;
/// DMA stays impossible until `Device::initialize_profile`.
pub fn rediscover(saved: &Discovery) -> Result<BarState, Error> {
    let function = saved.function;
    if function != Function::Primary {
        return Err(Error::Unsupported);
    }
    let now = IDENTITY.map(|offset| cfg_read(function, offset));
    identity_matches(&saved.config, now)?;
    // Decode and bus mastering off while the BARs may change.
    command(function, (cfg_read(function, 4) as u16 & !7) | (1 << 10));
    let mut old = [0u32; 6];
    let mut bars = [0u32; 6];
    for i in 0..6 {
        old[i] = saved_dword(&saved.config, 0x10 + i * 4);
        bars[i] = cfg_read(function, 0x10 + i as u16 * 4);
    }
    let state = classify_bars(old, bars)?;
    if state == BarState::Restored {
        for (i, value) in old.iter().enumerate() {
            cfg_write(function, 0x10 + i as u16 * 4, *value);
        }
        for (i, value) in old.iter().enumerate() {
            if cfg_read(function, 0x10 + i as u16 * 4) != *value {
                return Err(Error::Range);
            }
        }
    }
    // The discovery state: memory decode, INTx disabled, no bus mastering.
    command(
        function,
        (cfg_read(function, 4) as u16 & !7) | 2 | (1 << 10),
    );
    let cmd = cfg_read(function, 4);
    if cmd & 7 != 2 || cmd & (1 << 10) == 0 {
        return Err(Error::Protocol);
    }
    let mut config = [0u8; 256];
    for offset in (0..256).step_by(4) {
        config[offset..offset + 4]
            .copy_from_slice(&cfg_read(function, offset as u16).to_le_bytes());
    }
    if parse_capabilities(&config, &saved.bars)? != saved.regions {
        return Err(Error::Protocol);
    }
    quiesce_interrupts(function, &config)?;
    Ok(state)
}

/// Negotiated transport state recorded by `initialize_profile` (H05-T5c I7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Negotiated {
    /// Feature bits the device offered (before any test-only masking).
    pub offered: u64,
    /// Feature bits the driver accepted.
    pub features: u64,
    /// queue_size_max of queue 0.
    pub queue_max: u16,
    /// queue_notify_off of queue 0.
    pub notify_offset: u16,
    /// Capacity in 512-byte sectors.
    pub capacity: u64,
}

/// Pure negotiated-state comparison (H05-T5c I7): after re-init every field
/// must equal the pre-sleep value, and the queue must still hold `QUEUE_SIZE`.
pub fn negotiated_matches(saved: Negotiated, now: Negotiated) -> Result<(), Error> {
    if now.queue_max < QUEUE_SIZE {
        return Err(Error::Unsupported);
    }
    if now.capacity != saved.capacity {
        return Err(Error::Range);
    }
    if now != saved {
        return Err(Error::Protocol);
    }
    Ok(())
}
pub struct Device {
    function: Function,
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
    negotiated: Negotiated,
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
            function: info.function,
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
            negotiated: Negotiated {
                offered: 0,
                features: 0,
                queue_max: 0,
                notify_offset: 0,
                capacity: 0,
            },
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
            result.negotiated.offered = offered;
            if no_flush {
                offered &= !(1 << 9);
            }
            let features = if writable {
                virtio_core::negotiate_writable_features(offered)?
            } else {
                virtio_core::negotiate_features(offered)?
            };
            result.negotiated.features = features;
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
            result.negotiated.queue_max = read16(common + 24);
            if result.negotiated.queue_max < QUEUE_SIZE {
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
            result.negotiated.notify_offset = queue_offset;
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
            result.negotiated.capacity = result.capacity;
            if result.capacity < 8 {
                return Err(Error::Range);
            }
            fence(Ordering::SeqCst);
            let f = info.function;
            command(f, (cfg_read(f, 4) as u16) | 6 | (1 << 10));
            write8(common + 20, 15);
            if read8(common + 20) != 15 {
                // Failure after BME-enable is not an idle/offline device: first
                // request reset and disable bus mastering. Caller quarantines
                // its permanent pool; no future reuse is authorized by this Err.
                write8(common + 20, 0);
                command(f, (cfg_read(f, 4) as u16 & !4) | (1 << 10));
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
        // SAFETY: This device's own fixed function; stop DMA authorization and
        // request protocol reset. Pages stay pinned even without reset ack.
        unsafe {
            write8(self.common + 20, 0);
        }
        let f = self.function;
        command(f, (cfg_read(f, 4) as u16 & !4) | (1 << 10));
    }
    pub fn reset_acknowledged(&self) -> bool {
        // SAFETY: Own UC status register; BME read of this device's own fixed
        // function confirms its DMA disable.
        unsafe { read8(self.common + 20) == 0 && cfg_read(self.function, 4) & 4 == 0 }
    }
    pub fn pending(&self) -> bool {
        self.pending
    }
    /// The transport state recorded by `initialize_profile`; no device access.
    pub fn negotiated(&self) -> Negotiated {
        self.negotiated
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
mod tests {
    use super::*;
    use virtio_core::{Binding, Operation, Policy};

    /// QEMU q35, 00:05.0 BAR4 64-bit prefetchable, before and after a wake
    /// that cleared the addresses (the H05-T5b observation on 00:06.0).
    const DISCOVERED: [u32; 6] = [0, 0x8000_1000, 0, 0, 0x0000_800c, 0xc0];
    const WOKEN: [u32; 6] = [0, 0, 0, 0, 0xc, 0];

    #[test]
    fn classify_bars_keeps_or_restores_only_from_the_cleared_state() {
        assert_eq!(classify_bars(DISCOVERED, DISCOVERED), Ok(BarState::Kept));
        assert_eq!(classify_bars(DISCOVERED, WOKEN), Ok(BarState::Restored));
        assert_eq!(classify_bars([0; 6], [0; 6]), Ok(BarState::Kept));
        assert_eq!(
            classify_bars([0xc001, 0, 0, 0, 0, 0], [1, 0, 0, 0, 0, 0]),
            Ok(BarState::Restored)
        );
        for now in [
            [0, 0x8000_1000, 0, 0, 0xc, 0],
            [0, 0, 0, 0, 0x0000_800c, 0xc0],
            [0, 0x8000_2000, 0, 0, 0x0000_800c, 0xc0],
            [0, 0, 0, 0, 0xc, 0xc1],
            [0, 0, 0, 0, 0x0000_900c, 0],
            [0, 0, 0, 0, 0x4, 0],
            [0, 0, 0, 0, 0x0, 0],
            [0, 1, 0, 0, 0xc, 0],
            [0x1000, 0, 0, 0, 0xc, 0],
            [0, 0, 0, 0x8, 0xc, 0],
            [u32::MAX; 6],
        ] {
            assert_eq!(
                classify_bars(DISCOVERED, now),
                Err(Error::Range),
                "{now:x?}"
            );
        }
        assert_eq!(
            classify_bars([0x1000, 0, 0, 0, 0, 4], [0, 0, 0, 0, 0, 4]),
            Err(Error::Range)
        );
    }

    fn snapshot() -> ([u8; 256], [u32; 4]) {
        let mut config = [0u8; 256];
        let identity: [u32; 4] = [0x1042_1af4, 0x0100_0001, 0x0002_1af4, 0x0000_0098];
        for (offset, value) in IDENTITY.into_iter().zip(identity) {
            config[offset as usize..offset as usize + 4].copy_from_slice(&value.to_le_bytes());
        }
        (config, identity)
    }

    #[test]
    fn rediscover_identity_refuses_every_changed_field() {
        let (saved, now) = snapshot();
        assert_eq!(identity_matches(&saved, now), Ok(()));
        // Reserved bits above the capability pointer byte are ignored.
        let mut reserved = now;
        reserved[3] |= 0xff00;
        assert_eq!(identity_matches(&saved, reserved), Ok(()));
        for (i, bits) in [(1, 1), (1, 1 << 24), (2, 1), (2, 1 << 16), (3, 4)] {
            let mut changed = now;
            changed[i] ^= bits;
            assert_eq!(
                identity_matches(&saved, changed),
                Err(Error::Protocol),
                "{i} {bits:x}"
            );
        }
        // A different function or a vanished device: Offline.
        let mut other = now;
        other[0] = 0x1044_1af4;
        assert_eq!(identity_matches(&saved, other), Err(Error::Offline));
        other[0] = u32::MAX;
        assert_eq!(identity_matches(&saved, other), Err(Error::Offline));
        let mut wrong = saved;
        wrong[2] = 0x44;
        assert_eq!(identity_matches(&wrong, now), Err(Error::Offline));
    }

    const SAVED: Negotiated = Negotiated {
        offered: (1 << 32) | (1 << 9) | (1 << 5),
        features: (1 << 32) | (1 << 9),
        queue_max: 256,
        notify_offset: 0,
        capacity: 4096,
    };

    #[test]
    fn negotiated_state_compare_refuses_any_difference() {
        assert_eq!(negotiated_matches(SAVED, SAVED), Ok(()));
        let changes = [
            Negotiated {
                capacity: 4097,
                ..SAVED
            },
            Negotiated {
                capacity: 4095,
                ..SAVED
            },
            Negotiated {
                offered: SAVED.offered | (1 << 33),
                ..SAVED
            },
            Negotiated {
                features: SAVED.features & !(1 << 9),
                ..SAVED
            },
            Negotiated {
                queue_max: 128,
                ..SAVED
            },
            Negotiated {
                notify_offset: 1,
                ..SAVED
            },
        ];
        let expected = [
            Error::Range,
            Error::Range,
            Error::Protocol,
            Error::Protocol,
            Error::Protocol,
            Error::Protocol,
        ];
        for (now, error) in changes.into_iter().zip(expected) {
            assert_eq!(negotiated_matches(SAVED, now), Err(error), "{now:?}");
        }
        // A queue smaller than QUEUE_SIZE is refused even if it was so before.
        let small = Negotiated {
            queue_max: QUEUE_SIZE - 1,
            ..SAVED
        };
        assert_eq!(negotiated_matches(small, small), Err(Error::Unsupported));
    }

    /// I9 on the public Policy API, as the kernel re-init drives it: one
    /// `start_reset` -> `reset_completed(true)` on the same object keeps the
    /// ticket sequence (observed on a copy), advances the epoch by one and
    /// refuses every pre-sleep ticket and epoch.
    #[test]
    fn policy_epoch_ticket_floor_survives_the_reinit_reset() {
        let binding = Binding {
            pid: 7,
            object_key: 1 << 32,
        };
        let read = Operation::Read {
            sector: 0,
            offset: 0,
            length: 1,
        };
        let next = |policy: &Policy| {
            let mut probe = *policy;
            probe.prepare(binding, read, 8, 0, 1).unwrap().ticket
        };
        let mut policy = Policy::new();
        policy.reset_completed(true).unwrap();
        for _ in 0..3 {
            let r = policy.prepare(binding, read, 8, 0, 1).unwrap();
            policy.publish(r.epoch, r.ticket).unwrap();
            policy
                .complete(r.epoch, r.ticket, Completion::Success)
                .unwrap();
            policy.consume(binding, r.ticket).unwrap();
        }
        let floor = next(&policy);
        let epoch = policy.epoch();
        assert_eq!(floor, 4);
        policy.start_reset().unwrap();
        policy.reset_completed(true).unwrap();
        assert_eq!(policy.epoch(), epoch + 1);
        assert_eq!(next(&policy), floor, "the ticket floor never restarts");
        assert_eq!(policy.peek(binding, floor - 1), Err(Error::WrongTicket));
        let mut probe = policy;
        let r = probe.prepare(binding, read, 8, 0, 1).unwrap();
        assert_eq!(probe.publish(epoch, r.ticket), Err(Error::StaleEpoch));
        // A refused re-init quarantines the same object; no ticket restarts.
        let mut refused = policy;
        refused.start_reset().unwrap();
        assert_eq!(refused.reset_completed(false), Err(Error::Offline));
        assert_eq!(refused.phase(), virtio_core::Phase::Quarantined);
        assert_eq!(refused.start_reset(), Err(Error::Offline));
    }
    #[test]
    fn only_two_fixed_functions_are_reachable() {
        // Exhaustive match: adding a variant fails to compile here.
        for f in [Function::Primary, Function::Archive] {
            match f {
                Function::Primary => assert_eq!(f.bdf(), 5 << 11),
                Function::Archive => assert_eq!(f.bdf(), 6 << 11),
            }
            assert_eq!(f.bdf() & !(0x1f << 11), 0, "bus 0, function 0 only");
        }
    }
}
