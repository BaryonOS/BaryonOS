#![no_std]
#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const READ_CHUNK_BYTES: usize = 4096;
pub const PINNED_MODEL_LENGTH: u64 = 988_097_824;
pub const PINNED_MODEL_SHA256: [u8; 32] = [
    0xfd, 0xf7, 0x56, 0xfa, 0x7f, 0xcb, 0xe7, 0x40, 0x4d, 0x5c, 0x60, 0xe2, 0x6b, 0xff, 0x1a, 0x0c,
    0x8b, 0x8a, 0xa1, 0xf7, 0x2c, 0xed, 0x49, 0xe7, 0xdd, 0x02, 0x10, 0xfe, 0x28, 0x8f, 0xb7, 0xfe,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelPin {
    repository: &'static str,
    revision: &'static str,
    filename: &'static str,
    format: &'static str,
    license: &'static str,
    length: u64,
    sha256: [u8; 32],
}

impl ModelPin {
    pub const fn repository(&self) -> &'static str {
        self.repository
    }

    pub const fn revision(&self) -> &'static str {
        self.revision
    }

    pub const fn filename(&self) -> &'static str {
        self.filename
    }

    pub const fn format(&self) -> &'static str {
        self.format
    }

    pub const fn license(&self) -> &'static str {
        self.license
    }

    pub const fn length(&self) -> u64 {
        self.length
    }

    pub const fn sha256(&self) -> [u8; 32] {
        self.sha256
    }
}

