//! M6c object-service fixture and bounded block broker. Object policy lives in
//! ring 3; the observer below checks the fixed native acceptance fixture only.
//! Single CPU, IF0; owned snapshots, no retained userspace pointer or DMA alias.
use super::*;
use capability_core::{BlockScope, Message, BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE, RECV, SEND};
use catalog_core::{journal, wire};
use virtio_blk_driver::{Device, Discovery, DmaPool};
use virtio_core::{Binding, Completion, Operation, Phase, Policy, Request};
const TIMEOUT: u64 = 20;
const RESET_TIMEOUT: u64 = 10;
struct Active {
    header: wire::Header,
    domain: usize,
    status: wire::Status,
    response: [u8; wire::SIZE],
    candidate: Option<journal::Prepared>,
    calls: u64,
    publications: [u64; 3],
    completions: [u64; 3],
    write_done: bool,
    flush_done: bool,
}
struct Runtime {
    info: Option<Discovery>,
    mapped: [u64; 6],
    device: Option<Device>,
    pool: DmaPool,
    policy: Policy,
    data: [u8; 512],
    error: Option<(Request, i64)>,
    offline: i64,
    resetting: bool,
    reset_deadline: u64,
    quarantined: bool,
    now: u64,
    roots: [u64; 2],
    replies: [u64; 2],
    client_send: [u64; 2],
    client_reply: [u64; 2],
    calls: u64,
    publications: [u64; 3],
    completions: [u64; 3],
    records: [[u8; 512]; 32],
    read_mask: u128,
    current: Option<journal::State>,
    active: Option<Active>,
    ready: bool,
    requests: [u64; 2],
    reply_bytes: [[u8; 224]; 2],
    reply_pending: [bool; 2],
    mutation_pending: [bool; 2],
    ack_sent: u64,
    ack_received: u64,
    get_replies: u64,
    catalog_replies: u64,
    rejected: u64,
    app_device_denials: [u64; 2],
    app_ipc_denials: [u64; 2],
    stale_denials: [u64; 2],
    last_number: [u64; MAX],
    last_result: [i64; MAX],
    finish: bool,
    finish_counters: [u64; 2],
}
static mut RUNTIME: Runtime = Runtime {
    info: None,
    mapped: [0; 6],
    device: None,
    pool: DmaPool {
        descriptors: 0,
        available: 0,
        used: 0,
        request: 0,
        bounce: 0,
    },
    policy: Policy::new(),
    data: [0; 512],
    error: None,
    offline: 0,
    resetting: false,
    reset_deadline: 0,
    quarantined: false,
    now: 0,
    roots: [0; 2],
    replies: [0; 2],
    client_send: [0; 2],
    client_reply: [0; 2],
    calls: 0,
    publications: [0; 3],
    completions: [0; 3],
    records: [[0; 512]; 32],
    read_mask: 0,
    current: None,
    active: None,
    ready: false,
    requests: [0; 2],
    reply_bytes: [[0; 224]; 2],
    reply_pending: [false; 2],
    mutation_pending: [false; 2],
    ack_sent: 0,
    ack_received: 0,
    get_replies: 0,
    catalog_replies: 0,
    rejected: 0,
    app_device_denials: [0; 2],
    app_ipc_denials: [0; 2],
    stale_denials: [0; 2],
    last_number: [0; MAX],
    last_result: [0; MAX],
    finish: false,
    finish_counters: [0; 2],
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: The single CPU enters here with IF0 on the permanent kernel stack.
    // Callers end each borrow before re-entry, scheduling or cleanup/detach.
    unsafe { &mut *ptr::addr_of_mut!(RUNTIME) }
}
pub fn prepare(frames: &mut Frames<FRAME_COUNT>, kernel: &mut Paging) {
    let mut pages = [0; 5];
    for page in &mut pages {
        *page = frames
            .allocate(Owner::Kernel, paging::zero_dynamic)
            .unwrap();
        kernel.map(frames, *page, *page, true, false);
    }
    let pool = DmaPool {
        descriptors: pages[0],
        available: pages[1],
        used: pages[2],
        request: pages[3],
        bounce: pages[4],
    };
    pool.validate(PAGE * FRAME_COUNT as u64).unwrap();
    runtime().pool = pool;
    let info = match virtio_blk_driver::discover() {
        Ok(i) => i,
        Err(_) => {
            runtime().offline = -14;
            return;
        }
    };
    let mut mapped = [0u64; 6];
    for region in [
        info.regions.common,
        info.regions.notify,
        info.regions.isr,
        info.regions.device,
    ] {
        let index = region.bar as usize;
        let bar = info.bars[index];
        assert!(bar.base >= 0x80000000 && bar.size <= 65536);
        mapped[index] = 0x3f000000 + index as u64 * 65536;
        let first = u64::from(region.offset) & !(PAGE - 1);
        let end = (u64::from(region.offset) + u64::from(region.length) + PAGE - 1) & !(PAGE - 1);
        for offset in (first..end).step_by(PAGE as usize) {
            let va = mapped[index] + offset;
            let pa = bar.base + offset;
            if kernel.leaf(va) == 0 {
                kernel.map_mmio(frames, va, pa).unwrap();
            }
            assert_eq!(kernel.leaf(va) & (4 | NX | 8 | 16), NX | 8 | 16);
        }
    }
    // SAFETY: All validated BAR subregions have permanent supervisor UC/NX
    // mappings; five distinct owned kernel RAM pages remain pinned to reboot.
    let device = unsafe { Device::initialize_profile(info, mapped, pool, true, false) };
    let r = runtime();
    r.info = Some(info);
    r.mapped = mapped;
    match device {
        Ok(d) => {
            r.device = Some(d);
            r.policy.reset_completed(true).unwrap();
        }
        Err(_) => {
            r.offline = -14;
            r.quarantined = true;
        }
    }
}

