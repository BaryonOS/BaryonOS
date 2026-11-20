//! H03-T3a: kernel-only virtio-snd PCM output fixture. QEMU 8.2
//! `virtio-sound-pci` at the fixture address 00:08.0 only; IF=0, one CPU,
//! polling with TSC deadlines, no interrupt of any kind.
//!
//! The kernel itself generates a fixed stereo S16 48 kHz pattern and plays it
//! through the device; the host compares the WAV audiodev output exactly. There
//! is no syscall, ring-3 path, broker or capability (owner decisions B1/B2 stay
//! open), and no microphone path: the capture queue is never enabled.
//!
//! Trust model (`drivers/virtio-snd/TRANSPORT.md` §2): trusted QEMU kernel
//! transport without an IOMMU. Descriptors point only into six Kernel-owned,
//! identity-mapped RW/NX pages that stay pinned until VM exit, also after a
//! fence or quarantine; that bounds what this driver asks for, it is not DMA
//! isolation. Device-writable data is limited to the control response, event
//! buffers and TX status words; PCM is device-readable only.
use super::{paging, zero, FRAME_COUNT};
use crate::serial;
use core::{
    arch::asm,
    hint::spin_loop,
    sync::atomic::{fence, Ordering},
};
use kernel_core::{Frames, Owner, PAGE};
use nieuw_os_virtio_snd::{
    negotiate_features, pcm,
    queue::{
        self, Chain, FenceStep, Pattern, Phase, Pool, Ring, ACTIVE, AVAIL_NO_INTERRUPT,
        EVENT_BUFFERS, FIXTURE_PERIODS, FIXTURE_PERIOD_BYTES, FIXTURE_PERIOD_FRAMES,
        FIXTURE_TOTAL_PERIODS, QUEUES, QUEUE_SIZE, RXQ, STATUS_ACKNOWLEDGE, STATUS_DRIVER,
        STATUS_DRIVER_OK, STATUS_FAILED, STATUS_FEATURES_OK, STATUS_LIVE, STATUS_NEEDS_RESET,
        TX_SLOTS,
    },
    Config, Error as CodecError, MAX_RESPONSE_BYTES, VIRTIO_SND_PCI_DEVICE_ID,
};
use pci_core::{Mechanism1, PciAddress, PciConfigAccess};
use virtio_core::{parse_capabilities_for, Bar, Region, Regions};

/// IF0 fixture MMIO window (`if0-device-window`); xHCI is excluded here.
const WINDOW: u64 = 0x3f200000;
const WINDOW_PAGES: u64 = 64;
/// Provisional fixture address (B4); not a VM-FR inventory reservation.
const FIXTURE: PciAddress = PciAddress {
    segment: 0,
    bus: 0,
    device: 8,
    function: 0,
};
const VIRTIO_VENDOR: u16 = 0x1af4;
const IO: u16 = 1;
const MEMORY: u16 = 2;
const BUS_MASTER: u16 = 4;
const INT_DISABLE: u16 = 1 << 10;
const NO_VECTOR: u16 = 0xffff;
// struct virtio_pci_common_cfg (§4.1.4.3).
const DEVICE_FEATURE_SELECT: u32 = 0x00;
const DEVICE_FEATURE: u32 = 0x04;
const DRIVER_FEATURE_SELECT: u32 = 0x08;
const DRIVER_FEATURE: u32 = 0x0c;
const CONFIG_MSIX_VECTOR: u32 = 0x10;
const NUM_QUEUES: u32 = 0x12;
const DEVICE_STATUS: u32 = 0x14;
const CONFIG_GENERATION: u32 = 0x15;
const QUEUE_SELECT: u32 = 0x16;
const QUEUE_SIZE_REG: u32 = 0x18;
const QUEUE_MSIX_VECTOR: u32 = 0x1a;
const QUEUE_ENABLE: u32 = 0x1c;
const QUEUE_NOTIFY_OFF: u32 = 0x1e;
const QUEUE_DESC: u32 = 0x20;
const QUEUE_DRIVER: u32 = 0x28;
const QUEUE_DEVICE: u32 = 0x30;
// Ring indices into `ACTIVE`.
const CONTROL: usize = 0;
const EVENTS: usize = 1;
const TX: usize = 2;
// Deadlines in milliseconds of the PIT-calibrated TSC (virtual time under
// `-icount`). The drain lets QEMU's audio timer write the accepted samples.
const RESET_MS: u64 = 100;
const CONTROL_MS: u64 = 200;
const TX_PROGRESS_MS: u64 = 500;
const DRAIN_MS: u64 = 200;
const FENCE_MS: u64 = 100;
const STABLE_MS: u64 = 50;
/// Fence and quarantine fixtures interrupt pattern A after this many periods.
const FENCE_AFTER: u32 = 10;
const LEAF_MASK: u64 = 0x000f_ffff_ffff_f000 | 1 | 2 | 4 | (1 << 63);

#[derive(Clone, Copy, Debug)]
enum Fault {
    Pci,
    Profile,
    Mapping,
    Pit,
    Timeout(&'static [u8]),
    Status(u8),
    State,
    Queue(queue::Error),
    Pcm(pcm::Error),
    Codec,
    TxIo,
    Quarantined,
}

impl Fault {
    fn code(self) -> &'static [u8] {
        match self {
            Self::Pci => b"PCI",
            Self::Profile => b"PROFILE",
            Self::Mapping => b"MAPPING",
            Self::Pit => b"PIT",
            Self::Timeout(what) => what,
            Self::Status(_) => b"DEVICE_STATUS",
            Self::State => b"STATE",
            Self::Queue(error) if error.requires_fence() => b"USED_RING_VIOLATION",
            Self::Queue(_) => b"QUEUE",
            Self::Pcm(error) if error.requires_fence() => b"PCM_VIOLATION",
            Self::Pcm(_) => b"PCM",
            Self::Codec => b"CODEC",
            Self::TxIo => b"TX_IO_ERROR",
            Self::Quarantined => b"QUARANTINED",
        }
    }
}

impl From<queue::Error> for Fault {
    fn from(error: queue::Error) -> Self {
        Self::Queue(error)
    }
}
impl From<pcm::Error> for Fault {
    fn from(error: pcm::Error) -> Self {
        Self::Pcm(error)
    }
}
impl From<CodecError> for Fault {
    fn from(_: CodecError) -> Self {
        Self::Codec
    }
}

