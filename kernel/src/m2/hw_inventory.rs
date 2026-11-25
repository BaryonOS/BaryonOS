//! H06-T3: read-only native hardware inventory (protocol HW-INV v2).
//!
//! Reports CPUID, the loader's `SecureBoot` observation, the ACPI DMAR
//! discovery result and the identity dwords of every segment-0 PCI function
//! as `HWINV:` markers. Nothing is written to any device: no BAR is mapped, no
//! command register or bus master is enabled and no driver starts. The profile
//! ends in a CLI/HLT loop instead of QEMU's isa-debug-exit port, which is
//! undefined on physical PCs. A DMAR table is never evidence of containment.
use crate::serial::{self, Line};
use boot_protocol::{
    acpi::{Availability, Discovery},
    firmware::{self, SecureBoot, FACTS_OFFSET, FACTS_SIZE},
    BootInfo,
};
use core::{arch::asm, ptr};
use pci_core::{inventory, InventoryFunction, Mechanism1};

/// Upper bound shared with `tools/hw_evidence.py` (inventory.pci <= 1024).
const MAX_FUNCTIONS: u32 = 1024;

pub(super) fn run(info: &BootInfo, discovery: &Discovery) -> ! {
    serial::write(b"HWINV:BEGIN\n");
    if cfg!(feature = "test-hwinv-fail") {
        halt(b"HWINV:FAIL:FIXTURE\n");
    }
    #[cfg(feature = "test-hwinv-serial-timeout")]
    {
        serial::write(b"HWINV:FIXTURE:SERIAL_TIMEOUT_ARMED\n");
        serial::inject_timeout();
    }
    let Some(vendor) = cpu_vendor() else {
        halt(b"HWINV:FAIL:CPUID\n")
    };
    let mut line = Line::new();
    line.push(b"HWINV:CPUID:VENDOR=")
        .push(&vendor)
        .push(b":HYPERVISOR=")
        .push(if hypervisor_bit() { b"1\n" } else { b"0\n" });
    serial::write(line.as_bytes());

    match secure_boot(info) {
        Some(SecureBoot::Disabled) => serial::write(b"HWINV:SECUREBOOT=0\n"),
        Some(SecureBoot::Enabled) => serial::write(b"HWINV:SECUREBOOT=1\n"),
        Some(SecureBoot::Absent) => serial::write(b"HWINV:SECUREBOOT=ABSENT\n"),
        Some(SecureBoot::Unreadable) | None => halt(b"HWINV:FAIL:SECUREBOOT\n"),
    }

    let Some((present, units)) = dmar(discovery) else {
        halt(b"HWINV:FAIL:ACPI\n")
    };
    let mut line = Line::new();
    line.push(b"HWINV:ACPI:DMAR=")
        .push(if present { b"PRESENT" } else { b"ABSENT" })
        .push(b":DRHD=");
    push_decimal(&mut line, u32::from(units));
    serial::write(line.push(b"\n").as_bytes());

    // SAFETY: CPL0 on the single BSP with IF clear; this profile starts no AP,
    // driver or service, so nothing else uses CF8/CFC while the adapter lives.
    let mut access = unsafe { Mechanism1::new() };
    let mut listed = 0u32;
    let result = inventory(&mut access, |function| {
        listed += 1;
        if listed <= MAX_FUNCTIONS {
            report(&function);
        }
    });
    match result {
        Ok(0) => halt(b"HWINV:FAIL:PCI_EMPTY\n"),
        Ok(count) if count <= MAX_FUNCTIONS => {
            let mut line = Line::new();
            line.push(b"HWINV:PCI:COUNT=");
            push_decimal(&mut line, count);
            serial::write(line.push(b"\n").as_bytes());
        }
        Ok(_) => halt(b"HWINV:FAIL:PCI_LIMIT\n"),
        Err(_) => halt(b"HWINV:FAIL:PCI_CONFIG\n"),
    }
    serial::write(b"HWINV:PASS\n");
    halt(b"")
}

