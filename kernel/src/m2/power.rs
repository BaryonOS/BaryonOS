//! QEMU-profile ACPI S5 poweroff from the bounded FADT/DSDT snapshot.
//!
//! The BSP snapshots both tables once during ACPI discovery (IF=0, one CPU),
//! `nieuw-os-acpi-power` validates them, and only then does the kernel write
//! the PM1 control register(s). In `native-acpi-s5` nothing can request it;
//! in `native-system-power` only init's non-delegable POWER capability can,
//! and only behind the storage gate in m3/system.rs (ADR-076). The plan never
//! leaves the kernel. SCI_EN/ACPI-mode switching via SMI_CMD is not performed.
use crate::serial;
use boot_protocol::acpi::PhysicalRead;
use core::{arch::asm, fmt::Write, ptr};
use nieuw_os_acpi_power::{
    fadt_dsdt_address, parse_s5, AcpiTableSnapshot, Error, S5Plan, TableSnapshot,
};

const HEADER: usize = 36;
const FADT_MAX: usize = 4096;
// shortcut: the QEMU q35 DSDT is ~8 KiB; raise when a profile ships a larger one.
const DSDT_MAX: usize = 64 * 1024;

#[derive(Clone, Copy)]
enum State {
    Unavailable,
    Unreadable,
    Rejected(Error),
    Ready(S5Plan),
}

static mut FADT: [u8; FADT_MAX] = [0; FADT_MAX];
static mut DSDT: [u8; DSDT_MAX] = [0; DSDT_MAX];
static mut STATE: State = State::Unavailable;

/// Called once from ACPI discovery with the firmware-map-checked reader.
pub(super) fn snapshot(reader: &mut impl PhysicalRead, fadt_address: u64) {
    // SAFETY: Only caller is the once-guarded discovery on the single BSP with
    // IF=0; no other reference to these statics exists before `off`.
    let (fadt, dsdt, state) = unsafe {
        (
            &mut *ptr::addr_of_mut!(FADT),
            &mut *ptr::addr_of_mut!(DSDT),
            &mut *ptr::addr_of_mut!(STATE),
        )
    };
    *state = if fadt_address == 0 {
        State::Unavailable
    } else {
        extract(reader, fadt_address, fadt, dsdt)
    };
}

fn read_table<'a>(
    reader: &mut impl PhysicalRead,
    address: u64,
    buffer: &'a mut [u8],
) -> Option<&'a mut [u8]> {
    let mut header = [0u8; HEADER];
    reader.read(address, &mut header).ok()?;
    let length = u32::from_le_bytes(header[4..8].try_into().ok()?) as usize;
    if !(HEADER..=buffer.len()).contains(&length) {
        return None;
    }
    let table = &mut buffer[..length];
    reader.read(address, table).ok()?;
    Some(table)
}

fn extract(
    reader: &mut impl PhysicalRead,
    fadt_address: u64,
    fadt: &mut [u8],
    dsdt: &mut [u8],
) -> State {
    let Some(fadt) = read_table(reader, fadt_address, fadt) else {
        return State::Unreadable;
    };
    let dsdt_address = match fadt_dsdt_address(fadt) {
        Ok(address) => address,
        Err(error) => return State::Rejected(error),
    };
    let Some(dsdt) = read_table(reader, dsdt_address, dsdt) else {
        return State::Unreadable;
    };
    if cfg!(feature = "test-acpi-s5-corrupt-dsdt") {
        // Negative fixture: one flipped byte (last, always present) in the kernel copy only.
        dsdt[dsdt.len() - 1] ^= 0x01;
    }
    match parse_s5(AcpiTableSnapshot::new(
        TableSnapshot::new(fadt, fadt_address),
        TableSnapshot::new(dsdt, dsdt_address),
    )) {
        Ok(plan) => State::Ready(plan),
        Err(error) => State::Rejected(error),
    }
}

struct Serial;
impl Write for Serial {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        serial::write(text.as_bytes());
        Ok(())
    }
}

fn in16(port: u16) -> u16 {
    let value: u16;
    // SAFETY: Port comes only from the validated S5 plan; IF=0 single CPU.
    unsafe {
        asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn out16(port: u16, value: u16) {
    // SAFETY: Same validated PM1 control port; this is the S5 transition.
    unsafe {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
    }
}

/// True when `off` would write PM1 (a validated plan exists).
#[cfg(feature = "native-system-power")]
pub(crate) fn ready() -> bool {
    // SAFETY: Written once during discovery on this CPU; read-only from here.
    matches!(unsafe { *ptr::addr_of!(STATE) }, State::Ready(_))
}

/// Powers off through S5 when the snapshot validated; otherwise reports why
/// and returns without any port I/O, so the caller's normal exit stays visible.
/// Callers run with IF=0 on the only CPU, so nothing runs between their
/// checks and the PM1 write.
pub(crate) fn off() {
    // SAFETY: Written once during discovery on this CPU; read-only from here.
    let state = unsafe { *ptr::addr_of!(STATE) };
    let plan = match state {
        State::Unavailable => return serial::write(b"POWER:S5:UNAVAILABLE\n"),
        State::Unreadable => return serial::write(b"POWER:S5:UNREADABLE\n"),
        State::Rejected(error) => {
            let _ = writeln!(Serial, "POWER:S5:REJECTED:{error:?}");
            return;
        }
        State::Ready(plan) => plan,
    };
    let pm1a = plan.pm1a();
    let current = in16(pm1a.address());
    let _ = writeln!(
        Serial,
        "POWER:S5:PM1A={:#06x}:SLP_TYP={}:SCI_EN={}",
        pm1a.address(),
        pm1a.sleep_type(),
        current & 1
    );
    if let Some(pm1b) = plan.pm1b() {
        let _ = writeln!(
            Serial,
            "POWER:S5:PM1B={:#06x}:SLP_TYP={}",
            pm1b.address(),
            pm1b.sleep_type()
        );
    }
    serial::write(b"POWER:S5:WRITE\n");
    // ACPICA order: SLP_TYP with SLP_EN clear to PM1a then PM1b, then SLP_EN.
    const SLP_EN: u16 = 1 << 13;
    let a = pm1a.apply_to(current);
    let b = plan
        .pm1b()
        .map(|pm1b| (pm1b.address(), pm1b.apply_to(in16(pm1b.address()))));
    out16(pm1a.address(), a & !SLP_EN);
    if let Some((port, value)) = b {
        out16(port, value & !SLP_EN);
    }
    out16(pm1a.address(), a);
    if let Some((port, value)) = b {
        out16(port, value);
    }
    // No effect means no exit: the runner's timeout reports it, never a pass.
    loop {
        // SAFETY: Terminal CPL0 halt with interrupts disabled.
        unsafe { asm!("cli", "hlt", options(nomem, nostack)) };
    }
}
