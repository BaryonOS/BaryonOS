#![no_std]
#![forbid(unsafe_code)]

use artifact_store_core::{
    open_checked, read_bounded, ArtifactReadHandle, ArtifactStore, Error as StoreError,
    READ_CHUNK_MAX, SHA256_LEN,
};
pub use artifact_store_core::{ArtifactDescriptor, ArtifactRef};
use sha2::{Digest, Sha256};

pub const PINNED_MODEL_LENGTH: u64 = 988_097_824;
pub const PINNED_MODEL_SHA256: [u8; SHA256_LEN] = [
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
pub enum Error {
    InvalidDescriptor,
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
    StoreFull,
    StoreQuota,
    StoreBusy,
    StoreReplay,
    StoreOutOfOrder,
    StoreIncomplete,
    StoreCancelTooLate,
    StoreUnknown,
    StoreInvalid,
    Protocol,
    AlreadyVerified,
    IntegrityPending,
    Aborted,
}

/// Opens a caller-supplied descriptor through the shared store contract.
/// Metadata alone does not establish trust.
pub fn open<S: ArtifactStore>(
    store: &mut S,
    descriptor: ArtifactDescriptor,
) -> Result<ArtifactVerifier<S>, Error> {
    if descriptor.validate().is_err() {
        return Err(Error::InvalidDescriptor);
    }
    let handle = open_checked(store, &descriptor).map_err(map_store_error)?;
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
pub fn open_pinned_model<S: ArtifactStore>(
    store: &mut S,
    reference: ArtifactRef,
) -> Result<ArtifactVerifier<S>, Error> {
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

pub struct ArtifactVerifier<S: ArtifactStore> {
    descriptor: ArtifactDescriptor,
    handle: Option<ArtifactReadHandle<S::Handle>>,
    offset: u64,
    hasher: Option<Sha256>,
    state: VerificationState,
}

impl<S: ArtifactStore> ArtifactVerifier<S> {
    /// Reads one bounded verification chunk into private scratch memory. A
    /// positive short read is accepted and hashed; no artifact bytes are
    /// exposed until the complete stream matches the descriptor SHA-256.
    /// Requested bytes are charged before I/O, even when the attempt fails.
    pub fn verify_next<C: Cancellation>(
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
        let requested = core::cmp::min(READ_CHUNK_MAX as u64, remaining) as usize;
        if budget.remaining_bytes < requested as u64 {
            return Err(Error::BudgetExceeded);
        }
        budget.remaining_bytes -= requested as u64;

        let mut scratch = [0; READ_CHUNK_MAX];
        let read_result = match self.handle.as_mut() {
            Some(handle) => read_bounded(store, handle, self.offset, &mut scratch[..requested]),
            None => return self.fail(Error::Aborted),
        };
        let amount = match read_result {
            Ok(amount) => amount,
            Err(error) => return self.fail(map_store_error(error)),
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
    pub fn finish(mut self) -> Result<VerifiedArtifact<S::Handle>, Error> {
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

    pub const fn descriptor(&self) -> &ArtifactDescriptor {
        &self.descriptor
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
    handle: Option<ArtifactReadHandle<H>>,
    active: bool,
}

impl<H> VerifiedArtifact<H> {
    /// Reads from the verified immutable generation. Positive short reads are
    /// valid; callers advance by the returned count. Output is valid only on
    /// success, and the same opaque handle must remain bound to this generation.
    pub fn read_at<S: ArtifactStore<Handle = H>, C: Cancellation>(
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
        if output.is_empty() || output.len() > READ_CHUNK_MAX {
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

        let read_result = match self.handle.as_ref() {
            Some(handle) => read_bounded(store, handle, offset, &mut output[..requested]),
            None => return self.fail(Error::Aborted),
        };
        let amount = match read_result {
            Ok(amount) => amount,
            Err(error) => return self.fail(map_store_error(error)),
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
        StoreError::Invalid => Error::StoreInvalid,
        StoreError::Protocol => Error::Protocol,
        StoreError::NotFound => Error::NotFound,
        StoreError::Stale => Error::Stale,
        StoreError::Unauthorized => Error::Unauthorized,
        StoreError::Corrupt => Error::Corrupt,
        StoreError::Truncated => Error::Truncated,
        StoreError::Overflow => Error::Overflow,
        StoreError::EndOfArtifact => Error::EndOfArtifact,
        StoreError::OutOfRange => Error::OutOfRange,
        StoreError::Full => Error::StoreFull,
        StoreError::Quota => Error::StoreQuota,
        StoreError::Busy => Error::StoreBusy,
        StoreError::Replay => Error::StoreReplay,
        StoreError::OutOfOrder => Error::StoreOutOfOrder,
        StoreError::Incomplete => Error::StoreIncomplete,
        StoreError::CancelTooLate => Error::StoreCancelTooLate,
        StoreError::Unknown => Error::StoreUnknown,
        StoreError::Io => Error::StoreUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SingleByteStore {
        reads: usize,
    }

    impl ArtifactStore for SingleByteStore {
        type Handle = ();

        fn open(
            &mut self,
            descriptor: &ArtifactDescriptor,
        ) -> Result<(Self::Handle, ArtifactDescriptor), StoreError> {
            Ok(((), *descriptor))
        }

        fn read_at(
            &mut self,
            _handle: &Self::Handle,
            offset: u64,
            output: &mut [u8],
        ) -> Result<usize, StoreError> {
            self.reads += 1;
            if offset != 0 || output.is_empty() {
                return Ok(0);
            }
            output[0] = b'a';
            Ok(1)
        }

        fn capacity(&mut self) -> Result<artifact_store_core::StoreCapacity, StoreError> {
            Ok(artifact_store_core::StoreCapacity {
                capacity_bytes: 0,
                used_bytes: 0,
                reserved_bytes: 0,
                quota_bytes: 0,
            })
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
    fn shared_store_error_mapping_is_exhaustive() {
        for (store_error, expected) in [
            (StoreError::Invalid, Error::StoreInvalid),
            (StoreError::Unauthorized, Error::Unauthorized),
            (StoreError::Stale, Error::Stale),
            (StoreError::Corrupt, Error::Corrupt),
            (StoreError::Truncated, Error::Truncated),
            (StoreError::Overflow, Error::Overflow),
            (StoreError::EndOfArtifact, Error::EndOfArtifact),
            (StoreError::OutOfRange, Error::OutOfRange),
            (StoreError::NotFound, Error::NotFound),
            (StoreError::Full, Error::StoreFull),
            (StoreError::Quota, Error::StoreQuota),
            (StoreError::Busy, Error::StoreBusy),
            (StoreError::Replay, Error::StoreReplay),
            (StoreError::OutOfOrder, Error::StoreOutOfOrder),
            (StoreError::Incomplete, Error::StoreIncomplete),
            (StoreError::CancelTooLate, Error::StoreCancelTooLate),
            (StoreError::Unknown, Error::StoreUnknown),
            (StoreError::Io, Error::StoreUnavailable),
            (StoreError::Protocol, Error::Protocol),
        ] {
            assert_eq!(map_store_error(store_error), expected);
        }
    }

    #[test]
    fn exact_eof_and_out_of_range_do_not_reach_the_store() {
        let mut sha256 = [0; SHA256_LEN];
        sha256.copy_from_slice(&Sha256::digest(b"a"));
        let descriptor = ArtifactDescriptor {
            reference: ArtifactRef {
                artifact_id: [0xa5; 32],
                immutable_generation: 1,
            },
            length: 1,
            sha256,
        };
        let mut store = SingleByteStore { reads: 0 };
        let mut verifier = open(&mut store, descriptor).unwrap();
        let mut verification_budget = ReadBudget::new(1);
        assert!(verifier
            .verify_next(&mut store, &mut verification_budget, &NeverCancelled)
            .unwrap());
        let mut artifact = verifier.finish().unwrap();
        assert_eq!(store.reads, 1);
        let mut budget = ReadBudget::new(READ_CHUNK_MAX as u64);
        let mut output = [0; 1];
        assert_eq!(
            artifact.read_at(&mut store, 1, &mut budget, &NeverCancelled, &mut output),
            Err(Error::EndOfArtifact)
        );
        assert_eq!(
            artifact.read_at(&mut store, 2, &mut budget, &NeverCancelled, &mut output),
            Err(Error::OutOfRange)
        );
        assert_eq!(store.reads, 1);
    }
}
