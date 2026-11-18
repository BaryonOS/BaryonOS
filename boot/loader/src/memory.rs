//! Compiler memory intrinsics, explicitly implemented with instructions to prevent
//! optimizer-created recursive calls. All callers obey the C intrinsic contracts.
use core::arch::asm;

#[no_mangle]
pub unsafe extern "C" fn memcpy(
    dst: *mut core::ffi::c_void,
    src: *const core::ffi::c_void,
    n: usize,
) -> *mut core::ffi::c_void {
    let dst = dst.cast::<u8>();
    let src = src.cast::<u8>();
    // SAFETY: Caller supplies n readable/writable, valid nonoverlapping bytes;
    // rep movsb touches exactly n bytes, with DF cleared and no extra alignment.
    unsafe {
        asm!("cld", "rep movsb", inout("rdi") dst => _, inout("rsi") src => _, inout("rcx") n => _, options(nostack));
    }
    dst.cast()
}

#[no_mangle]
pub unsafe extern "C" fn memset(
    dst: *mut core::ffi::c_void,
    value: i32,
    n: usize,
) -> *mut core::ffi::c_void {
    let dst = dst.cast::<u8>();
    // SAFETY: Caller supplies n writable bytes; the operation writes exactly n.
    unsafe {
        asm!("cld", "rep stosb", inout("rdi") dst => _, in("al") value as u8, inout("rcx") n => _, options(nostack));
    }
    dst.cast()
}

#[no_mangle]
pub unsafe extern "C" fn memmove(
    dst: *mut core::ffi::c_void,
    src: *const core::ffi::c_void,
    n: usize,
) -> *mut core::ffi::c_void {
    let dst = dst.cast::<u8>();
    let src = src.cast::<u8>();
    if n == 0 {
        return dst.cast();
    }
    if (dst as usize) <= (src as usize) {
        // SAFETY: Forward order is correct when dst precedes src, even with overlap.
        unsafe {
            asm!("cld", "rep movsb", inout("rdi") dst => _, inout("rsi") src => _, inout("rcx") n => _, options(nostack));
        }
    } else {
        // SAFETY: Valid ranges are required by the intrinsic contract. Starting
        // at the final byte and moving backwards preserves overlapping source.
        unsafe {
            asm!("std", "rep movsb", "cld", inout("rdi") dst.add(n - 1) => _,
                inout("rsi") src.add(n - 1) => _, inout("rcx") n => _, options(nostack));
        }
    }
    dst.cast()
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(
    a: *const core::ffi::c_void,
    b: *const core::ffi::c_void,
    n: usize,
) -> i32 {
    let a = a.cast::<u8>();
    let b = b.cast::<u8>();
    for i in 0..n {
        // SAFETY: Caller guarantees both ranges readable for n bytes; volatile
        // byte reads prevent the compiler recognizing this loop as memcmp.
        let (x, y) = unsafe { (a.add(i).read_volatile(), b.add(i).read_volatile()) };
        if x != y {
            return i32::from(x) - i32::from(y);
        }
    }
    0
}
