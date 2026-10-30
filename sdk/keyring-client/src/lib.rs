#![no_std]
#![forbid(unsafe_code)]
//! An allocation-free KEY1 client for an already granted, exclusively owned
//! endpoint pair. Keys remain in the isolated Key Service; this crate exposes
//! no recovery/snapshot opcodes and grants no domain/object authority.
use core::fmt;
pub use keyring_core::records::{RecordContext, RecordKind};
pub use keyring_core::{health::HealthFailure, State, Status, MAX_PLAINTEXT, MAX_SEALED};
use keyring_core::{wire, DOMAINS, KEYS_PER_DOMAIN, NONCE_BYTES, TAG_BYTES};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use zeroize::{Zeroize, Zeroizing};

pub const MAX_POLL_BUDGET: usize = 1_000_000;

/// EAGAIN on send must mean nothing was enqueued. `Received` owns any actual
/// kernel-delivered handles until this client explicitly closes them.
pub trait Transport {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError>;
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError>;
    fn yield_now(&mut self) -> Result<(), RuntimeError>;
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError>;
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub struct NativeTransport;
#[cfg(all(target_arch = "x86_64", target_os = "none"))]
impl Transport for NativeTransport {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        nieuw_os_sdk_runtime::native::send(endpoint, message)
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        nieuw_os_sdk_runtime::native::receive(endpoint)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        nieuw_os_sdk_runtime::native::yield_now()
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        nieuw_os_sdk_runtime::native::close(handle)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyIndex(u8);
impl KeyIndex {
    /// Representation check only: the service still checks creation/revocation.
    pub const fn new(index: u8) -> Result<Self, Error> {
        if (index as usize) < KEYS_PER_DOMAIN {
            Ok(Self(index))
        } else {
            Err(Error::InvalidKey)
        }
    }
    pub const fn raw(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    NotEnqueued,
    Enqueued,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    NotPerformed,
    MayHaveCompleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidConfiguration,
    InvalidKey,
    InvalidInput,
    WrongDomain,
    Busy,
    NoPending,
    Closed,
    RequestIdsExhausted {
        cleanup: Option<RuntimeError>,
    },
    Service(Status),
    Interrupted {
        delivery: Delivery,
        outcome: Outcome,
        transport: Option<RuntimeError>,
        cleanup: Option<RuntimeError>,
    },
    Protocol {
        outcome: Outcome,
        cleanup: Option<RuntimeError>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cancelled {
    pub delivery: Delivery,
    pub outcome: Outcome,
    pub cleanup: Option<RuntimeError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceStatus {
    pub state: State,
    pub reason: HealthFailure,
    pub keys_created: [u16; DOMAINS],
    pub entropy_bytes: u64,
}

/// Decrypted caller-authorized bytes. No Clone/Copy/Debug of the contents;
/// the owned buffer is zeroized on drop. Caller copies remain its obligation.
pub struct Plaintext {
    bytes: Zeroizing<[u8; MAX_PLAINTEXT]>,
    length: usize,
}
impl Plaintext {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
}
impl fmt::Debug for Plaintext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Plaintext")
            .field("length", &self.length)
            .finish_non_exhaustive()
    }
}

pub struct Sealed {
    bytes: [u8; MAX_SEALED],
    length: usize,
}
impl Sealed {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
}
impl fmt::Debug for Sealed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sealed")
            .field("length", &self.length)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum Reply {
    Status(ServiceStatus),
    Generated(KeyIndex),
    Revoked,
    Sealed(Sealed),
    Plaintext(Plaintext),
}

/// The record context is a requested identity, not a permission grant. Its
/// domain must match expected_domain locally; the service derives the actual
/// authoritative domain from its ingress, since no domain is on the wire.
pub enum Operation<'a> {
    Status,
    Generate,
    Revoke(KeyIndex),
    Seal {
        key: KeyIndex,
        plaintext: &'a [u8],
    },
    Unseal {
        key: KeyIndex,
        sealed: &'a [u8],
    },
    SealRecord {
        key: KeyIndex,
        context: RecordContext,
        plaintext: &'a [u8],
    },
    UnsealRecord {
        key: KeyIndex,
        context: RecordContext,
        sealed: &'a [u8],
    },
}

#[derive(Clone, Copy)]
enum Expected {
    Status,
    Generated,
    Revoked,
    Sealed(usize),
    Plaintext(usize),
}

enum InvalidReply {
    Malformed,
    Refused(Status),
}
impl From<()> for InvalidReply {
    fn from(_: ()) -> Self {
        Self::Malformed
    }
}
impl Expected {
    fn outcome(self, delivery: Delivery) -> Outcome {
        if delivery != Delivery::NotEnqueued {
            Outcome::MayHaveCompleted
        } else {
            Outcome::NotPerformed
        }
    }
}

struct Pending {
    request: Zeroizing<[u8; wire::SIZE]>,
    header: wire::Header,
    expected: Expected,
    delivery: Delivery,
    remaining: usize,
}

#[derive(Debug)]
pub enum Poll {
    Pending { delivery: Delivery },
    Ready(Reply),
}

pub struct Session<T> {
    transport: T,
    service_pid: u64,
    expected_domain: usize,
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    budget: usize,
    next_id: u64,
    pending: Option<Pending>,
    closed: bool,
    uncertain: bool,
    last_status: Option<ServiceStatus>,
    minimum_created: [u16; DOMAINS],
}

impl<T: Transport> Session<T> {
    /// Transfers exclusive ownership of the endpoint pair. The caller must
    /// obtain it from the appropriate broker; these values cannot verify a
    /// kernel grant's domain or add any authority on their own.
    pub fn new(
        transport: T,
        service_pid: u64,
        expected_domain: usize,
        send: SendEndpoint,
        receive: ReceiveEndpoint,
        budget: usize,
    ) -> Result<Self, Error> {
        if service_pid == 0
            || expected_domain >= DOMAINS
            || send.handle() == receive.handle()
            || budget == 0
            || budget > MAX_POLL_BUDGET
        {
            return Err(Error::InvalidConfiguration);
        }
        Ok(Self {
            transport,
            service_pid,
            expected_domain,
            send,
            receive,
            budget,
            next_id: 1,
            pending: None,
            closed: false,
            uncertain: false,
            last_status: None,
            minimum_created: [0; DOMAINS],
        })
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    pub fn is_uncertain(&self) -> bool {
        self.uncertain
    }
    pub fn is_busy(&self) -> bool {
        self.pending.is_some()
    }
    pub fn transport(&self) -> &T {
        &self.transport
    }

    fn close_endpoints(&mut self) -> Option<RuntimeError> {
        if self.closed {
            return None;
        }
        self.closed = true;
        let send = self.transport.close(self.send.handle());
        let receive = self.transport.close(self.receive.handle());
        send.err().or(receive.err())
    }

    /// Close/destroy pending plaintext but never pretend to unsend KEY1.
    pub fn close(&mut self) -> Result<(), RuntimeError> {
        if let Some(p) = self.pending.take() {
            self.uncertain |= p.expected.outcome(p.delivery) == Outcome::MayHaveCompleted;
        }
        self.close_endpoints().map_or(Ok(()), Err)
    }

    fn interrupted(&mut self, pending: Pending, transport: Option<RuntimeError>) -> Error {
        let outcome = pending.expected.outcome(pending.delivery);
        self.uncertain |= outcome == Outcome::MayHaveCompleted;
        Error::Interrupted {
            delivery: pending.delivery,
            outcome,
            transport,
            cleanup: self.close_endpoints(),
        }
    }
    fn protocol(&mut self, expected: Expected, prior_cleanup: Option<RuntimeError>) -> Error {
        let outcome = expected.outcome(Delivery::Enqueued);
        self.uncertain |= outcome == Outcome::MayHaveCompleted;
        let endpoints = self.close_endpoints();
        Error::Protocol {
            outcome,
            cleanup: prior_cleanup.or(endpoints),
        }
    }

    pub fn begin(&mut self, operation: Operation<'_>) -> Result<u64, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        if self.pending.is_some() {
            return Err(Error::Busy);
        }
        let id = self.next_id;
        let (op, index, body, expected) = match operation {
            Operation::Status => (
                wire::STATUS,
                0,
                Zeroizing::new([0; wire::BODY]),
                Expected::Status,
            ),
            Operation::Generate => (
                wire::GENERATE,
                0,
                Zeroizing::new([0; wire::BODY]),
                Expected::Generated,
            ),
            Operation::Revoke(key) => (
                wire::REVOKE,
                key.raw(),
                Zeroizing::new([0; wire::BODY]),
                Expected::Revoked,
            ),
            Operation::Seal { key, plaintext } => {
                if plaintext.len() > MAX_PLAINTEXT {
                    return Err(Error::InvalidInput);
                }
                let mut body = Zeroizing::new([0; wire::BODY]);
                body[..plaintext.len()].copy_from_slice(plaintext);
                (
                    wire::SEAL,
                    key.raw(),
                    body,
                    Expected::Sealed(plaintext.len() + NONCE_BYTES + TAG_BYTES),
                )
            }
            Operation::Unseal { key, sealed } => {
                if !(NONCE_BYTES + TAG_BYTES..=MAX_SEALED).contains(&sealed.len()) {
                    return Err(Error::InvalidInput);
                }
                let mut body = Zeroizing::new([0; wire::BODY]);
                body[..sealed.len()].copy_from_slice(sealed);
                (
                    wire::UNSEAL,
                    key.raw(),
                    body,
                    Expected::Plaintext(sealed.len() - NONCE_BYTES - TAG_BYTES),
                )
            }
            Operation::SealRecord {
                key,
                context,
                plaintext,
            } => {
                self.record(context)?;
                if plaintext.len() != usize::from(context.length) {
                    return Err(Error::InvalidInput);
                }
                let request = Zeroizing::new(
                    wire::record_request(wire::SEAL_RECORD, key.raw(), id, context, plaintext)
                        .map_err(|_| Error::InvalidInput)?,
                );
                return self.install(
                    request,
                    Expected::Sealed(plaintext.len() + NONCE_BYTES + TAG_BYTES),
                );
            }
            Operation::UnsealRecord {
                key,
                context,
                sealed,
            } => {
                self.record(context)?;
                let request = Zeroizing::new(
                    wire::record_request(wire::UNSEAL_RECORD, key.raw(), id, context, sealed)
                        .map_err(|_| Error::InvalidInput)?,
                );
                return self.install(request, Expected::Plaintext(usize::from(context.length)));
            }
        };
        let length = match expected {
            Expected::Status | Expected::Generated | Expected::Revoked => 0,
            Expected::Sealed(n) => n - NONCE_BYTES - TAG_BYTES,
            Expected::Plaintext(n) => n + NONCE_BYTES + TAG_BYTES,
        };
        let request = Zeroizing::new(
            wire::request(op, index, id, &body[..length]).ok_or(Error::InvalidInput)?,
        );
        self.install(request, expected)
    }

