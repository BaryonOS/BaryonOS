use system_journal::{
    BuildIdentity, CrashRecord, CrashRecordError, DenialClass, EventCode, ExitClass, FailureClass,
    FaultClass, FaultContext, Journal, JournalError, JournalId, MetadataField, MetadataValue,
    Producer, RateLimit, Reader, ReaderPermit, RecordInput, ResourceKind, RestartOutcome,
    ServiceIdentity, Severity, Subsystem,
};

#[derive(Clone, Copy)]
struct ModelRecord {
    sequence: u64,
    monotonic_ticks: u64,
    producer: Producer,
    subsystem: Subsystem,
    event: EventCode,
}

fn assert_scoped_records<const N: usize>(
    journal: &Journal<N>,
    reader: &ReaderPermit,
    expected: &[ModelRecord],
    subsystem: Subsystem,
) {
    let visible = expected
        .iter()
        .filter(|record| record.subsystem == subsystem)
        .collect::<std::vec::Vec<_>>();
    assert_eq!(journal.len(Some(reader)), Ok(visible.len()));
    for (index, expected) in visible.into_iter().enumerate() {
        let actual = journal.read(Some(reader), index).unwrap();
        assert_eq!(actual.sequence, expected.sequence);
        assert_eq!(actual.monotonic_ticks, expected.monotonic_ticks);
        assert_eq!(actual.producer, expected.producer);
        assert_eq!(actual.subsystem, expected.subsystem);
        assert_eq!(actual.event, expected.event);
    }
}

fn journal<const N: usize>() -> (
    Journal<N>,
    system_journal::JournalAdmin,
    system_journal::ProducerPermit,
    system_journal::ReaderPermit,
) {
    let (journal, admin) = Journal::new(
        JournalId::new(1).unwrap(),
        RateLimit {
            window_ticks: 10,
            max_low_severity_events: 2,
        },
    )
    .unwrap();
    let producer = admin.producer(Producer::CapabilityEngine);
    let reader = admin
        .reader(
            Reader::DiagnosticsApp,
            &[Subsystem::Capability, Subsystem::Runtime],
        )
        .unwrap();
    (journal, admin, producer, reader)
}

fn event(journal: &mut Journal<3>, producer: &system_journal::ProducerPermit, now: u64) -> u64 {
    journal
        .record(
            producer,
            RecordInput {
                monotonic_ticks: now,
                wall_ticks: None,
                severity: Severity::Warning,
                subsystem: Subsystem::Process,
                event: EventCode::ProcessExited,
                correlation_id: None,
                metadata: &[MetadataField::exit_class(ExitClass::Normal)],
            },
        )
        .unwrap()
}

#[test]
fn consumer_reads_only_with_a_journal_scoped_permit() {
    let (mut journal, _admin, producer, reader) = journal::<2>();
    let correlation = journal.new_correlation().unwrap();
    journal
        .record(
            &producer,
            RecordInput {
                monotonic_ticks: 10,
                wall_ticks: Some(20),
                severity: Severity::Warning,
                subsystem: Subsystem::Capability,
                event: EventCode::CapabilityDenied,
                correlation_id: Some(correlation),
                metadata: &[MetadataField::denial_class(DenialClass::MissingRight)],
            },
        )
        .unwrap();

    assert!(matches!(
        journal.read(None, 0),
        Err(JournalError::Unauthorized)
    ));
    let (other, other_admin) = Journal::<2>::new(
        JournalId::new(2).unwrap(),
        RateLimit {
            window_ticks: 10,
            max_low_severity_events: 2,
        },
    )
    .unwrap();
    let foreign_reader = other_admin
        .reader(Reader::DiagnosticsApp, &[Subsystem::Capability])
        .unwrap();
    assert!(matches!(
        journal.read(Some(&foreign_reader), 0),
        Err(JournalError::WrongJournal)
    ));
    assert_eq!(other.len(Some(&foreign_reader)), Ok(0));

    let record = journal.read(Some(&reader), 0).unwrap();
    assert_eq!(record.producer, Producer::CapabilityEngine);
    assert_eq!(record.correlation_id, Some(correlation));
    assert_eq!(record.monotonic_ticks, 10);
    assert_eq!(record.wall_ticks, Some(20));
    assert_eq!(
        record.metadata().next(),
        Some(MetadataValue::DenialClass(DenialClass::MissingRight))
    );
    assert_eq!(record.metadata().count(), 1);
    assert!(matches!(
        journal.read(Some(&reader), 1),
        Err(JournalError::RecordOutOfRange)
    ));
    let process_only = _admin
        .reader(Reader::DiagnosticsApp, &[Subsystem::Process])
        .unwrap();
    assert_eq!(journal.len(Some(&process_only)), Ok(0));
    assert!(matches!(
        journal.read(Some(&process_only), 0),
        Err(JournalError::RecordOutOfRange)
    ));
}

