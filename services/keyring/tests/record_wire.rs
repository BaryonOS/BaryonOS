//! Test the exact record dispatcher called by the native Key Service adapter.
//! The entropy fixture is deterministic test data; no native execution claim.

use keyring_core::{
    records::{RecordContext, RecordKind},
    wire::{self, Header},
    EntropySource, Service, Status, DOMAINS, MAX_PLAINTEXT, MAX_SEALED,
};

struct TestEntropy(u8);
impl EntropySource for TestEntropy {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
        for byte in out {
            *byte = self.0;
            self.0 = self.0.wrapping_add(1);
        }
        Ok(())
    }
}

fn ready() -> (Service, TestEntropy) {
    let mut service = Service::new();
    let mut source = TestEntropy(0);
    service.start(&mut source).unwrap();
    service.generate(0, &mut source).unwrap();
    service.generate(1, &mut source).unwrap();
    (service, source)
}

fn context(length: usize) -> RecordContext {
    RecordContext {
        volume: [0x42; 16],
        domain: 0,
        object: 0,
        revision: 17,
        chunk: 2,
        length: length as u16,
        kind: RecordKind::Content,
        schema: 1,
    }
}

fn dispatch(
    service: &mut Service,
    source: &mut TestEntropy,
    domain: usize,
    request: &[u8; wire::SIZE],
) -> [u8; wire::SIZE] {
    let header = Header::decode(request).unwrap();
    wire::dispatch_record(
        service,
        source,
        domain,
        &header,
        &request[wire::HEADER..wire::HEADER + usize::from(header.length)],
    )
}

fn assert_error(request: &[u8; wire::SIZE], reply: &[u8; wire::SIZE], status: Status) {
    let request = Header::decode(request).unwrap();
    let header = Header::decode(reply).unwrap();
    assert_eq!(header.op, request.op | wire::REPLY);
    assert_eq!(header.index, request.index);
    assert_eq!(header.request, request.request);
    assert_eq!(header.status, status.code());
    assert_eq!(header.length, 0);
    assert_eq!(reply[wire::HEADER..], [0; wire::BODY]);
}

#[test]
fn canonical_context_round_trips_all_kinds_without_carrying_a_domain() {
    for kind in [
        RecordKind::Content,
        RecordKind::Name,
        RecordKind::ObjectMetadata,
        RecordKind::VolumeManifest,
        RecordKind::SearchIndex,
    ] {
        for schema in [1, u16::MAX] {
            for length in [0, MAX_PLAINTEXT] {
                let original = RecordContext {
                    kind,
                    schema,
                    ..context(length)
                };
                let bytes = wire::encode_record_context(original).unwrap();
                assert_eq!(&bytes[..8], b"NOSCTX01");
                assert_eq!(bytes.len(), 64);
                assert_eq!(wire::decode_record_context(0, &bytes), Ok(original));
                let another = RecordContext {
                    domain: 1,
                    ..original
                };
                assert_eq!(wire::encode_record_context(another).unwrap(), bytes);
                assert_eq!(wire::decode_record_context(1, &bytes), Ok(another));
            }
        }
    }
}

