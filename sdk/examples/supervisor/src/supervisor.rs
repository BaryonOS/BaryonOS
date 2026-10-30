#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! K05 supervisor: a ring-3 service manager built only from the K05 syscalls.
//! It owns two `Launch` capabilities (worker image, client image) from the
//! boot manifest, creates a registry endpoint, starts children with a SEND
//! capability to that registry, brokers the worker's ingress capability to
//! the client, watches the worker with `proc_status`, restarts it (after a
//! deliberate kill or an observed fault) and ends every child before it exits.
//! All results are written as evidence words that the kernel reads at exit.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("k05-supervisor is only for the Nieuw OS x86_64-unknown-none target");

mod support;

use k05_protocol::*;
use nieuw_os_sdk_runtime::{
    native, Capability, ChildProcess, Error, Handle, LaunchImage, Outgoing, ProcessStatus,
    ReceiveEndpoint, Received, Rights, SendEndpoint,
};

const LIMIT: usize = 400_000;
const QUOTA_WORKERS: usize = 15;

// Evidence words read by the kernel (see kernel supervisor.rs SUP_WORDS).
const EV_RESTARTS: u64 = 400;
const EV_REPLIES: u64 = 408;
const EV_CLOSED: u64 = 416;
const EV_INSTANCES: u64 = 424;
const EV_KILLS: u64 = 432;
const EV_SPAWNS_OK: u64 = 440;
const EV_SPAWN_QUOTA: u64 = 448;
const EV_KILLED_CONFIRMED: u64 = 456;
const EV_FAULTED: u64 = 464;
const EV_DENIED_MATCHED: u64 = 472;
const EV_DENIED_TOTAL: u64 = 480;
const EV_CLIENT_EXIT_SEEN: u64 = 488;
const EV_DENIED_SELF: u64 = 496;
const EV_SPARSE_CAP_CHECKS: u64 = 504;

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

fn fail(step: u64) -> ! {
    support::put(240, step);
    support::put(40, 0x100 + step);
    native::exit(1)
}

