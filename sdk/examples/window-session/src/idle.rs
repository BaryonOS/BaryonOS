//! Conservative userspace idle policy. A wake is only a hint to retry real IPC.
use nieuw_os_sdk_runtime::{supports_ipc_wait, validate_abi, Error, ReceiveEndpoint};

pub trait IdleIo {
    fn ticks(&mut self) -> Result<u64, Error>;
    fn wait(&mut self, endpoint: ReceiveEndpoint, deadline: u64) -> Result<u16, Error>;
    fn yield_now(&mut self) -> Result<(), Error>;
}

pub struct Idle {
    supported: bool,
    last_tick: Option<u64>,
}

impl Idle {
    pub fn new(abi: u64) -> Result<Self, Error> {
        validate_abi(abi)?;
        Ok(Self {
            supported: supports_ipc_wait(abi),
            last_tick: None,
        })
    }

    /// One existing receive grant and at most the next PIT tick. This does not
    /// replace, renew or retry any transaction's separate absolute deadline.
    pub fn pause(
        &mut self,
        quiescent: bool,
        endpoint: ReceiveEndpoint,
        io: &mut impl IdleIo,
    ) -> Result<(), Error> {
        if !self.supported || !quiescent {
            return io.yield_now();
        }
        let now = io.ticks()?;
        if self.last_tick.is_some_and(|previous| now < previous) {
            return Err(Error::InvalidReturn);
        }
        let deadline = now.checked_add(1).ok_or(Error::InvalidReturn)?;
        self.last_tick = Some(now);
        // A zero deadline wake or bit zero changes no model/ACK/transaction.
        // The SDK checks the mask too; retain the bound for injected host I/O.
        match io.wait(endpoint, deadline)? {
            0 | 1 => Ok(()),
            _ => Err(Error::InvalidReturn),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CompositorState {
    pub ingress_empty: bool,
    pub received: bool,
    pub reply_pending: bool,
    pub input_pending: bool,
    pub input_backlog: bool,
    pub surface_pending: bool,
    pub damage_pending: bool,
}
impl CompositorState {
    pub fn quiescent(self) -> bool {
        self.ingress_empty
            && !self.received
            && !self.reply_pending
            && !self.input_pending
            && !self.input_backlog
            && !self.surface_pending
            && !self.damage_pending
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InitState {
    pub registry_empty: bool,
    pub received: bool,
    pub running: bool,
    pub ready_mask: u8,
    pub recovering: bool,
    /// Any retained recovery phase must finish before the supervisor parks.
    pub recovery_pending: bool,
    pub stopping: bool,
}
impl InitState {
    pub fn quiescent(self) -> bool {
        self.registry_empty
            && !self.received
            && self.running
            && self.ready_mask == 31
            && !self.recovering
            && !self.recovery_pending
            && !self.stopping
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub struct NativeIdle;
#[cfg(all(target_arch = "x86_64", target_os = "none"))]
impl IdleIo for NativeIdle {
    fn ticks(&mut self) -> Result<u64, Error> {
        nieuw_os_sdk_runtime::clock::now_monotonic()
    }
    fn wait(&mut self, endpoint: ReceiveEndpoint, deadline: u64) -> Result<u16, Error> {
        let cancellation = nieuw_os_sdk_runtime::clock::TimerCancellation::new();
        let timer = nieuw_os_sdk_runtime::clock::TimerRegistration::at(deadline, &cancellation);
        match nieuw_os_sdk_runtime::clock::wait_until(
            &[nieuw_os_sdk_runtime::WaitCondition::receive(endpoint)],
            &timer,
        )? {
            nieuw_os_sdk_runtime::clock::WaitOutcome::Ready(mask) => Ok(mask),
            nieuw_os_sdk_runtime::clock::WaitOutcome::TimedOut => Ok(0),
            nieuw_os_sdk_runtime::clock::WaitOutcome::Cancelled => Err(Error::InvalidReturn),
        }
    }
    fn yield_now(&mut self) -> Result<(), Error> {
        nieuw_os_sdk_runtime::native::yield_now()
    }
}
