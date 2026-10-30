use core::{fmt, num::NonZeroU64};
use nieuw_os_keyring_random_client::{
    Error as SessionError, Outcome, ServiceStatus, Session, Transport, MAX_BYTES, MAX_POLL_BUDGET,
};
use nieuw_os_sdk_runtime::Error as RuntimeError;
use rustls::crypto::{GetRandomFailed, SecureRandom};
use spin::Mutex;
use zeroize::Zeroize;

pub trait MonotonicClock: Send {
    fn ticks(&mut self) -> Result<u64, RuntimeError>;
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub struct NativeClock;

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
impl MonotonicClock for NativeClock {
    fn ticks(&mut self) -> Result<u64, RuntimeError> {
        nieuw_os_sdk_runtime::native::monotonic_ticks()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Busy,
    InvalidLength,
    Clock(RuntimeError),
    ClockWentBackwards,
    DeadlineOverflow,
    Deadline,
    PollBudget,
    Session(SessionError),
    Service(ServiceStatus),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub failure: Failure,
    pub cleanup: Option<RuntimeError>,
}

struct State<T: Transport, C> {
    session: Session<T>,
    clock: C,
    last_ticks: Option<u64>,
}

/// One exclusively owned RANDOM lease. Bootstrap must already have bound its
/// endpoints, service PID and caller generation. Never spin waiting for a lock.
pub struct KeyServiceRandom<T: Transport, C> {
    state: Mutex<State<T, C>>,
    timeout_ticks: NonZeroU64,
}

impl<T: Transport, C> KeyServiceRandom<T, C> {
    pub fn new(session: Session<T>, clock: C, timeout_ticks: NonZeroU64) -> Self {
        Self {
            state: Mutex::new(State {
                session,
                clock,
                last_ticks: None,
            }),
            timeout_ticks,
        }
    }
}

impl<T: Transport, C> fmt::Debug for KeyServiceRandom<T, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("KeyServiceRandom(<redacted>)")
    }
}

impl<T: Transport, C: MonotonicClock> State<T, C> {
    fn now(&mut self) -> Result<u64, Failure> {
        let now = self.clock.ticks().map_err(Failure::Clock)?;
        if self.last_ticks.is_some_and(|last| now < last) {
            return Err(Failure::ClockWentBackwards);
        }
        self.last_ticks = Some(now);
        Ok(now)
    }

    fn check_deadline(&mut self, deadline: u64) -> Result<(), Failure> {
        if self.now()? >= deadline {
            return Err(Failure::Deadline);
        }
        Ok(())
    }

    fn fill(&mut self, output: &mut [u8], timeout: u64) -> Result<(), Failure> {
        let deadline = self
            .now()?
            .checked_add(timeout)
            .ok_or(Failure::DeadlineOverflow)?;
        self.session.begin(output.len()).map_err(Failure::Session)?;
        for _ in 0..=MAX_POLL_BUDGET {
            self.check_deadline(deadline)?;
            let outcome = self.session.poll().map_err(Failure::Session)?;
            self.check_deadline(deadline)?;
            match outcome {
                Some(Outcome::Bytes(bytes)) => {
                    output.copy_from_slice(bytes.as_slice());
                    return Ok(());
                }
                Some(Outcome::Service(status)) => return Err(Failure::Service(status)),
                None => {}
            }
        }
        Err(Failure::PollBudget)
    }
}

impl<T: Transport, C: MonotonicClock> KeyServiceRandom<T, C> {
    /// Full output or a zeroized buffer. No automatic retry, chunking, cached
    /// bytes, host RNG or regrant. Failures after starting retire the lease.
    pub fn try_fill(&self, output: &mut [u8]) -> Result<(), Error> {
        output.zeroize();
        if output.is_empty() {
            return Ok(());
        }
        if output.len() > MAX_BYTES {
            return Err(Error {
                failure: Failure::InvalidLength,
                cleanup: None,
            });
        }
        let Some(mut state) = self.state.try_lock() else {
            return Err(Error {
                failure: Failure::Busy,
                cleanup: None,
            });
        };
        match state.fill(output, self.timeout_ticks.get()) {
            Ok(()) => Ok(()),
            Err(failure) => Err(Error {
                failure,
                cleanup: state.session.cancel().err(),
            }),
        }
    }
}

impl<T: Transport + Send, C: MonotonicClock> SecureRandom for KeyServiceRandom<T, C> {
    fn fill(&self, output: &mut [u8]) -> Result<(), GetRandomFailed> {
        self.try_fill(output).map_err(|_| GetRandomFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nieuw_os_sdk_runtime::{Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint};

    struct NoIo;
    impl Transport for NoIo {
        fn send(&mut self, _: SendEndpoint, _: &Outgoing) -> Result<(), RuntimeError> {
            panic!("unexpected send")
        }
        fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
            panic!("unexpected receive")
        }
        fn yield_now(&mut self) -> Result<(), RuntimeError> {
            panic!("unexpected yield")
        }
        fn close(&mut self, _: Handle) -> Result<(), RuntimeError> {
            Ok(())
        }
    }
    impl MonotonicClock for NoIo {
        fn ticks(&mut self) -> Result<u64, RuntimeError> {
            panic!("unexpected clock read")
        }
    }

    #[test]
    fn contention_fails_without_spinning_io_or_cancelling_the_owner() {
        let handle = |slot| Handle::from_raw((1 << 32) | slot).unwrap();
        let session = Session::new(
            NoIo,
            SendEndpoint::from_handle(handle(1)),
            ReceiveEndpoint::from_handle(handle(2)),
            7,
            8,
        )
        .unwrap();
        let random = KeyServiceRandom::new(session, NoIo, NonZeroU64::new(10).unwrap());
        let owner = random.state.lock();
        let mut output = [0x5a; 32];
        assert_eq!(
            random.try_fill(&mut output).unwrap_err().failure,
            Failure::Busy
        );
        assert_eq!(output, [0; 32]);
        assert!(!owner.session.is_closed());
    }
}
