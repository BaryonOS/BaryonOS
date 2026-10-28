//! Host checks of the client against the real catalog state/wire/recovery core.
//! The fixture commits in memory; it does not simulate native WRITE/FLUSH proof.
use catalog_core::{
    client::{
        Client, Error, Failure, Operation, ProtocolError, Reply, Request, TransportFailure,
        MESSAGE_SIZE,
    },
    journal::{self, Recovery, State},
    wire::{self, Status},
};

const EPOCH: u64 = 19;
const SERVICE: u64 = 7;

fn client(domain: usize) -> Client {
    Client::new(EPOCH, SERVICE, domain).unwrap()
}

fn payload(request: &Request) -> [u8; wire::SIZE] {
    request.message()[64..64 + wire::SIZE].try_into().unwrap()
}

fn envelope(payload: &[u8; wire::SIZE]) -> [u8; MESSAGE_SIZE] {
    let mut message = [0; MESSAGE_SIZE];
    wire::put64(&mut message, 0, SERVICE);
    wire::put64(&mut message, 8, wire::TAG);
    wire::put32(&mut message, 16, wire::SIZE as u32);
    message[64..64 + wire::SIZE].copy_from_slice(payload);
    message
}

fn status_reply(request: &Request, status: Status) -> [u8; MESSAGE_SIZE] {
    let h = wire::header(&payload(request));
    envelope(&wire::reply(h, EPOCH, 0, status, None, None))
}

fn put<'a>(value: &'a [u8], revision: u64) -> Operation<'a> {
    Operation::Put {
        object: 0,
        expected_revision: revision,
        name: b"Notes",
        value,
    }
}

struct CoreService {
    state: State,
    records: [[u8; journal::SECTOR]; journal::SLOTS],
}
impl CoreService {
    fn new() -> Self {
        Self {
            state: State::new(),
            records: [[0; journal::SECTOR]; journal::SLOTS],
        }
    }

    fn exchange(&mut self, client: &mut Client, domain: usize, operation: Operation<'_>) -> Reply {
        let request = client.begin(operation).unwrap();
        let response = self.respond(&request, domain);
        client.receive(&response).unwrap()
    }

    fn respond(&mut self, request: &Request, domain: usize) -> [u8; MESSAGE_SIZE] {
        let bytes = payload(request);
        let h = wire::validate_request(&bytes, EPOCH, domain).unwrap();
        let response = match h.operation {
            wire::LIST => wire::catalog_reply(
                h,
                EPOCH,
                Status::Ok,
                Some(&self.state.catalog(domain).unwrap()),
            ),
            wire::GET => {
                let object = self.state.get(h.object).unwrap();
                wire::reply(
                    h,
                    EPOCH,
                    object.revision,
                    if object.present {
                        Status::Ok
                    } else {
                        Status::NotFound
                    },
                    Some(&object.name[..object.name_length]),
                    Some(&object.value[..object.length]),
                )
            }
            wire::PUT | wire::DELETE | wire::RENAME => {
                let result = self.state.prepare(
                    h.object,
                    h.revision,
                    h.request_id,
                    h.operation,
                    &bytes[192..192 + h.name_length as usize],
                    &bytes[64..64 + h.length as usize],
                );
                match result {
                    Ok(prepared) => {
                        self.records[self.state.sequence() as usize] = *prepared.record();
                        self.state.commit(prepared).unwrap();
                        wire::reply(h, EPOCH, h.revision + 1, Status::Ok, None, None)
                    }
                    Err(error) => {
                        let status = match error {
                            journal::Error::Invalid => Status::Invalid,
                            journal::Error::Range => Status::Range,
                            journal::Error::Stale => Status::Stale,
                            journal::Error::Full => Status::Full,
                            journal::Error::Corrupt => Status::Corrupt,
                            journal::Error::NotFound => Status::NotFound,
                        };
                        let revision = self.state.get(h.object).unwrap().revision;
                        wire::reply(h, EPOCH, revision, status, None, None)
                    }
                }
            }
            _ => panic!("validated request must use an implemented operation"),
        };
        envelope(&response)
    }

    fn recover(&self) -> State {
        let mut recovery = Recovery::new();
        for (index, record) in self.records.iter().enumerate() {
            recovery.push(index, record).unwrap();
        }
        recovery.finish().unwrap()
    }
}

