//! Bounded foreground-control codecs. Metadata never creates a capability.
//! Valid wire completion is not proof of quiescence, focus or rendered pixels;
//! the actual native adapters must establish those facts before encoding it.
#![no_std]
#![forbid(unsafe_code)]

pub mod app;
pub mod switch;

use nieuw_os_sdk_runtime::{Outgoing, Received};

pub const SIZE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Sender,
    Capabilities,
    Tag,
    Length,
    Header,
    Operation,
    Reserved,
    Binding,
    Zero,
    Replay,
    Exhausted,
    Closed,
}

pub(crate) fn value(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
pub(crate) fn put(bytes: &mut [u8; SIZE], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn counter(value: u64) -> Result<u64, Error> {
    match value {
        0 => Err(Error::Zero),
        u64::MAX => Err(Error::Exhausted),
        value => Ok(value),
    }
}
pub(crate) fn envelope(message: &Received, peer: u64, tag: u64) -> Result<&[u8], Error> {
    if message.sender() != peer {
        return Err(Error::Sender);
    }
    if message.cap_count() != 0 {
        return Err(Error::Capabilities);
    }
    if message.tag() != tag {
        return Err(Error::Tag);
    }
    if message.payload().len() != SIZE {
        return Err(Error::Length);
    }
    Ok(message.payload())
}
pub(crate) fn header(bytes: &[u8], magic: &[u8; 4]) -> Result<u16, Error> {
    if &bytes[..4] != magic || bytes[4..6] != 1u16.to_le_bytes() {
        return Err(Error::Header);
    }
    Ok(u16::from_le_bytes([bytes[6], bytes[7]]))
}
pub(crate) fn begin(magic: &[u8; 4], operation: u16) -> [u8; SIZE] {
    let mut bytes = [0; SIZE];
    bytes[..4].copy_from_slice(magic);
    bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    bytes[6..8].copy_from_slice(&operation.to_le_bytes());
    bytes
}
pub(crate) fn outgoing(tag: u64, bytes: &[u8; SIZE]) -> Outgoing {
    // SIZE is a compile-time bounded payload below the SDK's 256-byte limit.
    Outgoing::new(tag, bytes).unwrap()
}

/// Per-pair request allocator. No wrap, automatic retry or implicit new epoch.
#[derive(Clone, Debug, Default)]
pub struct RequestIds {
    last: u64,
}
impl RequestIds {
    pub const fn new() -> Self {
        Self { last: 0 }
    }
    pub fn issue(&mut self) -> Result<u64, Error> {
        let next = self.last.checked_add(1).ok_or(Error::Exhausted)?;
        counter(next)?;
        self.last = next;
        Ok(next)
    }
}

/// Authenticated ingress replay state. Validate binding before calling accept.
#[derive(Debug, Default)]
pub struct ReplayGuard {
    last: u64,
    exhausted: bool,
}
impl ReplayGuard {
    pub const fn new() -> Self {
        Self {
            last: 0,
            exhausted: false,
        }
    }
    pub fn accept(&mut self, id: u64) -> Result<(), Error> {
        if self.exhausted {
            return Err(Error::Exhausted);
        }
        if id == u64::MAX {
            self.exhausted = true;
            return Err(Error::Exhausted);
        }
        if id == 0 {
            return Err(Error::Zero);
        }
        if id <= self.last {
            return Err(Error::Replay);
        }
        self.last = id;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn issue_and_ingress_exhaustion_never_wrap_or_revive() {
        let mut ids = RequestIds { last: u64::MAX - 2 };
        assert_eq!(ids.issue(), Ok(u64::MAX - 1));
        assert_eq!(ids.issue(), Err(Error::Exhausted));
        assert_eq!(ids.issue(), Err(Error::Exhausted));
        let mut guard = ReplayGuard::new();
        assert_eq!(guard.accept(0), Err(Error::Zero));
        assert_eq!(guard.accept(1), Ok(()));
        assert_eq!(guard.accept(1), Err(Error::Replay));
        assert_eq!(guard.accept(u64::MAX), Err(Error::Exhausted));
        assert_eq!(guard.accept(2), Err(Error::Exhausted));
    }
}