fn decimal(label: &[u8], value: u64, suffix: &[u8]) {
    let mut digits = [0u8; 20];
    let mut index = digits.len();
    let mut rest = value;
    loop {
        index -= 1;
        digits[index] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    serial::write(label);
    serial::write(&digits[index..]);
    serial::write(suffix);
}

fn hex_byte(label: &[u8], value: u8) {
    let digit = |nibble: u8| {
        if nibble < 10 {
            b'0' + nibble
        } else {
            b'a' + nibble - 10
        }
    };
    serial::write(label);
    serial::write(&[digit(value >> 4), digit(value & 15), b'\n']);
}

fn rdtsc() -> u64 {
    let (low, high): (u32, u32);
    // SAFETY: RDTSC only reads the time-stamp counter into EDX:EAX.
    unsafe { asm!("rdtsc", out("eax") low, out("edx") high, options(nomem, nostack)) };
    u64::from(high) << 32 | u64::from(low)
}

fn inb(port: u16) -> u8 {
    let value: u8;
    // SAFETY: CPL0 IF=0 access to the PIT/port-B timer ports only; this profile
    // has no other PIT user (native-smp is excluded by compile_error).
    unsafe { asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack)) };
    value
}

fn outb(port: u16, value: u8) {
    // SAFETY: Same exclusive PIT/port-B contract as `inb`.
    unsafe { asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack)) };
}

/// One-shot PIT channel 2 wait (local copy of the native-smp calibration).
/// A spin bound turns a missing PIT into a failure instead of a hang.
fn wait_us(us: u32) -> bool {
    let ticks = (u64::from(us) * 1_193_182)
        .div_ceil(1_000_000)
        .clamp(2, 0xffff) as u16;
    let control = inb(0x61) & 0x0c;
    outb(0x61, control);
    outb(0x43, 0xb0);
    outb(0x42, ticks as u8);
    outb(0x42, (ticks >> 8) as u8);
    if inb(0x61) & 0x20 != 0 {
        return false;
    }
    outb(0x61, control | 0x01);
    let expired = (0..20_000_000u32).any(|_| {
        spin_loop();
        inb(0x61) & 0x20 != 0
    });
    outb(0x61, control);
    expired
}

#[derive(Clone, Copy)]
struct Clock {
    per_ms: u64,
}

impl Clock {
    /// Minimum of ten 10 ms PIT samples: delays only lengthen a sample.
    fn calibrate() -> Result<Self, Fault> {
        let mut per_ms = u64::MAX;
        for _ in 0..10 {
            let start = rdtsc();
            if !wait_us(10_000) {
                return Err(Fault::Pit);
            }
            per_ms = per_ms.min(rdtsc().wrapping_sub(start) / 10);
        }
        Ok(Self {
            per_ms: per_ms.max(1),
        })
    }

    fn after(self, ms: u64) -> u64 {
        rdtsc().saturating_add(ms.saturating_mul(self.per_ms))
    }
}

fn past(deadline: u64) -> bool {
    rdtsc() >= deadline
}

/// One validated virtio capability region inside the mapped fixture BAR.
#[derive(Clone, Copy)]
struct Window {
    base: u64,
    length: u32,
}

impl Window {
    fn at(self, offset: u32, width: u32) -> u64 {
        assert!(
            offset.is_multiple_of(width)
                && offset
                    .checked_add(width)
                    .is_some_and(|end| end <= self.length),
            "virtio-snd MMIO access outside its capability region"
        );
        self.base + u64::from(offset)
    }
    fn r8(self, offset: u32) -> u8 {
        let address = self.at(offset, 1);
        // SAFETY: `address` lies inside a validated capability region of the
        // fixture BAR, mapped supervisor UC/NX in the audited IF0 window; the
        // access is bounded and naturally aligned. No Rust reference exists.
        unsafe { (address as *const u8).read_volatile() }
    }
    fn r16(self, offset: u32) -> u16 {
        let address = self.at(offset, 2);
        // SAFETY: As `r8`; aligned 16-bit register.
        unsafe { (address as *const u16).read_volatile() }
    }
    fn r32(self, offset: u32) -> u32 {
        let address = self.at(offset, 4);
        // SAFETY: As `r8`; aligned 32-bit register.
        unsafe { (address as *const u32).read_volatile() }
    }
    fn w8(self, offset: u32, value: u8) {
        let address = self.at(offset, 1);
        // SAFETY: As `r8`; the kernel is the only writer of this function.
        unsafe { (address as *mut u8).write_volatile(value) }
    }
    fn w16(self, offset: u32, value: u16) {
        let address = self.at(offset, 2);
        // SAFETY: As `w8`; aligned 16-bit register.
        unsafe { (address as *mut u16).write_volatile(value) }
    }
    fn w32(self, offset: u32, value: u32) {
        let address = self.at(offset, 4);
        // SAFETY: As `w8`; aligned 32-bit register.
        unsafe { (address as *mut u32).write_volatile(value) }
    }
}

/// Volatile access to the pinned identity-mapped DMA pool only.
#[derive(Clone, Copy)]
struct Dma {
    pages: [u64; Pool::PAGES],
}

impl Dma {
    fn check(self, address: u64, width: u64) -> u64 {
        assert!(
            address.is_multiple_of(width)
                && self
                    .pages
                    .iter()
                    .any(|page| *page <= address && address + width <= *page + PAGE),
            "virtio-snd DMA access outside the pinned pool"
        );
        address
    }
    fn r8(self, address: u64) -> u8 {
        let address = self.check(address, 1);
        // SAFETY: Kernel-owned pool page, identity-mapped RW/NX and pinned for
        // the boot; the device may write it concurrently, so only volatile
        // integer accesses are used and no Rust reference is formed.
        unsafe { (address as *const u8).read_volatile() }
    }
    fn r16(self, address: u64) -> u16 {
        let address = self.check(address, 2);
        // SAFETY: As `r8`; aligned.
        unsafe { (address as *const u16).read_volatile() }
    }
    fn r32(self, address: u64) -> u32 {
        let address = self.check(address, 4);
        // SAFETY: As `r8`; aligned.
        unsafe { (address as *const u32).read_volatile() }
    }
    fn w8(self, address: u64, value: u8) {
        let address = self.check(address, 1);
        // SAFETY: As `r8`; only this single-CPU driver writes the pool.
        unsafe { (address as *mut u8).write_volatile(value) }
    }
    fn w16(self, address: u64, value: u16) {
        let address = self.check(address, 2);
        // SAFETY: As `w8`; aligned.
        unsafe { (address as *mut u16).write_volatile(value) }
    }
    fn w32(self, address: u64, value: u32) {
        let address = self.check(address, 4);
        // SAFETY: As `w8`; aligned.
        unsafe { (address as *mut u32).write_volatile(value) }
    }
    fn w64(self, address: u64, value: u64) {
        let address = self.check(address, 8);
        // SAFETY: As `w8`; aligned.
        unsafe { (address as *mut u64).write_volatile(value) }
    }
}

/// Configuration access bound to the fixture function.
struct Pci(Mechanism1);

