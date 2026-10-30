use artifact_store_core::{ArtifactStore, Error as StoreError, StoreCapacity, READ_CHUNK_MAX};
use core::cell::Cell;
use nieuw_os_model_store_contract::{
    open, open_pinned_model, ArtifactDescriptor, ArtifactRef, Cancellation, Error, ReadBudget,
    PINNED_MODEL, PINNED_MODEL_LENGTH, PINNED_MODEL_SHA256,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

const MODEL_REFERENCE: ArtifactRef = ArtifactRef {
    artifact_id: [0xa5; 32],
    immutable_generation: 3,
};
const SHA256_ABCD: [u8; 32] = [
    0x88, 0xd4, 0x26, 0x6f, 0xd4, 0xe6, 0x33, 0x8d, 0x13, 0xb8, 0x45, 0xfc, 0xf2, 0x89, 0x57, 0x9d,
    0x20, 0x9c, 0x89, 0x78, 0x23, 0xb9, 0x21, 0x7d, 0xa3, 0xe1, 0x61, 0x93, 0x6f, 0x03, 0x15, 0x89,
];

struct TestHandle {
    reference: ArtifactRef,
    closed: Option<Arc<AtomicBool>>,
}

impl Drop for TestHandle {
    fn drop(&mut self) {
        if let Some(closed) = &self.closed {
            closed.store(true, Ordering::SeqCst);
        }
    }
}

struct Store {
    descriptor: ArtifactDescriptor,
    bytes: Vec<u8>,
    open_error: Option<StoreError>,
    read_error: Option<StoreError>,
    max_read: Option<usize>,
    reported_count: Option<usize>,
    open_calls: usize,
    read_calls: usize,
    last_opened: Option<ArtifactDescriptor>,
    closed: Option<Arc<AtomicBool>>,
}

impl Store {
    fn pinned() -> Self {
        Self {
            descriptor: ArtifactDescriptor {
                reference: MODEL_REFERENCE,
                length: PINNED_MODEL_LENGTH,
                sha256: PINNED_MODEL_SHA256,
            },
            bytes: Vec::new(),
            open_error: None,
            read_error: None,
            max_read: None,
            reported_count: None,
            open_calls: 0,
            read_calls: 0,
            last_opened: None,
            closed: None,
        }
    }

    fn fixture(descriptor: ArtifactDescriptor, actual: &[u8]) -> Self {
        Self {
            descriptor,
            bytes: actual.to_vec(),
            open_error: None,
            read_error: None,
            max_read: None,
            reported_count: None,
            open_calls: 0,
            read_calls: 0,
            last_opened: None,
            closed: None,
        }
    }

    fn track_close(&mut self) -> Arc<AtomicBool> {
        let closed = Arc::new(AtomicBool::new(false));
        self.closed = Some(Arc::clone(&closed));
        closed
    }
}

fn fixture_descriptor(length: u64, sha256: [u8; 32]) -> ArtifactDescriptor {
    ArtifactDescriptor {
        reference: MODEL_REFERENCE,
        length,
        sha256,
    }
}

impl ArtifactStore for Store {
    type Handle = TestHandle;

    fn open(
        &mut self,
        expected: &ArtifactDescriptor,
    ) -> Result<(Self::Handle, ArtifactDescriptor), StoreError> {
        self.open_calls += 1;
        self.last_opened = Some(*expected);
        if let Some(error) = self.open_error {
            return Err(error);
        }
        Ok((
            TestHandle {
                reference: self.descriptor.reference,
                closed: self.closed.as_ref().map(Arc::clone),
            },
            self.descriptor,
        ))
    }

    fn read_at(
        &mut self,
        handle: &Self::Handle,
        offset: u64,
        output: &mut [u8],
    ) -> Result<usize, StoreError> {
        self.read_calls += 1;
        if let Some(error) = self.read_error {
            return Err(error);
        }
        if handle.reference != self.descriptor.reference {
            return Err(StoreError::Stale);
        }
        if let Some(count) = self.reported_count.take() {
            return Ok(count);
        }
        let start = usize::try_from(offset).map_err(|_| StoreError::Io)?;
        if start >= self.bytes.len() {
            return Ok(0);
        }
        let available = self.bytes.len() - start;
        let amount = core::cmp::min(output.len(), available);
        let amount = self
            .max_read
            .map_or(amount, |limit| core::cmp::min(amount, limit));
        output[..amount].copy_from_slice(&self.bytes[start..start + amount]);
        Ok(amount)
    }

    fn capacity(&mut self) -> Result<StoreCapacity, StoreError> {
        Ok(StoreCapacity {
            capacity_bytes: 0,
            used_bytes: 0,
            reserved_bytes: 0,
            quota_bytes: 0,
        })
    }
}

struct Cancelled<'a>(&'a Cell<bool>);

