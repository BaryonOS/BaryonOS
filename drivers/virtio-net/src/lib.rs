#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
//! Exact, polled QEMU modern virtio-net transport. No IP parser, routing,
//! capability authority or userspace pointers live here. Four caller-owned DMA
//! pages stay exclusively pinned until reboot, including failed initialization.
//! Host models exercise the same queue engine, not actual PCI or packet I/O.

use core::sync::atomic::{fence, Ordering};
use virtio_core::{parse_capabilities_for, Descriptor, ACCESS_PLATFORM, VERSION_1, WRITE};
pub use virtio_core::{Bar, Error, Regions};

mod pci;
pub use pci::discover;
use pci::Hardware;

pub const DEVICE_ID_NET: u16 = 0x1041;
pub const QUEUE_SIZE: u16 = 8;
pub const MAX_FRAME_BYTES: usize = 1514;
pub const HEADER_BYTES: usize = 12;
pub const BUFFER_BYTES: usize = HEADER_BYTES + MAX_FRAME_BYTES;
pub const FEATURE_MAC: u64 = 1 << 5;
pub const FEATURE_STATUS: u64 = 1 << 16;
const RAM_END: u64 = 256 * 1024 * 1024;
const RX: usize = 0;
const TX: usize = 1;

#[derive(Clone, Copy)]
pub struct Discovery {
    pub config: [u8; 256],
    pub bars: [Bar; 6],
    pub regions: Regions,
}

/// Four distinct, exclusive, identity-mapped and permanently pinned RAM pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetPool {
    pub rx_ring: u64,
    pub tx_ring: u64,
    pub rx_bounce: u64,
    pub tx_bounce: u64,
}

impl NetPool {
    pub fn validate(self, ram_end: u64) -> Result<(), Error> {
        let pages = [self.rx_ring, self.tx_ring, self.rx_bounce, self.tx_bounce];
        for (index, page) in pages.iter().enumerate() {
            if *page == 0
                || !page.is_multiple_of(4096)
                || page.checked_add(4096).ok_or(Error::Range)? > ram_end
                || pages[..index].contains(page)
            {
                return Err(Error::Range);
            }
        }
        Ok(())
    }

    fn queue(self, index: usize) -> QueueMemory {
        QueueMemory {
            ring: if index == RX {
                self.rx_ring
            } else {
                self.tx_ring
            },
            bounce: if index == RX {
                self.rx_bounce
            } else {
                self.tx_bounce
            },
        }
    }
}

#[derive(Clone, Copy)]
struct QueueMemory {
    ring: u64,
    bounce: u64,
}

impl QueueMemory {
    fn available(self) -> u64 {
        self.ring + 256
    }
    fn used(self) -> u64 {
        self.ring + 512
    }
}

/// Exact supported feature selection; offloads and mergeable buffers are never
/// accepted. ACCESS_PLATFORM needs a different DMA/IOMMU integration profile.
pub fn negotiate_features(offered: u64) -> Result<u64, Error> {
    let required = VERSION_1 | FEATURE_MAC;
    if offered & required != required || offered & ACCESS_PLATFORM != 0 {
        return Err(Error::Unsupported);
    }
    Ok(required | offered & FEATURE_STATUS)
}

/// Single-head queue completion, with wrapping 16-bit indices. A TX descriptor
/// has no writable bytes, so its used length must be zero. RX includes the full
/// modern virtio header; all copying waits until this validation succeeds.
pub fn validate_used(
    receive: bool,
    old: u16,
    next: u16,
    id: u32,
    length: u32,
) -> Result<usize, Error> {
    let delta = next.wrapping_sub(old);
    if delta == 0 {
        return Err(Error::Pending);
    }
    if delta != 1 || id != 0 {
        return Err(Error::Protocol);
    }
    if receive {
        if !(HEADER_BYTES as u32 + 14..=BUFFER_BYTES as u32).contains(&length) {
            return Err(Error::Protocol);
        }
        Ok(length as usize - HEADER_BYTES)
    } else if length == 0 {
        Ok(0)
    } else {
        Err(Error::Protocol)
    }
}

/// DATA_VALID is only a hint; upper layers still validate checksums themselves.
/// NEEDS_CSUM/GSO need offloads which were not negotiated. In this exact QEMU
/// profile num_buffers is 0/1, never a reason to read another descriptor.
pub fn validate_rx_header(header: &[u8; HEADER_BYTES]) -> Result<(), Error> {
    if header[0] & !2 != 0
        || header[1..10].iter().any(|&byte| byte != 0)
        || u16::from_le_bytes([header[10], header[11]]) > 1
    {
        return Err(Error::Protocol);
    }
    Ok(())
}

