//! M6a bounded READ/WRITE/FLUSH broker. Only fixed pinned kernel pages are
//! exposed to DMA. User input is copied before publication, never retained.
use super::*;
use block_protocol::{journal, m6_wire as wire};
use capability_core::{BlockScope, Message, BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE};
use virtio_blk_driver::{Device, Discovery, DmaPool};
use virtio_core::{Binding, Completion, Operation, Phase, Policy, Request};
const TIMEOUT: u64 = 20;
const RESET_TIMEOUT: u64 = 10;
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
    epoch: u64,
    restarts: u64,
    need_restart: bool,
    roots: [u64; 2],
    client_roots: [u64; 2],
    super_root: u64,
    initial_client: (u64, u64),
    initial_super: (u64, u64),
    initial_service: u64,
    calls: u64,
    publications: [u64; 3],
    completions: [u64; 3],
    errors: [u64; 19],
    resets: u64,
    injected: bool,
    unknown_seen: bool,
    active_put: bool,
    put_header: Option<wire::Header>,
    put_value: [u8; 128],
    write_completed: bool,
    flush_completed: bool,
    ack_sent: u64,
    ack_received: u64,
    cut_observed: u8,
    cut_fired: bool,
    gate: bool,
    crashes: u64,
    announcements: u64,
    forwards: u64,
    records: [[u8; 512]; 4],
    read_mask: u16,
    latest: Option<journal::State>,
    baseline: Option<journal::State>,
    candidate: Option<journal::State>,
    journal_snapshot: [u8; 512],
    get_replies: u64,
    finish: bool,
    finish_counter: u64,
    finish_super: u64,
    last_number: [u64; MAX],
    last_result: [i64; MAX],
    scratch_snapshot: [u8; 512],
    scratch_writes: u64,
    scratch_reads: u64,
    snapshot_count: u64,
    invalid_outputs: u64,
    close_count: u64,
    bad_requests: u64,
    bad_replies: u64,
    bad: Option<Bad>,
}
#[derive(Clone, Copy)]
struct Bad {
    header: wire::Header,
    expected: i64,
    calls: u64,
    publications: [u64; 3],
    completions: [u64; 3],
    nodes: usize,
    handles: usize,
    closes: u64,
    received: bool,
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
    epoch: 1,
    restarts: 0,
    need_restart: false,
    roots: [0; 2],
    client_roots: [0; 2],
    super_root: 0,
    initial_client: (0, 0),
    initial_super: (0, 0),
    initial_service: 0,
    calls: 0,
    publications: [0; 3],
    completions: [0; 3],
    errors: [0; 19],
    resets: 0,
    injected: false,
    unknown_seen: false,
    active_put: false,
    put_header: None,
    put_value: [0; 128],
    write_completed: false,
    flush_completed: false,
    ack_sent: 0,
    ack_received: 0,
    cut_observed: 0,
    cut_fired: false,
    gate: false,
    crashes: 0,
    announcements: 0,
    forwards: 0,
    records: [[0; 512]; 4],
    read_mask: 0,
    latest: None,
    baseline: None,
    candidate: None,
    journal_snapshot: [0; 512],
    get_replies: 0,
    finish: false,
    finish_counter: 0,
    finish_super: 0,
    last_number: [0; MAX],
    last_result: [0; MAX],
    scratch_snapshot: [0; 512],
    scratch_writes: 0,
    scratch_reads: 0,
    snapshot_count: 0,
    invalid_outputs: 0,
    close_count: 0,
    bad_requests: 0,
    bad_replies: 0,
    bad: None,
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: Single CPU, IF0, permanent kernel service stack; no device IRQ
    // borrows this state. No borrow is retained across cleanup or scheduling.
    unsafe { &mut *ptr::addr_of_mut!(RUNTIME) }
}
pub fn prepare(frames: &mut Frames<FRAME_COUNT>, kernel: &mut Paging) {
    let info = match virtio_blk_driver::discover() {
        Ok(i) => i,
        Err(_) => {
            runtime().offline = -14;
            return;
        }
    };
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
    let device = unsafe { Device::initialize_profile(info, mapped, pool, true, case() == 9) };
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

fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
}
pub fn manifest(s: &mut State) {
    let e = m4::engine();
    for slot in 0..s.amount {
        e.register(s.processes[slot].pid).unwrap();
    }
    let client = s.processes[1].pid;
    let sup = s.processes[2].pid;
    let reply = e.create_endpoint(client).unwrap();
    let control = e.create_endpoint(client).unwrap();
    let ann = e.create_endpoint(sup).unwrap();
    let r = runtime();
    r.client_roots = [reply, control];
    r.super_root = ann;
    r.initial_service = s.processes[0].pid;
    r.initial_client = (client, s.processes[1].paging.as_ref().unwrap().root);
    r.initial_super = (sup, s.processes[2].paging.as_ref().unwrap().root);
    for slot in 0..s.amount {
        for (o, v) in [
            (0, slot as u64),
            (32, 1),
            (40, 0),
            (96, 4),
            (104, 4),
            (112, u64::from(case())),
            (160, client),
            (168, sup),
            (176, r.initial_service),
        ] {
            put(s, slot, o, v);
        }
    }
    put(s, 1, 56, reply);
    put(s, 1, 80, control);
    put(s, 1, 64, 0);
    put(s, 2, 56, ann);
    put(
        s,
        2,
        64,
        e.bootstrap_delegate(client, control, sup, 1).unwrap(),
    );
    bind_service(s);
}
fn block(e: &mut capability_core::Engine, pid: u64, first: u64, count: u64) -> [u64; 4] {
    let admin = e
        .create_block_device_with_rights(
            pid,
            BlockScope {
                device: 0,
                first_sector: first,
                sector_count: count,
            },
            125,
        )
        .unwrap();
    let mut handles = [0; 4];
    for (h, rights) in handles
        .iter_mut()
        .zip([97, BLOCK_READ, BLOCK_WRITE, BLOCK_FLUSH])
    {
        *h = e.derive(pid, admin, rights).unwrap();
    }
    e.close(pid, admin).unwrap();
    handles
}
fn bind_service(s: &mut State) {
    let e = m4::engine();
    let pid = s.processes[0].pid;
    if runtime().restarts != 0 {
        e.register(pid).unwrap();
    }
    let roots = [
        e.create_endpoint(pid).unwrap(),
        e.create_endpoint(pid).unwrap(),
    ];
    runtime().roots = roots;
    let caps = block(e, pid, 4, 4);
    // Only the service's exact 97 authority survives startup. Auxiliary rights
    // are never granted to the app or retained accidentally by the manifest.
    for h in &caps[1..] {
        e.close(pid, *h).unwrap();
    }
    let reply = e
        .bootstrap_delegate(s.processes[1].pid, runtime().client_roots[0], pid, 1)
        .unwrap();
    let ann = e
        .bootstrap_delegate(s.processes[2].pid, runtime().super_root, pid, 1)
        .unwrap();
    for (o, v) in [
        (0, 0),
        (32, runtime().epoch),
        (40, 0),
        (48, caps[0]),
        (56, roots[0]),
        (64, ann),
        (72, ann),
        (80, roots[1]),
        (88, reply),
        (96, 4),
        (104, 4),
        (112, u64::from(case())),
        (160, s.processes[1].pid),
        (168, s.processes[2].pid),
        (176, runtime().initial_service),
    ] {
        put(s, 0, o, v);
    }
    if case() == 2 {
        let scratch = block(e, pid, 12, 1);
        for (o, h) in [200, 208, 216, 224].into_iter().zip(scratch) {
            put(s, 0, o, h);
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
pub fn syscall(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> (i64, u64) {
    if slot == 0 {
        runtime().calls += 1;
    }
    let result = device_call(s, slot, n, a);
    runtime().last_number[slot] = n;
    runtime().last_result[slot] = result.as_ref().err().copied().unwrap_or(0);
    match result {
        Ok(v) => (0, v),
        Err(c) => {
            if (-18..=-1).contains(&c) {
                runtime().errors[(-c) as usize] += 1;
            }
            (c, 0)
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
            .map(|(r, _)| r)
            .or_else(|| runtime().policy.request())
            .ok_or(-1)?;
        m4::engine()
            .check_block(pid, a[0], required(request.operation))
            .map_err(|e| e.code())?;
        if request.binding != binding || request.ticket != a[1] {
            return Err(-1);
        }
        let output = m4::plan(s, slot, a[2], 320, true).map_err(|e| {
            runtime().invalid_outputs += 1;
            e.code()
        })?;
        if let Some((_, code)) = runtime().error.take() {
            if runtime().policy.peek(binding, a[1]).is_ok() {
                let _ = runtime().policy.consume(binding, a[1]);
            }
            if code == -16 {
                runtime().unknown_seen = true;
            }
            return Err(code);
        }
        let request = runtime().policy.peek(binding, a[1]).map_err(err)?;
        if request.completion == Some(Completion::Unknown) {
            runtime().policy.consume(binding, a[1]).map_err(err)?;
            runtime().unknown_seen = true;
            return Err(-16);
        }
        if request.completion != Some(Completion::Success) {
            return Err(-12);
        }
        let mut m = Message::new();
        m.tag = a[1];
        if matches!(request.operation, Operation::Read { .. }) {
            m.len = u32::from(request.length);
            let start = usize::from(request.offset);
            let end = start + usize::from(request.length);
            m.data[..usize::from(request.length)].copy_from_slice(&runtime().data[start..end]);
        }
        let bytes = m.encode();
        m4::write_plan(&output, &bytes);
        assert_eq!(m4::read_plan(&output), bytes);
        runtime().policy.consume(binding, a[1]).map_err(err)?;
        return Ok(u64::from(m.len));
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
    let r = runtime();
    if r.offline != 0 {
        return Err(r.offline);
    }
    if r.resetting || r.error.is_some() {
        return Err(-8);
    }
    if r.policy.phase() != Phase::Ready {
        return Err(-8);
    }
    let capacity = r.device.as_ref().ok_or(-14)?.capacity;
    let input = if n == 0x42 {
        Some(input_bytes(&input_plan(s, slot, a[2])?))
    } else {
        None
    };
    let is_put = slot == 0 && runtime().active_put;
    if let Some(bytes) = input {
        runtime().snapshot_count += 1;
        if op == (Operation::Write { sector: 12 }) {
            assert_eq!(case(), 2);
            if runtime().scratch_writes == 0 {
                assert_eq!(bytes, [0xa7; 512]);
            } else {
                assert!(bytes[..255].iter().all(|b| *b == 0x3c));
                assert!(bytes[255..].iter().all(|b| *b == 0xa7));
            }
            runtime().scratch_snapshot = bytes;
            runtime().scratch_writes += 1;
        }
        if is_put {
            let h = runtime().put_header.unwrap();
            let current = runtime().latest.unwrap();
            let expected =
                journal::prepare(&current, h.generation, h.request_id, &runtime().put_value)
                    .unwrap();
            assert_eq!(bytes, expected);
            runtime().journal_snapshot = bytes;
            runtime().candidate =
                Some(journal::decode(&bytes, current.generation as usize).unwrap());
        }
    }
    let request = runtime()
        .policy
        .prepare(binding, op, capacity, s.ticks, TIMEOUT)
        .map_err(err)?;
    let bad = cfg!(feature = "test-m6a-bad-descriptor")
        && is_put
        && matches!(op, Operation::Write { .. })
        && !runtime().injected;
    let prepared = runtime()
        .device
        .as_mut()
        .unwrap()
        .prepare_operation(op, input.as_ref(), bad);
    if let Err(e) = prepared {
        runtime()
            .policy
            .reject_prepared(binding, request.ticket)
            .unwrap();
        runtime().injected |= bad;
        // A rejected draft has no exposed DMA. The service still treats this
        // failed PUT as uncertain; kernel does not claim data was written.
        return Err(err(e));
    }
    if is_put && matches!(op, Operation::Write { .. }) && cut(s, 1) {
        return Ok(request.ticket);
    }
    runtime()
        .policy
        .publish(request.epoch, request.ticket)
        .unwrap();
    let suppress = !runtime().injected
        && is_put
        && ((matches!(op, Operation::Write { .. }) && matches!(case(), 6 | 8))
            || (op == Operation::Flush && case() == 7));
    runtime()
        .device
        .as_mut()
        .unwrap()
        .publish_prepared(suppress)
        .unwrap();
    runtime().publications[operation_index(op)] += 1;
    if suppress {
        runtime().injected = true;
    }
    if is_put {
        if matches!(op, Operation::Write { .. }) {
            cut(s, 2);
        } else if op == Operation::Flush {
            cut(s, 4);
        }
    }
    Ok(request.ticket)
}
fn reset(now: u64, restart: bool) {
    let r = runtime();
    if r.quarantined {
        return;
    }
    r.policy.start_reset().unwrap();
    r.resetting = true;
    r.reset_deadline = now + RESET_TIMEOUT;
    r.need_restart |= restart;
    if let Some(d) = r.device.as_mut() {
        if d.begin_reset().is_err() {
            r.quarantined = true;
        }
    }
}
pub fn detach(pid: u64) {
    let r = runtime();
    if r.error.is_some_and(|(q, _)| q.binding.pid == pid) {
        r.error = None;
    }
    let request = r.policy.exit(pid);
    if r.quarantined {
        return;
    }
    if request.is_some() {
        reset(r.now, r.need_restart);
    }
}
fn advance(s: &State) {
    let now = s.ticks;
    let r = runtime();
    r.now = now;
    if r.gate {
        return;
    }
    if r.resetting {
        let ack = r.device.as_mut().is_some_and(Device::reset_acknowledged) && case() != 8;
        if ack {
            // SAFETY: Real status0 observed. Original UC mappings and exclusive
            // pinned pages are still live. Only this ack authorizes pool reuse.
            match unsafe {
                Device::initialize_profile(r.info.unwrap(), r.mapped, r.pool, true, case() == 9)
            } {
                Ok(d) => {
                    r.device = Some(d);
                    r.policy.reset_completed(true).unwrap();
                    r.resetting = false;
                    r.resets += 1;
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
    let q = r.policy.request().unwrap();
    if r.policy.timeout(now) {
        r.error = Some((q, if q.operation.is_mutating() { -16 } else { -13 }));
        reset(now, false);
        return;
    }
    let is_put = r.active_put;
    let injection = if !r.injected && is_put {
        match (case(), q.operation) {
            (4, Operation::Write { .. }) | (5, Operation::Flush) => 10,
            (12, Operation::Write { .. }) | (13, Operation::Flush) => 20,
            _ => 0,
        }
    } else {
        0
    };
    match r.device.as_mut().unwrap().poll(injection) {
        Ok(Some((data, Completion::Success))) => {
            r.data = data;
            r.completions[operation_index(q.operation)] += 1;
            r.policy
                .complete(q.epoch, q.ticket, Completion::Success)
                .unwrap();
            match q.operation {
                Operation::Read {
                    sector,
                    offset,
                    length,
                } if (4..8).contains(&sector) => {
                    let index = (sector - 4) as usize;
                    let start = offset as usize;
                    let end = start + length as usize;
                    r.records[index][start..end].copy_from_slice(&data[start..end]);
                    if length == 128 && offset % 128 == 0 {
                        r.read_mask |= 1 << (index * 4 + start / 128);
                    }
                }
                Operation::Read {
                    sector: 12,
                    offset,
                    length,
                } => {
                    assert_eq!(case(), 2);
                    let start = offset as usize;
                    let end = start + length as usize;
                    assert_eq!(&data[start..end], &r.scratch_snapshot[start..end]);
                    r.scratch_reads += 1;
                }
                Operation::Write { .. } if is_put => {
                    r.write_completed = true;
                    cut(s, 3);
                }
                Operation::Flush if is_put => {
                    assert!(r.write_completed);
                    r.flush_completed = true;
                    cut(s, 5);
                }
                _ => {}
            }
        }
        Ok(Some((_, _))) => {
            r.injected |= injection != 0;
            r.policy
                .complete(q.epoch, q.ticket, Completion::Io)
                .unwrap();
            r.error = Some((q, if q.operation.is_mutating() { -16 } else { -12 }));
            reset(now, false);
        }
        Ok(None) => {}
        Err(e) => {
            r.injected |= injection != 0;
            r.error = Some((
                q,
                if q.operation.is_mutating() {
                    -16
                } else {
                    err(e)
                },
            ));
            reset(now, false);
        }
    }
}
fn target_cut() -> u8 {
    if (18..=23).contains(&case()) {
        case() - 17
    } else if (24..=29).contains(&case()) {
        case() - 23
    } else {
        0
    }
}
fn cut(s: &State, n: u8) -> bool {
    let r = runtime();
    if target_cut() != n || r.cut_fired {
        return false;
    }
    r.cut_fired = true;
    r.cut_observed = n;
    if (24..=29).contains(&case()) {
        number(b"M6A:CUT:", u64::from(n));
        // Deliberate host-kill rendezvous, no PASS and no fabricated completion.
        // IF0 prevents scheduling; the harnas must terminate the live QEMU.
        loop {
            unsafe {
                asm!("cli; hlt", options(nomem, nostack));
            }
        }
    }
    r.gate = true;
    put(s, 0, 184, 1);
    true
}
pub fn observe_exit(s: &mut State, slot: usize) {
    diagnostic(s, slot);
    s.unexpected = true;
    s.observer_survivor(slot);
}
pub fn observe_fault(s: &mut State, slot: usize, c: &[u64; 22]) {
    let expected =
        slot == 0 && (18..=23).contains(&case()) && runtime().gate && c[15] == 6 && c[16] == 0;
    if expected {
        let r = runtime();
        r.gate = false;
        r.crashes += 1;
        r.need_restart = true;
        reset(s.ticks, true);
    } else {
        diagnostic(s, slot);
        s.unexpected = true;
    }
    s.observer_survivor(slot);
}
fn diagnostic(s: &State, slot: usize) {
    number(b"M6A:USER_SLOT:", slot as u64);
    let code = word(s, slot, 240);
    if code <= 255 {
        number(b"M6A:USER_CODE:", code);
    }
    number(b"M6A:USER_SYSCALL:", runtime().last_number[slot]);
    let e = runtime().last_result[slot];
    if (-18..=0).contains(&e) {
        number(b"M6A:USER_ERRNO:", (-e) as u64);
    }
}
pub fn cap_event(slot: usize, n: u64) {
    if slot == 0 && n == 0x10 {
        runtime().close_count += 1;
    }
}

pub fn sent(s: &State, slot: usize, _handle: u64, m: &Message) {
    let r = runtime();
    if slot == 0 && m.tag == wire::ANNOUNCE {
        assert_eq!(r.read_mask, 0xffff);
        assert!(r.completions[2] >= 1);
        let recovered = journal::recover(&r.records).unwrap();
        if r.baseline.is_none() {
            r.baseline = Some(recovered);
        }
        if r.restarts != 0 {
            let old = r.baseline.unwrap();
            let next = r.candidate.unwrap();
            assert!(recovered == old || recovered == next);
            if target_cut() >= 5 {
                assert_eq!(recovered, next);
            }
            if target_cut() == 1 {
                assert_eq!(recovered, old);
            }
        }
        r.latest = Some(recovered);
        r.announcements += 1;
        assert_eq!(m.cap_count, 2);
        assert!(m.caps.iter().all(|c| c.rights == 9));
        return;
    }
    if slot == 2 && m.tag == wire::REBIND {
        assert_eq!(m.cap_count, 2);
        assert!(m.caps.iter().all(|c| c.rights == 1));
        r.forwards += 1;
        return;
    }
    if m.tag != wire::TAG {
        return;
    }
    let mut body = [0; wire::SIZE];
    body.copy_from_slice(&m.data[..wire::SIZE]);
    let h = wire::header(&body);
    if slot == 1 {
        let badcode = if case() == 1 && (100..=114).contains(&h.request_id) {
            Some(match h.request_id {
                101 => -2,
                105..=108 => -3,
                109 => -10,
                114 => -5,
                _ => -1,
            })
        } else if case() == 16 && h.request_id == 99 {
            Some(-10)
        } else if case() == 3
            && h.operation == wire::PUT
            && r.latest.is_some_and(|x| x.generation == 4)
        {
            Some(-17)
        } else {
            None
        };
        if let Some(expected) = badcode {
            assert!(r.bad.is_none());
            let stats = m4::engine().stats();
            r.bad = Some(Bad {
                header: h,
                expected,
                calls: r.calls,
                publications: r.publications,
                completions: r.completions,
                nodes: stats.nodes,
                handles: stats.handles,
                closes: r.close_count,
                received: false,
            });
            r.bad_requests += 1;
            return;
        }
        if h.operation == wire::PUT {
            assert!(!r.active_put && !r.unknown_seen);
            assert_eq!(m.len, 192);
            assert_eq!(m.cap_count, 0);
            wire::validate_request(&body, r.epoch, true).unwrap();
            let old = r.latest.unwrap();
            assert_eq!(h.generation, old.generation);
            r.put_value.copy_from_slice(&body[48..176]);
            // Independent fixture transformation from actual recovered bytes.
            for (a, b) in r.put_value.iter().zip(old.value) {
                assert_eq!(*a, b.wrapping_add(0x31 + (old.generation + 1) as u8));
            }
            r.active_put = true;
            r.put_header = Some(h);
            r.write_completed = false;
            r.flush_completed = false;
        }
        return;
    }
    if slot != 0 {
        return;
    }
    assert_eq!(m.len, 192);
    assert_eq!(m.cap_count, 0);
    assert!(m.data[192..].iter().all(|b| *b == 0));
    if let Some(bad) = r.bad.take() {
        assert!(bad.received);
        assert_eq!(
            body,
            wire::reply(
                bad.header,
                r.epoch,
                r.latest.unwrap().generation,
                wire::Status::from_code(bad.expected),
                None
            )
        );
        assert_eq!(
            (r.calls, r.publications, r.completions),
            (bad.calls, bad.publications, bad.completions)
        );
        if bad.header.request_id == 113 {
            let stats = m4::engine().stats();
            assert_eq!(r.close_count, bad.closes + 2);
            assert_eq!(stats.handles, bad.handles);
            assert_eq!(stats.nodes + 2, bad.nodes);
        }
        r.bad_replies += 1;
        return;
    }
    if h.operation == wire::PUT {
        let request = r.put_header.unwrap();
        wire::validate_reply(&body, request, r.epoch).unwrap();
        if h.status == 0 {
            assert!(r.active_put && r.write_completed && r.flush_completed);
            let next = r.candidate.unwrap();
            assert_eq!(h.generation, next.generation);
            assert_eq!(
                body,
                wire::reply(request, r.epoch, next.generation, wire::Status::Ok, None)
            );
            r.latest = Some(next);
            r.ack_sent += 1;
        } else {
            assert_eq!(h.status, -16);
            assert!(r.injected && !r.flush_completed);
            r.unknown_seen = true;
        }
        r.active_put = false;
    } else if h.operation == wire::GET && h.status == 0 {
        let value = r.latest.unwrap();
        assert_eq!(h.generation, value.generation);
        assert_eq!(h.length, 128);
        assert_eq!(&body[48..176], &value.value);
        r.get_replies += 1;
    }
    let _ = s;
}
pub fn received(s: &State, slot: usize, handle: u64, m: &Message) {
    if m.tag != wire::TAG {
        return;
    }
    let mut body = [0; wire::SIZE];
    body.copy_from_slice(&m.data[..wire::SIZE]);
    let h = wire::header(&body);
    let r = runtime();
    if slot == 0 {
        if let Some(bad) = r.bad.as_mut() {
            assert_eq!(h, bad.header);
            if h.request_id == 114 {
                assert_eq!(handle, r.roots[0]);
            }
            bad.received = true;
        } else if h.operation == wire::PUT {
            assert_eq!(handle, r.roots[1]);
        }
    }
    if slot == 1 && h.operation == wire::PUT && h.status == 0 {
        assert_eq!(m.sender_pid, s.processes[0].pid);
        assert_eq!(r.ack_sent, r.ack_received + 1);
        assert!(r.flush_completed);
        r.ack_received += 1;
        cut(s, 6);
    }
}
pub fn poll(s: &mut State) -> Option<*const u64> {
    advance(s);
    if runtime().need_restart
        && !runtime().resetting
        && !runtime().quarantined
        && s.processes[0].status == Status::Dead
    {
        let r = runtime();
        r.restarts += 1;
        r.epoch += 1;
        r.need_restart = false;
        r.active_put = false;
        r.put_header = None;
        r.read_mask = 0;
        r.records = [[0; 512]; 4];
        assert_eq!(
            (
                s.processes[1].pid,
                s.processes[1].paging.as_ref().unwrap().root
            ),
            r.initial_client
        );
        assert_eq!(
            (
                s.processes[2].pid,
                s.processes[2].paging.as_ref().unwrap().root
            ),
            r.initial_super
        );
        s.spawn_process(0);
        assert!(s.processes[0].pid > r.initial_service);
        bind_service(s);
    }
    if s.unexpected {
        serial::fail(b"M6A:FAIL:USER_OR_PROTOCOL\n");
    }
    for slot in 0..s.amount {
        if s.processes[slot].status != Status::Dead {
            if word(s, slot, 40) >= 0x100 {
                diagnostic(s, slot);
                serial::fail(b"M6A:FAIL:USER_STATUS\n");
            }
            assert_eq!(word(s, slot, 16), s.processes[slot].pattern);
        }
    }
    let service_error = matches!(case(), 9 | 10 | 15 | 17);
    let done = if service_error {
        word(s, 0, 40) == 1
    } else {
        word(s, 1, 40) == 1
    };
    if !done {
        return None;
    }
    let r = runtime();
    if !r.finish {
        r.finish = true;
        r.finish_counter = word(s, 1, 8);
        r.finish_super = word(s, 2, 8);
        return None;
    }
    if word(s, 1, 8) <= r.finish_counter + 8 || word(s, 2, 8) <= r.finish_super + 8 || r.resetting {
        return None;
    }
    assert_eq!(
        (
            s.processes[1].pid,
            s.processes[1].paging.as_ref().unwrap().root
        ),
        r.initial_client
    );
    assert_eq!(
        (
            s.processes[2].pid,
            s.processes[2].paging.as_ref().unwrap().root
        ),
        r.initial_super
    );
    if service_error {
        assert_eq!(r.publications[1], 0);
        assert_eq!(r.ack_sent, 0);
        assert_eq!(r.announcements, 0);
        if case() == 15 {
            assert_eq!(journal::recover(&r.records), Err(journal::Error::Corrupt));
            assert_eq!(r.read_mask, 0xffff);
            assert_eq!(word(s, 0, 232) as i64, -18);
        } else {
            assert!(r.offline != 0);
        }
    } else if matches!(case(),4..=8|11..=13) {
        assert!(r.injected && r.unknown_seen);
        serial::write(b"M6A:OUTCOME:UNKNOWN\n");
        assert_eq!(r.ack_sent, 0);
        assert!(r.publications[1] <= 1);
        assert!(r.publications[2] <= 2);
        if case() == 8 {
            assert!(r.quarantined);
        }
        if matches!(case(), 6 | 7) {
            assert!(s.ticks >= TIMEOUT);
        }
    } else {
        let value = r.latest.unwrap();
        assert!(r.get_replies >= 1);
        assert_eq!(word(s, 1, 128), value.generation);
        assert_eq!(
            word(s, 1, 120),
            wire::checksum_update(wire::FNV_OFFSET, &value.value)
        );
        if case() == 3 {
            assert_eq!(value.generation, 4);
            assert_eq!(r.ack_received, 3);
        } else if matches!(case(), 14 | 30) {
            assert_eq!(r.publications[1], 0);
        } else if (18..=23).contains(&case()) {
            assert_eq!(r.crashes, 1);
            assert_eq!(r.restarts, 1);
            assert_eq!(r.cut_observed, target_cut());
            assert!(r.announcements >= 2 && r.forwards >= 2);
            assert_eq!(r.publications[1], u64::from(target_cut() != 1));
        } else {
            assert_eq!(value.generation, 2);
            assert_eq!(r.ack_received, 1);
        }
        if case() == 1 {
            assert_eq!(r.bad_requests, 15);
            assert_eq!(r.bad_replies, 15);
        }
        if case() == 16 {
            assert_eq!(r.bad_requests, 1);
            assert_eq!(r.bad_replies, 1);
        }
        if case() == 2 {
            assert_eq!(r.scratch_writes, 2);
            assert_eq!(r.scratch_reads, 8);
            let source = input_bytes(&input_plan(s, 0, 0x40013f01).unwrap());
            assert!(source[..255].iter().all(|b| *b == 0xe1));
            assert!(r.errors[3] > 0);
            assert!(
                r.snapshot_count >= 3
                    && r.invalid_outputs >= 1
                    && r.errors[5] > 0
                    && r.errors[6] > 0
                    && r.errors[11] > 0
            );
        }
        number(b"M6A:GENERATION:", value.generation);
        number(b"M6A:REQUEST_ID:", value.request_id);
        hex(
            b"M6A:VALUE_DIGEST:",
            wire::checksum_update(wire::FNV_OFFSET, &value.value),
        );
    }
    assert!(m4::engine().check_invariants());
    serial::write(marker());
    serial::write(b"M6A:SURVIVOR_PROGRESS:PASS\n");
    if !r.quarantined {
        if let Some(d) = r.device.as_mut() {
            assert!(d.begin_reset().is_ok());
            assert!(d.reset_acknowledged());
        }
    }
    // All live operations/results have either been consumed or belong to the
    // stopped service; cleanup may clear metadata, never the pinned pool.
    // End this Runtime borrow before cleanup invokes detach, which takes its
    // own exclusive borrow. IF0 does not permit overlapping Rust &mut values.
    let pool = r.pool;
    for slot in 0..s.amount {
        if s.processes[slot].status != Status::Dead {
            s.processes[slot].status = Status::Dying;
            s.cleanup(slot);
        }
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
    serial::write(b"M6A:RESOURCES:PASS\nM6A:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
fn hex(label: &[u8], value: u64) {
    let mut bytes = [b'0'; 16];
    for (i, b) in bytes.iter_mut().enumerate() {
        let v = ((value >> ((15 - i) * 4)) & 15) as u8;
        *b = if v < 10 { b'0' + v } else { b'a' + v - 10 };
    }
    serial::write(label);
    serial::write(&bytes);
    serial::write(b"\n");
}
fn case() -> u8 {
    if cfg!(feature = "test-m6a-service-errors") {
        return 1;
    }
    if cfg!(feature = "test-m6a-device-rights") {
        return 2;
    }
    if cfg!(feature = "test-m6a-full") {
        return 3;
    }
    if cfg!(feature = "test-m6a-write-error") {
        return 4;
    }
    if cfg!(feature = "test-m6a-flush-error") {
        return 5;
    }
    if cfg!(feature = "test-m6a-write-timeout") {
        return 6;
    }
    if cfg!(feature = "test-m6a-flush-timeout") {
        return 7;
    }
    if cfg!(feature = "test-m6a-reset-timeout") {
        return 8;
    }
    if cfg!(feature = "test-m6a-no-flush") {
        return 9;
    }
    if cfg!(feature = "test-m6a-readonly") {
        return 10;
    }
    if cfg!(feature = "test-m6a-bad-descriptor") {
        return 11;
    }
    if cfg!(feature = "test-m6a-used-write") {
        return 12;
    }
    if cfg!(feature = "test-m6a-used-flush") {
        return 13;
    }
    if cfg!(feature = "test-m6a-torn-tail") {
        return 14;
    }
    if cfg!(feature = "test-m6a-corrupt-all") {
        return 15;
    }
    if cfg!(feature = "test-m6a-stale-generation") {
        return 16;
    }
    if cfg!(feature = "test-m6a-no-device") {
        return 17;
    }
    if cfg!(feature = "test-m6a-service-cut-1") {
        return 18;
    }
    if cfg!(feature = "test-m6a-service-cut-2") {
        return 19;
    }
    if cfg!(feature = "test-m6a-service-cut-3") {
        return 20;
    }
    if cfg!(feature = "test-m6a-service-cut-4") {
        return 21;
    }
    if cfg!(feature = "test-m6a-service-cut-5") {
        return 22;
    }
    if cfg!(feature = "test-m6a-service-cut-6") {
        return 23;
    }
    if cfg!(feature = "test-m6a-vm-cut-1") {
        return 24;
    }
    if cfg!(feature = "test-m6a-vm-cut-2") {
        return 25;
    }
    if cfg!(feature = "test-m6a-vm-cut-3") {
        return 26;
    }
    if cfg!(feature = "test-m6a-vm-cut-4") {
        return 27;
    }
    if cfg!(feature = "test-m6a-vm-cut-5") {
        return 28;
    }
    if cfg!(feature = "test-m6a-vm-cut-6") {
        return 29;
    }
    if cfg!(feature = "test-m6a-recover") {
        return 30;
    }
    0
}
fn marker() -> &'static [u8] {
    match case() {
        0 => b"M6A:CASE:NORMAL:PASS\n",
        1 => b"M6A:CASE:SERVICE_ERRORS:PASS\n",
        2 => b"M6A:CASE:DEVICE_RIGHTS:PASS\n",
        3 => b"M6A:CASE:FULL:PASS\n",
        4 => b"M6A:CASE:WRITE_ERROR:PASS\n",
        5 => b"M6A:CASE:FLUSH_ERROR:PASS\n",
        6 => b"M6A:CASE:WRITE_TIMEOUT:PASS\n",
        7 => b"M6A:CASE:FLUSH_TIMEOUT:PASS\n",
        8 => b"M6A:CASE:RESET_TIMEOUT:PASS\n",
        9 => b"M6A:CASE:NO_FLUSH:PASS\n",
        10 => b"M6A:CASE:READONLY:PASS\n",
        11 => b"M6A:CASE:BAD_DESCRIPTOR:PASS\n",
        12 => b"M6A:CASE:USED_WRITE:PASS\n",
        13 => b"M6A:CASE:USED_FLUSH:PASS\n",
        14 => b"M6A:CASE:TORN_TAIL:PASS\n",
        15 => b"M6A:CASE:CORRUPT_ALL:PASS\n",
        16 => b"M6A:CASE:STALE_GENERATION:PASS\n",
        17 => b"M6A:CASE:NO_DEVICE:PASS\n",
        18 => b"M6A:CASE:SERVICE_CUT_1:PASS\n",
        19 => b"M6A:CASE:SERVICE_CUT_2:PASS\n",
        20 => b"M6A:CASE:SERVICE_CUT_3:PASS\n",
        21 => b"M6A:CASE:SERVICE_CUT_4:PASS\n",
        22 => b"M6A:CASE:SERVICE_CUT_5:PASS\n",
        23 => b"M6A:CASE:SERVICE_CUT_6:PASS\n",
        24 => b"M6A:CASE:VM_CUT_1:PASS\n",
        25 => b"M6A:CASE:VM_CUT_2:PASS\n",
        26 => b"M6A:CASE:VM_CUT_3:PASS\n",
        27 => b"M6A:CASE:VM_CUT_4:PASS\n",
        28 => b"M6A:CASE:VM_CUT_5:PASS\n",
        29 => b"M6A:CASE:VM_CUT_6:PASS\n",
        30 => b"M6A:CASE:RECOVER:PASS\n",
        _ => unreachable!(),
    }
}

fn number(label: &[u8], mut value: u64) {
    let mut bytes = [b'0'; 20];
    let mut at = 19;
    loop {
        bytes[at] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
        at -= 1;
    }
    serial::write(label);
    serial::write(&bytes[at..]);
    serial::write(b"\n");
}
