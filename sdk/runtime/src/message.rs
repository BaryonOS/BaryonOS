#![forbid(unsafe_code)]
use crate::{Capability, Error, Handle, Rights};
use core::fmt;
use zeroize::Zeroize;

pub const MESSAGE_SIZE: usize = 320;
pub const MAX_PAYLOAD: usize = 256;
pub const MAX_CAPS: usize = 2;

/// Canonical send envelope. Sender and reserved fields always remain zero.
#[derive(Clone, PartialEq, Eq)]
pub struct Outgoing {
    bytes: [u8; MESSAGE_SIZE],
}

impl Outgoing {
    pub fn new(tag: u64, payload: &[u8]) -> Result<Self, Error> {
        if payload.len() > MAX_PAYLOAD {
            return Err(Error::MessageSize);
        }
        let mut bytes = [0; MESSAGE_SIZE];
        put64(&mut bytes, 8, tag);
        bytes[16..20].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes[64..64 + payload.len()].copy_from_slice(payload);
        Ok(Self { bytes })
    }

    /// Accept a protocol-specific client envelope without trusting its lengths
    /// or allowing noncanonical padding to cross the syscall boundary.
    pub fn from_bytes(bytes: [u8; MESSAGE_SIZE]) -> Result<Self, Error> {
        validate(&bytes, None)?;
        if get64(&bytes, 0) != 0 {
            return Err(Error::InvalidMessage);
        }
        Ok(Self { bytes })
    }

    /// Append a requested delegated capability. The kernel checks TRANSFER,
    /// object type, and that these rights are a subset of the source rights.
    pub fn push_cap(&mut self, cap: Capability) -> Result<(), Error> {
        let count = get32(&self.bytes, 20) as usize;
        if count == MAX_CAPS {
            return Err(Error::MessageSize);
        }
        // The current ABI never transfers receive ownership to another PID.
        if cap.rights.bits() & Rights::RECV.bits() != 0 {
            return Err(Error::InvalidRights);
        }
        put64(&mut self.bytes, 32 + count * 16, cap.handle.raw());
        put64(&mut self.bytes, 40 + count * 16, cap.rights.bits());
        self.bytes[20..24].copy_from_slice(&((count + 1) as u32).to_le_bytes());
        Ok(())
    }

    pub const fn as_bytes(&self) -> &[u8; MESSAGE_SIZE] {
        &self.bytes
    }
}

impl fmt::Debug for Outgoing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Outgoing")
            .field("length", &get32(&self.bytes, 16))
            .field("cap_count", &get32(&self.bytes, 20))
            .finish_non_exhaustive()
    }
}

impl Drop for Outgoing {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

/// A structurally valid received envelope. `from_bytes` alone does not prove
/// provenance: native::receive obtains these bytes directly from the kernel.
/// Keeping the full original envelope lets services validate sender and reject
/// unexpected capabilities before the caller explicitly closes them.
#[derive(PartialEq, Eq)]
pub struct Received {
    bytes: [u8; MESSAGE_SIZE],
    length: usize,
    cap_count: usize,
}

impl Received {
    pub fn from_bytes(bytes: [u8; MESSAGE_SIZE], returned_length: u64) -> Result<Self, Error> {
        let (length, cap_count) = validate_received(&bytes, returned_length)?;
        Ok(Self {
            bytes,
            length,
            cap_count,
        })
    }

    #[cfg(any(test, all(target_arch = "x86_64", target_os = "none")))]
    pub(crate) fn from_bytes_mut(
        bytes: &mut [u8; MESSAGE_SIZE],
        returned_length: u64,
    ) -> Result<Self, Error> {
        let (length, cap_count) = validate_received(bytes, returned_length)?;
        let owned = core::mem::replace(bytes, [0; MESSAGE_SIZE]);
        Ok(Self {
            bytes: owned,
            length,
            cap_count,
        })
    }

