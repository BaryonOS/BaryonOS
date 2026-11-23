//! H03 native QEMU Bochs DISPI mode-set fixture (IF0, single CPU, no ring 3).
//!
//! Binds exactly one PCI function 1234:1111 class 0x0300 whose raw BAR0 (read
//! only: no sizing, no command-register write, no bus mastering, no DMA)
//! decodes to the GOP framebuffer base already mapped by `framebuffer::map`.
//! Modes come only from `MODES`; every pixel store is bounded by the live mode
//! and the existing GOP mapping. Ports 0x1CE/0x1CF are touched only after the
//! PCI binding succeeded; any other device or mismatch is refused fail-closed.
use crate::{
    framebuffer::{descriptor, fill_mode, mode_extent},
    graphics::{initialize_input, poll_scancode},
    serial,
};
use core::arch::asm;
use pci_core::{Mechanism1, PciAddress, PciConfigAccess};

const INDEX_PORT: u16 = 0x1ce;
const DATA_PORT: u16 = 0x1cf;
const ID: u16 = 0;
const XRES: u16 = 1;
const YRES: u16 = 2;
const BPP: u16 = 3;
const ENABLE: u16 = 4;
const VIRT_WIDTH: u16 = 6;
const VIRT_HEIGHT: u16 = 7;
const X_OFFSET: u16 = 8;
const Y_OFFSET: u16 = 9;
const VIDEO_MEMORY_64K: u16 = 0xa;
/// ENABLED | LFB_ENABLED; NOCLEARMEM is deliberately not requested.
const ENABLE_LFB: u16 = 0x41;
/// Runner handshake: PS/2 set-1 make code of the `n` key (via i8042 translation).
const NEXT_KEY: u8 = 0x31;
/// Const allowlist; the first mode differs from the 1280x720 GOP mode.
const MODES: [(u16, u16); 4] = [(640, 480), (800, 600), (1024, 768), (1280, 720)];

