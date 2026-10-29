#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#[allow(dead_code)]
mod protocol;
mod support;
use nieuw_os_sdk_runtime::{native, SendEndpoint};
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    if support::word(0) == 1 {
        // SAFETY: Deliberate #UD in this isolated acceptance worker. It touches
        // no memory and the kernel records the real fault and reclaims it.
        unsafe {
            core::arch::asm!("ud2", options(noreturn));
        }
    }
    protocol::send(
        SendEndpoint::from_handle(protocol::handle(56)),
        protocol::WORKER_READY,
    );
    loop {
        protocol::tick();
        let _ = native::yield_now();
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    protocol::fail(99)
}
