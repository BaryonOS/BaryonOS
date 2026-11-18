//! Explicit compatibility shim for the current fixed startup page (FR-I01 bootmanifest and K05 worker layout). It is
//! not a new OS bootstrap ABI and gives this app no ability to create handles.
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
    // SAFETY: The app linker reserves the fixed private 1024-byte startup area;
    // all callers use aligned in-range offsets. The kernel supplies metadata.
    unsafe { ((STARTUP + offset) as *const u64).read_volatile() }
}

pub fn put(offset: u64, value: u64) {
    assert!(offset < 1024 && offset.is_multiple_of(8));
    // SAFETY: Same app-owned writable startup range, one process thread.
    unsafe {
        ((STARTUP + offset) as *mut u64).write_volatile(value);
    }
}

// Compiler-required memory intrinsics. Volatile scalar accesses prevent these
// definitions from being replaced by recursive calls to themselves.
#[no_mangle]
pub unsafe extern "C" fn memcpy(
    dst: *mut core::ffi::c_void,
    src: *const core::ffi::c_void,
    n: usize,
) -> *mut core::ffi::c_void {
    let dst = dst.cast::<u8>();
    let src = src.cast::<u8>();
    for i in 0..n {
        // SAFETY: C intrinsic caller guarantees n readable/writable bytes and
        // nonoverlap; no pointer outside those ranges is dereferenced.
        unsafe {
            dst.add(i).write_volatile(src.add(i).read_volatile());
        }
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
    if dst as usize <= src as usize {
        for i in 0..n {
            // SAFETY: Caller supplies valid ranges; ascending copy handles this
            // overlap direction without retaining references to either range.
            unsafe {
                dst.add(i).write_volatile(src.add(i).read_volatile());
            }
        }
    } else {
        for i in (0..n).rev() {
            // SAFETY: Descending copy preserves source bytes under overlap.
            unsafe {
                dst.add(i).write_volatile(src.add(i).read_volatile());
            }
        }
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
    for i in 0..n {
        // SAFETY: Caller supplies n writable bytes.
        unsafe {
            dst.add(i).write_volatile(value as u8);
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
        // SAFETY: Caller supplies n readable bytes for both ranges.
        let (x, y) = unsafe { (a.add(i).read_volatile(), b.add(i).read_volatile()) };
        if x != y {
            return i32::from(x) - i32::from(y);
        }
    }
    0
}
