//! B03 native update profile (ADR-044). The kernel part is the same generic
//! scoped block broker as the volume profile, here over the whole `NOSSLOT1`
//! store, plus one labelled crash fixture. It knows sectors, rights and
//! tickets; never manifests, signatures, versions or slots. Verification and
//! the two-phase install run in the ring-3 update service; the loader verifies
//! again independently at the next boot and the host disk oracle checks the
//! stored bytes. Single CPU, IF0; owned snapshots, no retained user pointers.
use super::*;
use capability_core::{BlockScope, Message, BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE};
use virtio_blk_driver::{Device, Discovery, DmaPool};
use virtio_core::{Binding, Completion, Operation, Phase, Policy};

const TIMEOUT: u64 = 64;
const RESET_TIMEOUT: u64 = 10;
/// Granted scope: the whole slot store (update-core `STORE_SECTORS`; the
/// host harness pins this constant against the format crate).
const SECTORS: u64 = 12310;
/// Whole-run budget in 100 Hz timer ticks; the host also bounds wall time.
const RUN_TICKS: u64 = 60_000;
/// Crash fixture: a real host kill after this many completed writes outside
/// the header sector, i.e. while the service copies the image.
const CUT_AFTER_WRITES: u64 = 64;
/// This kernel's own image version (build env `NOS_IMAGE_VERSION`), printed
/// as evidence and handed to the service as its current version.
const IMAGE_VERSION_TEXT: &str = match option_env!("NOS_IMAGE_VERSION") {
    Some(text) => text,
    None => "0",
};
const fn parse_version(text: &str) -> u64 {
    let bytes = text.as_bytes();
    let mut value = 0u64;
    let mut index = 0;
    while index < bytes.len() {
        assert!(
            bytes[index] >= b'0' && bytes[index] <= b'9',
            "NOS_IMAGE_VERSION must be decimal"
        );
        value = value * 10 + (bytes[index] - b'0') as u64;
        index += 1;
    }
    value
}
pub const IMAGE_VERSION: u64 = parse_version(IMAGE_VERSION_TEXT);

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
    publications: [u64; 3],
    completions: [u64; 3],
    header_writes: u64,
    other_writes: u64,
    cut_done: bool,
    service_state: u64,
    finish: bool,
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
    publications: [0; 3],
    completions: [0; 3],
    header_writes: 0,
    other_writes: 0,
    cut_done: false,
    service_state: 0,
    finish: false,
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: The single CPU enters here with IF0 on the permanent kernel stack.
    // Callers end each borrow before re-entry, scheduling or cleanup/detach.
    unsafe { &mut *ptr::addr_of_mut!(RUNTIME) }
}

fn case() -> u64 {
    u64::from(cfg!(feature = "test-update-cut"))
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"UPDATE:FAIL:");
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