// Private interface: callers cannot use Hardware as an arbitrary safe MMIO/DMA
// accessor. Its only constructor is the unsafe, validated Device initializer.
trait Io {
    fn read8(&self, address: u64) -> u8;
    fn read16(&self, address: u64) -> u16;
    fn read32(&self, address: u64) -> u32;
    fn write8(&mut self, address: u64, value: u8);
    fn write16(&mut self, address: u64, value: u16);
    fn write32(&mut self, address: u64, value: u32);
    fn write64(&mut self, address: u64, value: u64);
    fn bus_master(&mut self, enabled: bool);
    fn bus_master_enabled(&self) -> bool;
}

#[derive(Clone, Copy, Default)]
struct QueueState {
    available: u16,
    used: u16,
    posted: bool,
    notify: u64,
}

struct Engine<T: Io> {
    io: T,
    info: Discovery,
    common: u64,
    device: u64,
    mapped: [u64; 6],
    pool: NetPool,
    queues: [QueueState; 2],
    features: u64,
    mac: [u8; 6],
    ready: bool,
    tx_deadline: u64,
    tx_length: usize,
}

impl<T: Io> Engine<T> {
    fn new(info: Discovery, mapped: [u64; 6], pool: NetPool, io: T) -> Result<Self, Error> {
        pool.validate(RAM_END)?;
        if parse_capabilities_for(&info.config, &info.bars, DEVICE_ID_NET)? != info.regions {
            return Err(Error::Protocol);
        }
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
        if !common.is_multiple_of(8) || !device.is_multiple_of(8) {
            return Err(Error::Range);
        }
        Ok(Self {
            io,
            info,
            common,
            device,
            mapped,
            pool,
            queues: [QueueState::default(); 2],
            features: 0,
            mac: [0; 6],
            ready: false,
            tx_deadline: 0,
            tx_length: 0,
        })
    }

    fn initialize(&mut self) -> Result<(), Error> {
        let c = self.common;
        self.io.bus_master(false);
        self.io.write8(c + 20, 0);
        if self.io.read8(c + 20) != 0 {
            return Err(Error::Timeout);
        }
        self.io.write8(c + 20, 1);
        self.io.write8(c + 20, 3);
        self.io.write32(c, 0);
        let low = self.io.read32(c + 4);
        self.io.write32(c, 1);
        let high = self.io.read32(c + 4);
        self.features = negotiate_features(u64::from(low) | u64::from(high) << 32)?;
        self.io.write32(c + 8, 0);
        self.io.write32(c + 12, self.features as u32);
        self.io.write32(c + 8, 1);
        self.io.write32(c + 12, (self.features >> 32) as u32);
        self.io.write8(c + 20, 11);
        if self.io.read8(c + 20) & 8 == 0 {
            return Err(Error::Unsupported);
        }
        if self.io.read16(c + 18) < 2 {
            return Err(Error::Protocol);
        }
        self.mac = self.read_mac()?;
        if self.mac == [0; 6] || self.mac[0] & 1 != 0 {
            return Err(Error::Protocol);
        }
        self.io.write16(c + 16, 0xffff); // no config MSI-X
        for index in 0..2 {
            let memory = self.pool.queue(index);
            self.io.write16(c + 22, index as u16);
            if self.io.read16(c + 28) != 0 || self.io.read16(c + 24) < QUEUE_SIZE {
                return Err(Error::Unsupported);
            }
            self.io.write16(c + 24, QUEUE_SIZE);
            self.io.write16(c + 26, 0xffff); // no queue MSI-X
            for page in [memory.ring, memory.bounce] {
                for offset in 0..4096 {
                    self.io.write8(page + offset, 0);
                }
            }
            self.io.write16(memory.available(), 1); // no interrupts
            self.io.write64(c + 32, memory.ring);
            self.io.write64(c + 40, memory.available());
            self.io.write64(c + 48, memory.used());
            let offset = self.io.read16(c + 30);
            let region = self.info.regions.notify;
            let relative = self.info.regions.notify_address(&self.info.bars, offset)?
                - self.info.bars[region.bar as usize].base;
            let notify = self.mapped[region.bar as usize]
                .checked_add(relative)
                .ok_or(Error::Range)?;
            if !notify.is_multiple_of(2) {
                return Err(Error::Range);
            }
            self.queues[index].notify = notify;
            self.io.write16(c + 28, 1);
            if self.io.read16(c + 28) != 1 {
                return Err(Error::Protocol);
            }
        }
        fence(Ordering::SeqCst);
        self.io.bus_master(true);
        self.io.write8(c + 20, 15);
        if self.io.read8(c + 20) != 15 {
            return Err(Error::Protocol);
        }
        self.ready = true;
        Ok(())
    }