#[test]
fn catalog_open_edit_save_rename_delete_and_recover_use_existing_core() {
    let mut service = CoreService::new();
    let mut a = client(0);
    let mut b = client(1);
    assert!(
        matches!(service.exchange(&mut a, 0, Operation::List), Reply::Catalog(c) if c.count == 0)
    );
    assert_eq!(
        service.exchange(&mut a, 0, put(b"first", 0)),
        Reply::Mutation {
            object: 0,
            revision: 1
        }
    );
    assert_eq!(
        service.exchange(
            &mut b,
            1,
            Operation::Put {
                object: 2,
                expected_revision: 0,
                name: b"Private B",
                value: b"other domain"
            }
        ),
        Reply::Mutation {
            object: 2,
            revision: 1
        }
    );
    let Reply::Document(opened) = service.exchange(&mut a, 0, Operation::Get { object: 0 }) else {
        panic!()
    };
    assert_eq!(opened.name(), b"Notes");
    assert_eq!(opened.value(), b"first");
    assert_eq!(
        service.exchange(&mut a, 0, put(b"edited", opened.revision)),
        Reply::Mutation {
            object: 0,
            revision: 2
        }
    );
    assert_eq!(
        service.exchange(&mut a, 0, put(b"stale overwrite", opened.revision)),
        Reply::ServiceError(Status::Stale)
    );
    assert_eq!(
        service.exchange(
            &mut a,
            0,
            Operation::Rename {
                object: 0,
                expected_revision: 2,
                name: "Café".as_bytes()
            }
        ),
        Reply::Mutation {
            object: 0,
            revision: 3
        }
    );
    let Reply::Document(renamed) = service.exchange(&mut a, 0, Operation::Get { object: 0 }) else {
        panic!()
    };
    assert_eq!(renamed.name(), "Café".as_bytes());
    assert_eq!(renamed.value(), b"edited");
    let Reply::Catalog(catalog) = service.exchange(&mut a, 0, Operation::List) else {
        panic!()
    };
    assert_eq!(
        (catalog.count, catalog.generation, catalog.entries[0].object),
        (1, 3, 0)
    );
    assert_eq!(
        service.exchange(
            &mut b,
            1,
            Operation::Delete {
                object: 2,
                expected_revision: 1
            }
        ),
        Reply::Mutation {
            object: 2,
            revision: 2
        }
    );
    service.state = service.recover();
    let mut recovered_a = client(0);
    let mut recovered_b = client(1);
    assert_eq!(
        service.exchange(&mut recovered_a, 0, Operation::Get { object: 0 }),
        Reply::Document(renamed)
    );
    assert_eq!(
        service.exchange(&mut recovered_b, 1, Operation::Get { object: 2 }),
        Reply::NotFound {
            object: 2,
            revision: 2
        }
    );
    assert!(
        matches!(service.exchange(&mut recovered_b, 1, Operation::List), Reply::Catalog(c) if c.count == 0 && c.generation == 2)
    );
    assert_eq!(service.state.sequence(), 5);
}

#[test]
fn empty_and_maximum_documents_are_not_truncated() {
    let mut service = CoreService::new();
    let mut c = client(0);
    let value = [0xa7; wire::VALUE];
    let name = [b'x'; wire::NAME];
    service.exchange(
        &mut c,
        0,
        Operation::Put {
            object: 0,
            expected_revision: 0,
            name: &name,
            value: &value,
        },
    );
    let Reply::Document(doc) = service.exchange(&mut c, 0, Operation::Get { object: 0 }) else {
        panic!()
    };
    assert_eq!(doc.name(), name);
    assert_eq!(doc.value(), value);
    service.exchange(&mut c, 0, put(&[], 1));
    let Reply::Document(doc) = service.exchange(&mut c, 0, Operation::Get { object: 0 }) else {
        panic!()
    };
    assert!(doc.value().is_empty());
    assert_eq!(doc.revision, 2);
}

#[test]
fn invalid_configuration_and_requests_never_start_transport() {
    for args in [
        (0, SERVICE, 0),
        (EPOCH, 0, 0),
        (EPOCH, SERVICE, 2),
        (EPOCH, SERVICE, usize::MAX),
    ] {
        assert!(matches!(
            Client::new(args.0, args.1, args.2),
            Err(Error::InvalidConfiguration)
        ));
    }
    let too_long_name = [b'a'; wire::NAME + 1];
    let too_long_value = [7; wire::VALUE + 1];
    let cases = [
        (Operation::Get { object: 2 }, Status::Access),
        (Operation::Get { object: u64::MAX }, Status::Range),
        (put(b"x", 33), Status::Range),
        (
            Operation::Put {
                object: 0,
                expected_revision: 0,
                name: &too_long_name,
                value: b"x",
            },
            Status::Range,
        ),
        (put(&too_long_value, 0), Status::Range),
        (
            Operation::Rename {
                object: 0,
                expected_revision: 0,
                name: b"",
            },
            Status::Invalid,
        ),
        (
            Operation::Rename {
                object: 0,
                expected_revision: 0,
                name: b"../other",
            },
            Status::Invalid,
        ),
        (
            Operation::Rename {
                object: 0,
                expected_revision: 0,
                name: &[0xff],
            },
            Status::Invalid,
        ),
        (
            Operation::Rename {
                object: 0,
                expected_revision: 0,
                name: "a\u{202e}b".as_bytes(),
            },
            Status::Invalid,
        ),
    ];
    let mut c = client(0);
    for (operation, expected) in cases {
        assert_eq!(c.begin(operation), Err(Error::InvalidRequest(expected)));
        assert!(!c.has_pending());
        assert!(!c.is_closed());
    }
    assert_eq!(c.begin(Operation::List).unwrap().id(), 1);
}

