//! GOP is used only in live firmware preparation, before final memory-map freeze.
use super::{aligned, check, Failure, Firmware, Result};
use boot_protocol::graphics::{FramebufferInfo, FRAMEBUFFER_MAGIC};
use core::{ffi::c_void, mem::size_of, ptr};
use r_efi::{efi, protocols::graphics_output as gop};

pub fn prepare(fw: &Firmware) -> Result<FramebufferInfo> {
    let mut raw: *mut c_void = ptr::null_mut();
    let mut guid = gop::PROTOCOL_GUID;
    // SAFETY: Live BootServices; valid GUID and output slot. Firmware owns the
    // protocol and is trusted for its function pointers, only before EBS.
    check(
        unsafe { (fw.services().locate_protocol)(&mut guid, ptr::null_mut(), &mut raw) },
        b"GOP_LOCATE",
    )?;
    let protocol = raw.cast::<gop::Protocol>();
    if !aligned(protocol) {
        return Err(b"GOP_POINTER");
    }
    // SAFETY: The GUID identifies GOP, whose mode pointer is live firmware data.
    let mode = unsafe { (*protocol).mode };
    if !aligned(mode) {
        return Err(b"GOP_MODE_POINTER");
    }
    // SAFETY: Exclusive firmware preparation; copy integer mode count only.
    let count = unsafe { (*mode).max_mode };
    if count == 0 || count > 128 {
        return Err(b"GOP_MODE_COUNT");
    }
    let (requested_width,requested_height)=if cfg!(feature="desktop-hidpi"){(1920,1080)}else if cfg!(feature="desktop-mode"){(1280,720)}else{(800,600)};
    let mut selected = None;
    for number in 0..count {
        let mut size = 0;
        let mut information = ptr::null_mut();
        // SAFETY: Valid live GOP and output slots; QueryMode allocates the info
        // buffer. No allocation or protocol pointer survives the freeze phase.
        let status =
            unsafe { ((*protocol).query_mode)(protocol, number, &mut size, &mut information) };
        if status != efi::Status::SUCCESS {
            continue;
        }
        if !aligned(information) || size < size_of::<gop::ModeInformation>() {
            return Err(b"GOP_QUERY_POINTER");
        }
        // SAFETY: Successful QueryMode initialized a full aligned info record.
        let copy = unsafe { information.read() };
        // SAFETY: QueryMode's allocated buffer belongs to this one call; after
        // copying it there are no references and FreePool receives the exact base.
        check(
            unsafe { (fw.services().free_pool)(information.cast()) },
            b"GOP_FREE",
        )?;
        if copy.version == 0
            && copy.horizontal_resolution == requested_width
            && copy.vertical_resolution == requested_height
            && copy.pixel_format <= 1
        {
            selected = Some(number);
            break;
        }
    }
    let selected = selected.ok_or(b"GOP_REQUESTED_MODE" as Failure)?;
    // SAFETY: Mode came from bounded QueryMode, still before final map capture.
    check(
        unsafe { ((*protocol).set_mode)(protocol, selected) },
        b"GOP_SET_MODE",
    )?;
    // SAFETY: SetMode leaves the protocol and its new mode record live; copy
    // metadata only, never the firmware's BLT callbacks or mode/info pointers.
    let (mode, information) = unsafe {
        let mode = (*protocol).mode;
        if !aligned(mode) {
            return Err(b"GOP_MODE_POINTER");
        }
        let info = (*mode).info;
        if !aligned(info) || (*mode).size_of_info < size_of::<gop::ModeInformation>() {
            return Err(b"GOP_INFO_POINTER");
        }
        ((*mode), *info)
    };
    if mode.mode != selected || information.version != 0 {
        return Err(b"GOP_MODE_CHANGED");
    }
    let result = FramebufferInfo {
        magic: FRAMEBUFFER_MAGIC,
        base: mode.frame_buffer_base,
        byte_len: mode.frame_buffer_size as u64,
        width: information.horizontal_resolution,
        height: information.vertical_resolution,
        stride: information.pixels_per_scan_line,
        pixel_format: information.pixel_format,
        reserved: [0; 3],
    };
    result.validate().map_err(|_| b"GOP_GEOMETRY" as Failure)?;
    Ok(result)
}