pub fn prepare(frames: &mut Frames<FRAME_COUNT>, kernel: &mut Paging) {
    print_number(b"KERNEL:IMAGE_VERSION:", IMAGE_VERSION);
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
        let bar = info.bar_grants[index].expect("matched PCI BAR grant");
        let driver_id = virtio_blk_driver::DRIVER_REGISTRATION.id.get();
        assert!(bar.matches_device(
            info.address,
            virtio_blk_driver::DRIVER_REGISTRATION.id,
            region.bar
        ));
        assert!(bar.base() >= 0x80000000 && bar.size() <= 65536);
        mapped[index] = 0x3f000000 + index as u64 * 65536;
        let first = u64::from(region.offset) & !(PAGE - 1);
        let end = (u64::from(region.offset) + u64::from(region.length) + PAGE - 1) & !(PAGE - 1);
        for offset in (first..end).step_by(PAGE as usize) {
            let va = mapped[index] + offset;
            if kernel.leaf(va) == 0 {
                kernel
                    .map_pci_bar(frames, va, bar, info.address, driver_id, region.bar, offset)
                    .unwrap();
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
    if s.amount != 1 {
        fail(b"PROCESS_COUNT");
    }
    let e = m4::engine();
    let service = s.processes[0].pid;
    e.register(service).unwrap();
    let block = e
        .create_block_device_with_rights(
            service,
            BlockScope {
                device: 0,
                first_sector: 0,
                sector_count: SECTORS,
            },
            BLOCK_READ | BLOCK_WRITE | BLOCK_FLUSH,
        )
        .unwrap();
    for (offset, value) in [
        (0, 0),
        (32, 1),
        (40, 0),
        (48, block),
        (112, case()),
        (120, IMAGE_VERSION),
        (176, service),
        (192, 0),
        (232, 0),
    ] {
        put(s, 0, offset, value);
    }
    if e.check_block(service, block, BLOCK_WRITE).is_err() || !e.check_invariants() {
        fail(b"MANIFEST_ENGINE");
    }
    s.free_baseline = s
        .free_count()
        .checked_add(releasable_frames(s))
        .unwrap_or_else(|| fail(b"FRAME_BASELINE_OVERFLOW"));
    serial::write(b"UPDATE:MANIFEST:PASS\n");
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
        Err(code) => (code, 0),
    }
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
        // One non-blocking device poll before answering: a completed request
        // is reported now instead of after the next timer tick. The kernel
        // still never waits in a syscall; an unfinished request is EAGAIN.
        advance(s);
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
        if device.begin_reset().is_err() {
            r.quarantined = true;
        }
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
        if r.device.as_mut().is_some_and(Device::reset_acknowledged) {
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
        let code = if request.operation.is_mutating() {
            -16
        } else {
            -13
        };
        r.error = Some((request, code));
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
            if let Operation::Write { sector } = request.operation {
                if sector == 0 {
                    r.header_writes += 1;
                } else {
                    r.other_writes += 1;
                    // Labelled crash fixture: the host kills the live guest
                    // in the middle of the image copy, before any header
                    // write, on a first attempt only (word 368 = resumed).
                    if case() == 1 && r.other_writes >= CUT_AFTER_WRITES && word(s, 0, 368) == 0 {
                        r.cut_done = true;
                        cut(b"UPDATE:CUT:IMAGE\n");
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

// This profile grants no endpoint: any IPC would already have failed in m4.
pub fn sent(_s: &State, _slot: usize, _handle: u64, _message: &Message) {
    fail(b"UNEXPECTED_IPC");
}
pub fn received(_s: &State, _slot: usize, _handle: u64, _message: &Message) {
    fail(b"UNEXPECTED_IPC");
}
pub fn cap_event(_slot: usize, _number: u64) {}

pub fn observe_exit(_s: &mut State, _slot: usize) {
    fail(b"SERVICE_EXIT");
}
pub fn observe_fault(_s: &mut State, slot: usize, _context: &[u64; 22]) {
    print_number(b"UPDATE:FAULT_SLOT:", slot as u64);
    fail(b"UNEXPECTED_FAULT");
}

pub fn poll(s: &mut State) -> Option<*const u64> {
    advance(s);
    if s.unexpected || s.fault {
        fail(b"USER_OR_PROTOCOL");
    }
    if s.ticks >= RUN_TICKS {
        print_number(b"UPDATE:SERVICE_STEP:", word(s, 0, 240));
        fail(b"TIMEOUT");
    }
    if s.processes[0].status == Status::Dead {
        fail(b"SERVICE_GONE");
    }
    let state = word(s, 0, 192);
    runtime().service_startup(state);
    if state == 9 {
        print_number(b"UPDATE:SERVICE_STEP:", word(s, 0, 240));
        print_number(b"UPDATE:SERVICE_ERROR:", word(s, 0, 232));
        fail(b"SERVICE_ERROR");
    }
    if state != 1 {
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

impl Runtime {
    fn service_startup(&mut self, state: u64) {
        self.service_state = state;
    }
}

fn reject_name(code: u64) -> &'static [u8] {
    match code {
        1 => b"FORMAT",
        2 => b"UNKNOWN_KEY",
        3 => b"SIGNATURE",
        4 => b"DIGEST",
        5 => b"DOWNGRADE",
        6 => b"READ",
        7 => b"KEY_REVOKED",
        8 => b"KEY_EXPIRED",
        9 => b"KEY_NOT_YET_VALID",
        10 => b"KEY_POLICY",
        _ => b"UNKNOWN",
    }
}

fn finish(s: &mut State) -> ! {
    let r = runtime();
    // The cut fixture must have fired on a first attempt; a resumed install
    // (service word 368) deliberately runs to completion on the same kernel.
    if case() == 1 && !r.cut_done && word(s, 0, 368) == 0 {
        fail(b"CUT_NOT_OBSERVED");
    }
    if r.publications != r.completions {
        fail(b"IO_ACCOUNTING");
    }
    let result = word(s, 0, 256);
    let writes = r.publications[1];
    // Policy outcomes the kernel can check without knowing the format: a
    // rejected, absent or already-current bundle never caused a write; an
    // install wrote the header exactly once and flushed at least three times.
    match result {
        1 => {
            if r.header_writes != 1 || r.publications[2] < 3 || r.other_writes == 0 {
                fail(b"INSTALL_WRITE_PATTERN");
            }
        }
        2..=4 => {
            if writes != 0 || r.publications[2] != 0 {
                fail(b"WROTE_WITHOUT_INSTALL");
            }
        }
        _ => fail(b"RESULT_CODE"),
    }
    serial::write(b"UPDATE:READY\n");
    for (label, offset) in [
        (&b"UPDATE:RESULT:"[..], 256u64),
        (b"UPDATE:HEADER_ACTIVE:", 264),
        (b"UPDATE:HEADER_GENERATION:", 272),
        (b"UPDATE:ACTIVE_VERSION:", 280),
        (b"UPDATE:ACTIVE_REJECT:", 288),
        (b"UPDATE:INBOX_VERSION:", 296),
        (b"UPDATE:INBOX_REJECT:", 304),
        (b"UPDATE:INSTALLED_VERSION:", 312),
        (b"UPDATE:TARGET_SLOT:", 320),
        (b"UPDATE:SERVICE_READS:", 328),
        (b"UPDATE:SERVICE_WRITES:", 336),
        (b"UPDATE:SERVICE_FLUSHES:", 344),
        (b"UPDATE:COPIED_SECTORS:", 352),
        (b"UPDATE:NEW_GENERATION:", 360),
        (b"UPDATE:RESUMED:", 368),
        (b"UPDATE:RUNNING_SLOT:", 384),
        (b"UPDATE:OTHER_VERSION:", 392),
        (b"UPDATE:OTHER_REJECT:", 400),
    ] {
        print_number(label, word(s, 0, offset));
    }
    print_number(b"UPDATE:CURRENT_VERSION:", IMAGE_VERSION);
    print_number(b"UPDATE:BROKER:READS:", r.publications[0]);
    print_number(b"UPDATE:BROKER:WRITES:", r.publications[1]);
    print_number(b"UPDATE:BROKER:FLUSHES:", r.publications[2]);
    print_number(b"UPDATE:BROKER:HEADER_WRITES:", r.header_writes);
    print_number(b"UPDATE:BROKER:OTHER_WRITES:", r.other_writes);
    print_number(b"UPDATE:TICKS:", s.ticks);
    match result {
        1 => print_number(b"UPDATE:INSTALLED:", word(s, 0, 312)),
        2 => serial::write(b"UPDATE:UP_TO_DATE\n"),
        3 => {
            serial::write(b"UPDATE:REJECTED:");
            serial::write(reject_name(word(s, 0, 304)));
            serial::write(b"\n");
        }
        _ => serial::write(b"UPDATE:NO_BUNDLE\n"),
    }
    // Resource accounting: release the still-live service, then require the
    // exact frame baseline and an empty capability engine.
    s.audit_process(0);
    let released = releasable_frames(s);
    if s.free_count().checked_add(released) != Some(s.free_baseline) {
        fail(b"FRAME_BASELINE_CHANGED");
    }
    if !r.quarantined {
        if let Some(device) = r.device.as_mut() {
            if device.begin_reset().is_err() || !device.reset_acknowledged() {
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
    serial::write(b"UPDATE:RESOURCES:PASS\nUPDATE:PASS\nM1:PASS\n");
    serial::exit(0x10)
}
