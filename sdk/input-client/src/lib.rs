#![no_std]
#![forbid(unsafe_code)]
//! Typed INP1 application channel. Hardware bytes, focus policy and grants
//! remain with the isolated source, input service and bootstrap authority.
mod reply;
#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub use nieuw_os_catalog_client::NativeTransport;
pub use nieuw_os_catalog_client::{Delivery, Transport, MAX_POLL_BUDGET};
use nieuw_os_input_core::wire::{Operation, Request, TAG};
pub use nieuw_os_input_core::{CancelReason, Error as ServiceError, Event};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};

/// Validated correlation expectations and already granted channels. Neither a
/// client ID, expected PID nor a dimension grants authority; kernel metadata
/// and the service's trusted ingress/bootstrap remain authoritative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub epoch: u64,
    pub service_pid: u64,
    pub client_id: u64,
    pub width: u32,
    pub height: u32,
    pub send: SendEndpoint,
    pub receive: ReceiveEndpoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offer {
    epoch: u64,
    sequence: u64,
    generation: u64,
    event: Event,
}
impl Offer {
    pub const fn epoch(self) -> u64 {
        self.epoch
    }
    pub const fn sequence(self) -> u64 {
        self.sequence
    }
    pub const fn generation(self) -> u64 {
        self.generation
    }
    pub const fn event(self) -> Event {
        self.event
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Peek {
    Empty,
    Offer(Offer),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AckOutcome {
    Acknowledged,
    /// A real canonical StaleAck: old work was superseded by focus/lifecycle
    /// cancellation. No new event/cancellation has been consumed or implied.
    Superseded,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    Envelope,
    Header,
    Body,
    Event,
    Sequence,
    Generation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidBinding,
    InvalidBudget,
    Closed,
    OutstandingOffer,
    WrongOffer,
    RequestExhausted {
        cleanup: Option<RuntimeError>,
    },
    Service {
        status: ServiceError,
        cleanup: Option<RuntimeError>,
    },
    Interrupted {
        delivery: Delivery,
        transport: Option<RuntimeError>,
        protocol: Option<ProtocolError>,
        cleanup: Option<RuntimeError>,
    },
}

enum Reply {
    Empty,
    Event(Offer),
    Acknowledged,
    Failure(ServiceError),
}

/// Exclusively owns this endpoint pair after successful construction. PEEK
/// offers once; until exact ACK the caller cannot issue another PEEK. After an
/// uncertain reply, creating a new session requires explicitly recovered/new
/// grants and application reconciliation, never an implicit replay here.
pub struct Session<T: Transport> {
    transport: T,
    binding: Binding,
    budget: usize,
    next_id: u64,
    last_sequence: u64,
    last_generation: u64,
    outstanding: Option<Offer>,
    closed: bool,
}
impl<T: Transport> Drop for Session<T> {
    fn drop(&mut self) {
        // Best-effort resource cleanup only, never an event ACK. Call close()
        // explicitly when the application must observe a transport close error.
        let _ = self.close();
    }
}
impl<T: Transport> Session<T> {
    pub fn new(transport: T, binding: Binding, poll_budget: usize) -> Result<Self, Error> {
        if binding.epoch == 0
            || binding.service_pid == 0
            || binding.client_id == 0
            || binding.width == 0
            || binding.width > 1920
            || binding.height == 0
            || binding.height > 1080
            || binding.send.handle() == binding.receive.handle()
        {
            return Err(Error::InvalidBinding);
        }
        if poll_budget == 0 || poll_budget > MAX_POLL_BUDGET {
            return Err(Error::InvalidBudget);
        }
        Ok(Self {
            transport,
            binding,
            budget: poll_budget,
            next_id: 1,
            last_sequence: 0,
            last_generation: 0,
            outstanding: None,
            closed: false,
        })
    }
    pub const fn is_closed(&self) -> bool {
        self.closed
    }
    pub const fn binding(&self) -> Binding {
        self.binding
    }
    pub const fn outstanding(&self) -> Option<Offer> {
        self.outstanding
    }
    pub fn transport(&self) -> &T {
        &self.transport
    }
    pub fn close(&mut self) -> Result<(), RuntimeError> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        self.outstanding = None;
        // Both are attempted once, even after the first close fails. Handles
        // are never retried after a possibly changed generation.
        let send = self.transport.close(self.binding.send.handle());
        let receive = self.transport.close(self.binding.receive.handle());
        send.and(receive)
    }
    /// Cancel this local session; no global-input/focus or implicit ACK opcode.
    pub fn cancel(&mut self) -> Result<(), RuntimeError> {
        self.close()
    }
    fn interrupted(
        &mut self,
        delivery: Delivery,
        transport: Option<RuntimeError>,
        protocol: Option<ProtocolError>,
        extra_cleanup: Option<RuntimeError>,
    ) -> Error {
        let cleanup = self.close().err();
        Error::Interrupted {
            delivery,
            transport,
            protocol,
            cleanup: extra_cleanup.or(cleanup),
        }
    }
    fn close_received_caps(&mut self, message: &Received) -> Option<RuntimeError> {
        let mut previous = None;
        let mut failure = None;
        for index in 0..message.cap_count() {
            if let Some(cap) = message.cap(index) {
                // Actual native receive installs distinct free handles. Also
                // fail safely for a hostile Transport repeating a handle.
                if previous == Some(cap.handle)
                    || cap.handle == self.binding.send.handle()
                    || cap.handle == self.binding.receive.handle()
                {
                    continue;
                }
                previous = Some(cap.handle);
                if let Err(error) = self.transport.close(cap.handle) {
                    failure.get_or_insert(error);
                }
            }
        }
        failure
    }
    fn exchange(&mut self, operation: Operation, arg0: u64) -> Result<Reply, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        let id = self.next_id;
        let Some(next) = id.checked_add(1) else {
            return Err(Error::RequestExhausted {
                cleanup: self.close().err(),
            });
        };
        self.next_id = next;
        let request = Request {
            operation,
            id,
            epoch: self.binding.epoch,
            arg0,
            arg1: 0,
        };
        let outgoing = Outgoing::new(TAG, &request.encode())
            .map_err(|error| self.interrupted(Delivery::NotEnqueued, Some(error), None, None))?;
        let mut queued = false;
        for _ in 0..self.budget {
            match self.transport.send(self.binding.send, &outgoing) {
                Ok(()) => {
                    queued = true;
                    break;
                }
                Err(error) if error.is_again() => {
                    if let Err(error) = self.transport.yield_now() {
                        return Err(self.interrupted(
                            Delivery::NotEnqueued,
                            Some(error),
                            None,
                            None,
                        ));
                    }
                }
                Err(error) => {
                    let delivery = if matches!(error, RuntimeError::Kernel(code) if code < 0) {
                        Delivery::NotEnqueued
                    } else {
                        Delivery::Unknown
                    };
                    return Err(self.interrupted(delivery, Some(error), None, None));
                }
            }
        }
        if !queued {
            return Err(self.interrupted(Delivery::NotEnqueued, None, None, None));
        }
        for _ in 0..self.budget {
            match self.transport.receive(self.binding.receive) {
                Ok(message) => {
                    let decoded = reply::decode(&message, self.binding, operation, id);
                    let cleanup = self.close_received_caps(&message);
                    return match decoded {
                        Ok(reply) if cleanup.is_none() => Ok(reply),
                        Ok(_) => Err(self.interrupted(Delivery::Enqueued, None, None, cleanup)),
                        Err(error) => {
                            Err(self.interrupted(Delivery::Enqueued, None, Some(error), cleanup))
                        }
                    };
                }
                Err(error) if error.is_again() => {
                    if let Err(error) = self.transport.yield_now() {
                        return Err(self.interrupted(Delivery::Enqueued, Some(error), None, None));
                    }
                }
                Err(error) => {
                    return Err(self.interrupted(Delivery::Enqueued, Some(error), None, None))
                }
            }
        }
        Err(self.interrupted(Delivery::Enqueued, None, None, None))
    }
    pub fn peek(&mut self) -> Result<Peek, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        if self.outstanding.is_some() {
            return Err(Error::OutstandingOffer);
        }
        match self.exchange(Operation::Peek, 0)? {
            Reply::Empty => Ok(Peek::Empty),
            Reply::Event(offer) => {
                if offer.sequence <= self.last_sequence {
                    return Err(self.interrupted(
                        Delivery::Enqueued,
                        None,
                        Some(ProtocolError::Sequence),
                        None,
                    ));
                }
                // A cancellation carries a through-watermark, which can be
                // older than a newly known focus generation. The controller
                // handles that watermark; it never becomes fresh key authority.
                if !matches!(offer.event, Event::Cancel(_)) {
                    if offer.generation < self.last_generation {
                        return Err(self.interrupted(
                            Delivery::Enqueued,
                            None,
                            Some(ProtocolError::Generation),
                            None,
                        ));
                    }
                    self.last_generation = offer.generation;
                }
                self.last_sequence = offer.sequence;
                self.outstanding = Some(offer);
                Ok(Peek::Offer(offer))
            }
            Reply::Failure(status) => Err(Error::Service {
                status,
                cleanup: self.close().err(),
            }),
            Reply::Acknowledged => {
                Err(self.interrupted(Delivery::Enqueued, None, Some(ProtocolError::Body), None))
            }
        }
    }
    pub fn ack(&mut self, offer: &Offer) -> Result<AckOutcome, Error> {
        if self.closed {
            return Err(Error::Closed);
        }
        if self.outstanding.as_ref() != Some(offer) {
            return Err(Error::WrongOffer);
        }
        match self.exchange(Operation::Ack, offer.sequence)? {
            Reply::Acknowledged => {
                self.outstanding = None;
                Ok(AckOutcome::Acknowledged)
            }
            Reply::Failure(ServiceError::StaleAck) => {
                self.outstanding = None;
                Ok(AckOutcome::Superseded)
            }
            Reply::Failure(status) => Err(Error::Service {
                status,
                cleanup: self.close().err(),
            }),
            _ => Err(self.interrupted(Delivery::Enqueued, None, Some(ProtocolError::Body), None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nieuw_os_sdk_runtime::Handle;
    struct Never {
        sends: usize,
        closes: usize,
    }
    impl Transport for Never {
        fn send(&mut self, _: SendEndpoint, _: &Outgoing) -> Result<(), RuntimeError> {
            self.sends += 1;
            panic!("must not enqueue");
        }
        fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
            panic!("must not receive");
        }
        fn yield_now(&mut self) -> Result<(), RuntimeError> {
            panic!("must not yield");
        }
        fn close(&mut self, _: Handle) -> Result<(), RuntimeError> {
            self.closes += 1;
            Ok(())
        }
    }
    #[test]
    fn request_exhaustion_closes_without_wrap_enqueue_or_replay() {
        let mut session = Session::new(
            Never {
                sends: 0,
                closes: 0,
            },
            Binding {
                epoch: 1,
                service_pid: 10,
                client_id: 7,
                width: 1280,
                height: 720,
                send: SendEndpoint::from_handle(Handle::from_raw((1 << 32) | 1).unwrap()),
                receive: ReceiveEndpoint::from_handle(Handle::from_raw((1 << 32) | 2).unwrap()),
            },
            1,
        )
        .unwrap();
        session.next_id = u64::MAX;
        assert_eq!(
            session.peek(),
            Err(Error::RequestExhausted { cleanup: None })
        );
        assert_eq!(session.peek(), Err(Error::Closed));
        assert_eq!(session.next_id, u64::MAX);
        assert_eq!((session.transport.sends, session.transport.closes), (0, 2));
    }
}