impl Cancellation for Cancelled<'_> {
    fn is_cancelled(&self) -> bool {
        self.0.get()
    }
}

struct NeverCancelled;

impl Cancellation for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

struct CancelDuringRead<'a> {
    inner: Store,
    cancelled: &'a Cell<bool>,
}

impl ArtifactStore for CancelDuringRead<'_> {
    type Handle = TestHandle;

    fn open(
        &mut self,
        expected: &ArtifactDescriptor,
    ) -> Result<(Self::Handle, ArtifactDescriptor), StoreError> {
        self.inner.open(expected)
    }

    fn read_at(
        &mut self,
        handle: &Self::Handle,
        offset: u64,
        output: &mut [u8],
    ) -> Result<usize, StoreError> {
        let result = self.inner.read_at(handle, offset, output);
        self.cancelled.set(true);
        result
    }

    fn capacity(&mut self) -> Result<StoreCapacity, StoreError> {
        self.inner.capacity()
    }
}

#[test]
fn candidate_pin_is_fixed_to_the_reviewed_model_revision() {
    assert_eq!(PINNED_MODEL.repository(), "Qwen/Qwen2.5-0.5B-Instruct");
    assert_eq!(
        PINNED_MODEL.revision(),
        "7ae557604adf67be50417f59c2c2f167def9a775"
    );
    assert_eq!(PINNED_MODEL.filename(), "model.safetensors");
    assert_eq!(PINNED_MODEL.format(), "safetensors");
    assert_eq!(PINNED_MODEL.license(), "Apache-2.0");
    assert_eq!(PINNED_MODEL.length(), 988_097_824);
    assert_eq!(PINNED_MODEL.length(), PINNED_MODEL_LENGTH);
    assert_eq!(PINNED_MODEL.sha256(), PINNED_MODEL_SHA256);
}

#[test]
fn model_open_uses_store_assigned_opaque_reference_without_mapping() {
    let mut store = Store::pinned();
    let verifier = open_pinned_model(&mut store, MODEL_REFERENCE).unwrap();
    let requested = store.last_opened.unwrap();
    assert_eq!(requested.reference.artifact_id, [0xa5; 32]);
    assert_eq!(requested.reference.immutable_generation, 3);
    assert_eq!(requested.length, PINNED_MODEL_LENGTH);
    assert_eq!(requested.sha256, PINNED_MODEL_SHA256);
    assert_eq!(store.open_calls, 1);
    verifier.abort();
}

#[test]
fn zero_reference_or_generation_is_rejected_before_store_open() {
    for reference in [
        ArtifactRef {
            artifact_id: [0; 32],
            immutable_generation: 1,
        },
        ArtifactRef {
            artifact_id: [0xa5; 32],
            immutable_generation: 0,
        },
    ] {
        let mut store = Store::pinned();
        let descriptor = ArtifactDescriptor {
            reference,
            length: PINNED_MODEL_LENGTH,
            sha256: PINNED_MODEL_SHA256,
        };
        assert_eq!(
            open(&mut store, descriptor).err(),
            Some(Error::InvalidDescriptor)
        );
        assert_eq!(store.open_calls, 0);
    }

    let mut store = Store::pinned();
    let descriptor = ArtifactDescriptor {
        reference: MODEL_REFERENCE,
        length: 0,
        sha256: PINNED_MODEL_SHA256,
    };
    assert_eq!(
        open(&mut store, descriptor).err(),
        Some(Error::InvalidDescriptor)
    );
    assert_eq!(store.open_calls, 0);
}