fn err(e: virtio_core::Error) -> i64 {
    use virtio_core::Error::*;
    match e {
        Busy | Pending => -8,
        Range | Invalid | WrongTicket | StaleEpoch => -1,
        Protocol => -15,
        Io | Unsupported => -12,
        Timeout => -13,
        Offline | Exhausted => -14,
    }
}
fn operation_index(op: Operation) -> usize {
    match op {
        Operation::Read { .. } => 0,
        Operation::Write { .. } => 1,
        Operation::Flush => 2,
    }
}
fn required(op: Operation) -> u64 {
    match op {
        Operation::Read { .. } => BLOCK_READ,
        Operation::Write { .. } => BLOCK_WRITE,
        Operation::Flush => BLOCK_FLUSH,
    }
}
fn input_plan(s: &State, slot: usize, pointer: u64) -> Result<user::CopyPlan, i64> {
    let p = &s.processes[slot];
    user::sector_input_plan(pointer, p.pid, |va| {
        let (physical, writable, _, user) = p.paging.as_ref()?.permissions(va)?;
        let page = physical & !(PAGE - 1);
        let Owner::User(owner, _) = s.frames.owner(page).ok()? else {
            return None;
        };
        Some(user::PageAccess {
            physical: page,
            owner,
            user,
            writable,
        })
    })
    .map_err(|_| -3)
}
fn input_bytes(plan: &user::CopyPlan) -> [u8; 512] {
    let mut bytes = [0; 512];
    let mut at = 0;
    for c in &plan.chunks[..plan.count] {
        paging::physical_read(c.physical, |source| {
            for i in 0..c.length {
                // SAFETY: Whole plan was validated caller-owned/effectively USER;
                // RO scratch reads exact bytes, without a writable text alias.
                bytes[at + i] = unsafe { source.add(c.offset + i).read_volatile() };
            }
        });
        at += c.length;
    }
    assert_eq!(at, 512);
    bytes
}

fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
}
pub fn manifest(s: &mut State) {
    assert_eq!(s.amount, 3);
    let e = m4::engine();
    for slot in 0..3 {
        e.register(s.processes[slot].pid).unwrap();
    }
    let service = s.processes[0].pid;
    let clients = [s.processes[1].pid, s.processes[2].pid];
    let r = runtime();
    for (domain, client) in clients.into_iter().enumerate() {
        let ingress = e.create_endpoint(service).unwrap();
        let reply = e.create_endpoint(client).unwrap();
        r.client_send[domain] = e
            .bootstrap_delegate(service, ingress, client, SEND)
            .unwrap();
        r.roots[domain] = e.derive(service, ingress, RECV).unwrap();
        r.replies[domain] = e.bootstrap_delegate(client, reply, service, SEND).unwrap();
        r.client_reply[domain] = e.derive(client, reply, RECV).unwrap();
        e.close(service, ingress).unwrap();
        e.close(client, reply).unwrap();
    }
    let block = e
        .create_block_device_with_rights(
            service,
            BlockScope {
                device: 0,
                first_sector: 4,
                sector_count: 32,
            },
            BLOCK_READ | BLOCK_WRITE | BLOCK_FLUSH,
        )
        .unwrap();
    for slot in 0..3 {
        for (offset, value) in [
            (0, slot as u64),
            (32, 1),
            (40, 0),
            (112, u64::from(case())),
            (160, clients[0]),
            (168, clients[1]),
            (176, service),
            (256, 0),
            (264, 0),
            (272, 0),
        ] {
            put(s, slot, offset, value);
        }
    }
    for (offset, value) in [
        (48, block),
        (56, r.roots[0]),
        (80, r.roots[1]),
        (88, r.replies[0]),
        (72, r.replies[1]),
        (96, 4),
        (104, 32),
    ] {
        put(s, 0, offset, value);
    }
    for domain in 0..2 {
        put(s, domain + 1, 56, r.client_reply[domain]);
        put(s, domain + 1, 64, r.client_send[domain]);
        assert!(e
            .check_block(clients[domain], r.client_send[domain], BLOCK_READ)
            .is_err());
        assert!(e
            .check_endpoint(clients[domain], r.client_send[domain], RECV)
            .is_err());
    }
    assert!(e.check_invariants());
}

