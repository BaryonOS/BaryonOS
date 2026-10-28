//! G0 diagnostic framebuffer and polled PS/2 bootstrap; not a desktop or M7.
//! IF remains clear, one CPU, no scheduling, user pointers or storage access.
use crate::{
    m2::{paging::Paging, FRAME_COUNT},
    serial,
};
use boot_protocol::{
    graphics::{FramebufferInfo, FRAMEBUFFER_WINDOW},
    BootInfo, MemoryMap,
};
use core::{arch::asm, mem::MaybeUninit, ptr};
use kernel_core::Frames;
use ui_core::{EditResult, Geometry, KeyCode, Keyboard, PixelFormat, TextLine, MAX_ROW_BYTES};

static mut FRAMEBUFFER: MaybeUninit<FramebufferInfo> = MaybeUninit::uninit();
static mut ROW: [u8; MAX_ROW_BYTES] = [0; MAX_ROW_BYTES];

pub fn capture(info: &BootInfo) {
    // SAFETY: Caller validated opt-in1.2; reserved0 equals its retained,
    // initialized BootInfo page+128. The complete64-byte extension fits in that
    // page. Firmware is finished and nothing can mutate the copied metadata.
    let framebuffer = unsafe { (info.reserved0 as *const FramebufferInfo).read() };
    framebuffer
        .validate()
        .unwrap_or_else(|_| serial::fail(b"G0_FRAMEBUFFER"));
    // SAFETY: Single IF0 entry before any use, static is private kernel RAM.
    unsafe {
        ptr::addr_of_mut!(FRAMEBUFFER).write(MaybeUninit::new(framebuffer));
    }
}
fn framebuffer() -> FramebufferInfo {
    // SAFETY: capture initialized this private static before M2 start; metadata
    // is immutable thereafter and copied out without an outstanding borrow.
    unsafe { (*ptr::addr_of!(FRAMEBUFFER)).assume_init() }
}
pub fn map(paging: &mut Paging, frames: &mut Frames<FRAME_COUNT>, memory: &MemoryMap<'_>) {
    paging
        .map_framebuffer(frames, &framebuffer(), memory)
        .unwrap_or_else(|_| serial::fail(b"G0_MAPPING"));
}
fn number(mut value: u64) {
    let mut digits = [0u8; 20];
    let mut at = digits.len();
    loop {
        at -= 1;
        digits[at] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    serial::write(&digits[at..]);
}
fn redraw(geometry: Geometry, line: &TextLine<48>) {
    // SAFETY: Exactly one IF0 kernel thread owns ROW for this synchronous draw.
    // No callback, interrupt or user can access it, and it is ordinary RAM.
    let row = unsafe { &mut *ptr::addr_of_mut!(ROW) };
    for y in 0..geometry.height() {
        ui_core::render_debug_row(geometry, line, y, row).unwrap();
        for x in 0..geometry.width() {
            let offset = x as usize * 4;
            let pixel = u32::from_le_bytes(row[offset..offset + 4].try_into().unwrap());
            let destination = FRAMEBUFFER_WINDOW
                + (u64::from(y) * u64::from(geometry.stride()) + u64::from(x)) * 4;
            // SAFETY: M2 audited the exact supervisorRW/NX/UC framebuffer leaves.
            // x<width,y<height,stride>=width and checked byte bounds establish
            // a four-byte aligned pixel wholly inside GOP's exposed byte span.
            // Only raw volatile stores touch MMIO; no Rust slice/reference does.
            unsafe {
                (destination as *mut u32).write_volatile(pixel);
            }
        }
    }
}
fn read(port: u16) -> u8 {
    let byte: u8;
    // SAFETY: CPL0, fixed QEMU i8042 controller ports only; scalar I/O, IF0.
    unsafe {
        asm!("in al,dx",in("dx")port,out("al")byte,options(nomem,nostack,preserves_flags));
    }
    byte
}
fn write(port: u16, byte: u8) {
    // SAFETY: CPL0, fixed QEMU i8042 controller ports only; scalar I/O, IF0.
    unsafe {
        asm!("out dx,al",in("dx")port,in("al")byte,options(nomem,nostack,preserves_flags));
    }
}
fn send(port: u16, byte: u8) {
    for _ in 0..100_000 {
        if read(0x64) & 2 == 0 {
            write(port, byte);
            return;
        }
        core::hint::spin_loop();
    }
    serial::fail(b"G0_KEYBOARD_WRITE_TIMEOUT");
}
fn response() -> u8 {
    for _ in 0..100_000 {
        let status = read(0x64);
        if status & 1 != 0 {
            let byte = read(0x60);
            if status & 0xe0 == 0 {
                return byte;
            }
        }
        core::hint::spin_loop();
    }
    serial::fail(b"G0_KEYBOARD_READ_TIMEOUT");
}
fn keyboard_setup() {
    // QEMU i8042 profile: disable interfaces, drain old firmware bytes, keep
    // IRQs disabled and select controller translation from Set2 to Set1.
    send(0x64, 0xad);
    send(0x64, 0xa7);
    for _ in 0..256 {
        if read(0x64) & 1 == 0 {
            break;
        }
        let _ = read(0x60);
    }
    if read(0x64) & 1 != 0 {
        serial::fail(b"G0_KEYBOARD_DRAIN");
    }
    send(0x64, 0x20);
    let mode = response();
    send(0x64, 0x60);
    send(0x60, (mode & 0x0c) | 0x60);
    send(0x64, 0xae);
    send(0x60, 0xf4);
    if response() != 0xfa {
        serial::fail(b"G0_KEYBOARD_ACK");
    }
}
pub fn run() -> ! {
    let f = framebuffer();
    let format = if f.pixel_format == 0 {
        PixelFormat::Rgb
    } else {
        PixelFormat::Bgr
    };
    let geometry = Geometry::new(f.width, f.height, f.stride, f.byte_len as usize, format).unwrap();
    serial::write(b"G0:FRAMEBUFFER:");
    number(u64::from(f.width));
    serial::write(b":");
    number(u64::from(f.height));
    serial::write(b":");
    number(u64::from(f.stride));
    serial::write(b":");
    number(u64::from(f.pixel_format));
    serial::write(b"\nG0:MAPPINGS:PASS\n");
    let mut line = TextLine::<48>::new();
    let mut keyboard = Keyboard::new();
    keyboard_setup();
    redraw(geometry, &line);
    serial::write(b"G0:READY\n");
    loop {
        let status = read(0x64);
        if status & 1 == 0 {
            core::hint::spin_loop();
            continue;
        }
        let byte = read(0x60);
        if status & 0xe0 != 0 {
            keyboard.reset();
            continue;
        }
        if let Some(event) = keyboard.feed(byte) {
            if event.pressed && event.code == KeyCode::Escape {
                serial::write(b"G0:PASS\nM1:PASS\n");
                serial::exit(0x10);
            }
            if line.apply(event) == EditResult::Changed {
                redraw(geometry, &line);
                serial::write(b"G0:INPUT:");
                number(line.len() as u64);
                serial::write(b"\n");
            }
        }
    }
}
