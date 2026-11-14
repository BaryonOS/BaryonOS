#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Test-only synthetic USB completions; publication and replies use real ring3 IPC.
//! No USB controller, hardware completion, DMA or physical device is exercised.
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_input_core::wire::Operation;
use nieuw_os_native_input_service::{
    boot::SourceBoot,
    hid::{Error, Progress, Source},
    NativeIpc,
};
use nieuw_os_sdk_runtime::native;
use nieuw_os_usb_core::{
    boot_interface::{Protocol, Speed},
    boot_session::{CompletionStatus, Error as UsbError, Lifetime, Session, Token},
};

// Private fixture support words, not a public ABI or capability grant.
const COMMAND: u64 = 768;
const DONE: u64 = 776;
const READY: u64 = 784;
const ERROR: u64 = 792;
const PUBLICATIONS: u64 = 800;
const REJECTIONS: u64 = 808;
const HELD_A: [u8; 8] = [1, 0, 4, 0, 0, 0, 0, 0];
const HELD_B: [u8; 8] = [0, 0, 5, 0, 0, 0, 0, 0];

fn fail(code: u64) -> ! {
    support::put(ERROR, code);
    native::exit(code)
}
fn tick() -> u64 {
    native::monotonic_ticks().unwrap_or_else(|_| fail(1))
}
fn require(ok: bool, code: u64) {
    if !ok {
        fail(code)
    }
}
fn rejected(result: Result<(), Error>, expected: UsbError) {
    require(result == Err(Error::Session(expected)), 2);
    support::put(REJECTIONS, support::word(REJECTIONS) + 1);
}
fn protocol(session: &mut Session) {
    let request = session.begin_protocol().unwrap_or_else(|_| fail(3));
    // Explicit synthetic completion of SET_PROTOCOL(boot), including zero data.
    session
        .complete_protocol(request.token, CompletionStatus::Success, 0)
        .unwrap_or_else(|_| fail(4));
}
fn session(protocol_byte: u8, generation: u64) -> Session {
    let configuration = [
        9,
        2,
        34,
        0,
        1,
        1,
        0,
        0x80,
        50,
        9,
        4,
        0,
        0,
        1,
        3,
        1,
        protocol_byte,
        0,
        9,
        0x21,
        0x11,
        1,
        0,
        1,
        0x22,
        63,
        0,
        7,
        5,
        0x81,
        3,
        8,
        0,
        10,
    ];
    let mut value = Session::new(
        Lifetime {
            device: u64::from(protocol_byte),
            generation,
        },
        &configuration,
        0,
        Speed::Full,
    )
    .unwrap_or_else(|_| fail(5));
    protocol(&mut value);
    value
}
fn finish(source: &mut Source, operation: Operation) {
    loop {
        match source.step(tick(), &mut NativeIpc) {
            Ok(Progress::Acknowledged(actual)) if actual == operation => {
                support::put(PUBLICATIONS, support::word(PUBLICATIONS) + 1);
                return;
            }
            Ok(Progress::Pending) => {
                native::yield_now().unwrap_or_else(|_| fail(6));
            }
            _ => fail(7),
        }
    }
}
fn complete(source: &mut Source, session: &mut Session, token: Token, bytes: &[u8]) {
    source
        .complete_input(tick(), session, token, CompletionStatus::Success, bytes)
        .unwrap_or_else(|_| fail(8));
    finish(
        source,
        match session.binding().protocol {
            Protocol::Keyboard => Operation::HidKeyboard,
            Protocol::Mouse => Operation::HidMouse,
        },
    );
}
fn publish(source: &mut Source, session: &mut Session, bytes: &[u8]) -> Token {
    let input = session.begin_input().unwrap_or_else(|_| fail(9));
    complete(source, session, input.token, bytes);
    input.token
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    let words = core::array::from_fn(|i| support::word(i as u64 * 8));
    let boot = SourceBoot::decode(&words).unwrap_or_else(|_| fail(10));
    let mut source = Source::new(boot.configuration).unwrap_or_else(|_| fail(11));
    let mut keyboard = session(1, 1);
    let mut mouse = session(2, 1);
    let mut initial: Option<Token> = None;
    source.synchronize(tick()).unwrap_or_else(|_| fail(12));
    finish(&mut source, Operation::Sync);
    support::put(READY, 1);
    let mut done = 0;
    loop {
        let command = support::word(COMMAND);
        if command == done {
            native::yield_now().unwrap_or_else(|_| fail(13));
            continue;
        }
        require(command == done + 1 && command <= 7, 14);
        match command {
            1 => initial = Some(publish(&mut source, &mut keyboard, &HELD_A)),
            2 => {
                publish(&mut source, &mut mouse, &[1, 5, 7]);
            }
            3 => {} // The controller changes focus before the next report.
            4 => {
                let old = keyboard.begin_input().unwrap_or_else(|_| fail(15));
                keyboard.reset().unwrap_or_else(|_| fail(16));
                require(keyboard.begin_input() == Err(UsbError::NotReady), 17);
                protocol(&mut keyboard);
                source.synchronize(tick()).unwrap_or_else(|_| fail(18));
                finish(&mut source, Operation::Sync);
                let current = keyboard.begin_input().unwrap_or_else(|_| fail(19));
                rejected(
                    source.complete_input(
                        tick(),
                        &mut keyboard,
                        old.token,
                        CompletionStatus::Success,
                        &HELD_A,
                    ),
                    UsbError::UnexpectedCompletion,
                );
                complete(&mut source, &mut keyboard, current.token, &HELD_A);
                publish(&mut source, &mut mouse, &[1, 1, 0]);
            }
            5 => {
                source
                    .detach(tick(), &mut keyboard)
                    .unwrap_or_else(|_| fail(20));
                finish(&mut source, Operation::HidDetach);
                publish(&mut source, &mut mouse, &[1, 1, 0]);
            }
            6 => {
                // Old detach was acknowledged before reusing the role. The new
                // input sequence equals the old first input sequence; only its
                // owner-assigned generation distinguishes the replayed token.
                keyboard = session(1, 2);
                let current = keyboard.begin_input().unwrap_or_else(|_| fail(21));
                rejected(
                    source.complete_input(
                        tick(),
                        &mut keyboard,
                        initial.unwrap_or_else(|| fail(22)),
                        CompletionStatus::Success,
                        &HELD_A,
                    ),
                    UsbError::UnexpectedCompletion,
                );
                complete(&mut source, &mut keyboard, current.token, &HELD_B);
            }
            7 => {
                let input = keyboard.begin_input().unwrap_or_else(|_| fail(23));
                rejected(
                    source.complete_input(
                        tick(),
                        &mut keyboard,
                        input.token,
                        CompletionStatus::Failed,
                        &HELD_A,
                    ),
                    UsbError::TransferFailed,
                );
                for (bytes, expected) in [
                    (
                        &[][..],
                        UsbError::Report(nieuw_os_usb_core::boot::Error::Length),
                    ),
                    (&[0; 9][..], UsbError::Length),
                ] {
                    let input = keyboard.begin_input().unwrap_or_else(|_| fail(24));
                    rejected(
                        source.complete_input(
                            tick(),
                            &mut keyboard,
                            input.token,
                            CompletionStatus::Success,
                            bytes,
                        ),
                        expected,
                    );
                }
                let input = keyboard.begin_input().unwrap_or_else(|_| fail(25));
                let malformed = source.complete_input(
                    tick(),
                    &mut keyboard,
                    input.token,
                    CompletionStatus::Success,
                    &[0, 0, 5, 5, 0, 0, 0, 0],
                );
                require(
                    malformed
                        == Err(Error::Session(UsbError::Report(
                            nieuw_os_usb_core::boot::Error::DuplicateKey(5),
                        ))),
                    26,
                );
                support::put(REJECTIONS, support::word(REJECTIONS) + 1);
                require(
                    source.step(tick(), &mut NativeIpc) == Ok(Progress::Idle),
                    27,
                );
                source
                    .detach(tick(), &mut mouse)
                    .unwrap_or_else(|_| fail(28));
                finish(&mut source, Operation::HidDetach);
                publish(&mut source, &mut keyboard, &HELD_B);
            }
            _ => fail(29),
        }
        done = command;
        support::put(DONE, done);
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}
