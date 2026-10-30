#![no_std]
#![forbid(unsafe_code)]
//! Bounded polled QEMU i8042 profile. This core does not grant port access,
//! interpret GUI events, or pretend a pending hardware reset is ready.

pub const PROFILE: u32 = 1;
pub const PER_CALL_ACTIONS: usize = 8;
pub const MAX_OPERATIONS: u32 = 8192;
pub const MAX_DRAIN: u32 = 256;
pub const STEP_TICKS: u64 = 64;
pub const RESET_TICKS: u64 = 1000;

/// Exclusive fixed port pair: status/command=0x64, data=0x60. One method is
/// one bounded scalar operation. The adapter serializes the entire call.
pub trait Ports {
    fn status(&mut self) -> u8;
    fn read_data(&mut self) -> u8;
    fn command(&mut self, byte: u8);
    fn data(&mut self, byte: u8);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Busy,
    NeedsReset,
    WrongTicket,
    Timeout,
    ClockRollback,
    Budget,
    DrainOverflow,
    Controller,
    Response,
    UnsupportedDevice,
    CounterExhausted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResetReport {
    pub ticket: u64,
    pub epoch: u64,
    pub discarded: u32,
    pub operations: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    Pending,
    Ready(ResetReport),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub epoch: u64,
    pub sequence: u64,
    pub status: u8,
    pub byte: u8,
}

#[derive(Clone, Copy)]
enum Action {
    Command(u8),
    Mode(bool),
    ReadMode,
    VerifyMode,
    Device(u8, bool),
    Expect(u8, bool),
    Drain,
    Done,
}
use Action::*;
const PLAN: &[Action] = &[
    Command(0xad),
    Command(0xa7),
    Drain,
    Command(0x20),
    ReadMode,
    Command(0x60),
    Mode(false),
    Command(0xae),
    Command(0xa8),
    Device(0xff, false),
    Expect(0xfa, false),
    Expect(0xaa, false),
    Device(0xf5, false),
    Expect(0xfa, false),
    Device(0xf0, false),
    Expect(0xfa, false),
    Device(2, false),
    Expect(0xfa, false),
    Command(0xd4),
    Device(0xff, true),
    Expect(0xfa, true),
    Expect(0xaa, true),
    Expect(0, true),
    Command(0xd4),
    Device(0xf6, true),
    Expect(0xfa, true),
    Command(0x60),
    Mode(true),
    Command(0x20),
    VerifyMode,
    Command(0xd4),
    Device(0xf4, true),
    Expect(0xfa, true),
    Device(0xf4, false),
    Expect(0xfa, false),
    Done,
];
#[derive(Clone, Copy)]
enum State {
    Offline,
    Running,
    Ready(ResetReport),
    Failed(u64, Fault),
    Closed,
}

pub struct Controller {
    state: State,
    next_ticket: u64,
    ticket: u64,
    epoch: u64,
    sequence: u64,
    step: usize,
    mode: u8,
    discarded: u32,
    operations: u32,
    drained: u32,
    resends: u8,
    last_device: Option<(usize, bool)>,
    started: u64,
    advanced: u64,
    last_now: u64,
}
impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}
impl Controller {
    pub const fn new() -> Self {
        Self {
            state: State::Offline,
            next_ticket: 1,
            ticket: 0,
            epoch: 0,
            sequence: 1,
            step: 0,
            mode: 0,
            discarded: 0,
            operations: 0,
            drained: 0,
            resends: 0,
            last_device: None,
            started: 0,
            advanced: 0,
            last_now: 0,
        }
    }
    /// No I/O. Losing the source requires a fresh protected reset; old tickets
    /// cannot resume that reset and no previous Ready state is retained.
    pub fn detach(&mut self) {
        if !matches!(self.state, State::Closed) {
            self.state = State::Offline;
        }
    }
    pub fn begin_reset(&mut self, now: u64) -> Result<u64, Fault> {
        if matches!(self.state, State::Running) {
            return Err(Fault::Busy);
        }
        if matches!(self.state, State::Closed) {
            return Err(Fault::CounterExhausted);
        }
        if now < self.last_now {
            return Err(Fault::ClockRollback);
        }
        let Some(next) = self.next_ticket.checked_add(1) else {
            self.state = State::Closed;
            return Err(Fault::CounterExhausted);
        };
        if self.epoch.checked_add(1).is_none() || now.checked_add(RESET_TICKS).is_none() {
            self.state = State::Closed;
            return Err(Fault::CounterExhausted);
        }
        self.ticket = self.next_ticket;
        self.next_ticket = next;
        self.state = State::Running;
        self.step = 0;
        self.mode = 0;
        self.discarded = 0;
        self.operations = 0;
        self.drained = 0;
        self.resends = 0;
        self.last_device = None;
        self.started = now;
        self.advanced = now;
        self.last_now = now;
        Ok(self.ticket)
    }
    fn op(&mut self) -> Result<(), Fault> {
        self.operations = self.operations.checked_add(1).ok_or(Fault::Budget)?;
        if self.operations > MAX_OPERATIONS {
            Err(Fault::Budget)
        } else {
            Ok(())
        }
    }
    fn status(&mut self, io: &mut impl Ports) -> Result<u8, Fault> {
        self.op()?;
        Ok(io.status())
    }
    fn read(&mut self, io: &mut impl Ports) -> Result<u8, Fault> {
        self.op()?;
        Ok(io.read_data())
    }
    fn send(&mut self, io: &mut impl Ports, value: u8, command: bool) -> Result<bool, Fault> {
        if self.status(io)? & 2 != 0 {
            return Ok(false);
        }
        self.op()?;
        if command {
            io.command(value);
        } else {
            io.data(value);
        }
        Ok(true)
    }
    fn incoming(&mut self, io: &mut impl Ports) -> Result<Option<(bool, u8)>, Fault> {
        let status = self.status(io)?;
        if status & 1 == 0 {
            return Ok(None);
        }
        let byte = self.read(io)?;
        if status & 0xc0 != 0 {
            return Err(Fault::Controller);
        }
        Ok(Some((status & 0x20 != 0, byte)))
    }
    fn advance(&mut self, now: u64) {
        self.step += 1;
        self.advanced = now;
    }
    fn run(&mut self, now: u64, io: &mut impl Ports) -> Result<Progress, Fault> {
        if now < self.last_now {
            return Err(Fault::ClockRollback);
        }
        self.last_now = now;
        if now - self.started >= RESET_TICKS || now - self.advanced >= STEP_TICKS {
            return Err(Fault::Timeout);
        }
        for _ in 0..PER_CALL_ACTIONS {
            match PLAN[self.step] {
                Command(value) => {
                    if !self.send(io, value, true)? {
                        return Ok(Progress::Pending);
                    }
                    self.advance(now);
                }
                Mode(translation) => {
                    let value = (self.mode & 0x0c) | if translation { 0x40 } else { 0 };
                    if !self.send(io, value, false)? {
                        return Ok(Progress::Pending);
                    }
                    self.advance(now);
                }
                Device(value, mouse) => {
                    if !self.send(io, value, false)? {
                        return Ok(Progress::Pending);
                    }
                    if self.last_device.is_none() {
                        self.resends = 0;
                    }
                    self.last_device = Some((self.step, mouse));
                    self.advance(now);
                }
                ReadMode | VerifyMode => {
                    let Some((mouse, byte)) = self.incoming(io)? else {
                        return Ok(Progress::Pending);
                    };
                    if mouse {
                        self.discarded += 1;
                        continue;
                    }
                    if matches!(PLAN[self.step], ReadMode) {
                        if byte & 0x80 != 0 {
                            return Err(Fault::Controller);
                        }
                        self.mode = byte;
                    } else if byte != ((self.mode & 0x0c) | 0x40) {
                        return Err(Fault::Controller);
                    }
                    self.advance(now);
                }
                Expect(expected, mouse) => {
                    let Some((actual_mouse, byte)) = self.incoming(io)? else {
                        return Ok(Progress::Pending);
                    };
                    // Opposite-channel input during reset is deliberately lost,
                    // counted, and covered by the mandatory new epoch/SYNC.
                    if actual_mouse != mouse {
                        self.discarded += 1;
                        continue;
                    }
                    if byte == 0xfe && expected == 0xfa {
                        let Some((index, aux)) = self.last_device else {
                            return Err(Fault::Response);
                        };
                        if self.resends >= 2 {
                            return Err(Fault::Response);
                        }
                        self.resends += 1;
                        self.step = index - usize::from(aux);
                        continue;
                    }
                    if byte != expected {
                        return Err(if expected == 0 {
                            Fault::UnsupportedDevice
                        } else {
                            Fault::Response
                        });
                    }
                    if expected == 0xfa {
                        self.last_device = None;
                        self.resends = 0;
                    }
                    self.advance(now);
                }
                Drain => {
                    let status = self.status(io)?;
                    if status & 1 == 0 {
                        self.advance(now);
                        continue;
                    }
                    if self.drained == MAX_DRAIN {
                        return Err(Fault::DrainOverflow);
                    }
                    let _ = self.read(io)?;
                    self.drained += 1;
                    self.discarded += 1;
                }
                Done => {
                    self.epoch = self.epoch.checked_add(1).ok_or(Fault::CounterExhausted)?;
                    self.sequence = 1;
                    let report = ResetReport {
                        ticket: self.ticket,
                        epoch: self.epoch,
                        discarded: self.discarded,
                        operations: self.operations,
                    };
                    self.state = State::Ready(report);
                    return Ok(Progress::Ready(report));
                }
            }
        }
        Ok(Progress::Pending)
    }
    /// At most 16 port operations. The kernel must validate output before this
    /// call. Pending can advance/drain the authorized reset, but is never Ready.
    pub fn poll_reset(
        &mut self,
        ticket: u64,
        now: u64,
        io: &mut impl Ports,
    ) -> Result<Progress, Fault> {
        match self.state {
            State::Closed => return Err(Fault::CounterExhausted),
            State::Ready(report) if report.ticket == ticket => return Ok(Progress::Ready(report)),
            State::Failed(id, fault) if id == ticket => return Err(fault),
            State::Running if ticket == self.ticket => {}
            _ => return Err(Fault::WrongTicket),
        }
        let result = self.run(now, io);
        if let Err(fault) = result {
            self.state = if fault == Fault::CounterExhausted {
                State::Closed
            } else {
                State::Failed(ticket, fault)
            };
        }
        result
    }
    /// Exactly one status and at most one FIFO read. No keyboard/mouse byte
    /// rewriting. Error status is delivered once, then a reset is mandatory.
    pub fn read_sample(&mut self, io: &mut impl Ports) -> Result<Option<Sample>, Fault> {
        if !matches!(self.state, State::Ready(_)) {
            return Err(Fault::NeedsReset);
        }
        let Some(next) = self.sequence.checked_add(1) else {
            self.state = State::Closed;
            return Err(Fault::CounterExhausted);
        };
        let status = io.status();
        if status & 1 == 0 {
            return Ok(None);
        }
        let sample = Sample {
            epoch: self.epoch,
            sequence: self.sequence,
            status,
            byte: io.read_data(),
        };
        self.sequence = next;
        if status & 0xc0 != 0 {
            self.state = State::Offline;
        }
        Ok(Some(sample))
    }
}