impl Pci {
    fn read(&mut self, offset: u8) -> Result<u32, Fault> {
        self.0.read_u32(FIXTURE, offset).map_err(|_| Fault::Pci)
    }
    fn write(&mut self, offset: u8, value: u32) -> Result<(), Fault> {
        self.0
            .write_u32(FIXTURE, offset, value)
            .map_err(|_| Fault::Pci)
    }
    fn command(&mut self) -> Result<u16, Fault> {
        Ok(self.read(4)? as u16)
    }
    /// 16-bit write (keeps the W1C status bits) plus exact read-back.
    fn set_command(&mut self, value: u16) -> Result<(), Fault> {
        self.0
            .write_u16(FIXTURE, 4, value)
            .map_err(|_| Fault::Pci)?;
        if self.command()? & (IO | MEMORY | BUS_MASTER | INT_DISABLE) != value {
            return Err(Fault::Pci);
        }
        Ok(())
    }
    fn bus_master(&mut self) -> bool {
        self.command()
            .map_or(true, |command| command & BUS_MASTER != 0)
    }
    /// MSI enable off and MSI-X off plus function mask, each read back. INTx is
    /// already disabled by command bit 10; IF stays 0 throughout.
    fn mask_interrupts(&mut self) -> Result<(), Fault> {
        if self.read(4)? & (1 << 20) == 0 {
            return Ok(());
        }
        let mut next = (self.read(0x34)? & 0xff) as u8;
        let mut visited = [false; 64];
        for _ in 0..48 {
            if next == 0 {
                return Ok(());
            }
            if next < 0x40 || next & 3 != 0 || visited[usize::from(next / 4)] {
                return Err(Fault::Profile);
            }
            visited[usize::from(next / 4)] = true;
            let cap = self.read(next)?;
            let control = (cap >> 16) as u16;
            let wanted = match cap as u8 {
                5 => Some(control & !1),
                0x11 => Some((control & !(1 << 15)) | (1 << 14)),
                _ => None,
            };
            if let Some(value) = wanted {
                self.0
                    .write_u16(FIXTURE, next + 2, value)
                    .map_err(|_| Fault::Pci)?;
                if (self.read(next)? >> 16) as u16 != value {
                    return Err(Fault::Pci);
                }
            }
            next = (cap >> 8) as u8;
        }
        Err(Fault::Profile)
    }
    /// Sizes the memory BARs with decoding disabled; every BAR is restored
    /// and read back. I/O BARs are never used.
    fn size_bars(&mut self) -> Result<[Bar; 6], Fault> {
        let mut bars = [Bar::NONE; 6];
        let mut index = 0u8;
        while index < 6 {
            let offset = 0x10 + index * 4;
            let low = self.read(offset)?;
            if low & 1 != 0 {
                index += 1;
                continue;
            }
            let wide = low & 6 == 4;
            if low & 6 != 0 && !wide || wide && index == 5 {
                return Err(Fault::Profile);
            }
            let high = if wide { self.read(offset + 4)? } else { 0 };
            self.write(offset, u32::MAX)?;
            if wide {
                self.write(offset + 4, u32::MAX)?;
            }
            let mask_low = self.read(offset)?;
            let mask_high = if wide {
                self.read(offset + 4)?
            } else {
                u32::MAX
            };
            self.write(offset, low)?;
            if wide {
                self.write(offset + 4, high)?;
            }
            if self.read(offset)? != low || wide && self.read(offset + 4)? != high {
                return Err(Fault::Pci);
            }
            let base = u64::from(low & !15) | u64::from(high) << 32;
            let mask = u64::from(mask_low & !15) | u64::from(mask_high) << 32;
            let size = (!mask).wrapping_add(1);
            if base != 0 && mask_low & !15 != 0 {
                if !size.is_power_of_two() || size < PAGE || base & (size - 1) != 0 {
                    return Err(Fault::Profile);
                }
                bars[usize::from(index)] = Bar {
                    base,
                    size,
                    is_memory: true,
                };
            }
            index += if wide { 2 } else { 1 };
        }
        Ok(bars)
    }
}

/// Before `map_tables`: six Kernel-owned frames, zeroed through the firmware
/// identity map and mapped identity RW/NX. They are never freed.
pub fn prepare(frames: &mut Frames<FRAME_COUNT>, paging: &mut paging::Paging) -> Pool {
    let mut pages = [0u64; Pool::PAGES];
    for page in &mut pages {
        *page = frames
            .allocate(Owner::Kernel, zero)
            .unwrap_or_else(|_| serial::fail(b"SND_POOL"));
        paging.map(frames, *page, *page, true, false);
    }
    Pool::new(pages, PAGE * FRAME_COUNT as u64).unwrap_or_else(|_| serial::fail(b"SND_POOL"))
}

enum Located {
    Fixture,
    Absent,
    Elsewhere,
}

fn locate(pci: &mut Pci) -> Located {
    let mut found = 0u32;
    let mut at_fixture = false;
    pci_core::enumerate(&mut pci.0, |device| {
        if device.vendor_id == VIRTIO_VENDOR && device.device_id == VIRTIO_SND_PCI_DEVICE_ID {
            found += 1;
            at_fixture |= device.address == FIXTURE;
        }
    })
    .unwrap_or_else(|_| serial::fail(b"SND_PCI_SCAN"));
    match (found, at_fixture) {
        (0, _) => Located::Absent,
        (1, true) => Located::Fixture,
        _ => Located::Elsewhere,
    }
}

struct Device {
    pci: Pci,
    common: Window,
    device_config: Window,
    notify_window: Window,
    notify: [u32; 3],
    regions: Regions,
    bars: [Bar; 6],
    bar_base: u64,
    pool: Pool,
    dma: Dma,
    rings: [Ring; 3],
    life: queue::Lifecycle,
    config: Option<Config>,
    clock: Clock,
    budget: pcm::EventBudget,
}

