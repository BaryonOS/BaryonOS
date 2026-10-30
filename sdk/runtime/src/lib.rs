#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
//! Allocation-free userspace bindings for the common Nieuw OS int80 ABI 1.x.
//!
//! A numeric handle is meaningful only in the calling process. These wrappers
//! check representation; only the kernel checks authority and object lifetime.
//! Message policy and automatic retries are deliberately left to the caller.

pub mod block;
mod message;
pub mod process;
pub use block::BlockDevice;
pub use entropy::{EntropySource, ENTROPY_MAX};
pub use message::{Outgoing, Received, MAX_CAPS, MAX_PAYLOAD, MESSAGE_SIZE};
pub use process::{ChildProcess, LaunchImage, ProcessStatus};
pub mod clock;
pub mod entropy;
pub mod memory;
pub mod network;
pub use network::{NetworkSource, NET_FRAME_MAX};

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub mod native;

pub const ABI_MAJOR: u64 = 1;
pub const ABI_MAX_MINOR: u64 = 8;

/// The common syscall contract is unchanged across the existing 1.0–1.7
/// profiles (1.3 adds the K05 process-management calls, 1.4 the B01 entropy
/// broker; 1.5 adds the raw NIC broker; 1.6 adds boot-monotonic ticks; 1.7 adds private heap growth). Unknown revisions require a separate compatibility check.
pub fn validate_abi(version: u64) -> Result<(), Error> {
    if version >> 32 == ABI_MAJOR && version as u32 as u64 <= ABI_MAX_MINOR {
        Ok(())
    } else {
        Err(Error::UnsupportedAbi)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidHandle,
    InvalidRights,
    MessageSize,
    InvalidMessage,
    InvalidReturn,
    UnsupportedAbi,
    Kernel(i64),
}

impl Error {
    /// The current kernel guarantees that EAGAIN publishes/receives nothing.
    pub const fn is_again(self) -> bool {
        matches!(self, Self::Kernel(-8))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle(u64);

impl Handle {
    /// Validate the ABI representation, without claiming that the slot is live
    /// or that the caller owns any particular right.
    pub const fn from_raw(raw: u64) -> Result<Self, Error> {
        let slot = raw as u32;
        if slot == 0 || slot >= 256 || raw >> 32 == 0 {
            Err(Error::InvalidHandle)
        } else {
            Ok(Self(raw))
        }
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendEndpoint(Handle);

impl SendEndpoint {
    pub const fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }
    pub const fn handle(self) -> Handle {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiveEndpoint(Handle);

impl ReceiveEndpoint {
    pub const fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }
    pub const fn handle(self) -> Handle {
        self.0
    }
}

/// The currently defined endpoint, block, launch and process right bits.
/// Rights are requested reductions; constructing this value never installs
/// or enlarges authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rights(u64);

impl Rights {
    pub const NONE: Self = Self(0);
    pub const SEND: Self = Self(1);
    pub const RECV: Self = Self(2);
    pub const DERIVE: Self = Self(4);
    pub const TRANSFER: Self = Self(8);
    pub const REVOKE: Self = Self(16);
    pub const BLOCK_READ: Self = Self(1);
    pub const BLOCK_WRITE: Self = Self(32);
    pub const BLOCK_FLUSH: Self = Self(64);
    /// K05: start the image a `Launch` object names.
    pub const LAUNCH: Self = Self(128);
    /// K05: read the lifecycle state of a spawned child.
    pub const OBSERVE: Self = Self(256);
    /// K05: end a spawned child.
    pub const KILL: Self = Self(512);
    /// B01: read raw bytes from the kernel entropy broker (Key Service only).
    pub const ENTROPY: Self = Self(1024);
    /// Raw NIC transport for the isolated network service, not an app grant.
    pub const NETWORK: Self = Self(2048);
    /// Manifest-only display transport; this representation grants no authority.
    pub const DISPLAY: Self = Self(4096);
    /// Manifest-only raw input transport for the isolated source service.
    pub const INPUT: Self = Self(8192);
    pub const KNOWN_MASK: u64 = 16383;

    pub const fn from_bits(bits: u64) -> Result<Self, Error> {
        if bits & !Self::KNOWN_MASK != 0 {
            Err(Error::InvalidRights)
        } else {
            Ok(Self(bits))
        }
    }
    pub const fn bits(self) -> u64 {
        self.0
    }
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capability {
    pub handle: Handle,
    pub rights: Rights,
}

/// Validate the register result before exposing a kernel result to safe code.
/// Unknown negative error codes are preserved for future service extensions.
pub fn decode_return(primary: i64, secondary: u64) -> Result<u64, Error> {
    match (primary, secondary) {
        (0, value) => Ok(value),
        (code, 0) if code < 0 => Err(Error::Kernel(code)),
        _ => Err(Error::InvalidReturn),
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub(crate) fn zero_return(result: Result<u64, Error>) -> Result<(), Error> {
    match result? {
        0 => Ok(()),
        _ => Err(Error::InvalidReturn),
    }
}