    fn read_mac(&self) -> Result<[u8; 6], Error> {
        for _ in 0..16 {
            let generation = self.io.read8(self.common + 21);
            let mut mac = [0; 6];
            for (index, byte) in mac.iter_mut().enumerate() {
                *byte = self.io.read8(self.device + index as u64);
            }
            if generation == self.io.read8(self.common + 21) {
                return Ok(mac);
            }
        }
        Err(Error::Busy)
    }

    fn link_up(&self) -> Result<bool, Error> {
        if !self.ready {
            return Err(Error::Offline);
        }
        if self.io.read8(self.common + 20) != 15 {
            return Err(Error::Io);
        }
        if self.features & FEATURE_STATUS == 0 {
            return Err(Error::Unsupported);
        }
        for _ in 0..16 {
            let generation = self.io.read8(self.common + 21);
            let status = self.io.read16(self.device + 6);
            if generation == self.io.read8(self.common + 21) {
                return Ok(status & 1 != 0);
            }
        }
        Err(Error::Busy)
    }

    fn publish(&mut self, index: usize, length: usize, receive: bool) {
        let memory = self.pool.queue(index);
        let descriptor = Descriptor {
            address: memory.bounce,
            length: length as u32,
            flags: if receive { WRITE } else { 0 },
            next: 0,
        };
        for (offset, byte) in descriptor.encode().into_iter().enumerate() {
            self.io.write8(memory.ring + offset as u64, byte);
        }
        let queue = &mut self.queues[index];
        self.io.write16(
            memory.available() + 4 + u64::from(queue.available % QUEUE_SIZE) * 2,
            0,
        );
        queue.posted = true;
        fence(Ordering::SeqCst);
        queue.available = queue.available.wrapping_add(1);
        self.io.write16(memory.available() + 2, queue.available);
        fence(Ordering::SeqCst);
        self.io.write16(queue.notify, index as u16);
    }

    fn post_rx(&mut self) -> Result<(), Error> {
        self.prepare(RX)?;
        for offset in 0..BUFFER_BYTES {
            self.io.write8(self.pool.rx_bounce + offset as u64, 0);
        }
        self.publish(RX, BUFFER_BYTES, true);
        Ok(())
    }

    fn send(&mut self, frame: &[u8], now: u64, timeout: u64) -> Result<(), Error> {
        self.prepare(TX)?;
        if !(14..=MAX_FRAME_BYTES).contains(&frame.len()) || timeout == 0 {
            return Err(Error::Range);
        }
        let deadline = now.checked_add(timeout).ok_or(Error::Range)?;
        let length = frame.len().max(60);
        // No header offloads; padding is zeroed and never leaks stale frames.
        for offset in 0..HEADER_BYTES + length {
            self.io.write8(self.pool.tx_bounce + offset as u64, 0);
        }
        for (offset, byte) in frame.iter().copied().enumerate() {
            self.io.write8(
                self.pool.tx_bounce + HEADER_BYTES as u64 + offset as u64,
                byte,
            );
        }
        self.tx_deadline = deadline;
        self.tx_length = length;
        self.publish(TX, HEADER_BYTES + length, false);
        Ok(())
    }

    fn prepare(&mut self, index: usize) -> Result<(), Error> {
        self.health()?;
        if self.queues[index].posted {
            return Err(Error::Busy);
        }
        // An unsolicited/stale used entry must not be attributed to the next
        // request after we publish it. Check before touching its bounce page.
        if self.io.read16(self.pool.queue(index).used() + 2) != self.queues[index].used {
            self.begin_reset();
            return Err(Error::Protocol);
        }
        Ok(())
    }

    fn completion(&mut self, index: usize) -> Result<Option<(u16, u32)>, Error> {
        self.health()?;
        let queue = self.queues[index];
        if !queue.posted {
            return Ok(None);
        }
        let memory = self.pool.queue(index);
        let next = self.io.read16(memory.used() + 2);
        if next == queue.used {
            return Ok(None);
        }
        fence(Ordering::SeqCst);
        let at = memory.used() + 4 + u64::from(queue.used % QUEUE_SIZE) * 8;
        let id = self.io.read32(at);
        let length = self.io.read32(at + 4);
        if let Err(error) = validate_used(index == RX, queue.used, next, id, length) {
            self.begin_reset();
            return Err(error);
        }
        Ok(Some((next, length)))
    }

    fn health(&mut self) -> Result<(), Error> {
        if !self.ready {
            return Err(Error::Offline);
        }
        if self.io.read8(self.common + 20) != 15 {
            self.begin_reset();
            return Err(Error::Io);
        }
        Ok(())
    }