#[test]
fn store_authority_must_match_requested_model_descriptor() {
    let mut store = Store::pinned();
    store.descriptor.sha256[0] ^= 1;
    assert_eq!(
        open_pinned_model(&mut store, MODEL_REFERENCE).err(),
        Some(Error::Corrupt)
    );
    assert_eq!(store.read_calls, 0);

    store = Store::pinned();
    store.descriptor.length -= 1;
    assert_eq!(
        open_pinned_model(&mut store, MODEL_REFERENCE).err(),
        Some(Error::Corrupt)
    );
    assert_eq!(store.read_calls, 0);
}

#[test]
fn stale_generation_fails_closed_during_verification_and_drops_handle() {
    let mut store = Store::pinned();
    let closed = store.track_close();
    let mut verifier = open_pinned_model(&mut store, MODEL_REFERENCE).unwrap();
    store.read_error = Some(StoreError::Stale);
    let mut budget = ReadBudget::new(READ_CHUNK_MAX as u64);
    assert_eq!(
        verifier.verify_next(&mut store, &mut budget, &NeverCancelled),
        Err(Error::Stale)
    );
    assert_eq!(verifier.verified_offset(), 0);
    assert_eq!(budget.remaining_bytes(), 0);
    assert!(closed.load(Ordering::SeqCst));
    assert_eq!(store.read_calls, 1);
}

#[test]
fn corruption_and_storage_truncation_are_not_treated_as_eof_success() {
    for (read_error, expected) in [
        (StoreError::Corrupt, Error::Corrupt),
        (StoreError::Truncated, Error::Truncated),
    ] {
        let mut store = Store::pinned();
        let mut verifier = open_pinned_model(&mut store, MODEL_REFERENCE).unwrap();
        store.read_error = Some(read_error);
        let mut budget = ReadBudget::new(READ_CHUNK_MAX as u64);
        assert_eq!(
            verifier.verify_next(&mut store, &mut budget, &NeverCancelled),
            Err(expected)
        );
        assert_eq!(verifier.verified_offset(), 0);
    }
}

#[test]
fn positive_short_reads_verify_then_read_the_complete_fixture() {
    let descriptor = fixture_descriptor(4, SHA256_ABCD);
    let mut store = Store::fixture(descriptor, b"abcd");
    store.max_read = Some(1);
    let closed = store.track_close();
    let mut verifier = open(&mut store, descriptor).unwrap();
    let mut budget = ReadBudget::new(10);

    for offset in 1..=4 {
        assert_eq!(
            verifier
                .verify_next(&mut store, &mut budget, &NeverCancelled)
                .unwrap(),
            offset == 4
        );
        assert_eq!(verifier.verified_offset(), offset);
    }
    assert_eq!(budget.remaining_bytes(), 0);
    assert_eq!(store.read_calls, 4);
    let mut artifact = verifier.finish().unwrap();
    assert_eq!(artifact.descriptor(), descriptor);
    assert!(!closed.load(Ordering::SeqCst));

    let mut output = [0; 4];
    let mut budget = ReadBudget::new(10);
    for offset in 0..4 {
        assert_eq!(
            artifact.read_at(
                &mut store,
                offset as u64,
                &mut budget,
                &NeverCancelled,
                &mut output[offset..]
            ),
            Ok(1)
        );
    }
    assert_eq!(&output, b"abcd");
    assert_eq!(budget.remaining_bytes(), 0);
    assert_eq!(
        artifact.read_at(&mut store, 4, &mut budget, &NeverCancelled, &mut output),
        Err(Error::EndOfArtifact)
    );
    assert_eq!(store.read_calls, 8);
    artifact.abort();
    assert!(closed.load(Ordering::SeqCst));
}

