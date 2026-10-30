#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Ring-3 dependency handshake and actual fault/restart/kill lifecycle.
#[allow(dead_code)]
mod protocol;
mod support;
use nieuw_os_sdk_runtime::{
    native, Capability, LaunchImage, ProcessStatus, ReceiveEndpoint, Rights, SendEndpoint,
};
use protocol::*;
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    main()
}

#[inline(never)]
fn main() -> ! {
    check(native::check_abi().is_ok(), 1);
    let image = LaunchImage::from_handle(handle(56));
    let registry = ReceiveEndpoint::from_handle(handle(64));
    let grant = Capability {
        handle: handle(72),
        rights: Rights::SEND,
    };
    let app_control = SendEndpoint::from_handle(handle(80));
    let app = support::word(160);
    // The app reports READY only after real STATUS/STAT round trips to both
    // boot services. Init does not read another process's evidence memory.
    let mut ready = false;
    for _ in 0..LIMIT {
        if let Some(m) = receive(registry) {
            check(m.sender() == app && m.payload() == [READY], 2);
            ready = true;
            break;
        }
    }
    check(ready, 3);
    support::put(240, 4);
    let failed = native::spawn(image, 1, &[grant]).unwrap_or_else(|_| fail(4));
    let mut faulted = false;
    for _ in 0..LIMIT {
        tick();
        match native::status(failed) {
            Ok(ProcessStatus::Faulted(6)) => {
                faulted = true;
                break;
            }
            Ok(ProcessStatus::Running) => {
                let _ = native::yield_now();
            }
            _ => fail(5),
        }
    }
    check(faulted, 6);
    check(native::close(failed.handle()).is_ok(), 7);
    support::put(240, 8);
    let worker = native::spawn(image, 0, &[grant]).unwrap_or_else(|_| fail(8));
    let mut restarted = false;
    for _ in 0..LIMIT {
        if let Some(m) = receive(registry) {
            check(m.sender() != app && m.payload() == [WORKER_READY], 9);
            restarted = true;
            break;
        }
    }
    check(restarted, 10);
    send(app_control, GO);
    support::put(240, 11);
    let mut done = false;
    for _ in 0..LIMIT {
        if let Some(m) = receive(registry) {
            check(m.sender() == app && m.payload() == [DONE], 11);
            done = true;
            break;
        }
    }
    check(done, 12);
    check(native::status(worker) == Ok(ProcessStatus::Running), 13);
    check(native::kill(worker).is_ok(), 14);
    check(native::status(worker) == Ok(ProcessStatus::Killed), 15);
    check(native::close(worker.handle()).is_ok(), 16);
    for (at, value) in [(400, 2), (408, 1), (416, 1), (424, 1), (40, 1)] {
        support::put(at, value);
    }
    native::exit(0)
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}