#[test]
fn exact_capacity_does_not_overwrite_and_capacity_plus_one_keeps_newest() {
    let (mut journal, admin, _producer, _reader) = journal::<3>();
    let producer = admin.producer(Producer::Supervisor);
    let reader = admin
        .reader(Reader::DiagnosticsApp, &[Subsystem::Process])
        .unwrap();
    for now in 0..3 {
        event(&mut journal, &producer, now);
    }
    assert_eq!(journal.len(Some(&reader)), Ok(3));
    assert_eq!(journal.stats(&admin).unwrap().overwritten, 0);

    event(&mut journal, &producer, 3);
    let records = (0..journal.len(Some(&reader)).unwrap())
        .map(|index| journal.read(Some(&reader), index).unwrap().sequence)
        .collect::<std::vec::Vec<_>>();
    assert_eq!(records, [2, 3, 4]);
    assert_eq!(
        journal.stats(&admin).unwrap(),
        system_journal::JournalStats {
            written: 4,
            overwritten: 1,
            rate_limited: 0,
            rejected: 0,
        }
    );
}

#[test]
fn oversized_and_wrong_schema_metadata_are_rejected_without_recording() {
    let (mut journal, admin, producer, _reader) = journal::<4>();
    let too_many = [
        MetadataField::count(1),
        MetadataField::duration_ticks(2),
        MetadataField::restart_attempt(1),
        MetadataField::resource_kind(ResourceKind::Endpoint),
        MetadataField::denial_class(DenialClass::Revoked),
    ];
    assert_eq!(
        journal.record(
            &producer,
            RecordInput {
                monotonic_ticks: 1,
                wall_ticks: None,
                severity: Severity::Warning,
                subsystem: Subsystem::Capability,
                event: EventCode::CapabilityDenied,
                correlation_id: None,
                metadata: &too_many,
            },
        ),
        Err(JournalError::TooManyMetadataFields)
    );
    assert_eq!(
        journal.record(
            &producer,
            RecordInput {
                monotonic_ticks: 2,
                wall_ticks: None,
                severity: Severity::Warning,
                subsystem: Subsystem::Capability,
                event: EventCode::CapabilityDenied,
                correlation_id: None,
                metadata: &[MetadataField::resource_kind(ResourceKind::Endpoint)],
            },
        ),
        Err(JournalError::MetadataNotAllowed)
    );
    assert_eq!(
        journal.record(
            &producer,
            RecordInput {
                monotonic_ticks: 3,
                wall_ticks: None,
                severity: Severity::Warning,
                subsystem: Subsystem::Capability,
                event: EventCode::CapabilityDenied,
                correlation_id: None,
                metadata: &[
                    MetadataField::denial_class(DenialClass::Revoked),
                    MetadataField::denial_class(DenialClass::MissingRight),
                ],
            },
        ),
        Err(JournalError::DuplicateMetadataField)
    );
    assert_eq!(journal.stats(&admin).unwrap().rejected, 3);
}

