//! Fixed boot-preset app selection, never a PID/endpoint/program-path grant.
use crate::{
    app::Binding, begin, counter, envelope, header, outgoing, put, value, Error, ReplayGuard, SIZE,
};
use nieuw_os_sdk_runtime::{Outgoing, Received};

pub const TAG: u64 = u64::from_le_bytes(*b"DESKSWC1");
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum App {
    Materiaal = 1,
    Calculator = 2,
}
impl App {
    fn decode(n: u64) -> Result<Self, Error> {
        match n {
            1 => Ok(Self::Materiaal),
            2 => Ok(Self::Calculator),
            _ => Err(Error::Binding),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pid: u64,
    binding: Binding,
}
impl Identity {
    pub fn new(pid: u64, binding: Binding) -> Result<Self, Error> {
        Ok(Self {
            pid: counter(pid)?,
            binding,
        })
    }
    pub const fn pid(self) -> u64 {
        self.pid
    }
    pub const fn binding(self) -> Binding {
        self.binding
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preset {
    entries: [Identity; 2],
}
impl Preset {
    pub fn new(materiaal: Identity, calculator: Identity) -> Result<Self, Error> {
        if materiaal.pid == calculator.pid
            || materiaal.binding.boot() != calculator.binding.boot()
            || materiaal.binding.surface() == calculator.binding.surface()
        {
            return Err(Error::Binding);
        }
        Ok(Self {
            entries: [materiaal, calculator],
        })
    }
    pub const fn boot(self) -> u64 {
        self.entries[0].binding.boot()
    }
    pub fn identity(self, app: App) -> Identity {
        self.entries[app as usize - 1]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Failure {
    Busy = 1,
    Denied = 2,
    Failed = 3,
}
impl Failure {
    fn decode(n: u64) -> Result<Self, Error> {
        match n {
            1 => Ok(Self::Busy),
            2 => Ok(Self::Denied),
            3 => Ok(Self::Failed),
            _ => Err(Error::Reserved),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Completed {
        activation: u64,
        surface: u64,
        frame: u64,
    },
    Rejected(Failure),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    id: u64,
    app: App,
    preset: Preset,
}
impl Request {
    pub fn new(id: u64, app: App, preset: Preset) -> Result<Self, Error> {
        Ok(Self {
            id: counter(id)?,
            app,
            preset,
        })
    }
    pub const fn id(self) -> u64 {
        self.id
    }
    pub const fn app(self) -> App {
        self.app
    }
    pub fn identity(self) -> Identity {
        self.preset.identity(self.app)
    }
    fn bytes(self, reply: bool) -> [u8; SIZE] {
        let mut b = begin(b"DSW1", if reply { 0x8001 } else { 1 });
        for (at, n) in [
            (8, self.id),
            (16, self.preset.boot()),
            (24, self.app as u64),
        ] {
            put(&mut b, at, n);
        }
        b
    }
    pub fn outgoing(self) -> Outgoing {
        outgoing(TAG, &self.bytes(false))
    }
    /// Call only after native focus ACK, app Resume ACK and actual full blits.
    pub fn completed(self, activation: u64, frame: u64) -> Result<Outgoing, Error> {
        let mut b = self.bytes(true);
        put(&mut b, 32, counter(activation)?);
        put(&mut b, 40, self.identity().binding.surface());
        put(&mut b, 48, counter(frame)?);
        Ok(outgoing(TAG, &b))
    }
    pub fn rejected(self, failure: Failure) -> Outgoing {
        let mut b = self.bytes(true);
        put(&mut b, 56, failure as u64);
        outgoing(TAG, &b)
    }
    pub fn expect(
        self,
        init_pid: u64,
        previous_activation: u64,
        previous_target_frame: u64,
    ) -> Result<Pending, Error> {
        if previous_activation == u64::MAX || previous_target_frame == u64::MAX {
            return Err(Error::Exhausted);
        }
        Ok(Pending {
            peer: counter(init_pid)?,
            request: self,
            previous_activation,
            previous_target_frame,
            closed: false,
        })
    }
}
pub struct Pending {
    peer: u64,
    request: Request,
    previous_activation: u64,
    previous_target_frame: u64,
    closed: bool,
}
impl Pending {
    pub fn abandon(&mut self) {
        self.closed = true;
    }
    pub fn receive(&mut self, message: &Received) -> Result<Outcome, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        self.closed = true;
        let b = envelope(message, self.peer, TAG)?;
        if header(b, b"DSW1")? != 0x8001 {
            return Err(Error::Operation);
        }
        if b[8..32] != self.request.bytes(false)[8..32] {
            return Err(Error::Binding);
        }
        if value(b, 56) == 0 {
            if value(b, 40) != self.request.identity().binding.surface() {
                return Err(Error::Binding);
            }
            let activation = counter(value(b, 32))?;
            let frame = counter(value(b, 48))?;
            if activation <= self.previous_activation || frame <= self.previous_target_frame {
                return Err(Error::Replay);
            }
            Ok(Outcome::Completed {
                activation,
                surface: value(b, 40),
                frame,
            })
        } else {
            let failure = Failure::decode(value(b, 56))?;
            if b[32..56].iter().any(|n| *n != 0) {
                return Err(Error::Reserved);
            }
            Ok(Outcome::Rejected(failure))
        }
    }
}
pub struct Ingress {
    peer: u64,
    preset: Preset,
    ids: ReplayGuard,
}
impl Ingress {
    pub fn new(compositor_pid: u64, preset: Preset) -> Result<Self, Error> {
        let peer = counter(compositor_pid)?;
        if preset.entries.iter().any(|e| e.pid == peer) {
            return Err(Error::Binding);
        }
        Ok(Self {
            peer,
            preset,
            ids: ReplayGuard::new(),
        })
    }
    pub fn accept(&mut self, message: &Received) -> Result<Request, Error> {
        let b = envelope(message, self.peer, TAG)?;
        if header(b, b"DSW1")? != 1 {
            return Err(Error::Operation);
        }
        if value(b, 16) != self.preset.boot() {
            return Err(Error::Binding);
        }
        let app = App::decode(value(b, 24))?;
        if b[32..].iter().any(|n| *n != 0) {
            return Err(Error::Reserved);
        }
        self.ids.accept(value(b, 8))?;
        Ok(Request {
            id: value(b, 8),
            app,
            preset: self.preset,
        })
    }
}
