//! Two-role native quantum job service and caller for L02 acceptance.
#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use core::{
    alloc::{GlobalAlloc, Layout},
    arch::{asm, global_asm},
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};
use nieuw_os_sdk_runtime::{native, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint};
use qsim_job_protocol::{
    decode, encode, Gate as WireGate, Identifiers, Request, ResultPage as PageRequest, Submit,
    MAX_GATES, MAX_WORK_BUDGET,
};
use qsim_native::{execute, Abort, Control, Error as QError, ExecuteError, Gate, Result as QResult, HEAP_BYTES};

const DATA: usize = 0x4001_0000;
const HEAP: usize = 0x6000_0000;
const REQUEST_TAG: u64 = 0x514a_4f42_5245_5131;
const REPLY_TAG: u64 = 0x514a_4f42_5250_4c31;
const REPLY_BYTES: usize = 128;
const COUNTS_PER_PAGE: usize = 16;

global_asm!(r#"
    .section .qsim.header,"aw",@progbits
    .balign 8
    .global qsim_startup
qsim_startup:
    .zero 512
"#);

struct ProcessArena(AtomicUsize);

// SAFETY: One thread per process and a private 64-KiB RW/NX arena. Monotone,
// checked allocations cannot overlap. The arena is reset only after execute()
// returned and all temporary qsim allocations have been dropped.
unsafe impl GlobalAlloc for ProcessArena {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut current = self.0.load(Ordering::Relaxed);
        loop {
            let Some((start, end)) = qsim_native::heap_budget::allocation_range(
                current, layout.size(), layout.align(), HEAP_BYTES,
            ) else {
                return ptr::null_mut();
            };
            match self.0.compare_exchange_weak(
                current, end, Ordering::Relaxed, Ordering::Relaxed,
            ) {
                Ok(_) => return (HEAP + start) as *mut u8,
                Err(actual) => current = actual,
            }
        }
    }
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {}
}

#[global_allocator]
static ARENA: ProcessArena = ProcessArena(AtomicUsize::new(0));

fn read(offset: usize) -> u64 {
    // SAFETY: Fixed aligned words in our private report page.
    unsafe { ((DATA + offset) as *const u64).read_volatile() }
}
fn write(offset: usize, value: u64) {
    // SAFETY: Fixed aligned words in our private report page.
    unsafe { ((DATA + offset) as *mut u64).write_volatile(value) }
}
fn bump() { write(8, read(8).wrapping_add(1)); }
fn fail(code: u64) -> ! {
    write(48, 0x100 + code);
    loop {
        if native::yield_now().is_err() { core::hint::spin_loop(); }
    }
}
fn yield_now() {
    bump();
    if native::yield_now().is_err() { fail(1); }
}
fn worker() -> ! { loop { yield_now(); } }
fn handle(offset: usize) -> Handle { Handle::from_raw(read(offset)).unwrap_or_else(|_| fail(2)) }
fn request_send() -> SendEndpoint { SendEndpoint::from_handle(handle(32)) }
fn request_recv() -> ReceiveEndpoint { ReceiveEndpoint::from_handle(handle(32)) }
fn reply_send() -> SendEndpoint { SendEndpoint::from_handle(handle(40)) }
fn reply_recv() -> ReceiveEndpoint { ReceiveEndpoint::from_handle(handle(40)) }
fn expected_peer() -> u64 { read(24) }

fn send_payload(endpoint: SendEndpoint, tag: u64, payload: &[u8]) {
    let message = Outgoing::new(tag, payload).unwrap_or_else(|_| fail(3));
    loop {
        bump();
        match native::send(endpoint, &message) {
            Ok(()) => return,
            Err(error) if error.is_again() => yield_now(),
            Err(_) => fail(4),
        }
    }
}

