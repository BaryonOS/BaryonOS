#![no_std]
#![forbid(unsafe_code)]
//! Pure M5/M6 policy: integer PCI snapshots, fixed DMA plans and request lifecycle.
//! Hardware ordering, DMA pinning, MMIO and actual reset acknowledgement are the
//! caller's responsibility. No policy result proves hardware quiescence.

pub const QUEUE_SIZE: u16 = 8;
pub const SECTOR_SIZE: u32 = 512;
pub const MAX_SLICE: u64 = 128;
pub const VERSION_1: u64 = 1 << 32;
pub const ACCESS_PLATFORM: u64 = 1 << 33;
pub const READ_ONLY: u64 = 1 << 5;
pub const FLUSH: u64 = 1 << 9;
pub const NEXT: u16 = 1;
pub const WRITE: u16 = 2;
pub const PHYSICAL_LIMIT: u64 = 1 << 52;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Unsupported,
    Range,
    Busy,
    Pending,
    WrongTicket,
    StaleEpoch,
    Protocol,
    Io,
    Offline,
    Timeout,
    Exhausted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completion {
    Success,
    Io,
    Unsupported,
    /// Software outcome: a published mutation may have changed storage. Never
    /// produced by the used-ring decoder, and never an authorization to replay.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Read {
        sector: u64,
        offset: u64,
        length: u64,
    },
    Write {
        sector: u64,
    },
    Flush,
}
impl Operation {
    pub const fn is_mutating(self) -> bool {
        !matches!(self, Self::Read { .. })
    }
    pub const fn sector(self) -> u64 {
        match self {
            Self::Read { sector, .. } | Self::Write { sector } => sector,
            Self::Flush => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bar {
    pub base: u64,
    pub size: u64,
    pub is_memory: bool,
}
impl Bar {
    pub const NONE: Self = Self {
        base: 0,
        size: 0,
        is_memory: false,
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub bar: u8,
    pub offset: u32,
    pub length: u32,
}
impl Region {
    pub fn address(self, bars: &[Bar; 6]) -> Result<u64, Error> {
        let b = bars.get(self.bar as usize).ok_or(Error::Range)?;
        if !b.is_memory
            || b.size == 0
            || b.base
                .checked_add(b.size)
                .is_none_or(|end| end > PHYSICAL_LIMIT)
            || self.length == 0
            || u64::from(self.offset)
                .checked_add(u64::from(self.length))
                .ok_or(Error::Range)?
                > b.size
        {
            return Err(Error::Range);
        }
        b.base
            .checked_add(u64::from(self.offset))
            .ok_or(Error::Range)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Regions {
    pub common: Region,
    pub notify: Region,
    pub isr: Region,
    pub device: Region,
    pub notify_multiplier: u32,
}
impl Regions {
    pub fn notify_address(self, bars: &[Bar; 6], queue_notify_off: u16) -> Result<u64, Error> {
        let offset = u64::from(queue_notify_off)
            .checked_mul(u64::from(self.notify_multiplier))
            .ok_or(Error::Range)?;
        if offset.checked_add(2).ok_or(Error::Range)? > u64::from(self.notify.length) {
            return Err(Error::Range);
        }
        let address = self
            .notify
            .address(bars)?
            .checked_add(offset)
            .ok_or(Error::Range)?;
        if address % 2 != 0 {
            return Err(Error::Range);
        }
        Ok(address)
    }
}
fn r16(b: &[u8; 256], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}
fn r32(b: &[u8; 256], i: usize) -> u32 {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}
fn checked_bar(config: &[u8; 256], bars: &[Bar; 6], index: u8) -> Result<(), Error> {
    if index >= 6 {
        return Err(Error::Range);
    }
    let mut i = 0;
    while i < index as usize {
        let low = r32(config, 0x10 + i * 4);
        let wide = low & 1 == 0 && low & 6 == 4;
        if wide && i + 1 == index as usize {
            return Err(Error::Invalid);
        }
        i += if wide { 2 } else { 1 };
    }
    let b = bars[index as usize];
    let low = r32(config, 0x10 + index as usize * 4);
    if low & 1 != 0
        || !b.is_memory
        || b.size == 0
        || !b.size.is_power_of_two()
        || b.base == 0
        || b.base & (b.size - 1) != 0
    {
        return Err(Error::Invalid);
    }
    let base = match low & 6 {
        0 => u64::from(low & !15),
        4 if index < 5 => {
            u64::from(low & !15) | (u64::from(r32(config, 0x14 + index as usize * 4)) << 32)
        }
        _ => return Err(Error::Unsupported),
    };
    if base != b.base
        || b.base
            .checked_add(b.size)
            .is_none_or(|end| end > PHYSICAL_LIMIT)
    {
        return Err(Error::Range);
    }
    Ok(())
}
/// Strict modern virtio-blk PCI profile, fixed header type zero. Discovery/BDF
/// selection and obtaining BAR size without enabling DMA belong to the kernel.
/// Modern virtio-pci device IDs (0x1040 + virtio device type).
pub const DEVICE_ID_BLOCK: u16 = 0x1042;
pub const DEVICE_ID_ENTROPY: u16 = 0x1044;
pub fn parse_capabilities(config: &[u8; 256], bars: &[Bar; 6]) -> Result<Regions, Error> {
    parse_capabilities_for(config, bars, DEVICE_ID_BLOCK)
}
/// Same capability walk for another modern virtio device type. The device
/// configuration region stays mandatory: every QEMU virtio-pci function
/// exposes one, even for types without device-specific fields.
pub fn parse_capabilities_for(
    config: &[u8; 256],
    bars: &[Bar; 6],
    device_id: u16,
) -> Result<Regions, Error> {
    if r16(config, 0) != 0x1af4
        || r16(config, 2) != device_id
        || config[0x0e] & 0x7f != 0
        || r16(config, 6) & 0x10 == 0
    {
        return Err(Error::Unsupported);
    }
    let mut found = [None; 4];
    let mut multiplier = 0;
    let mut seen = [false; 64];
    let mut at = config[0x34] as usize;
    while at != 0 {
        if !(0x40..=0xfc).contains(&at) || !at.is_multiple_of(4) || seen[at / 4] {
            return Err(Error::Protocol);
        }
        seen[at / 4] = true;
        if matches!(config[at], 0 | 0xff) {
            return Err(Error::Protocol);
        }
        let next = config[at + 1] as usize;
        if config[at] == 9 {
            let len = config[at + 2] as usize;
            if len < 4 || at.checked_add(len).ok_or(Error::Range)? > 256 {
                return Err(Error::Range);
            }
            let kind = config[at + 3];
            if (1..=4).contains(&kind) {
                if len < 16 || kind == 2 && len < 20 {
                    return Err(Error::Range);
                }
                let region = Region {
                    bar: config[at + 4],
                    offset: r32(config, at + 8),
                    length: r32(config, at + 12),
                };
                checked_bar(config, bars, region.bar)?;
                region.address(bars)?;
                let minimum = match kind {
                    1 => 56,
                    2 => 2,
                    3 => 1,
                    _ => 8,
                };
                let alignment = match kind {
                    1 | 4 => 8,
                    2 => 2,
                    _ => 1,
                };
                if region.length < minimum
                    || !region.offset.is_multiple_of(alignment)
                    || found[kind as usize - 1].is_some()
                {
                    return Err(Error::Protocol);
                }
                found[kind as usize - 1] = Some(region);
                if kind == 2 {
                    multiplier = r32(config, at + 16);
                }
            }
        }
        at = next;
    }
    let parts = [
        found[0].ok_or(Error::Unsupported)?,
        found[1].ok_or(Error::Unsupported)?,
        found[2].ok_or(Error::Unsupported)?,
        found[3].ok_or(Error::Unsupported)?,
    ];
    for (i, a) in parts.iter().enumerate() {
        let start = a.address(bars)?;
        let end = start.checked_add(u64::from(a.length)).ok_or(Error::Range)?;
        for b in &parts[..i] {
            let other = b.address(bars)?;
            let other_end = other.checked_add(u64::from(b.length)).ok_or(Error::Range)?;
            if start < other_end && other < end {
                return Err(Error::Protocol);
            }
        }
    }
    Ok(Regions {
        common: parts[0],
        notify: parts[1],
        isr: parts[2],
        device: parts[3],
        notify_multiplier: multiplier,
    })
}
pub fn negotiate_features(offered: u64) -> Result<u64, Error> {
    if offered & VERSION_1 == 0 || offered & ACCESS_PLATFORM != 0 || offered & READ_ONLY == 0 {
        return Err(Error::Unsupported);
    }
    Ok(VERSION_1 | READ_ONLY)
}
/// Separate opt-in profile: never silently weakens the readonly M5 contract.
pub fn negotiate_writable_features(offered: u64) -> Result<u64, Error> {
    if offered & (VERSION_1 | FLUSH) != (VERSION_1 | FLUSH)
        || offered & (READ_ONLY | ACCESS_PLATFORM) != 0
    {
        return Err(Error::Unsupported);
    }
    Ok(VERSION_1 | FLUSH)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub address: u64,
    pub length: u32,
    pub flags: u16,
    pub next: u16,
}
impl Descriptor {
    pub fn encode(self) -> [u8; 16] {
        let mut b = [0; 16];
        b[..8].copy_from_slice(&self.address.to_le_bytes());
        b[8..12].copy_from_slice(&self.length.to_le_bytes());
        b[12..14].copy_from_slice(&self.flags.to_le_bytes());
        b[14..].copy_from_slice(&self.next.to_le_bytes());
        b
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DmaPool {
    pub descriptors: u64,
    pub available: u64,
    pub used: u64,
    pub request: u64,
    pub bounce: u64,
}
impl DmaPool {
    pub fn validate(self, ram_end: u64) -> Result<(), Error> {
        if ram_end > PHYSICAL_LIMIT {
            return Err(Error::Range);
        }
        let pages = [
            self.descriptors,
            self.available,
            self.used,
            self.request,
            self.bounce,
        ];
        for (i, page) in pages.iter().enumerate() {
            if *page == 0
                || page % 4096 != 0
                || page.checked_add(4096).ok_or(Error::Range)? > ram_end
                || pages[..i].contains(page)
            {
                return Err(Error::Range);
            }
        }
        Ok(())
    }
    pub fn status_address(self) -> Result<u64, Error> {
        self.request.checked_add(16).ok_or(Error::Range)
    }
}
pub fn descriptor_plan(pool: DmaPool, ram_end: u64) -> Result<[Descriptor; 3], Error> {
    pool.validate(ram_end)?;
    Ok([
        Descriptor {
            address: pool.request,
            length: 16,
            flags: NEXT,
            next: 1,
        },
        Descriptor {
            address: pool.bounce,
            length: 512,
            flags: NEXT | WRITE,
            next: 2,
        },
        Descriptor {
            address: pool.status_address()?,
            length: 1,
            flags: WRITE,
            next: 0,
        },
    ])
}
pub fn validate_descriptors(
    descriptors: &[Descriptor; 3],
    pool: DmaPool,
    ram_end: u64,
) -> Result<(), Error> {
    if *descriptors != descriptor_plan(pool, ram_end)? {
        return Err(Error::Protocol);
    }
    Ok(())
}
pub fn read_header(sector: u64) -> [u8; 16] {
    header_for(Operation::Read {
        sector,
        offset: 0,
        length: MAX_SLICE,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DescriptorChain {
    pub entries: [Descriptor; 3],
    pub count: u8,
}
pub fn descriptor_plan_for(
    operation: Operation,
    pool: DmaPool,
    ram_end: u64,
) -> Result<DescriptorChain, Error> {
    let mut entries = descriptor_plan(pool, ram_end)?;
    match operation {
        Operation::Read { .. } => {}
        Operation::Write { .. } => entries[1].flags = NEXT,
        Operation::Flush => {
            entries[1] = entries[2];
            entries[2] = Descriptor {
                address: 0,
                length: 0,
                flags: 0,
                next: 0,
            };
        }
    }
    Ok(DescriptorChain {
        entries,
        count: if operation == Operation::Flush { 2 } else { 3 },
    })
}
pub fn validate_descriptors_for(
    chain: &DescriptorChain,
    operation: Operation,
    pool: DmaPool,
    ram_end: u64,
) -> Result<(), Error> {
    if *chain != descriptor_plan_for(operation, pool, ram_end)? {
        return Err(Error::Protocol);
    }
    Ok(())
}
pub fn header_for(operation: Operation) -> [u8; 16] {
    let mut b = [0; 16];
    let kind: u32 = match operation {
        Operation::Read { .. } => 0,
        Operation::Write { .. } => 1,
        Operation::Flush => 4,
    };
    b[..4].copy_from_slice(&kind.to_le_bytes());
    b[8..].copy_from_slice(&operation.sector().to_le_bytes());
    b
}
/// A status failure never authorizes payload exposure. A short successful used
/// length is rejected rather than assuming bytes beyond the reported length.
pub fn validate_used(
    expected_head: u16,
    old_idx: u16,
    new_idx: u16,
    used_id: u32,
    used_len: u32,
    status: u8,
) -> Result<Completion, Error> {
    validate_used_for(
        Operation::Read {
            sector: 0,
            offset: 0,
            length: MAX_SLICE,
        },
        expected_head,
        old_idx,
        new_idx,
        used_id,
        used_len,
        status,
    )
}
#[allow(clippy::too_many_arguments)]
pub fn validate_used_for(
    operation: Operation,
    expected_head: u16,
    old_idx: u16,
    new_idx: u16,
    used_id: u32,
    used_len: u32,
    status: u8,
) -> Result<Completion, Error> {
    let delta = new_idx.wrapping_sub(old_idx);
    if delta == 0 {
        return Err(Error::Pending);
    }
    if delta != 1
        || expected_head >= QUEUE_SIZE
        || used_id != u32::from(expected_head)
        || used_len > if operation.is_mutating() { 1 } else { 513 }
    {
        return Err(Error::Protocol);
    }
    match status {
        0 if used_len == if operation.is_mutating() { 1 } else { 513 } => Ok(Completion::Success),
        1 if used_len >= 1 => Ok(Completion::Io),
        2 if used_len >= 1 => Ok(Completion::Unsupported),
        _ => Err(Error::Protocol),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub pid: u64,
    pub object_key: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Read {
    pub sector: u64,
    pub offset: u64,
    pub length: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub binding: Binding,
    pub ticket: u64,
    pub epoch: u64,
    pub sector: u64,
    pub offset: u16,
    pub length: u16,
    pub deadline: u64,
    pub completion: Option<Completion>,
    pub operation: Operation,
    pub published: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Offline,
    Ready,
    Prepared,
    Published,
    Pending,
    Complete,
    Resetting,
    Quarantined,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    phase: Phase,
    epoch: u64,
    next_ticket: u64,
    pending: Option<Request>,
}
impl Default for Policy {
    fn default() -> Self {
        Self::new()
    }
}
impl Policy {
    pub const fn new() -> Self {
        Self {
            phase: Phase::Offline,
            epoch: 0,
            next_ticket: 1,
            pending: None,
        }
    }
    pub const fn phase(&self) -> Phase {
        self.phase
    }
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }
    pub const fn request(&self) -> Option<Request> {
        self.pending
    }
    /// Call only after an actual transport reset was requested/acknowledged.
    /// False acknowledgement permanently quarantines this policy until reboot.
    pub fn reset_completed(&mut self, acknowledged: bool) -> Result<(), Error> {
        if self.phase != Phase::Offline && self.phase != Phase::Resetting {
            return Err(Error::Offline);
        }
        if !acknowledged {
            self.phase = Phase::Quarantined;
            return Err(Error::Offline);
        }
        let Some(epoch) = self.epoch.checked_add(1) else {
            self.phase = Phase::Quarantined;
            return Err(Error::Exhausted);
        };
        // A reset is not a rollback. Keep a mutating terminal result, including
        // Unknown, until its original owner consumes it or exits. Its ticket and
        // original epoch remain unchanged while the transport epoch advances.
        self.pending = self
            .pending
            .filter(|r| r.published && r.operation.is_mutating() && r.completion.is_some());
        self.epoch = epoch;
        self.phase = if self.pending.is_some() {
            Phase::Complete
        } else {
            Phase::Ready
        };
        Ok(())
    }
    pub fn submit(
        &mut self,
        binding: Binding,
        read: Read,
        capacity: u64,
        now: u64,
        timeout: u64,
    ) -> Result<Request, Error> {
        let mut request = self.prepare(
            binding,
            Operation::Read {
                sector: read.sector,
                offset: read.offset,
                length: read.length,
            },
            capacity,
            now,
            timeout,
        )?;
        // Compatibility wrapper for the M5 caller, which already treats submit
        // as exposed. New transports explicitly prepare then publish.
        request.published = true;
        self.pending = Some(request);
        self.phase = Phase::Pending;
        Ok(request)
    }
    pub fn prepare(
        &mut self,
        binding: Binding,
        operation: Operation,
        capacity: u64,
        now: u64,
        timeout: u64,
    ) -> Result<Request, Error> {
        let (sector, offset, length) = match operation {
            Operation::Read {
                sector,
                offset,
                length,
            } => {
                if length == 0
                    || length > MAX_SLICE
                    || offset.checked_add(length).ok_or(Error::Range)? > 512
                {
                    return Err(Error::Range);
                }
                (sector, offset, length)
            }
            Operation::Write { sector } => (sector, 0, 512),
            Operation::Flush => (0, 0, 0),
        };
        if binding.pid == 0
            || binding.object_key >> 32 == 0
            || capacity == 0
            || sector >= capacity
            || timeout == 0
        {
            return Err(Error::Range);
        }
        let deadline = now.checked_add(timeout).ok_or(Error::Range)?;
        if self.phase != Phase::Ready {
            return Err(
                if matches!(
                    self.phase,
                    Phase::Prepared | Phase::Pending | Phase::Published | Phase::Complete
                ) {
                    Error::Busy
                } else {
                    Error::Offline
                },
            );
        }
        let next = self.next_ticket.checked_add(1).ok_or(Error::Exhausted)?;
        let request = Request {
            binding,
            ticket: self.next_ticket,
            epoch: self.epoch,
            sector,
            offset: offset as u16,
            length: length as u16,
            deadline,
            completion: None,
            operation,
            published: false,
        };
        self.next_ticket = next;
        self.pending = Some(request);
        self.phase = Phase::Prepared;
        Ok(request)
    }
    /// Call directly BEFORE writing avail.idx. A missing notify is not proof the
    /// device cannot observe the available entry. This deliberately errs toward
    /// Unknown if publication itself is interrupted.
    pub fn publish(&mut self, epoch: u64, ticket: u64) -> Result<(), Error> {
        if epoch != self.epoch {
            return Err(Error::StaleEpoch);
        }
        let mut r = self.pending.ok_or(Error::WrongTicket)?;
        if r.ticket != ticket {
            return Err(Error::WrongTicket);
        }
        if self.phase != Phase::Prepared {
            return Err(Error::Offline);
        }
        r.published = true;
        self.pending = Some(r);
        self.phase = Phase::Published;
        Ok(())
    }
    pub fn reject_prepared(&mut self, binding: Binding, ticket: u64) -> Result<(), Error> {
        let r = self.pending.ok_or(Error::WrongTicket)?;
        if r.binding != binding || r.ticket != ticket {
            return Err(Error::WrongTicket);
        }
        if self.phase != Phase::Prepared || r.published {
            return Err(Error::Offline);
        }
        self.pending = None;
        self.phase = Phase::Ready;
        Ok(())
    }
    /// Epoch/ticket are software provenance from the caller, never fields from
    /// a virtio used entry. This cannot detect malicious device replay after a
    /// reused head ID; actual acknowledged reset remains a hardware prerequisite.
    pub fn complete(&mut self, epoch: u64, ticket: u64, outcome: Completion) -> Result<(), Error> {
        if epoch != self.epoch {
            return Err(Error::StaleEpoch);
        }
        let mut r = self.pending.ok_or(Error::WrongTicket)?;
        if r.ticket != ticket {
            return Err(Error::WrongTicket);
        }
        if !matches!(self.phase, Phase::Pending | Phase::Published) {
            return Err(Error::Offline);
        }
        r.completion = Some(
            if r.operation.is_mutating() && outcome != Completion::Success {
                Completion::Unknown
            } else {
                outcome
            },
        );
        self.pending = Some(r);
        self.phase = Phase::Complete;
        Ok(())
    }
    pub fn peek(&self, binding: Binding, ticket: u64) -> Result<Request, Error> {
        let r = self.pending.ok_or(Error::WrongTicket)?;
        if r.binding != binding || r.ticket != ticket {
            return Err(Error::WrongTicket);
        }
        let retained_terminal = r.published && r.operation.is_mutating() && r.completion.is_some();
        if r.epoch != self.epoch && !retained_terminal {
            return Err(Error::StaleEpoch);
        }
        match self.phase {
            Phase::Prepared | Phase::Pending | Phase::Published => Err(Error::Pending),
            Phase::Complete => Ok(r),
            Phase::Resetting | Phase::Quarantined if retained_terminal => Ok(r),
            _ => Err(Error::Offline),
        }
    }
    /// Only consume after the caller validated the full user output. Device
    /// errors are terminal Completion values, not success payloads.
    pub fn consume(&mut self, binding: Binding, ticket: u64) -> Result<Completion, Error> {
        let r = self.peek(binding, ticket)?;
        let outcome = r.completion.ok_or(Error::Protocol)?;
        self.pending = None;
        if !matches!(self.phase, Phase::Resetting | Phase::Quarantined) {
            self.phase = Phase::Ready;
        }
        Ok(outcome)
    }
    pub fn start_reset(&mut self) -> Result<(), Error> {
        if self.phase == Phase::Quarantined {
            return Err(Error::Offline);
        }
        if let Some(r) = &mut self.pending {
            if r.published && r.operation.is_mutating() && r.completion.is_none() {
                r.completion = Some(Completion::Unknown);
            }
        }
        self.phase = Phase::Resetting;
        Ok(())
    }
    pub fn timeout(&mut self, now: u64) -> bool {
        if matches!(self.phase, Phase::Pending | Phase::Published)
            && self.pending.is_some_and(|r| now >= r.deadline)
        {
            // This transition cannot fail in Pending/Published, and preserves
            // an unobserved mutation as Unknown before any reset acknowledgement.
            self.start_reset()
                .expect("active request cannot be quarantined");
            true
        } else {
            false
        }
    }
    /// Drop the owner's result; an exposed, uncompleted request still requires
    /// transport reset. Neither owner exit nor consume clears quarantine.
    pub fn exit(&mut self, pid: u64) -> Option<Request> {
        let mut r = self.pending.filter(|r| r.binding.pid == pid)?;
        self.pending = None;
        if r.published && r.completion.is_none() {
            if r.operation.is_mutating() {
                r.completion = Some(Completion::Unknown);
            }
            if self.phase != Phase::Quarantined {
                self.phase = Phase::Resetting;
            }
        } else if !matches!(self.phase, Phase::Resetting | Phase::Quarantined) {
            self.phase = Phase::Ready;
        }
        Some(r)
    }
}

#[cfg(test)]
mod tests;
