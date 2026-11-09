use crate::ResourceKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessIdentity(u64);

impl ProcessIdentity {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceIdentity {
    Supervisor,
    BlockStorage,
    ObjectStore,
    Catalog,
    Keyring,
    Network,
    Input,
    Compositor,
    Update,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildIdentity([u8; 32]);

impl BuildIdentity {
    pub const fn new(digest: [u8; 32]) -> Self {
        Self(digest)
    }

    pub const fn digest(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FaultClass {
    ProcessExit,
    PageFault,
    ProtectionFault,
    InvalidInstruction,
    StackGuard,
    Timeout,
    ResourceExhaustion,
    ServiceFailure,
    KernelPanic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ServicePhase {
    Starting,
    Running,
    Stopping,
    Restarting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultContext {
    None,
    ExceptionVector(u8),
    SyscallNumber(u8),
    ExitCode(i32),
    Resource(ResourceKind),
    ServicePhase(ServicePhase),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestartOutcome {
    NotAttempted,
    Restarted(u32),
    SuppressedByPolicy,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashRecordError {
    InvalidProcessIdentity,
    InvalidRestartGeneration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrashRecord {
    process: ProcessIdentity,
    service: Option<ServiceIdentity>,
    executable: BuildIdentity,
    fault: FaultClass,
    context: FaultContext,
    restart: RestartOutcome,
}

impl CrashRecord {
    pub fn new(
        process: u64,
        service: Option<ServiceIdentity>,
        executable: BuildIdentity,
        fault: FaultClass,
        context: FaultContext,
        restart: RestartOutcome,
    ) -> Result<Self, CrashRecordError> {
        let process =
            ProcessIdentity::new(process).ok_or(CrashRecordError::InvalidProcessIdentity)?;
        if matches!(restart, RestartOutcome::Restarted(0)) {
            return Err(CrashRecordError::InvalidRestartGeneration);
        }
        Ok(Self {
            process,
            service,
            executable,
            fault,
            context,
            restart,
        })
    }

    pub const fn process(&self) -> ProcessIdentity {
        self.process
    }

    pub const fn service(&self) -> Option<ServiceIdentity> {
        self.service
    }

    pub const fn executable(&self) -> BuildIdentity {
        self.executable
    }

    pub const fn fault(&self) -> FaultClass {
        self.fault
    }

    pub const fn context(&self) -> FaultContext {
        self.context
    }

    pub const fn restart(&self) -> RestartOutcome {
        self.restart
    }
}
