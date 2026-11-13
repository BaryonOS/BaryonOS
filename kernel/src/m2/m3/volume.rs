//! M6e native volume profile (S04/S05). The kernel part is a generic scoped
//! block broker plus fixture barriers: it knows sectors, rights and tickets,
//! never the volume format, objects, names, revisions or domains. Volume
//! policy runs in the ring-3 service; acceptance of the stored bytes comes
//! from the app-reported evidence AND the independent host disk oracle.
//! Single CPU, IF0; owned snapshots, no retained userspace pointer or DMA alias.
use super::*;
use capability_core::{
    BlockScope, Engine, Message, BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE, RECV, SEND,
};
use virtio_blk_driver::{Device, Discovery, DmaPool};
use virtio_core::{Binding, Completion, Operation, Phase, Policy};

/// Completion budget in 100 Hz timer ticks; see the M6c broker for the
/// measured host fdatasync latency behind this value. This profile has no
/// native fault fixtures, so the budget costs no test time.
const TIMEOUT: u64 = 500;
const RESET_TIMEOUT: u64 = 10;
/// Granted scope: four reserved canary sectors precede the volume, which is
/// 2 x 34 table sectors plus 64 x 8 content sectors (volume-core v2 geometry).
const FIRST_SECTOR: u64 = 4;
const SECTORS: u64 = 580;
/// Only used by the crash fixtures to place a cut inside a table write.
const TABLE_REGION: u64 = 68;
/// Whole-run budget in 100 Hz timer ticks; the host also bounds wall time.
const RUN_TICKS: u64 = 60_000;
/// S09 archive fixtures only: a disjoint range of the same disposable test
/// image (sectors 1000..2040), granted to the slot-1 backup portal alone. It
/// ends below the 2048-sector image so the portal's past-the-end probe hits
/// a real sector and only the capability scope can refuse it. The largest v1
/// archive (525 328 bytes) needs 1027 sectors.
const ARCHIVE_FIRST_SECTOR: u64 = 1000;
const ARCHIVE_SECTORS: u64 = 1040;
const _: () = assert!(FIRST_SECTOR + SECTORS <= ARCHIVE_FIRST_SECTOR);

#[derive(Clone, Copy)]
struct AppEvidence {
    exited: bool,
    code: u64,
    status: u64,
    step: u64,
    docs: u64,
    bytes: u64,
    commits: u64,
    quota: u64,
    full: u64,
    digest: u64,
    recovered_revision: u64,
    recovered_collection: u64,
    recovered_second: u64,
    phase: u64,
}
const NO_APP: AppEvidence = AppEvidence {
    exited: false,
    code: 0,
    status: 0,
    step: 0,
    docs: 0,
    bytes: 0,
    commits: 0,
    quota: 0,
    full: 0,
    digest: 0,
    recovered_revision: 0,
    recovered_collection: 0,
    recovered_second: 0,
    phase: 0,
};

struct Runtime {
    info: Option<Discovery>,
    mapped: [u64; 6],
    device: Option<Device>,
    pool: DmaPool,
    policy: Policy,
    data: [u8; 512],
    error: Option<(virtio_core::Request, i64)>,
    offline: i64,
    resetting: bool,
    reset_deadline: u64,
    quarantined: bool,
    now: u64,
    roots: [u64; 2],
    replies: [u64; 2],
    client_send: [u64; 2],
    client_reply: [u64; 2],
    publications: [u64; 3],
    completions: [u64; 3],
    published_at: u64,
    max_completion_ticks: u64,
    timeouts: u64,
    table_writes: u64,
    data_writes: u64,
    cut_base: Option<(u64, u64)>,
    requests: [u64; 2],
    replies_sent: u64,
    app_device_denials: [u64; 2],
    apps: [AppEvidence; 2],
    service_startup: u64,
    finish: bool,
    /// Block object of the portal's archive range (archive fixtures only).
    archive_key: Option<u64>,
    archive_ops: [u64; 3],
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
    publications: [0; 3],
    completions: [0; 3],
    published_at: 0,
    max_completion_ticks: 0,
    timeouts: 0,
    table_writes: 0,
    data_writes: 0,
    cut_base: None,
    requests: [0; 2],
    replies_sent: 0,
    app_device_denials: [0; 2],
    apps: [NO_APP; 2],
    service_startup: 0,
    finish: false,
    archive_key: None,
    archive_ops: [0; 3],
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: Entered with KERNEL held and IF0 on the permanent kernel stack.
    // Callers end each borrow before re-entry, scheduling or cleanup/detach.
    unsafe { &mut *ptr::addr_of_mut!(RUNTIME) }
}