#[test]
fn low_severity_rate_limit_is_per_producer_and_window() {
    let (mut journal, admin, _producer, reader) = journal::<8>();
    let producer = admin.producer(Producer::RuntimeService);
    for now in [1, 2] {
        journal
            .record(
                &producer,
                RecordInput {
                    monotonic_ticks: now,
                    wall_ticks: None,
                    severity: Severity::Info,
                    subsystem: Subsystem::Runtime,
                    event: EventCode::ServiceStarted,
                    correlation_id: None,
                    metadata: &[],
                },
            )
            .unwrap();
    }
    assert_eq!(
        journal.record(
            &producer,
            RecordInput {
                monotonic_ticks: 3,
                wall_ticks: None,
                severity: Severity::Info,
                subsystem: Subsystem::Runtime,
                event: EventCode::ServiceStarted,
                correlation_id: None,
                metadata: &[],
            },
        ),
        Err(JournalError::RateLimited)
    );
    let other_producer = admin.producer(Producer::Supervisor);
    assert_eq!(
        journal.record(
            &other_producer,
            RecordInput {
                monotonic_ticks: 4,
                wall_ticks: None,
                severity: Severity::Info,
                subsystem: Subsystem::Runtime,
                event: EventCode::ServiceStarted,
                correlation_id: None,
                metadata: &[],
            },
        ),
        Ok(3)
    );
    assert_eq!(
        journal.record(
            &producer,
            RecordInput {
                monotonic_ticks: 12,
                wall_ticks: None,
                severity: Severity::Info,
                subsystem: Subsystem::Runtime,
                event: EventCode::ServiceStarted,
                correlation_id: None,
                metadata: &[],
            },
        ),
        Ok(4)
    );
    assert_eq!(journal.stats(&admin).unwrap().rate_limited, 1);
    assert_eq!(journal.len(Some(&reader)), Ok(4));
}

#[test]
fn correlation_ids_are_bound_to_their_journal() {
    let (mut journal, _admin, _producer, journal_reader) = journal::<3>();
    let correlation = journal.new_correlation().unwrap();
    let (mut other, other_admin) = Journal::<3>::new(
        JournalId::new(2).unwrap(),
        RateLimit {
            window_ticks: 10,
            max_low_severity_events: 2,
        },
    )
    .unwrap();
    let other_producer = other_admin.producer(Producer::Supervisor);
    let other_reader = other_admin
        .reader(Reader::DiagnosticsApp, &[Subsystem::Process])
        .unwrap();
    assert_eq!(
        other.record(
            &other_producer,
            RecordInput {
                monotonic_ticks: 1,
                wall_ticks: None,
                severity: Severity::Warning,
                subsystem: Subsystem::Runtime,
                event: EventCode::ProcessExited,
                correlation_id: Some(correlation),
                metadata: &[MetadataField::exit_class(ExitClass::Faulted)],
            },
        ),
        Err(JournalError::WrongJournal)
    );
    assert_eq!(other.len(Some(&other_reader)), Ok(0));
    assert_eq!(journal.len(Some(&journal_reader)), Ok(0));
}

#[test]
fn producer_scope_and_monotonic_time_are_checked() {
    let (mut journal, admin, producer, _reader) = journal::<4>();
    assert_eq!(
        journal.record(
            &producer,
            RecordInput {
                monotonic_ticks: 1,
                wall_ticks: None,
                severity: Severity::Critical,
                subsystem: Subsystem::Security,
                event: EventCode::IntegrityFailure,
                correlation_id: None,
                metadata: &[MetadataField::failure_class(
                    FailureClass::SignatureRejected,
                )],
            },
        ),
        Err(JournalError::ProducerSubsystemMismatch)
    );
    journal
        .record(
            &producer,
            RecordInput {
                monotonic_ticks: 5,
                wall_ticks: None,
                severity: Severity::Warning,
                subsystem: Subsystem::Capability,
                event: EventCode::CapabilityDenied,
                correlation_id: None,
                metadata: &[MetadataField::denial_class(DenialClass::Revoked)],
            },
        )
        .unwrap();
    assert_eq!(
        journal.record(
            &producer,
            RecordInput {
                monotonic_ticks: 4,
                wall_ticks: None,
                severity: Severity::Warning,
                subsystem: Subsystem::Capability,
                event: EventCode::CapabilityDenied,
                correlation_id: None,
                metadata: &[MetadataField::denial_class(DenialClass::Revoked)],
            },
        ),
        Err(JournalError::TimestampRegressed)
    );
    assert_eq!(journal.stats(&admin).unwrap().rejected, 2);
}

