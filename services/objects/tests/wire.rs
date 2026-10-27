use object_core::wire::{self, Status, DELETE, GET, PUT, SIZE};

#[test]
fn exact_offsets_and_domain_binding_are_canonical() {
    let bytes = wire::request(9, 7, 3, 2, PUT, Some(b"abc"));
    assert_eq!(bytes.len(), SIZE);
    assert_eq!(&bytes[..4], &[3, 0, 2, 0]);
    assert_eq!(wire::get64(&bytes, 8), 9);
    assert_eq!(wire::get64(&bytes, 16), 7);
    assert_eq!(wire::get64(&bytes, 24), 3);
    assert_eq!(wire::get64(&bytes, 32), 2);
    assert_eq!(wire::get32(&bytes, 40), 3);
    assert_eq!(&bytes[64..67], b"abc");
    assert!(wire::validate_request(&bytes, 7, 1).is_ok());
    assert_eq!(wire::validate_request(&bytes, 7, 0), Err(Status::Access));
    assert_eq!(wire::validate_request(&bytes, 7, 2), Err(Status::Access));
}
#[test]
fn all_objects_require_the_endpoint_bound_domain_for_every_operation() {
    for object in 0..4 {
        for op in [GET, PUT, DELETE] {
            let bytes = wire::request(1, 1, object, 0, op, None);
            assert!(wire::validate_request(&bytes, 1, (object / 2) as usize).is_ok());
            assert_eq!(
                wire::validate_request(&bytes, 1, 1 - (object / 2) as usize),
                Err(Status::Access)
            );
        }
    }
}
#[test]
fn reserved_fields_zero_tail_and_epoch_are_enforced() {
    let bytes = wire::request(1, 4, 0, 0, GET, None);
    for index in [0, 1, 4, 7, 44, 47, 48, 63, 64, 191, 192, 223] {
        let mut bad = bytes;
        bad[index] = 7;
        assert_eq!(
            wire::validate_request(&bad, 4, 0),
            Err(Status::Invalid),
            "byte{index}"
        );
    }
    assert_eq!(wire::validate_request(&bytes, 5, 0), Err(Status::Stale));
    assert_eq!(wire::validate_request(&bytes, 0, 0), Err(Status::Stale));
}
#[test]
fn sizes_large_integers_and_invalid_opcodes_are_bounded() {
    let bytes = wire::request(1, 1, 0, 0, PUT, Some(&[0; 129]));
    assert_eq!(wire::validate_request(&bytes, 1, 0), Err(Status::Range));
    for object in [4, u64::MAX] {
        assert_eq!(
            wire::validate_request(&wire::request(1, 1, object, 0, GET, None), 1, 0),
            Err(Status::Range)
        );
    }
    assert_eq!(
        wire::validate_request(&wire::request(0, 1, 0, 0, GET, None), 1, 0),
        Err(Status::Invalid)
    );
    assert_eq!(
        wire::validate_request(&wire::request(1, 1, 0, u64::MAX, PUT, None), 1, 0),
        Err(Status::Range)
    );
    assert_eq!(
        wire::validate_request(&wire::request(1, 1, 0, 0, 99, None), 1, 0),
        Err(Status::NoSys)
    );
}
#[test]
fn empty_put_is_valid_but_get_and_delete_cannot_smuggle_payloads() {
    assert!(wire::validate_request(&wire::request(1, 1, 0, 0, PUT, Some(&[])), 1, 0).is_ok());
    for op in [GET, DELETE] {
        assert_eq!(
            wire::validate_request(&wire::request(1, 1, 0, 0, op, Some(b"x")), 1, 0),
            Err(Status::Invalid)
        );
    }
    assert_eq!(
        wire::validate_request(&wire::request(1, 1, 0, 1, GET, None), 1, 0),
        Err(Status::Invalid)
    );
}
#[test]
fn successful_replies_bind_object_request_epoch_revision_and_payload() {
    let request = wire::header(&wire::request(1, 2, 3, 4, PUT, Some(b"x")));
    let response = wire::reply(request, 2, 5, Status::Ok, None);
    assert!(wire::validate_reply(&response, request, 2).is_ok());
    for index in [2, 8, 16, 24, 32, 40, 48, 64, 192] {
        let mut bad = response;
        bad[index] ^= 1;
        assert_eq!(
            wire::validate_reply(&bad, request, 2),
            Err(Status::Invalid),
            "byte{index}"
        );
    }
    let mut huge = response;
    wire::put32(&mut huge, 40, u32::MAX);
    assert_eq!(
        wire::validate_reply(&huge, request, 2),
        Err(Status::Invalid)
    );
}
#[test]
fn notfound_exposes_tombstone_revision_only_for_authorized_get_or_delete() {
    for op in [GET, DELETE] {
        let request = wire::header(&wire::request(1, 2, 0, 0, op, None));
        for revision in [0, 3, 32] {
            let response = wire::reply(
                request,
                2,
                revision,
                Status::NotFound,
                Some(b"must not leak"),
            );
            assert_eq!(
                wire::validate_reply(&response, request, 2)
                    .unwrap()
                    .revision,
                revision
            );
            assert!(response[64..].iter().all(|byte| *byte == 0));
        }
    }
    let put = wire::header(&wire::request(1, 2, 0, 0, PUT, None));
    assert!(wire::validate_reply(&wire::reply(put, 2, 2, Status::NotFound, None), put, 2).is_err());
}
#[test]
fn failure_replies_never_leak_object_revision_or_data() {
    let request = wire::header(&wire::request(1, 1, 3, 0, PUT, None));
    for status in [
        Status::Invalid,
        Status::Access,
        Status::Stale,
        Status::Unknown,
        Status::Full,
        Status::Corrupt,
    ] {
        let response = wire::reply(request, 1, 7, status, Some(b"private"));
        let actual = wire::validate_reply(&response, request, 1).unwrap();
        assert_eq!(actual.revision, 0);
        assert_eq!(actual.length, 0);
        assert!(response[64..].iter().all(|byte| *byte == 0));
    }
}
#[test]
fn get_can_return_a_present_empty_object_but_never_revision_zero() {
    let request = wire::header(&wire::request(1, 1, 0, 0, GET, None));
    assert!(wire::validate_reply(
        &wire::reply(request, 1, 1, Status::Ok, Some(&[])),
        request,
        1
    )
    .is_ok());
    assert!(wire::validate_reply(
        &wire::reply(request, 1, 0, Status::Ok, Some(&[])),
        request,
        1
    )
    .is_err());
}