#[test]
fn unfinished_verification_never_publishes_a_handle_and_releases_it() {
    for read_first_chunk in [false, true] {
        let descriptor = fixture_descriptor(4, SHA256_ABCD);
        let mut store = Store::fixture(descriptor, b"abcd");
        store.max_read = Some(1);
        let closed = store.track_close();
        let mut verifier = open(&mut store, descriptor).unwrap();
        if read_first_chunk {
            assert!(!verifier
                .verify_next(&mut store, &mut ReadBudget::new(4), &NeverCancelled)
                .unwrap());
        }
        assert_eq!(verifier.finish().err(), Some(Error::IntegrityPending));
        assert!(closed.load(Ordering::SeqCst));
        assert_eq!(store.read_calls, usize::from(read_first_chunk));
    }
}

#[test]
fn descriptor_mismatch_drops_the_root_handle_without_reading() {
    let mut store = Store::pinned();
    let closed = store.track_close();
    store.descriptor.sha256[0] ^= 1;
    assert_eq!(
        open_pinned_model(&mut store, MODEL_REFERENCE).err(),
        Some(Error::Corrupt)
    );
    assert!(closed.load(Ordering::SeqCst));
    assert_eq!(store.read_calls, 0);
}

#[test]
fn verified_reads_fail_closed_on_revocation_and_quota_errors() {
    for (store_error, expected) in [
        (StoreError::Unauthorized, Error::Unauthorized),
        (StoreError::Quota, Error::StoreQuota),
        (StoreError::Io, Error::StoreUnavailable),
    ] {
        let descriptor = fixture_descriptor(4, SHA256_ABCD);
        let mut store = Store::fixture(descriptor, b"abcd");
        let closed = store.track_close();
        let mut verifier = open(&mut store, descriptor).unwrap();
        assert!(verifier
            .verify_next(&mut store, &mut ReadBudget::new(4), &NeverCancelled)
            .unwrap());
        let mut artifact = verifier.finish().unwrap();
        store.read_error = Some(store_error);
        let mut budget = ReadBudget::new(4);
        let mut output = [0; 4];
        assert_eq!(
            artifact.read_at(&mut store, 0, &mut budget, &NeverCancelled, &mut output),
            Err(expected)
        );
        assert_eq!(budget.remaining_bytes(), 0);
        assert!(closed.load(Ordering::SeqCst));
        store.read_error = None;
        assert_eq!(
            artifact.read_at(&mut store, 0, &mut budget, &NeverCancelled, &mut output),
            Err(Error::Aborted)
        );
        assert_eq!(store.read_calls, 2);
    }
}

#[test]
fn cancellation_after_verification_before_or_during_io_releases_handle() {
    for cancel_during_io in [false, true] {
        let descriptor = fixture_descriptor(4, SHA256_ABCD);
        let mut store = Store::fixture(descriptor, b"abcd");
        let closed = store.track_close();
        let mut verifier = open(&mut store, descriptor).unwrap();
        assert!(verifier
            .verify_next(&mut store, &mut ReadBudget::new(4), &NeverCancelled)
            .unwrap());
        let mut artifact = verifier.finish().unwrap();
        let cancelled = Cell::new(!cancel_during_io);
        let mut store = CancelDuringRead {
            inner: store,
            cancelled: &cancelled,
        };
        let mut budget = ReadBudget::new(4);
        let mut output = [0; 4];
        assert_eq!(
            artifact.read_at(
                &mut store,
                0,
                &mut budget,
                &Cancelled(&cancelled),
                &mut output
            ),
            Err(Error::Cancelled)
        );
        assert!(closed.load(Ordering::SeqCst));
        assert_eq!(
            budget.remaining_bytes(),
            if cancel_during_io { 0 } else { 4 }
        );
        assert_eq!(store.inner.read_calls, if cancel_during_io { 2 } else { 1 });
        cancelled.set(false);
        assert_eq!(
            artifact.read_at(
                &mut store,
                0,
                &mut budget,
                &Cancelled(&cancelled),
                &mut output
            ),
            Err(Error::Aborted)
        );
        assert_eq!(store.inner.read_calls, if cancel_during_io { 2 } else { 1 });
    }
}

