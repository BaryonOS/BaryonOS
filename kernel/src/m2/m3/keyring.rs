//! B01 native keyring profile (ADR-043). The kernel owns the virtio-rng
//! device and brokers raw entropy bytes to holders of an `Entropy`
//! capability; it performs no cryptography, no mixing and no caching. Two
//! Key Service instances (slots 0 and 1), one app (slot 2), and a normal-case
//! management instance of the app ELF (slot 3) run in ring 3;
//! the app reports its observations through evidence words and the kernel
//! verifies them together with its own counters. Single CPU, IF=0.
use super::*;
use capability_core::{RECV, SEND};
use virtio_rng_driver::{Device, Discovery, RngPool, MAX_REQUEST};

/// Broker deadline for one device answer, in 100 Hz ticks.
const TIMEOUT: u64 = 64;
/// A caller must consume a completed answer within the same bounded interval.
const COMPLETED_LEASE: u64 = TIMEOUT;
const RESET_TIMEOUT: u64 = 10;
/// Whole-run budget in ticks; the host also bounds wall time.
const RUN_TICKS: u64 = 6000;
const SERVICE_SLOTS: [usize; 2] = [0, 1];
const APP_SLOT: usize = 2;
const ADMIN_SLOT: usize = 3;

const NORMAL: u64 = 0;
const NO_DEVICE: u64 = 1;
const STUCK: u64 = 2;
const NONRESPONSIVE: u64 = 3;
const RESTART: u64 = 4;

/// Service evidence words (startup page offsets), snapshotted at finish.
const SERVICE_WORDS: [u64; 10] = [192, 200, 208, 216, 224, 232, 240, 248, 256, 264];
/// App evidence words, snapshotted at app exit.
const APP_WORDS: [u64; 17] = [
    40, 240, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472, 480, 488, 496, 504, 528,
];

#[derive(Clone, Copy, PartialEq, Eq)]
struct EntropyBinding {
    pid: u64,
    /// Opaque capability handle; its high word is the capability generation.
    handle: u64,
    /// Entropy object index and generation returned by capability-core.
    object_key: u64,
}

#[derive(Clone, Copy)]
struct Pending {
    binding: EntropyBinding,
    ticket: u64,
    requested: u32,
    deadline: u64,
}

#[derive(Clone, Copy)]
struct Completed {
    binding: EntropyBinding,
    ticket: u64,
    result: Result<u32, i64>,
    deadline: u64,
}

struct Runtime {
    info: Option<Discovery>,
    mapped: [u64; 6],
    device: Option<Device>,
    pool: RngPool,
    offline: i64,
    /// One outstanding device request, bound to its exact capability and object.
    pending: Option<Pending>,
    /// One completed result with a bounded owner-consumption lease.
    completed: Option<Completed>,
    bytes: [u8; 256],
    next_ticket: u64,
    requests: u64,
    delivered_bytes: u64,
    denied: u64,
    offline_errors: u64,
    timeouts: u64,
    io_errors: u64,
    busy: u64,
    resetting: bool,
    reset_deadline: u64,
    reset_acks: u64,
    quarantined: bool,
    restart_stage: u8,
    restart_deadline: u64,
    restarted_bytes: u64,
    /// Completed results reclaimed because their consumption lease expired.
    lease_reclaims: u64,
    lease_stage: u8,
    lease_deadline: u64,
    lease_ticket: u64,
    app_exited: bool,
    app_code: u64,
    app: [u64; 17],
    admin_exited: bool,
    admin: [u64; 4],
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
    resetting: false,
    reset_deadline: 0,
    reset_acks: 0,
    quarantined: false,
    restart_stage: 0,
    restart_deadline: 0,
    restarted_bytes: 0,
    lease_reclaims: 0,
    lease_stage: 0,
    lease_deadline: 0,
    lease_ticket: 0,
    app_exited: false,
    app_code: 0,
    app: [0; 17],
    admin_exited: false,
    admin: [0; 4],
    finish: 0,
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: KERNEL held, IF=0; each borrow ends before scheduling or cleanup.
    unsafe { &mut *ptr::addr_of_mut!(RUNTIME) }
}

