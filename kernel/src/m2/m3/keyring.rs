//! B01 native keyring profile (ADR-043). The kernel owns the virtio-rng
//! device and brokers raw entropy bytes to holders of an `Entropy`
//! capability; it performs no cryptography, no mixing and no caching. Two
//! Key Service instances (slots 0 and 1) and one app (slot 2) run in ring 3;
//! the app reports its observations through evidence words and the kernel
//! verifies them together with its own counters. Single CPU, IF=0.
use super::*;
use capability_core::{RECV, SEND};
use virtio_rng_driver::{Device, Discovery, RngPool, MAX_REQUEST};

/// Broker deadline for one device answer, in 100 Hz ticks.
const TIMEOUT: u64 = 64;
/// Whole-run budget in ticks; the host also bounds wall time.
const RUN_TICKS: u64 = 6000;
const SERVICE_SLOTS: [usize; 2] = [0, 1];
const APP_SLOT: usize = 2;

const NORMAL: u64 = 0;
const NO_DEVICE: u64 = 1;
const STUCK: u64 = 2;

/// Service evidence words (startup page offsets), snapshotted at finish.
const SERVICE_WORDS: [u64; 10] = [192, 200, 208, 216, 224, 232, 240, 248, 256, 264];
/// App evidence words, snapshotted at app exit.
const APP_WORDS: [u64; 16] = [
    40, 240, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472, 480, 488, 496, 504,
];

struct Runtime {
    info: Option<Discovery>,
    mapped: [u64; 6],
    device: Option<Device>,
    pool: RngPool,
    offline: i64,
    /// One outstanding request: (pid, ticket, requested length, deadline tick).
    pending: Option<(u64, u64, u32, u64)>,
    /// A completed or failed request waiting for its poll: (pid, ticket, result).
    completed: Option<(u64, u64, Result<u32, i64>)>,
    bytes: [u8; 256],
    next_ticket: u64,
    requests: u64,
    delivered_bytes: u64,
    denied: u64,
    offline_errors: u64,
    timeouts: u64,
    io_errors: u64,
    busy: u64,
    app_exited: bool,
    app_code: u64,
    app: [u64; 16],
    finish: u8,
}
static mut RUNTIME: Runtime = Runtime {
    info: None,
    mapped: [0; 6],
    device: None,
    pool: RngPool {
        rings: 0,
        used: 0,
        bounce: 0,
    },
    offline: 0,
    pending: None,
    completed: None,
    bytes: [0; 256],
    next_ticket: 1,
    requests: 0,
    delivered_bytes: 0,
    denied: 0,
    offline_errors: 0,
    timeouts: 0,
    io_errors: 0,
    busy: 0,
    app_exited: false,
    app_code: 0,
    app: [0; 16],
    finish: 0,
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: Single CPU, IF=0; each borrow ends before scheduling or cleanup.
    unsafe { &mut *ptr::addr_of_mut!(RUNTIME) }
}