#[test]
fn positive_short_reads_hash_the_complete_tampered_stream_and_reject_it() {
    let expected = b"abcd";
    let mut store = Store::fixture(fixture_descriptor(4, SHA256_ABCD), b"abce");
    store.max_read = Some(1);
    let closed = store.track_close();
    let descriptor = store.descriptor;
    let mut verifier = open(&mut store, descriptor).unwrap();
    let mut budget = ReadBudget::new(32);

    for _ in 0..3 {
        assert!(!verifier
            .verify_next(&mut store, &mut budget, &NeverCancelled)
            .unwrap());
    }
    assert_eq!(
        verifier.verify_next(&mut store, &mut budget, &NeverCancelled),
        Err(Error::DigestMismatch)
    );
    assert_eq!(verifier.verified_offset(), expected.len() as u64);
    assert_eq!(store.read_calls, expected.len());
    assert_eq!(budget.remaining_bytes(), 22);
    assert!(closed.load(Ordering::SeqCst));
    assert_eq!(verifier.finish().err(), Some(Error::Aborted));
}

#[test]
fn zero_before_eof_after_positive_short_read_is_truncated() {
    let mut store = Store::fixture(fixture_descriptor(4, SHA256_ABCD), b"a");
    let descriptor = store.descriptor;
    let mut verifier = open(&mut store, descriptor).unwrap();
    let mut budget = ReadBudget::new(8);
    assert!(!verifier
        .verify_next(&mut store, &mut budget, &NeverCancelled)
        .unwrap());
    assert_eq!(verifier.verified_offset(), 1);
    assert_eq!(
        verifier.verify_next(&mut store, &mut budget, &NeverCancelled),
        Err(Error::Truncated)
    );
    assert_eq!(verifier.verified_offset(), 1);
    assert_eq!(store.read_calls, 2);
}

#[test]
fn provider_counts_larger_than_buffer_or_remaining_are_protocol_errors() {
    let mut store = Store::fixture(
        fixture_descriptor(READ_CHUNK_MAX as u64, [0; 32]),
        &vec![0; READ_CHUNK_MAX],
    );
    store.reported_count = Some(READ_CHUNK_MAX + 1);
    let descriptor = store.descriptor;
    let mut verifier = open(&mut store, descriptor).unwrap();
    let mut budget = ReadBudget::new(READ_CHUNK_MAX as u64);
    assert_eq!(
        verifier.verify_next(&mut store, &mut budget, &NeverCancelled),
        Err(Error::Protocol)
    );
    assert_eq!(budget.remaining_bytes(), 0);

    let mut store = Store::fixture(fixture_descriptor(1, [0; 32]), b"x");
    store.reported_count = Some(2);
    let descriptor = store.descriptor;
    let mut verifier = open(&mut store, descriptor).unwrap();
    budget = ReadBudget::new(1);
    assert_eq!(
        verifier.verify_next(&mut store, &mut budget, &NeverCancelled),
        Err(Error::Protocol)
    );
}

