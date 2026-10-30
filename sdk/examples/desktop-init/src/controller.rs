//! Test-only CPL3 controller; its own barrier is orchestration, not UI evidence.
use crate::wire::{self, StopRequest};
use nieuw_os_native_input_service::{discard_caps, Ipc};
use nieuw_os_sdk_runtime::{Handle, Outgoing, ReceiveEndpoint, SendEndpoint};
pub const MAGIC: u64 = u64::from_le_bytes(*b"NOSDCTL1");
pub const BARRIER_TICKS: u64 = 20_000;
pub const RPC_TICKS: u64 = 2_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Fatal {
    Configuration = 1,
    Clock = 2,
    Deadline = 3,
    Barrier = 4,
    Send = 5,
    Receive = 6,
    Capabilities = 7,
    Protocol = 8,
    Refused = 9,
    Failed = 10,
}
#[derive(Clone, Copy, Debug)]
pub struct Configuration {
    pub generation: u64,
    pub send: SendEndpoint,
    pub receive: ReceiveEndpoint,
    pub init_pid: u64,
    pub surface_epoch: u64,
}
impl Configuration {
    pub fn decode(w: &[u64; 128]) -> Result<Self, Fatal> {
        let at = |o: usize| w[o / 8];
        if at(32) != MAGIC
            || at(40) == 0
            || at(64) != 1
            || at(72) == 0
            || at(80) != 8
            || w[88 / 8..400 / 8].iter().any(|v| *v != 0)
            || w[464 / 8..].iter().any(|v| *v != 0)
        {
            return Err(Fatal::Configuration);
        }
        let send = Handle::from_raw(at(48)).map_err(|_| Fatal::Configuration)?;
        let receive = Handle::from_raw(at(56)).map_err(|_| Fatal::Configuration)?;
        if send == receive {
            return Err(Fatal::Configuration);
        }
        Ok(Self {
            generation: at(40),
            send: SendEndpoint::from_handle(send),
            receive: ReceiveEndpoint::from_handle(receive),
            init_pid: at(64),
            surface_epoch: at(72),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Phase {
    Barrier = 0,
    Send = 1,
    Wait = 2,
    Done = 3,
    Failed = 4,
}
pub struct Controller {
    config: Configuration,
    request: StopRequest,
    outgoing: Outgoing,
    phase: Phase,
    deadline: u64,
    last_tick: u64,
    start: u64,
    end: u64,
    sent: u64,
    validated: u64,
}
impl Controller {
    pub fn new(config: Configuration, now: u64) -> Result<Self, Fatal> {
        let request = StopRequest {
            id: 1,
            epoch: config.surface_epoch,
            generation: config.generation,
        };
        let outgoing =
            Outgoing::new(wire::CONTROL, &request.encode()).map_err(|_| Fatal::Configuration)?;
        Ok(Self {
            config,
            request,
            outgoing,
            phase: Phase::Barrier,
            deadline: now.checked_add(BARRIER_TICKS).ok_or(Fatal::Clock)?,
            last_tick: now,
            start: now,
            end: 0,
            sent: 0,
            validated: 0,
        })
    }
    pub const fn phase(&self) -> Phase {
        self.phase
    }
    pub const fn counts(&self) -> (u64, u64) {
        (self.sent, self.validated)
    }
    pub const fn ticks(&self) -> (u64, u64) {
        (self.start, self.end)
    }
    fn advance(&mut self, now: u64, barrier: u64, io: &mut impl Ipc) -> Result<(), Fatal> {
        if now < self.last_tick {
            return Err(Fatal::Clock);
        }
        self.last_tick = now;
        if self.phase == Phase::Done {
            return Ok(());
        }
        if self.phase == Phase::Failed {
            return Err(Fatal::Failed);
        }
        if now >= self.deadline {
            return Err(Fatal::Deadline);
        }
        if self.phase == Phase::Barrier {
            match barrier {
                0 => return Ok(()),
                1 => {
                    self.phase = Phase::Send;
                    self.deadline = now.checked_add(RPC_TICKS).ok_or(Fatal::Clock)?;
                }
                _ => return Err(Fatal::Barrier),
            }
        }
        if self.phase == Phase::Send {
            match io.send(self.config.send, &self.outgoing) {
                Ok(()) => {
                    self.phase = Phase::Wait;
                    self.sent = 1;
                }
                Err(e) if e.is_again() => return Ok(()),
                Err(_) => return Err(Fatal::Send),
            }
        }
        let m = match io.receive(self.config.receive) {
            Ok(m) => m,
            Err(e) if e.is_again() => return Ok(()),
            Err(_) => return Err(Fatal::Receive),
        };
        if m.cap_count() != 0 {
            discard_caps(io, &m).map_err(|_| Fatal::Capabilities)?;
            return Err(Fatal::Capabilities);
        }
        let b = m.payload();
        if m.sender() != self.config.init_pid
            || m.tag() != wire::CONTROL
            || b.len() != 48
            || &b[..4] != b"DIN1"
            || b[4..6] != 1u16.to_le_bytes()
            || b[6..8] != 0x8001u16.to_le_bytes()
            || b[8..32] != self.request.encode()[8..32]
            || b[40..].iter().any(|v| *v != 0)
        {
            return Err(Fatal::Protocol);
        }
        let status = u64::from_le_bytes(b[32..40].try_into().unwrap());
        match status {
            0 => {}
            1 => return Err(Fatal::Refused),
            2 => return Err(Fatal::Failed),
            _ => return Err(Fatal::Protocol),
        }
        self.validated = 1;
        self.end = now;
        self.phase = Phase::Done;
        Ok(())
    }
    pub fn step(&mut self, now: u64, barrier: u64, io: &mut impl Ipc) -> Result<Phase, Fatal> {
        if let Err(error) = self.advance(now, barrier, io) {
            self.phase = Phase::Failed;
            return Err(error);
        }
        Ok(self.phase)
    }
}
