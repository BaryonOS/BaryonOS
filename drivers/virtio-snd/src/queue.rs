//! Split-virtqueue bookkeeping, DMA pool layout, fence/epoch lifecycle and the
//! fixture PCM pattern for the kernel-only virtio-snd output fixture
//! (TASK H03-T3a, `TRANSPORT.md` §3-§6, OASIS virtio 1.3 §2.7 and §5.14).
//!
//! Pure logic: no MMIO, no DMA access, no `unsafe`, no heap. The kernel performs
//! every volatile access and passes the values it read from device-writable
//! memory here; this module decides whether they are acceptable. The transport
//! is a trusted QEMU kernel transport without an IOMMU: bounded descriptors into
//! pinned kernel pages are not DMA isolation.

use crate::pcm::{
    CONTROL_STATUS_BYTES, EVENT_BYTES, FRAME_BYTES, MAX_PERIOD_BYTES, SET_PARAMS_BYTES,
    TX_STATUS_BYTES, XFER_BYTES,
};
use crate::MAX_RESPONSE_BYTES;

pub const PAGE: u64 = 4096;
/// Entries per active queue; QEMU 8.2 offers 64, the fixture needs at most 8.
pub const QUEUE_SIZE: u16 = 8;
pub const CONTROLQ: u16 = 0;
pub const EVENTQ: u16 = 1;
pub const TXQ: u16 = 2;
/// Capture queue: never enabled and never given a buffer (B5: disabled).
pub const RXQ: u16 = 3;
/// virtio-snd defines exactly four queues.
pub const QUEUES: u16 = 4;
/// Queues the kernel enables, in ring-page order.
pub const ACTIVE: [u16; 3] = [CONTROLQ, EVENTQ, TXQ];
/// Empty 8-byte event buffers posted before DRIVER_OK (§5.14.5).
pub const EVENT_BUFFERS: u16 = 4;
/// Event completions handled per poll. A device that keeps completing the
/// re-posted event buffers must not hold the single-CPU IF0 kernel in one
/// loop past its TX-progress and drain deadlines.
pub const EVENTS_PER_POLL: u16 = EVENT_BUFFERS;
/// Four TX bounce slots of 2048 bytes: header plus one period each (§4).
pub const TX_SLOTS: u16 = 4;
pub const SLOT_BYTES: u32 = 2048;
/// Largest PCM payload one slot carries behind its 4-byte xfer header.
pub const MAX_SLOT_PCM: u32 = SLOT_BYTES - XFER_BYTES as u32;
/// Config space bytes without `controls` (CTLS is never negotiated).
pub const CONFIG_BYTES: usize = 12;

/// `VIRTQ_DESC_F_NEXT` / `VIRTQ_DESC_F_WRITE`.
pub const F_NEXT: u16 = 1;
pub const F_WRITE: u16 = 2;
/// `VIRTQ_AVAIL_F_NO_INTERRUPT`: the fixture polls with IF=0.
pub const AVAIL_NO_INTERRUPT: u16 = 1;

/// Device status bits (§2.1).
pub const STATUS_ACKNOWLEDGE: u8 = 1;
pub const STATUS_DRIVER: u8 = 2;
pub const STATUS_DRIVER_OK: u8 = 4;
pub const STATUS_FEATURES_OK: u8 = 8;
pub const STATUS_NEEDS_RESET: u8 = 0x40;
pub const STATUS_FAILED: u8 = 0x80;
/// The only status accepted while the device is live.
pub const STATUS_LIVE: u8 =
    STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK | STATUS_DRIVER_OK;

// Ring page: per active queue one 256-byte stride, descriptor table first and
// the available ring at +0x80. Used page: per active queue one used ring.
const QUEUE_STRIDE: u64 = 0x100;
const AVAIL_OFFSET: u64 = 0x80;
const DESC_BYTES: u64 = 16;
const AVAIL_BYTES: u64 = 6 + 2 * QUEUE_SIZE as u64;
const USED_BYTES: u64 = 6 + 8 * QUEUE_SIZE as u64;
// Response page: control response, event buffers, TX status. These are the only
// device-writable data bytes; everything else the device reads.
const RESPONSE_AREA: u64 = 0x800;
const EVENT_OFFSET: u64 = 0x800;
const TX_STATUS_OFFSET: u64 = 0xc00;
const SLOTS_PER_PAGE: u16 = (PAGE / SLOT_BYTES as u64) as u16;
const PCM_PAGES: usize = (TX_SLOTS / SLOTS_PER_PAGE) as usize;

