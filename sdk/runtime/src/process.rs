#![forbid(unsafe_code)]
//! Representation and status decoding for the K05 process-management ABI
//! (syscalls 0x30 and 0x50-0x52, ABI 1.3). These values are handle numbers;
//! only the kernel checks that a handle is live, typed and carries a right.
use crate::{Error, Handle};

/// Handle to a `Launch` object: permission to start one built-in image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaunchImage(Handle);

impl LaunchImage {
    pub const fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }
    pub const fn handle(self) -> Handle {
        self.0
    }
}

/// Handle to a `Process` object: observe and end one spawned child. The
/// kernel never lets this handle be derived, transferred or revoked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChildProcess(Handle);

impl ChildProcess {
    pub const fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }
    pub const fn handle(self) -> Handle {
        self.0
    }
}

pub const STATE_RUNNING: u64 = 1;
pub const STATE_EXITED: u64 = 2;
pub const STATE_FAULTED: u64 = 3;
pub const STATE_KILLED: u64 = 4;
/// Exit codes and fault vectors travel in the low 56 bits of the status word.
pub const CODE_MASK: u64 = (1 << 56) - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    /// Exit code modulo 2^56, as the child passed it to `exit`.
    Exited(u64),
    /// CPU exception vector that ended the child.
    Faulted(u64),
    /// Ended by the holder of a KILL right, or by the kernel as an orphan.
    Killed,
}

impl ProcessStatus {
    /// Decode the RDX result of `proc_status`. Unknown states, a nonzero
    /// code where the state defines none, or stray high bits are rejected.
    pub const fn decode(word: u64) -> Result<Self, Error> {
        let state = word >> 56;
        let code = word & CODE_MASK;
        match state {
            STATE_RUNNING if code == 0 => Ok(Self::Running),
            STATE_EXITED => Ok(Self::Exited(code)),
            STATE_FAULTED if code < 256 => Ok(Self::Faulted(code)),
            STATE_KILLED if code == 0 => Ok(Self::Killed),
            _ => Err(Error::InvalidReturn),
        }
    }

    pub const fn encode(self) -> u64 {
        match self {
            Self::Running => STATE_RUNNING << 56,
            Self::Exited(code) => (STATE_EXITED << 56) | (code & CODE_MASK),
            Self::Faulted(vector) => (STATE_FAULTED << 56) | (vector & CODE_MASK),
            Self::Killed => STATE_KILLED << 56,
        }
    }

    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }
}

/// Arguments for one delegated capability at spawn: handle 0 means none and
/// then requires rights 0, exactly as the kernel checks it.
pub fn spawn_cap_words(cap: Option<crate::Capability>) -> Result<(u64, u64), Error> {
    match cap {
        None => Ok((0, 0)),
        Some(cap) => {
            if cap.rights.bits() & crate::Rights::RECV.bits() != 0 {
                return Err(Error::InvalidRights);
            }
            Ok((cap.handle.raw(), cap.rights.bits()))
        }
    }
}