pub const PINNED_MODEL: ModelPin = ModelPin {
    repository: "Qwen/Qwen2.5-0.5B-Instruct",
    revision: "7ae557604adf67be50417f59c2c2f167def9a775",
    filename: "model.safetensors",
    format: "safetensors",
    license: "Apache-2.0",
    length: PINNED_MODEL_LENGTH,
    sha256: PINNED_MODEL_SHA256,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactRef {
    pub artifact_id: [u8; 32],
    pub immutable_generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactDescriptor {
    pub reference: ArtifactRef,
    pub length: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
    DescriptorMismatch,
    NotFound,
    Stale,
    Unauthorized,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadError {
    Stale,
    Unauthorized,
    Corrupt,
    Truncated,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidDescriptor,
    DescriptorMismatch,
    Corrupt,
    Stale,
    Unauthorized,
    DigestMismatch,
    InvalidBuffer,
    BudgetExceeded,
    Cancelled,
    Truncated,
    Overflow,
    OutOfRange,
    EndOfArtifact,
    NotFound,
    StoreUnavailable,
    ReadUnavailable,
    Protocol,
    AlreadyVerified,
    IntegrityPending,
    Aborted,
}

pub trait ReadOnlyArtifactStore {
    type ReadHandle;

    /// Opens only the store-authoritative immutable descriptor that was asked
    /// for. The returned handle is opaque and scoped by its implementation.
    fn open(&mut self, descriptor: ArtifactDescriptor) -> Result<Self::ReadHandle, StoreError>;

    /// Reads the handle's same immutable generation. The consumer verifies a
    /// complete pass before obtaining a handle that exposes artifact bytes.
    fn read_at(
        &mut self,
        handle: &mut Self::ReadHandle,
        offset: u64,
        output: &mut [u8],
    ) -> Result<usize, ReadError>;
}

/// Opens a caller-supplied descriptor after the store has checked it against
/// its authoritative descriptor. Metadata alone does not establish trust.
pub fn open<S: ReadOnlyArtifactStore>(
    store: &mut S,
    descriptor: ArtifactDescriptor,
) -> Result<ArtifactVerifier<S::ReadHandle>, Error> {
    if descriptor.reference.artifact_id == [0; 32] || descriptor.reference.immutable_generation == 0
    {
        return Err(Error::InvalidDescriptor);
    }
    let handle = store.open(descriptor).map_err(map_store_error)?;
    Ok(ArtifactVerifier {
        descriptor,
        handle: Some(handle),
        offset: 0,
        hasher: Some(Sha256::new()),
        state: VerificationState::Verifying,
    })
}

/// Resolves no identity itself: the store-issued reference is supplied by the
/// model catalog and the pinned model metadata supplies expected length/hash.
pub fn open_pinned_model<S: ReadOnlyArtifactStore>(
    store: &mut S,
    reference: ArtifactRef,
) -> Result<ArtifactVerifier<S::ReadHandle>, Error> {
    open(
        store,
        ArtifactDescriptor {
            reference,
            length: PINNED_MODEL.length,
            sha256: PINNED_MODEL.sha256,
        },
    )
}

pub trait Cancellation {
    fn is_cancelled(&self) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadBudget {
    remaining_bytes: u64,
}

impl ReadBudget {
    /// `bytes` is a caller-selected per-operation byte budget, not a product cap.
    pub const fn new(bytes: u64) -> Self {
        Self {
            remaining_bytes: bytes,
        }
    }

    pub const fn remaining_bytes(&self) -> u64 {
        self.remaining_bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VerificationState {
    Verifying,
    Verified,
    Failed,
}

pub struct ArtifactVerifier<H> {
    descriptor: ArtifactDescriptor,
    handle: Option<H>,
    offset: u64,
    hasher: Option<Sha256>,
    state: VerificationState,
}

impl<H> ArtifactVerifier<H> {
    /// Reads one bounded verification chunk into private scratch memory. A
    /// positive short read is accepted and hashed; no artifact bytes are
    /// exposed until the complete stream matches the descriptor SHA-256.
    /// Requested bytes are charged before I/O, even when the attempt fails.
    pub fn verify_next<S: ReadOnlyArtifactStore<ReadHandle = H>, C: Cancellation>(
        &mut self,
        store: &mut S,
        budget: &mut ReadBudget,
        cancellation: &C,
    ) -> Result<bool, Error> {
        match self.state {
            VerificationState::Verifying => {}
            VerificationState::Verified => return Err(Error::AlreadyVerified),
            VerificationState::Failed => return Err(Error::Aborted),
        }
        if cancellation.is_cancelled() {
            return self.fail(Error::Cancelled);
        }
        if self.offset > self.descriptor.length {
            return self.fail(Error::OutOfRange);
        }
        if self.offset == self.descriptor.length {
            return self.finish_verification();
        }

        let remaining = self.descriptor.length - self.offset;
        let requested = core::cmp::min(READ_CHUNK_BYTES as u64, remaining) as usize;
        if budget.remaining_bytes < requested as u64 {
            return Err(Error::BudgetExceeded);
        }
        budget.remaining_bytes -= requested as u64;

        let mut scratch = [0; READ_CHUNK_BYTES];
        let read_result = match self.handle.as_mut() {
            Some(handle) => store.read_at(handle, self.offset, &mut scratch[..requested]),
            None => return self.fail(Error::Aborted),
        };
        let amount = match read_result {
            Ok(amount) => amount,
            Err(error) => return self.fail(map_read_error(error)),
        };
        if amount > requested {
            return self.fail(Error::Protocol);
        }
        if amount == 0 {
            return self.fail(Error::Truncated);
        }
        if cancellation.is_cancelled() {
            return self.fail(Error::Cancelled);
        }

        let end = match checked_end(self.offset, amount, self.descriptor.length) {
            Ok(end) => end,
            Err(error) => return self.fail(error),
        };
        match self.hasher.as_mut() {
            Some(hasher) => hasher.update(&scratch[..amount]),
            None => return self.fail(Error::Aborted),
        }
        self.offset = end;
        if self.offset == self.descriptor.length {
            return self.finish_verification();
        }
        Ok(false)
    }

    /// Transfers the opaque handle only after the full stream matched.
    pub fn finish(mut self) -> Result<VerifiedArtifact<H>, Error> {
        if self.state != VerificationState::Verified {
            return Err(if self.state == VerificationState::Failed {
                Error::Aborted
            } else {
                Error::IntegrityPending
            });
        }
        let handle = self.handle.take().ok_or(Error::Aborted)?;
        Ok(VerifiedArtifact {
            descriptor: self.descriptor,
            handle: Some(handle),
            active: true,
        })
    }

    /// Explicitly aborts verification and drops the opaque handle. A handle
    /// implementation is responsible for closing its process-scoped resource.
    pub fn abort(mut self) {
        self.state = VerificationState::Failed;
        self.hasher.take();
        self.handle.take();
    }

    pub const fn descriptor(&self) -> ArtifactDescriptor {
        self.descriptor
    }

    pub const fn verified_offset(&self) -> u64 {
        self.offset
    }

    fn finish_verification(&mut self) -> Result<bool, Error> {
        let hasher = match self.hasher.take() {
            Some(hasher) => hasher,
            None => return self.fail(Error::Aborted),
        };
        if hasher.finalize().as_slice() != self.descriptor.sha256 {
            return self.fail(Error::DigestMismatch);
        }
        self.state = VerificationState::Verified;
        Ok(true)
    }

    fn fail<T>(&mut self, error: Error) -> Result<T, Error> {
        self.state = VerificationState::Failed;
        self.hasher.take();
        self.handle.take();
        Err(error)
    }
}

pub struct VerifiedArtifact<H> {
    descriptor: ArtifactDescriptor,
    handle: Option<H>,
    active: bool,
}

impl<H> VerifiedArtifact<H> {
    /// Reads from the verified immutable generation. Positive short reads are
    /// valid; callers advance by the returned count. Output is valid only on
    /// success, and the same opaque handle must remain bound to this generation.
    pub fn read_at<S: ReadOnlyArtifactStore<ReadHandle = H>, C: Cancellation>(
        &mut self,
        store: &mut S,
        offset: u64,
        budget: &mut ReadBudget,
        cancellation: &C,
        output: &mut [u8],
    ) -> Result<usize, Error> {
        if !self.active {
            return Err(Error::Aborted);
        }
        if cancellation.is_cancelled() {
            return self.fail(Error::Cancelled);
        }
        if output.is_empty() || output.len() > READ_CHUNK_BYTES {
            return Err(Error::InvalidBuffer);
        }
        if offset > self.descriptor.length {
            return Err(Error::OutOfRange);
        }
        if offset == self.descriptor.length {
            return Err(Error::EndOfArtifact);
        }

        let remaining = self.descriptor.length - offset;
        let requested = core::cmp::min(output.len() as u64, remaining) as usize;
        if budget.remaining_bytes < requested as u64 {
            return Err(Error::BudgetExceeded);
        }
        budget.remaining_bytes -= requested as u64;

        let read_result = match self.handle.as_mut() {
            Some(handle) => store.read_at(handle, offset, &mut output[..requested]),
            None => return self.fail(Error::Aborted),
        };
        let amount = match read_result {
            Ok(amount) => amount,
            Err(error) => return self.fail(map_read_error(error)),
        };
        if amount > requested {
            return self.fail(Error::Protocol);
        }
        if amount == 0 {
            return self.fail(Error::Truncated);
        }
        if cancellation.is_cancelled() {
            return self.fail(Error::Cancelled);
        }
        if let Err(error) = checked_end(offset, amount, self.descriptor.length) {
            return self.fail(error);
        }
        Ok(amount)
    }

    pub fn abort(mut self) {
        self.active = false;
        self.handle.take();
    }

    pub const fn descriptor(&self) -> ArtifactDescriptor {
        self.descriptor
    }

    fn fail<T>(&mut self, error: Error) -> Result<T, Error> {
        self.active = false;
        self.handle.take();
        Err(error)
    }
}

fn checked_end(offset: u64, length: usize, total_bytes: u64) -> Result<u64, Error> {
    let length = u64::try_from(length).map_err(|_| Error::Overflow)?;
    let end = offset.checked_add(length).ok_or(Error::Overflow)?;
    if end > total_bytes {
        return Err(Error::OutOfRange);
    }
    Ok(end)
}

fn map_store_error(error: StoreError) -> Error {
    match error {
        StoreError::DescriptorMismatch => Error::DescriptorMismatch,
        StoreError::NotFound => Error::NotFound,
        StoreError::Stale => Error::Stale,
        StoreError::Unauthorized => Error::Unauthorized,
        StoreError::Unavailable => Error::StoreUnavailable,
    }
}

fn map_read_error(error: ReadError) -> Error {
    match error {
        ReadError::Stale => Error::Stale,
        ReadError::Unauthorized => Error::Unauthorized,
        ReadError::Corrupt => Error::Corrupt,
        ReadError::Truncated => Error::Truncated,
        ReadError::Unavailable => Error::ReadUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoReadStore {
        reads: usize,
    }

    impl ReadOnlyArtifactStore for NoReadStore {
        type ReadHandle = ();

        fn open(
            &mut self,
            _descriptor: ArtifactDescriptor,
        ) -> Result<Self::ReadHandle, StoreError> {
            Ok(())
        }

        fn read_at(
            &mut self,
            _handle: &mut Self::ReadHandle,
            _offset: u64,
            _output: &mut [u8],
        ) -> Result<usize, ReadError> {
            self.reads += 1;
            Ok(0)
        }
    }

    struct NeverCancelled;

    impl Cancellation for NeverCancelled {
        fn is_cancelled(&self) -> bool {
            false
        }
    }

    #[test]
    fn checked_range_arithmetic_fails_closed() {
        assert_eq!(checked_end(u64::MAX, 1, u64::MAX), Err(Error::Overflow));
        assert_eq!(checked_end(9, 2, 10), Err(Error::OutOfRange));
        assert_eq!(checked_end(8, 2, 10), Ok(10));
    }

    #[test]
    fn sha256_matches_known_answer() {
        let expected = [
            0x88, 0xd4, 0x26, 0x6f, 0xd4, 0xe6, 0x33, 0x8d, 0x13, 0xb8, 0x45, 0xfc, 0xf2, 0x89,
            0x57, 0x9d, 0x20, 0x9c, 0x89, 0x78, 0x23, 0xb9, 0x21, 0x7d, 0xa3, 0xe1, 0x61, 0x93,
            0x6f, 0x03, 0x15, 0x89,
        ];
        assert_eq!(Sha256::digest(b"abcd").as_slice(), expected);
    }

    #[test]
    fn exact_eof_and_out_of_range_never_reach_the_store() {
        let descriptor = ArtifactDescriptor {
            reference: ArtifactRef {
                artifact_id: [0xa5; 32],
                immutable_generation: 1,
            },
            length: 1,
            sha256: [0; 32],
        };
        let mut artifact = VerifiedArtifact {
            descriptor,
            handle: Some(()),
            active: true,
        };
        let mut store = NoReadStore { reads: 0 };
        let mut budget = ReadBudget::new(READ_CHUNK_BYTES as u64);
        let mut output = [0; 1];
        assert_eq!(
            artifact.read_at(&mut store, 1, &mut budget, &NeverCancelled, &mut output),
            Err(Error::EndOfArtifact)
        );
        assert_eq!(
            artifact.read_at(&mut store, 2, &mut budget, &NeverCancelled, &mut output),
            Err(Error::OutOfRange)
        );
        assert_eq!(store.reads, 0);
    }
}