#[test]
fn outgoing_envelope_is_bounded_and_contains_no_authority() {
    let mut c = client(1);
    let request = c.begin(Operation::List).unwrap();
    let m = request.message();
    assert_eq!(wire::get64(m, 0), 0);
    assert_eq!(wire::get64(m, 8), wire::TAG);
    assert_eq!(wire::get32(m, 16), wire::SIZE as u32);
    assert!(m[20..64]
        .iter()
        .chain(m[64 + wire::SIZE..].iter())
        .all(|byte| *byte == 0));
    let h = wire::validate_request(&payload(&request), EPOCH, 1).unwrap();
    assert_eq!((h.object, h.operation, h.request_id), (2, wire::LIST, 1));
}

#[test]
fn only_one_request_is_pending_and_duplicate_or_unsolicited_replies_are_not_accepted() {
    let mut c = client(0);
    assert_eq!(c.receive(&[0; MESSAGE_SIZE]), Err(Error::NoPending));
    assert_eq!(
        c.transport_failed(TransportFailure::Timeout),
        Error::NoPending
    );
    let request = c.begin(Operation::Get { object: 0 }).unwrap();
    assert_eq!(c.begin(Operation::List), Err(Error::Busy));
    let response = status_reply(&request, Status::NotFound);
    assert_eq!(
        c.receive(&response),
        Ok(Reply::NotFound {
            object: 0,
            revision: 0
        })
    );
    assert_eq!(c.receive(&response), Err(Error::NoPending));
    assert!(!c.is_closed());
    assert_eq!(c.begin(Operation::List).unwrap().id(), 2);
}

fn assert_interrupted(
    c: &mut Client,
    message: &[u8],
    reason: ProtocolError,
    mutation_uncertain: bool,
) {
    assert_eq!(
        c.receive(message),
        Err(Error::Interrupted {
            reason: Failure::Protocol(reason),
            mutation_uncertain
        })
    );
    assert!(c.is_closed());
    assert!(!c.has_pending());
    assert_eq!(c.begin(Operation::List), Err(Error::Closed));
    assert_eq!(c.receive(message), Err(Error::Closed));
}

#[test]
fn every_truncated_envelope_and_oversized_envelope_is_rejected_without_panicking() {
    for length in (0..MESSAGE_SIZE).chain([MESSAGE_SIZE + 1, 4096]) {
        let mut c = client(0);
        c.begin(put(b"may be committed", 0)).unwrap();
        assert_interrupted(&mut c, &vec![0; length], ProtocolError::MessageLength, true);
    }
}

#[test]
fn envelope_sender_tag_size_caps_and_all_padding_are_bound() {
    let cases = [
        (0, ProtocolError::Sender),
        (8, ProtocolError::Tag),
        (16, ProtocolError::PayloadLength),
        (20, ProtocolError::Capabilities),
        (24, ProtocolError::Padding),
        (31, ProtocolError::Padding),
        (32, ProtocolError::Padding),
        (63, ProtocolError::Padding),
        (64 + wire::SIZE, ProtocolError::Padding),
        (MESSAGE_SIZE - 1, ProtocolError::Padding),
    ];
    for (offset, reason) in cases {
        let mut c = client(0);
        let request = c.begin(Operation::Get { object: 0 }).unwrap();
        let mut response = status_reply(&request, Status::NotFound);
        response[offset] ^= 1;
        assert_interrupted(&mut c, &response, reason, false);
    }
    let mut c = client(0);
    let request = c.begin(put(b"x", 0)).unwrap();
    let mut response = status_reply(&request, Status::Unknown);
    wire::put32(&mut response, 20, u32::MAX);
    assert_interrupted(&mut c, &response, ProtocolError::Capabilities, true);
}

#[test]
fn wire_version_request_id_epoch_object_operation_status_and_payload_are_bound() {
    for offset in [
        0, 2, 4, 8, 16, 24, 32, 40, 44, 48, 52, 56, 63, 64, 191, 192, 223,
    ] {
        let mut c = client(0);
        let request = c.begin(put(b"x", 0)).unwrap();
        let mut response = status_reply(&request, Status::Unknown);
        response[64 + offset] ^= 0x80;
        assert_interrupted(&mut c, &response, ProtocolError::Reply, true);
    }
}

