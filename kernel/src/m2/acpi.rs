use boot_protocol::{
    acpi::{self, AcpiHandoff, Availability, PhysicalRead, HANDOFF_OFFSET, HANDOFF_SIZE},
    BootInfo, MemoryMap, PAGE_SIZE,
};
use core::{
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};
use kernel_core::Range;

use super::{paging, serial};

struct FirmwareReader<'a, 'b> {
    paging: &'a mut paging::Paging,
    map: MemoryMap<'b>,
}

impl PhysicalRead for FirmwareReader<'_, '_> {
    fn read(&mut self, address: u64, output: &mut [u8]) -> Result<(), acpi::Error> {
        let length = u64::try_from(output.len()).map_err(|_| acpi::Error::Address)?;
        let end = address.checked_add(length).ok_or(acpi::Error::Address)?;
        let mapped_start = address & !(PAGE_SIZE - 1);
        let mapped_end = end
            .checked_add(PAGE_SIZE - 1)
            .map(|end| end & !(PAGE_SIZE - 1))
            .ok_or(acpi::Error::Address)?;
        if address == 0
            || end > (1 << 52)
            || mapped_end > (1 << 52)
            || !covers_acpi(&self.map, mapped_start, mapped_end)
        {
            return Err(acpi::Error::Address);
        }
        self.paging
            .read_firmware(address, output)
            .map_err(|_| acpi::Error::Read)
    }
}

// The 8-KiB table scratch and the inventory live in static bootstrap memory:
// on the 16-KiB BSP stack they overflowed into the guard page (M3 profiles).
static mut SCRATCH: acpi::Scratch = acpi::Scratch::new();
static mut DISCOVERY: acpi::Discovery = acpi::Discovery::empty(Availability::MissingHandoff);
static DISCOVERED: AtomicBool = AtomicBool::new(false);

/// Reports the bounded status pair and returns the inventory (incl. MADT).
/// Called once, by the BSP with IF=0, before any other CPU runs.
pub(super) fn discover(
    info: &BootInfo,
    map_bytes: &[u8],
    paging: &mut paging::Paging,
) -> &'static acpi::Discovery {
    assert!(
        !DISCOVERED.swap(true, Ordering::Relaxed),
        "ACPI discovery runs once"
    );
    // SAFETY: The once-guard above makes these the only references ever formed
    // to SCRATCH/DISCOVERY mutably; this runs on the single BSP with IF=0 before
    // any AP starts. After return DISCOVERY is only read through `&'static`.
    let (scratch, result) = unsafe {
        (
            &mut *ptr::addr_of_mut!(SCRATCH),
            &mut *ptr::addr_of_mut!(DISCOVERY),
        )
    };
    discover_tables(info, map_bytes, paging, scratch, result);
    report(result.availability);
    result
}

fn discover_tables(
    info: &BootInfo,
    map_bytes: &[u8],
    paging: &mut paging::Paging,
    scratch: &mut acpi::Scratch,
    result: &mut acpi::Discovery,
) {
    let Some(extension_address) = info.bootinfo_start.checked_add(HANDOFF_OFFSET) else {
        return reject(scratch, result);
    };
    let Some(bootinfo_end) = info.bootinfo_start.checked_add(info.bootinfo_len) else {
        return reject(scratch, result);
    };
    if extension_address
        .checked_add(HANDOFF_SIZE as u64)
        .is_none_or(|end| end > bootinfo_end)
    {
        return reject(scratch, result);
    }
    // SAFETY: BootInfo validation established an aligned, retained, one-page
    // handoff allocation; this extension lies within it and is copied before
    // parsing. The wire struct contains only fixed-width integers and byte arrays.
    let handoff = unsafe { ptr::read_unaligned(extension_address as *const AcpiHandoff) };
    let Ok(map) = MemoryMap::new(map_bytes, info.descriptor_size, info.descriptor_version) else {
        return reject(scratch, result);
    };
    let mut reader = FirmwareReader { paging, map };
    acpi::discover_into(&handoff, &mut reader, scratch, result);
}

/// A rejected handoff validates to an error: RejectedHandoff, no reads.
fn reject(scratch: &mut acpi::Scratch, result: &mut acpi::Discovery) {
    acpi::discover_into(&AcpiHandoff::rejected(), &mut NoRead, scratch, result);
}

/// Reader for the rejected-handoff path, which never reads firmware memory.
struct NoRead;
impl PhysicalRead for NoRead {
    fn read(&mut self, _: u64, _: &mut [u8]) -> Result<(), acpi::Error> {
        Err(acpi::Error::Read)
    }
}

fn covers_acpi(map: &MemoryMap<'_>, start: u64, end: u64) -> bool {
    let mut cursor = start;
    while cursor < end {
        let mut next = cursor;
        for index in 0..map.len() {
            let Some(descriptor) = map.descriptor(index) else {
                return false;
            };
            if !matches!(descriptor.memory_type, 9 | 10) {
                continue;
            }
            let Some(range) =
                Range::pages(descriptor.physical_start, descriptor.number_of_pages).ok()
            else {
                return false;
            };
            if range.start <= cursor && cursor < range.end {
                next = core::cmp::min(end, range.end);
                break;
            }
        }
        if next == cursor {
            return false;
        }
        cursor = next;
    }
    true
}

fn report(availability: Availability) {
    match availability {
        Availability::MissingHandoff => serial::write(b"ACPI:UNAVAILABLE:NO_HANDOFF\n"),
        Availability::RejectedHandoff => serial::write(b"ACPI:UNAVAILABLE:BAD_HANDOFF\n"),
        Availability::InvalidTables => serial::write(b"ACPI:UNAVAILABLE:INVALID_TABLES\n"),
        Availability::MissingMcfg => serial::write(b"ACPI:UNAVAILABLE:NO_MCFG\n"),
        Availability::MissingDmar => serial::write(b"ACPI:UNAVAILABLE:NO_DMAR\n"),
        Availability::DmarWithoutUnits => serial::write(b"ACPI:UNAVAILABLE:NO_DMAR_UNITS\n"),
        Availability::DiscoveryComplete => serial::write(b"ACPI:DISCOVERY:MCFG_DMAR\n"),
    }
    serial::write(b"DEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acpi_memory_ranges_must_be_firmware_reclaim_or_nvs() {
        let mut bytes = [0u8; 80];
        bytes[..4].copy_from_slice(&9u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&0x1000u64.to_le_bytes());
        bytes[24..32].copy_from_slice(&2u64.to_le_bytes());
        bytes[40..44].copy_from_slice(&7u32.to_le_bytes());
        bytes[48..56].copy_from_slice(&0x3000u64.to_le_bytes());
        bytes[64..72].copy_from_slice(&2u64.to_le_bytes());
        let map = MemoryMap::new(&bytes, 40, 1).unwrap();
        assert!(covers_acpi(&map, 0x1000, 0x3000));
        assert!(!covers_acpi(&map, 0x1000, 0x4000));
        assert!(!covers_acpi(&map, 0x3000, 0x4000));
    }
}