const _: () = assert!(QUEUE_SIZE.is_power_of_two());
const _: () = assert!(DESC_BYTES * QUEUE_SIZE as u64 <= AVAIL_OFFSET);
const _: () = assert!(AVAIL_OFFSET + AVAIL_BYTES <= QUEUE_STRIDE);
const _: () = assert!(USED_BYTES <= QUEUE_STRIDE);
const _: () = assert!(QUEUE_STRIDE * ACTIVE.len() as u64 <= PAGE);
const _: () = assert!(MAX_RESPONSE_BYTES as u64 <= RESPONSE_AREA);
const _: () = assert!(EVENT_OFFSET + EVENT_BUFFERS as u64 * EVENT_BYTES as u64 <= TX_STATUS_OFFSET);
const _: () = assert!(TX_STATUS_OFFSET + TX_SLOTS as u64 * TX_STATUS_BYTES as u64 <= PAGE);
const _: () = assert!(EVENT_BUFFERS <= QUEUE_SIZE && 2 * TX_SLOTS <= QUEUE_SIZE);
const _: () = assert!(TX_SLOTS.is_multiple_of(SLOTS_PER_PAGE) && PCM_PAGES == 2);
const _: () = assert!(FIXTURE_PERIOD_BYTES <= MAX_SLOT_PCM);
// One full period must always fit one TX slot behind its xfer header, so a
// period that `pcm::Stream::set_params` accepts can always be sent whole.
const _: () = assert!(MAX_PERIOD_BYTES == MAX_SLOT_PCM);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// A driver-side argument is outside its bound; nothing was published.
    Range,
    /// The descriptor head is already outstanding (or the ring is full).
    Busy,
    /// Used element names a descriptor head that is not outstanding.
    UnknownId,
    /// Used length differs from the exact device-writable length.
    Length,
    /// The used index moved by more than the outstanding requests, which
    /// includes any movement while the queue is idle.
    UsedOverrun,
    /// The configuration generation never stayed stable across a read.
    Unstable,
    /// The lifecycle does not allow this step now.
    State,
    /// Terminal until reboot: the pool stays pinned, the device offline.
    Quarantined,
}

impl Error {
    /// Device-delivered values violated the contract; the transport fences.
    pub const fn requires_fence(self) -> bool {
        matches!(self, Self::UnknownId | Self::Length | Self::UsedOverrun)
    }
}

/// Six exclusive, pinned, identity-mapped kernel RAM pages. Descriptors are
/// only ever derived from these addresses and bounded offsets inside them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pool {
    rings: u64,
    used: u64,
    requests: u64,
    responses: u64,
    pcm: [u64; PCM_PAGES],
}

impl Pool {
    pub const PAGES: usize = 4 + PCM_PAGES;

    /// Pages in order: rings, used, requests, responses, PCM slots.
    pub fn new(pages: [u64; Self::PAGES], ram_end: u64) -> Result<Self, Error> {
        for (index, page) in pages.iter().enumerate() {
            if *page == 0
                || !page.is_multiple_of(PAGE)
                || page.checked_add(PAGE).is_none_or(|end| end > ram_end)
                || pages[..index].contains(page)
            {
                return Err(Error::Range);
            }
        }
        Ok(Self {
            rings: pages[0],
            used: pages[1],
            requests: pages[2],
            responses: pages[3],
            pcm: [pages[4], pages[5]],
        })
    }

    pub const fn pages(&self) -> [u64; Self::PAGES] {
        [
            self.rings,
            self.used,
            self.requests,
            self.responses,
            self.pcm[0],
            self.pcm[1],
        ]
    }

