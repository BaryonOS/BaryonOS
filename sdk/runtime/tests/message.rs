use capability_core::{Engine, Message};
use nieuw_os_sdk_runtime::{
    decode_return, validate_abi, Capability, Error, Handle, Outgoing, Received, Rights,
    MESSAGE_SIZE,
};

fn handle(slot: u64) -> Handle {
    Handle::from_raw((1 << 32) | slot).unwrap()
}

fn incoming(tag: u64, payload: &[u8]) -> [u8; MESSAGE_SIZE] {
    let mut bytes = *Outgoing::new(tag, payload).unwrap().as_bytes();
    bytes[..8].copy_from_slice(&7u64.to_le_bytes());
    bytes
}

#[test]
fn handle_representation_does_not_accept_sentinels_or_slot_overflow() {
    for value in [0, 1, 255, 1 << 32, (1 << 32) | 256, u64::MAX] {
        assert_eq!(Handle::from_raw(value), Err(Error::InvalidHandle));
    }
    assert_eq!(handle(1).raw(), (1 << 32) | 1);
    assert!(Handle::from_raw((u32::MAX as u64) << 32 | 255).is_ok());
}

#[test]
fn known_rights_do_not_allow_new_permission_bits() {
    for value in 0..=127 {
        assert_eq!(Rights::from_bits(value).unwrap().bits(), value);
    }
    for value in [128, 1 << 31, 1 << 63, u64::MAX] {
        assert_eq!(Rights::from_bits(value), Err(Error::InvalidRights));
    }
}

#[test]
fn register_errors_remain_errors_and_secondary_errors_are_zero() {
    assert_eq!(decode_return(0, u64::MAX), Ok(u64::MAX));
    for code in -19..=-1 {
        assert_eq!(decode_return(code, 0), Err(Error::Kernel(code)));
        assert_eq!(decode_return(code, 1), Err(Error::InvalidReturn));
    }
    assert_eq!(decode_return(1, 0), Err(Error::InvalidReturn));
    assert!(Error::Kernel(-8).is_again());
    assert!(!Error::Kernel(-16).is_again());
}

#[test]
fn abi_check_accepts_only_existing_compatible_versions() {
    for minor in 0..=2 {
        assert_eq!(validate_abi((1 << 32) | minor), Ok(()));
    }
    for version in [0, 1, 1 << 31, (1 << 32) | 3, 2 << 32, u64::MAX] {
        assert_eq!(validate_abi(version), Err(Error::UnsupportedAbi));
    }
}

#[test]
fn every_payload_length_matches_the_current_kernel_decoder() {
    for length in 0..=256 {
        let payload = [0x73; 256];
        let output = Outgoing::new(0x0102_0304_0506_0708, &payload[..length]).unwrap();
        let decoded = Message::decode(output.as_bytes()).unwrap();
        assert_eq!(decoded.tag, 0x0102_0304_0506_0708);
        assert_eq!(decoded.len as usize, length);
        assert_eq!(&decoded.data[..length], &payload[..length]);
        assert_eq!(decoded.encode(), *output.as_bytes());
    }
    assert_eq!(Outgoing::new(1, &[0; 257]), Err(Error::MessageSize));
}

#[test]
fn refused_cap_append_does_not_modify_the_message() {
    let mut message = Outgoing::new(5, b"bounded").unwrap();
    let original = message.clone();
    assert_eq!(
        message.push_cap(Capability {
            handle: handle(1),
            rights: Rights::RECV
        }),
        Err(Error::InvalidRights)
    );
    assert_eq!(message, original);
    for slot in [1, 2] {
        message
            .push_cap(Capability {
                handle: handle(slot),
                rights: Rights::SEND,
            })
            .unwrap();
    }
    let original = message.clone();
    assert_eq!(
        message.push_cap(Capability {
            handle: handle(3),
            rights: Rights::SEND
        }),
        Err(Error::MessageSize)
    );
    assert_eq!(message, original);
    assert_eq!(Message::decode(message.as_bytes()).unwrap().cap_count, 2);
}

#[test]
fn raw_send_rejects_spoofed_sender_and_every_unused_byte() {
    let canonical = *Outgoing::new(17, b"x").unwrap().as_bytes();
    for offset in (0..8).chain(24..64).chain(65..320) {
        let mut bytes = canonical;
        bytes[offset] = 1;
        assert!(Outgoing::from_bytes(bytes).is_err(), "offset {offset}");
    }
}

#[test]
fn oversized_wire_lengths_never_index_or_truncate() {
    for field in [16, 20] {
        for value in [257u32, u32::MAX] {
            let mut bytes = incoming(1, &[]);
            bytes[field..field + 4].copy_from_slice(&value.to_le_bytes());
            assert_eq!(Received::from_bytes(bytes, 0), Err(Error::MessageSize));
            bytes[..8].fill(0);
            assert_eq!(Outgoing::from_bytes(bytes), Err(Error::MessageSize));
        }
    }
}

