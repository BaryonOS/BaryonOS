#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
mod common;
use common::*;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    main()
}
fn main() -> ! {
    boot();
    output_denied(0x80, word(PEER_DISPLAY), -4, CAP_DENIED, 10);
    output_denied(0x84, word(PEER_INPUT), -4, CAP_DENIED, 12);
    denied(0x82, [word(PEER_INPUT), 0, 0, 0, 0, 0], -4, CAP_DENIED, 14);
    output_denied(0x80, word(WRONG_TYPE), -6, CAP_DENIED, 16);
    denied(0x82, [word(WRONG_TYPE), 0, 0, 0, 0, 0], -6, CAP_DENIED, 18);
    output_denied(0x80, 0, -4, CAP_DENIED, 20);
    put(STAGE, 1);
    wait();
    finish()
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(999)
}
