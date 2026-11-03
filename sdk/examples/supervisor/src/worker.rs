#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! K05 worker: an ordinary SDK service that a supervisor starts at runtime.
//! It creates its own ingress endpoint, registers it with the supervisor and
//! answers numbered requests from the one client the supervisor names. In
//! crash mode it executes `ud2` after eight answers so the supervisor can
//! observe a real fault and restart it. It holds no block, device, launch or
//! process right of any kind.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("k05-worker is only for the Nieuw OS x86_64-unknown-none target");

mod support;

use k05_protocol::*;
use nieuw_os_sdk_runtime::{
    clock::{self, TimerRegistration, WaitOutcome},
    native, Capability, ChildProcess, Error, Handle, Outgoing, ReceiveEndpoint, Rights,
    SendEndpoint, WaitCondition,
};

const LIMIT: usize = 400_000;

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

fn fail(step: u64) -> ! {
    support::put(240, step);
    support::put(40, 0x100 + step);
    native::exit(1)
}

fn send_bounded(endpoint: SendEndpoint, message: &Outgoing) -> Result<(), Error> {
    for _ in 0..LIMIT {
        tick();
        match native::send(endpoint, message) {
            Ok(()) => return Ok(()),
            Err(error) if error.is_again() => {
                let _ = native::yield_now();
            }
            Err(error) => return Err(error),
        }
    }
    Err(Error::Kernel(-13))
}

fn frame(frame: Frame) -> Outgoing {
    Outgoing::new(TAG, &frame.encode()).unwrap_or_else(|_| fail(90))
}

fn code(result: Result<(), Error>) -> i64 {
    match result {
        Ok(()) => 0,
        Err(Error::Kernel(code)) => code,
        Err(_) => 1,
    }
}

/// Two negative checks a worker can perform without any process authority.
fn denied_checks(root: Handle) -> (u64, u64) {
    let mut matched = 0;
    // An endpoint handle is not a Process object.
    if native::status(ChildProcess::from_handle(root)) == Err(Error::Kernel(ETYPE)) {
        matched += 1;
    }
    // An empty slot of the own table is a bad handle, never someone else's child.
    let empty = Handle::from_raw((1 << 32) | 200).unwrap_or_else(|_| fail(91));
    if code(native::kill(ChildProcess::from_handle(empty))) == EBADHANDLE {
        matched += 1;
    }
    (matched, 2)
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    if native::check_abi().is_err() {
        fail(1);
    }
    let arg = support::word(0);
    let mode = arg & 0xff;
    let supervisor = arg >> 8;
    let pid = support::word(176);
    if supervisor == 0 || pid == 0 || support::word(32) != 1 {
        fail(2);
    }
    let registry =
        SendEndpoint::from_handle(Handle::from_raw(support::word(56)).unwrap_or_else(|_| fail(3)));
    // The second delegated slot must be empty: the spawner passed one cap.
    if support::word(64) != 0 {
        fail(4);
    }
    let root = native::endpoint_create().unwrap_or_else(|_| fail(5));
    let ingress = ReceiveEndpoint::from_handle(root);
    let mut register = frame(Frame::new(REGISTER, pid, mode, 0));
    register
        .push_cap(Capability {
            handle: root,
            rights: Rights::SEND.union(Rights::TRANSFER),
        })
        .unwrap_or_else(|_| fail(6));
    if send_bounded(registry, &register).is_err() {
        fail(7);
    }
    if mode == WORKER_DENIED_CHECKS {
        let (matched, total) = denied_checks(root);
        if send_bounded(
            registry,
            &frame(Frame::new(DENIED_REPORT, matched, total, 0)),
        )
        .is_err()
        {
            fail(8);
        }
    }
    if mode == WORKER_WAIT_SHORT || mode == WORKER_WAIT_LONG {
        let horizon = if mode == WORKER_WAIT_SHORT { 40 } else { 500 };
        let now = clock::now_monotonic().unwrap_or_else(|_| fail(20));
        let deadline = now.checked_add(horizon).unwrap_or_else(|| fail(21));
        let timer = TimerRegistration::at(deadline).unwrap_or_else(|| fail(21));
        match clock::wait_until(&[WaitCondition::receive(ingress)], &timer) {
            Ok(WaitOutcome::TimedOut) => {}
            Ok(WaitOutcome::Ready(_)) => fail(23),
            Ok(WaitOutcome::Cancelled) | Err(_) => fail(22),
        }
        // The authorized parent must kill this lifetime before its deadline.
        fail(23);
    }
    let mut reply_target: Option<SendEndpoint> = None;
    let mut client_pid = 0u64;
    let mut served = 0u64;
    loop {
        tick();
        let message = match native::receive(ingress) {
            Ok(message) => message,
            Err(_) => {
                let _ = native::yield_now();
                continue;
            }
        };
        let Some(request) = Frame::decode(message.payload()) else {
            let _ = native::close_received_caps(&message);
            continue;
        };
        if message.tag() != TAG {
            let _ = native::close_received_caps(&message);
            continue;
        }
        match request.op {
            REPLY_TARGET if message.sender() == supervisor && message.cap_count() == 1 => {
                if let Some(old) = reply_target.take() {
                    let _ = native::close(old.handle());
                }
                let cap = message.cap(0).unwrap_or_else(|| fail(9));
                reply_target = Some(SendEndpoint::from_handle(cap.handle));
                client_pid = request.a;
            }
            REQUEST if message.cap_count() == 0 && message.sender() == client_pid => {
                served += 1;
                support::put(248, served);
                if let Some(target) = reply_target {
                    let reply = frame(Frame::new(REPLY, request.a, pid, served));
                    // A closed client is the client's problem, not a worker fault.
                    let _ = send_bounded(target, &reply);
                }
                if mode == WORKER_CRASH_AFTER_8 && served == 8 {
                    // SAFETY: Deliberate architected #UD; the kernel ends this
                    // process and reports vector 6 to the supervisor.
                    unsafe {
                        core::arch::asm!("ud2", options(noreturn));
                    }
                }
            }
            _ => {
                let _ = native::close_received_caps(&message);
            }
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(40, 0x1ff);
    native::exit(3)
}
