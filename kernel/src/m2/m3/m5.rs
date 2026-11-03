//! Trusted, bounded block broker; user processes see typed READ + IPC only.
//! No user frame/address is ever a DMA target or retained across device I/O.
use super::*;
use block_protocol::wire;
use capability_core::{BlockScope, Message, BLOCK_READ};
use virtio_blk_driver::{Device, Discovery, DmaPool};
use virtio_core::{Binding, Completion, Phase, Policy, Read, Request};
const READ_TIMEOUT: u64 = 20;
const RESET_TIMEOUT: u64 = 10;
struct Runtime {
    info: Option<Discovery>,
    mapped: [u64; 6],
    device: Option<Device>,
    pool: DmaPool,
    policy: Policy,
    data: [u8; 512],
    error: Option<(Request, i64)>,
    resetting: bool,
    reset_deadline: u64,
    offline: i64,
    injected: bool,
    delayed: bool,
    kick_at: u64,
    service_epoch: u64,
    restarts: u64,
    need_restart: bool,
    submits: u64,
    completions: u64,
    errors: [u64; 16],
    resets: u64,
    quarantined: bool,
    service_roots: [u64; 2],
    client_roots: [u64; 2],
    super_root: u64,
    initial_client: (u64, u64),
    initial_super: (u64, u64),
    announcement: u64,
    forward: u64,
    correlations: u64,
    last_poll: bool,
    last_sector: u64,
    last_offset: u64,
    last_len: usize,
    last_bytes: [u8; 128],
    digest: [u64; 2],
    slices: [u64; 2],
    attacker_fault: bool,
    pending_crashes: u64,
    finishing: bool,
    finish_counter: u64,
    final_error_seen: bool,
    now: u64,
    service_calls: u64,
    service_closes: u64,
    bad_requests: u16,
    bad_replies: u16,
    bad: Option<BadRequest>,
    initial_service: (u64, u64),
    attacker_service_counter: u64,
    attacker_client_counter: u64,
    last_number: [u64; MAX],
    last_result: [i64; MAX],
}
#[derive(Clone, Copy)]
struct BadRequest {
    id: u64,
    header: wire::Header,
    status: i64,
    calls: u64,
    submits: u64,
    completions: u64,
    handles: usize,
    nodes: usize,
    closes: u64,
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
    resetting: false,
    reset_deadline: 0,
    offline: 0,
    injected: false,
    delayed: false,
    kick_at: 0,
    service_epoch: 1,
    restarts: 0,
    need_restart: false,
    submits: 0,
    completions: 0,
    errors: [0; 16],
    resets: 0,
    quarantined: false,
    service_roots: [0; 2],
    client_roots: [0; 2],
    super_root: 0,
    initial_client: (0, 0),
    initial_super: (0, 0),
    announcement: 0,
    forward: 0,
    correlations: 0,
    last_poll: false,
    last_sector: 0,
    last_offset: 0,
    last_len: 0,
    last_bytes: [0; 128],
    digest: [wire::FNV_OFFSET; 2],
    slices: [0; 2],
    attacker_fault: false,
    pending_crashes: 0,
    finishing: false,
    finish_counter: 0,
    final_error_seen: false,
    now: 0,
    service_calls: 0,
    service_closes: 0,
    bad_requests: 0,
    bad_replies: 0,
    bad: None,
    initial_service: (0, 0),
    attacker_service_counter: 0,
    attacker_client_counter: 0,
    last_number: [0; MAX],
    last_result: [0; MAX],
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: Single CPU/IF0 service-stack execution. No device IRQ borrows this
    // state, and no Runtime borrow is retained across cleanup/restart callbacks.
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
    let device = unsafe { Device::initialize(info, mapped, pool) };
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
    runtime().initial_service = (
        s.processes[0].pid,
        s.processes[0].paging.as_ref().unwrap().root,
    );
    let engine = m4::engine();
    for slot in 0..s.amount {
        engine.register(s.processes[slot].pid).unwrap();
    }
    let client = s.processes[1].pid;
    let supervisor = s.processes[2].pid;
    let reply = engine.create_endpoint(client).unwrap();
    let control = engine.create_endpoint(client).unwrap();
    let announcements = engine.create_endpoint(supervisor).unwrap();
    runtime().client_roots = [reply, control];
    runtime().super_root = announcements;
    runtime().initial_client = (client, s.processes[1].paging.as_ref().unwrap().root);
    runtime().initial_super = (supervisor, s.processes[2].paging.as_ref().unwrap().root);
    for slot in 1..s.amount {
        for (offset, value) in [
            (0, slot as u64),
            (32, 1),
            (40, 0),
            (96, 4),
            (104, 6),
            (112, m5_case() as u64),
            (160, client),
            (168, supervisor),
            (176, s.processes[0].pid),
            (200, 4),
        ] {
            put(s, slot, offset, value);
        }
    }
    match m5_case() {
        14 => put(s, 3, 208, 0x3f000000 + 4 * 65536),
        15 => put(s, 3, 216, runtime().pool.bounce),
        _ => {}
    }
    put(s, 1, 56, reply);
    put(s, 1, 80, control);
    put(s, 2, 56, announcements);
    let forward = engine
        .bootstrap_delegate(client, control, supervisor, 1)
        .unwrap();
    put(s, 2, 64, forward);
    put(s, 1, 64, 0);
    assert_eq!(word(s, 1, 64), 0);
    bind_service(s);
}
fn bind_service(s: &mut State) {
    let engine = m4::engine();
    let pid = s.processes[0].pid;
    if runtime().restarts != 0 {
        engine.register(pid).unwrap();
    }
    let roots = [
        engine.create_endpoint(pid).unwrap(),
        engine.create_endpoint(pid).unwrap(),
    ];
    runtime().service_roots = roots;
    let admin = engine
        .create_block_device(
            pid,
            BlockScope {
                device: 0,
                first_sector: 4,
                sector_count: 4,
            },
        )
        .unwrap();
    let read = engine.derive(pid, admin, BLOCK_READ).unwrap();
    if m5_case() == 2 && runtime().restarts == 0 {
        let zero = engine
            .bootstrap_delegate(pid, admin, s.processes[1].pid, 0)
            .unwrap();
        put(s, 1, 232, zero);
        put(s, 1, 224, read);
    }
    engine.close(pid, admin).unwrap();
    assert!(engine.check_block(pid, read, BLOCK_READ).is_ok());
    assert!(engine.derive(pid, read, 1).is_err());
    let reply = engine
        .bootstrap_delegate(s.processes[1].pid, runtime().client_roots[0], pid, 1)
        .unwrap();
    let notify = engine
        .bootstrap_delegate(s.processes[2].pid, runtime().super_root, pid, 1)
        .unwrap();
    for (offset, value) in [
        (0, 0),
        (32, runtime().service_epoch),
        (40, 0),
        (48, read),
        (56, roots[0]),
        (64, notify),
        (72, notify),
        (80, roots[1]),
        (88, reply),
        (96, 4),
        (104, 6),
        (112, m5_case() as u64),
        (160, s.processes[1].pid),
        (168, s.processes[2].pid),
        (176, pid),
        (
            184,
            if m5_case() == 13 && runtime().restarts < 3 {
                2
            } else {
                0
            },
        ),
        (200, 4),
    ] {
        put(s, 0, offset, value);
    }
}
fn err(error: virtio_core::Error) -> i64 {
    use virtio_core::Error::*;
    match error {
        Pending | Busy => -8,
        Range | Invalid | WrongTicket | StaleEpoch => -1,
        Protocol => -15,
        Io | Unsupported => -12,
        Timeout => -13,
        Offline | Exhausted => -14,
    }
}
pub fn syscall(s: &mut State, slot: usize, number: u64, a: [u64; 6]) -> (i64, u64) {
    if slot == 0 {
        runtime().service_calls += 1;
    }
    let result = device_call(s, slot, number, a);
    runtime().last_number[slot] = number;
    runtime().last_result[slot] = match result {
        Ok(_) => 0,
        Err(code) => code,
    };
    match result {
        Ok(value) => (0, value),
        Err(code) => {
            if code < 0 && code >= -15 {
                runtime().errors[(-code) as usize] += 1;
            }
            (code, 0)
        }
    }
}
pub fn cap_event(slot: usize, number: u64) {
    if slot == 0 && number == 0x10 {
        runtime().service_closes += 1;
    }
}
fn device_call(s: &mut State, slot: usize, number: u64, a: [u64; 6]) -> Result<u64, i64> {
    if a[4] != 0 || a[5] != 0 {
        return Err(-1);
    }
    if number == 0x41 && a[3] != 320 {
        return Err(-11);
    }
    let pid = s.processes[slot].pid;
    let auth = m4::engine()
        .check_block(pid, a[0], BLOCK_READ)
        .map_err(|e| e.code())?;
    if auth.scope().device != 0 {
        return Err(-5);
    }
    let binding = Binding {
        pid,
        object_key: auth.object_key(),
    };
    if number == 0x40 {
        if a[3] == 0 || a[3] > 128 {
            return Err(-11);
        }
        if a[2].checked_add(a[3]).filter(|end| *end <= 512).is_none() {
            return Err(-1);
        }
        // The core integer helper reports Range/Invalid; at the authority ABI a
        // well-formed sector outside this particular grant is EACCESS.
        if a[1] >= auth.scope().sector_count {
            return Err(-5);
        }
        let sector = auth.sector(a[1]).map_err(|e| e.code())?;
        let r = runtime();
        if r.offline != 0 {
            return Err(r.offline);
        }
        if r.resetting || r.error.is_some() {
            return Err(-8);
        }
        let capacity = r.device.as_ref().ok_or(-14)?.capacity;
        let request = r
            .policy
            .submit(
                binding,
                Read {
                    sector,
                    offset: a[2],
                    length: a[3],
                },
                capacity,
                s.ticks,
                READ_TIMEOUT,
            )
            .map_err(err)?;
        let first = !r.injected;
        let suppress = first && matches!(m5_case(), 3 | 11 | 12);
        let bad = first && m5_case() == 4;
        let submit = r.device.as_mut().unwrap().submit(sector, suppress, bad);
        if first && matches!(m5_case(), 3 | 4 | 11 | 12) {
            r.injected = true;
        }
        if let Err(error) = submit {
            let code = err(error);
            begin_reset(s.ticks, false);
            return Err(code);
        }
        r.submits += 1;
        if suppress && m5_case() == 3 {
            r.delayed = true;
            r.kick_at = s.ticks + 3;
        }
        Ok(request.ticket)
    } else {
        let r = runtime();
        let request = if let Some((request, _)) = r.error {
            request
        } else {
            r.policy.request().ok_or(-1)?
        };
        if request.binding != binding || request.ticket != a[1] {
            return Err(-1);
        }
        let output = m4::plan(s, slot, a[2], 320, true).map_err(|e| e.code())?;
        if let Some((_, code)) = r.error.take() {
            r.final_error_seen = true;
            return Err(code);
        }
        let request = r.policy.peek(binding, a[1]).map_err(err)?;
        let mut message = Message::new();
        message.tag = a[1];
        message.len = u32::from(request.length);
        let start = request.offset as usize;
        let length = request.length as usize;
        message.data[..length].copy_from_slice(&r.data[start..start + length]);
        let bytes = message.encode();
        m4::write_plan(&output, &bytes);
        assert_eq!(m4::read_plan(&output), bytes);
        r.last_poll = true;
        r.last_sector = request.sector;
        r.last_offset = u64::from(request.offset);
        r.last_len = length;
        r.last_bytes[..length].copy_from_slice(&message.data[..length]);
        assert_eq!(
            r.policy.consume(binding, a[1]).unwrap(),
            Completion::Success
        );
        Ok(length as u64)
    }
}
fn begin_reset(now: u64, restart: bool) {
    let r = runtime();
    r.policy.start_reset().unwrap();
    r.resetting = true;
    r.reset_deadline = now + RESET_TIMEOUT;
    r.need_restart |= restart;
    if let Some(device) = r.device.as_mut() {
        if device.begin_reset().is_err() {
            r.quarantined = true;
        }
    }
}
pub fn detach(pid: u64) {
    let r = runtime();
    // A reset may already have removed Policy.pending while an unconsumed
    // terminal result still names the dying PID. It must not block future reads.
    if r.error
        .is_some_and(|(request, _)| request.binding.pid == pid)
    {
        r.error = None;
    }
    if r.quarantined {
        return;
    }
    if r.policy
        .request()
        .is_some_and(|request| request.binding.pid == pid)
    {
        if m5_case() == 13 {
            r.pending_crashes += 1;
        }
        r.error = None;
        begin_reset(r.now, true);
    }
}
pub fn observe_exit(s: &mut State, slot: usize) {
    diagnostic(s, slot);
    s.unexpected = true;
    s.observer_survivor(slot);
}
fn diagnostic(s: &State, slot: usize) {
    let code = word(s, slot, 240);
    if code <= 255 {
        print_number(b"M5:USER_DIAG_SLOT:", slot as u64);
        print_number(b"M5:USER_DIAG_CODE:", code);
        print_number(b"M5:USER_DIAG_SYSCALL:", runtime().last_number[slot]);
        let result = runtime().last_result[slot];
        if (-15..=0).contains(&result) {
            print_number(b"M5:USER_DIAG_ERRNO:", (-result) as u64);
        }
    }
}
pub fn observe_fault(s: &mut State, slot: usize, c: &[u64; 22]) {
    let expected = if m5_case() == 13 {
        slot == 0
            && c[15] == 6
            && c[16] == 0
            && runtime().policy.request().is_some()
            && runtime().restarts < 3
    } else if (14..=16).contains(&m5_case()) {
        slot == 3
            && match m5_case() {
                14 | 15 => {
                    c[15] == 14
                        && c[16] == 5
                        && s.last_event.cr2 == word(s, 3, if m5_case() == 14 { 208 } else { 216 })
                }
                16 => c[15] == 13 && c[16] == 0,
                _ => false,
            }
    } else {
        false
    };
    s.unexpected |= !expected;
    if expected && slot == 3 {
        runtime().attacker_fault = true;
        runtime().attacker_service_counter = word(s, 0, 8);
        runtime().attacker_client_counter = word(s, 1, 8);
    }
    s.observer_survivor(slot);
}
pub fn sent(s: &mut State, slot: usize, message: &Message) {
    let r = runtime();
    if m5_case() == 1 && slot == 1 && message.tag == wire::TAG {
        let id = wire::get64(&message.data, 8);
        if (1..=13).contains(&id) {
            assert!(r.bad.is_none());
            let mut body = wire::request(id, 1, 0, 128, wire::READ);
            let status = match id {
                1 => {
                    body[0] = 2;
                    -1
                }
                2 => {
                    body[2] = 2;
                    -2
                }
                3 => {
                    wire::put32(&mut body, 4, 1);
                    -1
                }
                4 => {
                    wire::put64(&mut body, 40, 1);
                    -1
                }
                5 => {
                    wire::put32(&mut body, 32, 0);
                    -3
                }
                6 => {
                    wire::put32(&mut body, 32, 129);
                    -3
                }
                7 => {
                    wire::put64(&mut body, 24, u64::MAX);
                    -3
                }
                8 => {
                    wire::put64(&mut body, 24, 511);
                    wire::put32(&mut body, 32, 2);
                    -3
                }
                9 => {
                    wire::put64(&mut body, 16, 0);
                    -10
                }
                10 => {
                    body[48] = 1;
                    -1
                }
                11 | 12 | 13 => -1,
                _ => unreachable!(),
            };
            let mut expected = [0u8; 256];
            expected[..192].copy_from_slice(&body);
            assert_eq!(message.data, expected);
            assert_eq!(
                message.len,
                if id == 11 {
                    191
                } else if id == 12 {
                    193
                } else {
                    192
                }
            );
            assert_eq!(message.cap_count, if id == 13 { 2 } else { 0 });
            if id == 13 {
                assert!(message
                    .caps
                    .iter()
                    .all(|cap| cap.handle != 0 && cap.rights == 1));
            }
            let stats = m4::engine().stats();
            r.bad = Some(BadRequest {
                id,
                header: wire::header(&body),
                status,
                calls: r.service_calls,
                submits: r.submits,
                completions: r.completions,
                handles: stats.handles,
                nodes: stats.nodes,
                closes: r.service_closes,
            });
            r.bad_requests |= 1 << (id - 1);
        }
    }
    if slot == 0 && message.tag == wire::TAG {
        if let Some(bad) = r.bad {
            assert_eq!(wire::get64(&message.data, 8), bad.id);
            assert_eq!(message.len, 192);
            assert_eq!(message.cap_count, 0);
            let expected = wire::reply(
                bad.header,
                r.service_epoch,
                wire::Status::from_code(bad.status),
                &[],
            )
            .unwrap();
            assert_eq!(&message.data[..192], &expected);
            assert!(message.data[192..].iter().all(|byte| *byte == 0));
            assert_eq!(
                (r.service_calls, r.submits, r.completions),
                (bad.calls, bad.submits, bad.completions)
            );
            if bad.id == 13 {
                let stats = m4::engine().stats();
                assert_eq!(stats.handles, bad.handles);
                assert_eq!(stats.nodes + 2, bad.nodes);
            }
            assert_eq!(
                r.service_closes - bad.closes,
                if bad.id == 13 { 2 } else { 0 }
            );
            r.bad_replies |= 1 << (bad.id - 1);
            r.bad = None;
        }
    }
    if slot == 0 && message.tag == wire::ANNOUNCE {
        assert_eq!(message.cap_count, 2);
        assert!(message.caps.iter().all(|cap| cap.rights == 9));
        r.announcement += 1;
    }
    if slot == 2 && message.tag == wire::REBIND {
        assert_eq!(message.cap_count, 2);
        assert!(message.caps.iter().all(|cap| cap.rights == 1));
        r.forward += 1;
    }
    if slot == 0 && message.tag == wire::TAG && wire::get32(&message.data, 36) == 0 {
        let length = wire::get32(&message.data, 32) as usize;
        if message.len != wire::SIZE as u32
            || message.cap_count != 0
            || !r.last_poll
            || length != wire::SLICE
            || length != r.last_len
            || r.last_offset > 512 - wire::SLICE as u64
        {
            s.unexpected = true;
            return;
        }
        let sector = match r.last_sector {
            4 => 0,
            6 => 1,
            _ => {
                s.unexpected = true;
                return;
            }
        };
        if r.slices[sector] >= 4
            || r.last_offset != r.slices[sector] * wire::SLICE as u64
            || message.data[48..48 + length] != r.last_bytes[..length]
        {
            s.unexpected = true;
            return;
        }
        r.digest[sector] = wire::checksum_update(r.digest[sector], &message.data[48..48 + length]);
        r.slices[sector] += 1;
        r.correlations += 1;
        r.last_poll = false;
    }
}
fn advance(now: u64) {
    let r = runtime();
    r.now = now;
    if r.resetting {
        let acknowledged =
            r.device.as_ref().is_some_and(Device::reset_acknowledged) && m5_case() != 12;
        if acknowledged {
            // SAFETY: Real reset status0 observed before any pool byte is reused.
            // BAR mappings and pinned pages have not changed ownership/lifetime.
            let device = unsafe { Device::initialize(r.info.unwrap(), r.mapped, r.pool) };
            match device {
                Ok(device) => {
                    r.device = Some(device);
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
            r.policy.reset_completed(false).unwrap_err();
            r.resetting = false;
            r.offline = -13;
            r.quarantined = true;
        }
        return;
    }
    if r.offline != 0 || r.policy.phase() != Phase::Pending {
        return;
    }
    let request = r.policy.request().unwrap();
    if r.policy.timeout(now) {
        r.error = Some((request, -13));
        begin_reset(now, false);
        return;
    }
    if r.delayed && now >= r.kick_at {
        r.device.as_mut().unwrap().kick();
        r.delayed = false;
    }
    let injection = if !r.injected && (5..=10).contains(&m5_case()) {
        m5_case()
    } else {
        0
    };
    let completion = r.device.as_mut().unwrap().poll(injection);
    match completion {
        Ok(Some((data, Completion::Success))) => {
            r.data = data;
            r.completions += 1;
            r.policy
                .complete(request.epoch, request.ticket, Completion::Success)
                .unwrap();
        }
        Ok(Some((_, _))) => {
            r.injected |= injection != 0;
            r.error = Some((request, -12));
            begin_reset(now, false);
        }
        Ok(None) => {}
        Err(error) => {
            r.injected |= injection != 0;
            r.error = Some((request, err(error)));
            begin_reset(now, false);
        }
    }
}
pub fn poll(s: &mut State) -> Option<*const u64> {
    advance(s.ticks);
    if runtime().need_restart
        && !runtime().resetting
        && !runtime().quarantined
        && runtime().offline == 0
    {
        assert!(s.processes[0].status == Status::Dead);
        let r = runtime();
        r.need_restart = false;
        r.restarts += 1;
        r.service_epoch = r.service_epoch.checked_add(1).unwrap();
        assert!(r.restarts <= 3);
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
        bind_service(s);
    }
    for slot in 0..s.amount {
        if s.processes[slot].status != Status::Dead {
            assert_eq!(word(s, slot, 16), s.processes[slot].pattern);
            if word(s, slot, 40) >= 0x100 {
                diagnostic(s, slot);
                s.unexpected = true;
            }
        }
    }
    if s.unexpected {
        serial::write(b"M5:FAIL:USER_OR_PROTOCOL\n");
        serial::exit(0x11);
    }
    if word(s, 1, 40) != 1 {
        return None;
    }
    let r = runtime();
    if m5_case() == 12 && !r.quarantined {
        return None;
    }
    if (14..=16).contains(&m5_case()) && !r.attacker_fault {
        return None;
    }
    if m5_case() == 13 && r.restarts != 3 {
        return None;
    }
    if !r.finishing {
        r.finishing = true;
        r.finish_counter = word(s, 1, 8);
        return None;
    }
    if word(s, 1, 8) <= r.finish_counter + 8 {
        return None;
    }
    assert_eq!(
        (
            s.processes[1].pid,
            s.processes[1].paging.as_ref().unwrap().root
        ),
        r.initial_client
    );
    if !matches!(m5_case(), 12 | 17) {
        assert_eq!(r.slices, [4, 4]);
        assert_eq!(r.correlations, 8);
        assert!(r.submits >= 8 && r.completions >= 8);
        assert_eq!(word(s, 1, 120), r.digest[0]);
        assert_eq!(word(s, 1, 128), r.digest[1]);
        hex(b"M5:SECTOR4_DIGEST:", r.digest[0]);
        hex(b"M5:SECTOR6_DIGEST:", r.digest[1]);
    }
    match m5_case() {
        0 => {}
        1 => {
            assert_eq!(r.bad_requests, 0x1fff);
            assert_eq!(r.bad_replies, 0x1fff);
            assert!(r.bad.is_none());
        }
        2 => assert!(r.errors[3] > 0 && r.errors[5] > 0 && r.errors[6] > 0 && r.errors[11] > 0),
        3 => assert!(r.injected && r.errors[8] > 0 && s.ticks >= 3),
        4 => assert!(r.injected && r.errors[15] > 0),
        5..=9 => assert!(r.injected && r.errors[15] > 0 && r.resets >= 1),
        10 => assert!(r.injected && r.errors[12] > 0),
        11 => assert!(r.errors[13] > 0 && r.resets >= 1),
        12 => assert!(r.quarantined && r.errors[13] > 0),
        13 => assert!(
            r.restarts == 3
                && r.pending_crashes == 3
                && r.resets >= 3
                && r.announcement >= 4
                && r.forward >= 4
                && word(s, 1, 136) == 4
        ),
        14..=16 => {
            assert!(r.attacker_fault && r.restarts == 0);
            assert_eq!(
                (
                    s.processes[0].pid,
                    s.processes[0].paging.as_ref().unwrap().root
                ),
                r.initial_service
            );
            assert!(
                word(s, 0, 8) > r.attacker_service_counter
                    && word(s, 1, 8) > r.attacker_client_counter
            );
        }
        17 => assert!(r.device.is_none() && r.submits == 0 && r.errors[14] > 0),
        _ => unreachable!(),
    }
    assert!(r.announcement >= 1 && r.forward >= 1);
    serial::write(marker());
    serial::write(b"M5:SURVIVOR_PROGRESS:PASS\n");
    // End the finite test with actual reset acknowledgement unless this variant
    // explicitly proves quarantine. Pinned pages are never returned to Frames.
    if !r.quarantined {
        if let Some(device) = r.device.as_mut() {
            assert!(device.begin_reset().is_ok());
            assert!(device.reset_acknowledged());
        }
    }
    for slot in 0..s.amount {
        if s.processes[slot].status != Status::Dead {
            s.processes[slot].status = Status::Dying;
            s.cleanup(slot);
        }
    }
    let stats = m4::engine().stats();
    assert_eq!(stats.processes, 0);
    assert_eq!(stats.handles, 0);
    assert_eq!(stats.nodes, 0);
    assert_eq!(stats.endpoints, 0);
    assert_eq!(stats.block_objects, 0);
    assert_eq!(stats.queued_messages, 0);
    assert!(m4::engine().check_invariants());
    let pool = runtime().pool;
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
    serial::write(b"M5:RESOURCES:PASS\nM5:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
fn hex(label: &[u8], value: u64) {
    let mut bytes = [b'0'; 16];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let digit = ((value >> ((15 - i) * 4)) & 15) as u8;
        *byte = if digit < 10 {
            b'0' + digit
        } else {
            b'a' + digit - 10
        };
    }
    serial::write(label);
    serial::write(&bytes);
    serial::write(b"\n");
}
fn marker() -> &'static [u8] {
    match m5_case() {
        0 => b"M5:CASE:READ:PASS\n",
        1 => b"M5:CASE:SERVICE_ERRORS:PASS\n",
        2 => b"M5:CASE:DEVICE_RIGHTS:PASS\n",
        3 => b"M5:CASE:ASYNC:PASS\n",
        4 => b"M5:CASE:BAD_DESCRIPTOR:PASS\n",
        5 => b"M5:CASE:USED_ID:PASS\n",
        6 => b"M5:CASE:USED_INDEX:PASS\n",
        7 => b"M5:CASE:USED_SHORT:PASS\n",
        8 => b"M5:CASE:USED_LONG:PASS\n",
        9 => b"M5:CASE:USED_STATUS:PASS\n",
        10 => b"M5:CASE:IO_ERROR:PASS\n",
        11 => b"M5:CASE:TIMEOUT:PASS\n",
        12 => b"M5:CASE:RESET_TIMEOUT:PASS\n",
        13 => b"M5:CASE:RESTART:PASS\n",
        14 => b"M5:CASE:CPU_MMIO:PASS\n",
        15 => b"M5:CASE:CPU_DMA:PASS\n",
        16 => b"M5:CASE:CPU_PCI:PASS\n",
        17 => b"M5:CASE:NO_DEVICE:PASS\n",
        _ => unreachable!(),
    }
}