fn entropy_binding(pid: u64, handle: u64) -> Result<EntropyBinding, i64> {
    let authority = m4::engine()
        .authorize_entropy(pid, handle)
        .map_err(|error| error.code())?;
    Ok(EntropyBinding {
        pid,
        handle,
        object_key: authority.object_key(),
    })
}

fn binding_is_live(binding: EntropyBinding) -> bool {
    m4::engine()
        .authorize_entropy(binding.pid, binding.handle)
        .is_ok_and(|authority| authority.object_key() == binding.object_key)
}

fn reclaim_completed(now: u64) {
    let Some(completed) = runtime().completed else {
        return;
    };
    let expired = now >= completed.deadline;
    if expired || !binding_is_live(completed.binding) {
        let r = runtime();
        r.completed = None;
        r.bytes = [0; 256];
        r.lease_reclaims += u64::from(expired);
    }
}

/// H05-T4 evidence: one count per broker access to the virtio-rng device
/// (request submission, used-ring poll, reset, reset acknowledgement read),
/// counted at the call itself and independently of the devices_lost guards.
#[cfg(feature = "native-system-s3")]
static DEVICE_ACCESSES: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
fn device_access() {
    #[cfg(feature = "native-system-s3")]
    DEVICE_ACCESSES.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
}
#[cfg(feature = "native-system-s3")]
pub(super) fn device_accesses() -> u64 {
    DEVICE_ACCESSES.load(core::sync::atomic::Ordering::Relaxed)
}

fn quarantine_pending() {
    let r = runtime();
    // Dropping broker state while DMA remains live could let another holder
    // reuse its pages. Stop the device and retain the existing quarantine.
    // H05-T4 (review R1): after a validated S3 wake the power cycle already
    // stopped the device; the latch forbids its MMIO and PCI config space, so
    // only the quarantine below remains (and keeps the S5 gate at -8).
    #[cfg(feature = "native-system-s3")]
    let touch = if super::system::devices_lost() {
        super::system::device_access_suppressed();
        false
    } else {
        true
    };
    #[cfg(not(feature = "native-system-s3"))]
    let touch = true;
    if let Some(device) = r.device.as_mut().filter(|_| touch) {
        device_access();
        device.begin_reset();
    }
    r.offline = -14;
    r.quarantined = true;
    r.pending = None;
    r.bytes = [0; 256];
}