#[test]
fn crash_diagnostics_keep_bounded_identity_and_restart_context() {
    let (mut journal, admin, _producer, _reader) = journal::<4>();
    let producer = admin.producer(Producer::Supervisor);
    let reader = admin
        .reader(Reader::DiagnosticsApp, &[Subsystem::Process])
        .unwrap();
    let correlation = journal.new_correlation().unwrap();
    let crash = CrashRecord::new(
        77,
        Some(ServiceIdentity::BlockStorage),
        BuildIdentity::new([0xab; 32]),
        FaultClass::PageFault,
        FaultContext::ExceptionVector(14),
        RestartOutcome::Restarted(2),
    )
    .unwrap();
    assert_eq!(
        CrashRecord::new(
            0,
            None,
            BuildIdentity::new([0; 32]),
            FaultClass::ProcessExit,
            FaultContext::None,
            RestartOutcome::NotAttempted,
        ),
        Err(CrashRecordError::InvalidProcessIdentity)
    );
    assert_eq!(
        CrashRecord::new(
            77,
            None,
            BuildIdentity::new([0; 32]),
            FaultClass::ProcessExit,
            FaultContext::None,
            RestartOutcome::Restarted(0),
        ),
        Err(CrashRecordError::InvalidRestartGeneration)
    );
    assert_eq!(
        journal.record_crash(
            &producer,
            RecordInput {
                monotonic_ticks: 100,
                wall_ticks: None,
                severity: Severity::Warning,
                subsystem: Subsystem::Process,
                event: EventCode::ProcessExited,
                correlation_id: Some(correlation),
                metadata: &[],
            },
            crash,
        ),
        Err(JournalError::InvalidCrashRecord)
    );

    journal
        .record_crash(
            &producer,
            RecordInput {
                monotonic_ticks: 100,
                wall_ticks: None,
                severity: Severity::Warning,
                subsystem: Subsystem::Process,
                event: EventCode::ProcessFaulted,
                correlation_id: Some(correlation),
                metadata: &[MetadataField::exit_class(ExitClass::Faulted)],
            },
            crash,
        )
        .unwrap();
    let record = journal.read(Some(&reader), 0).unwrap();
    let crash = record.crash.unwrap();
    assert_eq!(record.monotonic_ticks, 100);
    assert_eq!(record.correlation_id, Some(correlation));
    assert_eq!(crash.process().get(), 77);
    assert_eq!(crash.service(), Some(ServiceIdentity::BlockStorage));
    assert_eq!(crash.executable().digest(), &[0xab; 32]);
    assert_eq!(crash.fault(), FaultClass::PageFault);
    assert_eq!(crash.context(), FaultContext::ExceptionVector(14));
    assert_eq!(crash.restart(), RestartOutcome::Restarted(2));
}