fn bump(offset: u64) {
    support::put(offset, support::word(offset) + 1);
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

/// Exercise the raw ABI's two independently optional capability positions.
/// The ordinary SDK slice wrapper deliberately supplies contiguous positions.
fn spawn_slots(image: LaunchImage, arg: u64, caps: [(u64, u64); 2]) -> Result<ChildProcess, Error> {
    let mut primary = 0x50u64;
    let mut secondary = caps[0].0;
    // SAFETY: K05's checked int80 ABI preserves every GPR except RAX/RDX.
    // Only scalar words cross this boundary; no pointers or borrowed buffers.
    // The kernel validates both handles and rights before allocating resources.
    unsafe {
        core::arch::asm!("int 0x80", inout("rax") primary,
            in("rdi") image.handle().raw(), in("rsi") arg,
            inout("rdx") secondary, in("r10") caps[0].1,
            in("r8") caps[1].0, in("r9") caps[1].1, options(nostack));
    }
    Handle::from_raw(nieuw_os_sdk_runtime::decode_return(
        primary as i64,
        secondary,
    )?)
    .map(ChildProcess::from_handle)
}

struct Supervisor {
    pid: u64,
    case: u64,
    worker_image: LaunchImage,
    client_image: LaunchImage,
    registry_root: Handle,
    registry: ReceiveEndpoint,
    worker: Option<ChildProcess>,
    worker_send: Option<Capability>,
    worker_pid: u64,
    client: Option<ChildProcess>,
    client_reply: Option<Capability>,
    client_pid: u64,
    instance: u64,
    replies: u64,
    closed: u64,
    done: bool,
    denied_matched: u64,
    denied_total: u64,
    denied_reports: u64,
}

enum Incoming {
    Progress,
    Done,
    Register(Capability, u64),
    Hello(Capability, u64),
    Denied(u64, u64),
}

impl Supervisor {
    fn spawn(&self, image: LaunchImage, mode: u64) -> Result<ChildProcess, Error> {
        let arg = mode | (self.pid << 8);
        native::spawn(
            image,
            arg,
            &[Capability {
                handle: self.registry_root,
                rights: Rights::SEND,
            }],
        )
    }

    fn classify(&mut self, message: &Received) -> Option<Incoming> {
        let f = Frame::decode(message.payload())?;
        if message.tag() != TAG {
            return None;
        }
        match (f.op, message.cap_count()) {
            (REGISTER, 1) => Some(Incoming::Register(message.cap(0)?, message.sender())),
            (CLIENT_HELLO, 1) => {
                if self.case == CASE_NORMAL {
                    if f.b != CLIENT_SECOND_CAP || f.c != 3 {
                        fail(34);
                    }
                    support::put(EV_SPARSE_CAP_CHECKS, f.c);
                }
                Some(Incoming::Hello(message.cap(0)?, message.sender()))
            }
            (PROGRESS, 0) if message.sender() == self.client_pid => {
                self.replies = f.a;
                self.closed = f.b;
                Some(Incoming::Progress)
            }
            (DONE, 0) if message.sender() == self.client_pid => {
                self.replies = f.a;
                self.closed = f.b;
                self.done = true;
                Some(Incoming::Done)
            }
            (DENIED_REPORT, 0) => {
                self.denied_matched += f.a;
                self.denied_total += f.b;
                self.denied_reports += 1;
                Some(Incoming::Denied(f.a, f.b))
            }
            _ => None,
        }
    }

    /// Receive one registry message, bounded; unexpected caps are closed.
    fn next(&mut self) -> Option<Incoming> {
        tick();
        match native::receive(self.registry) {
            Ok(message) => {
                let incoming = self.classify(&message);
                let keep_cap =
                    matches!(incoming, Some(Incoming::Register(..) | Incoming::Hello(..)));
                if !keep_cap {
                    let _ = native::close_received_caps(&message);
                }
                incoming
            }
            Err(error) if error.is_again() => {
                let _ = native::yield_now();
                None
            }
            Err(_) => fail(30),
        }
    }

    fn wait_register(&mut self) -> (Capability, u64) {
        for _ in 0..LIMIT {
            if let Some(Incoming::Register(cap, sender)) = self.next() {
                return (cap, sender);
            }
        }
        fail(31)
    }

    fn wait_hello(&mut self) -> (Capability, u64) {
        for _ in 0..LIMIT {
            if let Some(Incoming::Hello(cap, sender)) = self.next() {
                return (cap, sender);
            }
        }
        fail(32)
    }

    fn status_of(&self, child: ChildProcess) -> ProcessStatus {
        native::status(child).unwrap_or_else(|_| fail(33))
    }

    fn kill_and_confirm(&self, child: ChildProcess, step: u64) {
        if native::kill(child).is_err() {
            fail(step);
        }
        bump(EV_KILLS);
        if self.status_of(child) != ProcessStatus::Killed {
            fail(step + 1);
        }
        bump(EV_KILLED_CONFIRMED);
        if native::close(child.handle()).is_err() {
            fail(step + 2);
        }
    }

    fn start_worker(&mut self, mode: u64) {
        let worker = self
            .spawn(self.worker_image, mode)
            .unwrap_or_else(|_| fail(40));
        bump(EV_SPAWNS_OK);
        let (cap, sender) = self.wait_register();
        self.worker = Some(worker);
        self.worker_send = Some(cap);
        self.worker_pid = sender;
        self.instance += 1;
        support::put(EV_INSTANCES, self.instance);
    }

    /// Give the worker the client's reply target and the client the worker.
    fn connect(&mut self) {
        let worker_send = self.worker_send.unwrap_or_else(|| fail(41));
        let client_reply = self.client_reply.unwrap_or_else(|| fail(42));
        let mut target = frame(Frame::new(REPLY_TARGET, self.client_pid, 0, 0));
        target
            .push_cap(Capability {
                handle: client_reply.handle,
                rights: Rights::SEND,
            })
            .unwrap_or_else(|_| fail(43));
        if send_bounded(SendEndpoint::from_handle(worker_send.handle), &target).is_err() {
            fail(44);
        }
        let mut available = frame(Frame::new(WORKER_AVAILABLE, self.instance, 0, 0));
        available
            .push_cap(Capability {
                handle: worker_send.handle,
                rights: Rights::SEND,
            })
            .unwrap_or_else(|_| fail(45));
        if send_bounded(SendEndpoint::from_handle(client_reply.handle), &available).is_err() {
            fail(46);
        }
    }

    fn drop_worker_caps(&mut self) {
        if let Some(cap) = self.worker_send.take() {
            let _ = native::close(cap.handle);
        }
        if let Some(worker) = self.worker.take() {
            if native::close(worker.handle()).is_err() {
                fail(47);
            }
        }
    }

    fn restart_worker(&mut self) {
        self.drop_worker_caps();
        self.start_worker(WORKER_SERVE);
        self.connect();
        bump(EV_RESTARTS);
    }

    fn run_service(&mut self) {
        let worker_mode = if self.case == CASE_CRASH {
            WORKER_CRASH_AFTER_8
        } else {
            WORKER_SERVE
        };
        self.start_worker(worker_mode);
        let client = if self.case == CASE_NORMAL {
            spawn_slots(
                self.client_image,
                CLIENT_SECOND_CAP | (self.pid << 8),
                [(0, 0), (self.registry_root.raw(), Rights::SEND.bits())],
            )
        } else {
            self.spawn(self.client_image, CLIENT_RUN_24)
        }
        .unwrap_or_else(|_| fail(50));
        bump(EV_SPAWNS_OK);
        let (cap, sender) = self.wait_hello();
        self.client = Some(client);
        self.client_reply = Some(cap);
        self.client_pid = sender;
        self.connect();
        let mut restarted = false;
        for _ in 0..LIMIT {
            let _ = self.next();
            support::put(EV_REPLIES, self.replies);
            support::put(EV_CLOSED, self.closed);
            if self.case == CASE_PARENT_EXIT && self.replies >= 4 {
                // Leave both children alive: the kernel must end them because
                // their spawner is gone.
                support::put(40, 1);
                native::exit(0);
            }
            let worker = self.worker.unwrap_or_else(|| fail(51));
            match self.status_of(worker) {
                ProcessStatus::Running => {
                    if self.case == CASE_NORMAL && !restarted && self.replies >= 8 {
                        restarted = true;
                        self.kill_and_confirm(worker, 52);
                        self.worker = None;
                        self.restart_worker();
                    }
                }
                ProcessStatus::Faulted(6) if self.case == CASE_CRASH && !restarted => {
                    restarted = true;
                    bump(EV_FAULTED);
                    self.restart_worker();
                }
                _ => fail(55),
            }
            if self.done {
                break;
            }
        }
        if !self.done {
            fail(56);
        }
        // The client exits by itself after DONE; observe that through status.
        for _ in 0..LIMIT {
            tick();
            match self.status_of(client) {
                ProcessStatus::Exited(0) => {
                    support::put(EV_CLIENT_EXIT_SEEN, 1);
                    break;
                }
                ProcessStatus::Running => {
                    let _ = native::yield_now();
                }
                _ => fail(57),
            }
        }
        if support::word(EV_CLIENT_EXIT_SEEN) != 1 {
            fail(58);
        }
        if native::close(client.handle()).is_err() {
            fail(59);
        }
        let worker = self.worker.take().unwrap_or_else(|| fail(60));
        self.kill_and_confirm(worker, 61);
        if let Some(cap) = self.worker_send.take() {
            let _ = native::close(cap.handle);
        }
        if let Some(cap) = self.client_reply.take() {
            let _ = native::close(cap.handle);
        }
    }

    /// Thirteen negative checks with real authority present: typed handles,
    /// bad handles, forbidden delegation rights and double kills all fail
    /// with the documented codes and change nothing.
    fn denied_self_checks(&mut self, worker: ChildProcess, client: ChildProcess) -> u64 {
        let mut matched = 0;
        let bogus = Handle::from_raw((1 << 32) | 250).unwrap_or_else(|_| fail(70));
        if kernel_code(self.spawn(LaunchImage::from_handle(worker.handle()), 0)) == ETYPE {
            matched += 1;
        }
        if kernel_code(self.spawn(LaunchImage::from_handle(bogus), 0)) == EBADHANDLE {
            matched += 1;
        }
        if kernel_code(self.spawn(LaunchImage::from_handle(self.registry_root), 0)) == ETYPE {
            matched += 1;
        }
        if kernel_code(native::kill(ChildProcess::from_handle(
            self.worker_image.handle(),
        ))) == ETYPE
        {
            matched += 1;
        }
        if kernel_code(native::status(ChildProcess::from_handle(
            self.registry_root,
        ))) == ETYPE
        {
            matched += 1;
        }
        // Delegating a right the endpoint type does not know is invalid.
        let launch_on_endpoint = native::spawn(
            self.worker_image,
            0,
            &[Capability {
                handle: self.registry_root,
                rights: Rights::LAUNCH,
            }],
        );
        if kernel_code(launch_on_endpoint) == EINVAL {
            matched += 1;
        }
        // A SEND-only derivation lacks TRANSFER, so it cannot be delegated.
        let send_only =
            native::derive(self.registry_root, Rights::SEND).unwrap_or_else(|_| fail(71));
        let no_transfer = native::spawn(
            self.worker_image,
            0,
            &[Capability {
                handle: send_only,
                rights: Rights::SEND,
            }],
        );
        if kernel_code(no_transfer) == EACCESS {
            matched += 1;
        }
        // The second raw position enforces the same delegation rules.
        if kernel_code(spawn_slots(
            self.worker_image,
            0,
            [(0, 0), (send_only.raw(), Rights::SEND.bits())],
        )) == EACCESS
        {
            matched += 1;
        }
        if kernel_code(spawn_slots(
            self.worker_image,
            0,
            [(0, 0), (self.registry_root.raw(), Rights::RECV.bits())],
        )) == EACCESS
        {
            matched += 1;
        }
        let _ = native::close(send_only);
        // A missing handle with nonzero rights is invalid in either position,
        // even when the other position contains a valid delegable endpoint.
        for caps in [
            [
                (0, Rights::SEND.bits()),
                (self.registry_root.raw(), Rights::SEND.bits()),
            ],
            [
                (self.registry_root.raw(), Rights::SEND.bits()),
                (0, Rights::SEND.bits()),
            ],
        ] {
            if kernel_code(spawn_slots(self.worker_image, 0, caps)) == EINVAL {
                matched += 1;
            }
        }
        // Kill once (counted), then a second kill of a dead child is closed.
        self.kill_and_confirm_keep(worker, 72);
        if kernel_code(native::kill(worker)) == ECLOSED {
            matched += 1;
        }
        if native::close(worker.handle()).is_err() {
            fail(73);
        }
        // The client exits by itself after its report; its object still
        // reports Exited(0) until the handle is closed. After the close the
        // number is a bad handle, never a leaked object.
        for _ in 0..LIMIT {
            tick();
            match self.status_of(client) {
                ProcessStatus::Exited(0) => break,
                ProcessStatus::Running => {
                    let _ = native::yield_now();
                }
                _ => fail(74),
            }
        }
        if self.status_of(client) != ProcessStatus::Exited(0) {
            fail(75);
        }
        if native::close(client.handle()).is_err() {
            fail(76);
        }
        if kernel_code(native::status(client)) == EBADHANDLE {
            matched += 1;
        }
        matched
    }

    fn kill_and_confirm_keep(&self, child: ChildProcess, step: u64) {
        if native::kill(child).is_err() {
            fail(step);
        }
        bump(EV_KILLS);
        if self.status_of(child) != ProcessStatus::Killed {
            fail(step + 1);
        }
        bump(EV_KILLED_CONFIRMED);
    }

    fn run_denied(&mut self) {
        let worker = self
            .spawn(self.worker_image, WORKER_DENIED_CHECKS)
            .unwrap_or_else(|_| fail(80));
        bump(EV_SPAWNS_OK);
        let (cap, sender) = self.wait_register();
        self.worker_send = Some(cap);
        self.worker_pid = sender;
        let client = self
            .spawn(self.client_image, CLIENT_DENIED_CHECKS)
            .unwrap_or_else(|_| fail(81));
        bump(EV_SPAWNS_OK);
        let (cap, sender) = self.wait_hello();
        self.client_reply = Some(cap);
        self.client_pid = sender;
        // Collect both reports (2 + 6 checks) before the own checks (13). The
        // worker's report may already have been consumed while waiting for the
        // client's hello; `classify` counts every report wherever it arrives.
        for _ in 0..LIMIT {
            if self.denied_reports >= 2 {
                break;
            }
            let _ = self.next();
        }
        if self.denied_reports != 2 {
            fail(82);
        }
        let own = self.denied_self_checks(worker, client);
        support::put(EV_DENIED_SELF, own);
        self.denied_matched += own;
        self.denied_total += 13;
        support::put(EV_DENIED_MATCHED, self.denied_matched);
        support::put(EV_DENIED_TOTAL, self.denied_total);
        if let Some(cap) = self.worker_send.take() {
            let _ = native::close(cap.handle);
        }
        if let Some(cap) = self.client_reply.take() {
            let _ = native::close(cap.handle);
        }
    }

    fn wait_ticks(&self, ticks: u64) {
        let now = native::monotonic_ticks().unwrap_or_else(|_| fail(100));
        let deadline = now.checked_add(ticks).unwrap_or_else(|| fail(101));
        if native::ipc_wait(&[], deadline) != Ok(0) {
            fail(102);
        }
    }

    fn run_wait_lifecycle(&mut self) {
        self.start_worker(WORKER_WAIT_SHORT);
        let old = self.worker.unwrap_or_else(|| fail(103));
        let old_pid = self.worker_pid;
        self.wait_ticks(4);
        if kernel_code(native::derive(old.handle(), Rights::OBSERVE)) != EACCESS {
            fail(104);
        }
        if native::kill(ChildProcess::from_handle(self.registry_root)) != Err(Error::Kernel(ETYPE))
        {
            fail(105);
        }
        self.kill_and_confirm(old, 107);
        self.drop_worker_caps();
        self.start_worker(WORKER_WAIT_LONG);
        if self.worker_pid == old_pid
            || native::status(old) != Err(Error::Kernel(EBADHANDLE))
            || native::kill(old) != Err(Error::Kernel(EBADHANDLE))
        {
            fail(108);
        }
        support::put(EV_DENIED_MATCHED, 4);
        support::put(EV_DENIED_TOTAL, 4);
        bump(EV_RESTARTS);
        // Beyond the first lifetime's latest possible deadline, but well
        // before the fresh lifetime's own 500-tick deadline.
        self.wait_ticks(80);
        self.kill_and_confirm(self.worker.unwrap_or_else(|| fail(109)), 110);
        self.drop_worker_caps();
    }

    fn run_quota(&mut self) {
        let mut workers = [None; QUOTA_WORKERS];
        let mut count = 0;
        loop {
            match self.spawn(self.worker_image, WORKER_IDLE) {
                Ok(child) => {
                    if count == QUOTA_WORKERS {
                        fail(85);
                    }
                    workers[count] = Some(child);
                    count += 1;
                    bump(EV_SPAWNS_OK);
                }
                Err(Error::Kernel(EQUOTA)) => {
                    bump(EV_SPAWN_QUOTA);
                    break;
                }
                Err(_) => fail(86),
            }
        }
        if count != QUOTA_WORKERS {
            fail(87);
        }
        // Every idle worker registers; drain those messages and drop the caps.
        let mut registered = 0;
        for _ in 0..LIMIT {
            match self.next() {
                Some(Incoming::Register(cap, _)) => {
                    let _ = native::close(cap.handle);
                    registered += 1;
                }
                _ => {}
            }
            if registered == QUOTA_WORKERS {
                break;
            }
        }
        if registered != QUOTA_WORKERS {
            fail(88);
        }
        for worker in workers.iter().flatten() {
            if self.status_of(*worker) != ProcessStatus::Running {
                fail(89);
            }
            self.kill_and_confirm(*worker, 91);
        }
    }
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    if native::check_abi().is_err() {
        fail(1);
    }
    let case = support::word(0);
    let pid = support::word(176);
    if case != support::word(112) || pid == 0 || support::word(32) != 1 {
        fail(2);
    }
    let worker_image =
        LaunchImage::from_handle(Handle::from_raw(support::word(48)).unwrap_or_else(|_| fail(3)));
    let client_image =
        LaunchImage::from_handle(Handle::from_raw(support::word(72)).unwrap_or_else(|_| fail(4)));
    let registry_root = native::endpoint_create().unwrap_or_else(|_| fail(5));
    let mut supervisor = Supervisor {
        pid,
        case,
        worker_image,
        client_image,
        registry_root,
        registry: ReceiveEndpoint::from_handle(registry_root),
        worker: None,
        worker_send: None,
        worker_pid: 0,
        client: None,
        client_reply: None,
        client_pid: 0,
        instance: 0,
        replies: 0,
        closed: 0,
        done: false,
        denied_matched: 0,
        denied_total: 0,
        denied_reports: 0,
    };
    match case {
        CASE_NORMAL | CASE_CRASH | CASE_PARENT_EXIT => supervisor.run_service(),
        CASE_DENIED => supervisor.run_denied(),
        CASE_QUOTA => supervisor.run_quota(),
        CASE_WAIT => supervisor.run_wait_lifecycle(),
        _ => fail(6),
    }
    // Launch handles and the registry are closed explicitly; the kernel also
    // reclaims them at exit, and checks that nothing remains afterwards.
    let _ = native::close(worker_image.handle());
    let _ = native::close(client_image.handle());
    let _ = native::close(registry_root);
    support::put(40, 1);
    native::exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(40, 0x1ff);
    native::exit(3)
}