    /// Pages the device may write: the used rings and the response page.
    pub const fn device_writable_pages(&self) -> [u64; 2] {
        [self.used, self.responses]
    }

    fn stride(queue: u16) -> Result<u64, Error> {
        match queue {
            CONTROLQ | EVENTQ | TXQ => Ok(u64::from(queue) * QUEUE_STRIDE),
            _ => Err(Error::Range),
        }
    }

    pub fn descriptors(&self, queue: u16) -> Result<u64, Error> {
        Ok(self.rings + Self::stride(queue)?)
    }

    pub fn available(&self, queue: u16) -> Result<u64, Error> {
        Ok(self.rings + Self::stride(queue)? + AVAIL_OFFSET)
    }

    pub fn used(&self, queue: u16) -> Result<u64, Error> {
        Ok(self.used + Self::stride(queue)?)
    }

    pub const fn used_page(&self) -> u64 {
        self.used
    }

    /// Byte offset of a queue's used index inside the used page.
    pub fn used_index_offset(queue: u16) -> Result<u64, Error> {
        Ok(Self::stride(queue)? + 2)
    }

    pub const fn request(&self) -> u64 {
        self.requests
    }

    pub const fn response(&self) -> u64 {
        self.responses
    }

    pub fn event(&self, buffer: u16) -> Result<u64, Error> {
        if buffer >= EVENT_BUFFERS {
            return Err(Error::Range);
        }
        Ok(self.responses + EVENT_OFFSET + u64::from(buffer) * EVENT_BYTES as u64)
    }

    pub fn tx_status(&self, slot: u16) -> Result<u64, Error> {
        if slot >= TX_SLOTS {
            return Err(Error::Range);
        }
        Ok(self.responses + TX_STATUS_OFFSET + u64::from(slot) * TX_STATUS_BYTES as u64)
    }

    /// Device-readable slot: 4-byte xfer header followed by the PCM payload.
    pub fn tx_slot(&self, slot: u16) -> Result<u64, Error> {
        if slot >= TX_SLOTS {
            return Err(Error::Range);
        }
        let page = self.pcm[usize::from(slot / SLOTS_PER_PAGE)];
        Ok(page + u64::from(slot % SLOTS_PER_PAGE) * u64::from(SLOT_BYTES))
    }
}

/// One split-virtqueue descriptor (§2.7.5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Descriptor {
    pub address: u64,
    pub length: u32,
    pub flags: u16,
    pub next: u16,
}

impl Descriptor {
    pub fn encode(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        bytes[..8].copy_from_slice(&self.address.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.length.to_le_bytes());
        bytes[12..14].copy_from_slice(&self.flags.to_le_bytes());
        bytes[14..].copy_from_slice(&self.next.to_le_bytes());
        bytes
    }
}

/// A descriptor chain with its table positions, head and the exact used
/// length the device must report for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Chain<const N: usize> {
    pub head: u16,
    pub used_len: u32,
    /// A second accepted used length, 0 when `used_len` is the only one.
    pub alternate_used_len: u32,
    /// F17: a used length below `used_len` still completes this head, as an
    /// invalid (short) answer the caller must count against its event budget.
    /// Only event buffers set it; a longer used length is always `Length`.
    pub accept_short: bool,
    pub entries: [(u16, Descriptor); N],
}

/// Control request (device-readable) plus its exact response buffer.
pub fn control_chain(
    pool: &Pool,
    request_len: usize,
    response_len: usize,
) -> Result<Chain<2>, Error> {
    if request_len == 0
        || request_len > SET_PARAMS_BYTES
        || response_len == 0
        || response_len > MAX_RESPONSE_BYTES
    {
        return Err(Error::Range);
    }
    let request_len = u32::try_from(request_len).map_err(|_| Error::Range)?;
    let response_len = u32::try_from(response_len).map_err(|_| Error::Range)?;
    Ok(Chain {
        head: 0,
        used_len: response_len,
        alternate_used_len: 0,
        accept_short: false,
        entries: [
            (
                0,
                Descriptor {
                    address: pool.request(),
                    length: request_len,
                    flags: F_NEXT,
                    next: 1,
                },
            ),
            (
                1,
                Descriptor {
                    address: pool.response(),
                    length: response_len,
                    flags: F_WRITE,
                    next: 0,
                },
            ),
        ],
    })
}