/// The 12-byte CPUID leaf-0 vendor string; `None` unless printable ASCII.
fn cpu_vendor() -> Option<[u8; 12]> {
    let leaf = core::arch::x86_64::__cpuid(0);
    let mut vendor = [0u8; 12];
    vendor[..4].copy_from_slice(&leaf.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&leaf.edx.to_le_bytes());
    vendor[8..].copy_from_slice(&leaf.ecx.to_le_bytes());
    vendor
        .iter()
        .all(|byte| (b' '..=b'~').contains(byte))
        .then_some(vendor)
}

/// CPUID.1:ECX[31], set by hypervisors for their guests.
fn hypervisor_bit() -> bool {
    let leaf = core::arch::x86_64::__cpuid(1);
    leaf.ecx & (1 << 31) != 0
}

/// Copies and validates the loader's firmware-facts extension.
fn secure_boot(info: &BootInfo) -> Option<SecureBoot> {
    let address = info.bootinfo_start.checked_add(FACTS_OFFSET)?;
    let end = address.checked_add(FACTS_SIZE as u64)?;
    if end > info.bootinfo_start.checked_add(info.bootinfo_len)? {
        return None;
    }
    // SAFETY: BootInfo validation established an aligned, retained, identity-
    // mapped one-page handoff allocation; the 16 bytes lie within it and are
    // copied by value before parsing. A loader without the extension left them zero.
    let bytes = unsafe { ptr::read_unaligned(address as *const [u8; FACTS_SIZE]) };
    firmware::decode(&bytes).ok()
}

/// (DMAR table present, DRHD units) from the completed discovery; `None` when
/// discovery stopped before the DMAR table could be observed.
fn dmar(discovery: &Discovery) -> Option<(bool, u8)> {
    match discovery.availability {
        Availability::MissingDmar => Some((false, 0)),
        Availability::DmarWithoutUnits => Some((true, 0)),
        Availability::DiscoveryComplete if discovery.dmar_count > 0 => {
            Some((true, discovery.dmar_count))
        }
        _ => None,
    }
}

/// `HWINV:PCI:<ssss>:<bb>:<dd>.<f>:<vendor>:<device>:<class>:<subclass>:<prog_if>:<revision>`
fn report(function: &InventoryFunction) {
    let address = function.address;
    let mut line = Line::new();
    line.push(b"HWINV:PCI:");
    push_hex(&mut line, u32::from(address.segment), 4);
    line.push(b":");
    push_hex(&mut line, u32::from(address.bus), 2);
    line.push(b":");
    push_hex(&mut line, u32::from(address.device), 2);
    line.push(b".");
    push_hex(&mut line, u32::from(address.function), 1);
    for (value, digits) in [
        (u32::from(function.vendor), 4),
        (u32::from(function.device), 4),
        (u32::from(function.class), 2),
        (u32::from(function.subclass), 2),
        (u32::from(function.prog_if), 2),
        (u32::from(function.revision), 2),
    ] {
        line.push(b":");
        push_hex(&mut line, value, digits);
    }
    serial::write(line.push(b"\n").as_bytes());
}

fn push_hex(line: &mut Line, value: u32, digits: u32) {
    for index in (0..digits).rev() {
        let digit = ((value >> (index * 4)) & 15) as u8;
        line.push(&[if digit < 10 {
            b'0' + digit
        } else {
            b'a' + digit - 10
        }]);
    }
}

fn push_decimal(line: &mut Line, mut value: u32) {
    let mut digits = [0u8; 10];
    let mut index = digits.len();
    loop {
        index -= 1;
        digits[index] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    line.push(&digits[index..]);
}

/// Writes `marker` and stops this CPU without touching port 0xF4.
fn halt(marker: &[u8]) -> ! {
    serial::write(marker);
    loop {
        // SAFETY: Terminal CPL0 path of a read-only profile; IF stays clear.
        unsafe {
            asm!("cli", "hlt", options(nomem, nostack));
        }
    }
}
