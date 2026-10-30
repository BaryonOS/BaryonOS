use artifact_store_core::{
    open_checked, read_bounded, ArtifactDescriptor, ArtifactRef, ArtifactStore, Error,
    StoreCapacity, READ_CHUNK_MAX,
};

struct TestHandle {
    epoch: u64,
}

struct Store {
    descriptor: ArtifactDescriptor,
    opened_descriptor: Option<ArtifactDescriptor>,
    read_result: Result<usize, Error>,
    read_calls: usize,
    last_read_len: usize,
    epoch: u64,
}

impl Store {
    fn new() -> Self {
        Self {
            descriptor: descriptor(5000),
            opened_descriptor: None,
            read_result: Ok(1),
            read_calls: 0,
            last_read_len: 0,
            epoch: 1,
        }
    }
}

impl ArtifactStore for Store {
    type Handle = TestHandle;

    fn open(
        &mut self,
        _expected: &ArtifactDescriptor,
    ) -> Result<(Self::Handle, ArtifactDescriptor), Error> {
        Ok((
            TestHandle { epoch: self.epoch },
            self.opened_descriptor.unwrap_or(self.descriptor),
        ))
    }

    fn read_at(
        &mut self,
        handle: &Self::Handle,
        _offset: u64,
        output: &mut [u8],
    ) -> Result<usize, Error> {
        self.read_calls += 1;
        self.last_read_len = output.len();
        if handle.epoch != self.epoch {
            return Err(Error::Stale);
        }
        let amount = self.read_result?;
        if amount <= output.len() {
            output[..amount].fill(0x5a);
        }
        Ok(amount)
    }

    fn capacity(&mut self) -> Result<StoreCapacity, Error> {
        Ok(StoreCapacity {
            capacity_bytes: 4096,
            used_bytes: 1000,
            reserved_bytes: 500,
            quota_bytes: 3000,
        })
    }
}

fn descriptor(length: u64) -> ArtifactDescriptor {
    ArtifactDescriptor {
        reference: ArtifactRef {
            artifact_id: [7; 32],
            immutable_generation: 4,
        },
        length,
        sha256: [0; 32],
    }
}

#[test]
fn descriptor_requires_nonzero_id_generation_and_length() {
    let mut invalid = descriptor(10);
    invalid.reference.artifact_id = [0; 32];
    assert_eq!(invalid.validate(), Err(Error::Invalid));

    invalid = descriptor(10);
    invalid.reference.immutable_generation = 0;
    assert_eq!(invalid.validate(), Err(Error::Invalid));

    assert_eq!(descriptor(0).validate(), Err(Error::Invalid));
}

#[test]
fn open_rejects_noncanonical_or_different_store_descriptor() {
    let mut store = Store::new();
    let mut expected = descriptor(5000);
    expected.length = 0;
    assert_eq!(
        open_checked(&mut store, &expected).err(),
        Some(Error::Invalid)
    );
    assert_eq!(store.read_calls, 0);

    expected = descriptor(5000);
    store.opened_descriptor = Some(descriptor(4999));
    assert_eq!(
        open_checked(&mut store, &expected).err(),
        Some(Error::Corrupt)
    );
    assert_eq!(store.read_calls, 0);
}

#[test]
fn bounded_read_accepts_positive_short_reads_and_clamps_at_eof() {
    let mut store = Store::new();
    let handle = open_checked(&mut store, &descriptor(5000)).unwrap();
    let mut buffer = [0; READ_CHUNK_MAX];

    assert_eq!(
        read_bounded(&mut store, &handle, 0, &mut buffer).unwrap(),
        1
    );
    assert_eq!(buffer[0], 0x5a);
    assert_eq!(store.last_read_len, READ_CHUNK_MAX);
    assert_eq!(
        read_bounded(&mut store, &handle, 4999, &mut buffer).unwrap(),
        1
    );
    assert_eq!(store.last_read_len, 1);
    assert_eq!(store.read_calls, 2);
}

#[test]
fn invalid_buffers_eof_and_out_of_range_never_reach_store() {
    let mut store = Store::new();
    let handle = open_checked(&mut store, &descriptor(5000)).unwrap();
    let mut buffer = [0; READ_CHUNK_MAX + 1];

    assert_eq!(
        read_bounded(&mut store, &handle, 0, &mut []),
        Err(Error::Invalid)
    );
    assert_eq!(
        read_bounded(&mut store, &handle, 0, &mut buffer),
        Err(Error::Invalid)
    );
    assert_eq!(
        read_bounded(&mut store, &handle, 5000, &mut buffer[..1]),
        Err(Error::EndOfArtifact)
    );
    assert_eq!(
        read_bounded(&mut store, &handle, 5001, &mut buffer[..1]),
        Err(Error::OutOfRange)
    );
    assert_eq!(store.read_calls, 0);
}

#[test]
fn zero_short_and_impossible_backend_reads_fail_closed() {
    let mut store = Store::new();
    let handle = open_checked(&mut store, &descriptor(5000)).unwrap();
    let mut buffer = [0; 8];

    store.read_result = Ok(0);
    assert_eq!(
        read_bounded(&mut store, &handle, 0, &mut buffer),
        Err(Error::Truncated)
    );
    store.read_result = Ok(9);
    assert_eq!(
        read_bounded(&mut store, &handle, 0, &mut buffer),
        Err(Error::Protocol)
    );
}

#[test]
fn store_stale_revocation_and_io_errors_remain_explicit() {
    let mut store = Store::new();
    let handle = open_checked(&mut store, &descriptor(5000)).unwrap();
    let mut buffer = [0; 8];
    store.epoch = 2;
    assert_eq!(
        read_bounded(&mut store, &handle, 0, &mut buffer),
        Err(Error::Stale)
    );
    store.epoch = 1;
    store.read_result = Err(Error::Unauthorized);
    assert_eq!(
        read_bounded(&mut store, &handle, 0, &mut buffer),
        Err(Error::Unauthorized)
    );
    store.read_result = Err(Error::Io);
    assert_eq!(
        read_bounded(&mut store, &handle, 0, &mut buffer),
        Err(Error::Io)
    );
}

#[test]
fn capacity_uses_checked_u64_arithmetic_and_provider_quota() {
    let mut capacity = StoreCapacity {
        capacity_bytes: 4096,
        used_bytes: 1000,
        reserved_bytes: 500,
        quota_bytes: 3000,
    };
    assert_eq!(capacity.available_bytes(), Ok(1500));

    capacity.reserved_bytes = u64::MAX;
    assert_eq!(capacity.available_bytes(), Err(Error::Overflow));

    capacity.used_bytes = 3001;
    capacity.reserved_bytes = 0;
    assert_eq!(capacity.available_bytes(), Err(Error::Corrupt));
}
