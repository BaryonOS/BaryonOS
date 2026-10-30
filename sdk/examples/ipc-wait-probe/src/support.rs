use core::arch::global_asm;

const STARTUP: u64 = 0x40010000;

global_asm!(
    r#"
.section .app.startup,"aw",@progbits
.balign 8
.global app_startup
app_startup:
.zero 1024
"#
);

pub fn word(offset: u64) -> u64 {
    assert!(offset < 1024 && offset.is_multiple_of(8));
    // SAFETY: The existing Materiaal linker reserves this private startup page.
    unsafe { ((STARTUP + offset) as *const u64).read_volatile() }
}

pub fn put(offset: u64, value: u64) {
    assert!(offset < 1024 && offset.is_multiple_of(8));
    // SAFETY: Aligned scalar access inside the app's private startup page.
    unsafe { ((STARTUP + offset) as *mut u64).write_volatile(value) }
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        // SAFETY: The intrinsic caller supplies valid nonoverlapping ranges.
        unsafe { dst.add(i).write_volatile(src.add(i).read_volatile()) }
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if dst as usize <= src as usize {
        for i in 0..n {
            // SAFETY: Caller supplies valid ranges; ascending order handles overlap.
            unsafe { dst.add(i).write_volatile(src.add(i).read_volatile()) }
        }
    } else {
        for i in (0..n).rev() {
            // SAFETY: Descending order preserves bytes in the other overlap direction.
            unsafe { dst.add(i).write_volatile(src.add(i).read_volatile()) }
        }
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, value: i32, n: usize) -> *mut u8 {
    for i in 0..n {
        // SAFETY: The intrinsic caller supplies n writable bytes.
        unsafe { dst.add(i).write_volatile(value as u8) }
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    for i in 0..n {
        // SAFETY: The intrinsic caller supplies n readable bytes in both ranges.
        let (x, y) = unsafe { (a.add(i).read_volatile(), b.add(i).read_volatile()) };
        if x != y {
            return i32::from(x) - i32::from(y);
        }
    }
    0
}
