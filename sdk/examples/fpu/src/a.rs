#![no_std]
#![no_main]
core::arch::global_asm!(include_str!("probe.S"), role = const 13, control = const 0x077f, mxcsr = const 0x3f80);
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
