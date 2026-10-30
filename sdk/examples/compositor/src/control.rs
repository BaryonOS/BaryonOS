use nieuw_os_input_core::wire::{self, Operation as InputOperation, Request as InputRequest};
use nieuw_os_native_input_service::{discard_caps, protocol::command_reply, Ipc};
use nieuw_os_sdk_runtime::{Outgoing, ReceiveEndpoint, Received, SendEndpoint};

pub const TAG: u64 = u64::from_le_bytes(*b"CMPCTRL1");
pub const SIZE: usize = 48;
pub const DEADLINE: u64 = 200;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Operation {
    FocusStart = 1,
    FocusStop = 2,
    Retire = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Status {
    Ok = 0,
    Malformed = 1,
    Access = 2,
    Binding = 3,
    Replay = 4,
    Busy = 5,
    Input = 6,
    Uncertain = 7,
    Retired = 8,
    Exhausted = 9,
    Display = 10,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum State {
    Inactive = 0,
    Focused = 1,
    Retired = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub operation: Operation,
    pub id: u64,
    pub epoch: u64,
    pub generation: u64,
}
fn number(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
impl Request {
    pub fn encode(self) -> [u8; SIZE] {
        let mut bytes = [0; SIZE];
        bytes[..4].copy_from_slice(b"CMP1");
        bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
        bytes[6..8].copy_from_slice(&(self.operation as u16).to_le_bytes());
        for (at, value) in [(8, self.id), (16, self.epoch), (24, self.generation)] {
            bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, Status> {
        if bytes.len() != SIZE
            || &bytes[..4] != b"CMP1"
            || u16::from_le_bytes([bytes[4], bytes[5]]) != 1
            || bytes[32..].iter().any(|x| *x != 0)
        {
            return Err(Status::Malformed);
        }
        let operation = match u16::from_le_bytes([bytes[6], bytes[7]]) {
            1 => Operation::FocusStart,
            2 => Operation::FocusStop,
            3 => Operation::Retire,
            _ => return Err(Status::Malformed),
        };
        let result = Self {
            operation,
            id: number(bytes, 8),
            epoch: number(bytes, 16),
            generation: number(bytes, 24),
        };
        if result.id == 0 || result.epoch == 0 || result.generation == 0 {
            return Err(Status::Malformed);
        }
        Ok(result)
    }
    fn reply(self, status: Status, state: State) -> Result<Outgoing, Status> {
        let mut bytes = self.encode();
        bytes[6..8].copy_from_slice(&(self.operation as u16 | 0x8000).to_le_bytes());
        bytes[32..40].copy_from_slice(&(status as u64).to_le_bytes());
        bytes[40..].copy_from_slice(
            &(if status == Status::Ok {
                state as u64
            } else {
                0
            })
            .to_le_bytes(),
        );
        Outgoing::new(TAG, &bytes).map_err(|_| Status::Malformed)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Configuration {
    pub init_pid: u64,
    pub surface_epoch: u64,
    pub generation: u64,
    pub policy_pid: u64,
    pub policy_epoch: u64,
    pub client_id: u64,
    pub send: SendEndpoint,
    pub receive: ReceiveEndpoint,
}
struct Focus {
    request: InputRequest,
    outgoing: Outgoing,
    sent: bool,
    started: u64,
    control: Request,
}
pub struct Coordinator {
    config: Configuration,
    last_control: u64,
    next_focus: u64,
    last_tick: u64,
    state: State,
    pending: Option<Focus>,
    reply: Option<Outgoing>,
    reply_at: u64,
    terminal: bool,
    retire_requested: bool,
    sent: u64,
    acked: u64,
}
impl Coordinator {
    pub fn new(config: Configuration) -> Result<Self, Status> {
        if [
            config.init_pid,
            config.surface_epoch,
            config.generation,
            config.policy_pid,
            config.policy_epoch,
            config.client_id,
        ]
        .contains(&0)
            || config.init_pid == config.policy_pid
            || config.send.handle() == config.receive.handle()
        {
            return Err(Status::Binding);
        }
        Ok(Self {
            config,
            last_control: 0,
            next_focus: 1,
            last_tick: 0,
            state: State::Inactive,
            pending: None,
            reply: None,
            reply_at: 0,
            terminal: false,
            retire_requested: false,
            sent: 0,
            acked: 0,
        })
    }
    pub const fn state(&self) -> State {
        self.state
    }
    pub const fn is_terminal(&self) -> bool {
        self.terminal
    }
    pub const fn is_busy(&self) -> bool {
        self.pending.is_some() || self.reply.is_some()
    }
    pub const fn retire_requested(&self) -> bool {
        self.retire_requested
    }
    pub const fn counts(&self) -> (u64, u64) {
        (self.sent, self.acked)
    }
    pub fn accept(&mut self, now: u64, message: &Received) -> Result<bool, Status> {
        if self.terminal {
            return Err(Status::Uncertain);
        }
        if message.sender() != self.config.init_pid
            || message.tag() != TAG
            || message.cap_count() != 0
        {
            return Err(Status::Access);
        }
        let request = Request::decode(message.payload())?;
        if request.epoch != self.config.surface_epoch
            || request.generation != self.config.generation
        {
            return Err(Status::Binding);
        }
        if self.is_busy() {
            return Err(Status::Busy);
        }
        if request.id <= self.last_control {
            return Err(Status::Replay);
        }
        if request.id == u64::MAX {
            self.terminal = true;
            return Err(Status::Exhausted);
        }
        self.last_control = request.id;
        if self.state == State::Retired {
            self.reply = Some(request.reply(Status::Retired, self.state)?);
            self.reply_at = now;
            return Ok(false);
        }
        let focus = self.next_focus;
        self.next_focus = focus.checked_add(1).ok_or(Status::Exhausted)?;
        let input = InputRequest {
            operation: InputOperation::Focus,
            id: focus,
            epoch: self.config.policy_epoch,
            arg0: if request.operation == Operation::FocusStart {
                self.config.client_id
            } else {
                0
            },
            arg1: 0,
        };
        let outgoing = Outgoing::new(wire::TAG, &input.encode()).map_err(|_| Status::Malformed)?;
        self.retire_requested = request.operation == Operation::Retire;
        self.pending = Some(Focus {
            request: input,
            outgoing,
            sent: false,
            started: now,
            control: request,
        });
        Ok(self.retire_requested)
    }
    /// One real IPC attempt/wait. A published Focus request is never replayed.
    pub fn step(&mut self, now: u64, io: &mut impl Ipc) -> Result<bool, Status> {
        let result = self.advance(now, io);
        if result.is_err() {
            self.terminal = true;
            self.pending = None;
            self.reply = None;
        }
        result
    }
    fn advance(&mut self, now: u64, io: &mut impl Ipc) -> Result<bool, Status> {
        if self.terminal {
            return Err(Status::Uncertain);
        }
        if now < self.last_tick {
            return Err(Status::Uncertain);
        }
        self.last_tick = now;
        let Some(focus) = &mut self.pending else {
            return Ok(false);
        };
        if now.saturating_sub(focus.started) > DEADLINE {
            return Err(Status::Uncertain);
        }
        if !focus.sent {
            match io.send(self.config.send, &focus.outgoing) {
                Ok(()) => {
                    focus.sent = true;
                    self.sent = self.sent.saturating_add(1);
                }
                Err(error) if error.is_again() => return Ok(false),
                Err(_) => return Err(Status::Uncertain),
            }
        }
        let message = match io.receive(self.config.receive) {
            Ok(message) => message,
            Err(error) if error.is_again() => return Ok(false),
            Err(_) => return Err(Status::Uncertain),
        };
        if message.cap_count() != 0 {
            discard_caps(io, &message).map_err(|_| Status::Uncertain)?;
            return Err(Status::Uncertain);
        }
        let status = command_reply(&message, self.config.policy_pid, focus.request)
            .map_err(|_| Status::Uncertain)?;
        if status != 0 {
            return Err(Status::Input);
        }
        self.state = match focus.control.operation {
            Operation::FocusStart => State::Focused,
            Operation::FocusStop => State::Inactive,
            Operation::Retire => State::Retired,
        };
        self.reply = Some(focus.control.reply(Status::Ok, self.state)?);
        self.reply_at = now;
        self.pending = None;
        self.acked = self.acked.saturating_add(1);
        Ok(true)
    }
    pub fn flush(
        &mut self,
        now: u64,
        registry: SendEndpoint,
        io: &mut impl Ipc,
    ) -> Result<(), Status> {
        if self.reply.is_some() && now.saturating_sub(self.reply_at) > DEADLINE {
            self.terminal = true;
            self.reply = None;
            return Err(Status::Uncertain);
        }
        if let Some(reply) = &self.reply {
            match io.send(registry, reply) {
                Ok(()) => self.reply = None,
                Err(error) if error.is_again() => {}
                Err(_) => {
                    self.terminal = true;
                    return Err(Status::Uncertain);
                }
            }
        }
        Ok(())
    }
}