fn receive_message(endpoint: ReceiveEndpoint, peer: u64, tag: u64) -> Received {
    loop {
        bump();
        match native::receive(endpoint) {
            Ok(message) => {
                if message.cap_count() != 0 {
                    let _ = native::close_received_caps(&message);
                    fail(5);
                }
                if message.sender() != peer || message.tag() != tag { fail(6); }
                return message;
            }
            Err(error) if error.is_again() => yield_now(),
            Err(_) => fail(7),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum ReplyState { Completed = 1, Cancelled = 2, Deadline = 3, Resource = 4, Invalid = 5 }
impl ReplyState {
    fn decode(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Completed), 2 => Some(Self::Cancelled),
            3 => Some(Self::Deadline), 4 => Some(Self::Resource),
            5 => Some(Self::Invalid), _ => None,
        }
    }
}

#[derive(Clone, Copy)]
struct Reply {
    kind: u8,
    state: ReplyState,
    ids: Identifiers,
    bins: u8,
    page: u8,
    count: u8,
    shots: u32,
    buffer_bytes: u64,
    counts: [u32; COUNTS_PER_PAGE],
}
fn put_u64(bytes: &mut [u8], at: usize, value: u64) { bytes[at..at + 8].copy_from_slice(&value.to_le_bytes()); }
fn put_u32(bytes: &mut [u8], at: usize, value: u32) { bytes[at..at + 4].copy_from_slice(&value.to_le_bytes()); }
fn get_u64(bytes: &[u8], at: usize) -> u64 { u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()) }
fn get_u32(bytes: &[u8], at: usize) -> u32 { u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) }

fn encode_reply(reply: Reply) -> [u8; REPLY_BYTES] {
    let mut bytes = [0; REPLY_BYTES];
    bytes[..4].copy_from_slice(b"QR01");
    bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    bytes[6] = reply.kind;
    bytes[7] = reply.state as u8;
    put_u64(&mut bytes, 8, reply.ids.job_id());
    put_u64(&mut bytes, 16, reply.ids.request_id());
    bytes[24] = reply.bins; bytes[25] = reply.page; bytes[26] = reply.count;
    put_u32(&mut bytes, 28, reply.shots);
    put_u64(&mut bytes, 32, reply.buffer_bytes);
    for (index, count) in reply.counts.iter().enumerate() {
        put_u32(&mut bytes, 64 + index * 4, *count);
    }
    bytes
}

fn decode_reply(bytes: &[u8]) -> Reply {
    if bytes.len() != REPLY_BYTES || &bytes[..4] != b"QR01"
        || bytes[4..6] != 1u16.to_le_bytes() || !matches!(bytes[6], 1 | 2)
        || bytes[27] != 0 || bytes[40..64].iter().any(|byte| *byte != 0) {
        fail(8);
    }
    let state = ReplyState::decode(bytes[7]).unwrap_or_else(|| fail(9));
    let ids = Identifiers::new(get_u64(bytes, 8), get_u64(bytes, 16)).unwrap_or_else(|_| fail(10));
    let mut counts = [0; COUNTS_PER_PAGE];
    for (index, count) in counts.iter_mut().enumerate() { *count = get_u32(bytes, 64 + index * 4); }
    let reply = Reply {
        kind: bytes[6], state, ids, bins: bytes[24], page: bytes[25], count: bytes[26],
        shots: get_u32(bytes, 28), buffer_bytes: get_u64(bytes, 32), counts,
    };
    if reply.bins > 64 || reply.page >= 4 || reply.count as usize > COUNTS_PER_PAGE { fail(11); }
    if reply.kind == 1 {
        if reply.page != 0 || reply.count != 0 || reply.counts.iter().any(|count| *count != 0) { fail(12); }
        if reply.state == ReplyState::Completed && reply.bins == 0 { fail(13); }
        if reply.state != ReplyState::Completed && (reply.bins != 0 || reply.shots != 0 || reply.buffer_bytes != 0) { fail(14); }
    } else {
        let start = usize::from(reply.page) * COUNTS_PER_PAGE;
        let expected = core::cmp::min(COUNTS_PER_PAGE, usize::from(reply.bins).saturating_sub(start));
        if reply.state != ReplyState::Completed || reply.bins == 0 || start >= usize::from(reply.bins)
            || usize::from(reply.count) != expected || reply.shots != 0 || reply.buffer_bytes != 0
            || reply.counts[expected..].iter().any(|count| *count != 0) { fail(15); }
    }
    reply
}