    fn poll_tx(&mut self, now: u64) -> Result<Option<usize>, Error> {
        if !self.ready {
            return Err(Error::Offline);
        }
        if self.queues[TX].posted && now >= self.tx_deadline {
            self.begin_reset();
            return Err(Error::Timeout);
        }
        let Some((next, _)) = self.completion(TX)? else {
            return Ok(None);
        };
        self.queues[TX].used = next;
        self.queues[TX].posted = false;
        Ok(Some(self.tx_length))
    }

    fn poll_rx(&mut self, output: &mut [u8]) -> Result<Option<usize>, Error> {
        let Some((next, length)) = self.completion(RX)? else {
            return Ok(None);
        };
        let mut header = [0; HEADER_BYTES];
        for (offset, byte) in header.iter_mut().enumerate() {
            *byte = self.io.read8(self.pool.rx_bounce + offset as u64);
        }
        if let Err(error) = validate_rx_header(&header) {
            self.begin_reset();
            return Err(error);
        }
        let size = length as usize - HEADER_BYTES;
        if output.len() < size {
            return Err(Error::Range); // keep completion until a large enough buffer
        }
        for (offset, byte) in output.iter_mut().enumerate().take(size) {
            *byte = self
                .io
                .read8(self.pool.rx_bounce + HEADER_BYTES as u64 + offset as u64);
        }
        self.queues[RX].used = next;
        self.queues[RX].posted = false;
        Ok(Some(size))
    }

    /// Fault injection only: reset the actual device without changing the
    /// driver's ready/posted state. Ordinary health checks must detect it.
    #[cfg(any(test, feature = "test-device-reset"))]
    fn inject_device_reset(&mut self) -> Result<(u8, u8, bool, bool), Error> {
        let before = self.io.read8(self.common + 20);
        if !self.ready || !self.queues[TX].posted || before != 15 {
            return Err(Error::Io);
        }
        let master_before = self.io.bus_master_enabled();
        self.io.write8(self.common + 20, 0);
        Ok((
            before,
            self.io.read8(self.common + 20),
            master_before,
            self.io.bus_master_enabled(),
        ))
    }

    fn begin_reset(&mut self) {
        self.ready = false;
        self.io.write8(self.common + 20, 0);
        self.io.bus_master(false);
        // DMA pages are never freed here; a reset request alone is not an ACK.
        for queue in &mut self.queues {
            queue.posted = false;
        }
    }

    fn reset_acknowledged(&self) -> bool {
        !self.ready && self.io.read8(self.common + 20) == 0 && !self.io.bus_master_enabled()
    }
}

pub struct Device {
    engine: Engine<Hardware>,
}

impl Device {
    /// # Safety
    /// The caller supplies permanent, supervisor-only UC mappings for the
    /// validated BARs and four exclusive, identity-mapped, pinned normal-RAM
    /// pages. Neither firmware, another driver nor a process may access them.
    /// Pages remain pinned until reboot even on failure, and can be reused for
    /// reinitialization only after an observed reset ACK. One CPU/serialized I/O.
    pub unsafe fn initialize(
        info: Discovery,
        mapped: [u64; 6],
        pool: NetPool,
    ) -> Result<Self, Error> {
        let mut engine = Engine::new(info, mapped, pool, Hardware)?;
        if let Err(error) = engine.initialize() {
            engine.begin_reset();
            return Err(error);
        }
        Ok(Self { engine })
    }

    pub fn mac(&self) -> [u8; 6] {
        self.engine.mac
    }
    pub fn link_up(&self) -> Result<bool, Error> {
        self.engine.link_up()
    }
    pub fn post_rx(&mut self) -> Result<(), Error> {
        self.engine.post_rx()
    }
    pub fn poll_rx(&mut self, output: &mut [u8]) -> Result<Option<usize>, Error> {
        self.engine.poll_rx(output)
    }
    /// Queue publication only; does not mean device completion or peer delivery.
    pub fn send(&mut self, frame: &[u8], now: u64, timeout: u64) -> Result<(), Error> {
        self.engine.send(frame, now, timeout)
    }
    /// Device completion returns the padded frame byte count, not a peer ACK.
    pub fn poll_tx(&mut self, now: u64) -> Result<Option<usize>, Error> {
        self.engine.poll_tx(now)
    }
    pub fn begin_reset(&mut self) {
        self.engine.begin_reset();
    }
    /// Native fault fixture only; unavailable in ordinary driver builds.
    #[cfg(feature = "test-device-reset")]
    pub fn inject_device_reset(&mut self) -> Result<(u8, u8, bool, bool), Error> {
        self.engine.inject_device_reset()
    }
    pub fn reset_acknowledged(&self) -> bool {
        self.engine.reset_acknowledged()
    }
    pub fn tx_pending(&self) -> bool {
        self.engine.queues[TX].posted
    }
    pub fn rx_posted(&self) -> bool {
        self.engine.queues[RX].posted
    }
}

#[cfg(test)]
mod tests;
