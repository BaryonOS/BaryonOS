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
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: Single CPU, IF=0, permanent service stack. No borrow survives
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
                    // before any copy. One CPU, IF=0; mappings cannot change here.
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
            (code, 0)
        }
    }
}
fn call(s: &mut State, slot: usize, n: u64, a: [u64; 6]) -> Result<u64, i64> {
    if !matches!(n, 0x70..=0x72) {
        return Err(-2);
    }
    if a[4..].iter().any(|x| *x != 0)
        || n == 0x71 && a[2..4].iter().any(|x| *x != 0)
        || n == 0x72 && a[3] != 0
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
        _ => Err(-2),
    }
}
fn stop(r: &mut Runtime, now: u64, code: i64) {
    if let Some(device) = r.device.as_mut() {
        device.begin_reset();
    }
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
            }
            Ok(None) => {}
            Err(e) => stop(r, s.ticks, error(e)),
        }
    }
}
pub(super) fn finished() -> bool {
    let r = runtime();
    r.owner.is_none()
        && r.pending.is_none()
        && r.completed.is_none()
        && !r.resetting
        && !r.quarantined
        && r.reset_acks == 1
}
pub(super) fn print_and_check(tx: u64, rx: u64) {
    let r = runtime();
    assert_eq!(r.tx_submitted, r.tx_completed);
    assert_eq!(tx, r.tx_completed);
    assert_eq!(rx, r.rx_delivered);
    // Only observer expectations differ. Authority checks, owned copy plans,
    // DMA lifetime, finite TX deadlines and reset/quarantine are unchanged.
    if cfg!(feature = "native-system-tcp") {
        assert!(r.tx_completed > 0 && r.rx_delivered > 0 && r.rx_completed >= r.rx_delivered);
        assert_eq!(
            r.denied,
            if cfg!(feature = "native-net-service") {
                2
            } else {
                1
            }
        );
        assert_eq!(r.timeouts, 0); // Transport blackhole is not a DMA timeout.
        assert_eq!(r.resets, 1);
        assert_eq!(r.reset_acks, 1);
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
