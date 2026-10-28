use catalog_core::journal::State;
use catalog_core::wire::{
    self, Catalog, CatalogEntry, Header, Status, DELETE, GET, LIST, PUT, RENAME, SIZE,
};

fn req(op: u16, name: Option<&[u8]>, value: Option<&[u8]>) -> [u8; SIZE] {
    wire::request(17, 9, 0, 0, op, name, value)
}
fn entry(object: u64, revision: u64, name: &[u8]) -> CatalogEntry {
    let mut value = CatalogEntry {
        object,
        revision,
        length: 128,
        name_length: name.len(),
        ..CatalogEntry::EMPTY
    };
    value.name[..name.len()].copy_from_slice(name);
    value
}
fn list_header() -> Header {
    wire::header(&req(LIST, None, None))
}

#[test]
fn names_validate_bytes_utf8_controls_bidi_and_path_punctuation() {
    for name in ["x", "Café.txt", "東京", "a b", "a.b", "é", "e\u{301}"] {
        assert!(wire::valid_name(name.as_bytes()), "{name}");
    }
    assert!(wire::valid_name(&[b'x'; 32]));
    for name in [
        "",
        ".",
        "..",
        "a/b",
        "a\\b",
        "a\0b",
        "a\nb",
        "\u{7f}",
        "\u{85}",
        "\u{202e}evil",
        "\u{2067}x",
    ] {
        assert!(!wire::valid_name(name.as_bytes()), "{name:?}");
    }
    for bytes in [
        &[b'x'; 33][..],
        &[0xff][..],
        &[0xc0, 0xaf][..],
        &[0xed, 0xa0, 0x80][..],
        &[0xf4, 0x90, 0x80, 0x80][..],
        &[0xc3][..],
    ] {
        assert!(!wire::valid_name(bytes));
    }
}

#[test]
fn all_five_valid_requests_are_accepted_only_in_their_endpoint_domain() {
    for op in [GET, PUT, DELETE, RENAME, LIST] {
        let name = matches!(op, PUT | RENAME).then_some(&b"name"[..]);
        let value = (op == PUT).then_some(&b"data"[..]);
        let message = req(op, name, value);
        assert!(wire::validate_request(&message, 9, 0).is_ok());
        assert_eq!(wire::validate_request(&message, 9, 1), Err(Status::Access));
        assert_eq!(
            wire::validate_request(&message, 9, usize::MAX),
            Err(Status::Access)
        );
    }
}

#[test]
fn constructors_never_truncate_oversize_values_or_names() {
    let name = req(PUT, Some(&[b'n'; 33]), Some(b"ok"));
    assert_eq!(wire::header(&name).name_length, u32::MAX);
    assert_eq!(&name[192..], &[0; 32]);
    assert_eq!(wire::validate_request(&name, 9, 0), Err(Status::Range));
    let data = req(PUT, Some(b"ok"), Some(&[1; 129]));
    assert_eq!(wire::header(&data).length, u32::MAX);
    assert_eq!(&data[64..192], &[0; 128]);
    assert_eq!(wire::validate_request(&data, 9, 0), Err(Status::Range));
}

#[test]
fn request_identity_epoch_version_and_every_padding_byte_are_checked() {
    let original = req(PUT, Some(b"n"), Some(b"v"));
    for i in (4..8).chain(56..64).chain(65..192).chain(193..224) {
        let mut message = original;
        message[i] = 1;
        assert_eq!(
            wire::validate_request(&message, 9, 0),
            Err(Status::Invalid),
            "byte {i}"
        );
    }
    for (at, value, expected) in [
        (8, 0, Status::Invalid),
        (16, 8, Status::Stale),
        (24, 4, Status::Range),
        (32, 33, Status::Range),
    ] {
        let mut message = original;
        wire::put64(&mut message, at, value);
        assert_eq!(wire::validate_request(&message, 9, 0), Err(expected));
    }
    let mut old = original;
    old[0] = 3;
    assert_eq!(wire::validate_request(&old, 9, 0), Err(Status::Invalid));
    assert_eq!(wire::validate_request(&original, 0, 0), Err(Status::Stale));
    for at in [44, 52] {
        let mut message = original;
        wire::put32(&mut message, at, 1);
        assert_eq!(wire::validate_request(&message, 9, 0), Err(Status::Invalid));
    }
}