fn status_reply(ids: Identifiers, state: ReplyState, result: Option<QResult>) -> Reply {
    let (bins, shots, buffer_bytes) = result.map_or((0, 0, 0), |result| (
        u8::try_from(result.bins).unwrap_or_else(|_| fail(16)),
        result.shots_completed, result.buffer_bytes as u64,
    ));
    Reply { kind: 1, state, ids, bins, page: 0, count: 0, shots, buffer_bytes, counts: [0; COUNTS_PER_PAGE] }
}
fn page_reply(ids: Identifiers, page: u8, result: &QResult) -> Reply {
    let start = usize::from(page) * COUNTS_PER_PAGE;
    if start >= result.bins { return status_reply(ids, ReplyState::Invalid, None); }
    let count = core::cmp::min(COUNTS_PER_PAGE, result.bins - start);
    let mut counts = [0; COUNTS_PER_PAGE];
    counts[..count].copy_from_slice(&result.counts[start..start + count]);
    Reply {
        kind: 2, state: ReplyState::Completed, ids,
        bins: u8::try_from(result.bins).unwrap_or_else(|_| fail(17)),
        page, count: count as u8, shots: 0, buffer_bytes: 0, counts,
    }
}
fn send_reply(reply: Reply) { send_payload(reply_send(), REPLY_TAG, &encode_reply(reply)); }
fn receive_request() -> Request {
    let message = receive_message(request_recv(), expected_peer(), REQUEST_TAG);
    if message.payload().len() != qsim_job_protocol::PAYLOAD_BYTES { fail(18); }
    decode(message.payload()).unwrap_or_else(|_| fail(19))
}
fn receive_reply() -> Reply {
    let message = receive_message(reply_recv(), expected_peer(), REPLY_TAG);
    decode_reply(message.payload())
}

struct JobControl {
    receive: ReceiveEndpoint,
    peer: u64,
    job_id: u64,
    budget: u64,
    cancel: Option<Identifiers>,
    max_work: u64,
}
impl Control for JobControl {
    fn checkpoint(&mut self, work: u64) -> Result<(), Abort> {
        self.max_work = self.max_work.max(work);
        bump();
        if work > self.budget { return Err(Abort::DeadlineExceeded); }
        match native::receive(self.receive) {
            Err(error) if error.is_again() => Ok(()),
            Err(_) => fail(20),
            Ok(message) => {
                if message.cap_count() != 0 || message.sender() != self.peer
                    || message.tag() != REQUEST_TAG
                    || message.payload().len() != qsim_job_protocol::PAYLOAD_BYTES {
                    let _ = native::close_received_caps(&message);
                    fail(21);
                }
                match decode(message.payload()).unwrap_or_else(|_| fail(22)) {
                    Request::Cancel(ids) if ids.job_id() == self.job_id => {
                        self.cancel = Some(ids);
                        Err(Abort::Cancelled)
                    }
                    _ => fail(23),
                }
            }
        }
    }
}

fn convert_gates(submit: &Submit) -> ([Gate; MAX_GATES], usize) {
    let mut gates = [Gate::H { target: 0 }; MAX_GATES];
    for (index, gate) in submit.gates().iter().enumerate() {
        gates[index] = match *gate {
            WireGate::H { target } => Gate::H { target: u64::from(target) },
            WireGate::X { target } => Gate::X { target: u64::from(target) },
            WireGate::Z { target } => Gate::Z { target: u64::from(target) },
            WireGate::S { target } => Gate::S { target: u64::from(target) },
            WireGate::Cx { target, control } => Gate::Cx { target: u64::from(target), control: u64::from(control) },
        };
    }
    (gates, submit.gates().len())
}
#[derive(Clone, Copy)]
struct Stored { job_id: u64, result: QResult }

