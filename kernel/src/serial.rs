//! Fixed QEMU COM1 diagnostic endpoint; no firmware services or allocation.
//!
//! K06 F7 (native-smp only, DG-2): every line reaches the UART whole. `write`
//! collects fragments in this CPU's line buffer and emits the full line under
//! `SERIAL` (level 70, irqsave); terminal paths use `write_terminal`, which
//! never waits and marks a line it could not lock with a leading `!`. Before
//! `smp::start` sets the GS anchors, and in every one-CPU profile, bytes go
//! straight out exactly as before (DG-3).
use core::arch::asm;

fn out8(port: u16, value: u8) {
    // SAFETY: M1 runs at CPL0 in the documented QEMU profile. These ports are
    // used only by this module; native-smp serializes them with `SERIAL`.
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

fn in8(port: u16) -> u8 {
    let value: u8;
    // SAFETY: Same CPL0/profile contract as out8. UART status is only read.
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    }
    value
}

pub fn init() {
    out8(0x3f9, 0);
    out8(0x3fb, 0x80);
    out8(0x3f8, 1);
    out8(0x3f9, 0);
    out8(0x3fb, 3);
    out8(0x3fa, 0xc7);
    out8(0x3fc, 0x0b);
}

fn write_raw(bytes: &[u8]) {
    for &byte in bytes {
        let mut ready = false;
        for _ in 0..1_000_000 {
            if in8(0x3fd) & 0x20 != 0 {
                ready = true;
                break;
            }
        }
        if !ready {
            exit(0x11);
        }
        out8(0x3f8, byte);
    }
}

const LINE: usize = 160;

/// A line composed on the stack; overlong input is truncated, never wrapped.
pub struct Line {
    len: usize,
    bytes: [u8; LINE],
}

impl Line {
    pub const fn new() -> Self {
        Self {
            len: 0,
            bytes: [0; LINE],
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> &mut Self {
        let take = bytes.len().min(LINE - self.len);
        self.bytes[self.len..self.len + take].copy_from_slice(&bytes[..take]);
        self.len += take;
        self
    }

    #[cfg(all(feature = "native-smp", not(feature = "native-smp-legacy")))]
    pub fn decimal(&mut self, mut value: u64) -> &mut Self {
        let mut digits = [0u8; 20];
        let mut index = digits.len();
        loop {
            index -= 1;
            digits[index] = b'0' + (value % 10) as u8;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        self.push(&digits[index..])
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

#[cfg(all(feature = "native-smp", not(feature = "native-smp-legacy")))]
mod smp_line {
    use super::{write_raw, Line, LINE};
    use crate::m2::{
        lock::IrqLock,
        smp::{self, MAX_CPUS},
    };
    use core::{arch::asm, cell::UnsafeCell};
    use smp_lock::level;

    /// DG-2: held for exactly one whole line.
    static SERIAL: IrqLock<()> = IrqLock::new(level::SERIAL, ());

    /// One pending line per CPU (BSP 0, AP slot + 1).
    struct Pending(UnsafeCell<Line>);
    // SAFETY: slot `i` is touched only by CPU `i` with IF=0 (`write` masks
    // it); the terminal handlers that could nest never touch it.
    unsafe impl Sync for Pending {}
    static PENDING: [Pending; MAX_CPUS] =
        [const { Pending(UnsafeCell::new(Line::new())) }; MAX_CPUS];

    /// Bounded try_lock polls on a terminal path: never wait for a holder that
    /// may itself be stopped.
    const TERMINAL_TRIES: u32 = 100_000;

    fn write_line(bytes: &[u8]) {
        if cfg!(feature = "test-smp-serial-unlocked") {
            // Control profile (spec §4): CPUs may interleave inside a line.
            return write_raw(bytes);
        }
        let _line = SERIAL.lock_irqsave();
        write_raw(bytes);
    }

    pub fn write_terminal(bytes: &[u8]) {
        let locked = (0..TERMINAL_TRIES).find_map(|_| {
            core::hint::spin_loop();
            SERIAL.try_terminal()
        });
        if locked.is_none() {
            write_raw(b"!");
        }
        write_raw(bytes);
    }

    pub fn write(bytes: &[u8]) {
        let Some(cpu) = smp::anchored_index() else {
            return write_raw(bytes);
        };
        let flags: u64;
        // SAFETY: CPL0; saves RFLAGS (balanced push/pop) and masks interrupts
        // on this CPU so no handler interleaves with its pending line.
        unsafe { asm!("pushfq", "pop {}", "cli", out(reg) flags) };
        // SAFETY: this CPU's own slot with IF=0 (see `Pending`).
        let line = unsafe { &mut *PENDING[cpu].0.get() };
        for chunk in bytes.split_inclusive(|&byte| byte == b'\n') {
            line.push(chunk);
            if chunk.ends_with(b"\n") || line.len == LINE {
                write_line(line.as_bytes());
                line.len = 0;
            }
        }
        if flags & (1 << 9) != 0 {
            // SAFETY: restores the IF=1 this CPU had on entry.
            unsafe { asm!("sti", options(nomem, nostack)) };
        }
    }
}

/// Writes `bytes`; under native-smp a line leaves whole at its newline.
pub fn write(bytes: &[u8]) {
    #[cfg(all(feature = "native-smp", not(feature = "native-smp-legacy")))]
    smp_line::write(bytes);
    #[cfg(any(not(feature = "native-smp"), feature = "native-smp-legacy"))]
    write_raw(bytes);
}

/// Terminal paths (NMI, #DF, panic, fail, stuck lock): one whole line without
/// waiting; `!` first if `SERIAL` could not be taken (DG-2).
pub fn write_terminal(bytes: &[u8]) {
    #[cfg(all(feature = "native-smp", not(feature = "native-smp-legacy")))]
    smp_line::write_terminal(bytes);
    #[cfg(any(not(feature = "native-smp"), feature = "native-smp-legacy"))]
    write_raw(bytes);
}

/// Writes `label`, `value` as 16 lowercase hex digits, and a newline.
pub fn hex(label: &[u8], value: u64) {
    let mut digits = [b'0'; 16];
    for (index, byte) in digits.iter_mut().enumerate() {
        let digit = ((value >> ((15 - index) * 4)) & 15) as u8;
        *byte = if digit < 10 {
            b'0' + digit
        } else {
            b'a' + digit - 10
        };
    }
    write(label);
    write(&digits);
    write(b"\n");
}

pub fn fail(code: &'static [u8]) -> ! {
    init();
    write_terminal(
        Line::new()
            .push(b"M1:FAIL:")
            .push(code)
            .push(b"\n")
            .as_bytes(),
    );
    exit(0x11)
}

pub fn exit(value: u32) -> ! {
    // SAFETY: Dedicated isa-debug-exit QEMU device. This never returns to firmware.
    unsafe {
        asm!("out dx, eax", in("dx") 0xf4u16, in("eax") value, options(nomem, nostack));
    }
    loop {
        // SAFETY: Terminal CPL0 path; do not resume with potentially invalid firmware state.
        unsafe {
            asm!("cli", "hlt", options(nomem, nostack));
        }
    }
}
