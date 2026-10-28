//! Bounded client for the existing M6c catalog protocol and Message320 ABI.
//!
//! The adapter supplies the epoch, service PID and domain from trusted startup
//! data and passes unmodified messages received from the kernel. This module
//! performs no syscalls and grants no authority: only the endpoint capability
//! and the service's kernel-bound caller checks authorize an operation.
//!
//! One request may be outstanding. Request IDs never wrap. A lost, malformed
//! or uncertain result closes the session; no request is replayed here. A new
//! session requires independently established service/recovery state, never
//! merely calling `new` to retry an uncertain mutation on the old endpoint.
use crate::wire::{self, Catalog, Header, Status};

pub const MESSAGE_SIZE: usize = 320;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation<'a> {
    Get {
        object: u64,
    },
    Put {
        object: u64,
        expected_revision: u64,
        name: &'a [u8],
        value: &'a [u8],
    },
    Delete {
        object: u64,
        expected_revision: u64,
    },
    Rename {
        object: u64,
        expected_revision: u64,
        name: &'a [u8],
    },
    List,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    MessageLength,
    Sender,
    Tag,
    PayloadLength,
    Capabilities,
    Padding,
    Reply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFailure {
    Timeout,
    Transport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Protocol(ProtocolError),
    Transport(TransportFailure),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidConfiguration,
    InvalidRequest(Status),
    Busy,
    Closed,
    NoPending,
    RequestIdsExhausted,
    /// `true` means the mutation may already have committed. It is never safe
    /// to convert this into an ordinary retry or a claim that nothing changed.
    Interrupted {
        reason: Failure,
        mutation_uncertain: bool,
    },
}

