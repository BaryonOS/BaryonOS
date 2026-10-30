#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Real ring3 broker/IPC calls only; no raw ports or host input implementation.
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_display_client::{InputClient, NativeTransport};
use nieuw_os_native_input_service::{
    boot::{self, SourceBoot},
    source::{Progress, Source},
    Ipc, NativeIpc,
};
use nieuw_os_sdk_runtime::{native, Outgoing};

fn words() -> [u64; 128] {
    core::array::from_fn(|index| support::word(index as u64 * 8))
}
fn close_all(boot: &SourceBoot) {
    let handles = [
        boot.input.handle(),
        boot.configuration.request.handle(),
        boot.configuration.response.handle(),
        boot.registry.handle(),
    ];
    for (index, handle) in handles.iter().enumerate() {
        if !handles[..index].contains(handle) {
            let _ = native::close(*handle);
        }
    }
}
fn fail(code: u64, boot: Option<&SourceBoot>) -> ! {
    support::put(472, code);
    if let Some(boot) = boot {
        close_all(boot);
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
    let boot = SourceBoot::decode(&words()).unwrap_or_else(|_| fail(1, None));
    if native::abi_info() != Ok((1 << 32) | 8) {
        fail(2, Some(&boot));
    }
    let mut source = Source::new(boot.configuration).unwrap_or_else(|_| fail(3, Some(&boot)));
    let mut raw = InputClient::new(NativeTransport, boot.input);
    let mut io = NativeIpc;
    let mut pending_ready: Option<Outgoing> = None;
    let mut registered = false;
    let mut ready_started = 0;
    loop {
        let now = native::monotonic_ticks().unwrap_or_else(|_| fail(4, Some(&boot)));
        if let Some(message) = &pending_ready {
            if now.saturating_sub(ready_started)
                > nieuw_os_native_input_service::source::EXCHANGE_TICKS
            {
                fail(9, Some(&boot));
            }
            match io.send(boot.registry, message) {
                Ok(()) => {
                    pending_ready = None;
                    registered = true;
                    support::put(456, 1);
                }
                Err(error) if error.is_again() => {}
                Err(_) => fail(5, Some(&boot)),
            }
        } else {
            match source.step(now, &mut raw, &mut io) {
                Ok(Progress::Ready { raw_epoch }) => {
                    support::put(464, raw_epoch);
                    if !registered {
                        ready_started = now;
                        pending_ready = Some(
                            boot::ready(1, boot.configuration.epoch, boot.generation, raw_epoch)
                                .unwrap_or_else(|_| fail(6, Some(&boot))),
                        );
                    }
                }
                Ok(_) => {}
                Err(_) => fail(7, Some(&boot)),
            }
        }
        let c = source.counts();
        for (offset, value) in [
            (400, c.reset_started),
            (408, c.reset_ready),
            (416, c.sync_acked),
            (424, c.samples_read),
            (432, c.samples_acked),
            (440, c.send_again),
            (448, c.recoveries),
            (480, now),
        ] {
            support::put(offset, value);
        }
        if native::yield_now().is_err() {
            fail(8, Some(&boot));
        }
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(472, 99);
    native::exit(99)
}
