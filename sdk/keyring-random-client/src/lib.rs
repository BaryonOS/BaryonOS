#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
//! A bounded RANDOM-only lease. No entropy, seed, key, signing or recovery API.
//! The bootstrap owns principal/account/generation binding; callers supply only
//! a byte count. No uncertain publication is retried or silently regranted.
use core::fmt;
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use zeroize::Zeroizing;

pub const MAX_BYTES: usize = 128;
pub const MAX_POLL_BUDGET: usize = 1_000_000;
const TAG: u64 = 0x4b45_5931;
const SIZE: usize = 256;
const HEADER: usize = 16;
const PREFIX: usize = 16;
const RANDOM: u8 = 10;

/// EAGAIN must guarantee no enqueue/receive; received caps are already owned.
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

/// Debug never formats random output. The owned current buffer is zeroized on
/// drop, without claiming erasure of earlier transport/compiler copies.
pub struct RandomBytes {
    bytes: Zeroizing<[u8; MAX_BYTES]>,
    length: usize,
}
impl RandomBytes {
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
}
impl fmt::Debug for RandomBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RandomBytes(<redacted>)")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceStatus {
    NotReady,
    Quota,
    Degraded,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    NotEnqueued,
    Enqueued,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Closed,
    Busy,
    Idle,
    InvalidLength,
    InvalidConfig,
    Exhausted,
    Protocol,
    Timeout,
    Transport(RuntimeError),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub failure: Failure,
    pub delivery: Delivery,
    pub cleanup: Option<RuntimeError>,
}
#[derive(Debug)]
pub enum Outcome {
    Bytes(RandomBytes),
    Service(ServiceStatus),
}

