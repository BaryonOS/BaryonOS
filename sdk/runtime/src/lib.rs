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
pub mod tls_time;
pub use network::{NetworkSource, NET_FRAME_MAX};

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub mod native;

pub const ABI_MAJOR: u64 = 1;
pub const ABI_MAX_MINOR: u64 = 11;

/// ABI 1.10 bounded IPC queue wait. Representations do not grant authority.
pub const IPC_WAIT_MAX: usize = 16;
pub const IPC_WAIT_HORIZON_TICKS: u64 = 1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaitCondition {
    handle: Handle,
    interest: u64,
}
impl WaitCondition {
    pub const fn send(endpoint: SendEndpoint) -> Self {
        Self {
            handle: endpoint.handle(),
            interest: 1,
        }
    }
    pub const fn receive(endpoint: ReceiveEndpoint) -> Self {
        Self {
            handle: endpoint.handle(),
            interest: 2,
        }
    }
}

#[cfg(any(test, all(target_arch = "x86_64", target_os = "none")))]
fn encode_wait(conditions: &[WaitCondition]) -> Result<[u8; 256], Error> {
    if conditions.len() > IPC_WAIT_MAX {
        return Err(Error::MessageSize);
    }
    let mut bytes = [0; 256];
    for (condition, entry) in conditions.iter().zip(bytes.chunks_exact_mut(16)) {
        entry[..8].copy_from_slice(&condition.handle.raw().to_le_bytes());
        entry[8..].copy_from_slice(&condition.interest.to_le_bytes());
    }
    Ok(bytes)
}

#[cfg(any(test, all(target_arch = "x86_64", target_os = "none")))]
fn decode_wait_mask(value: u64, count: usize) -> Result<u16, Error> {
    if count > IPC_WAIT_MAX || value >> count != 0 {
        return Err(Error::InvalidReturn);
    }
    Ok(value as u16)
}

/// The common syscall contract is unchanged across the existing 1.0–1.10
/// profiles (1.3 adds process management, 1.4 entropy, 1.5 raw NIC,
/// 1.6 monotonic ticks, 1.7 private heap growth, 1.8 display/input,
/// 1.9 opt-in unauthenticated RTC UTC, and 1.10 opt-in IPC wait).
/// Unknown revisions require a separate compatibility check.
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

#[cfg(test)]
mod wait_tests {
    use super::*;
    fn send(raw: u64) -> WaitCondition {
        WaitCondition::send(SendEndpoint::from_handle(Handle::from_raw(raw).unwrap()))
    }
    #[test]
    fn all_full_handle_bits_and_interests_are_encoded_without_padding() {
        let first = send(0x1234567800000001);
        let last = WaitCondition::receive(ReceiveEndpoint::from_handle(
            Handle::from_raw(0xffffffff000000ff).unwrap(),
        ));
        let bytes = encode_wait(&[first, last]).unwrap();
        assert_eq!(&bytes[..8], &0x1234567800000001u64.to_le_bytes());
        assert_eq!(&bytes[8..16], &1u64.to_le_bytes());
        assert_eq!(&bytes[16..24], &0xffffffff000000ffu64.to_le_bytes());
        assert_eq!(&bytes[24..32], &2u64.to_le_bytes());
        assert!(bytes[32..].iter().all(|byte| *byte == 0));
        assert_eq!(encode_wait(&[]), Ok([0; 256]));
        assert!(encode_wait(&[first; 16]).is_ok());
        assert_eq!(encode_wait(&[first; 17]), Err(Error::MessageSize));
    }
    #[test]
    fn returned_mask_is_bounded_by_count_including_empty_sleep() {
        for count in 0..=16 {
            let allowed = (1u64 << count) - 1;
            assert_eq!(decode_wait_mask(0, count), Ok(0));
            assert_eq!(decode_wait_mask(allowed, count), Ok(allowed as u16));
            assert_eq!(
                decode_wait_mask(1 << count, count),
                Err(Error::InvalidReturn)
            );
        }
        assert_eq!(decode_wait_mask(0, 17), Err(Error::InvalidReturn));
        assert_eq!(decode_wait_mask(u64::MAX, 16), Err(Error::InvalidReturn));
    }
    #[test]
    fn minor_eleven_preserves_old_abi_and_return_rules() {
        for minor in 0..=11 {
            assert_eq!(validate_abi((1 << 32) | minor), Ok(()));
        }
        assert_eq!(validate_abi((1 << 32) | 12), Err(Error::UnsupportedAbi));
        assert_eq!(decode_return(0, 0xffff), Ok(0xffff));
        assert_eq!(decode_return(-10, 0), Err(Error::Kernel(-10)));
        assert_eq!(decode_return(-10, 1), Err(Error::InvalidReturn));
    }
}