    fn record(&self, context: RecordContext) -> Result<(), Error> {
        if usize::from(context.domain) != self.expected_domain {
            return Err(Error::WrongDomain);
        }
        wire::encode_record_context(context)
            .map(|_| ())
            .map_err(|_| Error::InvalidInput)
    }

    fn install(
        &mut self,
        request: Zeroizing<[u8; wire::SIZE]>,
        expected: Expected,
    ) -> Result<u64, Error> {
        let Some(next) = self.next_id.checked_add(1) else {
            return Err(Error::RequestIdsExhausted {
                cleanup: self.close_endpoints(),
            });
        };
        let header = wire::Header::decode(&*request).ok_or(Error::InvalidInput)?;
        self.next_id = next;
        self.pending = Some(Pending {
            request,
            header,
            expected,
            delivery: Delivery::NotEnqueued,
            remaining: self.budget,
        });
        Ok(header.request)
    }

    /// At most one IPC attempt (plus one yield after EAGAIN). The request and
    /// receive phases each have a fixed budget across calls, not per call.
    pub fn poll(&mut self) -> Result<Poll, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        let mut p = self.pending.take().ok_or(Error::NoPending)?;
        if p.delivery == Delivery::NotEnqueued {
            let message = Outgoing::new(wire::TAG, &*p.request).map_err(|_| Error::InvalidInput)?;
            match self.transport.send(self.send, &message) {
                Ok(()) => {
                    p.request.zeroize();
                    p.delivery = Delivery::Enqueued;
                    p.remaining = self.budget;
                }
                Err(e) if e.is_again() => {
                    p.remaining -= 1;
                    if let Err(e) = self.transport.yield_now() {
                        return Err(self.interrupted(p, Some(e)));
                    }
                    if p.remaining == 0 {
                        return Err(self.interrupted(p, None));
                    }
                }
                Err(e) => {
                    if !matches!(e, RuntimeError::Kernel(code) if code < 0) {
                        p.delivery = Delivery::Unknown;
                    }
                    return Err(self.interrupted(p, Some(e)));
                }
            }
            let delivery = p.delivery;
            self.pending = Some(p);
            return Ok(Poll::Pending { delivery });
        }
        match self.transport.receive(self.receive) {
            Ok(message) => {
                let mut cleanup = None;
                for i in 0..message.cap_count() {
                    if let Some(cap) = message.cap(i) {
                        if let Err(e) = self.transport.close(cap.handle) {
                            cleanup.get_or_insert(e);
                        }
                    }
                }
                if message.cap_count() != 0
                    || message.sender() != self.service_pid
                    || message.tag() != wire::TAG
                    || message.payload().len() != wire::SIZE
                {
                    return Err(self.protocol(p.expected, cleanup));
                }
                let mut body = Zeroizing::new([0; wire::SIZE]);
                body.copy_from_slice(message.payload());
                let reply = match self.validate(&p, &body) {
                    Ok(reply) => reply,
                    Err(InvalidReply::Refused(status)) => return Err(Error::Service(status)),
                    Err(InvalidReply::Malformed) => return Err(self.protocol(p.expected, None)),
                };
                Ok(Poll::Ready(reply))
            }
            Err(e) if e.is_again() => {
                p.remaining -= 1;
                if let Err(e) = self.transport.yield_now() {
                    return Err(self.interrupted(p, Some(e)));
                }
                if p.remaining == 0 {
                    return Err(self.interrupted(p, None));
                }
                let delivery = p.delivery;
                self.pending = Some(p);
                Ok(Poll::Pending { delivery })
            }
            Err(e) => Err(self.interrupted(p, Some(e))),
        }
    }

