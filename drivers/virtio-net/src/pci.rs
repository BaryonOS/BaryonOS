//! Fixed-BDF PCI discovery and the sole volatile/port-I/O boundary.
use super::{Discovery, Io, DEVICE_ID_NET};
use core::arch::asm;
use virtio_core::{parse_capabilities_for, Bar, Error};

const BDF: u32 = 7 << 11;

fn validate_registration(config: &[u8; 256]) -> Result<(), Error> {
    let address = driver_core::PciAddress {
        segment: 0,
        bus: 0,
        device: 7,
        function: 0,
    };
    let descriptor =
        driver_core::PciDeviceDescriptor::from_type0_config(address, config).map_err(|error| {
            match error {
                driver_core::PciHeaderError::NotPresent => Error::Offline,
                driver_core::PciHeaderError::InvalidAddress
                | driver_core::PciHeaderError::UnsupportedHeaderType => Error::Protocol,
            }
        })?;
    let mut registry = driver_core::DriverRegistry::<1>::new();
    registry
        .register(super::DRIVER_REGISTRATION)
        .map_err(|_| Error::Protocol)?;
    match registry.best_match(&descriptor) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(Error::Offline),
        Err(_) => Err(Error::Protocol),
    }
}

fn cfg_read(offset: u16) -> u32 {
    let value: u32;
    // SAFETY: CPL0, serialized single-CPU access to fixed owned PCI 00:07.0.
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
    // SAFETY: Only this fixed function's command register. A 16-bit write avoids
    // clearing PCI status W1C bits in the high half of the config dword.
    unsafe {
        asm!("out dx,eax",in("dx") 0xcf8u16,in("eax") 0x80000000|BDF|4,options(nomem,nostack));
        asm!("out dx,ax",in("dx") 0xcfcu16,in("ax") value,options(nomem,nostack));
    }
}

/// Probe only the exact configured modern network device. No other PCI function
/// or host device is touched; DMA remains disabled until validated initialization.
pub fn discover() -> Result<Discovery, Error> {
    if cfg_read(0) != (u32::from(DEVICE_ID_NET) << 16) | 0x1af4 {
        return Err(Error::Offline);
    }
    let mut config = [0; 256];
    for offset in (0..256).step_by(4) {
        config[offset..offset + 4].copy_from_slice(&cfg_read(offset as u16).to_le_bytes());
    }
    validate_registration(&config)?;
    let original = cfg_read(4) as u16;
    command((original & !7) | (1 << 10));
    let mut bars = [Bar::NONE; 6];
    let mut index = 0;
    while index < 6 {
        let offset = 0x10 + index as u16 * 4;
        let low = cfg_read(offset);
        if low & 1 != 0 {
            index += 1;
            continue;
        }
        let wide = low & 6 == 4;
        if low & 6 != 0 && !wide || wide && index == 5 {
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
        let base = u64::from(low & !15) | u64::from(high) << 32;
        let mask = u64::from(mask_low & !15) | u64::from(mask_high) << 32;
        let size = (!mask).wrapping_add(1);
        if base != 0 && size != 0 {
            if !size.is_power_of_two() || size > 65536 || !base.is_multiple_of(4096) {
                return Err(Error::Range);
            }
            bars[index] = Bar {
                base,
                size,
                is_memory: true,
            };
        }
        index += if wide { 2 } else { 1 };
    }
    command((original & !5) | 2 | (1 << 10));
    for offset in (0..256).step_by(4) {
        config[offset..offset + 4].copy_from_slice(&cfg_read(offset as u16).to_le_bytes());
    }
    validate_registration(&config)?;
    let regions = parse_capabilities_for(&config, &bars, DEVICE_ID_NET)?;
    let mut at = config[0x34] as usize;
    // parse_capabilities_for has already validated the complete capability walk.
    while at != 0 {
        let value = cfg_read(at as u16);
        if config[at] == 5 {
            cfg_write(at as u16, value & !(1 << 16));
        }
        if config[at] == 0x11 {
            cfg_write(at as u16, (value & !(1 << 31)) | (1 << 30));
        }
        at = config[at + 1] as usize;
    }
    Ok(Discovery {
        config,
        bars,
        regions,
    })
}

pub(super) struct Hardware;

impl Io for Hardware {
    fn read8(&self, address: u64) -> u8 {
        // SAFETY: Engine derives only bounded byte addresses in permanent UC
        // BAR mappings or exclusive pinned DMA pages; no references alias DMA.
        unsafe { (address as *const u8).read_volatile() }
    }
    fn read16(&self, address: u64) -> u16 {
        // SAFETY: Engine uses naturally aligned registers/ring words in the
        // same permanent, exclusively owned mapped regions.
        unsafe { (address as *const u16).read_volatile() }
    }
    fn read32(&self, address: u64) -> u32 {
        // SAFETY: Aligned validated common-register or used-element dword.
        unsafe { (address as *const u32).read_volatile() }
    }
    fn write8(&mut self, address: u64, value: u8) {
        // SAFETY: Validated exclusive MMIO/DMA byte, with publication ordering
        // handled by Engine; a posted bounce buffer is never overwritten.
        unsafe {
            (address as *mut u8).write_volatile(value);
        }
    }
    fn write16(&mut self, address: u64, value: u16) {
        // SAFETY: Aligned validated common, notify or driver-owned avail word.
        unsafe {
            (address as *mut u16).write_volatile(value);
        }
    }
    fn write32(&mut self, address: u64, value: u32) {
        // SAFETY: Aligned common config register in caller's permanent UC map.
        unsafe {
            (address as *mut u32).write_volatile(value);
        }
    }
    fn write64(&mut self, address: u64, value: u64) {
        // SAFETY: Naturally aligned common queue-address register; fixed DMA
        // address derives only from the validated pinned pool.
        unsafe {
            (address as *mut u64).write_volatile(value);
        }
    }
    fn bus_master(&mut self, enabled: bool) {
        let current = cfg_read(4) as u16;
        command(if enabled {
            current | 6 | (1 << 10)
        } else {
            (current & !4) | (1 << 10)
        });
    }
    fn bus_master_enabled(&self) -> bool {
        cfg_read(4) & 4 != 0
    }
}