/// H05 power gate (ADR-076): storage durability exactly as this broker
/// observes it. `dirty` = a WRITE was published after the last successful
/// FLUSH; any failed or timed-out request keeps publications != completions.
#[cfg(feature = "native-system-power")]
struct PowerTrack {
    dirty: bool,
    injected: bool,
    hold: bool,
    io_error: bool,
}
#[cfg(feature = "native-system-power")]
static mut POWER_TRACK: PowerTrack = PowerTrack {
    dirty: false,
    injected: false,
    hold: false,
    io_error: false,
};
#[cfg(feature = "native-system-power")]
fn power_track() -> &'static mut PowerTrack {
    // SAFETY: Same single-CPU IF0 discipline as `runtime`; borrows end per call.
    unsafe { &mut *ptr::addr_of_mut!(POWER_TRACK) }
}
/// 1 = completion I/O error, 2 = timeout, on the first FLUSH published after
/// init's first power request (i.e. init's SHUTDOWN flush); never earlier.
#[cfg(feature = "native-system-power")]
const POWER_FAULT: u8 = if cfg!(feature = "test-system-power-flush-error") {
    1
} else if cfg!(feature = "test-system-power-flush-timeout") {
    2
} else {
    0
};
#[cfg(feature = "native-system-power")]
fn power_published(op: Operation) -> bool {
    let t = power_track();
    if matches!(op, Operation::Write { .. }) {
        t.dirty = true;
    }
    if POWER_FAULT == 0 || op != Operation::Flush || t.injected || !super::system::power_requested()
    {
        return false;
    }
    t.injected = true;
    t.io_error = POWER_FAULT == 1;
    t.hold = POWER_FAULT == 2;
    serial::write(if t.io_error {
        b"SYSTEM:FAULT_INJECTED:FLUSH_ERROR\n"
    } else {
        b"SYSTEM:FAULT_INJECTED:FLUSH_TIMEOUT\n"
    });
    t.hold
}
/// Ok only when nothing is in flight and every published block request
/// completed successfully, ending with a FLUSH: -8 retry later, -16 never this boot.
#[cfg(feature = "native-system-power")]
pub(super) fn power_gate() -> Result<(), i64> {
    let r = runtime();
    if r.resetting || r.policy.phase() == Phase::Published {
        return Err(-8);
    }
    if r.offline != 0 || r.error.is_some() || power_track().dirty || r.publications != r.completions
    {
        return Err(-16);
    }
    Ok(())
}

fn case() -> u64 {
    if cfg!(feature = "test-volume-recover") {
        1
    } else if cfg!(feature = "test-volume-cut-data") {
        2
    } else if cfg!(feature = "test-volume-cut-table") {
        3
    } else if cfg!(feature = "test-volume-recover-cut") {
        4
    } else if cfg!(feature = "test-volume-corrupt") || cfg!(feature = "test-volume-foreign") {
        5
    } else if cfg!(feature = "test-volume-cut-txn") {
        6
    } else if cfg!(feature = "test-volume-archive-export") {
        7
    } else if cfg!(feature = "test-volume-archive-restore") {
        8
    } else if cfg!(feature = "test-volume-archive-corrupt") {
        9
    } else {
        0
    }
}
/// Device-level read-only mount (emulator, virtio negotiation and service).
fn read_only() -> bool {
    cfg!(feature = "test-system-recover") || matches!(case(), 1 | 4 | 5)
}
/// Archive export writes only the archive range: the device stays writable,
/// but the volume service receives READ alone.
fn service_read_only() -> bool {
    read_only() || case() == 7
}
fn marker() -> &'static [u8] {
    match case() {
        0 => b"VOLUME:CASE:NORMAL:PASS\n",
        1 => b"VOLUME:CASE:RECOVER:PASS\n",
        4 => b"VOLUME:CASE:RECOVER_CUT:PASS\n",
        5 => b"VOLUME:CASE:CORRUPT:PASS\n",
        7 => b"VOLUME:CASE:ARCHIVE_EXPORT:PASS\n",
        8 => b"VOLUME:CASE:ARCHIVE_RESTORE:PASS\n",
        9 => b"VOLUME:CASE:ARCHIVE_CORRUPT:PASS\n",
        _ => b"VOLUME:FAIL:CUT_NOT_OBSERVED\n",
    }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"VOLUME:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