    pub fn cancel(&mut self) -> Result<Cancelled, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        let p = self.pending.take().ok_or(Error::NoPending)?;
        let outcome = p.expected.outcome(p.delivery);
        self.uncertain |= outcome == Outcome::MayHaveCompleted;
        let cleanup = if p.delivery == Delivery::NotEnqueued {
            None
        } else {
            self.close_endpoints()
        };
        Ok(Cancelled {
            delivery: p.delivery,
            outcome,
            cleanup,
        })
    }

    pub fn exchange(&mut self, operation: Operation<'_>) -> Result<Reply, Error> {
        self.begin(operation)?;
        loop {
            if let Poll::Ready(reply) = self.poll()? {
                return Ok(reply);
            }
        }
    }

    fn validate(
        &mut self,
        pending: &Pending,
        body: &[u8; wire::SIZE],
    ) -> Result<Reply, InvalidReply> {
        let h = wire::Header::decode(body).ok_or(())?;
        if h.op != pending.header.op | wire::REPLY
            || h.index != pending.header.index
            || h.request != pending.header.request
        {
            return Err(().into());
        }
        let status = match h.status {
            0 => Status::Ok,
            -1 => Status::NotReady,
            -2 => Status::Quota,
            -3 => Status::BadHandle,
            -4 => Status::Revoked,
            -5 => Status::Auth,
            -6 => Status::Invalid,
            -7 => Status::Degraded,
            -8 => Status::Entropy,
            _ => return Err(().into()),
        };
        let data = &body[wire::HEADER..wire::HEADER + usize::from(h.length)];
        if status != Status::Ok {
            if !data.is_empty() {
                return Err(().into());
            }
            // A separate Result layer preserves a canonical service refusal
            // without treating it as malformed wire or a successful reply.
            return Err(InvalidReply::Refused(status));
        }
        match pending.expected {
            Expected::Status => {
                let status = self.status_body(data)?;
                self.last_status = Some(status);
                self.minimum_created = status.keys_created;
                Ok(Reply::Status(status))
            }
            Expected::Generated if data.len() == 1 => {
                let key = KeyIndex::new(data[0]).map_err(|_| ())?;
                if u16::from(key.raw()) < self.minimum_created[self.expected_domain] {
                    return Err(().into());
                }
                self.minimum_created[self.expected_domain] = u16::from(key.raw()) + 1;
                Ok(Reply::Generated(key))
            }
            Expected::Revoked if data.is_empty() => Ok(Reply::Revoked),
            Expected::Sealed(length) if data.len() == length => {
                let mut bytes = [0; MAX_SEALED];
                bytes[..length].copy_from_slice(data);
                Ok(Reply::Sealed(Sealed { bytes, length }))
            }
            Expected::Plaintext(length) if data.len() == length => {
                let mut bytes = Zeroizing::new([0; MAX_PLAINTEXT]);
                bytes[..length].copy_from_slice(data);
                Ok(Reply::Plaintext(Plaintext { bytes, length }))
            }
            _ => Err(().into()),
        }
    }

