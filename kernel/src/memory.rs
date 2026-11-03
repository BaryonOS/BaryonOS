//! Compiler memory intrinsics, explicitly implemented with instructions to prevent
//! optimizer-created recursive calls. All callers obey the C intrinsic contracts.
use core::arch::asm;

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // SAFETY: Caller supplies n readable/writable, valid nonoverlapping bytes;
    // x86 permits unaligned word accesses. Only n/8 complete words are copied,
    // then exactly n%8 bytes; neither side is touched beyond its valid range.
    // Separate count/tail registers cannot alias the early-clobbered RSI/RDI/RCX.
    unsafe {
        asm!("cld", "rep movsq", "mov rcx, {tail}", "rep movsb",
            tail = in(reg) n % 8,
            inout("rdi") dst => _, inout("rsi") src => _,
            inout("rcx") n / 8 => _, options(nostack));
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
    // SAFETY: Clear the direction flag even for an empty range; no memory is
    // accessed. The kernel ABI requires DF=0 on return.
    unsafe {
        asm!("cld", options(nomem, nostack));
    }
    let mut i = 0;
    while n - i >= 8 {
        let (x, y): (u64, u64);
        // SAFETY: i+8 <= n, so both exact eight-byte loads stay within the
        // caller's readable regions. Unaligned loads are valid on x86-64. Inline
        // instructions prevent optimizer-recognition/recursive memcmp calls.
        unsafe {
            asm!("mov {x}, qword ptr [{a}]", "mov {y}, qword ptr [{b}]",
                x = out(reg) x, y = out(reg) y,
                a = in(reg) a.add(i), b = in(reg) b.add(i),
                options(readonly, nostack, preserves_flags));
        }
        if x != y {
            // x86-64 is little endian. The least significant differing bit
            // identifies the FIRST differing byte in address order; comparing
            // whole words numerically would give the wrong lexicographic sign.
            let shift = (x ^ y).trailing_zeros() & !7;
            return i32::from((x >> shift) as u8) - i32::from((y >> shift) as u8);
        }
        i += 8;
    }
    while i < n {
        // SAFETY: Only the remaining 0..7 bytes are read. Volatile loads keep
        // this tail from becoming a recursive compiler-created memcmp call.
        let (x, y) = unsafe { (a.add(i).read_volatile(), b.add(i).read_volatile()) };
        if x != y {
            return i32::from(x) - i32::from(y);
        }
        i += 1;
    }
    0
}