fn case() -> u64 {
    if cfg!(feature = "test-keyring-no-device") {
        NO_DEVICE
    } else if cfg!(feature = "test-keyring-stuck") {
        STUCK
    } else {
        NORMAL
    }
}
fn marker() -> &'static [u8] {
    match case() {
        NORMAL => b"KEYRING:CASE:NORMAL:PASS\n",
        NO_DEVICE => b"KEYRING:CASE:NO_DEVICE:PASS\n",
        _ => b"KEYRING:CASE:STUCK:PASS\n",
    }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"KEYRING:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
}
fn require(condition: bool, reason: &[u8]) {
    if !condition {
        fail(reason);
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

/// Map the rng BARs and pin three DMA pages. Without a device the broker
/// stays Offline: every request then fails with -14 and nothing is cached.
pub fn prepare(frames: &mut Frames<FRAME_COUNT>, kernel: &mut Paging) {
    let mut pages = [0; 3];
    for page in &mut pages {
        *page = frames
            .allocate(Owner::Kernel, paging::zero_dynamic)
            .unwrap();
        kernel.map(frames, *page, *page, true, false);
    }
    let pool = RngPool {
        rings: pages[0],
        used: pages[1],
        bounce: pages[2],
    };
    pool.validate(PAGE * FRAME_COUNT as u64).unwrap();
    runtime().pool = pool;
    let info = match virtio_rng_driver::discover() {
        Ok(info) => info,
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
        // The block broker uses 0x3f000000..0x3f060000; the rng device gets
        // its own window above it inside the MMIO range the paging code allows.
        mapped[index] = 0x3f080000 + index as u64 * 65536;
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
    // mappings; three distinct owned kernel RAM pages remain pinned to reboot.
    let device = unsafe { Device::initialize(info, mapped, pool) };
    let r = runtime();
    r.info = Some(info);
    r.mapped = mapped;
    match device {
        Ok(device) => r.device = Some(device),
        Err(_) => r.offline = -14,
    }
}

pub fn manifest(s: &mut State) {
    if s.amount != 3 || s.fault || s.exited || s.unexpected {
        fail(b"MANIFEST_STATE");
    }
    let e = m4::engine();
    for slot in 0..3 {
        e.register(s.processes[slot].pid).unwrap();
    }
    let app = s.processes[APP_SLOT].pid;
    // App-owned reply endpoint; both services may send to it.
    let reply_root = e.create_endpoint(app).unwrap();
    let reply_recv = e.derive(app, reply_root, RECV).unwrap();
    let mut service_reply = [0u64; 2];
    for (i, slot) in SERVICE_SLOTS.into_iter().enumerate() {
        service_reply[i] = e
            .bootstrap_delegate(app, reply_root, s.processes[slot].pid, SEND)
            .unwrap();
    }
    e.close(app, reply_root).unwrap();
    // Each service owns two ingress endpoints (domain 0 and 1); the app gets SEND.
    let mut app_send = [[0u64; 2]; 2];
    let mut service_recv = [[0u64; 2]; 2];
    let mut entropy = [0u64; 2];
    for (i, slot) in SERVICE_SLOTS.into_iter().enumerate() {
        let service = s.processes[slot].pid;
        for domain in 0..2 {
            let root = e.create_endpoint(service).unwrap();
            app_send[i][domain] = e.bootstrap_delegate(service, root, app, SEND).unwrap();
            service_recv[i][domain] = e.derive(service, root, RECV).unwrap();
            e.close(service, root).unwrap();
        }
        entropy[i] = e.create_entropy(service).unwrap();
        if e.check_entropy(service, entropy[i]).is_err()
            || e.check_endpoint(service, entropy[i], SEND).is_ok()
            || e.check_entropy(service, service_recv[i][0]).is_ok()
        {
            fail(b"ENTROPY_TYPE");
        }
    }
    // The app holds no entropy right at all: every handle it owns is an endpoint.
    for handle in [
        reply_recv,
        app_send[0][0],
        app_send[0][1],
        app_send[1][0],
        app_send[1][1],
    ] {
        if e.check_entropy(app, handle).is_ok() {
            fail(b"APP_ENTROPY_RIGHT");
        }
    }
    if !e.check_invariants() || e.stats().entropy_objects != 2 {
        fail(b"MANIFEST_ENGINE");
    }
    for (i, slot) in SERVICE_SLOTS.into_iter().enumerate() {
        for (offset, value) in [
            (0, 0),
            (32, 1),
            (48, entropy[i]),
            (56, service_recv[i][0]),
            (80, service_recv[i][1]),
            (88, service_reply[i]),
            (112, case()),
            (160, app),
            (176, s.processes[slot].pid),
            (192, 0),
            (200, 0),
        ] {
            put(s, slot, offset, value);
        }
    }
    for (offset, value) in [
        (0, 2),
        (32, 1),
        (56, reply_recv),
        (64, app_send[0][0]),
        (72, app_send[0][1]),
        (80, app_send[1][0]),
        (96, app_send[1][1]),
        (112, case()),
        (160, s.processes[0].pid),
        (168, s.processes[1].pid),
        (176, app),
        (40, 0),
        (240, 0),
    ] {
        put(s, APP_SLOT, offset, value);
    }
    s.free_baseline = s
        .free_count()
        .checked_add(releasable_frames(s))
        .unwrap_or_else(|| fail(b"FRAME_BASELINE_OVERFLOW"));
    serial::write(b"KEYRING:MANIFEST:PASS\n");
}

pub fn syscall(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> (i64, u64) {
    match call(s, slot, n, a) {
        Ok(value) => (0, value),
        Err(code) => (code, 0),
    }
}

fn call(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> Result<u64, i64> {
    let pid = s.processes[slot].pid;
    match n {
        0x60 => {
            if a[2..].iter().any(|x| *x != 0) {
                return Err(-1);
            }
            if let Err(error) = m4::engine().check_entropy(pid, a[0]) {
                runtime().denied += 1;
                return Err(error.code());
            }
            if a[1] == 0 || a[1] > u64::from(MAX_REQUEST) {
                return Err(-11);
            }
            let r = runtime();
            r.requests += 1;
            if r.offline != 0 {
                r.offline_errors += 1;
                return Err(r.offline);
            }
            if r.pending.is_some() || r.completed.is_some() {
                r.busy += 1;
                return Err(-8);
            }
            let device = r.device.as_mut().ok_or(-14)?;
            device.request(a[1] as u32).map_err(|_| -8)?;
            let ticket = r.next_ticket;
            r.next_ticket += 1;
            r.pending = Some((pid, ticket, a[1] as u32, s.ticks + TIMEOUT));
            Ok(ticket)
        }
        0x61 => {
            if a[4..].iter().any(|x| *x != 0) {
                return Err(-1);
            }
            if let Err(error) = m4::engine().check_entropy(pid, a[0]) {
                runtime().denied += 1;
                return Err(error.code());
            }
            let r = runtime();
            let Some((owner, ticket, result)) = r.completed else {
                return match r.pending {
                    Some((owner, ticket, _, _)) if owner == pid && ticket == a[1] => Err(-8),
                    _ => Err(-1),
                };
            };
            if owner != pid || ticket != a[1] {
                return Err(-1);
            }
            let length = match result {
                Ok(length) => length,
                Err(code) => {
                    r.completed = None;
                    return Err(code);
                }
            };
            if a[3] != u64::from(length) {
                return Err(-11);
            }
            // Validate the whole output range before any byte is copied.
            let plan = m4::plan(s, slot, a[2], a[3], true).map_err(|e| e.code())?;
            let mut out = [0u8; 320];
            out[..length as usize].copy_from_slice(&r.bytes[..length as usize]);
            m4::write_plan(&plan, &out);
            r.bytes = [0; 256];
            r.completed = None;
            r.delivered_bytes += u64::from(length);
            Ok(u64::from(length))
        }
        _ => Err(-2),
    }
}

fn advance(s: &State) {
    let r = runtime();
    let Some((pid, ticket, requested, deadline)) = r.pending else {
        return;
    };
    let Some(device) = r.device.as_mut() else {
        r.pending = None;
        r.completed = Some((pid, ticket, Err(-14)));
        return;
    };
    let mut buffer = [0u8; 256];
    match device.poll(&mut buffer) {
        Ok(Some(length)) => {
            if length != requested {
                // The device answered short: that is an I/O error, never
                // padded or retried on the caller's behalf.
                r.io_errors += 1;
                r.pending = None;
                r.completed = Some((pid, ticket, Err(-12)));
                return;
            }
            if case() == STUCK {
                // Labelled fixture: a device that keeps returning one byte.
                buffer[..length as usize].fill(0x5a);
            }
            r.bytes = buffer;
            r.pending = None;
            r.completed = Some((pid, ticket, Ok(length)));
        }
        Ok(None) => {
            if s.ticks >= deadline {
                r.timeouts += 1;
                r.pending = None;
                r.completed = Some((pid, ticket, Err(-13)));
                device.begin_reset();
                r.offline = -13;
            }
        }
        Err(_) => {
            r.io_errors += 1;
            r.pending = None;
            r.completed = Some((pid, ticket, Err(-12)));
        }
    }
}

pub fn detach(pid: u64) {
    let r = runtime();
    if r.pending.is_some_and(|(owner, _, _, _)| owner == pid) {
        // The requester died: its answer is dropped, never handed to another.
        r.pending = None;
        r.bytes = [0; 256];
    }
    if r.completed.is_some_and(|(owner, _, _)| owner == pid) {
        r.completed = None;
        r.bytes = [0; 256];
    }
}

pub fn observe_exit(s: &mut State, slot: usize, _counter: u64) {
    if slot != APP_SLOT {
        fail(b"SERVICE_EXIT");
    }
    let r = runtime();
    r.app_exited = true;
    r.app_code = s.last_event.code;
    for (i, offset) in APP_WORDS.iter().enumerate() {
        r.app[i] = word(s, slot, *offset);
    }
}
pub fn observe_fault(_s: &mut State, slot: usize, context: &[u64; 22], _counter: u64) {
    print_number(b"KEYRING:FAULT_SLOT:", slot as u64);
    print_number(b"KEYRING:FAULT_VECTOR:", context[15]);
    fail(b"UNEXPECTED_FAULT");
}

pub fn poll(s: &mut State) -> Option<*const u64> {
    advance(s);
    if s.unexpected || s.fault {
        fail(b"USER_OR_PROTOCOL");
    }
    if s.ticks >= RUN_TICKS {
        for slot in SERVICE_SLOTS {
            print_number(b"KEYRING:SERVICE_STATE:", word(s, slot, 192));
        }
        print_number(b"KEYRING:APP_STEP:", word(s, APP_SLOT, 240));
        fail(b"TIMEOUT");
    }
    for slot in SERVICE_SLOTS {
        if s.processes[slot].status == Status::Dead {
            fail(b"SERVICE_GONE");
        }
    }
    let r = runtime();
    if !r.app_exited {
        return None;
    }
    // Let both services drain their queues for a few rounds before judging.
    if r.finish < 8 {
        r.finish += 1;
        return None;
    }
    if r.pending.is_some() {
        return None;
    }
    finish(s)
}

fn finish(s: &mut State) -> ! {
    let r = runtime();
    let app = r.app;
    let [status, step, generated, sealed, unsealed, tamper_denied, cross_denied, revoked_denied, quota_denied, pre_restore_denied, restore_wrong_denied, restored_unseal_ok, revoked_after_restore, not_ready, entropy_denied, status_calls] =
        app;
    let mut services = [[0u64; 10]; 2];
    for (i, slot) in SERVICE_SLOTS.into_iter().enumerate() {
        for (j, offset) in SERVICE_WORDS.iter().enumerate() {
            services[i][j] = word(s, slot, *offset);
        }
    }
    for (label, value) in [
        (&b"KEYRING:DIAG:APP_STATUS:"[..], status),
        (b"KEYRING:DIAG:APP_STEP:", step),
        (b"KEYRING:DIAG:SERVICE_A_STATE:", services[0][0]),
        (b"KEYRING:DIAG:SERVICE_A_REASON:", services[0][1]),
        (b"KEYRING:DIAG:SERVICE_B_STATE:", services[1][0]),
        (b"KEYRING:DIAG:SERVICE_B_REASON:", services[1][1]),
        (b"KEYRING:DIAG:REQUESTS:", r.requests),
        (b"KEYRING:DIAG:DELIVERED:", r.delivered_bytes),
        (b"KEYRING:DIAG:OFFLINE_ERRORS:", r.offline_errors),
        (b"KEYRING:DIAG:TIMEOUTS:", r.timeouts),
        (b"KEYRING:DIAG:IO_ERRORS:", r.io_errors),
    ] {
        print_number(label, value);
    }
    if r.app_code != 0 || status != 1 {
        fail(b"APP_RESULT");
    }
    require(
        entropy_denied >= 1 && r.denied >= 1,
        b"APP_ENTROPY_NOT_DENIED",
    );
    require(status_calls >= 2, b"STATUS_CALLS");
    require(r.timeouts == 0 && r.io_errors == 0, b"BROKER_ERRORS");
    match case() {
        NORMAL => {
            require(
                services[0][0] == 1 && services[1][0] == 1,
                b"SERVICES_NOT_READY",
            );
            require(
                services[0][1] == 0 && services[1][1] == 0,
                b"SERVICE_REASON",
            );
            require(r.offline == 0 && r.offline_errors == 0, b"DEVICE_OFFLINE");
            require(
                r.delivered_bytes >= 2048 && r.delivered_bytes % 256 == 0,
                b"ENTROPY_BYTES",
            );
            require(
                services[0][9] >= 1024 && services[1][9] >= 1024,
                b"SERVICE_ENTROPY_BYTES",
            );
            // 2 + 1 + 14 generated keys, one successful seal, two unseals.
            require(
                generated >= 17 && sealed >= 1 && unsealed >= 2,
                b"KEY_OPERATIONS",
            );
            require(tamper_denied >= 3, b"TAMPER_NOT_DENIED");
            require(
                cross_denied >= 1 && revoked_denied >= 1 && quota_denied >= 1,
                b"RIGHTS_NOT_DENIED",
            );
            require(
                pre_restore_denied >= 1 && restore_wrong_denied >= 1,
                b"RECOVERY_NEGATIVE",
            );
            require(
                restored_unseal_ok >= 1 && revoked_after_restore >= 1,
                b"RECOVERY_POSITIVE",
            );
            // Service A sealed once (k0); service B restored once.
            require(
                services[0][4] >= 1 && services[1][8] >= 1,
                b"SERVICE_COUNTERS",
            );
            require(not_ready == 0, b"UNEXPECTED_NOT_READY");
        }
        NO_DEVICE => {
            require(
                r.offline == -14 && r.offline_errors >= 2,
                b"NO_DEVICE_NOT_OBSERVED",
            );
            require(
                r.delivered_bytes == 0 && r.requests >= 2,
                b"NO_DEVICE_BYTES",
            );
            require(
                services[0][0] == 2 && services[1][0] == 2,
                b"SERVICES_NOT_BLOCKED",
            );
            require(services[0][1] == 1 && services[1][1] == 1, b"BLOCK_REASON");
            require(
                not_ready >= 2 && generated == 0 && sealed == 0,
                b"KEYS_WITHOUT_ENTROPY",
            );
            require(services[0][2] == 0 && services[1][2] == 0, b"SERVICE_KEYS");
        }
        _ => {
            require(
                r.offline == 0 && r.delivered_bytes >= 256,
                b"STUCK_DELIVERY",
            );
            require(
                services[0][0] == 2 && services[1][0] == 2,
                b"SERVICES_NOT_BLOCKED",
            );
            require(services[0][1] == 2 && services[1][1] == 2, b"BLOCK_REASON");
            require(
                not_ready >= 2 && generated == 0 && sealed == 0,
                b"KEYS_WITHOUT_ENTROPY",
            );
            require(services[0][2] == 0 && services[1][2] == 0, b"SERVICE_KEYS");
        }
    }
    serial::write(b"KEYRING:READY\n");
    for (label, value) in [
        (&b"KEYRING:REQUESTS:"[..], r.requests),
        (b"KEYRING:DELIVERED_BYTES:", r.delivered_bytes),
        (b"KEYRING:DENIED:", r.denied),
        (b"KEYRING:OFFLINE_ERRORS:", r.offline_errors),
        (b"KEYRING:BUSY:", r.busy),
        (b"KEYRING:SERVICE_A_STATE:", services[0][0]),
        (b"KEYRING:SERVICE_B_STATE:", services[1][0]),
        (b"KEYRING:SERVICE_A_REASON:", services[0][1]),
        (b"KEYRING:SERVICE_B_REASON:", services[1][1]),
        (b"KEYRING:SERVICE_A_KEYS:", services[0][2] + services[0][3]),
        (b"KEYRING:SERVICE_B_KEYS:", services[1][2] + services[1][3]),
        (b"KEYRING:SERVICE_A_ENTROPY_BYTES:", services[0][9]),
        (b"KEYRING:SERVICE_B_ENTROPY_BYTES:", services[1][9]),
        (b"KEYRING:APP_GENERATED:", generated),
        (b"KEYRING:APP_SEALED:", sealed),
        (b"KEYRING:APP_UNSEALED:", unsealed),
        (b"KEYRING:APP_TAMPER_DENIED:", tamper_denied),
        (b"KEYRING:APP_CROSS_DENIED:", cross_denied),
        (b"KEYRING:APP_REVOKED_DENIED:", revoked_denied),
        (b"KEYRING:APP_QUOTA_DENIED:", quota_denied),
        (b"KEYRING:APP_PRE_RESTORE_DENIED:", pre_restore_denied),
        (b"KEYRING:APP_RESTORE_WRONG_DENIED:", restore_wrong_denied),
        (b"KEYRING:APP_RESTORED_UNSEAL_OK:", restored_unseal_ok),
        (b"KEYRING:APP_REVOKED_AFTER_RESTORE:", revoked_after_restore),
        (b"KEYRING:APP_NOT_READY:", not_ready),
        (b"KEYRING:APP_ENTROPY_DENIED:", entropy_denied),
        (b"KEYRING:TICKS:", s.ticks),
    ] {
        print_number(label, value);
    }
    serial::write(marker());
    // Resource accounting: release both live services, then require the exact
    // frame baseline and an empty engine. The DMA pool stays kernel-owned.
    for slot in SERVICE_SLOTS {
        s.audit_process(slot);
    }
    let released = releasable_frames(s);
    if s.free_count().checked_add(released) != Some(s.free_baseline) {
        fail(b"FRAME_BASELINE_CHANGED");
    }
    if let Some(device) = r.device.as_mut() {
        device.begin_reset();
        if !device.reset_acknowledged() {
            fail(b"FINAL_RESET");
        }
    }
    let pool = r.pool;
    for slot in SERVICE_SLOTS {
        s.processes[slot].status = Status::Dying;
        s.cleanup(slot);
    }
    let stats = m4::engine().stats();
    if s.free_count() != s.free_baseline
        || stats.processes != 0
        || stats.handles != 0
        || stats.nodes != 0
        || stats.endpoints != 0
        || stats.entropy_objects != 0
        || stats.queued_messages != 0
    {
        fail(b"CLEANUP");
    }
    for page in [pool.rings, pool.used, pool.bounce] {
        if s.frames.owner(page) != Ok(Owner::Kernel) || s.kernel.leaf(page) & (4 | NX | 2) != NX | 2
        {
            fail(b"DMA_POOL");
        }
    }
    serial::write(b"KEYRING:RESOURCES:PASS\nKEYRING:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