fn service() -> ! {
    let peer = expected_peer();
    let receive = request_recv();
    let mut stored: Option<Stored> = None;
    let (mut submissions, mut pages, mut requests, mut replies) = (0u64, 0u64, 0u64, 0u64);
    let (mut cancellations, mut deadlines, mut completed, mut scenarios) = (0u64, 0u64, 0u64, 0u64);
    let (mut max_work, mut peak_arena) = (0u64, 0usize);
    loop {
        let request = receive_request();
        requests += 1;
        match request {
            Request::Submit(submit) => {
                submissions += 1;
                let ids = submit.identifiers();
                let (gates, count) = convert_gates(&submit);
                let mut control = JobControl {
                    receive, peer, job_id: ids.job_id(), budget: submit.work_budget(),
                    cancel: None, max_work: 0,
                };
                let outcome = execute(
                    u64::from(submit.qubits()), &gates[..count], u64::from(submit.shots()),
                    submit.seed(), &mut control,
                );
                max_work = max_work.max(control.max_work);
                peak_arena = peak_arena.max(ARENA.0.load(Ordering::Relaxed));
                ARENA.0.store(0, Ordering::Relaxed);
                match outcome {
                    Ok(result) => {
                        completed += 1; scenarios |= if completed == 1 { 1 } else { 2 };
                        stored = Some(Stored { job_id: ids.job_id(), result });
                        send_reply(status_reply(ids, ReplyState::Completed, Some(result))); replies += 1;
                    }
                    Err(QError::Execution(ExecuteError::Aborted(Abort::Cancelled))) => {
                        let cancel = control.cancel.unwrap_or_else(|| fail(24));
                        cancellations += 1; scenarios |= 4; requests += 1;
                        send_reply(status_reply(ids, ReplyState::Cancelled, None));
                        send_reply(status_reply(cancel, ReplyState::Cancelled, None)); replies += 2;
                    }
                    Err(QError::Execution(ExecuteError::Aborted(Abort::DeadlineExceeded))) => {
                        deadlines += 1; scenarios |= 8;
                        send_reply(status_reply(ids, ReplyState::Deadline, None)); replies += 1;
                    }
                    Err(QError::Execution(ExecuteError::ResourceExhausted)) => {
                        send_reply(status_reply(ids, ReplyState::Resource, None)); replies += 1;
                    }
                    Err(_) => { send_reply(status_reply(ids, ReplyState::Invalid, None)); replies += 1; }
                }
            }
            Request::ResultPage(page) => {
                pages += 1;
                let ids = page.identifiers();
                let reply = stored.filter(|stored| stored.job_id == ids.job_id()).map_or_else(
                    || status_reply(ids, ReplyState::Invalid, None),
                    |stored| page_reply(ids, page.page(), &stored.result),
                );
                send_reply(reply); replies += 1;
            }
            Request::Status(ids) => {
                let reply = stored.filter(|stored| stored.job_id == ids.job_id()).map_or_else(
                    || status_reply(ids, ReplyState::Invalid, None),
                    |stored| status_reply(ids, ReplyState::Completed, Some(stored.result)),
                );
                send_reply(reply); replies += 1;
            }
            Request::Cancel(ids) => { send_reply(status_reply(ids, ReplyState::Invalid, None)); replies += 1; }
        }
        if submissions == 4 && pages == 2 && cancellations == 1 && deadlines == 1 && completed == 2 {
            if requests != 7 || replies != 7 || scenarios != 15 || peak_arena == 0 || peak_arena > HEAP_BYTES { fail(25); }
            write(56, scenarios); write(64, replies); write(72, requests); write(80, max_work);
            write(88, cancellations); write(96, deadlines); write(104, completed); write(112, peak_arena as u64);
            write(48, 1); worker();
        }
    }
}

fn send_request(request: Request) { send_payload(request_send(), REQUEST_TAG, &encode(&request)); }
fn submit(ids: Identifiers, qubits: u8, seed: u64, shots: u32, budget: u64, gates: &[WireGate]) {
    send_request(Request::Submit(Submit::new(ids, qubits, seed, shots, budget, gates).unwrap_or_else(|_| fail(26))));
}
fn expect_status(ids: Identifiers, state: ReplyState, bins: u8, shots: u32) -> Reply {
    let reply = receive_reply();
    if reply.kind != 1 || reply.ids != ids || reply.state != state || reply.bins != bins || reply.shots != shots { fail(27); }
    reply
}