/// Diagnostic only: completion latency and timeout counters of the broker, so
/// a failing run shows whether a missed deadline preceded the failure. Printed
/// on success and before the run-level failures that user behaviour can cause.
fn broker_diagnostic(r: &Runtime) {
    print_number(
        b"VOLUME:BROKER:MAX_COMPLETION_TICKS:",
        r.max_completion_ticks,
    );
    print_number(b"VOLUME:BROKER:TIMEOUTS:", r.timeouts);
    if r.policy.phase() == Phase::Published {
        print_number(
            b"VOLUME:BROKER:OUTSTANDING_TICKS:",
            r.now.saturating_sub(r.published_at),
        );
    }
}
fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
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
    let device = unsafe { Device::initialize_profile(info, mapped, pool, !read_only(), false) };
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

fn releasable_frames(s: &State) -> usize {
    let mut total = 0usize;
    for process in &s.processes[..s.amount] {
        if process.status == Status::Dead {
            continue;
        }
        let Some(paging) = process.paging.as_ref() else {
            fail(b"MISSING_ADDRESS_SPACE");
        };
        total = total
            .checked_add(process.count)
            .and_then(|value| value.checked_add(process.kernel_frames.len()))
            .and_then(|value| value.checked_add(paging.owned_table_count()))
            .unwrap_or_else(|| fail(b"FRAME_COUNT_OVERFLOW"));
    }
    total
}