#[test]
fn operation_specific_fields_reject_impossible_requests() {
    for message in [
        req(GET, Some(b"n"), None),
        req(GET, None, Some(b"v")),
        req(DELETE, Some(b"n"), None),
        req(DELETE, None, Some(b"v")),
        req(RENAME, Some(b"n"), Some(b"v")),
        req(PUT, None, None),
        req(RENAME, Some(&[0xff]), None),
        req(LIST, Some(b"n"), None),
        req(LIST, None, Some(b"v")),
    ] {
        assert_eq!(wire::validate_request(&message, 9, 0), Err(Status::Invalid));
    }
    let odd = wire::request(1, 9, 1, 0, LIST, None, None);
    assert_eq!(wire::validate_request(&odd, 9, 0), Err(Status::Invalid));
    let foreign = wire::request(1, 9, 2, 0, LIST, None, None);
    assert_eq!(wire::validate_request(&foreign, 9, 0), Err(Status::Access));
    for op in [GET, LIST] {
        let message = wire::request(1, 9, 0, 1, op, None, None);
        assert_eq!(wire::validate_request(&message, 9, 0), Err(Status::Invalid));
    }
    assert_eq!(
        wire::validate_request(&req(99, None, None), 9, 0),
        Err(Status::NoSys)
    );
}

#[test]
fn get_reply_contains_canonical_name_and_value_including_empty_content() {
    let request = wire::header(&req(GET, None, None));
    for value in [&b"content"[..], &b""[..], &[0xff; 128][..]] {
        let response = wire::reply(
            request,
            9,
            1,
            Status::Ok,
            Some("Café".as_bytes()),
            Some(value),
        );
        let h = wire::validate_reply(&response, request, 9).unwrap();
        assert_eq!(h.length as usize, value.len());
        assert_eq!(h.name_length, 5);
        assert_eq!(&response[64..64 + value.len()], value);
        assert_eq!(&response[192..197], "Café".as_bytes());
    }
    let invalid = wire::reply(request, 9, 1, Status::Ok, None, Some(b"content"));
    assert_eq!(
        wire::validate_reply(&invalid, request, 9),
        Err(Status::Invalid)
    );
}

#[test]
fn mutation_acks_have_no_data_and_are_bound_to_next_revision() {
    for op in [PUT, DELETE, RENAME] {
        let h = wire::header(&wire::request(1, 9, 0, 4, op, Some(b"n"), None));
        let reply = wire::reply(h, 9, 5, Status::Ok, Some(b"secret"), Some(b"secret"));
        assert!(wire::validate_reply(&reply, h, 9).is_ok());
        assert!(reply[40..].iter().all(|b| *b == 0));
        let stale = wire::reply(h, 9, 4, Status::Ok, None, None);
        assert_eq!(wire::validate_reply(&stale, h, 9), Err(Status::Invalid));
    }
}

#[test]
fn error_replies_scrub_all_data_and_only_authorized_notfound_keeps_revision() {
    let h = wire::header(&req(GET, None, None));
    for code in [-1, -2, -3, -5, -10, -12, -13, -14, -15, -16, -17, -18] {
        let response = wire::reply(
            h,
            9,
            27,
            Status::from_code(code),
            Some(b"secret name"),
            Some(b"secret value"),
        );
        let parsed = wire::validate_reply(&response, h, 9).unwrap();
        assert_eq!(parsed.revision, 0);
        assert_eq!((parsed.length, parsed.name_length, parsed.count), (0, 0, 0));
        assert!(response[48..].iter().all(|b| *b == 0));
    }
    for op in [GET, DELETE, RENAME] {
        let h = wire::header(&req(op, None, None));
        let response = wire::reply(h, 9, 2, Status::NotFound, Some(b"hidden"), Some(b"hidden"));
        assert_eq!(wire::validate_reply(&response, h, 9).unwrap().revision, 2);
        assert!(response[48..].iter().all(|b| *b == 0));
    }
    let h = list_header();
    let response = wire::catalog_reply(
        h,
        9,
        Status::Unknown,
        Some(&Catalog {
            generation: 1,
            count: 1,
            entries: [entry(0, 1, b"hidden"), CatalogEntry::EMPTY],
        }),
    );
    assert_eq!(wire::decode_catalog(&response, h, 9), Err(Status::Unknown));
    assert!(response[48..].iter().all(|b| *b == 0));
}

#[test]
fn list_empty_zero_generation_and_deleted_generation_are_valid() {
    for generation in [0, 1, 32] {
        let catalog = Catalog {
            generation,
            ..Catalog::empty()
        };
        let h = list_header();
        let bytes = wire::catalog_reply(h, 9, Status::Ok, Some(&catalog));
        assert_eq!(wire::decode_catalog(&bytes, h, 9).unwrap(), catalog);
        assert!(bytes[40..].iter().all(|byte| *byte == 0));
    }
}

#[test]
fn list_roundtrip_preserves_sorted_metadata_and_no_contents() {
    let mut state = State::new();
    for id in [1, 0, 2] {
        let p = state
            .prepare(id, 0, id + 1, PUT, b"same label", b"SECRET-CONTENT")
            .unwrap();
        state.commit(p).unwrap();
    }
    let catalog = state.catalog(0).unwrap();
    let h = list_header();
    let bytes = wire::catalog_reply(h, 9, Status::Ok, Some(&catalog));
    assert_eq!(wire::header(&bytes).length, 112);
    assert_eq!(wire::decode_catalog(&bytes, h, 9).unwrap(), catalog);
    assert!(!bytes.windows(14).any(|part| part == b"SECRET-CONTENT"));
    assert_eq!(wire::header(&bytes).revision, 2);
    assert_eq!(state.sequence(), 3);
}