impl Device {
    fn attach(
        mut pci: Pci,
        frames: &mut Frames<FRAME_COUNT>,
        paging: &mut paging::Paging,
        pool: Pool,
        clock: Clock,
    ) -> Result<Self, Fault> {
        // Decoding, DMA and INTx off before anything is sized or mapped.
        pci.set_command(INT_DISABLE)?;
        pci.mask_interrupts()?;
        let bars = pci.size_bars()?;
        let mut config = [0u8; 256];
        for offset in (0..256usize).step_by(4) {
            config[offset..offset + 4].copy_from_slice(&pci.read(offset as u8)?.to_le_bytes());
        }
        let regions = parse_capabilities_for(&config, &bars, VIRTIO_SND_PCI_DEVICE_ID)
            .map_err(|_| Fault::Profile)?;
        let index = regions.common.bar;
        let bar = bars[usize::from(index)];
        if [regions.notify, regions.isr, regions.device]
            .iter()
            .any(|region| region.bar != index)
            || !bar.is_memory
            || bar.base < 0x8000_0000
            || bar.size > WINDOW_PAGES * PAGE
            || regions.device.length < queue::CONFIG_BYTES as u32
        {
            return Err(Fault::Profile);
        }
        serial::hex(b"SND:BAR_BASE:", bar.base);
        serial::hex(b"SND:BAR_SIZE:", bar.size);
        // Only the pages of the three regions the driver uses; ISR stays
        // unmapped (polling never reads it).
        let mut mapped = [false; WINDOW_PAGES as usize];
        let mut count = 0;
        for region in [regions.common, regions.notify, regions.device] {
            let first = u64::from(region.offset) & !(PAGE - 1);
            let end = (u64::from(region.offset) + u64::from(region.length)).next_multiple_of(PAGE);
            for page in (first..end).step_by(PAGE as usize) {
                let slot = (page / PAGE) as usize;
                if !mapped[slot] {
                    paging
                        .map_native_device(frames, WINDOW + page, bar.base + page)
                        .map_err(|_| Fault::Mapping)?;
                    mapped[slot] = true;
                    count += 1;
                }
            }
        }
        decimal(b"SND:MMIO_PAGES:", count, b"\n");
        pci.set_command(MEMORY | INT_DISABLE)?;
        let window = |region: Region| Window {
            base: WINDOW + u64::from(region.offset),
            length: region.length,
        };
        Ok(Self {
            pci,
            common: window(regions.common),
            device_config: window(regions.device),
            notify_window: window(regions.notify),
            notify: [0; 3],
            regions,
            bars,
            bar_base: bar.base,
            pool,
            dma: Dma {
                pages: pool.pages(),
            },
            rings: [Ring::new(); 3],
            life: queue::Lifecycle::new(),
            config: None,
            clock,
            budget: pcm::EventBudget::new(),
        })
    }

    fn status(&self) -> u8 {
        self.common.r8(DEVICE_STATUS)
    }

    fn set_status(&self, value: u8) -> Result<(), Fault> {
        self.common.w8(DEVICE_STATUS, value);
        let status = self.status();
        if status != value {
            return Err(Fault::Status(status));
        }
        Ok(())
    }

    fn select(&self, queue: u16) -> Result<(), Fault> {
        self.common.w16(QUEUE_SELECT, queue);
        if self.common.r16(QUEUE_SELECT) != queue {
            return Err(Fault::Profile);
        }
        Ok(())
    }

    fn write_address(&self, register: u32, address: u64) -> Result<(), Fault> {
        self.common.w32(register, address as u32);
        self.common.w32(register + 4, (address >> 32) as u32);
        let read =
            u64::from(self.common.r32(register)) | u64::from(self.common.r32(register + 4)) << 32;
        if read != address {
            return Err(Fault::Profile);
        }
        Ok(())
    }

    /// Zeroes every pool page. Only called with the device reset (status 0
    /// read back) and bus mastering off, so no DMA can race the clearing.
    fn clear_pool(&self) {
        for page in self.pool.pages() {
            for offset in (0..PAGE).step_by(8) {
                self.dma.w64(page + offset, 0);
            }
        }
        fence(Ordering::SeqCst);
    }

