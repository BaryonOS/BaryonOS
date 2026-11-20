//! QEMU-profile ACPI S5 poweroff from the bounded FADT/DSDT snapshot.
//!
//! The BSP snapshots both tables once during ACPI discovery (IF=0, one CPU),
//! `nieuw-os-acpi-power` validates them, and only then does the kernel write
//! the PM1 control register(s). In `native-acpi-s5` nothing can request it;
//! in `native-system-power` only init's non-delegable POWER capability can,
//! and only behind the storage gate in m3/system.rs (ADR-076). The plan never
//! leaves the kernel. SCI_EN/ACPI-mode switching via SMI_CMD is not performed.
#[cfg(any(feature = "native-acpi-s5", feature = "native-system-power"))]
use crate::serial;
use boot_protocol::acpi::PhysicalRead;
#[cfg(any(feature = "native-acpi-s5", feature = "native-system-power"))]
use core::fmt::Write;
use core::{arch::asm, ptr};
use nieuw_os_acpi_power::{
    fadt_dsdt_address, parse_s5, AcpiTableSnapshot, Error, S5Plan, TableSnapshot,
};
#[cfg(feature = "acpi-s3")]
use nieuw_os_acpi_power::{fadt_facs_address, parse_s3, S3Plan};

const HEADER: usize = 36;
const FADT_MAX: usize = 4096;
// shortcut: the QEMU q35 DSDT is ~8 KiB; raise when a profile ships a larger one.
const DSDT_MAX: usize = 64 * 1024;
/// The QEMU/OVMF FACS is 64 bytes; one page bounds any FACS this profile reads.
#[cfg(feature = "acpi-s3")]
const FACS_MAX: usize = 4096;

// The S3-only profile computes the S5 plan without a power-off caller.
#[cfg_attr(
    not(any(feature = "native-acpi-s5", feature = "native-system-power")),
    allow(dead_code)
)]
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

/// S3 observations from the same once-only discovery. `Ready` additionally
/// records that the whole FACS extent is UEFI ACPI NVS (type 10), the only
/// condition under which `sleep` may store its waking vector.
#[cfg(feature = "acpi-s3")]
#[derive(Clone, Copy)]
pub(crate) enum S3State {
    Unavailable,
    Unreadable,
    Rejected(Error),
    NotNvs,
    Ready(S3Plan),
}
#[cfg(feature = "acpi-s3")]
static mut FACS: [u8; FACS_MAX] = [0; FACS_MAX];
#[cfg(feature = "acpi-s3")]
static mut S3: S3State = S3State::Unavailable;

/// Called once from ACPI discovery with the firmware-map-checked reader and
/// the predicate that a physical span is entirely UEFI ACPI NVS memory.
pub(super) fn snapshot(
    reader: &mut impl PhysicalRead,
    fadt_address: u64,
    _is_nvs: impl Fn(u64, u64) -> bool,
) {
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
    #[cfg(feature = "acpi-s3")]
    {
        // SAFETY: Same once-guarded single-BSP discovery as above; nothing
        // else references FACS/S3 before `s3()` reads the final state.
        let (facs, s3) = unsafe { (&mut *ptr::addr_of_mut!(FACS), &mut *ptr::addr_of_mut!(S3)) };
        *s3 = if fadt_address == 0 {
            S3State::Unavailable
        } else {
            extract_s3(reader, fadt_address, &fadt[..], &dsdt[..], facs, &_is_nvs)
        };
    }
}

/// Re-reads the bounded FADT/DSDT copies (already validated or rejected by
/// the S5 path) together with the FADT-designated FACS.
#[cfg(feature = "acpi-s3")]
fn extract_s3(
    reader: &mut impl PhysicalRead,
    fadt_address: u64,
    fadt_buffer: &[u8],
    dsdt_buffer: &[u8],
    facs_buffer: &mut [u8],
    is_nvs: &impl Fn(u64, u64) -> bool,
) -> S3State {
    let length = |bytes: &[u8]| {
        bytes
            .get(4..8)
            .and_then(|field| field.try_into().ok())
            .map(|field| u32::from_le_bytes(field) as usize)
    };
    let Some(fadt) = length(fadt_buffer)
        .filter(|len| (HEADER..=fadt_buffer.len()).contains(len))
        .map(|len| &fadt_buffer[..len])
    else {
        return S3State::Unreadable;
    };
    let dsdt_address = match fadt_dsdt_address(fadt) {
        Ok(address) => address,
        Err(error) => return S3State::Rejected(error),
    };
    let Some(dsdt) = length(dsdt_buffer)
        .filter(|len| (HEADER..=dsdt_buffer.len()).contains(len))
        .map(|len| &dsdt_buffer[..len])
    else {
        return S3State::Unreadable;
    };
    let facs_address = match fadt_facs_address(fadt) {
        Ok(address) => address,
        Err(error) => return S3State::Rejected(error),
    };
    // FACS is not an SDT: its length field sits at the same offset, but the
    // header is only 8 bytes and there is no checksum.
    let mut header = [0u8; 8];
    if reader.read(facs_address, &mut header).is_err() {
        return S3State::Unreadable;
    }
    let declared = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
    if !(64..=facs_buffer.len()).contains(&declared) {
        return S3State::Rejected(Error::TableTooLarge);
    }
    let facs = &mut facs_buffer[..declared];
    if reader.read(facs_address, facs).is_err() {
        return S3State::Unreadable;
    }
    if cfg!(feature = "test-acpi-s3-corrupt-facs") {
        // Negative fixture: one flipped signature byte in the kernel copy only.
        facs[0] ^= 0x01;
    }
    let plan = match parse_s3(
        AcpiTableSnapshot::new(
            TableSnapshot::new(fadt, fadt_address),
            TableSnapshot::new(dsdt, dsdt_address),
        ),
        TableSnapshot::new(facs, facs_address),
    ) {
        Ok(plan) => plan,
        Err(error) => return S3State::Rejected(error),
    };
    if !is_nvs(facs_address, facs_address + declared as u64) {
        return S3State::NotNvs;
    }
    S3State::Ready(plan)
}

/// The S3 discovery result; read-only after discovery.
#[cfg(feature = "acpi-s3")]
pub(crate) fn s3() -> S3State {
    // SAFETY: Written once during discovery on this CPU; read-only from here.
    unsafe { *ptr::addr_of!(S3) }
}

/// Re-reads the FACS hardware signature (offset 8) for the post-wake check.
#[cfg(feature = "acpi-s3")]
pub(crate) fn facs_bytes() -> &'static [u8] {
    // SAFETY: The FACS copy is written only during discovery; afterwards it
    // is a read-only snapshot of the pre-sleep firmware state.
    unsafe { &*ptr::addr_of!(FACS) }
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

#[cfg(any(feature = "native-acpi-s5", feature = "native-system-power"))]
struct Serial;
#[cfg(any(feature = "native-acpi-s5", feature = "native-system-power"))]
impl Write for Serial {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        serial::write(text.as_bytes());
        Ok(())
    }
}

pub(super) fn in16(port: u16) -> u16 {
    let value: u16;
    // SAFETY: Callers pass only PM1 ports of a validated S5/S3 plan; IF=0 single CPU.
    unsafe {
        asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, nostack, preserves_flags));
    }
    value
}

pub(super) fn out16(port: u16, value: u16) {
    // SAFETY: Same validated PM1 control/event ports; IF=0 single CPU.
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

#[cfg(any(feature = "native-acpi-s5", feature = "native-system-power"))]
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