#[test]
fn wrong_ack_revision_cannot_turn_a_save_into_success() {
    for revision in [0, 1, 3, u64::MAX] {
        let mut c = client(0);
        let request = c.begin(put(b"edit", 1)).unwrap();
        let response = envelope(&wire::reply(
            wire::header(&payload(&request)),
            EPOCH,
            revision,
            Status::Ok,
            None,
            None,
        ));
        assert_interrupted(&mut c, &response, ProtocolError::Reply, true);
    }
}

#[test]
fn invalid_catalog_is_never_an_empty_or_cross_domain_result() {
    let mut service = CoreService::new();
    let mut c = client(0);
    service.exchange(&mut c, 0, put(b"x", 0));
    let request = c.begin(Operation::List).unwrap();
    let response = service.respond(&request, 0);
    for offset in [
        64 + 24,
        64 + 52,
        64 + 64,
        64 + 72,
        64 + 84,
        64 + 88,
        64 + 119,
    ] {
        let mut fresh = client(0);
        fresh.begin(Operation::List).unwrap();
        let mut bad = response;
        // Align request correlation to this fresh session, then alter catalog.
        wire::put64(&mut bad, 72, 1);
        bad[offset] ^= 0x80;
        assert_interrupted(&mut fresh, &bad, ProtocolError::Reply, false);
    }
}

#[test]
fn unknown_and_terminal_service_errors_close_without_replay() {
    for status in [
        Status::Unknown,
        Status::Io,
        Status::Timeout,
        Status::NoDevice,
        Status::Protocol,
        Status::Corrupt,
    ] {
        let mut c = client(0);
        let request = c.begin(put(b"uncertain", 0)).unwrap();
        let response = status_reply(&request, status);
        assert_eq!(c.receive(&response), Ok(Reply::ServiceError(status)));
        assert!(c.is_closed());
        assert!(!c.has_pending());
        assert_eq!(c.begin(put(b"uncertain", 0)), Err(Error::Closed));
        assert_eq!(c.begin(Operation::Get { object: 0 }), Err(Error::Closed));
        assert_eq!(c.receive(&response), Err(Error::Closed));
    }
}

#[test]
fn timeout_and_transport_error_preserve_mutation_uncertainty_and_reject_late_ack() {
    for (operation, mutation_uncertain) in [
        (put(b"x", 0), true),
        (
            Operation::Delete {
                object: 0,
                expected_revision: 1,
            },
            true,
        ),
        (
            Operation::Rename {
                object: 0,
                expected_revision: 1,
                name: b"New",
            },
            true,
        ),
        (Operation::Get { object: 0 }, false),
        (Operation::List, false),
    ] {
        for reason in [TransportFailure::Timeout, TransportFailure::Transport] {
            let mut c = client(0);
            let request = c.begin(operation).unwrap();
            assert_eq!(
                c.transport_failed(reason),
                Error::Interrupted {
                    reason: Failure::Transport(reason),
                    mutation_uncertain
                }
            );
            assert!(c.is_closed());
            assert_eq!(
                c.receive(&status_reply(&request, Status::Unknown)),
                Err(Error::Closed)
            );
            assert_eq!(c.begin(operation), Err(Error::Closed));
        }
    }
}

#[test]
fn ordinary_rejections_allow_an_explicit_fresh_request_with_a_new_id() {
    for status in [
        Status::Invalid,
        Status::NoSys,
        Status::Range,
        Status::Access,
        Status::Stale,
        Status::Full,
    ] {
        let mut c = client(0);
        let request = c.begin(put(b"x", 0)).unwrap();
        assert_eq!(
            c.receive(&status_reply(&request, status)),
            Ok(Reply::ServiceError(status))
        );
        assert!(!c.is_closed());
        assert_eq!(
            c.begin(Operation::Get { object: 0 }).unwrap().id(),
            request.id() + 1
        );
    }
}

#[test]
fn full_core_journal_reports_full_without_changing_the_last_value() {
    let mut service = CoreService::new();
    let mut c = client(0);
    for revision in 0..journal::SLOTS as u64 {
        assert_eq!(
            service.exchange(&mut c, 0, put(&revision.to_le_bytes(), revision)),
            Reply::Mutation {
                object: 0,
                revision: revision + 1
            }
        );
    }
    let before = service.records;
    assert_eq!(
        service.exchange(&mut c, 0, put(b"too late", 32)),
        Reply::ServiceError(Status::Full)
    );
    assert_eq!(service.records, before);
    let Reply::Document(doc) = service.exchange(&mut c, 0, Operation::Get { object: 0 }) else {
        panic!()
    };
    assert_eq!(doc.value(), &31u64.to_le_bytes());
    assert_eq!(doc.revision, 32);
}