fn case() -> u64 {
    if cfg!(feature = "test-keyring-no-device") {
        NO_DEVICE
    } else if cfg!(feature = "test-keyring-stuck") {
        STUCK
    } else if cfg!(feature = "test-keyring-timeout") {
        NONRESPONSIVE
    } else if cfg!(feature = "test-keyring-restart") {
        RESTART
    } else {
        NORMAL
    }
}
fn marker() -> &'static [u8] {
    match case() {
        NORMAL => b"KEYRING:CASE:NORMAL:PASS\n",
        NO_DEVICE => b"KEYRING:CASE:NO_DEVICE:PASS\n",
        STUCK => b"KEYRING:CASE:STUCK:PASS\n",
        NONRESPONSIVE => b"KEYRING:CASE:TIMEOUT:PASS\n",
        _ => b"KEYRING:CASE:RESTART:PASS\n",
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
    let expected_processes = if case() == NORMAL { 4 } else { 3 };
    if s.amount != expected_processes || s.fault || s.exited || s.unexpected {
        fail(b"MANIFEST_STATE");
    }
    let e = m4::engine();
    for slot in 0..s.amount {
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
    if case() == NORMAL {
        let admin = s.processes[ADMIN_SLOT].pid;
        let reply_root = e.create_endpoint(admin).unwrap();
        let reply_recv = e.derive(admin, reply_root, RECV).unwrap();
        put(s, ADMIN_SLOT, 56, reply_recv);
        for (i, slot) in SERVICE_SLOTS.into_iter().enumerate() {
            let service = s.processes[slot].pid;
            let root = e.create_endpoint(service).unwrap();
            let receive = e.derive(service, root, RECV).unwrap();
            let send = e.bootstrap_delegate(service, root, admin, SEND).unwrap();
            let reply = e
                .bootstrap_delegate(admin, reply_root, service, SEND)
                .unwrap();
            for (offset, value) in [(96, receive), (104, reply), (168, admin)] {
                put(s, slot, offset, value);
            }
            put(s, ADMIN_SLOT, 64 + i as u64 * 16, send);
            // Test-only alias: SEND cannot forge the kernel-supplied sender PID.
            if i == 1 {
                let alias = e.bootstrap_delegate(service, root, app, SEND).unwrap();
                put(s, APP_SLOT, 104, alias);
            }
            e.close(service, root).unwrap();
        }
        e.close(admin, reply_root).unwrap();
        for (offset, value) in [
            (0, 3),
            (32, 1),
            (160, s.processes[0].pid),
            (168, s.processes[1].pid),
            (176, admin),
        ] {
            put(s, ADMIN_SLOT, offset, value);
        }
        // Every live handle is accounted for, including the test-only alias.
        require(
            e.stats().handles == 21 && e.check_invariants(),
            b"ADMIN_MANIFEST",
        );
        for (slot, handles) in [(0, 6), (1, 6), (APP_SLOT, 6), (ADMIN_SLOT, 3)] {
            require(
                e.available_slots(s.processes[slot].pid)
                    == capability_core::MAX_SLOTS - 1 - handles,
                b"ADMIN_HANDLE_EQUATION",
            );
        }
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
            let binding = match entropy_binding(pid, a[0]) {
                Ok(binding) => binding,
                Err(error) => {
                    runtime().denied += 1;
                    return Err(error);
                }
            };
            if a[1] == 0 || a[1] > u64::from(MAX_REQUEST) {
                return Err(-11);
            }
            // H05-T4: after an S3 wake the virtio-rng device was reset and is
            // never reinitialized; refuse before any broker or device state.
            #[cfg(feature = "native-system-s3")]
            if super::system::devices_lost() {
                super::system::device_refused();
                return Err(-14);
            }
            reclaim_completed(s.ticks);
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
            // Counted whether the request is accepted, fails or stays pending.
            device_access();
            device.request(a[1] as u32).map_err(|_| -8)?;
            let ticket = r.next_ticket;
            r.next_ticket += 1;
            r.pending = Some(Pending {
                binding,
                ticket,
                requested: a[1] as u32,
                deadline: s.ticks + TIMEOUT,
            });
            Ok(ticket)
        }
        0x61 => {
            if a[4..].iter().any(|x| *x != 0) {
                return Err(-1);
            }
            let binding = match entropy_binding(pid, a[0]) {
                Ok(binding) => binding,
                Err(error) => {
                    runtime().denied += 1;
                    return Err(error);
                }
            };
            reclaim_completed(s.ticks);
            let r = runtime();
            let Some(completed) = r.completed else {
                return match r.pending {
                    Some(pending) if pending.binding == binding && pending.ticket == a[1] => {
                        Err(-8)
                    }
                    _ => Err(-1),
                };
            };
            if completed.binding != binding || completed.ticket != a[1] {
                return Err(-1);
            }
            let length = match completed.result {
                Ok(length) => length,
                Err(code) => {
                    r.completed = None;
                    r.bytes = [0; 256];
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
    reclaim_completed(s.ticks);
    // H05-T4 (review R1): no reset acknowledgement read or used-ring poll
    // after the S3 wake. Nothing can be active here: the S3 gate admitted
    // only an idle broker, every 0x60 since returned -14, and system::poll
    // stops the kernel before this tick otherwise.
    #[cfg(feature = "native-system-s3")]
    if super::system::devices_lost() {
        return;
    }
    if runtime().resetting {
        let r = runtime();
        if r.device.is_some() {
            device_access();
        }
        if r.device.as_ref().is_some_and(Device::reset_acknowledged) {
            r.resetting = false;
            r.reset_acks += 1;
        } else if s.ticks >= r.reset_deadline {
            // No memory may be cleared, recycled or reinitialized without
            // reset acknowledgement. All three DMA pages stay pinned.
            r.resetting = false;
            r.quarantined = true;
        }
        return;
    }
    let Some(pending) = runtime().pending else {
        return;
    };
    if !binding_is_live(pending.binding) {
        quarantine_pending();
        return;
    }
    let r = runtime();
    let Some(device) = r.device.as_mut() else {
        r.pending = None;
        r.completed = Some(Completed {
            binding: pending.binding,
            ticket: pending.ticket,
            result: Err(-14),
            deadline: s.ticks + COMPLETED_LEASE,
        });
        return;
    };
    let mut buffer = [0u8; 256];
    device_access();
    match device.poll(&mut buffer) {
        Ok(Some(length)) => {
            if length != pending.requested {
                // The device answered short: that is an I/O error, never
                // padded or retried on the caller's behalf.
                r.io_errors += 1;
                r.pending = None;
                r.completed = Some(Completed {
                    binding: pending.binding,
                    ticket: pending.ticket,
                    result: Err(-12),
                    deadline: s.ticks + COMPLETED_LEASE,
                });
                return;
            }
            if case() == STUCK {
                // Labelled fixture: a device that keeps returning one byte.
                buffer[..length as usize].fill(0x5a);
            }
            r.bytes = buffer;
            r.pending = None;
            r.completed = Some(Completed {
                binding: pending.binding,
                ticket: pending.ticket,
                result: Ok(length),
                deadline: s.ticks + COMPLETED_LEASE,
            });
        }
        Ok(None) => {
            if s.ticks >= pending.deadline {
                r.timeouts += 1;
                r.pending = None;
                r.completed = Some(Completed {
                    binding: pending.binding,
                    ticket: pending.ticket,
                    result: Err(-13),
                    deadline: s.ticks + COMPLETED_LEASE,
                });
                r.bytes = [0; 256];
                device_access();
                device.begin_reset();
                r.offline = -13;
                r.resetting = true;
                r.reset_deadline = s.ticks + RESET_TIMEOUT;
            }
        }
        Err(_) => {
            r.io_errors += 1;
            r.pending = None;
            r.completed = Some(Completed {
                binding: pending.binding,
                ticket: pending.ticket,
                result: Err(-12),
                deadline: s.ticks + COMPLETED_LEASE,
            });
        }
    }
}

pub fn detach(pid: u64) {
    if runtime()
        .pending
        .is_some_and(|request| request.binding.pid == pid)
    {
        // Dropping the broker ticket alone leaves the driver's DMA request
        // outstanding forever. Fail closed and stop the device; the pinned
        // pages are quarantined until reboot, never reused by another owner.
        quarantine_pending();
    }
    if runtime()
        .completed
        .is_some_and(|request| request.binding.pid == pid)
    {
        let r = runtime();
        r.completed = None;
        r.bytes = [0; 256];
    }
}

pub fn observe_exit(s: &mut State, slot: usize, _counter: u64) {
    if case() == NORMAL && slot == ADMIN_SLOT {
        require(
            s.last_event.code == 0 && word(s, slot, 40) == 1,
            b"ADMIN_RESULT",
        );
        let r = runtime();
        for (i, offset) in [400, 408, 416, 424].into_iter().enumerate() {
            r.admin[i] = word(s, slot, offset);
        }
        r.admin_exited = true;
        return;
    }
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
    if case() == NORMAL && !r.app_exited {
        if !r.admin_exited
            && word(s, ADMIN_SLOT, 512) == 0
            && word(s, APP_SLOT, 512) == 1
            && word(s, 1, 416) == 1
        {
            // Start management only after the bad caller was rejected without
            // a reply or any key-state change in the unused service B.
            require(
                word(s, 1, 208) == 0 && word(s, 1, 216) == 0,
                b"BAD_CALLER_STATE",
            );
            require(
                word(s, 1, 400) == 0 && word(s, 1, 408) == 0,
                b"BAD_CALLER_DISPATCH",
            );
            require(
                word(s, 1, 240) == 0 && word(s, 1, 256) == 0,
                b"BAD_CALLER_COUNTERS",
            );
            require(
                m4::engine().stats().queued_messages == 0,
                b"BAD_CALLER_REPLY",
            );
            put(s, ADMIN_SLOT, 512, 1);
        }
        if r.admin_exited {
            put(s, APP_SLOT, 520, 1);
        }
    }
    if !r.app_exited {
        return None;
    }
    // Let both services drain their queues for a few rounds before judging.
    if r.finish < 8 {
        r.finish += 1;
        return None;
    }
    if r.pending.is_some() || r.resetting {
        return None;
    }
    if case() == RESTART && !restart_fixture(s) {
        return None;
    }
    if case() == NORMAL && !lease_fixture(s) {
        return None;
    }
    finish(s)
}

/// Labelled native driver acceptance only: a responding device is reset and
/// initialized again with exactly the same kernel-owned pool. This does not
/// recover a failed Key Service or silently resume a timed-out entropy source.
fn restart_fixture(s: &State) -> bool {
    let r = runtime();
    if r.restart_stage == 0 {
        require(
            r.pending.is_none() && r.completed.is_none(),
            b"RESTART_BROKER_BUSY",
        );
        require(!r.quarantined && r.offline == 0, b"RESTART_OFFLINE");
        let device = r
            .device
            .as_mut()
            .unwrap_or_else(|| fail(b"RESTART_NO_DEVICE"));
        require(!device.pending(), b"RESTART_DEVICE_BUSY");
        device.begin_reset();
        r.resetting = true;
        r.reset_deadline = s.ticks + RESET_TIMEOUT;
        r.restart_stage = 1;
        return false;
    }
    require(!r.quarantined && r.reset_acks == 1, b"RESTART_RESET_ACK");
    if r.restart_stage == 1 {
        require(
            r.device.as_ref().is_some_and(Device::reset_acknowledged),
            b"RESTART_WITHOUT_ACK",
        );
        for page in [r.pool.rings, r.pool.used, r.pool.bounce] {
            require(
                s.frames.owner(page) == Ok(Owner::Kernel),
                b"RESTART_POOL_OWNER",
            );
        }
        require(
            m4::engine().stats().entropy_objects == 2,
            b"RESTART_CAPABILITIES",
        );
        // SAFETY: The same validated mappings and exclusively pinned pages
        // supplied to prepare remain owned. Reset was acknowledged before
        // initialize can clear the old rings or bounce page.
        let mut device = unsafe { Device::initialize(r.info.unwrap(), r.mapped, r.pool) }
            .unwrap_or_else(|_| fail(b"RESTART_INITIALIZE"));
        require(
            device.pool == r.pool && !device.pending(),
            b"RESTART_POOL_CHANGED",
        );
        device
            .request(32)
            .unwrap_or_else(|_| fail(b"RESTART_REQUEST"));
        r.device = Some(device);
        r.restart_stage = 2;
        r.restart_deadline = s.ticks + TIMEOUT;
        return false;
    }
    if r.restart_stage == 2 {
        let mut bytes = [0xa5; 256];
        match r.device.as_mut().unwrap().poll(&mut bytes) {
            Ok(Some(32)) => {
                // A shorter, fresh device transaction cannot write beyond
                // its declared output or be mistaken for a startup request.
                require(
                    bytes[32..].iter().all(|b| *b == 0xa5),
                    b"RESTART_OUTPUT_BOUNDS",
                );
                r.restarted_bytes = 32;
                r.restart_stage = 3;
            }
            Ok(None) if s.ticks < r.restart_deadline => return false,
            _ => fail(b"RESTART_COMPLETION"),
        }
    }
    true
}

fn entropy_handle(s: &State, slot: usize) -> u64 {
    word(s, slot, 48)
}

/// Labelled native fixture (normal case only), driven by the real syscall
/// handler and real timer ticks after the services went idle. Service A
/// abandons a completed answer; service B is refused with EAGAIN while the
/// lease runs and admitted once the kernel reclaimed and wiped the answer.
fn lease_fixture(s: &mut State) -> bool {
    let (a, b) = (entropy_handle(s, 0), entropy_handle(s, 1));
    match runtime().lease_stage {
        0 => {
            let r = runtime();
            require(
                r.completed.is_none() && r.offline == 0 && !r.resetting,
                b"LEASE_BROKER_BUSY",
            );
            let ticket =
                call(s, 0, 0x60, [a, 256, 0, 0, 0, 0]).unwrap_or_else(|_| fail(b"LEASE_REQUEST"));
            let r = runtime();
            r.lease_ticket = ticket;
            r.lease_deadline = s.ticks + TIMEOUT;
            r.lease_stage = 1;
            false
        }
        1 | 3 => {
            let r = runtime();
            let Some(completed) = r.completed else {
                require(s.ticks < r.lease_deadline, b"LEASE_NO_ANSWER");
                return false;
            };
            require(
                completed.ticket == r.lease_ticket
                    && completed.result == Ok(256)
                    && r.bytes.iter().any(|byte| *byte != 0),
                b"LEASE_ANSWER",
            );
            // poll() returns before this fixture while a request is pending, so
            // advance() created the answer on this tick: pin the documented 64.
            require(completed.deadline == s.ticks + 64, b"LEASE_LENGTH");
            r.lease_deadline = completed.deadline;
            if r.lease_stage == 1 {
                // The abandoned answer blocks the peer only while leased.
                require(
                    call(s, 1, 0x60, [b, 256, 0, 0, 0, 0]) == Err(-8),
                    b"LEASE_PEER_NOT_BUSY",
                );
            }
            runtime().lease_stage += 1;
            false
        }
        stage => {
            let r = runtime();
            if s.ticks < r.lease_deadline {
                require(r.completed.is_some(), b"LEASE_EARLY_RECLAIM");
                return false;
            }
            // poll() ran advance() for this tick: the lease has expired.
            require(
                r.completed.is_none() && r.bytes.iter().all(|byte| *byte == 0),
                b"LEASE_NOT_RECLAIMED",
            );
            if stage == 4 {
                return true;
            }
            let ticket = r.lease_ticket;
            require(
                call(s, 0, 0x61, [a, ticket, 0, 256, 0, 0]) == Err(-1),
                b"LEASE_STALE_TICKET",
            );
            let ticket = call(s, 1, 0x60, [b, 256, 0, 0, 0, 0])
                .unwrap_or_else(|_| fail(b"LEASE_PEER_BLOCKED"));
            let r = runtime();
            r.lease_ticket = ticket;
            r.lease_deadline = s.ticks + TIMEOUT;
            r.lease_stage = 3;
            false
        }
    }
}

/// Labelled native fixture (normal case only): a holder that closes its
/// entropy handle while its DMA request is pending stops the device. The
/// peer then sees Offline (-14) until reboot; see B01-ENTROPY-AND-KEYS.md.
fn authority_loss_fixture(s: &mut State) {
    let r = runtime();
    require(
        r.pending.is_none() && r.completed.is_none() && r.offline == 0 && !r.quarantined,
        b"QUARANTINE_BROKER_BUSY",
    );
    let (pid, a, b) = (
        s.processes[0].pid,
        entropy_handle(s, 0),
        entropy_handle(s, 1),
    );
    call(s, 0, 0x60, [a, 256, 0, 0, 0, 0]).unwrap_or_else(|_| fail(b"QUARANTINE_REQUEST"));
    require(runtime().pending.is_some(), b"QUARANTINE_NOT_PENDING");
    if m4::engine().close(pid, a).is_err() {
        fail(b"QUARANTINE_CLOSE");
    }
    advance(s);
    let r = runtime();
    require(
        r.quarantined
            && r.offline == -14
            && r.pending.is_none()
            && r.completed.is_none()
            && r.bytes.iter().all(|byte| *byte == 0),
        b"AUTHORITY_NOT_QUARANTINED",
    );
    require(
        call(s, 1, 0x60, [b, 256, 0, 0, 0, 0]) == Err(-14),
        b"QUARANTINE_PEER",
    );
    print_number(b"KEYRING:AUTHORITY_QUARANTINE:", 1);
}

fn finish(s: &mut State) -> ! {
    let r = runtime();
    let app = r.app;
    let [status, step, generated, sealed, unsealed, tamper_denied, cross_denied, revoked_denied, quota_denied, recovery_export_denied, recovery_restore_denied, _, _, not_ready, entropy_denied, status_calls, b_unseal] =
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
    require(r.io_errors == 0, b"BROKER_ERRORS");
    require(
        r.timeouts == u64::from(case() == NONRESPONSIVE),
        b"TIMEOUT_COUNT",
    );
    match case() {
        NORMAL | RESTART => {
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
            // 2 + 1 + 14 generated keys, one successful seal and unseal.
            require(
                generated >= 17 && sealed >= 1 && unsealed >= 1,
                b"KEY_OPERATIONS",
            );
            require(tamper_denied >= 3, b"TAMPER_NOT_DENIED");
            require(
                cross_denied >= 1 && revoked_denied >= 1 && quota_denied >= 1,
                b"RIGHTS_NOT_DENIED",
            );
            require(
                recovery_export_denied == 1 && recovery_restore_denied == 1,
                b"RECOVERY_AUTHORITY_NOT_RESTRICTED",
            );
            require(
                services[0][4] >= 1 && services[1][8] == u64::from(case() == NORMAL),
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
        NONRESPONSIVE => {
            require(
                r.offline == -13 && r.offline_errors >= 1,
                b"TIMEOUT_NOT_OBSERVED",
            );
            require(r.delivered_bytes == 0 && r.timeouts == 1, b"TIMEOUT_BYTES");
            require(r.reset_acks == 1 && !r.quarantined, b"TIMEOUT_RESET_ACK");
            require(r.bytes.iter().all(|b| *b == 0), b"TIMEOUT_OUTPUT");
            require(
                services.iter().all(|v| v[0] == 2 && v[1] == 1),
                b"TIMEOUT_NOT_BLOCKED",
            );
            require(
                not_ready >= 2 && generated == 0 && sealed == 0,
                b"KEYS_WITHOUT_ENTROPY",
            );
            require(
                services.iter().all(|v| v[2] == 0 && v[3] == 0 && v[9] == 0),
                b"SERVICE_KEYS",
            );
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
    if case() == NORMAL {
        require(
            r.admin_exited && r.admin == [1, 1, 1, 1],
            b"MANAGEMENT_RECOVERY",
        );
        require(b_unseal == 1, b"B_UNSEAL");
        require(
            word(s, 0, 392) == 1 && word(s, 0, 400) == 0,
            b"ADMIN_EXPORT",
        );
        require(
            word(s, 1, 392) == 0 && word(s, 1, 400) == 1,
            b"ADMIN_RESTORE",
        );
        require(
            word(s, 1, 408) == 2 && word(s, 1, 416) == 1,
            b"ADMIN_NEGATIVES",
        );
        require(
            word(s, 0, 384) == 1 && word(s, 1, 384) == 4,
            b"ADMIN_NO_REPLY",
        );
        for (label, value) in [
            (&b"KEYRING:MGMT_EXPORTS:"[..], r.admin[0]),
            (b"KEYRING:MGMT_RESTORES:", r.admin[1]),
            (b"KEYRING:MGMT_WRONG_SECRET_DENIED:", r.admin[2]),
            (b"KEYRING:ADMIN_GENERATE_DENIED:", r.admin[3]),
            (b"KEYRING:B_UNSEAL_OK:", b_unseal),
            (b"KEYRING:ADMIN_BAD_CALLER:", word(s, 1, 416)),
        ] {
            print_number(label, value);
        }
    }
    serial::write(b"KEYRING:READY\n");
    for (label, value) in [
        (&b"KEYRING:REQUESTS:"[..], r.requests),
        (b"KEYRING:DELIVERED_BYTES:", r.delivered_bytes),
        (b"KEYRING:DENIED:", r.denied),
        (b"KEYRING:OFFLINE_ERRORS:", r.offline_errors),
        (b"KEYRING:BUSY:", r.busy),
        (b"KEYRING:TIMEOUTS:", r.timeouts),
        (b"KEYRING:RESET_ACKS:", r.reset_acks),
        (b"KEYRING:QUARANTINED:", u64::from(r.quarantined)),
        (b"KEYRING:RESTARTED_BYTES:", r.restarted_bytes),
        (b"KEYRING:LEASE_RECLAIMS:", r.lease_reclaims),
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
        (
            b"KEYRING:APP_RECOVERY_EXPORT_DENIED:",
            recovery_export_denied,
        ),
        (
            b"KEYRING:APP_RECOVERY_RESTORE_DENIED:",
            recovery_restore_denied,
        ),
        (b"KEYRING:APP_NOT_READY:", not_ready),
        (b"KEYRING:APP_ENTROPY_DENIED:", entropy_denied),
        (b"KEYRING:TICKS:", s.ticks),
    ] {
        print_number(label, value);
    }
    if case() == NORMAL {
        authority_loss_fixture(s);
    }
    let r = runtime();
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

/// Device progress only; no B01 fixture observer or lifecycle policy.
#[cfg(feature = "native-system")]
pub(super) fn broker_tick(s: &State) {
    advance(s);
}
#[cfg(feature = "native-system")]
pub(super) fn broker_summary() -> (u64, u64, bool) {
    let r = runtime();
    (
        r.delivered_bytes,
        r.denied,
        r.offline == 0 && r.pending.is_none() && r.completed.is_none() && !r.resetting,
    )
}
