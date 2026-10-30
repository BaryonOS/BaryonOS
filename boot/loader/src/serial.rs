//! Fixed QEMU COM1 diagnostic endpoint; no firmware services or allocation.
use core::arch::asm;

fn out8(port: u16, value: u8) {
    // SAFETY: M1 runs at CPL0 in the documented QEMU profile. These ports are
    // exclusively used by this single-threaded bootstrap; no memory is accessed.
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

pub fn write(bytes: &[u8]) {
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

pub fn hex(value: u64) {
    let mut bytes = [b'0'; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let digit = ((value >> ((15 - index) * 4)) & 15) as u8;
        *byte = if digit < 10 {
            b'0' + digit
        } else {
            b'a' + digit - 10
        };
    }
    write(&bytes);
}

/// Decimal without leading zeros; evidence lines carry plain versions.
#[cfg_attr(not(feature = "signed-boot"), allow(dead_code))]
pub fn decimal(mut value: u64) {
    let mut digits = [b'0'; 20];
    let mut index = digits.len();
    loop {
        index -= 1;
        digits[index] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    write(&digits[index..]);
}

pub fn fail(code: &'static [u8]) -> ! {
    init();
    write(b"M1:FAIL:");
    write(code);
    write(b"\n");
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