/// Canonical outgoing Message320. There are no transferred capabilities.
/// The adapter must not submit it again after successful IPC enqueue. A kernel
/// WouldBlock result may be retried only when enqueue is known not to occur.
#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    message: [u8; MESSAGE_SIZE],
}
impl Request {
    pub fn message(&self) -> &[u8; MESSAGE_SIZE] {
        &self.message
    }
    pub fn id(&self) -> u64 {
        wire::get64(&self.message, 72)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Document {
    pub object: u64,
    pub revision: u64,
    name_length: usize,
    value_length: usize,
    name: [u8; wire::NAME],
    value: [u8; wire::VALUE],
}
impl Document {
    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_length]
    }
    pub fn value(&self) -> &[u8] {
        &self.value[..self.value_length]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reply {
    Document(Document),
    Catalog(Catalog),
    Mutation {
        object: u64,
        revision: u64,
    },
    NotFound {
        object: u64,
        revision: u64,
    },
    /// A service error is not an empty document/catalog or a mutation ACK.
    /// Unknown and terminal I/O/protocol/recovery errors close the session.
    ServiceError(Status),
}

pub struct Client {
    epoch: u64,
    service_pid: u64,
    domain: usize,
    next_id: u64,
    pending: Option<Header>,
    closed: bool,
}
impl Client {
    pub fn new(epoch: u64, service_pid: u64, domain: usize) -> Result<Self, Error> {
        if epoch == 0 || service_pid == 0 || domain >= 2 {
            return Err(Error::InvalidConfiguration);
        }
        Ok(Self {
            epoch,
            service_pid,
            domain,
            next_id: 1,
            pending: None,
            closed: false,
        })
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Validate a bounded operation before issuing an ID or touching transport.
    /// Domain checking here catches caller mistakes; it is not an authority grant.
    pub fn begin(&mut self, operation: Operation<'_>) -> Result<Request, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        if self.pending.is_some() {
            return Err(Error::Busy);
        }
        if self.next_id == 0 {
            self.closed = true;
            return Err(Error::RequestIdsExhausted);
        }
        let (object, revision, op, name, value) = match operation {
            Operation::Get { object } => (object, 0, wire::GET, None, None),
            Operation::Put {
                object,
                expected_revision,
                name,
                value,
            } => (
                object,
                expected_revision,
                wire::PUT,
                Some(name),
                Some(value),
            ),
            Operation::Delete {
                object,
                expected_revision,
            } => (object, expected_revision, wire::DELETE, None, None),
            Operation::Rename {
                object,
                expected_revision,
                name,
            } => (object, expected_revision, wire::RENAME, Some(name), None),
            Operation::List => (self.domain as u64 * 2, 0, wire::LIST, None, None),
        };
        let payload = wire::request(self.next_id, self.epoch, object, revision, op, name, value);
        let header = wire::validate_request(&payload, self.epoch, self.domain)
            .map_err(Error::InvalidRequest)?;
        let mut message = [0; MESSAGE_SIZE];
        wire::put64(&mut message, 8, wire::TAG);
        wire::put32(&mut message, 16, wire::SIZE as u32);
        message[64..64 + wire::SIZE].copy_from_slice(&payload);
        self.next_id = self.next_id.checked_add(1).unwrap_or(0);
        self.pending = Some(header);
        Ok(Request { message })
    }

    /// Consume the original kernel-delivered Message320, including cap metadata.
    /// The adapter must close every unexpected received capability even when
    /// this method rejects the message. Do not erase cap fields before checking.
    pub fn receive(&mut self, message: &[u8]) -> Result<Reply, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        let request = self.pending.ok_or(Error::NoPending)?;
        if let Err(reason) = self.validate_envelope(message) {
            return Err(self.interrupt(Failure::Protocol(reason)));
        }
        let mut payload = [0; wire::SIZE];
        payload.copy_from_slice(&message[64..64 + wire::SIZE]);
        let header = match wire::validate_reply(&payload, request, self.epoch) {
            Ok(header) => header,
            Err(_) => return Err(self.interrupt(Failure::Protocol(ProtocolError::Reply))),
        };
        let reply = if header.status == Status::NotFound as i32 {
            Reply::NotFound {
                object: header.object,
                revision: header.revision,
            }
        } else if header.status != Status::Ok as i32 {
            let status = Status::from_code(i64::from(header.status));
            self.closed = matches!(
                status,
                Status::Unknown
                    | Status::Io
                    | Status::Timeout
                    | Status::NoDevice
                    | Status::Protocol
                    | Status::Corrupt
            );
            Reply::ServiceError(status)
        } else {
            match header.operation {
                wire::LIST => match wire::decode_catalog(&payload, request, self.epoch) {
                    Ok(catalog) => Reply::Catalog(catalog),
                    Err(_) => return Err(self.interrupt(Failure::Protocol(ProtocolError::Reply))),
                },
                wire::GET => {
                    let mut name = [0; wire::NAME];
                    name.copy_from_slice(&payload[192..224]);
                    let mut value = [0; wire::VALUE];
                    value.copy_from_slice(&payload[64..192]);
                    Reply::Document(Document {
                        object: header.object,
                        revision: header.revision,
                        name_length: header.name_length as usize,
                        value_length: header.length as usize,
                        name,
                        value,
                    })
                }
                wire::PUT | wire::DELETE | wire::RENAME => Reply::Mutation {
                    object: header.object,
                    revision: header.revision,
                },
                _ => return Err(self.interrupt(Failure::Protocol(ProtocolError::Reply))),
            }
        };
        self.pending = None;
        Ok(reply)
    }

    /// Stop using this session after an external timeout or transport failure.
    /// This does not cancel service I/O, undo a write or close a capability.
    /// The adapter retains ownership of endpoint/cap cleanup and recovery.
    pub fn transport_failed(&mut self, reason: TransportFailure) -> Error {
        if self.closed {
            return Error::Closed;
        }
        if self.pending.is_none() {
            return Error::NoPending;
        }
        self.interrupt(Failure::Transport(reason))
    }

    fn interrupt(&mut self, reason: Failure) -> Error {
        let mutation_uncertain = self.pending.is_some_and(|header| {
            matches!(header.operation, wire::PUT | wire::DELETE | wire::RENAME)
        });
        self.pending = None;
        self.closed = true;
        Error::Interrupted {
            reason,
            mutation_uncertain,
        }
    }

    fn validate_envelope(&self, message: &[u8]) -> Result<(), ProtocolError> {
        if message.len() != MESSAGE_SIZE {
            return Err(ProtocolError::MessageLength);
        }
        if wire::get64(message, 0) != self.service_pid {
            return Err(ProtocolError::Sender);
        }
        if wire::get64(message, 8) != wire::TAG {
            return Err(ProtocolError::Tag);
        }
        if wire::get32(message, 16) as usize != wire::SIZE {
            return Err(ProtocolError::PayloadLength);
        }
        if wire::get32(message, 20) != 0 {
            return Err(ProtocolError::Capabilities);
        }
        if message[24..64]
            .iter()
            .chain(message[64 + wire::SIZE..].iter())
            .any(|byte| *byte != 0)
        {
            return Err(ProtocolError::Padding);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_exhaustion_does_not_wrap_or_reopen() {
        let mut client = Client::new(1, 2, 0).unwrap();
        client.next_id = u64::MAX;
        let request = client.begin(Operation::Get { object: 0 }).unwrap();
        assert_eq!(request.id(), u64::MAX);
        let mut payload = [0; wire::SIZE];
        payload.copy_from_slice(&request.message()[64..64 + wire::SIZE]);
        let response = wire::reply(wire::header(&payload), 1, 0, Status::NotFound, None, None);
        let mut message = [0; MESSAGE_SIZE];
        wire::put64(&mut message, 0, 2);
        wire::put64(&mut message, 8, wire::TAG);
        wire::put32(&mut message, 16, wire::SIZE as u32);
        message[64..64 + wire::SIZE].copy_from_slice(&response);
        assert_eq!(
            client.receive(&message),
            Ok(Reply::NotFound {
                object: 0,
                revision: 0
            })
        );
        assert_eq!(
            client.begin(Operation::List),
            Err(Error::RequestIdsExhausted)
        );
        assert!(client.is_closed());
        assert_eq!(client.begin(Operation::List), Err(Error::Closed));
    }
}
