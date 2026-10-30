//! APPCTRL2/APC2-v1. SYS-v1 Ready/GO/STOP is intentionally a different tag.
use crate::{begin, counter, envelope, header, outgoing, put, value, Error, ReplayGuard, SIZE};
use nieuw_os_sdk_runtime::{Outgoing, Received};

pub const TAG: u64 = u64::from_le_bytes(*b"APPCTRL2");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Operation {
    Pause = 1,
    Resume = 2,
}
impl Operation {
    fn decode(value: u16) -> Result<Self, Error> {
        match value {
            1 => Ok(Self::Pause),
            2 => Ok(Self::Resume),
            _ => Err(Error::Operation),
        }
    }
}

/// Expected values from the recipient's controlled boot record, not authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    boot: u64,
    instance: u64,
    surface: u64,
}
impl Binding {
    pub fn new(boot: u64, instance: u64, surface: u64) -> Result<Self, Error> {
        Ok(Self {
            boot: counter(boot)?,
            instance: counter(instance)?,
            surface: counter(surface)?,
        })
    }
    pub const fn boot(self) -> u64 {
        self.boot
    }
    pub const fn instance(self) -> u64 {
        self.instance
    }
    pub const fn surface(self) -> u64 {
        self.surface
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    operation: Operation,
    id: u64,
    binding: Binding,
    activation: u64,
}
impl Request {
    pub fn new(
        operation: Operation,
        id: u64,
        binding: Binding,
        activation: u64,
    ) -> Result<Self, Error> {
        Ok(Self {
            operation,
            id: counter(id)?,
            binding,
            activation: counter(activation)?,
        })
    }
    pub const fn operation(self) -> Operation {
        self.operation
    }
    pub const fn id(self) -> u64 {
        self.id
    }
    pub const fn binding(self) -> Binding {
        self.binding
    }
    pub const fn activation(self) -> u64 {
        self.activation
    }
    fn bytes(self, reply: bool, frame: u64) -> [u8; SIZE] {
        let mut b = begin(
            b"APC2",
            self.operation as u16 | if reply { 0x8000 } else { 0 },
        );
        for (at, n) in [
            (8, self.id),
            (16, self.binding.boot),
            (24, self.binding.instance),
            (32, self.activation),
            (40, self.binding.surface),
            (48, frame),
        ] {
            put(&mut b, at, n);
        }
        b
    }
    pub fn outgoing(self) -> Outgoing {
        outgoing(TAG, &self.bytes(false, 0))
    }
    /// Encode only after actual quiescence. This codec performs no I/O itself.
    pub fn paused(self) -> Result<Outgoing, Error> {
        if self.operation != Operation::Pause {
            return Err(Error::Operation);
        }
        Ok(outgoing(TAG, &self.bytes(true, 0)))
    }
    /// Encode only after all current frame tile ACKs and actual Presented.
    pub fn presented(self, frame: u64) -> Result<Outgoing, Error> {
        if self.operation != Operation::Resume {
            return Err(Error::Operation);
        }
        Ok(outgoing(TAG, &self.bytes(true, counter(frame)?)))
    }
    pub fn expect(self, peer: u64, previous_presented_frame: u64) -> Result<Pending, Error> {
        if previous_presented_frame == u64::MAX {
            return Err(Error::Exhausted);
        }
        Ok(Pending {
            peer: counter(peer)?,
            request: self,
            previous_presented_frame,
            closed: false,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completion {
    Paused,
    Presented { frame: u64 },
}

/// One-shot reply binding. Invalid/lost replies never authorize another send.
pub struct Pending {
    peer: u64,
    request: Request,
    previous_presented_frame: u64,
    closed: bool,
}
impl Pending {
    pub fn abandon(&mut self) {
        self.closed = true;
    }
    pub fn receive(&mut self, message: &Received) -> Result<Completion, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        self.closed = true;
        let b = envelope(message, self.peer, TAG)?;
        if header(b, b"APC2")? != (self.request.operation as u16 | 0x8000) {
            return Err(Error::Operation);
        }
        if b[8..48] != self.request.bytes(false, 0)[8..48] {
            return Err(Error::Binding);
        }
        if value(b, 56) != 0 {
            return Err(Error::Reserved);
        }
        match self.request.operation {
            Operation::Pause if value(b, 48) == 0 => Ok(Completion::Paused),
            Operation::Resume => {
                let frame = counter(value(b, 48))?;
                if frame <= self.previous_presented_frame {
                    return Err(Error::Replay);
                }
                Ok(Completion::Presented { frame })
            }
            _ => Err(Error::Reserved),
        }
    }
}

pub struct Ingress {
    peer: u64,
    binding: Binding,
    ids: ReplayGuard,
    activation: u64,
    operation: Option<Operation>,
}
impl Ingress {
    pub fn new(init_pid: u64, binding: Binding) -> Result<Self, Error> {
        Ok(Self {
            peer: counter(init_pid)?,
            binding,
            ids: ReplayGuard::new(),
            activation: 0,
            operation: None,
        })
    }
    pub fn accept(&mut self, message: &Received) -> Result<Request, Error> {
        let b = envelope(message, self.peer, TAG)?;
        let operation = Operation::decode(header(b, b"APC2")?)?;
        if value(b, 16) != self.binding.boot
            || value(b, 24) != self.binding.instance
            || value(b, 40) != self.binding.surface
        {
            return Err(Error::Binding);
        }
        if b[48..].iter().any(|x| *x != 0) {
            return Err(Error::Reserved);
        }
        let activation = counter(value(b, 32))?;
        if activation < self.activation
            || (activation == self.activation
                && !(self.operation == Some(Operation::Pause) && operation == Operation::Resume))
        {
            return Err(Error::Replay);
        }
        self.ids.accept(value(b, 8))?;
        self.activation = activation;
        self.operation = Some(operation);
        Ok(Request {
            operation,
            id: value(b, 8),
            binding: self.binding,
            activation,
        })
    }
}