struct Pending {
    id: u64,
    length: usize,
    queued: bool,
    polls: usize,
}
pub struct Session<T: Transport> {
    transport: T,
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    service_pid: u64,
    budget: usize,
    next_id: u64,
    pending: Option<Pending>,
    closed: bool,
}
impl<T: Transport> Session<T> {
    /// Ownership starts on success. Endpoints must be exclusively owned and
    /// already bound to the intended principal/generation by trusted bootstrap.
    /// Request IDs start at 1; a new lease needs a fresh principal slot/service
    /// lifecycle or an independently coordinated monotone request allocator.
    pub fn new(
        transport: T,
        send: SendEndpoint,
        receive: ReceiveEndpoint,
        service_pid: u64,
        budget: usize,
    ) -> Result<Self, Failure> {
        if service_pid == 0
            || send.handle() == receive.handle()
            || budget == 0
            || budget > MAX_POLL_BUDGET
        {
            return Err(Failure::InvalidConfig);
        }
        Ok(Self {
            transport,
            send,
            receive,
            service_pid,
            budget,
            next_id: 1,
            pending: None,
            closed: false,
        })
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    pub fn transport(&self) -> &T {
        &self.transport
    }
    pub fn close(&mut self) -> Result<(), RuntimeError> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        self.pending = None;
        let send = self.transport.close(self.send.handle());
        let receive = self.transport.close(self.receive.handle());
        send.and(receive)
    }
    fn delivery(&self) -> Delivery {
        if self.pending.as_ref().is_some_and(|p| p.queued) {
            Delivery::Enqueued
        } else {
            Delivery::NotEnqueued
        }
    }
    fn fail(&mut self, failure: Failure, delivery: Delivery) -> Error {
        Error {
            failure,
            delivery,
            cleanup: self.close().err(),
        }
    }
    /// Does not publish. One poll makes at most one send or receive attempt.
    pub fn begin(&mut self, length: usize) -> Result<(), Error> {
        let error = |failure| Error {
            failure,
            delivery: Delivery::NotEnqueued,
            cleanup: None,
        };
        if self.closed {
            return Err(error(Failure::Closed));
        }
        if self.pending.is_some() {
            return Err(error(Failure::Busy));
        }
        if !(1..=MAX_BYTES).contains(&length) {
            return Err(error(Failure::InvalidLength));
        }
        let id = self.next_id;
        let Some(next) = id.checked_add(1) else {
            return Err(self.fail(Failure::Exhausted, Delivery::NotEnqueued));
        };
        self.next_id = next;
        self.pending = Some(Pending {
            id,
            length,
            queued: false,
            polls: 0,
        });
        Ok(())
    }
    /// Cancellation always retires the pair, including before enqueue. It does
    /// not undo output previously given to this process or another receiver.
    pub fn cancel(&mut self) -> Result<(), RuntimeError> {
        self.close()
    }
    pub fn poll(&mut self) -> Result<Option<Outcome>, Error> {
        if self.closed {
            return Err(Error {
                failure: Failure::Closed,
                delivery: Delivery::NotEnqueued,
                cleanup: None,
            });
        }
        let Some(pending) = self.pending.as_mut() else {
            return Err(Error {
                failure: Failure::Idle,
                delivery: Delivery::NotEnqueued,
                cleanup: None,
            });
        };
        if pending.polls >= self.budget {
            let delivery = self.delivery();
            return Err(self.fail(Failure::Timeout, delivery));
        }
        pending.polls += 1;
        if !pending.queued {
            let outgoing = request(pending.id, pending.length);
            return match self.transport.send(self.send, &outgoing) {
                Ok(()) => {
                    self.pending.as_mut().unwrap().queued = true;
                    Ok(None)
                }
                Err(error) if error.is_again() => self.wait(),
                Err(error) => Err(self.fail(Failure::Transport(error), Delivery::Unknown)),
            };
        }
        match self.transport.receive(self.receive) {
            Ok(message) => {
                let result = decode(&message, self.service_pid, pending.id, pending.length);
                let mut cap_error = None;
                for i in 0..message.cap_count() {
                    if let Some(cap) = message.cap(i) {
                        if let Err(error) = self.transport.close(cap.handle) {
                            cap_error.get_or_insert(error);
                        }
                    }
                }
                drop(message);
                if let Some(error) = cap_error {
                    return Err(self.fail(Failure::Transport(error), Delivery::Enqueued));
                }
                match result {
                    Some(outcome) => {
                        self.pending = None;
                        if matches!(outcome, Outcome::Service(ServiceStatus::Degraded)) {
                            if let Err(error) = self.close() {
                                return Err(Error {
                                    failure: Failure::Transport(error),
                                    delivery: Delivery::Enqueued,
                                    cleanup: Some(error),
                                });
                            }
                        }
                        Ok(Some(outcome))
                    }
                    None => Err(self.fail(Failure::Protocol, Delivery::Enqueued)),
                }
            }
            Err(error) if error.is_again() => self.wait(),
            Err(error) => Err(self.fail(Failure::Transport(error), Delivery::Enqueued)),
        }
    }
    fn wait(&mut self) -> Result<Option<Outcome>, Error> {
        match self.transport.yield_now() {
            Ok(()) => Ok(None),
            Err(error) => {
                let delivery = self.delivery();
                Err(self.fail(Failure::Transport(error), delivery))
            }
        }
    }
}
impl<T: Transport> Drop for Session<T> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn request(id: u64, length: usize) -> Outgoing {
    let mut bytes = [0u8; SIZE];
    bytes[0] = RANDOM;
    bytes[2..4].copy_from_slice(&(PREFIX as u16).to_le_bytes());
    bytes[4..12].copy_from_slice(&id.to_le_bytes());
    bytes[HEADER..HEADER + 4].copy_from_slice(b"RND1");
    bytes[HEADER + 4..HEADER + 6].copy_from_slice(&1u16.to_le_bytes());
    bytes[HEADER + 6..HEADER + 8].copy_from_slice(&(length as u16).to_le_bytes());
    Outgoing::new(TAG, &bytes).expect("fixed RANDOM request")
}
fn decode(message: &Received, sender: u64, id: u64, length: usize) -> Option<Outcome> {
    if message.sender() != sender || message.tag() != TAG || message.cap_count() != 0 {
        return None;
    }
    let bytes = message.payload();
    if bytes.len() != SIZE
        || bytes[0] != (RANDOM | 0x80)
        || bytes[1] != 0
        || bytes[4..12] != id.to_le_bytes()
    {
        return None;
    }
    let body_length = usize::from(u16::from_le_bytes([bytes[2], bytes[3]]));
    if body_length > SIZE - HEADER || bytes[HEADER + body_length..].iter().any(|b| *b != 0) {
        return None;
    }
    let status = i32::from_le_bytes(bytes[12..16].try_into().ok()?);
    if status != 0 {
        if body_length != 0 {
            return None;
        }
        return Some(Outcome::Service(match status {
            -1 => ServiceStatus::NotReady,
            -2 => ServiceStatus::Quota,
            -7 => ServiceStatus::Degraded,
            _ => return None,
        }));
    }
    if body_length != PREFIX + length
        || &bytes[HEADER..HEADER + 4] != b"RND1"
        || bytes[HEADER + 4..HEADER + 6] != 1u16.to_le_bytes()
        || bytes[HEADER + 6..HEADER + 8] != (length as u16).to_le_bytes()
        || bytes[HEADER + 8..HEADER + PREFIX].iter().any(|b| *b != 0)
    {
        return None;
    }
    let mut output = RandomBytes {
        bytes: Zeroizing::new([0; MAX_BYTES]),
        length,
    };
    output.bytes[..length].copy_from_slice(&bytes[HEADER + PREFIX..HEADER + PREFIX + length]);
    Some(Outcome::Bytes(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdk_owned_envelopes_are_zeroized_by_their_destructors() {
        fn assert_erased<T>(value: T, view: fn(&T) -> &[u8; nieuw_os_sdk_runtime::MESSAGE_SIZE]) {
            let mut storage = core::mem::MaybeUninit::<T>::uninit();
            let bytes = view(storage.write(value)).as_ptr();
            // SAFETY: The envelope is initialized and exclusively owned.
            // MaybeUninit keeps its backing allocation alive without a second
            // drop; the saved byte slice locates the actual buffer, not a
            // guessed struct offset. Only initialized buffer bytes are read.
            unsafe {
                assert!((0..nieuw_os_sdk_runtime::MESSAGE_SIZE)
                    .any(|i| core::ptr::read_volatile(bytes.add(i)) != 0));
                core::ptr::drop_in_place(storage.as_mut_ptr());
                for i in 0..nieuw_os_sdk_runtime::MESSAGE_SIZE {
                    assert_eq!(core::ptr::read_volatile(bytes.add(i)), 0);
                }
            }
        }

        let mut bytes = *Outgoing::new(TAG, &[0xa5; SIZE]).unwrap().as_bytes();
        bytes[..8].copy_from_slice(&7u64.to_le_bytes());
        let message = Received::from_bytes(bytes, SIZE as u64).unwrap();
        assert_erased(message, Received::as_bytes);
        assert_erased(
            Outgoing::new(TAG, &[0xa5; SIZE]).unwrap(),
            Outgoing::as_bytes,
        );
    }
    #[test]
    fn exhausted_request_ids_close_before_any_publication() {
        struct NoPublication;
        impl Transport for NoPublication {
            fn send(&mut self, _: SendEndpoint, _: &Outgoing) -> Result<(), RuntimeError> {
                panic!("published")
            }
            fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
                panic!("received")
            }
            fn yield_now(&mut self) -> Result<(), RuntimeError> {
                panic!("yielded")
            }
            fn close(&mut self, _: Handle) -> Result<(), RuntimeError> {
                Ok(())
            }
        }
        let h = |slot: u64| Handle::from_raw((1u64 << 32) | slot).unwrap();
        let mut session = Session::new(
            NoPublication,
            SendEndpoint::from_handle(h(1)),
            ReceiveEndpoint::from_handle(h(2)),
            7,
            4,
        )
        .unwrap();
        session.next_id = u64::MAX;
        assert_eq!(session.begin(32).unwrap_err().failure, Failure::Exhausted);
        assert!(session.is_closed());
    }
}