pub fn manifest(s: &mut State) {
    if s.amount != 3 {
        fail(b"PROCESS_COUNT");
    }
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
    // Recovery fixtures mount the same image read-only at the emulator, the
    // virtio feature negotiation and this capability; only READ is granted.
    let rights = if service_read_only() {
        BLOCK_READ
    } else {
        BLOCK_READ | BLOCK_WRITE | BLOCK_FLUSH
    };
    let block = e
        .create_block_device_with_rights(
            service,
            BlockScope {
                device: 0,
                first_sector: FIRST_SECTOR,
                sector_count: SECTORS,
            },
            rights,
        )
        .unwrap();
    for slot in 0..3 {
        for (offset, value) in [
            (0, slot as u64),
            (32, 1),
            (40, 0),
            (112, case()),
            (160, clients[0]),
            (168, clients[1]),
            (176, service),
            (456, 0),
            (464, 0),
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
        (120, u64::from(service_read_only())),
        (192, 0),
        (232, 0),
    ] {
        put(s, 0, offset, value);
    }
    for domain in 0..2 {
        put(s, domain + 1, 56, r.client_reply[domain]);
        put(s, domain + 1, 64, r.client_send[domain]);
        // Apps hold endpoint handles only: no block right of any kind.
        for handle in [r.client_send[domain], r.client_reply[domain]] {
            if e.check_block(clients[domain], handle, BLOCK_READ).is_ok() {
                fail(b"APP_BLOCK_RIGHT");
            }
        }
        if e.check_endpoint(clients[domain], r.client_send[domain], RECV)
            .is_ok()
        {
            fail(b"APP_RECV_ON_SEND");
        }
    }
    if service_read_only()
        && (e.check_block(service, block, BLOCK_WRITE).is_ok()
            || e.check_block(service, block, BLOCK_FLUSH).is_ok())
    {
        fail(b"READONLY_RIGHTS");
    }
    if matches!(case(), 7..=9) {
        archive_grant(s, e, service, clients, block);
    }
    if !e.check_invariants() {
        fail(b"MANIFEST_ENGINE");
    }
    s.free_baseline = s
        .free_count()
        .checked_add(releasable_frames(s))
        .unwrap_or_else(|| fail(b"FRAME_BASELINE_OVERFLOW"));
    serial::write(b"VOLUME:MANIFEST:PASS\n");
}

/// S09 fixture grant: the slot-1 portal alone receives the archive range;
/// export READ|WRITE|FLUSH, restore and corrupt READ only. The service and
/// slot 2 keep exactly their earlier authority.
fn archive_grant(s: &mut State, e: &mut Engine, service: u64, clients: [u64; 2], block: u64) {
    let scope = BlockScope {
        device: 0,
        first_sector: ARCHIVE_FIRST_SECTOR,
        sector_count: ARCHIVE_SECTORS,
    };
    let rights = if case() == 7 {
        BLOCK_READ | BLOCK_WRITE | BLOCK_FLUSH
    } else {
        BLOCK_READ
    };
    let handles = |e: &Engine| {
        [service, clients[0], clients[1]].map(|pid| e.process_usage(pid).unwrap().handles)
    };
    let (before, held) = (e.stats(), handles(e));
    let handle = e
        .create_block_device_with_rights(clients[0], scope, rights)
        .unwrap();
    // No handle enumeration exists, so count instead: exactly one new block
    // object, node and handle, and that handle sits in the portal; service
    // and slot 2 hold exactly what they held, so neither can reach it.
    let after = e.stats();
    if (after.block_objects, after.nodes, after.handles)
        != (
            before.block_objects + 1,
            before.nodes + 1,
            before.handles + 1,
        )
        || handles(e) != [held[0], held[1] + 1, held[2]]
    {
        fail(b"ARCHIVE_ESCAPE");
    }
    let auth = e.check_block(clients[0], handle, rights).unwrap();
    if auth.scope() != scope {
        fail(b"ARCHIVE_SCOPE");
    }
    if case() != 7
        && (e.check_block(clients[0], handle, BLOCK_WRITE).is_ok()
            || e.check_block(clients[0], handle, BLOCK_FLUSH).is_ok())
    {
        fail(b"ARCHIVE_READONLY_RIGHTS");
    }
    if e.check_block(service, block, 0)
        .is_ok_and(|volume| volume.object_key() == auth.object_key())
    {
        fail(b"ARCHIVE_ESCAPE");
    }
    runtime().archive_key = Some(auth.object_key());
    put(s, 1, 504, handle);
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
fn input_bytes(s: &State, slot: usize, pointer: u64) -> Result<[u8; 512], i64> {
    let p = &s.processes[slot];
    let plan = user::sector_input_plan(pointer, p.pid, |va| {
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
    .map_err(|_| -3)?;
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
    Ok(bytes)
}

pub fn syscall(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> (i64, u64) {
    match device_call(s, slot, n, a) {
        Ok(v) => (0, v),
        Err(code) => {
            // Apps hold no volume block capability: every app call to it
            // fails at the authority check before any device effect. Calls
            // on the S09 portal's own archive object (pending polls, its
            // scope probe) are not authority denials and are not counted.
            #[cfg(not(feature = "native-system"))]
            if slot > 0 && !on_archive(s, slot, a[0]) {
                runtime().app_device_denials[slot - 1] += 1;
            }
            (code, 0)
        }
    }
}

#[cfg(not(feature = "native-system"))]
fn on_archive(s: &State, slot: usize, handle: u64) -> bool {
    runtime().archive_key.is_some_and(|key| {
        m4::engine()
            .check_block(s.processes[slot].pid, handle, 0)
            .is_ok_and(|auth| auth.object_key() == key)
    })
}

/// Generic broker: authority, scope, ticket binding and bounded copies only.
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
            return Err(-8);
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
        Some(input_bytes(s, slot, a[2])?)
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
    r.policy.publish(request.epoch, request.ticket).unwrap();
    #[cfg(feature = "native-system-power")]
    let suppress = power_published(op);
    #[cfg(not(feature = "native-system-power"))]
    let suppress = false;
    r.device
        .as_mut()
        .unwrap()
        .publish_prepared(suppress)
        .unwrap();
    r.publications[operation_index(op)] += 1;
    if r.archive_key == Some(binding.object_key) {
        r.archive_ops[operation_index(op)] += 1;
    }
    r.published_at = s.ticks;
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

fn cut(label: &[u8]) -> ! {
    serial::write(label);
    loop {
        // SAFETY: Deliberate IF0 test rendezvous on the kernel stack. No PASS
        // is emitted; the harness must kill this live QEMU child process.
        unsafe {
            asm!("cli; hlt", options(nomem, nostack));
        }
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
                Device::initialize_profile(r.info.unwrap(), r.mapped, r.pool, !read_only(), false)
            } {
                Ok(device) => {
                    r.device = Some(device);
                    r.policy.reset_completed(true).unwrap();
                    r.resetting = false;
                }
                Err(_) => {
                    let _ = r.policy.reset_completed(false);
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
        r.timeouts += 1;
        let code = if request.operation.is_mutating() {
            -16
        } else {
            -13
        };
        r.error = Some((request, code));
        #[cfg(feature = "native-system-power")]
        {
            power_track().hold = false;
        }
        reset(r, now);
        return;
    }
    // The timeout fixture also withholds observation until the real deadline.
    #[cfg(feature = "native-system-power")]
    if power_track().hold {
        return;
    }
    #[cfg(feature = "native-system-power")]
    let injection = if power_track().io_error { 10 } else { 0 };
    #[cfg(not(feature = "native-system-power"))]
    let injection = 0;
    let completion = r.device.as_mut().unwrap().poll(injection);
    // The injection applies only to an observed used-ring advance.
    #[cfg(feature = "native-system-power")]
    if !matches!(&completion, Ok(None)) {
        power_track().io_error = false;
    }
    // Err also follows an observed used-ring advance (malformed completion).
    // Only Ok(None) means that no completion arrived.
    if !matches!(&completion, Ok(None)) {
        r.max_completion_ticks = r
            .max_completion_ticks
            .max(now.saturating_sub(r.published_at));
    }
    match completion {
        Ok(Some((data, Completion::Success))) => {
            r.data = data;
            r.completions[operation_index(request.operation)] += 1;
            #[cfg(feature = "native-system-power")]
            if request.operation == Operation::Flush {
                power_track().dirty = false;
            }
            r.policy
                .complete(request.epoch, request.ticket, Completion::Success)
                .unwrap();
            if let Operation::Write { sector } = request.operation {
                // Archive sectors are neither volume table nor content writes.
                if r.archive_key == Some(request.binding.object_key) {
                    return;
                }
                let relative = sector - FIRST_SECTOR;
                if relative < TABLE_REGION {
                    r.table_writes += 1;
                } else {
                    r.data_writes += 1;
                }
                // Crash fixtures: once app A started its second upload, kill
                // after completed content writes or inside the table write.
                if matches!(case(), 2 | 3 | 6) && r.apps[0].phase >= 1 {
                    let base = *r.cut_base.get_or_insert((r.data_writes, r.table_writes));
                    if case() == 2 && r.data_writes >= base.0 + 40 {
                        cut(b"VOLUME:CUT:DATA\n");
                    }
                    if case() == 3 && r.table_writes >= base.1 + 4 {
                        cut(b"VOLUME:CUT:TABLE\n");
                    }
                    if case() == 6 && r.table_writes >= base.1 + 4 {
                        cut(b"VOLUME:CUT:TXN\n");
                    }
                }
            }
        }
        Ok(Some((_, _))) => {
            r.policy
                .complete(request.epoch, request.ticket, Completion::Io)
                .unwrap();
            let code = if request.operation.is_mutating() {
                -16
            } else {
                -12
            };
            r.error = Some((request, code));
            reset(r, now);
        }
        Ok(None) => {}
        Err(error) => {
            let code = if request.operation.is_mutating() {
                -16
            } else {
                err(error)
            };
            r.error = Some((request, code));
            reset(r, now);
        }
    }
}

pub fn sent(_s: &State, slot: usize, handle: u64, _message: &Message) {
    let r = runtime();
    if slot == 0 {
        if handle != r.replies[0] && handle != r.replies[1] {
            fail(b"SERVICE_SEND_HANDLE");
        }
        r.replies_sent += 1;
    } else if handle == r.client_send[slot - 1] {
        r.requests[slot - 1] += 1;
    }
}
pub fn received(_s: &State, _slot: usize, _handle: u64, _message: &Message) {}
pub fn cap_event(_slot: usize, _number: u64) {}

fn snapshot(s: &State, slot: usize) {
    let r = runtime();
    let app = &mut r.apps[slot - 1];
    app.status = word(s, slot, 40);
    app.step = word(s, slot, 240);
    app.docs = word(s, slot, 400);
    app.bytes = word(s, slot, 408);
    app.commits = word(s, slot, 416);
    app.quota = word(s, slot, 424);
    app.full = word(s, slot, 432);
    app.digest = word(s, slot, 448);
    app.phase = word(s, slot, 456);
    app.recovered_revision = word(s, slot, 480);
    app.recovered_collection = word(s, slot, 488);
    app.recovered_second = word(s, slot, 496);
}

pub fn observe_exit(s: &mut State, slot: usize) {
    if slot == 0 {
        broker_diagnostic(runtime());
        fail(b"SERVICE_EXIT");
    }
    snapshot(s, slot);
    let r = runtime();
    let app = &mut r.apps[slot - 1];
    app.exited = true;
    app.code = s.last_event.code;
    let (code, status, step) = (app.code, app.status, app.step);
    if code != 0 || status != 1 {
        print_number(b"VOLUME:APP_SLOT:", slot as u64);
        print_number(b"VOLUME:APP_STEP:", step);
        broker_diagnostic(r);
        fail(b"APP_RESULT");
    }
}
pub fn observe_fault(_s: &mut State, slot: usize, _context: &[u64; 22]) {
    print_number(b"VOLUME:FAULT_SLOT:", slot as u64);
    broker_diagnostic(runtime());
    fail(b"UNEXPECTED_FAULT");
}

fn barriers(s: &State) {
    // Fixture-only ordering of the quota/full phases, and in the S09 archive
    // cases of slot 2 after the portal's archive I/O. Gates carry no
    // authority: the service still authorizes every request by endpoint.
    for slot in 1..3 {
        if s.processes[slot].status != Status::Dead {
            runtime().apps[slot - 1].phase = word(s, slot, 456);
        }
    }
    let r = runtime();
    let (a, b) = (r.apps[0].phase, r.apps[1].phase);
    let mut gates = [0u64; 2];
    if a >= 1 && b >= 1 {
        gates = [1, 1];
    }
    if a >= 2 && gates[1] >= 1 {
        gates[1] = 2;
    }
    if b >= 2 && gates[0] >= 1 {
        gates[0] = 2;
    }
    if a >= 3 && gates[1] >= 2 {
        gates[1] = 3;
    }
    for slot in 1..3 {
        if s.processes[slot].status != Status::Dead {
            put(s, slot, 464, gates[slot - 1]);
        }
    }
}

pub fn poll(s: &mut State) -> Option<*const u64> {
    advance(s);
    if s.unexpected || s.fault {
        broker_diagnostic(runtime());
        fail(b"USER_OR_PROTOCOL");
    }
    if s.ticks >= RUN_TICKS {
        for slot in 1..3 {
            if s.processes[slot].status != Status::Dead {
                print_number(b"VOLUME:APP_SLOT:", slot as u64);
                print_number(b"VOLUME:APP_PHASE:", word(s, slot, 456));
            }
        }
        broker_diagnostic(runtime());
        fail(b"TIMEOUT");
    }
    if s.processes[0].status == Status::Dead {
        broker_diagnostic(runtime());
        fail(b"SERVICE_GONE");
    }
    let state = word(s, 0, 192);
    runtime().service_startup = state;
    if state == 9 && case() != 5 {
        print_number(b"VOLUME:SERVICE_ERROR:", word(s, 0, 232));
        broker_diagnostic(runtime());
        fail(b"SERVICE_STARTUP");
    }
    barriers(s);
    let done = runtime().apps.iter().all(|app| app.exited) && (case() != 5 || state == 9);
    if !done {
        return None;
    }
    {
        let r = runtime();
        if !r.finish {
            // Let the service drain one more scheduling round.
            r.finish = true;
            return None;
        }
        if r.resetting || r.error.is_some() || r.policy.phase() == Phase::Published {
            return None;
        }
    }
    finish(s)
}

fn finish(s: &mut State) -> ! {
    let r = runtime();
    // Include the broker state even when a final acceptance check fails.
    broker_diagnostic(r);
    let service_error = word(s, 0, 232) as i64;
    if case() == 5 {
        // Corrupt image: refused before READY with Corrupt; no write rights
        // were granted and no mutation, flush or app reply happened.
        if service_error != -18 || r.publications[1..] != [0, 0] || r.replies_sent != 0 {
            fail(b"CORRUPT_RESULT");
        }
    } else {
        if service_error != 0 || word(s, 0, 376) != 0 {
            fail(b"SERVICE_STATE");
        }
        if r.publications != r.completions {
            fail(b"IO_ACCOUNTING");
        }
        if read_only() && r.publications[1..] != [0, 0] {
            fail(b"READONLY_MUTATED");
        }
        // Export: every write and flush went to the archive range and the
        // portal read the archive back; restore/corrupt only read it.
        let a = r.archive_ops;
        let archive = match case() {
            7 => a[0] > 0 && a[1] > 0 && a[2] > 0 && r.publications[1..] == a[1..],
            8 | 9 => a[0] > 0 && a[1..] == [0, 0],
            _ => a == [0; 3],
        };
        if !archive {
            fail(b"ARCHIVE_IO");
        }
        if r.app_device_denials.iter().any(|count| *count == 0) {
            fail(b"APP_DEVICE_DENIAL_NOT_OBSERVED");
        }
    }
    if case() == 0 {
        let (a, b) = (r.apps[0], r.apps[1]);
        if a.quota != 1
            || b.full != 1
            || word(s, 0, 304) < 64
            || word(s, 0, 312) <= 64
            || word(s, 0, 384) != 1
            || word(s, 0, 392) != 2
        {
            fail(b"NORMAL_RESULT");
        }
    }
    serial::write(b"VOLUME:READY\n");
    for (index, app) in r.apps.iter().enumerate() {
        let prefix: [&[u8]; 6] = if index == 0 {
            [
                b"VOLUME:A:DOCS:",
                b"VOLUME:A:BYTES:",
                b"VOLUME:A:COMMITS:",
                b"VOLUME:A:DIGEST:",
                b"VOLUME:A:REQUESTS:",
                b"VOLUME:A:RECOVERED_REVISION:",
            ]
        } else {
            [
                b"VOLUME:B:DOCS:",
                b"VOLUME:B:BYTES:",
                b"VOLUME:B:COMMITS:",
                b"VOLUME:B:DIGEST:",
                b"VOLUME:B:REQUESTS:",
                b"VOLUME:B:RECOVERED_REVISION:",
            ]
        };
        print_number(prefix[0], app.docs);
        print_number(prefix[1], app.bytes);
        print_number(prefix[2], app.commits);
        print_number(prefix[3], app.digest);
        print_number(prefix[4], r.requests[index]);
        print_number(prefix[5], app.recovered_revision);
        if index == 0 {
            print_number(b"VOLUME:A:RECOVERED_COLLECTION:", app.recovered_collection);
            print_number(b"VOLUME:A:RECOVERED_SECOND:", app.recovered_second);
        }
    }
    for (label, offset) in [
        (&b"VOLUME:SERVICE:GENERATION:"[..], 128u64),
        (b"VOLUME:SERVICE:COMMITS:", 304),
        (b"VOLUME:SERVICE:CHUNK_ALLOCATIONS:", 312),
        (b"VOLUME:SERVICE:DATA_SECTORS:", 320),
        (b"VOLUME:SERVICE:TABLE_WRITES:", 328),
        (b"VOLUME:SERVICE:FLUSHES:", 336),
        (b"VOLUME:SERVICE:READ_SECTORS:", 344),
        (b"VOLUME:SERVICE:REJECTED:", 352),
        (b"VOLUME:SERVICE:ABORTED:", 360),
        (b"VOLUME:SERVICE:UNKNOWN:", 368),
        (b"VOLUME:SERVICE:TRANSACTIONS:", 384),
        (b"VOLUME:SERVICE:RESTORES:", 392),
    ] {
        print_number(label, word(s, 0, offset));
    }
    print_number(b"VOLUME:BROKER:READS:", r.publications[0]);
    print_number(b"VOLUME:BROKER:WRITES:", r.publications[1]);
    print_number(b"VOLUME:BROKER:FLUSHES:", r.publications[2]);
    print_number(b"VOLUME:BROKER:TABLE_SECTOR_WRITES:", r.table_writes);
    print_number(b"VOLUME:BROKER:DATA_SECTOR_WRITES:", r.data_writes);
    if matches!(case(), 7..=9) {
        print_number(b"VOLUME:ARCHIVE:READS:", r.archive_ops[0]);
        print_number(b"VOLUME:ARCHIVE:WRITES:", r.archive_ops[1]);
        print_number(b"VOLUME:ARCHIVE:FLUSHES:", r.archive_ops[2]);
    }
    print_number(
        b"VOLUME:APP_DEVICE_DENIALS:",
        r.app_device_denials[0] + r.app_device_denials[1],
    );
    serial::write(marker());
    // Resource accounting: release the still-live service, then require the
    // exact frame baseline and an empty capability engine.
    s.audit_process(0);
    let released = releasable_frames(s);
    if s.free_count().checked_add(released) != Some(s.free_baseline) {
        fail(b"FRAME_BASELINE_CHANGED");
    }
    if !r.quarantined {
        if let Some(device) = r.device.as_mut() {
            device.begin_reset();
            if !device.reset_acknowledged() {
                fail(b"FINAL_RESET");
            }
        }
    }
    let pool = r.pool;
    s.processes[0].status = Status::Dying;
    s.cleanup(0);
    let stats = m4::engine().stats();
    if s.free_count() != s.free_baseline
        || (
            stats.processes,
            stats.handles,
            stats.nodes,
            stats.endpoints,
            stats.block_objects,
            stats.queued_messages,
        ) != (0, 0, 0, 0, 0, 0)
    {
        fail(b"CLEANUP");
    }
    for page in [
        pool.descriptors,
        pool.available,
        pool.used,
        pool.request,
        pool.bounce,
    ] {
        if s.frames.owner(page) != Ok(Owner::Kernel) || s.kernel.leaf(page) & (4 | NX | 2) != NX | 2
        {
            fail(b"DMA_POOL");
        }
    }
    serial::write(b"VOLUME:RESOURCES:PASS\nVOLUME:PASS\nM1:PASS\n");
    serial::exit(0x10)
}

/// Device progress only; volume fixture barriers/observers are not product policy.
#[cfg(feature = "native-system")]
pub(super) fn broker_tick(s: &State) {
    advance(s);
}
#[cfg(feature = "native-system")]
pub(super) fn broker_summary() -> ([u64; 3], [u64; 3], bool) {
    let r = runtime();
    (
        r.publications,
        r.completions,
        r.offline == 0 && !r.resetting && r.error.is_none() && r.policy.phase() != Phase::Published,
    )
}
