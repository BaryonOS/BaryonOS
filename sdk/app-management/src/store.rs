#![forbid(unsafe_code)]

use crate::protocol::ProtocolError;

pub const READ_CHUNK_MAX: usize = 4096;

/// Store object identity; this generation is independent of publisher/source
/// generations carried in package metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactRef {
    pub artifact_id: [u8; 32],
    pub immutable_generation: u64,
}

/// Store metadata; these fields describe bytes but do not prove their origin,
/// publisher, freshness, or authorization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactDescriptor {
    pub reference: ArtifactRef,
    pub length: u64,
    pub sha256: [u8; 32],
}

fn valid_descriptor(descriptor: &ArtifactDescriptor) -> bool {
    descriptor
        .reference
        .artifact_id
        .iter()
        .any(|byte| *byte != 0)
        && descriptor.reference.immutable_generation != 0
        && descriptor.length != 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreCapacity {
    pub capacity_bytes: u64,
    pub used_bytes: u64,
    pub reserved_bytes: u64,
    pub quota_bytes: u64,
}

impl StoreCapacity {
    pub fn available_bytes(&self) -> Result<u64, StoreError> {
        let committed = self
            .used_bytes
            .checked_add(self.reserved_bytes)
            .ok_or(StoreError::Overflow)?;
        core::cmp::min(self.capacity_bytes, self.quota_bytes)
            .checked_sub(committed)
            .ok_or(StoreError::Corrupt)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
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

/// Read-only surface shared by package management and read-only consumers.
/// `open` checks immutable metadata and content digest; it does not assert
/// publisher trust or model safety. The associated handle type is opaque to
/// callers and is constructed only by the concrete reader. Each read is
/// checked against the authenticated client context and current store epoch.
pub trait ArtifactReader {
    type Handle;

    fn open(
        &mut self,
        descriptor: &ArtifactDescriptor,
        expected_store_epoch: u64,
    ) -> Result<Self::Handle, StoreError>;

    fn read_at(
        &mut self,
        handle: &Self::Handle,
        descriptor: &ArtifactDescriptor,
        expected_store_epoch: u64,
        offset: u64,
        output: &mut [u8],
    ) -> Result<usize, StoreError>;

    fn capacity(&mut self) -> Result<StoreCapacity, StoreError>;
}

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

pub fn open_checked<R: ArtifactReader>(
    reader: &mut R,
    descriptor: &ArtifactDescriptor,
    expected_store_epoch: u64,
) -> Result<R::Handle, StoreError> {
    if expected_store_epoch == 0 || !valid_descriptor(descriptor) {
        return Err(StoreError::Invalid);
    }
    reader.open(descriptor, expected_store_epoch)
}

pub fn read_bounded<R: ArtifactReader>(
    reader: &mut R,
    handle: &R::Handle,
    descriptor: &ArtifactDescriptor,
    expected_store_epoch: u64,
    offset: u64,
    output: &mut [u8],
) -> Result<usize, StoreError> {
    if expected_store_epoch == 0 {
        return Err(StoreError::Stale);
    }
    if !valid_descriptor(descriptor) {
        return Err(StoreError::Invalid);
    }
    if output.is_empty() || output.len() > READ_CHUNK_MAX {
        return Err(StoreError::Invalid);
    }
    if offset == descriptor.length {
        return Err(StoreError::EndOfArtifact);
    }
    if offset > descriptor.length {
        return Err(StoreError::OutOfRange);
    }
    let remaining = descriptor
        .length
        .checked_sub(offset)
        .ok_or(StoreError::Overflow)?;
    let requested = core::cmp::min(output.len() as u64, remaining) as usize;
    offset
        .checked_add(requested as u64)
        .ok_or(StoreError::Overflow)?;
    let read = reader.read_at(
        handle,
        descriptor,
        expected_store_epoch,
        offset,
        &mut output[..requested],
    )?;
    if read == 0 {
        return Err(StoreError::Truncated);
    }
    if read > requested {
        return Err(StoreError::Protocol);
    }
    Ok(read)
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
        if !valid_descriptor(&descriptor) {
            return Err(StoreError::Invalid);
        }
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

    struct FakeHandle {
        descriptor: ArtifactDescriptor,
        epoch: u64,
        locator: u64,
    }

    struct FakeReader {
        descriptor: ArtifactDescriptor,
        epoch: u64,
        locator: u64,
        read_count: usize,
        read_error: Option<StoreError>,
    }

    impl ArtifactReader for FakeReader {
        type Handle = FakeHandle;

        fn open(
            &mut self,
            descriptor: &ArtifactDescriptor,
            expected_store_epoch: u64,
        ) -> Result<Self::Handle, StoreError> {
            if descriptor != &self.descriptor {
                return Err(StoreError::Stale);
            }
            if expected_store_epoch != self.epoch || self.locator == 0 {
                return Err(StoreError::Stale);
            }
            Ok(FakeHandle {
                descriptor: self.descriptor,
                epoch: self.epoch,
                locator: self.locator,
            })
        }

        fn read_at(
            &mut self,
            handle: &Self::Handle,
            descriptor: &ArtifactDescriptor,
            expected_store_epoch: u64,
            _offset: u64,
            output: &mut [u8],
        ) -> Result<usize, StoreError> {
            if handle.epoch != expected_store_epoch
                || handle.locator == 0
                || handle.descriptor != *descriptor
            {
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
        let mut reader = FakeReader {
            descriptor,
            epoch: 7,
            locator: 11,
            read_count: READ_CHUNK_MAX,
            read_error: None,
        };
        let handle = open_checked(&mut reader, &descriptor, 7).unwrap();
        let mut bytes = [0; READ_CHUNK_MAX];
        assert_eq!(
            read_bounded(&mut reader, &handle, &descriptor, 7, 0, &mut bytes).unwrap(),
            READ_CHUNK_MAX
        );
        assert_eq!(bytes[0], 0x5a);
        assert_eq!(
            read_bounded(&mut reader, &handle, &descriptor, 8, 0, &mut bytes),
            Err(StoreError::Stale)
        );
        assert_eq!(
            read_bounded(&mut reader, &handle, &descriptor, 7, 5000, &mut bytes),
            Err(StoreError::EndOfArtifact)
        );
        assert_eq!(
            read_bounded(&mut reader, &handle, &descriptor, 7, 5001, &mut bytes),
            Err(StoreError::OutOfRange)
        );
        assert_eq!(
            read_bounded(
                &mut reader,
                &handle,
                &descriptor,
                7,
                0,
                &mut [0; READ_CHUNK_MAX + 1]
            ),
            Err(StoreError::Invalid)
        );
        assert_eq!(reader.capacity().unwrap().available_bytes(), Ok(1536));
    }

    #[test]
    fn read_errors_are_explicit_and_never_look_like_eof_success() {
        let descriptor = descriptor(8);
        let mut reader = FakeReader {
            descriptor,
            epoch: 1,
            locator: 1,
            read_count: 0,
            read_error: None,
        };
        let handle = open_checked(&mut reader, &descriptor, 1).unwrap();
        let mut bytes = [0; 8];
        assert_eq!(
            read_bounded(&mut reader, &handle, &descriptor, 1, 0, &mut bytes),
            Err(StoreError::Truncated)
        );
        reader.read_count = 9;
        assert_eq!(
            read_bounded(&mut reader, &handle, &descriptor, 1, 0, &mut bytes),
            Err(StoreError::Protocol)
        );
        reader.read_error = Some(StoreError::Overflow);
        assert_eq!(
            read_bounded(&mut reader, &handle, &descriptor, 1, 0, &mut bytes),
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