/// PCM_INFO request with its `4 + 32 * count` response buffer. Observed QEMU
/// 8.2.2 quirk (H03-T3a evidence): the device writes the records but reports
/// only `sizeof(virtio_snd_hdr)` = 4 as used length. That single header-only
/// length is accepted in addition to the exact one; every other length still
/// fences. The caller zeroes the response before publication and parses the
/// records strictly, so an answer that really carries only the header fails
/// closed (`channels_min` 0 is invalid). Other messages get no such exception.
pub fn pcm_info_chain(
    pool: &Pool,
    request_len: usize,
    response_len: usize,
) -> Result<Chain<2>, Error> {
    let mut chain = control_chain(pool, request_len, response_len)?;
    if response_len > CONTROL_STATUS_BYTES {
        chain.alternate_used_len = CONTROL_STATUS_BYTES as u32;
    }
    Ok(chain)
}

/// One empty, device-writable 8-byte event buffer; its head is `buffer`. A
/// used length below 8 completes it as an invalid event (F17: ignored and
/// counted by the caller); a used length above 8 is a used-ring violation.
pub fn event_chain(pool: &Pool, buffer: u16) -> Result<Chain<1>, Error> {
    Ok(Chain {
        head: buffer,
        used_len: EVENT_BYTES as u32,
        alternate_used_len: 0,
        accept_short: true,
        entries: [(
            buffer,
            Descriptor {
                address: pool.event(buffer)?,
                length: EVENT_BYTES as u32,
                flags: F_WRITE,
                next: 0,
            },
        )],
    })
}

/// TX message for `slot`: one device-readable buffer with the xfer header and
/// `pcm_len` PCM bytes, then the device-writable 8-byte status. Head `2*slot`.
pub fn tx_chain(pool: &Pool, slot: u16, pcm_len: usize) -> Result<Chain<2>, Error> {
    let pcm_len = u32::try_from(pcm_len).map_err(|_| Error::Range)?;
    if pcm_len == 0 || pcm_len > MAX_SLOT_PCM || !pcm_len.is_multiple_of(FRAME_BYTES) {
        return Err(Error::Range);
    }
    let address = pool.tx_slot(slot)?;
    let status = pool.tx_status(slot)?;
    let head = slot * 2;
    Ok(Chain {
        head,
        used_len: TX_STATUS_BYTES as u32,
        alternate_used_len: 0,
        accept_short: false,
        entries: [
            (
                head,
                Descriptor {
                    address,
                    length: XFER_BYTES as u32 + pcm_len,
                    flags: F_NEXT,
                    next: head + 1,
                },
            ),
            (
                head + 1,
                Descriptor {
                    address: status,
                    length: TX_STATUS_BYTES as u32,
                    flags: F_WRITE,
                    next: 0,
                },
            ),
        ],
    })
}

/// Where one publication goes: available-ring entry and the new index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Publication {
    pub entry: u16,
    pub index: u16,
}

/// Driver-side state of one split virtqueue. Each outstanding head carries
/// the exact used length it may complete with (plus at most one explicitly
/// published alternate, and for event buffers only a shorter, counted
/// answer); nothing else is accepted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ring {
    next_available: u16,
    last_used: u16,
    expected: [(u32, u32); QUEUE_SIZE as usize],
    short: [bool; QUEUE_SIZE as usize],
    outstanding: u16,
}

impl Default for Ring {
    fn default() -> Self {
        Self::new()
    }
}

impl Ring {
    pub const fn new() -> Self {
        Self {
            next_available: 0,
            last_used: 0,
            expected: [(0, 0); QUEUE_SIZE as usize],
            short: [false; QUEUE_SIZE as usize],
            outstanding: 0,
        }
    }

    pub const fn outstanding(&self) -> u16 {
        self.outstanding
    }

