#![forbid(unsafe_code)]

//! Store surface for package management. The read side is the shared
//! `artifact-store-core` contract (ADR-054): one definition of references,
//! descriptors, capacity, errors, the 4 KiB read bound and the checked
//! open/read helpers. This module adds only the streaming write transaction.
//!
//! `ArtifactRef` is store object identity; its generation is independent of
//! publisher/source generations carried in package metadata. Descriptors
//! describe bytes but do not prove origin, publisher, freshness, or
//! authorization. A store adapter binds each read handle to the
//! kernel-authenticated caller and its current store epoch, and rechecks both
//! on every read; `open_checked` rejects a store descriptor that differs from
//! the expected one.

use crate::protocol::ProtocolError;

pub use artifact_store_core::{
    open_checked, read_bounded, ArtifactDescriptor, ArtifactReadHandle, ArtifactRef, ArtifactStore,
    Error as StoreError, StoreCapacity, READ_CHUNK_MAX,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransactionKey {
    /// Must be taken from kernel-authenticated message sender state.
    pub caller: u64,
    pub request_id: u64,
    pub service_epoch: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionOutcome {
    Pending { received: u64 },
    Committed { reference: ArtifactRef },
    Cancelled,
    Unknown,
}

/// Write surface for immutable artifacts. `begin` atomically reserves the
/// whole declared object length before accepting chunks. `commit` checks
/// exact length and SHA-256, then publishes atomically. This is storage
/// integrity only: it is not publisher verification or model validation.
/// A separate trusted package policy must gate whether an artifact is
/// installable/launchable; no such persistent issuer API exists yet.
/// Every error except `Unknown` is definitive and means no mutation occurred.
pub trait ArtifactWriter {
    fn begin(
        &mut self,
        key: TransactionKey,
        descriptor: &ArtifactDescriptor,
    ) -> Result<(), StoreError>;

    fn write_chunk(
        &mut self,
        key: TransactionKey,
        offset: u64,
        bytes: &[u8],
    ) -> Result<u64, StoreError>;

    fn commit(
        &mut self,
        key: TransactionKey,
        descriptor: &ArtifactDescriptor,
    ) -> Result<ArtifactRef, StoreError>;

    fn cancel(&mut self, key: TransactionKey) -> Result<(), StoreError>;

    fn outcome(&mut self, key: TransactionKey) -> Result<TransactionOutcome, StoreError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallState {
    Idle,
    Receiving,
    Ready,
    Committing,
    Committed,
    Failed,
    Cancelled,
    Unknown,
}

/// In-memory transaction guard. It prevents replays and partial publication;
/// it is not a durable journal and cannot recover a transaction after restart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallTransaction {
    caller: u64,
    request_id: u64,
    service_epoch: u64,
    last_request_id: u64,
    descriptor: ArtifactDescriptor,
    received: u64,
    state: InstallState,
}

impl InstallTransaction {
    pub const fn new() -> Self {
        Self {
            caller: 0,
            request_id: 0,
            service_epoch: 0,
            last_request_id: 0,
            descriptor: ArtifactDescriptor {
                reference: ArtifactRef {
                    artifact_id: [0; 32],
                    immutable_generation: 0,
                },
                length: 0,
                sha256: [0; 32],
            },
            received: 0,
            state: InstallState::Idle,
        }
    }

    pub fn begin<W: ArtifactWriter>(
        &mut self,
        writer: &mut W,
        caller: u64,
        request_id: u64,
        service_epoch: u64,
        descriptor: ArtifactDescriptor,
    ) -> Result<(), StoreError> {
        if caller == 0 || request_id == 0 || service_epoch == 0 {
            return Err(StoreError::Invalid);
        }
        descriptor.validate()?;
        if matches!(
            self.state,
            InstallState::Receiving
                | InstallState::Ready
                | InstallState::Committing
                | InstallState::Failed
                | InstallState::Unknown
        ) {
            return Err(StoreError::Busy);
        }
        if request_id <= self.last_request_id {
            return Err(StoreError::Replay);
        }
        self.caller = caller;
        self.request_id = request_id;
        self.service_epoch = service_epoch;
        self.descriptor = descriptor;
        self.received = 0;
        self.state = InstallState::Receiving;
        let key = self.key();
        match writer.begin(key, &descriptor) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.last_request_id = request_id;
                self.state = if error == StoreError::Unknown {
                    InstallState::Unknown
                } else {
                    InstallState::Cancelled
                };
                Err(error)
            }
        }
    }

    pub fn append<W: ArtifactWriter>(
        &mut self,
        writer: &mut W,
        caller: u64,
        request_id: u64,
        service_epoch: u64,
        offset: u64,
        bytes: &[u8],
    ) -> Result<u64, StoreError> {
        self.check_transaction(caller, request_id, service_epoch)?;
        if self.state != InstallState::Receiving || bytes.is_empty() || bytes.len() > READ_CHUNK_MAX
        {
            return Err(StoreError::Invalid);
        }
        if offset != self.received {
            return Err(if offset < self.received {
                StoreError::Replay
            } else {
                StoreError::OutOfOrder
            });
        }
        let next = offset
            .checked_add(u64::try_from(bytes.len()).map_err(|_| StoreError::Overflow)?)
            .ok_or(StoreError::Overflow)?;
        if next > self.descriptor.length {
            return Err(StoreError::Overflow);
        }
        let key = self.key();
        let acknowledged = match writer.write_chunk(key, offset, bytes) {
            Ok(acknowledged) => acknowledged,
            Err(error) => {
                self.last_request_id = request_id;
                self.state = if error == StoreError::Unknown {
                    InstallState::Unknown
                } else {
                    InstallState::Failed
                };
                return Err(error);
            }
        };
        if acknowledged != next {
            self.state = InstallState::Unknown;
            self.last_request_id = request_id;
            return Err(StoreError::Protocol);
        }
        self.received = next;
        if self.received == self.descriptor.length {
            self.state = InstallState::Ready;
        }
        Ok(self.received)
    }

    pub fn commit<W: ArtifactWriter>(
        &mut self,
        writer: &mut W,
        caller: u64,
        request_id: u64,
        service_epoch: u64,
    ) -> Result<ArtifactRef, StoreError> {
        self.check_transaction(caller, request_id, service_epoch)?;
        if matches!(
            self.state,
            InstallState::Committing
                | InstallState::Committed
                | InstallState::Cancelled
                | InstallState::Unknown
        ) {
            return Err(StoreError::CancelTooLate);
        }
        if self.state != InstallState::Ready || self.received != self.descriptor.length {
            return Err(StoreError::Incomplete);
        }
        self.state = InstallState::Committing;
        match writer.commit(self.key(), &self.descriptor) {
            Ok(reference) => {
                if reference != self.descriptor.reference {
                    self.state = InstallState::Unknown;
                    self.last_request_id = request_id;
                    return Err(StoreError::Protocol);
                }
                self.state = InstallState::Committed;
                self.last_request_id = request_id;
                Ok(reference)
            }
            Err(error) => {
                self.state = if error == StoreError::Unknown {
                    InstallState::Unknown
                } else {
                    InstallState::Failed
                };
                self.last_request_id = request_id;
                Err(error)
            }
        }
    }

    pub fn cancel<W: ArtifactWriter>(
        &mut self,
        writer: &mut W,
        caller: u64,
        request_id: u64,
        service_epoch: u64,
    ) -> Result<(), StoreError> {
        self.check_transaction(caller, request_id, service_epoch)?;
        if !matches!(
            self.state,
            InstallState::Receiving | InstallState::Ready | InstallState::Failed
        ) {
            return Err(StoreError::CancelTooLate);
        }
        match writer.cancel(self.key()) {
            Ok(()) => {
                self.state = InstallState::Cancelled;
                self.last_request_id = request_id;
                Ok(())
            }
            Err(error) => {
                self.state = if error == StoreError::Unknown {
                    InstallState::Unknown
                } else {
                    InstallState::Failed
                };
                self.last_request_id = request_id;
                Err(error)
            }
        }
    }

    pub fn outcome<W: ArtifactWriter>(
        &mut self,
        writer: &mut W,
        caller: u64,
        request_id: u64,
        service_epoch: u64,
    ) -> Result<TransactionOutcome, StoreError> {
        self.check_transaction(caller, request_id, service_epoch)?;
        let outcome = writer.outcome(self.key())?;
        match outcome {
            TransactionOutcome::Committed { reference }
                if reference == self.descriptor.reference =>
            {
                self.state = InstallState::Committed;
                self.last_request_id = request_id;
                Ok(outcome)
            }
            TransactionOutcome::Committed { .. } => {
                self.state = InstallState::Unknown;
                Err(StoreError::Protocol)
            }
            TransactionOutcome::Pending { received } if received <= self.descriptor.length => {
                Ok(outcome)
            }
            TransactionOutcome::Pending { .. } => {
                self.state = InstallState::Unknown;
                Err(StoreError::Protocol)
            }
            TransactionOutcome::Cancelled => {
                self.state = InstallState::Cancelled;
                self.last_request_id = request_id;
                Ok(outcome)
            }
            TransactionOutcome::Unknown => {
                self.state = InstallState::Unknown;
                self.last_request_id = request_id;
                Ok(outcome)
            }
        }
    }

    pub const fn state(&self) -> InstallState {
        self.state
    }

    pub const fn received(&self) -> u64 {
        self.received
    }

    pub const fn descriptor(&self) -> &ArtifactDescriptor {
        &self.descriptor
    }

    fn key(&self) -> TransactionKey {
        TransactionKey {
            caller: self.caller,
            request_id: self.request_id,
            service_epoch: self.service_epoch,
        }
    }

    fn check_transaction(
        &self,
        caller: u64,
        request_id: u64,
        service_epoch: u64,
    ) -> Result<(), StoreError> {
        if caller != self.caller || request_id != self.request_id {
            return Err(StoreError::Replay);
        }
        if service_epoch != self.service_epoch {
            return Err(StoreError::Stale);
        }
        Ok(())
    }
}