#[test]
fn received_message_keeps_kernel_identity_and_exact_payload() {
    let bytes = incoming(99, b"document");
    let parsed = Received::from_bytes(bytes, 8).unwrap();
    assert_eq!(parsed.sender(), 7);
    assert_eq!(parsed.tag(), 99);
    assert_eq!(parsed.payload(), b"document");
    assert_eq!(parsed.as_bytes(), &bytes);
    assert_eq!(parsed.cap_count(), 0);
    assert!(parsed.cap(0).is_none());
    assert!(parsed.cap(usize::MAX).is_none());
}

#[test]
fn received_length_must_equal_the_syscall_result() {
    for length in [0, 7, 9, 257, u64::MAX] {
        assert_eq!(
            Received::from_bytes(incoming(99, b"document"), length),
            Err(Error::InvalidReturn)
        );
    }
}

#[test]
fn received_envelope_rejects_missing_sender_reserved_and_padding() {
    let bytes = incoming(9, b"value");
    let mut missing_sender = bytes;
    missing_sender[..8].fill(0);
    assert_eq!(
        Received::from_bytes(missing_sender, 5),
        Err(Error::InvalidMessage)
    );
    for offset in (24..64).chain(69..320) {
        let mut bad = bytes;
        bad[offset] = 1;
        assert!(Received::from_bytes(bad, 5).is_err(), "offset {offset}");
    }
}

#[test]
fn delegated_cap_descriptors_are_validated_and_preserved() {
    let mut output = Outgoing::new(4, b"caps").unwrap();
    output
        .push_cap(Capability {
            handle: handle(255),
            rights: Rights::SEND.union(Rights::TRANSFER),
        })
        .unwrap();
    let mut bytes = *output.as_bytes();
    bytes[..8].copy_from_slice(&17u64.to_le_bytes());
    let parsed = Received::from_bytes(bytes, 4).unwrap();
    assert_eq!(parsed.cap_count(), 1);
    assert_eq!(parsed.cap(0).unwrap().handle, handle(255));
    assert_eq!(parsed.cap(0).unwrap().rights.bits(), 9);
    assert_eq!(parsed.as_bytes(), &bytes);
    for rights in [2u64, 128, u64::MAX] {
        let mut bad = bytes;
        bad[40..48].copy_from_slice(&rights.to_le_bytes());
        assert_eq!(Received::from_bytes(bad, 4), Err(Error::InvalidRights));
    }
    let mut bad_handle = bytes;
    bad_handle[32..40].fill(0);
    assert_eq!(
        Received::from_bytes(bad_handle, 4),
        Err(Error::InvalidHandle)
    );
}

#[test]
fn sdk_messages_interoperate_with_actual_capability_engine() {
    // The real fixed-capacity engine uses a large static kernel allocation;
    // the host thread gets an explicit stack budget instead of the OS app one.
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let mut engine = Engine::new();
            engine.register(1).unwrap();
            engine.register(2).unwrap();
            let endpoint = engine.create_endpoint(2).unwrap();
            let send = engine
                .bootstrap_delegate(2, endpoint, 1, capability_core::SEND)
                .unwrap();
            let outgoing = Outgoing::new(44, b"owned").unwrap();
            engine
                .send(1, send, &Message::decode(outgoing.as_bytes()).unwrap())
                .unwrap();
            let received = engine.receive(2, endpoint).unwrap();
            let parsed = Received::from_bytes(received.encode(), received.len as u64).unwrap();
            assert_eq!(parsed.sender(), 1);
            assert_eq!(parsed.payload(), b"owned");
            // Wrapping the same number as ReceiveEndpoint cannot grant RECV.
            assert_eq!(engine.receive(1, send), Err(capability_core::Error::Access));
            assert_eq!(
                engine.derive(1, send, capability_core::SEND | capability_core::RECV),
                Err(capability_core::Error::Access)
            );
            engine.revoke(2, endpoint).unwrap();
            assert_eq!(
                engine.send(1, send, &Message::decode(outgoing.as_bytes()).unwrap()),
                Err(capability_core::Error::Revoked)
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn block_completion_accepts_only_the_canonical_ticket_envelope() {
    use nieuw_os_sdk_runtime::block::completion;
    let mut bytes = [0u8; MESSAGE_SIZE];
    bytes[8..16].copy_from_slice(&42u64.to_le_bytes());
    bytes[16..20].copy_from_slice(&4u32.to_le_bytes());
    bytes[64..68].copy_from_slice(b"data");
    assert_eq!(completion(&bytes, 4, 42, 4), Ok(&b"data"[..]));
    // Wrong ticket, length, return value, sender, caps or padding.
    assert!(completion(&bytes, 4, 41, 4).is_err());
    assert!(completion(&bytes, 3, 42, 4).is_err());
    assert!(completion(&bytes, 4, 42, 3).is_err());
    for (at, value) in [(0usize, 1u8), (20, 1), (30, 1), (68, 1), (319, 1)] {
        let mut bad = bytes;
        bad[at] ^= value;
        assert!(completion(&bad, 4, 42, 4).is_err(), "byte {at}");
    }
    assert!(completion(&bytes, 129, 42, 129).is_err());
    let mut flush = [0u8; MESSAGE_SIZE];
    flush[8..16].copy_from_slice(&9u64.to_le_bytes());
    assert_eq!(completion(&flush, 0, 9, 0), Ok(&[][..]));
}
