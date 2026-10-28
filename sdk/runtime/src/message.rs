#![forbid(unsafe_code)]
use crate::{Capability, Error, Handle, Rights};

pub const MESSAGE_SIZE: usize = 320;
pub const MAX_PAYLOAD: usize = 256;
pub const MAX_CAPS: usize = 2;

/// Canonical send envelope. Sender and reserved fields always remain zero.
#[derive(Clone, Debug, PartialEq, Eq)]
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

/// A structurally valid received envelope. `from_bytes` alone does not prove
/// provenance: native::receive obtains these bytes directly from the kernel.
/// Keeping the full original envelope lets services validate sender and reject
/// unexpected capabilities before the caller explicitly closes them.
#[derive(Debug, PartialEq, Eq)]
pub struct Received {
    bytes: [u8; MESSAGE_SIZE],
    length: usize,
    cap_count: usize,
}

impl Received {
    pub fn from_bytes(bytes: [u8; MESSAGE_SIZE], returned_length: u64) -> Result<Self, Error> {
        let (length, cap_count) = validate(&bytes, Some(returned_length))?;
        if get64(&bytes, 0) == 0 {
            return Err(Error::InvalidMessage);
        }
        Ok(Self {
            bytes,
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
