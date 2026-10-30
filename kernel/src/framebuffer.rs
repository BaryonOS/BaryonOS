//! Bounded GOP framebuffer transport. No renderer, input policy or user pointers.
//! Supervisor-only RW/NX/UC mapping; one IF=0 CPU owns synchronous stores.
use crate::{
    m2::{paging::Paging, FRAME_COUNT},
    serial,
};
use boot_protocol::{
    graphics::{FramebufferInfo, FRAMEBUFFER_WINDOW},
    BootInfo, MemoryMap,
};
use core::{
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};
use kernel_core::Frames;

static mut FRAMEBUFFER: Option<FramebufferInfo> = None;
static FRAMEBUFFER_MAPPED: AtomicBool = AtomicBool::new(false);
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
pub(crate) fn pixel_span(f: &FramebufferInfo, x: u32, y: u32, pixels: &[u32]) -> Result<u64, ()> {
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

pub(crate) fn device_pixel(color: u32, format: u32) -> u32 {
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
/// Preflight one complete scanline without touching device memory. Metadata is
/// immutable after boot; callers may validate all rows before the first store.
pub fn validate_pixels(x: u32, y: u32, pixels: &[u32]) -> Result<(), ()> {
    if !FRAMEBUFFER_MAPPED.load(Ordering::Acquire) {
        return Err(());
    }
    pixel_span(&framebuffer(), x, y, pixels).map(|_| ())
}