    /// §3.1.1 and §5.14.5. `Ok(false)`: the device offered a feature set the
    /// policy refuses (F2); it stays offline without bus mastering.
    fn initialize(&mut self) -> Result<bool, Fault> {
        if self.life.phase() != Phase::Offline {
            return Err(Fault::State);
        }
        self.pci.set_command(MEMORY | INT_DISABLE)?;
        self.common.w8(DEVICE_STATUS, 0);
        let deadline = self.clock.after(RESET_MS);
        while self.status() != 0 {
            if past(deadline) {
                return Err(Fault::Timeout(b"RESET"));
            }
            spin_loop();
        }
        self.clear_pool();
        self.rings = [Ring::new(); 3];
        self.notify = [0; 3];
        self.config = None;
        self.set_status(STATUS_ACKNOWLEDGE)?;
        self.set_status(STATUS_ACKNOWLEDGE | STATUS_DRIVER)?;
        self.common.w32(DEVICE_FEATURE_SELECT, 0);
        let low = self.common.r32(DEVICE_FEATURE);
        self.common.w32(DEVICE_FEATURE_SELECT, 1);
        let high = self.common.r32(DEVICE_FEATURE);
        let offered = u64::from(low) | u64::from(high) << 32;
        serial::hex(b"SND:FEATURES:", offered);
        let Ok(negotiated) = negotiate_features(offered) else {
            self.refuse_features()?;
            return Ok(false);
        };
        serial::hex(b"SND:DRIVER_FEATURES:", negotiated.bits());
        self.common.w32(DRIVER_FEATURE_SELECT, 0);
        self.common.w32(DRIVER_FEATURE, negotiated.bits() as u32);
        self.common.w32(DRIVER_FEATURE_SELECT, 1);
        self.common
            .w32(DRIVER_FEATURE, (negotiated.bits() >> 32) as u32);
        self.set_status(STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK)?;
        if self.common.r16(NUM_QUEUES) != QUEUES {
            return Err(Fault::Profile);
        }
        let device = self.device_config;
        let common = self.common;
        let bytes = queue::stable_config(
            || {
                let before = common.r8(CONFIG_GENERATION);
                let mut bytes = [0u8; queue::CONFIG_BYTES];
                for (index, chunk) in bytes.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                    *chunk = device.r32(index as u32 * 4).to_le_bytes();
                }
                (before, bytes, common.r8(CONFIG_GENERATION))
            },
            16,
        )?;
        let config = Config::parse(negotiated, &bytes)?;
        decimal(b"SND:CONFIG:JACKS=", config.jacks().into(), b"");
        decimal(b":STREAMS=", config.streams().into(), b"");
        decimal(b":CHMAPS=", config.chmaps().into(), b"\n");
        self.config = Some(config);
        for (index, queue) in ACTIVE.into_iter().enumerate() {
            self.setup_queue(index, queue)?;
        }
        // B5: the capture queue is never enabled and never gets a buffer.
        self.select(RXQ)?;
        self.common.w16(QUEUE_MSIX_VECTOR, NO_VECTOR);
        if self.common.r16(QUEUE_ENABLE) != 0 || self.common.r16(QUEUE_MSIX_VECTOR) != NO_VECTOR {
            return Err(Fault::Profile);
        }
        serial::write(b"SND:RXQ:DISABLED\n");
        self.common.w16(CONFIG_MSIX_VECTOR, NO_VECTOR);
        if self.common.r16(CONFIG_MSIX_VECTOR) != NO_VECTOR {
            return Err(Fault::Profile);
        }
        // Population before DRIVER_OK (§3.1.1 step 7). Bus mastering is still
        // off and nothing is notified, so the device cannot touch them yet.
        for buffer in 0..EVENT_BUFFERS {
            let chain = queue::event_chain(&self.pool, buffer)?;
            self.place(EVENTS, &chain)?;
        }
        fence(Ordering::SeqCst);
        // Bus mastering only right before DRIVER_OK; both read back.
        self.pci.set_command(MEMORY | BUS_MASTER | INT_DISABLE)?;
        self.set_status(STATUS_LIVE)?;
        self.life.go_live()?;
        self.notify(EVENTS)?;
        decimal(b"SND:DRIVER_OK:EPOCH=", self.life.epoch(), b"\n");
        Ok(true)
    }

    /// F2: refuse without DRIVER_OK; bus mastering was never enabled.
    fn refuse_features(&mut self) -> Result<(), Fault> {
        self.common.w8(
            DEVICE_STATUS,
            STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FAILED,
        );
        let status = self.status();
        if self.pci.bus_master() || status & STATUS_DRIVER_OK != 0 {
            return Err(Fault::Status(status));
        }
        serial::write(b"SND:OFFLINE:FEATURES\nSND:OFFLINE:BUS_MASTER=0:DRIVER_OK=0\n");
        self.common.w8(DEVICE_STATUS, 0);
        let deadline = self.clock.after(RESET_MS);
        while self.status() != 0 {
            if past(deadline) {
                return Err(Fault::Timeout(b"RESET"));
            }
            spin_loop();
        }
        Ok(())
    }

    fn setup_queue(&mut self, index: usize, queue: u16) -> Result<(), Fault> {
        self.select(queue)?;
        if self.common.r16(QUEUE_SIZE_REG) < QUEUE_SIZE || self.common.r16(QUEUE_ENABLE) != 0 {
            return Err(Fault::Profile);
        }
        self.common.w16(QUEUE_SIZE_REG, QUEUE_SIZE);
        self.common.w16(QUEUE_MSIX_VECTOR, NO_VECTOR);
        if self.common.r16(QUEUE_SIZE_REG) != QUEUE_SIZE
            || self.common.r16(QUEUE_MSIX_VECTOR) != NO_VECTOR
        {
            return Err(Fault::Profile);
        }
        let available = self.pool.available(queue)?;
        self.dma.w16(available, AVAIL_NO_INTERRUPT);
        self.write_address(QUEUE_DESC, self.pool.descriptors(queue)?)?;
        self.write_address(QUEUE_DRIVER, available)?;
        self.write_address(QUEUE_DEVICE, self.pool.used(queue)?)?;
        let notify = self
            .regions
            .notify_address(&self.bars, self.common.r16(QUEUE_NOTIFY_OFF))
            .map_err(|_| Fault::Profile)?;
        let start = self.bar_base + u64::from(self.regions.notify.offset);
        self.notify[index] = u32::try_from(notify - start).map_err(|_| Fault::Profile)?;
        self.common.w16(QUEUE_ENABLE, 1);
        if self.common.r16(QUEUE_ENABLE) != 1 {
            return Err(Fault::Profile);
        }
        Ok(())
    }

    fn notify(&self, index: usize) -> Result<(), Fault> {
        if !self.life.may_publish() {
            return Err(Fault::State);
        }
        self.notify_window.w16(self.notify[index], ACTIVE[index]);
        Ok(())
    }

    /// Writes one chain and publishes its head. The bookkeeping is updated
    /// first, so an outstanding head's descriptors are never rewritten.
    fn place<const N: usize>(&mut self, index: usize, chain: &Chain<N>) -> Result<(), Fault> {
        let queue = ACTIVE[index];
        let publication = self.rings[index].publish_chain(
            chain.head,
            chain.used_len,
            chain.alternate_used_len,
        )?;
        let table = self.pool.descriptors(queue)?;
        for (slot, descriptor) in chain.entries {
            let entry = table + u64::from(slot) * 16;
            let bytes = descriptor.encode();
            for (word, chunk) in bytes.as_chunks::<8>().0.iter().enumerate() {
                self.dma
                    .w64(entry + word as u64 * 8, u64::from_le_bytes(*chunk));
            }
        }
        let available = self.pool.available(queue)?;
        self.dma
            .w16(available + 4 + u64::from(publication.entry) * 2, chain.head);
        fence(Ordering::SeqCst);
        self.dma.w16(available + 2, publication.index);
        fence(Ordering::SeqCst);
        Ok(())
    }

    fn used_index(&self, index: usize) -> Result<u16, Fault> {
        Ok(self.dma.r16(self.pool.used(ACTIVE[index])? + 2))
    }

    /// Consumes one validated used element: exact length, published head and
    /// an index move bounded by the outstanding requests. Returns the head and
    /// the reported used length.
    fn take_used(&mut self, index: usize) -> Result<Option<(u16, u32)>, Fault> {
        let device_index = self.used_index(index)?;
        if self.rings[index].pending(device_index)? == 0 {
            return Ok(None);
        }
        fence(Ordering::Acquire);
        let element =
            self.pool.used(ACTIVE[index])? + 4 + u64::from(self.rings[index].used_entry()) * 8;
        let id = self.dma.r32(element);
        let length = self.dma.r32(element + 4);
        let head = self.rings[index].complete(id, length).inspect_err(|_| {
            decimal(b"SND:FAULT:USED:QUEUE=", index as u64, b"");
            decimal(b":ID=", id.into(), b"");
            decimal(b":LEN=", length.into(), b"\n");
        })?;
        fence(Ordering::Acquire);
        Ok(Some((head, length)))
    }

    /// F8/F10: any used-index move beyond the outstanding requests (also on
    /// an idle queue) and any status other than live are violations.
    fn check_device(&self) -> Result<(), Fault> {
        let status = self.status();
        if status != STATUS_LIVE {
            return Err(Fault::Status(status));
        }
        for index in 0..ACTIVE.len() {
            let device_index = self.used_index(index)?;
            self.rings[index].pending(device_index).inspect_err(|_| {
                decimal(b"SND:FAULT:USED:QUEUE=", index as u64, b"");
                decimal(b":INDEX=", device_index.into(), b"\n");
            })?;
        }
        Ok(())
    }

    /// One control request; `out` receives the whole zeroed-then-written
    /// response buffer. Returns the used length the device reported.
    fn control(
        &mut self,
        request: &[u8],
        response_len: usize,
        out: &mut [u8],
        pcm_info: bool,
    ) -> Result<usize, Fault> {
        if !self.life.may_publish() {
            return Err(Fault::State);
        }
        if self.rings[CONTROL].outstanding() != 0 || out.len() < response_len {
            return Err(Fault::Queue(queue::Error::Busy));
        }
        let chain = if pcm_info {
            queue::pcm_info_chain(&self.pool, request.len(), response_len)?
        } else {
            queue::control_chain(&self.pool, request.len(), response_len)?
        };
        for (offset, byte) in request.iter().enumerate() {
            self.dma.w8(self.pool.request() + offset as u64, *byte);
        }
        for offset in 0..response_len as u64 {
            self.dma.w8(self.pool.response() + offset, 0);
        }
        self.place(CONTROL, &chain)?;
        self.notify(CONTROL)?;
        let deadline = self.clock.after(CONTROL_MS);
        loop {
            self.check_device()?;
            if let Some((_, reported)) = self.take_used(CONTROL)? {
                for (offset, byte) in out[..response_len].iter_mut().enumerate() {
                    *byte = self.dma.r8(self.pool.response() + offset as u64);
                }
                return Ok(reported as usize);
            }
            if past(deadline) {
                return Err(Fault::Timeout(b"CONTROL_TIMEOUT"));
            }
            spin_loop();
        }
    }

    fn stream_control(
        &mut self,
        stream: &mut pcm::Stream,
        request: pcm::Request,
    ) -> Result<(), Fault> {
        let mut status = [0u8; pcm::CONTROL_STATUS_BYTES];
        let length = self.control(
            request.as_bytes(),
            request.response_len(),
            &mut status,
            false,
        )?;
        stream.complete_control(&status[..length])?;
        Ok(())
    }

    /// PCM_INFO for every advertised stream; B6: any capture stream rejects
    /// the profile, and only an output stream with stereo S16 48 kHz is used.
    fn open_stream(&mut self) -> Result<pcm::Stream, Fault> {
        let config = self.config.ok_or(Fault::State)?;
        let query = config.query_streams(0, config.streams())?;
        let length = query.response_len()?;
        let mut response = [0u8; MAX_RESPONSE_BYTES];
        let reported = self.control(&query.encode(), length, &mut response, true)?;
        decimal(b"SND:PCM_INFO:USED_LEN=", reported as u64, b"");
        decimal(b":RECORDS_LEN=", length as u64, b"\n");
        let infos = query.parse_response(&response[..length])?;
        if infos.as_slice().iter().any(|info| !info.is_output()) {
            return Err(Fault::Profile);
        }
        let selection = infos.first_stereo_s16_48000()?;
        let info = infos
            .as_slice()
            .iter()
            .find(|info| info.id() == selection.stream_id())
            .copied()
            .ok_or(Fault::Profile)?;
        let stream = pcm::Stream::new(info)?;
        decimal(b"SND:STREAM:", stream.id().into(), b":OUTPUT\n");
        Ok(stream)
    }

    fn configure(&mut self, stream: &mut pcm::Stream) -> Result<(), Fault> {
        let request = stream.set_params(FIXTURE_PERIODS, FIXTURE_PERIOD_BYTES)?;
        self.stream_control(stream, request)?;
        let params = stream.params().ok_or(Fault::State)?;
        decimal(b"SND:PARAMS:BUFFER=", params.buffer_bytes().into(), b"");
        decimal(b":PERIOD=", params.period_bytes().into(), b"");
        decimal(b":FEATURES=", params.features().into(), b"");
        serial::write(b":CHANNELS=2:FORMAT=S16:RATE=48000\n");
        let request = stream.prepare()?;
        self.stream_control(stream, request)?;
        serial::write(b"SND:PREPARE:OK\n");
        Ok(())
    }

    /// One period of `pattern` into `slot`, published on the TX queue.
    fn publish_tx(
        &mut self,
        stream: &pcm::Stream,
        slot: u16,
        pattern: Pattern,
        period: u32,
    ) -> Result<(), Fault> {
        if !self.life.may_publish() {
            return Err(Fault::State);
        }
        let length = FIXTURE_PERIOD_BYTES as usize;
        let header = stream.tx_header(length)?;
        let chain = queue::tx_chain(&self.pool, slot, length)?;
        let base = self.pool.tx_slot(slot)?;
        self.dma.w32(base, u32::from_le_bytes(header));
        let first = period * FIXTURE_PERIOD_FRAMES;
        for frame in 0..FIXTURE_PERIOD_FRAMES {
            let value = u32::from_le_bytes(pattern.frame(first + frame));
            self.dma.w32(base + 4 + u64::from(frame) * 4, value);
        }
        self.dma.w64(self.pool.tx_status(slot)?, 0);
        self.place(TX, &chain)?;
        self.notify(TX)
    }

    /// Re-posts consumed event buffers. F17 and the PR #197 advisory: every
    /// invalid event, including CTL_NOTIFY (`Unsupported`, CTLS is never
    /// negotiated), counts against the bound; the bound itself fences.
    fn poll_events(&mut self, mut stream: Option<&mut pcm::Stream>) -> Result<(), Fault> {
        let config = self.config.ok_or(Fault::State)?;
        while let Some((head, _)) = self.take_used(EVENTS)? {
            let address = self.pool.event(head)?;
            let mut bytes = [0u8; pcm::EVENT_BYTES];
            for (offset, byte) in bytes.iter_mut().enumerate() {
                *byte = self.dma.r8(address + offset as u64);
            }
            let outcome = match pcm::parse_event(config, &bytes) {
                Ok(event) => match stream.as_deref_mut() {
                    Some(stream) => stream.event(event),
                    None => Err(pcm::Error::InvalidEvent),
                },
                Err(error) => Err(error),
            };
            match outcome {
                Ok(()) | Err(pcm::Error::Stale) => {}
                Err(error) if error.requires_fence() => return Err(Fault::Pcm(error)),
                Err(_) => self.budget.note_invalid()?,
            }
            self.dma.w64(address, 0);
            let chain = queue::event_chain(&self.pool, head)?;
            self.place(EVENTS, &chain)?;
            self.notify(EVENTS)?;
        }
        Ok(())
    }

    /// Pre-queues while Prepared, STARTs, then refills each completed slot
    /// until `submit` periods are published; returns after `until` completions.
    fn play(
        &mut self,
        stream: &mut pcm::Stream,
        pattern: Pattern,
        submit: u32,
        until: u32,
    ) -> Result<(), Fault> {
        let mut free: u8 = (1 << TX_SLOTS) - 1;
        let mut submitted = 0;
        let mut completed = 0;
        while submitted < submit && free != 0 {
            let slot = free.trailing_zeros() as u16;
            free &= !(1 << slot);
            self.publish_tx(stream, slot, pattern, submitted)?;
            submitted += 1;
        }
        let request = stream.start()?;
        self.stream_control(stream, request)?;
        serial::write(b"SND:START:OK\n");
        let mut deadline = self.clock.after(TX_PROGRESS_MS);
        while completed < until {
            self.check_device()?;
            self.poll_events(Some(stream))?;
            while completed < until {
                let Some((head, _)) = self.take_used(TX)? else {
                    break;
                };
                let slot = head / 2;
                let address = self.pool.tx_status(slot)?;
                let mut status = [0u8; pcm::TX_STATUS_BYTES];
                for (offset, byte) in status.iter_mut().enumerate() {
                    *byte = self.dma.r8(address + offset as u64);
                }
                match stream.tx_completed(&status)? {
                    pcm::TxCompletion::Ok { .. } => {}
                    pcm::TxCompletion::IoError { .. } => return Err(Fault::TxIo),
                }
                completed += 1;
                free |= 1 << slot;
                deadline = self.clock.after(TX_PROGRESS_MS);
            }
            while completed < until && submitted < submit && free != 0 {
                let slot = free.trailing_zeros() as u16;
                free &= !(1 << slot);
                self.publish_tx(stream, slot, pattern, submitted)?;
                submitted += 1;
            }
            if completed < until && past(deadline) {
                return Err(Fault::Timeout(b"TX_TIMEOUT"));
            }
            spin_loop();
        }
        let frames = |periods: u32| u64::from(periods) * u64::from(FIXTURE_PERIOD_FRAMES);
        decimal(b"SND:SUBMITTED_FRAMES:", frames(submitted), b"\n");
        decimal(b"SND:FRAMES:", frames(completed), b"\n");
        decimal(b"SND:TX_COMPLETIONS:", completed.into(), b"\n");
        Ok(())
    }

    /// Bounded idle window: nothing outstanding, nothing may complete.
    fn idle(&mut self, stream: &mut pcm::Stream) -> Result<(), Fault> {
        if self.rings[TX].outstanding() != 0 || self.rings[CONTROL].outstanding() != 0 {
            return Err(Fault::State);
        }
        let end = self.clock.after(DRAIN_MS);
        while !past(end) {
            self.check_device()?;
            self.poll_events(Some(stream))?;
            spin_loop();
        }
        Ok(())
    }

    /// Drain, STOP, quiet window, RELEASE, and the refused second RELEASE.
    fn finish(&mut self, stream: &mut pcm::Stream) -> Result<(), Fault> {
        // A TX completion only means QEMU accepted the samples; its WAV
        // backend writes them on the virtual-clock audio timer, and QEMU 8.2
        // drops what is still buffered once STOP disables the voice (first
        // exploratory run: the last period was missing). So the stream first
        // stays RUNNING without TX for a bounded drain, then STOP, then a
        // bounded window in which the device must stay quiet.
        self.idle(stream)?;
        decimal(b"SND:DRAIN:", DRAIN_MS, b"MS:RUNNING\n");
        let request = stream.stop()?;
        self.stream_control(stream, request)?;
        serial::write(b"SND:STOP:OK\n");
        self.idle(stream)?;
        decimal(b"SND:IDLE:", DRAIN_MS, b"MS:STOPPED\n");
        let request = stream.release()?;
        self.stream_control(stream, request)?;
        serial::write(b"SND:RELEASE:OK\n");
        // F19: refused by the driver state machine before encoding; nothing is
        // sent. The refusal is the driver's own contract: QEMU 8.2.2 never
        // clears a released stream (its virtio_error path covers only unknown
        // stream ids), so the device itself would not catch the duplicate.
        let published = self.rings[CONTROL].available_index();
        match stream.release() {
            Err(pcm::Error::DoubleRelease) => {}
            _ => return Err(Fault::State),
        }
        let in_memory = self.dma.r16(self.pool.available(ACTIVE[CONTROL])? + 2);
        let status = self.status();
        if self.rings[CONTROL].available_index() != published
            || in_memory != published
            || status & STATUS_NEEDS_RESET != 0
            || status != STATUS_LIVE
        {
            return Err(Fault::Status(status));
        }
        serial::write(b"SND:RELEASE_TWICE:REFUSED:NOT_SENT\n");
        hex_byte(b"SND:DEVICE_STATUS:0x", status);
        decimal(b"SND:EVENTS:INVALID=", self.budget.invalid().into(), b"\n");
        Ok(())
    }

    /// Used indices through a separate supervisor RO/NX alias of the used page.
    fn used_snapshot(&self) -> [u16; 3] {
        let mut used = [0u16; 3];
        let mut offsets = [0usize; 3];
        for (offset, queue) in offsets.iter_mut().zip(ACTIVE) {
            *offset = Pool::used_index_offset(queue).unwrap_or(0) as usize;
        }
        fence(Ordering::SeqCst);
        paging::physical_read(self.pool.used_page(), |page| {
            for (value, offset) in used.iter_mut().zip(offsets) {
                // SAFETY: Read-only alias of the pinned used page; `offset` is
                // an aligned u16 inside that page (Pool layout).
                *value = unsafe { page.add(offset).cast::<u16>().read_volatile() };
            }
        });
        used
    }

    /// Fence (`TRANSPORT.md` §5 and its security correction): publication is
    /// frozen first, then status 0 and BME off are both read back before the
    /// deadline, the used rings stay unchanged for a window, and only then the
    /// epoch advances. Every miss quarantines; nothing is released selectively.
    fn fence(&mut self, budget_ms: u64, acknowledged: &[u8]) -> Result<u64, Fault> {
        let budget = budget_ms.saturating_mul(self.clock.per_ms);
        if self.life.begin_fence(rdtsc(), budget).is_err() {
            self.life.quarantine();
            return Err(Fault::Quarantined);
        }
        self.common.w8(DEVICE_STATUS, 0);
        // Raw write: the acknowledgement below reads it back.
        let _ = self.pci.0.write_u16(FIXTURE, 4, MEMORY | INT_DISABLE);
        loop {
            let status = self.status();
            let bus_master = self.pci.bus_master();
            match self.life.poll_fence(rdtsc(), status, bus_master) {
                FenceStep::Pending => spin_loop(),
                FenceStep::Acknowledged => break,
                FenceStep::Quarantined => return Err(Fault::Quarantined),
            }
        }
        serial::write(acknowledged);
        if self.life.record_snapshot(self.used_snapshot()).is_err() {
            return Err(Fault::Quarantined);
        }
        let end = self.clock.after(STABLE_MS);
        loop {
            let used = self.used_snapshot();
            let status = self.status();
            let bus_master = self.pci.bus_master();
            if self.life.check_snapshot(used, status, bus_master).is_err() {
                return Err(Fault::Quarantined);
            }
            if past(end) {
                break;
            }
            spin_loop();
        }
        if !acknowledged.is_empty() {
            serial::write(b"SND:USED:STABLE\n");
        }
        self.life.finish_fence().map_err(|_| Fault::Quarantined)
    }

    /// Every pool page is still Kernel-owned and mapped identity RW/NX.
    fn check_pages(
        &self,
        frames: &Frames<FRAME_COUNT>,
        paging: &paging::Paging,
    ) -> Result<(), Fault> {
        for page in self.pool.pages() {
            if frames.owner(page) != Ok(Owner::Kernel)
                || paging.leaf(page) & LEAF_MASK != page | 1 | 2 | (1 << 63)
            {
                return Err(Fault::Mapping);
            }
        }
        serial::write(b"SND:PAGES:RETAINED\n");
        Ok(())
    }

    fn abort(&mut self, fault: Fault) -> ! {
        serial::write(b"SND:FAULT:");
        serial::write(fault.code());
        serial::write(b"\n");
        if let Fault::Status(status) = fault {
            hex_byte(b"SND:FAULT:DEVICE_STATUS:0x", status);
        }
        if !matches!(self.life.phase(), Phase::Quarantined) {
            let _ = self.fence(FENCE_MS, b"");
        }
        serial::fail(b"SND")
    }
}