fn client() -> ! {
    let (mut requests, mut replies, mut scenarios) = (0u64, 0u64, 0u64);
    let bell_ids = Identifiers::new(1, 1).unwrap();
    let bell = [WireGate::H { target: 0 }, WireGate::Cx { target: 1, control: 0 }];
    submit(bell_ids, 2, 0, 4096, MAX_WORK_BUDGET, &bell); requests += 1;
    if expect_status(bell_ids, ReplyState::Completed, 4, 4096).buffer_bytes == 0 { fail(28); }
    replies += 1;
    let bell_page_ids = Identifiers::new(1, 2).unwrap();
    send_request(Request::ResultPage(PageRequest::new(bell_page_ids, 0).unwrap())); requests += 1;
    let page = receive_reply(); replies += 1;
    if page.kind != 2 || page.ids != bell_page_ids || page.bins != 4 || page.page != 0
        || page.count != 4 || page.counts[..4] != [2082, 0, 0, 2014] { fail(29); }
    scenarios |= 1;

    let six_ids = Identifiers::new(2, 1).unwrap();
    let six = core::array::from_fn::<WireGate, 6, _>(|target| WireGate::X { target: target as u8 });
    submit(six_ids, 6, u64::MAX, 4096, MAX_WORK_BUDGET, &six); requests += 1;
    expect_status(six_ids, ReplyState::Completed, 64, 4096); replies += 1;
    let six_page_ids = Identifiers::new(2, 2).unwrap();
    send_request(Request::ResultPage(PageRequest::new(six_page_ids, 3).unwrap())); requests += 1;
    let page = receive_reply(); replies += 1;
    if page.kind != 2 || page.ids != six_page_ids || page.bins != 64 || page.page != 3
        || page.count != 16 || page.counts[..15].iter().any(|count| *count != 0)
        || page.counts[15] != 4096 { fail(30); }
    scenarios |= 2;

    let cancel_submit = Identifiers::new(3, 1).unwrap();
    let cancel_ids = Identifiers::new(3, 2).unwrap();
    let long = [WireGate::H { target: 0 }; MAX_GATES];
    submit(cancel_submit, 6, 0, 4096, MAX_WORK_BUDGET, &long);
    send_request(Request::Cancel(cancel_ids)); requests += 2;
    expect_status(cancel_submit, ReplyState::Cancelled, 0, 0);
    expect_status(cancel_ids, ReplyState::Cancelled, 0, 0); replies += 2; scenarios |= 4;

    let deadline_ids = Identifiers::new(4, 1).unwrap();
    submit(deadline_ids, 6, 0, 4096, 1, &long); requests += 1;
    expect_status(deadline_ids, ReplyState::Deadline, 0, 0); replies += 1; scenarios |= 8;

    if requests != 7 || replies != 7 || scenarios != 15 { fail(31); }
    write(56, scenarios); write(64, replies); write(72, requests); write(80, 0);
    write(88, 1); write(96, 1); write(104, 2); write(112, 0); write(48, 1); worker()
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    if native::check_abi().is_err() || read(16) == 0 || expected_peer() == 0 { fail(32); }
    for offset in (48..120).step_by(8) { write(offset, 0); }
    match read(0) { 1 => service(), 2 => client(), _ => fail(33) }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { fail(0xff) }

#[no_mangle]
unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, size: usize) -> *mut u8 {
    unsafe { asm!("cld", "rep movsb", inout("rdi") dst => _, inout("rsi") src => _, inout("rcx") size => _, options(nostack)); }
    dst
}
#[no_mangle]
unsafe extern "C" fn memset(dst: *mut u8, byte: i32, size: usize) -> *mut u8 {
    unsafe { asm!("cld", "rep stosb", inout("rdi") dst => _, in("al") byte as u8, inout("rcx") size => _, options(nostack)); }
    dst
}
#[no_mangle]
unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, size: usize) -> *mut u8 {
    if dst as usize <= src as usize {
        unsafe { asm!("cld", "rep movsb", inout("rdi") dst => _, inout("rsi") src => _, inout("rcx") size => _, options(nostack)); }
    } else {
        for offset in (0..size).rev() { unsafe { dst.add(offset).write_volatile(src.add(offset).read_volatile()) }; }
    }
    dst
}
#[no_mangle]
unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, size: usize) -> i32 {
    for offset in 0..size {
        let (left, right) = unsafe { (a.add(offset).read_volatile(), b.add(offset).read_volatile()) };
        if left != right { return i32::from(left) - i32::from(right); }
    }
    0
}