    fn status_body(&self, data: &[u8]) -> Result<ServiceStatus, ()> {
        if data.len() != 14 {
            return Err(());
        }
        let state = match data[0] {
            0 => State::Starting,
            1 => State::Ready,
            2 => State::Blocked,
            3 => State::Degraded,
            _ => return Err(()),
        };
        let reason = match data[1] {
            0 => HealthFailure::None,
            1 => HealthFailure::Source,
            2 => HealthFailure::Repetition,
            3 => HealthFailure::Proportion,
            _ => return Err(()),
        };
        let keys_created = [
            u16::from_le_bytes([data[2], data[3]]),
            u16::from_le_bytes([data[4], data[5]]),
        ];
        let entropy_bytes = u64::from_le_bytes(data[6..14].try_into().map_err(|_| ())?);
        if keys_created
            .iter()
            .any(|n| usize::from(*n) > KEYS_PER_DOMAIN)
            || keys_created
                .iter()
                .zip(self.minimum_created)
                .any(|(n, min)| *n < min)
            || entropy_bytes % keyring_core::RESEED_ENTROPY as u64 != 0
            || self
                .last_status
                .is_some_and(|old| entropy_bytes < old.entropy_bytes)
            || match state {
                State::Starting => {
                    reason != HealthFailure::None
                        || keys_created != [0; DOMAINS]
                        || entropy_bytes != 0
                }
                State::Ready => {
                    reason != HealthFailure::None
                        || entropy_bytes < keyring_core::STARTUP_ENTROPY as u64
                }
                State::Blocked => reason == HealthFailure::None || keys_created != [0; DOMAINS],
                State::Degraded => reason == HealthFailure::None,
            }
        {
            return Err(());
        }
        Ok(ServiceStatus {
            state,
            reason,
            keys_created,
            entropy_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Port {
        sends: usize,
        closes: usize,
    }
    impl Transport for Port {
        fn send(&mut self, _: SendEndpoint, _: &Outgoing) -> Result<(), RuntimeError> {
            self.sends += 1;
            Ok(())
        }
        fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
            Err(RuntimeError::Kernel(-8))
        }
        fn yield_now(&mut self) -> Result<(), RuntimeError> {
            Ok(())
        }
        fn close(&mut self, _: Handle) -> Result<(), RuntimeError> {
            self.closes += 1;
            if self.closes == 1 {
                Err(RuntimeError::Kernel(-9))
            } else {
                Ok(())
            }
        }
    }
    fn session() -> Session<Port> {
        Session::new(
            Port::default(),
            2,
            0,
            SendEndpoint::from_handle(Handle::from_raw((1 << 32) | 1).unwrap()),
            ReceiveEndpoint::from_handle(Handle::from_raw((1 << 32) | 2).unwrap()),
            2,
        )
        .unwrap()
    }

    #[test]
    fn owned_plaintext_request_is_zeroized_immediately_after_enqueue() {
        let mut s = session();
        let key = KeyIndex::new(0).unwrap();
        s.begin(Operation::Seal {
            key,
            plaintext: b"client secret",
        })
        .unwrap();
        assert!(s
            .pending
            .as_ref()
            .unwrap()
            .request
            .windows(13)
            .any(|b| b == b"client secret"));
        assert!(matches!(
            s.poll(),
            Ok(Poll::Pending {
                delivery: Delivery::Enqueued
            })
        ));
        assert!(s.pending.as_ref().unwrap().request.iter().all(|b| *b == 0));
    }

    #[test]
    fn identifier_exhaustion_closes_both_handles_without_publication_or_wrap() {
        let mut s = session();
        s.next_id = u64::MAX;
        assert_eq!(
            s.begin(Operation::Generate),
            Err(Error::RequestIdsExhausted {
                cleanup: Some(RuntimeError::Kernel(-9))
            })
        );
        assert_eq!(s.transport.sends, 0);
        assert_eq!(s.transport.closes, 2);
        assert_eq!(s.next_id, u64::MAX);
        assert!(s.is_closed());
        assert_eq!(s.begin(Operation::Generate), Err(Error::Closed));
    }

    #[test]
    fn close_attempts_both_handles_once_even_when_first_close_fails() {
        let mut s = session();
        assert_eq!(s.close(), Err(RuntimeError::Kernel(-9)));
        assert_eq!(s.transport.closes, 2);
        assert_eq!(s.close(), Ok(()));
        assert_eq!(s.transport.closes, 2);
    }
}