fn normal(
    device: &mut Device,
    frames: &Frames<FRAME_COUNT>,
    paging: &paging::Paging,
) -> Result<(), Fault> {
    let mut stream = device.open_stream()?;
    device.configure(&mut stream)?;
    device.play(
        &mut stream,
        Pattern::A,
        FIXTURE_TOTAL_PERIODS,
        FIXTURE_TOTAL_PERIODS,
    )?;
    device.finish(&mut stream)?;
    device.fence(FENCE_MS, b"SND:QUIESCE:ACK\n")?;
    device.check_pages(frames, paging)
}

/// Pattern A is interrupted by a fence with no TX outstanding: QEMU 8.2.2's
/// `virtio_snd_reset` drains only the control queue, so already popped TX
/// elements are played and completed into the re-initialised rings
/// (`TRANSPORT.md` §10; an exploratory in-flight run failed closed on exactly
/// that). Then epoch 1 and a complete re-initialisation play pattern B in full.
fn fence_reinit(
    device: &mut Device,
    frames: &Frames<FRAME_COUNT>,
    paging: &paging::Paging,
) -> Result<(), Fault> {
    let mut stream = device.open_stream()?;
    device.configure(&mut stream)?;
    device.play(&mut stream, Pattern::A, FENCE_AFTER, FENCE_AFTER)?;
    let epoch = device.fence(FENCE_MS, b"SND:FENCE:ACK\n")?;
    decimal(b"SND:EPOCH:", epoch, b"\n");
    if !device.initialize()? {
        return Err(Fault::Profile);
    }
    let mut stream = device.open_stream()?;
    device.configure(&mut stream)?;
    device.play(
        &mut stream,
        Pattern::B,
        FIXTURE_TOTAL_PERIODS,
        FIXTURE_TOTAL_PERIODS,
    )?;
    device.finish(&mut stream)?;
    device.fence(FENCE_MS, b"SND:QUIESCE:ACK\n")?;
    device.check_pages(frames, paging)
}

