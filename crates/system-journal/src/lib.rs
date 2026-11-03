#![no_std]

pub const MAX_METADATA_FIELDS: usize = 4;
const PRODUCER_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalId(u16);

impl JournalId {
    pub const fn new(value: u16) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Severity {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Subsystem {
    Kernel = 0,
    Process = 1,
    Capability = 2,
    Storage = 3,
    Network = 4,
    Runtime = 5,
    Security = 6,
    Update = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Producer {
    Kernel = 0,
    Supervisor = 1,
    CapabilityEngine = 2,
    StorageService = 3,
    NetworkService = 4,
    RuntimeService = 5,
    SecurityService = 6,
    UpdateService = 7,
}

impl Producer {
    fn permits(self, subsystem: Subsystem) -> bool {
        match self {
            Self::Kernel => matches!(subsystem, Subsystem::Kernel | Subsystem::Process),
            Self::Supervisor => matches!(subsystem, Subsystem::Process | Subsystem::Runtime),
            Self::CapabilityEngine => subsystem == Subsystem::Capability,
            Self::StorageService => subsystem == Subsystem::Storage,
            Self::NetworkService => subsystem == Subsystem::Network,
            Self::RuntimeService => subsystem == Subsystem::Runtime,
            Self::SecurityService => subsystem == Subsystem::Security,
            Self::UpdateService => subsystem == Subsystem::Update,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reader {
    DiagnosticsApp,
    SystemAdministrator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum EventCode {
    ProcessStarted,
    ProcessExited,
    ProcessFaulted,
    ServiceStarted,
    ServiceStopped,
    ServiceRestarted,
    ResourceLimitReached,
    CapabilityDenied,
    IntegrityFailure,
}

impl EventCode {
    fn minimum_severity(self) -> Severity {
        match self {
            Self::ProcessStarted | Self::ServiceStarted | Self::ServiceStopped => Severity::Info,
            Self::ProcessExited | Self::ServiceRestarted => Severity::Warning,
            Self::ProcessFaulted | Self::ResourceLimitReached | Self::CapabilityDenied => {
                Severity::Warning
            }
            Self::IntegrityFailure => Severity::Critical,
        }
    }

    fn permits(self, key: MetadataKey) -> bool {
        match self {
            Self::ProcessStarted | Self::ServiceStarted | Self::ServiceStopped => {
                matches!(key, MetadataKey::DurationTicks)
            }
            Self::ProcessExited | Self::ProcessFaulted => {
                matches!(key, MetadataKey::ExitClass)
            }
            Self::ServiceRestarted => {
                matches!(
                    key,
                    MetadataKey::RestartAttempt | MetadataKey::DurationTicks
                )
            }
            Self::ResourceLimitReached => {
                matches!(key, MetadataKey::ResourceKind | MetadataKey::Count)
            }
            Self::CapabilityDenied => matches!(key, MetadataKey::DenialClass),
            Self::IntegrityFailure => matches!(key, MetadataKey::FailureClass),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitClass {
    Normal,
    Killed,
    Faulted,
    TimedOut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ResourceKind {
    ProcessSlot,
    HandleSlot,
    Endpoint,
    QueueCell,
    MemoryPage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DenialClass {
    MissingRight,
    Revoked,
    StaleIdentity,
    InvalidInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FailureClass {
    InvalidImage,
    InvalidRecord,
    SignatureRejected,
    VersionRejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataKey {
    Count,
    DurationTicks,
    ExitClass,
    RestartAttempt,
    ResourceKind,
    DenialClass,
    FailureClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataValue {
    Count(u32),
    DurationTicks(u64),
    ExitClass(ExitClass),
    RestartAttempt(u8),
    ResourceKind(ResourceKind),
    DenialClass(DenialClass),
    FailureClass(FailureClass),
}

impl MetadataValue {
    fn key(self) -> MetadataKey {
        match self {
            Self::Count(_) => MetadataKey::Count,
            Self::DurationTicks(_) => MetadataKey::DurationTicks,
            Self::ExitClass(_) => MetadataKey::ExitClass,
            Self::RestartAttempt(_) => MetadataKey::RestartAttempt,
            Self::ResourceKind(_) => MetadataKey::ResourceKind,
            Self::DenialClass(_) => MetadataKey::DenialClass,
            Self::FailureClass(_) => MetadataKey::FailureClass,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetadataField {
    value: MetadataValue,
}

impl MetadataField {
    pub const fn count(value: u32) -> Self {
        Self {
            value: MetadataValue::Count(value),
        }
    }

    pub const fn duration_ticks(value: u64) -> Self {
        Self {
            value: MetadataValue::DurationTicks(value),
        }
    }

    pub const fn exit_class(value: ExitClass) -> Self {
        Self {
            value: MetadataValue::ExitClass(value),
        }
    }

    pub const fn restart_attempt(value: u8) -> Self {
        Self {
            value: MetadataValue::RestartAttempt(value),
        }
    }

    pub const fn resource_kind(value: ResourceKind) -> Self {
        Self {
            value: MetadataValue::ResourceKind(value),
        }
    }

    pub const fn denial_class(value: DenialClass) -> Self {
        Self {
            value: MetadataValue::DenialClass(value),
        }
    }

    pub const fn failure_class(value: FailureClass) -> Self {
        Self {
            value: MetadataValue::FailureClass(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RateLimit {
    pub window_ticks: u64,
    pub max_low_severity_events: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JournalStats {
    pub written: u64,
    pub overwritten: u64,
    pub rate_limited: u64,
    pub rejected: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalError {
    InvalidCapacity,
    InvalidRateLimit,
    Unauthorized,
    WrongJournal,
    TooManyMetadataFields,
    DuplicateMetadataField,
    MetadataNotAllowed,
    ProducerSubsystemMismatch,
    SeverityTooLow,
    TimestampRegressed,
    RateLimited,
    CorrelationExhausted,
    SequenceExhausted,
    RecordOutOfRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CorrelationId {
    journal_id: JournalId,
    value: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalRecord {
    pub sequence: u64,
    pub monotonic_ticks: u64,
    pub wall_ticks: Option<u64>,
    pub severity: Severity,
    pub subsystem: Subsystem,
    pub producer: Producer,
    pub event: EventCode,
    pub correlation_id: Option<CorrelationId>,
    metadata: [Option<MetadataValue>; MAX_METADATA_FIELDS],
    metadata_len: u8,
}

pub struct RecordInput<'a> {
    pub monotonic_ticks: u64,
    pub wall_ticks: Option<u64>,
    pub severity: Severity,
    pub subsystem: Subsystem,
    pub event: EventCode,
    pub correlation_id: Option<CorrelationId>,
    pub metadata: &'a [MetadataField],
}

impl JournalRecord {
    pub fn metadata(&self) -> impl Iterator<Item = MetadataValue> + '_ {
        self.metadata[..usize::from(self.metadata_len)]
            .iter()
            .filter_map(|field| *field)
    }
}

impl CorrelationId {
    pub const fn get(self) -> u64 {
        self.value
    }
}

pub struct JournalAdmin {
    journal_id: JournalId,
}

impl JournalAdmin {
    pub fn producer(&self, producer: Producer) -> ProducerPermit {
        ProducerPermit {
            journal_id: self.journal_id,
            producer,
        }
    }

    pub fn reader(
        &self,
        reader: Reader,
        scopes: &[Subsystem],
    ) -> Result<ReaderPermit, JournalError> {
        if scopes.is_empty() || scopes.len() > 8 {
            return Err(JournalError::Unauthorized);
        }
        let mut allowed_subsystems = 0u8;
        for subsystem in scopes {
            allowed_subsystems |= 1 << (*subsystem as u8);
        }
        Ok(ReaderPermit {
            journal_id: self.journal_id,
            _reader: reader,
            allowed_subsystems,
        })
    }
}

pub struct ProducerPermit {
    journal_id: JournalId,
    producer: Producer,
}

pub struct ReaderPermit {
    journal_id: JournalId,
    _reader: Reader,
    allowed_subsystems: u8,
}

#[derive(Clone, Copy)]
struct RateBucket {
    window_start: u64,
    admitted: u16,
    initialized: bool,
}

impl RateBucket {
    const EMPTY: Self = Self {
        window_start: 0,
        admitted: 0,
        initialized: false,
    };
}

pub struct Journal<const CAPACITY: usize> {
    journal_id: JournalId,
    records: [Option<JournalRecord>; CAPACITY],
    oldest: usize,
    len: usize,
    next_sequence: Option<u64>,
    next_correlation: Option<u64>,
    last_monotonic: Option<u64>,
    rate_limit: RateLimit,
    rate_buckets: [RateBucket; PRODUCER_COUNT],
    stats: JournalStats,
}

impl<const CAPACITY: usize> Journal<CAPACITY> {
    pub fn new(
        journal_id: JournalId,
        rate_limit: RateLimit,
    ) -> Result<(Self, JournalAdmin), JournalError> {
        if CAPACITY == 0 {
            return Err(JournalError::InvalidCapacity);
        }
        if rate_limit.window_ticks == 0 || rate_limit.max_low_severity_events == 0 {
            return Err(JournalError::InvalidRateLimit);
        }

        Ok((
            Self {
                journal_id,
                records: [None; CAPACITY],
                oldest: 0,
                len: 0,
                next_sequence: Some(1),
                next_correlation: Some(1),
                last_monotonic: None,
                rate_limit,
                rate_buckets: [RateBucket::EMPTY; PRODUCER_COUNT],
                stats: JournalStats::default(),
            },
            JournalAdmin { journal_id },
        ))
    }

    pub fn new_correlation(&mut self) -> Result<CorrelationId, JournalError> {
        let value = self
            .next_correlation
            .ok_or(JournalError::CorrelationExhausted)?;
        self.next_correlation = value.checked_add(1);
        Ok(CorrelationId {
            journal_id: self.journal_id,
            value,
        })
    }

    pub fn record(
        &mut self,
        permit: &ProducerPermit,
        input: RecordInput<'_>,
    ) -> Result<u64, JournalError> {
        if permit.journal_id != self.journal_id
            || input
                .correlation_id
                .is_some_and(|id| id.journal_id != self.journal_id)
        {
            self.reject();
            return Err(JournalError::WrongJournal);
        }
        if input.metadata.len() > MAX_METADATA_FIELDS {
            self.reject();
            return Err(JournalError::TooManyMetadataFields);
        }
        if !permit.producer.permits(input.subsystem) {
            self.reject();
            return Err(JournalError::ProducerSubsystemMismatch);
        }
        if input.severity < input.event.minimum_severity() {
            self.reject();
            return Err(JournalError::SeverityTooLow);
        }
        if self
            .last_monotonic
            .is_some_and(|last| input.monotonic_ticks < last)
        {
            self.reject();
            return Err(JournalError::TimestampRegressed);
        }

        let mut stored_metadata = [None; MAX_METADATA_FIELDS];
        for (index, field) in input.metadata.iter().enumerate() {
            let value = field.value;
            let key = value.key();
            if !input.event.permits(key) {
                self.reject();
                return Err(JournalError::MetadataNotAllowed);
            }
            if input.metadata[..index]
                .iter()
                .any(|prior| prior.value.key() == key)
            {
                self.reject();
                return Err(JournalError::DuplicateMetadataField);
            }
            stored_metadata[index] = Some(value);
        }

        if input.severity <= Severity::Info
            && self.rate_limited(permit.producer, input.monotonic_ticks)
        {
            self.stats.rate_limited = self.stats.rate_limited.saturating_add(1);
            self.last_monotonic = Some(input.monotonic_ticks);
            return Err(JournalError::RateLimited);
        }

        let sequence = self.next_sequence.ok_or(JournalError::SequenceExhausted)?;
        self.next_sequence = sequence.checked_add(1);
        let record = JournalRecord {
            sequence,
            monotonic_ticks: input.monotonic_ticks,
            wall_ticks: input.wall_ticks,
            severity: input.severity,
            subsystem: input.subsystem,
            producer: permit.producer,
            event: input.event,
            correlation_id: input.correlation_id,
            metadata: stored_metadata,
            metadata_len: input.metadata.len() as u8,
        };
        self.append(record);
        self.last_monotonic = Some(input.monotonic_ticks);
        self.stats.written = self.stats.written.saturating_add(1);
        Ok(sequence)
    }

    pub fn read(
        &self,
        permit: Option<&ReaderPermit>,
        scoped_index: usize,
    ) -> Result<JournalRecord, JournalError> {
        let permit = permit.ok_or(JournalError::Unauthorized)?;
        if permit.journal_id != self.journal_id {
            return Err(JournalError::WrongJournal);
        }
        let mut visible_index = 0;
        for offset in 0..self.len {
            let slot = (self.oldest + offset) % CAPACITY;
            let record = self.records[slot].ok_or(JournalError::SequenceExhausted)?;
            if permit.allowed_subsystems & (1 << (record.subsystem as u8)) != 0 {
                if visible_index == scoped_index {
                    return Ok(record);
                }
                visible_index += 1;
            }
        }
        Err(JournalError::RecordOutOfRange)
    }

    pub fn len(&self, permit: Option<&ReaderPermit>) -> Result<usize, JournalError> {
        let permit = permit.ok_or(JournalError::Unauthorized)?;
        if permit.journal_id != self.journal_id {
            return Err(JournalError::WrongJournal);
        }
        Ok(self
            .records
            .iter()
            .flatten()
            .filter(|record| permit.allowed_subsystems & (1 << (record.subsystem as u8)) != 0)
            .count())
    }

    pub fn stats(&self, admin: &JournalAdmin) -> Result<JournalStats, JournalError> {
        if admin.journal_id != self.journal_id {
            return Err(JournalError::WrongJournal);
        }
        Ok(self.stats)
    }

    fn rate_limited(&mut self, producer: Producer, now: u64) -> bool {
        let bucket = &mut self.rate_buckets[producer as usize];
        if !bucket.initialized
            || now.saturating_sub(bucket.window_start) >= self.rate_limit.window_ticks
        {
            bucket.window_start = now;
            bucket.admitted = 0;
            bucket.initialized = true;
        }
        if bucket.admitted >= self.rate_limit.max_low_severity_events {
            return true;
        }
        bucket.admitted += 1;
        false
    }

    fn append(&mut self, record: JournalRecord) {
        if self.len < CAPACITY {
            let slot = (self.oldest + self.len) % CAPACITY;
            self.records[slot] = Some(record);
            self.len += 1;
        } else {
            self.records[self.oldest] = Some(record);
            self.oldest = (self.oldest + 1) % CAPACITY;
            self.stats.overwritten = self.stats.overwritten.saturating_add(1);
        }
    }

    fn reject(&mut self) {
        self.stats.rejected = self.stats.rejected.saturating_add(1);
    }
}
