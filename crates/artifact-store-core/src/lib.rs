#![no_std]
#![forbid(unsafe_code)]

//! Bounded immutable artifact-store contracts without wire or backend policy.

pub const ARTIFACT_ID_LEN: usize = 32;
pub const SHA256_LEN: usize = 32;
pub const READ_CHUNK_MAX: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactRef {
    pub artifact_id: [u8; ARTIFACT_ID_LEN],
    pub immutable_generation: u64,
}

impl ArtifactRef {
    /// Reject a zero identifier or generation.
    pub fn validate(&self) -> Result<(), Error> {
        if self.artifact_id.iter().all(|byte| *byte == 0) || self.immutable_generation == 0 {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactDescriptor {
    pub reference: ArtifactRef,
    pub length: u64,
    pub sha256: [u8; SHA256_LEN],
}

impl ArtifactDescriptor {
    /// Validate the reference and require a non-empty artifact.
    pub fn validate(&self) -> Result<(), Error> {
        self.reference.validate()?;
        if self.length == 0 {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreCapacity {
    pub capacity_bytes: u64,
    pub used_bytes: u64,
    pub reserved_bytes: u64,
    pub quota_bytes: u64,
}

impl StoreCapacity {
    /// Compute free bytes within the tighter of the store and caller quota.
    ///
    /// Returns [`Error::Overflow`] if used plus reserved bytes overflow, or
    /// [`Error::Corrupt`] if committed usage exceeds the applicable limit.
    pub fn available_bytes(&self) -> Result<u64, Error> {
        let limit = core::cmp::min(self.capacity_bytes, self.quota_bytes);
        let committed = self
            .used_bytes
            .checked_add(self.reserved_bytes)
            .ok_or(Error::Overflow)?;
        limit.checked_sub(committed).ok_or(Error::Corrupt)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Unauthorized,
    Stale,
    Corrupt,
    Truncated,
    Overflow,
    EndOfArtifact,
    OutOfRange,
    NotFound,
    Full,
    Quota,
    Busy,
    Replay,
    OutOfOrder,
    Incomplete,
    CancelTooLate,
    Unknown,
    Io,
    Protocol,
}

/// A store must bind handles to the authenticated caller and current epoch,
/// then recheck both on every operation. The handle's concrete type should
/// keep any locator and epoch private to that adapter.
pub trait ArtifactStore {
    type Handle;

    /// Returns the store-authoritative descriptor with the opaque handle.
    fn open(
        &mut self,
        expected: &ArtifactDescriptor,
    ) -> Result<(Self::Handle, ArtifactDescriptor), Error>;

    /// Must reauthorize the authenticated caller and reject stale/revoked
    /// handles on every call.
    fn read_at(
        &mut self,
        handle: &Self::Handle,
        offset: u64,
        output: &mut [u8],
    ) -> Result<usize, Error>;

    /// Return the store's current capacity, usage, reservations, and quota.
    fn capacity(&mut self) -> Result<StoreCapacity, Error>;
}

/// Opaque wrapper; callers can only obtain one through `open_checked`.
pub struct ArtifactReadHandle<H> {
    handle: H,
    descriptor: ArtifactDescriptor,
}

impl<H> ArtifactReadHandle<H> {
    /// Return the immutable descriptor validated when this handle was opened.
    pub const fn descriptor(&self) -> &ArtifactDescriptor {
        &self.descriptor
    }
}

/// Open an artifact only when the store returns the exact validated descriptor
/// requested by the caller.
pub fn open_checked<S: ArtifactStore>(
    store: &mut S,
    expected: &ArtifactDescriptor,
) -> Result<ArtifactReadHandle<S::Handle>, Error> {
    expected.validate()?;
    let (handle, actual) = store.open(expected)?;
    actual.validate()?;
    if actual != *expected {
        return Err(Error::Corrupt);
    }
    Ok(ArtifactReadHandle {
        handle,
        descriptor: actual,
    })
}

/// Perform one bounded read. Positive short reads are valid; callers continue
/// at `offset + returned_count`. Output bytes are valid only on `Ok`.
pub fn read_bounded<S: ArtifactStore>(
    store: &mut S,
    handle: &ArtifactReadHandle<S::Handle>,
    offset: u64,
    output: &mut [u8],
) -> Result<usize, Error> {
    let descriptor = handle.descriptor;
    descriptor.validate()?;
    if output.is_empty() || output.len() > READ_CHUNK_MAX {
        return Err(Error::Invalid);
    }
    if offset == descriptor.length {
        return Err(Error::EndOfArtifact);
    }
    if offset > descriptor.length {
        return Err(Error::OutOfRange);
    }

    let remaining = descriptor
        .length
        .checked_sub(offset)
        .ok_or(Error::Overflow)?;
    let requested = core::cmp::min(output.len() as u64, remaining) as usize;
    offset
        .checked_add(requested as u64)
        .ok_or(Error::Overflow)?;

    let read = store.read_at(&handle.handle, offset, &mut output[..requested])?;
    if read == 0 {
        return Err(Error::Truncated);
    }
    if read > requested {
        return Err(Error::Protocol);
    }
    let read = u64::try_from(read).map_err(|_| Error::Overflow)?;
    offset.checked_add(read).ok_or(Error::Overflow)?;
    Ok(read as usize)
}