/// A fence with a zero deadline during playback (TX still outstanding) must
/// quarantine: no acknowledgement, no publication or MMIO afterwards, and the
/// pool stays pinned and mapped.
fn quarantine(
    device: &mut Device,
    frames: &Frames<FRAME_COUNT>,
    paging: &paging::Paging,
) -> Result<(), Fault> {
    let mut stream = device.open_stream()?;
    device.configure(&mut stream)?;
    device.play(&mut stream, Pattern::A, FIXTURE_TOTAL_PERIODS, FENCE_AFTER)?;
    match device.fence(0, b"SND:FENCE:ACK\n") {
        Err(Fault::Quarantined) if device.life.phase() == Phase::Quarantined => {}
        _ => return Err(Fault::State),
    }
    serial::write(b"SND:QUARANTINED\n");
    let mut published = [0u16; 3];
    for (index, value) in published.iter_mut().enumerate() {
        *value = device.rings[index].available_index();
    }
    let refused_tx = matches!(
        device.publish_tx(&stream, 0, Pattern::A, 0),
        Err(Fault::State)
    );
    let mut status = [0u8; pcm::CONTROL_STATUS_BYTES];
    let refused_control = matches!(
        device.control(&[0; 8], 4, &mut status, false),
        Err(Fault::State)
    );
    let mut unchanged = true;
    for (index, value) in published.iter().enumerate() {
        let available = device.pool.available(ACTIVE[index])?;
        unchanged &= device.rings[index].available_index() == *value
            && device.dma.r16(available + 2) == *value;
    }
    if !(refused_tx && refused_control && unchanged) {
        return Err(Fault::State);
    }
    serial::write(b"SND:PUBLISH:REFUSED\n");
    device.check_pages(frames, paging)
}