    pub const fn as_bytes(&self) -> &[u8; MESSAGE_SIZE] {
        &self.bytes
    }
    pub fn sender(&self) -> u64 {
        get64(&self.bytes, 0)
    }
    pub fn tag(&self) -> u64 {
        get64(&self.bytes, 8)
    }
    pub fn payload(&self) -> &[u8] {
        &self.bytes[64..64 + self.length]
    }
    pub const fn cap_count(&self) -> usize {
        self.cap_count
    }
    pub fn cap(&self, index: usize) -> Option<Capability> {
        if index >= self.cap_count {
            return None;
        }
        // The constructor checked both fields, so the option is only a bound
        // guard and cannot panic on untrusted wire bytes.
        Some(Capability {
            handle: Handle::from_raw(get64(&self.bytes, 32 + index * 16)).ok()?,
            rights: Rights::from_bits(get64(&self.bytes, 40 + index * 16)).ok()?,
        })
    }
}

impl fmt::Debug for Received {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Received")
            .field("sender", &self.sender())
            .field("tag", &self.tag())
            .field("length", &self.length)
            .field("cap_count", &self.cap_count)
            .finish_non_exhaustive()
    }
}

impl Drop for Received {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

fn validate_received(
    bytes: &[u8; MESSAGE_SIZE],
    returned_length: u64,
) -> Result<(usize, usize), Error> {
    let parts = validate(bytes, Some(returned_length))?;
    if get64(bytes, 0) == 0 {
        return Err(Error::InvalidMessage);
    }
    Ok(parts)
}

fn validate(
    bytes: &[u8; MESSAGE_SIZE],
    returned_length: Option<u64>,
) -> Result<(usize, usize), Error> {
    let length = get32(bytes, 16) as usize;
    let caps = get32(bytes, 20) as usize;
    if length > MAX_PAYLOAD || caps > MAX_CAPS {
        return Err(Error::MessageSize);
    }
    if returned_length.is_some_and(|n| n != length as u64) {
        return Err(Error::InvalidReturn);
    }
    if get64(bytes, 24) != 0
        || bytes[32 + caps * 16..64].iter().any(|b| *b != 0)
        || bytes[64 + length..].iter().any(|b| *b != 0)
    {
        return Err(Error::InvalidMessage);
    }
    for i in 0..caps {
        Handle::from_raw(get64(bytes, 32 + i * 16))?;
        let rights = Rights::from_bits(get64(bytes, 40 + i * 16))?;
        if rights.bits() & Rights::RECV.bits() != 0 {
            return Err(Error::InvalidRights);
        }
    }
    Ok((length, caps))
}

pub(crate) fn get64(bytes: &[u8; MESSAGE_SIZE], offset: usize) -> u64 {
    let mut value = [0; 8];
    value.copy_from_slice(&bytes[offset..offset + 8]);
    u64::from_le_bytes(value)
}

fn get32(bytes: &[u8; MESSAGE_SIZE], offset: usize) -> u32 {
    let mut value = [0; 4];
    value.copy_from_slice(&bytes[offset..offset + 4]);
    u32::from_le_bytes(value)
}

fn put64(bytes: &mut [u8; MESSAGE_SIZE], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn received_handoff_clears_native_staging_bytes() {
        let outgoing = Outgoing::new(9, b"secret payload").unwrap();
        assert!(!std::format!("{outgoing:?}").contains("secret payload"));
        let mut bytes = *outgoing.as_bytes();
        bytes[..8].copy_from_slice(&7u64.to_le_bytes());
        let original = bytes;

        let received = Received::from_bytes_mut(&mut bytes, 14).unwrap();

        assert_eq!(bytes, [0; MESSAGE_SIZE]);
        assert_eq!(received.as_bytes(), &original);
        assert!(!std::format!("{received:?}").contains("secret payload"));
    }

    #[test]
    fn refused_native_handoff_preserves_bytes_for_capability_cleanup() {
        let mut bytes = *Outgoing::new(9, b"secret payload").unwrap().as_bytes();
        let original = bytes;

        assert_eq!(
            Received::from_bytes_mut(&mut bytes, 14),
            Err(Error::InvalidMessage)
        );
        assert_eq!(bytes, original);
    }
}
