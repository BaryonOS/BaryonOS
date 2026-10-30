use crate::{discard_caps, protocol, Ipc};
use nieuw_os_display_client::{
    wire::{Reset, Sample},
    Error as RawError, InputClient, ResetProgress, Transport,
};
use nieuw_os_input_core::wire::{self, Operation, Request};
use nieuw_os_sdk_runtime::{Error, Outgoing, ReceiveEndpoint, SendEndpoint};

pub const EXCHANGE_TICKS: u64 = 200;
pub const RESET_TICKS: u64 = 500;
pub trait RawInput {
    fn begin_reset(&mut self) -> Result<u64, RawError>;
    fn poll_reset(&mut self) -> Result<ResetProgress, RawError>;
    fn read(&mut self) -> Result<Option<Sample>, RawError>;
}
impl<T: Transport> RawInput for InputClient<T> {
    fn begin_reset(&mut self) -> Result<u64, RawError> {
        InputClient::begin_reset(self)
    }
    fn poll_reset(&mut self) -> Result<ResetProgress, RawError> {
        InputClient::poll_reset(self)
    }
    fn read(&mut self) -> Result<Option<Sample>, RawError> {
        InputClient::read(self)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Configuration {
    pub epoch: u64,
    pub policy_pid: u64,
    pub request: SendEndpoint,
    pub response: ReceiveEndpoint,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fatal {
    Configuration,
    Counter,
    Clock,
    Deadline,
    Raw(RawError),
    Transport(Error),
    Cleanup(Error),
    Protocol,
    Remote(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    Pending,
    Idle,
    Ready { raw_epoch: u64 },
    SampleAccepted,
    Recovering { reason: u16 },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub reset_started: u64,
    pub reset_ready: u64,
    pub sync_acked: u64,
    pub samples_read: u64,
    pub samples_acked: u64,
    pub send_again: u64,
    pub recoveries: u64,
}
struct Exchange {
    request: Request,
    outgoing: Outgoing,
    sent: bool,
    started: u64,
    sample_needs_reset: bool,
}
enum State {
    Begin,
    Resetting { started: u64 },
    Exchange,
    Reading,
    Failed,
}
pub struct Source {
    config: Configuration,
    next_request: u64,
    raw_epoch: u64,
    last_tick: u64,
    state: State,
    pending: Option<Exchange>,
    counts: Counts,
}
impl Source {
    pub fn new(config: Configuration) -> Result<Self, Fatal> {
        if config.epoch == 0
            || config.policy_pid == 0
            || config.request.handle() == config.response.handle()
        {
            return Err(Fatal::Configuration);
        }
        Ok(Self {
            config,
            next_request: 1,
            raw_epoch: 0,
            last_tick: 0,
            state: State::Begin,
            pending: None,
            counts: Counts::default(),
        })
    }
    pub const fn counts(&self) -> Counts {
        self.counts
    }
    fn exchange(
        &mut self,
        now: u64,
        operation: Operation,
        arg0: u64,
        arg1: u64,
        needs_reset: bool,
    ) -> Result<State, Fatal> {
        let id = self.next_request;
        self.next_request = id.checked_add(1).ok_or(Fatal::Counter)?;
        let request = Request {
            operation,
            id,
            epoch: self.config.epoch,
            arg0,
            arg1,
        };
        let outgoing = Outgoing::new(wire::TAG, &request.encode()).map_err(|_| Fatal::Protocol)?;
        self.pending = Some(Exchange {
            request,
            outgoing,
            sent: false,
            started: now,
            sample_needs_reset: needs_reset,
        });
        Ok(State::Exchange)
    }
    fn advance(
        &mut self,
        now: u64,
        raw: &mut impl RawInput,
        io: &mut impl Ipc,
    ) -> Result<Progress, Fatal> {
        if now < self.last_tick {
            return Err(Fatal::Clock);
        }
        self.last_tick = now;
        match &mut self.state {
            State::Begin => {
                raw.begin_reset().map_err(Fatal::Raw)?;
                self.counts.reset_started = self.counts.reset_started.saturating_add(1);
                self.state = State::Resetting { started: now };
                Ok(Progress::Pending)
            }
            State::Resetting { started } => {
                if now.saturating_sub(*started) > RESET_TICKS {
                    return Err(Fatal::Deadline);
                }
                match raw.poll_reset().map_err(Fatal::Raw)? {
                    ResetProgress::Pending => Ok(Progress::Pending),
                    ResetProgress::Ready(Reset { epoch, .. }) => {
                        if epoch == 0 || epoch <= self.raw_epoch {
                            return Err(Fatal::Protocol);
                        }
                        self.raw_epoch = epoch;
                        self.counts.reset_ready = self.counts.reset_ready.saturating_add(1);
                        self.state = self.exchange(now, Operation::Sync, 1, 0, false)?;
                        Ok(Progress::Pending)
                    }
                }
            }
            State::Reading => match raw.read().map_err(Fatal::Raw)? {
                None => Ok(Progress::Idle),
                Some(sample) => {
                    if sample.epoch != self.raw_epoch {
                        return Err(Fatal::Protocol);
                    }
                    self.counts.samples_read = self.counts.samples_read.saturating_add(1);
                    self.state = self.exchange(
                        now,
                        Operation::Sample,
                        u64::from(sample.status),
                        u64::from(sample.byte),
                        sample.needs_reset(),
                    )?;
                    Ok(Progress::Pending)
                }
            },
            State::Exchange => {
                let exchange = self.pending.as_mut().ok_or(Fatal::Protocol)?;
                if now.saturating_sub(exchange.started) > EXCHANGE_TICKS {
                    return Err(Fatal::Deadline);
                }
                if !exchange.sent {
                    match io.send(self.config.request, &exchange.outgoing) {
                        Ok(()) => exchange.sent = true,
                        Err(error) if error.is_again() => {
                            self.counts.send_again = self.counts.send_again.saturating_add(1);
                            return Ok(Progress::Pending);
                        }
                        Err(error) => return Err(Fatal::Transport(error)),
                    }
                }
                let message = match io.receive(self.config.response) {
                    Ok(message) => message,
                    Err(error) if error.is_again() => return Ok(Progress::Pending),
                    Err(error) => return Err(Fatal::Transport(error)),
                };
                if message.cap_count() != 0 {
                    discard_caps(io, &message).map_err(Fatal::Cleanup)?;
                    return Err(Fatal::Protocol);
                }
                let status =
                    protocol::command_reply(&message, self.config.policy_pid, exchange.request)
                        .map_err(|_| Fatal::Protocol)?;
                if status == 0 {
                    if exchange.sample_needs_reset {
                        return Err(Fatal::Protocol);
                    }
                    let sync = exchange.request.operation == Operation::Sync;
                    self.pending = None;
                    self.state = State::Reading;
                    if sync {
                        self.counts.sync_acked = self.counts.sync_acked.saturating_add(1);
                        Ok(Progress::Ready {
                            raw_epoch: self.raw_epoch,
                        })
                    } else {
                        self.counts.samples_acked = self.counts.samples_acked.saturating_add(1);
                        Ok(Progress::SampleAccepted)
                    }
                } else if exchange.request.operation == Operation::Sample
                    && (11..=15).contains(&status)
                {
                    // Core has explicitly invalidated the stream/generation.
                    // Never replay this already processed sample or invent one.
                    self.pending = None;
                    self.state = State::Begin;
                    self.counts.recoveries = self.counts.recoveries.saturating_add(1);
                    Ok(Progress::Recovering { reason: status })
                } else {
                    Err(Fatal::Remote(status))
                }
            }
            State::Failed => Err(Fatal::Protocol),
        }
    }
    /// One bounded native step; any fatal result permanently stops this source
    /// instance. Epoch recovery requires the real supervisor and fresh grants.
    pub fn step(
        &mut self,
        now: u64,
        raw: &mut impl RawInput,
        io: &mut impl Ipc,
    ) -> Result<Progress, Fatal> {
        let result = self.advance(now, raw, io);
        if result.is_err() {
            self.pending = None;
            self.state = State::Failed;
        }
        result
    }
}
