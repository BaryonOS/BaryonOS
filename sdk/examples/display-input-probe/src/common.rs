#![allow(dead_code)]
use core::arch::asm;
use nieuw_os_sdk_runtime::{native, Handle};
#[path = "contract.rs"]
pub mod contract;
#[path = "../../system/src/support.rs"]
pub mod support;
pub use contract::*;
pub use support::{put, word};

pub fn fail(step: u64) -> ! {
    put(FAILURE, step);
    native::exit(1)
}
pub fn check(ok: bool, step: u64) {
    if !ok {
        fail(step);
    }
}
pub fn bump(offset: u64) {
    put(
        offset,
        word(offset).checked_add(1).unwrap_or_else(|| fail(900)),
    );
}
pub fn handle(offset: u64) -> Handle {
    Handle::from_raw(word(offset)).unwrap_or_else(|_| fail(901))
}

/// Deliberately raw negative-wire injector. Normal operations use the typed SDK;
/// malformed pointers/arguments must actually cross the native kernel boundary.
pub fn raw(number: u64, args: [u64; 6]) -> (i64, u64) {
    let mut primary = number;
    let mut secondary = args[2];
    // SAFETY: Ring-3 Nieuw OS ABI1.8 preserves other GPRs and checks pointers.
    // Integer arguments do not form Rust references. There is no host build of
    // these binaries, no nomem promise, and no pointer retained past the call.
    unsafe {
        asm!("int 0x80", inout("rax") primary, in("rdi") args[0],
            in("rsi") args[1], inout("rdx") secondary, in("r10") args[3],
            in("r8") args[4], in("r9") args[5], options(nostack));
    }
    (primary as i64, secondary)
}
pub fn denied(number: u64, args: [u64; 6], code: i64, counter: u64, step: u64) {
    check(raw(number, args) == (code, 0), step);
    bump(counter);
}
pub fn boot() {
    check(native::abi_info() == Ok((1u64 << 32) | 8), 1);
    check(matches!(word(GENERATION), 1 | 2), 2);
    for offset in (400..=616).step_by(8) {
        put(offset, 0);
    }
    // The writable prefix belongs to this process's real initial heap; the
    // following page is absent. Only integers reach the malformed call.
    for offset in 0..320 {
        // SAFETY: Fixed final320 bytes of the mapped private64KiB heap, no Rust
        // references to it coexist. Never dereference the following guard page.
        unsafe {
            ((PARTIAL_PIXELS + offset) as *mut u8).write_volatile(if offset % 4 == 3 {
                0
            } else {
                0xee
            });
        }
    }
}
pub fn prefix_unchanged() {
    for offset in 0..16 {
        // SAFETY: Final16 bytes of the same mapped private heap.
        let byte = unsafe { ((BAD_POINTER + offset) as *const u8).read_volatile() };
        check(byte == if (304 + offset) % 4 == 3 { 0 } else { 0xee }, 902);
    }
    bump(UNCHANGED);
}
pub fn output_denied(number: u64, source: u64, code: i64, counter: u64, step: u64) {
    let mut out = [SENTINEL; 32];
    denied(
        number,
        [source, out.as_mut_ptr() as u64, 32, 0, 0, 0],
        code,
        counter,
        step,
    );
    check(out == [SENTINEL; 32], step + 1);
    bump(UNCHANGED);
}
pub fn cap_restrictions(source: Handle, right: u64) {
    denied(
        0x11,
        [source.raw(), right, 0, 0, 0, 0],
        SOURCE_RESTRICTED_ERROR,
        CAP_DENIED,
        30,
    );
    denied(
        0x11,
        [source.raw(), 0, 0, 0, 0, 0],
        SOURCE_RESTRICTED_ERROR,
        CAP_DENIED,
        31,
    );
    denied(
        0x12,
        [source.raw(), 0, 0, 0, 0, 0],
        SOURCE_RESTRICTED_ERROR,
        CAP_DENIED,
        32,
    );
    let mut message = [0u8; 320];
    message[20..24].copy_from_slice(&1u32.to_le_bytes());
    message[32..40].copy_from_slice(&source.raw().to_le_bytes());
    message[40..48].copy_from_slice(&right.to_le_bytes());
    denied(
        0x20,
        [word(WRONG_TYPE), message.as_ptr() as u64, 320, 0, 0, 0],
        SOURCE_RESTRICTED_ERROR,
        CAP_DENIED,
        33,
    );
}
pub fn wait() {
    while word(CONTROL) == 0 {
        bump(8);
        check(native::yield_now().is_ok(), 903);
    }
    check(word(CONTROL) == 1, 904);
}
pub fn finish() -> ! {
    put(DONE, 1);
    native::exit(0)
}
