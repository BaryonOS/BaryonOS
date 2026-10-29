//! Representation and completion checks for the existing scoped block ABI
//! (syscalls 0x40-0x43). A `BlockDevice` value is only a handle number; the
//! kernel checks scope, rights, binding and ticket on every call.
use crate::{Error, Handle, MESSAGE_SIZE};

pub const SECTOR_SIZE: usize = 512;
/// The broker returns at most this many bytes per READ completion.
pub const READ_MAX: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockDevice(Handle);

impl BlockDevice {
    pub const fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }
    pub const fn handle(self) -> Handle {
        self.0
    }
}

/// Validate a poll (0x41) completion envelope for `ticket`. Returns the data
/// bytes (empty for WRITE/FLUSH). Any non-canonical envelope is rejected so
/// a caller never interprets bytes that the broker did not define.
pub fn completion(
    bytes: &[u8; MESSAGE_SIZE],
    returned: u64,
    ticket: u64,
    expected: usize,
) -> Result<&[u8], Error> {
    let get64 = |at: usize| {
        let mut word = [0; 8];
        word.copy_from_slice(&bytes[at..at + 8]);
        u64::from_le_bytes(word)
    };
    let get32 =
        |at: usize| u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    if expected > READ_MAX
        || returned != expected as u64
        || get64(0) != 0
        || get64(8) != ticket
        || get32(16) as usize != expected
        || get32(20) != 0
        || bytes[24..64].iter().any(|byte| *byte != 0)
        || bytes[64 + expected..].iter().any(|byte| *byte != 0)
    {
        return Err(Error::InvalidMessage);
    }
    Ok(&bytes[64..64 + expected])
}
