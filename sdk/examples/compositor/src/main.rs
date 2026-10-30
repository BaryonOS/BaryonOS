#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_display_client::{DisplayClient, NativeTransport};
use nieuw_os_native_compositor::{
    boot::Bootstrap,
    chrome::Chrome,
    control::{Configuration, Coordinator, State},
    surface::{Fatal, Surface},
};
use nieuw_os_native_input_service::{discard_caps, Ipc, NativeIpc};
use nieuw_os_sdk_runtime::native;

fn words() -> [u64; 128] {
    core::array::from_fn(|index| support::word(index as u64 * 8))
}
fn close_all(boot: &Bootstrap) {
    let handles = boot.handles();
    for (index, handle) in handles.iter().enumerate() {
        if !handles[..index].contains(handle) {
            let _ = native::close(*handle);
        }
    }
}
fn fail(code: u64, boot: Option<&Bootstrap>) -> ! {
    support::put(488, code);
    if let Some(boot) = boot {
        close_all(boot);
    }
    native::exit(code)
}
fn tick(boot: &Bootstrap) -> u64 {
    native::monotonic_ticks().unwrap_or_else(|_| fail(4, Some(boot)))
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    run()
}
#[inline(never)]
fn run() -> ! {
    let boot = Bootstrap::decode(&words()).unwrap_or_else(|_| fail(1, None));
    if !matches!(native::abi_info(), Ok(0x1_0000_0008 | 0x1_0000_000a)) {
        fail(2, Some(&boot));
    }
    let mut display = DisplayClient::new(NativeTransport, boot.display);
    let native = display.geometry().unwrap_or_else(|_| fail(3, Some(&boot)));
    let chrome = Chrome::new(native).unwrap_or_else(|_| fail(5, Some(&boot)));
    let mut bytes = [0u8; 1024];
    let mut initial_border = 0u64;
    for tile in chrome.tiles() {
        let length = chrome
            .render(tile, State::Inactive, &mut bytes)
            .unwrap_or_else(|_| fail(7, Some(&boot)));
        display
            .blit(tile, &bytes[..length])
            .unwrap_or_else(|_| fail(8, Some(&boot)));
        initial_border += 1;
        if initial_border.is_multiple_of(32) && native::yield_now().is_err() {
            fail(11, Some(&boot));
        }
    }
    let mut initial_clear = 0u64;
    for tile in chrome.wipe_tiles() {
        let length = chrome
            .render_wipe(tile, &mut bytes)
            .unwrap_or_else(|_| fail(7, Some(&boot)));
        display
            .blit(tile, &bytes[..length])
            .unwrap_or_else(|_| fail(8, Some(&boot)));
        initial_clear += 1;
        if initial_clear.is_multiple_of(32) && native::yield_now().is_err() {
            fail(11, Some(&boot));
        }
    }
    support::put(400, initial_border);
    support::put(496, initial_clear);
    // The complete native border and app region are now actually overwritten.
    let mut surface = Surface::new(
        display,
        native,
        boot.surface_epoch,
        boot.app_pid,
        boot.surface,
        boot.app_reply,
    )
    .unwrap_or_else(|_| fail(13, Some(&boot)));
    let mut control = Coordinator::new(Configuration {
        init_pid: boot.init_pid,
        surface_epoch: boot.surface_epoch,
        generation: boot.generation,
        policy_pid: boot.policy_pid,
        policy_epoch: boot.policy_epoch,
        client_id: boot.client_id,
        send: boot.focus_send,
        receive: boot.focus_receive,
    })
    .unwrap_or_else(|_| fail(14, Some(&boot)));
    let ready = boot
        .ready(native.epoch)
        .unwrap_or_else(|_| fail(9, Some(&boot)));
    let start = tick(&boot);
    let mut io = NativeIpc;
    loop {
        if tick(&boot).saturating_sub(start) > 200 {
            fail(10, Some(&boot));
        }
        match io.send(boot.registry, &ready) {
            Ok(()) => break,
            Err(error) if error.is_again() => {
                if native::yield_now().is_err() {
                    fail(11, Some(&boot));
                }
            }
            Err(_) => fail(12, Some(&boot)),
        }
    }
    support::put(472, 1);
    let mut previous = State::Inactive;
    let mut tiles = chrome.tiles();
    let mut wiping = None;
    loop {
        let now = tick(&boot);
        if !control.is_busy() {
            match io.receive(boot.control) {
                Ok(message) => {
                    if message.cap_count() != 0 {
                        if discard_caps(&mut io, &message).is_err() {
                            surface.retire(&mut io);
                            fail(15, Some(&boot));
                        }
                    } else {
                        match control.accept(now, &message) {
                            Ok(retire) => {
                                if retire {
                                    surface.retire(&mut io);
                                }
                            }
                            Err(_) => {
                                if control.is_terminal() {
                                    surface.retire(&mut io);
                                    fail(16, Some(&boot));
                                }
                            }
                        }
                    }
                }
                Err(error) if error.is_again() => {}
                Err(_) => {
                    surface.retire(&mut io);
                    fail(17, Some(&boot));
                }
            }
        }
        if control.step(now, &mut io).is_err() {
            surface.retire(&mut io);
            fail(18, Some(&boot));
        }
        if control.state() != previous {
            previous = control.state();
            tiles = chrome.tiles();
            if previous == State::Retired {
                wiping = Some(chrome.wipe_tiles());
            }
        }
        for _ in 0..32 {
            let Some(wipe) = &mut wiping else {
                break;
            };
            if let Some(tile) = wipe.next() {
                if surface.clear(chrome, tile, &mut bytes).is_err() {
                    surface.retire(&mut io);
                    fail(23, Some(&boot));
                }
            } else {
                wiping = None;
                control.finish_retire_wipe(tick(&boot));
                break;
            }
        }
        if wiping.is_none() {
            let now = tick(&boot);
            if control.flush(now, boot.registry, &mut io).is_err() {
                surface.retire(&mut io);
                fail(19, Some(&boot));
            }
        }
        if surface
            .step(now, &mut io, || {
                native::monotonic_ticks().map_err(|_| Fatal::Clock)
            })
            .is_err()
        {
            surface.retire(&mut io);
            fail(20, Some(&boot));
        }
        if let Some(tile) = tiles.next() {
            if surface
                .chrome(chrome, tile, control.state(), &mut bytes)
                .is_err()
            {
                surface.retire(&mut io);
                fail(21, Some(&boot));
            }
        }
        let counts = surface.counts();
        let (focus_sent, focus_acked) = control.counts();
        for (offset, value) in [
            (400, counts.border + initial_border),
            (408, counts.received),
            (416, counts.committed),
            (424, counts.pixels),
            (432, counts.rejected),
            (440, counts.reply_again),
            (448, focus_sent),
            (456, focus_acked),
            (464, surface.completed_frame().unwrap_or(0)),
            (480, control.state() as u64),
            (496, counts.cleared + initial_clear),
        ] {
            support::put(offset, value);
        }
        if native::yield_now().is_err() {
            surface.retire(&mut io);
            fail(22, Some(&boot));
        }
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(488, 99);
    native::exit(99)
}
