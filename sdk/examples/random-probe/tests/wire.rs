//! Host fixture/oracle checks, never native entropy/RANDOM evidence.
use keyring_core::{wire as service, Status};
use nieuw_os_native_random_probe::wire;
use nieuw_os_sdk_runtime::{Capability, Handle, Outgoing, Received, Rights};
fn handle(slot: u64) -> Handle {
    Handle::from_raw((1 << 32) | slot).unwrap()
}
fn envelope(payload: &[u8; 256]) -> [u8; 320] {
    let mut bytes = *Outgoing::new(wire::TAG, payload).unwrap().as_bytes();
    bytes[..8].copy_from_slice(&7u64.to_le_bytes());
    bytes
}
fn success(count: usize) -> ([u8; 256], [u8; 320]) {
    let request = service::random_request(42, count).unwrap();
    let h = service::Header::decode(&request).unwrap();
    let mut body = [0; 240];
    body[..16].copy_from_slice(&request[16..32]);
    body[16..16 + count].fill(0x6b);
    (
        request,
        envelope(&service::reply(&h, Status::Ok, &body[..16 + count])),
    )
}
#[test]
fn every_native_raw_count_is_canonical_and_matches_actual_service_codec() {
    for count in 1..=224 {
        let (request, message) = success(count);
        assert_eq!(wire::request(10, 42, count), request);
        let received = Received::from_bytes(message, 256).unwrap();
        let reply = wire::reply(&received, 7, &request, count).unwrap();
        assert_eq!(reply.status, 0);
        assert_eq!(reply.length, count);
        assert!(reply.bytes().iter().all(|b| *b == 0x6b));
    }
}
#[test]
fn every_legacy_error_and_invalid_request_index_id_are_checked_against_exact_request() {
    for op in 1..=9 {
        let request = wire::request(op, 42, 0);
        let h = service::Header::decode(&request).unwrap();
        let received =
            Received::from_bytes(envelope(&service::reply(&h, Status::Invalid, &[])), 256).unwrap();
        let parsed = wire::reply(&received, 7, &request, 0).unwrap();
        assert_eq!(parsed.status, -6);
        assert!(parsed.bytes().is_empty());
    }
    let mut request = wire::request(10, 0, 1);
    request[1] = 1;
    let h = service::Header::decode(&request).unwrap();
    let received =
        Received::from_bytes(envelope(&service::reply(&h, Status::Invalid, &[])), 256).unwrap();
    assert_eq!(wire::reply(&received, 7, &request, 0).unwrap().status, -6);
}
#[test]
fn malformed_reply_fields_never_count_as_success() {
    let (request, good) = success(32);
    for offset in [0, 8, 64, 65, 66, 68, 76, 80, 84, 86, 88, 144] {
        let mut bytes = good;
        bytes[offset] ^= 1;
        let received = Received::from_bytes(bytes, 256).unwrap();
        assert!(
            wire::reply(&received, 7, &request, 32).is_none(),
            "offset{offset}"
        );
    }
    let mut outgoing = Outgoing::new(wire::TAG, &good[64..]).unwrap();
    outgoing
        .push_cap(Capability {
            handle: handle(3),
            rights: Rights::SEND,
        })
        .unwrap();
    let mut bytes = *outgoing.as_bytes();
    bytes[..8].copy_from_slice(&7u64.to_le_bytes());
    assert!(wire::reply(&Received::from_bytes(bytes, 256).unwrap(), 7, &request, 32).is_none());
    let h = service::Header::decode(&request).unwrap();
    let bad = service::message(
        service::Header {
            op: 10 | 0x80,
            index: 0,
            length: 1,
            request: 42,
            status: -6,
        },
        &[0],
    )
    .unwrap();
    assert!(wire::reply(
        &Received::from_bytes(envelope(&bad), 256).unwrap(),
        7,
        &request,
        32
    )
    .is_none());
    let mut short = good;
    short[16..20].copy_from_slice(&16u32.to_le_bytes());
    short[80..].fill(0);
    assert!(wire::reply(&Received::from_bytes(short, 16).unwrap(), 7, &request, 32).is_none());
    assert_eq!(h.request, 42);
}

#[test]
fn real_capability_engine_proves_closed_reply_and_narrow_revoke_rebind_without_aliases() {
    use capability_core::{Engine, Message, ALL_RIGHTS, DERIVE, RECV, REVOKE, SEND, TRANSFER};
    let mut e = Engine::new();
    for pid in 1..=5 {
        e.register(pid).unwrap();
    }
    let (key, probe) = (2, 5);
    let _old0 = e.create_endpoint(key).unwrap();
    let _old1 = e.create_endpoint(key).unwrap();
    let root = e.create_endpoint(key).unwrap();
    let receive = e.derive(key, root, RECV).unwrap();
    let ancestor = e
        .derive(key, root, SEND | DERIVE | TRANSFER | REVOKE)
        .unwrap();
    let first = e.bootstrap_delegate(key, ancestor, probe, SEND).unwrap();
    let reply_root = e.create_endpoint(probe).unwrap();
    let reply_receive = e.derive(probe, reply_root, RECV).unwrap();
    let reply_send = e.bootstrap_delegate(probe, reply_root, key, SEND).unwrap();
    e.close(probe, reply_root).unwrap();
    assert_eq!(e.available_slots(probe), 253);
    assert_eq!(
        e.check_entropy(probe, first),
        Err(capability_core::Error::Type)
    );
    assert_eq!(
        e.check_entropy(probe, reply_receive),
        Err(capability_core::Error::Type)
    );
    assert!(e.derive(probe, first, ALL_RIGHTS).is_err());
    e.close(probe, first).unwrap();
    let new = e.bootstrap_delegate(key, ancestor, probe, SEND).unwrap();
    assert_ne!(new, first);
    assert_eq!(
        e.send(probe, first, &Message::default()),
        Err(capability_core::Error::BadHandle)
    );
    e.close(probe, reply_receive).unwrap();
    assert_eq!(
        e.send(key, reply_send, &Message::default()),
        Err(capability_core::Error::Closed)
    );
    e.revoke(key, ancestor).unwrap();
    assert_eq!(
        e.check_endpoint(probe, new, SEND),
        Err(capability_core::Error::Revoked)
    );
    assert!(e.check_endpoint(key, receive, RECV).is_ok());
    assert!(e.check_endpoint(key, root, RECV).is_ok());
    assert!(e.check_invariants());
}