pub fn run(frames: &mut Frames<FRAME_COUNT>, paging: &mut paging::Paging, pool: Pool) {
    serial::write(b"SND:BEGIN\n");
    let clock = Clock::calibrate().unwrap_or_else(|_| serial::fail(b"SND_PIT"));
    decimal(b"SND:TSC_PER_MS:", clock.per_ms, b"\n");
    // SAFETY: CPL0, one CPU with IF=0; main.rs excludes every other PCI
    // configuration user (xHCI, SMP, M3 services) from this profile.
    let mut pci = Pci(unsafe { Mechanism1::new() });
    match locate(&mut pci) {
        Located::Absent => {
            serial::write(b"SND:ABSENT\n");
            return;
        }
        Located::Elsewhere => {
            serial::write(b"SND:ABSENT:PROFILE\n");
            return;
        }
        Located::Fixture => serial::write(b"SND:DEVICE:1af4:1059:00:08.0\n"),
    }
    let mut device = Device::attach(pci, frames, paging, pool, clock).unwrap_or_else(|fault| {
        serial::write(b"SND:FAULT:");
        serial::write(fault.code());
        serial::write(b"\n");
        serial::fail(b"SND")
    });
    match device.initialize() {
        Ok(true) => {}
        Ok(false) => return,
        Err(fault) => device.abort(fault),
    }
    let result = if cfg!(feature = "test-snd-quarantine") {
        quarantine(&mut device, frames, paging)
    } else if cfg!(feature = "test-snd-fence-reinit") {
        fence_reinit(&mut device, frames, paging)
    } else {
        normal(&mut device, frames, paging)
    };
    match result {
        Ok(()) => serial::write(b"SND:PASS\n"),
        Err(fault) => device.abort(fault),
    }
}
