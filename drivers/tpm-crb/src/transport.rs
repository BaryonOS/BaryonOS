//! QEMU10.0.13 locality0 CRB only. Device buffers are fixed MMIO inside the TPM
//! aperture, never allocator RAM/DMA descriptors. No volatile/raw-pointer adapter
//! is provided here. The future exclusive broker owns mapping and quarantine.
use crate::codec::{self, Command, Expectation, Response, MAX_RESPONSE};
pub const BASE: u64 = 0xfed4_0000;
pub const APERTURE: usize = 0x1000;
pub const BUFFER_OFFSET: usize = 0x80;
pub const BUFFER_SIZE: usize = APERTURE - BUFFER_OFFSET;
pub const MAX_TICKS: u64 = 60_000;
const IO_CHUNK: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Register {
    LocalityState = 0,
    LocalityControl = 8,
    LocalityStatus = 0xc,
    InterfaceId = 0x30,
    ControlRequest = 0x40,
    ControlStatus = 0x44,
    Cancel = 0x48,
    Start = 0x4c,
    CommandSize = 0x58,
    CommandAddressLow = 0x5c,
    CommandAddressHigh = 0x60,
    ResponseSize = 0x64,
    ResponseAddressLow = 0x68,
    ResponseAddressHigh = 0x6c,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IoError;
/// Values are 32-bit little-endian registers. Buffer offsets are relative to
/// BASE+0x80, not physical pointers. Implementations must independently enforce
/// offsets/lengths <=BUFFER_SIZE and IO_CHUNK, ordered device accesses and exact
/// copy completion; an error can mean a partial write. quarantine must retain
/// exclusive broker/device authority even after this adapter value is dropped.
/// It is not permission to free or regrant an outstanding hardware buffer.
pub trait Io {
    fn read32(&mut self, register: Register) -> Result<u32, IoError>;
    fn write32(&mut self, register: Register, value: u32) -> Result<(), IoError>;
    fn read_buffer(&mut self, offset: usize, out: &mut [u8]) -> Result<(), IoError>;
    fn write_buffer(&mut self, offset: usize, bytes: &[u8]) -> Result<(), IoError>;
    fn quarantine(&mut self);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    NotStarted,
    Unknown,
    Completed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Input,
    State,
    Ticket,
    Exhausted,
    Clock,
    Unsupported,
    Locality,
    Device,
    Busy,
    Buffer,
    Io,
    Timeout,
    Cancelled,
    CancelTimeout,
    Protocol,
    Tpm(u32),
    Quarantined,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Failure {
    pub fault: Fault,
    pub effect: Effect,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ticket(u64);
impl Ticket {
    pub const fn raw(self) -> u64 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Fresh,
    Acquiring,
    Preparing,
    Ready,
    Executing,
    Cancelling,
    Idling,
    Releasing,
    Released,
    Quarantined,
    Detached,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Poll {
    Pending,
    Complete(Response),
}
pub struct Transport<I: Io> {
    io: Option<I>,
    phase: Phase,
    notified: bool,
    next: u64,
    current: Ticket,
    last_tick: u64,
    deadline: u64,
    cancel_ticks: u64,
    cancel_fault: Fault,
    expect: Option<(Expectation, bool)>,
    effect: Effect,
    response: [u8; MAX_RESPONSE],
}
impl<I: Io> Transport<I> {
    pub fn new(io: I) -> Self {
        Self {
            io: Some(io),
            phase: Phase::Fresh,
            notified: false,
            next: 1,
            current: Ticket(0),
            last_tick: 0,
            deadline: 0,
            cancel_ticks: 0,
            cancel_fault: Fault::Cancelled,
            expect: None,
            effect: Effect::NotStarted,
            response: [0; MAX_RESPONSE],
        }
    }
    pub const fn phase(&self) -> Phase {
        self.phase
    }
    pub const fn is_quarantined(&self) -> bool {
        matches!(self.phase, Phase::Quarantined)
    }
    /// Only a never-used adapter or a fully Idle+LocalityReleased-ACKed adapter
    /// may leave the transport. This method never performs a software reset.
    pub fn detach(&mut self) -> Result<I, Failure> {
        if !matches!(self.phase, Phase::Fresh | Phase::Released) {
            return Err(self.failure(Fault::State));
        }
        let io = self.io.take().ok_or(self.failure(Fault::State))?;
        self.phase = Phase::Detached;
        Ok(io)
    }
    fn failure(&self, fault: Fault) -> Failure {
        Failure {
            fault,
            effect: self.effect,
        }
    }
    fn rejected(fault: Fault) -> Failure {
        Failure {
            fault,
            effect: Effect::NotStarted,
        }
    }
    fn quarantine(&mut self, fault: Fault) -> Failure {
        let result = self.failure(fault);
        self.phase = Phase::Quarantined;
        if !self.notified {
            self.notified = true;
            if let Some(io) = &mut self.io {
                io.quarantine();
            }
        }
        result
    }
    fn clock(&mut self, now: u64) -> Result<(), Failure> {
        if now < self.last_tick {
            return Err(self.quarantine(Fault::Clock));
        }
        self.last_tick = now;
        Ok(())
    }
    fn bounded(now: u64, deadline: u64) -> bool {
        deadline
            .checked_sub(now)
            .is_some_and(|delta| delta > 0 && delta <= MAX_TICKS)
    }
    fn ticket(&mut self) -> Result<Ticket, Failure> {
        let next = self
            .next
            .checked_add(1)
            .ok_or(Self::rejected(Fault::Exhausted))?;
        let result = Ticket(self.next);
        self.next = next;
        self.current = result;
        Ok(result)
    }
    fn matches(&self, ticket: Ticket) -> Result<(), Failure> {
        if ticket.0 == 0 || ticket != self.current {
            return Err(Self::rejected(Fault::Ticket));
        }
        if self.phase == Phase::Quarantined {
            return Err(self.failure(Fault::Quarantined));
        }
        Ok(())
    }
    fn read(&mut self, r: Register) -> Result<u32, Failure> {
        let result = self
            .io
            .as_mut()
            .ok_or(Self::rejected(Fault::State))?
            .read32(r);
        result.map_err(|_| self.quarantine(Fault::Io))
    }
    fn write(&mut self, r: Register, value: u32) -> Result<(), Failure> {
        let result = self
            .io
            .as_mut()
            .ok_or(Self::rejected(Fault::State))?
            .write32(r, value);
        result.map_err(|_| self.quarantine(Fault::Io))
    }
    fn write_buffer(&mut self, offset: usize, bytes: &[u8]) -> Result<(), Failure> {
        if bytes.len() > IO_CHUNK
            || offset
                .checked_add(bytes.len())
                .is_none_or(|end| end > BUFFER_SIZE)
        {
            return Err(self.quarantine(Fault::Buffer));
        }
        let result = self
            .io
            .as_mut()
            .ok_or(Self::rejected(Fault::State))?
            .write_buffer(offset, bytes);
        result.map_err(|_| self.quarantine(Fault::Io))
    }
    fn wipe(&mut self) -> Result<(), Failure> {
        let zeros = [0; IO_CHUNK];
        let mut at = 0;
        while at < BUFFER_SIZE {
            let n = IO_CHUNK.min(BUFFER_SIZE - at);
            self.write_buffer(at, &zeros[..n])?;
            at += n;
        }
        Ok(())
    }
    fn profile(&mut self) -> Result<(), Failure> {
        let id = self.read(Register::InterfaceId)?;
        // Active CRB/version1, locality0-only, no idle bypass, CRB supported and
        // selected. Vendor/revision and documented transfer-size bits are irrelevant.
        let mask = 0xf | 0xf0 | 0x100 | 0x200 | 0x4000 | 0x60000;
        if id & mask != 0x24011 {
            return Err(self.quarantine(Fault::Unsupported));
        }
        let command = self.read(Register::CommandAddressLow)? as u64
            | ((self.read(Register::CommandAddressHigh)? as u64) << 32);
        let response = self.read(Register::ResponseAddressLow)? as u64
            | ((self.read(Register::ResponseAddressHigh)? as u64) << 32);
        let csize = self.read(Register::CommandSize)? as usize;
        let rsize = self.read(Register::ResponseSize)? as usize;
        if command != BASE + BUFFER_OFFSET as u64
            || response != command
            || csize != BUFFER_SIZE
            || rsize != BUFFER_SIZE
        {
            return Err(self.quarantine(Fault::Buffer));
        }
        Ok(())
    }
    fn locality(&mut self, owned: bool) -> Result<bool, Failure> {
        let state = self.read(Register::LocalityState)?;
        let status = self.read(Register::LocalityStatus)?;
        if state & 0x80 == 0
            || state & !0x9f != 0
            || state & 0x1c != 0
            || status & !3 != 0
            || status & 2 != 0
        {
            return Err(self.quarantine(Fault::Locality));
        }
        let assigned = state & 2 != 0;
        let granted = status & 1 != 0;
        if assigned != granted {
            return Err(self.quarantine(Fault::Locality));
        }
        if owned && !assigned {
            return Err(self.quarantine(Fault::Locality));
        }
        Ok(assigned)
    }
    fn status(&mut self) -> Result<u32, Failure> {
        let status = self.read(Register::ControlStatus)?;
        if status & !3 != 0 || status & 1 != 0 {
            return Err(self.quarantine(Fault::Device));
        }
        Ok(status)
    }
    fn start(&mut self) -> Result<bool, Failure> {
        let start = self.read(Register::Start)?;
        if start & !1 != 0 {
            return Err(self.quarantine(Fault::Protocol));
        }
        Ok(start != 0)
    }
    fn owned_ready(&mut self) -> Result<(), Failure> {
        self.profile()?;
        self.locality(true)?;
        if self.status()? != 0 {
            return Err(self.quarantine(Fault::State));
        }
        Ok(())
    }
    pub fn acquire(&mut self, now: u64, deadline: u64) -> Result<Ticket, Failure> {
        if !matches!(self.phase, Phase::Fresh | Phase::Released) {
            return Err(Self::rejected(Fault::State));
        }
        if !Self::bounded(now, deadline) {
            return Err(Self::rejected(Fault::Input));
        }
        self.clock(now)?;
        self.effect = Effect::NotStarted;
        self.profile()?;
        self.locality(false)?;
        self.status()?;
        if self.start()? {
            return Err(self.quarantine(Fault::Busy));
        }
        let ticket = self.ticket()?;
        self.deadline = deadline;
        self.phase = Phase::Acquiring;
        self.write(Register::LocalityControl, 1)?;
        Ok(ticket)
    }
    pub fn poll_ready(&mut self, ticket: Ticket, now: u64) -> Result<bool, Failure> {
        self.matches(ticket)?;
        if self.phase == Phase::Ready {
            return Ok(true);
        }
        if !matches!(self.phase, Phase::Acquiring | Phase::Preparing) {
            return Err(Self::rejected(Fault::State));
        }
        self.clock(now)?;
        if now >= self.deadline {
            return Err(self.quarantine(Fault::Timeout));
        }
        self.profile()?;
        let assigned = self.locality(false)?;
        if !assigned {
            return Ok(false);
        }
        if self.start()? {
            return Err(self.quarantine(Fault::Busy));
        }
        if self.phase == Phase::Acquiring {
            self.phase = Phase::Preparing;
            self.write(Register::ControlRequest, 1)?;
            return Ok(false);
        }
        let req = self.read(Register::ControlRequest)?;
        if req & !1 != 0 {
            return Err(self.quarantine(Fault::Protocol));
        }
        if req & 1 != 0 || self.status()? & 2 != 0 {
            return Ok(false);
        }
        self.phase = Phase::Ready;
        Ok(true)
    }
    pub fn execute(
        &mut self,
        command: &Command<'_>,
        now: u64,
        deadline: u64,
        cancel_ticks: u64,
    ) -> Result<Ticket, Failure> {
        if self.phase != Phase::Ready {
            return Err(Self::rejected(Fault::State));
        }
        if !Self::bounded(now, deadline) || cancel_ticks == 0 || cancel_ticks > MAX_TICKS {
            return Err(Self::rejected(Fault::Input));
        }
        self.clock(now)?;
        self.owned_ready()?;
        if self.start()? {
            return Err(self.quarantine(Fault::Busy));
        }
        let ticket = self.ticket()?;
        self.effect = Effect::NotStarted;
        if self.read(Register::Cancel)? != 0 {
            return Err(self.quarantine(Fault::State));
        }
        self.wipe()?;
        for (i, chunk) in command.bytes().chunks(IO_CHUNK).enumerate() {
            self.write_buffer(i * IO_CHUNK, chunk)?;
        }
        self.expect = Some((command.expect, command.sessions));
        self.deadline = deadline;
        self.cancel_ticks = cancel_ticks;
        self.phase = Phase::Executing;
        // A failing START write can already have reached hardware. Never replay it.
        self.effect = Effect::Unknown;
        self.write(Register::Start, 1)?;
        Ok(ticket)
    }
    fn begin_cancel(&mut self, now: u64, fault: Fault) -> Result<(), Failure> {
        self.deadline = now
            .checked_add(self.cancel_ticks)
            .ok_or_else(|| self.quarantine(Fault::Clock))?;
        self.cancel_fault = fault;
        self.phase = Phase::Cancelling;
        self.write(Register::Cancel, 1)
    }
    pub fn cancel(&mut self, ticket: Ticket, now: u64) -> Result<(), Failure> {
        self.matches(ticket)?;
        if self.phase == Phase::Cancelling {
            return Ok(());
        }
        if self.phase != Phase::Executing {
            return Err(Self::rejected(Fault::State));
        }
        self.clock(now)?;
        self.owned_ready()?;
        self.begin_cancel(now, Fault::Cancelled)
    }
    fn read_response(&mut self, length: usize) -> Result<(), Failure> {
        let mut at = 0;
        while at < length {
            let end = (at + IO_CHUNK).min(length);
            let result = self
                .io
                .as_mut()
                .ok_or(Self::rejected(Fault::State))?
                .read_buffer(at, &mut self.response[at..end]);
            if result.is_err() {
                return Err(self.quarantine(Fault::Io));
            }
            at = end;
        }
        Ok(())
    }
    fn finish_without_output(&mut self, fault: Fault) -> Result<Poll, Failure> {
        self.write(Register::Cancel, 0)?;
        if self.read(Register::Cancel)? != 0 {
            return Err(self.quarantine(Fault::Protocol));
        }
        self.wipe()?;
        self.expect = None;
        self.response.fill(0);
        self.phase = Phase::Ready;
        self.effect = Effect::NotStarted;
        Err(Failure {
            fault,
            effect: Effect::Unknown,
        })
    }
    pub fn poll(&mut self, ticket: Ticket, now: u64) -> Result<Poll, Failure> {
        self.matches(ticket)?;
        if !matches!(self.phase, Phase::Executing | Phase::Cancelling) {
            return Err(Self::rejected(Fault::State));
        }
        self.clock(now)?;
        self.owned_ready()?;
        let running = self.start()?;
        if running {
            if now >= self.deadline {
                if self.phase == Phase::Cancelling {
                    return Err(self.quarantine(Fault::CancelTimeout));
                }
                self.begin_cancel(now, Fault::Timeout)?;
            }
            return Ok(Poll::Pending);
        }
        if self.phase == Phase::Cancelling {
            return self.finish_without_output(self.cancel_fault);
        }
        if now >= self.deadline {
            return self.finish_without_output(Fault::Timeout);
        }
        self.read_response(10)?;
        let mut header = [0; 10];
        header.copy_from_slice(&self.response[..10]);
        let length = codec::response_size(&header, BUFFER_SIZE)
            .map_err(|_| self.quarantine(Fault::Protocol))?;
        self.read_response(length)?;
        if self.response[..10] != header {
            return Err(self.quarantine(Fault::Protocol));
        }
        self.owned_ready()?;
        if self.start()? {
            return Err(self.quarantine(Fault::Busy));
        }
        let (expect, sessions) = self.expect.ok_or_else(|| self.quarantine(Fault::State))?;
        let result = codec::validate(expect, sessions, &self.response[..length]);
        if matches!(
            result,
            Err(codec::Error::Input | codec::Error::Length | codec::Error::Protocol)
        ) {
            return Err(self.quarantine(Fault::Protocol));
        }
        self.wipe()?;
        self.response.fill(0);
        self.expect = None;
        self.phase = Phase::Ready;
        self.effect = Effect::NotStarted;
        match result {
            Ok(response) => Ok(Poll::Complete(response)),
            Err(codec::Error::Tpm(rc)) => Err(Failure {
                fault: Fault::Tpm(rc),
                effect: Effect::Completed,
            }),
            _ => unreachable!(),
        }
    }
    pub fn release(&mut self, now: u64, deadline: u64) -> Result<Ticket, Failure> {
        if self.phase != Phase::Ready {
            return Err(Self::rejected(Fault::State));
        }
        if !Self::bounded(now, deadline) {
            return Err(Self::rejected(Fault::Input));
        }
        self.clock(now)?;
        self.owned_ready()?;
        if self.start()? {
            return Err(self.quarantine(Fault::Busy));
        }
        let ticket = self.ticket()?;
        self.wipe()?;
        self.deadline = deadline;
        self.phase = Phase::Idling;
        self.write(Register::ControlRequest, 2)?;
        Ok(ticket)
    }
    pub fn poll_released(&mut self, ticket: Ticket, now: u64) -> Result<bool, Failure> {
        self.matches(ticket)?;
        if self.phase == Phase::Released {
            return Ok(true);
        }
        if !matches!(self.phase, Phase::Idling | Phase::Releasing) {
            return Err(Self::rejected(Fault::State));
        }
        self.clock(now)?;
        if now >= self.deadline {
            return Err(self.quarantine(Fault::Timeout));
        }
        self.profile()?;
        if self.start()? {
            return Err(self.quarantine(Fault::Busy));
        }
        if self.phase == Phase::Idling {
            self.locality(true)?;
            let req = self.read(Register::ControlRequest)?;
            if req & !2 != 0 {
                return Err(self.quarantine(Fault::Protocol));
            }
            if req & 2 != 0 || self.status()? & 2 == 0 {
                return Ok(false);
            }
            self.phase = Phase::Releasing;
            self.write(Register::LocalityControl, 2)?;
            return Ok(false);
        }
        if self.locality(false)? {
            return Ok(false);
        }
        if self.status()? != 2 {
            return Err(self.quarantine(Fault::State));
        }
        self.phase = Phase::Released;
        Ok(true)
    }
}
impl<I: Io> Drop for Transport<I> {
    fn drop(&mut self) {
        if self.io.is_some()
            && !matches!(self.phase, Phase::Fresh | Phase::Released | Phase::Detached)
        {
            self.quarantine(Fault::Quarantined);
        }
    }
}