#[test]
fn context_rejects_noncanonical_fields_lengths_and_ingress_domains() {
    let bytes = wire::encode_record_context(context(1)).unwrap();
    for length in 0..wire::RECORD_CONTEXT {
        assert_eq!(
            wire::decode_record_context(0, &bytes[..length]),
            Err(Status::Invalid)
        );
    }
    assert_eq!(
        wire::decode_record_context(0, &[0; wire::RECORD_CONTEXT + 1]),
        Err(Status::Invalid)
    );
    for domain in [DOMAINS, 255, usize::MAX] {
        assert_eq!(
            wire::decode_record_context(domain, &bytes),
            Err(Status::Invalid)
        );
    }
    for index in (0..12).chain(13..16).chain(56..64) {
        let mut changed = bytes;
        changed[index] ^= 1;
        assert_eq!(
            wire::decode_record_context(0, &changed),
            Err(Status::Invalid),
            "offset {index}"
        );
    }
    for code in [0, 6, u8::MAX] {
        let mut changed = bytes;
        changed[12] = code;
        assert_eq!(
            wire::decode_record_context(0, &changed),
            Err(Status::Invalid)
        );
    }
    for (offset, value) in [(52, (MAX_PLAINTEXT + 1) as u16), (54, 0)] {
        let mut changed = bytes;
        changed[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        assert_eq!(
            wire::decode_record_context(0, &changed),
            Err(Status::Invalid)
        );
    }
    assert_eq!(
        wire::encode_record_context(RecordContext {
            schema: 0,
            ..context(1)
        }),
        Err(Status::Invalid)
    );
    assert_eq!(
        wire::encode_record_context(RecordContext {
            domain: 255,
            ..context(1)
        }),
        Err(Status::Invalid)
    );
}

#[test]
fn builders_and_bounds_cover_every_record_length_and_preserve_legacy_operations() {
    assert_eq!(wire::SEAL_RECORD, 8);
    assert_eq!(wire::UNSEAL_RECORD, 9);
    assert_eq!(wire::body_bounds(wire::SEAL_RECORD), Some((64, 192)));
    assert_eq!(wire::body_bounds(wire::UNSEAL_RECORD), Some((104, 232)));
    let payload = [0; MAX_SEALED];
    for length in 0..=MAX_PLAINTEXT {
        for (op, data_length) in [
            (wire::SEAL_RECORD, length),
            (wire::UNSEAL_RECORD, length + 40),
        ] {
            let request =
                wire::record_request(op, 7, 0x1234, context(length), &payload[..data_length])
                    .unwrap();
            let header = Header::decode(&request).unwrap();
            assert_eq!(header.op, op);
            assert_eq!(header.index, 7);
            assert_eq!(header.request, 0x1234);
            assert_eq!(
                usize::from(header.length),
                wire::RECORD_CONTEXT + data_length
            );
            assert_eq!(header.status, 0);
        }
    }
    for (op, expected) in [
        (1, (0, 0)),
        (2, (0, 0)),
        (3, (0, 128)),
        (4, (40, 168)),
        (5, (0, 0)),
        (6, (0, 0)),
        (7, (108, 108)),
    ] {
        assert_eq!(wire::body_bounds(op), Some(expected));
    }
    assert_eq!(
        wire::record_request(wire::SEAL_RECORD, 0, 1, context(2), b"x"),
        Err(Status::Invalid)
    );
    assert_eq!(
        wire::record_request(wire::UNSEAL_RECORD, 0, 1, context(2), &[0; 41]),
        Err(Status::Invalid)
    );
    assert_eq!(
        wire::record_request(
            wire::SEAL_RECORD,
            0,
            1,
            context(MAX_PLAINTEXT + 1),
            &[0; MAX_PLAINTEXT + 1]
        ),
        Err(Status::Invalid)
    );
    assert_eq!(
        wire::record_request(wire::RESTORE, 0, 1, context(0), &[]),
        Err(Status::Invalid)
    );
}

#[test]
fn dispatcher_round_trips_the_actual_record_service_at_all_chunk_sizes() {
    let (mut service, mut source) = ready();
    for length in 0..=MAX_PLAINTEXT {
        let plaintext = [0x92; MAX_PLAINTEXT];
        let seal = wire::record_request(
            wire::SEAL_RECORD,
            0,
            73,
            context(length),
            &plaintext[..length],
        )
        .unwrap();
        let sealed = dispatch(&mut service, &mut source, 0, &seal);
        let sealed_header = Header::decode(&sealed).unwrap();
        assert_eq!(sealed_header.op, wire::SEAL_RECORD | wire::REPLY);
        assert_eq!(sealed_header.status, 0);
        assert_eq!(sealed_header.request, 73);
        assert_eq!(usize::from(sealed_header.length), length + 40);
        let unseal = wire::record_request(
            wire::UNSEAL_RECORD,
            0,
            74,
            context(length),
            &sealed[wire::HEADER..wire::HEADER + usize::from(sealed_header.length)],
        )
        .unwrap();
        let result = dispatch(&mut service, &mut source, 0, &unseal);
        let header = Header::decode(&result).unwrap();
        assert_eq!(header.op, wire::UNSEAL_RECORD | wire::REPLY);
        assert_eq!(header.status, 0);
        assert_eq!(header.request, 74);
        assert_eq!(usize::from(header.length), length);
        assert_eq!(
            result[wire::HEADER..wire::HEADER + length],
            plaintext[..length]
        );
    }
}

#[test]
fn ingress_domain_is_authoritative_and_cross_domain_decrypt_returns_no_plaintext() {
    let (mut service, mut source) = ready();
    let seal = wire::record_request(wire::SEAL_RECORD, 0, 1, context(4), b"data").unwrap();
    let reply = dispatch(&mut service, &mut source, 0, &seal);
    let sealed = &reply[wire::HEADER..wire::HEADER + 44];
    // Setting the builder's local domain cannot put that authority on the wire.
    let unseal = wire::record_request(
        wire::UNSEAL_RECORD,
        0,
        2,
        RecordContext {
            domain: 1,
            ..context(4)
        },
        sealed,
    )
    .unwrap();
    let denied = dispatch(&mut service, &mut source, 1, &unseal);
    assert_error(&unseal, &denied, Status::Auth);
    let allowed = dispatch(&mut service, &mut source, 0, &unseal);
    assert_eq!(&allowed[wire::HEADER..wire::HEADER + 4], b"data");

    // A would-be injected domain in any reserved slot is a malformed request.
    let mut forged = unseal;
    forged[wire::HEADER + 13] = 1;
    let denied = dispatch(&mut service, &mut source, 1, &forged);
    assert_error(&forged, &denied, Status::Invalid);
}

#[test]
fn dispatch_rejects_bad_lengths_reserved_context_status_and_reply_opcodes() {
    let (mut service, mut source) = ready();
    let ctx = wire::encode_record_context(context(4)).unwrap();
    let mut body = [0; wire::BODY];
    body[..wire::RECORD_CONTEXT].copy_from_slice(&ctx);
    let before = service.counters;
    for op in [wire::SEAL_RECORD, wire::UNSEAL_RECORD] {
        for length in 0..=wire::BODY {
            // Neither request has the declared 4-byte plaintext or its exact
            // 44-byte sealed body. Only those lengths are syntactically valid.
            let valid = if op == wire::SEAL_RECORD { 68 } else { 108 };
            if length == valid {
                continue;
            }
            let request = wire::request(op, 0, 81, &body[..length]).unwrap();
            let reply = dispatch(&mut service, &mut source, 0, &request);
            assert_error(&request, &reply, Status::Invalid);
        }
    }
    for op in [
        10,
        11,
        wire::SEAL_RECORD | wire::REPLY,
        wire::UNSEAL_RECORD | wire::REPLY,
    ] {
        let request = wire::request(op, 0, 82, &body[..68]).unwrap();
        let reply = dispatch(&mut service, &mut source, 0, &request);
        assert_error(&request, &reply, Status::Invalid);
    }
    let mut request = wire::record_request(wire::SEAL_RECORD, 0, 83, context(4), b"data").unwrap();
    request[12..16].copy_from_slice(&Status::Auth.code().to_le_bytes());
    let reply = dispatch(&mut service, &mut source, 0, &request);
    assert_error(&request, &reply, Status::Invalid);
    let mut mismatch = Header::decode(&request).unwrap();
    mismatch.status = 0;
    mismatch.length += 1;
    let result = wire::dispatch_record(
        &mut service,
        &mut source,
        0,
        &mismatch,
        &request[wire::HEADER..wire::HEADER + 68],
    );
    assert_eq!(
        Header::decode(&result).unwrap().status,
        Status::Invalid.code()
    );
    assert_eq!(
        service.counters, before,
        "malformed requests do not reach entropy accounting"
    );
}

#[test]
fn dispatch_preserves_authentication_failure_bad_handle_and_revocation_errors() {
    let (mut service, mut source) = ready();
    let seal = wire::record_request(wire::SEAL_RECORD, 0, 90, context(4), b"data").unwrap();
    let response = dispatch(&mut service, &mut source, 0, &seal);
    let sealed = &response[wire::HEADER..wire::HEADER + 44];
    for changed in [
        RecordContext {
            object: 1,
            ..context(4)
        },
        RecordContext {
            revision: 18,
            ..context(4)
        },
        RecordContext {
            volume: [0x43; 16],
            ..context(4)
        },
        RecordContext {
            chunk: 3,
            ..context(4)
        },
        RecordContext {
            kind: RecordKind::Name,
            ..context(4)
        },
        RecordContext {
            schema: 2,
            ..context(4)
        },
    ] {
        let request = wire::record_request(wire::UNSEAL_RECORD, 0, 91, changed, sealed).unwrap();
        let reply = dispatch(&mut service, &mut source, 0, &request);
        assert_error(&request, &reply, Status::Auth);
    }
    let request = wire::record_request(wire::UNSEAL_RECORD, 15, 92, context(4), sealed).unwrap();
    let reply = dispatch(&mut service, &mut source, 0, &request);
    assert_error(&request, &reply, Status::BadHandle);
    service.revoke(0, 0).unwrap();
    let request = wire::record_request(wire::UNSEAL_RECORD, 0, 93, context(4), sealed).unwrap();
    let reply = dispatch(&mut service, &mut source, 0, &request);
    assert_error(&request, &reply, Status::Revoked);
}

#[test]
fn context_can_never_enable_operations_before_service_readiness() {
    let mut service = Service::new();
    let mut source = TestEntropy(0);
    let request = wire::record_request(wire::SEAL_RECORD, 0, 99, context(1), b"x").unwrap();
    let reply = dispatch(&mut service, &mut source, 0, &request);
    assert_error(&request, &reply, Status::NotReady);
    let reply = dispatch(&mut service, &mut source, DOMAINS, &request);
    assert_error(&request, &reply, Status::Invalid);
}
