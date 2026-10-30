#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_native_desktop_init::controller::{Configuration, Controller, Phase};
use nieuw_os_native_input_service::NativeIpc;
use nieuw_os_sdk_runtime::native;
fn fail(code: u64, config: Option<&Configuration>) -> ! {
    support::put(432, code);
    if let Some(config) = config {
        let _ = native::close(config.send.handle());
        let _ = native::close(config.receive.handle());
    }
    native::exit(code)
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    run()
}
#[inline(never)]
fn run() -> ! {
    if !matches!(native::abi_info(), Ok(0x1_0000_0008 | 0x1_0000_000a)) {
        fail(90, None);
    }
    let words = core::array::from_fn(|i| support::word(i as u64 * 8));
    let config = Configuration::decode(&words).unwrap_or_else(|_| fail(91, None));
    let now = native::monotonic_ticks().unwrap_or_else(|_| fail(92, Some(&config)));
    let mut controller =
        Controller::new(config, now).unwrap_or_else(|e| fail(e as u64, Some(&config)));
    let mut io = NativeIpc;
    loop {
        let now = native::monotonic_ticks().unwrap_or_else(|_| fail(93, Some(&config)));
        let result = controller.step(now, support::word(400), &mut io);
        let (sent, validated) = controller.counts();
        let (start, end) = controller.ticks();
        for (o, v) in [
            (408, sent),
            (416, validated),
            (424, controller.phase() as u64),
            (440, start),
            (448, end),
        ] {
            support::put(o, v);
        }
        match result {
            Ok(Phase::Done) => {
                support::put(456, 1);
                let _ = native::close(config.send.handle());
                let _ = native::close(config.receive.handle());
                native::exit(0)
            }
            Err(e) => fail(e as u64, Some(&config)),
            _ => {}
        }
        if native::yield_now().is_err() {
            fail(94, Some(&config));
        }
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99, None)
}
