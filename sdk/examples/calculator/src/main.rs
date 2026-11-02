#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Actual CPL3 executable. Only six endpoint grants, no document/Key/network/
//! raw-input/display grant, host backend, heap allocator or prepared results.
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_calculator::{app::App, bootstrap::Bootstrap};
use nieuw_os_input_client::{Binding, NativeTransport, Session as Input};
use nieuw_os_sdk_runtime::{native, Outgoing, Received, SendEndpoint};
use nieuw_os_surface_protocol::client::Session as Surface;
const TAG: u64 = 0x5359_5354_454d_0001;
const POLLS: usize = 100_000;
core::arch::global_asm!(
    r#"
.section .text.entry,"ax",@progbits
.global _start
_start:
    cld
    mov rax, rsp
    and eax, 15
    cmp eax, 8
    jne .Lcalculator_bad_entry
    jmp calculator_main
.Lcalculator_bad_entry:
    ud2
"#
);
fn fail(step: u64) -> ! {
    support::put(240, step);
    native::exit(1)
}
fn send(to: SendEndpoint, op: u8) {
    let message = Outgoing::new(TAG, &[op]).unwrap_or_else(|_| fail(90));
    for _ in 0..POLLS {
        match native::send(to, &message) {
            Ok(()) => return,
            Err(error) if error.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => fail(91),
        }
    }
    fail(92)
}
fn control(config: &Bootstrap) -> Option<Received> {
    match native::receive(config.control) {
        Ok(message) => {
            if message.cap_count() != 0 {
                let _ = native::close_received_caps(&message);
                fail(93)
            }
            if message.sender() != config.init_pid
                || message.tag() != TAG
                || message.payload().len() != 1
            {
                fail(94)
            }
            Some(message)
        }
        Err(error) if error.is_again() => None,
        Err(_) => fail(95),
    }
}
fn done(config: &Bootstrap) -> ! {
    send(config.registry, 3);
    let control = native::close(config.control.handle());
    let registry = native::close(config.registry.handle());
    if control.is_err() || registry.is_err() {
        fail(11)
    }
    support::put(40, 1);
    native::exit(0)
}
#[no_mangle]
pub extern "C" fn calculator_main() -> ! {
    native::check_abi().unwrap_or_else(|_| fail(1));
    let mut config = Bootstrap::read(support::word).unwrap_or_else(|| fail(2));
    send(config.registry, 1);
    let mut go = false;
    for _ in 0..POLLS {
        if let Some(message) = control(&config) {
            if message.payload() == [5] {
                config
                    .close_unstarted(native::close)
                    .unwrap_or_else(|_| fail(10));
                done(&config);
            }
            if message.payload() != [2] {
                fail(3)
            }
            go = true;
            break;
        }
        let _ = native::yield_now();
    }
    if !go {
        fail(4)
    }
    let mut surface = Surface::new(
        NativeTransport,
        config.compositor_pid,
        config.surface_epoch,
        config.surface_send,
        config.surface_receive,
        POLLS,
    )
    .unwrap_or_else(|_| fail(5));
    let geometry = surface.geometry().unwrap_or_else(|_| fail(6));
    let input = Input::new(
        NativeTransport,
        Binding {
            epoch: config.input_epoch,
            service_pid: config.policy_pid,
            client_id: config.input_client,
            width: geometry.width,
            height: geometry.height,
            send: config.input_send,
            receive: config.input_receive,
        },
        POLLS,
    )
    .unwrap_or_else(|_| fail(7));
    let mut app = App::new(input, surface, geometry, config.scale).unwrap_or_else(|_| fail(8));
    let window_session = support::word(656) == 1;
    let mut paused = false;
    loop {
        if !paused {
            match app.step() {
                Err(
                    nieuw_os_calculator::app::Error::Counter
                    | nieuw_os_calculator::app::Error::Binding,
                ) => fail(12),
                Err(nieuw_os_calculator::app::Error::Surface(_)) if !window_session => fail(12),
                _ => {}
            }
        }
        let e = app.evidence();
        let model = app.model();
        support::put(400, 0);
        for (at, value) in [
            (408, e.frames),
            (416, e.tiles),
            (424, e.keys),
            (432, e.mice),
            (440, e.input_acks),
            (448, e.errors),
            (456, e.presented_generation),
            (464, model.evaluations()),
            (472, model.failures()),
            (480, model.expression().len() as u64),
            (488, u64::from(model.current())),
            (496, u64::from(model.answer().is_some_and(|a| a.rounded))),
            (504, model.caret() as u64),
            (512, u64::from(model.active())),
            (632, u64::from(config.scale)),
        ] {
            support::put(at, value);
        }
        let raw = model
            .answer()
            .map_or(0, |answer| answer.value.raw())
            .to_le_bytes();
        support::put(520, u64::from_le_bytes(raw[..8].try_into().unwrap()));
        support::put(528, u64::from_le_bytes(raw[8..].try_into().unwrap()));
        support::put(536, u64::from(model.answer().is_some()));
        support::put(400, u64::from(e.ready));
        if let Some(message) = control(&config) {
            if window_session && message.payload() == [7] && !paused {
                app.detach_presentation().unwrap_or_else(|_| fail(13));
                paused = true;
                send(config.registry, 7);
                continue;
            }
            if window_session && message.payload() == [8] && paused {
                let next = Bootstrap::read(support::word).unwrap_or_else(|| fail(14));
                if next.registry != config.registry
                    || next.control != config.control
                    || next.self_pid != config.self_pid
                    || next.init_pid != config.init_pid
                    || next.compositor_pid == config.compositor_pid
                {
                    fail(15);
                }
                let mut surface = Surface::new(
                    NativeTransport,
                    next.compositor_pid,
                    next.surface_epoch,
                    next.surface_send,
                    next.surface_receive,
                    POLLS,
                )
                .unwrap_or_else(|_| fail(16));
                let geometry = surface.geometry().unwrap_or_else(|_| fail(17));
                let input = Input::new(
                    NativeTransport,
                    Binding {
                        epoch: next.input_epoch,
                        service_pid: next.policy_pid,
                        client_id: next.input_client,
                        width: geometry.width,
                        height: geometry.height,
                        send: next.input_send,
                        receive: next.input_receive,
                    },
                    POLLS,
                )
                .unwrap_or_else(|_| fail(18));
                app.rebind_presentation(input, surface, geometry)
                    .unwrap_or_else(|_| fail(19));
                config = next;
                paused = false;
                send(config.registry, 8);
                continue;
            }
            if message.payload() != [5] {
                fail(9)
            }
            app.stop_clean().unwrap_or_else(|_| fail(10));
            done(&config);
        }
        let _ = native::yield_now();
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}