#[cfg(test)]
mod counter_tests {
    use super::*;
    struct NoIo;
    impl Ports for NoIo {
        fn status(&mut self) -> u8 {
            panic!("unexpected I/O")
        }
        fn read_data(&mut self) -> u8 {
            panic!("unexpected I/O")
        }
        fn command(&mut self, _: u8) {
            panic!("unexpected I/O")
        }
        fn data(&mut self, _: u8) {
            panic!("unexpected I/O")
        }
    }
    #[test]
    fn ticket_epoch_sequence_and_tick_exhaustion_fail_before_io() {
        for kind in 0..3 {
            let mut c = Controller::new();
            if kind == 0 {
                c.next_ticket = u64::MAX;
            } else if kind == 1 {
                c.epoch = u64::MAX;
            }
            assert_eq!(
                c.begin_reset(if kind == 2 { u64::MAX } else { 0 }),
                Err(Fault::CounterExhausted)
            );
            c.detach();
            assert_eq!(c.begin_reset(0), Err(Fault::CounterExhausted));
        }
        let mut c = Controller::new();
        c.state = State::Ready(ResetReport {
            ticket: 1,
            epoch: 1,
            discarded: 0,
            operations: 1,
        });
        c.sequence = u64::MAX;
        assert_eq!(c.read_sample(&mut NoIo), Err(Fault::CounterExhausted));
        assert_eq!(c.begin_reset(0), Err(Fault::CounterExhausted));
        let mut c = Controller::new();
        c.last_now = 10;
        assert_eq!(c.begin_reset(9), Err(Fault::ClockRollback));
        assert_eq!(c.next_ticket, 1);
    }
}