pub fn syscall(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> (i64, u64) {
    if slot == 0 {
        runtime().calls += 1;
    }
    let result = device_call(s, slot, n, a);
    let r = runtime();
    r.last_number[slot] = n;
    r.last_result[slot] = result.as_ref().err().copied().unwrap_or(0);
    match result {
        Ok(v) => (0, v),
        Err(code) => {
            if slot > 0 && matches!(code, -4 | -6) {
                r.app_device_denials[slot - 1] += 1;
            }
            (code, 0)
        }
    }
}
fn device_call(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> Result<u64, i64> {
    if !matches!(n, 0x40..=0x43) {
        return Err(-2);
    }
    if a[4] != 0 || a[5] != 0 || n == 0x43 && a[1..4].iter().any(|v| *v != 0) {
        return Err(-1);
    }
    if n == 0x41 && a[3] != 320 || n == 0x42 && a[3] != 512 {
        return Err(-11);
    }
    let pid = s.processes[slot].pid;
    let rights = match n {
        0x40 => BLOCK_READ,
        0x42 => BLOCK_WRITE,
        0x43 => BLOCK_FLUSH,
        _ => 0,
    };
    let auth = m4::engine()
        .check_block(pid, a[0], rights)
        .map_err(|e| e.code())?;
    if auth.scope().device != 0 {
        return Err(-5);
    }
    let binding = Binding {
        pid,
        object_key: auth.object_key(),
    };
    if n == 0x41 {
        let request = runtime()
            .error
            .map(|(q, _)| q)
            .or_else(|| runtime().policy.request())
            .ok_or(-1)?;
        m4::engine()
            .check_block(pid, a[0], required(request.operation))
            .map_err(|e| e.code())?;
        if request.binding != binding || request.ticket != a[1] {
            return Err(-1);
        }
        let output = m4::plan(s, slot, a[2], 320, true).map_err(|e| e.code())?;
        let r = runtime();
        if let Some((_, code)) = r.error.take() {
            if r.policy.peek(binding, a[1]).is_ok() {
                let _ = r.policy.consume(binding, a[1]);
            }
            return Err(code);
        }
        let request = r.policy.peek(binding, a[1]).map_err(err)?;
        if request.completion == Some(Completion::Unknown) {
            r.policy.consume(binding, a[1]).map_err(err)?;
            return Err(-16);
        }
        if request.completion != Some(Completion::Success) {
            return Err(-12);
        }
        let mut message = Message::new();
        message.tag = a[1];
        if matches!(request.operation, Operation::Read { .. }) {
            message.len = u32::from(request.length);
            let start = usize::from(request.offset);
            let end = start + usize::from(request.length);
            message.data[..usize::from(request.length)].copy_from_slice(&r.data[start..end]);
        }
        let bytes = message.encode();
        m4::write_plan(&output, &bytes);
        assert_eq!(m4::read_plan(&output), bytes);
        r.policy.consume(binding, a[1]).map_err(err)?;
        return Ok(u64::from(message.len));
    }
    let op = if n == 0x43 {
        Operation::Flush
    } else {
        if n == 0x40 && (a[3] == 0 || a[3] > 128) {
            return Err(-11);
        }
        if n == 0x40 && a[2].checked_add(a[3]).filter(|v| *v <= 512).is_none() {
            return Err(-1);
        }
        if a[1] >= auth.scope().sector_count {
            return Err(-5);
        }
        let sector = auth.sector(a[1]).map_err(|e| e.code())?;
        if n == 0x40 {
            Operation::Read {
                sector,
                offset: a[2],
                length: a[3],
            }
        } else {
            Operation::Write { sector }
        }
    };
    let input = if n == 0x42 {
        Some(input_bytes(&input_plan(s, slot, a[2])?))
    } else {
        None
    };
    let r = runtime();
    if r.offline != 0 {
        return Err(r.offline);
    }
    if r.resetting || r.error.is_some() || r.policy.phase() != Phase::Ready {
        return Err(-8);
    }
    let capacity = r.device.as_ref().ok_or(-14)?.capacity;
    // Acceptance observer only: the broker's authority checks above do not
    // know object IDs, domains or journal revisions.
    if let Some(bytes) = input.as_ref() {
        let active = r
            .active
            .as_ref()
            .expect("WRITE without received object request");
        let candidate = active
            .candidate
            .as_ref()
            .expect("WRITE for rejected/nonmutating request");
        assert_eq!(slot, 0);
        assert_eq!(*bytes, *candidate.record());
        assert_eq!(
            op,
            Operation::Write {
                sector: 4 + r.current.as_ref().unwrap().sequence()
            }
        );
        assert!(!active.write_done && !active.flush_done);
    }
    if op == Operation::Flush && r.ready {
        let active = r.active.as_ref().expect("FLUSH without mutation");
        assert!(active.candidate.is_some() && active.write_done && !active.flush_done);
    }
    let request = r
        .policy
        .prepare(binding, op, capacity, s.ticks, TIMEOUT)
        .map_err(err)?;
    if let Err(error) = r
        .device
        .as_mut()
        .unwrap()
        .prepare_operation(op, input.as_ref(), false)
    {
        r.policy.reject_prepared(binding, request.ticket).unwrap();
        return Err(err(error));
    }
    if case() == 5
        && matches!(op, Operation::Write { .. })
        && r.active
            .as_ref()
            .is_some_and(|active| active.header.operation == wire::RENAME)
    {
        assert_eq!(r.current.as_ref().unwrap().sequence(), 1);
        cut(1);
    }
    r.policy.publish(request.epoch, request.ticket).unwrap();
    r.device.as_mut().unwrap().publish_prepared(false).unwrap();
    r.publications[operation_index(op)] += 1;
    Ok(request.ticket)
}
fn reset(r: &mut Runtime, now: u64) {
    if r.quarantined {
        return;
    }
    r.policy.start_reset().unwrap();
    r.resetting = true;
    r.reset_deadline = now + RESET_TIMEOUT;
    if let Some(device) = r.device.as_mut() {
        device.begin_reset();
    }
}
pub fn detach(pid: u64) {
    let r = runtime();
    if r.error.is_some_and(|(q, _)| q.binding.pid == pid) {
        r.error = None;
    }
    if r.policy.exit(pid).is_some() && !r.quarantined {
        let now = r.now;
        reset(r, now);
    }
}
fn advance(s: &State) {
    let r = runtime();
    let now = s.ticks;
    r.now = now;
    if r.resetting {
        if r.device.as_ref().is_some_and(Device::reset_acknowledged) {
            // SAFETY: Real reset acknowledgement permits reuse of the same
            // pinned owned RAM pages and permanent validated supervisor MMIO.
            match unsafe {
                Device::initialize_profile(r.info.unwrap(), r.mapped, r.pool, true, false)
            } {
                Ok(device) => {
                    r.device = Some(device);
                    r.policy.reset_completed(true).unwrap();
                    r.resetting = false;
                }
                Err(_) => {
                    r.offline = -14;
                    r.resetting = false;
                    r.quarantined = true;
                }
            }
        } else if now >= r.reset_deadline {
            let _ = r.policy.reset_completed(false);
            r.resetting = false;
            r.quarantined = true;
            r.offline = -13;
        }
        return;
    }
    if r.offline != 0 || r.policy.phase() != Phase::Published {
        return;
    }
    let request = r.policy.request().unwrap();
    if r.policy.timeout(now) {
        r.error = Some((
            request,
            if request.operation.is_mutating() {
                -16
            } else {
                -13
            },
        ));
        reset(r, now);
        return;
    }
    match r.device.as_mut().unwrap().poll(0) {
        Ok(Some((data, Completion::Success))) => {
            r.data = data;
            r.completions[operation_index(request.operation)] += 1;
            r.policy
                .complete(request.epoch, request.ticket, Completion::Success)
                .unwrap();
            match request.operation {
                Operation::Read {
                    sector,
                    offset,
                    length,
                } => {
                    assert!((4..36).contains(&sector));
                    let index = (sector - 4) as usize;
                    let start = offset as usize;
                    let end = start + length as usize;
                    r.records[index][start..end].copy_from_slice(&data[start..end]);
                    if length == 128 && offset % 128 == 0 {
                        r.read_mask |= 1u128 << (index * 4 + start / 128);
                    }
                }
                Operation::Write { .. } => {
                    let active = r.active.as_mut().unwrap();
                    assert!(active.candidate.is_some());
                    active.write_done = true;
                }
                Operation::Flush if r.ready => {
                    let active = r.active.as_mut().unwrap();
                    assert!(active.candidate.is_some() && active.write_done);
                    active.flush_done = true;
                    if case() == 6 && active.header.operation == wire::RENAME {
                        assert_eq!(r.current.as_ref().unwrap().sequence(), 1);
                        cut(2);
                    }
                }
                Operation::Flush => {}
            }
        }
        Ok(Some((_, _))) => {
            r.policy
                .complete(request.epoch, request.ticket, Completion::Io)
                .unwrap();
            r.error = Some((
                request,
                if request.operation.is_mutating() {
                    -16
                } else {
                    -12
                },
            ));
            reset(r, now);
        }
        Ok(None) => {}
        Err(error) => {
            r.error = Some((
                request,
                if request.operation.is_mutating() {
                    -16
                } else {
                    err(error)
                },
            ));
            reset(r, now);
        }
    }
}
fn cut(point: u64) -> ! {
    number(b"M6C:CUT:", point);
    loop {
        // SAFETY: Deliberate IF0 test rendezvous on the kernel stack. No PASS
        // is emitted; the harness must kill this live QEMU child process.
        unsafe {
            asm!("cli; hlt", options(nomem, nostack));
        }
    }
}

fn journal_status(error: journal::Error) -> wire::Status {
    match error {
        journal::Error::Invalid => wire::Status::Invalid,
        journal::Error::Range => wire::Status::Range,
        journal::Error::Stale => wire::Status::Stale,
        journal::Error::Full => wire::Status::Full,
        journal::Error::NotFound => wire::Status::NotFound,
        journal::Error::Corrupt => wire::Status::Corrupt,
    }
}
fn observe_ready(s: &State) {
    let r = runtime();
    if r.ready || word(s, 0, 192) != 1 {
        return;
    }
    assert_eq!(r.read_mask, u128::MAX);
    assert_eq!(r.publications, [128, 0, 1]);
    assert_eq!(r.completions, r.publications);
    assert_eq!(r.policy.phase(), Phase::Ready);
    let mut recovery = journal::Recovery::new();
    for (index, record) in r.records.iter().enumerate() {
        recovery.push(index, record).unwrap();
    }
    let state = recovery.finish().unwrap();
    assert_eq!(word(s, 0, 128), state.sequence());
    if case() != 1 {
        assert_eq!(state.sequence(), 0);
    }
    r.current = Some(state);
    r.ready = true;
    serial::write(b"M6C:READY\n");
    // Test fixture barrier only; creates/delegates no capability. Both clients
    // still use their distinct kernel-bound ingress and reply endpoints.
    for slot in 1..3 {
        put(s, slot, 256, 1);
    }
}
pub fn sent(_s: &State, slot: usize, handle: u64, message: &Message) {
    let r = runtime();
    if slot != 0 {
        assert_eq!(handle, r.client_send[slot - 1]);
        return;
    }
    let mut active = r.active.take().expect("reply without received request");
    assert_eq!(handle, r.replies[active.domain]);
    assert_eq!(message.tag, wire::TAG);
    assert_eq!(message.len, wire::SIZE as u32);
    assert_eq!(message.cap_count, 0);
    assert!(message.data[wire::SIZE..].iter().all(|b| *b == 0));
    let mut body = [0; wire::SIZE];
    body.copy_from_slice(&message.data[..wire::SIZE]);
    assert_eq!(body, active.response);
    let mutating = active.candidate.is_some();
    if mutating {
        assert!(active.write_done && active.flush_done);
        assert_eq!(active.status, wire::Status::Ok);
        assert_eq!(
            r.publications,
            [
                active.publications[0],
                active.publications[1] + 1,
                active.publications[2] + 1
            ]
        );
        assert_eq!(
            r.completions,
            [
                active.completions[0],
                active.completions[1] + 1,
                active.completions[2] + 1
            ]
        );
        r.current
            .as_mut()
            .unwrap()
            .commit(active.candidate.take().unwrap())
            .unwrap();
        r.ack_sent += 1;
    } else {
        // Malformed, wrong-domain, stale, full, absent and GET operations must
        // never cause even a rejected device syscall in this memory-backed view.
        assert_eq!(
            (r.calls, r.publications, r.completions),
            (active.calls, active.publications, active.completions)
        );
        if active.status != wire::Status::Ok {
            r.rejected += 1;
        } else if active.header.operation == wire::LIST {
            r.catalog_replies += 1;
        } else {
            assert_eq!(active.header.operation, wire::GET);
            r.get_replies += 1;
        }
    }
    assert!(!r.reply_pending[active.domain]);
    r.reply_bytes[active.domain] = body;
    r.reply_pending[active.domain] = true;
    r.mutation_pending[active.domain] = mutating;
}
pub fn received(s: &State, slot: usize, handle: u64, message: &Message) {
    let r = runtime();
    let mut body = [0; wire::SIZE];
    body.copy_from_slice(&message.data[..wire::SIZE]);
    if slot == 0 {
        assert!(r.ready && r.active.is_none());
        let domain = r.roots.iter().position(|root| *root == handle).unwrap();
        assert_eq!(message.sender_pid, s.processes[domain + 1].pid);
        let header = wire::header(&body);
        let mut active = Active {
            header,
            domain,
            status: wire::Status::Ok,
            response: [0; wire::SIZE],
            candidate: None,
            calls: r.calls,
            publications: r.publications,
            completions: r.completions,
            write_done: false,
            flush_done: false,
        };
        let valid = if message.cap_count != 0
            || message.tag != wire::TAG
            || message.len != wire::SIZE as u32
        {
            Err(wire::Status::Invalid)
        } else {
            wire::validate_request(&body, 1, domain)
        };
        match valid {
            Err(error) => {
                active.status = error;
                active.response = wire::reply(header, 1, 0, error, None, None);
            }
            Ok(h) => {
                let state = r.current.as_ref().unwrap();
                if h.operation == wire::LIST {
                    let catalog = state.catalog(domain).unwrap();
                    active.response = wire::catalog_reply(h, 1, wire::Status::Ok, Some(&catalog));
                } else {
                    let object = state.get(h.object).unwrap();
                    if h.operation == wire::GET {
                        if object.present {
                            active.response = wire::reply(
                                h,
                                1,
                                object.revision,
                                wire::Status::Ok,
                                Some(&object.name[..object.name_length]),
                                Some(&object.value[..object.length]),
                            );
                        } else {
                            active.status = wire::Status::NotFound;
                            active.response =
                                wire::reply(h, 1, object.revision, active.status, None, None);
                        }
                    } else {
                        match state.prepare(
                            h.object,
                            h.revision,
                            h.request_id,
                            h.operation,
                            &body[192..192 + h.name_length as usize],
                            &body[64..64 + h.length as usize],
                        ) {
                            Ok(candidate) => {
                                active.response =
                                    wire::reply(h, 1, h.revision + 1, wire::Status::Ok, None, None);
                                active.candidate = Some(candidate);
                            }
                            Err(error) => {
                                active.status = journal_status(error);
                                let revision = if active.status == wire::Status::NotFound {
                                    object.revision
                                } else {
                                    0
                                };
                                active.response =
                                    wire::reply(h, 1, revision, active.status, None, None);
                            }
                        }
                    }
                }
            }
        }
        r.requests[domain] += 1;
        r.active = Some(active);
    } else {
        let domain = slot - 1;
        assert_eq!(handle, r.client_reply[domain]);
        assert!(r.reply_pending[domain]);
        assert_eq!(message.sender_pid, s.processes[0].pid);
        assert_eq!(message.tag, wire::TAG);
        assert_eq!(message.len, wire::SIZE as u32);
        assert_eq!(message.cap_count, 0);
        assert_eq!(body, r.reply_bytes[domain]);
        r.reply_pending[domain] = false;
        if r.mutation_pending[domain] {
            r.ack_received += 1;
            assert!(r.ack_received <= r.ack_sent);
            if case() == 2 && r.ack_received == 4 {
                // Fixture-only barrier: both domains now have real documents.
                // It grants no authority and cannot fabricate service ACKs.
                for client in 1..3 {
                    put(s, client, 272, 1);
                }
            }
            if case() == 3 && r.ack_received == 32 {
                for client in 1..3 {
                    put(s, client, 264, 1);
                }
            }
        }
        r.mutation_pending[domain] = false;
    }
}
pub fn cap_event(_slot: usize, _number: u64) {}
pub fn ipc_result(slot: usize, number: u64, result: i64) {
    let r = runtime();
    r.last_number[slot] = number;
    r.last_result[slot] = result;
    if slot > 0 {
        if result == -5 {
            r.app_ipc_denials[slot - 1] += 1;
        }
        if number == 0x20 && result == -4 {
            r.stale_denials[slot - 1] += 1;
        }
    }
}
fn diagnostic(s: &State, slot: usize) {
    number(b"M6C:USER_SLOT:", slot as u64);
    number(b"M6C:USER_CODE:", word(s, slot, 240));
    number(b"M6C:USER_SYSCALL:", runtime().last_number[slot]);
    let error = runtime().last_result[slot];
    if (-19..=0).contains(&error) {
        number(b"M6C:USER_ERRNO:", (-error) as u64);
    }
}
pub fn observe_exit(s: &mut State, slot: usize) {
    diagnostic(s, slot);
    s.unexpected = true;
    s.observer_survivor(slot);
}
pub fn observe_fault(s: &mut State, slot: usize, _context: &[u64; 22]) {
    diagnostic(s, slot);
    s.unexpected = true;
    s.observer_survivor(slot);
}
fn expected_value(object: u64, revision: u64) -> [u8; 128] {
    let mut bytes = [0; 128];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = (0x31 + object * 29 + revision * 7 + index as u64 * 3) as u8;
    }
    bytes
}
fn expected_name(object: u64, revision: u64) -> &'static [u8] {
    let old: [&[u8]; 4] = [
        "Café A".as_bytes(),
        b"Werk A",
        "Café B".as_bytes(),
        b"Werk B",
    ];
    let new: [&[u8]; 4] = [
        "Café hernoemd A".as_bytes(),
        b"Werk hernoemd A",
        "Café hernoemd B".as_bytes(),
        b"Werk hernoemd B",
    ];
    if revision < 2 {
        old[object as usize]
    } else {
        new[object as usize]
    }
}
fn print_objects(current: &journal::State) {
    number(b"M6C:SEQUENCE:", current.sequence());
    let revisions: [&[u8]; 4] = [
        b"M6C:OBJECT0:REVISION:",
        b"M6C:OBJECT1:REVISION:",
        b"M6C:OBJECT2:REVISION:",
        b"M6C:OBJECT3:REVISION:",
    ];
    let present: [&[u8]; 4] = [
        b"M6C:OBJECT0:PRESENT:",
        b"M6C:OBJECT1:PRESENT:",
        b"M6C:OBJECT2:PRESENT:",
        b"M6C:OBJECT3:PRESENT:",
    ];
    let lengths: [&[u8]; 4] = [
        b"M6C:OBJECT0:LENGTH:",
        b"M6C:OBJECT1:LENGTH:",
        b"M6C:OBJECT2:LENGTH:",
        b"M6C:OBJECT3:LENGTH:",
    ];
    let digests: [&[u8]; 4] = [
        b"M6C:OBJECT0:DIGEST:",
        b"M6C:OBJECT1:DIGEST:",
        b"M6C:OBJECT2:DIGEST:",
        b"M6C:OBJECT3:DIGEST:",
    ];
    let name_lengths: [&[u8]; 4] = [
        b"M6C:OBJECT0:NAME_LENGTH:",
        b"M6C:OBJECT1:NAME_LENGTH:",
        b"M6C:OBJECT2:NAME_LENGTH:",
        b"M6C:OBJECT3:NAME_LENGTH:",
    ];
    let name_digests: [&[u8]; 4] = [
        b"M6C:OBJECT0:NAME_DIGEST:",
        b"M6C:OBJECT1:NAME_DIGEST:",
        b"M6C:OBJECT2:NAME_DIGEST:",
        b"M6C:OBJECT3:NAME_DIGEST:",
    ];
    for object in 0..4 {
        let entry = current.get(object as u64).unwrap();
        if entry.present {
            assert_eq!(entry.length, 128);
            let content_revision = if entry.revision == 2 {
                1
            } else {
                entry.revision
            };
            assert_eq!(entry.value, expected_value(object as u64, content_revision));
            assert_eq!(
                &entry.name[..entry.name_length],
                expected_name(object as u64, entry.revision)
            );
        }
        number(revisions[object], entry.revision);
        number(present[object], u64::from(entry.present));
        number(lengths[object], entry.length as u64);
        number(name_lengths[object], entry.name_length as u64);
        hex(
            name_digests[object],
            wire::checksum_update(wire::FNV_OFFSET, &entry.name[..entry.name_length]),
        );
        hex(
            digests[object],
            wire::checksum_update(wire::FNV_OFFSET, &entry.value[..entry.length]),
        );
    }
}
pub fn poll(s: &mut State) -> Option<*const u64> {
    advance(s);
    if s.unexpected {
        serial::fail(b"M6C:FAIL:USER_OR_PROTOCOL\n");
    }
    observe_ready(s);
    for slot in 0..s.amount {
        if word(s, slot, 40) >= 0x100 {
            diagnostic(s, slot);
            serial::fail(b"M6C:FAIL:USER_STATUS\n");
        }
        assert_eq!(word(s, slot, 16), s.processes[slot].pattern);
    }
    let done = if matches!(case(), 4 | 7) {
        word(s, 0, 40) == 1
    } else {
        word(s, 1, 40) == 1 && word(s, 2, 40) == 1
    };
    if !done {
        return None;
    }
    let pool = {
        let r = runtime();
        if !r.finish {
            r.finish = true;
            r.finish_counters = [word(s, 1, 8), word(s, 2, 8)];
            return None;
        }
        if (1..3).any(|slot| word(s, slot, 8) <= r.finish_counters[slot - 1] + 8) || r.resetting {
            return None;
        }
        assert!(r.active.is_none() && r.reply_pending == [false; 2]);
        if matches!(case(), 4 | 7) {
            assert!(!r.ready);
            assert_eq!(word(s, 0, 232) as i64, -18);
            assert_eq!(r.publications[1..], [0, 0]);
            assert_eq!((r.ack_sent, r.ack_received), (0, 0));
            let mut recovery = journal::Recovery::new();
            let mut rejected = false;
            for index in 0..32 {
                if (r.read_mask >> (index * 4)) & 15 != 15 {
                    break;
                }
                if recovery.push(index, &r.records[index]).is_err() {
                    rejected = true;
                    break;
                }
            }
            assert!(rejected);
        } else {
            assert!(r.ready && r.get_replies >= if case() == 1 { 0 } else { 2 });
            assert_eq!(r.ack_sent, r.ack_received);
            assert_eq!(r.publications, r.completions);
            assert_eq!(r.publications, [128, r.ack_received, r.ack_received + 1]);
            assert!(r.app_device_denials.iter().all(|count| *count >= 4));
            assert!(r.app_ipc_denials.iter().all(|count| *count >= 3));
            assert_eq!(r.stale_denials, [1, 1]);
            let requests = match case() {
                0 => 16,
                1 => 3,
                2 => 53,
                3 => 23,
                _ => 0,
            };
            assert_eq!(r.requests, [requests; 2]);
            let current = r.current.as_ref().unwrap();
            match case() {
                0 | 2 => {
                    assert_eq!((current.sequence(), r.ack_received), (10, 10));
                    assert_eq!(r.catalog_replies, 10);
                    for object in 0..4 {
                        let entry = current.get(object).unwrap();
                        assert_eq!(entry.revision, if object % 2 == 0 { 3 } else { 2 });
                        assert_eq!(entry.present, object % 2 == 0);
                    }
                    assert_eq!(r.rejected, if case() == 2 { 80 } else { 6 });
                }
                1 => {
                    assert_eq!(r.ack_received, 0);
                    assert_eq!(r.catalog_replies, 2);
                }
                3 => {
                    assert_eq!((current.sequence(), r.ack_received), (32, 32));
                    for object in 0..4 {
                        let entry = current.get(object).unwrap();
                        assert_eq!(entry.revision, if object % 2 == 0 { 16 } else { 0 });
                        assert_eq!(entry.present, object % 2 == 0);
                    }
                    assert_eq!(r.rejected, 8);
                    assert_eq!(r.catalog_replies, 4);
                }
                _ => panic!("crash fixture reached PASS"),
            }
            print_objects(current);
            number(b"M6C:ACKS:", r.ack_received);
            number(b"M6C:REJECTED_NO_IO:", r.rejected);
            number(b"M6C:CATALOG_REPLIES:", r.catalog_replies);
            number(
                b"M6C:DOMAIN0:REVISION:",
                current.catalog(0).unwrap().generation,
            );
            number(
                b"M6C:DOMAIN1:REVISION:",
                current.catalog(1).unwrap().generation,
            );
        }
        assert_eq!(r.policy.phase(), Phase::Ready);
        assert!(m4::engine().check_invariants());
        serial::write(marker());
        serial::write(b"M6C:SURVIVOR_PROGRESS:PASS\n");
        if let Some(device) = r.device.as_mut() {
            device.begin_reset();
            assert!(device.reset_acknowledged());
        }
        r.pool
    };
    // The Runtime borrow ended before cleanup calls detach. Pinned DMA pages
    // stay owned by the kernel; no live user mapping aliases them.
    for slot in 0..s.amount {
        s.processes[slot].status = Status::Dying;
        s.cleanup(slot);
    }
    let stats = m4::engine().stats();
    assert_eq!(
        (
            stats.processes,
            stats.handles,
            stats.nodes,
            stats.endpoints,
            stats.block_objects,
            stats.queued_messages
        ),
        (0, 0, 0, 0, 0, 0)
    );
    for page in [
        pool.descriptors,
        pool.available,
        pool.used,
        pool.request,
        pool.bounce,
    ] {
        assert_eq!(s.frames.owner(page), Ok(Owner::Kernel));
        assert_eq!(s.kernel.leaf(page) & (4 | NX | 2), NX | 2);
    }
    serial::write(b"M6C:RESOURCES:PASS\nM6C:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
fn case() -> u8 {
    if cfg!(feature = "test-m6c-recover") {
        1
    } else if cfg!(feature = "test-m6c-errors") {
        2
    } else if cfg!(feature = "test-m6c-full") {
        3
    } else if cfg!(feature = "test-m6c-corrupt") {
        4
    } else if cfg!(feature = "test-m6c-vm-cut-before-rename") {
        5
    } else if cfg!(feature = "test-m6c-vm-cut-after-rename") {
        6
    } else if cfg!(feature = "test-m6c-legacy") {
        7
    } else {
        0
    }
}
fn marker() -> &'static [u8] {
    match case() {
        0 => b"M6C:CASE:NORMAL:PASS\n",
        1 => b"M6C:CASE:RECOVER:PASS\n",
        2 => b"M6C:CASE:ERRORS:PASS\n",
        3 => b"M6C:CASE:FULL:PASS\n",
        4 => b"M6C:CASE:CORRUPT:PASS\n",
        7 => b"M6C:CASE:LEGACY:PASS\n",
        _ => b"M6C:FAIL:CRASH_NOT_OBSERVED\n",
    }
}
fn number(label: &[u8], mut value: u64) {
    let mut digits = [b'0'; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    serial::write(label);
    serial::write(&digits[start..]);
    serial::write(b"\n");
}
fn hex(label: &[u8], value: u64) {
    let mut digits = [b'0'; 16];
    for (index, byte) in digits.iter_mut().enumerate() {
        let digit = ((value >> ((15 - index) * 4)) & 15) as u8;
        *byte = if digit < 10 {
            b'0' + digit
        } else {
            b'a' + digit - 10
        };
    }
    serial::write(label);
    serial::write(&digits);
    serial::write(b"\n");
}