#[test]
fn byte_budget_and_pre_io_cancellation_block_reads_and_release_handle() {
    let mut store = Store::pinned();
    let mut verifier = open_pinned_model(&mut store, MODEL_REFERENCE).unwrap();
    let mut budget = ReadBudget::new((READ_CHUNK_MAX - 1) as u64);
    assert_eq!(
        verifier.verify_next(&mut store, &mut budget, &NeverCancelled),
        Err(Error::BudgetExceeded)
    );
    assert_eq!(store.read_calls, 0);
    assert_eq!(budget.remaining_bytes(), (READ_CHUNK_MAX - 1) as u64);
    verifier.abort();

    let mut store = Store::pinned();
    let closed = store.track_close();
    let mut verifier = open_pinned_model(&mut store, MODEL_REFERENCE).unwrap();
    let cancelled = Cell::new(true);
    let cancellation = Cancelled(&cancelled);
    budget = ReadBudget::new(READ_CHUNK_MAX as u64);
    assert_eq!(
        verifier.verify_next(&mut store, &mut budget, &cancellation),
        Err(Error::Cancelled)
    );
    assert_eq!(store.read_calls, 0);
    assert_eq!(budget.remaining_bytes(), READ_CHUNK_MAX as u64);
    assert!(closed.load(Ordering::SeqCst));
}

#[test]
fn stale_generation_after_verification_fails_read_and_drops_handle() {
    let bytes = b"abcd";
    let descriptor = fixture_descriptor(bytes.len() as u64, SHA256_ABCD);
    let mut store = Store::fixture(descriptor, bytes);
    let closed = store.track_close();
    let mut verifier = open(&mut store, descriptor).unwrap();
    let mut verification_budget = ReadBudget::new(bytes.len() as u64);

    assert!(verifier
        .verify_next(&mut store, &mut verification_budget, &NeverCancelled)
        .unwrap());
    let mut artifact = verifier.finish().unwrap();
    assert!(!closed.load(Ordering::SeqCst));

    store.read_error = Some(StoreError::Stale);
    let mut read_budget = ReadBudget::new(bytes.len() as u64);
    let mut output = [0; 4];
    assert_eq!(
        artifact.read_at(
            &mut store,
            0,
            &mut read_budget,
            &NeverCancelled,
            &mut output
        ),
        Err(Error::Stale)
    );
    assert!(closed.load(Ordering::SeqCst));
    assert_eq!(
        artifact.read_at(
            &mut store,
            0,
            &mut read_budget,
            &NeverCancelled,
            &mut output
        ),
        Err(Error::Aborted)
    );
    assert_eq!(store.read_calls, 2);
}

#[test]
fn cancellation_during_store_io_drops_handle_without_publishing_partial_data() {
    let mut inner = Store::fixture(
        fixture_descriptor((READ_CHUNK_MAX * 2) as u64, [0; 32]),
        &vec![1; READ_CHUNK_MAX],
    );
    inner.max_read = Some(READ_CHUNK_MAX);
    let closed = inner.track_close();
    let cancelled = Cell::new(false);
    let mut store = CancelDuringRead {
        inner,
        cancelled: &cancelled,
    };
    let descriptor = store.inner.descriptor;
    let mut verifier = open(&mut store, descriptor).unwrap();
    let mut budget = ReadBudget::new(READ_CHUNK_MAX as u64);
    assert_eq!(
        verifier.verify_next(&mut store, &mut budget, &Cancelled(&cancelled)),
        Err(Error::Cancelled)
    );
    assert_eq!(verifier.verified_offset(), 0);
    assert_eq!(budget.remaining_bytes(), 0);
    assert!(closed.load(Ordering::SeqCst));
}

#[test]
fn descriptor_open_errors_are_not_hidden() {
    for (store_error, expected) in [
        (StoreError::NotFound, Error::NotFound),
        (StoreError::Stale, Error::Stale),
        (StoreError::Unauthorized, Error::Unauthorized),
        (StoreError::Io, Error::StoreUnavailable),
    ] {
        let mut store = Store::pinned();
        store.open_error = Some(store_error);
        assert_eq!(
            open_pinned_model(&mut store, MODEL_REFERENCE).err(),
            Some(expected)
        );
        assert_eq!(store.read_calls, 0);
    }
}