    /// Available index the driver has published so far.
    pub const fn available_index(&self) -> u16 {
        self.next_available
    }

    /// Used index the driver has consumed so far.
    pub const fn used_index(&self) -> u16 {
        self.last_used
    }

    /// Used-ring entry of the next element to read.
    pub const fn used_entry(&self) -> u16 {
        self.last_used % QUEUE_SIZE
    }

    /// Records that `head` becomes visible to the device.
    pub fn publish(&mut self, head: u16, used_len: u32) -> Result<Publication, Error> {
        self.publish_chain(head, used_len, 0)
    }

    /// As [`Ring::publish`], with one alternate used length (0 = none). Never
    /// accepts a short answer.
    pub fn publish_chain(
        &mut self,
        head: u16,
        used_len: u32,
        alternate: u32,
    ) -> Result<Publication, Error> {
        self.record(head, used_len, alternate, false)
    }

    /// The kernel's only publication path. F8 before publication:
    /// `device_index`, read from the used ring right before this call, is
    /// checked against the requests outstanding **before** this one, so a used
    /// index that moved while the queue was idle, or by more than what is
    /// already outstanding, is `UsedOverrun` and the ring stays unchanged.
    /// Only then is `chain` recorded with its used lengths and short policy.
    pub fn publish_checked<const N: usize>(
        &mut self,
        device_index: u16,
        chain: &Chain<N>,
    ) -> Result<Publication, Error> {
        self.pending(device_index)?;
        self.record(
            chain.head,
            chain.used_len,
            chain.alternate_used_len,
            chain.accept_short,
        )
    }

    fn record(
        &mut self,
        head: u16,
        used_len: u32,
        alternate: u32,
        short: bool,
    ) -> Result<Publication, Error> {
        let index = usize::from(head);
        let (Some(slot), Some(short_slot)) =
            (self.expected.get(index).copied(), self.short.get_mut(index))
        else {
            return Err(Error::Range);
        };
        if used_len == 0 || alternate == used_len {
            return Err(Error::Range);
        }
        if slot.0 != 0 || self.outstanding >= QUEUE_SIZE {
            return Err(Error::Busy);
        }
        *short_slot = short;
        self.expected[index] = (used_len, alternate);
        self.outstanding += 1;
        let entry = self.next_available % QUEUE_SIZE;
        self.next_available = self.next_available.wrapping_add(1);
        Ok(Publication {
            entry,
            index: self.next_available,
        })
    }

    /// Number of new used elements for a device-written used index. A move
    /// beyond the outstanding requests (also while idle) is a violation.
    pub fn pending(&self, device_index: u16) -> Result<u16, Error> {
        let delta = device_index.wrapping_sub(self.last_used);
        if delta > self.outstanding {
            return Err(Error::UsedOverrun);
        }
        Ok(delta)
    }