#[test]
fn schema_boundary_redacts_free_form_data_and_enforces_minimum_severity() {
    let (mut journal, admin, _producer, _reader) = journal::<4>();
    let producer = admin.producer(Producer::SecurityService);
    let reader = admin
        .reader(Reader::DiagnosticsApp, &[Subsystem::Security])
        .unwrap();
    assert_eq!(
        journal.record(
            &producer,
            RecordInput {
                monotonic_ticks: 1,
                wall_ticks: None,
                severity: Severity::Debug,
                subsystem: Subsystem::Security,
                event: EventCode::IntegrityFailure,
                correlation_id: None,
                metadata: &[MetadataField::failure_class(
                    FailureClass::SignatureRejected,
                )],
            },
        ),
        Err(JournalError::SeverityTooLow)
    );
    assert_eq!(
        journal.record(
            &producer,
            RecordInput {
                monotonic_ticks: 2,
                wall_ticks: None,
                severity: Severity::Critical,
                subsystem: Subsystem::Security,
                event: EventCode::IntegrityFailure,
                correlation_id: None,
                metadata: &[MetadataField::failure_class(
                    FailureClass::SignatureRejected,
                )],
            },
        ),
        Ok(1)
    );
    let record = journal.read(Some(&reader), 0).unwrap();
    assert_eq!(
        record.metadata().next(),
        Some(MetadataValue::FailureClass(FailureClass::SignatureRejected))
    );
    assert_eq!(record.metadata().count(), 1);
    assert_eq!(journal.stats(&admin).unwrap().rejected, 1);
}

#[test]
fn deterministic_state_sequence_preserves_ring_order_and_accounting() {
    const CAPACITY: usize = 7;
    let (mut journal, admin) = Journal::<CAPACITY>::new(
        JournalId::new(42).unwrap(),
        RateLimit {
            window_ticks: 10,
            max_low_severity_events: 2,
        },
    )
    .unwrap();
    let supervisor = admin.producer(Producer::Supervisor);
    let capability = admin.producer(Producer::CapabilityEngine);
    let process_reader = admin
        .reader(Reader::DiagnosticsApp, &[Subsystem::Process])
        .unwrap();
    let capability_reader = admin
        .reader(Reader::DiagnosticsApp, &[Subsystem::Capability])
        .unwrap();
    let process_metadata = [MetadataField::exit_class(ExitClass::Normal)];
    let capability_metadata = [MetadataField::denial_class(DenialClass::MissingRight)];
    let mut model = std::vec::Vec::new();
    let mut written = 0u64;
    let mut rejected = 0u64;

    for step in 0..257u64 {
        let now = 100 + step * 2;
        let (permit, producer, subsystem, event, metadata) = if step % 2 == 0 {
            (
                &supervisor,
                Producer::Supervisor,
                Subsystem::Process,
                EventCode::ProcessExited,
                &process_metadata[..],
            )
        } else {
            (
                &capability,
                Producer::CapabilityEngine,
                Subsystem::Capability,
                EventCode::CapabilityDenied,
                &capability_metadata[..],
            )
        };

        if step > 0 && step % 9 == 4 {
            assert_eq!(
                journal.record(
                    permit,
                    RecordInput {
                        monotonic_ticks: now - 3,
                        wall_ticks: None,
                        severity: Severity::Warning,
                        subsystem,
                        event,
                        correlation_id: None,
                        metadata,
                    },
                ),
                Err(JournalError::TimestampRegressed)
            );
            rejected += 1;
        }

        written += 1;
        assert_eq!(
            journal.record(
                permit,
                RecordInput {
                    monotonic_ticks: now,
                    wall_ticks: None,
                    severity: Severity::Warning,
                    subsystem,
                    event,
                    correlation_id: None,
                    metadata,
                },
            ),
            Ok(written)
        );
        model.push(ModelRecord {
            sequence: written,
            monotonic_ticks: now,
            producer,
            subsystem,
            event,
        });

        let mut retained = model
            .iter()
            .rev()
            .take(CAPACITY)
            .copied()
            .collect::<std::vec::Vec<_>>();
        retained.reverse();
        assert_scoped_records(&journal, &process_reader, &retained, Subsystem::Process);
        assert_scoped_records(
            &journal,
            &capability_reader,
            &retained,
            Subsystem::Capability,
        );
        assert_eq!(
            journal.stats(&admin).unwrap(),
            system_journal::JournalStats {
                written,
                overwritten: written.saturating_sub(CAPACITY as u64),
                rate_limited: 0,
                rejected,
            }
        );
    }
}
