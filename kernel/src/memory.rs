//! Compiler memory intrinsics, explicitly implemented with instructions to prevent
//! optimizer-created recursive calls. All callers obey the C intrinsic contracts.
use core::arch::asm;

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // SAFETY: Caller supplies n readable/writable, valid nonoverlapping bytes;
    // rep movsb touches exactly n bytes, with DF cleared and no extra alignment.
    unsafe {
        asm!("cld", "rep movsb", inout("rdi") dst => _, inout("rsi") src => _, inout("rcx") n => _, options(nostack));
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, value: i32, n: usize) -> *mut u8 {
    // SAFETY: Caller supplies n writable bytes; the operation writes exactly n.
    unsafe {
        asm!("cld", "rep stosb", inout("rdi") dst => _, in("al") value as u8, inout("rcx") n => _, options(nostack));
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if n == 0 {
        return dst;
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
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
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
