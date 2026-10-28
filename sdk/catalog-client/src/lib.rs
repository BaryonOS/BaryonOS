#![no_std]
#![forbid(unsafe_code)]
//! Bounded transport for the existing B01 catalog client. One queued request
//! is never resent; uncertainty closes this session and its endpoint handles.
use catalog_core::client::{Client, Error as ClientError, TransportFailure};
pub use catalog_core::client::{Document, Operation, Reply};
pub use catalog_core::wire::Status;
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};

pub const MAX_POLL_BUDGET: usize = 1_000_000;

/// A send error with EAGAIN must guarantee that no message was enqueued. A
/// successful receive owns the real kernel-delivered cap handles in Received.
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
pub enum Delivery {
    NotEnqueued,
    Enqueued,
    /// A malformed syscall result cannot establish whether enqueue occurred.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidBudget,
    DuplicateEndpoints,
    Client(ClientError),
    Interrupted {
        source: ClientError,
        /// Preserve the transport fact separately from B01's conservative
        /// mutation uncertainty. A malformed syscall result is Unknown.
        delivery: Delivery,
        transport: Option<RuntimeError>,
        cleanup: Option<RuntimeError>,
    },
    Cleanup {
        service_status: Option<Status>,
        error: RuntimeError,
    },
}

/// Owns two already granted endpoint handles until explicit close, terminal
/// service failure, transport failure or invalid response. No recovery/reset
/// constructor is hidden in the exchange path. A new session after uncertainty
/// requires an independently recovered service and newly established grants.
pub struct Session<T> {
    client: Client,
    transport: T,
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    poll_budget: usize,
    closed: bool,
}

impl<T: Transport> Session<T> {
    pub fn new(
        transport: T,
        epoch: u64,
        service_pid: u64,
        domain: usize,
        send: SendEndpoint,
        receive: ReceiveEndpoint,
        poll_budget: usize,
    ) -> Result<Self, Error> {
        if poll_budget == 0 || poll_budget > MAX_POLL_BUDGET {
            return Err(Error::InvalidBudget);
        }
        if send.handle() == receive.handle() {
            return Err(Error::DuplicateEndpoints);
        }
        let client = Client::new(epoch, service_pid, domain).map_err(Error::Client)?;
        Ok(Self {
            client,
            transport,
            send,
            receive,
            poll_budget,
            closed: false,
        })
    }

    pub fn is_closed(&self) -> bool {
        self.closed || self.client.is_closed()
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn close(&mut self) -> Result<(), RuntimeError> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        // Attempt both closes even if the first fails. No automatic retry of a
        // failed close on a possibly changed handle generation occurs later.
        let send = self.transport.close(self.send.handle());
        let receive = self.transport.close(self.receive.handle());
        send.and(receive)
    }

    fn interrupted(
        &mut self,
        failure: TransportFailure,
        delivery: Delivery,
        transport: Option<RuntimeError>,
    ) -> Error {
        let source = self.client.transport_failed(failure);
        let cleanup = self.close().err();
        Error::Interrupted {
            source,
            delivery,
            transport,
            cleanup,
        }
    }

    pub fn exchange(&mut self, operation: Operation<'_>) -> Result<Reply, Error> {
        if self.is_closed() {
            return Err(Error::Client(ClientError::Closed));
        }
        let request = match self.client.begin(operation) {
            Ok(request) => request,
            Err(source) => {
                // In particular, exhausted request IDs close B01 before any
                // publication; release owned endpoints on that terminal path.
                if self.client.is_closed() {
                    if let Some(cleanup) = self.close().err() {
                        return Err(Error::Interrupted {
                            source,
                            delivery: Delivery::NotEnqueued,
                            transport: None,
                            cleanup: Some(cleanup),
                        });
                    }
                }
                return Err(Error::Client(source));
            }
        };
        let outgoing = match Outgoing::from_bytes(*request.message()) {
            Ok(message) => message,
            Err(error) => {
                return Err(self.interrupted(
                    TransportFailure::Transport,
                    Delivery::NotEnqueued,
                    Some(error),
                ))
            }
        };
        let mut queued = false;
        for _ in 0..self.poll_budget {
            match self.transport.send(self.send, &outgoing) {
                Ok(()) => {
                    queued = true;
                    break;
                }
                Err(error) if error.is_again() => {
                    if let Err(error) = self.transport.yield_now() {
                        return Err(self.interrupted(
                            TransportFailure::Transport,
                            Delivery::NotEnqueued,
                            Some(error),
                        ));
                    }
                }
                Err(error) => {
                    let delivery = if matches!(error, RuntimeError::Kernel(code) if code < 0) {
                        Delivery::NotEnqueued
                    } else {
                        Delivery::Unknown
                    };
                    return Err(self.interrupted(
                        TransportFailure::Transport,
                        delivery,
                        Some(error),
                    ));
                }
            }
        }
        if !queued {
            return Err(self.interrupted(TransportFailure::Timeout, Delivery::NotEnqueued, None));
        }
        for _ in 0..self.poll_budget {
            match self.transport.receive(self.receive) {
                Ok(message) => {
                    // B01 sees the complete original envelope BEFORE cleanup.
                    // Removing cap_count first would hide an invalid response.
                    let reply = self.client.receive(message.as_bytes());
                    let mut cleanup = None;
                    for index in 0..message.cap_count() {
                        if let Some(cap) = message.cap(index) {
                            if let Err(error) = self.transport.close(cap.handle) {
                                cleanup.get_or_insert(error);
                            }
                        }
                    }
                    match reply {
                        Err(source) => {
                            let endpoints = self.close().err();
                            return Err(Error::Interrupted {
                                source,
                                delivery: Delivery::Enqueued,
                                transport: None,
                                cleanup: cleanup.or(endpoints),
                            });
                        }
                        Ok(reply) => {
                            if self.client.is_closed() || cleanup.is_some() {
                                let endpoints = self.close().err();
                                if let Some(error) = cleanup.or(endpoints) {
                                    let service_status = match reply {
                                        Reply::ServiceError(status) => Some(status),
                                        _ => None,
                                    };
                                    return Err(Error::Cleanup {
                                        service_status,
                                        error,
                                    });
                                }
                            }
                            return Ok(reply);
                        }
                    }
                }
                Err(error) if error.is_again() => {
                    if let Err(error) = self.transport.yield_now() {
                        return Err(self.interrupted(
                            TransportFailure::Transport,
                            Delivery::Enqueued,
                            Some(error),
                        ));
                    }
                }
                Err(error) => {
                    return Err(self.interrupted(
                        TransportFailure::Transport,
                        Delivery::Enqueued,
                        Some(error),
                    ))
                }
            }
        }
        Err(self.interrupted(TransportFailure::Timeout, Delivery::Enqueued, None))
    }
}
