//! G0 framebuffer and polled PS/2 hardware boundary; not a desktop or M7.
//! One CPU, IF0 and synchronous calls: no user pointers or retained user buffers.
//! The G0 diagnostic renderer is separate from the bounded G1 pixel helpers.
use crate::{
    m2::{paging::Paging, FRAME_COUNT},
    serial,
};
use boot_protocol::{
    graphics::{FramebufferInfo, FRAMEBUFFER_WINDOW},
    BootInfo, MemoryMap,
};
use core::{
    arch::asm,
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};
use kernel_core::Frames;
use ui_core::{EditResult, Geometry, KeyCode, Keyboard, PixelFormat, TextLine, MAX_ROW_BYTES};

static mut FRAMEBUFFER: Option<FramebufferInfo> = None;
static FRAMEBUFFER_MAPPED: AtomicBool = AtomicBool::new(false);
static INPUT_READY: AtomicBool = AtomicBool::new(false);
static mut ROW: [u8; MAX_ROW_BYTES] = [0; MAX_ROW_BYTES];
pub const MAX_PIXEL_SPAN: usize = 80;

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
        ptr::addr_of_mut!(FRAMEBUFFER).write(Some(framebuffer));
    }
}
fn framebuffer() -> FramebufferInfo {
    // SAFETY: The single IF0 boot owner writes once before M2 starts. Metadata
    // is immutable after capture; this copies it without an outstanding borrow.
    // An early call fails explicitly rather than reading uninitialized memory.
    unsafe { ptr::addr_of!(FRAMEBUFFER).read() }.unwrap_or_else(|| serial::fail(b"G0_NOT_CAPTURED"))
}
/// A kernel-owned copy only. The syscall adapter must expose only public
/// geometry, never this descriptor's physical base or the supervisor window.
pub fn descriptor() -> FramebufferInfo {
    framebuffer()
}
pub fn map(paging: &mut Paging, frames: &mut Frames<FRAME_COUNT>, memory: &MemoryMap<'_>) {
    paging
        .map_framebuffer(frames, &framebuffer(), memory)
        .unwrap_or_else(|_| serial::fail(b"G0_MAPPING"));
    FRAMEBUFFER_MAPPED.store(true, Ordering::Release);
}

/// Validate the whole span before a single store. Canonical app colors are
/// 0x00RRGGBB; neither a stride-padding column nor another row is addressable.
fn pixel_span(f: &FramebufferInfo, x: u32, y: u32, pixels: &[u32]) -> Result<u64, ()> {
    f.validate().map_err(|_| ())?;
    if pixels.is_empty()
        || pixels.len() > MAX_PIXEL_SPAN
        || x >= f.width
        || y >= f.height
        || pixels.len() > (f.width - x) as usize
        || pixels.iter().any(|pixel| pixel & 0xff00_0000 != 0)
    {
        return Err(());
    }
    let start = u64::from(y)
        .checked_mul(u64::from(f.stride))
        .and_then(|n| n.checked_add(u64::from(x)))
        .and_then(|n| n.checked_mul(4))
        .ok_or(())?;
    let end = start.checked_add(pixels.len() as u64 * 4).ok_or(())?;
    if end > f.byte_len {
        return Err(());
    }
    FRAMEBUFFER_WINDOW.checked_add(start).ok_or(())
}

fn device_pixel(color: u32, format: u32) -> u32 {
    if format == 0 {
        ((color & 0xff) << 16) | (color & 0xff00) | ((color >> 16) & 0xff)
    } else {
        color
    }
}

