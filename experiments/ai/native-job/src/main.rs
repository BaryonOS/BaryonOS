//! Native ring-3 SSE2 AI job: runs the decoder fixture repeatedly under PIT
//! preemption and publishes its result bits in the private report page.
//! Built for the hard-float `x86_64-unknown-linux-gnu` ABI as a freestanding
//! static ELF (no libc, no Linux syscalls); see build.py for the instruction gate.
#![no_std]
#![no_main]
#![no_builtins]
#![deny(unsafe_op_in_unsafe_fn)]

use baryon_ai_native_job::{argmax, digest, run, Weights, OUTPUT};
use core::arch::{asm, global_asm};
use core::ffi::c_void;

const DATA: usize = 0x4001_0000;
/// Report byte offsets after the kernel-owned words 0..32.
const STATUS: usize = 64;
const ITERATIONS: usize = 72;
const START_TICKS: usize = 80;
const END_TICKS: usize = 88;
const MISMATCHES: usize = 96;
const MACS: usize = 104;
const ARGMAX: usize = 112;
const DIGEST: usize = 120;
const BITS: usize = 128;
const MIN_ITERATIONS: u64 = 2000;
const MIN_TICKS: u64 = 50;
const YIELD_EVERY: u64 = 64;

global_asm!(
    r#"
    .section .ai.header,"aw",@progbits
    .balign 8
    .global ai_startup
ai_startup:
    .zero 512
"#
);

fn write(offset: usize, value: u64) {
    // SAFETY: fixed aligned words inside our own private 512-byte report header.
    unsafe { ((DATA + offset) as *mut u64).write_volatile(value) }
}

fn call(number: u64) -> Result<u64, i64> {
    let (mut primary, mut secondary) = (number, 0u64);
    // SAFETY: the native int80 ABI with zero arguments; no memory is passed.
    unsafe {
        asm!("int 0x80", inout("rax") primary, in("rdi") 0u64, in("rsi") 0u64,
             inout("rdx") secondary, in("r10") 0u64, in("r8") 0u64, in("r9") 0u64,
             options(nostack));
    }
    match (primary as i64, secondary) {
        (0, value) => Ok(value),
        (code, _) => Err(code),
    }
}

fn fail(code: u64) -> ! {
    write(STATUS, 0x100 + code);
    loop {
        let _ = call(1);
    }
}

fn ticks() -> u64 {
    call(3).unwrap_or_else(|_| fail(1))
}

#[link_section = ".text.entry"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let weights = Weights::new();
    let (mut first, mut again) = ([0f32; OUTPUT], [0f32; OUTPUT]);
    let start = ticks();
    write(START_TICKS, start);
    let macs = run(&weights, &mut first).unwrap_or_else(|_| fail(2));
    let expected = digest(&first);
    let (mut iterations, mut mismatches) = (1u64, 0u64);
    loop {
        if run(&weights, &mut again).unwrap_or_else(|_| fail(3)) != macs
            || digest(&again) != expected
        {
            mismatches += 1;
        }
        iterations += 1;
        write(ITERATIONS, iterations);
        if iterations % YIELD_EVERY == 0 && call(1).is_err() {
            fail(4);
        }
        if iterations >= MIN_ITERATIONS && ticks().saturating_sub(start) >= MIN_TICKS {
            break;
        }
    }
    write(END_TICKS, ticks());
    write(MISMATCHES, mismatches);
    write(MACS, macs);
    write(
        ARGMAX,
        u64::from_le_bytes({
            let mut b = [0u8; 8];
            b[..4].copy_from_slice(&argmax(&first));
            b
        }),
    );
    write(DIGEST, expected);
    for (i, pair) in first.chunks_exact(2).enumerate() {
        write(
            BITS + 8 * i,
            u64::from(pair[0].to_bits()) | (u64::from(pair[1].to_bits()) << 32),
        );
    }
    write(STATUS, 1);
    loop {
        if call(1).is_err() {
            core::hint::spin_loop();
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    fail(0xff)
}

// The hard-float target's precompiled `core` expects these from libc.
// `#![no_builtins]` keeps LLVM from lowering these loops back into calls.
#[no_mangle]
unsafe extern "C" fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    // SAFETY: caller contract of memcpy (valid, non-overlapping ranges).
    unsafe { memmove(dest, src, n) }
}

#[no_mangle]
unsafe extern "C" fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let (d, s) = (dest.cast::<u8>(), src.cast::<u8>());
    // SAFETY: caller contract of memmove (valid ranges); direction handles overlap.
    unsafe {
        if (d as usize) <= (s as usize) {
            for i in 0..n {
                *d.add(i) = *s.add(i);
            }
        } else {
            for i in (0..n).rev() {
                *d.add(i) = *s.add(i);
            }
        }
    }
    dest
}

#[no_mangle]
unsafe extern "C" fn memset(dest: *mut c_void, value: i32, n: usize) -> *mut c_void {
    let d = dest.cast::<u8>();
    for i in 0..n {
        // SAFETY: caller contract of memset (valid writable range).
        unsafe { *d.add(i) = value as u8 };
    }
    dest
}

#[no_mangle]
unsafe extern "C" fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32 {
    let (a, b) = (a.cast::<u8>(), b.cast::<u8>());
    for i in 0..n {
        // SAFETY: caller contract of memcmp (valid readable ranges).
        let (x, y) = unsafe { (*a.add(i), *b.add(i)) };
        if x != y {
            return i32::from(x) - i32::from(y);
        }
    }
    0
}

#[no_mangle]
unsafe extern "C" fn bcmp(a: *const c_void, b: *const c_void, n: usize) -> i32 {
    // SAFETY: same contract as memcmp.
    unsafe { memcmp(a, b, n) }
}

/// Never called: `panic = "abort"`; the precompiled `core` still references it.
#[no_mangle]
extern "C" fn rust_eh_personality() {}