fn read_register(index: u16) -> u16 {
    let value: u16;
    // SAFETY: CPL0, IF0, single CPU; the fixed Bochs VBE index/data ports of the
    // PCI function bound in `bind`. Scalar port I/O, no memory operands.
    unsafe {
        asm!("out dx, ax", in("dx") INDEX_PORT, in("ax") index, options(nomem, nostack, preserves_flags));
        asm!("in ax, dx", in("dx") DATA_PORT, out("ax") value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn write_register(index: u16, value: u16) {
    // SAFETY: As in read_register; values come only from MODES or constants.
    unsafe {
        asm!("out dx, ax", in("dx") INDEX_PORT, in("ax") index, options(nomem, nostack, preserves_flags));
        asm!("out dx, ax", in("dx") DATA_PORT, in("ax") value, options(nomem, nostack, preserves_flags));
    }
}

fn number(value: u64) {
    let mut digits = [0u8; 20];
    let mut at = digits.len();
    let mut rest = value;
    loop {
        at -= 1;
        digits[at] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    serial::write(&digits[at..]);
}

fn refuse(reason: &[u8]) -> ! {
    serial::write(b"DISPI:REFUSED:");
    serial::write(reason);
    serial::write(b"\n");
    serial::fail(b"DISPI_REFUSED")
}

/// Published per-mode pattern; `tools/run_dispi.py::pattern` is the host oracle.
fn pattern(mode: u32, x: u32, y: u32) -> u32 {
    let red = (x * 3 + mode * 0x40) & 0xff;
    let green = (y * 5 + mode * 0x25) & 0xff;
    let blue = ((x >> 8) * 0x40 + (y >> 8) * 0x10 + (x ^ y)) & 0xff;
    red << 16 | green << 8 | blue
}

fn bind() {
    let gop = descriptor();
    serial::hex(b"DISPI:GOP_BASE:", gop.base);
    serial::hex(b"DISPI:GOP_BYTES:", gop.byte_len);
    if gop.width != 1280 || gop.height != 720 || gop.pixel_format != 1 {
        refuse(b"GOP");
    }
    // SAFETY: Single IF0 bootstrap CPU at CPL0; no other CF8/CFC user exists.
    let mut pci = unsafe { Mechanism1::new() };
    let mut displays = 0u32;
    let mut found: Option<PciAddress> = None;
    let mut supported = true;
    pci_core::enumerate(&mut pci, |d| {
        if d.class == 3 {
            displays += 1;
            found = Some(d.address);
            supported &= d.vendor_id == 0x1234
                && d.device_id == 0x1111
                && d.subclass == 0
                && d.programming_interface == 0
                && d.address.segment == 0
                && d.address.bus == 0;
        }
    })
    .unwrap_or_else(|_| refuse(b"PCI_SCAN"));
    let address = match found {
        Some(address) if displays == 1 && supported => address,
        _ => refuse(b"DEVICE"),
    };
    // Raw read only: the BAR is neither sized nor rewritten.
    let bar0 = pci
        .read_u32(address, 0x10)
        .unwrap_or_else(|_| refuse(b"PCI_BAR"));
    serial::hex(b"DISPI:BAR0_RAW:", u64::from(bar0));
    // Memory BAR (bit 0 clear), 32-bit type (bits 2:1 zero), exact GOP base.
    if bar0 & 0x7 != 0 || u64::from(bar0 & !0xf) != gop.base {
        refuse(b"BAR0");
    }
    let id = read_register(ID);
    serial::hex(b"DISPI:ID:", u64::from(id));
    if !(0xb0c0..=0xb0c5).contains(&id) {
        refuse(b"ID");
    }
    let vram = u64::from(read_register(VIDEO_MEMORY_64K)) * 0x1_0000;
    serial::hex(b"DISPI:VRAM:", vram);
    for (width, height) in MODES {
        match mode_extent(&gop, width.into(), height.into()) {
            Ok(extent) if extent <= vram => {}
            _ => refuse(b"EXTENT"),
        }
    }
    // Counterexamples of the store bound, checked before the first store.
    if mode_extent(&gop, 1281, 720).is_ok()
        || mode_extent(&gop, 1280, 721).is_ok()
        || mode_extent(&gop, 0, 480).is_ok()
        || mode_extent(&gop, 640, 0).is_ok()
    {
        serial::fail(b"DISPI_BOUND_SELFTEST");
    }
}

fn set_mode(width: u16, height: u16) {
    write_register(ENABLE, 0);
    write_register(XRES, width);
    write_register(YRES, height);
    write_register(BPP, 32);
    write_register(VIRT_WIDTH, width);
    write_register(X_OFFSET, 0);
    write_register(Y_OFFSET, 0);
    write_register(ENABLE, ENABLE_LFB);
    let read = [
        read_register(XRES),
        read_register(YRES),
        read_register(BPP),
        read_register(ENABLE),
        read_register(VIRT_WIDTH),
        read_register(VIRT_HEIGHT),
        read_register(X_OFFSET),
        read_register(Y_OFFSET),
    ];
    serial::write(b"DISPI:READBACK:");
    for (index, value) in read.iter().enumerate() {
        if index != 0 {
            serial::write(b":");
        }
        number(u64::from(*value));
    }
    serial::write(b"\n");
    // VIRT_HEIGHT is device-derived (VRAM / line length); only its lower bound matters.
    if read != [width, height, 32, ENABLE_LFB, width, read[5], 0, 0]
        || read[5] < height
        || read_register(ID) < 0xb0c0
    {
        serial::fail(b"DISPI_READBACK");
    }
}

fn wait_next() {
    loop {
        if poll_scancode() == Some(NEXT_KEY) {
            return;
        }
        core::hint::spin_loop();
    }
}

pub fn run() -> ! {
    bind();
    initialize_input();
    for (mode, (width, height)) in MODES.into_iter().enumerate() {
        set_mode(width, height);
        fill_mode(width.into(), height.into(), |x, y| {
            pattern(mode as u32, x, y)
        })
        .unwrap_or_else(|_| serial::fail(b"DISPI_PIXEL_BOUND"));
        serial::write(b"DISPI:MODE:");
        number(width.into());
        serial::write(b"x");
        number(height.into());
        serial::write(b":SET\n");
        wait_next();
    }
    serial::write(b"DISPI:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