#[test]
fn list_rejects_foreign_ids_unsorted_duplicate_or_forged_revisions() {
    let h = list_header();
    let catalog = Catalog {
        generation: 3,
        count: 2,
        entries: [entry(0, 1, b"a"), entry(1, 2, b"b")],
    };
    let original = wire::catalog_reply(h, 9, Status::Ok, Some(&catalog));
    for (at, value) in [
        (64, 2),
        (120, 0),
        (64, 1),
        (72, 0),
        (72, 33),
        (32, 2),
        (32, 4),
        (32, 33),
    ] {
        let mut bytes = original;
        wire::put64(&mut bytes, at, value);
        assert_eq!(
            wire::decode_catalog(&bytes, h, 9),
            Err(Status::Invalid),
            "offset {at}, {value}"
        );
    }
}

#[test]
fn list_rejects_bad_count_lengths_names_and_all_padding() {
    let h = list_header();
    let catalog = Catalog {
        generation: 2,
        count: 1,
        entries: [entry(0, 1, b"x"), CatalogEntry::EMPTY],
    };
    let original = wire::catalog_reply(h, 9, Status::Ok, Some(&catalog));
    for (at, value) in [
        (40, 57),
        (48, 1),
        (52, 3),
        (52, u32::MAX),
        (80, 129),
        (84, 0),
        (84, 33),
        (84, u32::MAX),
    ] {
        let mut bytes = original;
        wire::put32(&mut bytes, at, value);
        assert_eq!(
            wire::decode_catalog(&bytes, h, 9),
            Err(Status::Invalid),
            "offset {at}, {value}"
        );
    }
    for i in (4..8).chain(56..64).chain(89..224) {
        let mut bytes = original;
        bytes[i] = 1;
        assert_eq!(
            wire::decode_catalog(&bytes, h, 9),
            Err(Status::Invalid),
            "byte {i}"
        );
    }
    let mut bytes = original;
    bytes[88] = 0xff;
    assert_eq!(wire::decode_catalog(&bytes, h, 9), Err(Status::Invalid));
}

#[test]
fn replies_bind_request_epoch_operation_object_and_status() {
    let h = wire::header(&req(GET, None, None));
    let original = wire::reply(h, 9, 1, Status::Ok, Some(b"n"), Some(b"v"));
    for at in [2, 8, 16, 24] {
        let mut bytes = original;
        bytes[at] ^= 1;
        assert_eq!(wire::validate_reply(&bytes, h, 9), Err(Status::Invalid));
    }
    assert_eq!(wire::validate_reply(&original, h, 0), Err(Status::Invalid));
    let mut bytes = original;
    wire::put32(&mut bytes, 44, (-99i32) as u32);
    assert_eq!(wire::validate_reply(&bytes, h, 9), Err(Status::Invalid));
    let mut bytes = wire::reply(h, 9, 0, Status::Access, None, None);
    wire::put64(&mut bytes, 32, 1);
    assert_eq!(wire::validate_reply(&bytes, h, 9), Err(Status::Invalid));
}

#[test]
fn malformed_catalog_constructor_fails_closed_without_panicking() {
    let h = list_header();
    for catalog in [
        Catalog {
            count: usize::MAX,
            ..Catalog::empty()
        },
        Catalog {
            generation: u64::MAX,
            ..Catalog::empty()
        },
        Catalog {
            generation: 1,
            count: 1,
            entries: [
                CatalogEntry {
                    name_length: usize::MAX,
                    ..entry(0, 1, b"x")
                },
                CatalogEntry::EMPTY,
            ],
        },
    ] {
        let bytes = wire::catalog_reply(h, 9, Status::Ok, Some(&catalog));
        assert_eq!(wire::decode_catalog(&bytes, h, 9), Err(Status::Protocol));
    }
    let bytes = wire::catalog_reply(h, 9, Status::Ok, None);
    assert_eq!(wire::decode_catalog(&bytes, h, 9), Err(Status::Protocol));
}

#[test]
fn arbitrary_external_lengths_never_create_out_of_bounds_slices() {
    let h = list_header();
    for length in [
        0,
        1,
        31,
        32,
        33,
        56,
        112,
        127,
        128,
        129,
        224,
        65535,
        u32::MAX,
    ] {
        for at in [40, 48, 52, 80, 84] {
            let mut bytes = req(LIST, None, None);
            wire::put32(&mut bytes, at, length);
            let _ = wire::validate_request(&bytes, 9, 0);
            let _ = wire::validate_reply(&bytes, h, 9);
            let _ = wire::decode_catalog(&bytes, h, 9);
        }
    }
}