    /// Consumes one used element; returns its validated head. A head
    /// published with `accept_short` also completes with a used length below
    /// its expected one; the caller compares the length and counts it (F17).
    /// On any error the ring is unchanged.
    pub fn complete(&mut self, id: u32, used_len: u32) -> Result<u16, Error> {
        let head = u16::try_from(id).map_err(|_| Error::UnknownId)?;
        let index = usize::from(head);
        let (Some(slot), Some(short_slot)) =
            (self.expected.get_mut(index), self.short.get_mut(index))
        else {
            return Err(Error::UnknownId);
        };
        if slot.0 == 0 {
            return Err(Error::UnknownId);
        }
        let exact = slot.0 == used_len || (slot.1 != 0 && slot.1 == used_len);
        let short = *short_slot && used_len < slot.0;
        if !exact && !short {
            return Err(Error::Length);
        }
        *slot = (0, 0);
        *short_slot = false;
        self.outstanding -= 1;
        self.last_used = self.last_used.wrapping_add(1);
        Ok(head)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// Not initialised (also after an acknowledged fence).
    Offline,
    Live,
    /// Publication frozen; status 0 and BME off not yet both acknowledged.
    Fencing,
    /// Both acknowledged; the used rings are being watched for stability.
    Acknowledged,
    Quarantined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FenceStep {
    Pending,
    Acknowledged,
    Quarantined,
}

/// Device lifecycle: publication only while `Live`; a fence is status 0 read
/// back **and** bus mastering off read back before a deadline, a used-ring
/// baseline bracketed around that acknowledging read-back, and a stability
/// window ended by a final matching observation. Only then does the epoch
/// advance (checked).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Lifecycle {
    phase: Phase,
    epoch: u64,
    deadline: u64,
    snapshot: Option<[u16; 3]>,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl Lifecycle {
    pub const fn new() -> Self {
        Self {
            phase: Phase::Offline,
            epoch: 0,
            deadline: 0,
            snapshot: None,
        }
    }

    #[cfg(test)]
    pub(crate) const fn with_epoch(epoch: u64) -> Self {
        Self {
            phase: Phase::Offline,
            epoch,
            deadline: 0,
            snapshot: None,
        }
    }

    pub const fn phase(&self) -> Phase {
        self.phase
    }

    pub const fn epoch(&self) -> u64 {
        self.epoch
    }

    pub const fn may_publish(&self) -> bool {
        matches!(self.phase, Phase::Live)
    }

    /// Offline -> Live after DRIVER_OK was read back.
    pub fn go_live(&mut self) -> Result<(), Error> {
        match self.phase {
            Phase::Offline => {
                self.phase = Phase::Live;
                Ok(())
            }
            Phase::Quarantined => Err(Error::Quarantined),
            _ => Err(Error::State),
        }
    }

    /// Freezes publication before reset. `budget == 0` can never acknowledge.
    pub fn begin_fence(&mut self, now: u64, budget: u64) -> Result<(), Error> {
        match self.phase {
            Phase::Offline | Phase::Live => {}
            Phase::Quarantined => return Err(Error::Quarantined),
            _ => return Err(Error::State),
        }
        self.snapshot = None;
        match now.checked_add(budget) {
            Some(deadline) => {
                self.deadline = deadline;
                self.phase = Phase::Fencing;
                Ok(())
            }
            None => {
                self.phase = Phase::Quarantined;
                Err(Error::Quarantined)
            }
        }
    }

    /// One read-back observation at `now`. Acknowledgement requires both
    /// halves strictly before the deadline; a miss quarantines.
    pub fn poll_fence(&mut self, now: u64, status: u8, bus_master: bool) -> FenceStep {
        match self.phase {
            Phase::Fencing => {}
            Phase::Acknowledged => return FenceStep::Acknowledged,
            Phase::Quarantined => return FenceStep::Quarantined,
            Phase::Offline | Phase::Live => {
                self.phase = Phase::Quarantined;
                return FenceStep::Quarantined;
            }
        }
        if now >= self.deadline {
            self.phase = Phase::Quarantined;
            return FenceStep::Quarantined;
        }
        if status == 0 && !bus_master {
            self.phase = Phase::Acknowledged;
            return FenceStep::Acknowledged;
        }
        FenceStep::Pending
    }

    /// Baseline after both acknowledgements. `before` is sampled before the
    /// status/BME read-back that acknowledged and `after` right after it, so
    /// the baseline cannot absorb a completion that raced the acknowledgement:
    /// any difference is a completion after acknowledgement (F11) and
    /// quarantines, as does a wrong phase or a second baseline.
    pub fn record_snapshot(&mut self, before: [u16; 3], after: [u16; 3]) -> Result<(), Error> {
        match (self.phase, self.snapshot) {
            (Phase::Acknowledged, None) if before == after => {
                self.snapshot = Some(before);
                Ok(())
            }
            (Phase::Quarantined, _) => Err(Error::Quarantined),
            _ => {
                self.phase = Phase::Quarantined;
                Err(Error::Quarantined)
            }
        }
    }

    /// Any later change of a used index, status or bus master is a
    /// completion after the fence (F11): quarantine, never a selective release.
    pub fn check_snapshot(
        &mut self,
        used: [u16; 3],
        status: u8,
        bus_master: bool,
    ) -> Result<(), Error> {
        match (self.phase, self.snapshot) {
            (Phase::Acknowledged, Some(seen)) if seen == used && status == 0 && !bus_master => {
                Ok(())
            }
            (Phase::Quarantined, _) => Err(Error::Quarantined),
            _ => {
                self.phase = Phase::Quarantined;
                Err(Error::Quarantined)
            }
        }
    }

    /// Ends a stable fence: epoch + 1 (overflow quarantines), back to Offline.
    /// The epoch advances only when a final observation, taken after the
    /// stability window, still equals the baseline with status 0 and BME off;
    /// otherwise, or without a baseline, the device is quarantined.
    pub fn finish_fence(
        &mut self,
        used: [u16; 3],
        status: u8,
        bus_master: bool,
    ) -> Result<u64, Error> {
        self.check_snapshot(used, status, bus_master)?;
        match self.epoch.checked_add(1) {
            Some(epoch) => {
                self.epoch = epoch;
                self.snapshot = None;
                self.phase = Phase::Offline;
                Ok(epoch)
            }
            None => {
                self.phase = Phase::Quarantined;
                Err(Error::Quarantined)
            }
        }
    }

    pub fn quarantine(&mut self) {
        self.phase = Phase::Quarantined;
    }
}

/// Calls `step` until it reports that nothing more is pending, at most `limit`
/// times, and returns how many elements it handled. Anything left is handled
/// by the next poll, after the caller's own deadline checks.
pub fn bounded_poll<E>(limit: u16, mut step: impl FnMut() -> Result<bool, E>) -> Result<u16, E> {
    let mut handled = 0;
    while handled < limit && step()? {
        handled += 1;
    }
    Ok(handled)
}

/// Generation-stable config read (§4.1.4.3.1): `read` returns the generation
/// before, the 12 config bytes and the generation after. Bounded retries.
pub fn stable_config<F>(mut read: F, tries: u32) -> Result<[u8; CONFIG_BYTES], Error>
where
    F: FnMut() -> (u8, [u8; CONFIG_BYTES], u8),
{
    for _ in 0..tries {
        let (before, bytes, after) = read();
        if before == after {
            return Ok(bytes);
        }
    }
    Err(Error::Unstable)
}

/// Fixture: 1 s of stereo S16 at 48 kHz, in 10 ms periods (`TRANSPORT.md` §7).
pub const FIXTURE_FRAMES: u32 = 48_000;
pub const FIXTURE_PERIOD_FRAMES: u32 = 480;
pub const FIXTURE_PERIOD_BYTES: u32 = FIXTURE_PERIOD_FRAMES * FRAME_BYTES;
pub const FIXTURE_PERIODS: u32 = 4;
pub const FIXTURE_TOTAL_PERIODS: u32 = FIXTURE_FRAMES / FIXTURE_PERIOD_FRAMES;
const _: () = assert!(FIXTURE_FRAMES.is_multiple_of(FIXTURE_PERIOD_FRAMES));

/// Deterministic fixture patterns. Pattern A is the frozen `TRANSPORT.md` §7
/// oracle (`L = n`, `R = n ^ 0x8000`); pattern B is the post-fence pattern
/// (`L = !n`, `R = !n ^ 0x2000`). `L ^ R` separates A (0x8000), B (0x2000) and
/// silence (0), so no frame of one can be mistaken for the other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pattern {
    A,
    B,
}

impl Pattern {
    /// Little-endian `L, R` for frame `n` (`n` is taken modulo 65536).
    pub const fn frame(self, n: u32) -> [u8; FRAME_BYTES as usize] {
        let x = (n & 0xffff) as u16;
        let (left, right) = match self {
            Self::A => (x, x ^ 0x8000),
            Self::B => (!x, !x ^ 0x2000),
        };
        let l = left.to_le_bytes();
        let r = right.to_le_bytes();
        [l[0], l[1], r[0], r[1]]
    }
}

#[cfg(test)]
#[path = "queue_tests.rs"]
mod tests;
