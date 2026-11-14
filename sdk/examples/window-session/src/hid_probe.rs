#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Test-only real IPC consumer for synthetic HID reports. No rendering or USB I/O.
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_display_client::{DisplayClient, NativeTransport as DisplayTransport};
use nieuw_os_input_client::{AckOutcome, Binding, NativeTransport, Offer, Peek, Session};
use nieuw_os_input_core::{
    wire::{self, Operation, Request},
    CancelReason, Error as PolicyError, Event,
};
use nieuw_os_native_input_service::protocol::command_reply;
use nieuw_os_sdk_runtime::{native, Outgoing};
use nieuw_os_window_session::boot::Bootstrap;
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, MouseButtons, MouseEvent, PhysicalKey};

type Input = Session<NativeTransport>;
// Test support words carry observations only. Actual IPC grants come from boot.
const START: u64 = 760;
const COMMAND: u64 = 768;
const DONE: u64 = 776;
const READY: u64 = 784;
const ERROR: u64 = 792;
const EVENTS: u64 = 800;
const CANCELS: u64 = 808;
const SUPERSEDED: u64 = 816;
const DENIED: u64 = 824;

fn fail(code: u64) -> ! {
    support::put(ERROR, code);
    native::exit(code)
}
fn require(ok: bool, code: u64) {
    if !ok {
        fail(code)
    }
}
fn tick() -> u64 {
    native::monotonic_ticks().unwrap_or_else(|_| fail(1))
}
fn yield_now() {
    native::yield_now().unwrap_or_else(|_| fail(2));
}
fn command(boot: &Bootstrap, id: u64, operation: Operation, arg0: u64) -> u64 {
    let request = Request {
        operation,
        id,
        epoch: boot.policy_epoch,
        arg0,
        arg1: 0,
    };
    let outgoing = Outgoing::new(wire::TAG, &request.encode()).unwrap_or_else(|_| fail(3));
    let started = tick();
    loop {
        match native::send(boot.focus, &outgoing) {
            Ok(()) => break,
            Err(error) if error.is_again() && tick().saturating_sub(started) < 200 => yield_now(),
            Err(_) => fail(4),
        }
    }
    loop {
        match native::receive(boot.focus_reply) {
            Ok(reply) => {
                return u64::from(
                    command_reply(&reply, boot.policy_pid, request).unwrap_or_else(|_| fail(5)),
                )
            }
            Err(error) if error.is_again() && tick().saturating_sub(started) < 200 => yield_now(),
            Err(_) => fail(6),
        }
    }
}
fn take(input: &mut Input, expected: Event) -> Offer {
    let offer = match input.peek().unwrap_or_else(|_| fail(7)) {
        Peek::Offer(value) => value,
        Peek::Empty => fail(8),
    };
    require(offer.event() == expected, 9);
    support::put(EVENTS, support::word(EVENTS) + 1);
    if matches!(expected, Event::Cancel(_)) {
        support::put(CANCELS, support::word(CANCELS) + 1);
    }
    offer
}
fn consume(input: &mut Input, expected: Event) {
    let offer = take(input, expected);
    require(input.ack(&offer) == Ok(AckOutcome::Acknowledged), 10);
}
fn empty(input: &mut Input) {
    require(input.peek() == Ok(Peek::Empty), 11);
}
fn key(key: PhysicalKey, control: bool) -> Event {
    Event::Key(KeyboardEvent {
        key,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            left_control: control,
            ..KeyboardModifiers::default()
        },
    })
}
fn mouse(dx: i16, dy: i16, buttons: u8, changed: u8, x: u32, y: u32) -> Event {
    Event::Mouse {
        input: MouseEvent {
            dx,
            dy,
            buttons: MouseButtons::checked(buttons).unwrap_or_else(|| fail(12)),
            changed: MouseButtons::checked(changed).unwrap_or_else(|| fail(12)),
            x_overflow: false,
            y_overflow: false,
        },
        x,
        y,
    }
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    // The fixed manifest's unused roles run this ELF in passive mode. No role
    // acquires a controller identity or endpoint by choosing these bytes.
    if support::word(32) != u64::from_le_bytes(*b"WINCOMP1") {
        loop {
            yield_now();
        }
    }
    let words = core::array::from_fn(|i| support::word(i as u64 * 8));
    let boot = Bootstrap::decode(&words).unwrap_or_else(|_| fail(13));
    let geometry = DisplayClient::new(DisplayTransport, boot.display)
        .geometry()
        .unwrap_or_else(|_| fail(14));
    let mut input = Input::new(
        NativeTransport,
        Binding {
            epoch: boot.policy_epoch,
            service_pid: boot.policy_pid,
            client_id: 100,
            width: geometry.width,
            height: geometry.height,
            send: boot.input,
            receive: boot.input_reply,
        },
        100_000,
    )
    .unwrap_or_else(|_| fail(15));
    support::put(READY, 1);
    while support::word(START) == 0 {
        yield_now();
    }
    // The observer allows focus only after the source's real SYNC ACK. This
    // prevents a startup race from injecting an unrelated resync cancellation.
    require(command(&boot, 1, Operation::Focus, 100) == 0, 16);
    empty(&mut input);
    support::put(READY, 2);
    let mut done = 0;
    let mut old: Option<Offer> = None;
    loop {
        let next = support::word(COMMAND);
        if next == done {
            yield_now();
            continue;
        }
        require(next == done + 1 && next <= 7, 17);
        match next {
            1 => {
                consume(&mut input, key(PhysicalKey::LeftControl, true));
                consume(&mut input, key(PhysicalKey::A, true));
                empty(&mut input);
            }
            2 => old = Some(take(&mut input, mouse(5, -7, 1, 1, 5, 7))),
            3 => {
                require(command(&boot, 2, Operation::Focus, 0) == 0, 18);
                require(command(&boot, 3, Operation::Focus, 100) == 0, 19);
                require(
                    input.ack(&old.take().unwrap_or_else(|| fail(20)))
                        == Ok(AckOutcome::Superseded),
                    21,
                );
                support::put(SUPERSEDED, 1);
                consume(&mut input, Event::Cancel(CancelReason::FocusLost));
                empty(&mut input);
            }
            4 => {
                consume(&mut input, Event::Cancel(CancelReason::Resync));
                consume(&mut input, mouse(1, 0, 0, 0, 6, 7));
                empty(&mut input); // Held Control+A cannot cross focus/reset.
            }
            5 => {
                consume(&mut input, Event::Cancel(CancelReason::SourceLost));
                consume(&mut input, mouse(1, 0, 0, 0, 7, 7));
                empty(&mut input); // Keyboard detach did not rearm held mouse.
            }
            6 => {
                consume(&mut input, key(PhysicalKey::B, false));
                empty(&mut input);
            }
            7 => {
                consume(&mut input, Event::Cancel(CancelReason::SourceLost));
                empty(&mut input); // Failed reports and held B did not publish.
                require(
                    command(
                        &boot,
                        4,
                        Operation::HidKeyboard,
                        u64::from_le_bytes([0, 0, 4, 0, 0, 0, 0, 0]),
                    ) == PolicyError::WrongRole as u64,
                    22,
                );
                support::put(DENIED, 1);
                empty(&mut input);
            }
            _ => fail(23),
        }
        done = next;
        support::put(DONE, done);
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}