impl Default for InstallTransaction {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ProtocolError> for StoreError {
    fn from(error: ProtocolError) -> Self {
        match error {
            ProtocolError::WrongEpoch | ProtocolError::StaleGrant => Self::Stale,
            ProtocolError::Replay => Self::Replay,
            ProtocolError::OutOfOrder => Self::OutOfOrder,
            ProtocolError::Overflow => Self::Overflow,
            ProtocolError::Incomplete => Self::Incomplete,
            ProtocolError::CancelTooLate => Self::CancelTooLate,
            _ => Self::Protocol,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(length: u64) -> ArtifactDescriptor {
        ArtifactDescriptor {
            reference: ArtifactRef {
                artifact_id: [1; 32],
                immutable_generation: 3,
            },
            length,
            sha256: [5; 32],
        }
    }

    /// Adapter-private handle: the store epoch and locator stay inside the
    /// concrete store, as the shared contract requires.
    struct FakeHandle {
        epoch: u64,
        locator: u64,
    }

    struct FakeStore {
        authoritative: ArtifactDescriptor,
        epoch: u64,
        locator: u64,
        opens: usize,
        read_count: usize,
        read_error: Option<StoreError>,
    }

    impl FakeStore {
        fn new(authoritative: ArtifactDescriptor, read_count: usize) -> Self {
            Self {
                authoritative,
                epoch: 7,
                locator: 11,
                opens: 0,
                read_count,
                read_error: None,
            }
        }
    }

    impl ArtifactStore for FakeStore {
        type Handle = FakeHandle;

        fn open(
            &mut self,
            _expected: &ArtifactDescriptor,
        ) -> Result<(Self::Handle, ArtifactDescriptor), StoreError> {
            self.opens += 1;
            if self.locator == 0 {
                return Err(StoreError::NotFound);
            }
            Ok((
                FakeHandle {
                    epoch: self.epoch,
                    locator: self.locator,
                },
                self.authoritative,
            ))
        }

        fn read_at(
            &mut self,
            handle: &Self::Handle,
            _offset: u64,
            output: &mut [u8],
        ) -> Result<usize, StoreError> {
            if handle.epoch != self.epoch || handle.locator != self.locator {
                return Err(StoreError::Stale);
            }
            if let Some(error) = self.read_error {
                return Err(error);
            }
            output.fill(0x5a);
            Ok(self.read_count)
        }

        fn capacity(&mut self) -> Result<StoreCapacity, StoreError> {
            Ok(StoreCapacity {
                capacity_bytes: 4096,
                used_bytes: 1024,
                reserved_bytes: 512,
                quota_bytes: 3072,
            })
        }
    }

    #[test]
    fn capacity_math_is_u64_bounded_and_fail_closed() {
        assert_eq!(
            StoreCapacity {
                capacity_bytes: 4096,
                used_bytes: 1024,
                reserved_bytes: 512,
                quota_bytes: 3072,
            }
            .available_bytes(),
            Ok(1536)
        );
        assert_eq!(
            StoreCapacity {
                capacity_bytes: u64::MAX,
                used_bytes: u64::MAX,
                reserved_bytes: 1,
                quota_bytes: u64::MAX,
            }
            .available_bytes(),
            Err(StoreError::Overflow)
        );
        assert_eq!(
            StoreCapacity {
                capacity_bytes: 1,
                used_bytes: 2,
                reserved_bytes: 0,
                quota_bytes: 8,
            }
            .available_bytes(),
            Err(StoreError::Corrupt)
        );
    }

    #[test]
    fn immutable_handles_are_epoch_bound_and_reads_are_bounded() {
        let descriptor = descriptor(5000);
        let mut store = FakeStore::new(descriptor, READ_CHUNK_MAX);
        let handle = open_checked(&mut store, &descriptor).unwrap();
        assert_eq!(handle.descriptor(), &descriptor);
        let mut bytes = [0; READ_CHUNK_MAX];
        assert_eq!(
            read_bounded(&mut store, &handle, 0, &mut bytes).unwrap(),
            READ_CHUNK_MAX
        );
        assert_eq!(bytes[0], 0x5a);
        // A new store epoch invalidates every handle opened before it.
        store.epoch = 8;
        assert_eq!(
            read_bounded(&mut store, &handle, 0, &mut bytes),
            Err(StoreError::Stale)
        );
        store.epoch = 7;
        assert_eq!(
            read_bounded(&mut store, &handle, 5000, &mut bytes),
            Err(StoreError::EndOfArtifact)
        );
        assert_eq!(
            read_bounded(&mut store, &handle, 5001, &mut bytes),
            Err(StoreError::OutOfRange)
        );
        assert_eq!(
            read_bounded(&mut store, &handle, 0, &mut [0; READ_CHUNK_MAX + 1]),
            Err(StoreError::Invalid)
        );
        assert_eq!(store.capacity().unwrap().available_bytes(), Ok(1536));
    }

    #[test]
    fn open_rejects_a_store_descriptor_that_differs_from_the_expected_one() {
        let expected = descriptor(8);
        for authoritative in [
            ArtifactDescriptor {
                sha256: [6; 32],
                ..expected
            },
            ArtifactDescriptor {
                length: 9,
                ..expected
            },
            ArtifactDescriptor {
                reference: ArtifactRef {
                    artifact_id: [1; 32],
                    immutable_generation: 4,
                },
                ..expected
            },
            // A different artifact with identical generation, length and digest.
            ArtifactDescriptor {
                reference: ArtifactRef {
                    artifact_id: [2; 32],
                    immutable_generation: 3,
                },
                ..expected
            },
        ] {
            let mut store = FakeStore::new(authoritative, 8);
            assert!(matches!(
                open_checked(&mut store, &expected),
                Err(StoreError::Corrupt)
            ));
        }
        // An invalid expectation is refused before the store is consulted.
        let mut store = FakeStore::new(expected, 8);
        assert!(matches!(
            open_checked(&mut store, &descriptor(0)),
            Err(StoreError::Invalid)
        ));
        assert_eq!(store.opens, 0);
    }

    #[test]
    fn read_errors_are_explicit_and_never_look_like_eof_success() {
        let descriptor = descriptor(8);
        let mut store = FakeStore::new(descriptor, 0);
        let handle = open_checked(&mut store, &descriptor).unwrap();
        let mut bytes = [0; 8];
        assert_eq!(
            read_bounded(&mut store, &handle, 0, &mut bytes),
            Err(StoreError::Truncated)
        );
        store.read_count = 9;
        assert_eq!(
            read_bounded(&mut store, &handle, 0, &mut bytes),
            Err(StoreError::Protocol)
        );
        store.read_error = Some(StoreError::Overflow);
        assert_eq!(
            read_bounded(&mut store, &handle, 0, &mut bytes),
            Err(StoreError::Overflow)
        );
    }

    #[derive(Default)]
    struct FakeWriter {
        descriptor: Option<ArtifactDescriptor>,
        bytes: [u8; 8],
        received: usize,
        fail_begin: Option<StoreError>,
        fail_commit: Option<StoreError>,
        fail_write: Option<StoreError>,
        fail_cancel: Option<StoreError>,
    }

    impl ArtifactWriter for FakeWriter {
        fn begin(
            &mut self,
            _key: TransactionKey,
            descriptor: &ArtifactDescriptor,
        ) -> Result<(), StoreError> {
            if let Some(error) = self.fail_begin {
                return Err(error);
            }
            self.descriptor = Some(*descriptor);
            self.received = 0;
            Ok(())
        }

        fn write_chunk(
            &mut self,
            _key: TransactionKey,
            offset: u64,
            bytes: &[u8],
        ) -> Result<u64, StoreError> {
            if let Some(error) = self.fail_write {
                return Err(error);
            }
            let start = offset as usize;
            self.bytes[start..start + bytes.len()].copy_from_slice(bytes);
            self.received += bytes.len();
            Ok(self.received as u64)
        }

        fn commit(
            &mut self,
            _key: TransactionKey,
            descriptor: &ArtifactDescriptor,
        ) -> Result<ArtifactRef, StoreError> {
            if let Some(error) = self.fail_commit {
                return Err(error);
            }
            Ok(descriptor.reference)
        }

        fn cancel(&mut self, _key: TransactionKey) -> Result<(), StoreError> {
            if let Some(error) = self.fail_cancel {
                return Err(error);
            }
            self.descriptor = None;
            Ok(())
        }

        fn outcome(&mut self, _key: TransactionKey) -> Result<TransactionOutcome, StoreError> {
            Err(StoreError::Unknown)
        }
    }

    fn start(
        transaction: &mut InstallTransaction,
        writer: &mut FakeWriter,
        request_id: u64,
        length: u64,
    ) {
        transaction
            .begin(writer, 9, request_id, 4, descriptor(length))
            .unwrap();
    }

    #[test]
    fn begin_rejects_invalid_descriptors_without_reaching_the_writer() {
        let valid = descriptor(8);
        for invalid in [
            descriptor(0),
            ArtifactDescriptor {
                reference: ArtifactRef {
                    artifact_id: [0; 32],
                    immutable_generation: 3,
                },
                ..valid
            },
            ArtifactDescriptor {
                reference: ArtifactRef {
                    artifact_id: [1; 32],
                    immutable_generation: 0,
                },
                ..valid
            },
        ] {
            let mut transaction = InstallTransaction::new();
            let mut writer = FakeWriter::default();
            assert_eq!(
                transaction.begin(&mut writer, 9, 1, 4, invalid),
                Err(StoreError::Invalid)
            );
            assert_eq!(transaction.state(), InstallState::Idle);
            assert!(writer.descriptor.is_none());
        }
    }

    #[test]
    fn install_stream_is_ordered_bound_to_caller_and_commits_once() {
        let mut transaction = InstallTransaction::new();
        let mut writer = FakeWriter::default();
        start(&mut transaction, &mut writer, 1, 8);
        assert_eq!(
            transaction.append(&mut writer, 10, 1, 4, 0, &[1, 2]),
            Err(StoreError::Replay)
        );
        assert_eq!(
            transaction.append(&mut writer, 9, 1, 5, 0, &[1, 2]),
            Err(StoreError::Stale)
        );
        assert_eq!(
            transaction.append(&mut writer, 9, 1, 4, 1, &[1, 2]),
            Err(StoreError::OutOfOrder)
        );
        assert_eq!(transaction.append(&mut writer, 9, 1, 4, 0, &[1, 2]), Ok(2));
        assert_eq!(transaction.state(), InstallState::Receiving);
        assert_eq!(
            transaction.commit(&mut writer, 9, 1, 4),
            Err(StoreError::Incomplete)
        );
        assert_eq!(
            transaction.append(&mut writer, 9, 1, 4, 2, &[3, 4, 5, 6, 7, 8]),
            Ok(8)
        );
        assert_eq!(transaction.state(), InstallState::Ready);
        let reference = transaction.commit(&mut writer, 9, 1, 4).unwrap();
        assert_eq!(reference, descriptor(8).reference);
        assert_eq!(transaction.state(), InstallState::Committed);
        assert_eq!(
            transaction.commit(&mut writer, 9, 1, 4),
            Err(StoreError::CancelTooLate)
        );
        assert_eq!(
            transaction.begin(&mut writer, 9, 1, 4, descriptor(8)),
            Err(StoreError::Replay)
        );
    }

    #[test]
    fn full_cancel_unknown_and_stale_recovery_are_explicit() {
        let mut transaction = InstallTransaction::new();
        let mut writer = FakeWriter::default();
        start(&mut transaction, &mut writer, 1, 8);
        transaction.cancel(&mut writer, 9, 1, 4).unwrap();
        assert_eq!(transaction.state(), InstallState::Cancelled);
        assert_eq!(
            transaction.begin(&mut writer, 9, 1, 4, descriptor(8)),
            Err(StoreError::Replay)
        );

        start(&mut transaction, &mut writer, 2, 8);
        writer.fail_write = Some(StoreError::Full);
        assert_eq!(
            transaction.append(&mut writer, 9, 2, 4, 0, &[1]),
            Err(StoreError::Full)
        );
        assert_eq!(transaction.state(), InstallState::Failed);
        writer.fail_write = None;
        transaction.cancel(&mut writer, 9, 2, 4).unwrap();

        start(&mut transaction, &mut writer, 3, 1);
        writer.fail_commit = Some(StoreError::Unknown);
        transaction.append(&mut writer, 9, 3, 4, 0, &[1]).unwrap();
        assert_eq!(
            transaction.commit(&mut writer, 9, 3, 4),
            Err(StoreError::Unknown)
        );
        assert_eq!(transaction.state(), InstallState::Unknown);
        assert_eq!(
            transaction.begin(&mut writer, 9, 4, 4, descriptor(1)),
            Err(StoreError::Busy)
        );
        assert_eq!(
            transaction.outcome(&mut writer, 9, 3, 5),
            Err(StoreError::Stale)
        );
    }

    #[test]
    fn unknown_begin_and_cancel_do_not_replay() {
        let mut transaction = InstallTransaction::new();
        let mut writer = FakeWriter {
            fail_begin: Some(StoreError::Unknown),
            ..FakeWriter::default()
        };
        assert_eq!(
            transaction.begin(&mut writer, 9, 1, 4, descriptor(1)),
            Err(StoreError::Unknown)
        );
        assert_eq!(transaction.state(), InstallState::Unknown);
        assert_eq!(
            transaction.begin(&mut writer, 9, 2, 4, descriptor(1)),
            Err(StoreError::Busy)
        );
    }
}