/// Write one short scanline from already copied private kernel RAM. The caller
/// owns capability and usercopy checks; this function independently bounds all
/// hardware stores. No slice of MMIO or reference to caller data is retained.
pub fn write_pixels(x: u32, y: u32, pixels: &[u32]) -> Result<(), ()> {
    if !FRAMEBUFFER_MAPPED.load(Ordering::Acquire) {
        return Err(());
    }
    let f = framebuffer();
    let start = pixel_span(&f, x, y, pixels)?;
    for (index, pixel) in pixels.iter().copied().enumerate() {
        let destination = start + index as u64 * 4;
        // SAFETY: map() published readiness only after installing the audited
        // supervisor-RW/NX/UC mapping. pixel_span validated the entire slice,
        // visible row, stride and bytes before any store. The window is aligned;
        // one IF0 CPU owns synchronous access and each volatile store is 4 bytes.
        unsafe {
            (destination as *mut u32).write_volatile(device_pixel(pixel, f.pixel_format));
        }
    }
    Ok(())
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
fn redraw(geometry: Geometry, line: &TextLine<48>, rows: core::ops::Range<u32>) {
    // SAFETY: Exactly one IF0 kernel thread owns ROW for this synchronous draw.
    // No callback, interrupt or user can access it, and it is ordinary RAM.
    let row = unsafe { &mut *ptr::addr_of_mut!(ROW) };
    let format = framebuffer().pixel_format;
    let mut pixels = [0u32; MAX_PIXEL_SPAN];
    for y in rows.start..rows.end.min(geometry.height()) {
        ui_core::render_debug_row(geometry, line, y, row).unwrap();
        for x in (0..geometry.width()).step_by(MAX_PIXEL_SPAN) {
            let count = ((geometry.width() - x) as usize).min(MAX_PIXEL_SPAN);
            for (index, pixel) in pixels[..count].iter_mut().enumerate() {
                let offset = (x as usize + index) * 4;
                let native = u32::from_le_bytes(row[offset..offset + 4].try_into().unwrap());
                // Red/blue exchange is its own inverse: the existing renderer
                // emits GOP-order bytes; the shared helper takes 0x00RRGGBB.
                *pixel = device_pixel(native, format);
            }
            write_pixels(x, y, &pixels[..count]).unwrap_or_else(|_| serial::fail(b"G0_PIXEL_SPAN"));
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

/// Idempotent startup on the one IF0 kernel CPU. A later caller must not drain
/// already queued input or issue controller commands in the middle of a key.
pub fn initialize_input() {
    if !INPUT_READY.load(Ordering::Acquire) {
        keyboard_setup();
        INPUT_READY.store(true, Ordering::Release);
    }
}

fn scancode(status: u8, byte: u8) -> Option<u8> {
    if status & 1 == 0 || status & 0x20 != 0 {
        None
    } else if status & 0xc0 != 0 {
        // Controller parity/timeout, or the keyboard's own 0xff error, must
        // reset a user's prefix/modifier decoder; neither is a printable key.
        Some(0xff)
    } else {
        Some(byte)
    }
}

/// At most one status read and one data read; never wait in a syscall.
/// None means no keyboard data (or a discarded AUX byte). Some(0xff) is an
/// explicit stream-error sentinel: the ring-3 decoder must reset its state.
pub fn poll_scancode() -> Option<u8> {
    if !INPUT_READY.load(Ordering::Acquire) {
        return None;
    }
    let status = read(0x64);
    if status & 1 == 0 {
        return None;
    }
    scancode(status, read(0x60))
}
pub fn run() -> ! {
    let f = descriptor();
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
    initialize_input();
    redraw(geometry, &line, 0..geometry.height());
    serial::write(b"G0:READY\n");
    loop {
        let Some(byte) = poll_scancode() else {
            core::hint::spin_loop();
            continue;
        };
        if byte == 0xff {
            keyboard.reset();
            continue;
        }
        if let Some(event) = keyboard.feed(byte) {
            if event.pressed && event.code == KeyCode::Escape {
                serial::write(b"G0:PASS\nM1:PASS\n");
                serial::exit(0x10);
            }
            if line.apply(event) == EditResult::Changed {
                redraw(geometry, &line, ui_core::INPUT_ROWS);
                serial::write(b"G0:INPUT:");
                number(line.len() as u64);
                serial::write(b"\n");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame() -> FramebufferInfo {
        FramebufferInfo {
            magic: boot_protocol::graphics::FRAMEBUFFER_MAGIC,
            base: 0x8000_0000,
            byte_len: 4 * 1024 * 1024,
            width: 800,
            height: 600,
            stride: 832,
            pixel_format: 1,
            reserved: [0; 3],
        }
    }

    #[test]
    fn span_checks_last_pixel_stride_and_exact_extent() {
        let mut f = frame();
        f.byte_len = u64::from(f.stride) * u64::from(f.height) * 4;
        assert_eq!(
            pixel_span(&f, 799, 599, &[0x12_3456]),
            Ok(FRAMEBUFFER_WINDOW + (599 * 832 + 799) * 4)
        );
        assert!(pixel_span(&f, 720, 0, &[0; MAX_PIXEL_SPAN]).is_ok());
        assert!(pixel_span(&f, 800, 0, &[0]).is_err());
        assert!(pixel_span(&f, 799, 0, &[0, 0]).is_err());
        assert!(pixel_span(&f, 0, 600, &[0]).is_err());
    }

    #[test]
    fn span_rejects_oversize_empty_high_bits_and_arithmetic_edges() {
        let f = frame();
        for (x, y, pixels) in [
            (0, 0, &[][..]),
            (0, 0, &[0; MAX_PIXEL_SPAN + 1][..]),
            (u32::MAX, 0, &[0][..]),
            (0, u32::MAX, &[0][..]),
            (0, 0, &[0, 0xff00_0000][..]),
        ] {
            assert!(pixel_span(&f, x, y, pixels).is_err());
        }
    }

    #[test]
    fn span_revalidates_metadata_before_any_hardware_access() {
        for change in [
            |f: &mut FramebufferInfo| f.byte_len = 4,
            |f: &mut FramebufferInfo| f.stride = u32::MAX,
            |f: &mut FramebufferInfo| f.width = u32::MAX,
            |f: &mut FramebufferInfo| f.height = u32::MAX,
            |f: &mut FramebufferInfo| f.pixel_format = 2,
            |f: &mut FramebufferInfo| f.base = u64::MAX,
            |f: &mut FramebufferInfo| f.reserved[0] = 1,
        ] {
            let mut f = frame();
            change(&mut f);
            assert!(pixel_span(&f, 0, 0, &[0]).is_err());
        }
    }

    #[test]
    fn canonical_colors_encode_both_supported_device_formats() {
        assert_eq!(
            device_pixel(0x12_3456, 0).to_le_bytes(),
            [0x12, 0x34, 0x56, 0]
        );
        assert_eq!(
            device_pixel(0x12_3456, 1).to_le_bytes(),
            [0x56, 0x34, 0x12, 0]
        );
        assert_eq!(device_pixel(0, 0), 0);
        assert_eq!(device_pixel(0xff_ffff, 1), 0xff_ffff);
    }

    #[test]
    fn input_errors_reset_the_stream_and_aux_is_discarded() {
        assert_eq!(scancode(0, 0x1e), None);
        assert_eq!(scancode(0x21, 0x1e), None);
        assert_eq!(scancode(1, 0x1e), Some(0x1e));
        assert_eq!(scancode(1, 0x9e), Some(0x9e));
        assert_eq!(scancode(0x41, 0x1e), Some(0xff));
        assert_eq!(scancode(0x81, 0x1e), Some(0xff));
        assert_eq!(scancode(0xc1, 0x1e), Some(0xff));
        assert_eq!(scancode(1, 0xff), Some(0xff));
    }

    #[test]
    fn helpers_reject_use_before_mapping_or_input_setup() {
        assert_eq!(write_pixels(0, 0, &[0]), Err(()));
        assert_eq!(poll_scancode(), None);
    }
}
