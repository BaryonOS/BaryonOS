//! Raw NIC broker. Authority, owned copies and device lifetime only: ARP/IP/DNS
//! parsing and destination policy belong to an isolated ring-3 network service.
use super::*;
use kernel_core::user::CopyPlan;
use virtio_net_driver::{Device, NetPool, MAX_FRAME_BYTES};
#[path = "network_tx.rs"]
mod tx_state;
use tx_state::{Binding, Completion};

const PART: usize = 320;
const PARTS: usize = MAX_FRAME_BYTES.div_ceil(PART);
const RESET_TICKS: u64 = 10;
#[cfg(feature = "test-net-service-device-reset")]
struct ResetFault {
    armed: bool,
    tick: u64,
    ticket: u64,
    submitted: u64,
    received: u64,
    status: (u8, u8, bool, bool),
    terminal_reads: u64,
    terminal_code: i64,
    rx_errors: u64,
    rx_error_code: i64,
}
struct Runtime {
    device: Option<Device>,
    owner: Option<(u64, u64, u64)>, // PID, handle, object generation key
    pending: Option<Binding>,
    completed: Option<Completion>,
    next_ticket: u64,
    now: u64,
    offline: i64,
    rx: [u8; MAX_FRAME_BYTES],
    rx_length: usize,
    resetting: bool,
    reset_deadline: u64,
    quarantined: bool,
    tx_submitted: u64,
    tx_completed: u64,
    rx_completed: u64,
    rx_delivered: u64,
    denied: u64,
    timeouts: u64,
    resets: u64,
    reset_acks: u64,
    #[cfg(feature = "test-net-service-device-reset")]
    fault: ResetFault,
    #[cfg(feature = "test-system-net-link")]
    link: link::Link,
}
static mut RUNTIME: Runtime = Runtime {
    device: None,
    owner: None,
    pending: None,
    completed: None,
    next_ticket: 1,
    now: 0,
    offline: -14,
    rx: [0; MAX_FRAME_BYTES],
    rx_length: 0,
    resetting: false,
    reset_deadline: 0,
    quarantined: false,
    tx_submitted: 0,
    tx_completed: 0,
    rx_completed: 0,
    rx_delivered: 0,
    denied: 0,
    timeouts: 0,
    resets: 0,
    reset_acks: 0,
    #[cfg(feature = "test-net-service-device-reset")]
    fault: ResetFault {
        armed: false,
        tick: 0,
        ticket: 0,
        submitted: 0,
        received: 0,
        status: (0, 0, false, false),
        terminal_reads: 0,
        terminal_code: 0,
        rx_errors: 0,
        rx_error_code: 0,
    },
    #[cfg(feature = "test-system-net-link")]
    link: link::Link::NEW,
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: KERNEL held, IF=0, permanent service stack. No borrow survives
    // scheduling, a syscall return or re-entry; no raw user reference is stored.
    unsafe { &mut *ptr::addr_of_mut!(RUNTIME) }
}
fn error(e: virtio_core::Error) -> i64 {
    match e {
        virtio_core::Error::Busy | virtio_core::Error::Pending => -8,
        virtio_core::Error::Timeout => -13,
        virtio_core::Error::Offline => -14,
        virtio_core::Error::Io => -12,
        virtio_core::Error::Range => -11,
        _ => -15,
    }
}
pub(super) fn prepare(frames: &mut Frames<FRAME_COUNT>, kernel: &mut Paging) {
    let mut pages = [0; 4];
    for page in &mut pages {
        *page = frames
            .allocate(Owner::Kernel, paging::zero_dynamic)
            .unwrap();
        kernel.map(frames, *page, *page, true, false);
    }
    let pool = NetPool {
        rx_ring: pages[0],
        tx_ring: pages[1],
        rx_bounce: pages[2],
        tx_bounce: pages[3],
    };
    pool.validate(PAGE * FRAME_COUNT as u64).unwrap();
    let info = match virtio_net_driver::discover() {
        Ok(i) => i,
        Err(_) => return,
    };
    let mut mapped = [0; 6];
    for region in [
        info.regions.common,
        info.regions.notify,
        info.regions.isr,
        info.regions.device,
    ] {
        let index = region.bar as usize;
        let bar = info.bars[index];
        assert!(bar.base >= 0x80000000 && bar.size <= 65536);
        mapped[index] = 0x3f100000 + index as u64 * 65536;
        let first = u64::from(region.offset) & !(PAGE - 1);
        let end = (u64::from(region.offset) + u64::from(region.length) + PAGE - 1) & !(PAGE - 1);
        for offset in (first..end).step_by(PAGE as usize) {
            let va = mapped[index] + offset;
            if kernel.leaf(va) == 0 {
                kernel.map_mmio(frames, va, bar.base + offset).unwrap();
            }
            assert_eq!(kernel.leaf(va) & (4 | NX | 8 | 16), NX | 8 | 16);
        }
    }
    // SAFETY: Exact validated BAR spans have permanent supervisor UC/NX
    // mappings; four distinct owned normal-RAM pages are identity mapped and
    // pinned until reboot, even on error. This is the sole NIC instance.
    let mut device = match unsafe { Device::initialize(info, mapped, pool) } {
        Ok(d) => d,
        Err(e) => {
            runtime().offline = error(e);
            runtime().quarantined = true;
            return;
        }
    };
    if device.mac() != [2, 0, 0, 0, 0, 7] {
        device.begin_reset();
        runtime().offline = -15;
        runtime().quarantined = true;
        runtime().device = Some(device);
        return;
    }
    if let Err(e) = device.post_rx() {
        device.begin_reset();
        runtime().offline = error(e);
        runtime().quarantined = true;
        runtime().device = Some(device);
        return;
    }
    #[cfg(feature = "test-system-net-link")]
    link::attach(&mut runtime().link, &device);
    runtime().device = Some(device);
    runtime().offline = 0;
}
pub(super) fn bind(pid: u64, handle: u64, key: u64) {
    assert!(runtime().owner.is_none());
    runtime().owner = Some((pid, handle, key));
}
fn authorize(pid: u64, handle: u64) -> Result<u64, i64> {
    let key = m4::engine()
        .check_network(pid, handle)
        .map_err(|e| e.code())?;
    if runtime().owner != Some((pid, handle, key)) {
        return Err(-5);
    }
    Ok(key)
}
/// At most five prevalidated Message320-sized ranges. This does not enlarge
/// IPC or change kernel-core's copy policy, and validates *all* output first.
fn plans(
    s: &State,
    slot: usize,
    pointer: u64,
    length: usize,
    output: bool,
) -> Result<[Option<CopyPlan>; PARTS], i64> {
    let mut result = [None; PARTS];
    let mut offset = 0;
    for plan in &mut result {
        if offset == length {
            break;
        }
        let n = PART.min(length - offset);
        let address = pointer.checked_add(offset as u64).ok_or(-3)?;
        *plan = Some(m4::plan(s, slot, address, n as u64, output).map_err(|_| -3)?);
        offset += n;
    }
    if offset != length {
        return Err(-11);
    }
    Ok(result)
}
fn read(plans: &[Option<CopyPlan>; PARTS], length: usize) -> [u8; MAX_FRAME_BYTES] {
    let mut frame = [0; MAX_FRAME_BYTES];
    let mut at = 0;
    for plan in plans.iter().flatten() {
        let bytes = m4::read_plan(plan);
        let n = PART.min(length - at);
        frame[at..at + n].copy_from_slice(&bytes[..n]);
        at += n;
    }
    frame
}
fn write(plans: &[Option<CopyPlan>; PARTS], bytes: &[u8]) {
    let mut at = 0;
    for plan in plans.iter().flatten() {
        for chunk in &plan.chunks[..plan.count] {
            if at == bytes.len() {
                return;
            }
            let n = chunk.length.min(bytes.len() - at);
            paging::physical(chunk.physical, |destination| {
                for i in 0..n {
                    // SAFETY: The entire caller-owned writable range was validated
                    // before any copy. KERNEL held, IF=0; mappings cannot change here.
                    // Copies stop at actual frame length and preserve output tail.
                    unsafe {
                        destination
                            .add(chunk.offset + i)
                            .write_volatile(bytes[at + i]);
                    }
                }
            });
            at += n;
        }
    }
    assert_eq!(at, bytes.len());
}
pub(super) fn syscall(s: &mut State, slot: usize, number: u64, args: [u64; 6]) -> (i64, u64) {
    match call(s, slot, number, args) {
        Ok(v) => (0, v),
        Err(code) => {
            if code != -8 {
                runtime().denied += 1;
            }
            #[cfg(feature = "test-net-service-device-reset")]
            if code != -8 && number == 0x72 && runtime().fault.ticket != 0 {
                runtime().fault.rx_errors += 1;
                runtime().fault.rx_error_code = code;
            }
            (code, 0)
        }
    }
}
fn call(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> Result<u64, i64> {
    if !(matches!(n, 0x70..=0x72) || cfg!(feature = "test-system-net-link") && n == 0x74) {
        return Err(-2);
    }
    if a[4..].iter().any(|x| *x != 0)
        || n == 0x71 && a[2..4].iter().any(|x| *x != 0)
        || n == 0x72 && a[3] != 0
        || cfg!(feature = "test-system-net-link") && n == 0x74 && a[1..4].iter().any(|x| *x != 0)
    {
        return Err(-1);
    }
    let pid = s.processes[slot].pid;
    let key = authorize(pid, a[0])?;
    match n {
        0x70 => {
            if !(14..=MAX_FRAME_BYTES as u64).contains(&a[2]) {
                return Err(-11);
            }
            if !(1..=500).contains(&a[3]) {
                return Err(-1);
            }
            let copy = plans(s, slot, a[1], a[2] as usize, false)?;
            let frame = read(&copy, a[2] as usize);
            let r = runtime();
            if r.offline != 0 {
                return Err(r.offline);
            }
            // Before any publication, bounce-page (DMA) write or doorbell.
            #[cfg(feature = "test-system-net-link")]
            if !link::usable(r) {
                return Err(link::refuse(r, false));
            }
            if r.pending.is_some() || r.completed.is_some() {
                return Err(-8);
            }
            let ticket = r.next_ticket;
            let next = ticket.checked_add(1).ok_or(-7)?;
            r.device
                .as_mut()
                .ok_or(-14)?
                .send(&frame[..a[2] as usize], s.ticks, a[3])
                .map_err(error)?;
            r.next_ticket = next;
            r.pending = Some((pid, key, ticket));
            r.tx_submitted += 1;
            #[cfg(feature = "test-net-service-device-reset")]
            if r.fault.armed && r.fault.ticket == 0 {
                // The normal send has published/notified a real descriptor.
                // Do not call begin_reset: software still believes it is live.
                r.fault.status = r.device.as_mut().unwrap().inject_device_reset().unwrap();
                r.fault.tick = s.ticks;
                r.fault.ticket = ticket;
                r.fault.submitted = r.tx_submitted;
                r.fault.received = r.rx_delivered;
            }
            Ok(ticket)
        }
        0x71 => {
            if a[1] == 0 {
                return Err(-1);
            }
            let r = runtime();
            if let Some((binding, result)) = r.completed {
                if binding != (pid, key, a[1]) {
                    return Err(-1);
                }
                #[cfg(feature = "test-net-service-device-reset")]
                if r.fault.ticket == a[1] {
                    r.fault.terminal_reads += 1;
                    r.fault.terminal_code = result.err().unwrap_or(0);
                }
                r.completed = None;
                return result.map(|n| n as u64);
            }
            if r.pending == Some((pid, key, a[1])) {
                return Err(-8);
            }
            Err(-1)
        }
        0x72 => {
            if !(14..=MAX_FRAME_BYTES as u64).contains(&a[2]) {
                return Err(-11);
            }
            let copy = plans(s, slot, a[1], a[2] as usize, true)?;
            let r = runtime();
            if r.offline != 0 {
                return Err(r.offline);
            }
            // Before any copy-out or repost; no old-epoch bytes are held here.
            #[cfg(feature = "test-system-net-link")]
            if !link::usable(r) {
                return Err(link::refuse(r, true));
            }
            if r.rx_length == 0 {
                return Err(-8);
            }
            let count = r.rx_length;
            if count > a[2] as usize {
                return Err(-11);
            }
            write(&copy, &r.rx[..count]);
            r.rx.fill(0);
            r.rx_length = 0;
            r.rx_delivered += 1;
            // The prior frame was fully copied. Repost failure affects future
            // operations, not the valid completed receive or its output bytes.
            if let Err(e) = r.device.as_mut().unwrap().post_rx() {
                stop(r, s.ticks, error(e));
            }
            Ok(count as u64)
        }
        #[cfg(feature = "test-system-net-link")]
        0x74 => link::query(runtime()),
        _ => Err(-2),
    }
}
fn stop(r: &mut Runtime, now: u64, code: i64) {
    if let Some(device) = r.device.as_mut() {
        device.begin_reset();
    }
    // A requested reset ends the link lifetime: no later state or event.
    #[cfg(feature = "test-system-net-link")]
    link::latch(r);
    r.offline = code;
    // RX failures reset the same device as TX. Preserve every published TX's
    // binding and provide a terminal uncertain outcome instead of eternal Again.
    tx_state::terminalize(&mut r.pending, &mut r.completed, code);
    r.rx.fill(0);
    r.rx_length = 0;
    if !r.resetting {
        r.resets += 1;
        r.resetting = true;
        r.reset_deadline = now.saturating_add(RESET_TICKS);
    }
}
pub(super) fn detach(pid: u64) {
    if runtime().owner.is_some_and(|(owner, _, _)| owner == pid) {
        let r = runtime();
        r.owner = None;
        r.pending = None;
        r.completed = None;
        // Reset acknowledgement is observed later; pinned DMA is never freed.
        let now = r.now;
        stop(r, now, -14);
    }
}
pub(super) fn broker_tick(s: &State) {
    runtime().now = s.ticks;
    if let Some((pid, handle, key)) = runtime().owner {
        if m4::engine().check_network(pid, handle) != Ok(key) {
            detach(pid);
        }
    }
    let r = runtime();
    if r.resetting {
        if r.device.as_ref().is_some_and(Device::reset_acknowledged) {
            r.resetting = false;
            r.reset_acks += 1;
        } else if s.ticks >= r.reset_deadline {
            r.resetting = false;
            r.quarantined = true;
        }
        return;
    }
    if r.offline != 0 {
        return;
    }
    #[cfg(feature = "test-system-net-link")]
    if link::poll(r, s.ticks) {
        return;
    }
    if let Some(binding) = r.pending {
        match r.device.as_mut().unwrap().poll_tx(s.ticks) {
            Ok(Some(length)) => {
                r.pending = None;
                r.completed = Some((binding, Ok(length)));
                r.tx_completed += 1;
            }
            Ok(None) => {}
            Err(e) => {
                let code = error(e);
                if code == -13 {
                    r.timeouts += 1;
                }
                r.pending = None;
                r.completed = Some((binding, Err(code)));
                stop(r, s.ticks, code);
            }
        }
    }
    if r.offline == 0 && r.rx_length == 0 {
        match r.device.as_mut().unwrap().poll_rx(&mut r.rx) {
            Ok(Some(length)) => {
                r.rx_length = length;
                r.rx_completed += 1;
                #[cfg(feature = "test-system-net-link")]
                link::received(r, s.ticks);
            }
            Ok(None) => {}
            Err(e) => stop(r, s.ticks, error(e)),
        }
    }
}
pub(super) fn finished() -> bool {
    let r = runtime();
    #[cfg(feature = "test-system-net-link-storm")]
    if !link::held(r) {
        return false;
    }
    r.owner.is_none()
        && r.pending.is_none()
        && r.completed.is_none()
        && !r.resetting
        && !r.quarantined
        && if cfg!(feature = "test-net-service-device-reset") {
            (1..=2).contains(&r.reset_acks) && r.reset_acks == r.resets
        } else if cfg!(feature = "test-system-net-link-storm") {
            // Exhaustion reset, then the owner-exit reset: both acknowledged.
            r.resets == 2 && r.reset_acks == 2
        } else {
            r.reset_acks == 1
        }
}
#[cfg(feature = "test-net-service-device-reset")]
pub(super) fn arm_device_reset() {
    let r = runtime();
    assert!(!r.fault.armed && r.fault.ticket == 0 && r.offline == 0);
    r.fault.armed = true;
}
#[cfg(feature = "test-net-service-device-reset")]
pub(super) fn device_reset_evidence() -> (u64, u64, u64) {
    let r = runtime();
    assert!(r.fault.armed && r.fault.tick > 0 && r.fault.ticket > 0);
    assert_eq!(r.fault.status, (15, 0, true, true));
    assert_eq!(r.fault.terminal_reads, 1);
    assert_eq!(r.fault.terminal_code, -12);
    assert!(r.fault.rx_errors <= 1);
    assert_eq!(
        r.fault.rx_error_code,
        if r.fault.rx_errors == 1 { -12 } else { 0 }
    );
    // No new publication or delivery after the unexpected status change.
    assert_eq!(r.tx_submitted, r.fault.submitted);
    assert_eq!(r.rx_delivered, r.fault.received);
    for (label, value) in [
        (&b"SYSTEM:NET_FAULT_TICK:"[..], r.fault.tick),
        (b"SYSTEM:NET_FAULT_TICKET:", r.fault.ticket),
        (b"SYSTEM:NET_FAULT_TX:", r.fault.submitted),
        (b"SYSTEM:NET_FAULT_RX:", r.fault.received),
        (
            b"SYSTEM:NET_FAULT_STATUS_BEFORE:",
            u64::from(r.fault.status.0),
        ),
        (
            b"SYSTEM:NET_FAULT_STATUS_AFTER:",
            u64::from(r.fault.status.1),
        ),
        (b"SYSTEM:NET_FAULT_BME_BEFORE:", u64::from(r.fault.status.2)),
        (b"SYSTEM:NET_FAULT_BME_AFTER:", u64::from(r.fault.status.3)),
        (b"SYSTEM:NET_TERMINAL_READS:", r.fault.terminal_reads),
        (b"SYSTEM:NET_FAULT_RX_ERRORS:", r.fault.rx_errors),
        (
            b"SYSTEM:NET_FAULT_RX_ERROR_CODE:",
            (-r.fault.rx_error_code) as u64,
        ),
        (
            b"SYSTEM:NET_TERMINAL_ERROR:",
            (-r.fault.terminal_code) as u64,
        ),
    ] {
        print_number(label, value);
    }
    (r.fault.tick, r.fault.ticket, r.fault.rx_errors)
}
pub(super) fn print_and_check(tx: u64, rx: u64) {
    let r = runtime();
    assert_eq!(
        r.tx_submitted,
        r.tx_completed + u64::from(cfg!(feature = "test-net-service-device-reset"))
    );
    assert_eq!(tx, r.tx_completed);
    assert_eq!(rx, r.rx_delivered);
    // Only observer expectations differ. Authority checks, owned copy plans,
    // DMA lifetime, finite TX deadlines and reset/quarantine are unchanged.
    if cfg!(feature = "native-system-tcp") {
        if cfg!(feature = "test-net-service-exhaustion") {
            // The bounded discard peer sends no replies; allocator isolation
            // is proved through real service RPCs, not a TCP handshake.
            assert!(r.tx_completed > 0);
            assert_eq!(r.rx_completed, 0);
            assert_eq!(r.rx_delivered, 0);
        } else {
            assert!(r.tx_completed > 0 && r.rx_delivered > 0 && r.rx_completed >= r.rx_delivered);
        }
        #[cfg(feature = "test-net-service-device-reset")]
        assert_eq!(r.denied, 2 + r.fault.terminal_reads + r.fault.rx_errors);
        #[cfg(not(feature = "test-net-service-device-reset"))]
        assert_eq!(
            r.denied,
            if cfg!(feature = "native-net-service") {
                // Each app, plus the N04 policy process, proves it has no NIC.
                2 + u64::from(cfg!(feature = "test-net-service-revoke"))
                    + u64::from(cfg!(feature = "test-net-service-helper"))
            } else {
                1
            }
        );
        assert_eq!(r.timeouts, 0); // Transport blackhole is not a DMA timeout.
        if cfg!(feature = "test-net-service-device-reset") {
            // Owner exit may request a second reset after the fault ACK.
            // Both resets must really acknowledge; quarantine is never a pass.
            assert!((1..=2).contains(&r.resets));
            assert_eq!(r.reset_acks, r.resets);
        } else {
            assert_eq!(r.resets, 1);
            assert_eq!(r.reset_acks, 1);
        }
    } else {
        assert!(r.tx_completed >= 4 && r.rx_delivered >= 5 && r.denied >= 7 && r.timeouts == 0);
    }
    for (label, value) in [
        (&b"SYSTEM:NET_TX_SUBMITTED:"[..], r.tx_submitted),
        (b"SYSTEM:NET_TX_COMPLETED:", r.tx_completed),
        (b"SYSTEM:NET_RX_COMPLETED:", r.rx_completed),
        (b"SYSTEM:NET_RX_DELIVERED:", r.rx_delivered),
        (b"SYSTEM:NET_DENIED:", r.denied),
        (b"SYSTEM:NET_TX_TIMEOUTS:", r.timeouts),
        (b"SYSTEM:NET_RESETS:", r.resets),
        (b"SYSTEM:NET_RESET_ACKS:", r.reset_acks),
    ] {
        print_number(label, value);
    }
}

/// H04-T3, test profile only: kernel-owned virtio-net link state for the
/// NetworkSource owner. One coherent STATUS sample per PIT tick is numbered
/// with kernel epochs (`virtio_net_driver::LinkMonitor`); 0x70/0x72 are
/// refused while the link is down or not yet observed, before any copy-out,
/// descriptor publication, bounce-page (DMA) write or doorbell; undelivered RX
/// bytes are destroyed at every down edge and completions arriving while down
/// are discarded. 0x74 exposes only `(epoch, up)`: no registers, MAC or rights.
/// Without VIRTIO_NET_F_STATUS the link is assumed active (virtio 1.2,
/// 5.1.4.2) and 0x74 reports Unsupported (-15). Exhaustion or any reset
/// latches: stop() keeps the pinned DMA pages and waits for the reset ACK.
#[cfg(feature = "test-system-net-link")]
pub(super) mod link {
    use super::*;
    use virtio_net_driver::{LinkEvent, LinkMonitor};

    /// Distinct refusal: the link is down or not yet observed. Not Offline
    /// (-14, device unusable) and not Again (-8, retry without a state
    /// change); -12..-22 are taken by device, storage, service and clock codes.
    pub(super) const LINK_DOWN: i64 = -23;
    /// Transitions after the first sample per device lifetime, never refilled.
    const BUDGET: u32 = 8;
    /// Storm profile: the latched broker stays observable this many PIT ticks
    /// so that later QMP toggles land while the guest still runs.
    #[cfg(feature = "test-system-net-link-storm")]
    const HOLD_TICKS: u64 = 300;
    /// Probe evidence words (startup page offsets) observed at its exit.
    pub(in super::super) const PROBE_WORDS: [u64; 13] = [
        600, 608, 616, 624, 632, 640, 648, 656, 664, 672, 680, 688, 696,
    ];
    /// Startup word that tells the probe which link phase to run.
    pub(in super::super) const MODE_WORD: u64 = 592;
    pub(in super::super) const MODE: u64 = if cfg!(feature = "test-system-net-link-storm") {
        2
    } else {
        1
    };

    #[derive(Clone, Copy)]
    struct Counters {
        events: u64,
        ups: u64,
        downs: u64,
        synthetic: u64,
        busy: u64,
        tx_refused: u64,
        rx_refused: u64,
        rx_dropped: u64,
        windows: u64,
        window_delta: u64,
        queries: u64,
    }
    pub(super) struct Link {
        monitor: Option<LinkMonitor>,
        supported: bool,
        state: Option<(u64, bool)>,
        polled: Option<u64>,
        reported: Option<(u64, bool)>,
        // Open down window: TX submissions and (RX, TX) publications at its start.
        window: Option<(u64, (u16, u16))>,
        refused: (bool, bool, bool),
        exhausted: Option<u64>,
        counters: Counters,
    }
    impl Link {
        pub(super) const NEW: Self = Self {
            monitor: None,
            supported: false,
            state: None,
            polled: None,
            reported: None,
            window: None,
            refused: (false, false, false),
            exhausted: None,
            counters: Counters {
                events: 0,
                ups: 0,
                downs: 0,
                synthetic: 0,
                busy: 0,
                tx_refused: 0,
                rx_refused: 0,
                rx_dropped: 0,
                windows: 0,
                window_delta: 0,
                queries: 0,
            },
        };
    }

    pub(super) fn attach(link: &mut Link, device: &Device) {
        match LinkMonitor::new(device.negotiated_features(), BUDGET) {
            Ok(monitor) => {
                link.monitor = Some(monitor);
                link.supported = true;
                serial::write(b"NETLINK:ATTACH:STATUS\n");
            }
            Err(_) => serial::write(b"NETLINK:ATTACH:NO_STATUS\n"),
        }
    }

    /// Unknown counts as unusable; without STATUS the link is assumed active.
    pub(super) fn usable(r: &Runtime) -> bool {
        !r.link.supported || r.link.state.is_some_and(|(_, up)| up)
    }

    pub(super) fn refuse(r: &mut Runtime, receive: bool) -> i64 {
        let link = &mut r.link;
        if receive {
            link.counters.rx_refused += 1;
            link.refused.1 = true;
        } else {
            link.counters.tx_refused += 1;
            link.refused.0 = true;
        }
        // Handshake for the runner: both directions refused in this window.
        if link.window.is_some() && link.refused.0 && link.refused.1 && !link.refused.2 {
            link.refused.2 = true;
            print_number(
                b"NETLINK:DOWN_REFUSED:",
                link.state.map_or(0, |(epoch, _)| epoch),
            );
        }
        LINK_DOWN
    }

    pub(super) fn query(r: &mut Runtime) -> Result<u64, i64> {
        if r.offline != 0 {
            return Err(r.offline);
        }
        if !r.link.supported {
            return Err(error(virtio_core::Error::Unsupported));
        }
        let (epoch, up) = r.link.state.ok_or(-8)?;
        r.link.counters.queries += 1;
        if r.link.reported != Some((epoch, up)) {
            r.link.reported = Some((epoch, up));
            print_number(
                if up {
                    b"NETLINK:QUERY:UP:"
                } else {
                    b"NETLINK:QUERY:DOWN:"
                },
                epoch,
            );
        }
        // epoch <= BUDGET + 1: the shift cannot lose bits.
        Ok(epoch << 1 | u64::from(up))
    }

    pub(super) fn latch(r: &mut Runtime) {
        if let Some(monitor) = r.link.monitor.as_mut() {
            monitor.latch_reset();
        }
    }

    #[cfg(feature = "test-system-net-link-storm")]
    pub(super) fn held(r: &Runtime) -> bool {
        r.link
            .exhausted
            .is_some_and(|tick| r.now >= tick.saturating_add(HOLD_TICKS))
    }

    /// Destroy an undelivered old-epoch copy and hand the same pinned RX
    /// buffer back to the device. Returns true when the broker stopped.
    fn drop_rx(r: &mut Runtime, now: u64) -> bool {
        if r.rx_length == 0 {
            return false;
        }
        r.rx.fill(0);
        r.rx_length = 0;
        r.link.counters.rx_dropped += 1;
        print_number(b"NETLINK:RX_DROPPED:", r.link.counters.rx_dropped);
        if let Err(e) = r.device.as_mut().unwrap().post_rx() {
            stop(r, now, error(e));
            return true;
        }
        false
    }

    /// After an RX completion was copied into the broker buffer.
    pub(super) fn received(r: &mut Runtime, now: u64) {
        if !usable(r) {
            let _ = drop_rx(r, now);
        } else if r.link.supported {
            print_number(b"NETLINK:RX_HELD:", r.rx_completed);
        }
    }

    fn apply(r: &mut Runtime, event: LinkEvent, now: u64) -> bool {
        r.link.counters.events += 1;
        r.link.counters.synthetic += u64::from(event.synthetic);
        r.link.state = Some((event.epoch, event.up));
        let published = r.device.as_ref().unwrap().publications();
        if event.up {
            r.link.counters.ups += 1;
            if let Some((submitted, opened)) = r.link.window.take() {
                // Nothing may have been published or rung while down.
                let delta = r.tx_submitted.wrapping_sub(submitted)
                    + u64::from(published.0.wrapping_sub(opened.0))
                    + u64::from(published.1.wrapping_sub(opened.1));
                r.link.counters.windows += 1;
                r.link.counters.window_delta += delta;
                print_number(b"NETLINK:WINDOW_CLOSE_TX:", r.tx_submitted);
                print_number(
                    b"NETLINK:WINDOW_CLOSE_PUBLISHED:",
                    u64::from(published.0) << 16 | u64::from(published.1),
                );
            }
            print_number(
                if event.synthetic {
                    b"NETLINK:SYNTHETIC:UP:"
                } else {
                    b"NETLINK:EVENT:UP:"
                },
                event.epoch,
            );
            return false;
        }
        r.link.counters.downs += 1;
        // Epoch isolation: in-flight TX is not terminalized (QEMU drops it and
        // completes the descriptor with used.len 0, which is ordinary TX).
        if drop_rx(r, now) {
            return true;
        }
        let published = r.device.as_ref().unwrap().publications();
        r.link.window = Some((r.tx_submitted, published));
        r.link.refused = (false, false, false);
        print_number(b"NETLINK:WINDOW_OPEN_TX:", r.tx_submitted);
        print_number(
            b"NETLINK:WINDOW_OPEN_PUBLISHED:",
            u64::from(published.0) << 16 | u64::from(published.1),
        );
        print_number(
            if event.synthetic {
                b"NETLINK:SYNTHETIC:DOWN:"
            } else {
                b"NETLINK:EVENT:DOWN:"
            },
            event.epoch,
        );
        false
    }

    /// At most one coherent sample (<= LINK_READ_ATTEMPTS reads) and two
    /// events per PIT tick. Returns true when the broker stopped.
    pub(super) fn poll(r: &mut Runtime, now: u64) -> bool {
        if r.link.monitor.is_none() || r.link.polled == Some(now) {
            return false;
        }
        r.link.polled = Some(now);
        let sample = r.device.as_ref().unwrap().link_status();
        let monitor = r.link.monitor.as_mut().unwrap();
        let result = match sample {
            Ok((generation, up)) => monitor.observe(generation, up),
            Err(virtio_core::Error::Busy) => {
                r.link.counters.busy += 1;
                return false;
            }
            Err(e) => Err(e),
        };
        match result {
            Ok(events) => events
                .into_iter()
                .flatten()
                .any(|event| apply(r, event, now)),
            Err(virtio_core::Error::Exhausted) => {
                r.link.exhausted = Some(now);
                print_number(
                    b"NETLINK:EXHAUSTED:",
                    r.link.state.map_or(0, |(epoch, _)| epoch),
                );
                stop(r, now, -14);
                true
            }
            Err(e) => {
                stop(r, now, error(e));
                true
            }
        }
    }

    /// Kernel and probe link evidence after the probe exited.
    pub(in super::super) fn check_and_print(probe: [u64; 13]) {
        let r = runtime();
        let l = &r.link;
        let c = l.counters;
        let epoch = l.state.map_or(0, |(epoch, _)| epoch);
        let observations = l.monitor.as_ref().map_or(0, LinkMonitor::observations);
        let kernel = [
            u64::from(l.supported),
            epoch,
            c.events,
            c.ups,
            c.downs,
            c.synthetic,
            c.tx_refused,
            c.rx_refused,
            c.rx_dropped,
            c.windows,
            c.window_delta,
            u64::from(l.exhausted.is_some()),
        ];
        for (label, value) in [
            (&b"SYSTEM:NETLINK_SUPPORTED:"[..], kernel[0]),
            (b"SYSTEM:NETLINK_EPOCH:", kernel[1]),
            (b"SYSTEM:NETLINK_EVENTS:", kernel[2]),
            (b"SYSTEM:NETLINK_UPS:", kernel[3]),
            (b"SYSTEM:NETLINK_DOWNS:", kernel[4]),
            (b"SYSTEM:NETLINK_SYNTHETIC:", kernel[5]),
            (b"SYSTEM:NETLINK_TX_REFUSED:", kernel[6]),
            (b"SYSTEM:NETLINK_RX_REFUSED:", kernel[7]),
            (b"SYSTEM:NETLINK_RX_DROPPED:", kernel[8]),
            (b"SYSTEM:NETLINK_WINDOWS:", kernel[9]),
            (b"SYSTEM:NETLINK_WINDOW_DELTA:", kernel[10]),
            (b"SYSTEM:NETLINK_EXHAUSTED:", kernel[11]),
            (b"SYSTEM:NETLINK_OBSERVATIONS:", observations),
            (b"SYSTEM:NETLINK_BUSY:", c.busy),
            (b"SYSTEM:NETLINK_QUERIES:", c.queries),
            (b"SYSTEM:NETLINK_PROBE_SUPPORTED:", probe[0]),
            (b"SYSTEM:NETLINK_PROBE_FIRST_EPOCH:", probe[1]),
            (b"SYSTEM:NETLINK_PROBE_DOWN_EPOCH:", probe[2]),
            (b"SYSTEM:NETLINK_PROBE_UP_EPOCH:", probe[3]),
            (b"SYSTEM:NETLINK_PROBE_TX_REFUSED:", probe[4]),
            (b"SYSTEM:NETLINK_PROBE_RX_REFUSED:", probe[5]),
            (b"SYSTEM:NETLINK_PROBE_STALE_ABSENT:", probe[6]),
            (b"SYSTEM:NETLINK_PROBE_ARP_REPLIES:", probe[7]),
            (b"SYSTEM:NETLINK_PROBE_UDP_ECHOES:", probe[8]),
            (b"SYSTEM:NETLINK_PROBE_DENIED:", probe[9]),
            (b"SYSTEM:NETLINK_PROBE_OFFLINE:", probe[10]),
            (b"SYSTEM:NETLINK_PROBE_PREDOWN:", probe[11]),
            (b"SYSTEM:NETLINK_PROBE_LAST_EPOCH:", probe[12]),
        ] {
            print_number(label, value);
        }
        // supported, epoch, events, ups, downs, synthetic, tx/rx refused,
        // rx dropped, windows, window delta, exhausted.
        let (expected, probe_expected): ([u64; 12], [u64; 12]) =
            if cfg!(feature = "test-system-net-link-storm") {
                (
                    [1, 9, 9, 5, 4, 0, 0, 0, 1, 4, 0, 1],
                    [1, 1, 0, 0, 0, 0, 0, 0, 0, 3, 2, 1],
                )
            } else if l.supported {
                (
                    [1, 3, 3, 2, 1, 0, 1, 1, 1, 1, 0, 0],
                    [1, 1, 2, 3, 1, 1, 1, 1, 1, 3, 0, 1],
                )
            } else {
                ([0; 12], [0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0])
            };
        assert_eq!(kernel, expected, "kernel link evidence");
        assert_eq!(probe[..12], probe_expected, "probe link evidence");
        // The probe only reports epochs the kernel actually handed out.
        let last = probe[12];
        assert!(if l.supported {
            (1..=epoch).contains(&last)
        } else {
            last == 0
        });
        if !cfg!(feature = "test-system-net-link-storm") {
            assert_eq!(last, epoch);
        }
        assert!(observations >= c.events && (!l.supported || c.queries > 0));
    }
}
