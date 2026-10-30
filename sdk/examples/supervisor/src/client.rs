#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! K05 client: an ordinary SDK app started by the supervisor. It owns one
//! reply endpoint, learns the worker from the supervisor and sends 24
//! numbered requests. When the worker dies it sees ECLOSED on the old
//! capability, waits for the next WORKER_AVAILABLE and continues with the
//! same sequence number, so the supervisor can prove service continuity.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("k05-client is only for the Nieuw OS x86_64-unknown-none target");

mod support;

use k05_protocol::*;
use nieuw_os_sdk_runtime::{
    native, Capability, ChildProcess, Error, Handle, LaunchImage, Outgoing, ReceiveEndpoint,
    Rights, SendEndpoint,
};

const LIMIT: usize = 400_000;
/// Polls (with yields) to wait for one reply before the request is resent.
const REPLY_LIMIT: usize = 20_000;
const TARGET: u64 = 24;

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

fn fail(step: u64) -> ! {
    support::put(240, step);
    support::put(40, 0x100 + step);
    native::exit(1)
}

fn frame(frame: Frame) -> Outgoing {
    Outgoing::new(TAG, &frame.encode()).unwrap_or_else(|_| fail(90))
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

fn kernel_code<T>(result: Result<T, Error>) -> i64 {
    match result {
        Ok(_) => 0,
        Err(Error::Kernel(code)) => code,
        Err(_) => 1,
    }
}

struct Client {
    supervisor: u64,
    registry: SendEndpoint,
    reply: ReceiveEndpoint,
    worker: Option<SendEndpoint>,
    replies: u64,
    closed: u64,
    instances: u64,
}

impl Client {
    /// Take a newly published worker. If an old worker capability is still
    /// held, the old endpoint must already be closed (its owner died before
    /// the supervisor published a successor): a probe send has to fail with
    /// ECLOSED/EREVOKED. That makes the dependency release observable and
    /// the `closed` count deterministic, whatever the message timing.
    fn adopt(&mut self, cap: Capability) {
        if let Some(old) = self.worker.take() {
            match send_bounded(old, &frame(Frame::new(REQUEST, 0, 0, 0))) {
                Err(Error::Kernel(ECLOSED)) | Err(Error::Kernel(-10)) => {
                    self.closed += 1;
                    support::put(264, self.closed);
                }
                _ => fail(28),
            }
            let _ = native::close(old.handle());
        }
        self.worker = Some(SendEndpoint::from_handle(cap.handle));
        self.instances += 1;
        support::put(256, self.instances);
    }

    /// Block (bounded) until the supervisor publishes a worker capability.
    fn wait_for_worker(&mut self) -> SendEndpoint {
        for _ in 0..LIMIT {
            tick();
            match native::receive(self.reply) {
                Ok(message) => {
                    let decoded = Frame::decode(message.payload());
                    match decoded {
                        Some(f)
                            if f.op == WORKER_AVAILABLE
                                && message.tag() == TAG
                                && message.sender() == self.supervisor
                                && message.cap_count() == 1 =>
                        {
                            self.adopt(message.cap(0).unwrap_or_else(|| fail(20)));
                            return self.worker.unwrap_or_else(|| fail(21));
                        }
                        _ => {
                            let _ = native::close_received_caps(&message);
                        }
                    }
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => fail(22),
            }
        }
        fail(23)
    }

    fn progress(&mut self, op: u64) {
        let report = frame(Frame::new(op, self.replies, self.closed, self.instances));
        if send_bounded(self.registry, &report).is_err() {
            fail(24);
        }
    }

    /// One numbered request, retried across worker restarts until answered.
    fn exchange(&mut self, seq: u64) {
        loop {
            let worker = match self.worker {
                Some(worker) => worker,
                None => self.wait_for_worker(),
            };
            match send_bounded(worker, &frame(Frame::new(REQUEST, seq, 0, 0))) {
                Ok(()) => {}
                Err(Error::Kernel(ECLOSED)) | Err(Error::Kernel(-10)) => {
                    // The old worker is gone: its endpoint closed with it.
                    self.closed += 1;
                    support::put(264, self.closed);
                    let _ = native::close(worker.handle());
                    self.worker = None;
                    continue;
                }
                Err(_) => fail(25),
            }
            for _ in 0..REPLY_LIMIT {
                tick();
                match native::receive(self.reply) {
                    Ok(message) => {
                        let decoded = Frame::decode(message.payload());
                        match decoded {
                            Some(f)
                                if f.op == REPLY
                                    && message.tag() == TAG
                                    && message.cap_count() == 0
                                    && f.a == seq =>
                            {
                                self.replies += 1;
                                support::put(248, self.replies);
                                self.progress(PROGRESS);
                                return;
                            }
                            Some(f)
                                if f.op == WORKER_AVAILABLE
                                    && message.tag() == TAG
                                    && message.sender() == self.supervisor
                                    && message.cap_count() == 1 =>
                            {
                                self.adopt(message.cap(0).unwrap_or_else(|| fail(26)));
                                break;
                            }
                            _ => {
                                let _ = native::close_received_caps(&message);
                            }
                        }
                    }
                    Err(error) if error.is_again() => {
                        let _ = native::yield_now();
                    }
                    Err(_) => fail(27),
                }
            }
            // No answer in time: the worker may have died after receiving the
            // request. Resend the same sequence number; a duplicate reply is
            // filtered by the `f.a == seq` match above.
        }
    }
}

/// Six negative checks: an app without launch or process authority gets the
/// documented error codes and never a process, and endpoint creation stops at
/// the quota of four owned endpoints.
fn denied_checks(reply_root: Handle) -> (u64, u64) {
    let mut matched = 0;
    let bogus = Handle::from_raw((1 << 32) | 250).unwrap_or_else(|_| fail(40));
    if kernel_code(native::spawn(LaunchImage::from_handle(reply_root), 0, &[])) == ETYPE {
        matched += 1;
    }
    if kernel_code(native::spawn(LaunchImage::from_handle(bogus), 0, &[])) == EBADHANDLE {
        matched += 1;
    }
    if kernel_code(native::kill(ChildProcess::from_handle(reply_root))) == ETYPE {
        matched += 1;
    }
    if kernel_code(native::status(ChildProcess::from_handle(bogus))) == EBADHANDLE {
        matched += 1;
    }
    let mut created = 0;
    for _ in 0..3 {
        if native::endpoint_create().is_ok() {
            created += 1;
        }
    }
    if created == 3 {
        matched += 1;
    }
    if kernel_code(native::endpoint_create()) == EQUOTA {
        matched += 1;
    }
    (matched, 6)
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
    let sparse = mode == CLIENT_SECOND_CAP;
    let (registry_offset, empty_offset) = if sparse { (64, 56) } else { (56, 64) };
    if supervisor == 0 || pid == 0 || support::word(32) != 1 || support::word(empty_offset) != 0 {
        fail(2);
    }
    let registry = SendEndpoint::from_handle(
        Handle::from_raw(support::word(registry_offset)).unwrap_or_else(|_| fail(3)),
    );
    let sparse_checks = if sparse {
        // Correct positions alone do not establish authority: cap1 must be
        // SEND-only. The following HELLO proves SEND works on that same handle.
        if kernel_code(native::receive(ReceiveEndpoint::from_handle(
            registry.handle(),
        ))) != EACCESS
            || kernel_code(native::derive(registry.handle(), Rights::SEND)) != EACCESS
        {
            fail(8);
        }
        3
    } else {
        0
    };
    let reply_root = native::endpoint_create().unwrap_or_else(|_| fail(4));
    let mut hello = frame(Frame::new(CLIENT_HELLO, pid, mode, sparse_checks));
    hello
        .push_cap(Capability {
            handle: reply_root,
            rights: Rights::SEND.union(Rights::TRANSFER),
        })
        .unwrap_or_else(|_| fail(5));
    if send_bounded(registry, &hello).is_err() {
        fail(6);
    }
    let mut client = Client {
        supervisor,
        registry,
        reply: ReceiveEndpoint::from_handle(reply_root),
        worker: None,
        replies: 0,
        closed: 0,
        instances: 0,
    };
    if mode == CLIENT_DENIED_CHECKS {
        let (matched, total) = denied_checks(reply_root);
        if send_bounded(
            registry,
            &frame(Frame::new(DENIED_REPORT, matched, total, 0)),
        )
        .is_err()
        {
            fail(7);
        }
        client.progress(DONE);
        support::put(40, 1);
        native::exit(0);
    }
    for seq in 1..=TARGET {
        client.exchange(seq);
    }
    client.progress(DONE);
    support::put(40, 1);
    native::exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(40, 0x1ff);
    native::exit(3)
}
